//! File effects of the connection verbs on `.speclink.yaml`'s `remote:` section:
//! `init --store remote` writes the section (no spec tree, no legacy file),
//! `link` writes/updates it preserving other fields, `unlink` removes it and
//! later commands run in fs mode again. Plus the leftover-file migration
//! warning (one stderr line, never affecting results) and the missing-url
//! explicit failure naming both settings.
//!
//! Credential isolation: every run points USERPROFILE/HOME/XDG_CONFIG_HOME at
//! a throwaway "home" so tests never touch the real user's credentials file.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempEnv {
    dir: PathBuf,
    home: PathBuf,
}

impl TempEnv {
    fn new(tag: &str) -> TempEnv {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-section-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        TempEnv { dir, home }
    }

    fn with_openspec(self) -> TempEnv {
        std::fs::create_dir_all(self.dir.join("openspec").join("changes").join("archive"))
            .unwrap();
        std::fs::create_dir_all(self.dir.join("openspec").join("specs")).unwrap();
        self
    }

    fn with_app_yaml(self, yaml: &str) -> TempEnv {
        std::fs::write(self.dir.join(".speclink.yaml"), yaml).unwrap();
        self
    }

    fn with_leftover_remote_file(self, yaml: &str) -> TempEnv {
        std::fs::write(self.dir.join(".speclink.remote.yaml"), yaml).unwrap();
        self
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("run speclink binary")
    }

    fn app_yaml(&self) -> serde_yaml::Value {
        let text = std::fs::read_to_string(self.dir.join(".speclink.yaml"))
            .expect(".speclink.yaml exists");
        serde_yaml::from_str(&text).expect(".speclink.yaml parses")
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

const URL: &str = "https://team.example.com/speclink/projects/foo";

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn remote_of(yaml: &serde_yaml::Value) -> &serde_yaml::Value {
    yaml.get("remote").expect("remote section present")
}

fn str_of<'a>(v: &'a serde_yaml::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or_default()
}

// --- init --store remote ---

#[test]
fn init_store_remote_writes_the_remote_section() {
    let env = TempEnv::new("init");
    let out = env.run(&[
        "init", "--store", "remote", "--url", URL, "--repo", "backend", "--tools", "claude",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let yaml = env.app_yaml();
    let remote = remote_of(&yaml);
    assert_eq!(str_of(remote, "url"), URL);
    assert_eq!(str_of(remote, "repo"), "backend");
    assert!(env.dir.join("CLAUDE.md").is_file(), "marker file generated");
    assert!(!env.dir.join("openspec").exists(), "no local spec tree in remote mode");
    assert!(
        !env.dir.join(".speclink.remote.yaml").exists(),
        "the legacy connection file is never created"
    );
    // spec Scenario「Remote init 顯式選擇 Claude」的其餘產物：Skills、settings、
    // .gitignore 與記錄下來的 built-in 選集。
    let tools: Vec<&str> = yaml
        .get("tools")
        .and_then(|t| t.as_sequence())
        .expect("tools recorded")
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    assert_eq!(tools, vec!["claude"], "explicit selection is what gets recorded");
    assert!(
        env.dir.join(".claude").join("skills").join("speclink-propose").join("SKILL.md").is_file(),
        "Claude skills installed"
    );
    assert!(env.dir.join(".claude").join("settings.json").is_file(), "Claude settings written");
    assert!(env.dir.join(".gitignore").is_file(), ".gitignore written");
}

// --- link ---

#[test]
fn link_writes_the_section_and_preserves_existing_fields() {
    let env = TempEnv::new("link-preserve").with_app_yaml("tools:\n  - claude\n  - codex\n");
    let out = env.run(&["link", URL, "--repo", "backend"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let yaml = env.app_yaml();
    let remote = remote_of(&yaml);
    assert_eq!(str_of(remote, "url"), URL);
    assert_eq!(str_of(remote, "repo"), "backend");
    let tools: Vec<&str> = yaml
        .get("tools")
        .and_then(|t| t.as_sequence())
        .expect("tools list preserved")
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    assert_eq!(tools, vec!["claude", "codex"], "tools list keeps its original values");
    assert!(
        !env.dir.join(".speclink.remote.yaml").exists(),
        "link never creates the legacy connection file"
    );
}

/// spec Scenario「link 保留既有 tools 與其他欄位」：link 不觸發工具詢問，
/// 也不同步任何受管產物——自訂描述子與未知頂層鍵原值保留。
#[test]
fn link_leaves_custom_descriptors_unknown_keys_and_managed_artifacts_alone() {
    let env = TempEnv::new("link-descriptor").with_app_yaml(concat!(
        "tools:\n",
        "  - claude\n",
        "  - name: wad-harness\n",
        "    skills_dir: .wad/skills\n",
        "    instructions_file: WAD.md\n",
        "future_top_level: keep me\n",
    ));
    let out = env.run(&["link", URL, "--repo", "backend"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let yaml = env.app_yaml();
    assert_eq!(str_of(remote_of(&yaml), "url"), URL);
    assert_eq!(
        yaml.get("future_top_level").and_then(|v| v.as_str()),
        Some("keep me"),
        "unknown top-level key survives"
    );
    let descriptor = yaml
        .get("tools")
        .and_then(|t| t.as_sequence())
        .expect("tools list preserved")
        .iter()
        .find(|t| t.get("name").is_some())
        .expect("custom descriptor preserved");
    assert_eq!(str_of(descriptor, "skills_dir"), ".wad/skills");
    assert_eq!(str_of(descriptor, "instructions_file"), "WAD.md");
    assert!(!env.dir.join("CLAUDE.md").exists(), "link never generates managed artifacts");
    assert!(!env.dir.join(".claude").exists(), "link never generates skills");
    assert!(!env.dir.join("WAD.md").exists(), "link never generates custom artifacts");
}

#[test]
fn link_updates_an_existing_section() {
    let env = TempEnv::new("link-update")
        .with_app_yaml("remote:\n  url: https://old.example.com/speclink/projects/foo\n  repo: backend\n");
    let out = env.run(&["link", URL]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let yaml = env.app_yaml();
    assert_eq!(str_of(remote_of(&yaml), "url"), URL, "url updated in place");
}

#[test]
fn link_in_an_empty_dir_creates_the_file() {
    let env = TempEnv::new("link-create");
    let out = env.run(&["link", URL, "--repo", "backend"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let yaml = env.app_yaml();
    assert_eq!(str_of(remote_of(&yaml), "url"), URL);
}

// --- unlink ---

#[test]
fn unlink_removes_the_section_and_returns_to_fs_mode() {
    let env = TempEnv::new("unlink")
        .with_openspec()
        .with_app_yaml(&format!(
            "tools:\n  - claude\nremote:\n  url: {URL}\n  repo: backend\n"
        ));
    let out = env.run(&["unlink"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let yaml = env.app_yaml();
    assert!(yaml.get("remote").is_none(), "remote section removed");
    let tools: Vec<&str> = yaml
        .get("tools")
        .and_then(|t| t.as_sequence())
        .expect("tools list preserved")
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    assert_eq!(tools, vec!["claude"], "other fields survive unlink");

    // Subsequent commands run in fs mode again (no url, no credentials needed).
    let out = env.run(&["list", "--json"]);
    assert!(out.status.success(), "fs mode works after unlink: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(stdout_of(&out).trim()).expect("json output");
    assert!(v.get("changes").is_some(), "fs-mode list shape: {v}");
}

// --- leftover .speclink.remote.yaml: one-line migration warning ---

#[test]
fn leftover_connection_file_warns_once_without_affecting_the_command() {
    let env = TempEnv::new("leftover")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n")
        .with_leftover_remote_file(&format!("url: {URL}\nrepo: backend\n"));
    let out = env.run(&["list", "--json"]);
    assert!(
        out.status.success(),
        "the warning never affects the exit code: {}",
        stderr_of(&out)
    );
    // stdout unaffected: the command ran in fs mode with its normal output.
    let v: serde_json::Value = serde_json::from_str(stdout_of(&out).trim()).expect("json output");
    assert!(v.get("changes").is_some(), "fs-mode list shape intact: {v}");

    let stderr = stderr_of(&out);
    let warnings: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("speclink: warning:"))
        .collect();
    assert_eq!(warnings.len(), 1, "exactly one warning line: {stderr}");
    let w = warnings[0];
    assert!(w.contains(".speclink.remote.yaml"), "names the leftover file: {w}");
    assert!(
        w.contains("url") && w.contains("repo") && w.contains("remote"),
        "guides moving url/repo into the remote section: {w}"
    );
    assert!(w.contains(".speclink.yaml"), "points at the new home: {w}");
}

#[test]
fn leftover_warning_appears_on_other_verbs_too() {
    // "Any command" — pin a second verb so the warning isn't wired to `list` only.
    let env = TempEnv::new("leftover-status")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n")
        .with_leftover_remote_file(&format!("url: {URL}\n"));
    let out = env.run(&["list", "--specs"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let count = stderr_of(&out)
        .lines()
        .filter(|l| l.starts_with("speclink: warning:") && l.contains(".speclink.remote.yaml"))
        .count();
    assert_eq!(count, 1, "exactly one migration warning: {}", stderr_of(&out));
}

// --- url missing from both the section and the env var (CLI-level pin) ---

#[test]
fn missing_url_everywhere_fails_naming_both_settings() {
    let env = TempEnv::new("no-url").with_app_yaml("remote:\n  repo: backend\n");
    let out = env.run(&["list", "--json"]);
    assert!(!out.status.success(), "missing url must fail, not fall back to fs");
    assert!(out.stdout.is_empty(), "no data on stdout on failure");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("remote.url"), "names the config field: {stderr}");
    assert!(
        stderr.contains("SPECLINK_STORE_URL"),
        "names the env-var alternative: {stderr}"
    );
}

#[test]
fn unlink_without_a_section_is_an_error() {
    let env = TempEnv::new("unlink-none")
        .with_openspec()
        .with_app_yaml("tools:\n  - claude\n");
    let out = env.run(&["unlink"]);
    assert!(!out.status.success(), "nothing to unlink → non-zero exit");
}

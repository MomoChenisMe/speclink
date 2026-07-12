//! Integration tests: config parsing is fail-closed — a config file that EXISTS
//! but cannot be parsed stops the command with a non-zero exit code and an error
//! naming the file and the parse reason. Only a MISSING file falls to defaults.
//! Env vars never bypass a broken file.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Project skeleton with one change; `wf_yaml: None` = no openspec/config.yaml.
    fn new(tag: &str, app_yaml: &str, wf_yaml: Option<&str>) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-fail-closed-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("openspec").join("changes").join("demo-change");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        if let Some(wf) = wf_yaml {
            std::fs::write(dir.join("openspec").join("config.yaml"), wf).unwrap();
        }
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const BAD_YAML: &str = ": not yaml : [\n";

// --- broken openspec/config.yaml: reading policy fails, never silent defaults ---

#[test]
fn bad_workflow_config_fails_instructions_naming_file_and_reason() {
    // Spec scenario 壞 config.yaml 一律 fail-closed.
    let p = TempProject::new("bad-wf", "tools:\n  - claude\n", Some(BAD_YAML));
    let out = p.run(&["instructions", "tasks", "--change", "demo-change", "--json"], &[]);
    assert!(
        !out.status.success(),
        "must exit non-zero on a broken openspec/config.yaml"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("openspec/config.yaml"),
        "stderr names the file: {err}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        "no instructions payload on stdout"
    );
}

#[test]
fn env_var_does_not_bypass_a_broken_workflow_config() {
    // Spec scenario 環境變數不得繞過壞檔: SPECLINK_TDD=true never makes the
    // command ignore the unparseable file.
    let p = TempProject::new("env-no-bypass", "tools:\n  - claude\n", Some(BAD_YAML));
    let out = p.run(
        &["instructions", "tasks", "--change", "demo-change", "--json"],
        &[("SPECLINK_TDD", "true")],
    );
    assert!(!out.status.success(), "env var must not bypass the broken file");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("openspec/config.yaml"),
        "stderr still names the file"
    );
}

#[test]
fn missing_workflow_config_still_runs_with_defaults() {
    // Spec scenario 缺檔沿用內建預設: no config.yaml at all → defaults, exit 0.
    let p = TempProject::new("missing-wf", "tools:\n  - claude\n", None);
    let out = p.run(&["instructions", "tasks", "--change", "demo-change", "--json"], &[]);
    assert!(
        out.status.success(),
        "missing file must fall to defaults: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert_eq!(payload["locale"].as_str().unwrap(), "English");
}

// --- broken .speclink.yaml: mode resolution fails closed, never fs mode ---

#[test]
fn bad_app_yaml_fails_list_without_reading_openspec() {
    // Spec scenario 壞 .speclink.yaml 不落入 fs 模式: syntax error + local
    // openspec/ present. The command must fail naming the file — not list the
    // local changes (fs mode) and not attempt any remote request (the parse
    // error, not a connection error, is what stderr carries).
    let p = TempProject::new("bad-app", BAD_YAML, Some("schema: spec-driven\n"));
    let out = p.run(&["list"], &[]);
    assert!(!out.status.success(), "must exit non-zero on a broken .speclink.yaml");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(".speclink.yaml"), "stderr names the file: {err}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("demo-change"),
        "must not read local openspec/ (fs mode): {stdout}"
    );
}

#[test]
fn bad_remote_section_type_fails_instead_of_fs_mode() {
    // The dangerous legacy behavior: `remote: 42` used to silently parse as "no
    // remote section" = fs mode. It must now be a config error.
    let p = TempProject::new("bad-remote", "remote: 42\n", Some("schema: spec-driven\n"));
    let out = p.run(&["list"], &[]);
    assert!(!out.status.success(), "type mismatch must fail closed");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(".speclink.yaml"),
        "stderr names the file"
    );
}

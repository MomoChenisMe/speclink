//! Remote initialization and connection verbs: `init --store remote`,
//! `link`/`unlink`, `auth login`/`auth status`, and the advisory git-remote
//! reference warning (fork detection that never blocks).
//!
//! Credential isolation: every run points USERPROFILE/HOME/XDG_CONFIG_HOME at
//! a throwaway "home" so tests never touch the real user's credentials file.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;

// --- mock verb-contract server (whoami only) ---

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
}

fn whoami_server(status: u16, body: &'static str) -> MockServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default();
            let (code, text) = if path == "/api/speclink/v1/projects/demo/whoami" {
                (status, body.to_string())
            } else {
                (404, r#"{"reason":"not_found","message":"no route","resource":"route","name":"?"}"#.to_string())
            };
            let resp = tiny_http::Response::from_string(text)
                .with_status_code(code)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(resp);
        }
    });
    MockServer { server, base }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

const WHOAMI_TWO_REPOS: &str = r#"{"user":{"id":"u1","name":"王小明","handle":"xiaoming"},"project":"demo","repos":[{"name":"backend"},{"name":"frontend"}]}"#;

// --- throwaway project + isolated home ---

struct TempEnv {
    dir: PathBuf,
    home: PathBuf,
}

impl TempEnv {
    fn new(tag: &str) -> TempEnv {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-setup-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        TempEnv { dir, home }
    }

    fn with_connection(self, url: &str, repo: Option<&str>) -> TempEnv {
        let mut yaml = format!("remote:\n  url: {url}\n");
        if let Some(r) = repo {
            yaml.push_str(&format!("  repo: {r}\n"));
        }
        std::fs::write(self.dir.join(".speclink.yaml"), yaml).unwrap();
        self
    }

    fn cmd(&self, args: &[&str], token: Option<&str>) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home);
        if let Some(t) = token {
            cmd.env("SPECLINK_TOKEN", t);
        }
        cmd
    }

    fn run(&self, args: &[&str], token: Option<&str>) -> Output {
        self.cmd(args, token).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], token: Option<&str>, stdin: &str) -> Output {
        let mut child = self
            .cmd(args, token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child.stdin.as_mut().unwrap().write_all(stdin.as_bytes()).unwrap();
        child.wait_with_output().expect("wait speclink binary")
    }

    /// True when `.speclink.yaml` exists and carries a `remote:` section.
    fn has_remote_section(&self) -> bool {
        std::fs::read_to_string(self.dir.join(".speclink.yaml"))
            .ok()
            .and_then(|t| serde_yaml::from_str::<serde_yaml::Value>(&t).ok())
            .map(|v| v.get("remote").is_some())
            .unwrap_or(false)
    }

    /// The credentials file inside the fake home, wherever the platform
    /// convention placed it.
    fn credentials_file(&self) -> Option<PathBuf> {
        find_file(&self.home, "credentials.yaml")
    }

    fn init_git_with_origin(&self, url: &str) {
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(&self.dir)
                .output()
                .expect("run git");
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        };
        git(&["init", "-q"]);
        git(&["remote", "add", "origin", url]);
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

// --- init --store remote ---

#[test]
fn init_store_remote_scaffolds_workspace_without_spec_tree() {
    let env = TempEnv::new("init-remote");
    let out = env.run(
        &[
            "init",
            "--store",
            "remote",
            "--url",
            "https://team.example.com/api/speclink/v1/projects/foo",
            "--repo",
            "backend",
            "--tools",
            "claude",
        ],
        None,
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let conn =
        std::fs::read_to_string(env.dir.join(".speclink.yaml")).expect("app config written");
    assert!(conn.contains("https://team.example.com/api/speclink/v1/projects/foo"));
    assert!(conn.contains("repo: backend"));
    assert!(env.has_remote_section(), "remote section written");
    assert!(
        !env.dir.join(".speclink.remote.yaml").exists(),
        "the legacy connection file is never created"
    );
    assert!(env.dir.join("CLAUDE.md").is_file(), "marker file generated");
    assert!(
        std::fs::read_to_string(env.dir.join("CLAUDE.md")).unwrap().contains("SPECLINK:START"),
        "CLAUDE.md carries the SPECLINK marker block"
    );
    assert!(env.dir.join(".claude").join("skills").is_dir(), "skills installed");
    assert!(!env.dir.join("openspec").exists(), "no local spec tree in remote mode");
}

/// spec「Remote Workspace bootstrap 跨入口一致性」的 CLI 端：Codex Remote init
/// 產生 Remote 措辭的 `AGENTS.md` 區塊、Codex Skills、`tools: [codex]` 與 remote
/// section，且不建 `openspec/`——與 Desktop bind 走同一份 Core 正典來源。
#[test]
fn init_store_remote_codex_bootstrap_is_canonical() {
    let env = TempEnv::new("init-remote-codex");
    let out = env.run(
        &[
            "init",
            "--store",
            "remote",
            "--url",
            "https://team.example.com/api/speclink/v1/projects/foo",
            "--repo",
            "backend",
            "--tools",
            "codex",
        ],
        None,
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let agents = std::fs::read_to_string(env.dir.join("AGENTS.md")).expect("AGENTS.md written");
    assert!(agents.contains("<!-- SPECLINK:START"), "AGENTS.md carries the marker block");
    assert!(
        agents.contains("team system's spec store"),
        "AGENTS.md uses the remote wording, not local paths:\n{agents}"
    );
    assert!(!agents.contains("openspec/specs/"), "no local spec paths in remote mode");
    let conn = std::fs::read_to_string(env.dir.join(".speclink.yaml")).expect("app config");
    assert!(conn.contains("codex"), "tools records codex: {conn}");
    assert!(!env.dir.join("CLAUDE.md").exists(), "codex-only: no CLAUDE.md");
    assert!(
        env.dir.join(".agents").join("skills").join("speclink-propose").join("SKILL.md").is_file(),
        "Codex skills installed"
    );
    assert!(!env.dir.join("openspec").exists(), "no local spec tree in remote mode");
}

#[test]
fn init_store_remote_requires_url() {
    let env = TempEnv::new("init-remote-nourl");
    let out = env.run(&["init", "--store", "remote", "--tools", "claude"], None);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("--url"), "stderr names the flag: {}", stderr_of(&out));
}

// --- link / unlink ---

#[test]
fn link_with_credentials_validates_the_repo() {
    let mock = whoami_server(200, WHOAMI_TWO_REPOS);
    let env = TempEnv::new("link-ok");
    let out = env.run(&["link", &mock.base, "--repo", "backend"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(env.has_remote_section(), "remote section written");
    let text = stdout_of(&out);
    assert!(text.contains("backend"), "reports the validated repo: {text}");
}

#[test]
fn link_rejects_a_repo_missing_from_the_registry() {
    let mock = whoami_server(200, WHOAMI_TWO_REPOS);
    let env = TempEnv::new("link-typo");
    let out = env.run(&["link", &mock.base, "--repo", "typo-name"], Some("tok"));
    assert!(!out.status.success(), "unknown repo must fail");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("backend"), "lists available repos: {stderr}");
    assert!(stderr.contains("frontend"), "lists available repos: {stderr}");
    assert!(!env.has_remote_section(), "no remote section on failed validation");
}

#[test]
fn link_without_credentials_hints_login_and_defers_validation() {
    let env = TempEnv::new("link-nologin");
    let out = env.run(
        &["link", "https://team.example.com/api/speclink/v1/projects/foo", "--repo", "backend"],
        None,
    );
    assert!(out.status.success(), "offline link must not block: {}", stderr_of(&out));
    assert!(env.has_remote_section(), "remote section still written");
    let text = format!("{}{}", stdout_of(&out), stderr_of(&out));
    assert!(text.contains("speclink auth login"), "hints at login: {text}");
}

#[test]
fn unlink_removes_the_remote_section() {
    let env = TempEnv::new("unlink")
        .with_connection("https://team.example.com/api/speclink/v1/projects/foo", Some("backend"));
    let out = env.run(&["unlink"], None);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!env.has_remote_section(), "remote section removed");
}

// --- auth login / status ---

#[test]
fn auth_login_stores_the_validated_token() {
    let mock = whoami_server(200, WHOAMI_TWO_REPOS);
    let env = TempEnv::new("login-ok").with_connection(&mock.base, Some("backend"));
    let out = env.run_stdin(&["auth", "login", "--token-stdin"], None, "pat-abc\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("xiaoming"), "shows the identity: {}", stdout_of(&out));

    let creds = env.credentials_file().expect("credentials file created in the user dir");
    let text = std::fs::read_to_string(&creds).unwrap();
    assert!(text.contains("pat-abc"), "token stored: {text}");
    assert!(!creds.starts_with(&env.dir), "credentials never live inside the repo");
    // Nothing new appears in the project directory.
    let repo_entries: Vec<_> = std::fs::read_dir(&env.dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(repo_entries, vec![".speclink.yaml"], "repo untouched by login");
}

#[test]
fn auth_login_with_rejected_token_stores_nothing() {
    let mock = whoami_server(401, r#"{"reason":"token_invalid","message":"bad token"}"#);
    let env = TempEnv::new("login-bad").with_connection(&mock.base, Some("backend"));
    let out = env.run_stdin(&["auth", "login", "--token-stdin"], None, "pat-bad\n");
    assert!(!out.status.success(), "invalid token must fail");
    assert!(env.credentials_file().is_none(), "rejected token never written");
}

#[test]
fn auth_status_without_credentials_is_nonzero() {
    let env = TempEnv::new("status-nologin")
        .with_connection("https://team.example.com/api/speclink/v1/projects/foo", Some("backend"));
    let out = env.run(&["auth", "status"], None);
    assert!(!out.status.success(), "not logged in → non-zero exit");
    let text = format!("{}{}", stdout_of(&out), stderr_of(&out));
    assert!(text.contains("speclink auth login"), "points at login: {text}");
}

#[test]
fn auth_status_reports_the_identity() {
    let mock = whoami_server(200, WHOAMI_TWO_REPOS);
    let env = TempEnv::new("status-ok").with_connection(&mock.base, Some("backend"));
    let out = env.run(&["auth", "status"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("xiaoming"), "stdout: {}", stdout_of(&out));
}

// --- git remote reference warning (advisory only) ---

const WHOAMI_WITH_GITURL: &str = r#"{"user":{"id":"u1","name":"王小明","handle":"xiaoming"},"project":"demo","repos":[{"name":"backend","gitUrl":"https://github.com/original/repo.git"}]}"#;

#[test]
fn link_on_a_fork_warns_once_without_failing() {
    let mock = whoami_server(200, WHOAMI_WITH_GITURL);
    let env = TempEnv::new("fork-warn");
    env.init_git_with_origin("https://github.com/fork/repo.git");
    let out = env.run(&["link", &mock.base, "--repo", "backend"], Some("tok"));
    assert!(out.status.success(), "warning never blocks: {}", stderr_of(&out));
    assert!(env.has_remote_section(), "remote section still written");
    let stderr = stderr_of(&out);
    let warnings: Vec<&str> = stderr.lines().filter(|l| l.contains("fork")).collect();
    assert_eq!(warnings.len(), 1, "exactly one fork warning line: {stderr}");
}

#[test]
fn auth_status_is_silent_without_a_reference_value() {
    let mock = whoami_server(200, WHOAMI_TWO_REPOS); // no gitUrl anywhere
    let env = TempEnv::new("no-ref").with_connection(&mock.base, Some("backend"));
    env.init_git_with_origin("https://github.com/fork/repo.git");
    let out = env.run(&["auth", "status"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!stderr_of(&out).contains("fork"), "no reference value → silence: {}", stderr_of(&out));
}

#[test]
fn link_in_a_non_git_dir_skips_the_reference_check() {
    let mock = whoami_server(200, WHOAMI_WITH_GITURL);
    let env = TempEnv::new("non-git");
    let out = env.run(&["link", &mock.base, "--repo", "backend"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!stderr_of(&out).contains("fork"), "non-git dir → silence: {}", stderr_of(&out));
}

//! Integration tests: frozen behavior for the dispatch layer's three mode-shape
//! boundary classes (characterization — green before AND after the dispatch
//! refactor, with byte-identical messages).
//!
//! - ModeFree: verbs that read no project config (completion, config) run
//!   untouched under a broken .speclink.yaml.
//! - FsOnly: demo, trace, plan and change depends under a remote-mode project
//!   are refused at mode resolution — no server request leaves the process, so
//!   offline refuses identically.
//! - RemoteOnly: claim under an fs project is refused with the frozen message.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Bare project root carrying only the given `.speclink.yaml`.
    fn new(tag: &str, app_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-mode-dispatch-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
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
        // Hermetic global config: point every home lookup (macOS HOME, Windows
        // USERPROFILE, Linux XDG_CONFIG_HOME) at the temp dir so the
        // developer's real global config never leaks into assertions.
        cmd.env("HOME", &self.dir)
            .env("USERPROFILE", &self.dir)
            .env("XDG_CONFIG_HOME", &self.dir);
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const BAD_YAML: &str = ": not yaml : [\n";

// --- ModeFree: a broken .speclink.yaml must not affect verbs that read no project config ---

#[test]
fn mode_free_completion_runs_under_broken_app_yaml() {
    // Spec scenario ModeFree 動詞不受壞連線設定影響.
    let p = TempProject::new("modefree-completion", BAD_YAML);
    let out = p.run(&["completion", "generate", "zsh"]);
    assert!(
        out.status.success(),
        "completion must ignore the broken file: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.stdout.is_empty(), "completion script on stdout");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains(".speclink.yaml"),
        "stderr must not mention the config file: {err}"
    );
}

#[test]
fn mode_free_config_list_runs_under_broken_app_yaml() {
    // Spec scenario ModeFree 動詞不受壞連線設定影響: config operates on the
    // global config file only, never the project one.
    let p = TempProject::new("modefree-config", BAD_YAML);
    let out = p.run(&["config", "list"]);
    assert!(
        out.status.success(),
        "config list must ignore the broken file: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains(".speclink.yaml"),
        "stderr must not mention the config file: {err}"
    );
}

// --- FsOnly: demo, trace, plan and change depends refuse remote mode at mode resolution — zero server requests ---

/// FsOnly 動詞於 remote 模式且 server 不可達：exit code 非零、stderr 含各自
/// 拒絕句、stdout 空、零請求（listener 在線但從不說 HTTP——若動詞發出任何請求，
/// 連線會躺在 backlog 裡被 accept() 撿到）。
fn assert_fs_only_refuses_remote(tag: &str, args: &[&str], refusal: &str) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let p = TempProject::new(
        tag,
        &format!("remote:\n  url: http://127.0.0.1:{port}/api/speclink/v1/projects/demo\n"),
    );
    let out = p.run(args);
    assert!(!out.status.success(), "{args:?} must refuse remote mode");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(refusal), "{args:?} frozen refusal text: {err}");
    assert!(
        String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        "{args:?} refusal writes nothing to stdout"
    );
    listener.set_nonblocking(true).unwrap();
    match listener.accept() {
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        other => panic!("{args:?}: no server request may be emitted, got {other:?}"),
    }
}

#[test]
fn fs_only_demo_rejects_remote_mode_without_any_server_request() {
    // Spec scenario FsOnly 動詞於 remote 模式零請求拒絕.
    assert_fs_only_refuses_remote(
        "fsonly-demo",
        &["demo"],
        "demo is not available in remote mode — it seeds a demo change into a local openspec/ tree",
    );
}

#[test]
fn fs_only_demo_rejects_remote_mode_when_no_server_listens() {
    // Spec scenario FsOnly 動詞於 remote 模式零請求拒絕（離線變體）: bind then
    // drop to get a port nobody listens on — the refusal must still be the
    // mode-resolution text, never a connection error.
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let p = TempProject::new(
        "fsonly-demo-offline",
        &format!("remote:\n  url: http://127.0.0.1:{port}/api/speclink/v1/projects/demo\n"),
    );
    let out = p.run(&["demo"]);
    assert!(!out.status.success(), "demo must refuse remote mode while offline");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(
            "demo is not available in remote mode — it seeds a demo change into a local openspec/ tree"
        ),
        "frozen refusal text: {err}"
    );
}

#[test]
fn fs_only_trace_rejects_remote_mode_without_any_server_request() {
    // Spec scenario trace 於 remote 明確拒絕.
    assert_fs_only_refuses_remote(
        "fsonly-trace",
        &["trace", "some-capability"],
        "trace is not available in remote mode — it assembles the provenance chain from the local openspec/ tree",
    );
}

#[test]
fn fs_only_plan_rejects_remote_mode_without_any_server_request() {
    // Spec scenario FsOnly 動詞於 remote 模式零請求拒絕（change-plan design D6）.
    assert_fs_only_refuses_remote(
        "fsonly-plan",
        &["plan", "--json"],
        "plan is not available in remote mode yet — it reads the local openspec/ tree",
    );
}

#[test]
fn fs_only_change_depends_rejects_remote_mode_without_any_server_request() {
    // Spec scenario FsOnly 動詞於 remote 模式零請求拒絕（change-plan design D6）.
    assert_fs_only_refuses_remote(
        "fsonly-change-depends",
        &["change", "depends", "some-change", "--on", "other"],
        "change depends is not available in remote mode yet — it writes the local change metadata",
    );
}

// --- RemoteOnly: claim refuses fs mode with the frozen message ---

#[test]
fn remote_only_claim_rejects_fs_mode_with_frozen_text() {
    // Spec scenario RemoteOnly 動詞於 fs 模式明確拒絕.
    let p = TempProject::new("remoteonly-claim", "tools:\n  - claude\n");
    let out = p.run(&["claim", "some-change"]);
    assert!(!out.status.success(), "claim must refuse fs mode");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("claim requires a remote store — this project uses the local fs store"),
        "frozen refusal text: {err}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        "refusal writes nothing to stdout"
    );
}

//! Spec terminal-activity-report, scenario 「Windows 上不回報」: where the
//! standard library has no Unix domain socket, the Herdr variables change
//! nothing. The tests in `terminal_report.rs` need a Unix socket for their
//! fake core, so this one keeps its own small project.
#![cfg(not(unix))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `task done 1 --change add-auth` in a fresh project at `dir`, with or
/// without the Herdr variables.
fn task_done(dir: &Path, herdr: bool) -> Output {
    let _ = std::fs::remove_dir_all(dir);
    let change = dir.join("speclink").join("changes").join("add-auth");
    std::fs::create_dir_all(&change).unwrap();
    std::fs::write(change.join(".meta.yaml"), "schema: spec-driven\ncreated: 2026-07-01\n").unwrap();
    std::fs::write(change.join("tasks.md"), "## 1. Group\n\n- [ ] 1.1 One\n").unwrap();
    for args in [&["init", "-q"][..], &["config", "user.name", "Sandbox Tester"], &["config", "user.email", "sandbox@example.com"]] {
        assert!(Command::new("git").args(args).current_dir(dir).output().unwrap().status.success(), "git {args:?}");
    }
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
    cmd.args(["task", "done", "1", "--change", "add-auth"])
        .current_dir(dir)
        .env_remove("HC_TERMINAL_ID")
        .env_remove("HC_TERMINAL_SOCKET")
        .env_remove("SPECLINK_TERMINAL_REPORT")
        .env_remove("SPECLINK_STORE_URL")
        .env_remove("SPECLINK_TOKEN");
    if herdr {
        cmd.env("HC_TERMINAL_ID", "t2p").env("HC_TERMINAL_SOCKET", dir.join("c.sock"));
    }
    cmd.output().unwrap()
}

#[test]
fn the_herdr_variables_change_nothing_without_unix_sockets() {
    let base: PathBuf = std::env::temp_dir().join(format!("sl-trwin-{}", std::process::id()));
    let plain = task_done(&base.join("plain"), false);
    let herdr = task_done(&base.join("herdr"), true);
    let socket_created = base.join("herdr").join("c.sock").exists();
    let _ = std::fs::remove_dir_all(&base);
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    assert_eq!(herdr.status.code(), plain.status.code());
    assert_eq!(herdr.stdout, plain.stdout, "{}", String::from_utf8_lossy(&herdr.stdout));
    assert_eq!(herdr.stderr, plain.stderr, "{}", String::from_utf8_lossy(&herdr.stderr));
    assert!(!socket_created);
}

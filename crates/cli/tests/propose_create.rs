//! `speclink propose create` end-to-end 命令介面測試。
//!
//! 使用 `assert_cmd` 觸發 binary、`tempfile` 隔離 CWD，並比對 exit code + stdout JSON。

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const FIXED_REQ: &str = "req_00000000000000000000000000000000";

fn cmd(cwd: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("speclink").expect("cargo bin");
    c.current_dir(cwd);
    // 隔離 user-level config + 強制可預期 requestId
    c.env_remove("SPECLINK_PROVIDER");
    c.env("SPECLINK_TEST_REQUEST_ID", FIXED_REQ);
    c.env("SPECLINK_CONFIG_HOME", cwd.join("__no_global__"));
    c
}

#[test]
fn success_exit_zero_with_single_line_json() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "test summary",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    // 必須只有一行 JSON
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one stdout line, got: {stdout:?}"
    );
    let env: Value = serde_json::from_str(lines[0]).expect("valid json");
    assert_eq!(env["ok"], Value::Bool(true));
    assert!(env["error"].is_null());
    assert_eq!(env["requestId"], FIXED_REQ);
    let data = &env["data"];
    assert_eq!(data["changeId"], "demo");
    assert_eq!(data["state"], "proposed");
    assert_eq!(data["mode"], "local");
    assert_eq!(data["artifactPath"], ".speclink/changes/demo/proposal.md");
}

#[test]
fn invalid_change_id_exits_2() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "Add-Feature",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    if !stdout.is_empty() {
        let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).expect("json");
        assert_eq!(env["ok"], Value::Bool(false));
        assert_eq!(env["error"]["code"], "change.invalid_id");
    }
}

#[test]
fn empty_summary_exits_2() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn over_long_summary_exits_2() {
    let tmp = TempDir::new().unwrap();
    let long = "x".repeat(201);
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            &long,
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn fallback_to_local_emits_warning() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join(".speclink")).unwrap();
    std::fs::write(
        root.join(".speclink").join("config.toml"),
        "provider = \"acme\"\nfallback = \"local\"\n",
    )
    .unwrap();

    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).expect("json");
    let warnings = env["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["code"], "provider.not_authenticated");
    let msg = warnings[0]["message"].as_str().unwrap();
    assert!(msg.contains("acme"), "warning must mention acme: {msg}");
}

#[test]
fn fallback_disabled_exits_6() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::create_dir_all(root.join(".speclink")).unwrap();
    std::fs::write(
        root.join(".speclink").join("config.toml"),
        "provider = \"acme\"\nfallback = \"disabled\"\n",
    )
    .unwrap();

    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(6),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).expect("json");
    assert_eq!(env["ok"], Value::Bool(false));
    assert_eq!(env["error"]["code"], "provider.not_authenticated");
}

#[test]
fn change_already_exists_exits_1() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    // 預先建立 .speclink/changes/demo/
    std::fs::create_dir_all(root.join(".speclink/changes/demo")).unwrap();
    std::fs::write(root.join(".speclink/changes/demo/proposal.md"), "EXISTING").unwrap();

    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let env: Value = serde_json::from_str(stdout.lines().next().unwrap()).expect("json");
    assert_eq!(env["error"]["code"], "change.already_exists");
    // 既有 proposal 未被覆寫
    let body = std::fs::read_to_string(root.join(".speclink/changes/demo/proposal.md")).unwrap();
    assert_eq!(body, "EXISTING");
}

#[test]
fn successful_run_produces_all_side_effects() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "test summary",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));

    // 1) proposal.md
    let proposal = root.join(".speclink/changes/demo/proposal.md");
    let body = std::fs::read_to_string(&proposal).unwrap();
    assert_eq!(body, "## Why\n\ntest summary\n");

    // 2) metadata.json with state = proposed
    let meta = root.join(".speclink/changes/demo/metadata.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&meta).unwrap()).unwrap();
    assert_eq!(v["state"], "proposed");

    // 3) SQLite in_progress_change has the row
    let conn = rusqlite::Connection::open(root.join(".speclink/state.db")).unwrap();
    let id: String = conn
        .query_row("SELECT change_id FROM in_progress_change", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(id, "demo");
}

#[test]
fn output_contains_no_secret_strings() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let out = cmd(root)
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));

    let stdout = String::from_utf8(out.stdout).unwrap();
    let proposal = std::fs::read_to_string(root.join(".speclink/changes/demo/proposal.md"))
        .unwrap_or_default();
    let metadata = std::fs::read_to_string(root.join(".speclink/changes/demo/metadata.json"))
        .unwrap_or_default();
    let db_bytes = std::fs::read(root.join(".speclink/state.db")).unwrap_or_default();

    let forbidden: [&str; 6] = [
        "token",
        "access_token",
        "refresh_token",
        "api_key",
        "password",
        "Bearer ",
    ];
    for needle in forbidden {
        assert!(
            !stdout.to_lowercase().contains(&needle.to_lowercase()),
            "stdout contains forbidden secret pattern '{needle}': {stdout}"
        );
        assert!(
            !proposal.to_lowercase().contains(&needle.to_lowercase()),
            "proposal.md contains forbidden secret pattern '{needle}'"
        );
        assert!(
            !metadata.to_lowercase().contains(&needle.to_lowercase()),
            "metadata.json contains forbidden secret pattern '{needle}': {metadata}"
        );
        // SQLite binary: case-insensitive byte search
        let lower_needle = needle.to_lowercase();
        let lower_bytes: Vec<u8> = db_bytes.iter().map(|b| b.to_ascii_lowercase()).collect();
        assert!(
            !lower_bytes
                .windows(lower_needle.len())
                .any(|w| w == lower_needle.as_bytes()),
            "state.db contains forbidden secret pattern '{needle}'"
        );
    }
}

#[test]
fn stdout_uses_lf_line_ending() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args([
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    // bytes 必須以 \n 結尾，且不可含 \r
    assert!(out.stdout.ends_with(b"\n"));
    assert!(
        !out.stdout.windows(2).any(|w| w == b"\r\n"),
        "stdout must not contain CRLF; got: {:?}",
        out.stdout
    );
}

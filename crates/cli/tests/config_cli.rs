//! CLI integration tests for `speclink config show` / `set` / `edit`.
//!
//! 對應 `config-rw` capability requirements 與 design contract 觀察行為條目。

use assert_cmd::Command as AssertCommand;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(out.status.success(), "git command failed: {cmd:?}");
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn speclink(working: &Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("speclink").expect("binary");
    cmd.current_dir(working);
    cmd
}

fn init_project(working: &Path) {
    git_init(working);
    speclink(working).arg("init").assert().success();
    // A5 Section 9 完成後，init 已自動寫入 `.speclink/config.yaml` 與 `config_state`
    // singleton row，這裡不再需要 manual setup。
}

fn parse_json(output: &[u8]) -> serde_json::Value {
    serde_json::from_slice(output)
        .unwrap_or_else(|e| panic!("not JSON: {e}\n{}", String::from_utf8_lossy(output)))
}

// ----- 7.1 / 7.2 show --json -----

#[test]
fn config_show_returns_full_versioned_config_in_json_envelope() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args(["--json", "config", "show"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert!(
        data["value"]["rules"]["require_artifact_review"] == false,
        "data.value.rules.require_artifact_review SHALL be false"
    );
    let etag = data["etag"].as_str().expect("etag string");
    assert!(etag.starts_with("v1."));
}

#[test]
fn config_show_with_key_returns_leaf_value_and_envelope_etag() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args([
            "--json",
            "config",
            "show",
            "--key",
            "rules.require_code_review",
        ])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["key"], "rules.require_code_review");
    assert_eq!(v["data"]["value"], false);
    let etag = v["data"]["etag"].as_str().expect("etag");
    assert!(etag.starts_with("v1."));
}

#[test]
fn config_show_with_wildcard_key_rejects_with_config_key_not_found() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args(["--json", "config", "show", "--key", "rules.*"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "config.key_not_found");
}

#[test]
fn config_show_wildcard_key_preserves_user_input_in_message() {
    // polish-config-error-messages spec scenario「JSONPath parse failure preserves the
    // user's `--key` argument in the error envelope」：parse 失敗時 message 內含 user
    // 原始 key 字面字串（`rules.*`），診斷理由（wildcard / filter）走 message hint、
    // 不再被當 key 名 interpolate 進「config key `wildcards not supported` not found」
    // 這種誤導訊息。
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args(["--json", "config", "show", "--key", "rules.*"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "config.key_not_found");
    let msg = v["error"]["message"].as_str().expect("message string");
    assert!(msg.contains("rules.*"), "message missing `rules.*`: {msg}");
    assert!(
        !msg.contains("`wildcards not supported`"),
        "diagnostic string SHALL NOT appear as key name: {msg}"
    );
}

// ----- 7.3 set -----

#[test]
fn config_set_boolean_flag_succeeds_and_emits_keys_changed() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args([
            "--json",
            "config",
            "set",
            "rules.require_code_review",
            "true",
        ])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["value"]["rules"]["require_code_review"], true);
    assert_eq!(v["data"]["keys_changed"][0], "rules.require_code_review");
    let etag = v["data"]["etag"].as_str().expect("etag");
    assert!(etag.starts_with("v2."));

    // 重讀確認檔案落實。
    let assert = speclink(&w)
        .args([
            "--json",
            "config",
            "show",
            "--key",
            "rules.require_code_review",
        ])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["data"]["value"], true);
}

#[test]
fn config_set_with_wrong_expected_etag_exits_7_state_etag_mismatch() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args([
            "--json",
            "config",
            "set",
            "rules.require_code_review",
            "true",
            "--expected-etag",
            "v1.WRONGSHA1234",
        ])
        .assert()
        .failure()
        .code(7);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "state.etag_mismatch");
}

#[test]
fn config_set_wrong_expected_etag_message_does_not_leak_rust_debug() {
    // polish-config-error-messages spec scenario「`state.etag_mismatch` message does
    // not leak Rust Debug formatting」：CLI envelope `error.message` SHALL 含 expected
    // / actual 兩條 etag 字面字串、SHALL NOT 含 `Some(` 或 `None` Debug wrapper。
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    // 先取得當前 etag（給 actual 字串比對用）。
    let show = speclink(&w)
        .args(["--json", "config", "show"])
        .assert()
        .success();
    let show_v = parse_json(&show.get_output().stdout);
    let actual_etag = show_v["data"]["etag"]
        .as_str()
        .expect("etag string")
        .to_string();

    let bogus = "v99.bogus0000000";
    let assert = speclink(&w)
        .args([
            "--json",
            "config",
            "set",
            "rules.require_code_review",
            "true",
            "--expected-etag",
            bogus,
        ])
        .assert()
        .failure()
        .code(7);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "state.etag_mismatch");
    let msg = v["error"]["message"].as_str().expect("message string");
    assert!(msg.contains(bogus), "message missing expected etag: {msg}");
    assert!(
        msg.contains(&actual_etag),
        "message missing actual etag `{actual_etag}`: {msg}"
    );
    assert!(
        !msg.contains("Some("),
        "message SHALL NOT leak Rust `Some(...)`: {msg}"
    );
    assert!(
        !msg.contains("None"),
        "message SHALL NOT leak Rust `None`: {msg}"
    );
}

#[test]
fn config_set_with_unknown_key_exits_2_config_key_not_found() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .args(["--json", "config", "set", "rules.unknown_flag", "true"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "config.key_not_found");
}

// ----- 7.4 edit --stdin -----

#[test]
fn config_edit_via_stdin_replaces_file_and_returns_new_etag() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let new_yaml = "rules:\n  require_artifact_review: true\n  require_code_review: true\n";
    let assert = speclink(&w)
        .args(["--json", "config", "edit", "--stdin"])
        .write_stdin(new_yaml)
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["value"]["rules"]["require_artifact_review"], true);
    assert_eq!(v["data"]["value"]["rules"]["require_code_review"], true);
    let on_disk = std::fs::read_to_string(w.join(".speclink").join("config.yaml")).unwrap();
    assert!(on_disk.contains("require_artifact_review: true"));
}

#[test]
fn config_edit_with_malformed_yaml_exits_3_config_malformed() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let yaml_before = std::fs::read_to_string(w.join(".speclink").join("config.yaml")).unwrap();
    let bad = "rules:\n  require_code_review: [unclosed";
    let assert = speclink(&w)
        .args(["--json", "config", "edit", "--stdin"])
        .write_stdin(bad)
        .assert()
        .failure()
        .code(3);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "config.malformed");
    // File unchanged.
    assert_eq!(
        std::fs::read_to_string(w.join(".speclink").join("config.yaml")).unwrap(),
        yaml_before,
    );
}

// ----- 7.5 edit --editor cat -----

#[test]
fn config_edit_via_editor_no_op_returns_same_sha_with_bumped_version() {
    // `true`（POSIX builtin / standalone binary）總是 exit 0、不接觸檔案。對 CLI
    // 而言 child 結束後 tempfile 內容仍是當前 config bytes → no-op edit、整檔覆寫
    // 成同 bytes → version 仍 +1（沒做 no-op 偵測），但 sha 不變。
    //
    // 註：tasks.md 原始描述用 `$EDITOR=cat` 當 mock；實測 cat 會把檔案內容噴到
    // stdout、污染 JSON envelope，所以這裡改用 `true` 達到同樣語意（spawn child、
    // exit 0、不改檔）。
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let initial = speclink(&w)
        .args(["--json", "config", "show"])
        .assert()
        .success();
    let initial_v = parse_json(&initial.get_output().stdout);
    let initial_sha = initial_v["data"]["etag"]
        .as_str()
        .expect("etag")
        .split_once('.')
        .map(|(_, s)| s.to_string())
        .expect("etag has .");

    let assert = speclink(&w)
        .args(["--json", "config", "edit", "--editor", "true"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    let new_etag = v["data"]["etag"].as_str().expect("etag").to_string();
    let new_sha = new_etag
        .split_once('.')
        .map(|(_, s)| s)
        .expect("etag has .");
    assert_eq!(new_sha, initial_sha);
    assert!(new_etag.starts_with("v2."));
}

#[test]
fn config_edit_without_stdin_or_editor_returns_config_edit_mode_required() {
    // polish-config-error-messages spec scenario「`config edit` without input mode emits
    // `config.edit_mode_required`」：三條輸入路徑（--stdin / --editor / $EDITOR）皆缺時
    // SHALL 抛新 code、message 字面含 `--stdin` 與 `$EDITOR` 提示 caller 兩條可行路徑。
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    let assert = speclink(&w)
        .env_remove("EDITOR")
        .args(["--json", "config", "edit"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "config.edit_mode_required");
    let msg = v["error"]["message"].as_str().expect("message string");
    assert!(msg.contains("--stdin"), "message missing --stdin: {msg}");
    assert!(msg.contains("$EDITOR"), "message missing $EDITOR: {msg}");
}

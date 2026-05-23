//! CLI integration tests for `speclink instructions <kind>`.
//!
//! 對應 `instructions-resolver` capability requirements 與 design.md Implementation
//! Contract 內 Observable behavior / JSON output envelope / Failure modes /
//! Acceptance criteria（第 7 條 — CLI smoke + role/discussion accepted-ignored）。

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
}

fn parse_json(output: &[u8]) -> serde_json::Value {
    serde_json::from_slice(output)
        .unwrap_or_else(|e| panic!("not JSON: {e}\n{}", String::from_utf8_lossy(output)))
}

// ----- 6.2 Happy path: artifact kind 11-field envelope -----

#[test]
fn instructions_proposal_emits_11_field_envelope() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args(["--json", "instructions", "proposal"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];

    // 11-field envelope spec assertion
    assert_eq!(data["kind"], "proposal");
    assert_eq!(data["schema_id"], "spec-driven");
    assert!(data["instruction"].is_string());
    assert!(!data["instruction"].as_str().unwrap().is_empty());
    assert!(data["template"].is_string());
    assert!(!data["template"].as_str().unwrap().is_empty());
    assert!(data["context"].is_null());
    assert!(data["rules"].is_null());
    assert!(data["dependencies"].is_array());
    assert_eq!(data["dependencies"].as_array().unwrap().len(), 0);
    assert_eq!(data["output_path"], "proposal.md");
    assert!(data["locale"].is_null());
    assert!(data["available_roles"].is_null());
    assert!(data["linked_changes_context"].is_null());
}

#[test]
fn instructions_apply_returns_phase_kind_envelope_with_null_template() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args(["--json", "instructions", "apply"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    let data = &v["data"];
    assert_eq!(data["kind"], "apply");
    assert!(
        data["template"].is_null(),
        "phase kind template SHALL be null"
    );
    assert!(
        data["output_path"].is_null(),
        "phase kind output_path SHALL be null"
    );
    assert!(data["instruction"].is_string());
    assert_eq!(
        data["dependencies"].as_array().unwrap().len(),
        3,
        "apply has 3 deps (proposal, spec, tasks)"
    );
}

// ----- 6.3 Unknown kind → exit 2 + `instructions.unknown_kind` -----

#[test]
fn instructions_discuss_kind_exits_2_with_unknown_kind_code() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args(["--json", "instructions", "discuss"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "instructions.unknown_kind");
    let msg = v["error"]["message"].as_str().expect("message");
    assert!(msg.contains("discuss"), "message should echo kind: {msg}");
    let hint = v["error"]["hint"].as_str().expect("hint");
    for k in [
        "proposal", "spec", "design", "tasks", "apply", "ingest", "archive", "commit",
    ] {
        assert!(
            hint.contains(k),
            "hint missing supported kind `{k}`: {hint}"
        );
    }
}

#[test]
fn instructions_arbitrary_typo_exits_2() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args(["--json", "instructions", "xyz_typo"])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "instructions.unknown_kind");
}

// ----- 6.4 Change context: --change <id> existence check -----

#[test]
fn instructions_change_not_found_exits_2() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args([
            "--json",
            "instructions",
            "proposal",
            "--change",
            "nonexistent-change",
        ])
        .assert()
        .failure()
        .code(2);
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["error"]["code"], "change.not_found");
    let msg = v["error"]["message"].as_str().expect("message");
    assert!(
        msg.contains("nonexistent-change"),
        "message should echo change id: {msg}"
    );
}

#[test]
fn instructions_change_existing_returns_schema_id() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    // 建立一個 change 讓 --change 命中
    speclink(&w)
        .args(["new", "change", "my-feature"])
        .assert()
        .success();

    let assert = speclink(&w)
        .args([
            "--json",
            "instructions",
            "proposal",
            "--change",
            "my-feature",
        ])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    assert_eq!(v["data"]["schema_id"], "spec-driven");
}

// ----- 6.5 + 6.6 --role / --discussion accepted but ignored + help text -----

#[test]
fn instructions_role_and_discussion_accepted_but_ignored() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args([
            "--json",
            "instructions",
            "proposal",
            "--role",
            "pm",
            "--discussion",
            "abc-123",
        ])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    let data = &v["data"];
    assert!(
        data["available_roles"].is_null(),
        "available_roles SHALL stay null in P1-3"
    );
    assert!(
        data["linked_changes_context"].is_null(),
        "linked_changes_context SHALL stay null in P1-3"
    );
    // warnings 不該因 --role / --discussion 而 emit
    let warnings = v["warnings"].as_array().expect("warnings array");
    let has_role_warning = warnings
        .iter()
        .any(|w| w["code"].as_str().is_some_and(|c| c.contains("role")));
    assert!(!has_role_warning, "--role should NOT emit a warning");
}

#[test]
fn instructions_help_text_mentions_phase_2_for_role_and_discussion() {
    // `instructions --help` 應該標示 --role / --discussion 是 reserved for Phase 2。
    let assert = AssertCommand::cargo_bin("speclink")
        .expect("binary")
        .args(["instructions", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    // 兩個 flag 都要在 help 內出現
    assert!(stdout.contains("--role"), "help missing --role: {stdout}");
    assert!(
        stdout.contains("--discussion"),
        "help missing --discussion: {stdout}"
    );
    // help text 標示 reserved-for-Phase-2 — 任一 substring 通過即可（中英混雜寬容）。
    assert!(
        stdout.contains("Phase 2") || stdout.contains("reserved"),
        "help should mark --role/--discussion as reserved for Phase 2: {stdout}"
    );
}

// ----- 6.7 Config missing → three fields null in envelope -----

#[test]
fn instructions_config_missing_returns_three_null_fields() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);
    // init_project 已建立 default config (`require_*_review: false`)；
    // context / locale / instructions 未設定 → 三欄應為 null。

    let assert = speclink(&w)
        .args(["--json", "instructions", "proposal"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    let data = &v["data"];
    assert!(
        data["context"].is_null(),
        "context SHALL be null when not set"
    );
    assert!(data["rules"].is_null(), "rules SHALL be null when not set");
    assert!(
        data["locale"].is_null(),
        "locale SHALL be null when not set"
    );
}

// ----- Bonus: dependencies envelope shape -----

#[test]
fn instructions_tasks_dependencies_include_three_predecessors() {
    let tmp = TempDir::new().unwrap();
    let w = canonical(tmp.path());
    init_project(&w);

    let assert = speclink(&w)
        .args(["--json", "instructions", "tasks"])
        .assert()
        .success();
    let v = parse_json(&assert.get_output().stdout);
    let deps = v["data"]["dependencies"].as_array().expect("deps array");
    assert_eq!(deps.len(), 3);
    let kinds: Vec<&str> = deps
        .iter()
        .map(|d| d["kind"].as_str().expect("kind"))
        .collect();
    assert_eq!(kinds, vec!["proposal", "spec", "design"]);
    // capability is always null in P1-3
    for d in deps {
        assert!(d["capability"].is_null());
    }
}

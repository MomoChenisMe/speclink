//! Integration tests for `speclink describe-tools`.
//!
//! 對應 specs/describe-tools-cli/spec.md 5 個 Requirement。
//! Read-only：所有 case 都跑在 empty tempdir、不需 git / .speclink 環境。

use std::fs;
use std::path::Path;

use assert_cmd::Command as AssertCommand;
use serde_json::Value;
use tempfile::TempDir;

fn cmd(dir: &Path) -> AssertCommand {
    let mut c = AssertCommand::cargo_bin("speclink").expect("speclink bin");
    c.current_dir(dir);
    c
}

fn run_ok(dir: &Path, args: &[&str]) -> Value {
    let out = cmd(dir).args(args).arg("--json").output().expect("spawn");
    assert!(
        out.status.success(),
        "expected success; got status={:?}\nstdout={}\nstderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    serde_json::from_slice(&out.stdout).expect("valid JSON envelope")
}

fn data(env: &Value) -> &Value {
    env.get("data").expect("envelope.data")
}

#[test]
fn describe_tools_default_invocation_emits_curated_12_json() {
    // Requirement: `speclink describe-tools` SHALL emit catalogue subsets in three supported formats
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools"]);
    assert_eq!(env["ok"], true);
    assert_eq!(data(&env)["format"], "json");
    let arr = data(&env)["content"].as_array().expect("array");
    assert_eq!(arr.len(), 12, "curated subset must be exactly 12 ops");
    for el in arr {
        let obj = el.as_object().expect("object");
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("description"));
        assert!(obj.contains_key("parameters"));
    }
}

#[test]
fn describe_tools_full_flag_emits_37_json() {
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools", "--full"]);
    let arr = data(&env)["content"].as_array().expect("array");
    assert_eq!(arr.len(), 37);
}

#[test]
fn describe_tools_format_text_emits_markdown_table() {
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools", "--format", "text"]);
    assert_eq!(data(&env)["format"], "text");
    let s = data(&env)["content"].as_str().expect("text string");
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[0].starts_with('|'), "first line must start with |");
    assert!(lines[1].contains("---"), "second line is separator");
    let data_rows = lines.len() - 2;
    assert_eq!(data_rows, 12, "12 curated rows + header + separator");
}

#[test]
fn describe_tools_format_copilot_sdk_emits_define_tool_descriptors() {
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools", "--format", "copilot-sdk"]);
    assert_eq!(data(&env)["format"], "copilot-sdk");
    let arr = data(&env)["content"].as_array().expect("array");
    assert_eq!(arr.len(), 12);
    for el in arr {
        let obj = el.as_object().expect("object");
        assert_eq!(obj.len(), 3, "copilot-sdk must have exactly 3 keys");
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("description"));
        assert!(obj.contains_key("parameters"));
        assert!(!obj.contains_key("id"), "copilot-sdk must omit id");
    }
}

#[test]
fn describe_tools_categories_change_filter_change_delete_returns_one() {
    // Requirement: Filter flags SHALL apply as AND intersection
    let tmp = TempDir::new().unwrap();
    let env = run_ok(
        tmp.path(),
        &[
            "describe-tools",
            "--full",
            "--categories",
            "change",
            "--filter",
            "change.delete",
        ],
    );
    let arr = data(&env)["content"].as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "change.delete");
}

#[test]
fn describe_tools_empty_intersection_returns_empty_array() {
    let tmp = TempDir::new().unwrap();
    let env = run_ok(
        tmp.path(),
        &[
            "describe-tools",
            "--full",
            "--categories",
            "change",
            "--filter",
            "discuss.new",
        ],
    );
    let arr = data(&env)["content"].as_array().expect("array");
    assert!(arr.is_empty(), "empty intersection must produce []");
}

#[test]
fn describe_tools_phases_discuss_returns_only_discuss_ids() {
    let tmp = TempDir::new().unwrap();
    let env = run_ok(
        tmp.path(),
        &["describe-tools", "--full", "--phases", "discuss"],
    );
    let arr = data(&env)["content"].as_array().expect("array");
    assert!(!arr.is_empty());
    for el in arr {
        let id = el["id"].as_str().expect("id");
        assert!(
            id.starts_with("discuss.") || id == "instructions.get",
            "phase=discuss returned non-discuss id: {id}"
        );
    }
}

#[test]
fn describe_tools_runs_outside_speclink_project() {
    // Requirement: `describe-tools` SHALL be read-only and require no project context
    let tmp = TempDir::new().unwrap();
    // 確認 tempdir 沒有 .git/ 也沒有 .speclink/
    assert!(!tmp.path().join(".git").exists());
    assert!(!tmp.path().join(".speclink").exists());
    let env = run_ok(tmp.path(), &["describe-tools"]);
    assert_eq!(env["ok"], true);
    let arr = data(&env)["content"].as_array().expect("array");
    assert_eq!(arr.len(), 12);
}

fn snapshot_dir(p: &Path) -> Vec<String> {
    let mut entries: Vec<String> = fs::read_dir(p)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();
    entries
}

#[test]
fn describe_tools_filesystem_untouched() {
    let tmp = TempDir::new().unwrap();
    let before = snapshot_dir(tmp.path());
    let _ = run_ok(tmp.path(), &["describe-tools", "--full"]);
    let after = snapshot_dir(tmp.path());
    assert_eq!(before, after, "filesystem must be untouched");
}

// ----- Error path -----

#[test]
fn describe_tools_format_mcp_exits_2_with_format_not_supported() {
    // Requirement: Unsupported formats SHALL fail fast with `tool.format_not_supported`
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["describe-tools", "--format", "mcp", "--json"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_slice(&out.stdout).expect("JSON envelope");
    assert_eq!(env["ok"], false);
    assert_eq!(env["error"]["code"], "tool.format_not_supported");
    let hint = env["error"]["hint"].as_str().unwrap_or("");
    assert!(
        hint.contains("deferred") || hint.contains("post-MVP"),
        "hint should mention deferred / post-MVP status; got: {hint}"
    );
}

#[test]
fn describe_tools_format_banana_rejected_by_clap() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["describe-tools", "--format", "banana"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid value"),
        "clap should reject invalid format value; stderr={stderr}"
    );
}

#[test]
fn describe_tools_filter_unknown_op_exits_2_with_tool_unknown_op() {
    // Requirement: Unknown filter values SHALL be rejected with category-specific error codes
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["describe-tools", "--filter", "no.such.op", "--json"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_slice(&out.stdout).expect("JSON envelope");
    assert_eq!(env["error"]["code"], "tool.unknown_op");
    assert!(
        env["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("no.such.op"),
        "message must mention the offending id"
    );
}

#[test]
fn describe_tools_categories_bogus_exits_2_with_tool_unknown_category() {
    let tmp = TempDir::new().unwrap();
    let out = cmd(tmp.path())
        .args(["describe-tools", "--categories", "bogus", "--json"])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2));
    let env: Value = serde_json::from_slice(&out.stdout).expect("JSON envelope");
    assert_eq!(env["error"]["code"], "tool.unknown_category");
    assert!(
        env["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("bogus"),
        "message must mention the offending category"
    );
}

// ----- insta snapshots (task 8.5) -----

fn redact_request_id(mut env: Value) -> Value {
    if let Some(map) = env.as_object_mut() {
        if map.contains_key("requestId") {
            map["requestId"] = Value::String("00000000-0000-4000-8000-000000000000".into());
        }
    }
    env
}

#[test]
fn snapshot_describe_tools_envelope() {
    // describe_tools_envelope.snap — single-op envelope shape stays stable
    let tmp = TempDir::new().unwrap();
    let env = run_ok(
        tmp.path(),
        &["describe-tools", "--full", "--filter", "change.create"],
    );
    let env = redact_request_id(env);
    insta::assert_snapshot!(
        "describe_tools_envelope",
        serde_json::to_string_pretty(&env).unwrap()
    );
}

#[test]
fn snapshot_describe_tools_default_json() {
    // describe_tools_default_json.snap — curated 12 ops id+name only
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools"]);
    let arr = env["data"]["content"].as_array().expect("array");
    let summary: Vec<Value> = arr
        .iter()
        .map(|el| {
            serde_json::json!({
                "id": el["id"].clone(),
                "name": el["name"].clone(),
            })
        })
        .collect();
    insta::assert_snapshot!(
        "describe_tools_default_json",
        serde_json::to_string_pretty(&Value::Array(summary)).unwrap()
    );
}

#[test]
fn snapshot_describe_tools_full_text() {
    // describe_tools_full_text.snap — full markdown table
    let tmp = TempDir::new().unwrap();
    let env = run_ok(
        tmp.path(),
        &["describe-tools", "--format", "text", "--full"],
    );
    let s = env["data"]["content"].as_str().expect("text").to_string();
    insta::assert_snapshot!("describe_tools_full_text", s);
}

#[test]
fn snapshot_describe_tools_curated_copilot_sdk() {
    // describe_tools_curated_copilot_sdk.snap — name+description only (skip parameters for stability)
    let tmp = TempDir::new().unwrap();
    let env = run_ok(tmp.path(), &["describe-tools", "--format", "copilot-sdk"]);
    let arr = env["data"]["content"].as_array().expect("array");
    let summary: Vec<Value> = arr
        .iter()
        .map(|el| {
            serde_json::json!({
                "name": el["name"].clone(),
                "description": el["description"].clone(),
            })
        })
        .collect();
    insta::assert_snapshot!(
        "describe_tools_curated_copilot_sdk",
        serde_json::to_string_pretty(&Value::Array(summary)).unwrap()
    );
}

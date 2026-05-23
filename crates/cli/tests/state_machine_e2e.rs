//! Walking-skeleton 端到端 acceptance test — design.md「Walking-skeleton 端到端 acceptance」
//! 11 步序列。

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use assert_cmd::Command as AssertCommand;
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
        .args(["config", "user.email", "t@e.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "t"])
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

fn write_stdin(
    working: &Path,
    kind: &str,
    change: &str,
    cap: Option<&str>,
    body: &[u8],
) -> std::process::Output {
    let bin = assert_cmd::cargo::cargo_bin("speclink");
    let mut cmd = Command::new(bin);
    cmd.current_dir(working)
        .arg("--json")
        .arg("new")
        .arg("artifact")
        .arg(kind)
        .arg("--change")
        .arg(change)
        .arg("--stdin");
    if let Some(c) = cap {
        cmd.arg("--capability").arg(c);
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body)
        .expect("write");
    child.wait_with_output().expect("wait")
}

fn data(out: &std::process::Output) -> serde_json::Value {
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    env["data"].clone()
}

fn warnings(out: &std::process::Output) -> Vec<serde_json::Value> {
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    env["warnings"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .clone()
}

fn show_state(working: &Path, change: &str) -> String {
    let out = speclink(working)
        .args(["--json", "show", "change", change])
        .output()
        .expect("show");
    assert!(out.status.success());
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    env["data"]["change"]["state"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[test]
fn walking_skeleton_eleven_step_end_to_end_passes() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    // Step 1: init
    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();

    // Step 2: new change wse-demo
    speclink(&working)
        .args(["--json", "new", "change", "wse-demo"])
        .assert()
        .success();

    // Step 3: write proposal only → state stays `proposing` (DAG incomplete)
    let out = write_stdin(&working, "proposal", "wse-demo", None, b"## Why\n");
    assert!(out.status.success());
    assert!(
        !warnings(&out)
            .iter()
            .any(|w| w["code"] == "state_transitioned"),
        "step 3: no transition yet"
    );
    assert_eq!(show_state(&working, "wse-demo"), "proposing");

    // Step 4: write spec (still missing tasks) → state stays `proposing`
    let out = write_stdin(
        &working,
        "spec",
        "wse-demo",
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    assert!(out.status.success());
    assert!(
        !warnings(&out)
            .iter()
            .any(|w| w["code"] == "state_transitioned"),
        "step 4: still no transition"
    );
    assert_eq!(show_state(&working, "wse-demo"), "proposing");

    // Step 5: write tasks → DAG complete → state == ready + state_transitioned warning
    let out = write_stdin(
        &working,
        "tasks",
        "wse-demo",
        None,
        b"- [ ] step1\n- [ ] step2\n",
    );
    assert!(out.status.success());
    assert!(
        warnings(&out)
            .iter()
            .any(|w| w["code"] == "state_transitioned"),
        "step 5: DAG complete emits state_transitioned"
    );
    assert_eq!(show_state(&working, "wse-demo"), "ready");

    // Step 6: apply start → state == in_progress, actor != null
    let out = speclink(&working)
        .args(["--json", "apply", "start", "wse-demo", "--actor", "claude"])
        .output()
        .expect("start");
    assert!(out.status.success());
    let d = data(&out);
    assert_eq!(d["state"], "in_progress");
    assert!(!d["actor"].is_null());

    // Step 7: task list returns 2 entries
    let out = speclink(&working)
        .args(["--json", "task", "list", "--change", "wse-demo"])
        .output()
        .expect("list");
    let d = data(&out);
    assert_eq!(d["tasks"].as_array().unwrap().len(), 2);

    // Step 8: task done 1 → all_tasks_done=false
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "wse-demo"])
        .output()
        .expect("done 1");
    let d = data(&out);
    assert_eq!(d["all_tasks_done"], false);

    // Step 9: task done 2 → all_tasks_done=true, auto_transitioned=false (walking-skeleton)
    let out = speclink(&working)
        .args(["--json", "task", "done", "2", "--change", "wse-demo"])
        .output()
        .expect("done 2");
    let d = data(&out);
    assert_eq!(d["all_tasks_done"], true);
    assert_eq!(d["auto_transitioned"], false);
    assert_eq!(d["state"], "in_progress");

    // Step 10: apply pause → state == ready, actor cleared
    let out = speclink(&working)
        .args(["--json", "apply", "pause", "wse-demo"])
        .output()
        .expect("pause");
    let d = data(&out);
    assert_eq!(d["state"], "ready");
    assert!(d["actor"].is_null());

    // Step 11: apply start again → idempotent reassign, state in_progress
    let out = speclink(&working)
        .args(["--json", "apply", "start", "wse-demo", "--actor", "cursor"])
        .output()
        .expect("start#2");
    let d = data(&out);
    assert_eq!(d["state"], "in_progress");
    assert_eq!(d["actor"]["agent_host"], "cursor");
}

// ----- task 12.1：config set rules.require_code_review true holds at code_reviewing -----

#[test]
fn config_set_require_code_review_true_holds_in_progress_through_code_reviewing() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();

    // Write all three DAG members.
    let out = write_stdin(&working, "proposal", "demo", None, b"## Why\n");
    assert!(out.status.success());
    let out = write_stdin(
        &working,
        "spec",
        "demo",
        Some("auth"),
        b"## ADDED Requirements\n",
    );
    assert!(out.status.success());
    let out = write_stdin(&working, "tasks", "demo", None, b"- [ ] t1\n");
    assert!(out.status.success());
    assert_eq!(show_state(&working, "demo"), "ready");

    speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "claude"])
        .assert()
        .success();

    // Flip require_code_review=true mid-cycle.
    speclink(&working)
        .args([
            "--json",
            "config",
            "set",
            "rules.require_code_review",
            "true",
        ])
        .assert()
        .success();

    // Last task done → policy now says transition to code_reviewing.
    let out = speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .output()
        .expect("done last");
    assert!(out.status.success());
    let d = data(&out);
    assert_eq!(
        d["state"], "code_reviewing",
        "require_code_review=true SHALL transition to code_reviewing"
    );
    assert_eq!(d["auto_transitioned"], true);

    // archive 此時應被拒：A5 不接 review.approve，archive 對 code_reviewing 不在 transition
    // table（design §6.2）→ exit 7、`state.transition_invalid`。
    let out = speclink(&working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive");
    assert!(!out.status.success());
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["error"]["code"], "state.transition_invalid");
}

// ----- task 12.2：A5 沒破 A4 既有 walking-skeleton archive happy path -----

#[test]
fn walking_skeleton_happy_path_still_completes_archive_after_a5() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    speclink(&working)
        .args(["--json", "init"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "new", "change", "demo"])
        .assert()
        .success();

    let _ = write_stdin(&working, "proposal", "demo", None, b"## Why\n");
    let _ = write_stdin(
        &working,
        "spec",
        "demo",
        Some("foo"),
        b"## ADDED Requirements\n\n### Requirement: x SHALL y\n\n#### Scenario: z\n\n- a\n",
    );
    let _ = write_stdin(&working, "tasks", "demo", None, b"- [ ] t1\n- [ ] t2\n");

    speclink(&working)
        .args(["--json", "apply", "start", "demo", "--actor", "claude"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "task", "done", "1", "--change", "demo"])
        .assert()
        .success();
    speclink(&working)
        .args(["--json", "task", "done", "2", "--change", "demo"])
        .assert()
        .success();

    // Walking-skeleton 默認 require_code_review=false → archive 直接 OK。
    let out = speclink(&working)
        .args(["--json", "archive", "demo"])
        .output()
        .expect("archive");
    assert!(
        out.status.success(),
        "archive SHALL succeed under walking-skeleton defaults; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["data"]["state"], "archived");
}

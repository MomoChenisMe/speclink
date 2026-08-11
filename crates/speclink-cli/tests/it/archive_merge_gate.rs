//! `speclink archive` 的 fail-closed 合併守門 — fs 模式契約
//! (spec archive-merge「封存合併 fail-closed 守門」「兩階段合併計畫與零半套寫入」)。
//!
//! 過期或自相矛盾的 delta 以非零 exit code 拒絕:stderr 逐條列明 capability／
//! 操作／需求名／原因並附 drift → ingest 補救動線,正典未動、無 snapshot、
//! change 留在原位。批次封存的預檢讀同一判定,以拒絕語意提前過濾。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const CANONICAL: &str = "# demo-cap Specification\n\n## Purpose\n\nDemo.\n\n## Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
/// 撞名的 ADDED:正典已有「Demo works」——舊引擎靜默跳過,新引擎拒絕。
const STALE_DELTA: &str = "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 一份完備的 change `demo`(任務全勾),delta 與正典由呼叫端給定;無 git,
    /// @trace 探測 fail soft。
    fn new(tag: &str, delta: &str, canonical: Option<&str>) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-merge-gate-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("demo-cap")).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [x] 1.1 a\n").unwrap();
        std::fs::write(change.join("specs").join("demo-cap").join("spec.md"), delta).unwrap();
        if let Some(text) = canonical {
            let canon = dir.join("openspec").join("specs").join("demo-cap");
            std::fs::create_dir_all(&canon).unwrap();
            std::fs::write(canon.join("spec.md"), text).unwrap();
        }
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn canonical_spec(&self) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("specs").join("demo-cap").join("spec.md"),
        )
        .unwrap()
    }

    fn change_dir(&self) -> PathBuf {
        self.dir.join("openspec").join("changes").join("demo")
    }

    fn snapshots_dir(&self) -> PathBuf {
        self.dir.join(".speclink").join("snapshots")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn stale_delta_refuses_with_a_named_violation_and_the_remediation_route() {
    // spec Scenario「過期 ADDED 被拒絕」的人眼面:--no-color 下 stderr 逐條列明
    // capability／操作／需求名／原因,並指出 drift → ingest 的補救動線。
    let p = TempProject::new("refuse", STALE_DELTA, Some(CANONICAL));
    let out = p.run(&["archive", "demo", "--no-color"]);
    assert!(!out.status.success(), "a stale delta must refuse archive");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("demo-cap"), "capability named: {stderr}");
    assert!(stderr.contains("ADDED"), "operation named: {stderr}");
    assert!(stderr.contains("Demo works"), "requirement named: {stderr}");
    assert!(stderr.contains("already exists"), "reason given: {stderr}");
    assert!(stderr.contains("speclink drift demo"), "drift remediation named: {stderr}");
    assert!(stderr.contains("speclink-ingest demo"), "ingest remediation named: {stderr}");
    assert!(!stderr.contains("\u{1b}["), "--no-color leaves no ANSI escapes: {stderr}");
}

#[test]
fn a_refused_archive_leaves_zero_file_effect() {
    // spec「兩階段合併計畫與零半套寫入」:拒絕後正典逐位元不變、無 snapshot 目錄、
    // change 仍在進行區原位。
    let p = TempProject::new("zero-effect", STALE_DELTA, Some(CANONICAL));
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success());
    assert_eq!(p.canonical_spec(), CANONICAL, "canonical spec byte-identical");
    assert!(p.change_dir().join("tasks.md").is_file(), "change stays in the active area");
    assert!(!p.snapshots_dir().exists(), "no snapshot directory was created");
}

#[test]
fn no_validate_does_not_unlock_the_merge_gate() {
    // spec Scenario「no-validate 不解鎖守門」:文件驗證略過,合併守門照常拒絕。
    let p = TempProject::new("no-validate", STALE_DELTA, Some(CANONICAL));
    let out = p.run(&["archive", "demo", "--no-validate"]);
    assert!(!out.status.success(), "--no-validate must not unlock the merge gate");
    assert!(stderr_of(&out).contains("already exists"), "gate still speaks: {}", stderr_of(&out));
}

#[test]
fn bulk_archive_prefilters_with_the_same_verdict_in_refusal_wording() {
    // spec「過期判定單源共用」的批次面:預檢讀引擎的判定,以拒絕語意提前過濾,
    // 該 change 未被封存。
    let p = TempProject::new("bulk", STALE_DELTA, Some(CANONICAL));
    let out = p.run(&["archive", "--all", "--no-color"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Skipped: demo"), "the change is pre-filtered: {stdout}");
    assert!(stdout.contains("archive would refuse"), "refusal wording, not skip: {stdout}");
    assert!(p.change_dir().join("tasks.md").is_file(), "change stays in the active area");
    assert_eq!(p.canonical_spec(), CANONICAL, "canonical spec untouched");
}

#[test]
fn bulk_archive_names_the_missing_purpose_in_the_skip_reason() {
    // spec archive-merge「新 capability 缺 Purpose 的違規呈現三處一致」的批次面：
    // 略過原因不只給計數，點名缺 `## Purpose` 的 capability。
    const NO_PURPOSE_DELTA: &str = "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let p = TempProject::new("bulk-purpose", NO_PURPOSE_DELTA, None);
    let out = p.run(&["archive", "--all", "--no-color"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Skipped: demo"), "the change is pre-filtered: {stdout}");
    assert!(stdout.contains("## Purpose"), "the real cause is named: {stdout}");
    assert!(stdout.contains("demo-cap"), "the offending capability is named: {stdout}");
}

#[test]
fn a_refused_archive_leaves_tasks_untouched_even_with_mark_tasks_complete() {
    // 零檔案效果涵蓋 --mark-tasks-complete 的前置全勾:守門拒絕時 tasks.md
    // 必須逐位元不變——runtime 的守門順序與 guard_meta 同(先守門、後預寫)。
    let p = TempProject::new("mark-gate", STALE_DELTA, Some(CANONICAL));
    const OPEN_TASKS: &str = "- [ ] 1.1 a\n";
    std::fs::write(p.change_dir().join("tasks.md"), OPEN_TASKS).unwrap();
    let out = p.run(&["archive", "demo", "--mark-tasks-complete"]);
    assert!(!out.status.success(), "the merge gate still refuses");
    assert!(stderr_of(&out).contains("already exists"), "gate speaks: {}", stderr_of(&out));
    let tasks = std::fs::read_to_string(p.change_dir().join("tasks.md")).unwrap();
    assert_eq!(tasks, OPEN_TASKS, "tasks.md byte-identical after a refused archive");
}

#[test]
fn bulk_archive_with_skip_specs_is_not_prefiltered_by_the_merge_gate() {
    // --skip-specs 整段跳過規格套用(spec「封存合併 fail-closed 守門」),引擎
    // 守門不會跑,預檢也就不得以「archive would refuse」濾掉——否則預檢與引擎分歧。
    let p = TempProject::new("bulk-skip", STALE_DELTA, Some(CANONICAL));
    let out = p.run(&["archive", "--all", "--skip-specs", "--no-color"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("Skipped: demo"),
        "skip-specs bulk must not pre-filter on the merge gate: {stdout}"
    );
    assert!(!p.change_dir().exists(), "change archived: {stdout}");
    assert_eq!(p.canonical_spec(), CANONICAL, "canonical spec untouched (specs skipped)");
}

#[test]
fn a_clean_delta_still_archives_and_merges() {
    // 回歸網:未過期的 delta 照常封存並併入正典——守門只擋不符項。
    const FRESH: &str = "## ADDED Requirements\n\n### Requirement: Demo also logs\n\nIt SHALL log.\n\n#### Scenario: logs\n\n- **WHEN** used\n- **THEN** logs\n";
    let p = TempProject::new("clean", FRESH, Some(CANONICAL));
    let out = p.run(&["archive", "demo"]);
    assert!(out.status.success(), "a clean delta archives: {}", stderr_of(&out));
    let canon = p.canonical_spec();
    assert!(canon.contains("### Requirement: Demo works"), "existing requirement kept: {canon}");
    assert!(canon.contains("### Requirement: Demo also logs"), "delta merged in: {canon}");
    assert!(!p.change_dir().exists(), "change moved into the archive");
}

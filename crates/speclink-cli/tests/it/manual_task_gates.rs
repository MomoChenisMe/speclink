//! `[M]` 手動測試標記的端到端契約 — fs 模式(manual-task-marker spec 全條文、
//! review-station／verify-station 的守門與失效判定、change-lifecycle
//! 「封存的章失效守門」)。
//!
//! 覆蓋 design Implementation Contract 的行為 1–5:payload 曝光、verify
//! add-round 的放行與拒絕、兩站 stamp 的放行與拒絕、失效判定四情境、封存的
//! 章失效守門(含任務守門先拒的順序)。守門的判準一律是「寫碼任務」——`[M]`
//! 任務不計入,但封存仍要求全勾(手測強制力保留)。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const DELTA_SPEC: &str = "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
/// 兩個寫碼任務全勾＋一個未勾的 `[M]`:寫碼任務全完成、全量 2/3。
const CODE_DONE_MANUAL_OPEN: &str = "- [x] 1.1 a\n- [x] 1.2 b\n- [ ] [M] 1.3 手動驗證\n";
/// 一個寫碼任務未勾＋一個未勾的 `[M]`:寫碼任務 1/2,守門該拒。
const CODE_OPEN: &str = "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] [M] 1.3 手動驗證\n";
/// 全勾(含 `[M]`)——封存的任務完成度守門要求的狀態。
const ALL_DONE: &str = "- [x] 1.1 a\n- [x] 1.2 b\n- [x] [M] 1.3 手動驗證\n";

/// 審查／驗證輪的 scope 指向這支受審檔——內容錨的判定對象。
const SCOPE_FILE: &str = "src/lib.rs";
const SCOPE_BODY: &str = "pub fn a() {}\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 用後即丟的專案:一個結構完整的 change `demo`(proposal + delta spec +
    /// 指定的 tasks.md)與一支受審程式檔;無 git——@trace 探測 fail soft。
    fn new(tag: &str, tasks_md: &str) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-manual-gates-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join(SCOPE_FILE), SCOPE_BODY).unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("demo-cap")).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), tasks_md).unwrap();
        std::fs::write(change.join("specs").join("demo-cap").join("spec.md"), DELTA_SPEC).unwrap();
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

    /// 自 stdin 落一輪乾淨(零 findings)的工單。
    fn add_clean_round(&self, station: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args([station, "add-round", "demo", "--stdin"])
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn speclink");
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(format!("**Scope**: {SCOPE_FILE}\n").as_bytes())
            .expect("write round");
        child.wait_with_output().expect("round output")
    }

    fn change_dir(&self) -> PathBuf {
        self.dir.join("openspec").join("changes").join("demo")
    }

    fn meta(&self) -> String {
        std::fs::read_to_string(self.change_dir().join(".openspec.yaml")).expect("meta")
    }

    fn write_tasks(&self, tasks_md: &str) {
        std::fs::write(self.change_dir().join("tasks.md"), tasks_md).expect("write tasks");
    }

    /// 改動受審檔的內容——打破章的內容錨。
    fn touch_scope_file(&self) {
        std::fs::write(self.dir.join(SCOPE_FILE), format!("{SCOPE_BODY}pub fn b() {{}}\n"))
            .expect("modify scope file");
    }

    fn archived_demo_exists(&self) -> bool {
        let archive = self.dir.join("openspec").join("changes").join("archive");
        std::fs::read_dir(archive).is_ok_and(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path().file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with("-demo"))
            })
        })
    }

    /// 兩站都落一輪乾淨工單並蓋章——封存守門測試的前置。
    fn stamp_both(&self) {
        for station in ["review", "verify"] {
            assert!(self.add_clean_round(station).status.success(), "{station} round");
            let out = self.run(&[station, "stamp", "demo", "--agent", "claude"]);
            assert!(out.status.success(), "{station} stamp: {:?}", String::from_utf8_lossy(&out.stderr));
        }
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// --- 行為 1:payload 曝光(manual-task-marker spec「任務 payload 的 manual 欄位與寫碼進度」)---

#[test]
fn apply_payload_exposes_manual_flag_and_code_progress() {
    // spec Scenario「手動任務上線 payload」:[M] 任務 manual=true,progress 的
    // code 三欄排除該任務;描述不含標記。
    let p = TempProject::new("payload", CODE_DONE_MANUAL_OPEN);
    let out = p.run(&["instructions", "apply", "--change", "demo", "--json"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("apply payload is valid JSON");
    assert_eq!(v["progress"]["total"], 3);
    assert_eq!(v["progress"]["complete"], 2);
    assert_eq!(v["progress"]["codeTotal"], 2, "[M] task is out of the code counts");
    assert_eq!(v["progress"]["codeComplete"], 2);
    assert_eq!(v["progress"]["codeRemaining"], 0, "code work is finished");
    assert_eq!(v["tasks"][2]["manual"], true);
    assert_eq!(v["tasks"][2]["description"], "1.3 手動驗證", "description drops the marker");
    assert_eq!(v["tasks"][0]["manual"], false);
}

#[test]
fn code_counts_equal_full_counts_without_manual_tasks() {
    // spec Scenario「無手動任務時欄位一致」:無 [M] 時 code 三欄＝全量三欄。
    let p = TempProject::new("nomanual", "- [x] 1.1 a\n- [ ] 1.2 b\n");
    let out = p.run(&["instructions", "apply", "--change", "demo", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    for (full, code) in [("total", "codeTotal"), ("complete", "codeComplete"), ("remaining", "codeRemaining")] {
        assert_eq!(v["progress"][full], v["progress"][code], "{full} vs {code}");
    }
}

// --- 行為 2:verify add-round 的落工單守門(verify-station spec「驗證工單的建立與追加」)---

#[test]
fn verify_add_round_lands_when_only_manual_tasks_remain() {
    // spec Scenario「僅餘手動任務可落工單」:寫碼全勾 → 放行、工單建立。
    let p = TempProject::new("vround-ok", CODE_DONE_MANUAL_OPEN);
    let out = p.add_clean_round("verify");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(p.change_dir().join("verify.md").is_file(), "ticket must be created");
}

#[test]
fn verify_add_round_refuses_while_a_code_task_is_open() {
    // spec Scenario「寫碼任務未完成即拒絕落工單」:1/2 → 拒絕、無工單、點名寫碼任務。
    let p = TempProject::new("vround-no", CODE_OPEN);
    let out = p.add_clean_round("verify");
    assert!(!out.status.success(), "open code task must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1/2"), "counts exclude the [M] task: {stderr}");
    assert!(stderr.contains("code task"), "message names code tasks: {stderr}");
    assert!(!p.change_dir().join("verify.md").exists(), "no ticket may appear");
}

// --- 行為 3:兩站蓋章守門(review-station／verify-station 的蓋章條文)---

#[test]
fn both_stations_stamp_when_only_manual_tasks_remain() {
    // spec Scenario「僅餘手動任務可蓋章」＋Example「蓋章寫入的任務錨」:
    // 蓋章成功、工單刪除,錨記全任務總數 3(含未勾的 [M])。
    let p = TempProject::new("stamp-ok", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    let meta = p.meta();
    for field in ["reviewed_at:", "reviewed_tasks_total: 3", "verified_at:", "verified_tasks_total: 3"] {
        assert!(meta.contains(field), "meta must carry {field}: {meta}");
    }
    assert!(!p.change_dir().join("review.md").exists(), "review ticket deleted");
    assert!(!p.change_dir().join("verify.md").exists(), "verify ticket deleted");
}

#[test]
fn stamping_refuses_while_a_code_task_is_open() {
    // 寫碼任務未完成 → 兩站蓋章皆拒,訊息點名寫碼任務計數,meta 不變。
    let p = TempProject::new("stamp-no", CODE_DONE_MANUAL_OPEN);
    assert!(p.add_clean_round("review").status.success(), "round lands while code is done");
    p.write_tasks(CODE_OPEN); // 工單開立後退勾一個寫碼任務
    let out = p.run(&["review", "stamp", "demo", "--agent", "claude"]);
    assert!(!out.status.success(), "open code task must refuse the stamp");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1/2"), "counts exclude the [M] task: {stderr}");
    assert_eq!(p.meta(), META, "refusal leaves metadata byte-identical");
    assert!(p.change_dir().join("review.md").is_file(), "ticket survives refusal");
}

// --- 行為 4 + 5:失效判定與封存的章失效守門(change-lifecycle spec 的 Example 表)---

#[test]
fn checking_a_manual_task_after_the_stamp_still_archives() {
    // Example 表第二列:兩章齊備、補勾 [M]、scope 檔零改動 → 封存放行。
    let p = TempProject::new("arch-manual", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.write_tasks(ALL_DONE);
    let out = p.run(&["archive", "demo"]);
    assert!(out.status.success(), "manual toggle must not stale the stamps: {}", String::from_utf8_lossy(&out.stderr));
    assert!(p.archived_demo_exists(), "change must land in the archive");
}

#[test]
fn touching_a_scope_file_after_the_stamp_refuses_archive() {
    // Example 表第一列:章齊備、scope 檔內容改變 → 拒絕、點名站別與檔案,零檔案效果。
    let p = TempProject::new("arch-stale", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.write_tasks(ALL_DONE);
    p.touch_scope_file();
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success(), "stale stamps must refuse archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("review") && stderr.contains("verify"), "both stations named: {stderr}");
    assert!(stderr.contains(SCOPE_FILE), "the changed file is named: {stderr}");
    assert!(!p.archived_demo_exists(), "no archived directory appears");
    assert!(p.change_dir().join("tasks.md").is_file(), "change stays in place");
}

#[test]
fn adding_a_task_after_the_stamp_refuses_archive() {
    // Example 表第三列:任務總數自 3 變 4 → 任務錨破 → 拒絕。
    let p = TempProject::new("arch-recount", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.write_tasks(&format!("{ALL_DONE}- [x] 1.4 late\n"));
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success(), "task anchor break must refuse archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("3 at stamp time, 4 now"), "counts explain the break: {stderr}");
    assert!(!p.archived_demo_exists(), "no archived directory appears");
}

#[test]
fn an_unstamped_change_archives_exactly_as_before() {
    // Example 表第四列:無章 → 放行,行為與守門引入前一致(scope 檔改過也無妨)。
    let p = TempProject::new("arch-nostamp", ALL_DONE);
    p.touch_scope_file();
    let out = p.run(&["archive", "demo"]);
    assert!(out.status.success(), "no stamp → no stale gate: {}", String::from_utf8_lossy(&out.stderr));
    assert!(p.archived_demo_exists(), "change must land in the archive");
}

#[test]
fn the_task_readiness_gate_refuses_before_the_stale_gate() {
    // 順序契約:未勾的 [M] 讓任務完成度守門先拒——訊息維持既有樣式,不提章失效。
    let p = TempProject::new("arch-order", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.touch_scope_file(); // 章同時已失效
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success(), "incomplete tasks must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("2/3 tasks complete"), "既有任務守門訊息: {stderr}");
    assert!(stderr.contains("--mark-tasks-complete"), "既有出路: {stderr}");
    assert!(!stderr.contains("no longer holds"), "章失效訊息不得搶先: {stderr}");
}

#[test]
fn mark_tasks_complete_leaves_tasks_untouched_when_stale_refuses() {
    // 拒絕路徑零寫入:--mark-tasks-complete 的前置全勾寫入之前先判章失效,
    // stale 拒絕時 tasks.md 逐位元不變——未手測的 [M] 不得被代勾。
    let p = TempProject::new("arch-prewrite", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.touch_scope_file(); // 內容錨破 → 兩章 stale
    let out = p.run(&["archive", "demo", "--mark-tasks-complete"]);
    assert!(!out.status.success(), "stale stamps must refuse archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no longer holds"), "拒絕來自章失效守門: {stderr}");
    let tasks = std::fs::read_to_string(p.change_dir().join("tasks.md")).expect("tasks");
    assert_eq!(tasks, CODE_DONE_MANUAL_OPEN, "refusal leaves tasks.md byte-identical");
    assert!(!p.archived_demo_exists(), "no archived directory appears");
}

#[test]
fn a_reopened_ticket_escapes_the_stale_gate_via_carry() {
    // 工單開立中的站,其舊章不入失效判定——該站的封存處置由未結工單守門
    // (--carry-*)承載,已被重開工單取代的章不得把 carry 堵成死路。
    let p = TempProject::new("arch-carry", CODE_DONE_MANUAL_OPEN);
    p.stamp_both();
    p.write_tasks(ALL_DONE);
    p.touch_scope_file(); // 兩章內容錨皆破
    for station in ["review", "verify"] {
        assert!(p.add_clean_round(station).status.success(), "reopen the {station} ticket");
    }
    let out = p.run(&["archive", "demo", "--carry-review", "--carry-verify"]);
    assert!(
        out.status.success(),
        "open tickets own the disposal — stale superseded stamps must not block: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.archived_demo_exists(), "change must land in the archive");
}

#[test]
fn bulk_archive_fails_fast_on_a_stale_stamp() {
    // change-lifecycle spec:批次封存沿既有 fail-fast 樣式——stale 章的拒絕
    // 中止批次並點名該 change,SHALL NOT 靜默跳過;後續 change 原地不動。
    let p = TempProject::new("arch-bulk", ALL_DONE);
    p.stamp_both();
    p.touch_scope_file(); // demo 的兩章 stale
    let later = p.dir.join("openspec").join("changes").join("later");
    std::fs::create_dir_all(later.join("specs").join("later-cap")).unwrap();
    std::fs::write(later.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-07-02\n")
        .unwrap();
    std::fs::write(later.join("proposal.md"), "## Why\n\nLater.\n").unwrap();
    std::fs::write(later.join("tasks.md"), "- [x] 1.1 a\n").unwrap();
    std::fs::write(
        later.join("specs").join("later-cap").join("spec.md"),
        DELTA_SPEC.replace("Demo works", "Later works"),
    )
    .unwrap();
    let out = p.run(&["archive", "--all"]);
    assert!(!out.status.success(), "a stale stamp must abort the bulk run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Bulk archive aborted at 'demo'"), "fail-fast names the change: {stdout}");
    assert!(!stdout.contains("Skipped: demo"), "stale is a refusal, never a skip: {stdout}");
    assert!(!p.archived_demo_exists(), "demo stays active");
    assert!(later.join("tasks.md").is_file(), "the later change stays untouched");
}

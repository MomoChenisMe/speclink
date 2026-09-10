use super::*;
use crate::store::Store;
use crate::teststore::TestStore;
use crate::workspace::Workspace;

const META_UNSTARTED: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";
const TASKS_TWO_OPEN: &str = "## 1. Group\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";

/// Throwaway host workspace rooted in the OS temp dir; removed on drop.
struct TempWs {
    ws: Workspace,
}

impl TempWs {
    fn new(tag: &str) -> TempWs {
        let dir = std::env::temp_dir().join(format!(
            "speclink-core-tasks-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempWs {
            ws: Workspace {
                root: dir,
                spec_dir_name: "openspec".to_string(),
            },
        }
    }

    /// Workspace root as a git repo carrying one dirty (untracked) code file.
    fn with_dirty_file(tag: &str, rel: &str) -> TempWs {
        let t = TempWs::new(tag);
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(&t.ws.root)
            .args(["init", "-q"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(ok, "git init failed");
        let p = t.ws.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "content\n").unwrap();
        t
    }

}

impl Drop for TempWs {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.ws.root);
    }
}

fn store_with(meta: &str, tasks_md: &str) -> TestStore {
    let store = TestStore::with_meta("demo", meta);
    store.put_artifact("demo", "tasks.md", tasks_md);
    store
}

#[test]
fn complete_first_task_marks_stamps_and_records_touched() {
    let t = TempWs::with_dirty_file("first", "src/app.rs");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);

    let out = complete(&store, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution { identity: Some("Tester <t@example.com>"), ..Default::default() }, TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap();

    assert!(!out.already);
    assert_eq!(out.description, "1.1 first task");
    let tasks = store.read_artifact("demo", "tasks.md").unwrap();
    assert!(tasks.contains("- [x] 1.1 first task"), "task 1 must be checked: {tasks}");
    assert!(tasks.contains("- [ ] 1.2 second task"), "task 2 must stay open: {tasks}");
    // Touched record gains this task's entry with the unclaimed dirty file.
    let json = store.read_evidence("demo").expect("touched record written");
    let rec: TouchedRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(rec.change, "demo");
    assert_eq!(rec.touched.len(), 1);
    assert_eq!(rec.touched[0].task_id, "1");
    assert_eq!(rec.touched[0].task_desc, "1.1 first task");
    assert!(rec.touched[0].files.contains(&"src/app.rs".to_string()));
    // Meta gains the work stamp; existing fields byte-for-byte preserved.
    let meta = store.meta("demo");
    assert!(
        meta.starts_with(META_UNSTARTED),
        "existing meta fields must be preserved verbatim: {meta}"
    );
    assert!(meta.contains(&format!("started_at: {}", util::today())));
    assert!(meta.contains("started_by: Tester <t@example.com>"));
    assert!(!meta.contains("started_with"));
}

#[test]
fn complete_without_new_dirty_files_appends_no_touched_entry() {
    // No .git in the workspace root → git_changed_files is empty → nothing appended,
    // no record file created (matches the CLI's current semantics).
    let t = TempWs::new("nodirty");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    let out = complete(&store, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default(), TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap();
    assert!(!out.already);
    assert_eq!(store.read_evidence("demo"), None, "no unclaimed dirty files must append nothing");
}

#[test]
fn complete_on_started_change_keeps_first_stamp_verbatim() {
    let started = format!(
        "{META_UNSTARTED}started_at: 2026-07-01\nstarted_by: First <first@example.com>\nstarted_with: claude\n"
    );
    let t = TempWs::new("started");
    let store = store_with(&started, TASKS_TWO_OPEN);
    complete(&store, "demo", &TaskAddr::Ordinal(2), &CompleteAttribution { identity: Some("Second <second@example.com>"), agent: Some("codex"), repo: None }, TouchedCandidates::ProbeWorkspace(&t.ws))
        .unwrap();
    assert_eq!(store.meta("demo"), started, "first stamp must be kept verbatim");
    assert_eq!(*store.meta_writes.borrow(), 0, "already-started change must not write meta");
}

#[test]
fn complete_already_done_task_reports_already_without_any_file_effect() {
    let t = TempWs::with_dirty_file("already", "src/lib.rs");
    let tasks_md = "- [x] 1.1 finished\n- [ ] 1.2 open\n";
    let store = store_with(META_UNSTARTED, tasks_md);
    let out = complete(&store, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution { identity: Some("Tester <t@example.com>"), ..Default::default() }, TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap();
    assert!(out.already);
    assert_eq!(out.description, "1.1 finished");
    assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), tasks_md);
    assert_eq!(*store.artifact_writes.borrow(), 0, "already-done must not rewrite tasks.md");
    assert_eq!(*store.meta_writes.borrow(), 0, "already-done must not stamp meta");
    assert_eq!(store.read_evidence("demo"), None, "already-done must not record touched files");
}

#[test]
fn complete_with_absent_identity_and_agent_stamps_only_started_at() {
    // Attribution follows the created_* rule: what the caller cannot attribute
    // is absent, not defaulted.
    let t = TempWs::new("absent");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    complete(&store, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default(), TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap();
    let meta = store.meta("demo");
    assert!(meta.contains("started_at: "));
    assert!(!meta.contains("started_by"));
    assert!(!meta.contains("started_with"));
}

#[test]
fn complete_out_of_range_task_id_errors_without_writes() {
    let t = TempWs::new("range");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    let err = complete(&store, "demo", &TaskAddr::Ordinal(5), &CompleteAttribution::default(), TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap_err();
    assert_eq!(err.to_string(), "Task 5 not found (total: 2)");
    assert_eq!(*store.artifact_writes.borrow(), 0);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

// Checked tasks in both bullet styles, one indented, one still open.
const TASKS_MIXED_DONE: &str =
    "## 1. Group\n\n- [x] 1.1 first task\n    - [x] 1.2 indented task\n* [X] 1.3 star task\n- [ ] 1.4 open task\n";

#[test]
fn uncomplete_flips_only_target_line_preserving_indent_and_trailing_newline() {
    let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
    let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(2)).unwrap();
    assert!(!out.already);
    assert_eq!(out.description, "1.2 indented task");
    assert_eq!(
        store.read_artifact("demo", "tasks.md").unwrap(),
        "## 1. Group\n\n- [x] 1.1 first task\n    - [ ] 1.2 indented task\n* [X] 1.3 star task\n- [ ] 1.4 open task\n"
    );
    // Pure state flip: tasks.md is the only write, meta stays byte-for-byte.
    assert_eq!(*store.artifact_writes.borrow(), 1);
    assert_eq!(*store.meta_writes.borrow(), 0, "uncomplete must not touch meta");
    assert_eq!(store.meta("demo"), META_UNSTARTED);
}

#[test]
fn uncomplete_star_bullet_keeps_style_and_no_trailing_newline() {
    let tasks_md = "- [x] 1.1 first\n* [X] 1.2 star task";
    let store = store_with(META_UNSTARTED, tasks_md);
    let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(2)).unwrap();
    assert!(!out.already);
    assert_eq!(out.description, "1.2 star task");
    assert_eq!(
        store.read_artifact("demo", "tasks.md").unwrap(),
        "- [x] 1.1 first\n* [ ] 1.2 star task",
        "star bullet style and absent trailing newline must be preserved"
    );
}

#[test]
fn uncomplete_already_open_task_reports_already_without_any_file_effect() {
    let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
    let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(4)).unwrap();
    assert!(out.already);
    assert_eq!(out.description, "1.4 open task");
    assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), TASKS_MIXED_DONE);
    assert_eq!(*store.artifact_writes.borrow(), 0, "already-open must not rewrite tasks.md");
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn uncomplete_out_of_range_task_id_errors_without_writes() {
    let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
    let err = uncomplete(&store, "demo", &TaskAddr::Ordinal(9)).unwrap_err();
    assert_eq!(err.to_string(), "Task 9 not found (total: 4)");
    assert_eq!(*store.artifact_writes.borrow(), 0);
    assert_eq!(*store.meta_writes.borrow(), 0);
}

#[test]
fn uncomplete_missing_tasks_md_errors() {
    let store = TestStore::with_meta("demo", META_UNSTARTED);
    let err = uncomplete(&store, "demo", &TaskAddr::Ordinal(1)).unwrap_err();
    assert_eq!(err.to_string(), "tasks.md not found for change 'demo'");
}

// --- Stable task ID: comment parsing, duplicate detection, generator ---

const TASKS_WITH_IDS: &str = "## 1. Group\n\n- [ ] 1.1 first task <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n- [x] 1.2 second task <!-- speclink-task:tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ -->\n";

#[test]
fn parse_extracts_stable_id_and_strips_comment_from_description() {
    let tasks = parse(TASKS_WITH_IDS);
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].description, "1.1 first task");
    assert_eq!(tasks[0].stable_id.as_deref(), Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"));
    assert_eq!(tasks[1].description, "1.2 second task");
    assert_eq!(tasks[1].stable_id.as_deref(), Some("tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ"));
}

#[test]
fn parse_without_comment_yields_no_stable_id_and_unchanged_description() {
    let tasks = parse(TASKS_TWO_OPEN);
    assert!(tasks.iter().all(|t| t.stable_id.is_none()), "no comment → no stable id");
    assert_eq!(tasks[0].description, "1.1 first task");
    assert_eq!(tasks[1].description, "1.2 second task");
    // sharp-edge：空 ID 註解不是身分——空字串 ID 會讓 key/定址靜默歧義。
    let empty = parse("- [ ] 1.1 x <!-- speclink-task: -->\n");
    assert!(empty[0].stable_id.is_none(), "empty marker id must not count as identity");
}

// --- spec「任務行的手動任務標記與解析」---

#[test]
fn parse_reads_manual_marker_and_strips_it_from_description() {
    let tasks = parse("- [ ] [M] 手測匯入\n- [ ] 寫解析器\n");
    assert!(tasks[0].manual, "[M] 前綴須解析為 manual");
    assert_eq!(tasks[0].description, "手測匯入", "描述須剝除標記");
    assert!(!tasks[1].manual, "無標記任務不得為 manual");
    assert_eq!(tasks[1].description, "寫解析器");
}

#[test]
fn parse_accepts_both_markers_in_either_order() {
    for line in ["- [x] [M] [P] 手測匯入\n", "- [x] [P] [M] 手測匯入\n"] {
        let t = &parse(line)[0];
        assert!(t.manual, "[M] 須順序不敏感：{line}");
        assert_eq!(t.description, "手測匯入", "兩標記皆須剝除：{line}");
    }
}

#[test]
fn parse_strips_legacy_parallel_marker_without_carrying_a_flag() {
    // spec Example「前綴解析」表逐列：任務行 → (manual, 描述)。
    // 案例表與 packages/ui/src/tasks.ts 的 stripMarkers 測試對齊——UI 解析
    // 與引擎同構,動標記規則兩處要一起改。
    let rows: [(&str, bool, &str); 7] = [
        ("- [ ] [M] 手測匯入\n", true, "手測匯入"),
        // 非測試類的手動任務同樣解析——標記語意不判讀描述內容。
        ("- [ ] [M] 至外部服務放置金鑰\n", true, "至外部服務放置金鑰"),
        ("- [x] [P] 舊任務\n", false, "舊任務"),
        ("- [x] [P] [M] 混用\n", true, "混用"),
        ("- [ ] 寫解析器\n", false, "寫解析器"),
        // checkbox 後恰一個空格才進標記槽:兩空格＝標記不成立、字面留在描述。
        ("- [ ]  [M] 手測\n", false, "[M] 手測"),
        // 標記後多餘空白剝除後修剪(display.trim())。
        ("- [ ] [M]  雙空格描述\n", true, "雙空格描述"),
    ];
    for (line, manual, desc) in rows {
        let t = &parse(line)[0];
        assert_eq!(t.manual, manual, "manual 判定：{line}");
        assert_eq!(t.description, desc, "描述須剝除全部前綴標記：{line}");
    }
    // 舊 `[P]` 只剝不承載——與同內容的無標記行解析結果全等
    let legacy = &parse("- [x] [P] 舊任務\n")[0];
    let plain = &parse("- [x] 舊任務\n")[0];
    assert_eq!((legacy.manual, &legacy.description), (plain.manual, &plain.description));
    // 至多一次：第二個 [P] 留在描述裡,不被吃掉
    assert_eq!(parse("- [ ] [P] [P] 兩個\n")[0].description, "[P] 兩個");
}

#[test]
fn flip_task_reports_marker_free_description() {
    // spec「任務行的手動任務標記與解析」:顯示描述 SHALL 剝除全部前綴標記——
    // 勾選回報的 desc 與 parse() 的乾淨描述必須同一份真相。
    let rows: [(&str, &str); 3] = [
        ("- [ ] [M] 手測匯入\n", "手測匯入"),
        ("- [ ] [P] 舊任務\n", "舊任務"),
        ("- [ ] [P] [M] 混用\n", "混用"),
    ];
    for (md, want) in rows {
        let (_, desc, _, _) = mark_done(md, 1).expect("task found");
        assert_eq!(desc, want, "flip 回報描述須剝除標記:{md}");
    }
}

#[test]
fn parse_leaves_unmarked_lines_untouched() {
    let tasks = parse(TASKS_WITH_IDS);
    assert!(tasks.iter().all(|t| !t.manual));
    assert_eq!(tasks[0].description, "1.1 first task");
    assert_eq!(tasks[0].stable_id.as_deref(), Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"));
}

// --- spec「標記位置的 change 驗證檢查」---

#[test]
fn misplaced_markers_follow_the_spec_example_table() {
    // spec Example「誤置判定」表逐列：任務行 → 判定。
    let rows: [(&str, Option<MisplacedMarker>); 5] = [
        ("- [ ] [M] 3.2 手測匯入\n", None),
        ("- [ ] 3.2 [M] 手測匯入\n", Some(MisplacedMarker::AfterNumber)),
        ("- [ ] 1.10 [M] 手測\n", Some(MisplacedMarker::AfterNumber)),
        ("- [ ]  [M] 手測\n", Some(MisplacedMarker::PrefixSlotMissed)),
        ("- [ ] 說明 `[M]` 剝除規則\n", None),
    ];
    for (line, want) in rows {
        let found = misplaced_markers(&parse(line));
        assert_eq!(found.first().map(|m| m.kind), want, "誤置判定：{line}");
    }
}

#[test]
fn misplaced_markers_report_task_id_and_description() {
    let md = "- [x] 1.1 寫解析器\n- [ ] 6.2 [M] 手動驗收\n";
    let found = misplaced_markers(&parse(md));
    assert_eq!(found.len(), 1, "只有第二行誤置");
    assert_eq!(found[0].task_id, 2, "序號須為全檔 checkbox 順序");
    assert_eq!(found[0].description, "6.2 [M] 手動驗收", "描述原樣回報供訊息引文");
    assert!(!found[0].done, "未勾任務回報 done=false");
    assert_eq!(found[0].stable_id, None, "無 ID 註解回報 None");
}

#[test]
fn misplaced_markers_carry_checkbox_state_and_stable_id() {
    // 修復例要能忠實重建原行:勾選狀態與尾部 ID 註解都得跟著回報,
    // 否則照訊息逐字改行會退勾、斷任務身分。
    let md = "- [x] 5.2 [M] 手動驗收 <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n";
    let found = misplaced_markers(&parse(md));
    assert_eq!(found.len(), 1);
    assert!(found[0].done, "已勾任務回報 done=true");
    assert_eq!(
        found[0].stable_id.as_deref(),
        Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        "ID 註解須原樣攜帶"
    );
    assert_eq!(found[0].description, "5.2 [M] 手動驗收", "描述仍為去尾顯示文字");
}

#[test]
fn misplaced_markers_ignore_mid_description_mentions() {
    // 反引號包裹或行文中段提及 [M] 不構成違規——本 repo 既有 tasks.md 大量存在。
    let md = "- [x] 1.1 前綴剝除迴圈同時接受 `[P]` 與 `[M]` 的說明文字\n\
                  - [ ] 2.1 改寫 [M] 起草指引\n\
                  - [ ] [M] 手測匯入\n";
    assert!(misplaced_markers(&parse(md)).is_empty(), "中段提及與正確前綴皆不得命中");
}

#[test]
fn misplaced_markers_check_done_tasks_alike() {
    // 誤置是格式錯誤,與完成狀態無關。
    let found = misplaced_markers(&parse("- [x] 3.3 [M] 已勾的手測\n"));
    assert_eq!(found.len(), 1, "已勾任務同等檢查");
    assert_eq!(found[0].kind, MisplacedMarker::AfterNumber);
}

#[test]
fn misplaced_markers_stay_silent_on_clean_task_lists() {
    // 無命中時零輸出——validate 既有輸出逐位元不變的前提。
    for md in ["", "- [ ] 1.1 寫解析器\n- [x] [M] 手測\n- [ ] [P] [M] 混用\n"] {
        assert!(misplaced_markers(&parse(md)).is_empty(), "乾淨清單須零回報：{md:?}");
    }
}

// --- spec「寫碼任務全完成預測子」---

#[test]
fn counts_split_code_tasks_from_manual_ones() {
    // spec Example 表逐列：全量(完成/總數) → 寫碼(完成/總數) → 預測子
    let rows: [(&str, (usize, usize), (usize, usize), bool); 4] = [
        // 9/10 全量、9/9 寫碼 → 成立
        (
            "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] e\n\
                 - [x] f\n- [x] g\n- [x] h\n- [x] i\n- [ ] [M] 手測\n",
            (9, 10),
            (9, 9),
            true,
        ),
        // 7/10 全量、7/8 寫碼 → 不成立
        (
            "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] e\n\
                 - [x] f\n- [x] g\n- [ ] h\n- [ ] [M] m1\n- [ ] [M] m2\n",
            (7, 10),
            (7, 8),
            false,
        ),
        // 0/0 → 空真成立
        ("", (0, 0), (0, 0), true),
        // 全為 [M]：0/2 全量、0/0 寫碼 → 空真成立
        ("- [ ] [M] m1\n- [ ] [M] m2\n", (0, 2), (0, 0), true),
    ];
    for (md, (complete, total), (code_complete, code_total), predicate) in rows {
        let c = counts(&parse(md));
        assert_eq!((c.complete, c.total), (complete, total), "全量計數：{md}");
        assert_eq!(
            (c.code_complete, c.code_total),
            (code_complete, code_total),
            "寫碼計數：{md}"
        );
        assert_eq!(c.code_done(), predicate, "預測子：{md}");
    }
}

#[test]
fn progress_tuple_stays_on_full_counts() {
    let tasks = parse("- [x] a\n- [ ] [M] 手測\n");
    assert_eq!(progress(&tasks), (2, 1, 1), "既有 progress 契約仍為全量計數");
}

#[test]
fn duplicate_stable_ids_are_detected_and_listed() {
    let md = "- [ ] 1.1 a <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n\
                  - [ ] 1.2 b <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n\
                  - [ ] 1.3 c <!-- speclink-task:tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ -->\n";
    let tasks = parse(md);
    assert_eq!(
        duplicate_stable_ids(&tasks),
        vec!["tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()]
    );
    assert!(duplicate_stable_ids(&parse(TASKS_WITH_IDS)).is_empty());
}

#[test]
fn new_stable_id_has_tsk_prefix_and_26_ulid_chars() {
    let id = new_stable_id();
    let ulid = id.strip_prefix("tsk_").expect("id must carry tsk_ prefix");
    assert_eq!(ulid.len(), 26, "ULID must be 26 chars: {id}");
    assert!(ulid.chars().all(|c| c.is_ascii_alphanumeric()), "ULID must be alphanumeric: {id}");
}

#[test]
fn new_stable_id_is_time_ordered() {
    let a = new_stable_id();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let b = new_stable_id();
    assert!(a < b, "later id must sort after earlier: {a} vs {b}");
}

// --- 蓋章時機：產出全檔蓋章、task done 單行補章（spec task-identity）---

#[test]
fn stamp_all_assigns_unique_ids_and_preserves_everything_else() {
    let md = "## 1. Group\n\n- [ ] 1.1 first task\n- [x] 1.2 second task <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n* [ ] 1.3 star task\n\nProse line.\n";
    let stamped = stamp_all(md);
    let tasks = parse(&stamped);
    assert_eq!(tasks.len(), 3);
    assert!(tasks.iter().all(|t| t.stable_id.is_some()), "every task gains an id: {stamped}");
    assert_eq!(
        tasks[1].stable_id.as_deref(),
        Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
        "already-stamped line keeps its id"
    );
    let mut ids: Vec<String> = tasks.iter().filter_map(|t| t.stable_id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3, "ids must be unique");
    assert_eq!(tasks[0].description, "1.1 first task");
    assert_eq!(tasks[2].description, "1.3 star task");
    let orig: Vec<&str> = md.lines().collect();
    let new: Vec<&str> = stamped.lines().collect();
    assert_eq!(orig.len(), new.len());
    for (o, n) in orig.iter().zip(&new) {
        let t = o.trim_start();
        if t.starts_with("- [ ]") || t.starts_with("* [ ]") {
            assert!(n.starts_with(o), "stamp appends at the line end: {n}");
            assert!(n.ends_with("-->"), "stamp is a trailing comment: {n}");
        } else {
            assert_eq!(o, n, "non-target lines byte-identical");
        }
    }
    assert!(stamped.ends_with('\n'), "trailing newline preserved");
}

#[test]
fn reorder_keeps_stable_ids_and_swaps_ordinals() {
    let stamped = stamp_all("- [ ] 1.1 alpha\n- [ ] 1.2 beta\n");
    let before = parse(&stamped);
    let mut lines: Vec<&str> = stamped.lines().collect();
    lines.swap(0, 1);
    let swapped = format!("{}\n", lines.join("\n"));
    let after = parse(&swapped);
    assert_eq!(after[0].description, "1.2 beta");
    assert_eq!(after[0].stable_id, before[1].stable_id, "id follows the task, not the slot");
    assert_eq!(after[1].stable_id, before[0].stable_id);
    assert_eq!((after[0].id, after[1].id), (1, 2), "ordinals follow position");
}

#[test]
fn mark_done_stamps_only_the_unstamped_target_line() {
    let md = "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [ ] 1.3 third\n- [ ] 1.4 fourth\n";
    let (new_content, desc, already, _) = mark_done(md, 3).unwrap();
    assert!(!already);
    assert_eq!(desc, "1.3 third");
    let orig: Vec<&str> = md.lines().collect();
    let new: Vec<&str> = new_content.lines().collect();
    assert_eq!(orig.len(), new.len());
    for (i, (o, n)) in orig.iter().zip(&new).enumerate() {
        if i == 4 {
            assert!(
                n.starts_with("- [x] 1.3 third <!-- speclink-task:tsk_"),
                "target line must be checked and stamped: {n}"
            );
            assert!(n.ends_with("-->"));
        } else {
            assert_eq!(o, n, "line {i} must stay byte-identical");
        }
    }
    let tasks = parse(&new_content);
    assert!(tasks[2].stable_id.is_some(), "freshly stamped id is addressable on re-read");
}

#[test]
fn mark_done_on_stamped_line_keeps_comment_verbatim() {
    let md = "- [ ] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n";
    let (new_content, desc, _, _) = mark_done(md, 1).unwrap();
    assert_eq!(
        new_content,
        "- [x] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n"
    );
    assert_eq!(desc, "1.1 first", "description strips the marker");
}

#[test]
fn mark_undone_never_stamps() {
    let md = "- [x] 1.1 first\n";
    let (new_content, desc, _, _) = mark_undone(md, 1).unwrap();
    assert_eq!(new_content, "- [ ] 1.1 first\n", "undone flips without stamping");
    assert_eq!(desc, "1.1 first");
}

// --- evidence 記錄 v2（spec verify-evidence: task done 寫入逐任務 evidence）---

impl TempWs {
    /// Workspace root as a git repo with a fixed identity, one commit
    /// (HEAD exists), and one unclaimed dirty file.
    fn with_commit_and_dirty(tag: &str, dirty_rel: &str) -> TempWs {
        let t = TempWs::new(tag);
        let git = |args: &[&str]| {
            let ok = std::process::Command::new("git")
                .arg("-C")
                .arg(&t.ws.root)
                .args(args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.name", "Evidence Tester"]);
        git(&["config", "user.email", "ev@example.com"]);
        std::fs::write(t.ws.root.join("seed.txt"), "seed\n").unwrap();
        git(&["add", "seed.txt"]);
        git(&["commit", "-q", "-m", "seed"]);
        let p = t.ws.root.join(dirty_rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "dirty\n").unwrap();
        t
    }
}

#[test]
fn git_changed_files_decodes_quoted_non_ascii_paths() {
    // `git status --porcelain` C-quotes any non-ASCII path unless
    // core.quotepath=false. The change-diff resolver already passes that
    // flag, so an escaped path here would never compare equal to the
    // resolver's — silently opening the dirty-at-start fail-closed guard.
    let t = TempWs::with_commit_and_dirty("quotepath", "src/app.rs");
    std::fs::write(t.ws.root.join("規格.md"), "內容\n").unwrap();
    let files = git_changed_files(&t.ws);
    assert!(
        files.contains(&"規格.md".to_string()),
        "non-ASCII path must come back decoded: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f.contains('\\')),
        "no octal escapes may survive: {files:?}"
    );
}

#[test]
fn the_spec_dir_exclusion_follows_the_workspace_name() {
    // 排除的是「這個 workspace 的 spec 目錄」，不是字面上的 openspec/——
    // 自訂目錄名的專案裡，`.evidence.json` 落在 `<custom>/changes/<name>/` 之下，
    // 寫死字串會讓第二次 task done 把證據檔自己記成碰過的程式檔。
    let mut t = TempWs::with_dirty_file("customdir", "src/app.rs");
    t.ws.spec_dir_name = "customspec".to_string();
    let ev = t.ws.root.join("customspec/changes/demo/.evidence.json");
    std::fs::create_dir_all(ev.parent().unwrap()).unwrap();
    std::fs::write(&ev, "{}\n").unwrap();

    let files = git_changed_files(&t.ws);
    assert_eq!(
        files,
        vec!["src/app.rs".to_string()],
        "spec artifacts under the workspace's own dir name stay out: {files:?}"
    );
}

#[test]
fn complete_writes_v2_evidence_entry_with_attribution_and_no_basis() {
    let t = TempWs::with_commit_and_dirty("evidence", "src/app.rs");
    let store = store_with(
        META_UNSTARTED,
        "- [ ] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n",
    );
    store.put_artifact("demo", "specs/auth/spec.md", "## ADDED Requirements\n");
    let attr = CompleteAttribution {
        identity: Some("Tester <t@example.com>"),
        agent: None,
        repo: Some("main"),
    };
    complete(&store, "demo", &TaskAddr::Ordinal(1), &attr, TouchedCandidates::ProbeWorkspace(&t.ws)).unwrap();

    let json = store.read_evidence("demo").expect("evidence record written");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["version"], 2, "writes are always v2: {json}");
    assert_eq!(v["change"], "demo");
    let e = &v["entries"][0];
    assert_eq!(e["taskId"], "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV");
    assert_eq!(e["actor"], "Tester <t@example.com>");
    assert_eq!(e["repo"], "main");
    let head = e["headCommit"].as_str().expect("headCommit present");
    assert_eq!(head.len(), 40, "full HEAD sha expected: {head}");
    let files: Vec<&str> =
        e["touchedFiles"].as_array().unwrap().iter().filter_map(|f| f.as_str()).collect();
    assert!(files.contains(&"src/app.rs"), "dirty file recorded: {files:?}");
    assert!(e.get("basisDigests").is_none(), "the entry records no verification basis: {json}");
    let at = e["recordedAt"].as_str().expect("recordedAt present");
    assert!(at.ends_with('Z'), "recordedAt is UTC: {at}");
    chrono::DateTime::parse_from_rfc3339(at).expect("recordedAt parses as RFC3339");
}

#[test]
fn a_record_carrying_basis_digests_still_reads_and_new_writes_omit_them() {
    // spec Scenario「舊格式記錄可讀」：前一版寫入的 basisDigests 是未知欄位——
    // 讀取端忽略它、all_files 不變；本次寫入的 entry 不再帶該欄位。
    let t = TempWs::with_commit_and_dirty("basis-compat", "src/app.rs");
    let store = store_with(META_UNSTARTED, "- [ ] 1.1 new\n");
    store.put_evidence(
        "demo",
        r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_OLD","taskDesc":"1.1 old","touchedFiles":["src/old.rs"],"basisDigests":{"spec":"sha256:0","tasks":"sha256:0","policy":"sha256:0"},"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
    );

    let rec = TouchedRecord::load(&store, "demo");
    assert_eq!(rec.entries.len(), 1, "the legacy-shaped entry survives the read");
    assert_eq!(rec.entries[0].task_id, "tsk_OLD");
    assert_eq!(rec.all_files(), vec!["src/old.rs".to_string()], "all_files unchanged");

    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::ProbeWorkspace(&t.ws),
    )
    .unwrap();
    let json = store.read_evidence("demo").expect("record rewritten");
    assert!(
        !json.contains("basisDigests"),
        "writes no longer record a verification basis: {json}"
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["version"], 2, "writes are always v2, whatever version was read");
    assert_eq!(v["entries"].as_array().unwrap().len(), 2, "the old entry is carried forward");
}

// --- Host 注入候選（spec verify-evidence: store 模式的 evidence 記錄與查詢）---

#[test]
fn injected_candidates_replace_the_workspace_probe() {
    // 注入即以之為準：server 模式沒有本機 workspace，wire 上送到的清單就是
    // 候選來源。此時髒檔探測必須完全不發生——否則跑在有 checkout 的機器上的
    // server 會把自己的髒檔記成使用者的實作證據。
    let t = TempWs::with_commit_and_dirty("injected", "src/local-only.rs");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);

    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected {
            files: &["remote/wire.rs".to_string()],
            head_commit: None,
        },
    )
    .unwrap();

    let rec = TouchedRecord::load(&store, "demo");
    assert_eq!(
        rec.all_files(),
        vec!["remote/wire.rs".to_string()],
        "only the injected candidates may be recorded: {:?}",
        rec.all_files()
    );
    drop(t);
}

#[test]
fn injected_head_commit_lands_in_the_entry_and_absent_stays_absent() {
    // spec verify-evidence「遠端 task done 攜檔案後 evidence 可查」的 THEN 把
    // headCommit 列為 entry 欄位（design contract：由 wire 攜入或 host 解析）
    // ——wire 報了就入帳；沒報就缺席，絕不從無關 checkout 補一個。
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected {
            files: &["src/a.rs".to_string()],
            head_commit: Some("0123456789012345678901234567890123456789"),
        },
    )
    .unwrap();
    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(2),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected { files: &["src/b.rs".to_string()], head_commit: None },
    )
    .unwrap();

    let rec = TouchedRecord::load(&store, "demo");
    assert_eq!(
        rec.entries[0].head_commit.as_deref(),
        Some("0123456789012345678901234567890123456789"),
        "the wire-reported commit travels into the entry"
    );
    assert_eq!(rec.entries[1].head_commit, None, "unreported stays absent, never guessed");

    // 非完整 sha 形狀的 wire 值是垃圾輸入：與路徑過濾同一取向，缺席而非逐字入帳。
    let store2 = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    complete(
        &store2,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected {
            files: &["src/a.rs".to_string()],
            head_commit: Some("not-a-sha; rm -rf /"),
        },
    )
    .unwrap();
    let rec2 = TouchedRecord::load(&store2, "demo");
    assert_eq!(rec2.entries[0].head_commit, None, "junk wire input stays absent");
}

#[test]
fn injected_paths_normalize_separators_and_collapse_duplicates() {
    // 注入清單是另一台機器的 wire 輸入：Windows 反斜線正規化為邏輯正斜線
    // 正典、重複項收斂，逐字入帳的原始形狀不落進 evidence。
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected {
            files: &[
                "src\\win.rs".to_string(),
                "src/win.rs".to_string(),
                "src/a.rs".to_string(),
                "src/a.rs".to_string(),
                String::new(),
                "/etc/passwd".to_string(),
                "../outside.rs".to_string(),
                "src/../../up.rs".to_string(),
                "C:\\evil.rs".to_string(),
                ".git/hooks/pre-commit".to_string(),
                ".speclink/touched/x.json".to_string(),
            ],
            head_commit: None,
        },
    )
    .unwrap();

    let rec = TouchedRecord::load(&store, "demo");
    assert_eq!(
        rec.entries[0].touched_files,
        vec!["src/win.rs".to_string(), "src/a.rs".to_string()],
        "separators normalized; duplicates, empties, absolute/escaping and scaffolding paths dropped"
    );
}

#[test]
fn injected_candidates_are_filtered_by_earlier_claims() {
    // 歸屬過濾是單點實作：注入與探測兩來源同一套——已被先前任務認領的檔案
    // 不會第二次入帳。
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected { files: &["src/a.rs".to_string()], head_commit: None },
    )
    .unwrap();

    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(2),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected {
            files: &["src/a.rs".to_string(), "src/b.rs".to_string()],
            head_commit: None,
        },
    )
    .unwrap();

    let rec = TouchedRecord::load(&store, "demo");
    assert_eq!(rec.entries.len(), 2);
    assert_eq!(
        rec.entries[1].touched_files,
        vec!["src/b.rs".to_string()],
        "an already-claimed file must not be recorded twice: {:?}",
        rec.entries[1].touched_files
    );
}

#[test]
fn an_empty_injected_list_appends_no_entry_and_never_falls_back_to_probing() {
    // 「未攜帶 touchedFiles」是正常狀態，不是錯誤，也不是「改去探本機」：
    // 沿無新髒檔語意不新增任何記錄。
    let t = TempWs::with_commit_and_dirty("injected-empty", "src/local-only.rs");
    let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);

    complete(
        &store,
        "demo",
        &TaskAddr::Ordinal(1),
        &CompleteAttribution::default(),
        TouchedCandidates::Injected { files: &[], head_commit: None },
    )
    .unwrap();

    assert_eq!(store.read_evidence("demo"), None, "no candidates means no record at all");
    drop(t);
}

#[test]
fn uncomplete_never_writes_or_alters_evidence() {
    let seeded = "{\"change\":\"demo\",\"touched\":[{\"task_id\":\"1\",\"task_desc\":\"1.1 a\",\"files\":[\"src/a.rs\"]}]}";
    let store = store_with(META_UNSTARTED, "- [x] 1.1 a\n");
    store.put_evidence("demo", seeded);
    uncomplete(&store, "demo", &TaskAddr::Ordinal(1)).unwrap();
    assert_eq!(
        store.read_evidence("demo").as_deref(),
        Some(seeded),
        "undone must not write or alter any evidence record"
    );
}

// --- 任務搬移＋重編號（自 desktop core 遷入；spec「任務搬移端點與重編號效果」）---

fn tasks_text(store: &TestStore) -> String {
    store.read_artifact("demo", "tasks.md").unwrap()
}

fn move_lines(store: &TestStore) -> Vec<String> {
    tasks_text(store)
        .lines()
        .filter(|l| l.trim_start().starts_with("- ["))
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn move_within_group_renumbers_prefixes_per_spec_example() {
    // spec Example「組內移動重編號」：把 1.1 甲拖到末位 → 乙丙甲，前綴重寫 1.1/1.2/1.3。
    let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n");
    let out = move_task(&store, "demo", 1, 3, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲"],
        "prefixes must follow the new order"
    );
    assert_eq!(out.description, "1.3 甲", "outcome carries the post-move description");
    assert!(tasks_text(&store).contains("## 1. 群組"), "group heading untouched");
}

#[test]
fn move_up_infers_insert_before_the_anchor() {
    let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n");
    let out = move_task(&store, "demo", 3, 1, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.1 丙", "- [ ] 1.2 甲", "- [x] 1.3 乙"],
        "an upward move must insert before the anchor"
    );
    assert_eq!(out.description, "1.1 丙");
}

#[test]
fn move_across_groups_takes_the_new_groups_numbering() {
    // spec scenario「跨群組搬移重編號」：把第 1 個任務移到第 3 個任務之後。
    let store = store_with(
        META_UNSTARTED,
        "## 1. 前段\n\n說明文字原樣保留。\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
    );
    let out = move_task(&store, "demo", 1, 3, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.1 乙", "- [ ] 2.1 丙", "- [ ] 2.2 甲", "- [ ] 2.3 丁"]
    );
    assert_eq!(out.description, "2.2 甲", "moved task takes the new group's numbering");
    let text = tasks_text(&store);
    assert!(text.contains("說明文字原樣保留。"), "prose lines byte-identical");
    assert!(text.contains("## 1. 前段") && text.contains("## 2. 後段"));
}

#[test]
fn downward_move_to_group_end_stays_in_the_origin_group() {
    // 向下拖到群組末位必須落在錨 checkbox 之後（留在原群組），不得吞進下一群組。
    let store = store_with(
        META_UNSTARTED,
        "## 1. 前段\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n\n## 2. 後段\n\n- [ ] 2.1 丁\n- [ ] 2.2 戊\n",
    );
    move_task(&store, "demo", 1, 3, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲", "- [ ] 2.1 丁", "- [ ] 2.2 戊"],
        "a downward move onto the last task of a group must not leak into the next group"
    );
    let text = tasks_text(&store);
    let g2 = text.split("## 2. 後段").nth(1).unwrap();
    assert!(!g2.contains("甲"), "甲 must stay under group 1: {text}");
}

#[test]
fn before_true_crosses_the_heading_and_becomes_group_head() {
    // 明確 before=true：插於錨任務行之前，跨過群組標題成為錨所屬群組的組首。
    let store = store_with(
        META_UNSTARTED,
        "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
    );
    move_task(&store, "demo", 2, 3, Some(true)).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.1 甲", "- [ ] 2.1 乙", "- [ ] 2.2 丙", "- [ ] 2.3 丁"],
        "before=true must insert ahead of the anchor line, across the heading"
    );
    let text = tasks_text(&store);
    let g2 = text.split("## 2. 後段").nth(1).unwrap();
    assert!(g2.contains("乙"), "乙 must live under group 2: {text}");
}

#[test]
fn before_false_explicitly_inserts_after_the_anchor() {
    // 明確側別覆蓋方向推斷：向上移動＋before=false → 落在錨任務之後。
    let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n- [ ] 1.3 丙\n");
    move_task(&store, "demo", 3, 1, Some(false)).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.1 甲", "- [ ] 1.2 丙", "- [ ] 1.3 乙"],
        "before=false anchors after task 1 even on an upward move"
    );
}

#[test]
fn renumber_leaves_unprefixed_and_sub_versioned_text_verbatim() {
    // 重編號只改「數字.數字＋空白」前綴：無前綴與子版號（1.2.3）逐字元保留。
    let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 補充說明不帶編號\n- [ ] 1.2 乙\n");
    move_task(&store, "demo", 3, 1, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.1 乙", "- [ ] 1.2 甲", "- [ ] 補充說明不帶編號"],
        "unprefixed task text must stay untouched while others renumber"
    );
    let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2.3 子版號文字\n");
    move_task(&store, "demo", 2, 1, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.2.3 子版號文字", "- [ ] 1.2 甲"],
        "sub-versioned prefixes must be preserved verbatim"
    );
}

#[test]
fn groups_without_numeric_heading_are_not_renumbered() {
    let store = store_with(META_UNSTARTED, "## 準備\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
    move_task(&store, "demo", 1, 2, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
        "a heading without a numeric prefix must leave its tasks' numbers alone"
    );
    // sharp-edge：標題數字超出 u64——解析失敗即視為無數字標題，任務保留原文。
    let store = store_with(META_UNSTARTED, "## 99999999999999999999999. 巨數\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
    move_task(&store, "demo", 1, 2, None).unwrap();
    assert_eq!(
        move_lines(&store),
        vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
        "unparseable group numbers must not rewrite anything"
    );
}

#[test]
fn out_of_range_move_errors_without_writes() {
    // spec scenario「越界拒絕零副作用」：只有 3 個任務時 from=5 拒絕，內容不變。
    const TASKS: &str = "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n- [ ] 1.3 丙\n";
    let store = store_with(META_UNSTARTED, TASKS);
    for (from, to) in [(5usize, 1usize), (0, 1), (1, 0), (1, 9)] {
        let err = move_task(&store, "demo", from, to, None).unwrap_err();
        assert!(
            err.to_string().contains("out of range"),
            "({from},{to}) must name the out-of-range refusal: {err}"
        );
    }
    assert_eq!(tasks_text(&store), TASKS, "failed moves must not rewrite the file");
    assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
}

#[test]
fn move_missing_tasks_md_errors() {
    let store = TestStore::with_meta("demo", META_UNSTARTED);
    let err = move_task(&store, "demo", 1, 2, None).unwrap_err();
    assert_eq!(err.to_string(), "tasks.md not found for change 'demo'");
    assert_eq!(*store.artifact_writes.borrow(), 0);
}

#[test]
fn move_preserves_trailing_newline_state() {
    let store = store_with(META_UNSTARTED, "- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
    move_task(&store, "demo", 1, 2, None).unwrap();
    assert!(tasks_text(&store).ends_with("\n"), "trailing newline preserved");
    let store = store_with(META_UNSTARTED, "- [ ] 1.1 甲\n- [ ] 1.2 乙");
    move_task(&store, "demo", 1, 2, None).unwrap();
    assert!(!tasks_text(&store).ends_with("\n"), "absent trailing newline preserved");
}

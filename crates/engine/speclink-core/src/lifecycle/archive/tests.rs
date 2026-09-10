use super::{
    archive, merge_capability, merge_violations, skip_reason, ArchiveOptions, SkipReason,
};
use crate::store::Store;
use crate::tasks::TouchedRecord;
use crate::teststore::TestStore;
use crate::util;
use crate::workspace::Workspace;

#[test]
fn archive_preserves_started_fields_and_stamps_the_archived_station() {
    // A change carrying all three started_* fields (plus created_*) must
    // arrive in the archive with every lifecycle station intact —
    // started_* byte-for-byte, archived_at appended by the stamp. The host
    // root deliberately does not exist: the skip-specs path touches no
    // host files (git probes fail soft, no snapshot is written), so the
    // test needs no filesystem at all.
    let ws = Workspace {
        root: std::env::temp_dir().join("speclink-archive-test-ghost-root"),
        spec_dir_name: "openspec".to_string(),
    };
    let meta = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\nstarted_at: 2026-07-03\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";
    let store = TestStore::with_meta("demo", meta);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    let change = crate::model::find_change(&store, "demo").unwrap();

    let outcome = archive(
        &ws,
        &store,
        &change,
        &ArchiveOptions {
            skip_specs: true,
            carry_review: false,
            carry_verify: false,
            no_validate: true,
            mark_tasks_complete: false,
        },
        None,
    )
    .unwrap();

    let today = util::today();
    assert_eq!(outcome.dated_name, format!("{today}-demo"));
    let archived = store.read_archived_meta(&outcome.dated_name).unwrap();
    assert!(
        archived.starts_with(meta),
        "created_* and started_* must survive archive byte-for-byte, got: {archived}"
    );
    assert!(archived.contains(&format!("archived_at: {today}\n")));
    // All three stations coexist on the archived document.
    for field in ["created:", "started_at:", "started_by:", "started_with:", "archived_at:"] {
        assert!(archived.contains(field), "missing station field {field}");
    }
    assert!(!store.change_exists("demo"), "active change moved into the archive");
}

// --- spec change-lifecycle「封存的章失效守門」（design D5）---

mod stale_stamp_gate {
    use super::*;
    use crate::station::content_fingerprint;

    const SCOPE_PATH: &str = "crates/a/src/lib.rs";
    const FILE_A: &str = "fn a() {}\n";
    /// 全任務完成（含 `[M]`）——封存的任務完成度守門要求全勾，手測強制力保留。
    const TASKS_ALL_DONE: &str = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] [M] 手測\n";

    /// 有工作樹的 workspace：scope 檔以 `content` 寫入真實暫存目錄。
    fn ws_with_scope_file(tag: &str, content: &str) -> Workspace {
        let root = std::env::temp_dir().join(format!("speclink-stale-stamp-{tag}"));
        let file = root.join(SCOPE_PATH);
        std::fs::create_dir_all(file.parent().expect("scope parent")).expect("mkdir");
        std::fs::write(&file, content).expect("write scope file");
        Workspace { root, spec_dir_name: "openspec".to_string() }
    }

    /// remote 封存通道：空 root＝無本地工作樹（沿 guard_linked_worktree 的慣例）。
    fn remote_ws() -> Workspace {
        Workspace { root: std::path::PathBuf::new(), spec_dir_name: "openspec".to_string() }
    }

    /// 帶指定站別章的 meta：錨記全任務總數 5、scope 為 FILE_A 的現值指紋。
    fn meta_with_stamps(prefixes: &[&str]) -> String {
        let hash = content_fingerprint(FILE_A);
        let mut meta = "schema: spec-driven\ncreated: 2026-07-01\n".to_string();
        for p in prefixes {
            meta.push_str(&format!(
                "{p}_at: 2026-08-01\n{p}_by: R <r@example.com>\n{p}_with: claude\n\
                     {p}_tasks_total: 5\n{p}_scope:\n  - path: {SCOPE_PATH}\n    hash: {hash}\n"
            ));
        }
        meta
    }

    fn try_archive(
        ws: &Workspace,
        store: &TestStore,
        opts: ArchiveOptions,
    ) -> anyhow::Result<()> {
        let change = crate::model::find_change(store, "demo").expect("change");
        archive(ws, store, &change, &opts, None).map(|_| ())
    }

    #[test]
    fn stale_content_anchor_refuses_and_names_the_station() {
        // Example 表第一列：review 章齊備、scope 檔內容改變 → 拒絕、點名 review。
        let ws = ws_with_scope_file("content", &format!("{FILE_A}fn extra() {{}}\n"));
        let store = TestStore::with_meta("demo", &meta_with_stamps(&["reviewed"]));
        store.put_artifact("demo", "tasks.md", TASKS_ALL_DONE);
        let err = try_archive(&ws, &store, skip_opts()).expect_err("stale stamp must refuse");
        let msg = err.to_string();
        assert!(msg.contains("review"), "must name the station: {msg}");
        assert!(msg.contains(SCOPE_PATH), "must name the changed file: {msg}");
        assert!(store.change_exists("demo"), "refusal must not move the change");
    }

    #[test]
    fn checking_a_manual_task_after_the_stamp_still_archives() {
        // Example 表第二列：兩章齊備、補勾 [M]、scope 檔零改動 → 放行。
        let ws = ws_with_scope_file("manual", FILE_A);
        let store =
            TestStore::with_meta("demo", &meta_with_stamps(&["reviewed", "verified"]));
        store.put_artifact("demo", "tasks.md", TASKS_ALL_DONE);
        try_archive(&ws, &store, skip_opts()).expect("manual toggle must not stale the stamp");
        assert!(!store.change_exists("demo"), "change must be archived");
    }

    #[test]
    fn a_new_task_after_the_stamp_breaks_the_task_anchor() {
        // Example 表第三列：兩章齊備、任務總數自 5 變 6 → 拒絕（任務錨破）。
        let ws = ws_with_scope_file("recount", FILE_A);
        let store =
            TestStore::with_meta("demo", &meta_with_stamps(&["reviewed", "verified"]));
        store.put_artifact("demo", "tasks.md", &format!("{TASKS_ALL_DONE}- [x] f\n"));
        let err = try_archive(&ws, &store, skip_opts()).expect_err("task anchor must refuse");
        let msg = err.to_string();
        assert!(msg.contains("review") && msg.contains("verify"), "both stamps: {msg}");
        assert!(store.change_exists("demo"), "refusal must not move the change");
    }

    #[test]
    fn no_stamp_and_unknown_stamp_both_pass_through() {
        // Example 表第四、五列：無章與章欄位不全 → 放行，行為與守門引入前一致。
        for (tag, meta) in [
            ("nostamp", "schema: spec-driven\ncreated: 2026-07-01\n".to_string()),
            // 章欄位不全（缺 scope）→ Unknown，視同無章。
            (
                "partial",
                "schema: spec-driven\ncreated: 2026-07-01\nreviewed_at: 2026-08-01\n\
                     reviewed_tasks_total: 5\n"
                    .to_string(),
            ),
        ] {
            let ws = ws_with_scope_file(tag, &format!("{FILE_A}changed\n"));
            let store = TestStore::with_meta("demo", &meta);
            store.put_artifact("demo", "tasks.md", TASKS_ALL_DONE);
            try_archive(&ws, &store, skip_opts()).unwrap_or_else(|e| panic!("{tag}: {e}"));
            assert!(!store.change_exists("demo"), "{tag}: change must be archived");
        }
    }

    #[test]
    fn a_stale_stamp_refuses_before_the_merge_gate() {
        // 章失效＋delta 過期 → 章失效先拒；merge 拒絕字串不出現。
        let ws = ws_with_scope_file("stale-before-merge", FILE_A);
        let store = TestStore::with_meta("demo", &meta_with_stamps(&["reviewed"]));
        store.put_artifact("demo", "tasks.md", &format!("{TASKS_ALL_DONE}- [x] e\n"));
        store.put_artifact("demo", "specs/auth/spec.md", ADDED_R1);
        store.canonical.borrow_mut().insert("auth".to_string(), CANON_R1.to_string());
        let err = try_archive(&ws, &store, apply_opts()).expect_err("stale stamp must refuse");
        let msg = err.to_string();
        assert!(msg.contains("tasks moved after the stamp"), "stale wording: {msg}");
        assert!(!msg.contains("cannot be archived"), "merge refusal never reached: {msg}");
    }

    #[test]
    fn the_task_readiness_gate_refuses_first() {
        // Example 表之外的順序契約：寫碼任務未完成＋章已失效 → 任務守門先拒，
        // 訊息維持既有樣式、不提章失效。
        let ws = ws_with_scope_file("order", &format!("{FILE_A}changed\n"));
        let store = TestStore::with_meta("demo", &meta_with_stamps(&["reviewed"]));
        store.put_artifact("demo", "tasks.md", "- [x] a\n- [ ] b\n");
        let err = try_archive(&ws, &store, skip_opts()).expect_err("task gate must refuse");
        let msg = err.to_string();
        assert!(msg.contains("1/2 tasks complete"), "既有任務守門訊息: {msg}");
        assert!(!msg.contains("stamp"), "任務守門訊息不得提及章失效: {msg}");
    }

    #[test]
    fn the_remote_channel_judges_only_the_task_anchor() {
        // spec Scenario「remote 通道僅判任務錨」：無工作樹 → 內容錨跳過（放行），
        // 任務錨破仍拒絕。scope 檔在 remote 側根本讀不到，等同「改過」。
        let store = TestStore::with_meta("demo", &meta_with_stamps(&["reviewed"]));
        store.put_artifact("demo", "tasks.md", TASKS_ALL_DONE);
        try_archive(&remote_ws(), &store, skip_opts())
            .expect("no work tree → content anchor is not judged");

        let store = TestStore::with_meta("demo", &meta_with_stamps(&["reviewed"]));
        store.put_artifact("demo", "tasks.md", &format!("{TASKS_ALL_DONE}- [x] f\n"));
        let err = try_archive(&remote_ws(), &store, skip_opts())
            .expect_err("task anchor still refuses on the remote channel");
        assert!(err.to_string().contains("review"), "{err}");
    }
}

// --- 封存共行逐 slug（design D3；spec「多來源討論的變更封存逐一共行」）---

fn ghost_ws() -> Workspace {
    Workspace {
        root: std::env::temp_dir().join("speclink-archive-co-travel-ghost-root"),
        spec_dir_name: "openspec".to_string(),
    }
}

fn skip_opts() -> ArchiveOptions {
    ArchiveOptions {
        skip_specs: true,
        no_validate: true,
        mark_tasks_complete: false,
        carry_review: false,
        carry_verify: false,
    }
}

fn discussion_doc(slug: &str) -> String {
    format!(
        "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: x\n"
    )
}

#[test]
fn archive_co_travels_every_unreferenced_source_discussion() {
    // 兩份來源討論皆無其他在途變更引用 → 兩份皆隨行封存。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1, d2\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
    store.discussions.borrow_mut().insert("d2".into(), discussion_doc("d2"));
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    let slugs: Vec<&str> =
        outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(slugs, vec!["d1", "d2"], "both unreferenced discussions co-archive");
    assert!(store.archived_discussion_exists("d1"));
    assert!(store.archived_discussion_exists("d2"));
}

#[test]
fn archive_leaves_discussion_still_referenced_by_another_change() {
    // d2 仍被另一在途變更 cut2 引用 → 僅 d1 隨行，d2 留在途。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1, d2\n",
    );
    store.metas.borrow_mut().insert(
        "cut2".into(),
        "schema: spec-driven\ncreated: 2026-07-02\nfrom_discussion: d2\n".into(),
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
    store.discussions.borrow_mut().insert("d2".into(), discussion_doc("d2"));
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    let slugs: Vec<&str> =
        outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(slugs, vec!["d1"], "only the unreferenced discussion co-archives");
    assert!(store.archived_discussion_exists("d1"));
    assert!(!store.archived_discussion_exists("d2"), "d2 stays live — still referenced");
    assert!(store.live_discussion_exists("d2"));
}

#[test]
fn archive_leaves_unconcluded_discussion_live() {
    // 未結論（Conclusion 仍為佔位註解）的來源討論不隨變更封存，維持在途。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: pending\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert(
        "pending".into(),
        "---\ntopic: pending\nslug: pending\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n".into(),
    );
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    assert!(
        outcome.archived_discussions.is_empty(),
        "unconcluded discussion must not co-archive"
    );
    assert!(store.live_discussion_exists("pending"), "record stays live");
    assert!(!store.archived_discussion_exists("pending"));
}

#[test]
fn archive_leaves_held_discussion_live() {
    // 已結論但帶 hold（還欠下一刀）的來源討論不隨變更封存，維持在途。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: staged\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert(
        "staged".into(),
        "---\ntopic: staged\nslug: staged\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\nhold: true\n---\n\n## Conclusion\n\n**Decision**: cut-b later\n".into(),
    );
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    assert!(
        outcome.archived_discussions.is_empty(),
        "held discussion must not co-archive"
    );
    assert!(store.live_discussion_exists("staged"), "record stays live");
    assert!(!store.archived_discussion_exists("staged"));
}

#[test]
fn archive_judges_hold_per_source_discussion() {
    // 多來源逐一共行：同一 from_discussion 清單中，無 hold 者隨行封存、帶 hold 者留在途。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1, staged\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
    store.discussions.borrow_mut().insert(
        "staged".into(),
        "---\ntopic: staged\nslug: staged\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\nhold: true\n---\n\n## Conclusion\n\n**Decision**: cut-b later\n".into(),
    );
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    let slugs: Vec<&str> =
        outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(slugs, vec!["d1"], "only the unheld discussion co-archives");
    assert!(store.archived_discussion_exists("d1"));
    assert!(store.live_discussion_exists("staged"), "held sibling stays live");
    assert!(!store.archived_discussion_exists("staged"));
}

#[test]
fn archive_single_source_discussion_co_travels_as_before() {
    // 單一來源情境：與變更前一致——恰一份討論隨行封存。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: only\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.discussions.borrow_mut().insert("only".into(), discussion_doc("only"));
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    let slugs: Vec<&str> =
        outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(slugs, vec!["only"]);
    assert!(store.archived_discussion_exists("only"));
}

#[test]
fn archive_leaves_discussion_live_when_an_in_flight_meta_is_corrupt() {
    // spec discussion-docs Example「壞 meta 的在途變更擋住隨行封存」：d1 已結論、無 hold；
    // cut 的 meta 含 from_discussion: d1、tasks 全完成；另一在途變更 broken 的
    // .openspec.yaml 為 `schema: [unclosed`（解析失敗）→ cut 照常封存、d1 留在途。
    // 壞 meta 視同仍引用（與 conclude 閉環步同一條規則）。
    let store = TestStore::with_meta(
        "cut",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1\n",
    );
    store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
    store.metas.borrow_mut().insert("broken".into(), "schema: [unclosed\n".into());
    store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
    let change = crate::model::find_change(&store, "cut").unwrap();

    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

    assert!(!store.change_exists("cut"), "cut 移入封存區");
    assert!(
        outcome.archived_discussions.is_empty(),
        "corrupt in-flight meta counts as a live reference — no co-archive"
    );
    assert!(store.live_discussion_exists("d1"), "d1 stays live");
    assert!(!store.archived_discussion_exists("d1"));
}

// --- 單筆封存任務完成度守門（design D1；spec change-lifecycle「單筆封存的任務完成度守門」）---

fn gate_store(tasks_md: &str) -> TestStore {
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", tasks_md);
    store
}

#[test]
fn incomplete_tasks_refuse_archive_with_evidence_and_zero_writes() {
    // spec Example 守門判定：3 任務僅 1 勾、未帶 --mark-tasks-complete → 拒絕，
    // 訊息載明 1/3 與兩條出路，store 零寫入。
    let store = gate_store("- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect_err("incomplete tasks must refuse archive");
    assert!(err.to_string().contains("1/3"), "evidence N/M in message: {err}");
    assert!(err.to_string().contains("--mark-tasks-complete"), "exit route named: {err}");
    assert!(
        err.downcast_ref::<crate::command::Refusal>().is_some(),
        "typed Refusal so the runtime classifies refused"
    );
    assert!(store.change_exists("demo"), "change stays in place");
    assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
    assert!(store.canonical.borrow().is_empty(), "no canonical spec writes");
    assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

#[test]
fn all_tasks_complete_passes_the_gate() {
    // spec Example 守門判定：3/3 → 照常封存。
    let store = gate_store("- [x] 1.1 a\n- [x] 1.2 b\n- [x] 1.3 c\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert!(store.archived_change_exists(&outcome.dated_name));
}

#[test]
fn zero_tasks_passes_the_gate() {
    // spec Example 守門判定：任務總數 0 → 照常封存（條件與批次預過濾一致：總數>0 才擋）。
    let store = gate_store("## Tasks\n\n(none)\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
    assert!(!store.change_exists("demo"), "zero-task change archives as before");
}

#[test]
fn mark_tasks_complete_flag_checks_every_task_inside_archive() {
    // design D1：豁免＝旗標本身，代勾也在 archive() 內——直呼入口（desktop）
    // 帶旗標時與 CLI 同語意：封存成功，且封存後的 tasks.md 全部已勾。
    let store = gate_store("- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { mark_tasks_complete: true, ..skip_opts() };
    let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
    assert!(!store.change_exists("demo"), "flag exempts the gate");
    assert_eq!(
        store.read_archived_artifact(&outcome.dated_name, "tasks.md").as_deref(),
        Some("- [x] 1.1 a\n- [x] 1.2 b\n- [x] 1.3 c\n"),
        "archive() itself checks every task before the move"
    );
}

const UNCHECKED_TASKS: &str = "- [x] 1.1 a\n- [ ] 1.2 b\n* [ ] 1.3 c\n- [ ]\t1.4 d\n";

#[test]
fn mark_tasks_complete_leaves_tasks_untouched_when_validation_refuses() {
    // design D1（spec archive-merge「兩階段合併計畫與零半套寫入」）：結構 validate
    // 拒絕（delta 檔存在但零操作——結構檢查對缺 proposal 寬鬆放行，這是它的硬錯誤）
    // 落在代勾之前，tasks.md 逐位元不變、零 artifact 寫入。
    let store = gate_store(UNCHECKED_TASKS);
    store.put_artifact("demo", "specs/auth/spec.md", "## ADDED Requirements\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    // CLI `speclink archive <name> --mark-tasks-complete` 的形狀：validate 與 spec 合併都開。
    let opts = ArchiveOptions { mark_tasks_complete: true, ..apply_opts_validating() };
    let err = archive(&ghost_ws(), &store, &change, &opts, None)
        .expect_err("a structurally invalid change must refuse archive");
    assert!(err.to_string().contains("Validation failed"), "frozen wording: {err}");
    assert!(store.change_exists("demo"), "change stays in place");
    assert_eq!(
        store.read_artifact("demo", "tasks.md").as_deref(),
        Some(UNCHECKED_TASKS),
        "tasks.md byte-for-byte untouched"
    );
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

#[test]
fn mark_tasks_complete_leaves_tasks_untouched_when_the_dated_name_collides() {
    // design D1：同日撞名拒絕也落在代勾之前——tasks.md 逐位元不變。
    let store = gate_store(UNCHECKED_TASKS);
    // archive() 自取一次日期；測試跨午夜時只放今天會撞不到，今天與明天都放。
    let today = chrono::NaiveDate::parse_from_str(&util::now_stamps().date, "%Y-%m-%d")
        .expect("now_stamps date is YYYY-MM-DD");
    for date in [today, today.succ_opt().expect("tomorrow exists")] {
        store
            .archived_metas
            .borrow_mut()
            .insert(format!("{date}-demo"), "schema: spec-driven\n".to_string());
    }
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { mark_tasks_complete: true, ..skip_opts() };
    let err = archive(&ghost_ws(), &store, &change, &opts, None)
        .expect_err("a same-day name collision must refuse archive");
    assert!(err.to_string().contains("already exists"), "frozen wording: {err}");
    assert!(store.change_exists("demo"), "change stays in place");
    assert_eq!(
        store.read_artifact("demo", "tasks.md").as_deref(),
        Some(UNCHECKED_TASKS),
        "tasks.md byte-for-byte untouched"
    );
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

#[test]
fn mark_tasks_complete_checks_star_bullets_and_tab_checkboxes() {
    // design D1：四條 replace 的凍結規則隨代勾搬進 archive()——星號條列與
    // tab 分隔的 checkbox 同樣被勾，全部守門通過時封存成功。
    let store = gate_store(UNCHECKED_TASKS);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { mark_tasks_complete: true, ..skip_opts() };
    let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert_eq!(
        store.read_archived_artifact(&outcome.dated_name, "tasks.md").as_deref(),
        Some("- [x] 1.1 a\n- [x] 1.2 b\n* [x] 1.3 c\n- [x]\t1.4 d\n"),
        "every checkbox form is checked in the archived tasks.md"
    );
}

// --- 封存的未結工單守門（design D5；spec review-station「封存的未結工單守門」）---

const TICKET: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n\n- [WARNING] src/a.rs — possible smell\n";

#[test]
fn open_review_ticket_refuses_archive_with_three_disposals() {
    // spec Scenario「有工單預設拒絕」：stderr 同列 stamp／discard／--carry-review
    // 三處置，change 未被搬移、零寫入。
    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect_err("open ticket must refuse archive");
    let msg = err.to_string();
    assert!(msg.contains("review stamp"), "stamp disposal named: {msg}");
    assert!(msg.contains("review discard"), "discard disposal named: {msg}");
    assert!(msg.contains("--carry-review"), "carry disposal named: {msg}");
    assert!(
        err.downcast_ref::<crate::command::Refusal>().is_some(),
        "typed Refusal so the runtime classifies refused"
    );
    assert!(store.change_exists("demo"), "change stays in place");
    assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
    assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

#[test]
fn carry_review_archives_and_the_ticket_travels() {
    // spec Scenario「明示帶走」：--carry-review 放行，封存目錄內含 review.md
    //（化石工單——封存側「曾審查未通過」標示的證據）。
    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { carry_review: true, ..skip_opts() };
    let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert_eq!(
        store.read_archived_artifact(&outcome.dated_name, crate::station::REVIEW_DOC).as_deref(),
        Some(TICKET),
        "ticket rides the directory move byte-identically"
    );
}

#[test]
fn archive_without_ticket_is_unaffected_by_the_gate() {
    // spec Scenario「無工單行為不變」：本檔其餘測試全數無工單、即回歸網；
    // 此處釘住 carry_review 預設 false 下無工單照常封存、封存區無工單檔。
    let store = gate_store("- [x] 1.1 a\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert!(
        store.read_archived_artifact(&outcome.dated_name, crate::station::REVIEW_DOC).is_none(),
        "no fossil ticket appears out of nowhere"
    );
}

// --- 封存的驗證工單守門與雙工單並存（design D4；spec verify-station）---

const VERIFY_TICKET: &str = "# Verify — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n\n- [CRITICAL] src/a.rs — requirement R2 has no implementation\n";

#[test]
fn open_verify_ticket_refuses_archive_with_three_disposals() {
    // spec Scenario「僅驗證工單時拒絕」：stderr 同列 stamp／discard／--carry-verify
    // 三處置，change 未被搬移、零寫入。
    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::VERIFY_DOC, VERIFY_TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect_err("open verify ticket must refuse archive");
    let msg = err.to_string();
    assert!(msg.contains("verify stamp"), "stamp disposal named: {msg}");
    assert!(msg.contains("verify discard"), "discard disposal named: {msg}");
    assert!(msg.contains("--carry-verify"), "carry disposal named: {msg}");
    assert!(
        err.downcast_ref::<crate::command::Refusal>().is_some(),
        "typed Refusal so the runtime classifies refused"
    );
    assert!(store.change_exists("demo"), "change stays in place");
    assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
    assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

#[test]
fn both_open_tickets_list_both_disposal_groups() {
    // spec Scenario「雙工單並存」：兩站處置並列——只報一站會讓使用者處理完
    // 一張工單再撞一次同樣的牆。
    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
    store.put_artifact("demo", crate::station::VERIFY_DOC, VERIFY_TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect_err("two open tickets must refuse archive");
    let msg = err.to_string();
    for needle in [
        "review stamp",
        "review discard",
        "--carry-review",
        "verify stamp",
        "verify discard",
        "--carry-verify",
    ] {
        assert!(msg.contains(needle), "both disposal groups listed, missing {needle}: {msg}");
    }
    assert!(store.change_exists("demo"), "change stays in place");
}

#[test]
fn carry_verify_archives_and_the_ticket_travels() {
    // spec Scenario「明示帶走驗證工單」：--carry-verify 放行，封存目錄內含
    // verify.md（化石工單——封存側「曾驗證未通過」標示的證據）。
    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::VERIFY_DOC, VERIFY_TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { carry_verify: true, ..skip_opts() };
    let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert_eq!(
        store.read_archived_artifact(&outcome.dated_name, crate::station::VERIFY_DOC).as_deref(),
        Some(VERIFY_TICKET),
        "ticket rides the directory move byte-identically"
    );
}

#[test]
fn the_two_carry_flags_are_independent_and_combine() {
    // spec「`--carry-review` 與 `--carry-verify` 可同時帶」：單帶一支仍被
    // 另一站擋下（帶走哪種工單是兩個獨立決定），兩支齊帶才放行。
    let one_flag_still_refuses = |carry_review: bool, carry_verify: bool, expect: &str| {
        let store = gate_store("- [x] 1.1 a\n");
        store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
        store.put_artifact("demo", crate::station::VERIFY_DOC, VERIFY_TICKET);
        let change = crate::model::find_change(&store, "demo").unwrap();
        let opts = ArchiveOptions { carry_review, carry_verify, ..skip_opts() };
        let err = archive(&ghost_ws(), &store, &change, &opts, None)
            .expect_err("the other station's ticket must still refuse");
        assert!(err.to_string().contains(expect), "names the remaining station: {err}");
    };
    one_flag_still_refuses(true, false, "--carry-verify");
    one_flag_still_refuses(false, true, "--carry-review");

    let store = gate_store("- [x] 1.1 a\n");
    store.put_artifact("demo", crate::station::REVIEW_DOC, TICKET);
    store.put_artifact("demo", crate::station::VERIFY_DOC, VERIFY_TICKET);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { carry_review: true, carry_verify: true, ..skip_opts() };
    let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
    assert!(!store.change_exists("demo"), "both flags archive the change");
    assert!(store
        .read_archived_artifact(&outcome.dated_name, crate::station::REVIEW_DOC)
        .is_some());
    assert!(store
        .read_archived_artifact(&outcome.dated_name, crate::station::VERIFY_DOC)
        .is_some());
}

#[test]
fn archive_without_a_verify_ticket_is_unaffected_by_the_gate() {
    // spec「皆無工單時 archive 行為 SHALL 維持不變」的回歸斷言：本檔其餘測試
    // 全數無工單即回歸網；此處釘住封存區不會憑空長出 verify.md。
    let store = gate_store("- [x] 1.1 a\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert!(
        store.read_archived_artifact(&outcome.dated_name, crate::station::VERIFY_DOC).is_none(),
        "no fossil ticket appears out of nowhere"
    );
}

// --- archive trace 由 evidence 建立（spec verify-evidence）---

/// 新開 capability 的 delta：Purpose 守門（design D3）要求它自帶合格 Purpose，
/// 否則封存被拒——這裡帶著，讓測試專注在 trace 與 evidence 面。
const DELTA_SPEC: &str = "## Purpose\n\n本 capability 負責身分驗證的簽發與撤銷，涵蓋權杖生命週期各階段的可觀察行為、失敗處置與稽核紀錄。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

fn temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("speclink-archive-trace-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn trace_store() -> TestStore {
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact("demo", "specs/auth/spec.md", DELTA_SPEC);
    store
}

fn apply_opts() -> ArchiveOptions {
    ArchiveOptions { skip_specs: false, ..skip_opts() }
}

/// bulk 與 CLI 單筆的預設旗標形狀：validate 與 spec 合併都開、不代勾。
fn apply_opts_validating() -> ArchiveOptions {
    ArchiveOptions { no_validate: false, ..apply_opts() }
}

/// spec「archive trace 注入與零證據提示」：`updated` 是 RFC 3339 帶偏移量的秒級時戳，
/// 其日曆日等於封存目錄名前綴。回傳擷取到的時戳。
fn assert_trace_stamp(canon: &str, dated_name: &str) -> String {
    let re = regex::Regex::new(
        r"<!-- @trace\nsource: demo\nupdated: (\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2})\n-->",
    )
    .unwrap();
    let caps = re.captures(canon).unwrap_or_else(|| panic!("trace block is exactly source + RFC 3339 updated: {canon}"));
    let stamp = caps[1].to_string();
    assert_eq!(&stamp[..10], &dated_name[..10], "stamp day equals the dated directory prefix");
    stamp
}

/// A workspace whose root exists on disk, so evidence can be written under it.
struct TraceWs {
    ws: Workspace,
}

impl TraceWs {
    fn new(tag: &str) -> TraceWs {
        TraceWs {
            ws: Workspace { root: temp_root(tag), spec_dir_name: "openspec".to_string() },
        }
    }

    /// Record one v2 evidence entry for "demo" — the record's mere presence
    /// is all archive reads now.
    fn record_evidence(&self, store: &TestStore) {
        let record = TouchedRecord {
            version: Some(2),
            change: "demo".to_string(),
            touched: Vec::new(),
            entries: vec![crate::tasks::EvidenceEntry {
                task_id: "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                task_desc: "1.1 done".to_string(),
                actor: Some("Tester <t@example.com>".to_string()),
                repo: None,
                head_commit: None,
                touched_files: vec!["src/a.rs".to_string()],
                recorded_at: "2026-07-13T00:00:00Z".to_string(),
            }],
        };
        record.save(store).unwrap();
    }
}

impl Drop for TraceWs {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.ws.root);
    }
}

#[test]
fn trace_carries_only_source_and_updated_on_a_fresh_canonical() {
    // spec Scenario「trace 兩欄一律注入」：ADDED 物化到新正典，trace 僅兩欄、無 code 清單。
    let t = TraceWs::new("fresh");
    let store = trace_store();
    t.record_evidence(&store);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    assert!(outcome.evidence_recorded, "a change with a v2 entry reports evidence recorded");
    let canon = store.read_canonical_spec("auth").unwrap();
    assert_trace_stamp(&canon, &outcome.dated_name);
    assert!(!canon.contains("code:"), "no file list may survive: {canon}");
    assert!(!canon.contains("  - src/a.rs"), "no file list may survive: {canon}");
}

#[test]
fn trace_is_injected_for_modified_even_with_a_clean_work_tree() {
    // spec Scenario「trace 兩欄一律注入」：注入不再依檔案清單有無決定——
    // 乾淨工作樹、無任何髒檔的 MODIFIED 一樣拿到 trace。
    let t = TraceWs::new("modified");
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
    );
    store.canonical.borrow_mut().insert("auth".to_string(), CANON_R1.to_string());
    t.record_evidence(&store);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    let canon = store.read_canonical_spec("auth").unwrap();
    assert!(canon.contains("<!-- @trace"), "MODIFIED gets a trace block: {canon}");
    assert!(canon.contains("source: demo"), "{canon}");
    assert!(!canon.contains("code:"), "no file list may survive: {canon}");
}

#[test]
fn modified_leaves_other_requirements_date_only_trace_untouched() {
    // spec Scenario「既有純日期 trace 不回改」：只有被物化的需求拿到新時戳，
    // 正典內其他需求的純日期 updated 行逐位元不變。
    let t = TraceWs::new("keep-dates");
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
    );
    let r2_trace = "<!-- @trace\nsource: older\nupdated: 2026-07-10\n-->";
    let canon_before = format!("{CANON_R1_R2}\n{r2_trace}\n");
    store.canonical.borrow_mut().insert("auth".to_string(), canon_before);
    t.record_evidence(&store);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    let canon = store.read_canonical_spec("auth").unwrap();
    assert_trace_stamp(&canon, &outcome.dated_name);
    assert!(canon.contains(r2_trace), "R2's date-only trace survives byte-for-byte: {canon}");
    assert_eq!(canon.matches("updated: ").count(), 2, "one stamp per requirement: {canon}");
}

#[test]
fn a_change_without_evidence_archives_and_reports_it() {
    // spec Scenario「零證據照常封存並提示」的引擎面：無任何 v2 entry 不再是拒絕
    // 理由——封存照常完成，outcome 帶著「沒有證據」這個事實供 CLI 呈現。
    let t = TraceWs::new("no-evidence");
    let store = trace_store();
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    assert!(!outcome.evidence_recorded, "zero entries is reported, not refused");
    assert!(!store.change_exists("demo"), "the change still archives");
    let canon = store.read_canonical_spec("auth").unwrap();
    assert_trace_stamp(&canon, &outcome.dated_name);
}

#[test]
fn evidence_content_never_blocks_the_archive() {
    // 討論 evidence-gate-false-blocks：記錄的內容（含前版寫入的 basis digests）
    // 不再被判讀——只要記錄在，封存就通過,連「過期」這個概念都不存在了。
    let t = TraceWs::new("stale-shaped");
    let store = trace_store();
    // 前一版格式：帶 basisDigests 且必然對不上當前基準。
    store.put_evidence(
        "demo",
        r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_LEGACY","taskDesc":"1.1 done","touchedFiles":["src/a.rs"],"basisDigests":{"spec":"sha256:0","tasks":"sha256:0","policy":"sha256:0"},"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
    );
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    assert!(outcome.evidence_recorded, "the entry counts however its basis reads");
    assert!(!store.change_exists("demo"), "no staleness judgment stands in the way");
}

#[test]
fn archive_sweeps_the_legacy_touched_record_with_the_change() {
    // 舊路徑殘檔不得比 change 活得久：留著的話，同名新 change 的第一次 load
    // 會把死帳讀成活帳（seen 汙染、零證據提示被吞）。封存比照 `.started`
    // 標記順手帶走。evidence_recorded 的事實經 Store seam 讀取（舊路徑內容
    // 「仍算這個 change 的事實」由 speclink-fs 的回退讀取測試釘住）。
    let t = TraceWs::new("legacy-sweep");
    let store = trace_store();
    store.put_evidence(
        "demo",
        r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_LEGACY","taskDesc":"1.1 done","touchedFiles":["src/a.rs"],"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
    );
    let legacy = t.ws.legacy_touched_file("demo");
    util::write_file(&legacy, "{\"change\":\"demo\",\"touched\":[]}").unwrap();
    let change = crate::model::find_change(&store, "demo").unwrap();
    let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    assert!(outcome.evidence_recorded, "the seam-read record still counts as this change's fact");
    assert!(!legacy.exists(), "the legacy touched record dies with the change");
}

#[test]
fn a_pure_removed_merge_keeps_the_trailing_newline() {
    // 純 REMOVED（或純 RENAMED）的合併不注入任何 @trace——這種輸出維持文字檔
    // 的結尾換行；以 `-->` 收尾者除外（不論來自本輪注入或前次封存的殘尾，
    // 見下一測試），凍結為無結尾換行的形狀。
    let t = TraceWs::new("removed-newline");
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## REMOVED Requirements\n\n### Requirement: R1\n\n**Reason**: retired.\n**Migration**: none.\n",
    );
    store.canonical.borrow_mut().insert("auth".to_string(), CANON_R1_R2.to_string());
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    let canon = store.read_canonical_spec("auth").unwrap();
    assert!(!canon.contains("@trace"), "a pure REMOVED merge injects nothing: {canon}");
    assert!(
        canon.ends_with('\n') && !canon.ends_with("\n\n"),
        "a trace-less merge ends with exactly one trailing newline: {:?}",
        &canon[canon.len().saturating_sub(20)..]
    );
}

#[test]
fn a_trace_tailed_canon_keeps_its_frozen_tail_through_a_pure_removed_merge() {
    // 補結尾換行的規則有一道例外，與 fresh 路徑同規則：末塊以先前封存注入的
    // `-->` 收尾的正典維持凍結形狀——`-->` 之後永不補換行。
    let t = TraceWs::new("trace-tail");
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## REMOVED Requirements\n\n### Requirement: R1\n\n**Reason**: retired.\n**Migration**: none.\n",
    );
    let traced_tail_canon = format!(
        "{}\n\n<!-- @trace\nsource: earlier\nupdated: 2026-07-01\n-->",
        CANON_R1_R2.trim_end()
    );
    store.canonical.borrow_mut().insert("auth".to_string(), traced_tail_canon);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

    let canon = store.read_canonical_spec("auth").unwrap();
    assert!(
        canon.ends_with("-->"),
        "no newline may ever follow a trailing `-->`: {:?}",
        &canon[canon.len().saturating_sub(20)..]
    );
}

// --- 封存合併 fail-closed 守門（design「違規清單與聚合錯誤形狀」；
//     spec archive-merge「封存合併 fail-closed 守門」）---

const CANON_R1: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
const CANON_R1_R2: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n---\n\n### Requirement: R2\n\nIt SHALL also work.\n\n#### Scenario: fine\n\n- **WHEN** used\n- **THEN** fine\n";
const ADDED_R1: &str = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

/// 一份可封存的 change（任務全勾、無工單），delta 與正典由呼叫端指定。
fn merge_store(deltas: &[(&str, &str)], canon: &[(&str, &str)]) -> TestStore {
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    for (cap, text) in deltas {
        store.put_artifact("demo", &crate::model::delta_spec_artifact(cap), text);
    }
    for (cap, text) in canon {
        store.canonical.borrow_mut().insert((*cap).to_string(), (*text).to_string());
    }
    store
}

/// 封存必須被守門拒絕：typed Refusal、正典與 change 零效果，回傳錯誤訊息供逐條斷言。
fn refuse_merge(store: &TestStore) -> String {
    let before = store.canonical.borrow().clone();
    let change = crate::model::find_change(store, "demo").unwrap();
    let err = archive(&ghost_ws(), store, &change, &apply_opts(), None)
        .expect_err("a violating delta must refuse archive");
    assert!(
        err.downcast_ref::<crate::command::Refusal>().is_some(),
        "typed Refusal so the runtime classifies refused: {err}"
    );
    assert!(store.change_exists("demo"), "change stays in place");
    assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
    assert_eq!(*store.canonical.borrow(), before, "canonical specs untouched");
    err.to_string()
}

// --- bulk 預檢的唯讀投影（design D4；spec archive-merge「過期判定單源共用」）---

/// 過期 delta（ADDED 撞正典）＋結構不合法（零操作 delta）＋任務未完成（1/2）
/// 三條同時成立的 change。
fn triple_skip_store() -> TestStore {
    let store = merge_store(&[("auth", ADDED_R1), ("bad", "## ADDED Requirements\n")], &[("auth", CANON_R1)]);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 a\n- [ ] 1.2 b\n");
    store
}

#[test]
fn skip_reason_picks_merge_then_validate_then_tasks_in_bulk_order() {
    // 判定順序是 bulk 的凍結輸出契約（merge → validate → tasks），不是 archive() 的守門序：
    // 三條同時成立時回 merge；每豁免一條，下一條浮上來。呼叫過程零寫入。
    let store = triple_skip_store();
    let change = crate::model::find_change(&store, "demo").unwrap();

    let reason = skip_reason(&store, &change, &apply_opts_validating());
    assert!(
        matches!(&reason, Some(SkipReason::MergeRefused(vs)) if !vs.is_empty()),
        "merge comes first: {reason:?}"
    );

    let opts = ArchiveOptions { skip_specs: true, ..apply_opts_validating() };
    let reason = skip_reason(&store, &change, &opts);
    assert!(
        matches!(reason, Some(SkipReason::StructuralInvalid)),
        "--skip-specs drops merge, validate is next: {reason:?}"
    );

    let opts = ArchiveOptions { skip_specs: true, no_validate: true, ..apply_opts_validating() };
    let reason = skip_reason(&store, &change, &opts);
    assert!(
        matches!(reason, Some(SkipReason::TasksIncomplete { complete: 1, total: 2 })),
        "--no-validate drops validate, tasks is last: {reason:?}"
    );

    assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
    assert!(store.change_exists("demo"), "a projection never moves the change");
}

#[test]
fn skip_reason_is_none_when_every_condition_is_exempted() {
    // 三旗標齊帶 → 沒有可跳過的理由；--mark-tasks-complete 在投影裡從不代勾。
    let store = triple_skip_store();
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions {
        skip_specs: true,
        no_validate: true,
        mark_tasks_complete: true,
        ..apply_opts_validating()
    };
    let reason = skip_reason(&store, &change, &opts);
    assert!(reason.is_none(), "every reason exempted: {reason:?}");
    assert_eq!(*store.artifact_writes.borrow(), 0, "the flag never pre-writes here");
}

#[test]
fn skip_reason_is_none_for_a_ready_change() {
    // 全就緒（delta 對得上正典、結構合法、任務全勾）→ None，bulk 不跳過。
    let store = merge_store(&[("auth", DELTA_SPEC)], &[]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let reason = skip_reason(&store, &change, &apply_opts_validating());
    assert!(reason.is_none(), "ready change has no skip reason: {reason:?}");
    assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
}

// --- 守門序只在 archive() 一份：任務、章失效、結構 validate 皆先於 merge 拒絕 ---

#[test]
fn incomplete_tasks_refuse_before_the_merge_gate() {
    // 任務未完成＋delta 過期 → 任務守門先拒（訊息不變），merge 拒絕字串不出現。
    let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 a\n");
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &apply_opts_validating(), None)
        .expect_err("incomplete tasks must refuse");
    let msg = err.to_string();
    assert!(msg.contains("0/1 tasks complete"), "task gate wording: {msg}");
    assert!(!msg.contains("cannot be archived"), "merge refusal never reached: {msg}");
}

#[test]
fn structural_invalidity_refuses_before_the_merge_gate() {
    // 結構不合法＋delta 過期 → `Validation failed:` 先於 merge 拒絕。
    let store = merge_store(
        &[("auth", ADDED_R1), ("bad", "## ADDED Requirements\n")],
        &[("auth", CANON_R1)],
    );
    let change = crate::model::find_change(&store, "demo").unwrap();
    let err = archive(&ghost_ws(), &store, &change, &apply_opts_validating(), None)
        .expect_err("structural invalidity must refuse");
    let msg = err.to_string();
    assert!(msg.contains("Validation failed"), "validate wording: {msg}");
    assert!(!msg.contains("cannot be archived"), "merge refusal never reached: {msg}");
}

#[test]
fn added_requirement_already_in_canon_refuses_archive() {
    // spec Scenario「過期 ADDED 被拒絕」：撞名的 ADDED 不再靜默跳過。
    let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("auth"), "capability named: {msg}");
    assert!(msg.contains("ADDED"), "operation named: {msg}");
    assert!(msg.contains("R1"), "requirement named: {msg}");
    assert!(msg.contains("already exists"), "reason given: {msg}");
}

#[test]
fn modified_removed_renamed_missing_target_refuses_archive() {
    // spec Scenario「缺目標的 MODIFIED 被拒絕」：三種操作缺來源需求時一致拒絕。
    for (op, delta) in [
        ("MODIFIED", "## MODIFIED Requirements\n\n### Requirement: Ghost\n\nIt SHALL change.\n"),
        ("REMOVED", "## REMOVED Requirements\n\n### Requirement: Ghost\n"),
        (
            "RENAMED",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: Ghost`\n- TO: `### Requirement: Spirit`\n",
        ),
    ] {
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains(op), "operation named ({op}): {msg}");
        assert!(msg.contains("Ghost"), "requirement named ({op}): {msg}");
        assert!(msg.contains("no longer exists"), "reason given ({op}): {msg}");
    }
}

#[test]
fn same_requirement_in_two_operation_sections_refuses_archive() {
    // spec Scenario「多區段互撞被拒絕」：同名需求橫跨 MODIFIED 與 REMOVED。
    let delta = "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## REMOVED Requirements\n\n### Requirement: R1\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("R1"), "requirement named: {msg}");
    assert!(
        msg.contains("MODIFIED") && msg.contains("REMOVED"),
        "both colliding operations listed: {msg}"
    );
}

#[test]
fn renamed_target_name_already_in_canon_refuses_archive() {
    // spec Scenario「多區段互撞被拒絕」之 RENAMED 目標側：改名後會撞既有需求。
    let store = merge_store(
        &[(
            "auth",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: R2`\n",
        )],
        &[("auth", CANON_R1_R2)],
    );
    let msg = refuse_merge(&store);
    assert!(msg.contains("RENAMED"), "operation named: {msg}");
    assert!(msg.contains("R2"), "rename target named: {msg}");
    assert!(msg.contains("already exists"), "reason given: {msg}");
}

#[test]
fn fresh_capability_with_non_added_operation_refuses_archive() {
    // spec Scenario「新 capability 僅接受 ADDED」：正典不存在時 MODIFIED 不再物化成新規格。
    let store = merge_store(
        &[("fresh", "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL change.\n")],
        &[],
    );
    let msg = refuse_merge(&store);
    assert!(msg.contains("fresh"), "capability named: {msg}");
    assert!(msg.contains("MODIFIED"), "operation named: {msg}");
    assert!(msg.contains("does not exist"), "reason given: {msg}");
    assert!(store.canonical.borrow().is_empty(), "no canonical spec materialized");
}

#[test]
fn every_violation_is_reported_at_once_with_remediation_guidance() {
    // spec Scenario「違規聚合一次回報」：跨 capability 的違規單次列齊，並附 drift → ingest 動線。
    let store = merge_store(
        &[
            ("auth", ADDED_R1),
            ("billing", "## MODIFIED Requirements\n\n### Requirement: Ghost\n\nIt SHALL change.\n"),
        ],
        &[("auth", CANON_R1), ("billing", CANON_R1)],
    );
    let msg = refuse_merge(&store);
    assert!(msg.contains("auth") && msg.contains("R1"), "first violation listed: {msg}");
    assert!(
        msg.contains("billing") && msg.contains("Ghost"),
        "second violation listed in the same report: {msg}"
    );
    assert!(msg.contains("drift"), "drift remediation named: {msg}");
    assert!(msg.contains("ingest"), "ingest remediation named: {msg}");
}

#[test]
fn no_validate_does_not_unlock_the_merge_gate() {
    // spec Scenario「no-validate 不解鎖守門」：文件驗證略過，合併守門照常拒絕。
    let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    let opts = ArchiveOptions { skip_specs: false, no_validate: true, ..skip_opts() };
    let err = archive(&ghost_ws(), &store, &change, &opts, None)
        .expect_err("--no-validate must not unlock the merge gate");
    assert!(err.to_string().contains("already exists"), "gate still speaks: {err}");
}

#[test]
fn skip_specs_bypasses_the_merge_gate_as_before() {
    // 既有逃生口：--skip-specs 整段跳過規格套用，守門自然不觸發。
    let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect("--skip-specs keeps its existing escape-hatch semantics");
    assert!(!store.change_exists("demo"), "change moved into the archive");
    assert_eq!(
        store.read_canonical_spec("auth").as_deref(),
        Some(CANON_R1),
        "canonical spec untouched when spec application is skipped"
    );
}

// --- 兩階段合併計畫與零半套寫入（design「兩階段合併」；
//     spec archive-merge「兩階段合併計畫與零半套寫入」）---

#[test]
fn one_violating_capability_leaves_every_capability_untouched() {
    // spec Scenario「任一 capability 違規則全部不寫」：雙 capability 其一合法、
    // 其一違規 → 兩正典皆未變、無 snapshot 落地、change 仍在進行區原位。
    let root = temp_root("two-phase");
    let ws = Workspace { root: root.clone(), spec_dir_name: "openspec".to_string() };
    let store = merge_store(
        &[
            ("auth", "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
            ("billing", ADDED_R1),
        ],
        &[("auth", CANON_R1), ("billing", CANON_R1)],
    );
    let before = store.canonical.borrow().clone();
    let change = crate::model::find_change(&store, "demo").unwrap();

    let err = archive(&ws, &store, &change, &apply_opts(), None)
        .expect_err("a single violating capability refuses the whole archive");

    assert!(err.to_string().contains("billing"), "the violating capability is named: {err}");
    assert_eq!(*store.canonical.borrow(), before, "no canonical spec was written");
    assert!(store.change_exists("demo"), "change stays in the active area");
    assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
    assert!(
        !ws.snapshots_dir().exists(),
        "zero file effect: no snapshot directory was created"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn all_snapshots_land_before_any_canonical_write() {
    // spec Scenario「snapshot 先於正典寫入」：對第一個 capability 的正典寫入注入
    // 失敗；順序正確時第二個 capability 的 snapshot 已在磁碟上（交錯寫入則不會）。
    let root = temp_root("write-order");
    let ws = Workspace { root: root.clone(), spec_dir_name: "openspec".to_string() };
    let store = merge_store(
        &[
            ("auth", "## ADDED Requirements\n\n### Requirement: Fresh A\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
            ("billing", "## ADDED Requirements\n\n### Requirement: Fresh B\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
        ],
        &[("auth", CANON_R1), ("billing", CANON_R1)],
    );
    *store.fail_canonical_write.borrow_mut() = Some("auth".to_string());
    let change = crate::model::find_change(&store, "demo").unwrap();

    let err = archive(&ws, &store, &change, &apply_opts(), None)
        .expect_err("the injected canonical write failure surfaces");
    // design 風險表：commit 階段失敗的錯誤訊息指出 snapshot 位置。
    assert!(
        err.to_string().contains(&ws.snapshots_dir().display().to_string()),
        "the failure names the snapshot location: {err}"
    );

    let dated = format!("{}-demo", util::today());
    for cap in ["auth", "billing"] {
        assert!(
            ws.snapshots_dir().join(&dated).join("specs").join(cap).join("spec.md").is_file(),
            "every snapshot backup lands before the first canonical write ({cap})"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

// --- MODIFIED 的 scenario 保全與明示刪除聲明（design「scenario superset check
//     與明示刪除聲明」；spec archive-merge 同名需求）---

/// spec Example 的正典目標需求：兩個 scenario「逾時重試」與「離線佇列」。
const CANON_TWO_SCENARIOS: &str = "# net Specification\n\n## Purpose\n\nNet.\n\n## Requirements\n\n### Requirement: 重試策略\n\nIt SHALL retry.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry\n\n#### Scenario: 離線佇列\n\n- **WHEN** offline\n- **THEN** queue\n";

#[test]
fn modified_dropping_a_canonical_scenario_refuses_and_names_it() {
    // spec Scenario「漏抄 scenario 被拒絕並點名」：delta 只留「逾時重試」、
    // 無刪除聲明 → 拒絕並點名遺失的「離線佇列」。
    let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
    let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("重試策略"), "requirement named: {msg}");
    assert!(msg.contains("離線佇列"), "the dropped scenario is named: {msg}");
    assert!(!msg.contains("逾時重試"), "the surviving scenario is not flagged: {msg}");
}

#[test]
fn declared_scenario_removal_passes_and_the_note_is_stripped() {
    // spec Scenario「明示聲明後允許刪除」：聲明放行，合併後正典含「逾時重試」、
    // 不含「離線佇列」、也不含聲明註解本身。
    let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- REMOVED-SCENARIO: 離線佇列 -->\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
    let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &apply_opts(), None)
        .expect("an explicit removal declaration lets the merge through");
    let canon = store.read_canonical_spec("net").expect("canonical spec written");
    assert!(canon.contains("#### Scenario: 逾時重試"), "kept scenario survives: {canon}");
    assert!(!canon.contains("離線佇列"), "declared scenario is gone: {canon}");
    assert!(!canon.contains("REMOVED-SCENARIO"), "the declaration itself is stripped: {canon}");
}

#[test]
fn crlf_authored_delta_matches_scenario_names_the_same_way() {
    // design Risk「Windows 換行使 scenario 名比對失準」：CRLF 樣本的判定與 LF 一致
    // ——完整抄錄放行、漏抄則拒絕並點名。
    let complete = "## MODIFIED Requirements\r\n\r\n### Requirement: 重試策略\r\n\r\nIt SHALL retry harder.\r\n\r\n#### Scenario: 逾時重試\r\n\r\n- **WHEN** timeout\r\n- **THEN** retry twice\r\n\r\n#### Scenario: 離線佇列\r\n\r\n- **WHEN** offline\r\n- **THEN** queue\r\n";
    let store = merge_store(&[("net", complete)], &[("net", CANON_TWO_SCENARIOS)]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &apply_opts(), None)
        .expect("a CRLF delta carrying every scenario passes the superset check");

    let partial = "## MODIFIED Requirements\r\n\r\n### Requirement: 重試策略\r\n\r\nIt SHALL retry harder.\r\n\r\n#### Scenario: 逾時重試\r\n\r\n- **WHEN** timeout\r\n- **THEN** retry twice\r\n";
    let store = merge_store(&[("net", partial)], &[("net", CANON_TWO_SCENARIOS)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("離線佇列"), "CRLF authoring still names the dropped scenario: {msg}");
}

// --- 新 capability 的 Purpose 自 delta 帶入（design 同名決策；
//     spec archive-merge「新 capability 的 Purpose 自 delta 帶入」）---

const DELTA_WITH_PURPOSE: &str = "## Purpose\n\n本 capability 管理權杖的輪替與撤銷，涵蓋簽發、驗證與失效三段生命週期的可觀察行為與清理時機。\n\n## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

#[test]
fn delta_purpose_becomes_the_new_canonical_purpose() {
    // spec Scenario「delta 提供 Purpose」：新建正典的 Purpose 為 delta 區段內容，非占位文字。
    let store = merge_store(&[("token", DELTA_WITH_PURPOSE)], &[]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &apply_opts(), None).unwrap();
    let canon = store.read_canonical_spec("token").expect("canonical spec created");
    let purpose = crate::model::purpose_content(DELTA_WITH_PURPOSE).expect("fixture purpose");
    assert!(
        canon.contains(&format!("## Purpose\n\n{purpose}\n")),
        "delta Purpose copied verbatim: {canon}"
    );
    assert!(!canon.contains("TBD"), "no placeholder skeleton remains: {canon}");
    assert!(
        !canon.contains(&format!("{purpose}\n\n## ADDED")),
        "the delta's operation heading does not leak into the canon: {canon}"
    );
}

const ADDED_ONLY: &str = "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

#[test]
fn new_capability_without_purpose_refuses_archive() {
    // spec Scenario「新 capability 缺 Purpose 封存被拒」：守門取代靜默寫佔位，
    // 拒絕時零檔案效果（refuse_merge 逐項斷言）。
    let store = merge_store(&[("token", ADDED_ONLY)], &[]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("token"), "不合格的 capability 被點名: {msg}");
    assert!(msg.contains("Purpose"), "不合格原因指向 Purpose: {msg}");
}

#[test]
fn new_capability_with_a_too_short_purpose_refuses_archive() {
    // spec Scenario「新 capability 的 Purpose 過短封存被拒」。
    let short = format!("## Purpose\n\n管權杖。\n\n{ADDED_ONLY}");
    let store = merge_store(&[("token", &short)], &[]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("token"), "capability 被點名: {msg}");
    assert!(
        msg.contains(&crate::model::MIN_PURPOSE_LENGTH.to_string()),
        "回報不足門檻的原因: {msg}"
    );
}

#[test]
fn an_existing_capability_never_hits_the_purpose_gate() {
    // 既有 capability 的 delta 無論帶不帶 Purpose 都不構成封存拒絕理由
    // （spec：忽略不報錯）——正典已有 Purpose，守門只管新開的。
    for delta in [
        ADDED_R1.replace("R1", "Brand new"),
        format!("## Purpose\n\n短。\n\n{}", ADDED_R1.replace("R1", "Brand new")),
    ] {
        let store = merge_store(&[("auth", &delta)], &[("auth", CANON_R1)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None)
            .expect("既有 capability 不受 Purpose 守門影響");
    }
}

#[test]
fn skip_specs_archive_does_not_trigger_the_purpose_gate() {
    // spec：skip_specs 封存不觸發此守門（無 delta 可驗）。
    let store = merge_store(&[("token", ADDED_ONLY)], &[]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &skip_opts(), None)
        .expect("--skip-specs 不套用 delta，也就不驗 Purpose");
    assert!(store.read_canonical_spec("token").is_none(), "正典未被建立");
}

#[test]
fn purpose_reason_carries_the_refusal_wording() {
    // spec archive-merge「新 capability 缺 Purpose 的違規呈現三處一致」：
    // 三處（drift／bulk 預檢／單筆 archive）共用同一 reason 字串，語意比照
    // ADDED_EXISTS 的「archive would refuse it」。
    let store = merge_store(&[("token", ADDED_ONLY)], &[]);
    let violations = merge_violations(&store, "demo");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].is_purpose_gate(), "the purpose violation self-identifies");
    assert!(
        violations[0].reason.contains("archive would refuse it"),
        "refusal wording travels in the shared reason: {}",
        violations[0].reason
    );
}

#[test]
fn purpose_only_refusal_names_the_remedy_not_drift_ingest() {
    // 純 Purpose 違規的拒絕訊息：說對原因（缺 `## Purpose`）、給對修法
    // （補區段＋validate 指引），不再指向修不了它的 drift → ingest。
    let store = merge_store(&[("token", ADDED_ONLY)], &[]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("## Purpose"), "the real cause is named: {msg}");
    assert!(
        msg.contains(&crate::model::MIN_PURPOSE_LENGTH.to_string()),
        "the remedy names the threshold: {msg}"
    );
    assert!(msg.contains("speclink validate"), "the remedy points at validate: {msg}");
    assert!(!msg.contains("/speclink-ingest"), "ingest cannot fix this: {msg}");
    assert!(
        !msg.contains("no longer match the canonical spec"),
        "the stale preamble does not misdescribe a purpose violation: {msg}"
    );
}

#[test]
fn mixed_refusal_lists_both_classes_with_both_remedies() {
    // 過期操作與 Purpose 違規並存：兩類各自列明、兩套補救動線並列。
    let stale = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let store = merge_store(&[("auth", stale), ("token", ADDED_ONLY)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("no longer match the canonical spec"), "stale class kept: {msg}");
    assert!(msg.contains("/speclink-ingest"), "stale remedy kept: {msg}");
    assert!(msg.contains("## Purpose"), "purpose class listed: {msg}");
    assert!(msg.contains("speclink validate"), "purpose remedy listed: {msg}");
}

#[test]
fn the_placeholder_skeleton_survives_as_an_unreachable_branch() {
    // 守門上線後 delta 缺 Purpose 走不到合併，佔位分支成為理論不可達的
    // 死路防禦（design D3）——分支本身仍在，且文案仍取自單一常數。
    let (canon, _) = merge_capability("token", "demo", "2026-08-11", ADDED_ONLY, None);
    assert!(
        canon.contains(&format!(
            "{} change 'demo'. Update Purpose after archive.",
            crate::model::PURPOSE_TBD_PREFIX
        )),
        "佔位文案沿用 core 常數: {canon}"
    );
}

#[test]
fn delta_purpose_never_rewrites_an_existing_canonical_purpose() {
    // spec Scenario「既有正典 Purpose 不受 delta 影響」。
    let delta = "## Purpose\n\n這段不該進正典。\n\n## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
    let change = crate::model::find_change(&store, "demo").unwrap();
    archive(&ghost_ws(), &store, &change, &apply_opts(), None).unwrap();
    let canon = store.read_canonical_spec("auth").expect("canonical spec merged");
    assert!(canon.contains("## Purpose\n\nAuth.\n"), "existing Purpose survives: {canon}");
    assert!(!canon.contains("這段不該進正典"), "delta Purpose is not applied: {canon}");
}

// --- 自相矛盾 delta 與註解剝除的守門補強（design「違規清單與聚合錯誤形狀」）---

#[test]
fn duplicate_added_names_in_one_delta_refuse_archive() {
    // 自相矛盾的 delta 必須拒絕：同一 delta 內重複 ADDED 名稱若放行，
    // 合併端（已無去重）會在正典寫出兩個同名需求。
    let delta = "## ADDED Requirements\n\n### Requirement: Fresh\n\nA.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n### Requirement: Fresh\n\nB.\n\n#### Scenario: ok2\n\n- **WHEN** used\n- **THEN** works\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("Fresh"), "requirement named: {msg}");
    assert!(msg.contains("more than once"), "duplication reason given: {msg}");
}

#[test]
fn two_renames_to_the_same_target_refuse_archive() {
    // A→C 與 B→C 兩對 rename 指向同一目標（C 不在正典）若放行，
    // 合併後正典出現兩個名為 C 的需求——以 mention 計數攔下。
    let delta = "## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: C`\n\n- FROM: `### Requirement: R2`\n- TO: `### Requirement: C`\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1_R2)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains('C'), "colliding rename target named: {msg}");
    assert!(msg.contains("more than once"), "duplication reason given: {msg}");
}

#[test]
fn scenario_quoted_only_inside_a_before_note_is_not_carried() {
    // 守門必須以剝除後文字判定：BEFORE 註解內引用的 scenario 行不算已抄錄，
    // 否則寫入前剝除會整段刪掉 → 正典靜默掉 scenario。
    let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- BEFORE:\n#### Scenario: 離線佇列\n-->\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
    let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("離線佇列"), "the dropped scenario is named: {msg}");
}

#[test]
fn unterminated_removed_scenario_declaration_refuses_archive() {
    // 畸形聲明必須拒絕：漏打 `-->` 的聲明若被接受，多行剝除會吞掉其後整個
    // block → 需求本體消失。
    let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- REMOVED-SCENARIO: 離線佇列\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
    let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("REMOVED-SCENARIO"), "malformed declaration named: {msg}");
}

#[test]
fn renamed_header_without_to_line_refuses_archive() {
    // fail-closed 守門：header 形式的 RENAMED 缺 TO: 行套用不到任何目標，
    // 必須拒絕而非靜默忽略。
    let delta = "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## RENAMED Requirements\n\n### Requirement: R1\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("RENAMED") && msg.contains("R1"), "dangling rename named: {msg}");
    assert!(msg.contains("TO:"), "missing-target reason given: {msg}");
}

#[test]
fn bullet_rename_without_to_refuses_archive() {
    // fail-closed 守門的 bullet 形式對稱面：孤兒 `- FROM:`（無 TO 行）與
    // 空值 TO 皆套用不到目標，必須拒絕而非靜默忽略。
    const ADDED_OK: &str = "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n";
    let orphan = format!(
        "{ADDED_OK}## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n"
    );
    let store = merge_store(&[("auth", &orphan)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("RENAMED") && msg.contains("R1"), "orphan FROM named: {msg}");

    let empty_to = format!(
        "{ADDED_OK}## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: ``\n"
    );
    let store = merge_store(&[("auth", &empty_to)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("RENAMED") && msg.contains("R1"), "empty TO named: {msg}");
}

#[test]
fn unterminated_before_note_refuses_archive() {
    // 註解剝除守門：未終結的 `<!-- BEFORE:` 會讓剝除吞到 block 結尾，
    // 需求內容靜默消失——與畸形 REMOVED-SCENARIO 同類，必須拒絕。
    let delta = "## ADDED Requirements\n\n### Requirement: Brand new\n\n<!-- BEFORE:\nold text\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
    let msg = refuse_merge(&store);
    assert!(msg.contains("BEFORE"), "malformed BEFORE note named: {msg}");
}

//! 驗證品質站（design D1／D2／D3）：站別常數組＋委派 [`crate::station`] 的共通
//! 生命週期。與審查站的唯一刻意不對稱是 `add_round` 的引擎守門——verify 檢查
//! 可中途跑（進度盤點是 Completeness 維度的既有功能），工單語意限定為「成品
//! 驗證」，故任務未全數完成時拒絕落工單。
//!
//! 工單 `verify.md` 是 sidecar：不註冊進 workflow schema，僅由動詞經 `&dyn Store`
//! 讀寫——本地隨 git、remote 走 store 文件管道。

use crate::model::ChangeMeta;
use crate::station::{self, StampAnchors, Station};
use crate::store::Store;
use anyhow::Result;

pub use crate::station::{
    content_fingerprint, fingerprint_scope, scope_union, Finding, Freshness, NotFound, Round,
    RoundPhase, Severity, Ticket,
};

/// 驗證站工單文件（change 目錄下的相對路徑）。
pub const VERIFY_DOC: &str = "verify.md";

/// 驗證站的站別常數組（design D1）。`round_requires_tasks_complete` 為 true 是
/// design D3 的刻意不對稱：盤點輪誤落工單會讓「未結工單」失去語意、還會誤觸
/// archive 守門。
pub const STATION: Station = Station {
    doc: VERIFY_DOC,
    meta_prefix: "verified",
    title: "Verify",
    noun: "verify",
    noun_phrase: "verification",
    recheck: "re-verify",
    round_requires_tasks_complete: true,
};

/// 追加一輪驗證（工單不存在則建立，自 Round 1 起算）。回傳本輪編號。
/// 任務未全數完成時拒絕（design D3）。
pub fn add_round(store: &dyn Store, change: &str, content: &str) -> Result<usize> {
    station::add_round(&STATION, store, change, content)
}

/// 讀取並解析工單（spec「驗證工單的讀取」）。無工單回錯誤。
/// `show` 加帶工單原文（供人眼輸出與 wire 的 content 欄位）——同站其餘動詞
/// 一樣經本門面走 station，維持 command 層只認站別門面的分層。純解析版
/// 直接用 `station::show(&STATION, ..)`。
pub fn show_with_content(store: &dyn Store, change: &str) -> Result<(Ticket, Option<String>)> {
    station::show_with_content(&STATION, store, change)
}

/// 放棄驗證：刪除工單、不寫任何 metadata（spec「放棄驗證」）。無工單回錯誤。
pub fn discard(store: &dyn Store, change: &str) -> Result<()> {
    station::discard(&STATION, store, change)
}

/// 蓋章（spec「驗證蓋章守門與蓋章效果」）：守門與審查站同一條——任務全完成＋
/// 末輪零未解必修（CRITICAL／WARNING）findings，SUGGESTION 不擋章（`accept` 僅
/// 豁免必修條件）；通過時於同一原子寫入內落五個 `verified_*` 欄位並刪除工單。
pub fn stamp(
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    read_file: &dyn Fn(&str) -> Option<String>,
    file_exists: &dyn Fn(&str) -> bool,
) -> Result<()> {
    station::stamp(&STATION, store, change, accept, actor, tool, read_file, file_exists)
}

/// scope 注入蓋章（design D4a 的驗證面）：remote 承載——工作樹持有者預算好的
/// (path, hash) 清單直接入章，server 無工作樹不重算。
pub fn stamp_with_scope(
    store: &dyn Store,
    change: &str,
    accept: bool,
    actor: Option<&str>,
    tool: Option<&str>,
    scope: Vec<crate::model::ReviewedScopeEntry>,
    missing: Vec<String>,
) -> Result<()> {
    station::stamp_with_scope(&STATION, store, change, accept, actor, tool, scope, missing)
}

/// 失效判定純函式（spec「驗證指紋錨與失效判定」）：任務錨＋內容錨，規則與審查站
/// 位元級同構（共用 [`station::freshness`]）。
pub fn freshness(
    meta: &ChangeMeta,
    counts: &crate::tasks::Counts,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Freshness {
    station::freshness(
        StampAnchors {
            stamped_at: meta.verified_at.as_deref(),
            tasks_total: meta.verified_tasks_total,
            scope: &meta.verified_scope,
        },
        counts,
        read_file,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
    const TASKS_5_DONE: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [x] 5 e\n";
    const TASKS_4_OF_5: &str = "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] 5 e\n";
    /// 四個寫碼任務全勾、第五個是未勾的 `[M]` 手測——寫碼任務全完成預測子成立。
    const TASKS_CODE_DONE_MANUAL_OPEN: &str =
        "- [x] 1 a\n- [x] 2 b\n- [x] 3 c\n- [x] 4 d\n- [ ] [M] 5 手測\n";

    const ROUND_1: &str = "**Scope**: crates/a/src/lib.rs, crates/b/src/util.rs\n\n- [CRITICAL] crates/a/src/lib.rs — requirement R2 has no implementation\n- [SUGGESTION] crates/b/src/util.rs — design says otherwise\n";
    const ROUND_2: &str = "**Scope**: crates/a/src/lib.rs\n\n- [WARNING] crates/a/src/lib.rs — scenario 3 untested\n";
    const SUGGESTION_ROUND: &str =
        "**Scope**: crates/b/src/util.rs\n\n- [SUGGESTION] crates/b/src/util.rs — design says otherwise\n";

    /// total 個寫碼任務、前 done 個已勾（無 `[M]`）——失效判定的計數輸入。
    fn code_counts(total: usize, done: usize) -> crate::tasks::Counts {
        let md: String =
            (0..total).map(|i| if i < done { "- [x] t\n" } else { "- [ ] t\n" }).collect();
        crate::tasks::counts(&crate::tasks::parse(&md))
    }

    /// 任務全數完成的 change——驗證工單的前提（design D3）。
    fn finished_change() -> TestStore {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        store
    }

    fn structured_round(phase: &str, hex: &str) -> String {
        format!(
            "**Phase**: {phase}\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — requirement R2 has no implementation\n"
        )
    }

    // --- spec「驗證工單的建立與追加」---

    #[test]
    fn add_round_refuses_until_every_task_is_done() {
        // spec Scenario「任務未全完成即拒絕落工單」＋design D3：verify 檢查可中途
        // 跑，但盤點輪不得落工單——4/5 → 非零拒絕、零寫入、無檔案建立。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS_4_OF_5);
        let err = add_round(&store, "demo", ROUND_1).expect_err("incomplete tasks must refuse");
        let msg = err.to_string();
        assert!(msg.contains("4/5"), "error must show the count: {msg}");
        assert!(
            err.downcast_ref::<crate::command::Refusal>().is_some(),
            "typed Refusal so the runtime classifies refused"
        );
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "no ticket may appear");
    }

    #[test]
    fn add_round_lands_the_ticket_when_only_manual_tasks_remain() {
        // spec Scenario「僅餘手動任務可落工單」：寫碼 4/4 全勾、一個 [M] 未勾 → 放行。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS_CODE_DONE_MANUAL_OPEN);
        assert_eq!(add_round(&store, "demo", ROUND_1).expect("manual-only remainder lands"), 1);
        assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must be created");
    }

    #[test]
    fn add_round_refusal_names_the_code_task_counts() {
        // spec「驗證工單的建立與追加」：拒絕訊息點名寫碼任務——手測任務不計入。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] a\n- [ ] b\n- [ ] [M] 手測\n");
        let err = add_round(&store, "demo", ROUND_1).expect_err("open code task must refuse");
        let msg = err.to_string();
        assert!(msg.contains("1/2"), "counts must exclude the [M] task: {msg}");
        assert!(msg.contains("code task"), "message must name code tasks: {msg}");
    }

    #[test]
    fn add_round_creates_ticket_with_round_1_on_first_call() {
        // spec Scenario「首輪建立工單」：任務全完成＋合法內容 → 建檔且自 Round 1 起算。
        let store = finished_change();
        let round = add_round(&store, "demo", ROUND_1).expect("first round");
        assert_eq!(round, 1);
        let doc = store.read_artifact("demo", VERIFY_DOC).expect("ticket must be created");
        assert!(doc.starts_with("# Verify — demo\n"), "station title heads the ticket: {doc}");
        assert!(doc.contains("## Round 1"), "fixed skeleton must carry the round header: {doc}");
        assert!(
            doc.contains("**Scope**: crates/a/src/lib.rs"),
            "round content must be carried verbatim: {doc}"
        );
    }

    #[test]
    fn add_round_appends_round_2_keeping_round_1_byte_identical() {
        // spec Scenario「追加輪次不改寫既有輪」：append-only，Round 1 位元級不變。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("first round");
        let after_first = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
        let round = add_round(&store, "demo", ROUND_2).expect("second round");
        assert_eq!(round, 2);
        let after_second = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
        assert!(
            after_second.starts_with(&after_first),
            "append-only: Round 1 must stay byte-identical\nbefore: {after_first}\nafter: {after_second}"
        );
        assert!(after_second.contains("## Round 2"));
    }

    #[test]
    fn add_round_rejects_content_without_scope_without_writing() {
        // spec Scenario「內容缺少 Scope」：缺 `**Scope**:`（或清單為空）→ 拒絕、零寫入。
        let store = finished_change();
        for bad in ["", "   \n", "- [CRITICAL] a.rs — no scope line\n", "**Scope**:  \n"] {
            let Err(err) = add_round(&store, "demo", bad) else {
                panic!("content {bad:?} must be rejected");
            };
            assert!(err.to_string().contains("**Scope**:"), "error must explain the format: {err}");
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
        assert!(!store.artifact_exists("demo", VERIFY_DOC));
    }

    #[test]
    fn add_round_rejects_missing_change_and_unsafe_names() {
        // 沿 in-progress add／set_board_rank 的同款防護：不存在的 change 與
        // 非單一路徑段名稱皆拒絕、零寫入。
        let store = finished_change();
        let err = add_round(&store, "ghost", ROUND_1).expect_err("missing change must be rejected");
        assert!(err.to_string().contains("ghost"), "error must name the change: {err}");
        assert!(add_round(&store, "../evil", ROUND_1).is_err());
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn structured_round_rejects_unpaired_or_malformed_fields_without_writing() {
        // spec Scenario「phase 與 patch 必須成對」＋格式驗證：兩欄只出現其一、
        // phase token 無效、hash 非 64 lowercase hex → 非零拒絕且工單零寫入。
        let store = finished_change();
        let hex = "a".repeat(64);
        for bad in [
            "**Phase**: discovery\n**Scope**: src/lib.rs\n".to_string(),
            format!("**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n"),
            structured_round("exploration", &hex),
            structured_round("discovery", "zz"),
            structured_round("discovery", &"A".repeat(64)),
            format!("**Phase**: discovery\n**Patch**: {hex}\n**Scope**: src/lib.rs\n"),
        ] {
            assert!(add_round(&store, "demo", &bad).is_err(), "content {bad:?} must be rejected");
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusals must not write");
        assert!(!store.artifact_exists("demo", VERIFY_DOC));
    }

    #[test]
    fn structured_discovery_round_parses_phase_and_patch_hash() {
        // spec Scenario「追加 structured validation」的前半：Round 1 是 discovery，
        // phase／patch 進工單原文並解析回值。
        let store = finished_change();
        let hex = "a".repeat(64);
        add_round(&store, "demo", &structured_round("discovery", &hex)).expect("structured round");
        let ticket = station::show(&STATION, &store, "demo").expect("ticket parses");
        assert_eq!(ticket.rounds[0].phase, Some(RoundPhase::Discovery));
        assert_eq!(ticket.rounds[0].patch_hash.as_deref(), Some(format!("sha256:{hex}").as_str()));
        let doc = store.read_artifact("demo", VERIFY_DOC).expect("ticket doc");
        assert!(doc.contains("**Phase**: discovery"), "{doc}");
        assert!(doc.contains(&format!("**Patch**: sha256:{hex}")), "{doc}");
    }

    #[test]
    fn discovery_then_validation_is_the_only_allowed_sequence() {
        // spec Scenario「第二個 discovery 被拒絕」＋「追加 structured validation」：
        // structured Round 1 只能是 discovery，其後只能是 validation；第二個
        // discovery 拒絕且工單位元級不變。
        let store = finished_change();
        add_round(&store, "demo", &structured_round("discovery", &"a".repeat(64))).expect("r1");
        let before = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
        let err = add_round(&store, "demo", &structured_round("discovery", &"b".repeat(64)))
            .expect_err("a second discovery must be rejected");
        assert!(err.to_string().contains("validation"), "error explains the sequence: {err}");
        assert_eq!(store.read_artifact("demo", VERIFY_DOC).as_deref(), Some(before.as_str()));

        let round = add_round(&store, "demo", &structured_round("validation", &"c".repeat(64)))
            .expect("validation may follow discovery");
        assert_eq!(round, 2);
        let after = store.read_artifact("demo", VERIFY_DOC).expect("ticket");
        assert!(after.starts_with(&before), "append-only across structured rounds");
        assert_eq!(station::show(&STATION, &store, "demo").expect("parses").last_round().phase, Some(RoundPhase::Validation));
    }

    #[test]
    fn validation_is_rejected_on_a_fresh_ticket() {
        // 無任何輪次可驗收：首輪 validation 拒絕、零寫入。
        let store = finished_change();
        let err = add_round(&store, "demo", &structured_round("validation", &"a".repeat(64)))
            .expect_err("fresh-ticket validation must be rejected");
        assert!(err.to_string().contains("discovery"), "error names the required phase: {err}");
        assert_eq!(*store.artifact_writes.borrow(), 0);
    }

    #[test]
    fn legacy_round_parses_with_null_phase_and_patch() {
        // spec Scenario「legacy round 保持相容」：兩欄缺席解析為 None，行為不變。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("legacy round");
        let ticket = station::show(&STATION, &store, "demo").expect("parses");
        assert_eq!(ticket.rounds[0].phase, None);
        assert_eq!(ticket.rounds[0].patch_hash, None);
    }

    #[test]
    fn add_round_rejects_scope_paths_that_escape_the_repo_root() {
        // Scope 是指紋讀檔的路徑來源；remote 模式的工單來自 server，等於由 server
        // 指定 client 讀哪個本機檔——守門落在文法層（兩站共用同一道）。
        let store = finished_change();
        for bad in [
            "**Scope**: /etc/passwd\n",
            "**Scope**: ../../../etc/passwd\n",
            "**Scope**: C:\\Windows\\win.ini\n",
        ] {
            let Err(err) = add_round(&store, "demo", bad) else {
                panic!("scope {bad:?} must be rejected");
            };
            assert!(
                err.to_string().contains("repo-root relative"),
                "error must explain the requirement: {err}"
            );
        }
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    // --- spec「驗證工單的讀取」---

    #[test]
    fn show_round_trips_rounds_scope_findings_and_last_round() {
        // spec Scenario「讀取 JSON」的解析核心：rounds 長度、每輪 index、scope 清單
        // 與分級 findings 逐欄位；lastRound 指向末輪。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round 1");
        add_round(&store, "demo", ROUND_2).expect("round 2");
        let ticket = station::show(&STATION, &store, "demo").expect("ticket parses");
        assert_eq!(ticket.rounds.len(), 2);
        let r1 = &ticket.rounds[0];
        assert_eq!(r1.index, 1);
        assert_eq!(
            r1.scope,
            vec!["crates/a/src/lib.rs".to_string(), "crates/b/src/util.rs".to_string()]
        );
        assert_eq!(
            r1.findings,
            vec![
                Finding {
                    severity: Severity::Critical,
                    path: "crates/a/src/lib.rs".to_string(),
                    text: "requirement R2 has no implementation".to_string(),
                },
                Finding {
                    severity: Severity::Suggestion,
                    path: "crates/b/src/util.rs".to_string(),
                    text: "design says otherwise".to_string(),
                },
            ]
        );
        let last = ticket.last_round();
        assert_eq!(last.index, 2);
        assert_eq!(last.scope, vec!["crates/a/src/lib.rs".to_string()]);
        assert_eq!(last.findings.len(), 1);
        assert_eq!(last.findings[0].severity, Severity::Warning);
        assert_eq!(last.findings[0].text, "scenario 3 untested");
    }

    #[test]
    fn show_errors_when_change_has_no_ticket() {
        // spec Scenario「無工單」：錯誤說明該 change 無「驗證」工單——站別詞不得
        // 說成 review，否則使用者被送去看另一個站的工單。
        let store = finished_change();
        let err = station::show(&STATION, &store, "demo").expect_err("no ticket must error");
        assert!(err.to_string().contains("no verify ticket"), "error must say so: {err}");
        assert!(
            err.downcast_ref::<NotFound>().is_some(),
            "typed NotFound so the server maps 404"
        );
    }

    // --- spec「放棄驗證」---

    #[test]
    fn discard_deletes_the_verify_ticket_and_leaves_meta_untouched() {
        // spec Scenario「放棄既有工單」：工單刪除、`.openspec.yaml` 位元級不變。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round 1");
        discard(&store, "demo").expect("discard");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be gone");
        assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "discard must not write metadata");
    }

    #[test]
    fn discard_leaves_the_review_ticket_alone() {
        // 兩站互不遮蔽：放棄驗證不得動到審查工單（cleanup namespace 站別化的
        // 引擎面；snapshot 面由 CLI 測試覆蓋）。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("verify round");
        crate::review::add_round(&store, "demo", ROUND_1).expect("review round");
        let review_doc = store.read_artifact("demo", crate::review::REVIEW_DOC).expect("ticket");
        discard(&store, "demo").expect("discard");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "verify ticket must be gone");
        assert_eq!(
            store.read_artifact("demo", crate::review::REVIEW_DOC).as_deref(),
            Some(review_doc.as_str()),
            "the review ticket must survive byte-identically"
        );
    }

    #[test]
    fn discard_errors_when_no_ticket() {
        // spec：無工單時非零 exit code。
        let store = finished_change();
        let err = discard(&store, "demo").expect_err("no ticket must error");
        assert!(err.to_string().contains("no verify ticket"), "error must say so: {err}");
    }

    // --- spec「驗證蓋章守門與蓋章效果」---

    const CLEAN_ROUND: &str = "**Scope**: crates/a/src/lib.rs\n";
    const FILE_A: &str = "fn a() {}\n";
    const FILE_B: &str = "fn b() {}\n";
    const REPO: &[(&str, &str)] =
        &[("crates/a/src/lib.rs", FILE_A), ("crates/b/src/util.rs", FILE_B)];

    /// repo 檔案讀取替身：固定 (path, content) 表。
    fn files<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |p: &str| map.iter().find(|(k, _)| *k == p).map(|(_, v)| v.to_string())
    }

    /// repo 檔案存在替身：與 `files` 共用同一張表。
    fn present<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> bool + 'a {
        move |p: &str| map.iter().any(|(k, _)| *k == p)
    }

    fn stamp_demo(store: &TestStore, accept: bool) -> Result<()> {
        stamp(
            store,
            "demo",
            accept,
            Some("Ver <v@example.com>"),
            Some("claude"),
            &files(REPO),
            &present(REPO),
        )
    }

    #[test]
    fn stamp_refuses_unresolved_findings_without_accept() {
        // spec Scenario「末輪有未解 findings 且未帶 --accept」：拒絕並提示 --accept
        // 或先修正重驗；站別祈使詞為 re-verify（審查站是 re-review）。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round with findings");
        let err = stamp_demo(&store, false).expect_err("unresolved findings must refuse");
        let msg = err.to_string();
        assert!(msg.contains("--accept"), "error must offer --accept: {msg}");
        assert!(msg.contains("re-verify"), "station word must be the verify one: {msg}");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
    }

    #[test]
    fn stamp_refuses_when_tasks_regress_after_the_ticket_was_opened() {
        // 守門 (1) 與審查站同一條：工單開立後任務被退勾 → 4/5 拒絕，訊息用
        // 「verify stamp」而非另一站的動詞。
        let store = finished_change();
        add_round(&store, "demo", CLEAN_ROUND).expect("round 1");
        store.put_artifact("demo", "tasks.md", TASKS_4_OF_5);
        let err = stamp_demo(&store, false).expect_err("incomplete tasks must refuse");
        let msg = err.to_string();
        assert!(msg.contains("4/5"), "error must show the count: {msg}");
        assert!(msg.contains("verify stamp"), "error must name the verb: {msg}");
        assert_eq!(store.meta("demo"), META, "metadata must stay byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
    }

    #[test]
    fn stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total() {
        // spec Scenario「僅餘手動任務可蓋章」：寫碼全勾、[M] 未勾 → 蓋章成功，
        // verified_tasks_total 記全任務總數（含 [M]）。
        let store = finished_change();
        add_round(&store, "demo", CLEAN_ROUND).expect("round 1");
        store.put_artifact("demo", "tasks.md", TASKS_CODE_DONE_MANUAL_OPEN);
        stamp_demo(&store, false).expect("manual-only remainder must stamp");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta");
        assert_eq!(meta.verified_tasks_total, Some(5), "anchor counts the [M] task too");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
    }

    #[test]
    fn stamp_clean_round_writes_five_fields_and_deletes_the_ticket() {
        // spec Scenario「乾淨蓋章」：五個 verified 欄位齊備、`verify.md` 不存在。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round 1 with findings");
        add_round(&store, "demo", CLEAN_ROUND).expect("round 2 clean");
        stamp_demo(&store, false).expect("clean stamp");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
        let raw = store.meta("demo");
        let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta parses");
        assert_eq!(meta.verified_at.as_deref(), Some(crate::util::today().as_str()));
        assert_eq!(meta.verified_by.as_deref(), Some("Ver <v@example.com>"));
        assert_eq!(meta.verified_with.as_deref(), Some("claude"));
        assert_eq!(meta.verified_tasks_total, Some(5));
        assert!(!meta.verified_scope.is_empty(), "scope fingerprints must be recorded");
        assert!(raw.starts_with(META), "existing fields preserved byte-for-byte: {raw}");
    }

    #[test]
    fn stamp_writes_the_task_anchor_from_the_ticket_moment() {
        // spec Example「蓋章寫入的任務錨」：8 個任務、7 個寫碼任務全勾、1 個
        // `[M]` 未勾 → `verified_tasks_total` 為 8（未勾的 [M] 也計入錨）。
        let store = TestStore::with_meta("demo", META);
        let tasks: String = (1..=7).map(|i| format!("- [x] {i} t\n")).collect::<String>()
            + "- [ ] [M] 8 hand check\n";
        store.put_artifact("demo", "tasks.md", &tasks);
        add_round(&store, "demo", CLEAN_ROUND).expect("clean round 1");
        stamp_demo(&store, false).expect("stamp");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
        assert_eq!(meta.verified_tasks_total, Some(8));
    }

    #[test]
    fn stamp_with_accept_overrides_findings_and_stamps() {
        // spec「`--accept` SHALL 僅豁免後者」：帶保留蓋章 → 章寫入且工單刪除。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round with findings");
        stamp_demo(&store, true).expect("--accept must stamp");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
        assert!(meta.verified_at.is_some());
    }

    #[test]
    fn stamp_allows_a_suggestion_only_last_round() {
        // spec Scenario「僅 SUGGESTION 的末輪乾淨蓋章」：SUGGESTION 不是必修，
        // 無 `--accept` 也放行——五欄寫入且工單刪除。分界計數本身歸
        // station.rs 的守門測試；這裡釘站別 wiring 與蓋章效果。
        let store = finished_change();
        add_round(&store, "demo", SUGGESTION_ROUND).expect("suggestion-only round");
        stamp_demo(&store, false).expect("suggestion-only round must stamp clean");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "ticket must be deleted");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
        assert_eq!(meta.verified_at.as_deref(), Some(crate::util::today().as_str()));
        assert_eq!(meta.verified_by.as_deref(), Some("Ver <v@example.com>"));
        assert_eq!(meta.verified_with.as_deref(), Some("claude"));
        assert_eq!(meta.verified_tasks_total, Some(5));
        assert!(!meta.verified_scope.is_empty(), "scope fingerprints must be recorded");
    }

    #[test]
    fn stamp_leaves_the_review_station_untouched() {
        // 兩站互不遮蔽：驗證蓋章不得動到審查工單或 `reviewed_*` 欄位。
        const STAMPED: &str = "schema: spec-driven\nreviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 5\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: dead\n";
        let store = TestStore::with_meta("demo", STAMPED);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        crate::review::add_round(&store, "demo", CLEAN_ROUND).expect("review round");
        add_round(&store, "demo", CLEAN_ROUND).expect("verify round");
        stamp_demo(&store, false).expect("verify stamp");
        assert!(!store.artifact_exists("demo", VERIFY_DOC), "verify ticket must be gone");
        assert!(
            store.artifact_exists("demo", crate::review::REVIEW_DOC),
            "the review ticket must survive a verify stamp"
        );
        let raw = store.meta("demo");
        let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta parses");
        assert_eq!(meta.reviewed_at.as_deref(), Some("2026-08-01"), "review stamp intact: {raw}");
        assert_eq!(meta.reviewed_scope.len(), 1, "review anchors intact: {raw}");
        assert_eq!(meta.reviewed_scope[0].hash, "dead");
        assert!(meta.verified_at.is_some(), "verify stamp landed: {raw}");
    }

    #[test]
    fn stamp_scope_is_the_sorted_union_of_all_rounds() {
        // spec「驗證指紋錨與失效判定」：指紋範圍＝工單各輪 Scope 聯集（去重、排序）。
        let store = finished_change();
        add_round(&store, "demo", ROUND_1).expect("round 1: a + b");
        add_round(&store, "demo", CLEAN_ROUND).expect("round 2: a only");
        stamp_demo(&store, false).expect("stamp");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
        let paths: Vec<&str> = meta.verified_scope.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["crates/a/src/lib.rs", "crates/b/src/util.rs"]);
        assert_eq!(meta.verified_scope[0].hash, content_fingerprint(FILE_A));
        assert_eq!(meta.verified_scope[1].hash, content_fingerprint(FILE_B));
    }

    #[test]
    fn restamp_replaces_verified_fields_without_duplication() {
        // 重驗後重蓋：五欄位原位更新（含多行 verified_scope 區塊），不留重複鍵，
        // 其餘欄位（含審查章）逐位元組保留。
        let old = "schema: spec-driven\ncreated: 2026-07-01\nverified_at: 2026-07-10\nverified_by: Old <o@example.com>\nverified_with: codex\nverified_tasks_total: 3\nverified_scope:\n  - path: old/file.rs\n    hash: deadbeef\nboard_rank: n\n";
        let store = TestStore::with_meta("demo", old);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", CLEAN_ROUND).expect("round");
        stamp_demo(&store, false).expect("re-stamp");
        let raw = store.meta("demo");
        assert_eq!(raw.matches("verified_at:").count(), 1, "no duplicate keys: {raw}");
        assert_eq!(raw.matches("verified_scope:").count(), 1, "no duplicate keys: {raw}");
        assert!(!raw.contains("old/file.rs"), "stale scope block must be gone: {raw}");
        assert!(raw.contains("board_rank: n\n"), "unrelated fields survive: {raw}");
        let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("re-stamped meta parses");
        assert_eq!(meta.verified_tasks_total, Some(5));
        assert_eq!(meta.verified_scope.len(), 1);
        assert_eq!(meta.verified_scope[0].path, "crates/a/src/lib.rs");
    }

    #[test]
    fn stamp_normalizes_backslash_paths_into_meta() {
        // spec「路徑正規化…與審查站位元級同構」：Windows 路徑 `\` → `/` 後入章，
        // 否則同一檔在兩個平台指紋成兩筆、章永遠 stale。
        let store = finished_change();
        add_round(&store, "demo", "**Scope**: crates\\a\\src\\lib.rs\n").expect("round");
        stamp_demo(&store, false).expect("stamp");
        let meta = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).expect("parses");
        assert_eq!(meta.verified_scope[0].path, "crates/a/src/lib.rs");
    }

    #[test]
    fn stamp_quotes_scalars_so_yaml_metacharacters_survive() {
        // 身分／工具／path 以未引號純量直寫會注入欄位或炸掉整份 meta——而工單已在
        // 同一步刪除，無從回復。寫入端負責引號（與審查站同一道 yaml_scalar）。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&store, "demo", "**Scope**: src/@odd #1.rs\n").expect("round");
        let repo: &[(&str, &str)] = &[("src/@odd #1.rs", FILE_A)];
        stamp(
            &store,
            "demo",
            false,
            Some("Ver: the #1 <v@example.com>\nboard_rank: injected"),
            Some("codex: cli"),
            &files(repo),
            &present(repo),
        )
        .expect("stamp");
        let raw = store.meta("demo");
        let meta = crate::model::ChangeMeta::from_text(Some(&raw)).expect("meta must parse: {raw}");
        assert_eq!(meta.verified_scope[0].path, "src/@odd #1.rs", "path round-trips: {raw}");
        assert_eq!(meta.verified_with.as_deref(), Some("codex: cli"), "{raw}");
        assert!(!raw.contains("\nboard_rank: injected"), "no field injection: {raw}");
    }

    #[test]
    fn stamp_refuses_on_corrupt_meta_and_without_a_ticket() {
        // 沿 set_board_rank 的 fail-closed gate：壞 metadata 不得被疊寫；無工單
        // 不可蓋章。
        let store = finished_change();
        let err = stamp_demo(&store, false).expect_err("no ticket must refuse");
        assert!(err.to_string().contains("no verify ticket"), "{err}");

        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let corrupt = TestStore::with_meta("demo", BAD);
        corrupt.put_artifact("demo", "tasks.md", TASKS_5_DONE);
        add_round(&corrupt, "demo", CLEAN_ROUND).expect("round");
        let err = stamp_demo(&corrupt, false).expect_err("corrupt meta must refuse");
        assert!(
            err.to_string().contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(corrupt.meta("demo"), BAD);
        assert_eq!(*corrupt.meta_writes.borrow(), 0);
        assert!(corrupt.artifact_exists("demo", VERIFY_DOC), "ticket must survive refusal");
    }

    // --- spec「驗證指紋錨與失效判定」---

    /// 帶完整驗證章的 meta：tasks_total 任務錨＋entries 內容錨。
    fn stamped_meta(tasks_total: usize, entries: &[(&str, &str)]) -> crate::model::ChangeMeta {
        let mut y = format!(
            "schema: spec-driven\nverified_at: 2026-08-01\nverified_by: Ver <v@example.com>\nverified_with: claude\nverified_tasks_total: {tasks_total}\nverified_scope:\n"
        );
        for (p, h) in entries {
            y.push_str(&format!("  - path: {p}\n    hash: {h}\n"));
        }
        crate::model::ChangeMeta::from_text(Some(&y)).expect("meta parses")
    }

    #[test]
    fn freshness_matches_the_review_station_bit_for_bit() {
        // spec「路徑正規化與行尾 CRLF→LF 後 SHA-256 規則 SHALL 與審查站位元級
        // 同構（共用同一實作）」：同一組錨與現值，兩站判定必須逐一相同。
        let h = content_fingerprint(FILE_A);
        let path = "crates/a/src/lib.rs";
        let grown = format!("{FILE_A}fn extra() {{}}\n");
        let crlf = FILE_A.replace('\n', "\r\n");
        let cases: &[(&str, &[(&str, &str)], usize, usize, Freshness)] = &[
            ("all anchors match", REPO, 5, 5, Freshness::Fresh),
            ("scope file modified", &[(path, "")], 5, 5, Freshness::Stale),
            ("scope file gone", &[], 5, 5, Freshness::Stale),
            ("task count grew", REPO, 6, 6, Freshness::Stale),
            ("task unchecked", REPO, 5, 4, Freshness::Stale),
        ];
        for (label, repo, total, complete, want) in cases {
            let repo: Vec<(&str, &str)> = repo
                .iter()
                .map(|(p, c)| (*p, if *p == path && c.is_empty() { grown.as_str() } else { *c }))
                .collect();
            let verified = stamped_meta(5, &[(path, h.as_str())]);
            assert_eq!(
                freshness(&verified, &code_counts(*total, *complete), &files(&repo)),
                *want,
                "verify station, case: {label}"
            );
            let reviewed = crate::model::ChangeMeta {
                reviewed_at: verified.verified_at.clone(),
                reviewed_tasks_total: verified.verified_tasks_total,
                reviewed_scope: verified.verified_scope.clone(),
                ..Default::default()
            };
            assert_eq!(
                crate::review::freshness(&reviewed, &code_counts(*total, *complete), &files(&repo)),
                *want,
                "review station must agree, case: {label}"
            );
        }
        // spec Scenario「行尾差異不觸發失效」：LF → CRLF 仍 fresh。
        let now = [(path, crlf.as_str())];
        let meta = stamped_meta(5, &[(path, h.as_str())]);
        assert_eq!(freshness(&meta, &code_counts(5, 5), &files(&now)), Freshness::Fresh);

        // spec Scenario「蓋章後補勾手動任務不失效」：勾 [M] 前後皆 fresh。
        let counts = |md: &str| crate::tasks::counts(&crate::tasks::parse(md));
        let open = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] [M] 手測\n";
        let checked = "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] [M] 手測\n";
        assert_eq!(freshness(&meta, &counts(open), &files(REPO)), Freshness::Fresh, "手測未勾");
        assert_eq!(freshness(&meta, &counts(checked), &files(REPO)), Freshness::Fresh, "手測補勾");
    }

    #[test]
    fn freshness_reads_absence_as_unverified() {
        // 缺席讀作未驗證：無驗證章的 meta 沒有可判定的錨，即使審查章齊備。
        const REVIEWED_ONLY: &str = "schema: spec-driven\nreviewed_at: 2026-08-01\nreviewed_tasks_total: 5\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: dead\n";
        let meta = crate::model::ChangeMeta::from_text(Some(REVIEWED_ONLY)).expect("parses");
        assert_eq!(freshness(&meta, &code_counts(5, 5), &files(REPO)), Freshness::Unknown);
    }
}

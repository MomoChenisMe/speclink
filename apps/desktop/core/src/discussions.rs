//! 討論橋接：看板討論欄與已封存頁討論節的查詢，以及促轉／歸檔兩個推進動詞。
//!
//! 消費 speclink-core 的 discuss 模組（促轉流程與 promoted_to 查詢皆為 core 單一
//! 真相，見 design D1/D2）；本層只做 root 定址、路徑參數防護與 camelCase 組裝。

use std::path::Path;

use serde_json::{json, Value};
use speclink_core::discuss::{self, DiscussionInfo};
use speclink_core::store::Store;

use crate::init_core_context;
use crate::query::is_safe_path_param;
use crate::verbs::open;

fn entry(store: &dyn Store, info: &DiscussionInfo) -> Value {
    let mut v = json!({
        "slug": info.slug,
        "topic": info.topic,
        "status": info.status,
        "rounds": info.rounds,
        "created": info.created,
        "promotedTo": discuss::promoted_to(store, &info.slug),
    });
    // 建立者（createdBy，camelCase）——缺席時省略該鍵（比照 change 的 fromDiscussions 樣式）。
    if let Some(cb) = &info.created_by {
        v["createdBy"] = json!(cb);
    }
    v
}

/// 討論清單：`{ "active": [...], "archived": [...] }`，項含 slug／topic／status／
/// rounds／created／promotedTo（camelCase）。非 speclink 專案回兩個空清單。
pub fn list_discussions_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "active": [], "archived": [] });
    };
    let store: &dyn Store = &ctx.store;
    let active: Vec<Value> =
        board_sorted_active(store).iter().map(|(_, i)| entry(store, i)).collect();
    let archived: Vec<Value> =
        discuss::list_archived(store).iter().map(|i| entry(store, i)).collect();
    json!({ "active": active, "archived": archived })
}

/// 看板顯示序的 active 討論清單（design D2）：slug 序當回退，穩定排序疊上
/// board_rank 複合鍵——缺值置頂維持 slug 序、具值依 rank 升冪、同值以 slug 決斷。
/// rank 經獨立讀取函式取得（不進 DiscussionInfo），CLI `discuss list --json`
/// 逐位元不變。
pub(crate) fn board_sorted_active(store: &dyn Store) -> Vec<(Option<String>, DiscussionInfo)> {
    let mut ranked: Vec<(Option<String>, DiscussionInfo)> = discuss::list_discussions(store)
        .into_iter()
        .map(|i| (discuss::board_rank(store, &i.slug), i))
        .collect();
    ranked.sort_by(|(ra, a), (rb, b)| match (ra, rb) {
        (None, None) => std::cmp::Ordering::Equal, // 穩定排序保留 slug 回退序
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(x), Some(y)) => x.cmp(y).then_with(|| a.slug.cmp(&b.slug)),
    });
    ranked
}

/// 讀取討論記錄全文（slug 定址；live 優先、封存為後備——同 CLI `discuss show`）。
/// 不存在回 `None`；含路徑穿越的 slug 一律拒絕。
pub fn discussion_document_at(root: &Path, slug: &str) -> Option<String> {
    if !is_safe_path_param(slug) {
        return None;
    }
    let ctx = init_core_context(root)?;
    discuss::show_discussion(&ctx.store, slug)
}

/// 促轉討論為新 change（可選 change 名，省略時由 slug 衍生）。成功回
/// `{ "change": ..., "path": ... }`；失敗回單行訊息（同名 change、討論已封存等）。
pub fn promote_discussion_at(root: &Path, slug: &str, name: Option<&str>) -> Result<Value, String> {
    if !is_safe_path_param(slug) {
        return Err(format!("invalid discussion slug: {slug}"));
    }
    let ctx = open(root)?;
    let outcome = discuss::promote(&ctx.workspace, &ctx.store, slug, name)
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "change": outcome.change,
        "path": speclink_core::util::to_slash(&outcome.path),
    }))
}

/// 歸檔一筆 live 討論。成功回 `{ "archivedTo": "discussions/archive/<file>" }`；
/// 無此 live 討論回 `Err`。
pub fn archive_discussion_at(root: &Path, slug: &str) -> Result<Value, String> {
    if !is_safe_path_param(slug) {
        return Err(format!("invalid discussion slug: {slug}"));
    }
    let ctx = open(root)?;
    match discuss::archive_discussion(&ctx.store, slug).map_err(|e| e.to_string())? {
        Some(file) => Ok(json!({ "archivedTo": format!("discussions/archive/{file}") })),
        None => Err(format!("discussion '{slug}' not found")),
    }
}

#[cfg(test)]
mod tests {
    use crate::testfixture::FixtureRoot;

    /// 鷹架版討論記錄；`extra_fm` 插在 status 行之後（如 promoted_to）。
    fn discussion_doc(slug: &str, topic: &str, status: &str, extra_fm: &str, rounds: usize, conclusion: &str) -> String {
        let round_entries: String = (1..=rounds)
            .map(|n| format!("### Round {n} — assumptions (2026-01-02)\n\n**Focus**: scope\n\n"))
            .collect();
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: {status}\n{extra_fm}created: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n{round_entries}\
             ## Conclusion\n\n{conclusion}\n"
        )
    }

    fn fx_with_discussions(tag: &str) -> FixtureRoot {
        let fx = FixtureRoot::new(tag);
        fx.write(
            "openspec/discussions/alpha-search.md",
            &discussion_doc("alpha-search", "Alpha search", "concluded", "", 1, "**Decision**: build alpha search"),
        );
        fx.write(
            "openspec/discussions/beta-open.md",
            &discussion_doc("beta-open", "Beta open", "open", "", 2, "<!-- placeholder -->"),
        );
        fx.write(
            "openspec/discussions/gamma-promoted.md",
            &discussion_doc("gamma-promoted", "Gamma promoted", "promoted", "promoted_to: cut-a, cut-b\n", 3, "**Decision**: split"),
        );
        fx.write(
            "openspec/discussions/archive/2026-01-03-old-topic.md",
            &discussion_doc("old-topic", "Old topic", "promoted", "promoted_to: first-cut\n", 1, "**Decision**: done"),
        );
        fx
    }

    // --- 討論清單（active＋archived，camelCase） ---

    #[test]
    fn list_discussions_returns_active_and_archived_camel_case() {
        let fx = fx_with_discussions("d-list");
        let v = super::list_discussions_at(fx.root());
        let active = v["active"].as_array().expect("active array");
        let archived = v["archived"].as_array().expect("archived array");
        assert_eq!(active.len(), 3, "active: {active:?}");
        assert_eq!(archived.len(), 1, "archived: {archived:?}");

        // slug 排序：alpha-search, beta-open, gamma-promoted。
        assert_eq!(active[0]["slug"], "alpha-search");
        assert_eq!(active[0]["topic"], "Alpha search");
        assert_eq!(active[0]["status"], "concluded");
        assert_eq!(active[0]["rounds"], 1);
        assert_eq!(active[0]["created"], "2026-01-02");
        assert_eq!(active[0]["promotedTo"].as_array().unwrap().len(), 0);
        assert!(active[0].get("promoted_to").is_none(), "camelCase only");

        assert_eq!(active[1]["slug"], "beta-open");
        assert_eq!(active[1]["status"], "open");
        assert_eq!(active[1]["rounds"], 2);

        // promoted 討論帶 promotedTo 累積值。
        assert_eq!(active[2]["slug"], "gamma-promoted");
        assert_eq!(active[2]["status"], "promoted");
        assert_eq!(
            active[2]["promotedTo"],
            serde_json::json!(["cut-a", "cut-b"])
        );

        // 封存節：日期＋topic 可得，promotedTo 照帶。
        assert_eq!(archived[0]["slug"], "old-topic");
        assert_eq!(archived[0]["topic"], "Old topic");
        assert_eq!(archived[0]["promotedTo"], serde_json::json!(["first-cut"]));
    }

    #[test]
    fn list_discussions_exposes_created_by_camel_case_when_present() {
        let fx = FixtureRoot::new("d-createdby");
        fx.write(
            "openspec/discussions/with-author.md",
            &discussion_doc("with-author", "With author", "open", "created_by: Base Line <base@example.com>\n", 1, "<!-- x -->"),
        );
        fx.write(
            "openspec/discussions/no-author.md",
            &discussion_doc("no-author", "No author", "open", "", 1, "<!-- x -->"),
        );
        let v = super::list_discussions_at(fx.root());
        let active = v["active"].as_array().expect("active array");
        let by = |slug: &str| active.iter().find(|d| d["slug"] == slug).unwrap();
        // 有 created_by → createdBy（camelCase）帶值、snake_case 不外洩。
        assert_eq!(by("with-author")["createdBy"], "Base Line <base@example.com>");
        assert!(by("with-author").get("created_by").is_none(), "camelCase only");
        // 無 created_by → 省略該鍵。
        assert!(by("no-author").get("createdBy").is_none(), "omit when absent");
    }

    #[test]
    fn list_discussions_sorts_active_by_board_rank_with_unranked_on_top() {
        // spec「看板卡片順序以 board_rank 欄位為真相」討論側＋ design D2：
        // 缺值卡置頂維持回退序（slug 升冪），具值卡依 rank 升冪；同值以 slug 決斷。
        let fx = FixtureRoot::new("d-rank-sort");
        fx.write(
            "openspec/discussions/delta.md",
            &discussion_doc("delta", "Delta", "open", "board_rank: b\n", 0, "<!-- placeholder -->"),
        );
        fx.write(
            "openspec/discussions/charlie.md",
            &discussion_doc("charlie", "Charlie", "open", "board_rank: n\n", 0, "<!-- placeholder -->"),
        );
        fx.write(
            "openspec/discussions/echo.md",
            &discussion_doc("echo", "Echo", "open", "board_rank: n\n", 0, "<!-- placeholder -->"),
        );
        fx.write(
            "openspec/discussions/beta.md",
            &discussion_doc("beta", "Beta", "open", "", 0, "<!-- placeholder -->"),
        );
        fx.write(
            "openspec/discussions/alpha.md",
            &discussion_doc("alpha", "Alpha", "open", "", 0, "<!-- placeholder -->"),
        );
        let v = super::list_discussions_at(fx.root());
        let slugs: Vec<String> = v["active"]
            .as_array()
            .expect("active array")
            .iter()
            .map(|d| d["slug"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            slugs,
            ["alpha", "beta", "delta", "charlie", "echo"],
            "unranked (slug order) on top, then rank asc (delta=b before charlie=n) with slug tiebreak (charlie before echo at n)"
        );
    }

    #[test]
    fn list_discussions_on_non_project_root_is_empty() {
        let dir = std::env::temp_dir().join(format!("speclink-dtcore-dlist-none-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let v = super::list_discussions_at(&dir);
        assert_eq!(v["active"].as_array().unwrap().len(), 0);
        assert_eq!(v["archived"].as_array().unwrap().len(), 0);
    }

    // --- 記錄全文讀取（slug 定址、穿越拒絕） ---

    #[test]
    fn discussion_document_reads_live_and_archived_by_slug() {
        let fx = fx_with_discussions("d-doc");
        let live = super::discussion_document_at(fx.root(), "alpha-search").expect("live doc");
        assert!(live.contains("**Decision**: build alpha search"), "live: {live}");
        let archived = super::discussion_document_at(fx.root(), "old-topic").expect("archived fallback");
        assert!(archived.contains("topic: Old topic"), "archived: {archived}");
        assert!(super::discussion_document_at(fx.root(), "no-such-slug").is_none());
    }

    #[test]
    fn discussion_document_rejects_path_traversal() {
        let fx = fx_with_discussions("d-trav");
        for bad in ["../alpha-search", "a/../b", "..", "/etc/passwd", "C:\\x", ""] {
            assert!(
                super::discussion_document_at(fx.root(), bad).is_none(),
                "traversal param must be rejected: {bad}"
            );
        }
    }

    // --- 促轉橋接（端到端建出 change） ---

    #[test]
    fn promote_builds_change_end_to_end() {
        let fx = fx_with_discussions("d-promote");
        let v = super::promote_discussion_at(fx.root(), "alpha-search", None).expect("promote ok");
        assert_eq!(v["change"], "alpha-search");

        let meta = std::fs::read_to_string(
            fx.root().join("openspec/changes/alpha-search/.openspec.yaml"),
        )
        .expect("change meta exists");
        assert!(meta.contains("from_discussion: alpha-search\n"), "meta: {meta}");

        let proposal = std::fs::read_to_string(
            fx.root().join("openspec/changes/alpha-search/proposal.md"),
        )
        .expect("proposal exists");
        assert!(proposal.starts_with("## Why\n\n**Decision**: build alpha search\n"), "proposal: {proposal}");

        let doc = std::fs::read_to_string(fx.root().join("openspec/discussions/alpha-search.md")).unwrap();
        assert!(doc.contains("status: promoted\n"), "doc: {doc}");
        assert!(doc.contains("promoted_to: alpha-search\n"), "doc: {doc}");
    }

    #[test]
    fn promote_missing_or_duplicate_errors_with_message() {
        let fx = fx_with_discussions("d-promote-err");
        let err = super::promote_discussion_at(fx.root(), "no-such-slug", None).unwrap_err();
        assert!(err.contains("not found"), "err: {err}");

        // 同名 change 已存在 → Err 單行訊息，討論不被標記。
        fx.add_change("alpha-search", "schema: spec-driven\ncreated: 2026-07-01\n");
        let err = super::promote_discussion_at(fx.root(), "alpha-search", None).unwrap_err();
        assert!(err.contains("already exists"), "err: {err}");
        let doc = std::fs::read_to_string(fx.root().join("openspec/discussions/alpha-search.md")).unwrap();
        assert!(doc.contains("status: concluded\n"), "discussion must stay concluded: {doc}");
    }

    #[test]
    fn promote_with_explicit_name_creates_that_change() {
        let fx = fx_with_discussions("d-promote-name");
        let v = super::promote_discussion_at(fx.root(), "gamma-promoted", Some("cut-c")).expect("re-promote ok");
        assert_eq!(v["change"], "cut-c");
        let doc = std::fs::read_to_string(fx.root().join("openspec/discussions/gamma-promoted.md")).unwrap();
        assert!(doc.contains("promoted_to: cut-a, cut-b, cut-c\n"), "doc: {doc}");
    }

    // --- 歸檔橋接（記錄移入 discussions/archive/） ---

    #[test]
    fn archive_moves_record_into_archive_dir() {
        let fx = fx_with_discussions("d-archive");
        let v = super::archive_discussion_at(fx.root(), "alpha-search").expect("archive ok");
        assert_eq!(v["archivedTo"], "discussions/archive/2026-01-02-alpha-search.md");
        assert!(!fx.root().join("openspec/discussions/alpha-search.md").exists());
        assert!(fx
            .root()
            .join("openspec/discussions/archive/2026-01-02-alpha-search.md")
            .exists());
    }

    #[test]
    fn archive_missing_slug_errors() {
        let fx = fx_with_discussions("d-archive-err");
        let err = super::archive_discussion_at(fx.root(), "no-such-slug").unwrap_err();
        assert!(err.contains("not found"), "err: {err}");
    }

    #[test]
    fn promote_and_archive_reject_path_traversal_slugs() {
        // sharp-edges：寫入動詞的 slug 與讀取動詞同等防護——穿越參數不得
        // 觸及 discussions 目錄外的任何路徑（mark_promoted／archive 會寫檔）。
        let fx = fx_with_discussions("d-verb-trav");
        for bad in ["../alpha-search", "a/../b", "..", "/etc/passwd", "C:\\x", ""] {
            let err = super::promote_discussion_at(fx.root(), bad, None).unwrap_err();
            assert!(err.contains("invalid"), "promote must reject '{bad}': {err}");
            let err = super::archive_discussion_at(fx.root(), bad).unwrap_err();
            assert!(err.contains("invalid"), "archive must reject '{bad}': {err}");
        }
        // 防護不誤傷合法 slug：正常促轉仍可用。
        assert!(super::promote_discussion_at(fx.root(), "alpha-search", None).is_ok());
    }
}

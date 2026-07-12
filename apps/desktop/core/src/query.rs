//! 唯讀查詢：桌面 command 的資料來源。
//!
//! 每個函式吃專案 `root`，經內嵌 core 回傳與對應 CLI `--json` 同形狀的 payload。
//! 非 speclink 專案目錄時回傳明確的空狀態（空清單／None），不 panic。

use std::path::Path;

use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// 對應 `speclink list --json`：`{ "changes": [ … ] }`。非專案回傳 `{ "changes": [] }`。
pub fn list_changes_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "changes": [] });
    };
    let store: &dyn Store = &ctx.store;
    let changes = board_sorted_changes(store);
    // 桌面 payload 在 CLI 同形項上疊加生命週期標記欄位（parity 紅線：CLI 的
    // changes_json 本身不動）。meta 類欄位取自 list_changes 已解析的 meta；
    // whyExcerpt 例外——另讀各 change 的 proposal.md 首段（描述列資料源）。
    let items: Vec<Value> = speclink_core::listing::changes_json(store, &changes)
        .iter()
        .zip(changes.iter())
        .map(|(item, c)| {
            let mut v = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
            v["startedAt"] = json!(c.meta.started_at);
            v["startedBy"] = json!(c.meta.started_by);
            v["startedWith"] = json!(c.meta.started_with);
            v["createdBy"] = json!(c.meta.created_by);
            v["created"] = json!(c.meta.created);
            v["fromDiscussions"] = json!(c.meta.from_discussions());
            // 「待重新反映」徽章的資料源：恆存在（空陣列＝無旗標），供看板卡片渲染。
            v["restaleFrom"] = json!(c.meta.restale_from());
            // 變更卡描述列的資料源：恆存在 key（缺件為 null），前端對 null 隱藏描述列。
            v["whyExcerpt"] =
                json!(store.read_artifact(&c.name, "proposal.md").as_deref().and_then(why_excerpt));
            v
        })
        .collect();
    json!({ "changes": items })
}

/// 看板顯示序的變更清單（design D2）：先取 CLI 預設 modified 序當回退，再以穩定
/// 排序疊上 board_rank 複合鍵——缺值置頂維持回退序、具值依 rank 升冪、同值以
/// 名稱決斷。CLI 的 `speclink list --json` 排序不經此路徑，逐位元不變。
pub(crate) fn board_sorted_changes(store: &dyn Store) -> Vec<speclink_core::model::Change> {
    let mut changes = speclink_core::model::list_changes(store);
    speclink_core::listing::sort_changes(store, &mut changes, "modified");
    changes.sort_by(|x, y| match (&x.meta.board_rank, &y.meta.board_rank) {
        (None, None) => std::cmp::Ordering::Equal, // 穩定排序保留回退序
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b).then_with(|| x.name.cmp(&y.name)),
    });
    changes
}

/// 對應 `speclink list --specs --json`：`{ "specs": … }`。非專案回傳 `{ "specs": [] }`。
pub fn list_specs_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "specs": [] });
    };
    let store: &dyn Store = &ctx.store;
    // 桌面 payload 在 CLI 同形項上疊加呈現層輔助欄位（design D2、spec-archive-drawer
    // design D4；parity 紅線：CLI 的 specs_json_items 本身不動）。mtime 不可得時不插
    // key；規格卡欄位恆存在（不可讀／缺席檔案為 0／null，清單照常回傳）。
    let mut specs = speclink_core::listing::specs_json_items(store);
    if let Value::Array(items) = &mut specs {
        for item in items {
            let Some(id) = item["id"].as_str().map(str::to_string) else { continue };
            if let Some(date) = modified_date(&store.canonical_spec_path(&id)) {
                item["modifiedAt"] = json!(date);
            }
            let doc = store.read_canonical_spec(&id);
            let purpose = doc.as_deref().and_then(purpose_excerpt);
            item["requirementCount"] = json!(doc.as_deref().map_or(0, requirement_count));
            item["purposeTbd"] =
                json!(purpose.as_deref().is_some_and(|p| p.starts_with(PURPOSE_TBD_PREFIX)));
            item["purposeExcerpt"] = json!(purpose);
            item["traceCount"] = json!(doc.as_deref().map_or(0, trace_count));
        }
    }
    json!({ "specs": specs })
}

/// archive 產生新正典 spec 時的 Purpose 佔位文案前綴（speclink-core archive.rs）；
/// 偵測一致性由 list_specs_purpose_tbd_flags_archive_placeholder 以真實 archive 釘住。
const PURPOSE_TBD_PREFIX: &str = "TBD - created by archiving";

/// 正典 spec 的 `### Requirement:` 標題數。
fn requirement_count(doc: &str) -> usize {
    doc.lines().filter(|l| l.trim().starts_with("### Requirement:")).count()
}

/// 正典 spec `## Purpose` 區段首個非空行原文；區段缺席或無內容時 `None`。
fn purpose_excerpt(doc: &str) -> Option<String> {
    let mut in_purpose = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_purpose = heading.trim() == "Purpose";
            continue;
        }
        if in_purpose && !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// 全文 `@trace` 註解區塊內 `source:` 值的去重數（同一變更多次溯源計一次）。
fn trace_count(doc: &str) -> usize {
    let mut sources = std::collections::HashSet::new();
    let mut in_trace = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<!-- @trace") {
            in_trace = true;
            continue;
        }
        if in_trace {
            if let Some(src) = trimmed.strip_prefix("source:") {
                sources.insert(src.trim().to_string());
            }
            if trimmed.ends_with("-->") {
                in_trace = false;
            }
        }
    }
    sources.len()
}

/// proposal.md 的 `## Why` 區段首個非空行原文（board-card-anatomy design D2）；
/// 區段缺席或區段內無內容時 `None`。
pub(crate) fn why_excerpt(doc: &str) -> Option<String> {
    let mut in_why = false;
    for line in doc.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("## ") {
            in_why = heading.trim() == "Why";
            continue;
        }
        if in_why && !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// 檔案 mtime 衍生的本地日期（YYYY-MM-DD）；metadata 或 mtime 不可得時 `None`。
pub(crate) fn modified_date(path: &Path) -> Option<String> {
    let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(chrono::DateTime::<chrono::Local>::from(mtime).format("%Y-%m-%d").to_string())
}

/// 對應 `speclink status --change <name> --json`（`StatusReport` 序列化）。
/// change 不存在或非專案時回傳 `Err` 附訊息。
pub fn status_at(root: &Path, change: &str) -> Result<Value, String> {
    let ctx = init_core_context(root)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let store: &dyn Store = &ctx.store;
    let change = store
        .find_change(change)
        .ok_or_else(|| format!("change not found: {change}"))?;
    let schema = resolve_schema(&ctx.workspace, &change.meta.schema_name())?;
    let report = speclink_core::status::build(store, &change, &schema);
    serde_json::to_value(&report).map_err(|e| e.to_string())
}

/// 讀取一個 change 的 artifact markdown。`artifact` 為 output path（如 `proposal.md`、
/// `design.md`、`specs/<cap>/spec.md`，同 status 的 `outputPath`）。無則 `None`。
///
/// 邊界防護：`change`／`artifact` 含路徑穿越（`..`）或絕對路徑時視為無效回傳 `None`，
/// 不讓拼接後的路徑逃出 change 目錄讀取任意檔案。
pub fn document_at(root: &Path, change: &str, artifact: &str) -> Option<String> {
    if !is_safe_path_param(change) || !is_safe_path_param(artifact) {
        return None;
    }
    let ctx = init_core_context(root)?;
    ctx.store.read_artifact(change, artifact)
}

/// 讀取一個 capability 的正典 spec.md。無則 `None`。含路徑穿越的 `capability` 回傳 `None`。
pub fn spec_document_at(root: &Path, capability: &str) -> Option<String> {
    if !is_safe_path_param(capability) {
        return None;
    }
    let ctx = init_core_context(root)?;
    ctx.store.read_canonical_spec(capability)
}

/// 列出一個 change 的 delta capability 名（供規格分頁載入 specs/<cap>/spec.md）。
pub fn change_capabilities_at(root: &Path, change: &str) -> Vec<String> {
    if !is_safe_path_param(change) {
        return Vec::new();
    }
    match init_core_context(root) {
        Some(ctx) => ctx.store.delta_capabilities(change),
        None => Vec::new(),
    }
}

/// 讀取一個已封存 change 的 artifact 原文（dated_name 如 `2026-07-04-old`，
/// artifact 為 output path）。缺件或不存在回 `None`；路徑穿越參數一律拒絕。
pub fn archived_document_at(root: &Path, dated_name: &str, artifact: &str) -> Option<String> {
    if !is_safe_path_param(dated_name) || !is_safe_path_param(artifact) {
        return None;
    }
    let ctx = init_core_context(root)?;
    ctx.store.read_archived_artifact(dated_name, artifact)
}

/// 列出一個已封存 change 的 delta capability 名（供唯讀規格分頁載入）。
pub fn archived_capabilities_at(root: &Path, dated_name: &str) -> Vec<String> {
    if !is_safe_path_param(dated_name) {
        return Vec::new();
    }
    match init_core_context(root) {
        Some(ctx) => ctx.store.archived_delta_capabilities(dated_name),
        None => Vec::new(),
    }
}

/// 拒絕會逃出目標目錄的路徑參數：`..` 段、絕對路徑、Windows 磁碟前綴。
pub(crate) fn is_safe_path_param(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.starts_with('/') || s.starts_with('\\') || s.contains(':') {
        return false;
    }
    !s.split(['/', '\\']).any(|seg| seg == "..")
}

fn resolve_schema(
    ws: &speclink_core::workspace::Workspace,
    name: &str,
) -> Result<speclink_core::schema::Schema, String> {
    match speclink_core::schema::resolve_with(Some(ws), name) {
        Some(Ok(s)) => Ok(s),
        Some(Err(e)) => Err(e),
        None => Err(speclink_core::schema::not_found_msg(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    const OLD_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\ncreated_with: claude\n";
    const STARTED_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\ncreated_with: claude\nstarted_at: 2026-07-06\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";

    fn fresh_non_project_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("speclink-query-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn list_changes_shape_matches_cli_and_includes_active_change() {
        let fx = FixtureRoot::new("q-list");
        fx.add_change("demo", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 1);
        let item = &arr[0];
        for key in ["name", "status", "totalTasks", "completedTasks", "summary"] {
            assert!(item.get(key).is_some(), "change item missing camelCase key {key}");
        }
        assert_eq!(item["name"], "demo");
        assert_eq!(item["totalTasks"], 2);
        assert_eq!(item["completedTasks"], 1);
    }

    #[test]
    fn list_changes_overlays_lifecycle_marker_fields() {
        // D2：桌面 payload 在 CLI 同形欄位之上疊加 startedAt/startedBy/startedWith
        // （camelCase；未開工為 null）。資料來自已解析 meta，CLI 輸出不受影響。
        let fx = FixtureRoot::new("q-overlay");
        fx.add_change("not-started", OLD_META);
        fx.add_change("underway", STARTED_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 2);
        for item in arr {
            for key in ["startedAt", "startedBy", "startedWith"] {
                assert!(item.get(key).is_some(), "missing overlay key {key} on {item}");
            }
        }
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        let idle = by_name("not-started");
        assert!(idle["startedAt"].is_null());
        assert!(idle["startedBy"].is_null());
        assert!(idle["startedWith"].is_null());
        let underway = by_name("underway");
        assert_eq!(underway["startedAt"], "2026-07-06");
        assert_eq!(underway["startedBy"], "Worker <w@example.com>");
        assert_eq!(underway["startedWith"], "claude");
        // createdBy（camelCase）由 meta 蓋章曝露、snake_case 不外洩（審計：邊界無型別混淆）。
        assert_eq!(underway["createdBy"], "momo");
        assert!(underway.get("created_by").is_none(), "camelCase only");
        // 既有欄位形狀不變。
        assert_eq!(underway["totalTasks"], 2);
        assert_eq!(underway["completedTasks"], 1);
    }

    #[test]
    fn list_changes_overlays_created_date() {
        // 建立時間篩選的資料源（desktop-ux-polish design D5）：created（YYYY-MM-DD）
        // 由已解析 meta 曝露；CLI 輸出不經此路徑、逐位元不變。
        let fx = FixtureRoot::new("q-created");
        fx.add_change("demo", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr[0]["created"], "2026-07-01");
    }

    #[test]
    fn list_changes_overlays_from_discussions_as_array() {
        // 同源連結（多值化）：來自討論的 change 於清單項帶 fromDiscussions 陣列，
        // 依 meta 順序；多來源全列、單來源一元、非討論而來為空陣列。舊單值鍵 fromDiscussion 不再出現。
        let fx = FixtureRoot::new("q-fromdisc");
        fx.add_change(
            "multi",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: alpha-search, beta-cache\n",
        );
        fx.add_change(
            "single",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: alpha-search\n",
        );
        fx.add_change("plain", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(
            by_name("multi")["fromDiscussions"],
            serde_json::json!(["alpha-search", "beta-cache"])
        );
        assert_eq!(by_name("single")["fromDiscussions"], serde_json::json!(["alpha-search"]));
        assert_eq!(by_name("plain")["fromDiscussions"], serde_json::json!([]));
        assert!(by_name("multi").get("fromDiscussion").is_none(), "old single-value key gone");
    }

    #[test]
    fn list_changes_overlays_why_excerpt_from_proposal() {
        // 變更卡描述列資料源（board-card-anatomy design D2）：whyExcerpt 為 proposal.md
        // `## Why` 區段首個非空行原文（camelCase、恆存在 key）；Why 前有其他區段不影響。
        let fx = FixtureRoot::new("q-why");
        fx.add_change("demo", OLD_META);
        fx.add_change("prefixed", OLD_META);
        fx.write(
            "openspec/changes/prefixed/proposal.md",
            "## Summary\n\nOne liner.\n\n## Why\n\nReal reason here.\nSecond line ignored.\n",
        );
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(by_name("demo")["whyExcerpt"], "Demo change.");
        assert_eq!(by_name("prefixed")["whyExcerpt"], "Real reason here.");
        assert!(by_name("demo").get("why_excerpt").is_none(), "camelCase only");
    }

    #[test]
    fn why_excerpt_is_null_when_proposal_or_why_missing() {
        // 缺件容錯（board-card-anatomy design D2）：無 proposal.md、Why 區段缺席或
        // 區段為空時 whyExcerpt 為 null，清單照常回傳（描述列由前端缺席處理）。
        let fx = FixtureRoot::new("q-why-missing");
        fx.write("openspec/changes/no-proposal/.openspec.yaml", OLD_META);
        fx.write("openspec/changes/no-proposal/tasks.md", "- [ ] 1.1 t\n");
        fx.add_change("empty-why", OLD_META);
        fx.write(
            "openspec/changes/empty-why/proposal.md",
            "## Why\n\n## What Changes\n\n- something\n",
        );
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert!(by_name("no-proposal")["whyExcerpt"].is_null(), "no proposal.md → null");
        assert!(by_name("empty-why")["whyExcerpt"].is_null(), "empty Why section → null");
    }

    #[test]
    fn list_changes_overlays_restale_from_as_array() {
        // 「待重新反映」徽章資料源：restale_from 非空的 change 於清單項帶 restaleFrom 陣列
        // （依 meta 順序），無旗標為空陣列。
        let fx = FixtureRoot::new("q-restale");
        fx.add_change(
            "stale",
            "schema: spec-driven\ncreated: 2026-07-01\nrestale_from: alpha, beta\n",
        );
        fx.add_change("fresh", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(by_name("stale")["restaleFrom"], serde_json::json!(["alpha", "beta"]));
        assert_eq!(by_name("fresh")["restaleFrom"], serde_json::json!([]));
    }

    #[test]
    fn list_changes_marks_invalid_card_and_keeps_the_board_open() {
        // spec「看板照常開啟並標記損壞卡」：壞 metadata 卡帶 metaError 診斷、
        // 有效卡不帶，看板照常列出全部卡片。
        let fx = FixtureRoot::new("q-invalid-meta");
        fx.add_change("good", OLD_META);
        fx.add_change("broken", ": : :\n\t bad yaml [unclosed\n");
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        assert_eq!(arr.len(), 2, "one corrupt card must not break the board");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        let broken = by_name("broken");
        assert!(broken["metaError"].is_string(), "invalid card carries the diagnostic: {broken}");
        assert!(!broken["metaError"].as_str().unwrap().is_empty());
        assert!(by_name("good").get("metaError").is_none(), "valid card carries no diagnostic");
    }

    #[test]
    fn list_changes_sorts_by_board_rank_with_unranked_on_top() {
        // spec「看板卡片順序以 board_rank 欄位為真相」＋ design D2：缺值卡置頂
        // 維持回退序（mtime 平手時名稱升冪），具值卡依 rank 字典序升冪殿後。
        let fx = FixtureRoot::new("q-rank-sort");
        fx.add_change("ranked-n", &format!("{OLD_META}board_rank: n\n"));
        fx.add_change("ranked-b", &format!("{OLD_META}board_rank: b\n"));
        fx.add_change("unranked-y", OLD_META);
        fx.add_change("unranked-x", OLD_META);
        let v = list_changes_at(fx.root());
        let names: Vec<String> = v["changes"]
            .as_array()
            .expect("changes array")
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            ["unranked-x", "unranked-y", "ranked-b", "ranked-n"],
            "unranked on top (fallback order), ranked ascending by key"
        );
    }

    #[test]
    fn list_changes_breaks_equal_rank_ties_by_name() {
        // design D2 同值決斷：rank 相同以名稱字典序，跨機器確定。
        let fx = FixtureRoot::new("q-rank-tie");
        fx.add_change("beta", &format!("{OLD_META}board_rank: n\n"));
        fx.add_change("alpha", &format!("{OLD_META}board_rank: n\n"));
        let v = list_changes_at(fx.root());
        let names: Vec<String> = v["changes"]
            .as_array()
            .expect("changes array")
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, ["alpha", "beta"]);
    }

    #[test]
    fn list_specs_shape_matches_cli() {
        let fx = FixtureRoot::new("q-specs");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n\n## Requirements\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let ids: Vec<&str> = arr.iter().filter_map(|s| s["id"].as_str()).collect();
        assert!(ids.contains(&"cap-x"), "canonical spec present: {ids:?}");
    }

    #[test]
    fn list_specs_includes_card_info_fields() {
        // 規格卡收合資訊資料源（spec-archive-drawer design D4）：requirementCount 為
        // `### Requirement:` 標題數、purposeExcerpt 為 Purpose 區段首個非空行原文、
        // traceCount 為全文 @trace 標記的 source 去重數；欄位 camelCase。
        let fx = FixtureRoot::new("q-spec-card");
        fx.write(
            "openspec/specs/cap-x/spec.md",
            concat!(
                "# cap-x Specification\n\n",
                "## Purpose\n\nSearch behavior for the desktop app.\nSecond line ignored.\n\n",
                "## Requirements\n\n",
                "### Requirement: Alpha\n\nIt SHALL alpha.\n\n",
                "<!-- @trace\nsource: change-one\nupdated: 2026-07-01\ncode:\n  - a.rs\n-->\n\n---\n",
                "### Requirement: Beta\n\nIt SHALL beta.\n\n",
                "<!-- @trace\nsource: change-two\nupdated: 2026-07-02\ncode:\n  - b.rs\n-->\n\n---\n",
                "### Requirement: Gamma\n\nIt SHALL gamma.\n\n",
                "<!-- @trace\nsource: change-one\nupdated: 2026-07-03\ncode:\n  - c.rs\n-->",
            ),
        );
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-x").expect("cap-x listed");
        assert_eq!(item["requirementCount"], 3);
        assert_eq!(item["purposeExcerpt"], "Search behavior for the desktop app.");
        assert_eq!(item["purposeTbd"], false);
        assert_eq!(item["traceCount"], 2, "source 去重：change-one 出現兩次計一次");
        for key in ["requirement_count", "purpose_excerpt", "purpose_tbd", "trace_count"] {
            assert!(item.get(key).is_none(), "camelCase only: {key}");
        }
    }

    #[test]
    fn list_specs_purpose_tbd_flags_archive_placeholder() {
        // 佔位偵測與封存產生器文案的一致性以同一測試釘住（design 風險項）：
        // 經真實 archive 動詞為新 capability 產生正典 spec，其 Purpose 即佔位文案，
        // purposeTbd 必為 true。speclink-core 的佔位文案若變動，此測試即紅。
        let fx = FixtureRoot::new("q-spec-tbd");
        fx.write(
            "openspec/changes/demo/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-01\n",
        );
        fx.write(
            "openspec/changes/demo/proposal.md",
            "## Why\n\nDemo.\n\n## What Changes\n\n- something\n",
        );
        fx.write("openspec/changes/demo/tasks.md", "- [x] 1.1 done\n");
        fx.write(
            "openspec/changes/demo/specs/cap-new/spec.md",
            "## ADDED Requirements\n\n### Requirement: Fresh works\n\nIt SHALL work.\n\n#### Scenario: works\n\n- **WHEN** used\n- **THEN** it works\n",
        );
        crate::verbs::archive_at(fx.root(), "demo").expect("archive ok");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-new").expect("cap-new created by archive");
        assert_eq!(item["purposeTbd"], true);
        assert_eq!(item["requirementCount"], 1);
    }

    #[test]
    fn spec_card_fields_tolerate_minimal_content() {
        // 容錯（Implementation Contract 失敗模式）：無 Purpose 區段、無 Requirement、
        // 無 @trace 的規格計數欄位為 0、excerpt 為 null，清單照常回傳。
        let fx = FixtureRoot::new("q-spec-card-bare");
        fx.write("openspec/specs/bare/spec.md", "# bare Specification\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "bare").expect("bare listed");
        assert_eq!(item["requirementCount"], 0);
        assert!(item["purposeExcerpt"].is_null());
        assert_eq!(item["purposeTbd"], false);
        assert_eq!(item["traceCount"], 0);
    }

    #[cfg(unix)]
    #[test]
    fn spec_card_fields_tolerate_unreadable_file() {
        // 不可讀檔案（列出後權限被拒）：該筆欄位 0／null，清單照常回傳且其他規格不受影響，
        // 不因單筆壞檔讓整頁失敗。
        use std::os::unix::fs::PermissionsExt;
        let fx = FixtureRoot::new("q-spec-card-unreadable");
        fx.write("openspec/specs/ok/spec.md", "## Purpose\n\nFine.\n\n### Requirement: A\n\nx\n");
        fx.write("openspec/specs/locked/spec.md", "## Purpose\n\nHidden.\n");
        let locked = fx.root().join("openspec/specs/locked/spec.md");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
        let v = list_specs_at(fx.root());
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).unwrap();
        let arr = v["specs"].as_array().expect("specs array");
        let bad = arr.iter().find(|s| s["id"] == "locked").expect("locked still listed");
        assert_eq!(bad["requirementCount"], 0);
        assert!(bad["purposeExcerpt"].is_null());
        assert_eq!(bad["purposeTbd"], false);
        assert_eq!(bad["traceCount"], 0);
        let good = arr.iter().find(|s| s["id"] == "ok").expect("ok listed");
        assert_eq!(good["requirementCount"], 1);
        assert_eq!(good["purposeExcerpt"], "Fine.");
    }

    #[test]
    fn list_specs_includes_modified_at_from_mtime() {
        // 呈現層輔助欄位（design D2）：規格清單查詢對每個 spec 帶 modifiedAt——
        // spec.md 檔案系統 mtime 的本地日期（YYYY-MM-DD）。剛寫入的檔案即今天。
        let fx = FixtureRoot::new("q-specs-mtime");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n");
        let v = list_specs_at(fx.root());
        let arr = v["specs"].as_array().expect("specs array");
        let item = arr.iter().find(|s| s["id"] == "cap-x").expect("cap-x listed");
        assert_eq!(item["modifiedAt"], speclink_core::util::today().as_str());
    }

    #[test]
    fn modified_date_is_absent_when_mtime_unavailable() {
        // mtime 不可得時欄位缺席（衍生為 None、overlay 不插 key，而非塞 null）。
        let fx = FixtureRoot::new("q-specs-no-mtime");
        let ghost = fx.root().join("openspec/specs/ghost/spec.md");
        assert!(modified_date(&ghost).is_none());
    }

    #[test]
    fn status_shape_matches_cli() {
        let fx = FixtureRoot::new("q-status");
        fx.add_change("demo", OLD_META);
        let v = status_at(fx.root(), "demo").expect("status ok");
        for key in ["changeName", "schemaName", "isComplete", "applyRequires", "artifacts"] {
            assert!(v.get(key).is_some(), "status missing camelCase key {key}");
        }
        assert_eq!(v["changeName"], "demo");
    }

    #[test]
    fn status_unknown_change_errors() {
        let fx = FixtureRoot::new("q-status-unknown");
        fx.add_change("demo", OLD_META);
        assert!(status_at(fx.root(), "no-such-change-xyz").is_err());
    }

    #[test]
    fn document_reads_change_artifact() {
        // artifact 以 output path 定址（與 status 的 outputPath 一致）。
        let fx = FixtureRoot::new("q-doc");
        fx.add_change("demo", OLD_META);
        let doc = document_at(fx.root(), "demo", "proposal.md").expect("proposal exists");
        assert!(doc.contains("## Why"));
    }

    #[test]
    fn spec_document_reads_canonical() {
        let fx = FixtureRoot::new("q-canonical");
        fx.write("openspec/specs/cap-x/spec.md", "# cap-x Specification\n");
        let doc = spec_document_at(fx.root(), "cap-x").expect("spec exists");
        assert!(!doc.trim().is_empty());
    }

    #[test]
    fn document_rejects_path_traversal() {
        // 沒有防護時 `../../../secret.txt` 會逃出 change 目錄讀到專案根的檔案。
        let fx = FixtureRoot::new("q-traversal");
        fx.add_change("demo", OLD_META);
        fx.write("secret.txt", "top secret");
        assert!(
            document_at(fx.root(), "demo", "../../../secret.txt").is_none(),
            "traversal in artifact must be rejected"
        );
        assert!(
            document_at(fx.root(), "../..", "secret.txt").is_none(),
            "traversal in change name must be rejected"
        );
        assert!(
            spec_document_at(fx.root(), "../../secret").is_none(),
            "traversal in capability must be rejected"
        );
    }

    #[test]
    fn archived_document_reads_content_and_lists_capabilities() {
        let fx = FixtureRoot::new("q-archived");
        fx.write(
            "openspec/changes/archive/2026-07-04-old/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-01\narchived_at: 2026-07-04\n",
        );
        fx.write("openspec/changes/archive/2026-07-04-old/proposal.md", "## Why\n\nOld body.\n");
        fx.write("openspec/changes/archive/2026-07-04-old/tasks.md", "- [x] 1.1 done\n");
        fx.write(
            "openspec/changes/archive/2026-07-04-old/specs/cap-x/spec.md",
            "## ADDED Requirements\n",
        );

        assert_eq!(
            archived_document_at(fx.root(), "2026-07-04-old", "proposal.md").unwrap(),
            "## Why\n\nOld body.\n"
        );
        assert_eq!(archived_capabilities_at(fx.root(), "2026-07-04-old"), vec!["cap-x"]);
        // 缺件文件回 None（前端顯示空狀態，不是錯誤）。
        assert!(archived_document_at(fx.root(), "2026-07-04-old", "design.md").is_none());
        assert!(archived_document_at(fx.root(), "2026-01-01-ghost", "proposal.md").is_none());
    }

    #[test]
    fn archived_document_rejects_path_traversal() {
        let fx = FixtureRoot::new("q-archived-traversal");
        fx.write("openspec/changes/archive/2026-07-04-old/proposal.md", "## Why\n");
        fx.write("secret.txt", "top secret");

        assert!(
            archived_document_at(fx.root(), "2026-07-04-old", "../../../../secret.txt").is_none(),
            "traversal in artifact must be rejected"
        );
        assert!(
            archived_document_at(fx.root(), "../..", "secret.txt").is_none(),
            "traversal in dated name must be rejected"
        );
        assert!(
            archived_document_at(fx.root(), "2026-07-04-old", "C:\\evil.txt").is_none(),
            "drive-prefixed artifact must be rejected"
        );
        assert!(archived_capabilities_at(fx.root(), "../..").is_empty());
    }

    #[test]
    fn non_project_yields_empty_state_not_panic() {
        let root = fresh_non_project_dir();
        assert_eq!(list_changes_at(&root), json!({ "changes": [] }));
        assert_eq!(list_specs_at(&root), json!({ "specs": [] }));
        assert!(status_at(&root, "anything").is_err());
        assert!(document_at(&root, "anything", "proposal").is_none());
    }
}

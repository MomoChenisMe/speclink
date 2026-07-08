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
    // changes_json 本身不動）。資料取自 list_changes 已解析的 meta，不另讀檔。
    let items: Vec<Value> = speclink_core::listing::changes_json(store, &changes)
        .iter()
        .zip(changes.iter())
        .map(|(item, c)| {
            let mut v = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
            v["startedAt"] = json!(c.meta.started_at);
            v["startedBy"] = json!(c.meta.started_by);
            v["startedWith"] = json!(c.meta.started_with);
            v["fromDiscussion"] = json!(c.meta.from_discussion);
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
    // 桌面 payload 在 CLI 同形項上疊加呈現層輔助欄位 modifiedAt（design D2；
    // parity 紅線：CLI 的 specs_json_items 本身不動）。mtime 不可得時不插 key。
    let mut specs = speclink_core::listing::specs_json_items(store);
    if let Value::Array(items) = &mut specs {
        for item in items {
            let Some(id) = item["id"].as_str() else { continue };
            if let Some(date) = modified_date(&store.canonical_spec_path(id)) {
                item["modifiedAt"] = json!(date);
            }
        }
    }
    json!({ "specs": specs })
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
        // 既有欄位形狀不變。
        assert_eq!(underway["totalTasks"], 2);
        assert_eq!(underway["completedTasks"], 1);
    }

    #[test]
    fn list_changes_overlays_from_discussion() {
        // 同源連結（design D6）：來自討論的 change 於清單項帶 fromDiscussion，
        // 供卡片徽章與抽屜同源清單派生；非討論而來為 null。
        let fx = FixtureRoot::new("q-fromdisc");
        fx.add_change(
            "cut-a",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: alpha-search\n",
        );
        fx.add_change("plain", OLD_META);
        let v = list_changes_at(fx.root());
        let arr = v["changes"].as_array().expect("changes array");
        let by_name = |name: &str| arr.iter().find(|c| c["name"] == name).unwrap().clone();
        assert_eq!(by_name("cut-a")["fromDiscussion"], "alpha-search");
        assert!(by_name("plain")["fromDiscussion"].is_null());
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

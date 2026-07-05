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
    let mut changes = speclink_core::model::list_changes(store);
    // GUI 沿用 CLI 的預設排序，使清單順序與 `speclink list --json` 一致。
    speclink_core::listing::sort_changes(store, &mut changes, "modified");
    let items = speclink_core::listing::changes_json(store, &changes);
    json!({ "changes": items })
}

/// 對應 `speclink list --specs --json`：`{ "specs": … }`。非專案回傳 `{ "specs": [] }`。
pub fn list_specs_at(root: &Path) -> Value {
    let Some(ctx) = init_core_context(root) else {
        return json!({ "specs": [] });
    };
    let store: &dyn Store = &ctx.store;
    json!({ "specs": speclink_core::listing::specs_json_items(store) })
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
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..")
    }

    fn fresh_non_project_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("speclink-query-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn list_changes_shape_matches_cli_and_includes_active_change() {
        let v = list_changes_at(&repo_root());
        let arr = v["changes"].as_array().expect("changes array");
        assert!(!arr.is_empty(), "repo has active changes");
        let item = &arr[0];
        for key in ["name", "status", "totalTasks", "completedTasks"] {
            assert!(item.get(key).is_some(), "change item missing camelCase key {key}");
        }
        let names: Vec<&str> = arr.iter().filter_map(|c| c["name"].as_str()).collect();
        assert!(names.contains(&"desktop-shell-and-browser"));
    }

    #[test]
    fn list_specs_shape_matches_cli() {
        let v = list_specs_at(&repo_root());
        let arr = v["specs"].as_array().expect("specs array");
        let ids: Vec<&str> = arr.iter().filter_map(|s| s["id"].as_str()).collect();
        assert!(ids.contains(&"verb-contract"), "known canonical spec present");
    }

    #[test]
    fn status_shape_matches_cli() {
        let v = status_at(&repo_root(), "desktop-shell-and-browser").expect("status ok");
        for key in ["changeName", "schemaName", "isComplete", "applyRequires", "artifacts"] {
            assert!(v.get(key).is_some(), "status missing camelCase key {key}");
        }
        assert_eq!(v["changeName"], "desktop-shell-and-browser");
    }

    #[test]
    fn status_unknown_change_errors() {
        assert!(status_at(&repo_root(), "no-such-change-xyz").is_err());
    }

    #[test]
    fn document_reads_change_artifact() {
        // artifact 以 output path 定址（與 status 的 outputPath 一致）。
        let doc = document_at(&repo_root(), "desktop-shell-and-browser", "proposal.md")
            .expect("proposal exists");
        assert!(doc.contains("## Why"));
    }

    #[test]
    fn spec_document_reads_canonical() {
        let doc = spec_document_at(&repo_root(), "verb-contract").expect("spec exists");
        assert!(!doc.trim().is_empty());
    }

    #[test]
    fn document_rejects_path_traversal() {
        // 沒有防護時 `../../../Cargo.toml` 會逃出 change 目錄讀到 repo 根的檔案。
        assert!(
            document_at(&repo_root(), "desktop-shell-and-browser", "../../../Cargo.toml").is_none(),
            "traversal in artifact must be rejected"
        );
        assert!(
            document_at(&repo_root(), "../../..", "Cargo.toml").is_none(),
            "traversal in change name must be rejected"
        );
        assert!(
            spec_document_at(&repo_root(), "../../Cargo").is_none(),
            "traversal in capability must be rejected"
        );
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

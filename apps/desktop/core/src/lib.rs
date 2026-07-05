//! speclink-desktop-core — 桌面 app 的純邏輯層。
//!
//! Tauri 殼（speclink-desktop）以 #[tauri::command] 薄包裝本 crate 的函式。
//! 本 crate 只依賴 speclink-core 與 speclink-fs，不依賴 Tauri，故可獨立 `cargo test`。

use std::path::Path;

use speclink_core::workspace::Workspace;
use speclink_fs::FsStore;

pub mod cache;
pub mod manage;
pub mod query;
pub mod verbs;

/// 桌面 app 對單一本地 openspec/ 專案的執行語境：探索到的 workspace ＋ 其 fs store。
pub struct ProjectContext {
    pub workspace: Workspace,
    pub store: FsStore,
}

/// 自 `root` 起向上探索 speclink 專案；找到則建構 [`ProjectContext`]，否則回傳 `None`
/// （非 speclink 專案目錄）。不 spawn CLI、不移動任何文件真相。
pub fn init_core_context(root: &Path) -> Option<ProjectContext> {
    let workspace = Workspace::discover(root)?;
    let store = FsStore::new(&workspace.root, &workspace.spec_dir_name);
    Some(ProjectContext { workspace, store })
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::store::Store;
    use std::path::PathBuf;

    /// 本 crate 位於 <repo>/apps/desktop/core，故 repo 根為 manifest 的上三層。
    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
    }

    fn fresh_non_project_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("speclink-desktop-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn init_core_context_on_project_root_discovers_workspace() {
        let ctx = init_core_context(&repo_root()).expect("should discover the speclink project");
        assert!(
            ctx.workspace.spec_dir().join("changes").exists(),
            "discovered workspace should expose the openspec changes dir"
        );
        // 不得 panic：驗證內嵌 core 可讀取本地文件。
        let _ = ctx.store.list_changes();
    }

    #[test]
    fn init_core_context_on_non_project_returns_none() {
        assert!(
            init_core_context(&fresh_non_project_dir()).is_none(),
            "a non-speclink directory should yield no project context"
        );
    }
}

//! speclink-desktop-core — 桌面 app 的純邏輯層。
//!
//! Tauri 殼（speclink-desktop）以 #[tauri::command] 薄包裝本 crate 的函式。
//! 本 crate 只依賴 speclink-core 與 speclink-fs，不依賴 Tauri，故可獨立 `cargo test`。

use std::path::Path;

use speclink_core::workspace::Workspace;
use speclink_fs::FsStore;

pub mod cache;
pub mod discussions;
pub mod manage;
pub mod project;
pub mod query;
pub mod rank;
pub mod search;
pub mod settings;
#[cfg(test)]
pub(crate) mod testfixture;
pub mod verbs;

/// 桌面 app 對單一本地 openspec/ 專案的執行語境：探索到的 workspace ＋ 其 fs store。
pub struct ProjectContext {
    pub workspace: Workspace,
    pub store: FsStore,
}

/// 自 `root` 起向上探索 speclink 專案；找到則建構 [`ProjectContext`]，否則回傳 `None`
/// （非 speclink 專案目錄；壞 `.speclink.yaml` 依引擎 fail-closed 同樣視為建構
/// 失敗——Phase 3 WorkspaceSession 收編時再浮出錯誤細節）。不 spawn CLI、不移動
/// 任何文件真相。
pub fn init_core_context(root: &Path) -> Option<ProjectContext> {
    let workspace = Workspace::discover(root).ok().flatten()?;
    let store = FsStore::new(&workspace.root, &workspace.spec_dir_name);
    Some(ProjectContext { workspace, store })
}

/// 單一 change 的執行語境（design D1）：該 change 有 worktree 映射時，以那份
/// worktree 副本為根重建 [`ProjectContext`]；否則沿用主 checkout。
///
/// worktree 是完整 checkout，Workspace 與 store 在其中天然成立，因此讀與寫共用
/// 同一機制：任務完成的側效（touched 記錄、git 髒檔歸因、head commit、開工章）
/// 隨定根一併落在 worktree 內。每次呼叫現取 observed_facts、不快取——映射條件
/// 已含「worktree 內 change 目錄可讀」，資料夾被移除的下一次呼叫自然回讀主
/// checkout，沒有 stale 視窗。政策關、非主 checkout、git 不可用時 facts 為空，
/// 回傳的就是主 checkout context（行為與本函式出現前完全相同）。
pub(crate) fn context_for_change(root: &Path, change: &str) -> Option<ProjectContext> {
    let ctx = init_core_context(root)?;
    let facts = speclink_host::worktree::observed_facts(&ctx.workspace, &ctx.store, |key| {
        std::env::var(key).ok()
    });
    match facts.get(change) {
        // worktree 副本自身探索不成立（.speclink.yaml 損壞等）時靜默回讀主副本，
        // 沿用 discovery 的 fail-open 慣例。
        Some(entry) => Some(init_core_context(&entry.path).unwrap_or(ctx)),
        None => Some(ctx),
    }
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

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
pub mod manual;
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

/// 現取 worktree 觀察面（design D4：每次現取、不快取）。env 讀取慣例的唯一落點。
pub(crate) fn facts_for(ctx: &ProjectContext) -> speclink_host::worktree::WorktreeFacts {
    speclink_host::worktree::observed_facts(&ctx.workspace, &ctx.store, |key| {
        std::env::var(key).ok()
    })
}

/// 「這個 change 該讀／寫哪個根」的唯一裁決：有映射且副本仍持有該 change 目錄
/// → 該 worktree 根；否則 `None`（＝主 checkout）。存在性檢查與
/// `WorktreeOverlay::of` 的回退同步——facts 取得後、使用前副本被移除的競態
/// 窗口內，整組讀寫一起回主 checkout，不出現半套視圖。
pub(crate) fn worktree_root_for<'a>(
    ctx: &ProjectContext,
    facts: &'a speclink_host::worktree::WorktreeFacts,
    change: &str,
) -> Option<&'a Path> {
    facts
        .get(change)
        .filter(|e| {
            e.path.join(&ctx.workspace.spec_dir_name).join("changes").join(change).is_dir()
        })
        .map(|e| e.path.as_path())
}

/// 單一 change 的執行語境（design D1）：該 change 有 worktree 映射時，以那份
/// worktree 副本為根建構 [`ProjectContext`]；否則沿用主 checkout。
///
/// worktree 是完整 checkout，Workspace 與 store 在其中天然成立，因此讀與寫共用
/// 同一機制：任務完成的側效（touched 記錄、git 髒檔歸因、head commit、開工章）
/// 隨定根一併落在 worktree 內。每次呼叫現取 observed_facts、不快取——映射條件
/// 已含「worktree 內 change 目錄可讀」，資料夾被移除的下一次呼叫自然回讀主
/// checkout，沒有 stale 視窗。政策關、非主 checkout、git 不可用時 facts 為空，
/// 回傳的就是主 checkout context（行為與本函式出現前完全相同）。
pub(crate) fn context_for_change(root: &Path, change: &str) -> Option<ProjectContext> {
    let ctx = init_core_context(root)?;
    let facts = facts_for(&ctx);
    match worktree_root_for(&ctx, &facts, change) {
        // 以 worktree 路徑＋主 checkout 的 spec 目錄名直接建構（與 overlay 的
        // 組裝同款），不走 discovery——向上探索在極端情境會走出 worktree、落到
        // 祖先目錄的別的專案。
        Some(wt_root) => {
            let workspace = Workspace {
                root: wt_root.to_path_buf(),
                spec_dir_name: ctx.workspace.spec_dir_name.clone(),
            };
            let store = FsStore::new(&workspace.root, &workspace.spec_dir_name);
            Some(ProjectContext { workspace, store })
        }
        None => Some(ctx),
    }
}

/// [`context_for_change`] 的必得版：非 speclink 專案時回統一錯誤訊息——
/// 動詞層（verbs.rs 與 manage.rs 的寫入動詞）共用的唯一轉譯。
pub(crate) fn require_context_for_change(
    root: &Path,
    change: &str,
) -> Result<ProjectContext, String> {
    context_for_change(root, change)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))
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
            init_core_context(&crate::testfixture::fresh_non_project_dir("lib")).is_none(),
            "a non-speclink directory should yield no project context"
        );
    }

    #[test]
    fn worktree_root_resolution_falls_back_when_the_copy_is_gone() {
        // 審查 finding（Round 1）：讀根解析要與 overlay 的存在性回退同步——facts
        // 取得後、使用前 worktree 被移除的競態窗口內，解析須回主 checkout，
        // 不得回傳指向已消失路徑的根（指紋錨才不會誤判 Stale）。
        let fx = crate::testfixture::FixtureRoot::new("lib-root-gone");
        fx.add_change("add-auth", "schema: spec-driven\ncreated: 2026-07-01\n");
        let ctx = init_core_context(fx.root()).expect("project context");
        let mut facts = speclink_host::worktree::WorktreeFacts::new();
        facts.insert(
            "add-auth".to_string(),
            speclink_host::worktree::WorktreeEntry {
                path: fx.root().join("no-such-worktree"),
                branch: "speclink/add-auth".to_string(),
            },
        );
        assert!(
            worktree_root_for(&ctx, &facts, "add-auth").is_none(),
            "消失的副本須回退主 checkout"
        );
        assert!(worktree_root_for(&ctx, &facts, "other").is_none(), "無映射本就回主 checkout");
    }
}

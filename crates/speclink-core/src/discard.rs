//! Discard a change: guard against started work, unlink its source discussions, delete it.
//!
//! The counterpart to `archive` for a change that should not survive — a proposal cut
//! before (or, with `--force`, after) work began. Mirrors archive's top-level verb module
//! shape: core owns the orchestration (guard → unlink → delete → report), the Store trait
//! owns the physical removal.

use crate::model::Change;
use crate::store::Store;
use crate::workspace::Workspace;
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct DiscardOutcome {
    pub change_name: String,
    /// Each source discussion unlinked: (slug, status after unlinking). Skipped slugs
    /// (no live record, or the change was not in its `promoted_to` list) are omitted.
    pub unlinked_discussions: Vec<(String, String)>,
}

/// Discard a change. Guards run before any write: a missing change errors, and a change
/// carrying started work (its meta has `started_at`, or `tasks.md` has any checked task)
/// is refused unless `force`. Once guarded, each source discussion is unlinked (the
/// discussion-side `promoted_to` maintenance, reverting status when its list empties),
/// then the change directory is deleted, then the touched record removed — in that order,
/// so a delete failure leaves the unlinks already applied and re-running discard idempotent.
pub fn discard(
    ws: &Workspace,
    store: &dyn Store,
    change_name: &str,
    force: bool,
) -> Result<DiscardOutcome> {
    let Some(change) = crate::model::find_change(store, change_name) else {
        bail!("Change '{change_name}' not found.");
    };

    if !force && has_started_work(store, &change) {
        // Typed refusal: same frozen text, but the command layer classifies it
        // `refused` (needs --force) instead of a plain error.
        return Err(crate::command::Refusal(format!(
            "change '{change_name}' has started work (started_at set or tasks checked) — \
             discard refuses to delete it; pass --force to discard anyway"
        ))
        .into());
    }

    // Unlink BEFORE deleting the directory: an interrupted run leaves the discussions
    // already reverted (re-running discard is idempotent on them), and the change-side
    // link is exactly what we are removing.
    let unlinked: Vec<(String, String)> = change
        .meta
        .from_discussions()
        .into_iter()
        .filter_map(|slug| {
            crate::discuss::unlink_discarded(store, &slug, change_name)
                .ok()
                .flatten()
                .map(|status| (slug, status))
        })
        .collect();

    // Delete the change directory (the irreversible step). On failure the unlinks are NOT
    // rolled back — the error names them so the half-done state is visible; a re-run
    // finishes the delete and no-ops the already-unlinked discussions.
    if let Err(e) = store.delete_change(change_name) {
        let note = if unlinked.is_empty() {
            String::new()
        } else {
            let slugs: Vec<&str> = unlinked.iter().map(|(s, _)| s.as_str()).collect();
            format!(" (source discussions already unlinked: {})", slugs.join(", "))
        };
        bail!(
            "failed to remove the change directory for '{change_name}': {e}{note} — re-run discard to finish"
        );
    }

    // Remove the workspace touched record (present only after `task done`; ignore absence).
    let _ = crate::util::remove_file(&ws.touched_dir().join(format!("{change_name}.json")));

    Ok(DiscardOutcome {
        change_name: change_name.to_string(),
        unlinked_discussions: unlinked,
    })
}

/// A change carries "started work" when its metadata bears the `started_at` stamp or its
/// tasks file has any checked box — the two engine-owned signals that implementation began.
fn has_started_work(store: &dyn Store, change: &Change) -> bool {
    if change.meta.started_at.is_some() {
        return true;
    }
    store
        .read_artifact(&change.name, "tasks.md")
        .map(|t| crate::tasks::parse(&t).iter().any(|task| task.done))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::discard;
    use crate::store::Store;
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    fn ghost_ws() -> Workspace {
        // Nonexistent root: the touched-record removal targets a path that does not
        // exist and is ignored, so guard/unlink behavior is fully deterministic.
        Workspace {
            root: std::env::temp_dir().join("speclink-discard-test-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        }
    }

    const UNSTARTED: &str = "schema: spec-driven\ncreated: 2026-07-09\n";
    const STARTED: &str = "schema: spec-driven\ncreated: 2026-07-09\nstarted_at: 2026-07-09\n";

    fn store_with_tasks(meta: &str, tasks_md: &str) -> TestStore {
        let store = TestStore::with_meta("cut", meta);
        store.put_artifact("cut", "tasks.md", tasks_md);
        store
    }

    // --- guard matrix: started_at × checked task (spec Example「動工痕跡判定」) ---

    #[test]
    fn guard_allows_only_when_no_started_at_and_no_checked_task() {
        // (started_at=否, 已勾=否) → 放行。
        let store = store_with_tasks(UNSTARTED, "- [ ] 1.1 open\n- [ ] 1.2 open\n");
        assert!(store.change_exists("cut"));
        discard(&ghost_ws(), &store, "cut", false).unwrap();
        assert!(!store.change_exists("cut"), "unstarted change is deleted");
    }

    #[test]
    fn guard_refuses_when_started_at_set() {
        // (started_at=是, 已勾=否) → 拒絕。
        let store = store_with_tasks(STARTED, "- [ ] 1.1 open\n");
        let err = discard(&ghost_ws(), &store, "cut", false).unwrap_err();
        assert!(err.to_string().contains("--force"), "err: {err}");
        assert!(store.change_exists("cut"), "refused discard must not delete");
    }

    #[test]
    fn guard_refuses_when_any_task_checked() {
        // (started_at=否, 已勾=是) → 拒絕。
        let store = store_with_tasks(UNSTARTED, "- [ ] 1.1 open\n- [x] 1.2 done\n");
        let err = discard(&ghost_ws(), &store, "cut", false).unwrap_err();
        assert!(err.to_string().contains("--force"), "err: {err}");
        assert!(store.change_exists("cut"));
    }

    #[test]
    fn guard_refuses_when_both_started_and_checked() {
        // (started_at=是, 已勾=是) → 拒絕。
        let store = store_with_tasks(STARTED, "- [x] 1.1 done\n");
        let err = discard(&ghost_ws(), &store, "cut", false).unwrap_err();
        assert!(err.to_string().contains("--force"), "err: {err}");
        assert!(store.change_exists("cut"));
    }

    #[test]
    fn guard_rejection_is_zero_write() {
        // 守衛拒絕時零寫入：變更留存、來源討論逐位元不變。
        let store = store_with_tasks(
            "schema: spec-driven\ncreated: 2026-07-09\nstarted_at: 2026-07-09\nfrom_discussion: d1\n",
            "- [ ] 1.1 open\n",
        );
        let doc = "---\ntopic: d1\nslug: d1\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: x\n";
        store.discussions.borrow_mut().insert("d1".into(), doc.to_string());

        assert!(discard(&ghost_ws(), &store, "cut", false).is_err());
        assert!(store.change_exists("cut"), "change untouched");
        assert_eq!(store.discussion("d1"), doc, "discussion untouched — no unlink on a rejected discard");
    }

    #[test]
    fn force_discards_a_started_change() {
        // --force 放行動過工的變更。
        let store = store_with_tasks(STARTED, "- [x] 1.1 done\n");
        discard(&ghost_ws(), &store, "cut", true).unwrap();
        assert!(!store.change_exists("cut"));
    }

    #[test]
    fn missing_change_errors() {
        // spec「變更不存在報錯」：不存在的變更名 → 錯誤、無檔案效果。
        let store = TestStore::default();
        let err = discard(&ghost_ws(), &store, "ghost", false).unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
    }

    // --- unlink + delete orchestration (design D1；spec「討論隨變更廢棄解鏈」) ---

    #[test]
    fn success_unlinks_every_source_discussion_then_deletes() {
        // 多來源討論逐一解鏈：d1 唯一值 → 回退 concluded；d2 仍被 keep 引用 → 縮減維持 promoted。
        let store = store_with_tasks(
            "schema: spec-driven\ncreated: 2026-07-09\nfrom_discussion: d1, d2\n",
            "- [ ] 1.1 open\n",
        );
        store.metas.borrow_mut().insert(
            "keep".into(),
            "schema: spec-driven\ncreated: 2026-07-08\nfrom_discussion: d2\n".into(),
        );
        store.discussions.borrow_mut().insert(
            "d1".into(),
            "---\ntopic: d1\nslug: d1\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: x\n".into(),
        );
        store.discussions.borrow_mut().insert(
            "d2".into(),
            "---\ntopic: d2\nslug: d2\nstatus: promoted\npromoted_to: cut, keep\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: y\n".into(),
        );

        let outcome = discard(&ghost_ws(), &store, "cut", false).unwrap();

        assert_eq!(outcome.change_name, "cut");
        assert_eq!(
            outcome.unlinked_discussions,
            vec![("d1".to_string(), "concluded".to_string()), ("d2".to_string(), "promoted".to_string())],
        );
        assert!(!store.change_exists("cut"), "change deleted after unlink");
        assert!(store.discussion("d1").contains("status: concluded\n"), "d1: {}", store.discussion("d1"));
        assert!(!store.discussion("d1").contains("promoted_to"), "d1 promoted_to line dropped");
        assert!(store.discussion("d2").contains("status: promoted\n"), "d2 still promoted");
        assert!(store.discussion("d2").contains("promoted_to: keep\n"), "d2 shrunk to keep: {}", store.discussion("d2"));
    }

    #[test]
    fn missing_source_record_is_skipped_not_failed() {
        // spec「缺失記錄跳過」：from_discussion 指向的 slug 無 live 記錄 → 跳過、指令不失敗。
        let store = store_with_tasks(
            "schema: spec-driven\ncreated: 2026-07-09\nfrom_discussion: ghost-slug\n",
            "- [ ] 1.1 open\n",
        );
        let outcome = discard(&ghost_ws(), &store, "cut", false).unwrap();
        assert!(outcome.unlinked_discussions.is_empty(), "missing slug contributes no report entry");
        assert!(!store.change_exists("cut"), "change still deleted");
    }

    #[test]
    fn delete_failure_keeps_unlinks_and_names_them() {
        // 失敗模式：目錄刪除失敗 → 已完成解鏈不回滾、錯誤明示已解鏈清單。
        let store = store_with_tasks(
            "schema: spec-driven\ncreated: 2026-07-09\nfrom_discussion: d1\n",
            "- [ ] 1.1 open\n",
        );
        store.discussions.borrow_mut().insert(
            "d1".into(),
            "---\ntopic: d1\nslug: d1\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: x\n".into(),
        );
        *store.fail_delete_change.borrow_mut() = true;

        let err = discard(&ghost_ws(), &store, "cut", false).unwrap_err();
        assert!(err.to_string().contains("d1"), "error must name the already-unlinked discussion: {err}");
        assert!(store.change_exists("cut"), "delete failed — change remains");
        assert!(
            store.discussion("d1").contains("status: concluded\n"),
            "unlink already applied and NOT rolled back: {}",
            store.discussion("d1")
        );
    }

    #[test]
    fn success_removes_touched_record_file() {
        // 成功路徑刪除 touched 紀錄檔（--force 路徑，動過工才有 touched）。
        let dir = std::env::temp_dir().join(format!("speclink-discard-touched-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ws = Workspace { root: dir.clone(), spec_dir_name: "openspec".to_string() };
        let touched = ws.touched_dir().join("cut.json");
        std::fs::create_dir_all(touched.parent().unwrap()).unwrap();
        std::fs::write(&touched, "{\"change\":\"cut\",\"touched\":[]}").unwrap();
        assert!(touched.is_file());

        let store = store_with_tasks(STARTED, "- [x] 1.1 done\n");
        discard(&ws, &store, "cut", true).unwrap();

        assert!(!touched.exists(), "touched record removed on discard");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

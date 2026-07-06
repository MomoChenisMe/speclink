//! openspec/ 檔案監看（design D3）：遞迴監看 `<root>/openspec/`，事件合併
//! 去抖後以回呼通知——不帶 payload，訂閱端一律整批 refresh，不做細粒度 diff。
//! 回呼式核心與 Tauri 事件發送分離：本模組不知道 Tauri 存在，lib.rs 的 setup
//! 把回呼接到 `workspace-changed` 事件上。
//!
//! 監看範圍即專案根下的 openspec/ 一棵樹——`.speclink/`（快取）與其他兄弟目錄
//! 天然不在範圍內，無自迴圈。

use std::path::Path;
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

/// 存活期間持續監看；drop 即停止（app 生命週期內由 Tauri state 持有）。
pub struct WorkspaceWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
}

/// 遞迴監看 `<root>/openspec/`；去抖窗口內的事件合併為一次 `on_change()` 呼叫。
/// 監看目標不存在或建立失敗時回傳可記錄的 `Err`——呼叫端記錄後照常運作，不 panic。
pub fn watch_openspec(
    root: &Path,
    debounce: Duration,
    on_change: impl Fn() + Send + 'static,
) -> Result<WorkspaceWatcher, String> {
    let target = root.join("openspec");
    if !target.is_dir() {
        return Err(format!("watch target missing: {}", target.display()));
    }
    let mut debouncer = new_debouncer(debounce, move |result: DebounceEventResult| {
        // 事件內容不重要（訂閱端整批 refresh）；錯誤批次不通知也不中斷監看。
        if matches!(&result, Ok(events) if !events.is_empty()) {
            on_change();
        }
    })
    .map_err(|e| format!("watcher setup failed: {e}"))?;
    debouncer
        .watcher()
        .watch(&target, RecursiveMode::Recursive)
        .map_err(|e| format!("watch {} failed: {e}", target.display()))?;
    Ok(WorkspaceWatcher {
        _debouncer: debouncer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    struct TempRoot(std::path::PathBuf);

    impl TempRoot {
        fn new(tag: &str) -> TempRoot {
            let dir = std::env::temp_dir().join(format!(
                "speclink-watch-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
            TempRoot(dir)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn writes_inside_openspec_coalesce_into_a_single_notification() {
        let root = TempRoot::new("coalesce");
        let hits = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hits);
        let _watcher = watch_openspec(&root.0, Duration::from_millis(300), move || {
            h.fetch_add(1, Ordering::SeqCst);
        })
        .expect("watcher starts on an existing openspec tree");
        // 監看就緒緩衝（Windows ReadDirectoryChangesW 掛載非同步）。
        std::thread::sleep(Duration::from_millis(400));

        // 一波連續寫入（外部 CLI 動詞的典型效果：meta＋tasks＋新檔）。
        let changes = root.0.join("openspec").join("changes");
        std::fs::create_dir_all(changes.join("demo")).unwrap();
        std::fs::write(changes.join("demo").join(".openspec.yaml"), "schema: spec-driven\n").unwrap();
        std::fs::write(changes.join("demo").join("tasks.md"), "- [ ] 1.1 t\n").unwrap();

        std::thread::sleep(Duration::from_millis(2000));
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "a burst of writes must coalesce into exactly one debounced notification"
        );
    }

    #[test]
    fn writes_outside_openspec_do_not_notify() {
        let root = TempRoot::new("outside");
        std::fs::create_dir_all(root.0.join(".speclink")).unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hits);
        let _watcher = watch_openspec(&root.0, Duration::from_millis(200), move || {
            h.fetch_add(1, Ordering::SeqCst);
        })
        .expect("watcher starts");
        std::thread::sleep(Duration::from_millis(400));

        // 快取與專案雜項都在 openspec/ 外——不得觸發。
        std::fs::write(root.0.join(".speclink").join("desktop-cache.db"), "cache").unwrap();
        std::fs::write(root.0.join("note.txt"), "not a spec doc").unwrap();

        std::thread::sleep(Duration::from_millis(1200));
        assert_eq!(hits.load(Ordering::SeqCst), 0, "writes outside openspec/ must not notify");
    }

    #[test]
    fn missing_watch_target_errors_without_panic() {
        // 專案根本身不存在。
        let ghost = std::env::temp_dir().join("speclink-watch-ghost-root-nonexistent");
        let _ = std::fs::remove_dir_all(&ghost);
        let err = watch_openspec(&ghost, Duration::from_millis(100), || {});
        assert!(err.is_err(), "missing root must be a loggable error, not a panic");

        // 根存在但沒有 openspec/。
        let root = std::env::temp_dir().join(format!("speclink-watch-nospec-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let err = watch_openspec(&root, Duration::from_millis(100), || {});
        assert!(err.is_err(), "root without openspec/ must be a loggable error");
        let _ = std::fs::remove_dir_all(&root);
    }
}

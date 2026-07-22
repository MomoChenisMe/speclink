//! openspec/ 檔案監看（design D3）：遞迴監看探索出的 spec 目錄樹，事件合併
//! 去抖後以回呼通知——回呼不帶參數，訂閱端一律整批 refresh，不做細粒度 diff。
//! 回呼式核心與 Tauri 事件發送分離：本模組不知道 Tauri 存在，lib.rs 的
//! watch_workspace command 把回呼接到 `workspace-changed` 事件上（payload 為
//! 被監看的 root，由掛載處閉包補上——workspace-session 決策 5）。
//!
//! 監看目標由 [`resolve_watch_target`] 解析：自起點向上探索 speclink 專案
//! （與查詢指令的 `Workspace::discover` 同源），取其實際 spec 目錄——監看根
//! SHALL 與查詢一致，不以未經探索的啟動 cwd 拼接固定目錄名。`.speclink/`
//! （快取）與其他兄弟目錄天然不在範圍內，無自迴圈。

use std::path::{Path, PathBuf};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};

/// 存活期間持續監看；drop 即停止（app 生命週期內由 Tauri state 持有）。
pub struct WorkspaceWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
}

/// 自 `start` 向上探索 speclink 專案，回傳其 spec 目錄作為監看目標。
/// 探索不到專案時回傳可記錄的 `Err`（含起點路徑）——呼叫端記錄後照常運作，
/// 僅失去自動刷新。
pub fn resolve_watch_target(start: &Path) -> Result<PathBuf, String> {
    speclink_desktop_core::init_core_context(start)
        .map(|ctx| ctx.workspace.spec_dir())
        .ok_or_else(|| format!("no speclink project found from {}", start.display()))
}

/// 遞迴監看已解析的 spec 目錄；去抖窗口內的事件合併為一次 `on_change()` 呼叫。
/// 監看目標不存在或建立失敗時回傳可記錄的 `Err`——呼叫端記錄後照常運作，不 panic。
pub fn watch_openspec(
    target: &Path,
    debounce: Duration,
    on_change: impl Fn() + Send + 'static,
) -> Result<WorkspaceWatcher, String> {
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
        .watch(target, RecursiveMode::Recursive)
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
            // 不用系統 tmpdir：CI 上系統排程（如 systemd-tmpfiles）會遍歷 /tmp，
            // 而 notify 的 inotify 遮罩含 OPEN/ATTRIB——外部行程 open 監看樹內的
            // 目錄就會變成事件，讓計數型斷言被環境雜訊擊穿。target/ 無人遍歷。
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("..")
                .join("target")
                .join(format!("speclink-watch-{tag}-{}", std::process::id()));
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
        // demo/ 先於監看存在：Linux inotify 對新目錄要事件後補掛 watch，目錄
        // 建立與其內容寫入會被拆進不同 debounce 批次，「恰一次」斷言只能建立在
        // 已被監看的目錄上（macOS FSEvents 無此拆分，先前不可見）。
        let changes = root.0.join("openspec").join("changes");
        std::fs::create_dir_all(changes.join("demo")).unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hits);
        let _watcher = watch_openspec(
            &root.0.join("openspec"),
            Duration::from_millis(300),
            move || {
                h.fetch_add(1, Ordering::SeqCst);
            },
        )
        .expect("watcher starts on an existing openspec tree");
        // 監看就緒緩衝（Windows ReadDirectoryChangesW 掛載非同步），並讓 Linux 的掛載
        // 自我通知流完：notify 以 WalkDir 遞迴補掛 watch，父目錄的 OPEN 遮罩把走訪
        // 本身變成一批事件（FSEvents／RDCW 不報 open，僅 inotify 可見）。
        std::thread::sleep(Duration::from_millis(600));
        hits.store(0, Ordering::SeqCst); // 丟棄掛載批次，只計入動作後的通知

        // 一波連續寫入（外部 CLI 動詞的典型效果：meta＋tasks）。
        std::fs::write(
            changes.join("demo").join(".openspec.yaml"),
            "schema: spec-driven\n",
        )
        .unwrap();
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
        let _watcher = watch_openspec(
            &root.0.join("openspec"),
            Duration::from_millis(200),
            move || {
                h.fetch_add(1, Ordering::SeqCst);
            },
        )
        .expect("watcher starts");
        // 同 coalesce 測試：等掛載自我通知（Linux inotify 的 OPEN 遮罩）流完後歸零。
        std::thread::sleep(Duration::from_millis(600));
        hits.store(0, Ordering::SeqCst);

        // 快取與專案雜項都在 openspec/ 外——不得觸發。
        std::fs::write(root.0.join(".speclink").join("desktop-cache.db"), "cache").unwrap();
        std::fs::write(root.0.join("note.txt"), "not a spec doc").unwrap();

        std::thread::sleep(Duration::from_millis(1200));
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            "writes outside openspec/ must not notify"
        );
    }

    #[test]
    fn resolve_watch_target_walks_up_from_project_subdir() {
        // 模擬自建置輸出目錄（如 target/release）為 cwd 啟動：起點在專案根
        // 之下，仍須向上探索出專案的 spec 目錄。
        let root = TempRoot::new("resolve-subdir");
        let sub = root.0.join("target").join("release");
        std::fs::create_dir_all(&sub).unwrap();
        let target = resolve_watch_target(&sub).expect("subdir inside a project must resolve");
        assert_eq!(target, root.0.join("openspec"));
    }

    #[test]
    fn resolve_watch_target_respects_custom_spec_dir_name() {
        let dir =
            std::env::temp_dir().join(format!("speclink-watch-customdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("myspecs")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "spec_dir: myspecs\n").unwrap();
        let target = resolve_watch_target(&dir).expect("project with custom spec dir must resolve");
        assert_eq!(
            target,
            dir.join("myspecs"),
            "watch target is the discovered spec dir, not a hardcoded name"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_watch_target_outside_any_project_errors() {
        let dir =
            std::env::temp_dir().join(format!("speclink-watch-noproj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let err =
            resolve_watch_target(&dir).expect_err("non-project start must be a loggable error");
        assert!(
            err.contains(&dir.display().to_string()),
            "error must name the start path for the log: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_watch_target_errors_without_panic() {
        // 監看目標（含其整條路徑）不存在。
        let ghost = std::env::temp_dir().join("speclink-watch-ghost-root-nonexistent");
        let _ = std::fs::remove_dir_all(&ghost);
        let err = watch_openspec(&ghost.join("openspec"), Duration::from_millis(100), || {});
        assert!(
            err.is_err(),
            "missing target must be a loggable error, not a panic"
        );

        // 上層目錄存在但監看目標本身不存在。
        let root =
            std::env::temp_dir().join(format!("speclink-watch-nospec-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let err = watch_openspec(&root.join("openspec"), Duration::from_millis(100), || {});
        assert!(
            err.is_err(),
            "nonexistent target dir must be a loggable error"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

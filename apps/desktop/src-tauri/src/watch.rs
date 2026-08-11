//! openspec/ 檔案監看（design D3）：遞迴監看探索出的 spec 目錄樹，事件合併
//! 去抖後以回呼通知——回呼不帶參數，訂閱端一律整批 refresh，不做細粒度 diff。
//! 回呼式核心與 Tauri 事件發送分離：本模組不知道 Tauri 存在，lib.rs 的
//! watch_workspace command 把回呼接到 `workspace-changed` 事件上（payload 為
//! 被監看的 root，由掛載處閉包補上——workspace-session 決策 5）。
//!
//! 監看目標由 [`resolve_watch_targets`] 解析：自起點向上探索 speclink 專案
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

/// 事件路徑的比對用正規化：已刪除的路徑 canonicalize 必敗，退一步正規化
/// 父目錄再接回檔名（哨兵比對的父目錄 .git 恆存在）；再不行原樣退回。
fn normalized_for_match(p: &Path) -> PathBuf {
    if let Ok(c) = p.canonicalize() {
        return c;
    }
    if let (Some(parent), Some(name)) = (p.parent(), p.file_name()) {
        if let Ok(c) = parent.canonicalize() {
            return c.join(name);
        }
    }
    p.to_path_buf()
}

/// 自 `start` 向上探索 speclink 專案，回傳所有監看目標——第一項為其 spec 目錄，
/// 其後為 worktree 流程下的副本 change 目錄與 worktree 登記簿（推導歸 desktop-core）。
/// 探索不到專案時回傳可記錄的 `Err`（含起點路徑）——呼叫端記錄後照常運作，
/// 僅失去自動刷新。
pub fn resolve_watch_targets(start: &Path) -> Result<Vec<PathBuf>, String> {
    let targets = speclink_desktop_core::query::watch_targets_at(start);
    if targets.is_empty() {
        return Err(format!("no speclink project found from {}", start.display()));
    }
    Ok(targets)
}

/// 遞迴監看已解析的目標；去抖窗口內的事件合併為一次 `on_change()` 呼叫。
/// 第一個目標（spec 目錄）不存在或建立失敗時回傳可記錄的 `Err`；其後的
/// worktree 目標為盡力而為——推導與實際掛載之間 worktree 可能已被移除，
/// 少監看一個路徑只是少一次自動刷新，不該讓整個監看失敗。
///
/// 例外：名為 `.git` 的目標是「登記簿出生前」的哨兵（推導層在
/// `.git/worktrees/` 尚不存在時以它代位），非遞迴掛載——只等直接子目錄
/// worktrees/ 的建立事件，不吞 objects/ 的海量寫入。
pub fn watch_openspec(
    targets: &[PathBuf],
    debounce: Duration,
    on_change: impl Fn() + Send + 'static,
) -> Result<WorkspaceWatcher, String> {
    let Some(target) = targets.first() else {
        return Err("no watch target".to_string());
    };
    if !target.is_dir() {
        return Err(format!("watch target missing: {}", target.display()));
    }
    // 哨兵目標只等 worktrees/ 出生；index、HEAD 等 .git 直接子項幾乎每個
    // git 指令都寫，這些事件於事件層過濾，不變成看板空刷新。比對前兩側都
    // 正規化——FSEvents 回報解析後路徑，監看目標卻可能經 symlink 或含 `..`。
    let sentinel = targets
        .iter()
        .skip(1)
        .find(|p| p.file_name() == Some(std::ffi::OsStr::new(".git")))
        .map(|p| p.canonicalize().unwrap_or_else(|_| p.clone()));
    let mut debouncer = new_debouncer(debounce, move |result: DebounceEventResult| {
        // 事件內容只看一件事：是否全為哨兵雜訊；錯誤批次不通知也不中斷監看。
        let Ok(events) = &result else { return };
        let relevant = events.iter().any(|e| match &sentinel {
            Some(git_dir) => {
                let path = normalized_for_match(&e.path);
                path != *git_dir
                    && !(path.parent() == Some(git_dir.as_path())
                        && path.file_name() != Some(std::ffi::OsStr::new("worktrees")))
            }
            None => true,
        });
        if !events.is_empty() && relevant {
            on_change();
        }
    })
    .map_err(|e| format!("watcher setup failed: {e}"))?;
    debouncer
        .watcher()
        .watch(target, RecursiveMode::Recursive)
        .map_err(|e| format!("watch {} failed: {e}", target.display()))?;
    for extra in targets.iter().skip(1).filter(|p| p.is_dir()) {
        let mode = if extra.file_name() == Some(std::ffi::OsStr::new(".git")) {
            RecursiveMode::NonRecursive
        } else {
            RecursiveMode::Recursive
        };
        let _ = debouncer.watcher().watch(extra, mode);
    }
    Ok(WorkspaceWatcher {
        _debouncer: debouncer,
    })
}

/// 活躍監看與其目標集合的槽位：重掛時目標集合不變即沿用原監看。
///
/// 重掛由 workspace-changed 事件驅動（前端每次刷新順手重掛），而 Linux
/// inotify 的掛載走訪本身會變成一批事件——每次都整顆重建就是事件迴圈。
/// 比對後只在拓撲真的改變（worktree 增減、登記簿出生）時重建。
///
/// async 化後重掛跑在執行緒池，兩條鏈（activation 與事件驅動重掛）可交錯：
/// 呼叫端取號後傳入 `ticket`，序號較小的慢工視為陳舊、直接略過，不得反超
/// 較新的掛載。
pub struct WatchSlot {
    current: Option<(Vec<PathBuf>, WorkspaceWatcher)>,
    last_ticket: u64,
}

impl WatchSlot {
    pub fn new() -> WatchSlot {
        WatchSlot {
            current: None,
            last_ticket: 0,
        }
    }

    /// 解析 `start` 的監看目標並視需要重掛；回傳是否重建。陳舊序號略過並回
    /// `Ok(false)`。解析或掛載失敗時清空槽位並回 `Err`——呼叫端記錄後照常
    /// 運作，僅失去自動刷新。
    pub fn rearm(
        &mut self,
        ticket: u64,
        start: &Path,
        debounce: Duration,
        on_change: impl Fn() + Send + 'static,
    ) -> Result<bool, String> {
        if ticket < self.last_ticket {
            return Ok(false);
        }
        let targets = match resolve_watch_targets(start) {
            Ok(t) => t,
            Err(e) => {
                self.current = None;
                return Err(e);
            }
        };
        self.rearm_with_targets(ticket, targets, debounce, on_change)
    }

    /// 判等基準是「可掛載集合」（首項＋其餘存在的目錄），不是解析集合——
    /// 掛載時被存在性過濾略過的目錄（如 checkout 中的 worktree 副本），
    /// 之後出現必須觸發重建補掛，否則它永久失去監看。
    fn rearm_with_targets(
        &mut self,
        ticket: u64,
        targets: Vec<PathBuf>,
        debounce: Duration,
        on_change: impl Fn() + Send + 'static,
    ) -> Result<bool, String> {
        if ticket < self.last_ticket {
            return Ok(false);
        }
        self.last_ticket = ticket;
        let mountable: Vec<PathBuf> = targets
            .iter()
            .enumerate()
            .filter(|(i, p)| *i == 0 || p.is_dir())
            .map(|(_, p)| p.clone())
            .collect();
        if matches!(&self.current, Some((prev, _)) if *prev == mountable) {
            return Ok(false);
        }
        match watch_openspec(&targets, debounce, on_change) {
            Ok(watcher) => {
                self.current = Some((mountable, watcher));
                Ok(true)
            }
            Err(e) => {
                self.current = None;
                Err(e)
            }
        }
    }
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
            &[root.0.join("openspec")],
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
            &[root.0.join("openspec")],
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
    fn the_git_sentinel_is_watched_non_recursively() {
        // .git 哨兵只為等 worktrees/ 出生：遞迴掛載會吞下 objects/ 的海量寫入，
        // 每次 git 操作都變成看板整批 refresh。深層寫入不得通知、直接子目錄
        // 的建立（登記簿出生）必須通知。
        let root = TempRoot::new("git-sentinel");
        let git_dir = root.0.join(".git");
        std::fs::create_dir_all(git_dir.join("objects").join("aa")).unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hits);
        let _watcher = watch_openspec(
            &[root.0.join("openspec"), git_dir.clone()],
            Duration::from_millis(200),
            move || {
                h.fetch_add(1, Ordering::SeqCst);
            },
        )
        .expect("watcher starts");
        std::thread::sleep(Duration::from_millis(600));
        hits.store(0, Ordering::SeqCst); // 丟棄掛載批次

        std::fs::write(git_dir.join("objects").join("aa").join("blob"), "x").unwrap();
        std::thread::sleep(Duration::from_millis(1200));
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            ".git 深層寫入不得通知（哨兵非遞迴）"
        );

        // index、HEAD 等 .git 直接子檔幾乎每個 git 指令都寫——哨兵只等
        // worktrees/ 出生，其餘直接子項的事件不得變成看板空刷新。
        std::fs::write(git_dir.join("index"), "idx").unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        std::thread::sleep(Duration::from_millis(1200));
        assert_eq!(
            hits.load(Ordering::SeqCst),
            0,
            ".git 直接子檔寫入不得通知（哨兵只等 worktrees/ 出生）"
        );

        std::fs::create_dir_all(git_dir.join("worktrees")).unwrap();
        std::thread::sleep(Duration::from_millis(1200));
        assert!(
            hits.load(Ordering::SeqCst) >= 1,
            "登記簿出生（.git 直接子目錄建立）必須通知"
        );
    }

    #[test]
    fn rearm_rebuilds_when_a_previously_missing_target_dir_appears() {
        // worktree 副本的 change 目錄可能晚於 facts 出現（大 repo checkout 慢於
        // 去抖窗）：掛載時被存在性過濾略過的目錄，之後出現必須觸發重建補掛，
        // 否則該目錄永久失去監看。判等基準是「可掛載集合」，不是解析集合。
        let root = TempRoot::new("rearm-late");
        let late = root.0.join("late-dir");
        let targets = vec![root.0.join("openspec"), late.clone()];
        let mut slot = WatchSlot::new();
        let rebuilt = slot
            .rearm_with_targets(1, targets.clone(), Duration::from_millis(200), || {})
            .expect("first mount");
        assert!(rebuilt, "首次必為重建");
        let rebuilt = slot
            .rearm_with_targets(2, targets.clone(), Duration::from_millis(200), || {})
            .expect("same targets resolve");
        assert!(!rebuilt, "目錄仍缺席、可掛載集合不變須沿用");

        std::fs::create_dir_all(&late).unwrap();
        let rebuilt = slot
            .rearm_with_targets(3, targets.clone(), Duration::from_millis(200), || {})
            .expect("rearm after the dir appears");
        assert!(rebuilt, "晚出生的目錄出現＝可掛載集合改變，須重建補掛");
        let rebuilt = slot
            .rearm_with_targets(4, targets, Duration::from_millis(200), || {})
            .expect("stable targets resolve");
        assert!(!rebuilt, "補掛後無變化須沿用");
    }

    #[test]
    fn rearm_keeps_the_watcher_when_targets_are_unchanged() {
        // 事件驅動的重掛若每次都整顆重建，Linux inotify 的掛載自我通知會形成
        // 事件迴圈——目標集合不變就必須保留原監看。
        let root = TempRoot::new("rearm");
        let mut slot = WatchSlot::new();
        let rebuilt = slot
            .rearm(1, &root.0, Duration::from_millis(200), || {})
            .expect("first rearm mounts");
        assert!(rebuilt, "首次必為重建");
        let rebuilt = slot
            .rearm(2, &root.0, Duration::from_millis(200), || {})
            .expect("second rearm resolves");
        assert!(!rebuilt, "目標集合不變須沿用原監看");

        // 拓撲改變（git repo 出生 → 哨兵進場）→ 重建。
        std::fs::create_dir_all(root.0.join(".git")).unwrap();
        let rebuilt = slot
            .rearm(3, &root.0, Duration::from_millis(200), || {})
            .expect("rearm after topology change");
        assert!(rebuilt, "目標集合改變須重建監看");
    }

    #[test]
    fn a_stale_ticket_never_overrides_a_newer_rearm() {
        // async 化後 activation 與事件驅動重掛是兩條可交錯的鏈：舊 root 的慢
        // 掛載後完成時不得反超新 root——序號較小＝陳舊，直接略過。
        let root_new = TempRoot::new("ticket-new");
        let root_old = TempRoot::new("ticket-old");
        let mut slot = WatchSlot::new();
        let newer = slot
            .rearm(2, &root_new.0, Duration::from_millis(200), || {})
            .expect("newer rearm mounts");
        assert!(newer, "新 root 掛載必為重建");
        let stale = slot
            .rearm(1, &root_old.0, Duration::from_millis(200), || {})
            .expect("stale rearm is a no-op, not an error");
        assert!(!stale, "陳舊序號不得重建");
        // 槽位仍在新 root 上：同 root 再掛（新序號）應回報「未重建」。
        let unchanged = slot
            .rearm(3, &root_new.0, Duration::from_millis(200), || {})
            .expect("re-rearm on the current root");
        assert!(!unchanged, "槽位必須仍監看較新的 root");
    }

    #[test]
    fn resolve_watch_target_walks_up_from_project_subdir() {
        // 模擬自建置輸出目錄（如 target/release）為 cwd 啟動：起點在專案根
        // 之下，仍須向上探索出專案的 spec 目錄。
        let root = TempRoot::new("resolve-subdir");
        let sub = root.0.join("target").join("release");
        std::fs::create_dir_all(&sub).unwrap();
        let target = resolve_watch_targets(&sub).expect("subdir inside a project must resolve")[0].clone();
        assert_eq!(target, root.0.join("openspec"));
    }

    #[test]
    fn resolve_watch_target_respects_custom_spec_dir_name() {
        let dir =
            std::env::temp_dir().join(format!("speclink-watch-customdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("myspecs")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "spec_dir: myspecs\n").unwrap();
        let target = resolve_watch_targets(&dir).expect("project with custom spec dir must resolve")[0].clone();
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
            resolve_watch_targets(&dir).expect_err("non-project start must be a loggable error");
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
        let err = watch_openspec(&[ghost.join("openspec")], Duration::from_millis(100), || {});
        assert!(
            err.is_err(),
            "missing target must be a loggable error, not a panic"
        );

        // 上層目錄存在但監看目標本身不存在。
        let root =
            std::env::temp_dir().join(format!("speclink-watch-nospec-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let err = watch_openspec(&[root.join("openspec")], Duration::from_millis(100), || {});
        assert!(
            err.is_err(),
            "nonexistent target dir must be a loggable error"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

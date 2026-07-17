//! Speclink 桌面 app 的 Tauri 殼。
//!
//! 每個 #[tauri::command] 是對 speclink-desktop-core 的單行委派（薄包裝）——
//! 真正的邏輯與測試在 speclink-desktop-core，此層只做 IPC 接線。
//! Rust 側無 current-root 可變全域（workspace-session 決策 4）：所有讀寫
//! command 逐呼叫收 root，直通 desktop-core 的帶路徑函式；分頁切換不再改寫
//! 任何全域，前一分頁 in-flight 呼叫以其原 root 結算。

mod watch;

#[cfg(target_os = "macos")]
mod panel;

use std::path::PathBuf;

use serde_json::Value;
use tauri::{Emitter, Manager};

#[tauri::command]
fn list_changes(root: PathBuf) -> Value {
    speclink_desktop_core::query::list_changes_at(&root)
}

#[tauri::command]
fn list_specs(root: PathBuf) -> Value {
    speclink_desktop_core::query::list_specs_at(&root)
}

#[tauri::command]
fn status(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::query::status_at(&root, &change)
}

#[tauri::command]
fn document(root: PathBuf, change: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::document_at(&root, &change, &artifact)
}

#[tauri::command]
fn spec_document(root: PathBuf, capability: String) -> Option<String> {
    speclink_desktop_core::query::spec_document_at(&root, &capability)
}

#[tauri::command]
fn search_workspace(root: PathBuf, query: String) -> Value {
    speclink_desktop_core::search::search_workspace_at(&root, &query)
}

#[tauri::command]
fn change_capabilities(root: PathBuf, change: String) -> Vec<String> {
    speclink_desktop_core::query::change_capabilities_at(&root, &change)
}

#[tauri::command]
fn change_meta(root: PathBuf, change: String) -> Option<Value> {
    speclink_desktop_core::manage::change_meta_at(&root, &change)
}

#[tauri::command]
fn delete_change(root: PathBuf, change: String) -> Result<(), String> {
    speclink_desktop_core::manage::delete_change_at(&root, &change)
}

#[tauri::command]
// 寫入型 command 一律 async＋spawn_blocking（design D2）：完成路徑可能秒級
// （git spawn 在部分環境極慢），非 async command 會佔用主執行緒凍結整窗。
// 委派移至執行緒池；並發寫回由 desktop-core 的全域寫鎖序列化。
async fn set_task_done(
    root: PathBuf,
    change: String,
    task: String,
    done: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_task_done_at(&root, &change, &task, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn set_all_tasks(root: PathBuf, change: String, done: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_all_tasks_at(&root, &change, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn move_task(
    root: PathBuf,
    change: String,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::move_task_at(&root, &change, from, to, before)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn reorder_card(
    root: PathBuf,
    kind: String,
    id: String,
    prev_id: Option<String>,
    next_id: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::reorder_card_at(
            &root,
            &kind,
            &id,
            prev_id.as_deref(),
            next_id.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
fn validate(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::validate_at(&root, &change)
}

#[tauri::command]
fn analyze(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::analyze_at(&root, &change)
}

#[tauri::command]
fn archive(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::archive_at(&root, &change)
}

#[tauri::command]
fn archived_changes(root: PathBuf) -> Value {
    speclink_desktop_core::cache::archived_changes_at(&root)
}

#[tauri::command]
fn archived_document(root: PathBuf, dated_name: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::archived_document_at(&root, &dated_name, &artifact)
}

#[tauri::command]
fn archived_capabilities(root: PathBuf, dated_name: String) -> Vec<String> {
    speclink_desktop_core::query::archived_capabilities_at(&root, &dated_name)
}

#[tauri::command]
fn list_discussions(root: PathBuf) -> Value {
    speclink_desktop_core::discussions::list_discussions_at(&root)
}

#[tauri::command]
fn discussion_document(root: PathBuf, slug: String) -> Option<String> {
    speclink_desktop_core::discussions::discussion_document_at(&root, &slug)
}

#[tauri::command]
fn promote_discussion(root: PathBuf, slug: String, name: Option<String>) -> Result<Value, String> {
    speclink_desktop_core::discussions::promote_discussion_at(&root, &slug, name.as_deref())
}

#[tauri::command]
fn archive_discussion(root: PathBuf, slug: String) -> Result<Value, String> {
    speclink_desktop_core::discussions::archive_discussion_at(&root, &slug)
}

/// 監看器槽位：重掛時整顆替換（drop 舊監看即停止）。
type WatcherState = std::sync::Mutex<Option<watch::WorkspaceWatcher>>;

/// git 身分預熱（design D1）：首抓可能秒級（GUI 進程 spawn git 極慢的環境），
/// 掛監看（＝專案成為活躍）時背景執行緒先填快取——首次勾選不再付這筆成本。
/// 失敗靜默，完成路徑的 cached_git_identity 會自行補抓。
fn prewarm_identity(root: PathBuf) {
    std::thread::spawn(move || {
        let _ = speclink_desktop_core::manage::cached_git_identity(&root);
    });
}

/// 開啟專案＝純探測（決策 4）：只回報三態 payload，不改寫任何全域、不重掛
/// 監看——同一路徑重複呼叫冪等無副作用。監看跟隨由前端顯式 watch_workspace。
#[tauri::command]
fn open_project(path: String) -> Result<Value, String> {
    let probe = speclink_desktop_core::project::open_project_at(std::path::Path::new(&path))?;
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

#[tauri::command]
fn init_project(path: String, tools: Vec<String>) -> Result<Value, String> {
    let probe = speclink_desktop_core::project::init_project_at(std::path::Path::new(&path), &tools)?;
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

/// 啟動語境的預設目錄（決策 4 首啟路徑）：回傳行程啟動時的工作目錄——前端
/// 首啟無持久化分頁時據此顯式 openProjectAt（自專案目錄啟動的自動開啟語意
/// 凍結）。純讀、無任何可變全域。
#[tauri::command]
fn startup_dir() -> String {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .display()
        .to_string()
}

/// 專案統計（背景分頁徽章快照）：唯讀、收路徑參數。
#[tauri::command]
fn project_stats(path: String) -> Result<Value, String> {
    speclink_desktop_core::project::project_stats_at(std::path::Path::new(&path))
}

/// 監看重掛（決策 5）：顯式跟隨活躍 session——整顆替換單一 watcher 並預熱
/// git 身分；workspace-changed 事件 payload 為被監看的 root 字串（session 的
/// 事件來源據此過濾）。root 收字串並原樣回送，避免 PathBuf 往返改寫字面。
/// 監看不可用僅記錄、不回錯——app 照常、僅失去自動刷新（既有降級語意）。
#[tauri::command]
fn watch_workspace(app: tauri::AppHandle, root: String) {
    let root_path = PathBuf::from(&root);
    prewarm_identity(root_path.clone());
    let emitter = app.clone();
    let watcher = watch::resolve_watch_target(&root_path).and_then(|target| {
        watch::watch_openspec(&target, std::time::Duration::from_millis(400), move || {
            let _ = emitter.emit("workspace-changed", root.clone());
        })
    });
    if let Some(slot) = app.try_state::<WatcherState>() {
        *slot.lock().expect("watcher lock poisoned") = match watcher {
            Ok(w) => Some(w),
            Err(e) => {
                eprintln!("speclink-desktop: file watching unavailable: {e}");
                None
            }
        };
    }
}

#[tauri::command]
fn read_settings(root: PathBuf) -> Result<Value, String> {
    let snapshot = speclink_desktop_core::settings::read_settings_at(&root)?;
    serde_json::to_value(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_app_tools(root: PathBuf, tools: Vec<String>) -> Result<(), String> {
    speclink_desktop_core::settings::write_tools_at(&root, &tools)
}

#[tauri::command]
fn write_workflow_config(
    root: PathBuf,
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
) -> Result<(), String> {
    let fields =
        speclink_desktop_core::settings::WorkflowPolicyFields { locale, spec_locale, tdd, audit };
    speclink_desktop_core::settings::write_workflow_fields_at(&root, &fields)
}

/// 寫入 config.yaml 的「專案說明」與「產出規則」。`context: None`＝不動、
/// `Some(文字)`＝設值（空白即移除鍵，core 落實）；`rules: None`＝不動、
/// `Some(節序清單)`＝整份代換。政策欄位不受本 command 波及。
#[tauri::command]
fn write_workflow_content(
    root: PathBuf,
    context: Option<String>,
    rules: Option<Vec<(String, Vec<String>)>>,
) -> Result<(), String> {
    let edit = match context {
        Some(text) => speclink_desktop_core::settings::ContextEdit::Set(text),
        None => speclink_desktop_core::settings::ContextEdit::Keep,
    };
    speclink_desktop_core::settings::write_workflow_content_at(&root, &edit, rules.as_deref())
}

/// 系統匣面板 toggle（tray-status-menu「面板樣式（macOS）」）：macOS 委派
/// panel 模組；其他平台恆回 Err（面板樣式偏好在非 macOS 不可達，此為守門）。
#[cfg(target_os = "macos")]
#[tauri::command]
fn toggle_tray_panel(app: tauri::AppHandle) -> Result<(), String> {
    panel::toggle(&app)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn toggle_tray_panel() -> Result<(), String> {
    Err("tray panel is macOS-only".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init());
    // 面板樣式相依僅 macOS 註冊（design D6）：positioner 供 tray 相對定位、
    // nspanel 供不搶焦點的 NSPanel 容器。
    #[cfg(target_os = "macos")]
    let builder = builder
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_nspanel::init());
    builder
        .setup(|app| {
            // 監看槽位（決策 5）：啟動僅註冊空槽——前端還原分頁後以
            // watch_workspace 顯式掛上活躍專案（監看與資料載入同由前端
            // session 驅動；建立失敗僅記錄，app 照常、只失去自動刷新）。
            let slot: WatcherState = std::sync::Mutex::new(None);
            app.manage(slot);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_changes,
            list_specs,
            status,
            document,
            spec_document,
            search_workspace,
            change_capabilities,
            change_meta,
            delete_change,
            set_task_done,
            set_all_tasks,
            move_task,
            reorder_card,
            validate,
            analyze,
            archive,
            archived_changes,
            archived_document,
            archived_capabilities,
            list_discussions,
            discussion_document,
            promote_discussion,
            archive_discussion,
            open_project,
            init_project,
            startup_dir,
            project_stats,
            watch_workspace,
            read_settings,
            write_app_tools,
            write_workflow_config,
            write_workflow_content,
            toggle_tray_panel
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speclink desktop app");
}

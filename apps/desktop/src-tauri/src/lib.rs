//! Speclink 桌面 app 的 Tauri 殼。
//!
//! 每個 #[tauri::command] 是對 speclink-desktop-core 的單行委派（薄包裝）——
//! 真正的邏輯與測試在 speclink-desktop-core，此層只做 IPC 接線。

mod watch;

#[cfg(target_os = "macos")]
mod panel;

use std::path::PathBuf;

use serde_json::Value;
use tauri::{Emitter, Manager, State};

/// app 對其 openspec/ 專案根的執行語境。專案根自啟動時的工作目錄向上探索
/// （與查詢的 Workspace::discover 同源）；探索不到專案時退回工作目錄本身。
/// root 為執行期可變（開啟專案／點分頁切換）——鎖粒度僅止於讀取路徑，
/// desktop-core 維持無狀態、逐呼叫收 root（design D1）。
struct AppState {
    root: std::sync::Mutex<PathBuf>,
}

impl AppState {
    /// 鎖內複製當前 root，委派照舊逐呼叫傳路徑。
    fn root(&self) -> PathBuf {
        self.root.lock().expect("root lock poisoned").clone()
    }
}

#[tauri::command]
fn list_changes(state: State<AppState>) -> Value {
    speclink_desktop_core::query::list_changes_at(&state.root())
}

#[tauri::command]
fn list_specs(state: State<AppState>) -> Value {
    speclink_desktop_core::query::list_specs_at(&state.root())
}

#[tauri::command]
fn status(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::query::status_at(&state.root(), &change)
}

#[tauri::command]
fn document(state: State<AppState>, change: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::document_at(&state.root(), &change, &artifact)
}

#[tauri::command]
fn spec_document(state: State<AppState>, capability: String) -> Option<String> {
    speclink_desktop_core::query::spec_document_at(&state.root(), &capability)
}

#[tauri::command]
fn search_workspace(state: State<AppState>, query: String) -> Value {
    speclink_desktop_core::search::search_workspace_at(&state.root(), &query)
}

#[tauri::command]
fn change_capabilities(state: State<AppState>, change: String) -> Vec<String> {
    speclink_desktop_core::query::change_capabilities_at(&state.root(), &change)
}

#[tauri::command]
fn change_meta(state: State<AppState>, change: String) -> Option<Value> {
    speclink_desktop_core::manage::change_meta_at(&state.root(), &change)
}

#[tauri::command]
fn delete_change(state: State<AppState>, change: String) -> Result<(), String> {
    speclink_desktop_core::manage::delete_change_at(&state.root(), &change)
}

#[tauri::command]
// 寫入型 command 一律 async＋spawn_blocking（design D2）：完成路徑可能秒級
// （git spawn 在部分環境極慢），非 async command 會佔用主執行緒凍結整窗。
// 委派移至執行緒池；並發寫回由 desktop-core 的全域寫鎖序列化。
async fn set_task_done(
    state: State<'_, AppState>,
    change: String,
    task: String,
    done: bool,
) -> Result<(), String> {
    let root = state.root();
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_task_done_at(&root, &change, &task, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn set_all_tasks(state: State<'_, AppState>, change: String, done: bool) -> Result<(), String> {
    let root = state.root();
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_all_tasks_at(&root, &change, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn move_task(
    state: State<'_, AppState>,
    change: String,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    let root = state.root();
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::move_task_at(&root, &change, from, to, before)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn reorder_card(
    state: State<'_, AppState>,
    kind: String,
    id: String,
    prev_id: Option<String>,
    next_id: Option<String>,
) -> Result<(), String> {
    let root = state.root();
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
fn validate(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::validate_at(&state.root(), &change)
}

#[tauri::command]
fn analyze(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::analyze_at(&state.root(), &change)
}

#[tauri::command]
fn archive(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::archive_at(&state.root(), &change)
}

#[tauri::command]
fn archived_changes(state: State<AppState>) -> Value {
    speclink_desktop_core::cache::archived_changes_at(&state.root())
}

#[tauri::command]
fn archived_document(state: State<AppState>, dated_name: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::archived_document_at(&state.root(), &dated_name, &artifact)
}

#[tauri::command]
fn archived_capabilities(state: State<AppState>, dated_name: String) -> Vec<String> {
    speclink_desktop_core::query::archived_capabilities_at(&state.root(), &dated_name)
}

#[tauri::command]
fn list_discussions(state: State<AppState>) -> Value {
    speclink_desktop_core::discussions::list_discussions_at(&state.root())
}

#[tauri::command]
fn discussion_document(state: State<AppState>, slug: String) -> Option<String> {
    speclink_desktop_core::discussions::discussion_document_at(&state.root(), &slug)
}

#[tauri::command]
fn promote_discussion(state: State<AppState>, slug: String, name: Option<String>) -> Result<Value, String> {
    speclink_desktop_core::discussions::promote_discussion_at(&state.root(), &slug, name.as_deref())
}

#[tauri::command]
fn archive_discussion(state: State<AppState>, slug: String) -> Result<Value, String> {
    speclink_desktop_core::discussions::archive_discussion_at(&state.root(), &slug)
}

/// 監看器槽位：切換專案時整顆替換（drop 舊監看即停止）。
type WatcherState = std::sync::Mutex<Option<watch::WorkspaceWatcher>>;

/// git 身分預熱（design D1）：首抓可能秒級（GUI 進程 spawn git 極慢的環境），
/// 啟動與切根時背景執行緒先填快取——首次勾選不再付這筆成本。失敗靜默，
/// 完成路徑的 cached_git_identity 會自行補抓。
fn prewarm_identity(root: PathBuf) {
    std::thread::spawn(move || {
        let _ = speclink_desktop_core::manage::cached_git_identity(&root);
    });
}

/// 切換專案 root：更新 AppState 並對新 root 重掛 spec 目錄監看。
/// 監看重掛失敗僅記錄、不阻斷切換（與啟動時的降級行為一致）。
fn switch_root(app: &tauri::AppHandle, state: &AppState, new_root: PathBuf) {
    *state.root.lock().expect("root lock poisoned") = new_root.clone();
    prewarm_identity(new_root.clone());
    let emitter = app.clone();
    let watcher = watch::resolve_watch_target(&new_root).and_then(|target| {
        watch::watch_openspec(&target, std::time::Duration::from_millis(400), move || {
            let _ = emitter.emit("workspace-changed", ());
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
fn open_project(app: tauri::AppHandle, state: State<AppState>, path: String) -> Result<Value, String> {
    let probe = speclink_desktop_core::project::open_project_at(std::path::Path::new(&path))?;
    if let speclink_desktop_core::project::ProjectProbe::Project { ref root, .. } = probe {
        switch_root(&app, &state, PathBuf::from(root));
    }
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

#[tauri::command]
fn init_project(
    app: tauri::AppHandle,
    state: State<AppState>,
    path: String,
    tools: Vec<String>,
) -> Result<Value, String> {
    let probe = speclink_desktop_core::project::init_project_at(std::path::Path::new(&path), &tools)?;
    if let speclink_desktop_core::project::ProjectProbe::Project { ref root, .. } = probe {
        switch_root(&app, &state, PathBuf::from(root));
    }
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

#[tauri::command]
fn current_project(state: State<AppState>) -> Value {
    let root = state.root();
    serde_json::json!({
        "root": root.display().to_string(),
        "name": speclink_desktop_core::project::project_name(&root),
    })
}

#[tauri::command]
fn project_stats(path: String) -> Result<Value, String> {
    speclink_desktop_core::project::project_stats_at(std::path::Path::new(&path))
}

#[tauri::command]
fn read_settings(state: State<AppState>) -> Result<Value, String> {
    let snapshot = speclink_desktop_core::settings::read_settings_at(&state.root())?;
    serde_json::to_value(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_app_tools(state: State<AppState>, tools: Vec<String>) -> Result<(), String> {
    speclink_desktop_core::settings::write_tools_at(&state.root(), &tools)
}

#[tauri::command]
fn write_workflow_config(
    state: State<AppState>,
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
) -> Result<(), String> {
    let fields =
        speclink_desktop_core::settings::WorkflowPolicyFields { locale, spec_locale, tdd, audit };
    speclink_desktop_core::settings::write_workflow_fields_at(&state.root(), &fields)
}

/// 寫入 config.yaml 的「專案說明」與「產出規則」。`context: None`＝不動、
/// `Some(文字)`＝設值（空白即移除鍵，core 落實）；`rules: None`＝不動、
/// `Some(節序清單)`＝整份代換。政策欄位不受本 command 波及。
#[tauri::command]
fn write_workflow_content(
    state: State<AppState>,
    context: Option<String>,
    rules: Option<Vec<(String, Vec<String>)>>,
) -> Result<(), String> {
    let edit = match context {
        Some(text) => speclink_desktop_core::settings::ContextEdit::Set(text),
        None => speclink_desktop_core::settings::ContextEdit::Keep,
    };
    speclink_desktop_core::settings::write_workflow_content_at(
        &state.root(),
        &edit,
        rules.as_deref(),
    )
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
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // 監看根解析與專案探索一致：自 cwd 向上探索出實際專案根，監看與查詢共用
    // 同一根語意——自檔案總管雙擊啟動（cwd 為 exe 所在目錄）時自動刷新仍生效。
    // 探索不到專案時退回 cwd：app 照常、僅無自動刷新（維持既有降級行為）。
    let root = speclink_desktop_core::init_core_context(&cwd)
        .map(|ctx| ctx.workspace.root)
        .unwrap_or(cwd);
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
        .setup(move |app| {
            // spec 目錄監看：外部寫者（CLI、agent、編輯器）的變更去抖後以
            // workspace-changed 事件通知前端整批 refresh。建立失敗只記錄——
            // app 照常提供其餘功能，僅失去自動刷新（spec：監看不可用時功能照常）。
            let handle = app.handle().clone();
            let watcher = watch::resolve_watch_target(&root).and_then(|target| {
                watch::watch_openspec(&target, std::time::Duration::from_millis(400), move || {
                    let _ = handle.emit("workspace-changed", ());
                })
            });
            // 槽位無論成敗都註冊——切換專案時 switch_root 才有地方重掛。
            let slot: WatcherState = std::sync::Mutex::new(match watcher {
                Ok(w) => Some(w),
                Err(e) => {
                    eprintln!("speclink-desktop: file watching unavailable: {e}");
                    None
                }
            });
            app.manage(slot);
            prewarm_identity(root.clone());
            app.manage(AppState { root: std::sync::Mutex::new(root) });
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
            current_project,
            project_stats,
            read_settings,
            write_app_tools,
            write_workflow_config,
            write_workflow_content,
            toggle_tray_panel
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speclink desktop app");
}

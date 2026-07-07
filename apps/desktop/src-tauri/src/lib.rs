//! Speclink 桌面 app 的 Tauri 殼。
//!
//! 每個 #[tauri::command] 是對 speclink-desktop-core 的單行委派（薄包裝）——
//! 真正的邏輯與測試在 speclink-desktop-core，此層只做 IPC 接線。

mod watch;

use std::path::PathBuf;

use serde_json::Value;
use tauri::{Emitter, Manager, State};

/// app 對其 openspec/ 專案根的執行語境。專案根自啟動時的工作目錄向上探索
/// （與查詢的 Workspace::discover 同源）；探索不到專案時退回工作目錄本身。
struct AppState {
    root: PathBuf,
}

#[tauri::command]
fn list_changes(state: State<AppState>) -> Value {
    speclink_desktop_core::query::list_changes_at(&state.root)
}

#[tauri::command]
fn list_specs(state: State<AppState>) -> Value {
    speclink_desktop_core::query::list_specs_at(&state.root)
}

#[tauri::command]
fn status(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::query::status_at(&state.root, &change)
}

#[tauri::command]
fn document(state: State<AppState>, change: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::document_at(&state.root, &change, &artifact)
}

#[tauri::command]
fn spec_document(state: State<AppState>, capability: String) -> Option<String> {
    speclink_desktop_core::query::spec_document_at(&state.root, &capability)
}

#[tauri::command]
fn change_capabilities(state: State<AppState>, change: String) -> Vec<String> {
    speclink_desktop_core::query::change_capabilities_at(&state.root, &change)
}

#[tauri::command]
fn change_meta(state: State<AppState>, change: String) -> Option<Value> {
    speclink_desktop_core::manage::change_meta_at(&state.root, &change)
}

#[tauri::command]
fn delete_change(state: State<AppState>, change: String) -> Result<(), String> {
    speclink_desktop_core::manage::delete_change_at(&state.root, &change)
}

#[tauri::command]
fn set_task_done(state: State<AppState>, change: String, ordinal: usize, done: bool) -> Result<(), String> {
    speclink_desktop_core::manage::set_task_done_at(&state.root, &change, ordinal, done)
}

#[tauri::command]
fn move_task(
    state: State<AppState>,
    change: String,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    speclink_desktop_core::manage::move_task_at(&state.root, &change, from, to, before)
}

#[tauri::command]
fn validate(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::validate_at(&state.root, &change)
}

#[tauri::command]
fn analyze(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::analyze_at(&state.root, &change)
}

#[tauri::command]
fn archive(state: State<AppState>, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::archive_at(&state.root, &change)
}

#[tauri::command]
fn archived_changes(state: State<AppState>) -> Value {
    speclink_desktop_core::cache::archived_changes_at(&state.root)
}

#[tauri::command]
fn archived_document(state: State<AppState>, dated_name: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::archived_document_at(&state.root, &dated_name, &artifact)
}

#[tauri::command]
fn archived_capabilities(state: State<AppState>, dated_name: String) -> Vec<String> {
    speclink_desktop_core::query::archived_capabilities_at(&state.root, &dated_name)
}

#[tauri::command]
fn list_discussions(state: State<AppState>) -> Value {
    speclink_desktop_core::discussions::list_discussions_at(&state.root)
}

#[tauri::command]
fn discussion_document(state: State<AppState>, slug: String) -> Option<String> {
    speclink_desktop_core::discussions::discussion_document_at(&state.root, &slug)
}

#[tauri::command]
fn promote_discussion(state: State<AppState>, slug: String, name: Option<String>) -> Result<Value, String> {
    speclink_desktop_core::discussions::promote_discussion_at(&state.root, &slug, name.as_deref())
}

#[tauri::command]
fn archive_discussion(state: State<AppState>, slug: String) -> Result<Value, String> {
    speclink_desktop_core::discussions::archive_discussion_at(&state.root, &slug)
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
    tauri::Builder::default()
        .setup(move |app| {
            // spec 目錄監看：外部寫者（CLI、agent、編輯器）的變更去抖後以
            // workspace-changed 事件通知前端整批 refresh。建立失敗只記錄——
            // app 照常提供其餘功能，僅失去自動刷新（spec：監看不可用時功能照常）。
            let handle = app.handle().clone();
            match watch::resolve_watch_target(&root).and_then(|target| {
                watch::watch_openspec(&target, std::time::Duration::from_millis(400), move || {
                    let _ = handle.emit("workspace-changed", ());
                })
            }) {
                Ok(watcher) => {
                    app.manage(std::sync::Mutex::new(Some(watcher)));
                }
                Err(e) => eprintln!("speclink-desktop: file watching unavailable: {e}"),
            }
            app.manage(AppState { root });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_changes,
            list_specs,
            status,
            document,
            spec_document,
            change_capabilities,
            change_meta,
            delete_change,
            set_task_done,
            move_task,
            validate,
            analyze,
            archive,
            archived_changes,
            archived_document,
            archived_capabilities,
            list_discussions,
            discussion_document,
            promote_discussion,
            archive_discussion
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speclink desktop app");
}

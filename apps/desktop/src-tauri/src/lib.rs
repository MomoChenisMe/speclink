//! Speclink 桌面 app 的 Tauri 殼。
//!
//! 每個 #[tauri::command] 是對 speclink-desktop-core 的單行委派（薄包裝）——
//! 真正的邏輯與測試在 speclink-desktop-core，此層只做 IPC 接線。

use std::path::PathBuf;

use serde_json::Value;
use tauri::{Manager, State};

/// app 對其 openspec/ 專案根的執行語境。v1 以啟動時的工作目錄為專案根。
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
fn move_task(state: State<AppState>, change: String, from: usize, to: usize) -> Result<(), String> {
    speclink_desktop_core::manage::move_task_at(&state.root, &change, from, to)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    tauri::Builder::default()
        .setup(move |app| {
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
            archived_changes
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speclink desktop app");
}

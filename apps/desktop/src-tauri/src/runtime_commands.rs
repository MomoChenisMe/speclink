//! IPC adapters for device preferences and diagnostics.
use speclink_desktop_core::{
    diagnostics::ExportReceipt,
    runtime_preferences::{DeviceRuntime, RuntimePreferences, RuntimeSnapshot},
};
use speclink_host::network::NetworkDiagnostic;
use std::path::PathBuf;
#[derive(Debug, serde::Serialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
}
impl From<String> for RuntimeError {
    fn from(code: String) -> Self {
        Self {
            message: code.clone(),
            code,
        }
    }
}
fn worker_error(_: tauri::Error) -> RuntimeError {
    "runtime_worker_failed".to_owned().into()
}
#[tauri::command]
pub async fn runtime_preferences_read(
    runtime: tauri::State<'_, DeviceRuntime>,
) -> Result<RuntimeSnapshot, RuntimeError> {
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.read())
        .await
        .map_err(worker_error)
}
#[tauri::command]
pub async fn runtime_preferences_write(
    runtime: tauri::State<'_, DeviceRuntime>,
    preferences: RuntimePreferences,
) -> Result<RuntimeSnapshot, RuntimeError> {
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || runtime.write(preferences))
        .await
        .map_err(worker_error)?
        .map_err(Into::into)
}
#[tauri::command]
pub fn diagnostics_append(
    runtime: tauri::State<'_, DeviceRuntime>,
    mut event: NetworkDiagnostic,
) -> Result<(), RuntimeError> {
    if matches!(event.area, speclink_host::network::DiagnosticArea::Frontend)
        && event.operation_id.is_empty()
    {
        event.operation_id = ulid::Ulid::new().to_string();
    }
    runtime.diagnostics.record(event).map_err(Into::into)
}
#[tauri::command]
pub async fn diagnostics_export(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, DeviceRuntime>,
    path: PathBuf,
) -> Result<ExportReceipt, RuntimeError> {
    let runtime = runtime.inner().clone();
    let version = app.package_info().version.to_string();
    tauri::async_runtime::spawn_blocking(move || runtime.diagnostics.export(&path, &version))
        .await
        .map_err(worker_error)?
        .map_err(Into::into)
}
pub fn update_endpoints(app: &tauri::AppHandle) -> Vec<String> {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|v| v.get("endpoints"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
#[tauri::command]
pub async fn network_probe_update(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, DeviceRuntime>,
) -> Result<speclink_desktop_core::network::ProbeResult, RuntimeError> {
    let runtime = runtime.inner().clone();
    let endpoints = update_endpoints(&app);
    let worker = runtime.clone();
    let policy = tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::network::prepare_probe(&worker)
    })
    .await
    .map_err(worker_error)?
    .map_err(RuntimeError::from)?;
    speclink_desktop_core::network::probe_update(&runtime, policy, endpoints)
        .await
        .map_err(Into::into)
}

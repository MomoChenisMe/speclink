//! Tauri updater adapter. Routing and diagnostics are shared with other device operations.
use super::runtime_commands::RuntimeError;
use speclink_desktop_core::runtime_preferences::DeviceRuntime;
use speclink_host::network::{DiagnosticAction, DiagnosticArea, NetworkDiagnostic};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::State;
use tauri_plugin_updater::{Update, UpdaterExt};
#[derive(Default)]
pub struct UpdaterJobs(Mutex<Option<(String, Arc<Candidate>)>>);
struct Candidate {
    policy: speclink_host::network::ProxyPolicy,
    update: Update,
    bytes: Mutex<Option<Vec<u8>>>,
    busy: AtomicBool,
}
#[derive(serde::Serialize)]
pub struct CandidateInfo {
    id: String,
    version: String,
}
fn outcome(
    mut event: NetworkDiagnostic,
    error: Option<&tauri_plugin_updater::Error>,
    elapsed: std::time::Duration,
) -> NetworkDiagnostic {
    event.duration_ms = Some(elapsed.as_millis() as u64);
    event.event = if error.is_some() {
        DiagnosticAction::Failed
    } else {
        DiagnosticAction::Completed
    };
    if let Some(error) = error {
        let initial = if let tauri_plugin_updater::Error::Reqwest(error) = error {
            event.http_status = error.status().map(|s| s.as_u16());
            if error.is_timeout() {
                "timeout"
            } else if error.is_connect() {
                "connect"
            } else {
                "unknown"
            }
        } else {
            "unknown"
        };
        let (code, chain) = speclink_host::network::error_details(error, initial);
        event.error_code = Some(code);
        event.error_chain = chain;
    } else {
        event.error_code = Some("updater-response-unavailable".into());
    }
    event
}
fn lookup(jobs: &UpdaterJobs, id: &str) -> Result<Arc<Candidate>, RuntimeError> {
    jobs.0
        .lock()
        .unwrap()
        .as_ref()
        .filter(|(key, _)| key == id)
        .map(|(_, c)| c.clone())
        .ok_or_else(|| "unknown_update".to_owned().into())
}
#[tauri::command]
pub async fn updater_check(
    app: tauri::AppHandle,
    runtime: State<'_, DeviceRuntime>,
    jobs: State<'_, UpdaterJobs>,
) -> Result<Option<CandidateInfo>, RuntimeError> {
    let owned = runtime.inner().clone();
    let policy = tauri::async_runtime::spawn_blocking(move || owned.policy())
        .await
        .map_err(|_| RuntimeError::from("runtime_worker_failed".to_owned()))?
        .map_err(RuntimeError::from)?;
    let configured = policy.clone();
    // Reject unsupported routes before entering reqwest's infallible custom selector.
    let _ = configured
        .configure_client(reqwest::Client::builder())
        .map_err(RuntimeError::from)?;
    let updater = app
        .updater_builder()
        .no_proxy()
        .configure_client(move |builder| {
            configured
                .configure_client(builder)
                .expect("validated immutable proxy policy")
        })
        .build()
        .map_err(|_| RuntimeError::from("update_failed".to_owned()))?;
    let id = ulid::Ulid::new().to_string();
    let start = std::time::Instant::now();
    let mut event = NetworkDiagnostic::new(
        DiagnosticArea::Updater,
        DiagnosticAction::Started,
        id.clone(),
    );
    event.proxy_mode = Some(policy.settings.mode.clone());
    let endpoints = super::runtime_commands::update_endpoints(&app);
    if endpoints.len() == 1 {
        event.destination = endpoints.first().cloned();
        event
            .set_route(&policy, &endpoints[0])
            .map_err(RuntimeError::from)?;
    }
    let _ = runtime.diagnostics.record(event.clone());
    let result = updater.check().await;
    let mut end = outcome(event, result.as_ref().err(), start.elapsed());
    if let Some(code) = policy.client_error() {
        end.error_code = Some(code.into());
        end.event = DiagnosticAction::Failed;
    }
    let _ = runtime.diagnostics.record(end);
    if let Some(code) = policy.client_error() {
        return Err(code.to_owned().into());
    }
    let Some(update) = result.map_err(|_| RuntimeError::from("update_failed".to_owned()))? else {
        return Ok(None);
    };
    let version = update.version.clone();
    let candidate = Arc::new(Candidate {
        policy,
        update,
        bytes: Mutex::new(None),
        busy: AtomicBool::new(false),
    });
    *jobs.0.lock().unwrap() = Some((id.clone(), candidate));
    Ok(Some(CandidateInfo { id, version }))
}
#[tauri::command]
pub async fn updater_download(
    runtime: State<'_, DeviceRuntime>,
    jobs: State<'_, UpdaterJobs>,
    id: String,
) -> Result<(), RuntimeError> {
    let candidate = lookup(&jobs, &id)?;
    if candidate.busy.swap(true, Ordering::AcqRel) {
        return Err("busy".to_owned().into());
    }
    let start = std::time::Instant::now();
    let mut event = NetworkDiagnostic::new(DiagnosticArea::Updater, DiagnosticAction::Started, id);
    event.destination = Some(candidate.update.download_url.to_string());
    event.proxy_mode = Some(candidate.policy.settings.mode.clone());
    if let Err(code) = event.set_route(&candidate.policy, candidate.update.download_url.as_str()) {
        candidate.busy.store(false, Ordering::Release);
        return Err(code.into());
    }
    let _ = runtime.diagnostics.record(event.clone());
    let result = candidate.update.download(|_, _| {}, || {}).await;
    let mut end = outcome(event, result.as_ref().err(), start.elapsed());
    if let Some(code) = candidate.policy.client_error() {
        end.error_code = Some(code.into());
        end.event = DiagnosticAction::Failed;
    }
    let _ = runtime.diagnostics.record(end);
    let outcome = if let Some(code) = candidate.policy.client_error() {
        Err(code.to_owned().into())
    } else {
        match result {
            Ok(bytes) => {
                *candidate.bytes.lock().unwrap() = Some(bytes);
                Ok(())
            }
            Err(_) => Err(RuntimeError::from("update_failed".to_owned())),
        }
    };
    candidate.busy.store(false, Ordering::Release);
    outcome
}
#[tauri::command]
pub async fn updater_install(jobs: State<'_, UpdaterJobs>, id: String) -> Result<(), RuntimeError> {
    let candidate = lookup(&jobs, &id)?;
    let bytes = candidate
        .bytes
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| RuntimeError::from("update_not_downloaded".to_owned()))?;
    tauri::async_runtime::spawn_blocking(move || candidate.update.install(bytes))
        .await
        .map_err(|_| RuntimeError::from("runtime_worker_failed".to_owned()))?
        .map_err(|_| RuntimeError::from("update_failed".to_owned()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn updater_does_not_invent_http_status_from_error_messages() {
        let event = NetworkDiagnostic::new(
            DiagnosticArea::Updater,
            DiagnosticAction::Started,
            ulid::Ulid::new().to_string(),
        );
        let error = tauri_plugin_updater::Error::Network("secret URL returned 407".into());
        let result = outcome(event, Some(&error), std::time::Duration::ZERO);
        assert_eq!(result.http_status, None);
        assert_eq!(result.error_code.as_deref(), Some("unknown"));
        assert!(!serde_json::to_string(&result).unwrap().contains("secret"));
    }
}

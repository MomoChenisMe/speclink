//! Thin Desktop boundary: registry lookup is local; only explicit prepare can fetch.
use crate::{connection_origin, ConnectionsState};
use serde::Deserialize;
use speclink_desktop_core::security::{self, JobError, JobTicket, ReviewTargetInput, SecurityJobs};
use std::{path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Target {
    Local {
        root: PathBuf,
    },
    Remote {
        #[serde(rename = "connectionId")]
        connection_id: String,
        project: String,
        repo: String,
        #[serde(rename = "checkoutRoot")]
        checkout_root: Option<PathBuf>,
    },
}
impl Target {
    fn resolve(&self, app: &tauri::AppHandle) -> Result<ReviewTargetInput, JobError> {
        Ok(match self {
            Self::Local { root } => ReviewTargetInput::Local { root: root.clone() },
            Self::Remote {
                connection_id,
                project,
                repo,
                checkout_root,
            } => ReviewTargetInput::Remote {
                origin: connection_origin(app, connection_id)
                    .map_err(|_| JobError::review("binding_mismatch"))?,
                project: project.clone(),
                repo: repo.clone(),
                checkout_root: checkout_root.clone(),
            },
        })
    }
}
fn data_dir() -> Result<PathBuf, JobError> {
    speclink_host::context::security_data_dir().map_err(|_| JobError::directory_unavailable())
}
#[tauri::command]
pub async fn security_inspect(
    app: tauri::AppHandle,
    target: Target,
    change: Option<String>,
) -> Result<security::ReviewInspection, JobError> {
    tauri::async_runtime::spawn_blocking(move || {
        security::inspect_review_at(&data_dir()?, &target.resolve(&app)?, change)
    })
    .await
    .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_start(
    app: tauri::AppHandle,
    jobs: tauri::State<'_, SecurityJobs>,
    target: Target,
    change: Option<String>,
) -> Result<JobTicket, JobError> {
    let jobs = jobs.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let input = target.resolve(&app)?;
        let progress_app = app.clone();
        jobs.start_review_at(
            data_dir()?,
            input,
            change,
            move |p| {
                let _ = progress_app.emit("security-progress", p);
            },
            move |p| {
                let _ = app.emit("security-completed", p);
            },
        )
    })
    .await
    .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_prepare(
    app: tauri::AppHandle,
    jobs: tauri::State<'_, SecurityJobs>,
    target: Target,
    change: Option<String>,
) -> Result<JobTicket, JobError> {
    let jobs = jobs.inner().clone();
    let state = app.state::<Arc<ConnectionsState>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let input = target.resolve(&app)?;
        jobs.start_prepare_at(
            data_dir()?,
            input,
            change,
            move |review, budget| match &target {
                Target::Remote {
                    connection_id,
                    project,
                    repo,
                    ..
                } => {
                    let origin = review
                        .binding
                        .origin
                        .as_deref()
                        .ok_or_else(|| JobError::review("binding_mismatch"))?;
                    let manager = state.manager_for_connection(connection_id, origin);
                    crate::remote::RemoteWorkspace::at(origin, project, repo, &manager)
                        .security_preparation(state.credentials, review, budget)
                }
                _ => Err(JobError::review("binding_mismatch")),
            },
            move |p| {
                let _ = app.emit("security-completed", p);
            },
        )
    })
    .await
    .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_wait(
    jobs: tauri::State<'_, SecurityJobs>,
    job_id: String,
    project_key: String,
) -> Result<serde_json::Value, JobError> {
    let jobs = jobs.inner().clone();
    tauri::async_runtime::spawn_blocking(move || jobs.wait(&job_id, &project_key))
        .await
        .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_report(
    id: String,
) -> Result<speclink_core::security::report::Report, JobError> {
    tauri::async_runtime::spawn_blocking(move || security::read_report_at(&data_dir()?, &id))
        .await
        .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_report_summary(id: String) -> Result<String, JobError> {
    tauri::async_runtime::spawn_blocking(move || {
        security::read_report_at(&data_dir()?, &id).map(|r| r.summary())
    })
    .await
    .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_report_export(
    id: String,
    format: String,
    path: PathBuf,
) -> Result<(), JobError> {
    tauri::async_runtime::spawn_blocking(move || {
        security::export_report_at(&data_dir()?, &id, &format, &path)
    })
    .await
    .map_err(|_| JobError::worker_failed())?
}
#[tauri::command]
pub async fn security_reveal(
    app: tauri::AppHandle,
    target: Target,
    id: String,
    path: String,
    line: u64,
) -> Result<(), JobError> {
    let resolver = app.clone();
    let _ = line;
    let path = tauri::async_runtime::spawn_blocking(move || {
        security::reveal_report_path_at(&data_dir()?, &target.resolve(&resolver)?, &id, &path)
    })
    .await
    .map_err(|_| JobError::worker_failed())??;
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|_| JobError::review("source_unavailable"))
}

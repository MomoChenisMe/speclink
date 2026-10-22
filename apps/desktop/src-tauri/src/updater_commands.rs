//! Thin Tauri bindings; the core owns policy snapshots, jobs and diagnostics.
use speclink_desktop_core::{
    runtime_preferences::DeviceRuntime,
    updater::{self, Candidate, CandidateInfo, Job, UpdateError, UpdateFailure, UpdaterBackend},
};
use speclink_host::network::ProxyPolicy;
use std::sync::Arc;
use tauri::State;
use tauri_plugin_updater::{Update, UpdaterExt};

pub type UpdaterJobs = updater::UpdaterJobs<Update>;
struct PluginUpdater(tauri::AppHandle);

fn failure(error: tauri_plugin_updater::Error) -> UpdateFailure {
    use tauri_plugin_updater::Error;
    if let Error::Reqwest(error) = &error {
        let (code, http_status, chain) = speclink_host::network::request_error(error);
        return UpdateFailure {
            code,
            http_status,
            chain,
        };
    }
    let code = match &error {
        Error::Serialization(_)
        | Error::ReleaseNotFound
        | Error::TargetNotFound(_)
        | Error::TargetsNotFound(_) => "parse",
        Error::EmptyEndpoints | Error::UnsupportedArch | Error::UnsupportedOs => {
            "updater-unavailable"
        }
        Error::Network(_) => "http",
        _ => "update_failed",
    };
    UpdateFailure::code(code)
}

impl UpdaterBackend for PluginUpdater {
    type Update = Update;
    async fn check(&self, policy: ProxyPolicy) -> Result<Option<Candidate<Update>>, UpdateFailure> {
        let updater = self
            .0
            .updater_builder()
            .no_proxy()
            .configure_client(move |builder| {
                policy
                    .configure_client(builder)
                    .expect("validated immutable proxy policy")
            })
            .build()
            .map_err(failure)?;
        Ok(updater
            .check()
            .await
            .map_err(failure)?
            .map(|update| Candidate {
                version: update.version.clone(),
                download_url: update.download_url.to_string(),
                update,
            }))
    }
    async fn download(&self, update: &Update) -> Result<Vec<u8>, UpdateFailure> {
        update.download(|_, _| {}, || {}).await.map_err(failure)
    }
    async fn install(&self, job: Arc<Job<Update>>, bytes: Vec<u8>) -> Result<(), UpdateFailure> {
        tauri::async_runtime::spawn_blocking(move || job.update.install(bytes))
            .await
            .map_err(|_| UpdateFailure::code("runtime_worker_failed"))?
            .map_err(failure)
    }
}

#[tauri::command]
pub async fn desktop_release(
    runtime: State<'_, DeviceRuntime>,
    repo: updater::ReleaseRepository,
) -> Result<Option<updater::DesktopRelease>, UpdateError> {
    updater::desktop_release(&runtime, repo).await
}
#[tauri::command]
pub async fn updater_check(
    app: tauri::AppHandle,
    runtime: State<'_, DeviceRuntime>,
    jobs: State<'_, UpdaterJobs>,
) -> Result<Option<CandidateInfo>, UpdateError> {
    updater::check(
        &PluginUpdater(app.clone()),
        &runtime,
        &jobs,
        super::runtime_commands::update_endpoints(&app),
    )
    .await
}
#[tauri::command]
pub async fn updater_download(
    app: tauri::AppHandle,
    runtime: State<'_, DeviceRuntime>,
    jobs: State<'_, UpdaterJobs>,
    id: String,
) -> Result<(), UpdateError> {
    updater::download(&PluginUpdater(app), &runtime, &jobs, &id).await
}
#[tauri::command]
pub async fn updater_install(
    app: tauri::AppHandle,
    jobs: State<'_, UpdaterJobs>,
    id: String,
) -> Result<(), UpdateError> {
    updater::install(&PluginUpdater(app), &jobs, &id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn updater_does_not_invent_http_status_from_error_messages() {
        let result = failure(tauri_plugin_updater::Error::Network(
            "secret URL returned 407".into(),
        ));
        assert_eq!(result.http_status, None);
        assert_eq!(result.code, "http");
        assert!(!serde_json::to_string(&result.chain)
            .unwrap()
            .contains("secret"));
    }
}

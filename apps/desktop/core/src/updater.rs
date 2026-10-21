//! Update job lifecycle and diagnostic settlement, independent of the desktop shell.
#[cfg(test)]
mod release_metadata_tests {
    use super::*;
    use serde_json::json;

    fn release(repo: &str, tag: &str, date: &str) -> serde_json::Value {
        json!({"tag_name":tag,"published_at":date,"draft":false,"prerelease":false,
            "html_url":format!("https://github.com/{repo}/releases/tag/{tag}"),
            "assets":[{"name":"Speclink_0.10.0.4_x64-setup.exe"}]})
    }
    #[test]
    fn desktop_release_filters_cli_drafts_and_prereleases_and_keeps_repos_separate() {
        let mut draft = release("mosutw/speclink", "desktop-v9.0.0", "2026-10-23T00:00:00Z");
        draft["draft"] = json!(true);
        let mut prerelease = draft.clone();
        prerelease["draft"] = json!(false);
        prerelease["prerelease"] = json!(true);
        let releases = json!([
            draft,
            prerelease,
            release("mosutw/speclink", "v10.0.0", "2026-10-22T00:00:00Z"),
            release(
                "mosutw/speclink",
                "desktop-v0.10.0.4",
                "2026-10-21T00:00:00Z"
            )
        ]);
        let selected = select_desktop_release(ReleaseRepository::Fork, &releases)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(selected.version, "0.10.0.4");
        assert_eq!(selected.repo_url, "https://github.com/mosutw/speclink");
        assert!(select_desktop_release(ReleaseRepository::Upstream, &releases).is_err());
    }
    #[test]
    fn release_lookup_is_bounded_and_requires_real_desktop_assets() {
        let mut releases = vec![release("mosutw/speclink", "v1.0.0", "2026-10-21T00:00:00Z"); 100];
        releases.push(release(
            "mosutw/speclink",
            "desktop-v0.10.0.4",
            "2026-10-21T00:00:00Z",
        ));
        assert!(
            select_desktop_release(ReleaseRepository::Fork, &json!(releases))
                .ok()
                .flatten()
                .is_none()
        );
        let mut no_assets = release(
            "mosutw/speclink",
            "desktop-v0.10.0.4",
            "2026-10-21T00:00:00Z",
        );
        no_assets["assets"] = json!([]);
        assert!(
            select_desktop_release(ReleaseRepository::Fork, &json!([no_assets]))
                .ok()
                .flatten()
                .is_none()
        );
    }
    #[test]
    fn release_http_errors_keep_auth_rate_limits_and_missing_descriptor_distinct() {
        for (status, code) in [
            (407, "proxy-auth"),
            (403, "http"),
            (429, "http"),
            (404, "http"),
        ] {
            let failure = release_http_failure(status);
            assert_eq!(failure.code, code);
            assert_eq!(failure.http_status, Some(status));
        }
    }
}
use crate::runtime_preferences::DeviceRuntime;
use serde::{Deserialize, Serialize};
use speclink_host::network::{
    DiagnosticAction, DiagnosticArea, ErrorDetail, NetworkDiagnostic, ProxyPolicy,
};
use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateError {
    pub code: String,
    pub message: String,
    pub operation_id: Option<String>,
    pub http_status: Option<u16>,
}
impl From<String> for UpdateError {
    fn from(code: String) -> Self {
        Self {
            message: code.clone(),
            code,
            operation_id: None,
            http_status: None,
        }
    }
}

pub struct UpdateFailure {
    pub code: String,
    pub http_status: Option<u16>,
    pub chain: Vec<ErrorDetail>,
}
impl UpdateFailure {
    pub fn code(code: &str) -> Self {
        Self {
            code: code.into(),
            http_status: None,
            chain: vec![],
        }
    }
}

pub struct Candidate<T> {
    pub update: T,
    pub version: String,
    pub download_url: String,
}

/// The shell supplies only plugin calls and typed error conversion; job policy lives here.
pub trait UpdaterBackend: Sync {
    type Update: Send + Sync + 'static;
    fn check(
        &self,
        policy: ProxyPolicy,
    ) -> impl Future<Output = Result<Option<Candidate<Self::Update>>, UpdateFailure>> + Send;
    fn download(
        &self,
        update: &Self::Update,
    ) -> impl Future<Output = Result<Vec<u8>, UpdateFailure>> + Send;
    fn install(
        &self,
        update: Arc<Job<Self::Update>>,
        bytes: Vec<u8>,
    ) -> impl Future<Output = Result<(), UpdateFailure>> + Send;
}

pub struct Job<T> {
    pub update: T,
    policy: ProxyPolicy,
    download_url: String,
    bytes: Mutex<Option<Vec<u8>>>,
    busy: AtomicBool,
}
pub struct UpdaterJobs<T>(Mutex<Option<(String, Arc<Job<T>>)>>);
impl<T> Default for UpdaterJobs<T> {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}
impl<T> UpdaterJobs<T> {
    fn lookup(&self, id: &str) -> Result<Arc<Job<T>>, UpdateError> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .filter(|(key, _)| key == id)
            .map(|(_, job)| job.clone())
            .ok_or_else(|| "unknown_update".to_owned().into())
    }
}
#[derive(Serialize)]
pub struct CandidateInfo {
    pub id: String,
    pub version: String,
}

pub fn finish(
    mut event: NetworkDiagnostic,
    failure: Option<&UpdateFailure>,
    elapsed: Duration,
) -> (NetworkDiagnostic, Option<UpdateError>) {
    event.duration_ms = Some(elapsed.as_millis() as u64);
    event.event = if failure.is_some() {
        DiagnosticAction::Failed
    } else {
        DiagnosticAction::Completed
    };
    let error = failure.map(|failure| {
        event.error_code = Some(failure.code.clone());
        event.http_status = failure.http_status;
        event.error_chain = failure.chain.clone();
        UpdateError {
            code: failure.code.clone(),
            message: failure.code.clone(),
            operation_id: Some(event.operation_id.clone()),
            http_status: failure.http_status,
        }
    });
    if failure.is_none() {
        // The updater plugin does not expose the successful HTTP response.
        event.error_code = None;
    }
    (event, error)
}

fn settle<T>(
    runtime: &DeviceRuntime,
    policy: &ProxyPolicy,
    event: NetworkDiagnostic,
    result: Result<T, UpdateFailure>,
    started: Instant,
) -> Result<T, UpdateError> {
    let result = match policy.client_error() {
        Some(code) => Err(UpdateFailure::code(code)),
        None => result,
    };
    let (event, error) = finish(event, result.as_ref().err(), started.elapsed());
    let _ = runtime.diagnostics.record(event);
    match (result, error) {
        (Ok(value), _) => Ok(value),
        (_, Some(error)) => Err(error),
        _ => unreachable!(),
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseRepository {
    Fork,
    Upstream,
}
impl ReleaseRepository {
    fn repo(self) -> &'static str {
        match self {
            Self::Fork => "mosutw/speclink",
            Self::Upstream => "MomoChenisMe/speclink",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRelease {
    pub version: String,
    pub repo_url: String,
    pub release_url: String,
    pub published_at: String,
}
fn release_http_failure(status: u16) -> UpdateFailure {
    UpdateFailure {
        code: if status == 407 { "proxy-auth" } else { "http" }.into(),
        http_status: Some(status),
        chain: vec![],
    }
}
fn select_desktop_release(
    repo: ReleaseRepository,
    value: &serde_json::Value,
) -> Result<Option<DesktopRelease>, UpdateFailure> {
    let items = value
        .as_array()
        .ok_or_else(|| UpdateFailure::code("parse"))?;
    let repo_url = format!("https://github.com/{}", repo.repo());
    let mut releases = Vec::new();
    for item in items.iter().take(100) {
        if item["draft"].as_bool() != Some(false) || item["prerelease"].as_bool() != Some(false) {
            continue;
        }
        let Some(tag) = item["tag_name"].as_str() else {
            return Err(UpdateFailure::code("parse"));
        };
        let Some(version) = tag.strip_prefix("desktop-v") else {
            continue;
        };
        let parts: Vec<_> = version.split('.').collect();
        if !(3..=4).contains(&parts.len())
            || parts
                .iter()
                .any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit()))
        {
            continue;
        }
        let has_installer = item["assets"].as_array().is_some_and(|assets| {
            assets.iter().any(|asset| {
                asset["name"].as_str().is_some_and(|name| {
                    name.starts_with("Speclink_")
                        && (name.ends_with("-setup.exe")
                            || name.ends_with(".dmg")
                            || name.ends_with(".AppImage")
                            || name.ends_with("-portable.zip"))
                })
            })
        });
        if !has_installer {
            continue;
        }
        let release_url = item["html_url"]
            .as_str()
            .ok_or_else(|| UpdateFailure::code("parse"))?;
        if release_url != format!("{repo_url}/releases/tag/{tag}") {
            return Err(UpdateFailure::code("untrusted_source"));
        }
        let date = item["published_at"]
            .as_str()
            .ok_or_else(|| UpdateFailure::code("parse"))?;
        let timestamp =
            chrono::DateTime::parse_from_rfc3339(date).map_err(|_| UpdateFailure::code("parse"))?;
        releases.push((
            timestamp,
            DesktopRelease {
                version: version.into(),
                repo_url: repo_url.clone(),
                release_url: release_url.into(),
                published_at: date.into(),
            },
        ));
    }
    Ok(releases
        .into_iter()
        .max_by_key(|(date, _)| *date)
        .map(|(_, release)| release))
}
pub async fn desktop_release(
    runtime: &DeviceRuntime,
    repo: ReleaseRepository,
) -> Result<Option<DesktopRelease>, UpdateError> {
    let policy = runtime.policy().map_err(UpdateError::from)?;
    let started = Instant::now();
    let mut event = NetworkDiagnostic::new(
        DiagnosticArea::Updater,
        DiagnosticAction::Started,
        ulid::Ulid::new().to_string(),
    );
    let endpoint = format!(
        "https://api.github.com/repos/{}/releases?per_page=100",
        repo.repo()
    );
    event.destination = Some(endpoint.clone());
    event
        .set_route(&policy, &endpoint)
        .map_err(UpdateError::from)?;
    let _ = runtime.diagnostics.record(event.clone());
    let result = match speclink_host::network::github_release_metadata(&policy, repo.repo()).await {
        Ok(value) => select_desktop_release(repo, &value),
        Err((code, status, chain)) => {
            let mut error = status
                .map(release_http_failure)
                .unwrap_or_else(|| UpdateFailure::code(&code));
            error.code = code;
            error.chain = chain;
            Err(error)
        }
    };
    settle(runtime, &policy, event, result, started)
}

pub async fn check<B: UpdaterBackend>(
    backend: &B,
    runtime: &DeviceRuntime,
    jobs: &UpdaterJobs<B::Update>,
    endpoints: Vec<String>,
) -> Result<Option<CandidateInfo>, UpdateError> {
    let policy = runtime.policy().map_err(UpdateError::from)?;
    let _ = policy
        .configure_client(reqwest::Client::builder())
        .map_err(UpdateError::from)?;
    let id = ulid::Ulid::new().to_string();
    let started = Instant::now();
    let mut event = NetworkDiagnostic::new(
        DiagnosticArea::Updater,
        DiagnosticAction::Started,
        id.clone(),
    );
    event.proxy_mode = Some(policy.settings.mode.clone());
    if endpoints.len() == 1 {
        event.destination = endpoints.first().cloned();
        event
            .set_route(&policy, &endpoints[0])
            .map_err(UpdateError::from)?;
    }
    let _ = runtime.diagnostics.record(event.clone());
    let result = backend.check(policy.clone()).await.and_then(|candidate| {
        if let Some(candidate) = &candidate {
            let trusted = url::Url::parse(&candidate.download_url).is_ok_and(|url| {
                url.scheme() == "https"
                    && url.host_str() == Some("github.com")
                    && url.port().is_none()
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.query().is_none()
                    && url.fragment().is_none()
                    && url
                        .path()
                        .strip_prefix("/mosutw/speclink/releases/download/")
                        .is_some_and(|path| {
                            path.split_once('/')
                                .is_some_and(|(tag, file)| !tag.is_empty() && !file.is_empty())
                        })
            });
            if !trusted {
                return Err(UpdateFailure::code("untrusted_source"));
            }
        }
        Ok(candidate)
    });
    let Some(candidate) = settle(runtime, &policy, event, result, started)? else {
        return Ok(None);
    };
    let info = CandidateInfo {
        id: id.clone(),
        version: candidate.version,
    };
    *jobs.0.lock().unwrap() = Some((
        id,
        Arc::new(Job {
            policy,
            update: candidate.update,
            download_url: candidate.download_url,
            bytes: Mutex::new(None),
            busy: AtomicBool::new(false),
        }),
    ));
    Ok(Some(info))
}

pub async fn download<B: UpdaterBackend>(
    backend: &B,
    runtime: &DeviceRuntime,
    jobs: &UpdaterJobs<B::Update>,
    id: &str,
) -> Result<(), UpdateError> {
    let job = jobs.lookup(id)?;
    if job.busy.swap(true, Ordering::AcqRel) {
        return Err("busy".to_owned().into());
    }
    let result = async {
        let started = Instant::now();
        let mut event = NetworkDiagnostic::new(
            DiagnosticArea::Updater,
            DiagnosticAction::Started,
            id.into(),
        );
        event.destination = Some(job.download_url.clone());
        event
            .set_route(&job.policy, &job.download_url)
            .map_err(UpdateError::from)?;
        let _ = runtime.diagnostics.record(event.clone());
        let result = backend.download(&job.update).await;
        let bytes = settle(runtime, &job.policy, event, result, started)?;
        *job.bytes.lock().unwrap() = Some(bytes);
        Ok(())
    }
    .await;
    job.busy.store(false, Ordering::Release);
    result
}

pub async fn install<B: UpdaterBackend>(
    backend: &B,
    jobs: &UpdaterJobs<B::Update>,
    id: &str,
) -> Result<(), UpdateError> {
    let job = jobs.lookup(id)?;
    let bytes = job
        .bytes
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| UpdateError::from("update_not_downloaded".to_owned()))?;
    backend
        .install(job, bytes)
        .await
        .map_err(|failure| UpdateError {
            message: failure.code.clone(),
            code: failure.code,
            operation_id: Some(id.into()),
            http_status: failure.http_status,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend {
        code: Option<&'static str>,
        source: &'static str,
        installs: AtomicBool,
    }
    impl UpdaterBackend for Backend {
        type Update = ();
        async fn check(&self, _: ProxyPolicy) -> Result<Option<Candidate<()>>, UpdateFailure> {
            if let Some(code) = self.code {
                return Err(UpdateFailure::code(code));
            }
            Ok(Some(Candidate {
                update: (),
                version: "1.2.3".into(),
                download_url: self.source.into(),
            }))
        }
        async fn download(&self, _: &()) -> Result<Vec<u8>, UpdateFailure> {
            Ok(vec![1, 2, 3])
        }
        async fn install(&self, _: Arc<Job<()>>, bytes: Vec<u8>) -> Result<(), UpdateFailure> {
            assert_eq!(bytes, [1, 2, 3]);
            self.installs.store(true, Ordering::Release);
            Ok(())
        }
    }
    #[tokio::test]
    async fn failures_keep_codes_and_job_never_installs_before_download() {
        let root = tempfile::tempdir().unwrap();
        let runtime = DeviceRuntime::new(root.path().to_owned());
        let jobs = UpdaterJobs::default();
        for code in [
            "connect",
            "timeout",
            "tls",
            "proxy-auth",
            "http",
            "parse",
            "updater-unavailable",
        ] {
            let backend = Backend {
                code: Some(code),
                source:
                    "https://github.com/mosutw/speclink/releases/download/desktop-v1.2.3/update.zip",
                installs: AtomicBool::new(false),
            };
            let error = check(&backend, &runtime, &jobs, vec![])
                .await
                .err()
                .unwrap();
            assert_eq!(error.code, code);
            assert!(error.operation_id.is_some());
            assert_eq!(
                install(&backend, &jobs, "unknown").await.unwrap_err().code,
                "unknown_update"
            );
            assert!(!backend.installs.load(Ordering::Acquire));
        }
        let backend = Backend {
            code: None,
            source:
                "https://github.com/mosutw/speclink/releases/download/desktop-v1.2.3/update.zip",
            installs: AtomicBool::new(false),
        };
        let info = check(&backend, &runtime, &jobs, vec![])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            install(&backend, &jobs, &info.id).await.unwrap_err().code,
            "update_not_downloaded"
        );
        download(&backend, &runtime, &jobs, &info.id).await.unwrap();
        install(&backend, &jobs, &info.id).await.unwrap();
        assert!(backend.installs.load(Ordering::Acquire));
    }
    #[tokio::test]
    async fn a_signed_manifest_cannot_redirect_installation_to_the_upstream_repository() {
        let root = tempfile::tempdir().unwrap();
        let runtime = DeviceRuntime::new(root.path().into());
        let backend = Backend {
            code: None,
            source: "https://github.com/MomoChenisMe/speclink/releases/download/v99/update.zip",
            installs: AtomicBool::new(false),
        };
        let result = check(&backend, &runtime, &UpdaterJobs::default(), vec![]).await;
        assert_eq!(
            result.err().expect("must reject upstream package").code,
            "untrusted_source"
        );
    }
}

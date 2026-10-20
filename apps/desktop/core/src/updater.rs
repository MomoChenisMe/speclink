//! Update job lifecycle and diagnostic settlement, independent of the desktop shell.
use crate::runtime_preferences::DeviceRuntime;
use serde::Serialize;
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
}
impl From<String> for UpdateError {
    fn from(code: String) -> Self {
        Self {
            message: code.clone(),
            code,
            operation_id: None,
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
    let result = backend.check(policy.clone()).await;
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
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Backend {
        code: Option<&'static str>,
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
                download_url: "https://github.com/update".into(),
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
}

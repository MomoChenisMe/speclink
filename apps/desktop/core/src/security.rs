//! Background security jobs shared by the Desktop commands.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use speclink_core::security::{data::StandardRepository, standards::StandardId, JobProgress};
use speclink_host::security_data::{self, DataJob, OfficialTransport};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobError {
    pub code: String,
    pub message: String,
    pub stage: String,
    pub target: String,
}
impl JobError {
    pub fn review(message: &str) -> Self {
        let code = message.split(':').next().unwrap_or("review_error");
        let mut error = Self::new(match code {
            "invalid_config"
            | "invalid_change"
            | "binding_mismatch"
            | "unauthorized_root"
            | "missing_preparation"
            | "incomplete_preparation"
            | "missing_report"
            | "source_unavailable"
            | "unsupported_version"
            | "remote_prepare_failed"
            | "cancelled"
            | "budget_exceeded"
            | "storage_limit"
            | "busy"
            | "integrity_error"
            | "file_exists"
            | "invalid_path" => code,
            _ => "review_error",
        });
        error.stage = "security-review".into();
        error.target = "authorized-project".into();
        error
    }
    pub fn directory_unavailable() -> Self {
        Self::new("data_directory_unavailable")
    }
    pub fn worker_failed() -> Self {
        Self::new("data_error")
    }
    fn new(code: &str) -> Self {
        Self {
            code: code.into(),
            message: code.into(),
            stage: "security-data".into(),
            target: "application-data".into(),
        }
    }
    fn from_message(message: &str) -> Self {
        let code = message.split(':').next().unwrap_or("data_error");
        Self::new(match code {
            "missing_data"
            | "unsupported_version"
            | "unsupported_standard"
            | "budget_exceeded"
            | "cancelled"
            | "invalid_package"
            | "version_mismatch"
            | "license_missing"
            | "busy"
            | "storage_limit"
            | "size_limit"
            | "integrity_error"
            | "in_use"
            | "already_installed"
            | "download_failed"
            | "untrusted_source"
            | "data_directory_unavailable" => code,
            _ => "data_error",
        })
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTicket {
    pub job_id: String,
    pub project_key: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCompleted {
    pub job_id: String,
    pub project_key: String,
    pub result: std::result::Result<Value, JobError>,
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum DataRequest {
    Refresh,
    Install {
        #[serde(rename = "standardId")]
        standard_id: StandardId,
        version: String,
        #[serde(default)]
        reinstall: bool,
    },
    Import {
        path: PathBuf,
    },
    Remove {
        #[serde(rename = "standardId")]
        standard_id: StandardId,
        version: String,
    },
}
type Outcome = std::result::Result<Value, JobError>;
struct JobState {
    project_key: String,
    cancel: Arc<AtomicBool>,
    outcome: Mutex<Option<Outcome>>,
    ready: Condvar,
    started: Instant,
}
#[derive(Clone, Default)]
pub struct SecurityJobs {
    jobs: Arc<Mutex<BTreeMap<String, Arc<JobState>>>>,
}
impl SecurityJobs {
    fn launch(
        &self,
        job: DataJob,
        project_key: &str,
        work: impl FnOnce(DataJob) -> Outcome + Send + 'static,
    ) -> std::result::Result<JobTicket, JobError> {
        let mut jobs = self.jobs.lock().map_err(|_| JobError::new("data_error"))?;
        jobs.retain(|_, state| {
            state.started.elapsed() < Duration::from_secs(600)
                || state.outcome.lock().is_ok_and(|o| o.is_none())
        });
        if jobs.len() >= 16 {
            return Err(JobError::new("busy"));
        }
        let ticket = JobTicket {
            job_id: job.id.clone(),
            project_key: project_key.into(),
        };
        let state = Arc::new(JobState {
            project_key: project_key.into(),
            cancel: job.cancellation_token(),
            outcome: Mutex::new(None),
            ready: Condvar::new(),
            started: Instant::now(),
        });
        jobs.insert(job.id.clone(), state.clone());
        let worker = state.clone();
        if std::thread::Builder::new()
            .name("security-data".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(job)))
                    .unwrap_or_else(|_| Err(JobError::new("data_error")));
                if let Ok(mut outcome) = worker.outcome.lock() {
                    *outcome = Some(result);
                    worker.ready.notify_all();
                }
            })
            .is_err()
        {
            jobs.remove(&ticket.job_id);
            return Err(JobError::new("data_error"));
        }
        Ok(ticket)
    }
    fn state(&self, id: &str, project_key: &str) -> std::result::Result<Arc<JobState>, JobError> {
        let state = self
            .jobs
            .lock()
            .map_err(|_| JobError::new("data_error"))?
            .get(id)
            .cloned()
            .ok_or_else(|| JobError::new("unknown_job"))?;
        if state.project_key != project_key {
            return Err(JobError::new("invalid_project_key"));
        }
        Ok(state)
    }
    pub fn cancel(&self, id: &str, project_key: &str) -> std::result::Result<bool, JobError> {
        let state = self.state(id, project_key)?;
        if state
            .outcome
            .lock()
            .map_err(|_| JobError::new("data_error"))?
            .is_some()
        {
            return Ok(false);
        }
        state.cancel.store(true, Ordering::Relaxed);
        Ok(true)
    }
    pub fn wait(&self, id: &str, project_key: &str) -> Outcome {
        let state = self.state(id, project_key)?;
        let mut outcome = state
            .outcome
            .lock()
            .map_err(|_| JobError::new("data_error"))?;
        while outcome.is_none() {
            outcome = state
                .ready
                .wait(outcome)
                .map_err(|_| JobError::new("data_error"))?;
        }
        let result = outcome.as_ref().unwrap().clone();
        drop(outcome);
        self.jobs
            .lock()
            .map_err(|_| JobError::new("data_error"))?
            .remove(id);
        result
    }
    pub fn start_data(
        &self,
        application_data: PathBuf,
        request: DataRequest,
        progress: impl Fn(JobProgress) + Send + Sync + 'static,
        completed: impl Fn(JobCompleted) + Send + 'static,
    ) -> std::result::Result<JobTicket, JobError> {
        self.launch(
            DataJob::new(Arc::new(AtomicBool::new(false))),
            "application-data",
            move |job| {
                let result = execute_data(&application_data, request, &job, &progress);
                completed(JobCompleted {
                    job_id: job.id.clone(),
                    project_key: "application-data".into(),
                    result: result.clone(),
                });
                result
            },
        )
    }
}

pub fn list_data_at(
    application_data: &Path,
) -> std::result::Result<security_data::StandardDataList, JobError> {
    let result = speclink_fs::security::FsSecurityData::new(application_data)
        .and_then(|repository| security_data::list_data(&repository));
    result.map_err(|e| JobError::from_message(&e.to_string()))
}
pub fn remember_policy_at(
    application_data: &Path,
    key: &str,
    policy: &speclink_core::security::policy::SecurityPolicy,
) -> std::result::Result<(), JobError> {
    speclink_fs::security::FsSecurityData::new(application_data)
        .and_then(|r| r.remember_policy(key, policy))
        .map_err(|e| JobError::from_message(&e.to_string()))
}
fn execute_data(
    application_data: &Path,
    request: DataRequest,
    job: &DataJob,
    progress: &dyn Fn(JobProgress),
) -> Outcome {
    let run = || -> Result<Value, Box<dyn std::error::Error>> {
        let repository = speclink_fs::security::FsSecurityData::new(application_data)?;
        job.check()?;
        let value = match request {
            DataRequest::Refresh => serde_json::to_value(security_data::refresh_data(
                &repository,
                &OfficialTransport::new(job),
                job,
                progress,
            )?)?,
            DataRequest::Install {
                standard_id,
                version,
                reinstall,
            } => serde_json::to_value(security_data::install_data(
                &repository,
                standard_id,
                &version,
                reinstall,
                &OfficialTransport::new(job),
                &speclink_fs::security::decode_sources,
                job,
                progress,
            )?)?,
            DataRequest::Import { path } => {
                serde_json::to_value(security_data::import_data(&repository, &path, &|| {
                    job.check().is_err()
                })?)?
            }
            DataRequest::Remove {
                standard_id,
                version,
            } => serde_json::to_value(security_data::remove_data(
                &repository,
                standard_id,
                &version,
            )?)?,
        };
        Ok(value)
    };
    run().map_err(|e| JobError::from_message(&e.to_string()))
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum ReviewTargetInput {
    Local {
        root: PathBuf,
    },
    Remote {
        origin: String,
        project: String,
        repo: String,
        #[serde(rename = "checkoutRoot")]
        checkout_root: Option<PathBuf>,
    },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewInspection {
    pub project_key: String,
    pub enabled: Option<bool>,
    pub requires_preparation: bool,
    pub latest: Option<speclink_core::security::report::Report>,
    pub selected_standards: Vec<SelectedStandard>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedStandard {
    standard_id: String,
    version: String,
    level: Option<speclink_core::security::policy::AsvsLevel>,
}
type BoundaryResult<T> = Result<T, Box<dyn std::error::Error>>;
fn resolve_review(
    input: &ReviewTargetInput,
    change: Option<String>,
) -> BoundaryResult<(
    speclink_host::security::ReviewTarget,
    Option<speclink_fs::security::scan::AuthorizedProject>,
)> {
    use speclink_fs::security::scan::AuthorizedProject;
    use speclink_host::security::ReviewTarget;
    let reader = match input {
        ReviewTargetInput::Local { root } => Some(AuthorizedProject::open(root)?),
        ReviewTargetInput::Remote { checkout_root, .. } => checkout_root
            .as_deref()
            .map(AuthorizedProject::open)
            .transpose()?,
    };
    let target = match input {
        ReviewTargetInput::Local { .. } => {
            let reader = reader.as_ref().unwrap();
            let text = reader
                .read_optional_text(".speclink.yaml", 64 * 1024, &|| None)?
                .unwrap_or_default();
            let config: speclink_core::config::AppConfig = if text.trim().is_empty() {
                Default::default()
            } else {
                serde_yaml::from_str(&text).map_err(|_| "invalid_config")?
            };
            if config.remote.is_some() {
                return Err("binding_mismatch".into());
            }
            ReviewTarget::local(
                reader.root(),
                config.spec_dir.as_deref().unwrap_or("openspec"),
                change,
            )?
        }
        ReviewTargetInput::Remote {
            origin,
            project,
            repo,
            ..
        } => ReviewTarget {
            binding: speclink_core::security::prepared::ReviewBinding {
                origin: Some(origin.clone()),
                project: Some(project.clone()),
                repo: Some(repo.clone()),
                checkout: reader
                    .as_ref()
                    .map(|r| r.root().to_string_lossy().into_owned()),
            },
            spec_dir: "openspec".into(),
            change_name: change,
            project_name: project.clone(),
        },
    };
    target.validate()?;
    Ok((target, reader))
}
pub fn inspect_review_at(
    application_data: &Path,
    input: &ReviewTargetInput,
    change: Option<String>,
) -> Result<ReviewInspection, JobError> {
    use speclink_core::security::{prepared::PreparedRepository, report::ReportRepository};
    let run = || -> BoundaryResult<ReviewInspection> {
        let (target, reader) = resolve_review(input, change)?;
        let project_key = target.binding.project_key();
        let text = if target.binding.is_remote() {
            speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?
                .load_policy(&project_key, target.change_name.as_deref())?
                .map(|s| s.policy_text)
        } else {
            Some(
                reader
                    .as_ref()
                    .unwrap()
                    .read_optional_text(
                        &format!("{}/config.yaml", target.spec_dir),
                        64 * 1024,
                        &|| None,
                    )?
                    .unwrap_or_default(),
            )
        };
        let policy = text
            .as_deref()
            .map(|text| {
                speclink_core::config::WorkflowConfig::from_text(Some(text))
                    .map(|w| w.security())
                    .map_err(|_| "invalid_config")
            })
            .transpose()?;
        let selected_standards = policy
            .as_ref()
            .map(|p| {
                p.standards
                    .selected()
                    .into_iter()
                    .map(|(id, version, level)| SelectedStandard {
                        standard_id: id.into(),
                        version: version.into(),
                        level,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let latest = speclink_fs::security::reports::FsSecurityReports::new(application_data)?
            .latest(&project_key, target.change_name.as_deref())?;
        Ok(ReviewInspection {
            project_key,
            enabled: policy.as_ref().map(|p| p.enabled),
            requires_preparation: text.is_none(),
            latest,
            selected_standards,
        })
    };
    run().map_err(|e| JobError::review(&e.to_string()))
}
pub fn read_report_at(
    application_data: &Path,
    id: &str,
) -> Result<speclink_core::security::report::Report, JobError> {
    use speclink_core::security::report::ReportRepository;
    let run = || -> BoundaryResult<_> {
        Ok(
            speclink_fs::security::reports::FsSecurityReports::new(application_data)?
                .get(id)?
                .ok_or("missing_report")?,
        )
    };
    run().map_err(|e| JobError::review(&e.to_string()))
}
impl SecurityJobs {
    pub fn start_review_at(
        &self,
        application_data: PathBuf,
        input: ReviewTargetInput,
        change: Option<String>,
        progress: impl Fn(JobProgress) + Send + Sync + 'static,
        completed: impl Fn(JobCompleted) + Send + 'static,
    ) -> Result<JobTicket, JobError> {
        let (target, reader) =
            resolve_review(&input, change).map_err(|e| JobError::review(&e.to_string()))?;
        let key = target.binding.project_key();
        let worker_key = key.clone();
        self.launch(DataJob::new(Arc::new(AtomicBool::new(false))),&key,move|job|{
            let run=||->BoundaryResult<Value>{
                let data=speclink_fs::security::FsSecurityData::new(&application_data)?;
                let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
                let reports=speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
                let budget=speclink_host::security::ReviewBudget::with_job_id(job.id.clone(),job.cancellation_token());
                let outcome=speclink_host::security::run_review(&target,reader.as_ref().map(|r|r as &dyn speclink_core::security::input::ProjectInput),&data,&prepared,&reports,&budget,&progress)?;
                Ok(serde_json::json!({"reportId":outcome.report.report_id(),"projectKey":worker_key,"warnings":outcome.receipt.warnings}))
            };
            let result=run().map_err(|e|JobError::review(&e.to_string()));
            completed(JobCompleted{job_id:job.id,project_key:worker_key,result:result.clone()});result
        })
    }
    /// Only this explicit preparation operation may receive a remote fetch capability.
    pub fn start_prepare_at(
        &self,
        application_data: PathBuf,
        input: ReviewTargetInput,
        change: Option<String>,
        fetch: impl Fn(
                &speclink_host::security::ReviewTarget,
                &speclink_host::security::ReviewBudget,
            ) -> Result<speclink_host::security::PreparationInput, JobError>
            + Send
            + 'static,
        completed: impl Fn(JobCompleted) + Send + 'static,
    ) -> Result<JobTicket, JobError> {
        let (target, reader) =
            resolve_review(&input, change).map_err(|e| JobError::review(&e.to_string()))?;
        let key = target.binding.project_key();
        let worker_key = key.clone();
        self.launch(
            DataJob::new(Arc::new(AtomicBool::new(false))),
            &key,
            move |job| {
                let run = || -> BoundaryResult<Value> {
                    let budget = speclink_host::security::ReviewBudget::with_job_id(
                        job.id.clone(),
                        job.cancellation_token(),
                    );
                    let prepared =
                        speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
                    let receipt = speclink_host::security::prepare_review(
                        &target,
                        &|| {
                            if let Some(reader) = &reader {
                                if !target.binding.is_remote() {
                                    return speclink_host::security::local_preparation(
                                        &target, reader, &budget,
                                    );
                                }
                            }
                            fetch(&target, &budget).map_err(|e| anyhow::anyhow!(e.code))
                        },
                        &prepared,
                        &budget,
                    )?;
                    Ok(serde_json::to_value(receipt)?)
                };
                let result = run().map_err(|e| JobError::review(&e.to_string()));
                completed(JobCompleted {
                    job_id: job.id,
                    project_key: worker_key,
                    result: result.clone(),
                });
                result
            },
        )
    }
}
/// Revalidate both report identity and the original authorized root before a UI reveal.
pub fn reveal_report_path_at(
    application_data: &Path,
    input: &ReviewTargetInput,
    id: &str,
    path: &str,
) -> Result<PathBuf, JobError> {
    let run = || -> BoundaryResult<PathBuf> {
        let report = read_report_at(application_data, id).map_err(|_| "missing_report")?;
        let (target, reader) = resolve_review(input, report.change_name().map(str::to_owned))?;
        if target.binding.project_key() != report.project_key() || !report.has_source(path) {
            return Err("binding_mismatch".into());
        }
        Ok(reader.ok_or("source_unavailable")?.verified_path(path)?)
    };
    run().map_err(|e| JobError::review(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn security_report_export_never_overwrites_or_follows_links() {
        let device = crate::testfixture::FixtureRoot::new("security-export");
        let path = device.root().join("report.json");
        speclink_fs::security::reports::export_to_path(&path, b"redacted").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"redacted");
        assert!(speclink_fs::security::reports::export_to_path(&path, b"replacement").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"redacted");
        #[cfg(unix)]
        {
            let link = device.root().join("link.json");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(speclink_fs::security::reports::export_to_path(&link, b"replacement").is_err());
        }
    }
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };
    #[test]
    fn security_jobs_keep_identity_cancel_and_fast_completion() {
        let jobs = SecurityJobs::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let job = speclink_host::security_data::DataJob::new(cancel);
        let ticket = jobs
            .launch(job, "application-data", |job| {
                while job.check().is_ok() {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(JobError::new("cancelled"))
            })
            .unwrap();
        assert!(jobs.cancel(&ticket.job_id, "another-project").is_err());
        let started = Instant::now();
        assert!(jobs.cancel(&ticket.job_id, "application-data").unwrap());
        assert_eq!(
            jobs.wait(&ticket.job_id, "application-data")
                .unwrap_err()
                .code,
            "cancelled"
        );
        assert!(started.elapsed() < Duration::from_secs(2));
        let ticket = jobs
            .launch(
                speclink_host::security_data::DataJob::new(Arc::new(AtomicBool::new(false))),
                "application-data",
                |_| Ok(serde_json::json!({"complete":true})),
            )
            .unwrap();
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(
            jobs.wait(&ticket.job_id, "application-data").unwrap()["complete"],
            true
        );
        assert!(jobs.wait(&ticket.job_id, "application-data").is_err());
        assert_eq!(
            JobError::from_message("unrecognized raw-secret-sentinel").message,
            "data_error"
        );
        let _ = Ordering::Relaxed;
    }
    #[test]
    fn security_review_jobs_resolve_root_and_save_report_without_quality_tickets() {
        let project = crate::testfixture::FixtureRoot::new("security-review-project");
        let device = crate::testfixture::FixtureRoot::new("security-review-device");
        let input = ReviewTargetInput::Local {
            root: project.root().to_path_buf(),
        };
        let inspection = inspect_review_at(device.root(), &input, None).unwrap();
        assert_eq!(inspection.enabled, Some(false));
        let jobs = SecurityJobs::default();
        let ticket = jobs
            .start_review_at(device.root().to_path_buf(), input, None, |_| {}, |_| {})
            .unwrap();
        assert_eq!(ticket.project_key, inspection.project_key);
        let completed = jobs.wait(&ticket.job_id, &ticket.project_key).unwrap();
        let id = completed["reportId"].as_str().unwrap();
        let report = read_report_at(device.root(), id).unwrap();
        assert!(report.is_complete());
        assert_eq!(report.configuration_state(), "disabled");
        assert!(!project
            .root()
            .join("openspec/changes/demo/verify.md")
            .exists());
    }
}

pub fn export_report_at(
    application_data: &Path,
    id: &str,
    format: &str,
    path: &Path,
) -> Result<(), JobError> {
    let report = read_report_at(application_data, id)?;
    let text = match format {
        "json" => {
            serde_json::to_string_pretty(&report).map_err(|_| JobError::review("review_error"))?
        }
        "markdown" => report.markdown(),
        _ => return Err(JobError::review("invalid_config")),
    };
    speclink_fs::security::reports::export_to_path(path, text.as_bytes())
        .map_err(|e| JobError::review(&e.to_string()))
}

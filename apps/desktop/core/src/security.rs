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
            | "invalid_path"
            | "missing_knowledge" | "invalid_knowledge" | "security_report_stale" | "missing_check_record" | "history_recovery_required" | "source_freshness_unverified" | "unknown_change" | "unknown_report" | "security_disabled" | "unconfigured" | "size_limit" => code,
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
            | "data_directory_unavailable"
            | "proxy_unsupported"
            | "proxy_invalid"
            | "preferences_invalid"
            | "invalid_request"
            | "file_exists"
            | "shared_data"
            | "settings_write_failed"
            | "invalid_path"
            | "storage_error" => code,
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
    Export {
        #[serde(rename = "standardId")]
        standard_id: Option<StandardId>,
        version: Option<String>,
        #[serde(default)]
        all: bool,
        path: PathBuf,
    },
    Remove {
        #[serde(rename = "standardId")]
        standard_id: StandardId,
        version: String,
    },
}
type Outcome = std::result::Result<Value, JobError>;
#[cfg(test)]
static REVIEW_TEST_LOCK: Mutex<()> = Mutex::new(());
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
    pub fn write_data_paths(&self, application_data:&Path, path:Option<String>) -> std::result::Result<speclink_host::security_paths::PathSnapshot,JobError> {
        let jobs=self.jobs.lock().map_err(|_|JobError::new("data_error"))?;
        if jobs.values().any(|job|job.outcome.lock().map_or(true,|outcome|outcome.is_none())) { return Err(JobError::new("busy")); }
        speclink_host::security_paths::write_at(&speclink_host::context::global_config_dir(),application_data,path).map_err(|e|JobError::from_message(&e.to_string()))
    }
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
        self.start_data_with_network(application_data, request, None, progress, completed)
    }
    pub fn start_data_managed(
        &self,
        application_data: PathBuf,
        request: DataRequest,
        runtime: &crate::runtime_preferences::DeviceRuntime,
        progress: impl Fn(JobProgress) + Send + Sync + 'static,
        completed: impl Fn(JobCompleted) + Send + 'static,
    ) -> Result<JobTicket, JobError> {
        let network = if matches!(&request, DataRequest::Refresh | DataRequest::Install { .. }) {
            Some((
                runtime.policy().map_err(|code| JobError::new(&code))?,
                runtime.diagnostics.sink(),
            ))
        } else {
            None
        };
        self.start_data_with_network(application_data, request, network, progress, completed)
    }
    fn start_data_with_network(
        &self,
        application_data: PathBuf,
        request: DataRequest,
        network: Option<(
            speclink_host::network::ProxyPolicy,
            speclink_host::network::DiagnosticSink,
        )>,
        progress: impl Fn(JobProgress) + Send + Sync + 'static,
        completed: impl Fn(JobCompleted) + Send + 'static,
    ) -> Result<JobTicket, JobError> {
        self.launch(
            DataJob::new(Arc::new(AtomicBool::new(false))),
            "application-data",
            move |job| {
                let result =
                    execute_data_with_network(&application_data, request, &job, &progress, network);
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
    let result = repository_at(application_data)
        .and_then(|repository| security_data::list_data(&repository));
    result.map_err(|e| JobError::from_message(&e.to_string()))
}
pub fn repository_at(application_data:&Path)->anyhow::Result<speclink_fs::security::FsSecurityData> {
    let paths=speclink_host::security_paths::lease(application_data)?;
    Ok(speclink_fs::security::FsSecurityData::with_standard_root(application_data,&paths.standards_root)?.hold_path_lease(paths.lock))
}
pub fn data_paths_at(application_data:&Path)->std::result::Result<speclink_host::security_paths::PathSnapshot,JobError> {
    speclink_host::security_paths::read_at(&speclink_host::context::global_config_dir(),application_data).map_err(|e|JobError::from_message(&e.to_string()))
}
pub fn remember_policy_at(
    application_data: &Path,
    key: &str,
    policy: &speclink_core::security::policy::SecurityPolicy,
) -> std::result::Result<(), JobError> {
    repository_at(application_data)
        .and_then(|r| r.remember_policy(key, policy))
        .map_err(|e| JobError::from_message(&e.to_string()))
}
fn execute_data_with_network(
    application_data: &Path,
    request: DataRequest,
    job: &DataJob,
    progress: &dyn Fn(JobProgress),
    network: Option<(
        speclink_host::network::ProxyPolicy,
        speclink_host::network::DiagnosticSink,
    )>,
) -> Outcome {
    let transport = match network {
        Some((policy, sink)) => OfficialTransport::with_policy(job, policy, sink),
        None => OfficialTransport::new(job),
    };
    let run = || -> Result<Value, Box<dyn std::error::Error>> {
        let repository = repository_at(application_data)?;
        job.check()?;
        let value = match request {
            DataRequest::Refresh => serde_json::to_value(security_data::refresh_data(
                &repository,
                &transport,
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
                &transport,
                &speclink_fs::security::decode_sources,
                job,
                progress,
            )?)?,
            DataRequest::Import { path } => {
                serde_json::to_value(security_data::import_data_bundle(
                    &repository,
                    &path,
                    job,
                    progress,
                    &speclink_fs::security_packages::prepare_collection,
                )?)?
            }
            DataRequest::Export {
                standard_id,
                version,
                all,
                path,
            } => {
                let selection = match (standard_id, version, all) {
                    (None, None, true) => security_data::ExportSelection::All,
                    (Some(id), Some(version), false) => {
                        security_data::ExportSelection::One(id, version)
                    }
                    _ => return Err("invalid_request".into()),
                };
                serde_json::to_value(security_data::export_data(
                    &repository,
                    selection,
                    job,
                    progress,
                    &|selected, all, skipped, load| {
                        speclink_fs::security_packages::write_export(
                            &path,
                            selected,
                            all,
                            skipped,
                            load,
                            &|| job.check().is_err(),
                        )
                    },
                )?)?
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
    pub device_local: bool,
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
        ReviewTargetInput::Local { root } => {
            let selected=change.as_deref().and_then(|name|crate::context_for_change(root,name)).map(|ctx|ctx.workspace.root).unwrap_or_else(||root.clone());
            Some(AuthorizedProject::open(&selected)?)
        },
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
            proposal_project_key:None,
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
fn restore_proposal_key(application_data:&Path,target:&mut speclink_host::security::ReviewTarget)->BoundaryResult<()> {
    use speclink_core::security::prepared::PreparedRepository;
    if target.binding.is_remote(){target.proposal_project_key=speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?.load_policy(&target.binding.project_key(),target.change_name.as_deref())?.and_then(|s|s.proposal_project_key);}
    Ok(())
}
pub fn inspect_review_at(
    application_data: &Path,
    input: &ReviewTargetInput,
    change: Option<String>,
) -> Result<ReviewInspection, JobError> {
    use speclink_core::security::prepared::PreparedRepository;
    let run = || -> BoundaryResult<ReviewInspection> {
        let (mut target, reader) = resolve_review(input, change)?;
        restore_proposal_key(application_data,&mut target)?;
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
        let reports=speclink_fs::security::reports::FsSecurityReports::new(application_data)?;
        let store=reader.as_ref().map(|r|speclink_fs::FsStore::new(r.root(),&target.spec_dir));
        let proposal=if !target.binding.is_remote(){target.change_name.as_deref().zip(store.as_ref()).map(|(change,store)|speclink_host::security::ProposalReports{store,change})}else{None};
        let repository:&dyn speclink_core::security::report::ReportRepository=proposal.as_ref().map(|p|p as _).unwrap_or(&reports);
        let latest=repository.latest(&target.report_project_key(),target.change_name.as_deref())?;
        Ok(ReviewInspection {
            device_local:target.binding.is_remote(),
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
        let (mut target, reader) =
            resolve_review(&input, change).map_err(|e| JobError::review(&e.to_string()))?;
        restore_proposal_key(&application_data,&mut target).map_err(|e|JobError::review(&e.to_string()))?;
        let key = target.binding.project_key();
        let worker_key = key.clone();
        self.launch(DataJob::new(Arc::new(AtomicBool::new(false))),&key,move|job|{
            let run=||->BoundaryResult<Value>{
                let data=repository_at(&application_data)?;
                let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
                let reports=speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
                let budget=speclink_host::security::ReviewBudget::with_job_id(job.id.clone(),job.cancellation_token());
                let store=reader.as_ref().map(|r|speclink_fs::FsStore::new(r.root(),&target.spec_dir));
                let proposal=if !target.binding.is_remote(){target.change_name.as_deref().zip(store.as_ref()).map(|(change,store)|speclink_host::security::ProposalReports{store,change})}else{None};
                let repository:&dyn speclink_core::security::report::ReportRepository=proposal.as_ref().map(|p|p as _).unwrap_or(&reports);
                let outcome=speclink_host::security::run_review(&target,reader.as_ref().map(|r|r as &dyn speclink_core::security::input::ProjectInput),&data,&prepared,repository,&budget,&progress)?;
                Ok(serde_json::json!({"reportId":outcome.report.report_id(),"projectKey":worker_key,"warnings":outcome.receipt.warnings,"pendingPublication":target.binding.is_remote()&&target.change_name.is_some()}))
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
    fn changing_paths_refuses_a_pending_job_before_touching_preferences() {
        let jobs=SecurityJobs::default();let root=tempfile::tempdir().unwrap();
        let (finish,wait)=std::sync::mpsc::channel();
        let ticket=jobs.launch(DataJob::new(Arc::new(AtomicBool::new(false))),"application-data",move |_|{wait.recv().unwrap();Ok(serde_json::json!({}))}).unwrap();
        assert_eq!(jobs.write_data_paths(root.path(),Some("relative".into())).unwrap_err().code,"busy");
        finish.send(()).unwrap();jobs.wait(&ticket.job_id,&ticket.project_key).unwrap();
    }
    #[test]
    fn network_job_errors_preserve_safe_codes_without_external_details() {
        for code in ["proxy_unsupported","proxy_invalid","preferences_invalid"] {
            let error=JobError::from_message(&format!("{code}: SECRET"));assert_eq!(error.code,code);assert_eq!(error.message,code);
        }
    }
    #[test]
    fn export_request_is_device_local_and_existing_destination_is_preserved() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("already.zip");
        std::fs::write(&path, b"keep").unwrap();
        let request: DataRequest =
            serde_json::from_value(serde_json::json!({"action":"export","all":true,"path":path}))
                .expect("export request");
        let jobs = SecurityJobs::default();
        let ticket = jobs
            .start_data(root.path().to_path_buf(), request, |_| {}, |_| {})
            .unwrap();
        let error = jobs.wait(&ticket.job_id, "application-data").unwrap_err();
        assert_eq!(error.code, "missing_data");
        assert_eq!(std::fs::read(&path).unwrap(), b"keep");
    }
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
        let _guard=REVIEW_TEST_LOCK.lock().unwrap();
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

#[cfg(test)]
mod proposal_tests {
    use super::*;
    #[test]
    fn change_review_jobs_publish_into_the_proposal_and_keep_scope_isolation() {
        let _guard=REVIEW_TEST_LOCK.lock().unwrap();
        let project=crate::testfixture::FixtureRoot::new("security-proposal-record");project.add_change("demo","schema: spec-driven\n");
        let other=crate::testfixture::FixtureRoot::new("security-other-record");other.add_change("demo","schema: spec-driven\n");
        let device=crate::testfixture::FixtureRoot::new("security-proposal-device");
        let input=ReviewTargetInput::Local{root:project.root().into()};let jobs=SecurityJobs::default();
        let ticket=jobs.start_review_at(device.root().into(),input.clone(),Some("demo".into()),|_|{},|_|{}).unwrap();
        let result=jobs.wait(&ticket.job_id,&ticket.project_key).unwrap();let id=result["reportId"].as_str().unwrap();
        let store=speclink_fs::FsStore::new(project.root(),"openspec");
        assert!(speclink_core::security::report::read_change_report(&store,"demo",id).unwrap().is_some(),"a change report belongs in its proposal");
        assert!(read_report_at(device.root(),id).is_err(),"device cache is not the local proposal authority");
        let store=speclink_fs::FsStore::new(other.root(),"openspec");assert!(speclink_core::security::report::read_change_report(&store,"demo",id).unwrap().is_none());
        assert!(jobs.wait("unknown",&ticket.project_key).is_err());
    }
}

/// Read the explicitly selected authority. Local proposal reads never fall back to device data.
pub fn read_target_basic_at(application_data:&Path,input:&ReviewTargetInput,change:Option<&str>,id:&str)->Result<speclink_core::security::report::Report,JobError> {
    let run=||->BoundaryResult<_>{
        let (mut target,reader)=resolve_review(input,change.map(str::to_owned))?;restore_proposal_key(application_data,&mut target)?;
        let report=if let Some(change)=change.filter(|_|!target.binding.is_remote()) {
            let reader=reader.as_ref().ok_or("source_unavailable")?;let store=speclink_fs::FsStore::new(reader.root(),&target.spec_dir);
            match speclink_core::security::report::read_change_report(&store,change,id)?.ok_or("missing_report")? {speclink_core::security::report::VersionedReport::Builtin(b)=>b,_=>return Err("invalid_report".into())}
        }else{read_report_at(application_data,id).map_err(|_|"missing_report")?};
        if report.project_key()!=target.report_project_key()||report.change_name()!=change{return Err("binding_mismatch".into());}Ok(report)
    };run().map_err(|e|JobError::review(&e.to_string()))
}
pub fn export_target_basic_at(application_data:&Path,input:&ReviewTargetInput,change:Option<&str>,id:&str,format:&str,path:&Path)->Result<(),JobError>{
    let report=read_target_basic_at(application_data,input,change,id)?;
    let text=match format{"json"=>serde_json::to_string_pretty(&report).map_err(|_|JobError::review("invalid_report"))?,"markdown"=>report.markdown(),_=>return Err(JobError::review("invalid_config"))};
    speclink_fs::security::reports::export_to_path(path,text.as_bytes()).map_err(|e|JobError::review(&e.to_string()))
}
pub fn reveal_target_basic_at(application_data:&Path,input:&ReviewTargetInput,change:Option<&str>,id:&str,path:&str)->Result<PathBuf,JobError>{
    let report=read_target_basic_at(application_data,input,change,id)?;
    if !report.has_source(path){return Err(JobError::review("binding_mismatch"));}
    let (_,reader)=resolve_review(input,change.map(str::to_owned)).map_err(|e|JobError::review(&e.to_string()))?;
    reader.ok_or_else(||JobError::review("source_unavailable"))?.verified_path(path).map_err(|e|JobError::review(&e.to_string()))
}

impl SecurityJobs {
    /// Prepares only from installed data and an explicitly prepared Remote policy.
    pub fn start_knowledge_at(&self,application_data:PathBuf,input:ReviewTargetInput,change:Option<String>,completed:impl Fn(JobCompleted)+Send+'static)->Result<JobTicket,JobError>{
        use speclink_core::security::prepared::PreparedRepository;
        let (mut target,reader)=resolve_review(&input,change).map_err(|e|JobError::review(&e.to_string()))?;
        restore_proposal_key(&application_data,&mut target).map_err(|e|JobError::review(&e.to_string()))?;
        let key=target.binding.project_key();let worker_key=key.clone();
        self.launch(DataJob::new(Arc::new(AtomicBool::new(false))),&key,move|job|{
            let run=||->BoundaryResult<Value>{
                let reader=reader.as_ref().ok_or("source_unavailable")?;
                let data=repository_at(&application_data)?;
                let text=if target.binding.is_remote(){
                    if target.proposal_project_key.is_none(){return Err("unsupported_version".into());}
                    speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?.load_policy(&target.binding.project_key(),target.change_name.as_deref())?.ok_or("missing_preparation")?.policy_text
                }else{reader.read_optional_text(&format!("{}/config.yaml",target.spec_dir),64*1024,&||if job.check().is_err(){Some("cancelled")}else{None})?.unwrap_or_default()};
                let receipt=speclink_fs::security::knowledge::prepare(reader.root(),&target.knowledge_project_key(),&text,&data,&||job.check().is_err())?;
                Ok(serde_json::to_value(receipt)?)
            };let result=run().map_err(|e|JobError::review(&e.to_string()));completed(JobCompleted{job_id:job.id,project_key:worker_key,result:result.clone()});result
        })
    }
}

pub fn remote_attestation_at(application_data:&Path,input:&ReviewTargetInput,change:&str)->Result<speclink_protocol::security::SecuritySourceAttestation,JobError>{
    use speclink_core::security::prepared::PreparedRepository;
    let run=||->BoundaryResult<_>{
        let (mut target,reader)=resolve_review(input,Some(change.into()))?;restore_proposal_key(application_data,&mut target)?;
        if !target.binding.is_remote(){return Err("binding_mismatch".into());}
        let reader=reader.as_ref().ok_or("source_unavailable")?;
        let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?;
        let snapshot=prepared.load(&target.binding.project_key(),Some(change))?.ok_or("missing_preparation")?;
        let data=repository_at(application_data)?;
        let budget=speclink_host::security::ReviewBudget::new(Arc::new(AtomicBool::new(false)));
        let (context,_)=speclink_host::security::prepared_submission_context(&target,reader,&data,&snapshot,None,None,&budget)?;
        Ok(speclink_host::security::source_attestation(&context,&data)?)
    };run().map_err(|e|JobError::review(&e.to_string()))
}
pub fn review_view_at(application_data:&Path,root:&Path,change:&str)->Result<speclink_protocol::security::SecurityReviewView,String>{
    use speclink_core::store::Store;
    let ctx=crate::require_context_for_change(root,change)?;
    let reader=speclink_fs::security::scan::AuthorizedProject::open(&ctx.workspace.root).map_err(|e|e.to_string())?;
    let target=speclink_host::security::ReviewTarget::local(reader.root(),&ctx.workspace.spec_dir_name,Some(change.into())).map_err(|e|e.to_string())?;
    let policy=speclink_core::config::WorkflowConfig::from_text(ctx.store.read_workflow_config().as_deref()).map_err(|e|e.to_string())?.security();
    let data=repository_at(application_data).map_err(|e|e.to_string())?;
    let budget=speclink_host::security::ReviewBudget::new(Arc::new(AtomicBool::new(false)));
    let context=if ctx.store.change_exists(change){speclink_host::security::submission_context(&target,&reader,&data,&ctx.store,None,&budget).ok()}else{None};
    let fresh=context.as_ref().map(|(c,_)|speclink_host::security::fresh_evidence(c));
    let mut view=speclink_host::security::review_view(&ctx.store,change,&target.report_project_key(),&policy,fresh.as_ref()).map_err(|e|e.to_string())?;
    if let Some((context,id))=context {let mut s=speclink_host::security::Submission::from(&context);s.builtin_report_id=id;view.submission=Some(serde_json::from_value(serde_json::to_value(s).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?);}
    Ok(view)
}

#[cfg(test)]
mod history_tests {
    use super::*;
    #[test]
    fn desktop_completed_history_survives_stamps_and_archive_names() {
        use speclink_core::{station,store::Store};
        let root=crate::testfixture::FixtureRoot::new("history-query");root.add_change("demo","schema: spec-driven\n");root.write("openspec/changes/demo/tasks.md","- [x] 1.1 Done\n");root.write("app.ts","const value=1;\n");
        let store=speclink_fs::FsStore::new(root.root(),"openspec");
        station::add_round(&station::REVIEW,&store,"demo","**Scope**: app.ts\n").unwrap();
        station::stamp(&station::REVIEW,&store,"demo",false,Some("Tester"),None,&|p|std::fs::read(root.root().join(p)).ok(),&|p|root.root().join(p).is_file()).unwrap();
        let list=crate::query::check_history_at(root.root(),"demo","review",None).unwrap();let value=serde_json::to_value(list).unwrap();let id=value["records"][0]["recordId"].as_str().unwrap();
        let original=crate::query::check_history_at(root.root(),"demo","review",Some(id)).unwrap();
        store.archive_change("demo","2026-10-21-demo").unwrap();assert_eq!(serde_json::to_value(crate::query::check_history_at(root.root(),"2026-10-21-demo","review",Some(id)).unwrap()).unwrap(),serde_json::to_value(original).unwrap());
        assert!(crate::query::check_history_at(root.root(),"2026-10-21-demo","verify",Some(id)).is_err());
        assert!(crate::query::check_history_at(root.root(),"../demo","review",None).is_err());
    }
    #[test]
    fn knowledge_jobs_refuse_missing_checkout_and_never_fetch_remote_resources() {
        let device=crate::testfixture::FixtureRoot::new("knowledge-no-checkout");
        let input=ReviewTargetInput::Remote{origin:"http://127.0.0.1:1".into(),project:"demo".into(),repo:"backend".into(),checkout_root:None};
        let jobs=SecurityJobs::default();let ticket=jobs.start_knowledge_at(device.root().into(),input,None,|_|{}).unwrap();
        assert!(jobs.cancel(&ticket.job_id,"another-project").is_err());
        assert_eq!(jobs.wait(&ticket.job_id,&ticket.project_key).unwrap_err().code,"source_unavailable");
        assert!(!device.root().join(".security_kb").exists());
    }
}

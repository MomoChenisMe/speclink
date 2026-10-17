//! Offline preliminary review orchestration. Does not call the data downloader.
use anyhow::{ensure, Result};
use speclink_core::security::{
    data::{sha256, StandardRepository},
    input::{
        CoverageEntry, InputKind, Language, ProjectInput, ScanInputs, ScanLimits, SourceDocument,
    },
    prepared::{
        valid_relative, validate_change, PreparedDocument, PreparedRepository, PreparedSnapshot,
        ReviewBinding,
    },
    report::{Report, ReportContext, ReportReceipt, ReportRepository},
    rules::{RuleResult, RuleSet},
    standards::{associate_standards, StandardId},
    JobProgress, ReviewStatus,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
#[derive(Clone)]
pub struct ReviewBudget {
    job_id: String,
    cancelled: Arc<AtomicBool>,
    elapsed: Arc<dyn Fn() -> Duration + Send + Sync>,
}
impl ReviewBudget {
    pub fn new(cancelled: Arc<AtomicBool>) -> Self {
        let start = Instant::now();
        Self {
            job_id: ulid::Ulid::new().to_string(),
            cancelled,
            elapsed: Arc::new(move || start.elapsed()),
        }
    }
    pub fn with_job_id(job_id: String, cancelled: Arc<AtomicBool>) -> Self {
        let mut budget = Self::new(cancelled);
        budget.job_id = job_id;
        budget
    }
    pub fn job_id(&self) -> &str {
        &self.job_id
    }
    pub fn stop_reason(&self) -> Option<&'static str> {
        if self.cancelled.load(Ordering::Relaxed) {
            Some("cancelled")
        } else if (self.elapsed)() >= Duration::from_secs(60) {
            Some("budget_exceeded")
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReviewTarget {
    pub binding: ReviewBinding,
    pub spec_dir: String,
    pub change_name: Option<String>,
    pub project_name: String,
}
impl ReviewTarget {
    pub fn local(
        root: &std::path::Path,
        spec_dir: &str,
        change_name: Option<String>,
    ) -> Result<Self> {
        let target = Self {
            binding: ReviewBinding {
                origin: None,
                project: None,
                repo: None,
                checkout: Some(root.to_string_lossy().into_owned()),
            },
            spec_dir: spec_dir.into(),
            change_name,
            project_name: root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        };
        target.validate()?;
        Ok(target)
    }
    pub fn validate(&self) -> Result<()> {
        self.binding.validate()?;
        validate_change(self.change_name.as_deref())?;
        ensure!(
            valid_relative(&self.spec_dir),
            "invalid_config: spec directory must stay within the project"
        );
        Ok(())
    }
}
#[derive(Clone)]
pub struct PreparationInput {
    pub policy_text: String,
    pub policy_revision: Option<u64>,
    pub documents: Vec<PreparedDocument>,
    pub locale: String,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareReceipt {
    pub project_key: String,
    pub prepared_at: String,
    pub policy_revision: Option<u64>,
    pub document_count: usize,
    pub checkout_available: bool,
}
pub fn prepare_review(
    target: &ReviewTarget,
    fetch: &dyn Fn() -> Result<PreparationInput>,
    repository: &dyn PreparedRepository,
    budget: &ReviewBudget,
) -> Result<PrepareReceipt> {
    target.validate()?;
    check_budget(budget)?;
    let input = fetch()?;
    check_budget(budget)?;
    let snapshot = PreparedSnapshot {
        format_version: 1,
        project_key: target.binding.project_key(),
        binding: target.binding.clone(),
        change_name: target.change_name.clone(),
        policy_digest: sha256(input.policy_text.as_bytes()),
        policy_text: input.policy_text,
        policy_revision: input.policy_revision,
        prepared_at: chrono::Utc::now().to_rfc3339(),
        documents_digest: sha256(&serde_json::to_vec(&input.documents)?),
        document_count: input.documents.len(),
        document_bytes: input.documents.iter().map(|d| d.text.len()).sum(),
        documents: input.documents,
        locale: input.locale,
    };
    snapshot.validate()?;
    check_budget(budget)?;
    repository.save(&snapshot)?;
    Ok(PrepareReceipt {
        project_key: snapshot.project_key,
        prepared_at: snapshot.prepared_at,
        policy_revision: snapshot.policy_revision,
        document_count: snapshot.documents.len(),
        checkout_available: target.binding.checkout.is_some(),
    })
}
fn check_budget(budget: &ReviewBudget) -> Result<()> {
    if let Some(reason) = budget.stop_reason() {
        anyhow::bail!(reason);
    }
    Ok(())
}
fn empty_inputs() -> ScanInputs {
    ScanInputs {
        visited_files: 0,
        documents: Vec::new(),
        coverage: Vec::new(),
        remaining_targets: Vec::new(),
        total_bytes: 0,
        complete: true,
        stop_reason: None,
    }
}
fn merge_inputs(into: &mut ScanInputs, mut next: ScanInputs) {
    into.visited_files += next.visited_files;
    into.documents.append(&mut next.documents);
    into.coverage.append(&mut next.coverage);
    into.remaining_targets.append(&mut next.remaining_targets);
    into.total_bytes += next.total_bytes;
    into.complete &= next.complete;
    if next.stop_reason.is_some() {
        into.stop_reason = next.stop_reason;
    }
}
pub fn local_preparation(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    budget: &ReviewBudget,
) -> Result<PreparationInput> {
    target.validate()?;
    ensure!(
        target.binding.checkout.as_deref() == reader.root().to_str(),
        "binding_mismatch"
    );
    let policy_text = reader
        .read_optional_text(
            &format!("{}/config.yaml", target.spec_dir),
            64 * 1024,
            &|| budget.stop_reason(),
        )?
        .unwrap_or_default();
    let locale = effective_locale(&policy_text)?;
    let mut inputs = empty_inputs();
    local_specs(target, reader, &[], budget, &mut inputs)?;
    ensure!(
        inputs.complete,
        "incomplete_preparation: could not read all selected specifications"
    );
    Ok(PreparationInput {
        policy_text,
        policy_revision: None,
        documents: inputs
            .documents
            .into_iter()
            .map(|d| PreparedDocument::new(d.path, d.text.to_string(), None))
            .collect(),
        locale,
    })
}
fn effective_locale(policy_text: &str) -> Result<String> {
    let policy =
        crate::policy::resolve_effective_policy(|key| std::env::var(key).ok(), Some(policy_text))
            .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?;
    Ok(match policy.resolved().locale.as_str() {
        "Traditional Chinese (繁體中文)" => "tw",
        "Japanese (日本語)" => "ja",
        _ => "en",
    }
    .into())
}
pub fn preparation_from_context(
    target: &ReviewTarget,
    snapshot: speclink_protocol::context::ContextSnapshot,
) -> Result<PreparationInput> {
    ensure!(
        snapshot.documents.len() <= 10_000
            && snapshot
                .documents
                .iter()
                .map(|d| d.content.len())
                .sum::<usize>()
                <= 104_857_600,
        "size_limit: remote specifications"
    );
    let mut policy_text = None;
    let mut documents = Vec::new();
    let mut paths = std::collections::BTreeSet::new();
    for document in snapshot.documents {
        ensure!(
            valid_relative(&document.path)
                && paths.insert(document.path.clone())
                && format!("sha256:{}", sha256(document.content.as_bytes())) == document.digest,
            "invalid_preparation: remote document digest or path"
        );
        if document.path == "openspec/config.yaml" {
            ensure!(
                document.content.len() <= 64 * 1024,
                "size_limit: remote policy"
            );
            policy_text = Some(document.content);
            continue;
        }
        let selected = if let Some(change) = &target.change_name {
            let prefix = format!("openspec/changes/{change}/");
            document.path.strip_prefix(&prefix).is_some_and(|path| {
                matches!(path, "proposal.md" | "design.md" | "tasks.md")
                    || (path.starts_with("specs/") && path.ends_with("/spec.md"))
            })
        } else {
            document.path.starts_with("openspec/specs/") && document.path.ends_with("/spec.md")
        };
        if selected {
            documents.push(PreparedDocument::new(
                document.path,
                document.content,
                document.revision,
            ));
        }
    }
    let policy_text = policy_text.ok_or_else(|| {
        anyhow::anyhow!("invalid_preparation: remote snapshot lacks canonical policy")
    })?;
    let locale = effective_locale(&policy_text)?;
    Ok(PreparationInput {
        policy_text,
        policy_revision: snapshot.policy_revision,
        documents,
        locale,
    })
}
fn remaining_limits(inputs: &ScanInputs) -> ScanLimits {
    let defaults = ScanLimits::default();
    ScanLimits {
        total_bytes: defaults
            .total_bytes
            .saturating_sub(inputs.total_bytes as usize),
        files: defaults.files.saturating_sub(inputs.visited_files),
        ..defaults
    }
}
fn local_specs(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    exclude: &[String],
    budget: &ReviewBudget,
    inputs: &mut ScanInputs,
) -> Result<Vec<SourceDocument>> {
    let mut documents = Vec::new();
    let base = if let Some(change) = &target.change_name {
        let base = format!("{}/changes/{change}", target.spec_dir);
        ensure!(reader.directory_exists(&base)?, "unknown_change");
        for artifact in ["proposal", "design", "tasks"] {
            check_budget(budget)?;
            let path = format!("{base}/{artifact}.md");
            if speclink_core::security::input::is_excluded(std::path::Path::new(&path), exclude) {
                inputs.coverage.push(CoverageEntry {
                    path,
                    status: ReviewStatus::NotApplicable,
                    reason: Some("excluded".into()),
                    bytes: 0,
                });
                continue;
            }
            let limits = remaining_limits(inputs);
            if limits.files == 0 || limits.total_bytes == 0 {
                inputs.complete = false;
                inputs.stop_reason = Some(
                    if limits.files == 0 {
                        "file_limit"
                    } else {
                        "byte_limit"
                    }
                    .into(),
                );
                inputs.remaining_targets.push(path);
                return Ok(documents);
            }
            match reader.read_optional_text(
                &path,
                limits.file_bytes.min(limits.total_bytes),
                &|| budget.stop_reason(),
            ) {
                Ok(Some(text)) => {
                    inputs.visited_files += 1;
                    inputs.total_bytes += text.len() as u64;
                    let doc = SourceDocument::new(path.clone(), text);
                    inputs.coverage.push(CoverageEntry {
                        path,
                        status: ReviewStatus::NotVerified,
                        reason: Some("awaiting_rules".into()),
                        bytes: doc.text.len() as u64,
                    });
                    inputs.documents.push(doc.clone());
                    documents.push(doc);
                }
                Ok(None) => {}
                Err(error) => {
                    let failure =
                        error.downcast_ref::<speclink_core::security::input::InputReadFailure>();
                    let consumed = failure.map(|f| f.bytes_read).unwrap_or(0);
                    inputs.total_bytes += consumed as u64;
                    inputs.visited_files += 1;
                    inputs.complete = false;
                    inputs.coverage.push(CoverageEntry {
                        path,
                        status: ReviewStatus::NotVerified,
                        reason: Some(
                            budget
                                .stop_reason()
                                .or_else(|| failure.map(|f| f.code))
                                .unwrap_or("io_error")
                                .into(),
                        ),
                        bytes: consumed as u64,
                    });
                }
            }
        }
        format!("{base}/specs")
    } else {
        format!("{}/specs", target.spec_dir)
    };
    if speclink_core::security::input::is_excluded(std::path::Path::new(&base), exclude) {
        inputs.coverage.push(CoverageEntry {
            path: base,
            status: ReviewStatus::NotApplicable,
            reason: Some("excluded".into()),
            bytes: 0,
        });
        return Ok(documents);
    }
    if reader.directory_exists(&base)? {
        let scanned = reader.scan_selected(
            &[base],
            InputKind::Spec,
            exclude,
            &remaining_limits(inputs),
            &|| budget.stop_reason(),
        )?;
        documents.extend(scanned.documents.iter().cloned());
        merge_inputs(inputs, scanned);
    }
    Ok(documents)
}
pub struct RunOutcome {
    pub report: Report,
    pub receipt: ReportReceipt,
}
pub fn run_review(
    target: &ReviewTarget,
    reader: Option<&dyn ProjectInput>,
    data: &dyn StandardRepository,
    prepared: &dyn PreparedRepository,
    reports: &dyn ReportRepository,
    budget: &ReviewBudget,
    progress: &dyn Fn(JobProgress),
) -> Result<RunOutcome> {
    static RUNNING: AtomicBool = AtomicBool::new(false);
    ensure!(
        RUNNING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok(),
        "busy: another security review is running in this process"
    );
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            RUNNING.store(false, Ordering::Release);
        }
    }
    let _guard = Guard;
    target.validate()?;
    check_budget(budget)?;
    if let Some(reader) = reader {
        ensure!(
            target.binding.checkout.as_deref() == reader.root().to_str(),
            "binding_mismatch: selected checkout differs"
        );
    }
    let project_key = target.binding.project_key();
    let header = if target.binding.is_remote() {
        let header = prepared
            .load_policy(&project_key, target.change_name.as_deref())?
            .ok_or_else(|| {
                anyhow::anyhow!("missing_preparation: explicitly prepare this remote target first")
            })?;
        header.validate_header()?;
        ensure!(
            header.binding == target.binding && header.change_name == target.change_name,
            "binding_mismatch"
        );
        Some(header)
    } else {
        None
    };
    let policy_text = if let Some(header) = &header {
        header.policy_text.clone()
    } else {
        reader
            .ok_or_else(|| anyhow::anyhow!("unauthorized_root"))?
            .read_optional_text(
                &format!("{}/config.yaml", target.spec_dir),
                64 * 1024,
                &|| budget.stop_reason(),
            )?
            .unwrap_or_default()
    };
    let workflow = speclink_core::config::WorkflowConfig::from_text(Some(&policy_text))
        .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?;
    let policy = workflow.security();
    let locale = if let Some(header) = &header {
        header.locale.clone()
    } else {
        effective_locale(&policy_text)?
    };
    let configuration_state = if !policy.enabled {
        "disabled"
    } else if policy.standards.selected().is_empty() {
        "unconfigured"
    } else {
        "configured"
    };
    let mut inputs = empty_inputs();
    let mut results = Vec::new();
    let rules = RuleSet::builtin()?;
    let mut installed = Vec::new();
    let mut pins = Vec::new();
    let mut invalid = std::collections::BTreeSet::new();
    let emit = |phase: &str, inputs: &ScanInputs| {
        progress(JobProgress {
            job_id: budget.job_id.clone(),
            project_key: project_key.clone(),
            phase: phase.into(),
            completed_bytes: inputs.total_bytes,
            total_bytes: None,
            completed_files: inputs.documents.len(),
            message: phase.into(),
        })
    };
    data.remember_policy(
        &if target.binding.is_remote() {
            format!("remote:{project_key}")
        } else {
            format!("local:{}", target.binding.checkout.as_deref().unwrap_or(""))
        },
        &policy,
    )?;
    if policy.enabled {
        for (id, version, _) in policy.standards.selected() {
            let id: StandardId = id.parse()?;
            match data.pin(id, version).and_then(|pin| {
                let item = data.read(id, version)?;
                Ok((pin, item))
            }) {
                Ok((pin, Some(item))) => {
                    pins.push(pin);
                    installed.push(item);
                }
                Ok((_, None)) => {}
                Err(error) => {
                    if !error.to_string().starts_with("missing_data") {
                        invalid.insert(id);
                    }
                }
            }
        }
        if let Some(reader) = reader {
            if let Some(change) = target
                .change_name
                .as_ref()
                .filter(|_| !target.binding.is_remote())
            {
                ensure!(
                    reader.directory_exists(&format!("{}/changes/{change}", target.spec_dir))?,
                    "unknown_change"
                );
            }
            if policy.reviews.code
                || (policy.reviews.api_boundary
                    && policy.scope != speclink_core::security::policy::SecurityScope::Web)
            {
                emit("reading", &inputs);
                match reader.scan_selected(
                    &[],
                    if policy.reviews.code {
                        InputKind::Code
                    } else {
                        InputKind::Api
                    },
                    &policy.exclude_dirs,
                    &remaining_limits(&inputs),
                    &|| budget.stop_reason(),
                ) {
                    Ok(next) => merge_inputs(&mut inputs, next),
                    Err(_) => {
                        inputs.complete = false;
                        inputs.coverage.push(CoverageEntry {
                            path: String::new(),
                            status: ReviewStatus::NotVerified,
                            reason: Some(budget.stop_reason().unwrap_or("io_error").into()),
                            bytes: 0,
                        });
                    }
                }
            }
        }
    }
    let code_documents = inputs.documents.clone();
    emit("checking", &inputs);
    let mut finding_count = 0usize;
    if policy.enabled && policy.reviews.code {
        if reader.is_none() || code_documents.is_empty() {
            results.extend(family_results(
                &rules,
                "code-review",
                ReviewStatus::NotVerified,
                if reader.is_none() {
                    "source_unavailable"
                } else {
                    "target_information_insufficient"
                },
            ));
        }
        for (index, document) in code_documents.iter().enumerate() {
            if let Some(reason) = budget.stop_reason() {
                inputs.complete = false;
                inputs.stop_reason = Some(reason.into());
                inputs
                    .remaining_targets
                    .extend(code_documents[index..].iter().map(|d| d.path.clone()));
                break;
            }
            match rules.review_code(document, &policy.disabled_rules, &|| budget.stop_reason()) {
                Ok(mut checked) => {
                    if policy.scope == speclink_core::security::policy::SecurityScope::Api {
                        for result in checked.iter_mut().filter(|r| r.rule_id == "SL-SEC-003") {
                            result.status = ReviewStatus::NotApplicable;
                            result.reason = Some("scope_api_excludes_web_output".into());
                            result.findings.clear();
                        }
                    }
                    for result in &mut checked {
                        let remaining = 10_000usize.saturating_sub(finding_count);
                        if result.findings.len() > remaining {
                            result.findings.truncate(remaining);
                            result.status = ReviewStatus::NotVerified;
                            result.reason = Some("finding_limit".into());
                        }
                        finding_count += result.findings.len();
                    }
                    results.append(&mut checked);
                }
                Err(_) => {
                    inputs.complete = false;
                    inputs.stop_reason = Some(budget.stop_reason().unwrap_or("rule_error").into());
                    inputs
                        .remaining_targets
                        .extend(code_documents[index..].iter().map(|d| d.path.clone()));
                    break;
                }
            }
            if finding_count >= 10_000 && index + 1 < code_documents.len() {
                inputs.complete = false;
                inputs.stop_reason = Some("finding_limit".into());
                inputs
                    .remaining_targets
                    .extend(code_documents[index + 1..].iter().map(|d| d.path.clone()));
                break;
            }
        }
    } else {
        results.extend(family_results(
            &rules,
            "code-review",
            ReviewStatus::Disabled,
            "review_disabled",
        ));
    }
    if policy.enabled && policy.reviews.spec {
        let specifications = if let Some(header) = &header {
            let limits = remaining_limits(&inputs);
            let budget_reason = if header.document_count > limits.files {
                Some("file_limit")
            } else if header.document_bytes > limits.total_bytes {
                Some("byte_limit")
            } else {
                budget.stop_reason()
            };
            let loaded = if let Some(reason) = budget_reason {
                inputs.complete = false;
                inputs.stop_reason = Some(reason.into());
                Ok(None)
            } else {
                prepared.load(&project_key, target.change_name.as_deref())
            };
            let full = loaded.ok().flatten().filter(|full| {
                full.validate_binding(&target.binding, target.change_name.as_deref())
                    .is_ok()
                    && full.prepared_at == header.prepared_at
                    && full.policy_digest == header.policy_digest
                    && full.documents_digest == header.documents_digest
            });
            let docs = if let Some(full) = full {
                full.documents
                    .into_iter()
                    .map(|d| SourceDocument::new(d.path, d.text))
                    .collect::<Vec<_>>()
            } else {
                inputs.complete = false;
                inputs.coverage.push(CoverageEntry {
                    path: String::new(),
                    status: ReviewStatus::NotVerified,
                    reason: Some("prepared_snapshot_unavailable_or_changed".into()),
                    bytes: 0,
                });
                Vec::new()
            };
            let total = docs.iter().map(|d| d.text.len()).sum::<usize>();
            if total > remaining_limits(&inputs).total_bytes {
                inputs.complete = false;
                inputs.stop_reason = Some("byte_limit".into());
                Vec::new()
            } else {
                inputs.total_bytes += total as u64;
                inputs.visited_files += docs.len();
                inputs.documents.extend(docs.iter().cloned());
                docs
            }
        } else {
            match local_specs(
                target,
                reader.unwrap(),
                &policy.exclude_dirs,
                budget,
                &mut inputs,
            ) {
                Ok(docs) => docs,
                Err(_) => {
                    inputs.complete = false;
                    inputs.stop_reason = Some(budget.stop_reason().unwrap_or("io_error").into());
                    Vec::new()
                }
            }
        };
        match rules.review_specs(&specifications, &policy.disabled_rules, &|| {
            budget.stop_reason()
        }) {
            Ok(mut checked) => results.append(&mut checked),
            Err(_) => results.extend(family_results(
                &rules,
                "spec-review",
                ReviewStatus::NotVerified,
                budget.stop_reason().unwrap_or("rule_error"),
            )),
        }
    } else {
        results.extend(family_results(
            &rules,
            "spec-review",
            ReviewStatus::Disabled,
            "review_disabled",
        ));
    }
    if policy.enabled && policy.reviews.api_boundary {
        if policy.scope == speclink_core::security::policy::SecurityScope::Web {
            results.extend(family_results(
                &rules,
                "api-boundary",
                ReviewStatus::NotApplicable,
                "scope_web_excludes_api",
            ));
        } else if reader.is_none() {
            results.extend(family_results(
                &rules,
                "api-boundary",
                ReviewStatus::NotVerified,
                "source_unavailable",
            ));
        } else {
            let api_documents = code_documents
                .iter()
                .filter(|d| matches!(d.language, Language::Json | Language::Yaml))
                .cloned()
                .collect::<Vec<_>>();
            match rules.review_boundaries(&api_documents, &policy.disabled_rules, &|| {
                budget.stop_reason()
            }) {
                Ok(mut checked) => results.append(&mut checked),
                Err(_) => results.extend(family_results(
                    &rules,
                    "api-boundary",
                    ReviewStatus::NotVerified,
                    budget.stop_reason().unwrap_or("rule_error"),
                )),
            }
        }
    } else {
        results.extend(family_results(
            &rules,
            "api-boundary",
            ReviewStatus::Disabled,
            "review_disabled",
        ));
    }
    if let Some(reason) = budget.stop_reason() {
        inputs.complete = false;
        inputs.stop_reason = Some(reason.into());
    }
    let mut remaining = 10_000usize;
    let mut per_file = std::collections::BTreeMap::<String, usize>::new();
    for result in &mut results {
        if policy.disabled_rules.contains(&result.rule_id) {
            result.status = ReviewStatus::Disabled;
            result.reason = Some("rule_disabled".into());
            result.findings.clear();
        }
        let mut limit_hit = false;
        result.findings.retain(|finding| {
            let count = per_file.entry(finding.path.clone()).or_default();
            if remaining == 0 || *count >= 100 {
                limit_hit = true;
                false
            } else {
                *count += 1;
                remaining -= 1;
                true
            }
        });
        if limit_hit {
            result.status = ReviewStatus::NotVerified;
            result.reason = Some("finding_limit".into());
            inputs.complete = false;
            inputs.stop_reason = Some("finding_limit".into());
        }
    }
    let mut assessments =
        associate_standards(&policy, &installed, &mut results, &|| budget.stop_reason());
    for assessment in &mut assessments {
        if invalid.contains(&assessment.standard_id) {
            assessment.reason = Some("invalid_data".into());
        }
    }
    if let Some(reason) = budget.stop_reason() {
        inputs.complete = false;
        inputs.stop_reason = Some(reason.into());
    }
    let context = ReportContext {
        report_id: ulid::Ulid::new().to_string(),
        project_key: project_key.clone(),
        project_name: target.project_name.clone(),
        change_name: target.change_name.clone(),
        reviewed_at: chrono::Utc::now().to_rfc3339(),
        commit_sha: if policy.enabled && budget.stop_reason().is_none() {
            reader.and_then(|r| git_commit(r.root(), budget))
        } else {
            None
        },
        policy_digest: sha256(policy_text.as_bytes()),
        policy_revision: header.as_ref().and_then(|h| h.policy_revision),
        prepared_at: header.map(|h| h.prepared_at),
        rule_version: rules.rule_version,
        scope: policy.scope,
        locale,
        review_methods: if policy.enabled {
            [
                (policy.reviews.spec, "spec"),
                (policy.reviews.code, "code"),
                (policy.reviews.api_boundary, "api-boundary"),
            ]
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| name.into())
            .collect()
        } else {
            Vec::new()
        },
    };
    if let Some(reason) = budget.stop_reason() {
        inputs.complete = false;
        inputs.stop_reason = Some(reason.into());
    }
    let report = Report::new(
        context,
        configuration_state,
        inputs.complete,
        results,
        assessments,
        &inputs,
    )?;
    emit("saving", &inputs);
    let receipt = reports.save(&report)?;
    drop(pins);
    Ok(RunOutcome { report, receipt })
}
fn family_results(
    rules: &RuleSet,
    family: &str,
    status: ReviewStatus,
    reason: &str,
) -> Vec<RuleResult> {
    rules
        .rules
        .iter()
        .filter(|r| r.rule_type == family)
        .map(|r| RuleResult {
            rule_id: r.id.clone(),
            path: String::new(),
            status,
            reason: Some(reason.into()),
            findings: Vec::new(),
        })
        .collect()
}
fn git_commit(root: &std::path::Path, budget: &ReviewBudget) -> Option<String> {
    #[cfg(target_os = "macos")]
    let candidates = [
        "/Library/Developer/CommandLineTools/usr/bin/git",
        "/Applications/Xcode.app/Contents/Developer/usr/bin/git",
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    let candidates = ["/usr/bin/git", "/bin/git"];
    #[cfg(windows)]
    let candidates = [
        r"C:\Program Files\Git\cmd\git.exe",
        r"C:\Program Files (x86)\Git\cmd\git.exe",
    ];
    #[cfg(not(any(unix, windows)))]
    {
        let _ = root;
        return None;
    }
    #[cfg(any(unix, windows))]
    {
        use std::process::{Command, Stdio};
        let binary = candidates
            .into_iter()
            .find(|p| std::path::Path::new(p).is_file())?;
        let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let mut child = Command::new(binary)
            .current_dir(root)
            .env_clear()
            .env(
                "PATH",
                if cfg!(windows) {
                    r"C:\Windows\System32"
                } else {
                    "/usr/bin:/bin"
                },
            )
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", null)
            .env("GIT_TERMINAL_PROMPT", "0")
            .args([
                "--no-pager",
                "--no-optional-locks",
                "-c",
                "core.fsmonitor=false",
                "-c",
                if cfg!(windows) {
                    "core.hooksPath=NUL"
                } else {
                    "core.hooksPath=/dev/null"
                },
                "rev-parse",
                "--verify",
                "HEAD",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let start = Instant::now();
        loop {
            if child.try_wait().ok()?.is_some() {
                break;
            }
            if start.elapsed() >= Duration::from_secs(2) || budget.stop_reason().is_some() {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().ok()?;
        if !output.status.success() {
            return None;
        }
        let sha = String::from_utf8(output.stdout).ok()?.trim().to_owned();
        (matches!(sha.len(), 40 | 64) && sha.bytes().all(|b| b.is_ascii_hexdigit())).then_some(sha)
    }
}
#[derive(Clone, Copy)]
pub enum ReportFormat {
    Json,
    Markdown,
}
pub fn export_report(
    repository: &dyn speclink_core::security::report::ReportRepository,
    id: &str,
    format: ReportFormat,
) -> anyhow::Result<String> {
    let report = repository
        .get(id)?
        .ok_or_else(|| anyhow::anyhow!("report_not_found"))?;
    match format {
        ReportFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
        ReportFormat::Markdown => Ok(report.markdown()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    static REVIEW_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    #[test]
    fn security_budget_deadline_and_cancel_are_not_success() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };
        let token = Arc::new(AtomicBool::new(false));
        let budget = ReviewBudget::new(token.clone());
        assert_eq!(budget.stop_reason(), None);
        token.store(true, Ordering::Relaxed);
        assert_eq!(budget.stop_reason(), Some("cancelled"));
        token.store(false, Ordering::Relaxed);
        let budget = ReviewBudget {
            job_id: "test-budget".into(),
            cancelled: token,
            elapsed: Arc::new(|| std::time::Duration::from_secs(60)),
        };
        assert_eq!(budget.stop_reason(), Some("budget_exceeded"));
    }
    #[test]
    fn security_report_unknown_and_traversal_never_return_a_report() {
        let dir = tempfile::tempdir().unwrap();
        let repository =
            speclink_fs::security::reports::FsSecurityReports::new(dir.path()).unwrap();
        for id in ["../../secret", "01ARZ3NDEKTSV4RRFFQ69G5FAV"] {
            assert!(export_report(&repository, id, ReportFormat::Json).is_err());
        }
        assert!(!dir.path().join("security").exists());
    }
    #[test]
    fn security_run_disabled_reads_only_policy_and_basic_rules_remain_independent() {
        let _test_guard = REVIEW_TEST_LOCK.lock().unwrap();
        use speclink_core::security::input::{InputKind, ProjectInput, ScanInputs, ScanLimits};
        struct Spy {
            inner: speclink_fs::security::scan::AuthorizedProject,
            scans: std::sync::atomic::AtomicUsize,
        }
        impl ProjectInput for Spy {
            fn root(&self) -> &std::path::Path {
                self.inner.root()
            }
            fn read_optional_text(
                &self,
                path: &str,
                limit: usize,
                stop: &dyn Fn() -> Option<&'static str>,
            ) -> anyhow::Result<Option<String>> {
                assert_eq!(path, "openspec/config.yaml");
                self.inner.read_optional_text(path, limit, stop)
            }
            fn directory_exists(&self, _: &str) -> anyhow::Result<bool> {
                panic!("disabled policy must not inspect artifacts");
            }
            fn scan_selected(
                &self,
                paths: &[String],
                kind: InputKind,
                exclude: &[String],
                limits: &ScanLimits,
                stop: &dyn Fn() -> Option<&'static str>,
            ) -> anyhow::Result<ScanInputs> {
                self.scans.fetch_add(1, Ordering::Relaxed);
                self.inner.scan_selected(paths, kind, exclude, limits, stop)
            }
        }
        let project = tempfile::tempdir().unwrap();
        let device = tempfile::tempdir().unwrap();
        std::fs::create_dir(project.path().join("openspec")).unwrap();
        std::fs::write(
            project.path().join("openspec/config.yaml"),
            "tdd: true\naudit: true\n",
        )
        .unwrap();
        std::fs::write(
            project.path().join("app.ts"),
            "const token='real-credential-sentinel-111';",
        )
        .unwrap();
        let reader = Spy {
            inner: speclink_fs::security::scan::AuthorizedProject::open(project.path()).unwrap(),
            scans: std::sync::atomic::AtomicUsize::new(0),
        };
        let target = ReviewTarget::local(reader.root(), "openspec", None).unwrap();
        let data = speclink_fs::security::FsSecurityData::new(device.path()).unwrap();
        let prepared =
            speclink_fs::security::prepared::FsPreparedReviews::new(device.path()).unwrap();
        let reports =
            speclink_fs::security::reports::FsSecurityReports::new(device.path()).unwrap();
        let outcome = run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &reports,
            &ReviewBudget::new(Arc::new(AtomicBool::new(false))),
            &|_| {},
        )
        .unwrap();
        assert_eq!(outcome.report.configuration_state(), "disabled");
        assert_eq!(reader.scans.load(Ordering::Relaxed), 0);
        assert!(outcome.report.is_complete());
        let policy="security_verification:\n  enabled: true\n  reviews: {spec: false, code: true, api_boundary: false}\n";
        std::fs::write(project.path().join("openspec/config.yaml"), policy).unwrap();
        let outcome = run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &reports,
            &ReviewBudget::new(Arc::new(AtomicBool::new(false))),
            &|_| {},
        )
        .unwrap();
        assert_eq!(outcome.report.configuration_state(), "unconfigured");
        assert_eq!(outcome.report.counts().findings, 1);
        assert!(outcome.report.is_complete());
        assert!(!serde_json::to_string(&outcome.report)
            .unwrap()
            .contains("real-credential-sentinel-111"));
    }
    #[test]
    fn security_prepare_failure_preserves_snapshot_and_remote_run_cannot_borrow_another_checkout() {
        let _test_guard = REVIEW_TEST_LOCK.lock().unwrap();
        use speclink_core::security::prepared::{PreparedRepository, ReviewBinding};
        let device = tempfile::tempdir().unwrap();
        let prepared =
            speclink_fs::security::prepared::FsPreparedReviews::new(device.path()).unwrap();
        let data = speclink_fs::security::FsSecurityData::new(device.path()).unwrap();
        let reports =
            speclink_fs::security::reports::FsSecurityReports::new(device.path()).unwrap();
        let target = ReviewTarget {
            binding: ReviewBinding {
                origin: Some("https://example.invalid".into()),
                project: Some("p".into()),
                repo: Some("r".into()),
                checkout: None,
            },
            spec_dir: "openspec".into(),
            change_name: Some("demo".into()),
            project_name: "Remote".into(),
        };
        let budget = ReviewBudget::new(Arc::new(AtomicBool::new(false)));
        let input = PreparationInput {
            policy_text: "security_verification: {enabled: true}\n".into(),
            policy_revision: Some(7),
            documents: Vec::new(),
            locale: "en".into(),
        };
        prepare_review(&target, &|| Ok(input.clone()), &prepared, &budget).unwrap();
        let before = prepared
            .load(&target.binding.project_key(), Some("demo"))
            .unwrap()
            .unwrap();
        assert!(prepare_review(
            &target,
            &|| anyhow::bail!("401 authentication failed"),
            &prepared,
            &budget
        )
        .is_err());
        let after = prepared
            .load(&target.binding.project_key(), Some("demo"))
            .unwrap()
            .unwrap();
        assert_eq!(before.policy_revision, after.policy_revision);
        assert_eq!(before.prepared_at, after.prepared_at);
        let outcome =
            run_review(&target, None, &data, &prepared, &reports, &budget, &|_| {}).unwrap();
        assert!(!outcome.report.is_complete());
        assert!(serde_json::to_string(&outcome.report)
            .unwrap()
            .contains("source_unavailable"));
        let mut wrong = target.clone();
        wrong.binding.origin = Some("https://another.invalid".into());
        assert!(before
            .validate_binding(&wrong.binding, wrong.change_name.as_deref())
            .is_err());
        let root = tempfile::tempdir().unwrap();
        let reader = speclink_fs::security::scan::AuthorizedProject::open(root.path()).unwrap();
        assert!(run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &reports,
            &budget,
            &|_| {}
        )
        .is_err());
    }
    #[test]
    fn security_run_cancel_preserves_partial_report_without_source_evidence() {
        let _test_guard = REVIEW_TEST_LOCK.lock().unwrap();
        let root = tempfile::tempdir().unwrap();
        let device = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("openspec")).unwrap();
        std::fs::write(root.path().join("openspec/config.yaml"),"security_verification:\n  enabled: true\n  reviews: {spec: false, api_boundary: false}\n").unwrap();
        std::fs::write(
            root.path().join("app.ts"),
            "const token='cancel-private-secret-sentinel';",
        )
        .unwrap();
        let reader = speclink_fs::security::scan::AuthorizedProject::open(root.path()).unwrap();
        let target = ReviewTarget::local(reader.root(), "openspec", None).unwrap();
        let data = speclink_fs::security::FsSecurityData::new(device.path()).unwrap();
        let prepared =
            speclink_fs::security::prepared::FsPreparedReviews::new(device.path()).unwrap();
        let reports =
            speclink_fs::security::reports::FsSecurityReports::new(device.path()).unwrap();
        let token = Arc::new(AtomicBool::new(false));
        let budget = ReviewBudget::new(token.clone());
        let outcome = run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &reports,
            &budget,
            &|event| {
                if event.phase == "checking" {
                    token.store(true, Ordering::Relaxed);
                }
            },
        )
        .unwrap();
        assert!(outcome.report.is_cancelled());
        assert!(!outcome.report.is_complete());
        assert!(outcome.report.counts().not_verified > 0);
        let saved = reports.get(outcome.report.report_id()).unwrap().unwrap();
        assert!(saved.is_cancelled());
        assert!(!saved.markdown().contains("cancel-private-secret-sentinel"));
    }
    #[test]
    fn security_run_shares_per_file_finding_limit_across_review_families() {
        let _guard = REVIEW_TEST_LOCK.lock().unwrap();
        let root = tempfile::tempdir().unwrap();
        let device = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("openspec")).unwrap();
        std::fs::write(root.path().join("openspec/config.yaml"), "security_verification:\n  enabled: true\n  reviews: {spec: false, code: true, api_boundary: true}\n").unwrap();
        let mut api = serde_json::json!({"openapi":"3.0.3", "paths":{}});
        for index in 0..100 {
            api[format!("x-unsafe-{index}")] = serde_json::json!({"rejectUnauthorized":false});
            api["paths"][format!("/item-{index}")] = serde_json::json!({"get":{"parameters":[{"name":"q","in":"query","schema":{"type":"string"}}]}});
        }
        std::fs::write(
            root.path().join("openapi.json"),
            serde_json::to_string_pretty(&api).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.path().join("other.json"),
            r#"{"rejectUnauthorized": false}"#,
        )
        .unwrap();
        let reader = speclink_fs::security::scan::AuthorizedProject::open(root.path()).unwrap();
        let target = ReviewTarget::local(reader.root(), "openspec", None).unwrap();
        let data = speclink_fs::security::FsSecurityData::new(device.path()).unwrap();
        let prepared =
            speclink_fs::security::prepared::FsPreparedReviews::new(device.path()).unwrap();
        let reports =
            speclink_fs::security::reports::FsSecurityReports::new(device.path()).unwrap();
        let outcome = run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &reports,
            &ReviewBudget::new(Arc::new(AtomicBool::new(false))),
            &|_| {},
        )
        .unwrap();
        let report = serde_json::to_value(&outcome.report).unwrap();
        let findings = report["findings"].as_array().unwrap();
        assert_eq!(
            findings
                .iter()
                .filter(|f| f["path"] == "openapi.json")
                .count(),
            100
        );
        assert_eq!(
            findings
                .iter()
                .filter(|f| f["path"] == "other.json")
                .count(),
            1
        );
        assert!(!outcome.report.is_complete());
        assert!(report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["target"] == "openapi.json"
                && c["status"] == "NOT_VERIFIED"
                && c["reason"] == "finding_limit"));
    }
}

//! Validates skill submissions against local, authorized evidence. No downloader is used.
use super::*;
use speclink_core::security::{
    acceptance::{self, Acceptance, FreshEvidence, RiskDecision, SourceFreshness},
    policy::SecurityPolicy,
};
use speclink_core::{
    history,
    security::{knowledge::KnowledgeManifest, report::*},
    store::Store,
};
use std::collections::{BTreeMap, BTreeSet};

/// Stable proposal identity supplied by the server; checkout identity remains device-local.
pub fn remote_project_key(public_url: &str, project: &str, repo: &str) -> Result<String> {
    let url = url::Url::parse(public_url)
        .map_err(|_| anyhow::anyhow!("invalid_config: server public_url"))?;
    ensure!(
        matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
        "invalid_config: server public_url"
    );
    let binding = ReviewBinding {
        origin: Some(url.origin().ascii_serialization()),
        project: Some(project.into()),
        repo: Some(repo.into()),
        checkout: None,
    };
    binding.validate()?;
    Ok(binding.project_key())
}

pub fn fresh_evidence(context: &DeepReportContext) -> FreshEvidence {
    FreshEvidence {
        policy_digest: context.common.policy_digest.clone(),
        knowledge_digest: context.knowledge_digest.clone(),
        scope_digest: context.scope_digest.clone(),
        required_checks: context.required_checks.clone(),
        source_freshness: SourceFreshness::Local,
    }
}

pub fn acceptance_proof_for_target(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    store: &dyn Store,
    budget: &ReviewBudget,
) -> Result<Option<acceptance::AcceptanceProof>> {
    target.validate()?;
    ensure!(
        target.binding.checkout.as_deref() == reader.root().to_str(),
        "binding_mismatch"
    );
    let change = target
        .change_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("unknown_change"))?;
    ensure!(store.change_exists(change), "unknown_change");
    let text = reader
        .read_optional_text(
            &format!("{}/config.yaml", target.spec_dir),
            64 * 1024,
            &|| budget.stop_reason(),
        )?
        .unwrap_or_default();
    let policy = speclink_core::config::WorkflowConfig::from_text(Some(&text))
        .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?
        .security();
    if !(policy.enabled && policy.required_for_acceptance) {
        return Ok(None);
    }
    store.ensure_checks_readable(change)?;
    let index_digest = store
        .read_artifact(change, history::INDEX)
        .map(|s| sha256(s.as_bytes()));
    let (builtin, deep) = current_reports(store, change)?;
    let mut decisions = vec![];
    for id in deep
        .iter()
        .map(DeepReport::report_id)
        .chain(builtin.iter().map(Report::report_id))
    {
        decisions.extend(read_decisions(store, change, id)?);
    }
    let mut reasons = vec![];
    let fresh = if deep.is_some() {
        match submission_context(target, reader, data, store, None, budget) {
            Ok((context, _)) => Some(fresh_evidence(&context)),
            Err(error) => {
                reasons.push(
                    error
                        .to_string()
                        .split(':')
                        .next()
                        .unwrap_or("source_freshness_unverified")
                        .into(),
                );
                None
            }
        }
    } else {
        None
    };
    store.ensure_checks_readable(change)?;
    ensure!(
        index_digest
            == store
                .read_artifact(change, history::INDEX)
                .map(|s| sha256(s.as_bytes())),
        "revision_conflict"
    );
    Ok(Some(acceptance::AcceptanceProof {
        change_name: change.into(),
        project_key: target.report_project_key(),
        index_digest,
        builtin,
        deep,
        decisions,
        fresh,
        reasons,
    }))
}

/// Choose the latest deep submission; an incomplete latest round cannot fall back to an older PASS.
pub fn current_reports(
    store: &dyn Store,
    change: &str,
) -> Result<(Option<Report>, Option<DeepReport>)> {
    let index = history::index(store, change)?;
    for entry in index
        .entries
        .iter()
        .rev()
        .filter(|e| e.kind == history::CheckKind::Security && e.outcome != "legacy")
    {
        if let Some(VersionedReport::Deep(deep)) =
            read_change_report(store, change, &entry.record_id)?
        {
            let builtin = if let Some(id) = deep.builtin_report_id().filter(|id| {
                index.entries.iter().any(|e| {
                    e.record_id == *id
                        && e.kind == history::CheckKind::Security
                        && e.outcome != "legacy"
                })
            }) {
                match read_change_report(store, change, id)? {
                    Some(VersionedReport::Builtin(basic)) => Some(basic),
                    _ => None,
                }
            } else {
                None
            };
            return Ok((builtin, Some(deep)));
        }
    }
    Ok((None, None))
}
pub fn read_decisions(store: &dyn Store, change: &str, id: &str) -> Result<Vec<RiskDecision>> {
    history::validate_id(id)?;
    store.ensure_checks_readable(change)?;
    let path = format!("checks/security/{id}/decisions.jsonl");
    let text = if store.change_exists(change) {
        store.read_artifact(change, &path)
    } else {
        store.read_archived_artifact(change, &path)
    }
    .unwrap_or_default();
    ensure!(
        text.len() <= 16 * 1024 * 1024,
        "storage_limit: security decisions"
    );
    let records = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut record: RiskDecision = serde_json::from_str(line)
                .map_err(|_| anyhow::anyhow!("invalid_security_decision"))?;
            record.validate()?;
            ensure!(record.report_id == id, "invalid_security_decision: report");
            record.reason = redact_text(&record.reason);
            record.actor = redact_text(&record.actor);
            Ok(record)
        })
        .collect();
    store.ensure_checks_readable(change)?;
    records
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionReceipt {
    pub decision_id: String,
    pub report_id: String,
    pub finding_id: String,
    pub status: String,
}
pub fn accept_current_risk(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    store: &dyn Store,
    input: AcceptRiskInput,
    actor: Option<String>,
    budget: &ReviewBudget,
) -> Result<DecisionReceipt> {
    input.validate()?;
    let actor = actor
        .filter(|a| !a.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("identity_required"))?;
    let change = target
        .change_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("unknown_change"))?;
    let (context, _) = submission_context(target, reader, data, store, None, budget)?;
    accept_risk_with_context(store, change, input, actor, context, budget)
}
pub(super) fn accept_risk_with_context(
    store: &dyn Store,
    change: &str,
    input: AcceptRiskInput,
    actor: String,
    context: DeepReportContext,
    budget: &ReviewBudget,
) -> Result<DecisionReceipt> {
    input.validate()?;
    let (builtin, deep) = current_reports(store, change)?;
    let deep = deep.ok_or_else(|| anyhow::anyhow!("missing_security_review"))?;
    ensure!(
        deep.project_key() == context.common.project_key && deep.change_name() == change,
        "binding_mismatch"
    );
    ensure!(
        deep.policy_digest() == context.common.policy_digest
            && deep.scope_digest() == context.scope_digest
            && deep.knowledge_digest() == context.knowledge_digest,
        "security_report_stale"
    );
    let current = input.report_id == deep.report_id()
        || builtin.as_ref().is_some_and(|b| {
            b.report_id() == input.report_id && b.project_key() == context.common.project_key
        });
    ensure!(
        current,
        "security_report_stale: only the current paired reports can accept risk"
    );
    let decision = RiskDecision {
        decision_id: ulid::Ulid::new().to_string(),
        action: "accepted".into(),
        report_id: input.report_id,
        finding_id: input.finding_id,
        reason: input.reason,
        actor,
        accepted_at: chrono::Utc::now().to_rfc3339(),
        policy_digest: context.common.policy_digest,
        scope_digest: context.scope_digest,
    };
    let plan = prepare_decision(store, change, &decision)?;
    ensure!(
        current_reports(store, change)?
            .1
            .as_ref()
            .is_some_and(|current| current.report_id() == deep.report_id()),
        "revision_conflict"
    );
    check_budget(budget)?;
    store.commit_checks(change, &plan)?;
    Ok(DecisionReceipt {
        decision_id: decision.decision_id,
        report_id: decision.report_id,
        finding_id: decision.finding_id,
        status: "accepted".into(),
    })
}
pub fn acceptance_from_fresh(
    store: &dyn Store,
    change: &str,
    policy: &SecurityPolicy,
    project_key: &str,
    fresh: Option<&FreshEvidence>,
) -> Result<Acceptance> {
    if !(policy.enabled && policy.required_for_acceptance) {
        return Ok(acceptance::evaluate(
            policy,
            project_key,
            change,
            None,
            None,
            &[],
            fresh,
        ));
    }
    let (builtin, deep) = current_reports(store, change)?;
    let mut decisions = Vec::new();
    for id in deep
        .iter()
        .map(DeepReport::report_id)
        .chain(builtin.iter().map(Report::report_id))
    {
        decisions.extend(read_decisions(store, change, id)?);
    }
    let bound_change = if store.change_exists(change) {
        change
    } else {
        deep.as_ref().map(DeepReport::change_name).unwrap_or(change)
    };
    Ok(acceptance::evaluate(
        policy,
        project_key,
        bound_change,
        builtin.as_ref(),
        deep.as_ref(),
        &decisions,
        fresh,
    ))
}
pub fn acceptance_for_target(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    store: &dyn Store,
    budget: &ReviewBudget,
) -> Result<Acceptance> {
    target.validate()?;
    ensure!(
        target.binding.checkout.as_deref() == reader.root().to_str(),
        "binding_mismatch"
    );
    let change = target
        .change_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("unknown_change"))?;
    ensure!(store.change_exists(change), "unknown_change");
    let text = reader
        .read_optional_text(
            &format!("{}/config.yaml", target.spec_dir),
            64 * 1024,
            &|| budget.stop_reason(),
        )?
        .unwrap_or_default();
    let policy = speclink_core::config::WorkflowConfig::from_text(Some(&text))
        .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?
        .security();
    let key = target.report_project_key();
    if !(policy.enabled && policy.required_for_acceptance) {
        return acceptance_from_fresh(store, change, &policy, &key, None);
    }
    match submission_context(target, reader, data, store, None, budget) {
        Ok((context, _)) => acceptance_from_fresh(
            store,
            change,
            &policy,
            &key,
            Some(&fresh_evidence(&context)),
        ),
        Err(error) => {
            let mut result = acceptance_from_fresh(store, change, &policy, &key, None)?;
            let code = error
                .to_string()
                .split(':')
                .next()
                .unwrap_or("source_freshness_unverified")
                .to_owned();
            if !result.reasons.contains(&code) {
                result.reasons.push(code);
            }
            Ok(result)
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub project_key: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub targets: Vec<String>,
    pub required_checks: Vec<String>,
    pub builtin_report_id: Option<String>,
    pub unverified_checks: Vec<String>,
}
impl From<&DeepReportContext> for Submission {
    fn from(c: &DeepReportContext) -> Self {
        Self {
            project_key: c.common.project_key.clone(),
            policy_digest: c.common.policy_digest.clone(),
            knowledge_digest: c.knowledge_digest.clone(),
            scope_digest: c.scope_digest.clone(),
            targets: c.source_lines.keys().cloned().collect(),
            required_checks: c.required_checks.iter().cloned().collect(),
            builtin_report_id: None,
            unverified_checks: c.builtin_unverified.iter().cloned().collect(),
        }
    }
}

/// Tool-owned files and other changes never become evidence for this change.
pub fn is_review_source(path: &str, spec_dir: &str, change: &str) -> bool {
    if path == ".speclink.yaml" {
        return false;
    }
    if path.starts_with(&format!("{spec_dir}/")) {
        let base = format!("{spec_dir}/changes/{change}/");
        return path.strip_prefix(&base).is_some_and(|name| {
            matches!(name, "proposal.md" | "design.md" | "tasks.md") || name.starts_with("specs/")
        });
    }
    !speclink_core::security::input::is_excluded(std::path::Path::new(path), &[])
}
pub fn source_digest(sources: &[SourceIdentity]) -> Result<String> {
    let mut sources = sources.to_vec();
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(sha256(&serde_json::to_vec(&sources)?))
}

pub fn submission_context(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    store: &dyn Store,
    actor: Option<String>,
    budget: &ReviewBudget,
) -> Result<(DeepReportContext, Option<String>)> {
    target.validate()?;
    ensure!(!target.binding.is_remote(), "source_freshness_unverified");
    ensure!(
        target.binding.checkout.as_deref() == reader.root().to_str(),
        "binding_mismatch"
    );
    let change = target
        .change_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("unknown_change"))?;
    ensure!(store.change_exists(change), "unknown_change");
    let policy_text = reader
        .read_optional_text(
            &format!("{}/config.yaml", target.spec_dir),
            64 * 1024,
            &|| budget.stop_reason(),
        )?
        .unwrap_or_default();
    let mut basic = None;
    for entry in history::index(store, change)?
        .entries
        .iter()
        .rev()
        .filter(|e| e.kind == history::CheckKind::Security && e.outcome != "legacy")
    {
        if let Some(VersionedReport::Builtin(report)) =
            read_change_report(store, change, &entry.record_id)?
        {
            basic = Some(report);
            break;
        }
    }
    build_submission_context(
        target,
        reader,
        data,
        &policy_text,
        None,
        basic.as_ref(),
        actor,
        budget,
    )
}

pub fn prepared_submission_context(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    snapshot: &PreparedSnapshot,
    basic: Option<&Report>,
    actor: Option<String>,
    budget: &ReviewBudget,
) -> Result<(DeepReportContext, Option<String>)> {
    snapshot.validate_binding(&target.binding, target.change_name.as_deref())?;
    ensure!(
        target.proposal_project_key.is_some()
            && snapshot.proposal_project_key == target.proposal_project_key,
        "unsupported_version: prepare this remote target with a supported server"
    );
    let (mut context, id) = build_submission_context(
        target,
        reader,
        data,
        &snapshot.policy_text,
        Some(&snapshot.documents),
        basic,
        actor,
        budget,
    )?;
    context.common.policy_revision = snapshot.policy_revision;
    context.common.prepared_at = Some(snapshot.prepared_at.clone());
    Ok((context, id))
}

pub(super) struct SelectedKnowledge {
    pub manifest: KnowledgeManifest,
    pub standards: Vec<ReportStandard>,
    pub licenses: Vec<SourceLicense>,
    pub required: BTreeSet<String>,
    pub permitted: BTreeSet<(StandardId, String, String, String)>,
    _pins: Vec<Box<dyn speclink_core::security::data::StandardPin>>,
}
pub(super) fn selected_knowledge(
    policy: &SecurityPolicy,
    data: &dyn StandardRepository,
    project_key: &str,
) -> Result<SelectedKnowledge> {
    let mut generations = Vec::new();
    let mut snapshots = Vec::new();
    let mut pins = Vec::new();
    let mut standards = Vec::new();
    let mut licenses = Vec::new();
    let mut required = BTreeSet::new();
    let mut permitted = BTreeSet::new();
    for (id, version, level) in policy.standards.selected() {
        let id: StandardId = id.parse()?;
        pins.push(data.pin(id, version)?);
        let installed = data
            .read(id, version)?
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
        let prefix = format!("standards/{}/{version}/", id.as_str());
        let source = speclink_core::security::standards::standard_source(id, version)?;
        let controls = installed
            .snapshot
            .items
            .iter()
            .filter(|item| {
                id != StandardId::OwaspAsvs
                    || item.levels.contains(&level.unwrap_or_default().number())
            })
            .map(|item| {
                required.insert(format!("{}/{version}/{}", id.as_str(), item.id));
                for file in source.files.iter().filter(|file| {
                    file.item_id.as_deref() == Some(item.id.as_str())
                        || file.item_id.is_none()
                            && matches!(
                                std::path::Path::new(&file.name)
                                    .extension()
                                    .and_then(|s| s.to_str()),
                                Some("xml" | "csv" | "json")
                            )
                }) {
                    let path = format!("{prefix}{}", file.name);
                    permitted.insert((
                        id,
                        version.to_owned(),
                        item.id.clone(),
                        format!(".security_kb/{path}"),
                    ));
                }
                speclink_core::security::standards::ControlAssessment {
                    control_id: item.id.clone(),
                    status: ReviewStatus::NotVerified,
                    reason: "awaiting_deep_review".into(),
                }
            })
            .collect();
        generations.push(installed.metadata.clone());
        snapshots.push(installed.snapshot.clone());
        let m = installed.metadata;
        standards.push(ReportStandard {
            standard_id: id,
            version: Some(version.into()),
            level,
            data_sha256: Some(m.sha256.clone()),
            source_revision: Some(m.source_revision.clone()),
            installed_status: "installed".into(),
            status: ReviewStatus::NotVerified,
            reason: Some("awaiting_deep_review".into()),
            controls,
        });
        licenses.push(SourceLicense {
            standard_id: id,
            version: version.into(),
            source_url: m.source_url,
            license_name: m.license_name,
            license_url: m.license_url,
            attribution: m.attribution,
            modification_notice: m.modification_notice,
        });
    }
    let files = speclink_core::security::knowledge::snapshot_hashes(&generations, &snapshots)?;
    let policy_digest = policy.review_digest()?;
    let knowledge_digest = speclink_core::security::knowledge::snapshot_digest(
        project_key,
        &policy_digest,
        &files,
        &generations,
    )?;
    Ok(SelectedKnowledge {
        manifest: KnowledgeManifest {
            format_version: 1,
            generated_by: "SpecLink".into(),
            project_key: project_key.into(),
            policy_digest,
            knowledge_digest,
            files,
            data_generations: generations,
        },
        standards,
        licenses,
        required,
        permitted,
        _pins: pins,
    })
}

fn build_submission_context(
    target: &ReviewTarget,
    reader: &dyn ProjectInput,
    data: &dyn StandardRepository,
    policy_text: &str,
    prepared_documents: Option<&[PreparedDocument]>,
    basic: Option<&Report>,
    actor: Option<String>,
    budget: &ReviewBudget,
) -> Result<(DeepReportContext, Option<String>)> {
    target.validate()?;
    ensure!(
        target.binding.checkout.as_deref() == reader.root().to_str(),
        "binding_mismatch"
    );
    let change = target
        .change_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("unknown_change"))?;
    let policy = speclink_core::config::WorkflowConfig::from_text(Some(&policy_text))
        .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?
        .security();
    ensure!(policy.enabled, "security_disabled");
    ensure!(!policy.standards.selected().is_empty(), "unconfigured");
    let project_key = target.report_project_key();
    let text = reader
        .read_optional_text(".security_kb/manifest.json", 4 * 1024 * 1024, &|| {
            budget.stop_reason()
        })?
        .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
    let manifest: KnowledgeManifest =
        serde_json::from_str(&text).map_err(|_| anyhow::anyhow!("invalid_knowledge"))?;
    ensure!(
        manifest.format_version == 1
            && manifest.generated_by == "SpecLink"
            && manifest.project_key == project_key
            && manifest.policy_digest == policy.review_digest()?,
        "missing_knowledge: prepare the current policy"
    );
    ensure!(manifest.files.len() <= 10_000, "size_limit: knowledge");
    let digest = speclink_core::security::knowledge::snapshot_digest(
        &manifest.project_key,
        &manifest.policy_digest,
        &manifest.files,
        &manifest.data_generations,
    )?;
    ensure!(
        manifest.knowledge_digest == digest,
        "integrity_error: knowledge manifest"
    );
    let selected = selected_knowledge(&policy, data, &project_key)?;
    ensure!(
        manifest.files == selected.manifest.files
            && manifest.knowledge_digest == selected.manifest.knowledge_digest
            && manifest.data_generations.len() == selected.manifest.data_generations.len(),
        "integrity_error: knowledge identity"
    );
    let standards = selected.standards.clone();
    let licenses = selected.licenses.clone();
    let mut required = selected.required.clone();
    let permitted = selected.permitted.clone();
    let mut total = 0usize;
    for (path, digest) in &manifest.files {
        ensure!(valid_relative(path), "invalid_knowledge_path");
        check_budget(budget)?;
        let bytes = reader
            .read_optional_bytes(&format!(".security_kb/{path}"), 128 * 1024 * 1024, &|| {
                budget.stop_reason()
            })?
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
        total += bytes.len();
        ensure!(
            total <= 256 * 1024 * 1024 && sha256(&bytes) == *digest,
            "integrity_error: knowledge file"
        );
    }
    let mut inputs = empty_inputs();
    let mut evidence = BTreeMap::new();
    if policy.reviews.code || policy.reviews.api_boundary {
        inputs = reader.scan_selected(
            &[],
            InputKind::Deep,
            &policy.exclude_dirs,
            &ScanLimits::default(),
            &|| budget.stop_reason(),
        )?;
        inputs.documents.retain(|d| {
            is_review_source(&d.path, &target.spec_dir, change)
                && !d.path.starts_with(&format!("{}/", target.spec_dir))
        });
        let paths: BTreeSet<_> = inputs.documents.iter().map(|d| d.path.clone()).collect();
        for (enabled, id) in [
            (policy.reviews.code, "review/code"),
            (policy.reviews.api_boundary, "review/api_boundary"),
        ] {
            if enabled {
                required.insert(id.into());
                evidence.insert(id.into(), paths.clone());
            }
        }
    }
    if policy.reviews.spec {
        let specs = if let Some(prepared) = prepared_documents {
            let docs = prepared
                .iter()
                .filter(|d| {
                    !speclink_core::security::input::is_excluded(
                        std::path::Path::new(&d.path),
                        &policy.exclude_dirs,
                    )
                })
                .map(|d| SourceDocument::new(d.path.clone(), d.text.clone()))
                .collect::<Vec<_>>();
            let bytes = docs.iter().map(|d| d.text.len()).sum::<usize>();
            let limits = remaining_limits(&inputs);
            ensure!(
                docs.len() <= limits.files
                    && bytes <= limits.total_bytes
                    && docs.iter().all(|d| d.text.len() <= limits.file_bytes),
                "source_freshness_unverified"
            );
            inputs.total_bytes += bytes as u64;
            inputs.documents.extend(docs.iter().cloned());
            docs
        } else {
            local_specs(target, reader, &policy.exclude_dirs, budget, &mut inputs)?
        };
        required.insert("review/spec".into());
        evidence.insert(
            "review/spec".into(),
            specs.iter().map(|d| d.path.clone()).collect(),
        );
    }
    ensure!(inputs.complete, "source_freshness_unverified");
    let documents: BTreeMap<_, _> = inputs
        .documents
        .into_iter()
        .map(|d| (d.path.clone(), d))
        .collect();
    let sources: Vec<_> = documents
        .values()
        .map(|d| SourceIdentity {
            path: d.path.clone(),
            content_digest: d.content_digest.clone(),
        })
        .collect();
    let scope_digest = source_digest(&sources)?;
    let mut unverified = BTreeSet::new();
    let mut builtin_id = None;
    if let Some(report) = basic {
        ensure!(report.project_key() == project_key, "binding_mismatch");
        let old_sources: Vec<_> = report
            .sources()
            .iter()
            .filter(|s| {
                is_review_source(&s.path, &target.spec_dir, change)
                    && (policy.reviews.spec
                        || !s.path.starts_with(&format!("{}/", target.spec_dir)))
            })
            .cloned()
            .collect();
        let basic_sources: Vec<_> = sources
            .iter()
            .filter(|s| Language::from_path(&s.path) != Language::Unsupported)
            .cloned()
            .collect();
        if report.policy_digest() == sha256(policy_text.as_bytes())
            && source_digest(&old_sources)? == source_digest(&basic_sources)?
        {
            for check in report
                .checks()
                .iter()
                .filter(|c| c.status == ReviewStatus::NotVerified)
            {
                unverified.insert(format!("{}::{}", check.rule_id, check.target));
            }
            for item in report.coverage().reasons.iter().filter(|c| {
                c.status == ReviewStatus::NotVerified && documents.contains_key(&c.path)
            }) {
                unverified.insert(format!("coverage::{}", item.path));
            }
            builtin_id = Some(report.report_id().into());
        }
    }
    Ok((
        DeepReportContext {
            common: ReportContext {
                report_id: ulid::Ulid::new().to_string(),
                project_key,
                project_name: target.project_name.clone(),
                change_name: Some(change.into()),
                reviewed_at: chrono::Utc::now().to_rfc3339(),
                commit_sha: None,
                policy_digest: policy.review_digest()?,
                policy_revision: None,
                prepared_at: None,
                rule_version: RuleSet::builtin()?.rule_version,
                scope: policy.scope,
                locale: "tw".into(),
                review_methods: vec![
                    "architecture".into(),
                    "business-logic".into(),
                    "cross-module-data-flow".into(),
                ],
            },
            actor,
            knowledge_digest: manifest.knowledge_digest,
            scope_digest,
            standards,
            source_licenses: licenses,
            sources,
            source_lines: documents
                .iter()
                .map(|(path, d)| (path.clone(), d.text.lines().count().max(1)))
                .collect(),
            required_checks: required,
            required_evidence: evidence,
            permitted_references: permitted,
            builtin_unverified: unverified,
            bytes_read: inputs.total_bytes,
        },
        builtin_id,
    ))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundReceipt {
    pub report_id: String,
    pub change_name: String,
    pub relative_path: String,
    pub complete: bool,
}
pub fn add_deep_round(
    store: &dyn Store,
    change: &str,
    input: DeepReportInput,
    context: DeepReportContext,
    builtin_id: Option<String>,
) -> Result<RoundReceipt> {
    ensure!(
        input.builtin_report_id == builtin_id && builtin_id.is_some(),
        "missing_security_review: run current built-in review first"
    );
    let report = VersionedReport::Deep(DeepReport::new(input, context)?);
    let receipt = RoundReceipt {
        report_id: report.report_id().into(),
        change_name: change.into(),
        relative_path: format!("checks/security/{}/report.md", report.report_id()),
        complete: report.is_complete(),
    };
    save_change_report(store, change, &report)?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::{
        data::{IntegrityKind, ValidatedStandard},
        standards::standard_source,
    };
    #[test]
    fn current_checkout_detects_added_deleted_and_changed_sources_and_knowledge() {
        let _guard = crate::security::tests::REVIEW_TEST_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        std::fs::create_dir_all(root.join("openspec/changes/demo")).unwrap();
        let policy="security_verification:\n  enabled: true\n  required_for_acceptance: false\n  standards:\n    owasp_top10: {enabled: true, version: '2025'}\n  reviews: {spec: true, code: true, api_boundary: false}\n";
        std::fs::write(root.join("openspec/config.yaml"), policy).unwrap();
        std::fs::write(
            root.join("openspec/changes/demo/tasks.md"),
            "- [ ] 1.1 pending\n",
        )
        .unwrap();
        std::fs::write(root.join("app.ts"), "const value = 1;\n").unwrap();
        let store = speclink_fs::FsStore::new(&root, "openspec");
        let data = speclink_fs::security::FsSecurityData::new(&temp.path().join("data")).unwrap();
        let mut source = standard_source(StandardId::OwaspTop10, "2025").unwrap();
        let mut files = BTreeMap::new();
        for file in &mut source.files {
            file.sha256 = None;
            files.insert(
                file.name.clone(),
                file.item_id
                    .as_ref()
                    .map(|id| format!("# {id} Synthetic fixture\n").into_bytes())
                    .unwrap_or_else(|| b"Synthetic license".to_vec()),
            );
        }
        use std::io::Read;
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
            "../../../../engine/speclink-fs/tests/fixtures/security-guidance.zip"
        )))
        .unwrap();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            let mut bytes = vec![];
            file.read_to_end(&mut bytes).unwrap();
            files.insert(file.name().into(), bytes);
        }
        data.begin_install(StandardId::OwaspTop10, "2025")
            .unwrap()
            .commit(
                ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap(),
            )
            .unwrap();
        let reader = speclink_fs::security::scan::AuthorizedProject::open(&root).unwrap();
        let target = ReviewTarget::local(reader.root(), "openspec", Some("demo".into())).unwrap();
        let key = target.report_project_key();
        speclink_fs::security::knowledge::prepare(reader.root(), &key, policy, &data, &|| false)
            .unwrap();
        let prepared =
            speclink_fs::security::prepared::FsPreparedReviews::new(&temp.path().join("prepared"))
                .unwrap();
        let budget = ReviewBudget::new(Arc::new(AtomicBool::new(false)));
        run_review(
            &target,
            Some(&reader),
            &data,
            &prepared,
            &ProposalReports {
                store: &store,
                change: "demo",
            },
            &budget,
            &|_| {},
        )
        .unwrap();
        let (context, builtin) =
            submission_context(&target, &reader, &data, &store, None, &budget).unwrap();
        assert!(builtin.is_some());
        assert!(
            !context
                .permitted_references
                .iter()
                .any(|(_, _, id, path)| id == "A01:2025"
                    && path.ends_with("A02_2025-Security_Misconfiguration.md")),
            "a control cannot cite a different control document"
        );
        let original = context.scope_digest.clone();
        let submission = Submission::from(&context);
        let input:DeepReportInput=serde_json::from_value(serde_json::json!({"formatVersion":2,"changeName":"demo","policyDigest":submission.policy_digest,"knowledgeDigest":submission.knowledge_digest,"scopeDigest":submission.scope_digest,"builtinReportId":builtin,"complete":true,"targets":submission.targets,"checks":submission.required_checks.iter().map(|id|serde_json::json!({"checkId":id,"status":"PASS","reason":"人工追蹤已確認","evidencePaths":submission.targets,"resolvesCheckIds":submission.unverified_checks})).collect::<Vec<_>>(),"findings":[{"findingId":"H1","title":"授權缺口","severity":"high","confidence":"high","confidenceReason":"已確認程式控制缺失；無可信對照","references":[],"evidence":[{"path":"app.ts","line":1,"endLine":1,"function":"handler"}],"codeSnippet":"const value = 1;","dataFlow":"input → service → store","impact":"程式已證明控制缺失；外部防護未確認","remediation":"SHALL 查本機 Authorization 指南並核對擁有者。","acceptance":"以不同帳戶要求資源應403。"}]})).unwrap();
        let saved = add_deep_round(&store, "demo", input, context, builtin).unwrap();
        assert_eq!(
            acceptance_for_target(&target, &reader, &data, &store, &budget)
                .unwrap()
                .status,
            "reference"
        );
        std::fs::write(
            root.join("openspec/config.yaml"),
            policy.replace(
                "required_for_acceptance: false",
                "required_for_acceptance: true",
            ),
        )
        .unwrap();
        assert_eq!(
            acceptance_for_target(&target, &reader, &data, &store, &budget)
                .unwrap()
                .status,
            "blocked"
        );
        let accept = AcceptRiskInput {
            report_id: saved.report_id.clone(),
            finding_id: "H1".into(),
            reason: "本次發布由人工逐筆覆核，明確接受殘餘風險".into(),
        };
        assert!(accept_current_risk(
            &target,
            &reader,
            &data,
            &store,
            accept.clone(),
            None,
            &budget
        )
        .unwrap_err()
        .to_string()
        .contains("identity_required"));
        let mut unknown = accept.clone();
        unknown.finding_id = "H2".into();
        assert!(accept_current_risk(
            &target,
            &reader,
            &data,
            &store,
            unknown,
            Some("Owner".into()),
            &budget
        )
        .is_err());
        accept_current_risk(
            &target,
            &reader,
            &data,
            &store,
            accept.clone(),
            Some("Owner".into()),
            &budget,
        )
        .unwrap();
        let verdict = acceptance_for_target(&target, &reader, &data, &store, &budget).unwrap();
        assert_eq!(verdict.status, "passed", "{:?}", verdict);
        let proof = acceptance_proof_for_target(&target, &reader, &data, &store, &budget)
            .unwrap()
            .unwrap();
        speclink_core::security::require_acceptance(&store, "demo", Some(&proof)).unwrap();
        let change = speclink_core::model::find_change(&store, "demo").unwrap();
        let ws = speclink_core::workspace::Workspace {
            root: root.clone(),
            spec_dir_name: "openspec".into(),
        };
        let result = speclink_core::archive::archive(
            &ws,
            &store,
            &change,
            &speclink_core::archive::ArchiveOptions {
                skip_specs: true,
                no_validate: true,
                mark_tasks_complete: true,
                security_acceptance: Some(proof.clone()),
                ..Default::default()
            },
            None,
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("security_report_stale"),
            "marking a reviewed source must not reuse its old security proof"
        );
        assert_eq!(
            store.read_artifact("demo", "tasks.md").as_deref(),
            Some("- [ ] 1.1 pending\n")
        );
        assert!(speclink_core::security::require_acceptance(&store, "demo", None).is_err());
        let index = store.read_artifact("demo", history::INDEX).unwrap();
        store
            .write_artifact("demo", history::INDEX, &format!("{index}\n"))
            .unwrap();
        assert!(
            speclink_core::security::require_acceptance(&store, "demo", Some(&proof))
                .unwrap_err()
                .to_string()
                .contains("security_report_stale")
        );
        store
            .write_artifact("demo", history::INDEX, &index)
            .unwrap();
        for (path, text) in [
            ("added.py", "print('hello')\n"),
            ("app.ts", "const value = 2;\n"),
        ] {
            std::fs::write(root.join(path), text).unwrap();
            assert_ne!(
                submission_context(&target, &reader, &data, &store, None, &budget)
                    .unwrap()
                    .0
                    .scope_digest,
                original
            );
            assert_eq!(
                acceptance_for_target(&target, &reader, &data, &store, &budget)
                    .unwrap()
                    .status,
                "stale"
            );
            assert!(accept_current_risk(
                &target,
                &reader,
                &data,
                &store,
                accept.clone(),
                Some("Owner".into()),
                &budget
            )
            .is_err());
        }
        std::fs::remove_file(root.join("app.ts")).unwrap();
        assert_eq!(
            acceptance_for_target(&target, &reader, &data, &store, &budget)
                .unwrap()
                .status,
            "stale"
        );
        std::fs::write(
            root.join(".security_kb/cheatsheets/Authorization_Cheat_Sheet.md"),
            "tampered",
        )
        .unwrap();
        let result = acceptance_for_target(&target, &reader, &data, &store, &budget).unwrap();
        assert_eq!(result.status, "blocked");
        assert!(result.reasons.iter().any(|r| r == "integrity_error"));
    }
}

/// Source text never enters this frame; only bounded identities and line counts do.
pub fn source_attestation(
    context: &DeepReportContext,
    data: &dyn StandardRepository,
) -> Result<speclink_protocol::security::SecuritySourceAttestation> {
    use speclink_protocol::security::*;
    let mut standards = Vec::new();
    for standard in &context.standards {
        let version = standard
            .version
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
        let installed = data
            .read(standard.standard_id, version)?
            .ok_or_else(|| anyhow::anyhow!("missing_knowledge"))?;
        standards.push(SecurityKnowledgeIdentity {
            standard_id: standard.standard_id.as_str().into(),
            version: version.into(),
            data_sha256: installed.metadata.sha256,
            source_revision: installed.metadata.source_revision,
            parser_version: installed.metadata.parser_version,
        });
    }
    Ok(SecuritySourceAttestation {
        project_key: context.common.project_key.clone(),
        change_name: context
            .common
            .change_name
            .clone()
            .ok_or_else(|| anyhow::anyhow!("unknown_change"))?,
        policy_digest: context.common.policy_digest.clone(),
        knowledge_digest: context.knowledge_digest.clone(),
        scope_digest: context.scope_digest.clone(),
        checked_at: chrono::Utc::now().to_rfc3339(),
        sources: context
            .sources
            .iter()
            .map(|s| SecurityAttestedSource {
                path: s.path.clone(),
                content_digest: s.content_digest.clone(),
                line_count: context.source_lines[&s.path],
            })
            .collect(),
        standards,
    })
}

/// The same indexed, redacted history and acceptance view for every adapter.
pub fn review_view(
    store: &dyn Store,
    change: &str,
    key: &str,
    policy: &SecurityPolicy,
    fresh: Option<&FreshEvidence>,
) -> Result<speclink_protocol::security::SecurityReviewView> {
    use speclink_protocol::security::*;
    validate_change(Some(change))?;
    ensure!(
        store.change_exists(change) || store.archived_change_exists(change),
        "unknown_change"
    );
    let mut reports = Vec::new();
    let mut decisions = Vec::new();
    for entry in history::index(store, change)?
        .entries
        .into_iter()
        .filter(|e| e.kind == history::CheckKind::Security && e.outcome != "omitted")
    {
        let report = read_change_report(store, change, &entry.record_id)?
            .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
        ensure!(
            entry.outcome == "legacy" || key.is_empty() || report.project_key() == key,
            "binding_mismatch"
        );
        for d in read_decisions(store, change, &entry.record_id)? {
            decisions.push(serde_json::from_value(serde_json::to_value(d)?)?);
        }
        reports.push(SecurityReportSummary {
            report_id: entry.record_id,
            execution_mode: report.execution_mode().into(),
            relative_path: entry.relative_path,
            complete: report.is_complete(),
            legacy: entry.outcome == "legacy",
        });
    }
    Ok(SecurityReviewView {
        change_name: change.into(),
        reports,
        acceptance: serde_json::from_value(serde_json::to_value(acceptance_from_fresh(
            store, change, policy, key, fresh,
        )?)?)?,
        submission: None,
        decisions,
        expected_revision: None,
    })
}

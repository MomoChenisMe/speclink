//! Scoped remote publication. Resource lookup stays in the installed repository.
use super::*;
use crate::{bridge::BridgeStore, context::SpeclinkExecutionContext};
use speclink_core::{
    history,
    security::{
        acceptance::{AcceptanceProof, SourceFreshness},
        knowledge::KnowledgeIdentity,
        report::*,
    },
    store::Store,
};
use speclink_protocol::security as wire;
use speclink_store::{DocumentId, Scope, StagedOp, TeamStore};
use std::collections::{BTreeMap, BTreeSet};

pub enum Operation {
    Show(Option<wire::SecuritySourceAttestation>),
    Report(String),
    Submit(wire::SecurityRoundRequest),
    Accept(wire::SecurityAcceptRequest),
}
#[derive(Debug)]
pub enum Outcome {
    View(wire::SecurityReviewView),
    Report(wire::SecurityReportDocument),
    Submitted(wire::SecurityRoundResponse),
    Accepted(wire::SecurityAcceptResponse),
}
fn convert<T: serde::de::DeserializeOwned>(v: &impl serde::Serialize) -> Result<T> {
    Ok(serde_json::from_value(serde_json::to_value(v)?)?)
}
fn index_id(change: &str) -> DocumentId {
    DocumentId::ChangeArtifact {
        change: change.into(),
        artifact: history::INDEX.into(),
    }
}
fn expected(view: &BridgeStore, change: &str, revision: u64) -> Result<()> {
    ensure!(
        view.revision(&index_id(change)) == revision,
        "revision_conflict: check index changed; refresh before retrying"
    );
    Ok(())
}

/// Historical reports require only the scoped store, never installed standard data.
pub fn report(
    store: &dyn TeamStore,
    ctx: &SpeclinkExecutionContext,
    change: &str,
    id: &str,
) -> Result<wire::SecurityReportDocument> {
    validate_change(Some(change))?;
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    let view = BridgeStore::materialize(store, &scope)?;
    ensure!(view.change_exists(change) || view.archived_change_exists(change), "unknown_change");
    report_document(&view, change, id)
}
fn report_document(store: &dyn Store, change: &str, id: &str) -> Result<wire::SecurityReportDocument> {
    let report = read_change_report(store, change, id)?.ok_or_else(|| anyhow::anyhow!("unknown_report"))?;
    Ok(serde_json::from_str(&report.json()?)?)
}

/// One materialized scope, one CAS commit. Errors drop every staged document.
pub fn execute(
    store: &dyn TeamStore,
    ctx: &SpeclinkExecutionContext,
    key: &str,
    change: &str,
    data: &dyn StandardRepository,
    operation: Operation,
) -> Result<Outcome> {
    validate_change(Some(change))?;
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    let view = BridgeStore::materialize(store, &scope)?;
    ensure!(
        view.change_exists(change) || view.archived_change_exists(change),
        "unknown_change"
    );
    let policy_text = view.read_workflow_config().unwrap_or_default();
    let policy = speclink_core::config::WorkflowConfig::from_text(Some(&policy_text))?.security();
    let actor = ctx.actor.display().map(str::to_owned);
    let output = match operation {
        Operation::Show(attestation) => {
            let context = attestation
                .as_ref()
                .map(|a| {
                    context(
                        &view,
                        key,
                        change,
                        &policy_text,
                        data,
                        a,
                        actor.clone(),
                        None,
                    )
                })
                .transpose()?;
            let fresh = context.as_ref().map(|(c, _)| {
                let mut f = fresh_evidence(c);
                f.source_freshness = SourceFreshness::ClientAttested;
                f
            });
            let mut result = review_view(&view, change, key, &policy, fresh.as_ref())?;
            result.submission = context
                .map(|(c, id)| {
                    let mut s = Submission::from(&c);
                    s.builtin_report_id = id;
                    convert(&s)
                })
                .transpose()?;
            result.expected_revision = Some(view.revision(&index_id(change)));
            Outcome::View(result)
        }
        Operation::Report(id) => {
            Outcome::Report(report_document(&view, change, &id)?)
        }
        Operation::Submit(request) => {
            ensure!(view.change_exists(change), "unknown_change");
            ensure!(request.project_key == key, "binding_mismatch");
            expected(&view, change, request.expected_revision)?;
            let actor = actor
                .clone()
                .ok_or_else(|| anyhow::anyhow!("identity_required"))?;
            match request.report {
                wire::SecurityRoundPayload::Deep(input) => {
                    let attestation = request
                        .security_source_attestation
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("source_freshness_unverified"))?;
                    let incoming = request
                        .builtin_report
                        .as_ref()
                        .map(|b| Report::from_json(&serde_json::to_vec(b)?))
                        .transpose()?;
                    let existing = if let Some(id) = input.builtin_report_id.as_deref() {
                        match read_change_report(&view, change, id)? {
                            Some(VersionedReport::Builtin(b)) => Some(b),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let basic = incoming.as_ref().or(existing.as_ref()).ok_or_else(|| {
                        anyhow::anyhow!("missing_security_review: built-in report")
                    })?;
                    ensure!(
                        input.builtin_report_id.as_deref() == Some(basic.report_id()),
                        "binding_mismatch"
                    );
                    let (context, id) = context(
                        &view,
                        key,
                        change,
                        &policy_text,
                        data,
                        attestation,
                        Some(actor),
                        Some(basic),
                    )?;
                    if existing.is_none() {
                        save_change_report(
                            &view,
                            change,
                            &VersionedReport::Builtin(basic.clone()),
                        )?;
                    } else if let Some(incoming) = &incoming {
                        ensure!(
                            serde_json::to_value(incoming)?
                                == serde_json::to_value(existing.as_ref().unwrap())?,
                            "duplicate_report_id"
                        );
                    }
                    let receipt = add_deep_round(&view, change, convert(&input)?, context, id)?;
                    Outcome::Submitted(convert(&receipt)?)
                }
                wire::SecurityRoundPayload::Legacy(input) => {
                    ensure!(
                        request.security_source_attestation.is_none(),
                        "invalid_argument: legacy attestation"
                    );
                    let b = request
                        .builtin_report
                        .ok_or_else(|| anyhow::anyhow!("unknown_report"))?;
                    let basic = Report::from_json(&serde_json::to_vec(&b)?)?;
                    ensure!(
                        basic.report_id() == input.legacy_import_report_id
                            && basic.project_key() == input.legacy_project_key
                            && basic.change_name() == Some(change),
                        "binding_mismatch"
                    );
                    let receipt = save_legacy_change_report(&view, change, &basic)?;
                    Outcome::Submitted(wire::SecurityRoundResponse {
                        report_id: receipt.report_id,
                        change_name: change.into(),
                        relative_path: format!("checks/security/{}/report.md", basic.report_id()),
                        complete: basic.is_complete(),
                    })
                }
            }
        }
        Operation::Accept(request) => {
            ensure!(view.change_exists(change), "unknown_change");
            expected(&view, change, request.expected_revision)?;
            let actor = actor.ok_or_else(|| anyhow::anyhow!("identity_required"))?;
            let (context, _) = context(
                &view,
                key,
                change,
                &policy_text,
                data,
                &request.security_source_attestation,
                Some(actor.clone()),
                None,
            )?;
            let result = super::deep::accept_risk_with_context(
                &view,
                change,
                AcceptRiskInput {
                    report_id: request.report_id,
                    finding_id: request.finding_id,
                    reason: request.reason,
                },
                actor,
                context,
                &ReviewBudget::new(Arc::new(AtomicBool::new(false))),
            )?;
            // Even unchanged index bytes participate in the write CAS and advance its revision.
            view.write_artifact(
                change,
                history::INDEX,
                &view
                    .read_artifact(change, history::INDEX)
                    .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?,
            )?;
            Outcome::Accepted(convert(&result)?)
        }
    };
    let staged = view.into_staged();
    if !staged.is_empty() {
        let mut uow = crate::commit::begin_unit_of_work(store, ctx, "security-review")?;
        for op in staged {
            match op {
                StagedOp::Put {
                    doc,
                    content,
                    expected,
                } => uow.put(doc, content, expected),
                StagedOp::Delete { doc, expected } => uow.delete(doc, expected),
            }
        }
        crate::commit::commit_with_events(
            store,
            ctx,
            uow,
            &[speclink_core::command::DomainEvent::ArtifactCreated {
                change: change.into(),
                artifact: history::INDEX.into(),
                occurred_at: chrono::Utc::now(),
            }],
        )?;
    }
    Ok(output)
}

/// Validates everything the server owns; local source identities remain client-attested.
pub(crate) fn context(
    store: &dyn Store,
    key: &str,
    change: &str,
    policy_text: &str,
    data: &dyn StandardRepository,
    a: &wire::SecuritySourceAttestation,
    actor: Option<String>,
    submitted_basic: Option<&Report>,
) -> Result<(DeepReportContext, Option<String>)> {
    ensure!(
        a.project_key == key && a.change_name == change,
        "binding_mismatch"
    );
    let checked = chrono::DateTime::parse_from_rfc3339(&a.checked_at)
        .map_err(|_| anyhow::anyhow!("source_freshness_unverified"))?;
    let age = chrono::Utc::now()
        .signed_duration_since(checked)
        .num_seconds();
    ensure!(
        (-30..=300).contains(&age),
        "source_freshness_unverified: attestation expired"
    );
    let policy = speclink_core::config::WorkflowConfig::from_text(Some(policy_text))?.security();
    ensure!(
        policy.enabled && !policy.standards.selected().is_empty(),
        "missing_knowledge: security policy unconfigured"
    );
    ensure!(
        a.policy_digest == policy.review_digest()?,
        "security_report_stale: policy"
    );
    let selected = super::deep::selected_knowledge(&policy, data, key)?;
    ensure!(
        a.knowledge_digest == selected.manifest.knowledge_digest,
        "security_report_stale: knowledge"
    );
    let actual = selected
        .manifest
        .data_generations
        .iter()
        .map(KnowledgeIdentity::from)
        .map(|x| {
            (
                x.standard_id.as_str().to_owned(),
                x.version.clone(),
                x.data_sha256,
                x.source_revision,
                x.parser_version,
            )
        })
        .collect::<BTreeSet<_>>();
    let claimed = a
        .standards
        .iter()
        .map(|x| {
            (
                x.standard_id.clone(),
                x.version.clone(),
                x.data_sha256.clone(),
                x.source_revision.clone(),
                x.parser_version.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    ensure!(
        actual == claimed && claimed.len() == a.standards.len(),
        "security_report_stale: knowledge identity"
    );
    ensure!(a.sources.len() <= 10_000, "size_limit: sources");
    let mut lines = BTreeMap::new();
    let mut sources = Vec::new();
    for source in &a.sources {
        ensure!(
            valid_relative(&source.path)
                && is_review_source(&source.path, "openspec", change)
                && history::valid_digest(&source.content_digest)
                && source.line_count > 0
                && source.line_count <= 1_048_576
                && !speclink_core::security::input::is_excluded(
                    std::path::Path::new(&source.path),
                    &policy.exclude_dirs
                ),
            "invalid_argument: attested source"
        );
        ensure!(
            lines
                .insert(source.path.clone(), source.line_count)
                .is_none(),
            "invalid_argument: duplicate source"
        );
        let spec = source.path.starts_with("openspec/");
        ensure!(
            if spec {
                policy.reviews.spec
            } else {
                policy.reviews.code || policy.reviews.api_boundary
            },
            "invalid_argument: unselected source family"
        );
        sources.push(SourceIdentity {
            path: source.path.clone(),
            content_digest: source.content_digest.clone(),
        });
    }
    let mut specs = BTreeSet::new();
    if policy.reviews.spec {
        let names = ["proposal.md".into(), "design.md".into(), "tasks.md".into()]
            .into_iter()
            .chain(
                store
                    .delta_capabilities(change)
                    .into_iter()
                    .map(|c| format!("specs/{c}/spec.md")),
            );
        for name in names {
            if let Some(text) = store.read_artifact(change, &name) {
                let path = format!("openspec/changes/{change}/{name}");
                if speclink_core::security::input::is_excluded(
                    std::path::Path::new(&path),
                    &policy.exclude_dirs,
                ) {
                    continue;
                }
                ensure!(
                    a.sources.iter().any(|s| s.path == path
                        && s.content_digest == sha256(text.as_bytes())
                        && s.line_count == text.lines().count().max(1)),
                    "security_report_stale: SDD source"
                );
                specs.insert(path);
            }
        }
    }
    ensure!(
        sources
            .iter()
            .filter(|s| s.path.starts_with("openspec/"))
            .map(|s| s.path.clone())
            .collect::<BTreeSet<_>>()
            == specs,
        "security_report_stale: SDD source set"
    );
    ensure!(
        source_digest(&sources)? == a.scope_digest,
        "security_report_stale: scope digest"
    );
    let code = sources
        .iter()
        .filter(|s| !s.path.starts_with("openspec/"))
        .map(|s| s.path.clone())
        .collect::<BTreeSet<_>>();
    let mut required = selected.required.clone();
    let mut evidence = BTreeMap::new();
    for (enabled, id, paths) in [
        (policy.reviews.spec, "review/spec", specs),
        (policy.reviews.code, "review/code", code.clone()),
        (policy.reviews.api_boundary, "review/api_boundary", code),
    ] {
        if enabled {
            required.insert(id.into());
            evidence.insert(id.into(), paths);
        }
    }
    let current_basic = current_reports(store, change)?.0;
    let basic = submitted_basic.or(current_basic.as_ref());
    let mut unverified = BTreeSet::new();
    let mut builtin_id = None;
    if let Some(b) = basic {
        ensure!(
            b.project_key() == key && b.change_name() == Some(change),
            "binding_mismatch"
        );
        let basic_sources = b
            .sources()
            .iter()
            .filter(|s| is_review_source(&s.path, "openspec", change))
            .cloned()
            .collect::<Vec<_>>();
        let selected_sources = sources
            .iter()
            .filter(|s| Language::from_path(&s.path) != Language::Unsupported)
            .cloned()
            .collect::<Vec<_>>();
        ensure!(
            source_digest(&basic_sources)? == source_digest(&selected_sources)?
                && b.rule_version() == RuleSet::builtin()?.rule_version,
            "security_report_stale: basic sources"
        );
        if submitted_basic.is_some() {
            ensure!(
                b.policy_digest() == sha256(policy_text.as_bytes()),
                "security_report_stale: basic policy"
            );
        }
        for c in b
            .checks()
            .iter()
            .filter(|c| c.status == ReviewStatus::NotVerified)
        {
            unverified.insert(format!("{}::{}", c.rule_id, c.target));
        }
        for c in b
            .coverage()
            .reasons
            .iter()
            .filter(|c| c.status == ReviewStatus::NotVerified && lines.contains_key(&c.path))
        {
            unverified.insert(format!("coverage::{}", c.path));
        }
        builtin_id = Some(b.report_id().into());
    }
    Ok((
        DeepReportContext {
            common: ReportContext {
                report_id: ulid::Ulid::new().to_string(),
                project_key: key.into(),
                project_name: "Remote".into(),
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
            knowledge_digest: a.knowledge_digest.clone(),
            scope_digest: a.scope_digest.clone(),
            standards: selected.standards.clone(),
            source_licenses: selected.licenses.clone(),
            sources,
            source_lines: lines,
            required_checks: required,
            required_evidence: evidence,
            permitted_references: selected.permitted.clone(),
            builtin_unverified: unverified,
            bytes_read: 0,
        },
        builtin_id,
    ))
}

pub(crate) fn proof(
    store: &dyn Store,
    key: &str,
    change: &str,
    policy_text: &str,
    data: &dyn StandardRepository,
    a: &wire::SecuritySourceAttestation,
) -> Result<AcceptanceProof> {
    let (context, _) = context(store, key, change, policy_text, data, a, None, None)?;
    let (builtin, deep) = current_reports(store, change)?;
    let decisions = report_decisions(store, change, builtin.as_ref(), deep.as_ref())?;
    let mut fresh = fresh_evidence(&context);
    fresh.source_freshness = SourceFreshness::ClientAttested;
    Ok(AcceptanceProof {
        change_name: change.into(),
        project_key: key.into(),
        index_digest: store
            .read_artifact(change, history::INDEX)
            .map(|s| sha256(s.as_bytes())),
        builtin,
        deep,
        decisions,
        fresh: Some(fresh),
        reasons: vec![],
    })
}

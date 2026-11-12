//! Engine outcomes as protocol types (design D4, moved here from the demo
//! server): typed field mapping, no raw JSON.

use speclink_core::analyzer::{AnalyzeReport, Msg};
use speclink_core::discuss::DiscussionInfo as EngineDiscussionInfo;
use speclink_core::instructions as engine;
use speclink_core::listing::ListChangeJson;
use speclink_core::model::ChangeMeta;
use speclink_core::plan::Plan;
use speclink_core::station::Round;
use speclink_core::status::StatusReport;
use speclink_protocol::query::{
    AnalyzeDimension, AnalyzeFinding, AnalyzeMsg, AnalyzeReportResponse, ApplyInstructions, ArtifactInstructions,
    ArtifactStatus, ChangeStatus, ChangeSummary, DependencyEntry, DiscussionInfo, PlanChangeEntry, PlanOverlap,
    PlanRequirementOverlap, PlanResponse, PlanSkipped, PlanUnpromotedCut, PlanWave, Progress, TaskEntry,
};
use speclink_protocol::command::{ReviewFindingDto, ReviewRoundDto, ReviewRulesDto};

pub(crate) fn change_summary(change: ListChangeJson, meta: Option<&ChangeMeta>) -> ChangeSummary {
    ChangeSummary {
        name: change.name,
        summary: change.summary,
        status: change.status,
        completed_tasks: change.completed_tasks,
        total_tasks: change.total_tasks,
        restale_from: change.restale_from,
        meta_error: change.meta_error,
        repo: None,
        lifecycle: None,
        claimed_by: meta.and_then(|m| m.claimed_by.clone()),
        started_at: meta.and_then(|m| m.started_at.clone()),
        created_by: meta.and_then(|m| m.created_by.clone()),
        created: meta.and_then(|m| m.created.clone()),
        from_discussions: meta.map(|m| m.from_discussions()).unwrap_or_default(),
        delta_capabilities: change.delta_capabilities,
    }
}

pub(crate) fn change_status(report: StatusReport) -> ChangeStatus {
    ChangeStatus {
        change_name: report.change_name,
        schema_name: report.schema_name,
        is_complete: report.is_complete,
        apply_requires: report.apply_requires,
        artifacts: report
            .artifacts
            .into_iter()
            .map(|a| ArtifactStatus {
                id: a.id,
                output_path: a.output_path,
                status: a.status,
                missing_deps: a.blocked_by,
                version: None,
            })
            .collect(),
        status_version: None,
        repo: None,
        lifecycle: None,
        claimed_by: None,
        created: None,
        from_discussions: Vec::new(),
        delta_capabilities: Vec::new(),
        created_by: None,
        created_with: None,
        started_at: None,
        started_by: None,
    }
}

pub(crate) fn apply_instructions(apply: engine::ApplyInstructions) -> ApplyInstructions {
    ApplyInstructions {
        change_name: apply.change_name,
        change_dir: apply.change_dir,
        schema_name: apply.schema_name,
        context_files: apply.context_files,
        progress: Progress {
            total: apply.progress.total,
            complete: apply.progress.complete,
            remaining: apply.progress.remaining,
            code_total: apply.progress.code_total,
            code_complete: apply.progress.code_complete,
            code_remaining: apply.progress.code_remaining,
        },
        tasks: apply.tasks.into_iter().map(task_entry).collect(),
        state: apply.state,
        missing_artifacts: apply.missing_artifacts,
        locale: apply.locale,
        tdd: apply.tdd,
        audit: apply.audit,
        instruction: apply.instruction,
    }
}

pub(crate) fn artifact_instructions(instr: engine::ArtifactInstructions) -> ArtifactInstructions {
    ArtifactInstructions {
        change_name: instr.change_name,
        artifact_id: instr.artifact_id,
        schema_name: instr.schema_name,
        change_dir: instr.change_dir,
        output_path: instr.output_path,
        description: instr.description,
        instruction: instr.instruction,
        context: instr.context,
        rules: instr.rules,
        locale: instr.locale,
        template: instr.template,
        dependencies: instr
            .dependencies
            .into_iter()
            .map(|d| DependencyEntry { id: d.id, done: d.done, path: d.path, description: d.description })
            .collect(),
        unlocks: instr.unlocks,
    }
}

fn task_entry(task: engine::TaskJson) -> TaskEntry {
    TaskEntry { id: task.id, description: task.description, done: task.done, manual: task.manual }
}

fn analyze_msg(m: Msg) -> AnalyzeMsg {
    AnalyzeMsg { key: m.key, params: m.params }
}

pub(crate) fn analyze_report(report: AnalyzeReport) -> AnalyzeReportResponse {
    AnalyzeReportResponse {
        change_id: report.change_id,
        dimensions: report
            .dimensions
            .into_iter()
            .map(|d| AnalyzeDimension { dimension: d.dimension, status: d.status, finding_count: d.finding_count })
            .collect(),
        findings: report
            .findings
            .into_iter()
            .map(|f| AnalyzeFinding {
                id: f.id,
                dimension: f.dimension,
                severity: f.severity,
                location: f.location,
                summary: f.summary,
                recommendation: f.recommendation,
                summary_msg: analyze_msg(f.summary_msg),
                recommendation_msg: analyze_msg(f.recommendation_msg),
            })
            .collect(),
        artifacts_analyzed: report.artifacts_analyzed,
        artifacts_missing: report.artifacts_missing,
    }
}

pub(crate) fn plan(plan: Plan) -> PlanResponse {
    PlanResponse {
        waves: plan.waves.into_iter().map(|w| PlanWave { index: w.index, changes: w.changes }).collect(),
        changes: plan
            .changes
            .into_iter()
            .map(|c| PlanChangeEntry {
                name: c.name,
                wave: c.wave,
                stage: c.stage.as_str().to_string(),
                depends_on: c.depends_on,
                overlaps: c
                    .overlaps
                    .into_iter()
                    .map(|o| PlanOverlap { change: o.change, capabilities: o.capabilities })
                    .collect(),
                blocked_by: c.blocked_by,
                ready: c.ready,
                requirement_overlap: c
                    .requirement_overlap
                    .into_iter()
                    .map(|o| PlanRequirementOverlap {
                        change: o.change,
                        capability: o.capability,
                        requirement: o.requirement,
                        own_operation: o.own_operation.as_str().to_string(),
                        other_operation: o.other_operation.as_str().to_string(),
                        conflict: o.conflict,
                    })
                    .collect(),
                archive_after: c.archive_after,
            })
            .collect(),
        next: plan.next,
        skipped: plan.skipped.into_iter().map(|s| PlanSkipped { change: s.change, reason: s.reason }).collect(),
        unpromoted_cuts: plan
            .unpromoted_cuts
            .into_iter()
            .map(|c| PlanUnpromotedCut {
                discussion: c.discussion,
                cut: c.cut,
                title: c.title,
                changes: c.changes,
                unpromoted: c.unpromoted,
            })
            .collect(),
    }
}

pub(crate) fn review_round(r: &Round) -> ReviewRoundDto {
    ReviewRoundDto {
        index: r.index as u64,
        phase: r.phase.map(|p| p.as_str().to_string()),
        patch_hash: r.patch_hash.clone(),
        rules: r.rules.as_ref().map(|rules| ReviewRulesDto {
            baseline: rules.baseline.as_str().to_string(),
            critical: rules.critical.clone(),
            warning: rules.warning.clone(),
            suggestion: rules.suggestion.clone(),
        }),
        scope: r.scope.clone(),
        findings: r
            .findings
            .iter()
            .map(|f| ReviewFindingDto { severity: f.severity.as_str().to_string(), path: f.path.clone(), text: f.text.clone() })
            .collect(),
    }
}

/// One engine discussion info → wire DTO with `promotedTo`, `concluded` and
/// `hold` filled from the engine's own projection: `DiscussionInfo` carries
/// the three as `serde(skip)` fields (parsed from that info's own record,
/// live or archived, so a reused slug never answers for its namesake), kept
/// out of the CLI's JSON so it stays byte-identical. `concluded` and `hold`
/// are always filled: an absent key means "old server", never false.
pub(crate) fn discussion(info: EngineDiscussionInfo) -> DiscussionInfo {
    DiscussionInfo {
        slug: info.slug,
        topic: info.topic,
        status: info.status,
        rounds: info.rounds,
        created: info.created,
        created_by: info.created_by,
        kind: info.kind,
        promoted_to: info.head.promoted_to,
        concluded: Some(info.concluded),
        hold: Some(info.head.hold),
        path: info.path,
        archived: info.archived,
    }
}

pub(crate) fn discussions(infos: Vec<EngineDiscussionInfo>) -> Vec<DiscussionInfo> {
    infos.into_iter().map(discussion).collect()
}

/// The artifact id → change-relative document path (the `artifact cat`
/// vocabulary), mirroring the Engine's mapping.
pub(crate) fn artifact_rel_path(artifact: &str) -> Option<String> {
    match artifact {
        "proposal" => Some("proposal.md".to_string()),
        "design" => Some("design.md".to_string()),
        "tasks" => Some("tasks.md".to_string()),
        _ => artifact
            .strip_prefix("specs/")
            .filter(|cap| !cap.is_empty() && !cap.contains('/'))
            .map(|cap| format!("specs/{cap}/spec.md")),
    }
}

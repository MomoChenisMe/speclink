//! Wire DTO → 引擎型別的共用轉換（remote-verb-parity）：CLI 與桌面把
//! validate/analyze/plan 端點回應轉回本地型別後走各自既有的渲染／序列化路徑，
//! 輸出與 fs 模式逐位元同形。

use crate::RemoteError;
use speclink_protocol::query::{
    AnalyzeMsg, AnalyzeReportResponse, ChangeEvidenceResponse, EvidenceEntry, PlanResponse,
    TraceResponse, ValidateChangeResponse,
};

/// One quality station of a wire response as the engine's status, which
/// `show --json` and the desktop's badges are built from. `None` for a state
/// this client does not know: it is read as no data, not as an error.
pub fn station_status(s: &speclink_protocol::query::StationState) -> Option<speclink_core::station::StationStatus> {
    use speclink_core::station::StationProgress;
    use speclink_protocol::query::StationPhase;
    let progress = match &s.state {
        StationPhase::None => StationProgress::None,
        StationPhase::InProgress => StationProgress::InProgress,
        StationPhase::Stamped => StationProgress::Stamped,
        StationPhase::Unknown(_) => return None,
    };
    Some(speclink_core::station::StationStatus {
        progress,
        stamped_at: s.stamped_at.clone(),
        stamped_by: s.stamped_by.clone(),
        stamped_with: s.stamped_with.clone(),
        rounds: s.rounds,
        tasks_total: s.tasks_total,
        commit: s.commit.clone(),
        scope: s
            .scope
            .iter()
            .map(|e| speclink_core::model::ReviewedScopeEntry { path: e.path.clone(), hash: e.hash.clone() })
            .collect(),
    })
}

/// `GET /changes/{name}/validate` 回應 → 引擎 `ValidationResult`。
pub fn validation_result(p: ValidateChangeResponse) -> speclink_core::validate::ValidationResult {
    speclink_core::validate::ValidationResult {
        change: p.change,
        errors: p.errors,
        valid: p.valid,
        warnings: p.warnings,
    }
}

fn analyze_msg(m: AnalyzeMsg) -> speclink_core::analyzer::Msg {
    speclink_core::analyzer::Msg { key: m.key, params: m.params }
}

/// `GET /changes/{name}/analyze` 回應 → 引擎 `AnalyzeReport`。
pub fn analyze_report(p: AnalyzeReportResponse) -> speclink_core::analyzer::AnalyzeReport {
    speclink_core::analyzer::AnalyzeReport {
        change_id: p.change_id,
        dimensions: p
            .dimensions
            .into_iter()
            .map(|d| speclink_core::analyzer::DimensionStatus {
                dimension: d.dimension,
                status: d.status,
                finding_count: d.finding_count,
            })
            .collect(),
        findings: p
            .findings
            .into_iter()
            .map(|f| speclink_core::analyzer::Finding {
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
        artifacts_analyzed: p.artifacts_analyzed,
        artifacts_missing: p.artifacts_missing,
    }
}

/// `GET /plan` 回應 → 引擎 `PlanReport`，changes 九鍵與 unpromotedCuts 五鍵逐欄搬運。`stage` 以引擎 `Stage::parse`、
/// requirement 重疊的兩個操作以 `DeltaOp::parse` 轉回（字串表只有引擎那一份）。不認得的
/// 階段或操作、或 wave 列了 changes 沒有的名稱，都是 server 違約：回錯誤，不猜，也不讓
/// 渲染端逐波查無此項而 panic。
pub fn plan_report(p: PlanResponse) -> Result<speclink_core::command::PlanReport, RemoteError> {
    use speclink_core::model::Stage;
    use speclink_core::plan::{
        DeltaOp, Overlap, Plan, PlanChange, RequirementOverlap, Skipped, UnpromotedCut, Wave,
    };
    let violation = |detail: String| RemoteError {
        message: format!("unexpected server response — {detail}"),
        reason: None,
        status: None,
        evidence: None,
    };
    let op = |s: String| {
        DeltaOp::parse(&s).ok_or_else(|| violation(format!("unknown delta operation '{s}'")))
    };
    let changes = p
        .changes
        .into_iter()
        .map(|c| {
            Ok(PlanChange {
                stage: Stage::parse(&c.stage)
                    .ok_or_else(|| violation(format!("unknown plan stage '{}'", c.stage)))?,
                name: c.name,
                wave: c.wave,
                depends_on: c.depends_on,
                overlaps: c
                    .overlaps
                    .into_iter()
                    .map(|o| Overlap { change: o.change, capabilities: o.capabilities })
                    .collect(),
                blocked_by: c.blocked_by,
                ready: c.ready,
                requirement_overlap: c
                    .requirement_overlap
                    .into_iter()
                    .map(|o| {
                        Ok(RequirementOverlap {
                            change: o.change,
                            capability: o.capability,
                            requirement: o.requirement,
                            own_operation: op(o.own_operation)?,
                            other_operation: op(o.other_operation)?,
                            conflict: o.conflict,
                        })
                    })
                    .collect::<Result<_, RemoteError>>()?,
                archive_after: c.archive_after,
            })
        })
        .collect::<Result<Vec<_>, RemoteError>>()?;
    let orphan = p.waves.iter().find_map(|w| {
        w.changes
            .iter()
            .find(|name| !changes.iter().any(|c| &c.name == *name))
            .map(|name| (w.index, name))
    });
    if let Some((index, name)) = orphan {
        return Err(violation(format!("plan wave {index} lists '{name}' without a change entry")));
    }
    Ok(Plan {
        waves: p
            .waves
            .into_iter()
            .map(|w| Wave { index: w.index, changes: w.changes })
            .collect(),
        changes,
        next: p.next,
        skipped: p
            .skipped
            .into_iter()
            .map(|s| Skipped { change: s.change, reason: s.reason })
            .collect(),
        unpromoted_cuts: p
            .unpromoted_cuts
            .into_iter()
            .map(|c| UnpromotedCut {
                discussion: c.discussion,
                cut: c.cut,
                title: c.title,
                changes: c.changes,
                unpromoted: c.unpromoted,
            })
            .collect(),
    })
}

/// `GET /trace/{capability}` 回應 → 引擎的溯源報告，逐欄搬運；`fromDiscussion` 與
/// `evidence` 的 null 照樣是 None，共用的輸出函式印出來和本機逐位元相同。
pub fn trace_report(t: TraceResponse) -> speclink_core::trace::TraceReport {
    use speclink_core::trace::{
        ChangeTrace, DiscussionTrace, EvidenceTask, PromotedChange, RequirementTrace, TraceReport,
    };
    TraceReport {
        capability: t.capability,
        requirements: t
            .requirements
            .into_iter()
            .map(|r| RequirementTrace { name: r.name, source: r.source })
            .collect(),
        changes: t
            .changes
            .into_iter()
            .map(|c| ChangeTrace {
                name: c.name,
                archived_dir: c.archived_dir,
                from_discussion: c.from_discussion,
                evidence: c.evidence.map(|tasks| {
                    tasks.into_iter().map(|e| EvidenceTask { task_id: e.task_id, files: e.files }).collect()
                }),
            })
            .collect(),
        discussions: t
            .discussions
            .into_iter()
            .map(|d| DiscussionTrace {
                slug: d.slug,
                archived: d.archived,
                promoted_to: d
                    .promoted_to
                    .into_iter()
                    .map(|p| PromotedChange { change: p.change, capabilities: p.capabilities })
                    .collect(),
            })
            .collect(),
    }
}

/// `evidence show` 對一個不帶 `change` 的回應說的話。
const EVIDENCE_SHOW_UNSUPPORTED: &str =
    "this server does not support evidence show — update the server";

/// `GET /changes/{name}/evidence` 回應 → 引擎 `evidence show` 的結果。變更名稱取自回應
/// 的 `change`（server 以引擎的規則解析後的名稱）。回應沒有 `change` 是舊版 server：它
/// 只讀進行中的變更，對已封存或不存在的名稱都回空集合——把那個答案當成「沒有記錄」
/// 交回去是錯的，所以回報錯誤。每筆記錄逐欄拆開再組回，協定多一個欄位時這裡不會靜
/// 默丟掉。
pub fn evidence_show(r: ChangeEvidenceResponse) -> Result<speclink_core::command::EvidenceShowOutcome, RemoteError> {
    let Some(change) = r.change else {
        return Err(RemoteError {
            message: EVIDENCE_SHOW_UNSUPPORTED.to_string(),
            reason: None,
            status: None,
            evidence: None,
        });
    };
    let entries = r
        .entries
        .into_iter()
        .map(|entry| {
            let EvidenceEntry { task_id, task_desc, actor, repo, head_commit, touched_files, recorded_at } = entry;
            speclink_core::tasks::EvidenceEntry {
                task_id,
                task_desc,
                actor,
                repo,
                head_commit,
                touched_files,
                recorded_at,
            }
        })
        .collect();
    Ok(speclink_core::command::EvidenceShowOutcome { change, archived_as: r.archived_as, entries })
}

#[cfg(test)]
mod tests {
    use speclink_core::model::Stage;
    use speclink_core::plan::{
        DeltaOp, Overlap, Plan, PlanChange, RequirementOverlap, Skipped, UnpromotedCut, Wave,
    };
    use speclink_protocol::query::PlanResponse;

    /// 三個階段、重疊、壞 meta、尚未轉出的刀各一的引擎 plan；add-b 帶一項 requirement
    /// 重疊與封存順序。
    fn engine_plan() -> Plan {
        Plan {
            waves: vec![
                Wave { index: 1, changes: vec!["add-a".into(), "add-c".into()] },
                Wave { index: 2, changes: vec!["add-b".into()] },
            ],
            changes: vec![
                PlanChange {
                    name: "add-a".into(),
                    wave: 1,
                    stage: Stage::InProgress,
                    depends_on: Vec::new(),
                    overlaps: vec![Overlap { change: "add-b".into(), capabilities: vec!["auth".into()] }],
                    blocked_by: Vec::new(),
                    ready: true,
                    requirement_overlap: Vec::new(),
                    archive_after: Vec::new(),
                },
                PlanChange {
                    name: "add-c".into(),
                    wave: 1,
                    stage: Stage::Ready,
                    depends_on: vec!["old-change".into()],
                    overlaps: Vec::new(),
                    blocked_by: Vec::new(),
                    ready: true,
                    requirement_overlap: Vec::new(),
                    archive_after: Vec::new(),
                },
                PlanChange {
                    name: "add-b".into(),
                    wave: 2,
                    stage: Stage::Proposed,
                    depends_on: vec!["add-a".into()],
                    overlaps: vec![Overlap { change: "add-a".into(), capabilities: vec!["auth".into()] }],
                    blocked_by: vec!["add-a".into()],
                    ready: false,
                    requirement_overlap: vec![RequirementOverlap {
                        change: "add-a".into(),
                        capability: "auth".into(),
                        requirement: "Login".into(),
                        own_operation: DeltaOp::Modified,
                        other_operation: DeltaOp::Removed,
                        conflict: false,
                    }],
                    archive_after: vec!["add-a".into()],
                },
            ],
            next: None,
            skipped: vec![Skipped { change: "broken".into(), reason: "bad yaml".into() }],
            unpromoted_cuts: vec![UnpromotedCut {
                discussion: "talk-a".into(),
                cut: "cut 3".into(),
                title: "收尾".into(),
                changes: vec!["add-d".into()],
                unpromoted: vec!["add-d".into()],
            }],
        }
    }

    #[test]
    fn plan_report_round_trips_the_engine_plan_through_the_wire() {
        // 引擎序列化 → wire DTO → 轉回引擎型別：兩端同形，CLI remote 臂印出的
        // 就是 fs 模式那份。
        let plan = engine_plan();
        let wire: PlanResponse =
            serde_json::from_value(serde_json::to_value(&plan).unwrap()).unwrap();
        assert_eq!(super::plan_report(wire).unwrap(), plan);
    }

    // --- remote-read-verbs: the trace report and the evidence of a change ---

    fn engine_trace() -> speclink_core::trace::TraceReport {
        use speclink_core::trace::{
            ChangeTrace, DiscussionTrace, EvidenceTask, PromotedChange, RequirementTrace, TraceReport,
        };
        TraceReport {
            capability: "auth".into(),
            requirements: vec![RequirementTrace { name: "Session".into(), source: "beta".into() }],
            changes: vec![
                ChangeTrace {
                    name: "alpha".into(),
                    archived_dir: "2026-11-01-alpha".into(),
                    from_discussion: Some("origin-talk".into()),
                    evidence: Some(vec![
                        EvidenceTask { task_id: "tsk_a".into(), files: vec!["src/a.rs".into(), "src/b.rs".into()] },
                        EvidenceTask { task_id: "tsk_b".into(), files: Vec::new() },
                    ]),
                },
                // No discussion, no record: both print as null.
                ChangeTrace {
                    name: "beta".into(),
                    archived_dir: "2026-11-02-beta".into(),
                    from_discussion: None,
                    evidence: None,
                },
                // A record with no task is not "no record".
                ChangeTrace {
                    name: "gamma".into(),
                    archived_dir: "2026-11-03-gamma".into(),
                    from_discussion: None,
                    evidence: Some(Vec::new()),
                },
            ],
            discussions: vec![DiscussionTrace {
                slug: "origin-talk".into(),
                archived: true,
                promoted_to: vec![
                    PromotedChange { change: "alpha".into(), capabilities: vec!["auth".into(), "billing".into()] },
                    PromotedChange { change: "other".into(), capabilities: Vec::new() },
                ],
            }],
        }
    }

    #[test]
    fn a_trace_report_comes_back_from_the_wire_with_the_same_json() {
        // 規格「溯源回應 payload」Scenario「溯源回應轉回報告後輸出相同」：報告 → 回應 →
        // 序列化 → 解析 → 報告，`trace --json` 印的文字（縮排 JSON）逐位元相同。
        let original = engine_trace();
        let sent = serde_json::to_string(&original).unwrap();
        let wire: speclink_protocol::query::TraceResponse = serde_json::from_str(&sent).unwrap();
        let back = super::trace_report(serde_json::from_str(&serde_json::to_string(&wire).unwrap()).unwrap());
        assert_eq!(
            serde_json::to_string_pretty(&back).unwrap(),
            serde_json::to_string_pretty(&original).unwrap()
        );
        assert_eq!(back.changes[1].evidence.as_ref().map(Vec::len), None);
        assert_eq!(back.changes[2].evidence.as_ref().map(Vec::len), Some(0));
    }

    fn wire_evidence(change: Option<&str>, archived_as: Option<&str>) -> speclink_protocol::query::ChangeEvidenceResponse {
        use speclink_protocol::query::{ChangeEvidenceResponse, EvidenceEntry};
        ChangeEvidenceResponse {
            change: change.map(str::to_string),
            entries: vec![EvidenceEntry {
                task_id: "tsk_a".into(),
                task_desc: "1.1 First".into(),
                actor: Some("Alice".into()),
                repo: Some("web".into()),
                head_commit: Some("abc123".into()),
                touched_files: vec!["src/a.rs".into()],
                recorded_at: "2026-11-12T03:00:00Z".into(),
            }],
            archived_as: archived_as.map(str::to_string),
        }
    }

    #[test]
    fn the_evidence_of_a_change_comes_back_field_by_field() {
        let shown = super::evidence_show(wire_evidence(Some("demo"), None)).unwrap();
        assert_eq!(shown.change, "demo");
        assert_eq!(shown.archived_as, None);
        let entry = &shown.entries[0];
        assert_eq!(
            (entry.task_id.as_str(), entry.task_desc.as_str(), entry.recorded_at.as_str()),
            ("tsk_a", "1.1 First", "2026-11-12T03:00:00Z")
        );
        assert_eq!(
            (entry.actor.as_deref(), entry.repo.as_deref(), entry.head_commit.as_deref()),
            (Some("Alice"), Some("web"), Some("abc123"))
        );
        assert_eq!(entry.touched_files, ["src/a.rs"]);
    }

    #[test]
    fn the_evidence_of_an_archived_change_keeps_the_servers_names() {
        let shown = super::evidence_show(wire_evidence(Some("demo"), Some("2026-11-12-demo"))).unwrap();
        assert_eq!(shown.change, "demo");
        assert_eq!(shown.archived_as.as_deref(), Some("2026-11-12-demo"));
    }

    #[test]
    fn evidence_without_the_change_name_is_a_server_that_cannot_answer() {
        // A server from before the Engine lookup reads active changes only and
        // answers an empty set for an archived or unknown name. An empty set
        // and one with entries are refused alike: the answer cannot be told
        // from a wrong one.
        let mut old = wire_evidence(None, None);
        let err = super::evidence_show(old.clone()).unwrap_err();
        assert_eq!(err.message, "this server does not support evidence show — update the server");
        old.entries.clear();
        assert_eq!(super::evidence_show(old).unwrap_err().message, err.message);
    }

    #[test]
    fn plan_report_carries_requirement_overlap_and_archive_after() {
        // 規格「plan 回應 payload」Scenario「轉回引擎報告逐欄搬運」。
        let wire: PlanResponse =
            serde_json::from_value(serde_json::to_value(engine_plan()).unwrap()).unwrap();
        let report = super::plan_report(wire).unwrap();
        let b = report.changes.iter().find(|c| c.name == "add-b").unwrap();
        assert_eq!(b.requirement_overlap.len(), 1);
        assert_eq!(b.requirement_overlap[0].other_operation, DeltaOp::Removed);
        assert_eq!(b.archive_after, ["add-a"]);
    }

    #[test]
    fn plan_report_reads_an_old_server_without_unpromoted_cuts() {
        // 規格「plan 回應 payload」Scenario「舊 server 缺尚未轉出的刀」。
        let wire: PlanResponse = serde_json::from_value(serde_json::json!({
            "waves": [], "changes": [], "next": null, "skipped": []
        }))
        .unwrap();
        assert!(super::plan_report(wire).unwrap().unpromoted_cuts.is_empty());
    }

    #[test]
    fn plan_report_carries_unpromoted_cuts_field_by_field() {
        // 規格「plan 回應 payload」Scenario「尚未轉出的刀逐欄搬回」。
        let wire: PlanResponse = serde_json::from_value(serde_json::json!({
            "waves": [], "changes": [], "next": null, "skipped": [],
            "unpromotedCuts": [{
                "discussion": "talk-a", "cut": "cut 2", "title": "第二刀",
                "changes": ["add-b", "add-c"], "unpromoted": ["add-c"]
            }]
        }))
        .unwrap();
        let report = super::plan_report(wire).unwrap();
        assert_eq!(
            report.unpromoted_cuts,
            [UnpromotedCut {
                discussion: "talk-a".into(),
                cut: "cut 2".into(),
                title: "第二刀".into(),
                changes: vec!["add-b".into(), "add-c".into()],
                unpromoted: vec!["add-c".into()],
            }]
        );
    }

    #[test]
    fn plan_report_refuses_a_delta_operation_it_does_not_know() {
        let mut wire: PlanResponse =
            serde_json::from_value(serde_json::to_value(engine_plan()).unwrap()).unwrap();
        wire.changes[2].requirement_overlap[0].own_operation = "MOVED".into();
        let err = super::plan_report(wire).unwrap_err();
        assert!(err.message.contains("'MOVED'"), "{}", err.message);
    }

    #[test]
    fn plan_report_refuses_a_stage_it_does_not_know() {
        let mut wire: PlanResponse =
            serde_json::from_value(serde_json::to_value(engine_plan()).unwrap()).unwrap();
        wire.changes[1].stage = "archived".into();
        let err = super::plan_report(wire).unwrap_err();
        assert!(err.message.contains("'archived'"), "{}", err.message);
    }

    #[test]
    fn plan_report_refuses_a_wave_member_without_a_change_entry() {
        // 缺 changes 鍵（serde default 為空）但 waves 有成員：server 違約，回錯誤——
        // 渲染端逐波查 changes，不能讓它在 CLI 裡 panic。
        let wire: PlanResponse = serde_json::from_value(serde_json::json!({
            "waves": [{ "index": 1, "changes": ["add-a"] }],
            "next": "add-a"
        }))
        .unwrap();
        let err = super::plan_report(wire).unwrap_err();
        assert!(err.message.contains("'add-a'"), "{}", err.message);
    }
}

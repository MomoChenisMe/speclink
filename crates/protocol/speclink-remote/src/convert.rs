//! Wire DTO → 引擎型別的共用轉換（remote-verb-parity）：CLI 與桌面把
//! validate/analyze/plan 端點回應轉回本地型別後走各自既有的渲染／序列化路徑，
//! 輸出與 fs 模式逐位元同形。

use crate::RemoteError;
use speclink_protocol::query::{
    AnalyzeMsg, AnalyzeReportResponse, PlanResponse, ValidateChangeResponse,
};

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

/// `GET /plan` 回應 → 引擎 `PlanReport`。`stage` 以引擎 `Stage::parse` 轉回（字串表只有
/// 引擎那一份）。不認得的階段、或 wave 列了 changes 沒有的名稱，都是 server 違約：回錯誤，
/// 不猜階段，也不讓渲染端逐波查無此項而 panic。
pub fn plan_report(p: PlanResponse) -> Result<speclink_core::command::PlanReport, RemoteError> {
    use speclink_core::model::Stage;
    use speclink_core::plan::{Overlap, Plan, PlanChange, Skipped, Wave};
    let violation = |detail: String| RemoteError {
        message: format!("unexpected server response — {detail}"),
        reason: None,
        status: None,
        evidence: None,
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
    })
}

#[cfg(test)]
mod tests {
    use speclink_core::model::Stage;
    use speclink_core::plan::{Overlap, Plan, PlanChange, Skipped, Wave};
    use speclink_protocol::query::PlanResponse;

    /// 三個階段、重疊、壞 meta 各一的引擎 plan。
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
                },
                PlanChange {
                    name: "add-c".into(),
                    wave: 1,
                    stage: Stage::Ready,
                    depends_on: vec!["old-change".into()],
                    overlaps: Vec::new(),
                    blocked_by: Vec::new(),
                    ready: true,
                },
                PlanChange {
                    name: "add-b".into(),
                    wave: 2,
                    stage: Stage::Proposed,
                    depends_on: vec!["add-a".into()],
                    overlaps: vec![Overlap { change: "add-a".into(), capabilities: vec!["auth".into()] }],
                    blocked_by: vec!["add-a".into()],
                    ready: false,
                },
            ],
            next: None,
            skipped: vec![Skipped { change: "broken".into(), reason: "bad yaml".into() }],
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

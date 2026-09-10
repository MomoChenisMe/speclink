//! Wire DTO → 引擎型別的共用轉換（remote-verb-parity）：CLI 與桌面把
//! validate/analyze 端點回應轉回本地型別後走各自既有的渲染／序列化路徑，
//! 輸出與 fs 模式逐位元同形。

use speclink_protocol::query::{AnalyzeMsg, AnalyzeReportResponse, ValidateChangeResponse};

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

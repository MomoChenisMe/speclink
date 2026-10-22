//! Shared check history, security review and source-attestation wire shapes.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SecurityAcceptanceStatus {
    Reference,
    Passed,
    Blocked,
    Stale,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SecuritySourceFreshness {
    Local,
    ClientAttested,
    #[default]
    Unverified,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAcceptance {
    pub required: bool,
    pub status: SecurityAcceptanceStatus,
    pub blocking_findings: usize,
    pub not_verified: usize,
    pub reasons: Vec<String>,
    #[serde(default)]
    pub source_freshness: SecuritySourceFreshness,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReportSummary {
    pub report_id: String,
    pub execution_mode: String,
    pub relative_path: String,
    pub complete: bool,
    #[serde(default)]
    pub legacy: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySubmissionContext {
    pub project_key: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub targets: Vec<String>,
    pub required_checks: Vec<String>,
    pub builtin_report_id: Option<String>,
    pub unverified_checks: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRiskDecision {
    pub decision_id: String,
    pub action: String,
    pub report_id: String,
    pub finding_id: String,
    pub reason: String,
    pub actor: String,
    pub accepted_at: String,
    pub policy_digest: String,
    pub scope_digest: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReviewView {
    pub change_name: String,
    pub reports: Vec<SecurityReportSummary>,
    pub acceptance: SecurityAcceptance,
    #[serde(default)]
    pub submission: Option<SecuritySubmissionContext>,
    #[serde(default)]
    pub decisions: Vec<SecurityRiskDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityAttestedSource {
    pub path: String,
    pub content_digest: String,
    pub line_count: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityKnowledgeIdentity {
    pub standard_id: String,
    pub version: String,
    pub data_sha256: String,
    pub source_revision: String,
    pub parser_version: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecuritySourceAttestation {
    pub project_key: String,
    pub change_name: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub checked_at: String,
    pub sources: Vec<SecurityAttestedSource>,
    pub standards: Vec<SecurityKnowledgeIdentity>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityCheckStatus {
    Pass,
    Finding,
    NotVerified,
    NotApplicable,
    Disabled,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityDeepCheck {
    pub check_id: String,
    pub status: SecurityCheckStatus,
    pub reason: String,
    pub evidence_paths: Vec<String>,
    pub resolves_check_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityDeepReference {
    pub standard_id: String,
    pub version: String,
    pub control_id: String,
    pub local_path: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityEvidence {
    pub path: String,
    pub line: usize,
    pub end_line: Option<usize>,
    pub function: Option<String>,
}
fn default_domain() -> String {
    "安全與業務邏輯".into()
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityDeepFinding {
    pub finding_id: String,
    #[serde(default = "default_domain")]
    pub domain: String,
    pub title: String,
    pub severity: String,
    pub confidence: String,
    pub confidence_reason: String,
    pub references: Vec<SecurityDeepReference>,
    pub evidence: Vec<SecurityEvidence>,
    pub code_snippet: String,
    #[serde(default)]
    pub redaction_notice: Option<String>,
    pub data_flow: String,
    pub impact: String,
    pub remediation: String,
    pub acceptance: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityDeepInput {
    pub format_version: u8,
    pub change_name: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub builtin_report_id: Option<String>,
    pub complete: bool,
    pub targets: Vec<String>,
    pub checks: Vec<SecurityDeepCheck>,
    pub findings: Vec<SecurityDeepFinding>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityCounts {
    pub checks_succeeded: usize,
    pub checks_failed: usize,
    pub findings: usize,
    pub not_verified: usize,
    pub not_applicable: usize,
    pub disabled: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityCoverageEntry {
    pub path: String,
    pub status: SecurityCheckStatus,
    pub reason: Option<String>,
    pub bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityCoverage {
    pub inspected_files: usize,
    pub skipped_files: usize,
    pub failed_items: usize,
    pub bytes_read: u64,
    pub remaining_targets: Vec<String>,
    pub reasons: Vec<SecurityCoverageEntry>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySourceIdentity {
    pub path: String,
    pub content_digest: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityControl {
    pub control_id: String,
    pub status: SecurityCheckStatus,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReportStandard {
    pub standard_id: String,
    pub version: Option<String>,
    pub level: Option<String>,
    pub data_sha256: Option<String>,
    pub source_revision: Option<String>,
    pub installed_status: String,
    pub status: SecurityCheckStatus,
    pub reason: Option<String>,
    pub controls: Vec<SecurityControl>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySourceLicense {
    pub standard_id: String,
    pub version: String,
    pub source_url: String,
    pub license_name: String,
    pub license_url: String,
    pub attribution: String,
    pub modification_notice: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReportHeader {
    pub format_version: u8,
    pub report_id: String,
    pub project_key: String,
    pub project_name: String,
    pub change_name: Option<String>,
    pub reviewed_at: String,
    pub commit_sha: Option<String>,
    pub policy_digest: String,
    pub policy_revision: Option<u64>,
    pub prepared_at: Option<String>,
    pub standards: Vec<SecurityReportStandard>,
    pub rule_version: String,
    pub scope: String,
    pub execution_mode: String,
    pub review_methods: Vec<String>,
    pub complete: bool,
    pub configuration_state: String,
    pub counts: SecurityCounts,
    pub coverage: SecurityCoverage,
    pub disclaimer: String,
    pub source_licenses: Vec<SecuritySourceLicense>,
    pub sources: Vec<SecuritySourceIdentity>,
    pub locale: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityBasicReference {
    pub standard_id: String,
    pub version: String,
    pub control_id: String,
    pub source_url: String,
    pub source_revision: String,
    pub section: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityBasicFinding {
    pub finding_id: String,
    pub cwe_ids: Vec<String>,
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub confidence: String,
    pub kind: String,
    pub confirmation_status: String,
    pub path: String,
    pub line: usize,
    pub evidence: String,
    pub description: String,
    pub remediation: String,
    pub references: Vec<SecurityBasicReference>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityBasicCheck {
    pub rule_id: String,
    pub target: String,
    pub status: SecurityCheckStatus,
    pub reason: Option<String>,
    pub related_standard_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityBuiltinDocument {
    #[serde(flatten)]
    pub header: SecurityReportHeader,
    pub findings: Vec<SecurityBasicFinding>,
    pub checks: Vec<SecurityBasicCheck>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityDeepDocument {
    #[serde(flatten)]
    pub header: SecurityReportHeader,
    pub actor: Option<String>,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub builtin_report_id: Option<String>,
    pub targets: Vec<String>,
    pub required_checks: Vec<String>,
    pub findings: Vec<SecurityDeepFinding>,
    pub checks: Vec<SecurityDeepCheck>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SecurityReportDocument {
    Deep(SecurityDeepDocument),
    Builtin(SecurityBuiltinDocument),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityLegacyInput {
    pub legacy_import_report_id: String,
    pub legacy_project_key: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum SecurityRoundPayload {
    Deep(SecurityDeepInput),
    Legacy(SecurityLegacyInput),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityRoundRequest {
    pub project_key: String,
    pub expected_revision: u64,
    pub report: SecurityRoundPayload,
    #[serde(default)]
    pub builtin_report: Option<SecurityBuiltinDocument>,
    #[serde(default)]
    pub security_source_attestation: Option<SecuritySourceAttestation>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRoundResponse {
    pub report_id: String,
    pub change_name: String,
    pub relative_path: String,
    pub complete: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityAcceptRequest {
    pub expected_revision: u64,
    pub report_id: String,
    pub finding_id: String,
    pub reason: String,
    pub security_source_attestation: SecuritySourceAttestation,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityAcceptResponse {
    pub decision_id: String,
    pub report_id: String,
    pub finding_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityCheckRequest {
    pub security_source_attestation: SecuritySourceAttestation,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityGateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_source_attestation: Option<SecuritySourceAttestation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn security_view_and_attestation_have_one_camel_case_contract() {
        let view:SecurityReviewView=serde_json::from_value(serde_json::json!({"changeName":"demo","reports":[{"reportId":"01ARZ3NDEKTSV4RRFFQ69G5FAV","executionMode":"deep-skill","relativePath":"checks/security/01ARZ3NDEKTSV4RRFFQ69G5FAV/report.md","complete":true}],"acceptance":{"required":false,"status":"reference","blockingFindings":0,"notVerified":0,"reasons":[]}})).unwrap();
        assert_eq!(view.acceptance.status, SecurityAcceptanceStatus::Reference);
        assert!(!view.reports[0].legacy);
        assert!(view.submission.is_none());
        assert!(view.decisions.is_empty());
        let value = serde_json::to_value(&view).unwrap();
        assert!(value["acceptance"]["required"].is_boolean());
        assert!(value["acceptance"]["blockingFindings"].is_u64());
        let attestation:SecuritySourceAttestation=serde_json::from_value(serde_json::json!({"projectKey":"a".repeat(64),"changeName":"demo","policyDigest":"b".repeat(64),"knowledgeDigest":"c".repeat(64),"scopeDigest":"d".repeat(64),"checkedAt":"2026-10-20T00:00:00Z","sources":[{"path":"src/app.ts","contentDigest":"e".repeat(64),"lineCount":10}],"standards":[]})).unwrap();
        assert_eq!(attestation.sources[0].line_count, 10);
        let value = serde_json::to_value(&attestation).unwrap();
        assert!(value["scopeDigest"].is_string());
        assert!(value["sources"][0]["lineCount"].is_u64());
        assert!(value.get("sourceCode").is_none());
        let schema = schemars::schema_for!(SecuritySourceAttestation);
        assert!(serde_json::to_string(&schema)
            .unwrap()
            .contains("contentDigest"));
    }
    #[test]
    fn new_security_capabilities_default_to_unsupported_on_older_servers() {
        let caps: crate::binding::Capabilities = serde_json::from_str("{}").unwrap();
        assert!(!caps.security_review_v2);
        assert!(!caps.check_history);
    }
}

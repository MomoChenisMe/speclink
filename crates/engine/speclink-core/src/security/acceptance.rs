//! Pure acceptance policy. Filesystem/source freshness is supplied by Host.
use super::{
    policy::SecurityPolicy,
    report::{DeepReport, Report},
    ReviewStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceFreshness {
    Local,
    ClientAttested,
    Unverified,
}
#[derive(Debug, Clone)]
pub struct FreshEvidence {
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub required_checks: BTreeSet<String>,
    pub source_freshness: SourceFreshness,
}

/// Created by Host for one operation; intentionally cannot be deserialized from client input.
#[derive(Debug, Clone)]
pub struct AcceptanceProof {
    pub change_name: String,
    pub project_key: String,
    pub index_digest: Option<String>,
    pub builtin: Option<Report>,
    pub deep: Option<DeepReport>,
    pub decisions: Vec<RiskDecision>,
    pub fresh: Option<FreshEvidence>,
    pub reasons: Vec<String>,
}
impl AcceptanceProof {
    pub fn outcome(&self, policy: &SecurityPolicy) -> Acceptance {
        let mut outcome = evaluate(
            policy,
            &self.project_key,
            &self.change_name,
            self.builtin.as_ref(),
            self.deep.as_ref(),
            &self.decisions,
            self.fresh.as_ref(),
        );
        if outcome.required {
            for reason in &self.reasons {
                outcome.reason(reason);
            }
        }
        outcome
    }
}
#[derive(Debug)]
pub struct GateFailure {
    pub code: &'static str,
    pub reasons: Vec<String>,
}
impl std::fmt::Display for GateFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}; run speclink-security-review for the current change",
            self.code,
            self.reasons.join(", ")
        )
    }
}
impl std::error::Error for GateFailure {}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RiskDecision {
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
impl RiskDecision {
    pub fn validate(&self) -> anyhow::Result<()> {
        super::report::AcceptRiskInput {
            report_id: self.report_id.clone(),
            finding_id: self.finding_id.clone(),
            reason: self.reason.clone(),
        }
        .validate()?;
        anyhow::ensure!(
            !self.actor.trim().is_empty() && !self.actor.chars().any(char::is_control),
            "identity_required"
        );
        crate::history::validate_id(&self.decision_id)?;
        anyhow::ensure!(
            self.action == "accepted"
                && chrono::DateTime::parse_from_rfc3339(&self.accepted_at).is_ok()
                && crate::history::valid_digest(&self.policy_digest)
                && crate::history::valid_digest(&self.scope_digest),
            "invalid_security_decision"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Acceptance {
    pub required: bool,
    pub status: String,
    pub blocking_findings: usize,
    pub not_verified: usize,
    pub reasons: Vec<String>,
    pub source_freshness: SourceFreshness,
}
impl Acceptance {
    fn reason(&mut self, reason: &str) {
        if !self.reasons.iter().any(|r| r == reason) {
            self.reasons.push(reason.into());
        }
    }
    pub fn passes(&self) -> bool {
        !self.required || self.status == "passed"
    }
}

pub fn evaluate(
    policy: &SecurityPolicy,
    project_key: &str,
    change: &str,
    builtin: Option<&Report>,
    deep: Option<&DeepReport>,
    decisions: &[RiskDecision],
    fresh: Option<&FreshEvidence>,
) -> Acceptance {
    let required = policy.enabled && policy.required_for_acceptance;
    let mut result = Acceptance {
        required,
        status: if required { "blocked" } else { "reference" }.into(),
        blocking_findings: 0,
        not_verified: 0,
        reasons: vec![],
        source_freshness: fresh
            .map(|f| f.source_freshness)
            .unwrap_or(SourceFreshness::Unverified),
    };
    if !required {
        return result;
    }
    if policy.standards.selected().is_empty() {
        result.reason("unconfigured");
    }
    if !(policy.reviews.spec || policy.reviews.code || policy.reviews.api_boundary) {
        result.reason("reviews_disabled");
    }
    let Some(deep) = deep else {
        result.reason("missing_security_review");
        return result;
    };
    let Some(builtin) = builtin else {
        result.reason("missing_builtin_review");
        return result;
    };
    if deep.project_key() != project_key
        || deep.change_name() != change
        || builtin.project_key() != project_key
        || builtin.change_name() != Some(change)
        || deep.builtin_report_id() != Some(builtin.report_id())
    {
        result.reason("binding_mismatch");
        return result;
    }
    let Some(fresh) = fresh.filter(|f| f.source_freshness != SourceFreshness::Unverified) else {
        result.reason("source_freshness_unverified");
        return result;
    };
    if deep.policy_digest() != fresh.policy_digest
        || policy.review_digest().ok().as_deref() != Some(fresh.policy_digest.as_str())
        || deep.knowledge_digest() != fresh.knowledge_digest
        || deep.scope_digest() != fresh.scope_digest
    {
        result.status = "stale".into();
        result.reason("security_report_stale");
        return result;
    }
    if !deep.is_complete() {
        result.reason("incomplete_security_review");
    }
    if deep.sources().is_empty() {
        result.reason("source_unavailable");
    }
    for id in &fresh.required_checks {
        if !deep.checks().iter().any(|check| {
            check.check_id == *id
                && matches!(
                    check.status,
                    ReviewStatus::Pass | ReviewStatus::Finding | ReviewStatus::NotApplicable
                )
                && (check.status != ReviewStatus::NotApplicable
                    || (!check.reason.trim().is_empty() && !check.evidence_paths.is_empty()))
        }) {
            result.not_verified += 1;
        }
    }
    let resolved = |id: &str, target: &str| {
        deep.checks().iter().any(|check| {
            check.resolves_check_ids.iter().any(|r| r == id)
                && matches!(
                    check.status,
                    ReviewStatus::Pass | ReviewStatus::Finding | ReviewStatus::NotApplicable
                )
                && !check.reason.trim().is_empty()
                && !check.evidence_paths.is_empty()
                && (target.is_empty() || check.evidence_paths.iter().any(|p| p == target))
        })
    };
    let mut necessary = 0;
    for check in builtin
        .checks()
        .iter()
        .filter(|c| c.status == ReviewStatus::NotVerified)
    {
        if check.target.is_empty() || deep.sources().iter().any(|s| s.path == check.target) {
            necessary += 1;
            if !resolved(
                &format!("{}::{}", check.rule_id, check.target),
                &check.target,
            ) {
                result.not_verified += 1;
            }
        }
    }
    let mut benign_unselected = 0;
    for item in builtin
        .coverage()
        .reasons
        .iter()
        .filter(|c| c.status == ReviewStatus::NotVerified)
    {
        if deep.sources().iter().any(|s| s.path == item.path)
            && matches!(
                item.reason.as_deref(),
                Some("unsupported_language" | "rules_recorded")
            )
        {
            necessary += 1;
            if !resolved(&format!("coverage::{}", item.path), &item.path) {
                result.not_verified += 1;
            }
        } else if !deep.sources().iter().any(|s| s.path == item.path)
            && matches!(
                item.reason.as_deref(),
                Some("binary" | "unsupported_language" | "review_not_selected")
            )
        {
            benign_unselected += 1;
        } else {
            result.reason("incomplete_builtin_review");
        }
    }
    if !builtin.coverage().remaining_targets.is_empty()
        || builtin.is_cancelled()
        || (!builtin.is_complete() && necessary == 0 && benign_unselected == 0)
    {
        result.reason("incomplete_builtin_review");
    }
    if result.not_verified > 0 {
        result.reason("necessary_checks_not_verified");
    }
    let accepted = |report_id: &str, finding_id: &str| {
        decisions.iter().any(|d| {
            d.validate().is_ok()
                && d.report_id == report_id
                && d.finding_id == finding_id
                && d.policy_digest == fresh.policy_digest
                && d.scope_digest == fresh.scope_digest
        })
    };
    for finding in deep.findings() {
        if finding.severity != "low" && !accepted(deep.report_id(), &finding.finding_id) {
            result.blocking_findings += 1;
        }
    }
    for finding in builtin.findings() {
        if finding.finding.severity != "low" && !accepted(builtin.report_id(), &finding.finding_id)
        {
            result.blocking_findings += 1;
        }
    }
    if result.blocking_findings > 0 {
        result.reason("unresolved_security_findings");
    }
    if result.reasons.is_empty() {
        result.status = "passed".into();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{input::*, policy::*, report::*, rules::*};
    use std::collections::{BTreeMap, BTreeSet};
    fn policy() -> SecurityPolicy {
        let mut p = SecurityPolicy::default();
        p.enabled = true;
        p.required_for_acceptance = true;
        p.standards.owasp_top10 = StandardSelection {
            enabled: true,
            version: Some("2025".into()),
        };
        p.reviews = SecurityReviews {
            spec: false,
            code: true,
            api_boundary: false,
        };
        p
    }
    fn context(id: &str, p: &SecurityPolicy) -> ReportContext {
        ReportContext {
            report_id: id.into(),
            project_key: "a".repeat(64),
            project_name: "Fixture".into(),
            change_name: Some("demo".into()),
            reviewed_at: "2026-10-20T00:00:00Z".into(),
            commit_sha: None,
            policy_digest: p.review_digest().unwrap(),
            policy_revision: None,
            prepared_at: None,
            rule_version: "1".into(),
            scope: SecurityScope::WebApi,
            locale: "tw".into(),
            review_methods: vec!["code".into()],
        }
    }
    fn evidence(
        severity: Option<&str>,
        complete: bool,
        basic_status: ReviewStatus,
    ) -> (SecurityPolicy, Report, DeepReport, FreshEvidence) {
        let p = policy();
        let basic_id = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
        let deep_id = "01ARZ3NDEKTSV4RRFFQ69G5FAW";
        let source = SourceDocument::new("app.ts".into(), "route(id)".into());
        let basic = Report::new(
            context(basic_id, &p),
            "configured",
            true,
            vec![RuleResult {
                rule_id: "SL-SEC-001".into(),
                path: "app.ts".into(),
                status: basic_status,
                reason: Some("needs evidence".into()),
                findings: vec![],
            }],
            vec![],
            &ScanInputs {
                visited_files: 1,
                documents: vec![source.clone()],
                coverage: vec![],
                remaining_targets: vec![],
                total_bytes: 9,
                complete: true,
                stop_reason: None,
            },
        )
        .unwrap();
        let mut input:DeepReportInput=serde_json::from_value(serde_json::json!({"formatVersion":2,"changeName":"demo","policyDigest":p.review_digest().unwrap(),"knowledgeDigest":"b".repeat(64),"scopeDigest":"c".repeat(64),"builtinReportId":basic_id,"complete":complete,"targets":["app.ts"],"checks":[{"checkId":"review/code","status":"PASS","reason":"已逐入口追查","evidencePaths":["app.ts"],"resolvesCheckIds":if basic_status==ReviewStatus::NotVerified {vec!["SL-SEC-001::app.ts"]}else{vec![]}}],"findings":[]})).unwrap();
        if let Some(severity) = severity {
            input.findings.push(serde_json::from_value(serde_json::json!({"findingId":"H1","title":"跨帳戶修改缺少檢查","severity":severity,"confidence":"high","confidenceReason":"已確認程式缺少授權；外部前提未確認；無可信對照","references":[],"evidence":[{"path":"app.ts","line":1,"endLine":1,"function":"route"}],"codeSnippet":"route(id)","dataFlow":"id → service → storage","impact":"程式已證明跨帳戶路徑；閘道防護未確認","remediation":"SHALL 依本機 Authorization 指南檢查擁有者。","acceptance":"帳戶A修改B應403且資料不變。"})).unwrap());
        }
        let required = BTreeSet::from(["review/code".into()]);
        let deep = DeepReport::new(
            input,
            DeepReportContext {
                common: context(deep_id, &p),
                actor: Some("Reviewer".into()),
                knowledge_digest: "b".repeat(64),
                scope_digest: "c".repeat(64),
                standards: vec![],
                source_licenses: vec![],
                sources: vec![SourceIdentity {
                    path: source.path.clone(),
                    content_digest: source.content_digest,
                }],
                source_lines: BTreeMap::from([("app.ts".into(), 1)]),
                required_checks: required.clone(),
                required_evidence: BTreeMap::from([(
                    "review/code".into(),
                    BTreeSet::from(["app.ts".into()]),
                )]),
                permitted_references: BTreeSet::new(),
                builtin_unverified: BTreeSet::from(["SL-SEC-001::app.ts".into()]),
                bytes_read: 9,
            },
        )
        .unwrap();
        let fresh = FreshEvidence {
            policy_digest: p.review_digest().unwrap(),
            knowledge_digest: "b".repeat(64),
            scope_digest: "c".repeat(64),
            required_checks: required,
            source_freshness: SourceFreshness::Local,
        };
        (p, basic, deep, fresh)
    }
    fn decision(deep: &DeepReport) -> RiskDecision {
        RiskDecision {
            decision_id: "01ARZ3NDEKTSV4RRFFQ69G5FAX".into(),
            action: "accepted".into(),
            report_id: deep.report_id().into(),
            finding_id: "H1".into(),
            reason: "已有人工交易覆核，接受本次殘餘風險".into(),
            actor: "Owner".into(),
            accepted_at: "2026-10-20T01:00:00Z".into(),
            policy_digest: deep.policy_digest().into(),
            scope_digest: deep.scope_digest().into(),
        }
    }
    #[test]
    fn acceptance_matrix_from_spec() {
        for (required, complete, severity, accepted, want) in [
            (false, false, "high", false, "reference"),
            (true, true, "low", false, "passed"),
            (true, true, "medium", false, "blocked"),
            (true, true, "high", false, "blocked"),
            (true, true, "critical", true, "passed"),
            (true, false, "low", false, "blocked"),
        ] {
            let (mut p, b, d, f) = evidence(Some(severity), complete, ReviewStatus::Pass);
            p.required_for_acceptance = required;
            let decisions = if accepted { vec![decision(&d)] } else { vec![] };
            let result = evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&b),
                Some(&d),
                &decisions,
                Some(&f),
            );
            assert_eq!(
                result.status, want,
                "{required} {complete} {severity} {accepted}"
            );
        }
    }
    #[test]
    fn required_incomplete_zero_findings_and_missing_data_still_blocks() {
        let (p, b, d, mut f) = evidence(None, false, ReviewStatus::NotVerified);
        f.required_checks.insert("missing-standard-data".into());
        let result = evaluate(&p, &"a".repeat(64), "demo", Some(&b), Some(&d), &[], Some(&f));
        assert!(result.required);
        assert_eq!(result.status, "blocked");
        assert_eq!(result.blocking_findings, 0);
        assert!(result.not_verified > 0);
        assert!(result.reasons.iter().any(|r| r == "incomplete_security_review"));
        assert!(result.reasons.iter().any(|r| r == "necessary_checks_not_verified"), "{:?}", result.reasons);
    }
    #[test]
    fn acceptance_digests_missing_evidence_and_explicit_resolutions() {
        let (p, b, d, mut f) = evidence(None, true, ReviewStatus::NotVerified);
        assert_eq!(
            evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&b),
                Some(&d),
                &[],
                Some(&f)
            )
            .status,
            "passed"
        );
        for kind in ["policy", "scope", "knowledge"] {
            let original = f.clone();
            match kind {
                "policy" => f.policy_digest = "0".repeat(64),
                "scope" => f.scope_digest = "0".repeat(64),
                _ => f.knowledge_digest = "0".repeat(64),
            }
            assert_eq!(
                evaluate(
                    &p,
                    &"a".repeat(64),
                    "demo",
                    Some(&b),
                    Some(&d),
                    &[],
                    Some(&f)
                )
                .status,
                "stale"
            );
            f = original;
        }
        assert_eq!(
            evaluate(&p, &"a".repeat(64), "demo", Some(&b), None, &[], Some(&f)).status,
            "blocked"
        );
        assert_eq!(
            evaluate(&p, &"a".repeat(64), "demo", Some(&b), Some(&d), &[], None).status,
            "blocked"
        );
        f.required_checks
            .insert("owasp_asvs/5.0.0/new-level-control".into());
        assert_eq!(
            evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&b),
                Some(&d),
                &[],
                Some(&f)
            )
            .status,
            "blocked"
        );
        let (p, b, d, f) = evidence(Some("high"), true, ReviewStatus::Pass);
        let mut accepted = decision(&d);
        accepted.scope_digest = "0".repeat(64);
        assert_eq!(
            evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&b),
                Some(&d),
                &[accepted],
                Some(&f)
            )
            .status,
            "blocked"
        );
    }
    #[test]
    fn disabled_policy_partial_basic_cancellation_and_unproven_resolutions() {
        let (mut p, b, d, f) = evidence(None, true, ReviewStatus::NotVerified);
        let mut value = serde_json::to_value(&d).unwrap();
        value["checks"][0]["resolvesCheckIds"] = serde_json::json!([]);
        let unresolved = DeepReport::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&b),
                Some(&unresolved),
                &[],
                Some(&f)
            )
            .not_verified,
            1
        );
        let mut value = serde_json::to_value(&b).unwrap();
        value["coverage"]["reasons"] = serde_json::json!([{"path":"app.ts","status":"NOT_VERIFIED","reason":"cancelled","bytes":0}]);
        let cancelled = Report::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(
            evaluate(
                &p,
                &"a".repeat(64),
                "demo",
                Some(&cancelled),
                Some(&d),
                &[],
                Some(&f)
            )
            .status,
            "blocked"
        );
        p.enabled = false;
        assert_eq!(
            evaluate(&p, &"a".repeat(64), "demo", None, None, &[], None).status,
            "reference"
        );
        p.enabled = true;
        p.standards = SecurityStandards::default();
        assert!(!evaluate(
            &p,
            &"a".repeat(64),
            "demo",
            Some(&b),
            Some(&d),
            &[],
            Some(&f)
        )
        .passes());
        let (p, b, d, mut f) = evidence(None, true, ReviewStatus::Disabled);
        f.source_freshness = SourceFreshness::ClientAttested;
        let result = evaluate(
            &p,
            &"a".repeat(64),
            "demo",
            Some(&b),
            Some(&d),
            &[],
            Some(&f),
        );
        assert_eq!(result.status, "passed");
        assert_eq!(
            serde_json::to_value(&result).unwrap()["sourceFreshness"],
            "client-attested"
        );
    }
}

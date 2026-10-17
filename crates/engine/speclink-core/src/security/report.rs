//! A single redacted model for JSON, Markdown and Desktop display.
use super::{
    input::{CoverageEntry, ScanInputs},
    policy::{AsvsLevel, SecurityScope},
    rules::{Finding, RuleResult},
    standards::{ControlAssessment, StandardAssessment, StandardId},
    ReviewStatus,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub struct ReportContext {
    pub report_id: String,
    pub project_key: String,
    pub project_name: String,
    pub change_name: Option<String>,
    pub reviewed_at: String,
    pub commit_sha: Option<String>,
    pub policy_digest: String,
    pub policy_revision: Option<u64>,
    pub prepared_at: Option<String>,
    pub rule_version: String,
    pub scope: SecurityScope,
    pub locale: String,
    pub review_methods: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub checks_succeeded: usize,
    pub checks_failed: usize,
    pub findings: usize,
    pub not_verified: usize,
    pub not_applicable: usize,
    pub disabled: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub rule_id: String,
    pub target: String,
    pub status: ReviewStatus,
    pub reason: Option<String>,
    pub related_standard_ids: Vec<StandardId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFinding {
    pub finding_id: String,
    #[serde(flatten)]
    pub finding: Finding,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceIdentity {
    pub path: String,
    pub content_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coverage {
    pub inspected_files: usize,
    pub skipped_files: usize,
    pub failed_items: usize,
    pub bytes_read: u64,
    pub remaining_targets: Vec<String>,
    pub reasons: Vec<CoverageEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportStandard {
    pub standard_id: StandardId,
    pub version: Option<String>,
    pub level: Option<AsvsLevel>,
    pub data_sha256: Option<String>,
    pub source_revision: Option<String>,
    pub installed_status: String,
    pub status: ReviewStatus,
    pub reason: Option<String>,
    pub controls: Vec<ControlAssessment>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLicense {
    pub standard_id: StandardId,
    pub version: String,
    pub source_url: String,
    pub license_name: String,
    pub license_url: String,
    pub attribution: String,
    pub modification_notice: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReportWire {
    format_version: u8,
    report_id: String,
    project_key: String,
    project_name: String,
    change_name: Option<String>,
    reviewed_at: String,
    commit_sha: Option<String>,
    policy_digest: String,
    policy_revision: Option<u64>,
    prepared_at: Option<String>,
    standards: Vec<ReportStandard>,
    rule_version: String,
    scope: SecurityScope,
    execution_mode: String,
    review_methods: Vec<String>,
    complete: bool,
    configuration_state: String,
    counts: Counts,
    findings: Vec<ReportFinding>,
    checks: Vec<Check>,
    coverage: Coverage,
    disclaimer: String,
    source_licenses: Vec<SourceLicense>,
    sources: Vec<SourceIdentity>,
    locale: String,
}
/// Fields are private so source evidence cannot be added after redaction.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Report(ReportWire);
impl Report {
    pub fn new(
        context: ReportContext,
        configuration_state: &str,
        complete: bool,
        results: Vec<RuleResult>,
        assessments: Vec<StandardAssessment>,
        inputs: &ScanInputs,
    ) -> Result<Self> {
        let mut checks = Vec::new();
        let mut findings = Vec::new();
        for result in results {
            let mut related = std::collections::BTreeSet::new();
            for mut risk in result.findings {
                if let Some(text) = localized_rule_text(&risk.rule_id, &context.locale) {
                    risk.title = text.title.clone();
                    risk.description = text.description.clone();
                    risk.remediation = text.remediation.clone();
                }
                related.extend(risk.references.iter().map(|r| r.standard_id));
                findings.push(ReportFinding {
                    finding_id: format!("{}:{:05}", context.report_id, findings.len() + 1),
                    finding: risk,
                });
            }
            checks.push(Check {
                rule_id: result.rule_id,
                target: result.path,
                status: result.status,
                reason: result.reason,
                related_standard_ids: related.into_iter().collect(),
            });
        }
        let mut standards = Vec::new();
        let mut licenses = Vec::new();
        for assessment in assessments {
            for control in &assessment.controls {
                checks.push(Check {
                    rule_id: format!(
                        "{}/{}/{}",
                        assessment.standard_id.as_str(),
                        assessment.version.as_deref().unwrap_or(""),
                        control.control_id
                    ),
                    target: String::new(),
                    status: control.status,
                    reason: Some(control.reason.clone()),
                    related_standard_ids: vec![assessment.standard_id],
                });
            }
            if assessment.status == ReviewStatus::NotVerified && assessment.controls.is_empty() {
                checks.push(Check {
                    rule_id: assessment.standard_id.as_str().into(),
                    target: String::new(),
                    status: ReviewStatus::NotVerified,
                    reason: assessment.reason.clone(),
                    related_standard_ids: vec![assessment.standard_id],
                });
            }
            if let Some(metadata) = &assessment.metadata {
                licenses.push(SourceLicense {
                    standard_id: assessment.standard_id,
                    version: metadata.version.clone(),
                    source_url: metadata.source_url.clone(),
                    license_name: metadata.license_name.clone(),
                    license_url: metadata.license_url.clone(),
                    attribution: metadata.attribution.clone(),
                    modification_notice: metadata.modification_notice.clone(),
                });
            }
            standards.push(ReportStandard {
                standard_id: assessment.standard_id,
                version: assessment.version,
                level: assessment.level,
                data_sha256: assessment.metadata.as_ref().map(|m| m.sha256.clone()),
                source_revision: assessment
                    .metadata
                    .as_ref()
                    .map(|m| m.source_revision.clone()),
                installed_status: if assessment.reason.as_deref() == Some("invalid_data") {
                    "invalid"
                } else if assessment.status == ReviewStatus::Disabled {
                    "disabled"
                } else if assessment.metadata.is_some() {
                    "installed"
                } else {
                    "missing"
                }
                .into(),
                status: assessment.status,
                reason: assessment.reason,
                controls: assessment.controls,
            });
        }
        if configuration_state == "disabled" && checks.is_empty() {
            checks.push(Check {
                rule_id: "security_verification".into(),
                target: String::new(),
                status: ReviewStatus::Disabled,
                reason: Some("policy_disabled".into()),
                related_standard_ids: Vec::new(),
            });
        }
        let source_paths = inputs
            .documents
            .iter()
            .map(|d| d.path.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for entry in &inputs.coverage {
            if entry.status == ReviewStatus::NotVerified
                && !matches!(
                    entry.reason.as_deref(),
                    Some("excluded" | "generated" | "awaiting_rules")
                )
            {
                checks.push(Check {
                    rule_id: String::new(),
                    target: entry.path.clone(),
                    status: ReviewStatus::NotVerified,
                    reason: entry.reason.clone(),
                    related_standard_ids: Vec::new(),
                });
            }
        }
        if let Some(reason) = &inputs.stop_reason {
            checks.push(Check {
                rule_id: String::new(),
                target: String::new(),
                status: ReviewStatus::NotVerified,
                reason: Some(reason.clone()),
                related_standard_ids: Vec::new(),
            });
        }
        let mut by_target = std::collections::BTreeMap::new();
        let priority = |status| match status {
            ReviewStatus::NotVerified => 5,
            ReviewStatus::Finding => 4,
            ReviewStatus::Pass => 3,
            ReviewStatus::NotApplicable => 2,
            ReviewStatus::Disabled => 1,
        };
        for check in &checks {
            by_target
                .entry(check.target.as_str())
                .and_modify(|status| {
                    if priority(check.status) > priority(*status) {
                        *status = check.status;
                    }
                })
                .or_insert(check.status);
        }
        let mut coverage = inputs.coverage.clone();
        for entry in &mut coverage {
            if entry.reason.as_deref() == Some("awaiting_rules") {
                let status = by_target.get(entry.path.as_str()).copied();
                entry.status = status.unwrap_or(ReviewStatus::NotApplicable);
                entry.reason = Some(
                    if status.is_none() {
                        "no_applicable_rule"
                    } else {
                        "rules_recorded"
                    }
                    .into(),
                );
            }
        }
        if let Some(reason) = &inputs.stop_reason {
            coverage.push(CoverageEntry {
                path: String::new(),
                status: ReviewStatus::NotVerified,
                reason: Some(reason.clone()),
                bytes: 0,
            });
        }
        let failed_items = coverage
            .iter()
            .filter(|c| {
                c.status == ReviewStatus::NotVerified
                    && !matches!(c.reason.as_deref(), Some("excluded" | "generated"))
            })
            .count();
        let skipped_files = coverage
            .iter()
            .filter(|c| !source_paths.contains(c.path.as_str()))
            .count();
        let sources = inputs
            .documents
            .iter()
            .map(|d| SourceIdentity {
                path: d.path.clone(),
                content_digest: d.content_digest.clone(),
            })
            .collect();
        let locale = match context.locale.as_str() {
            "tw" => "tw",
            "ja" => "ja",
            _ => "en",
        }
        .to_owned();
        let mut report = Self(ReportWire {
            format_version: 1,
            report_id: context.report_id,
            project_key: context.project_key,
            project_name: context.project_name,
            change_name: context.change_name,
            reviewed_at: context.reviewed_at,
            commit_sha: context.commit_sha,
            policy_digest: context.policy_digest,
            policy_revision: context.policy_revision,
            prepared_at: context.prepared_at,
            standards,
            rule_version: context.rule_version,
            scope: context.scope,
            execution_mode: "builtin-offline".into(),
            review_methods: context.review_methods,
            complete: complete && inputs.complete,
            configuration_state: configuration_state.into(),
            counts: Counts::default(),
            findings,
            checks,
            coverage: Coverage {
                inspected_files: inputs.documents.len(),
                skipped_files,
                failed_items,
                bytes_read: inputs.total_bytes,
                remaining_targets: inputs.remaining_targets.clone(),
                reasons: coverage,
            },
            disclaimer: disclaimer(&locale).into(),
            source_licenses: licenses,
            sources,
            locale,
        });
        report.sanitize()?;
        Ok(report)
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let mut report = Self(
            serde_json::from_slice(bytes)
                .map_err(|_| anyhow::anyhow!("invalid_report: invalid JSON model"))?,
        );
        report.sanitize()?;
        Ok(report)
    }
    fn sanitize(&mut self) -> Result<()> {
        let report = &mut self.0;
        ensure!(
            report.format_version == 1
                && valid_report_id(&report.report_id)
                && valid_project_key(&report.project_key),
            "invalid_report: identity or format"
        );
        ensure!(
            matches!(
                report.configuration_state.as_str(),
                "disabled" | "unconfigured" | "configured"
            ) && report.execution_mode == "builtin-offline",
            "invalid_report: execution mode"
        );
        ensure!(
            report.findings.len() <= 10_000
                && report.checks.len() <= 200_000
                && report.sources.len() <= 10_000,
            "size_limit: report"
        );
        report.project_name = plain(&report.project_name);
        report.change_name = report.change_name.as_deref().map(plain);
        report.disclaimer = disclaimer(&report.locale).into();
        report.commit_sha = report.commit_sha.take().filter(|sha| {
            matches!(sha.len(), 40 | 64) && sha.bytes().all(|b| b.is_ascii_hexdigit())
        });
        let mut counts = Counts {
            findings: report.findings.len(),
            ..Default::default()
        };
        for check in &mut report.checks {
            check.target = plain(&check.target);
            check.reason = check.reason.as_deref().map(reason_code);
            match check.status {
                ReviewStatus::Pass | ReviewStatus::Finding => counts.checks_succeeded += 1,
                ReviewStatus::NotVerified => {
                    counts.not_verified += 1;
                    counts.checks_failed += 1;
                }
                ReviewStatus::NotApplicable => counts.not_applicable += 1,
                ReviewStatus::Disabled => counts.disabled += 1,
            }
        }
        for finding in &mut report.findings {
            let risk = &mut finding.finding;
            ensure!(
                matches!(
                    risk.severity.as_str(),
                    "low" | "medium" | "high" | "critical"
                ) && matches!(risk.confidence.as_str(), "low" | "medium" | "high")
                    && matches!(risk.kind.as_str(), "potential_risk" | "missing_requirement"),
                "invalid_report: finding classification"
            );
            risk.confirmation_status = "needs_review".into();
            risk.path = plain(&risk.path);
            risk.title = plain(&risk.title);
            risk.description = plain(&risk.description);
            risk.remediation = plain(&risk.remediation);
            risk.evidence = if risk.rule_id == "SL-SEC-001" {
                "[REDACTED: potential credential assignment]".into()
            } else {
                super::rules::redact_evidence(&risk.evidence)
            };
        }
        for source in &mut report.sources {
            source.path = plain(&source.path);
            ensure!(
                source.content_digest.len() == 64
                    && source.content_digest.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid_report: source fingerprint"
            );
        }
        for path in &mut report.coverage.remaining_targets {
            *path = plain(path);
        }
        for entry in &mut report.coverage.reasons {
            entry.path = plain(&entry.path);
            entry.reason = entry.reason.as_deref().map(reason_code);
        }
        report.complete &= counts.not_verified == 0
            && report.coverage.failed_items == 0
            && report.coverage.remaining_targets.is_empty();
        report.counts = counts;
        Ok(())
    }
    pub fn report_id(&self) -> &str {
        &self.0.report_id
    }
    pub fn has_source(&self, path: &str) -> bool {
        self.0.sources.iter().any(|s| s.path == path)
    }
    pub fn project_key(&self) -> &str {
        &self.0.project_key
    }
    pub fn change_name(&self) -> Option<&str> {
        self.0.change_name.as_deref()
    }
    pub fn is_complete(&self) -> bool {
        self.0.complete
    }
    pub fn counts(&self) -> &Counts {
        &self.0.counts
    }
    pub fn configuration_state(&self) -> &str {
        &self.0.configuration_state
    }
    pub fn is_cancelled(&self) -> bool {
        self.0
            .coverage
            .reasons
            .iter()
            .any(|c| c.reason.as_deref() == Some("cancelled"))
    }
    pub fn summary(&self) -> String {
        format!(
            "{}: {} · {} · findings={} · notVerified={}\n{}",
            title(&self.0.locale),
            self.0.report_id,
            self.0.configuration_state,
            self.0.counts.findings,
            self.0.counts.not_verified,
            self.0.disclaimer
        )
    }
    pub fn markdown(&self) -> String {
        use std::fmt::Write;
        let report = &self.0;
        let mut output = format!("# {}\n\n{}\n\n", title(&report.locale), report.disclaimer);
        for (name, value) in [
            ("reportId", report.report_id.as_str()),
            ("project", report.project_name.as_str()),
            ("reviewedAt", report.reviewed_at.as_str()),
            ("commitSha", report.commit_sha.as_deref().unwrap_or("null")),
            (
                "changeName",
                report.change_name.as_deref().unwrap_or("null"),
            ),
            ("configurationState", report.configuration_state.as_str()),
            ("executionMode", report.execution_mode.as_str()),
            ("ruleVersion", report.rule_version.as_str()),
            ("policyDigest", report.policy_digest.as_str()),
            (
                "preparedAt",
                report.prepared_at.as_deref().unwrap_or("null"),
            ),
        ] {
            let _ = writeln!(output, "- {name}: {}", markdown_text(value));
        }
        let _=writeln!(output,"- policyRevision: {}\n- scope: {}\n- reviewMethods: {}\n- commitNote: HEAD reference only; contentDigest identifies the actual inspected text",report.policy_revision.map(|r|r.to_string()).unwrap_or_else(||"null".into()),serde_json::to_value(report.scope).unwrap_or_default().as_str().unwrap_or("off"),markdown_text(&report.review_methods.join(", ")));
        let _=writeln!(output,"- complete: {}\n- checksSucceeded: {}\n- checksFailed: {}\n- findings: {}\n- notVerified: {}\n- notApplicable: {}\n- disabled: {}\n",report.complete,report.counts.checks_succeeded,report.counts.checks_failed,report.counts.findings,report.counts.not_verified,report.counts.not_applicable,report.counts.disabled);
        let _ = writeln!(
            output,
            "## {}\n",
            report_section(&report.locale, "standards")
        );
        for standard in &report.standards {
            let _ = writeln!(
                output,
                "- {} {} {:?}: {} · {}\n  dataSha256: {}\n  sourceRevision: {}",
                standard.standard_id.as_str(),
                markdown_text(standard.version.as_deref().unwrap_or("off")),
                standard.level,
                standard.installed_status,
                status_label(standard.status),
                standard.data_sha256.as_deref().unwrap_or("null"),
                markdown_text(standard.source_revision.as_deref().unwrap_or("null"))
            );
        }
        let _ = writeln!(
            output,
            "\n## {}\n",
            report_section(&report.locale, "findings")
        );
        for item in &report.findings {
            let f = &item.finding;
            let _=writeln!(output,"### {} · {}\n\n- {}:{}\n- severity: {} · confidence: {} · {} · {}\n\n{}\n\n{}\n\n{}\n",markdown_text(&f.rule_id),markdown_text(&f.title),markdown_text(&f.path),f.line,f.severity,f.confidence,f.kind,f.confirmation_status,markdown_text(&f.evidence),markdown_text(&f.description),markdown_text(&f.remediation));
            for reference in &f.references {
                let _ = writeln!(
                    output,
                    "- {} {} {} · {}",
                    reference.standard_id.as_str(),
                    markdown_text(&reference.version),
                    markdown_text(&reference.control_id),
                    markdown_text(&reference.source_url)
                );
            }
        }
        let _ = writeln!(
            output,
            "\n## {}\n\n{}\n| --- | --- | --- | --- |",
            report_section(&report.locale, "checks"),
            report_section(&report.locale, "columns")
        );
        for check in &report.checks {
            let status = serde_json::to_value(check.status).unwrap_or_default();
            let _ = writeln!(
                output,
                "| {} | {} | {} | {} |",
                markdown_text(&check.rule_id),
                markdown_text(&check.target),
                status.as_str().unwrap_or("NOT_VERIFIED"),
                markdown_text(check.reason.as_deref().unwrap_or(""))
            );
        }
        let _=writeln!(output,"\n## {}\n\n- inspectedFiles: {}\n- skippedFiles: {}\n- failedItems: {}\n- bytesRead: {}\n",report_section(&report.locale,"coverage"),report.coverage.inspected_files,report.coverage.skipped_files,report.coverage.failed_items,report.coverage.bytes_read);
        for entry in &report.coverage.reasons {
            let _ = writeln!(
                output,
                "- {}: {}",
                markdown_text(&entry.path),
                markdown_text(entry.reason.as_deref().unwrap_or(""))
            );
        }
        for path in &report.coverage.remaining_targets {
            let _ = writeln!(output, "- remaining: {}", markdown_text(path));
        }
        for source in &report.sources {
            let _ = writeln!(
                output,
                "- {} · contentDigest: {}",
                markdown_text(&source.path),
                source.content_digest
            );
        }
        let _ = writeln!(
            output,
            "\n## {}\n",
            report_section(&report.locale, "sources")
        );
        for source in &report.source_licenses {
            let _ = writeln!(
                output,
                "- {} {}: {} · {}\n  {}\n  {}\n  {}",
                source.standard_id.as_str(),
                markdown_text(&source.version),
                markdown_text(&source.license_name),
                markdown_text(&source.license_url),
                markdown_text(&source.attribution),
                markdown_text(&source.source_url),
                markdown_text(&source.modification_notice)
            );
        }
        output
    }
}

#[derive(Deserialize)]
struct RuleText {
    title: String,
    description: String,
    remediation: String,
}
fn localized_rule_text(id: &str, locale: &str) -> Option<&'static RuleText> {
    static TEXT: std::sync::OnceLock<
        std::collections::BTreeMap<String, std::collections::BTreeMap<String, RuleText>>,
    > = std::sync::OnceLock::new();
    TEXT.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/security/rule-locales.json"))
            .expect("validated built-in translations")
    })
    .get(locale)?
    .get(id)
}
fn report_section(locale: &str, key: &str) -> &'static str {
    match (locale, key) {
        ("tw", "standards") => "審查標準",
        ("ja", "standards") => "審査基準",
        (_, "standards") => "Standards",
        ("tw", "findings") => "疑似風險與安全要求缺漏",
        ("ja", "findings") => "疑わしいリスクと要件の不足",
        (_, "findings") => "Findings",
        ("tw", "checks") => "逐項檢查",
        ("ja", "checks") => "個別の確認",
        (_, "checks") => "Checks",
        ("tw", "columns") => "| 規則／控制項 | 目標 | 狀態 | 原因 |",
        ("ja", "columns") => "| ルール／管理策 | 対象 | 状態 | 理由 |",
        (_, "columns") => "| Rule / control | Target | Status | Reason |",
        ("tw", "coverage") => "審查覆蓋",
        ("ja", "coverage") => "審査範囲",
        (_, "coverage") => "Coverage",
        ("tw", "sources") => "來源與授權",
        ("ja", "sources") => "出典とライセンス",
        _ => "Sources and licenses",
    }
}

pub fn valid_report_id(value: &str) -> bool {
    value.len() == 26
        && ulid::Ulid::from_string(value).is_ok()
        && value
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}
pub fn valid_project_key(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn plain(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(4096)
        .collect()
}
fn status_label(status: ReviewStatus) -> &'static str {
    match status {
        ReviewStatus::Pass => "PASS",
        ReviewStatus::Finding => "FINDING",
        ReviewStatus::NotVerified => "NOT_VERIFIED",
        ReviewStatus::NotApplicable => "NOT_APPLICABLE",
        ReviewStatus::Disabled => "DISABLED",
    }
}
fn reason_code(value: &str) -> String {
    if value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        value.into()
    } else {
        "review_error".into()
    }
}
fn markdown_text(value: &str) -> String {
    plain(value)
        .chars()
        .flat_map(|c| match c {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect(),
            '>' => "&gt;".chars().collect(),
            '\\' | '`' | '*' | '_' | '[' | ']' | '(' | ')' | '#' | '!' | '|' => vec!['\\', c],
            _ => vec![c],
        })
        .collect()
}
fn title(locale: &str) -> &'static str {
    match locale {
        "tw" => "OWASP / CWE 初步安全審查報告",
        "ja" => "OWASP / CWE 予備セキュリティレビューレポート",
        _ => "OWASP / CWE Preliminary Security Review Report",
    }
}
fn disclaimer(locale: &str) -> &'static str {
    match locale {"tw"=>"PASS 只代表本次執行的指定規則沒有發現問題，不代表不存在其他漏洞。這是初步安全審查，並非安全認證或完整標準符合性驗證。","ja"=>"PASS は今回実行した指定ルールで問題が見つからなかったことのみを示し、他の脆弱性がないことを保証しません。これは予備レビューであり、認証や規格全体への適合検証ではありません。",_=>"PASS means only that the specified rules executed in this review found no matching issue. It does not establish the absence of other vulnerabilities. This is a preliminary review, not certification or full standards compliance verification."}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportReceipt {
    pub report_id: String,
    pub project_key: String,
    pub warnings: Vec<String>,
}
pub trait ReportRepository: Send + Sync {
    fn save(&self, report: &Report) -> Result<ReportReceipt>;
    fn get(&self, report_id: &str) -> Result<Option<Report>>;
    fn latest(&self, project_key: &str, change: Option<&str>) -> Result<Option<Report>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn security_report_localizes_owned_rule_explanations_and_markdown_sections() {
        for locale in ["tw", "ja"] {
            for rule in RuleSet::builtin().unwrap().rules {
                let text = localized_rule_text(&rule.id, locale)
                    .expect("every owned rule has a translation");
                assert!(!text.title.is_empty());
                assert!(!text.description.is_empty());
                assert!(!text.remediation.is_empty());
                assert_ne!(text.title, rule.title);
            }
        }
        let inputs = ScanInputs {
            visited_files: 0,
            documents: vec![],
            coverage: vec![],
            remaining_targets: vec![],
            total_bytes: 0,
            complete: true,
            stop_reason: None,
        };
        for (locale, section) in [
            ("tw", "## 審查標準"),
            ("ja", "## 審査基準"),
            ("en", "## Standards"),
        ] {
            let mut ctx = context();
            ctx.locale = locale.into();
            let report = Report::new(ctx, "disabled", true, vec![], vec![], &inputs).unwrap();
            assert!(report.markdown().contains(section));
        }
    }
    use crate::security::{
        input::{ScanInputs, SourceDocument},
        policy::SecurityScope,
        rules::RuleSet,
    };
    fn context() -> ReportContext {
        ReportContext {
            report_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
            project_key: "a".repeat(64),
            project_name: "Example <script>\u{1b}[31m".into(),
            change_name: Some("demo".into()),
            reviewed_at: "2026-10-17T00:00:00Z".into(),
            commit_sha: None,
            policy_digest: "b".repeat(64),
            policy_revision: Some(3),
            prepared_at: None,
            rule_version: "1.0.0".into(),
            scope: SecurityScope::WebApi,
            locale: "tw".into(),
            review_methods: vec!["code".into()],
        }
    }
    #[test]
    fn security_report_serializes_one_redacted_model_and_preserves_status_counts() {
        let document = SourceDocument::new(
            "app.ts".into(),
            "const token = 'unique-private-credential-9953';\nnode.innerHTML = input;".into(),
        );
        let mut checks = RuleSet::builtin()
            .unwrap()
            .review_code(&document, &["SL-SEC-004".into()], &|| None)
            .unwrap();
        checks.push(crate::security::rules::RuleResult {
            rule_id: "SL-API-001".into(),
            path: "openapi.yaml".into(),
            status: crate::security::ReviewStatus::NotVerified,
            reason: Some("missing_schema".into()),
            findings: Vec::new(),
        });
        let inputs = ScanInputs {
            visited_files: 1,
            documents: vec![document],
            coverage: Vec::new(),
            remaining_targets: vec!["unknown.py".into()],
            total_bytes: 100,
            complete: false,
            stop_reason: Some("budget_exceeded".into()),
        };
        let report =
            Report::new(context(), "configured", false, checks, Vec::new(), &inputs).unwrap();
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["formatVersion"], 1);
        assert_eq!(json["commitSha"], serde_json::Value::Null);
        assert_eq!(json["executionMode"], "builtin-offline");
        assert_eq!(json["counts"]["findings"], 2);
        assert_eq!(json["counts"]["notVerified"], 2);
        assert_eq!(json["counts"]["disabled"], 1);
        assert!(!json["complete"].as_bool().unwrap());
        assert_eq!(
            json["sources"][0]["contentDigest"].as_str().unwrap().len(),
            64
        );
        assert!(json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["status"] == "PASS"));
        assert!(json["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["status"] == "FINDING"));
        let markdown = report.markdown();
        for output in [
            serde_json::to_string(&report).unwrap(),
            markdown.clone(),
            report.summary(),
        ] {
            assert!(!output.contains("unique-private-credential-9953"));
            assert!(!output.contains('\u{1b}'));
            assert!(output.contains("不代表不存在其他漏洞"));
        }
        assert!(!markdown.contains("<script>"));
        assert_eq!(
            Report::from_json(&serde_json::to_vec(&report).unwrap())
                .unwrap()
                .markdown(),
            markdown
        );
        let mut corrupt = json.clone();
        corrupt["formatVersion"] = serde_json::json!(99);
        assert!(Report::from_json(&serde_json::to_vec(&corrupt).unwrap()).is_err());
    }
    #[test]
    fn security_report_disabled_cancelled_and_language_are_explicit() {
        let inputs = ScanInputs {
            visited_files: 0,
            documents: Vec::new(),
            coverage: Vec::new(),
            remaining_targets: Vec::new(),
            total_bytes: 0,
            complete: true,
            stop_reason: None,
        };
        let report =
            Report::new(context(), "disabled", true, Vec::new(), Vec::new(), &inputs).unwrap();
        assert!(report.is_complete());
        assert!(report.summary().contains("disabled"));
        for locale in ["tw", "ja", "en"] {
            let mut context = context();
            context.locale = locale.into();
            let report = Report::new(
                context,
                "unconfigured",
                false,
                Vec::new(),
                Vec::new(),
                &inputs,
            )
            .unwrap();
            assert!(report.markdown().contains("PASS"));
            assert!(report.markdown().contains("1.0.0"));
        }
    }
}

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
mod deep;
pub use deep::*;

/// Redact credential-shaped values in prose without removing file paths or ordinary quotes.
pub fn redact_text(text: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;
    static ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)((?:api[_-]?key|password|secret|(?:access|refresh)[_-]?token|authorization)["']?\s*[:=]\s*)(\[REDACTED(?: PRIVATE KEY| TOKEN)?\]|"[^"\r\n]*"|'[^'\r\n]*'|[^\s,;}\]]+)"#).expect("credential redaction")
    });
    static PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?s)-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----.*?(?:-----END (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|$)").expect("private key redaction")
    });
    static JWT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b")
            .expect("JWT redaction")
    });
    let text = PRIVATE_KEY.replace_all(text, "[REDACTED PRIVATE KEY]");
    let text = JWT.replace_all(&text, "[REDACTED TOKEN]");
    ASSIGNMENT
        .replace_all(&text, |captures: &regex::Captures<'_>| {
            let value = &captures[2];
            format!(
                "{}{}",
                &captures[1],
                if matches!(
                    value,
                    "[REDACTED]" | "[REDACTED PRIVATE KEY]" | "[REDACTED TOKEN]"
                ) {
                    value
                } else {
                    "[REDACTED]"
                }
            )
        })
        .into_owned()
}

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
    pub fn sources(&self) -> &[SourceIdentity] {
        &self.0.sources
    }
    pub fn policy_digest(&self) -> &str {
        &self.0.policy_digest
    }
    pub fn checks(&self) -> &[Check] {
        &self.0.checks
    }
    pub fn coverage(&self) -> &Coverage {
        &self.0.coverage
    }
    pub fn findings(&self) -> &[ReportFinding] {
        &self.0.findings
    }
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
        report.project_name = redact_text(&plain(&report.project_name));
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
            risk.title = redact_text(&plain(&risk.title));
            risk.description = redact_text(&plain(&risk.description));
            risk.remediation = redact_text(&plain(&risk.remediation));
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
    pub fn rule_version(&self) -> &str {
        &self.0.rule_version
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
    fn deep_policy_digest_ignores_acceptance_but_tracks_review_scope() {
        let mut policy = crate::security::policy::SecurityPolicy::default();
        let before = policy.review_digest().unwrap();
        policy.required_for_acceptance = true;
        assert_eq!(before, policy.review_digest().unwrap());
        policy.reviews.code = !policy.reviews.code;
        assert_ne!(before, policy.review_digest().unwrap());
    }
    fn deep_input() -> DeepReportInput {
        serde_json::from_value(serde_json::json!({
            "formatVersion":2,"changeName":"demo","policyDigest":"b".repeat(64),"knowledgeDigest":"d".repeat(64),"scopeDigest":"c".repeat(64),"builtinReportId":null,"complete":true,"targets":["app.ts"],
            "checks":[{"checkId":"review/code","status":"FINDING","reason":"已追蹤入口","evidencePaths":["app.ts"],"resolvesCheckIds":[]}],
            "findings":[{"findingId":"H1","domain":"認證與業務授權","title":"跨帳戶修改缺少擁有者檢查","severity":"high","confidence":"high","confidenceReason":"程式已證明缺少檢查；外部ACL未確認；無可信對照","references":[],"evidence":[{"path":"app.ts","line":1,"endLine":2,"function":"updateAccount"}],"codeSnippet":"const api_key = 'unique-deep-secret-sentinel';","dataFlow":"request.id → updateAccount → database；缺少帳戶擁有者比對","impact":"已證明跨帳戶路徑；unique-deep-secret-sentinel 不得出現在報告","remediation":"SHALL 依 .security_kb/cheatsheets/Authorization_Cheat_Sheet.md 驗證資源擁有者。","acceptance":"以帳戶A修改B的ID應回403，且資料不變。"}]
        })).unwrap()
    }
    fn deep_context() -> DeepReportContext {
        DeepReportContext {
            common: context(),
            actor: Some("Reviewer".into()),
            knowledge_digest: "d".repeat(64),
            scope_digest: "c".repeat(64),
            standards: vec![],
            source_licenses: vec![],
            sources: vec![SourceIdentity {
                path: "app.ts".into(),
                content_digest: "e".repeat(64),
            }],
            source_lines: std::collections::BTreeMap::from([("app.ts".into(), 3)]),
            required_checks: std::collections::BTreeSet::from(["review/code".into()]),
            required_evidence: std::collections::BTreeMap::from([(
                "review/code".into(),
                std::collections::BTreeSet::from(["app.ts".into()]),
            )]),
            permitted_references: std::collections::BTreeSet::new(),
            builtin_unverified: std::collections::BTreeSet::new(),
            bytes_read: 20,
        }
    }
    #[test]
    fn deep_report_v2_masks_every_representation_and_has_all_nine_finding_groups() {
        let report = DeepReport::new(deep_input(), deep_context()).unwrap();
        assert!(serde_json::to_string(&report)
            .unwrap()
            .contains("原始碼中的金鑰值在此省略"));
        let json = serde_json::to_string(&report).unwrap();
        let markdown = report.markdown();
        assert!(!json.contains("unique-deep-secret-sentinel"));
        assert!(!markdown.contains("unique-deep-secret-sentinel"));
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["formatVersion"], 2);
        assert_eq!(value["executionMode"], "deep-skill");
        assert_eq!(value["actor"], "Reviewer");
        for heading in [
            "## 認證與業務授權",
            "### [H1]",
            "等級",
            "信心",
            "對照",
            "證據",
            "片段",
            "資料流",
            "影響",
            "修正",
            "驗收",
            "原始碼中的金鑰值在此省略",
        ] {
            assert!(markdown.contains(heading), "{heading}");
        }
        let reread = VersionedReport::from_json(json.as_bytes()).unwrap();
        assert!(matches!(reread, VersionedReport::Deep(_)));
        assert_eq!(reread.markdown(), markdown);
        let mut bad = serde_json::from_str::<serde_json::Value>(&json).unwrap();
        bad["formatVersion"] = 3.into();
        assert!(VersionedReport::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    #[test]
    fn deep_report_rejects_missing_groups_unknown_ids_outside_evidence_and_401_chars() {
        let mut value = serde_json::to_value(deep_input()).unwrap();
        for key in [
            "title",
            "severity",
            "confidence",
            "confidenceReason",
            "references",
            "evidence",
            "codeSnippet",
            "dataFlow",
            "impact",
            "remediation",
            "acceptance",
        ] {
            let mut missing = value.clone();
            missing["findings"][0].as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<DeepReportInput>(missing).is_err(),
                "{key}"
            );
        }
        for pointer in ["/findings/0/evidence/0/path", "/checks/0/checkId"] {
            let mut bad = value.clone();
            *bad.pointer_mut(pointer).unwrap() = "../outside".into();
            assert!(DeepReport::new(serde_json::from_value(bad).unwrap(), deep_context()).is_err());
        }
        for line in [0, 4] {
            value["findings"][0]["evidence"][0]["line"] = line.into();
            assert!(DeepReport::new(
                serde_json::from_value(value.clone()).unwrap(),
                deep_context()
            )
            .is_err());
        }
        let mut input = deep_input();
        input.findings[0].code_snippet = "x".repeat(400);
        assert!(DeepReport::new(input.clone(), deep_context()).is_ok());
        input.findings[0].code_snippet.push('x');
        assert!(DeepReport::new(input, deep_context()).is_err());
        let mut input = deep_input();
        input.findings.push(input.findings[0].clone());
        assert!(DeepReport::new(input, deep_context()).is_err());
    }
    #[test]
    fn deep_report_does_not_claim_complete_without_target_evidence() {
        let mut input = deep_input();
        input.checks[0].evidence_paths.clear();
        input.findings[0].code_snippet = "api_key=unique-bare-credential".into();
        input.findings[0].impact = "unique-bare-credential 不得外洩".into();
        let report = DeepReport::new(input, deep_context()).unwrap();
        assert!(!report.is_complete());
        assert!(!serde_json::to_string(&report)
            .unwrap()
            .contains("unique-bare-credential"));
    }
    #[test]
    fn deep_empty_family_requires_evidenced_not_applicable_and_standard_status_follows_checks() {
        let mut input = deep_input();
        let mut ctx = deep_context();
        ctx.required_checks.insert("review/spec".into());
        ctx.required_evidence
            .insert("review/spec".into(), Default::default());
        input.checks.push(DeepCheck {
            check_id: "review/spec".into(),
            status: ReviewStatus::Pass,
            reason: String::new(),
            evidence_paths: vec![],
            resolves_check_ids: vec![],
        });
        assert!(
            !DeepReport::new(input.clone(), ctx).unwrap().is_complete(),
            "empty family must not pass vacuously"
        );
        let mut ctx = deep_context();
        ctx.required_checks.insert("review/spec".into());
        ctx.required_evidence
            .insert("review/spec".into(), Default::default());
        input.checks[1].status = ReviewStatus::NotApplicable;
        input.checks[1].reason = "已確認此提案沒有規格文件".into();
        input.checks[1].evidence_paths = vec!["app.ts".into()];
        assert!(DeepReport::new(input, ctx).unwrap().is_complete());
        let mut input = deep_input();
        let mut ctx = deep_context();
        let id = "owasp_top10/2021/A01:2021";
        ctx.required_checks.insert(id.into());
        input.checks.push(DeepCheck {
            check_id: id.into(),
            status: ReviewStatus::Pass,
            reason: "已追蹤授權路徑".into(),
            evidence_paths: vec!["app.ts".into()],
            resolves_check_ids: vec![],
        });
        ctx.standards.push(ReportStandard {
            standard_id: StandardId::OwaspTop10,
            version: Some("2021".into()),
            level: None,
            data_sha256: None,
            source_revision: None,
            installed_status: "installed".into(),
            status: ReviewStatus::NotVerified,
            reason: Some("awaiting_deep_review".into()),
            controls: vec![ControlAssessment {
                control_id: "A01:2021".into(),
                status: ReviewStatus::NotVerified,
                reason: "awaiting_deep_review".into(),
            }],
        });
        let value = serde_json::to_value(DeepReport::new(input, ctx).unwrap()).unwrap();
        assert_eq!(value["standards"][0]["controls"][0]["status"], "PASS");
        assert_eq!(value["standards"][0]["status"], "PASS");
    }
    #[test]
    fn basic_wire_prose_cannot_echo_secret_assignments() {
        let report = Report::new(
            context(),
            "configured",
            true,
            vec![],
            vec![],
            &ScanInputs {
                documents: vec![],
                coverage: vec![],
                visited_files: 0,
                total_bytes: 0,
                complete: true,
                remaining_targets: vec![],
                stop_reason: None,
            },
        )
        .unwrap();
        let mut value = serde_json::to_value(report).unwrap();
        value["projectName"] = "password=unique-prose-secret".into();
        let report = Report::from_json(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(!serde_json::to_string(&report)
            .unwrap()
            .contains("unique-prose-secret"));
        assert!(!report.markdown().contains("unique-prose-secret"));
    }
    #[test]
    fn security_risk_decision_validates_reason_identity_and_cas_without_rewriting_report() {
        use crate::{security::acceptance::RiskDecision, store::Store, teststore::TestStore};
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        let report = VersionedReport::Deep(DeepReport::new(deep_input(), deep_context()).unwrap());
        save_change_report(&store, "demo", &report).unwrap();
        let original = report.json().unwrap();
        let mut d = RiskDecision {
            decision_id: "01ARZ3NDEKTSV4RRFFQ69G5FAW".into(),
            action: "accepted".into(),
            report_id: report.report_id().into(),
            finding_id: "H1".into(),
            reason: "接受本次範圍的殘餘風險".into(),
            actor: "Owner".into(),
            accepted_at: "2026-10-20T00:00:00Z".into(),
            policy_digest: "b".repeat(64),
            scope_digest: "c".repeat(64),
        };
        for reason in [" ".into(), "字".repeat(2001)] {
            d.reason = reason;
            assert!(prepare_decision(&store, "demo", &d).is_err());
        }
        d.reason = "字".repeat(2000);
        assert!(prepare_decision(&store, "demo", &d).is_ok());
        d.actor.clear();
        assert!(prepare_decision(&store, "demo", &d).is_err());
        d.actor = "Owner".into();
        d.reason = "API_KEY=unique-decision-private-sentinel 已經輪替".into();
        let plan = prepare_decision(&store, "demo", &d).unwrap();
        let old = store.read_artifact("demo", crate::history::INDEX).unwrap();
        store
            .write_artifact("demo", crate::history::INDEX, &format!("{old}\n"))
            .unwrap();
        assert!(store.commit_checks("demo", &plan).is_err());
        let path = format!("checks/security/{}/decisions.jsonl", report.report_id());
        assert!(store.read_artifact("demo", &path).is_none());
        let plan = prepare_decision(&store, "demo", &d).unwrap();
        store.commit_checks("demo", &plan).unwrap();
        assert!(!store
            .read_artifact("demo", &path)
            .unwrap()
            .contains("unique-decision-private-sentinel"));
        assert_eq!(
            store
                .read_artifact(
                    "demo",
                    &format!("checks/security/{}/report.json", report.report_id())
                )
                .unwrap(),
            original
        );
    }
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

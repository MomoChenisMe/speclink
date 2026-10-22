//! Version 2 deep reports. Only validated, redacted models can be published.
use super::super::{data::sha256, prepared::valid_relative};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepReportInput {
    pub format_version: u8,
    pub change_name: String,
    pub policy_digest: String,
    pub knowledge_digest: String,
    pub scope_digest: String,
    #[serde(deserialize_with = "optional_string")]
    pub builtin_report_id: Option<String>,
    pub complete: bool,
    pub targets: Vec<String>,
    pub checks: Vec<DeepCheck>,
    pub findings: Vec<DeepFinding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptRiskInput {
    pub report_id: String,
    pub finding_id: String,
    pub reason: String,
}
impl AcceptRiskInput {
    pub fn validate(&self) -> Result<()> {
        crate::history::validate_id(&self.report_id)?;
        ensure!(
            !self.finding_id.is_empty()
                && self.finding_id.len() <= 128
                && self
                    .finding_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':')),
            "invalid_finding_id"
        );
        ensure!(
            (1..=2000).contains(&self.reason.trim().chars().count()),
            "invalid_acceptance_reason"
        );
        Ok(())
    }
}
fn optional_string<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepCheck {
    pub check_id: String,
    pub status: ReviewStatus,
    pub reason: String,
    pub evidence_paths: Vec<String>,
    pub resolves_check_ids: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepReference {
    pub standard_id: StandardId,
    pub version: String,
    pub control_id: String,
    pub local_path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepEvidence {
    pub path: String,
    pub line: usize,
    pub end_line: usize,
    pub function: String,
}
fn default_domain() -> String {
    "安全與業務邏輯".into()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeepFinding {
    pub finding_id: String,
    #[serde(default = "default_domain")]
    pub domain: String,
    pub title: String,
    pub severity: String,
    pub confidence: String,
    pub confidence_reason: String,
    pub references: Vec<DeepReference>,
    pub evidence: Vec<DeepEvidence>,
    pub code_snippet: String,
    #[serde(default)]
    pub redaction_notice: Option<String>,
    pub data_flow: String,
    pub impact: String,
    pub remediation: String,
    pub acceptance: String,
}

pub struct DeepReportContext {
    pub common: ReportContext,
    pub actor: Option<String>,
    pub knowledge_digest: String,
    pub scope_digest: String,
    pub standards: Vec<ReportStandard>,
    pub source_licenses: Vec<SourceLicense>,
    pub sources: Vec<SourceIdentity>,
    pub source_lines: BTreeMap<String, usize>,
    pub required_checks: BTreeSet<String>,
    pub required_evidence: BTreeMap<String, BTreeSet<String>>,
    pub permitted_references: BTreeSet<(StandardId, String, String, String)>,
    pub builtin_unverified: BTreeSet<String>,
    pub bytes_read: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DeepWire {
    format_version: u8,
    report_id: String,
    project_key: String,
    project_name: String,
    change_name: Option<String>,
    reviewed_at: String,
    actor: Option<String>,
    commit_sha: Option<String>,
    policy_digest: String,
    knowledge_digest: String,
    scope_digest: String,
    builtin_report_id: Option<String>,
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
    findings: Vec<DeepFinding>,
    checks: Vec<DeepCheck>,
    coverage: Coverage,
    disclaimer: String,
    source_licenses: Vec<SourceLicense>,
    sources: Vec<SourceIdentity>,
    locale: String,
    targets: Vec<String>,
    #[serde(default)]
    required_checks: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct DeepReport(DeepWire);
impl DeepReport {
    pub fn new(input: DeepReportInput, context: DeepReportContext) -> Result<Self> {
        ensure!(
            input.format_version == 2
                && Some(&input.change_name) == context.common.change_name.as_ref(),
            "invalid_report: change or version"
        );
        ensure!(
            input.policy_digest == context.common.policy_digest
                && input.knowledge_digest == context.knowledge_digest
                && input.scope_digest == context.scope_digest,
            "security_report_stale"
        );
        let targets: BTreeSet<_> = input.targets.iter().collect();
        ensure!(
            targets.len() == input.targets.len()
                && targets == context.source_lines.keys().collect(),
            "invalid_report: targets must match the authorized source set"
        );
        for check in &input.checks {
            ensure!(
                context.required_checks.contains(&check.check_id),
                "invalid_report: unknown check"
            );
            ensure!(
                check
                    .evidence_paths
                    .iter()
                    .all(|path| context.source_lines.contains_key(path)),
                "invalid_report: check evidence outside target"
            );
            ensure!(
                check
                    .resolves_check_ids
                    .iter()
                    .all(|id| context.builtin_unverified.contains(id)),
                "invalid_report: unknown unverified check"
            );
        }
        for finding in &input.findings {
            for evidence in &finding.evidence {
                let lines = context
                    .source_lines
                    .get(&evidence.path)
                    .ok_or_else(|| anyhow::anyhow!("invalid_report: evidence outside target"))?;
                ensure!(
                    evidence.line > 0
                        && evidence.line <= *lines
                        && evidence.end_line >= evidence.line
                        && evidence.end_line <= *lines,
                    "invalid_report: evidence line range"
                );
            }
            for reference in &finding.references {
                ensure!(
                    context.permitted_references.contains(&(
                        reference.standard_id,
                        reference.version.clone(),
                        reference.control_id.clone(),
                        reference.local_path.clone()
                    )),
                    "invalid_report: reference is not a selected local control"
                );
            }
        }
        let missing_evidence = context
            .required_evidence
            .iter()
            .filter(|(id, paths)| {
                !input.checks.iter().any(|check| {
                    check.check_id == **id
                        && if paths.is_empty() {
                            check.status == ReviewStatus::NotApplicable
                                && !check.reason.trim().is_empty()
                                && !check.evidence_paths.is_empty()
                        } else {
                            paths.iter().all(|path| check.evidence_paths.contains(path))
                        }
                })
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let c = context.common;
        let mut report = Self(DeepWire {
            format_version: 2,
            report_id: c.report_id,
            project_key: c.project_key,
            project_name: c.project_name,
            change_name: c.change_name,
            reviewed_at: c.reviewed_at,
            actor: context.actor,
            commit_sha: c.commit_sha,
            policy_digest: input.policy_digest,
            knowledge_digest: input.knowledge_digest,
            scope_digest: input.scope_digest,
            builtin_report_id: input.builtin_report_id,
            policy_revision: c.policy_revision,
            prepared_at: c.prepared_at,
            standards: context.standards,
            rule_version: c.rule_version,
            scope: c.scope,
            execution_mode: "deep-skill".into(),
            review_methods: c.review_methods,
            complete: input.complete,
            configuration_state: "configured".into(),
            counts: Counts::default(),
            findings: input.findings,
            checks: input.checks,
            coverage: Coverage {
                inspected_files: context.sources.len(),
                skipped_files: 0,
                failed_items: missing_evidence.len(),
                bytes_read: context.bytes_read,
                remaining_targets: vec![],
                reasons: missing_evidence
                    .into_iter()
                    .map(|id| CoverageEntry {
                        path: id,
                        status: ReviewStatus::NotVerified,
                        reason: Some("incomplete_target_evidence".into()),
                        bytes: 0,
                    })
                    .collect(),
            },
            disclaimer: disclaimer("tw").into(),
            source_licenses: context.source_licenses,
            sources: context.sources,
            locale: "tw".into(),
            targets: input.targets,
            required_checks: context.required_checks.into_iter().collect(),
        });
        report.sanitize()?;
        Ok(report)
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let mut report = Self(
            serde_json::from_slice(bytes)
                .map_err(|_| anyhow::anyhow!("invalid_report: deep JSON model"))?,
        );
        report.sanitize()?;
        Ok(report)
    }
    fn sanitize(&mut self) -> Result<()> {
        let r = &mut self.0;
        ensure!(
            r.format_version == 2
                && r.execution_mode == "deep-skill"
                && valid_report_id(&r.report_id)
                && valid_project_key(&r.project_key),
            "invalid_report: identity or version"
        );
        ensure!(
            r.change_name
                .as_ref()
                .is_some_and(|name| valid_relative(name) && !name.contains('/')),
            "invalid_report: change"
        );
        ensure!(
            [&r.policy_digest, &r.knowledge_digest, &r.scope_digest]
                .iter()
                .all(|value| crate::history::valid_digest(value)),
            "invalid_report: digest"
        );
        ensure!(
            r.builtin_report_id.as_deref().is_none_or(valid_report_id),
            "invalid_report: builtin ID"
        );
        ensure!(
            r.findings.len() <= 10_000 && r.checks.len() <= 200_000 && r.targets.len() <= 10_000,
            "size_limit: deep report"
        );
        let targets: BTreeSet<_> = r.targets.iter().collect();
        ensure!(
            targets.len() == r.targets.len() && targets.iter().all(|path| valid_relative(path)),
            "invalid_report: target path"
        );
        let secrets = sensitive_literals(&r.findings);
        let mask = |text: &str| {
            let mut text = text.to_owned();
            for secret in &secrets {
                text = text.replace(secret, "[REDACTED]");
            }
            redact_text(&text)
        };
        r.project_name = mask(&plain(&r.project_name));
        r.actor = r.actor.as_deref().map(|actor| mask(&plain(actor)));
        let mut ids = BTreeSet::new();
        let mut check_ids = BTreeSet::new();
        let mut counts = Counts {
            findings: r.findings.len(),
            ..Counts::default()
        };
        for check in &mut r.checks {
            ensure!(
                check_ids.insert(check.check_id.clone()) && !check.check_id.trim().is_empty(),
                "invalid_report: duplicate check ID"
            );
            ensure!(
                check
                    .evidence_paths
                    .iter()
                    .all(|path| targets.contains(path)),
                "invalid_report: check path"
            );
            if matches!(
                check.status,
                ReviewStatus::NotVerified | ReviewStatus::NotApplicable
            ) {
                ensure!(
                    !check.reason.trim().is_empty(),
                    "invalid_report: reason required"
                );
            }
            if !check.resolves_check_ids.is_empty() {
                ensure!(
                    matches!(
                        check.status,
                        ReviewStatus::Pass | ReviewStatus::Finding | ReviewStatus::NotApplicable
                    ) && !check.evidence_paths.is_empty()
                        && !check.reason.trim().is_empty(),
                    "invalid_report: a resolution needs verified evidence"
                );
            }
            check.reason = mask(&check.reason);
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
        for finding in &mut r.findings {
            ensure!(
                ids.insert(finding.finding_id.clone())
                    && !finding.finding_id.is_empty()
                    && finding.finding_id.len() <= 64
                    && finding
                        .finding_id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')),
                "invalid_report: finding ID"
            );
            ensure!(
                matches!(
                    finding.severity.as_str(),
                    "low" | "medium" | "high" | "critical"
                ) && matches!(finding.confidence.as_str(), "low" | "medium" | "high"),
                "invalid_report: classification"
            );
            ensure!(
                [
                    &finding.title,
                    &finding.confidence_reason,
                    &finding.code_snippet,
                    &finding.data_flow,
                    &finding.impact,
                    &finding.remediation,
                    &finding.acceptance
                ]
                .iter()
                .all(|text| !text.trim().is_empty()),
                "invalid_report: all nine finding groups are required"
            );
            ensure!(
                finding.code_snippet.chars().count() <= 400
                    && finding.data_flow.contains('→')
                    && finding.remediation.contains("SHALL"),
                "invalid_report: snippet or engineering format"
            );
            ensure!(
                !finding.evidence.is_empty(),
                "invalid_report: missing evidence"
            );
            for evidence in &mut finding.evidence {
                ensure!(
                    targets.contains(&evidence.path)
                        && evidence.line > 0
                        && evidence.end_line >= evidence.line
                        && !evidence.function.trim().is_empty(),
                    "invalid_report: evidence range"
                );
                evidence.function = mask(&evidence.function);
            }
            if finding.references.is_empty() {
                ensure!(
                    finding.confidence_reason.contains("無可信對照"),
                    "invalid_report: explicitly disclose unmapped references"
                );
            }
            for reference in &finding.references {
                ensure!(
                    valid_relative(&reference.local_path)
                        && reference.local_path.starts_with(".security_kb/"),
                    "invalid_report: non-local reference"
                );
            }
            finding.domain = mask(&finding.domain);
            finding.title = mask(&finding.title);
            finding.confidence_reason = mask(&finding.confidence_reason);
            let snippet = super::super::rules::redact_evidence(&mask(&finding.code_snippet));
            if snippet != finding.code_snippet || finding.redaction_notice.is_some() {
                finding.redaction_notice = Some("原始碼中的金鑰值在此省略".into());
            }
            finding.code_snippet = snippet;
            finding.data_flow = mask(&finding.data_flow);
            finding.impact = mask(&finding.impact);
            finding.remediation = mask(&finding.remediation);
            finding.acceptance = mask(&finding.acceptance);
        }
        for required in &r.required_checks {
            if !check_ids.contains(required)
                || r.checks.iter().any(|check| {
                    check.check_id == *required && check.status == ReviewStatus::Disabled
                })
            {
                counts.not_verified += 1;
                counts.checks_failed += 1;
            }
        }
        r.complete &= !r.required_checks.is_empty()
            && counts.not_verified == 0
            && r.coverage.failed_items == 0
            && !r.targets.is_empty();
        if r.checks
            .iter()
            .any(|check| check.status == ReviewStatus::Finding)
            && r.findings.is_empty()
        {
            r.complete = false;
        }
        for standard in &mut r.standards {
            for control in &mut standard.controls {
                let id = format!(
                    "{}/{}/{}",
                    standard.standard_id.as_str(),
                    standard.version.as_deref().unwrap_or_default(),
                    control.control_id
                );
                if let Some(check) = r.checks.iter().find(|c| c.check_id == id) {
                    control.status = check.status;
                    control.reason = check.reason.clone();
                } else {
                    control.status = ReviewStatus::NotVerified;
                    control.reason = "missing_deep_check".into();
                }
            }
            standard.status = if standard.controls.is_empty()
                || standard
                    .controls
                    .iter()
                    .any(|c| matches!(c.status, ReviewStatus::NotVerified | ReviewStatus::Disabled))
            {
                ReviewStatus::NotVerified
            } else if standard
                .controls
                .iter()
                .any(|c| c.status == ReviewStatus::Finding)
            {
                ReviewStatus::Finding
            } else if standard
                .controls
                .iter()
                .all(|c| c.status == ReviewStatus::NotApplicable)
            {
                ReviewStatus::NotApplicable
            } else {
                ReviewStatus::Pass
            };
            standard.reason = None;
        }
        r.counts = counts;
        r.disclaimer = disclaimer("tw").into();
        r.locale = "tw".into();
        Ok(())
    }
    pub fn report_id(&self) -> &str {
        &self.0.report_id
    }
    pub fn project_key(&self) -> &str {
        &self.0.project_key
    }
    pub fn change_name(&self) -> &str {
        self.0.change_name.as_deref().expect("validated change")
    }
    pub fn is_complete(&self) -> bool {
        self.0.complete
    }
    pub fn policy_digest(&self) -> &str {
        &self.0.policy_digest
    }
    pub fn scope_digest(&self) -> &str {
        &self.0.scope_digest
    }
    pub fn knowledge_digest(&self) -> &str {
        &self.0.knowledge_digest
    }
    pub fn builtin_report_id(&self) -> Option<&str> {
        self.0.builtin_report_id.as_deref()
    }
    pub fn findings(&self) -> &[DeepFinding] {
        &self.0.findings
    }
    pub fn checks(&self) -> &[DeepCheck] {
        &self.0.checks
    }
    pub fn sources(&self) -> &[SourceIdentity] {
        &self.0.sources
    }
    pub fn reviewed_at(&self) -> &str {
        &self.0.reviewed_at
    }
    pub fn markdown(&self) -> String {
        let r = &self.0;
        let mut out = format!("# OWASP / CWE 深度安全審查\n\n- 報告：{}\n- 提案：{}\n- 時間：{}\n- 完整：{}\n- 檢查完成：{}；未驗證：{}；發現：{}\n\n{}\n", r.report_id, markdown_text(self.change_name()), markdown_text(&r.reviewed_at), r.complete, r.counts.checks_succeeded, r.counts.not_verified, r.findings.len(), r.disclaimer);
        let mut domains: BTreeMap<&str, Vec<&DeepFinding>> = BTreeMap::new();
        for finding in &r.findings {
            domains.entry(&finding.domain).or_default().push(finding);
        }
        for (domain, findings) in domains {
            out.push_str(&format!("\n## {}\n", markdown_text(domain)));
            for f in findings {
                out.push_str(&format!(
                    "\n### [{}]：{}\n\n- 等級：{}。信心：{}（{}）。\n",
                    f.finding_id,
                    markdown_text(&f.title),
                    f.severity.to_uppercase(),
                    markdown_text(&f.confidence),
                    paragraphs(&f.confidence_reason)
                ));
                out.push_str("- 對照：");
                if f.references.is_empty() {
                    out.push_str("無可信對照\n");
                } else {
                    out.push_str(
                        &f.references
                            .iter()
                            .map(|reference| {
                                format!(
                                    "{} {}；{}；{}",
                                    reference.standard_id.as_str(),
                                    markdown_text(&reference.version),
                                    markdown_text(&reference.control_id),
                                    markdown_text(&reference.local_path)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("、"),
                    );
                    out.push('\n');
                }
                for evidence in &f.evidence {
                    out.push_str(&format!(
                        "- 證據：[{}]({}#L{})\n",
                        markdown_text(&evidence.function),
                        link_path(&evidence.path),
                        evidence.line
                    ));
                }
                out.push_str(&format!(
                    "- 片段：{}\n\n```text\n{}\n```\n",
                    f.redaction_notice.as_deref().unwrap_or(""),
                    f.code_snippet.replace("```", "` ` `")
                ));
                for (label, text) in [
                    ("資料流", &f.data_flow),
                    ("影響", &f.impact),
                    ("修正", &f.remediation),
                    ("驗收", &f.acceptance),
                ] {
                    out.push_str(&format!("- {label}：{}\n", paragraphs(text)));
                }
            }
        }
        out.push_str("\n## 檢核紀錄\n");
        for check in &r.checks {
            out.push_str(&format!(
                "- {}：{}；{}\n",
                markdown_text(&check.check_id),
                status_label(check.status),
                paragraphs(&check.reason)
            ));
        }
        out.push_str("\n## 審查標準與來源\n");
        for standard in &r.standards {
            out.push_str(&format!(
                "- {} {}：{}；來源版本 {}；資料SHA {}\n",
                standard.standard_id.as_str(),
                markdown_text(standard.version.as_deref().unwrap_or("未指定")),
                status_label(standard.status),
                markdown_text(standard.source_revision.as_deref().unwrap_or("未驗證")),
                standard.data_sha256.as_deref().unwrap_or("未驗證")
            ));
        }
        out.push_str(&format!("\n- 規則版本：{}\n- 政策指紋：{}\n- 知識庫指紋：{}\n- 來源範圍指紋：{}\n- 覆蓋檔案：{}；位元組：{}；失敗項目：{}\n", markdown_text(&r.rule_version), r.policy_digest, r.knowledge_digest, r.scope_digest, r.coverage.inspected_files, r.coverage.bytes_read, r.coverage.failed_items));
        for license in &r.source_licenses {
            out.push_str(&format!(
                "- 授權：{}；{}；{}\n",
                markdown_text(&license.license_name),
                markdown_text(&license.attribution),
                markdown_text(&license.license_url)
            ));
        }
        bind_markdown(out, self)
    }
}
fn sensitive_literals(findings: &[DeepFinding]) -> Vec<String> {
    use regex::Regex;
    use std::sync::LazyLock;
    static QUOTES: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#""([^"\r\n]+)"|'([^'\r\n]+)'|`([^`\r\n]+)`"#).expect("code literal redaction")
    });
    static ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)(?:api[_-]?key|password|secret|(?:access|refresh)[_-]?token|authorization)["']?\s*[:=]\s*(?:"([^"\r\n]*)"|'([^'\r\n]*)'|([^\s,;}\]]+))"#).expect("shared secret values")
    });
    let mut values = BTreeSet::new();
    for finding in findings {
        for text in [
            &finding.code_snippet,
            &finding.title,
            &finding.confidence_reason,
            &finding.data_flow,
            &finding.impact,
            &finding.remediation,
            &finding.acceptance,
        ] {
            for captures in ASSIGNMENT.captures_iter(text) {
                if let Some(value) = captures.iter().skip(1).flatten().next() {
                    if !value.as_str().is_empty() && !value.as_str().starts_with("[REDACTED") {
                        values.insert(value.as_str().to_owned());
                    }
                }
            }
        }
        for captures in QUOTES.captures_iter(&finding.code_snippet) {
            if let Some(value) = captures.iter().skip(1).flatten().next() {
                if value.as_str().chars().count() >= 4 && !value.as_str().starts_with("[REDACTED") {
                    values.insert(value.as_str().to_owned());
                }
            }
        }
    }
    let mut values: Vec<_> = values.into_iter().collect();
    values.sort_by_key(|value| std::cmp::Reverse(value.len()));
    values
}
fn paragraphs(text: &str) -> String {
    text.lines()
        .map(markdown_text)
        .collect::<Vec<_>>()
        .join("\n  ")
}
fn link_path(path: &str) -> String {
    path.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum VersionedReport {
    Builtin(Report),
    Deep(DeepReport),
}
impl VersionedReport {
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("invalid_report: JSON"))?;
        match value
            .get("formatVersion")
            .and_then(serde_json::Value::as_u64)
        {
            Some(1) => Ok(Self::Builtin(Report::from_json(bytes)?)),
            Some(2) => Ok(Self::Deep(DeepReport::from_json(bytes)?)),
            _ => anyhow::bail!("unsupported_version: security report"),
        }
    }
    pub fn report_id(&self) -> &str {
        match self {
            Self::Builtin(r) => r.report_id(),
            Self::Deep(r) => r.report_id(),
        }
    }
    pub fn project_key(&self) -> &str {
        match self {
            Self::Builtin(r) => r.project_key(),
            Self::Deep(r) => r.project_key(),
        }
    }
    pub fn change_name(&self) -> Option<&str> {
        match self {
            Self::Builtin(r) => r.change_name(),
            Self::Deep(r) => Some(r.change_name()),
        }
    }
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Builtin(r) => r.is_complete(),
            Self::Deep(r) => r.is_complete(),
        }
    }
    pub fn reviewed_at(&self) -> &str {
        match self {
            Self::Builtin(r) => &r.0.reviewed_at,
            Self::Deep(r) => r.reviewed_at(),
        }
    }
    pub fn execution_mode(&self) -> &'static str {
        match self {
            Self::Builtin(_) => "builtin-offline",
            Self::Deep(_) => "deep-skill",
        }
    }
    pub fn json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    pub fn markdown(&self) -> String {
        match self {
            Self::Builtin(r) => bind_markdown(r.markdown(), r),
            Self::Deep(r) => r.markdown(),
        }
    }
}

fn bind_markdown(mut text: String, report: &impl Serialize) -> String {
    let json = serde_json::to_string_pretty(report).expect("serializable security report");
    text.push_str(&format!(
        "\n<!-- speclink-report-sha256: {} -->\n",
        sha256(json.as_bytes())
    ));
    text
}

pub fn save_change_report(
    store: &dyn crate::store::Store,
    change: &str,
    report: &VersionedReport,
) -> Result<ReportReceipt> {
    save_change_report_kind(store, change, report, false)
}
pub fn save_legacy_change_report(
    store: &dyn crate::store::Store,
    change: &str,
    report: &Report,
) -> Result<ReportReceipt> {
    save_change_report_kind(
        store,
        change,
        &VersionedReport::Builtin(report.clone()),
        true,
    )
}
fn save_change_report_kind(
    store: &dyn crate::store::Store,
    change: &str,
    report: &VersionedReport,
    legacy: bool,
) -> Result<ReportReceipt> {
    ensure!(
        store.change_exists(change) && report.change_name() == Some(change),
        "invalid_report: change binding"
    );
    store.recover_checks(change)?;
    let old_index = store.read_artifact(change, crate::history::INDEX);
    let mut index = crate::history::index(store, change)?;
    ensure!(
        index
            .entries
            .iter()
            .filter(|entry| entry.kind == crate::history::CheckKind::Security)
            .count()
            < 1000,
        "storage_limit"
    );
    ensure!(
        !index
            .entries
            .iter()
            .any(|entry| entry.record_id == report.report_id()),
        "duplicate_report_id"
    );
    let prefix = format!("checks/security/{}", report.report_id());
    let json = report.json()?;
    let markdown = report.markdown();
    let mut bytes = json.len() + markdown.len();
    for entry in index
        .entries
        .iter()
        .filter(|entry| entry.kind == crate::history::CheckKind::Security)
    {
        bytes += store
            .read_artifact(
                change,
                &entry.relative_path.replace("report.md", "decisions.jsonl"),
            )
            .map(|s| s.len())
            .unwrap_or(0);
        bytes += store
            .read_artifact(change, &entry.relative_path)
            .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?
            .len();
        bytes += store
            .read_artifact(
                change,
                &entry.relative_path.replace("report.md", "report.json"),
            )
            .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?
            .len();
    }
    ensure!(bytes <= 200 * 1024 * 1024, "storage_limit");
    index.append(crate::history::CheckEntry {
        record_id: report.report_id().into(),
        kind: crate::history::CheckKind::Security,
        created_at: report.reviewed_at().into(),
        outcome: if legacy {
            "legacy"
        } else if report.is_complete() {
            "complete"
        } else {
            "incomplete"
        }
        .into(),
        relative_path: format!("{prefix}/report.md"),
        content_digest: sha256(markdown.as_bytes()),
    })?;
    let writes = BTreeMap::from([
        (format!("{prefix}/report.json"), json),
        (format!("{prefix}/report.md"), markdown),
        (
            crate::history::INDEX.into(),
            serde_json::to_string_pretty(&index)?,
        ),
    ]);
    let mut expected: BTreeMap<_, _> = writes.keys().map(|path| (path.clone(), None)).collect();
    expected.insert(
        crate::history::INDEX.into(),
        old_index.map(|text| sha256(text.as_bytes())),
    );
    store.commit_checks(
        change,
        &crate::history::ChecksCommit {
            writes,
            deletes: vec![],
            expected,
        },
    )?;
    Ok(ReportReceipt {
        report_id: report.report_id().into(),
        project_key: report.project_key().into(),
        warnings: vec![],
    })
}
pub fn read_change_report(
    store: &dyn crate::store::Store,
    change: &str,
    id: &str,
) -> Result<Option<VersionedReport>> {
    crate::history::validate_id(id)?;
    let index = crate::history::index(store, change)?;
    let Some(entry) = index
        .entries
        .iter()
        .find(|entry| entry.kind == crate::history::CheckKind::Security && entry.record_id == id && entry.outcome != "omitted")
    else {
        return Ok(None);
    };
    let load = |path: &str| {
        if store.change_exists(change) {
            store.read_artifact(change, path)
        } else {
            store.read_archived_artifact(change, path)
        }
    };
    let json = load(&entry.relative_path.replace("report.md", "report.json"))
        .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
    let markdown = load(&entry.relative_path).unwrap_or_default();
    Ok(Some(validate_report_pair(id, &json, &markdown)?))
}
pub fn validate_report_pair(id: &str, json: &str, markdown: &str) -> Result<VersionedReport> {
    crate::history::validate_id(id)?;
    let report = VersionedReport::from_json(json.as_bytes())?;
    ensure!(
        report.report_id() == id
            && report.markdown() == markdown
            && markdown.ends_with(&format!(
                "<!-- speclink-report-sha256: {} -->\n",
                sha256(json.as_bytes())
            )),
        "integrity_error: report representations"
    );
    Ok(report)
}

/// Keep decisions separate so the original report and its index entry remain immutable.
pub fn prepare_decision(
    store: &dyn crate::store::Store,
    change: &str,
    decision: &super::super::acceptance::RiskDecision,
) -> Result<crate::history::ChecksCommit> {
    decision.validate()?;
    ensure!(store.change_exists(change), "unknown_change");
    store.recover_checks(change)?;
    let old_index = store.read_artifact(change, crate::history::INDEX);
    let index = crate::history::index(store, change)?;
    ensure!(
        index
            .entries
            .iter()
            .any(|e| e.kind == crate::history::CheckKind::Security
                && e.record_id == decision.report_id
                && e.outcome != "legacy"),
        "invalid_security_decision: report"
    );
    let report = read_change_report(store, change, &decision.report_id)?
        .ok_or_else(|| anyhow::anyhow!("report_not_found"))?;
    let exists = match &report {
        VersionedReport::Builtin(r) => r
            .findings()
            .iter()
            .any(|f| f.finding_id == decision.finding_id),
        VersionedReport::Deep(r) => r
            .findings()
            .iter()
            .any(|f| f.finding_id == decision.finding_id),
    };
    ensure!(exists, "finding_not_found");
    let mut decision = decision.clone();
    decision.reason = redact_text(decision.reason.trim());
    decision.actor = redact_text(&decision.actor);
    decision.validate()?;
    let path = format!("checks/security/{}/decisions.jsonl", decision.report_id);
    let old = store.read_artifact(change, &path);
    for line in old
        .as_deref()
        .unwrap_or("")
        .lines()
        .filter(|line| !line.trim().is_empty())
    {
        let record: super::super::acceptance::RiskDecision =
            serde_json::from_str(line).map_err(|_| anyhow::anyhow!("invalid_security_decision"))?;
        record.validate()?;
        ensure!(
            record.report_id == decision.report_id,
            "invalid_security_decision: report"
        );
    }
    let mut text = old.clone().unwrap_or_default();
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&serde_json::to_string(&decision)?);
    text.push('\n');
    ensure!(
        text.len() <= 16 * 1024 * 1024,
        "storage_limit: security decisions"
    );
    let mut bytes = text.len();
    for entry in index
        .entries
        .iter()
        .filter(|e| e.kind == crate::history::CheckKind::Security)
    {
        for file in ["report.json", "report.md", "decisions.jsonl"] {
            let file_path = format!("checks/security/{}/{file}", entry.record_id);
            if file_path != path {
                bytes += store
                    .read_artifact(change, &file_path)
                    .map(|s| s.len())
                    .unwrap_or(0);
            }
        }
    }
    ensure!(bytes <= 200 * 1024 * 1024, "storage_limit");
    let report_path = format!("checks/security/{}/report.json", decision.report_id);
    Ok(crate::history::ChecksCommit {
        writes: BTreeMap::from([(path.clone(), text)]),
        deletes: vec![],
        expected: BTreeMap::from([
            (path, old.map(|s| sha256(s.as_bytes()))),
            (
                crate::history::INDEX.into(),
                old_index.map(|s| sha256(s.as_bytes())),
            ),
            (
                report_path.clone(),
                store
                    .read_artifact(change, &report_path)
                    .map(|s| sha256(s.as_bytes())),
            ),
        ]),
    })
}

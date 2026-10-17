//! SpecLink-owned heuristics. Findings require human confirmation.
use super::{
    input::{Language, SourceDocument},
    ReviewStatus,
};
use anyhow::{ensure, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleSet {
    pub rule_version: String,
    pub rules: Vec<Rule>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub severity: String,
    pub languages: Vec<Language>,
    pub description: String,
    pub references: RuleReferences,
    pub check: RuleCheck,
    pub remediation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleReferences {
    pub cwe: Vec<String>,
    pub sources: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleCheck {
    pub mode: String,
    pub selector: Selector,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Selector {
    HardcodedSecret,
    ErrorDisclosure,
    SqlInterpolation,
    ShellInterpolation,
    HtmlOutput,
    UnsafeConfig,
    SpecAuthentication,
    SpecAuthorization,
    SpecInput,
    SpecSensitiveData,
    SpecErrors,
    SpecAcceptance,
    BoundaryNull,
    BoundaryType,
    BoundaryLength,
    BoundaryNumber,
    BoundaryInvalidInput,
    BoundaryAuthorization,
    BoundaryErrors,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardReference {
    pub standard_id: super::standards::StandardId,
    pub version: String,
    pub control_id: String,
    pub source_url: String,
    pub source_revision: String,
    pub section: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
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
    pub references: Vec<StandardReference>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleResult {
    pub rule_id: String,
    pub path: String,
    pub status: ReviewStatus,
    pub reason: Option<String>,
    pub findings: Vec<Finding>,
}
impl RuleSet {
    pub fn builtin() -> Result<Self> {
        let catalog: Self = serde_json::from_str(include_str!("../../assets/security/rules.json"))?;
        let mut ids = BTreeSet::new();
        ensure!(
            !catalog.rule_version.is_empty() && !catalog.rules.is_empty(),
            "invalid_rules"
        );
        for rule in &catalog.rules {
            let expected_type = match rule.check.selector {
                Selector::HardcodedSecret
                | Selector::ErrorDisclosure
                | Selector::SqlInterpolation
                | Selector::ShellInterpolation
                | Selector::HtmlOutput
                | Selector::UnsafeConfig => "code-review",
                Selector::SpecAuthentication
                | Selector::SpecAuthorization
                | Selector::SpecInput
                | Selector::SpecSensitiveData
                | Selector::SpecErrors
                | Selector::SpecAcceptance => "spec-review",
                _ => "api-boundary",
            };
            static ID: OnceLock<Regex> = OnceLock::new();
            ensure!(
                ids.insert(&rule.id)
                    && ID
                        .get_or_init(|| Regex::new(r"^SL-(?:SEC|SPEC|API)-[0-9]{3}$")
                            .expect("rule ID grammar"))
                        .is_match(&rule.id)
                    && rule.rule_type == expected_type
                    && !rule.title.is_empty()
                    && !rule.description.is_empty()
                    && !rule.remediation.is_empty()
                    && !rule.languages.is_empty()
                    && !rule.languages.contains(&Language::Unsupported)
                    && matches!(rule.severity.as_str(), "low" | "medium" | "high")
                    && matches!(
                        rule.rule_type.as_str(),
                        "code-review" | "spec-review" | "api-boundary"
                    )
                    && rule.check.mode == "pattern",
                "invalid_rules: incomplete or duplicate rule metadata"
            );
            ensure!(
                rule.references.cwe.iter().all(|id| id
                    .strip_prefix("CWE-")
                    .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())))
                    && rule.references.cwe.len() == rule.references.sources.len()
                    && rule
                        .references
                        .cwe
                        .iter()
                        .all(|id| rule.references.sources.contains(&format!(
                            "https://cwe.mitre.org/data/definitions/{}.html",
                            id.trim_start_matches("CWE-")
                        ))),
                "invalid_rules: untrusted reference"
            );
        }
        Ok(catalog)
    }
    pub fn review_code(
        &self,
        document: &SourceDocument,
        disabled: &[String],
        stop: &dyn Fn() -> Option<&'static str>,
    ) -> Result<Vec<RuleResult>> {
        let mut results = Vec::new();
        let mut total = 0;
        let lines = document.text.lines().collect::<Vec<_>>();
        for rule in self.rules.iter().filter(|r| r.rule_type == "code-review") {
            if let Some(reason) = stop() {
                anyhow::bail!(reason);
            }
            let mut result = RuleResult {
                rule_id: rule.id.clone(),
                path: document.path.clone(),
                status: ReviewStatus::Pass,
                reason: None,
                findings: Vec::new(),
            };
            if disabled.contains(&rule.id) {
                result.status = ReviewStatus::Disabled;
            } else if document.language == Language::Unsupported {
                result.status = ReviewStatus::NotVerified;
                result.reason = Some("unsupported_language".into());
            } else if !rule.languages.contains(&document.language) {
                result.status = ReviewStatus::NotApplicable;
                result.reason = Some("language_not_applicable".into());
            } else {
                for (index, line) in lines.iter().enumerate() {
                    if let Some(reason) = stop() {
                        anyhow::bail!(reason);
                    }
                    if line.len() > 8192 {
                        result.status = ReviewStatus::NotVerified;
                        result.reason = Some("line_limit".into());
                        continue;
                    }
                    let matched = match rule.check.selector {
                        Selector::HardcodedSecret => secret_pattern()
                            .captures_iter(line)
                            .any(|c| !placeholder(&c[1])),
                        Selector::ErrorDisclosure => error_disclosure(line, document.language),
                        Selector::SqlInterpolation => {
                            sql_interpolation(line, &lines, index, document.language)
                        }
                        Selector::ShellInterpolation => {
                            shell_interpolation(line, &lines, index, document.language)
                        }
                        Selector::HtmlOutput => html_output(line, &lines, index, document.language),
                        Selector::UnsafeConfig => unsafe_config(line),
                        _ => anyhow::bail!("invalid_rules: selector is not a code check"),
                    };
                    if matched {
                        if total >= 100 {
                            result.status = ReviewStatus::NotVerified;
                            result.reason = Some("finding_limit".into());
                            break;
                        }
                        total += 1;
                        let mut risk = finding(
                            rule,
                            document,
                            index + 1,
                            if matches!(rule.check.selector, Selector::HardcodedSecret) {
                                "[REDACTED: potential credential assignment]".into()
                            } else {
                                redact_evidence(line)
                            },
                        );
                        if matches!(rule.check.selector, Selector::UnsafeConfig) {
                            risk.cwe_ids = if line.contains("httpOnly") {
                                vec!["CWE-1004".into()]
                            } else if line.contains("rejectUnauthorized")
                                || line.contains("verify_ssl")
                            {
                                vec!["CWE-295".into()]
                            } else {
                                Vec::new()
                            };
                        }
                        result.findings.push(risk);
                    }
                }
                if !result.findings.is_empty() && result.status != ReviewStatus::NotVerified {
                    result.status = ReviewStatus::Finding;
                }
            }
            if let Some(reason) = stop() {
                anyhow::bail!(reason);
            }
            results.push(result);
        }
        Ok(results)
    }
}

impl RuleSet {
    pub fn review_specs(
        &self,
        documents: &[SourceDocument],
        disabled: &[String],
        stop: &dyn Fn() -> Option<&'static str>,
    ) -> Result<Vec<RuleResult>> {
        ensure!(
            documents.len() <= 10_000
                && documents.iter().map(|d| d.text.len()).sum::<usize>() <= 104_857_600,
            "size_limit: specification input"
        );
        let mut text = String::new();
        for document in documents {
            if let Some(reason) = stop() {
                anyhow::bail!(reason);
            }
            text.push_str(&document.text.to_lowercase());
            text.push('\n');
        }
        let absent = documents.is_empty();
        let explicit_no_target = (text.contains("no network services")
            && (text.contains("no sensitive operations")
                || text.contains("or sensitive operations")))
            || (text.contains("不提供網路服務") && text.contains("沒有敏感操作"))
            || (text.contains("ネットワーク機能なし") && text.contains("機密操作なし"));
        static TARGET: OnceLock<Regex> = OnceLock::new();
        let target=TARGET.get_or_init(||Regex::new(r"(?i)\b(?:api|endpoint|graphql|administrator)\b|刪除帳號|敏感功能|管理者|エンドポイント|アカウントを削除").expect("SDD target phrases")).is_match(&text);
        let mentions = |words: &[&str]| words.iter().any(|word| text.contains(word));
        let mut results = Vec::new();
        for rule in self.rules.iter().filter(|r| r.rule_type == "spec-review") {
            if let Some(reason) = stop() {
                anyhow::bail!(reason);
            }
            let mut result = RuleResult {
                rule_id: rule.id.clone(),
                path: documents
                    .first()
                    .map(|d| d.path.clone())
                    .unwrap_or_default(),
                status: ReviewStatus::Pass,
                reason: Some("requirement_mentioned_only".into()),
                findings: Vec::new(),
            };
            if disabled.contains(&rule.id) {
                result.status = ReviewStatus::Disabled;
                result.reason = None;
            } else if absent {
                result.status = ReviewStatus::NotVerified;
                result.reason = Some("missing_artifacts".into());
            } else if explicit_no_target && !target {
                result.status = ReviewStatus::NotApplicable;
                result.reason =
                    Some("explicit_no_network_or_sensitive_operation_in_specification".into());
            } else if !target {
                result.status = ReviewStatus::NotVerified;
                result.reason = Some("target_information_insufficient".into());
            } else {
                let described = match rule.check.selector {
                    Selector::SpecAuthentication => {
                        mentions(&["authentication", "身分驗證", "身份驗證", "認証"])
                    }
                    Selector::SpecAuthorization => {
                        mentions(&["authorization", "authorisation", "授權", "認可"])
                    }
                    Selector::SpecInput => {
                        mentions(&["input type", "parameters define type", "型別", "入力の型"])
                            && mentions(&["length", "長度", "最大長"])
                    }
                    Selector::SpecSensitiveData => {
                        mentions(&["sensitive data", "secret", "敏感資料", "機密情報"])
                    }
                    Selector::SpecErrors => {
                        mentions(&["error handling", "public error", "錯誤處理", "エラー処理"])
                    }
                    Selector::SpecAcceptance => {
                        mentions(&["security acceptance", "安全驗收", "セキュリティ受入"])
                    }
                    _ => anyhow::bail!("invalid_rules: selector is not a specification check"),
                };
                if !described {
                    result.status = ReviewStatus::Finding;
                    result.reason = Some("missing_requirement".into());
                    let document = documents
                        .iter()
                        .find(|d| TARGET.get().unwrap().is_match(&d.text))
                        .unwrap_or(&documents[0]);
                    let line = document
                        .text
                        .lines()
                        .position(|line| TARGET.get().unwrap().is_match(line))
                        .unwrap_or(0)
                        + 1;
                    let mut gap=finding(rule,document,line,"[REDACTED: specification target; security requirement not found in selected artifacts]".into());
                    gap.kind = "missing_requirement".into();
                    result.path = document.path.clone();
                    result.findings.push(gap);
                }
            }
            results.push(result);
        }
        Ok(results)
    }
}
pub(super) fn finding(
    rule: &Rule,
    document: &SourceDocument,
    line: usize,
    evidence: String,
) -> Finding {
    Finding {
        cwe_ids: rule.references.cwe.clone(),
        rule_id: rule.id.clone(),
        title: rule.title.clone(),
        severity: rule.severity.clone(),
        confidence: "low".into(),
        kind: "potential_risk".into(),
        confirmation_status: "needs_review".into(),
        path: document.path.clone(),
        line,
        evidence,
        description: rule.description.clone(),
        remediation: rule.remediation.clone(),
        references: Vec::new(),
    }
}
fn secret_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(||Regex::new(r#"(?i)\b(?:password|passwd|secret|token|api_?key|access_?token|client_?secret)\b\s*(?::\s*(?:string|&str|String))?\s*(?:=|:)\s*(?:r\#*)?["'`]([^"'`\r\n]*)["'`]"#).expect("built-in secret pattern"))
}
fn placeholder(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value.is_empty()
        || value.contains("${")
        || value.contains("example")
        || value.contains("redacted")
        || value.contains("placeholder")
        || value.contains("changeme")
        || value.starts_with("your_")
        || value.starts_with("your-")
        || value == "test"
        || value == "dummy"
}
fn error_disclosure(line: &str, language: Language) -> bool {
    match language {
        Language::Rust => {
            (line.contains("HttpResponse::")
                || line.contains("Response::")
                || line.contains("Json("))
                && (line.contains(".body(") || line.contains("Json("))
                && (line.contains("{:?}") || line.contains("{:#?}"))
                && (line.contains("error") || line.contains("err"))
        }
        Language::TypeScript | Language::JavaScript => {
            static RE: OnceLock<Regex> = OnceLock::new();
            let re=RE.get_or_init(||Regex::new(r"\b(?:res|response|reply)(?:\.status\([^)]*\)|\.code\([^)]*\))?\.(?:send|json|end)\s*\([^;]*(?:\b(?:error|err)\.(?:stack|message)\b|\b(?:error|err)\s*[,)}])").expect("built-in error pattern"));
            re.is_match(line)
        }
        _ => false,
    }
}
fn bounded_window(lines: &[&str], start: usize) -> String {
    let mut window = String::new();
    for line in lines.iter().skip(start).take(20) {
        if window.len() + line.len() + 1 > 8192 {
            break;
        }
        window.push_str(line);
        window.push('\n');
    }
    window
}
fn sql_interpolation(line: &str, lines: &[&str], index: usize, language: Language) -> bool {
    static SQL: OnceLock<Regex> = OnceLock::new();
    static SINK: OnceLock<Regex> = OnceLock::new();
    let sql = SQL.get_or_init(|| {
        Regex::new(r"(?i)\b(?:SELECT|INSERT|UPDATE|DELETE)\b").expect("SQL keywords")
    });
    if !sql.is_match(line) {
        return false;
    }
    let window = bounded_window(lines, index);
    if !SINK
        .get_or_init(|| Regex::new(r"\b(?:query|execute)\s*\(").expect("SQL sink"))
        .is_match(&window)
    {
        return false;
    }
    match language {
        Language::Rust => {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(||Regex::new(r#"(?is)(?:format!\s*\(\s*(?:r\#*)?"[^"\r\n]*\b(?:SELECT|INSERT|UPDATE|DELETE)\b[^"\r\n]*\{[^"\r\n]*"|"[^"\r\n]*\b(?:SELECT|INSERT|UPDATE|DELETE)\b[^"\r\n]*"\s*(?:\.to_(?:owned|string)\(\))?\s*\+\s*&?\w)"#).expect("Rust SQL interpolation")).is_match(&window)
        }
        Language::TypeScript | Language::JavaScript => {
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(||Regex::new(r#"(?is)(?:`[^`]*\b(?:SELECT|INSERT|UPDATE|DELETE)\b[^`]*\$\{|["'][^"'\r\n]*\b(?:SELECT|INSERT|UPDATE|DELETE)\b[^"'\r\n]*["']\s*\+\s*[\w(])"#).expect("JS SQL interpolation")).is_match(&window)
        }
        _ => false,
    }
}
fn shell_interpolation(line: &str, lines: &[&str], index: usize, language: Language) -> bool {
    match language {
        Language::Rust => {
            if !line.contains("Command::new") {
                return false;
            }
            let window = bounded_window(lines, index);
            static SHELL: OnceLock<Regex> = OnceLock::new();
            static FLAG: OnceLock<Regex> = OnceLock::new();
            static VALUE: OnceLock<Regex> = OnceLock::new();
            SHELL.get_or_init(||Regex::new(r#"Command::new\s*\(\s*"(?:sh|bash|cmd(?:\.exe)?)"\s*\)"#).expect("shell constructor")).is_match(&window)
                && FLAG.get_or_init(||Regex::new(r#"\.arg\s*\(\s*"(?:-c|/C)"\s*\)"#).expect("shell flag")).is_match(&window)
                && VALUE.get_or_init(||Regex::new(r#"(?s)\.arg\s*\(\s*(?:format!\s*\(\s*"[^"]*\{|"[^"]*"\s*(?:\.to_(?:owned|string)\(\))?\s*\+)"#).expect("shell interpolation")).is_match(&window)
        }
        Language::TypeScript | Language::JavaScript => {
            if !line.contains("exec") {
                return false;
            }
            static RE: OnceLock<Regex> = OnceLock::new();
            RE.get_or_init(|| {
                Regex::new(r#"(?s)\bexec(?:Sync)?\s*\(\s*(?:`[^`]*\$\{|["'][^"']*["']\s*\+)"#)
                    .expect("JS shell interpolation")
            })
            .is_match(&bounded_window(lines, index))
        }
        _ => false,
    }
}
fn html_output(line: &str, lines: &[&str], index: usize, language: Language) -> bool {
    match language {
        Language::Rust => {
            if !line.contains("Html(") && !line.contains("content_type(") {
                return false;
            }
            let window = bounded_window(lines, index);
            static RE: OnceLock<Regex> = OnceLock::new();
            let formatted = RE.get_or_init(|| {
                Regex::new(r#"format!\s*\(\s*"([^"]*)""#).expect("HTML format literal")
            });
            (window.contains("Html(") || window.contains("text/html"))
                && formatted
                    .captures_iter(&window)
                    .any(|c| c[1].replace("{{", "").replace("}}", "").contains('{'))
        }
        Language::TypeScript | Language::JavaScript => {
            if !line.contains("innerHTML")
                && !line.contains("outerHTML")
                && !line.contains("document.write")
                && !line.contains("dangerouslySetInnerHTML")
            {
                return false;
            }
            static SINK: OnceLock<Regex> = OnceLock::new();
            static LITERAL: OnceLock<Regex> = OnceLock::new();
            let sink=SINK.get_or_init(||Regex::new(r"(?:\.(?:innerHTML|outerHTML)\s*=|document\.write\s*\(|\b__html\s*:)\s*([^;\r\n]*)").expect("HTML sink"));
            let literal = LITERAL.get_or_init(|| {
                Regex::new(r#"^\s*(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`[^`$]*`)\s*[);,}\s]*$"#)
                    .expect("static HTML literal")
            });
            sink.captures_iter(&bounded_window(lines, index))
                .any(|c| !c[1].trim().is_empty() && !literal.is_match(&c[1]))
        }
        _ => false,
    }
}
fn unsafe_config(line: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(||Regex::new(r#"\b(?:(?:rejectUnauthorized|verify_ssl|httpOnly)["']?\s*[:=]\s*false\b|insecure["']?\s*[:=]\s*true\b)"#).expect("unsafe configuration literals")).is_match(line)
}
pub fn redact_evidence(line: &str) -> String {
    // A heuristic cannot safely parse every language's quoting rules. Retain
    // operator/identifier context only when every quote can be bounded.
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#""(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`"#)
            .expect("built-in evidence redaction")
    });
    let redacted = re.replace_all(line, "[REDACTED]");
    if redacted.contains(['"', '\'', '`'])
        || redacted.contains("http://")
        || redacted.contains("https://")
        || redacted.contains("//")
        || redacted.contains("/*")
    {
        return "[REDACTED]".into();
    }
    static TOKEN: OnceLock<Regex> = OnceLock::new();
    let redacted = TOKEN
        .get_or_init(|| {
            Regex::new(r"[A-Za-z0-9_+/=-]{24,}").expect("token-like evidence redaction")
        })
        .replace_all(&redacted, "[REDACTED]");
    redacted
        .chars()
        .filter(|c| !c.is_control())
        .take(400)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::input::SourceDocument;
    fn code(path: &str, text: &str, disabled: &[String]) -> Vec<RuleResult> {
        RuleSet::builtin()
            .unwrap()
            .review_code(
                &SourceDocument::new(path.into(), text.into()),
                disabled,
                &|| None,
            )
            .unwrap()
    }
    fn found(results: &[RuleResult], id: &str) -> bool {
        results
            .iter()
            .any(|r| r.rule_id == id && r.status == crate::security::ReviewStatus::Finding)
    }
    #[test]
    fn security_rules_secret_error_language_matrix_and_redaction() {
        for path in ["main.rs", "app.ts", "app.js"] {
            let secret = "unique-real-credential-sentinel-9911";
            let results = code(path, &format!("const api_key = \"{secret}\";"), &[]);
            assert!(found(&results, "SL-SEC-001"), "{path}");
            let finding = &results
                .iter()
                .find(|r| r.rule_id == "SL-SEC-001")
                .unwrap()
                .findings[0];
            assert_eq!(finding.confirmation_status, "needs_review");
            assert_eq!(finding.confidence, "low");
            assert_eq!(finding.line, 1);
            assert!(!serde_json::to_string(&results).unwrap().contains(secret));
            for safe in [
                "const token = process.env.API_TOKEN;",
                "let token = std::env::var(\"TOKEN\");",
                "const secret = \"EXAMPLE_TOKEN\";",
                "const password = \"changeme\";",
                "const api_key = \"REDACTED\";",
                "const token = \"\";",
            ] {
                assert!(
                    !found(&code(path, safe, &[]), "SL-SEC-001"),
                    "{path}: {safe}"
                );
            }
            let disabled = code(
                path,
                "const token = \"real-secret-value\";",
                &["SL-SEC-001".into()],
            );
            assert_eq!(
                disabled
                    .iter()
                    .find(|r| r.rule_id == "SL-SEC-001")
                    .unwrap()
                    .status,
                crate::security::ReviewStatus::Disabled
            );
        }
        for (path, unsafe_source, safe_source) in [
            (
                "main.rs",
                r#"HttpResponse::InternalServerError().body(format!("{:?}", error));"#,
                r#"tracing::error!("{:?}", error);"#,
            ),
            (
                "app.ts",
                "res.status(500).json({error: error.stack});",
                "console.error(error.stack);",
            ),
            (
                "app.js",
                "res.send(error);",
                "res.status(500).json({error: 'Request failed'});",
            ),
        ] {
            assert!(
                found(&code(path, unsafe_source, &[]), "SL-SEC-005"),
                "{path}"
            );
            assert!(
                !found(&code(path, safe_source, &[]), "SL-SEC-005"),
                "{path}"
            );
        }
        let unsupported = code("main.py", "password = 'secret-value'", &[]);
        assert!(unsupported
            .iter()
            .all(|r| r.status == crate::security::ReviewStatus::NotVerified
                && r.reason.as_deref() == Some("unsupported_language")));
        assert!(RuleSet::builtin()
            .unwrap()
            .review_code(
                &SourceDocument::new("main.rs".into(), "fn main(){}".into()),
                &[],
                &|| Some("cancelled")
            )
            .is_err());
    }

    #[test]
    fn security_rules_injection_sql_and_shell_language_matrix() {
        for (path, sql, safe_sql, shell, safe_shell) in [
            (
                "main.rs",
                r#"let sql = format!("SELECT * FROM users WHERE id = {}", user_id); db.query(&sql);"#,
                r#"db.query("SELECT * FROM users WHERE id = $1", &[&user_id]);"#,
                r#"Command::new("sh").arg("-c").arg(format!("echo {}", input)).output();"#,
                r#"Command::new("echo").arg(input).output();"#,
            ),
            (
                "app.ts",
                "db.query(`SELECT * FROM users WHERE id = ${userId}`);",
                "db.query('SELECT * FROM users WHERE id = ?', [userId]);",
                "exec(`convert ${input}`);",
                "execFile('convert', [input]);",
            ),
            (
                "app.js",
                r#"db.query("SELECT * FROM users WHERE id = " + userId);"#,
                r#"db.query("SELECT * FROM users WHERE id = ?", [userId]);"#,
                r#"exec('convert ' + input);"#,
                r#"execFile('convert', [input]);"#,
            ),
        ] {
            assert!(found(&code(path, sql, &[]), "SL-SEC-002"), "{path}: {sql}");
            assert!(!found(&code(path, safe_sql, &[]), "SL-SEC-002"), "{path}");
            assert!(
                found(&code(path, shell, &[]), "SL-SEC-004"),
                "{path}: {shell}"
            );
            assert!(!found(&code(path, safe_shell, &[]), "SL-SEC-004"), "{path}");
            assert!(!found(
                &code(
                    path,
                    "const s = 'SELECT documentation only'; console.log(s);",
                    &[]
                ),
                "SL-SEC-002"
            ));
            assert!(!found(&code(path, "exec('date');", &[]), "SL-SEC-004"));
            assert!(!found(
                &code(
                    path,
                    "db.query('SELECT * FROM users', []);\nconst message = 'hello ' + name;",
                    &[]
                ),
                "SL-SEC-002"
            ));
        }
        let source = format!(
            "const sql = `SELECT ${{userId}}`;\n{}\ndb.query(sql);",
            "// padding\n".repeat(20)
        );
        assert!(!found(&code("app.ts", &source, &[]), "SL-SEC-002"));
        assert!(code("main.py", "db.query('SELECT ' + input)", &[])
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
    }

    #[test]
    fn security_rules_html_config_dynamic_output_and_literal_settings() {
        for path in ["app.ts", "app.js"] {
            for dangerous in [
                "node.innerHTML = userHtml;",
                "node.outerHTML = userHtml;",
                "document.write(userHtml);",
                "const view = {dangerouslySetInnerHTML: {__html: userHtml}};",
            ] {
                assert!(
                    found(&code(path, dangerous, &[]), "SL-SEC-003"),
                    "{dangerous}"
                );
            }
            for safe in [
                "node.textContent = userHtml;",
                "node.innerHTML = '<b>fixed</b>';",
                "document.write('<b>fixed</b>');",
                "const text = 'innerHTML';",
            ] {
                assert!(!found(&code(path, safe, &[]), "SL-SEC-003"), "{safe}");
            }
        }
        assert!(found(
            &code("main.rs", r#"Html(format!("<div>{}</div>", input))"#, &[]),
            "SL-SEC-003"
        ));
        assert!(!found(
            &code("main.rs", r#"Html("<div>fixed</div>")"#, &[]),
            "SL-SEC-003"
        ));
        for path in ["main.rs", "app.ts", "app.js", "config.json", "config.yaml"] {
            for dangerous in [
                "rejectUnauthorized: false",
                "verify_ssl = false",
                "insecure: true",
                "httpOnly: false",
                r#"{"rejectUnauthorized": false}"#,
            ] {
                assert!(
                    found(&code(path, dangerous, &[]), "SL-SEC-006"),
                    "{path}: {dangerous}"
                );
            }
            for safe in [
                "rejectUnauthorized: true",
                "verify_ssl = true",
                "insecure: false",
                "httpOnly: true",
                "allow_insecure_name: true",
                "rejectUnauthorized: configuredValue",
            ] {
                assert!(
                    !found(&code(path, safe, &[]), "SL-SEC-006"),
                    "{path}: {safe}"
                );
            }
        }
        assert!(code("script.py", "insecure=True", &[])
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
        let results = code(
            "app.ts",
            "node.innerHTML=user;\nhttpOnly:false",
            &["SL-SEC-003".into()],
        );
        assert!(found(&results, "SL-SEC-006"));
        assert_eq!(
            results
                .iter()
                .find(|r| r.rule_id == "SL-SEC-003")
                .unwrap()
                .status,
            ReviewStatus::Disabled
        );
        let rules = RuleSet::builtin().unwrap();
        assert_eq!(
            rules
                .rules
                .iter()
                .filter(|r| r.rule_type == "code-review")
                .count(),
            6
        );
        assert_eq!(
            rules
                .rules
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len(),
            rules.rules.len()
        );
    }

    #[test]
    fn security_spec_missing_requirements_are_not_confirmed_vulnerabilities() {
        let rules = RuleSet::builtin().unwrap();
        for description in [
            "API endpoint deletes accounts. Authentication uses bearer tokens. Authorization checks the owner. Input parameters define type and maximum length. Secrets and sensitive data are redacted. Public error handling hides details. Security acceptance criteria cover these controls.",
            "API 端點刪除帳號。身分驗證採 Token。授權檢查擁有者。輸入參數定義型別及長度限制。敏感資料遮罩。安全錯誤處理隱藏細節。定義安全驗收條件。",
            "API エンドポイントでアカウントを削除。認証を実施。認可で所有者を確認。入力の型と最大長を定義。機密情報はマスク。エラー処理で詳細を隠す。セキュリティ受入条件を定義。",
        ] {
            let docs=vec![SourceDocument::new("design.md".into(),description.into())];
            let result=rules.review_specs(&docs,&[],&||None).unwrap();
            assert_eq!(result.len(),6);
            assert!(result.iter().all(|r|r.status==ReviewStatus::Pass),"{description}");
        }
        let docs = vec![SourceDocument::new(
            "proposal.md".into(),
            "API endpoint deletes accounts".into(),
        )];
        let missing = rules.review_specs(&docs, &[], &|| None).unwrap();
        assert_eq!(missing.len(), 6);
        assert!(missing.iter().all(|r| r.status == ReviewStatus::Finding));
        assert!(missing
            .iter()
            .flat_map(|r| &r.findings)
            .all(|f| f.kind == "missing_requirement" && f.confirmation_status == "needs_review"));
        assert!(rules
            .review_specs(&[], &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
        let unknown = vec![SourceDocument::new(
            "tasks.md".into(),
            "Render pictures".into(),
        )];
        assert!(rules
            .review_specs(&unknown, &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
        let no_target = vec![SourceDocument::new(
            "design.md".into(),
            "Offline image viewer; no network services or sensitive operations.".into(),
        )];
        assert!(rules
            .review_specs(&no_target, &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.status == ReviewStatus::NotApplicable && r.reason.is_some()));
        let disabled = rules
            .review_specs(&docs, &["SL-SPEC-001".into()], &|| None)
            .unwrap();
        assert_eq!(
            disabled
                .iter()
                .find(|r| r.rule_id == "SL-SPEC-001")
                .unwrap()
                .status,
            ReviewStatus::Disabled
        );
    }

    #[test]
    fn security_boundary_openapi_30_31_internal_refs_and_seven_requirements() {
        let rules = RuleSet::builtin().unwrap();
        for version in ["3.0.3", "3.1.0"] {
            let document = serde_json::json!({
                "openapi":version,"security":[{"bearer":[]}],
                "components":{
                    "securitySchemes":{"bearer":{"type":"http","scheme":"bearer"}},
                    "schemas":{"Name":{"type":"string","nullable":false,"minLength":1,"maxLength":32}}
                },
                "paths":{"/users/{id}":{"get":{
                    "parameters":[
                        {"name":"name","in":"query","required":true,"schema":{"$ref":"#/components/schemas/Name"}},
                        {"name":"id","in":"path","required":true,"schema":{"type":"integer","minimum":1,"maximum":1000,"nullable":false}}
                    ],
                    "responses":{
                        "200":{"description":"Success"},
                        "400":{"description":"Invalid parameters","content":{"application/json":{"schema":{
                            "type":"object","additionalProperties":false,"properties":{"code":{"type":"string"}}
                        }}}}
                    }
                }}}
            });
            let source = SourceDocument::new("openapi.json".into(), document.to_string());
            let result = rules.review_boundaries(&[source], &[], &|| None).unwrap();
            assert_eq!(result.len(), 7);
            assert!(
                result
                    .iter()
                    .all(|r| r.status == ReviewStatus::Pass
                        || r.status == ReviewStatus::NotApplicable),
                "{version}: {result:?}"
            );
        }
        let missing=SourceDocument::new("openapi.yaml".into(),"openapi: 3.0.3\npaths:\n  /users:\n    get:\n      parameters:\n        - name: name\n          in: query\n          schema:\n            type: string\n      responses: {}\n".into());
        let result = rules.review_boundaries(&[missing], &[], &|| None).unwrap();
        assert!(found(&result, "SL-API-003"));
        assert!(result
            .iter()
            .flat_map(|r| &r.findings)
            .all(|f| f.kind == "missing_requirement" && f.confirmation_status == "needs_review"));
        for reference in [
            "https://evil.invalid/schema.json",
            "../secret.json",
            "#/components/schemas/Cycle",
        ] {
            let source=SourceDocument::new("openapi.json".into(),serde_json::json!({"openapi":"3.1.0","components":{"schemas":{"Cycle":{"$ref":"#/components/schemas/Cycle"}}},"paths":{"/users":{"get":{"parameters":[{"name":"input","in":"query","schema":{"$ref":reference}}],"responses":{}}}}}).to_string());
            let result = rules.review_boundaries(&[source], &[], &|| None).unwrap();
            assert!(result.iter().any(|r| r.status == ReviewStatus::NotVerified));
            assert!(result.iter().any(|r| r
                .reason
                .as_deref()
                .is_some_and(|v| v == "external_ref_not_read" || v == "unresolved_ref")));
        }
        assert!(rules
            .review_boundaries(&[], &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
        let invalid = SourceDocument::new("openapi.yaml".into(), "openapi: [not yaml".into());
        assert!(rules
            .review_boundaries(&[invalid], &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.status == ReviewStatus::NotVerified));
    }
    #[test]
    fn security_boundary_unsupported_yaml_and_schema_constructs_never_pass() {
        let rules = RuleSet::builtin().unwrap();
        let alias = SourceDocument::new(
            "openapi.yaml".into(),
            "openapi: 3.1.0\na: &a [x, x]\nb: &b [*a, *a]\npaths: {}\n".into(),
        );
        assert!(rules
            .review_boundaries(&[alias], &[], &|| None)
            .unwrap()
            .iter()
            .all(|r| r.reason.as_deref() == Some("unsupported_yaml_feature")));
        for schema in [
            serde_json::json!({"$id":"https://outside.invalid/schema","type":"string","minLength":1,"maxLength":20}),
            serde_json::json!({"$dynamicRef":"https://outside.invalid/schema"}),
            serde_json::json!({"type":"object","additionalProperties":{"$ref":"https://outside.invalid/schema"}}),
        ] {
            let document=SourceDocument::new("openapi.json".into(),serde_json::json!({"openapi":"3.1.0","paths":{"/users":{"get":{"parameters":[{"name":"name","in":"query","required":true,"schema":schema}],"responses":{}}}}}).to_string());
            assert!(rules
                .review_boundaries(&[document], &[], &|| None)
                .unwrap()
                .iter()
                .all(|r| r.status == ReviewStatus::NotVerified));
        }
    }
    #[test]
    fn security_boundary_operation_parameter_overrides_path_parameter_and_types_stay_versioned() {
        let rules = RuleSet::builtin().unwrap();
        let document=SourceDocument::new("openapi.json".into(),serde_json::json!({"openapi":"3.1.0","paths":{"/users":{"parameters":[{"name":"q","in":"query","schema":{"type":"string"}}],"get":{"parameters":[{"name":"q","in":"query","required":true,"schema":{"type":["string","null"],"minLength":0,"maxLength":40}}],"responses":{}}}}}).to_string());
        let results = rules.review_boundaries(&[document], &[], &|| None).unwrap();
        assert_eq!(
            results
                .iter()
                .find(|r| r.rule_id == "SL-API-003")
                .unwrap()
                .status,
            ReviewStatus::Pass
        );
        assert_eq!(
            results
                .iter()
                .find(|r| r.rule_id == "SL-API-001")
                .unwrap()
                .status,
            ReviewStatus::Pass
        );
    }
}

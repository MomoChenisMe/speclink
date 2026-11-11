//! Output of the check verbs: validate, analyze, drift.

use serde_json::Value;
use speclink_core::analyzer::AnalyzeReport;
use speclink_core::drift::{CombinedDriftReport, DriftReport};
use speclink_core::validate::ValidationResult;

use super::{value_of, JsonForm, Paint, RenderOptions, Rendered, Text};

/// `validate --json`: the result array.
pub fn validate_json(results: &[ValidationResult]) -> Value {
    value_of(results)
}

/// `validate`: one line per item, its errors and its warnings under it.
/// Whether an invalid item fails the run is the caller's decision.
pub fn validate_human(results: &[ValidationResult], paint: &Paint) -> Text {
    let mut t = Text::default();
    for r in results {
        if r.valid {
            outln!(t, "{} {} — valid", paint.green("✓"), r.change);
        } else {
            outln!(t, "{} {} — invalid", paint.red("✗"), r.change);
            for e in &r.errors {
                outln!(t, "  {} {e}", paint.red("error:"));
            }
        }
        for w in &r.warnings {
            outln!(t, "  {} {w}", paint.yellow("warn:"));
        }
    }
    t
}

pub fn validate(results: &[ValidationResult], options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(results)), |paint| validate_human(results, paint))
}

/// `analyze --json`: the report as the engine serializes it.
pub fn analyze_json(report: &AnalyzeReport) -> Value {
    value_of(report)
}

/// `analyze`: the dimension table, then the findings.
pub fn analyze_human(report: &AnalyzeReport, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{}: {}", paint.bold("Change"), report.change_id);
    outln!(t);
    for d in &report.dimensions {
        let sym = if d.finding_count == 0 { paint.green("✓") } else { paint.yellow("●") };
        // The bold span covers a 14-wide padded name; the 15th column separator
        // space stays outside it (plain bytes are identical to the old {:<15} form).
        outln!(
            t,
            "  {sym} {} {} ({} findings)",
            paint.bold(&format!("{:<14}", d.dimension)),
            paint.dim(&d.status),
            d.finding_count
        );
    }
    // The blank separator is tied to the "Analyzed:" line; an empty change (nothing analyzed)
    // prints "Missing:" directly after the dimensions (frozen output shape).
    if !report.artifacts_analyzed.is_empty() {
        outln!(t);
        outln!(t, "  {} {}", paint.dim("Analyzed:"), report.artifacts_analyzed.join(", "));
    }
    if !report.artifacts_missing.is_empty() {
        outln!(t, "  {} {}", paint.yellow("Missing:"), report.artifacts_missing.join(", "));
    }
    if report.findings.is_empty() {
        outln!(t);
        outln!(t, "  {} No issues found", paint.green("✓"));
        return t;
    }
    outln!(t);
    outln!(t, "  {} ({}):", paint.bold("Findings"), report.findings.len());
    outln!(t);
    for f in &report.findings {
        let tag = match f.severity.as_str() {
            "Critical" => paint.bold_red("CRITICAL"),
            "Warning" => paint.yellow("WARNING"),
            _ => paint.dim("SUGGEST"),
        };
        outln!(t, "  [{tag}] {}", f.summary);
        outln!(t, "    {} {}", paint.dim("at:"), paint.dim(&f.location));
        outln!(t, "    {} {}", paint.dim("→"), paint.dim(&f.recommendation));
    }
    t
}

pub fn analyze(report: &AnalyzeReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(report)), |paint| analyze_human(report, paint))
}

/// `drift --json`: the merged report as the engine serializes it.
pub fn drift_json(report: &CombinedDriftReport) -> Value {
    value_of(report)
}

/// `drift`: the dimension table, the detail sections, then the severity.
pub fn drift_human(report: &DriftReport, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{}: {}", paint.bold("Drift Report"), report.change_id);
    if let Some(created) = &report.created {
        outln!(t, "  Created: {created}");
    }
    outln!(t);
    // Bold spans cover 11/35-wide padded cells with the separator spaces outside, plus
    // a leading-space " Score" cell — plain bytes are identical to the old format.
    outln!(
        t,
        "  {} {} {}",
        paint.bold(&format!("{:<11}", "Dimension")),
        paint.bold(&format!("{:<35}", "Status")),
        paint.bold(" Score")
    );
    for d in &report.dimensions {
        let score = if d.contributes_to_total { format!("+{}", d.score) } else { "—".to_string() };
        outln!(t, "  {:<11} {:<36} {:>5}", d.kind, d.status, score);
    }
    outln!(
        t,
        "  {} {} {}",
        paint.bold(&format!("{:<11}", "Total")),
        " ".repeat(35),
        paint.bold(&format!("{:>6}", report.total_score))
    );
    if !report.broken_anchors.is_empty() {
        outln!(t);
        outln!(t, "Broken anchors");
        for b in &report.broken_anchors {
            outln!(t, "  - {} ({}) — {}", b.anchor, b.category, b.reason);
        }
    }
    if !report.spec_assumptions.is_empty() {
        outln!(t);
        outln!(t, "Stale delta assumptions");
        for a in &report.spec_assumptions {
            outln!(t, "  - {} '{}' in {} — {}", a.operation, a.requirement, a.capability, a.reason);
        }
    }
    if !report.tasks_maybe_resolved.is_empty() {
        outln!(t);
        outln!(t, "Tasks maybe already done");
        for task in &report.tasks_maybe_resolved {
            outln!(t, "  - {task}");
        }
    }
    if !report.tasks_blocked_external.is_empty() {
        outln!(t);
        outln!(t, "Tasks blocked by external changes");
        for task in &report.tasks_blocked_external {
            outln!(t, "  - {task}");
        }
    }
    outln!(t);
    let sev = report.severity.to_uppercase();
    let sev_colored = match sev.as_str() {
        "LIGHT" => paint.bold_green(&sev),
        "MEDIUM" => paint.bold_yellow(&sev),
        _ => paint.bold_red(&sev),
    };
    outln!(t, "{}: {} drift", paint.bold("Severity"), sev_colored);
    outln!(t, "> {}", paint.bold_cyan(&report.primary_recommendation));
    t
}

pub fn drift(report: &CombinedDriftReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(report)), |paint| drift_human(&report.report, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::analyzer::{DimensionStatus, Finding, Msg};
    use speclink_core::drift::{BrokenAnchor, Coverage, DriftDimension, SpecAssumption};

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    // --- validate ---

    fn results() -> Vec<ValidationResult> {
        vec![
            ValidationResult {
                change: "alpha".to_string(),
                errors: vec!["specs/auth/spec.md: MODIFIED 'Password sign-in': drops a scenario".to_string()],
                valid: false,
                warnings: Vec::new(),
            },
            ValidationResult {
                change: "started".to_string(),
                errors: Vec::new(),
                valid: true,
                warnings: vec!["No delta specs found".to_string()],
            },
        ]
    }

    #[test]
    fn validate_human_prints_each_item_with_its_errors_and_warnings() {
        assert_eq!(
            validate_human(&results(), &PLAIN).stdout,
            "✗ alpha — invalid\n  error: specs/auth/spec.md: MODIFIED 'Password sign-in': drops a scenario\n✓ started — valid\n  warn: No delta specs found\n"
        );
        assert_eq!(
            validate_human(&results(), &COLOR).stdout,
            "\x1b[31m✗\x1b[0m alpha — invalid\n  \x1b[31merror:\x1b[0m specs/auth/spec.md: MODIFIED 'Password sign-in': drops a scenario\n\x1b[32m✓\x1b[0m started — valid\n  \x1b[33mwarn:\x1b[0m No delta specs found\n"
        );
    }

    #[test]
    fn validate_json_is_the_result_array() {
        let one = &results()[1..];
        assert_eq!(
            validate_json(one),
            json!([{ "change": "started", "errors": [], "valid": true, "warnings": ["No delta specs found"] }])
        );
        assert_eq!(
            validate(one, RenderOptions { json: true, color: false }).stdout,
            "[\n  {\n    \"change\": \"started\",\n    \"errors\": [],\n    \"valid\": true,\n    \"warnings\": [\n      \"No delta specs found\"\n    ]\n  }\n]\n"
        );
    }

    // --- analyze ---

    fn finding(severity: &str, summary: &str) -> Finding {
        Finding {
            id: "COV-1".to_string(),
            dimension: "Coverage".to_string(),
            severity: severity.to_string(),
            location: "specs/auth/spec.md".to_string(),
            summary: summary.to_string(),
            recommendation: "Add a task".to_string(),
            summary_msg: Msg { key: "k.summary".to_string(), params: Default::default() },
            recommendation_msg: Msg { key: "k.recommendation".to_string(), params: Default::default() },
        }
    }

    fn analyzed(findings: Vec<Finding>) -> AnalyzeReport {
        let dimension = |name: &str, status: &str, count: usize| DimensionStatus {
            dimension: name.to_string(),
            status: status.to_string(),
            finding_count: count,
        };
        AnalyzeReport {
            change_id: "alpha".to_string(),
            dimensions: vec![
                dimension("Coverage", "1 issue(s) found", findings.len()),
                dimension("Gaps", "Clean", 0),
            ],
            findings,
            artifacts_analyzed: vec!["proposal".to_string(), "tasks".to_string()],
            artifacts_missing: vec!["design".to_string()],
        }
    }

    #[test]
    fn analyze_human_prints_the_dimensions_then_the_findings() {
        let report = analyzed(vec![
            finding("Warning", "Requirement 'Password sign-in' has no matching task"),
            finding("Critical", "No tasks"),
            finding("Suggestion", "Scenario 'It works' has no concrete examples"),
        ]);
        assert_eq!(
            analyze_human(&report, &PLAIN).stdout,
            "Change: alpha\n\n  ● Coverage       1 issue(s) found (3 findings)\n  ✓ Gaps           Clean (0 findings)\n\n  Analyzed: proposal, tasks\n  Missing: design\n\n  Findings (3):\n\n  [WARNING] Requirement 'Password sign-in' has no matching task\n    at: specs/auth/spec.md\n    → Add a task\n  [CRITICAL] No tasks\n    at: specs/auth/spec.md\n    → Add a task\n  [SUGGEST] Scenario 'It works' has no concrete examples\n    at: specs/auth/spec.md\n    → Add a task\n"
        );
    }

    #[test]
    fn analyze_human_colors_the_table_and_the_severity_tags() {
        let report = analyzed(vec![finding("Critical", "No tasks")]);
        assert_eq!(
            analyze_human(&report, &COLOR).stdout,
            "\x1b[1mChange\x1b[0m: alpha\n\n  \x1b[33m●\x1b[0m \x1b[1mCoverage      \x1b[0m \x1b[2m1 issue(s) found\x1b[0m (1 findings)\n  \x1b[32m✓\x1b[0m \x1b[1mGaps          \x1b[0m \x1b[2mClean\x1b[0m (0 findings)\n\n  \x1b[2mAnalyzed:\x1b[0m proposal, tasks\n  \x1b[33mMissing:\x1b[0m design\n\n  \x1b[1mFindings\x1b[0m (1):\n\n  [\x1b[1;31mCRITICAL\x1b[0m] No tasks\n    \x1b[2mat:\x1b[0m \x1b[2mspecs/auth/spec.md\x1b[0m\n    \x1b[2m→\x1b[0m \x1b[2mAdd a task\x1b[0m\n"
        );
    }

    #[test]
    fn analyze_human_with_no_findings_and_nothing_analyzed() {
        let mut report = analyzed(Vec::new());
        report.artifacts_analyzed.clear();
        assert_eq!(
            analyze_human(&report, &PLAIN).stdout,
            "Change: alpha\n\n  ✓ Coverage       1 issue(s) found (0 findings)\n  ✓ Gaps           Clean (0 findings)\n  Missing: design\n\n  ✓ No issues found\n"
        );
    }

    #[test]
    fn analyze_json_is_the_report() {
        let value = analyze_json(&analyzed(vec![finding("Warning", "No tasks")]));
        assert_eq!(value["change_id"], "alpha");
        assert_eq!(value["dimensions"][0], json!({ "dimension": "Coverage", "status": "1 issue(s) found", "finding_count": 1 }));
        assert_eq!(value["findings"][0]["summary_msg"], json!({ "key": "k.summary", "params": {} }));
        assert_eq!(value["artifacts_missing"], json!(["design"]));
        let text = analyze(&analyzed(Vec::new()), RenderOptions { json: true, color: false }).stdout;
        assert!(text.starts_with("{\n  \"change_id\": \"alpha\",\n  \"dimensions\": [\n"), "{text}");
        assert!(text.ends_with("  \"artifacts_missing\": [\n    \"design\"\n  ]\n}\n"), "{text}");
    }

    // --- drift ---

    fn drifted() -> CombinedDriftReport {
        let dimension = |kind: &str, status: &str, score: i64, counts: bool| DriftDimension {
            kind: kind.to_string(),
            status: status.to_string(),
            score,
            contributes_to_total: counts,
        };
        CombinedDriftReport {
            report: DriftReport {
                change_id: "alpha".to_string(),
                created: Some("2026-07-01T10:00:00+08:00".to_string()),
                last_commit: None,
                dimensions: vec![
                    dimension("Time", "stale (133d)", 3, true),
                    dimension("Environment", "2 commits (1 touching this change's files)", 0, false),
                ],
                broken_anchors: vec![BrokenAnchor {
                    anchor: "sign_in".to_string(),
                    category: "symbol".to_string(),
                    reason: "not found".to_string(),
                }],
                tasks_maybe_resolved: vec!["1.1 first".to_string()],
                tasks_blocked_external: vec!["1.2 second".to_string()],
                spec_assumptions: vec![SpecAssumption {
                    capability: "auth".to_string(),
                    operation: "MODIFIED".to_string(),
                    requirement: "Password sign-in".to_string(),
                    reason: "drops a scenario".to_string(),
                }],
                commits_since_created: 2,
                total_score: 7,
                severity: "medium".to_string(),
                primary_recommendation: "/speclink-ingest alpha".to_string(),
            },
            coverage: Coverage::Full,
            stale: None,
        }
    }

    #[test]
    fn drift_human_prints_the_table_the_sections_and_the_severity() {
        assert_eq!(
            drift_human(&drifted().report, &PLAIN).stdout,
            "Drift Report: alpha\n  Created: 2026-07-01T10:00:00+08:00\n\n  Dimension   Status                               Score\n  Time        stale (133d)                            +3\n  Environment 2 commits (1 touching this change's files)     —\n  Total                                                7\n\nBroken anchors\n  - sign_in (symbol) — not found\n\nStale delta assumptions\n  - MODIFIED 'Password sign-in' in auth — drops a scenario\n\nTasks maybe already done\n  - 1.1 first\n\nTasks blocked by external changes\n  - 1.2 second\n\nSeverity: MEDIUM drift\n> /speclink-ingest alpha\n"
        );
    }

    #[test]
    fn drift_human_colors_the_headers_the_total_and_the_severity() {
        let mut report = drifted().report;
        report.broken_anchors.clear();
        report.spec_assumptions.clear();
        report.tasks_maybe_resolved.clear();
        report.tasks_blocked_external.clear();
        report.created = None;
        report.dimensions.truncate(1);
        assert_eq!(
            drift_human(&report, &COLOR).stdout,
            "\x1b[1mDrift Report\x1b[0m: alpha\n\n  \x1b[1mDimension  \x1b[0m \x1b[1mStatus                             \x1b[0m \x1b[1m Score\x1b[0m\n  Time        stale (133d)                            +3\n  \x1b[1mTotal      \x1b[0m                                     \x1b[1m     7\x1b[0m\n\n\x1b[1mSeverity\x1b[0m: \x1b[1;33mMEDIUM\x1b[0m drift\n> \x1b[1;36m/speclink-ingest alpha\x1b[0m\n"
        );
        for (severity, code) in [("light", "1;32"), ("heavy", "1;31")] {
            report.severity = severity.to_string();
            let text = drift_human(&report, &COLOR).stdout;
            let line = format!("\x1b[1mSeverity\x1b[0m: \x1b[{code}m{}\x1b[0m drift\n", severity.to_uppercase());
            assert!(text.contains(&line), "{text}");
        }
    }

    #[test]
    fn drift_json_is_the_merged_report_without_the_absent_markers() {
        let value = drift_json(&drifted());
        assert_eq!(value["change_id"], "alpha");
        assert_eq!(value["last_commit"], Value::Null);
        assert_eq!(
            value["dimensions"][0],
            json!({ "kind": "Time", "status": "stale (133d)", "score": 3, "contributes_to_total": true })
        );
        assert_eq!(value["total_score"], 7);
        assert!(value.get("coverage").is_none() && value.get("stale").is_none(), "{value}");
        let text = drift(&drifted(), RenderOptions { json: true, color: false }).stdout;
        assert!(text.starts_with("{\n  \"change_id\": \"alpha\",\n  \"created\": \"2026-07-01T10:00:00+08:00\",\n  \"last_commit\": null,\n"), "{text}");
        assert!(text.ends_with("  \"primary_recommendation\": \"/speclink-ingest alpha\"\n}\n"), "{text}");
    }
}

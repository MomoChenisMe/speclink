//! Four-dimension consistency analysis (Coverage / Consistency / Ambiguity / Gaps).

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::util;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
    Suggestion,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "Critical",
            Severity::Warning => "Warning",
            Severity::Suggestion => "Suggestion",
        }
    }
    /// Short tag used in human output.
    pub fn tag(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::Warning => "WARNING",
            Severity::Suggestion => "SUGGEST",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Msg {
    pub key: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub id: String,
    pub dimension: String,
    pub severity: String,
    pub location: String,
    pub summary: String,
    pub recommendation: String,
    pub summary_msg: Msg,
    pub recommendation_msg: Msg,
}

#[derive(Debug, Serialize)]
pub struct DimensionStatus {
    pub dimension: String,
    pub status: String,
    pub finding_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeReport {
    pub change_id: String,
    pub dimensions: Vec<DimensionStatus>,
    pub findings: Vec<Finding>,
    pub artifacts_analyzed: Vec<String>,
    pub artifacts_missing: Vec<String>,
}

// --- Parsed structures ---

struct Scenario {
    name: String,
    has_example: bool,
}
struct Requirement {
    name: String,
    operation: String,
    scenarios: Vec<Scenario>,
}

fn parse_delta_spec(text: &str) -> Vec<Requirement> {
    let mut reqs: Vec<Requirement> = Vec::new();
    let mut operation = String::new();
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(op) = t.strip_prefix("## ") {
            if op.ends_with("Requirements") {
                operation = op.split_whitespace().next().unwrap_or("").to_string();
            }
        } else if let Some(name) = t.strip_prefix("### Requirement:") {
            reqs.push(Requirement {
                name: name.trim().to_string(),
                operation: operation.clone(),
                scenarios: Vec::new(),
            });
        } else if let Some(name) = t.strip_prefix("#### Scenario:") {
            if let Some(req) = reqs.last_mut() {
                req.scenarios.push(Scenario {
                    name: name.trim().to_string(),
                    has_example: false,
                });
            }
        } else if t.starts_with("##### Example:") {
            if let Some(req) = reqs.last_mut() {
                if let Some(sc) = req.scenarios.last_mut() {
                    sc.has_example = true;
                }
            }
        }
    }
    reqs
}

/// Extract capability names from proposal `### New Capabilities` / `### Modified Capabilities`.
fn parse_capabilities(proposal: &str) -> (Vec<String>, Vec<String>) {
    let mut new_caps = Vec::new();
    let mut mod_caps = Vec::new();
    let mut section = 0; // 0 none, 1 new, 2 modified
    for line in proposal.lines() {
        let t = line.trim();
        if t.starts_with("### New Capabilities") {
            section = 1;
            continue;
        } else if t.starts_with("### Modified Capabilities") {
            section = 2;
            continue;
        } else if t.starts_with("## ") || t.starts_with("### ") {
            section = 0;
        }
        if section != 0 {
            if let Some(cap) = parse_cap_bullet(t) {
                if section == 1 {
                    new_caps.push(cap);
                } else {
                    mod_caps.push(cap);
                }
            }
        }
    }
    (new_caps, mod_caps)
}

fn parse_cap_bullet(line: &str) -> Option<String> {
    // Matches: - `cap-name`: description
    let rest = line.strip_prefix("- ")?;
    let rest = rest.strip_prefix('`')?;
    let end = rest.find('`')?;
    let name = &rest[..end];
    if name.is_empty() || name.starts_with('<') {
        None
    } else {
        Some(name.to_string())
    }
}

fn design_headings(design: &str) -> Vec<String> {
    design
        .lines()
        .filter_map(|l| l.strip_prefix("### ").map(|s| s.trim().to_string()))
        .collect()
}

fn task_descriptions(tasks: &str) -> Vec<String> {
    crate::tasks::parse(tasks)
        .into_iter()
        .map(|t| t.description)
        .collect()
}

pub fn analyze(change: &Change, schema: &Schema) -> AnalyzeReport {
    let proposal = util::read_opt(&change.dir.join("proposal.md")).unwrap_or_default();
    let design = util::read_opt(&change.dir.join("design.md")).unwrap_or_default();
    let tasks_text = util::read_opt(&change.dir.join("tasks.md")).unwrap_or_default();
    let spec_files = model::spec_files(&change.dir);

    let (new_caps, mod_caps) = parse_capabilities(&proposal);
    let tasks = task_descriptions(&tasks_text);

    let mut coverage: Vec<Finding> = Vec::new();
    let mut consistency: Vec<Finding> = Vec::new();
    let mut ambiguity: Vec<Finding> = Vec::new();
    let mut gaps: Vec<Finding> = Vec::new();

    // --- Coverage: capability without spec file ---
    let mut cov_n = 0;
    for cap in &new_caps {
        let spec = change.dir.join("specs").join(cap).join("spec.md");
        if !util::has_content(&spec) {
            cov_n += 1;
            coverage.push(make_finding(
                "COV", cov_n, "Coverage", Severity::Critical,
                &format!("specs/{cap}/spec.md"),
                &format!("Capability '{cap}' has no spec file"),
                &format!("Create specs/{cap}/spec.md for capability '{cap}'"),
                "covMissingSpec", [("capability", cap.as_str())],
            ));
        }
    }

    // Parse all delta specs.
    let mut all_reqs: Vec<(String, Requirement)> = Vec::new();
    for sp in &spec_files {
        let rel = sp
            .strip_prefix(&change.dir)
            .map(util::to_slash)
            .unwrap_or_default();
        let text = util::read_opt(sp).unwrap_or_default();
        for req in parse_delta_spec(&text) {
            all_reqs.push((rel.clone(), req));
        }
    }

    // --- Coverage: requirement without task ---
    for (loc, req) in &all_reqs {
        let covered = tasks.iter().any(|t| t.contains(&req.name));
        if !covered {
            cov_n += 1;
            coverage.push(make_finding(
                "COV", cov_n, "Coverage", Severity::Warning,
                loc,
                &format!("Requirement '{}' has no corresponding task", req.name),
                &format!("Add a task that implements '{}'", req.name),
                "covMissingTask", [("requirement", req.name.as_str())],
            ));
        }
    }

    // --- Consistency: design heading not referenced in tasks ---
    let mut con_n = 0;
    if !design.trim().is_empty() {
        for h in design_headings(&design) {
            let referenced = tasks.iter().any(|t| t.contains(&h));
            if !referenced {
                con_n += 1;
                consistency.push(make_finding(
                    "CON", con_n, "Consistency", Severity::Warning,
                    "design.md",
                    &format!("Design topic '{h}' is not referenced by any task"),
                    &format!("Add a task covering design decision '{h}'"),
                    "conDesignNotInTasks", [("topic", h.as_str())],
                ));
            }
        }
    }

    // --- Ambiguity: no scenario / abstract scenario ---
    let mut amb_n = 0;
    for (loc, req) in &all_reqs {
        if req.scenarios.is_empty() {
            amb_n += 1;
            ambiguity.push(make_finding(
                "AMB", amb_n, "Ambiguity", Severity::Warning,
                loc,
                &format!("Requirement '{}' has no scenario", req.name),
                "Add a #### Scenario: with WHEN/THEN",
                "ambNoScenario", [("requirement", req.name.as_str())],
            ));
        }
        for sc in &req.scenarios {
            if !sc.has_example {
                amb_n += 1;
                ambiguity.push(make_finding(
                    "AMB", amb_n, "Ambiguity", Severity::Suggestion,
                    loc,
                    &format!("Scenario '{}' has no concrete examples", sc.name),
                    "Add ##### Example: with concrete GIVEN/WHEN/THEN data",
                    "ambAbstractScenario", [("scenario", sc.name.as_str())],
                ));
            }
        }
    }

    // --- Gaps: spec but no proposal / modified not found ---
    let mut gap_n = 0;
    if !spec_files.is_empty() && proposal.trim().is_empty() {
        gap_n += 1;
        gaps.push(make_finding(
            "GAP", gap_n, "Gaps", Severity::Critical,
            "proposal.md",
            "Specs exist but no proposal was found",
            "Create a proposal.md describing why this change is needed",
            "gapNoProposal", [],
        ));
    }
    for (loc, req) in &all_reqs {
        if req.operation == "MODIFIED" {
            // Modified capability should have a canonical spec.
            let cap = loc.split('/').nth(1).unwrap_or("");
            let canonical = change
                .dir
                .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
                .map(|root| root.join("specs").join(cap).join("spec.md"));
            let exists = canonical.as_ref().map(|p| util::has_content(p)).unwrap_or(false);
            if !exists {
                gap_n += 1;
                gaps.push(make_finding(
                    "GAP", gap_n, "Gaps", Severity::Critical,
                    loc,
                    &format!("MODIFIED requirement '{}' has no canonical spec", req.name),
                    "Ensure the capability exists in openspec/specs/ before modifying it",
                    "gapModifiedNotFound", [("requirement", req.name.as_str())],
                ));
            }
        }
    }

    let _ = mod_caps;

    // Assemble in dimension order: Coverage, Consistency, Ambiguity, Gaps.
    let dims = [
        ("Coverage", &coverage),
        ("Consistency", &consistency),
        ("Ambiguity", &ambiguity),
        ("Gaps", &gaps),
    ];
    let dimensions = dims
        .iter()
        .map(|(name, list)| DimensionStatus {
            dimension: name.to_string(),
            status: if list.is_empty() {
                "Clean".to_string()
            } else {
                format!("{} issue(s) found", list.len())
            },
            finding_count: list.len(),
        })
        .collect();

    let mut findings = Vec::new();
    findings.extend(coverage);
    findings.extend(consistency);
    findings.extend(ambiguity);
    findings.extend(gaps);

    // Artifacts analyzed / missing.
    let mut analyzed = Vec::new();
    let mut missing = Vec::new();
    for a in &schema.artifacts {
        if model::artifact_done(&change.dir, a) {
            analyzed.push(a.id.to_string());
        } else {
            missing.push(a.id.to_string());
        }
    }

    AnalyzeReport {
        change_id: change.name.clone(),
        dimensions,
        findings,
        artifacts_analyzed: analyzed,
        artifacts_missing: missing,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_finding<'a>(
    prefix: &str,
    n: usize,
    dimension: &str,
    severity: Severity,
    location: &str,
    summary: &str,
    recommendation: &str,
    key_base: &str,
    params: impl IntoIterator<Item = (&'static str, &'a str)>,
) -> Finding {
    let mut pmap = BTreeMap::new();
    for (k, v) in params {
        pmap.insert(k.to_string(), v.to_string());
    }
    Finding {
        id: format!("{prefix}-{n}"),
        dimension: dimension.to_string(),
        severity: severity.as_str().to_string(),
        location: location.to_string(),
        summary: summary.to_string(),
        recommendation: recommendation.to_string(),
        summary_msg: Msg {
            key: format!("{key_base}.summary"),
            params: pmap.clone(),
        },
        recommendation_msg: Msg {
            key: format!("{key_base}.recommendation"),
            params: pmap,
        },
    }
}

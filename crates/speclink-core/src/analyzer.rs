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
    /// Whether the WHEN/THEN body already contains concrete values (e.g. numbers).
    has_concrete: bool,
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
                    has_concrete: false,
                });
            }
        } else if t.starts_with("##### Example:") {
            if let Some(req) = reqs.last_mut() {
                if let Some(sc) = req.scenarios.last_mut() {
                    sc.has_example = true;
                }
            }
        } else if !t.starts_with('#') {
            // Concrete = ASCII digits, backticked code, or a double-quoted string (matches
            // Spectra; single/fullwidth quotes and non-ASCII digits do NOT count).
            if t.chars().any(|c| c.is_ascii_digit() || c == '`' || c == '"') {
                if let Some(req) = reqs.last_mut() {
                    if let Some(sc) = req.scenarios.last_mut() {
                        sc.has_concrete = true;
                    }
                }
            }
        }
    }
    reqs
}

/// Extract capability names from the proposal (probed against Spectra): a
/// "Capabilities"-titled section opens extraction — `## Capabilities` (h2) or
/// `### New/Modified Capabilities` (h3) — and EVERY following line contributes its first
/// backticked whitespace-free token (any bullet style, numbered lists, plain prose),
/// continuing through unrelated h3 headings until the next h2. Angle-bracket
/// placeholders are kept (Spectra flags `<placeholder>` as a missing spec); a backticked
/// token containing whitespace disqualifies the line with no fallback to later pairs.
fn parse_capabilities(proposal: &str) -> (Vec<String>, Vec<String>) {
    let mut new_caps = Vec::new();
    let mut mod_caps = Vec::new();
    // 0 = outside any capabilities section; 1 = new bucket; 2 = modified bucket.
    let mut section = 0;
    for line in proposal.lines() {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("### ") {
            let h = h.trim_start();
            if h.starts_with("New Capabilities") {
                section = 1;
                continue;
            }
            if h.starts_with("Modified Capabilities") {
                section = 2;
                continue;
            }
            // Any other h3 does NOT close the section — extraction keeps going.
        } else if let Some(h) = t.strip_prefix("## ") {
            section = if h.trim().starts_with("Capabilities") { 1 } else { 0 };
            continue;
        }
        if section != 0 {
            if let Some(cap) = first_backtick_token(t) {
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

fn first_backtick_token(line: &str) -> Option<String> {
    let open = line.find('`')?;
    let rest = &line[open + 1..];
    let close = rest.find('`')?;
    let tok = &rest[..close];
    if tok.is_empty() || tok.contains(char::is_whitespace) {
        None
    } else {
        Some(tok.to_string())
    }
}

fn design_headings(design: &str) -> Vec<String> {
    // Leading whitespace is trimmed before the `### ` check (probed: Spectra recognizes
    // headings indented by spaces or tabs, beyond CommonMark's 3-space limit).
    design
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("### ").map(|s| s.trim().to_string()))
        .collect()
}

fn task_descriptions(tasks: &str) -> Vec<String> {
    crate::tasks::parse(tasks)
        .into_iter()
        .map(|t| t.description)
        .collect()
}

/// A requirement is "covered" if its name appears as a contiguous (case-insensitive) substring in
/// some task line — matches Spectra (e.g. "CSV Export" ↔ "Implement CSV exporter", but NOT
/// "csv-export" or "export_csv"). No identifier-token splitting.
fn req_covered(name: &str, tasks: &[String]) -> bool {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return true;
    }
    tasks.iter().any(|t| t.to_lowercase().contains(&n))
}

/// First weak/vague pattern on a spec line — at most ONE finding per line, taken in
/// Spectra's fixed check order (should, may, might, consider, possibly, TBD, TODO, ???,
/// TKTK). All English patterns match as case-insensitive SUBSTRINGS ("shoulder" flags
/// 'should', "outbdoor" flags 'TBD' — probed), and heading lines ('#'-leading after
/// trim) are never scanned. The CJK additions (speclink divergence) follow in the same
/// one-per-line discipline, with the 不可能 exemption (「盡可能」 still flags via 可能).
fn weak_pattern_in(line: &str) -> Option<String> {
    if line.trim_start().starts_with('#') {
        return None;
    }
    let lower = line.to_lowercase();
    for p in ["should", "may", "might", "consider", "possibly"] {
        if lower.contains(p) {
            return Some(p.to_string());
        }
    }
    for p in ["TBD", "TODO", "???", "TKTK"] {
        let hit = if p == "???" {
            line.contains("???")
        } else {
            lower.contains(&p.to_lowercase())
        };
        if hit {
            return Some(p.to_string());
        }
    }
    let cjk = ["應該", "也許", "或許", "大概", "考慮", "盡量", "儘量", "待定"];
    for p in cjk {
        if line.contains(p) {
            return Some(p.to_string());
        }
    }
    if line.replace("不可能", "").contains("可能") {
        return Some("可能".to_string());
    }
    None
}

pub fn analyze(change: &Change, schema: &Schema) -> AnalyzeReport {
    let proposal = util::read_opt(&change.dir.join("proposal.md")).unwrap_or_default();
    let design = util::read_opt(&change.dir.join("design.md")).unwrap_or_default();
    let tasks_text = util::read_opt(&change.dir.join("tasks.md")).unwrap_or_default();
    let spec_files = model::spec_files(&change.dir);

    let (new_caps, mod_caps) = parse_capabilities(&proposal);
    let tasks = task_descriptions(&tasks_text);

    // Parse all delta specs, keeping per-file text for line-based checks.
    let mut all_reqs: Vec<(String, Requirement)> = Vec::new();
    let mut spec_texts: Vec<(String, String)> = Vec::new();
    for sp in &spec_files {
        let rel = sp.strip_prefix(&change.dir).map(util::to_slash).unwrap_or_default();
        let text = util::read_opt(sp).unwrap_or_default();
        for req in parse_delta_spec(&text) {
            all_reqs.push((rel.clone(), req));
        }
        spec_texts.push((rel, text));
    }

    // Artifact presence is EXISTS-based (an empty or op-less file still counts), matching Spectra.
    let proposal_present = change.dir.join("proposal.md").is_file();
    let specs_present = !spec_files.is_empty();
    let tasks_present = change.dir.join("tasks.md").is_file();
    let design_present = change.dir.join("design.md").is_file();

    // A dimension is skipped when its prerequisite artifacts are missing. Coverage needs a proposal
    // plus at least one of specs/tasks to check against; Gaps always runs.
    let coverage_skipped = !(proposal_present && (specs_present || tasks_present));
    let consistency_skipped = !(design_present && tasks_present);
    let ambiguity_skipped = !specs_present;
    // Gaps runs whenever the change has any artifact at all; skipped for an empty change.
    let gaps_skipped = !(proposal_present || specs_present || tasks_present || design_present);

    let mut coverage: Vec<Finding> = Vec::new();
    let mut consistency: Vec<Finding> = Vec::new();
    let mut ambiguity: Vec<Finding> = Vec::new();
    let mut gaps: Vec<Finding> = Vec::new();

    // --- Coverage ---
    if !coverage_skipped {
        let mut n = 0;
        // covMissingSpec applies to Modified Capabilities too (probed).
        for cap in new_caps.iter().chain(mod_caps.iter()) {
            // Flagged only when the delta spec FILE is missing; an empty file counts as present.
            let spec = change.dir.join("specs").join(cap).join("spec.md");
            if !spec.is_file() {
                n += 1;
                coverage.push(make_finding(
                    "COV", n, "Coverage", Severity::Critical,
                    "proposal.md → Capabilities",
                    &format!("Capability `{cap}` has no corresponding spec file"),
                    &format!("Create specs/{cap}/spec.md with requirements"),
                    "covMissingSpec", [("cap", cap.as_str())],
                ));
            }
        }
        // covMissingTask only applies when there are tasks to match against.
        for (loc, req) in all_reqs.iter().filter(|_| tasks_present) {
            if !req_covered(&req.name, &tasks) {
                n += 1;
                coverage.push(make_finding(
                    "COV", n, "Coverage", Severity::Warning, loc,
                    &format!("Requirement '{}' has no matching task", req.name),
                    &format!("Add a task in tasks.md that references '{}'", req.name),
                    "covMissingTask", [("req", req.name.as_str())],
                ));
            }
        }
    }

    // --- Consistency ---
    if !consistency_skipped {
        let mut n = 0;
        // Matched against the FULL tasks.md text (probed: a prose mention outside any
        // checkbox suppresses the finding), lowercased on both sides.
        let tasks_lower = tasks_text.to_lowercase();
        for h in design_headings(&design) {
            let hl = h.to_lowercase();
            if !tasks_lower.contains(&hl) {
                n += 1;
                let mut f = make_finding(
                    "CON", n, "Consistency", Severity::Warning, "design.md",
                    &format!("Design topic '{hl}' not referenced in tasks"),
                    "Verify tasks cover this design decision",
                    "conDesignNotInTasks", [("keyword", hl.as_str())],
                );
                // Spectra sends this recommendation without params.
                f.recommendation_msg.params.clear();
                consistency.push(f);
            }
        }
    }

    // --- Ambiguity — grouped per FILE (matches Spectra), each file emitting its
    // no-scenario, then abstract-scenario, then weak-language findings in turn.
    if !ambiguity_skipped {
        let mut n = 0;
        for (rel, text) in &spec_texts {
            for (loc, req) in all_reqs.iter().filter(|(l, _)| l == rel) {
                if req.scenarios.is_empty() {
                    n += 1;
                    ambiguity.push(make_finding(
                        "AMB", n, "Ambiguity", Severity::Warning, loc,
                        &format!("Requirement '{}' has no scenarios", req.name),
                        &format!("Add #### Scenario: sections with WHEN/THEN for '{}'", req.name),
                        "ambNoScenario", [("req", req.name.as_str())],
                    ));
                }
            }
            for (loc, req) in all_reqs.iter().filter(|(l, _)| l == rel) {
                for sc in &req.scenarios {
                    if !sc.has_example && !sc.has_concrete {
                        n += 1;
                        ambiguity.push(make_finding(
                            "AMB", n, "Ambiguity", Severity::Suggestion, loc,
                            &format!("Scenario '{}' has no concrete examples", sc.name),
                            "Add ##### Example: with concrete GIVEN/WHEN/THEN data",
                            "ambAbstractScenario", [("scenario", sc.name.as_str())],
                        ));
                    }
                }
            }
            for (idx, line) in text.lines().enumerate() {
                if let Some(pat) = weak_pattern_in(line) {
                    n += 1;
                    let loc = format!("{rel}:{}", idx + 1);
                    ambiguity.push(make_finding(
                        "AMB", n, "Ambiguity", Severity::Suggestion, &loc,
                        &format!("Vague language '{pat}' found"),
                        &format!("Replace '{pat}' with SHALL/SHALL NOT for clarity"),
                        "ambWeakLanguage", [("pattern", pat.as_str())],
                    ));
                }
            }
        }
    }

    // --- Gaps ---
    if !gaps_skipped {
        let mut n = 0;
        // Fires only when the proposal FILE is missing (an empty file counts as present).
        if specs_present && !proposal_present {
            n += 1;
            gaps.push(make_finding(
                "GAP", n, "Gaps", Severity::Critical, "change directory",
                "Specs exist but no proposal.md found",
                "Create proposal.md describing the change purpose",
                "gapNoProposal", [],
            ));
        }
        // change.dir = <root>/openspec/changes/<name>; canonical = <root>/openspec/specs/<cap>/spec.md
        let openspec = change.dir.parent().and_then(|p| p.parent());
        let mut cap_no_main: Vec<String> = Vec::new();
        for (loc, req) in &all_reqs {
            if req.operation != "MODIFIED" {
                continue;
            }
            let cap = loc.split('/').nth(1).unwrap_or("");
            let canonical = openspec.map(|o| o.join("specs").join(cap).join("spec.md"));
            // Canonical presence is EXISTS-based: an empty canonical spec falls through to the
            // gapModifiedNotFound branch (the requirement can't be found in it).
            let canonical_text = canonical
                .as_ref()
                .filter(|p| p.is_file())
                .map(|p| util::read_opt(p).unwrap_or_default());
            match canonical_text {
                None => {
                    // No canonical spec for this capability — reported once per capability.
                    if !cap_no_main.contains(&cap.to_string()) {
                        cap_no_main.push(cap.to_string());
                        n += 1;
                        gaps.push(make_finding(
                            "GAP", n, "Gaps", Severity::Warning, loc,
                            &format!("MODIFIED requirements reference capability '{cap}' but no main spec found"),
                            &format!("Check if openspec/specs/{cap}/spec.md exists"),
                            "gapNoMainSpec", [("spec", cap)],
                        ));
                    }
                }
                Some(text) => {
                    if !text.contains(&format!("### Requirement: {}", req.name)) {
                        n += 1;
                        // Spectra's summary params are {name}; the recommendation additionally
                        // carries {spec}.
                        let mut f = make_finding(
                            "GAP", n, "Gaps", Severity::Warning, loc,
                            &format!("MODIFIED requirement '{}' not found in main spec", req.name),
                            &format!("Verify requirement '{}' exists in openspec/specs/{cap}/spec.md", req.name),
                            "gapModifiedNotFound", [("name", req.name.as_str())],
                        );
                        f.recommendation_msg
                            .params
                            .insert("spec".to_string(), cap.to_string());
                        gaps.push(f);
                    }
                }
            }
        }
    }

    let dims: [(&str, &Vec<Finding>, bool); 4] = [
        ("Coverage", &coverage, coverage_skipped),
        ("Consistency", &consistency, consistency_skipped),
        ("Ambiguity", &ambiguity, ambiguity_skipped),
        ("Gaps", &gaps, gaps_skipped),
    ];
    let dimensions = dims
        .iter()
        .map(|(name, list, skipped)| DimensionStatus {
            dimension: name.to_string(),
            status: if *skipped {
                "Skipped (insufficient artifacts)".to_string()
            } else if list.is_empty() {
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

    // Spectra's analyzer is hard-wired to the classic four artifacts regardless of the change's
    // schema (a custom schema's own artifacts are never listed here).
    let _ = schema;
    let mut analyzed = Vec::new();
    let mut missing = Vec::new();
    for (id, present) in [
        ("proposal", proposal_present),
        ("specs", specs_present),
        ("design", design_present),
        ("tasks", tasks_present),
    ] {
        if present {
            analyzed.push(id.to_string());
        } else {
            missing.push(id.to_string());
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

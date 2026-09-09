//! Four-dimension consistency analysis (Coverage / Consistency / Ambiguity / Gaps).

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
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
    /// Whether the requirement body carries a `**Reason**` line — only REMOVED
    /// requirements are checked for it.
    has_reason: bool,
    /// Whether the requirement body carries a `**Migration**` line.
    has_migration: bool,
}

/// Concrete = ASCII digits, backticked code, a double-quoted string, a
/// fullwidth-quoted string, or fullwidth digits. Single quotes and Chinese
/// numerals (一、二、三…, which also open everyday words) do NOT count.
fn is_concrete_char(c: char) -> bool {
    c.is_ascii_digit()
        || matches!(c, '`' | '"' | '\u{300c}' | '\u{300d}' | '\u{300e}' | '\u{300f}')
        || ('\u{ff10}'..='\u{ff19}').contains(&c)
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
                has_reason: false,
                has_migration: false,
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
            if let Some(req) = reqs.last_mut() {
                req.has_reason |= t.starts_with("**Reason**");
                req.has_migration |= t.starts_with("**Migration**");
                if t.chars().any(is_concrete_char) {
                    if let Some(sc) = req.scenarios.last_mut() {
                        sc.has_concrete = true;
                    }
                }
            }
        }
    }
    reqs
}

/// Extract capability names from the proposal (frozen extraction rules): a
/// "Capabilities"-titled section opens extraction — `## Capabilities` (h2) or
/// `### New/Modified Capabilities` (h3) — and EVERY following line contributes its first
/// backticked whitespace-free token (any bullet style, numbered lists, plain prose),
/// continuing through unrelated h3 headings until the next h2. Angle-bracket
/// placeholders are kept (`<placeholder>` is flagged as a missing spec); a backticked
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
    // Leading whitespace is trimmed before the `### ` check: headings indented by spaces
    // or tabs are recognized, beyond CommonMark's 3-space limit.
    design
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("### ").map(|s| s.trim().to_string()))
        .collect()
}

/// Split a design `### ` heading into its ordinal label and its body.
///
/// Only three prefix shapes count (frozen by spec): `D<digits>`, `決策<digits or
/// 一..十>` and `Decision <digits>`, each optionally followed by spaces, at most
/// one colon (`:` or `：`) and more spaces. `D`/`Decision` match
/// case-insensitively. A heading with no such prefix yields `(None, whole
/// heading)`; a bare ordinal yields `(Some(label), "")` — both fall back to
/// whole-string matching at the call site.
fn split_heading_label(heading: &str) -> (Option<String>, String) {
    let h = heading.trim();
    match ordinal_prefix_end(h) {
        Some(end) => {
            let label = h[..end].trim_end().to_string();
            let rest = h[end..].trim_start();
            let rest = rest.strip_prefix('：').or_else(|| rest.strip_prefix(':')).unwrap_or(rest);
            (Some(label), rest.trim().to_string())
        }
        None => (None, h.to_string()),
    }
}

/// Byte index just past the ordinal number, or `None` when the heading carries
/// no recognised prefix.
fn ordinal_prefix_end(h: &str) -> Option<usize> {
    // `D<digits>` — `Decision` never matches here because `e` is not a digit.
    if let Some(rest) = strip_prefix_ci(h, "D") {
        let n = ascii_digit_run(rest);
        if n > 0 {
            return Some(h.len() - rest.len() + n);
        }
    }
    if let Some(rest) = strip_prefix_ci(h, "Decision") {
        let after = rest.trim_start();
        let n = ascii_digit_run(after);
        if n > 0 {
            return Some(h.len() - after.len() + n);
        }
    }
    if let Some(rest) = h.strip_prefix("決策") {
        let n = ascii_digit_run(rest);
        if n > 0 {
            return Some(h.len() - rest.len() + n);
        }
        let c = rest.chars().next()?;
        if "一二三四五六七八九十".contains(c) {
            return Some(h.len() - rest.len() + c.len_utf8());
        }
    }
    None
}

/// ASCII-case-insensitive `strip_prefix`. Safe to slice: an ASCII prefix that
/// matches means the first `prefix.len()` bytes are all single-byte chars.
fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let n = prefix.len();
    if s.len() >= n && s.as_bytes()[..n].eq_ignore_ascii_case(prefix.as_bytes()) {
        Some(&s[n..])
    } else {
        None
    }
}

fn ascii_digit_run(s: &str) -> usize {
    s.bytes().take_while(u8::is_ascii_digit).count()
}

/// Whether tasks.md references a design heading: its body appears anywhere in
/// the task text, or its ordinal appears and is not followed by another ASCII
/// digit (so `D1` is not matched by `D12`). A heading with no ordinal, or one
/// that is nothing but an ordinal, falls back to whole-string matching.
///
/// `tasks_lower` is the FULL lowercased tasks.md text — a prose mention outside
/// any checkbox still suppresses the finding (frozen rule).
fn heading_referenced(heading: &str, tasks_lower: &str) -> bool {
    let (label, body) = split_heading_label(heading);
    match label {
        Some(label) if !body.is_empty() => {
            tasks_lower.contains(&body.to_lowercase())
                || ordinal_referenced(&label.to_lowercase(), tasks_lower)
        }
        _ => tasks_lower.contains(&heading.trim().to_lowercase()),
    }
}

/// An ordinal counts as referenced only where the next character is not an
/// ASCII digit.
fn ordinal_referenced(label_lower: &str, tasks_lower: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = tasks_lower[from..].find(label_lower) {
        let end = from + pos + label_lower.len();
        if !tasks_lower[end..].starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
        from = end;
    }
    false
}

fn task_descriptions(tasks: &str) -> Vec<String> {
    crate::tasks::parse(tasks)
        .into_iter()
        .map(|t| t.description)
        .collect()
}

/// A requirement is "covered" if its name appears as a contiguous (case-insensitive) substring in
/// some task line (e.g. "CSV Export" ↔ "Implement CSV exporter", but NOT
/// "csv-export" or "export_csv"). No identifier-token splitting.
fn req_covered(name: &str, tasks: &[String]) -> bool {
    let n = name.trim().to_lowercase();
    if n.is_empty() {
        return true;
    }
    tasks.iter().any(|t| t.to_lowercase().contains(&n))
}

/// Whether `needle` occurs in `haystack` with no ASCII letter on either side.
/// Both sides are already lowercased by the caller.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(needle) {
        let at = from + pos;
        let end = at + needle.len();
        let before_ok = !haystack[..at].ends_with(|c: char| c.is_ascii_alphabetic());
        let after_ok = !haystack[end..].starts_with(|c: char| c.is_ascii_alphabetic());
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// First weak/vague pattern on a spec line — at most ONE finding per line, taken in
/// the frozen check order (should, may, might, consider, possibly, TBD, TODO, ???,
/// TKTK). The five English patterns match case-insensitively on WORD BOUNDARIES, so
/// "shoulder", "considerable" and "mayor" no longer flag; TBD/TODO/???/TKTK stay
/// substring matches ("outbdoor" still flags 'TBD' — probed), and heading lines
/// ('#'-leading after trim) are never scanned. The CJK additions (speclink divergence)
/// follow in the same one-per-line discipline, with the 不可能 exemption (「盡可能」
/// still flags via 可能) — Chinese has no word-boundary character, so substring
/// matching is the only meaning available there.
fn weak_pattern_in(line: &str) -> Option<String> {
    if line.trim_start().starts_with('#') {
        return None;
    }
    let lower = line.to_lowercase();
    for p in ["should", "may", "might", "consider", "possibly"] {
        if contains_word(&lower, p) {
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

/// Which removal notes a REMOVED requirement lacks: the `missing` param value
/// and the bold-marked wording used in the summary. `None` when both are there.
fn missing_removal_notes(req: &Requirement) -> Option<(&'static str, &'static str)> {
    match (req.has_reason, req.has_migration) {
        (true, true) => None,
        (false, true) => Some(("Reason", "**Reason**")),
        (true, false) => Some(("Migration", "**Migration**")),
        (false, false) => Some(("Reason and Migration", "**Reason** and **Migration**")),
    }
}

pub fn analyze(store: &dyn Store, change: &Change, schema: &Schema) -> AnalyzeReport {
    let proposal = store.read_artifact(&change.name, "proposal.md").unwrap_or_default();
    let design = store.read_artifact(&change.name, "design.md").unwrap_or_default();
    let tasks_text = store.read_artifact(&change.name, "tasks.md").unwrap_or_default();
    let delta_caps = store.delta_capabilities(&change.name);

    let (new_caps, mod_caps) = parse_capabilities(&proposal);
    let tasks = task_descriptions(&tasks_text);

    // Parse all delta specs, keeping per-file text for line-based checks.
    let mut all_reqs: Vec<(String, Requirement)> = Vec::new();
    let mut spec_texts: Vec<(String, String)> = Vec::new();
    for cap in &delta_caps {
        let rel = model::delta_spec_artifact(cap);
        let text = store.read_artifact(&change.name, &rel).unwrap_or_default();
        for req in parse_delta_spec(&text) {
            all_reqs.push((rel.clone(), req));
        }
        spec_texts.push((rel, text));
    }

    // Artifact presence is EXISTS-based (an empty or op-less file still counts) — frozen rule.
    let proposal_present = store.artifact_exists(&change.name, "proposal.md");
    let specs_present = !delta_caps.is_empty();
    let tasks_present = store.artifact_exists(&change.name, "tasks.md");
    let design_present = store.artifact_exists(&change.name, "design.md");

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
            if !store.artifact_exists(&change.name, &model::delta_spec_artifact(cap)) {
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
            if !heading_referenced(&h, &tasks_lower) {
                n += 1;
                let mut f = make_finding(
                    "CON", n, "Consistency", Severity::Warning, "design.md",
                    &format!("Design topic '{hl}' not referenced in tasks"),
                    "Verify tasks cover this design decision",
                    "conDesignNotInTasks", [("keyword", hl.as_str())],
                );
                // This recommendation is sent without params (frozen output shape).
                f.recommendation_msg.params.clear();
                consistency.push(f);
            }
        }
    }

    // --- Ambiguity — grouped per FILE (frozen output shape), each file emitting its
    // no-scenario, then abstract-scenario, then weak-language findings in turn.
    if !ambiguity_skipped {
        let mut n = 0;
        for (rel, text) in &spec_texts {
            for (loc, req) in all_reqs.iter().filter(|(l, _)| l == rel) {
                // A REMOVED requirement carries no scenarios by design; it is held to
                // **Reason** / **Migration** instead.
                if req.operation == "REMOVED" {
                    if let Some((missing, listed)) = missing_removal_notes(req) {
                        n += 1;
                        ambiguity.push(make_finding(
                            "AMB", n, "Ambiguity", Severity::Warning, loc,
                            &format!("REMOVED requirement '{}' has no {listed}", req.name),
                            &format!("Add **Reason**: and **Migration**: lines under '{}'", req.name),
                            "ambRemovedNoReason",
                            [("req", req.name.as_str()), ("missing", missing)],
                        ));
                    }
                } else if req.scenarios.is_empty() {
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
        // Informational (speclink extension): the change reflected discussion(s) that were
        // later re-concluded, so it is stale against the new conclusion and needs re-ingest.
        // Fires only when `restale_from` is non-empty — the common case is byte-identical.
        for slug in change.meta.restale_from() {
            n += 1;
            gaps.push(make_finding(
                "GAP", n, "Gaps", Severity::Suggestion, "change meta",
                &format!("Change reflects discussion '{slug}' which was re-concluded — stale, needs re-ingest"),
                &format!("Run `/speclink-ingest {}` to fold the new conclusion, then seal to clear the flag", change.name),
                "gapRestale", [("discussion", slug.as_str())],
            ));
        }
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
        let mut cap_no_main: Vec<String> = Vec::new();
        for (loc, req) in &all_reqs {
            if req.operation != "MODIFIED" {
                continue;
            }
            let cap = loc.split('/').nth(1).unwrap_or("");
            // Canonical presence is EXISTS-based: an empty canonical spec falls through to the
            // gapModifiedNotFound branch (the requirement can't be found in it).
            let canonical_text = if store.canonical_spec_exists(cap) {
                Some(store.read_canonical_spec(cap).unwrap_or_default())
            } else {
                None
            };
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
                        // The summary params are {name}; the recommendation additionally
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

    // The analyzer is hard-wired to the classic four artifacts regardless of the change's
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    /// Build an in-memory change out of `(artifact rel path, content)` pairs.
    fn store_with(artifacts: &[(&str, &str)]) -> TestStore {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        for (rel, text) in artifacts {
            store.put_artifact("demo", rel, text);
        }
        store
    }

    fn report_of(store: &TestStore) -> AnalyzeReport {
        let change = crate::model::find_change(store, "demo").expect("change resolves");
        analyze(store, &change, &crate::schema::spec_driven())
    }

    fn summaries_of(report: &AnalyzeReport, dimension: &str) -> Vec<String> {
        report
            .findings
            .iter()
            .filter(|f| f.dimension == dimension)
            .map(|f| f.summary.clone())
            .collect()
    }

    /// Consistency findings for a design/tasks pair (the other dimensions have
    /// no prerequisites here, so they stay silent).
    fn consistency_of(design: &str, tasks: &str) -> Vec<String> {
        let store = store_with(&[("design.md", design), ("tasks.md", tasks)]);
        summaries_of(&report_of(&store), "Consistency")
    }

    /// Ambiguity findings for a single delta spec under capability `auth`.
    fn ambiguity_report(delta: &str) -> AnalyzeReport {
        report_of(&store_with(&[("specs/auth/spec.md", delta)]))
    }

    // --- D1：design 標題的編號拆解（spec「Consistency 的 design 標題引用判定」
    //     的 Example 表「編號拆解」）---

    #[test]
    fn a_design_heading_splits_into_ordinal_and_body() {
        let cases: [(&str, Option<&str>, &str); 5] = [
            (
                "決策一：整個移除 listDepthLimit 擴充",
                Some("決策一"),
                "整個移除 listDepthLimit 擴充",
            ),
            ("D3: 搜尋列元件化", Some("D3"), "搜尋列元件化"),
            (
                "Decision 2 device code 分權責",
                Some("Decision 2"),
                "device code 分權責",
            ),
            (
                "索引 JSON 的形狀與推導規則",
                None,
                "索引 JSON 的形狀與推導規則",
            ),
            ("D4", Some("D4"), ""),
        ];
        for (heading, label, body) in cases {
            let (got_label, got_body) = split_heading_label(heading);
            assert_eq!(got_label.as_deref(), label, "標題 '{heading}' 的編號");
            assert_eq!(got_body, body, "標題 '{heading}' 的本文");
        }
    }

    // --- D1：本文或編號任一命中即算引用（spec「Consistency 的 design 標題引用
    //     判定」的四個 Scenario）---

    #[test]
    fn a_heading_body_found_in_tasks_counts_as_referenced() {
        // spec Scenario「本文命中即算引用」。
        let found = consistency_of(
            "### 決策一：整個移除 listDepthLimit 擴充\n",
            "## 2. 實作：整個移除 listDepthLimit 擴充\n",
        );
        assert!(found.is_empty(), "本文命中不得報 finding: {found:?}");
    }

    #[test]
    fn a_heading_ordinal_found_in_tasks_counts_as_referenced() {
        // spec Scenario「編號命中即算引用」。
        let found = consistency_of(
            "### D1 違規清單與聚合錯誤形狀\n",
            "- [ ] 1.1 彙整違規（design D1）\n",
        );
        assert!(found.is_empty(), "編號命中不得報 finding: {found:?}");
    }

    #[test]
    fn an_ordinal_followed_by_a_digit_is_not_a_reference() {
        // spec Scenario「編號後接數字不算命中」：D1 不得被 D12 命中。
        let found = consistency_of("### D1 違規清單與聚合錯誤形狀\n", "- [ ] 1.1 見 D12\n");
        assert_eq!(
            found,
            vec!["Design topic 'd1 違規清單與聚合錯誤形狀' not referenced in tasks".to_string()],
            "D12 不得算 D1 的引用"
        );
    }

    #[test]
    fn a_heading_without_an_ordinal_keeps_whole_string_matching() {
        // spec Scenario「無編號標題維持整串比對」。
        let design = "### 索引 JSON 的形狀與推導規則\n";
        let partial = consistency_of(design, "- [ ] 1.1 索引 JSON 的形狀\n");
        assert_eq!(
            partial,
            vec![
                "Design topic '索引 json 的形狀與推導規則' not referenced in tasks".to_string()
            ],
            "只含一半仍要報"
        );
        let whole = consistency_of(design, "- [ ] 1.1 索引 JSON 的形狀與推導規則\n");
        assert!(whole.is_empty(), "含整串不得報: {whole:?}");
    }

    // --- D3：具體值字元集（spec「Ambiguity 的具體值判定」的兩個 Scenario 與
    //     Example 表）---

    /// 一條 ADDED 需求帶單一 scenario，內文為 `body`、沒有 Example。
    fn scenario_is_abstract(body: &str) -> bool {
        let delta = format!(
            "## ADDED Requirements\n\n### Requirement: R\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** {body}\n"
        );
        ambiguity_report(&delta)
            .findings
            .iter()
            .any(|f| f.summary_msg.key == "ambAbstractScenario.summary")
    }

    #[test]
    fn concrete_values_cover_fullwidth_quotes_and_digits() {
        // spec Example 表「具體值判定」逐列。
        let cases: [(&str, bool); 6] = [
            ("回傳 3 筆結果", false),
            ("顯示「已封存」", false),
            ("顯示『完成』", false),
            ("第１頁", false),
            ("回傳三筆結果", true),
            ("顯示 '完成'", true),
        ];
        for (body, abstract_expected) in cases {
            assert_eq!(
                scenario_is_abstract(body),
                abstract_expected,
                "內文 '{body}' 的抽象判定"
            );
        }
    }

    #[test]
    fn a_fullwidth_quoted_scenario_reports_nothing() {
        // spec Scenario「全形引號內的字串算具體值」。
        assert!(!scenario_is_abstract("卡片顯示「已封存」標籤"));
    }

    #[test]
    fn a_plain_narrative_scenario_is_still_abstract() {
        // spec Scenario「純敘述仍判為抽象」。
        let delta = "## ADDED Requirements\n\n### Requirement: R\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** 顯示成功訊息\n";
        let f = ambiguity_report(delta)
            .findings
            .into_iter()
            .find(|f| f.summary_msg.key == "ambAbstractScenario.summary")
            .expect("報一筆 ambAbstractScenario");
        assert_eq!(f.severity, "Suggestion");
        assert_eq!(f.summary, "Scenario 's' has no concrete examples");
    }

    // --- D6：跨平台換行（design Risks「跨平台」）---

    #[test]
    fn crlf_line_endings_do_not_break_ordinal_matching() {
        // CRLF 的 design.md 與 tasks.md：`\r` 不得讓 D1 判定失準。
        let found = consistency_of(
            "### D1 違規清單與聚合錯誤形狀\r\n",
            "- [ ] 1.1 彙整違規（design D1）\r\n",
        );
        assert!(found.is_empty(), "CRLF 下編號命中不得報: {found:?}");

        let missed = consistency_of("### D1 違規清單與聚合錯誤形狀\r\n", "- [ ] 1.1 見 D12\r\n");
        assert_eq!(
            missed,
            vec!["Design topic 'd1 違規清單與聚合錯誤形狀' not referenced in tasks".to_string()],
            "CRLF 下 D12 仍不得算命中"
        );
    }

    // --- D5：本次不動的規則的回歸網（spec「面向的前置 artifact 與報告形狀」、
    //     「Coverage 規則」、「Gaps 規則」）---

    const ONE_REQ: &str = "## ADDED Requirements\n\n### Requirement: R\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n";

    fn dimension_status<'a>(report: &'a AnalyzeReport, name: &str) -> &'a DimensionStatus {
        report
            .dimensions
            .iter()
            .find(|d| d.dimension == name)
            .expect("面向存在")
    }

    #[test]
    fn consistency_is_skipped_when_design_is_missing() {
        // spec Scenario「缺 design 時 Consistency 跳過」。
        let store = store_with(&[
            ("proposal.md", "## Why\n\nBecause.\n"),
            ("tasks.md", "- [ ] 1.1 做事\n"),
            ("specs/auth/spec.md", ONE_REQ),
        ]);
        let report = report_of(&store);
        let con = dimension_status(&report, "Consistency");
        assert_eq!(con.status, "Skipped (insufficient artifacts)");
        assert_eq!(con.finding_count, 0);
        assert!(
            report.artifacts_missing.contains(&"design".to_string()),
            "artifacts_missing 含 design: {:?}",
            report.artifacts_missing
        );
    }

    #[test]
    fn an_empty_tasks_file_still_counts_as_present() {
        // spec Scenario「空的 tasks.md 仍算存在」：Consistency 不跳過，每個標題各報一筆。
        let store = store_with(&[
            ("design.md", "### 甲的形狀\n\n### 乙的形狀\n"),
            ("tasks.md", ""),
        ]);
        let report = report_of(&store);
        assert_eq!(dimension_status(&report, "Consistency").finding_count, 2);
        assert!(
            !report.artifacts_missing.contains(&"tasks".to_string()),
            "空檔仍算存在: {:?}",
            report.artifacts_missing
        );
    }

    #[test]
    fn a_requirement_name_inside_a_task_description_is_covered() {
        // spec Scenario「需求名以子字串命中任務描述」。
        let store = store_with(&[
            ("proposal.md", "## Why\n\nBecause.\n"),
            (
                "specs/auth/spec.md",
                "## ADDED Requirements\n\n### Requirement: CSV Export\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n",
            ),
            ("tasks.md", "- [ ] 1.1 Implement CSV exporter\n"),
        ]);
        let found = summaries_of(&report_of(&store), "Coverage");
        assert!(found.is_empty(), "命中不得報 covMissingTask: {found:?}");
    }

    #[test]
    fn a_requirement_name_only_in_a_group_heading_is_not_covered() {
        // spec Scenario「需求名只出現在群組標題不算命中」。
        let store = store_with(&[
            ("proposal.md", "## Why\n\nBecause.\n"),
            (
                "specs/auth/spec.md",
                "## ADDED Requirements\n\n### Requirement: CSV Export\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n",
            ),
            ("tasks.md", "## 2. CSV Export\n"),
        ]);
        let report = report_of(&store);
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "covMissingTask.summary")
            .expect("報一筆 covMissingTask");
        assert_eq!(f.severity, "Warning");
        assert_eq!(f.summary, "Requirement 'CSV Export' has no matching task");
        assert_eq!(f.location, "specs/auth/spec.md");
    }

    #[test]
    fn a_modified_requirement_absent_from_the_canonical_spec_is_a_gap() {
        // spec Scenario「MODIFIED 需求不在正典」。
        let store = store_with(&[
            ("proposal.md", "## Why\n\nBecause.\n"),
            (
                "specs/auth/spec.md",
                "## MODIFIED Requirements\n\n### Requirement: R9\n\nIt SHALL work harder.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n",
            ),
        ]);
        store.canonical.borrow_mut().insert(
            "auth".to_string(),
            "# auth Specification\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n".to_string(),
        );
        let report = report_of(&store);
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "gapModifiedNotFound.summary")
            .expect("報一筆 gapModifiedNotFound");
        assert_eq!(f.summary, "MODIFIED requirement 'R9' not found in main spec");
        assert_eq!(f.recommendation_msg.params.get("spec").map(String::as_str), Some("auth"));
    }

    #[test]
    fn a_missing_canonical_spec_is_reported_once_per_capability() {
        // spec Scenario「無正典時每 capability 一筆」。
        let store = store_with(&[
            ("proposal.md", "## Why\n\nBecause.\n"),
            (
                "specs/auth/spec.md",
                "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n\n### Requirement: R2\n\nIt SHALL work.\n\n#### Scenario: t\n\n- **THEN** 回傳 4 筆\n",
            ),
        ]);
        let report = report_of(&store);
        let hits: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.summary_msg.key == "gapNoMainSpec.summary")
            .collect();
        assert_eq!(hits.len(), 1, "2 條 MODIFIED 只報 1 筆: {hits:?}");
        assert_eq!(
            hits[0].summary,
            "MODIFIED requirements reference capability 'auth' but no main spec found"
        );
    }

    // --- D4：英文弱語氣詞改字邊界（spec「Ambiguity 的弱語氣詞偵測——英文樣式
    //     should、may、might、consider、possibly 以字邊界比對」）---

    #[test]
    fn english_weak_language_matches_on_word_boundaries() {
        // spec Example 表「字邊界判定」逐列。
        let cases: [(&str, Option<&str>); 6] = [
            ("it should lock", Some("should")),
            ("Should lock", Some("should")),
            ("the shoulder strap", None),
            ("a considerable delay", None),
            ("the mayor", None),
            ("outbdoor", Some("TBD")),
        ];
        for (line, pattern) in cases {
            assert_eq!(
                weak_pattern_in(line).as_deref(),
                pattern,
                "行 '{line}' 的弱語氣詞"
            );
        }
    }

    #[test]
    fn a_boundary_blocked_line_reports_no_weak_language() {
        // spec Scenario「字邊界擋住 shoulder」。
        let delta = "## ADDED Requirements\n\n### Requirement: R\n\nThe shoulder strap SHALL lock.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n";
        let found = summaries_of(&ambiguity_report(delta), "Ambiguity");
        assert!(found.is_empty(), "shoulder 不得命中: {found:?}");
    }

    #[test]
    fn a_standalone_should_still_reports() {
        // spec Scenario「獨立的 should 仍命中」。
        let delta = "## ADDED Requirements\n\n### Requirement: R\n\nThe strap should lock.\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n";
        let f = ambiguity_report(delta)
            .findings
            .into_iter()
            .find(|f| f.summary_msg.key == "ambWeakLanguage.summary")
            .expect("報一筆 ambWeakLanguage");
        assert_eq!(f.severity, "Suggestion");
        assert_eq!(f.summary, "Vague language 'should' found");
        assert_eq!(f.location, "specs/auth/spec.md:5");
    }

    #[test]
    fn cjk_weak_language_keeps_substring_matching() {
        // spec Scenario「CJK 樣式維持子字串——可能、應該、考慮」。
        assert_eq!(weak_pattern_in("系統盡可能鎖定").as_deref(), Some("可能"));
        assert_eq!(weak_pattern_in("這不可能發生"), None);
    }

    // --- D2：REMOVED 需求改查 Reason／Migration（spec「Ambiguity 的 scenario
    //     缺席與 REMOVED 需求檢查」的三個 Scenario）---

    const REMOVED_COMPLETE: &str = "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Reason**: Replaced by v2\n**Migration**: Use /api/v2/export\n";

    #[test]
    fn a_removed_requirement_with_reason_and_migration_is_clean() {
        // spec Scenario「REMOVED 需求齊備時零 finding」。
        let report = ambiguity_report(REMOVED_COMPLETE);
        let found = summaries_of(&report, "Ambiguity");
        assert!(found.is_empty(), "齊備的 REMOVED 需求不得報 finding: {found:?}");
    }

    #[test]
    fn a_removed_requirement_missing_migration_is_flagged() {
        // spec Scenario「REMOVED 需求缺 Migration」。
        let report = ambiguity_report(
            "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Reason**: Replaced by v2\n",
        );
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambRemovedNoReason.summary")
            .expect("報一筆 ambRemovedNoReason");
        assert_eq!(f.severity, "Warning");
        assert_eq!(f.summary, "REMOVED requirement 'Legacy export' has no **Migration**");
        assert_eq!(
            f.recommendation,
            "Add **Reason**: and **Migration**: lines under 'Legacy export'"
        );
        assert_eq!(f.summary_msg.params.get("req").map(String::as_str), Some("Legacy export"));
        assert_eq!(f.summary_msg.params.get("missing").map(String::as_str), Some("Migration"));
        assert_eq!(f.recommendation_msg.key, "ambRemovedNoReason.recommendation");
        assert_eq!(f.recommendation_msg.params, f.summary_msg.params);
    }

    #[test]
    fn a_removed_requirement_missing_both_lists_them_together() {
        let report = ambiguity_report("## REMOVED Requirements\n\n### Requirement: Legacy export\n\nGone.\n");
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambRemovedNoReason.summary")
            .expect("報一筆 ambRemovedNoReason");
        assert_eq!(
            f.summary,
            "REMOVED requirement 'Legacy export' has no **Reason** and **Migration**"
        );
        assert_eq!(
            f.summary_msg.params.get("missing").map(String::as_str),
            Some("Reason and Migration")
        );
    }

    #[test]
    fn an_added_requirement_without_scenarios_still_reports_no_scenario() {
        // spec Scenario「ADDED 需求無 scenario 仍報」。
        let report = ambiguity_report("## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n");
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambNoScenario.summary")
            .expect("報一筆 ambNoScenario");
        assert_eq!(f.summary, "Requirement 'Fresh' has no scenarios");
    }

    #[test]
    fn the_removed_finding_serializes_both_params() {
        // `--json` 契約：summary_msg.params 帶 req 與 missing 兩鍵。
        let report = ambiguity_report("## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Migration**: Use v2\n");
        let json = serde_json::to_value(&report).expect("報告可序列化");
        let params = json["findings"]
            .as_array()
            .expect("findings 是陣列")
            .iter()
            .find(|f| f["summary_msg"]["key"] == "ambRemovedNoReason.summary")
            .expect("找得到該 finding")["summary_msg"]["params"]
            .clone();
        assert_eq!(params["req"], "Legacy export");
        assert_eq!(params["missing"], "Reason");
    }

    #[test]
    fn the_wadpilot_false_positive_is_gone() {
        // 回溯來源：本文與編號同時出現在一行群組標題裡。
        let found = consistency_of(
            "### 決策一：整個移除 listDepthLimit 擴充\n",
            "## 2. 實作：整個移除 listDepthLimit 擴充（design 決策一）\n",
        );
        assert!(found.is_empty(), "wadpilot 實例必須零 finding: {found:?}");
    }
}

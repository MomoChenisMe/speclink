//! Four-dimension consistency analysis (Coverage / Consistency / Ambiguity / Gaps).

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

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
    /// The removal notes found in the requirement body (the lines before its
    /// first scenario) — only REMOVED requirements are held to them.
    notes: RemovalNotes,
}

#[derive(Default)]
struct RemovalNotes {
    reason: bool,
    migration: bool,
}

impl RemovalNotes {
    /// Which notes are missing, as the `missing` param value; `None` when both
    /// are present.
    fn missing(&self) -> Option<&'static str> {
        match (self.reason, self.migration) {
            (true, true) => None,
            (false, true) => Some("Reason"),
            (true, false) => Some("Migration"),
            (false, false) => Some("Reason and Migration"),
        }
    }
}

/// A removal-note line: `**Reason**`, `**Reason:**` or `**Reason：**` at the start
/// of the trimmed line (`**Reasoning**` is not one).
fn is_note_line(t: &str, word: &str) -> bool {
    t.strip_prefix("**")
        .and_then(|r| r.strip_prefix(word))
        .is_some_and(|r| r.starts_with("**") || r.starts_with(":**") || r.starts_with("：**"))
}

/// Concrete = an ASCII or fullwidth digit, a backtick, a double quote, or any
/// of the fullwidth quote marks 「」『』 (each mark counts on its own; no pairing
/// is checked). Single quotes and Chinese numerals (一、二、三…, which also open
/// everyday words) do NOT count.
fn is_concrete_char(c: char) -> bool {
    c.is_ascii_digit() || ('０'..='９').contains(&c) || matches!(c, '`' | '"' | '「' | '」' | '『' | '』')
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
                notes: RemovalNotes::default(),
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
                if req.scenarios.is_empty() {
                    req.notes.reason |= is_note_line(t, "Reason");
                    req.notes.migration |= is_note_line(t, "Migration");
                }
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

/// The Chinese numerals an ordinal may be written in (`決策一` … `決策十一`).
const CJK_NUMERALS: &str = "一二三四五六七八九十";

/// The ordinal prefix a design heading may carry (frozen by spec): `D<digits>`,
/// `Decision <digits>` or `決策<digits | run of 一..十>` — `D`/`Decision`
/// case-insensitive, digits ASCII only (`[0-9]`, not Unicode `\d`, so fullwidth
/// `D１` is not an ordinal — matching the ASCII-only guard in
/// [`ordinal_referenced`]) — then optional spaces, at most one colon (`:` or
/// `：`) and more spaces. Group 1 is the ordinal itself.
static ORDINAL_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i)^(d[0-9]+|decision\s*[0-9]+|決策(?:[0-9]+|[{CJK_NUMERALS}]+))\s*[:：]?\s*"
    ))
    .unwrap()
});

/// Split a design `### ` heading into its ordinal label and its body. The label
/// has its inner whitespace collapsed (`Decision  2` → `Decision 2`). A heading
/// with no recognised prefix yields `(None, whole heading)`; a bare ordinal
/// yields `(Some(label), "")`.
fn split_heading_label(heading: &str) -> (Option<String>, String) {
    let h = heading.trim();
    match ORDINAL_PREFIX.captures(h) {
        Some(caps) => {
            let label = caps[1].split_whitespace().collect::<Vec<_>>().join(" ");
            (Some(label), h[caps[0].len()..].trim().to_string())
        }
        None => (None, h.to_string()),
    }
}

/// Whether tasks.md references a design heading: its body appears anywhere in
/// the task text, or its ordinal appears as a standalone token. A heading with
/// no ordinal falls back to whole-string matching; a bare ordinal is matched as
/// an ordinal only (so `D4` is not satisfied by `D42`).
///
/// `tasks_lower` is the FULL lowercased tasks.md text — a prose mention outside
/// any checkbox still suppresses the finding (frozen rule).
fn heading_referenced(heading: &str, tasks_lower: &str) -> bool {
    match split_heading_label(heading) {
        (Some(label), body) => {
            (!body.is_empty() && tasks_lower.contains(&body.to_lowercase()))
                || ordinal_referenced(&label.to_lowercase(), tasks_lower)
        }
        (None, _) => tasks_lower.contains(&heading.trim().to_lowercase()),
    }
}

/// An ordinal counts as referenced only where no ASCII letter or digit sits on
/// either side (`D1` is neither `D12` nor the `d1` inside a task ULID) and no
/// Chinese numeral follows (`決策十` is not `決策十二`).
fn ordinal_referenced(label_lower: &str, tasks_lower: &str) -> bool {
    occurs_bounded(tasks_lower, label_lower, |before, after| {
        !before.is_some_and(|c| c.is_ascii_alphanumeric())
            && !after.is_some_and(|c| c.is_ascii_alphanumeric() || CJK_NUMERALS.contains(c))
    })
}

/// Whether `needle` occurs in `haystack` at a position whose neighbours
/// (`before`, `after`; `None` at either end of the text) pass `bounded`.
///
/// Hand-rolled rather than regex `\b` (which drift.rs uses for identifiers):
/// `\b` counts digits and `_` as word characters and has no meaning next to CJK,
/// while each caller here needs its own idea of a boundary.
fn occurs_bounded(
    haystack: &str,
    needle: &str,
    bounded: impl Fn(Option<char>, Option<char>) -> bool,
) -> bool {
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(needle) {
        let at = from + pos;
        let end = at + needle.len();
        if bounded(haystack[..at].chars().next_back(), haystack[end..].chars().next()) {
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

/// An English weak word counts only with no ASCII letter on either side —
/// except the `n't` contraction, so `shouldn't` still flags `should`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    occurs_bounded(haystack, needle, |before, after| {
        !before.is_some_and(|c| c.is_ascii_alphabetic())
            && !after.is_some_and(|c| c.is_ascii_alphabetic())
    }) || occurs_bounded(haystack, &format!("{needle}n't"), |before, _| {
        !before.is_some_and(|c| c.is_ascii_alphabetic())
    })
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
                    if let Some(missing) = req.notes.missing() {
                        let listed = missing.replace("Reason", "**Reason**").replace("Migration", "**Migration**");
                        n += 1;
                        ambiguity.push(make_finding(
                            "AMB", n, "Ambiguity", Severity::Warning, loc,
                            &format!("REMOVED requirement '{}' has no {listed}", req.name),
                            &format!("Add **Reason**: and **Migration**: lines under '{}'", req.name),
                            "ambRemovedNoNotes",
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
        let cases: [(&str, Option<&str>, &str); 7] = [
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
            // 標題內部多餘空白收成一個，tasks 才找得到。
            ("Decision  2 device code", Some("Decision 2"), "device code"),
            // 連續的中文數字是同一個編號。
            ("決策十一：整個移除", Some("決策十一"), "整個移除"),
        ];
        for (heading, label, body) in cases {
            let (got_label, got_body) = split_heading_label(heading);
            assert_eq!(got_label.as_deref(), label, "標題 '{heading}' 的編號");
            assert_eq!(got_body, body, "標題 '{heading}' 的本文");
        }
    }

    #[test]
    fn a_fullwidth_digit_is_not_an_ordinal() {
        // `D<數字>` 只認 ASCII 數字：全形 `D１` 不當編號，整串退回子字串比對，
        // 所以守衛（只擋 ASCII 英數）與辨識面判準一致。
        let cases: [(&str, Option<&str>, &str); 2] = [
            ("D１ 全形編號", None, "D１ 全形編號"),
            ("決策１：全形編號", None, "決策１：全形編號"),
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
        // 對照組：同一份 design，tasks 換成真正的 D1 引用就不報——證明上面那筆
        // 是數字守衛擋下的，不是本文沒對上。
        let control = consistency_of("### D1 違規清單與聚合錯誤形狀\n", "- [ ] 1.1 見 D12 與 D1\n");
        assert!(control.is_empty(), "D1 本身仍要算引用: {control:?}");
    }

    #[test]
    fn an_ordinal_inside_an_identifier_is_not_a_reference() {
        // 編號前後都不得是 ASCII 字母或數字：tasks.md 每行都帶的 ULID 註解、
        // 檔名 card1 之類都不算 D1 的引用。
        for tasks in [
            "- [ ] 1.1 彙整違規 <!-- speclink-task:tsk_01M22B3PGGD1XQ8R -->\n",
            "- [ ] 1.1 重畫 card1 元件\n",
        ] {
            let found = consistency_of("### D1 違規清單與聚合錯誤形狀\n", tasks);
            assert_eq!(found.len(), 1, "識別符內的 d1 不得算引用: {tasks:?} → {found:?}");
        }
    }

    #[test]
    fn a_chinese_numeral_run_is_one_ordinal() {
        // 決策十一 不得被切成 決策十＋一；決策十 也不得被 決策十二 命中。
        let design = "### 決策十一：整個移除 listDepthLimit 擴充\n";
        let hit = consistency_of(design, "- [ ] 1.1 拆除擴充（design 決策十一）\n");
        assert!(hit.is_empty(), "決策十一 要被整個認成編號: {hit:?}");
        let miss = consistency_of("### 決策十：拆分模組\n", "- [ ] 1.1 見決策十二\n");
        assert_eq!(miss.len(), 1, "決策十二 不得算 決策十 的引用: {miss:?}");
    }

    #[test]
    fn a_bare_ordinal_heading_keeps_the_digit_guard() {
        // spec Example 表「編號拆解」第 5 列：本文為空時只比編號，守衛照樣生效。
        let missed = consistency_of("### D4\n", "- [ ] 1.1 見 D42\n");
        assert_eq!(missed.len(), 1, "D42 不得算 D4 的引用: {missed:?}");
        let hit = consistency_of("### D4\n", "- [ ] 1.1 見 (design D4)\n");
        assert!(hit.is_empty(), "D4 本身要算引用: {hit:?}");
        // 空本文帶冒號的標題也不逼 tasks 抄冒號。
        let colon = consistency_of("### 決策一：\n", "- [ ] 1.1 見決策一）\n");
        assert!(colon.is_empty(), "決策一： 只比編號: {colon:?}");
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
    fn requirement_names_match_task_descriptions_as_contiguous_substrings() {
        // spec Example 表「命中判定」逐列。
        let cases: [(&str, bool); 3] = [
            ("Implement csv export", true),
            ("Implement csv-export", false),
            ("Export CSV", false),
        ];
        for (task, covered) in cases {
            assert_eq!(
                req_covered("CSV Export", &[task.to_string()]),
                covered,
                "任務描述 '{task}' 對 'CSV Export' 的命中"
            );
        }
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
        let cases: [(&str, Option<&str>); 9] = [
            ("it should lock", Some("should")),
            ("Should lock", Some("should")),
            ("the shoulder strap", None),
            ("a considerable delay", None),
            ("the mayor", None),
            ("outbdoor", Some("TBD")),
            // 字邊界的代價（design D4 刻意接受）：maybe／considered 不再命中。
            ("maybe lock", None),
            ("considered done", None),
            // 撇號不是字母，shouldn't 的 should 照樣命中。
            ("it shouldn't lock", Some("should")),
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
    fn removal_note_lines_accept_colon_variants() {
        // `**Reason:**` 與全形 `**Migration：**` 都是本 repo 實際出現的寫法。
        let report = ambiguity_report(
            "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Reason:** Replaced by v2\n**Migration：** Use v2\n",
        );
        let found = summaries_of(&report, "Ambiguity");
        assert!(found.is_empty(), "冒號在粗體內也要認: {found:?}");
        // 但 `**Reasoning**` 不算。
        let report = ambiguity_report(
            "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Reasoning** Replaced by v2\n**Migration**: Use v2\n",
        );
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambRemovedNoNotes.summary")
            .expect("Reasoning 不算 Reason");
        assert_eq!(f.summary_msg.params.get("missing").map(String::as_str), Some("Reason"));
    }

    #[test]
    fn a_removed_requirement_with_scenarios_is_still_held_to_its_notes() {
        // REMOVED 需求帶 scenario 不違規，也不需要 scenario；但 Reason／Migration
        // 只認需求本文（scenario 之前）的行。
        let clean = ambiguity_report(
            "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Reason**: Replaced\n**Migration**: Use v2\n\n#### Scenario: s\n\n- **THEN** 回傳 3 筆\n",
        );
        let found = summaries_of(&clean, "Ambiguity");
        assert!(found.is_empty(), "帶 scenario 的 REMOVED 需求齊備時零 finding: {found:?}");
        let inside = ambiguity_report(
            "## REMOVED Requirements\n\n### Requirement: Legacy export\n\n**Migration**: Use v2\n\n#### Scenario: s\n\n**Reason**: 回傳 3 筆\n",
        );
        let f = inside
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambRemovedNoNotes.summary")
            .expect("scenario 內的 **Reason** 不算");
        assert_eq!(f.summary_msg.params.get("missing").map(String::as_str), Some("Reason"));
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
            .find(|f| f.summary_msg.key == "ambRemovedNoNotes.summary")
            .expect("報一筆 ambRemovedNoNotes");
        assert_eq!(f.severity, "Warning");
        assert_eq!(f.summary, "REMOVED requirement 'Legacy export' has no **Migration**");
        assert_eq!(
            f.recommendation,
            "Add **Reason**: and **Migration**: lines under 'Legacy export'"
        );
        assert_eq!(f.summary_msg.params.get("req").map(String::as_str), Some("Legacy export"));
        assert_eq!(f.summary_msg.params.get("missing").map(String::as_str), Some("Migration"));
        assert_eq!(f.recommendation_msg.key, "ambRemovedNoNotes.recommendation");
        assert_eq!(f.recommendation_msg.params, f.summary_msg.params);
    }

    #[test]
    fn a_removed_requirement_missing_both_lists_them_together() {
        let report = ambiguity_report("## REMOVED Requirements\n\n### Requirement: Legacy export\n\nGone.\n");
        let f = report
            .findings
            .iter()
            .find(|f| f.summary_msg.key == "ambRemovedNoNotes.summary")
            .expect("報一筆 ambRemovedNoNotes");
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
            .find(|f| f["summary_msg"]["key"] == "ambRemovedNoNotes.summary")
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

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
mod tests;

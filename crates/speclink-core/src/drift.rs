//! Drift detection between a change and the current codebase.

use crate::model::Change;
use crate::preflight::days_old;
use crate::store::Store;
use regex::Regex;
use serde::Serialize;
// `analyze` and its git-backed helpers below are retained only as the frozen
// parity oracle for the workspace-drift tests (the production drift path is the
// Host-collected client/server pipeline). Their git/workspace imports are
// therefore test-only.
#[cfg(test)]
use crate::util;
#[cfg(test)]
use crate::workspace::Workspace;

const ANCHOR_CAP: usize = 50;

/// Stopwords filtered out of symbol-anchor extraction (frozen word-by-word list).
// Rust type/keyword names and the GWT keywords are filtered out, but ordinary English words
// ("The", "Also", "Should", …) and — surprisingly — Eq/Ord/PartialEq/PartialOrd are KEPT.
const STOPWORDS: &[&str] = &[
    "Context", "State", "Result", "Error", "Option", "Vec", "Rust", "JSON", "CLI", "API",
    "Box", "String", "Self", "Ok", "Err", "Some", "None",
    "Display", "Default", "Debug", "Clone", "Copy",
    "From", "Into", "Iterator", "Send", "Sync", "Sized",
    "Given", "When", "Then",
    "Struct", "Enum", "Trait", "Type", "Path", "Value", "Item", "Fn",
];

#[derive(Debug, Serialize)]
pub struct BrokenAnchor {
    pub anchor: String,
    pub category: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct DriftDimension {
    pub kind: String,
    pub status: String,
    pub score: i64,
    pub contributes_to_total: bool,
}

/// A delta-spec operation the archive merge gate would refuse — the engine's own
/// `MergeViolation`, re-exported under drift's vocabulary so the two dimensions can
/// never disagree. The serialized shape is unchanged; only `operation`'s value domain
/// widened (a comma-joined list on a multi-section collision).
pub use crate::archive::MergeViolation as SpecAssumption;

/// Filesystem kind of a probed path (Tasks dimension & path anchors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathKind {
    /// Path does not exist.
    Missing,
    /// A regular file.
    File,
    /// Exists but is not a regular file (directory, symlink, …).
    Other,
}

/// The closed set of local git/worktree facts the workspace-side drift
/// computation consumes. The Host collects these; [`compute_workspace_drift`]
/// reads only this struct for its git/fs inputs and never runs git itself.
/// Each git-derived field keeps the current three-value semantics — a value,
/// empty, or unavailable — so the existing "git unavailable" fallbacks
/// reproduce byte-for-byte. `None` on the git-backed fields means unavailable
/// (git subprocess failed / not a work tree); `Some(empty)` means available
/// but no data.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceFacts {
    /// `git log --since` window as per-commit file lists. `None` = git
    /// unavailable (drives the no-commits fallback); `Some(empty)` = available
    /// with zero commits; `Some([..])` = commits in the window.
    pub commit_window: Option<Vec<Vec<String>>>,
    /// Work-tree contents of tracked `*.md`/`*.txt` docs (the change's own
    /// directory excluded) — the in-process half of symbol resolution. `None`
    /// = `git ls-files` unavailable; `Some(empty)` = no such docs.
    pub tracked_docs: Option<Vec<String>>,
    /// Anchor symbols found via `git grep HEAD` — the committed half of symbol
    /// resolution. `None` = git grep unavailable; `Some(names)` = the found
    /// subset. Combined with `tracked_docs` to reproduce `symbol_found`.
    pub symbol_head_hits: Option<Vec<String>>,
    /// Filesystem kind of paths referenced by tasks and by path anchors, for
    /// the Tasks/Structure exists/is_file probes. An absent key means the path
    /// was not probed.
    pub path_status: std::collections::BTreeMap<String, PathKind>,
    /// Files recorded in the change's touched/evidence record — the
    /// Environment dimension's relevance set.
    pub touched_files: Vec<String>,
}

/// Spec-side drift: the Specs dimension plus the stale delta-spec assumptions,
/// computed from Store spec facts alone. Merged into the combined report by
/// [`merge_drift_reports`]; the assumptions also steer the recommendation.
#[derive(Debug)]
pub struct SpecDriftReport {
    /// The Specs dimension (`kind = "Specs"`, contributes to the total).
    pub dimension: DriftDimension,
    /// Stale delta-spec assumptions surfaced for this change.
    pub spec_assumptions: Vec<SpecAssumption>,
}

/// One workspace-side dimension, either scored or unavailable. A checkout
/// absence (no [`WorkspaceFacts`]) yields `Unavailable` — distinct from a
/// zero-score `Available`, so the merger can honour the "unavailable is not
/// clean and not a score" rule.
#[derive(Debug)]
pub enum WorkspaceDimension {
    Available(DriftDimension),
    Unavailable { kind: String },
}

/// Workspace-side drift: the four client dimensions (Time, Structure, Tasks,
/// Environment, in that order) plus the anchor and task findings, computed
/// from a [`WorkspaceFacts`] snapshot with no git access. When facts are
/// absent every dimension is `Unavailable`.
#[derive(Debug)]
pub struct WorkspaceDriftReport {
    /// Time, Structure, Tasks, Environment — in that order.
    pub dimensions: Vec<WorkspaceDimension>,
    pub broken_anchors: Vec<BrokenAnchor>,
    pub tasks_maybe_resolved: Vec<String>,
    pub tasks_blocked_external: Vec<String>,
    pub commits_since_created: i64,
    /// Broken-anchor ratio, for the merger's severity rule. 0.0 when there are
    /// no anchors or when facts are absent.
    pub broken_ratio: f64,
}

#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub change_id: String,
    pub created: Option<String>,
    pub last_commit: Option<String>,
    pub dimensions: Vec<DriftDimension>,
    pub broken_anchors: Vec<BrokenAnchor>,
    pub tasks_maybe_resolved: Vec<String>,
    pub tasks_blocked_external: Vec<String>,
    pub spec_assumptions: Vec<SpecAssumption>,
    pub commits_since_created: i64,
    pub total_score: i64,
    pub severity: String,
    pub primary_recommendation: String,
}

/// How much of the drift picture a combined report covers. `Full` is the
/// normal (and only local) case and is omitted from serialization; `SpecOnly`
/// appears when the workspace side is unavailable (no checkout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Coverage {
    Full,
    SpecOnly,
}

impl Coverage {
    /// Full coverage is the frozen local case — skipped in serialization so
    /// the local `drift --json` output stays byte-for-byte unchanged.
    fn is_full(&self) -> bool {
        matches!(self, Coverage::Full)
    }
}

/// One of the three verification bases a stale drift report can cite, in the
/// fixed spec → tasks → policy order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DriftBasisItem {
    Spec,
    Tasks,
    Policy,
}

/// Basis comparison for the merger: the digests a bundle fixed (`expected`)
/// versus the change's current-state digests (`current`). Any mismatch marks
/// the combined report stale — mixed-basis reports are never emitted silently.
#[derive(Debug, Clone)]
pub struct DriftBasis {
    pub expected: crate::tasks::BasisDigests,
    pub current: crate::tasks::BasisDigests,
}

/// The stale marker: which basis items no longer match. Present only when at
/// least one item mismatches.
#[derive(Debug, Clone, Serialize)]
pub struct StaleInfo {
    pub mismatched: Vec<DriftBasisItem>,
}

/// The single merged drift report: the frozen [`DriftReport`] shape plus two
/// optional markers. On the full-coverage, non-stale (local) path both markers
/// are omitted, so serialization is byte-identical to the current report.
#[derive(Debug, Serialize)]
pub struct CombinedDriftReport {
    #[serde(flatten)]
    pub report: DriftReport,
    /// Coverage marker, omitted when `Full`.
    #[serde(skip_serializing_if = "Coverage::is_full")]
    pub coverage: Coverage,
    /// Stale marker, omitted when the basis matches (or no basis was given).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<StaleInfo>,
}

/// One design anchor: a code-like symbol, or a file-path reference (checked for existence
/// instead of grepped).
struct Anchor {
    name: String,
    is_path: bool,
}

/// Whether a token looks like code rather than prose: snake_case / SCREAMING_CASE (any
/// underscore), camelCase, or multi-hump PascalCase (`DriftReport`). Single capitalized
/// prose words (`Decisions`), plain acronyms (`CSV`), and heading labels (`D1`) do not.
fn code_like(token: &str) -> bool {
    if token.contains('_') && token.chars().any(|c| c.is_ascii_alphanumeric()) {
        return true;
    }
    let camel = Regex::new(r"^[a-z][a-z0-9]*[A-Z][A-Za-z0-9]*$").unwrap();
    let pascal_multi = Regex::new(r"^(?:[A-Z][a-z0-9]+){2,}$").unwrap();
    camel.is_match(token) || pascal_multi.is_match(token)
}

/// Anchor extraction, the second deliberate design decision here: a bare
/// `\b[A-Z]\w+\b` prose scan surfaces prose capitalized words (headings, sentence starts)
/// as broken-anchor noise once the change's own directory is excluded from the corpus. Anchors are therefore restricted to code-like tokens anywhere in the
/// design plus backtick spans, and backticked file paths become existence-checked anchors.
fn extract_anchors(design: &str) -> Vec<Anchor> {
    // name -> is_path; BTreeMap keeps the output order stable.
    let mut seen: std::collections::BTreeMap<String, bool> = std::collections::BTreeMap::new();

    // Code-like identifiers anywhere in the prose (includes backtick span contents).
    let ident = Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap();
    for m in ident.find_iter(design) {
        let w = m.as_str();
        if code_like(w) && !STOPWORDS.contains(&w) {
            seen.entry(w.to_string()).or_insert(false);
        }
    }

    // Backtick spans: a whitespace-free span containing '/' is a file-path anchor
    // (`hr/index.html`, trailing `:42` line refs stripped); otherwise the leading
    // identifier of a code expression counts when it is code-like (`pressKey(code)`).
    let span = Regex::new(r"`([^`]+)`").unwrap();
    let leading = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*").unwrap();
    let line_ref = Regex::new(r":\d+$").unwrap();
    for m in span.captures_iter(design) {
        let content = m[1].trim();
        if content.contains('/') && !content.contains(char::is_whitespace) {
            let path = line_ref
                .replace(content.trim_start_matches("./"), "")
                .trim_end_matches('/')
                .to_string();
            if !path.is_empty() {
                seen.insert(path.replace('\\', "/"), true);
            }
            continue;
        }
        if let Some(l) = leading.find(content) {
            let head = l.as_str();
            if code_like(head) && !STOPWORDS.contains(&head) {
                seen.entry(head.to_string()).or_insert(false);
            }
        }
    }

    seen.into_iter()
        .map(|(name, is_path)| Anchor { name, is_path })
        .take(ANCHOR_CAP)
        .collect()
}

/// The design anchors a Host-side collector resolves against the worktree:
/// `(name, is_path)` — path anchors are existence-checked, symbol anchors are
/// grepped. Exposed so the collector drives the exact anchor set
/// [`compute_workspace_drift`] later consumes (both derive from the same
/// design text, so they never disagree).
pub fn design_anchors(design: &str) -> Vec<(String, bool)> {
    extract_anchors(design).into_iter().map(|a| (a.name, a.is_path)).collect()
}

/// Work-tree contents of tracked `*.md` / `*.txt` documents (via `git ls-files`), excluding
/// the change's own directory so a committed design.md cannot self-satisfy its anchors.
#[cfg(test)]
fn tracked_doc_contents(ws: &Workspace, exclude_prefix: &str) -> Vec<String> {
    let Some(list) = util::git(&ws.root, &["ls-files"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in list.lines() {
        let f = line.trim();
        if f.starts_with(exclude_prefix) {
            continue;
        }
        if f.ends_with(".md") || f.ends_with(".txt") {
            if let Some(content) = util::read_opt(&ws.root.join(f)) {
                out.push(content);
            }
        }
    }
    out
}

/// Whether an anchor matches (whole-word, case-sensitive) in the search corpus: the committed
/// content of tracked files (HEAD) plus the work-tree content of tracked markdown/text
/// documents. Deliberate design: the change's own directory is excluded, so broken anchors
/// keep working after the design is committed (a corpus including the design itself would
/// make Structure permanently silent post-commit).
#[cfg(test)]
fn symbol_found(ws: &Workspace, doc_contents: &[String], exclude_prefix: &str, symbol: &str) -> bool {
    // ASCII word boundary, matching `git grep --word-regexp` semantics.
    let re = Regex::new(&format!(r"(?-u:\b){}(?-u:\b)", regex::escape(symbol)));
    if let Ok(re) = re {
        if doc_contents.iter().any(|c| re.is_match(c)) {
            return true;
        }
    }
    let exclude = format!(":(exclude){exclude_prefix}");
    util::git(
        &ws.root,
        &["grep", "-q", "--word-regexp", "--fixed-strings", symbol, "HEAD", "--", &exclude],
    )
    .is_some()
}

/// Delta-spec operations archive would refuse to merge — the same judgement the archive
/// engine gates on and bulk archive pre-filters with, so drift, the pre-check and a single
/// archive can never disagree (spec archive-merge「過期判定單源共用」).
pub fn spec_assumptions(store: &dyn Store, change: &Change) -> Vec<SpecAssumption> {
    crate::archive::merge_violations(store, &change.name)
}

/// Spec-side drift computation: consumes only Store spec facts (the change's
/// delta capabilities and the canonical specs). Runs no git and performs no
/// workspace I/O, so it is reproducible for a fixed Store snapshot — the
/// Server-side half of the client/server drift split.
pub fn compute_spec_drift(store: &dyn Store, change: &Change) -> SpecDriftReport {
    let delta_caps = store.delta_capabilities(&change.name);
    let spec_assumptions = spec_assumptions(store, change);
    let specs_status = if delta_caps.is_empty() {
        "no delta specs".to_string()
    } else if spec_assumptions.is_empty() {
        "delta assumptions hold".to_string()
    } else {
        format!("{} stale assumptions", spec_assumptions.len())
    };
    let specs_score = std::cmp::min(4 * spec_assumptions.len() as i64, 9);
    SpecDriftReport {
        dimension: DriftDimension {
            kind: "Specs".to_string(),
            status: specs_status,
            score: specs_score,
            contributes_to_total: true,
        },
        spec_assumptions,
    }
}

/// Whole-word (case-sensitive) match of an anchor against a [`WorkspaceFacts`]
/// snapshot: the in-process regex over `tracked_docs` (git ls-files half) OR
/// membership in `symbol_head_hits` (git grep HEAD half) — reproducing the
/// original `symbol_found` without touching git.
fn symbol_found_facts(facts: &WorkspaceFacts, symbol: &str) -> bool {
    // ASCII word boundary, matching `git grep --word-regexp` semantics.
    if let Ok(re) = Regex::new(&format!(r"(?-u:\b){}(?-u:\b)", regex::escape(symbol))) {
        if facts.tracked_docs.as_deref().unwrap_or(&[]).iter().any(|c| re.is_match(c)) {
            return true;
        }
    }
    facts.symbol_head_hits.as_deref().unwrap_or(&[]).iter().any(|s| s == symbol)
}

/// Workspace-side drift computation: the four client dimensions from a
/// [`WorkspaceFacts`] snapshot (plus the change's design/tasks/created read
/// from the Store). Runs no git and performs no direct filesystem probes —
/// every git/fs signal arrives through `facts`. `facts = None` (no checkout)
/// marks all four dimensions unavailable.
pub fn compute_workspace_drift(
    store: &dyn Store,
    change: &Change,
    facts: Option<&WorkspaceFacts>,
) -> WorkspaceDriftReport {
    let Some(facts) = facts else {
        return WorkspaceDriftReport {
            dimensions: ["Time", "Structure", "Tasks", "Environment"]
                .iter()
                .map(|k| WorkspaceDimension::Unavailable { kind: k.to_string() })
                .collect(),
            broken_anchors: Vec::new(),
            tasks_maybe_resolved: Vec::new(),
            tasks_blocked_external: Vec::new(),
            commits_since_created: 0,
            broken_ratio: 0.0,
        };
    };

    let design = store.read_artifact(&change.name, "design.md").unwrap_or_default();
    let anchors = extract_anchors(&design);
    let total_anchors = anchors.len();

    // Commit window: Some => git available (empty = zero commits); None => git unavailable.
    let git_has_commits = facts.commit_window.is_some();
    let commit_files: &[Vec<String>] = facts.commit_window.as_deref().unwrap_or(&[]);

    // Broken anchors (Structure): path anchors via fs status, symbol anchors via facts.
    let mut broken = Vec::new();
    for a in &anchors {
        let found = if a.is_path {
            facts.path_status.get(&a.name).is_some_and(|k| *k != PathKind::Missing)
        } else {
            symbol_found_facts(facts, &a.name)
        };
        if !found {
            broken.push(if a.is_path {
                BrokenAnchor {
                    anchor: a.name.clone(),
                    category: "File".to_string(),
                    reason: "file not found in repo".to_string(),
                }
            } else {
                BrokenAnchor {
                    anchor: a.name.clone(),
                    category: "Symbol".to_string(),
                    reason: "symbol not found in repo".to_string(),
                }
            });
        }
    }
    let broken_ratio = if total_anchors == 0 {
        0.0
    } else {
        broken.len() as f64 / total_anchors as f64
    };

    let days = days_old(change.meta.created.as_deref());

    // Time dimension
    let time_status = if change.meta.created.is_none() {
        "no created date".to_string()
    } else if git_has_commits {
        if days > 5 {
            format!("stale ({days}d)")
        } else {
            format!("fresh ({days}d)")
        }
    } else if days > 5 {
        format!("stale ({days}d), git unavailable")
    } else {
        format!("fresh ({days}d), git unavailable")
    };
    let time_score = if days > 14 {
        3
    } else if days > 5 {
        1
    } else {
        0
    };

    // Structure dimension — "design absent" when there is no design.md to anchor on.
    let design_present = store.artifact_exists(&change.name, "design.md");
    let (structure_status, structure_score) = if !design_present {
        ("design absent".to_string(), 0)
    } else {
        let score = if broken_ratio >= 0.5 {
            4
        } else if broken_ratio >= 0.25 {
            3
        } else if broken_ratio > 0.0 {
            2
        } else {
            0
        };
        (format!("{}/{} anchors broken", broken.len(), total_anchors), score)
    };

    // Tasks dimension — an unchecked task is "maybe-done" when a file it references was
    // committed in the window and still exists as a file; "blocked" when a referenced file
    // was touched in the window and is now gone.
    let mut tasks_maybe_resolved: Vec<String> = Vec::new();
    let mut tasks_blocked_external: Vec<String> = Vec::new();
    let tasks_present = store.artifact_exists(&change.name, "tasks.md");
    let task_list =
        crate::tasks::parse(&store.read_artifact(&change.name, "tasks.md").unwrap_or_default());
    let window_files: std::collections::BTreeSet<&str> =
        commit_files.iter().flatten().map(|s| s.as_str()).collect();
    for t in &task_list {
        if t.done {
            continue;
        }
        let refs = task_file_refs(&t.description);
        if refs.is_empty() {
            continue;
        }
        let vanished = refs.iter().any(|r| {
            window_files.contains(r.as_str())
                && facts.path_status.get(r) == Some(&PathKind::Missing)
        });
        if vanished {
            tasks_blocked_external.push(t.description.clone());
        } else if refs.iter().any(|r| {
            window_files.contains(r.as_str())
                && facts.path_status.get(r) == Some(&PathKind::File)
        }) {
            tasks_maybe_resolved.push(t.description.clone());
        }
    }
    let tasks_status = if !tasks_present {
        "no tasks.md".to_string()
    } else if git_has_commits {
        format!(
            "{} blocked, {} maybe-done",
            tasks_blocked_external.len(),
            tasks_maybe_resolved.len()
        )
    } else {
        "git unavailable".to_string()
    };
    let tasks_score = 0;

    // Environment dimension (display only): commits in the window, plus how many touch files
    // this change cares about (touched record ∪ task references).
    let commits_since = commit_files.len() as i64;
    let mut relevant: std::collections::BTreeSet<String> =
        facts.touched_files.iter().cloned().collect();
    for t in &task_list {
        relevant.extend(task_file_refs(&t.description));
    }
    let env_status = if relevant.is_empty() || commits_since == 0 {
        format!("{commits_since} commits")
    } else {
        let touching = commit_files
            .iter()
            .filter(|files| files.iter().any(|f| relevant.contains(f)))
            .count();
        format!("{commits_since} commits ({touching} touching this change's files)")
    };

    let dimensions = vec![
        WorkspaceDimension::Available(DriftDimension {
            kind: "Time".to_string(),
            status: time_status,
            score: time_score,
            contributes_to_total: true,
        }),
        WorkspaceDimension::Available(DriftDimension {
            kind: "Structure".to_string(),
            status: structure_status,
            score: structure_score,
            contributes_to_total: true,
        }),
        WorkspaceDimension::Available(DriftDimension {
            kind: "Tasks".to_string(),
            status: tasks_status,
            score: tasks_score,
            contributes_to_total: true,
        }),
        WorkspaceDimension::Available(DriftDimension {
            kind: "Environment".to_string(),
            status: env_status,
            score: 0,
            contributes_to_total: false,
        }),
    ];

    WorkspaceDriftReport {
        dimensions,
        broken_anchors: broken,
        tasks_maybe_resolved,
        tasks_blocked_external,
        commits_since_created: commits_since,
        broken_ratio,
    }
}

/// Merge the spec-side and workspace-side reports into the single combined
/// report — the one place scoring, coverage, and staleness are adjudicated
/// (CLI, Node and Desktop never re-implement these). Full coverage reproduces
/// the current [`DriftReport`] field-for-field; an unavailable workspace side
/// yields spec-only coverage with the four dimensions kept as unavailable
/// entries; a mismatched `basis` marks the report stale, listing only the
/// items that differ.
pub fn merge_drift_reports(
    change: &Change,
    spec: SpecDriftReport,
    workspace: WorkspaceDriftReport,
    basis: Option<&DriftBasis>,
) -> CombinedDriftReport {
    let SpecDriftReport { dimension: spec_dim, spec_assumptions } = spec;
    let WorkspaceDriftReport {
        dimensions: wdims,
        broken_anchors,
        tasks_maybe_resolved,
        tasks_blocked_external,
        commits_since_created,
        broken_ratio,
    } = workspace;

    let coverage = if wdims.iter().all(|d| matches!(d, WorkspaceDimension::Available(_))) {
        Coverage::Full
    } else {
        Coverage::SpecOnly
    };

    // Flatten each workspace dimension to a DriftDimension; an unavailable one becomes an
    // "unavailable" entry that is excluded from the total (never counted as clean or a score).
    let flatten = |wd: WorkspaceDimension| match wd {
        WorkspaceDimension::Available(d) => d,
        WorkspaceDimension::Unavailable { kind } => DriftDimension {
            kind,
            status: "unavailable".to_string(),
            score: 0,
            contributes_to_total: false,
        },
    };
    // Frozen order: Time, Structure, Tasks, Specs, Environment — Specs sits between the
    // workspace Tasks and Environment dimensions.
    let mut wit = wdims.into_iter();
    let time = flatten(wit.next().expect("Time dimension"));
    let structure = flatten(wit.next().expect("Structure dimension"));
    let tasks = flatten(wit.next().expect("Tasks dimension"));
    let environment = flatten(wit.next().expect("Environment dimension"));
    let dimensions = vec![time, structure, tasks, spec_dim, environment];

    let total_score: i64 = dimensions
        .iter()
        .filter(|d| d.contributes_to_total)
        .map(|d| d.score)
        .sum();

    let severity = if total_score > 8 || broken_ratio > 0.30 {
        "heavy"
    } else if total_score >= 4 {
        "medium"
    } else {
        "light"
    }
    .to_string();

    // Stale delta assumptions always route to ingest first: the archive merge gate
    // refuses them (--skip-specs skips spec application entirely).
    let primary_recommendation = if !spec_assumptions.is_empty() {
        format!("/speclink-ingest {}", change.name)
    } else {
        match severity.as_str() {
            "heavy" => format!("speclink archive {} --skip-specs", change.name),
            "medium" => format!("/speclink-ingest {}", change.name),
            _ => format!("/speclink-apply {}", change.name),
        }
    };

    let stale = basis.and_then(|b| {
        let mut mismatched = Vec::new();
        if b.expected.spec != b.current.spec {
            mismatched.push(DriftBasisItem::Spec);
        }
        if b.expected.tasks != b.current.tasks {
            mismatched.push(DriftBasisItem::Tasks);
        }
        if b.expected.policy != b.current.policy {
            mismatched.push(DriftBasisItem::Policy);
        }
        (!mismatched.is_empty()).then_some(StaleInfo { mismatched })
    });

    CombinedDriftReport {
        report: DriftReport {
            change_id: change.name.clone(),
            created: change.meta.created.clone(),
            last_commit: None,
            dimensions,
            broken_anchors,
            tasks_maybe_resolved,
            tasks_blocked_external,
            spec_assumptions,
            commits_since_created,
            total_score,
            severity,
            primary_recommendation,
        },
        coverage,
        stale,
    }
}

/// Path-like backtick references in a task description (`bomberman/index.html` yes,
/// `reset(seed)` no). References are read from the checkbox line only. Public so
/// the Host collector stats the same task references the Tasks dimension probes.
pub fn task_file_refs(desc: &str) -> Vec<String> {
    let re = Regex::new(r"`([^`\s()]+)`").unwrap();
    re.captures_iter(desc)
        .map(|c| c[1].replace('\\', "/"))
        .filter(|s| s.contains('/') || s.contains('.'))
        .collect()
}

/// Parse the `--since` log output into per-commit file lists (paths are repo-relative,
/// forward-slashed, one commit per `COMMIT|` header). Public so the Host collector
/// builds `WorkspaceFacts::commit_window` from the raw git log.
pub fn parse_commit_files(log: &str) -> Vec<Vec<String>> {
    let mut commits: Vec<Vec<String>> = Vec::new();
    for line in log.lines() {
        if line.starts_with("COMMIT|") {
            commits.push(Vec::new());
        } else if !line.trim().is_empty() {
            if let Some(last) = commits.last_mut() {
                last.push(line.trim().replace('\\', "/"));
            }
        }
    }
    commits
}

/// Frozen parity oracle: the pre-split monolithic drift computation, retained
/// under `#[cfg(test)]` so the workspace-drift tests can assert the new pipeline
/// reproduces it field-for-field under the same clock. Not a production path —
/// the CLI runs Host-collected facts through `compute_*` + `merge_drift_reports`.
#[cfg(test)]
pub fn analyze(ws: &Workspace, store: &dyn Store, change: &Change) -> DriftReport {
    let design = store.read_artifact(&change.name, "design.md").unwrap_or_default();
    let anchors = extract_anchors(&design);
    let total_anchors = anchors.len();

    let git_ok = util::git_available(&ws.root);
    // Deliberate design: the created DATE is anchored to midnight. Passing the bare date
    // would let git's approxidate fill the missing time-of-day from the current clock —
    // making same-day changes always count 0 commits.
    let since_arg = match change.meta.created.as_deref() {
        Some(c) if !c.is_empty() => format!("--since={c} 00:00:00"),
        _ => "--since=".to_string(),
    };
    let since_log = if git_ok {
        util::git(
            &ws.root,
            &["log", &since_arg, "--pretty=format:COMMIT|%H|%at|%s", "--name-only"],
        )
    } else {
        None
    };
    // The log call fails in a repo with no commits — that is what the
    // "git unavailable" statuses key off, not `git_available`.
    let git_has_commits = since_log.is_some();
    let commit_files = parse_commit_files(since_log.as_deref().unwrap_or(""));
    // The CLI never populates last_commit (it is an app-side field).
    let last_commit: Option<String> = None;

    // The change's storage location relative to the project root, as a git
    // pathspec prefix (forward-slashed, trailing '/').
    let exclude_prefix = change
        .dir
        .strip_prefix(&ws.root)
        .map(|rel| format!("{}/", util::to_slash(rel)))
        .unwrap_or_else(|_| format!("{}/changes/{}/", ws.spec_dir_name, change.name));
    let doc_contents = tracked_doc_contents(ws, &exclude_prefix);
    let mut broken = Vec::new();
    for a in &anchors {
        let found = if a.is_path {
            ws.root.join(&a.name).exists()
        } else {
            symbol_found(ws, &doc_contents, &exclude_prefix, &a.name)
        };
        if !found {
            broken.push(if a.is_path {
                BrokenAnchor {
                    anchor: a.name.clone(),
                    category: "File".to_string(),
                    reason: "file not found in repo".to_string(),
                }
            } else {
                BrokenAnchor {
                    anchor: a.name.clone(),
                    category: "Symbol".to_string(),
                    reason: "symbol not found in repo".to_string(),
                }
            });
        }
    }
    let broken_ratio = if total_anchors == 0 {
        0.0
    } else {
        broken.len() as f64 / total_anchors as f64
    };

    let days = days_old(change.meta.created.as_deref());

    // Time dimension
    let time_status = if change.meta.created.is_none() {
        "no created date".to_string()
    } else if git_has_commits {
        if days > 5 {
            format!("stale ({days}d)")
        } else {
            format!("fresh ({days}d)")
        }
    } else if days > 5 {
        format!("stale ({days}d), git unavailable")
    } else {
        format!("fresh ({days}d), git unavailable")
    };
    let time_score = if days > 14 {
        3
    } else if days > 5 {
        1
    } else {
        0
    };

    // Structure dimension — falls back to "design absent" when there is no design.md to anchor on.
    let design_present = store.artifact_exists(&change.name, "design.md");
    let (structure_status, structure_score) = if !design_present {
        ("design absent".to_string(), 0)
    } else {
        let score = if broken_ratio >= 0.5 {
            4
        } else if broken_ratio >= 0.25 {
            3
        } else if broken_ratio > 0.0 {
            2
        } else {
            0
        };
        (format!("{}/{} anchors broken", broken.len(), total_anchors), score)
    };

    // Tasks dimension — real signals (deliberate design; the arrays are never left empty):
    // an unchecked task is "maybe-done" when a file it references was committed
    // in the window and exists; "blocked" when a referenced file was touched and is now gone.
    let mut tasks_maybe_resolved: Vec<String> = Vec::new();
    let mut tasks_blocked_external: Vec<String> = Vec::new();
    let tasks_present = store.artifact_exists(&change.name, "tasks.md");
    let task_list = crate::tasks::parse(
        &store.read_artifact(&change.name, "tasks.md").unwrap_or_default(),
    );
    let window_files: std::collections::BTreeSet<&str> = commit_files
        .iter()
        .flatten()
        .map(|s| s.as_str())
        .collect();
    for t in &task_list {
        if t.done {
            continue;
        }
        let refs = task_file_refs(&t.description);
        if refs.is_empty() {
            continue;
        }
        let vanished = refs
            .iter()
            .any(|r| window_files.contains(r.as_str()) && !ws.root.join(r).exists());
        if vanished {
            tasks_blocked_external.push(t.description.clone());
        } else if refs
            .iter()
            .any(|r| window_files.contains(r.as_str()) && ws.root.join(r).is_file())
        {
            tasks_maybe_resolved.push(t.description.clone());
        }
    }
    let tasks_status = if !tasks_present {
        "no tasks.md".to_string()
    } else if git_has_commits {
        format!(
            "{} blocked, {} maybe-done",
            tasks_blocked_external.len(),
            tasks_maybe_resolved.len()
        )
    } else {
        "git unavailable".to_string()
    };
    let tasks_score = 0;

    // Specs dimension (speclink-specific): do the delta's MODIFIED/REMOVED/RENAMED targets
    // still exist in the canonical specs, and would an ADDED requirement collide? The
    // archive merge gate refuses both cases — drift is where they surface early.
    let delta_caps = store.delta_capabilities(&change.name);
    let spec_assumptions = spec_assumptions(store, change);
    let specs_status = if delta_caps.is_empty() {
        "no delta specs".to_string()
    } else if spec_assumptions.is_empty() {
        "delta assumptions hold".to_string()
    } else {
        format!("{} stale assumptions", spec_assumptions.len())
    };
    let specs_score = std::cmp::min(4 * spec_assumptions.len() as i64, 9);

    // Environment dimension (display only): commits in the window, plus how many of them
    // touch files this change cares about (touched record ∪ task references).
    let commits_since = commit_files.len() as i64;
    let mut relevant: std::collections::BTreeSet<String> =
        crate::tasks::TouchedRecord::load(ws, &change.name)
            .all_files()
            .into_iter()
            .collect();
    for t in &task_list {
        relevant.extend(task_file_refs(&t.description));
    }
    let env_status = if relevant.is_empty() || commits_since == 0 {
        format!("{commits_since} commits")
    } else {
        let touching = commit_files
            .iter()
            .filter(|files| files.iter().any(|f| relevant.contains(f)))
            .count();
        format!("{commits_since} commits ({touching} touching this change's files)")
    };

    let dimensions = vec![
        DriftDimension {
            kind: "Time".to_string(),
            status: time_status,
            score: time_score,
            contributes_to_total: true,
        },
        DriftDimension {
            kind: "Structure".to_string(),
            status: structure_status,
            score: structure_score,
            contributes_to_total: true,
        },
        DriftDimension {
            kind: "Tasks".to_string(),
            status: tasks_status,
            score: tasks_score,
            contributes_to_total: true,
        },
        DriftDimension {
            kind: "Specs".to_string(),
            status: specs_status,
            score: specs_score,
            contributes_to_total: true,
        },
        DriftDimension {
            kind: "Environment".to_string(),
            status: env_status,
            score: 0,
            contributes_to_total: false,
        },
    ];

    let total_score: i64 = dimensions
        .iter()
        .filter(|d| d.contributes_to_total)
        .map(|d| d.score)
        .sum();

    let severity = if total_score > 8 || broken_ratio > 0.30 {
        "heavy"
    } else if total_score >= 4 {
        "medium"
    } else {
        "light"
    }
    .to_string();

    // Stale delta assumptions always route to ingest first: the archive merge gate
    // refuses them (--skip-specs skips spec application entirely).
    let primary_recommendation = if !spec_assumptions.is_empty() {
        format!("/speclink-ingest {}", change.name)
    } else {
        match severity.as_str() {
            "heavy" => format!("speclink archive {} --skip-specs", change.name),
            "medium" => format!("/speclink-ingest {}", change.name),
            _ => format!("/speclink-apply {}", change.name),
        }
    };

    DriftReport {
        change_id: change.name.clone(),
        created: change.meta.created.clone(),
        last_commit,
        dimensions,
        broken_anchors: broken,
        tasks_maybe_resolved,
        tasks_blocked_external,
        spec_assumptions,
        commits_since_created: commits_since,
        total_score,
        severity,
        primary_recommendation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-13\n";

    // --- Task 1: compute_spec_drift 純函式（只吃 Store 規格事實、確定性、無 git）---

    #[test]
    fn spec_drift_reports_specs_dimension_and_assumptions_deterministically() {
        // 規格面運算只吃 Store：delta 有一筆 ADDED 但正典已存在該需求 → 一條 stale
        // assumption（archive 會靜默 skip，drift 必須浮出）。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## ADDED Requirements\n\n### Requirement: Login\n",
        );
        store
            .write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n")
            .unwrap();
        let change = store.find_change("demo").unwrap();

        let a = compute_spec_drift(&store, &change);
        let b = compute_spec_drift(&store, &change);

        assert_eq!(a.dimension.kind, "Specs");
        assert_eq!(a.dimension.status, "1 stale assumptions");
        assert_eq!(a.dimension.score, 4);
        assert!(a.dimension.contributes_to_total);
        assert_eq!(a.spec_assumptions.len(), 1);
        assert_eq!(a.spec_assumptions[0].operation, "ADDED");
        assert_eq!(a.spec_assumptions[0].requirement, "Login");

        // 相同輸入重複呼叫逐欄相同（純函式、無隱藏狀態、無時間依賴）。
        assert_eq!(a.dimension.status, b.dimension.status);
        assert_eq!(a.dimension.score, b.dimension.score);
        assert_eq!(a.spec_assumptions.len(), b.spec_assumptions.len());
    }

    #[test]
    fn spec_drift_no_delta_specs_is_a_distinct_status() {
        let store = TestStore::with_meta("demo", META);
        let change = store.find_change("demo").unwrap();
        let r = compute_spec_drift(&store, &change);
        assert_eq!(r.dimension.status, "no delta specs");
        assert_eq!(r.dimension.score, 0);
        assert!(r.spec_assumptions.is_empty());
    }

    #[test]
    fn spec_drift_holds_when_delta_targets_are_consistent() {
        // delta 有 ADDED 且正典尚無該需求 → 假設成立、零分。
        // 新開 capability 的 delta 自帶合格 Purpose：Purpose 守門與 archive 共用
        // 同一支判定（spec archive-merge「過期判定單源共用」），缺席會在這裡記一筆。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## Purpose\n\n本 capability 負責登入與登出的可觀察行為，涵蓋工作階段的建立、續期與撤銷三段流程及其失敗處置。\n\n## ADDED Requirements\n\n### Requirement: Logout\n",
        );
        let change = store.find_change("demo").unwrap();
        let r = compute_spec_drift(&store, &change);
        assert_eq!(r.dimension.status, "delta assumptions hold");
        assert_eq!(r.dimension.score, 0);
        assert!(r.spec_assumptions.is_empty());
    }

    // --- 過期判定單源共用（design「判定共用」；spec archive-merge「過期判定單源共用」）---

    #[test]
    fn drift_bulk_precheck_and_single_archive_share_one_verdict() {
        // spec Scenario「三處判定一致」：同一過期 MODIFIED 之下，drift 的 spec
        // assumption、bulk 預檢讀的違規清單與單筆 archive 的拒絕逐欄指向同一
        // capability 與需求名——三處共用 merge_violations 這一支判定。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## MODIFIED Requirements\n\n### Requirement: Rotate tokens\n\nIt SHALL rotate.\n",
        );
        store
            .write_canonical_spec("auth", "## Purpose\n\nAuth.\n\n### Requirement: Login\n")
            .unwrap();
        let change = store.find_change("demo").unwrap();

        // bulk 預檢的來源＝引擎守門的來源。
        let violations = crate::archive::merge_violations(&store, "demo");
        assert_eq!(violations.len(), 1, "the pre-check sees exactly one violation");
        assert_eq!(violations[0].capability, "auth");
        assert_eq!(violations[0].operation, "MODIFIED");
        assert_eq!(violations[0].requirement, "Rotate tokens");

        // drift 的 Specs 維度逐欄同源。
        let assumptions = spec_assumptions(&store, &change);
        assert_eq!(assumptions.len(), violations.len(), "drift reports the same count");
        assert_eq!(assumptions[0].capability, violations[0].capability);
        assert_eq!(assumptions[0].operation, violations[0].operation);
        assert_eq!(assumptions[0].requirement, violations[0].requirement);
        assert_eq!(assumptions[0].reason, violations[0].reason, "one reason string, one source");

        // 單筆 archive 拒絕，訊息指向同一 capability 與需求名。
        let ws = Workspace {
            root: std::env::temp_dir().join("speclink-drift-single-source-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        };
        let opts = crate::archive::ArchiveOptions { no_validate: true, ..Default::default() };
        let err = crate::archive::archive(&ws, &store, &change, &opts, None)
            .expect_err("the same stale delta refuses a single archive");
        let msg = err.to_string();
        assert!(msg.contains("auth"), "capability named: {msg}");
        assert!(msg.contains("Rotate tokens"), "requirement named: {msg}");
        assert!(msg.contains(&violations[0].reason), "the shared reason is rendered: {msg}");
    }

    #[test]
    fn drift_reason_speaks_refusal_not_skip() {
        // design「判定共用」：reason 文案改為拒絕語意，欄位結構不變。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## ADDED Requirements\n\n### Requirement: Login\n",
        );
        store
            .write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n")
            .unwrap();
        let change = store.find_change("demo").unwrap();
        let assumptions = spec_assumptions(&store, &change);
        assert_eq!(assumptions.len(), 1);
        assert!(
            assumptions[0].reason.contains("archive would refuse it"),
            "refusal wording, not skip: {}",
            assumptions[0].reason
        );
    }

    // --- Task 1: WorkspaceFacts 封閉結構逐欄可表達 有值／空值／不可用 ---

    #[test]
    fn workspace_facts_express_three_value_semantics_per_field() {
        use std::collections::BTreeMap;

        // 不可用：git 相關欄位皆 None（區別於「空值」）。
        let unavailable = WorkspaceFacts::default();
        assert!(unavailable.commit_window.is_none(), "commit window 不可用");
        assert!(unavailable.tracked_docs.is_none(), "tracked docs 不可用");
        assert!(unavailable.symbol_head_hits.is_none(), "symbol hits 不可用");

        // 空值：git 可用但無內容（Some(empty) 明確有別於 None）。
        let empty = WorkspaceFacts {
            commit_window: Some(Vec::new()),
            tracked_docs: Some(Vec::new()),
            symbol_head_hits: Some(Vec::new()),
            path_status: BTreeMap::new(),
            touched_files: Vec::new(),
        };
        assert_eq!(empty.commit_window.as_deref(), Some(&[][..]), "空 commit window");
        assert!(empty.tracked_docs.as_deref().unwrap().is_empty());

        // 有值：commit window 一筆、符號命中、路徑三態俱全。
        let mut path_status = BTreeMap::new();
        path_status.insert("a.rs".to_string(), PathKind::File);
        path_status.insert("gone.rs".to_string(), PathKind::Missing);
        path_status.insert("dir".to_string(), PathKind::Other);
        let present = WorkspaceFacts {
            commit_window: Some(vec![vec!["a.rs".to_string()]]),
            tracked_docs: Some(vec!["# doc\nLogin\n".to_string()]),
            symbol_head_hits: Some(vec!["Login".to_string()]),
            path_status,
            touched_files: vec!["a.rs".to_string()],
        };
        assert_eq!(present.commit_window.as_ref().unwrap().len(), 1);
        assert_eq!(present.path_status.get("a.rs"), Some(&PathKind::File));
        assert_eq!(present.path_status.get("gone.rs"), Some(&PathKind::Missing));
        assert_eq!(present.path_status.get("dir"), Some(&PathKind::Other));
    }

    // --- Task 2: compute_workspace_drift（四維度只讀 facts、缺席即 unavailable）---

    fn assert_available(dim: &WorkspaceDimension, expected: &DriftDimension) {
        match dim {
            WorkspaceDimension::Available(d) => {
                assert_eq!(d.kind, expected.kind, "kind");
                assert_eq!(d.status, expected.status, "status of {}", expected.kind);
                assert_eq!(d.score, expected.score, "score of {}", expected.kind);
                assert_eq!(
                    d.contributes_to_total, expected.contributes_to_total,
                    "contributes of {}", expected.kind
                );
            }
            WorkspaceDimension::Unavailable { kind } => {
                panic!("expected {} available, got unavailable {kind}", expected.kind)
            }
        }
    }

    #[test]
    fn workspace_facts_absent_marks_all_four_dimensions_unavailable_without_scores() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "design.md", "Uses `DriftReport`.");
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
        let change = store.find_change("demo").unwrap();

        let r = compute_workspace_drift(&store, &change, None);

        assert_eq!(r.dimensions.len(), 4);
        for (i, kind) in ["Time", "Structure", "Tasks", "Environment"].iter().enumerate() {
            match &r.dimensions[i] {
                WorkspaceDimension::Unavailable { kind: k } => assert_eq!(k, kind),
                other => panic!("dim {i} expected unavailable {kind}, got {other:?}"),
            }
        }
        // 缺席不得帶任何工作區訊號（不是 clean、不是零分）。
        assert!(r.broken_anchors.is_empty());
        assert!(r.tasks_maybe_resolved.is_empty());
        assert!(r.tasks_blocked_external.is_empty());
        assert_eq!(r.commits_since_created, 0);
    }

    #[test]
    fn git_unavailable_facts_match_current_analyze_fallback_byte_for_byte() {
        // 「有 checkout 但 git 不可用」：facts 存在但 git 相關欄位 None。以非 git 的
        // 臨時 workspace 跑現行 analyze（git_available=false → 同一 fallback 路徑），
        // 與 compute_workspace_drift(facts=git 不可用) 的四維度逐欄對照。同一時鐘 →
        // 日數依賴的字串兩邊一致，測試與執行日期無關。
        let tmp = std::env::temp_dir()
            .join(format!("speclink-drift-wsparity-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let ws = Workspace { root: tmp.clone(), spec_dir_name: "openspec".to_string() };

        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "design.md", "Uses `DriftReport` and helper_fn.");
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
        let change = store.find_change("demo").unwrap();

        let expected = analyze(&ws, &store, &change);
        // git 不可用、checkout 存在：所有 git 欄位 None、fs 探測皆缺（tmp 為空）。
        let facts = WorkspaceFacts::default();
        let got = compute_workspace_drift(&store, &change, Some(&facts));

        // expected.dimensions 序：Time, Structure, Tasks, Specs, Environment。
        // got.dimensions 序：Time, Structure, Tasks, Environment。
        assert_available(&got.dimensions[0], &expected.dimensions[0]); // Time
        assert_available(&got.dimensions[1], &expected.dimensions[1]); // Structure
        assert_available(&got.dimensions[2], &expected.dimensions[2]); // Tasks
        assert_available(&got.dimensions[3], &expected.dimensions[4]); // Environment

        let got_broken: Vec<_> = got.broken_anchors.iter().map(|b| &b.anchor).collect();
        let exp_broken: Vec<_> = expected.broken_anchors.iter().map(|b| &b.anchor).collect();
        assert_eq!(got_broken, exp_broken, "broken anchors identical under git-unavailable");
        assert_eq!(got.tasks_maybe_resolved, expected.tasks_maybe_resolved);
        assert_eq!(got.tasks_blocked_external, expected.tasks_blocked_external);
        assert_eq!(got.commits_since_created, expected.commits_since_created);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn valid_facts_produce_the_git_available_four_dimensions_per_field() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "design.md", "Refers to `DriftReport` and helper_fn here.");
        store.put_artifact(
            "demo",
            "tasks.md",
            "- [ ] 1.1 wire `src/app.rs`\n- [ ] 1.2 remove `src/old.rs`\n",
        );
        let change = store.find_change("demo").unwrap();

        let mut path_status = std::collections::BTreeMap::new();
        path_status.insert("src/app.rs".to_string(), PathKind::File);
        path_status.insert("src/old.rs".to_string(), PathKind::Missing);
        let facts = WorkspaceFacts {
            commit_window: Some(vec![
                vec!["src/app.rs".to_string()],
                vec!["README.md".to_string()],
            ]),
            tracked_docs: Some(Vec::new()),
            symbol_head_hits: Some(vec!["DriftReport".to_string()]),
            path_status,
            touched_files: Vec::new(),
        };
        let r = compute_workspace_drift(&store, &change, Some(&facts));

        // Structure: DriftReport 命中、helper_fn 未命中 → 1/2 broken → score 4。
        assert_available(
            &r.dimensions[1],
            &DriftDimension {
                kind: "Structure".to_string(),
                status: "1/2 anchors broken".to_string(),
                score: 4,
                contributes_to_total: true,
            },
        );
        assert_eq!(r.broken_anchors.len(), 1);
        assert_eq!(r.broken_anchors[0].anchor, "helper_fn");

        // Tasks: git 可用 → 依 commit window 判定 1 maybe-done、0 blocked。
        assert_available(
            &r.dimensions[2],
            &DriftDimension {
                kind: "Tasks".to_string(),
                status: "0 blocked, 1 maybe-done".to_string(),
                score: 0,
                contributes_to_total: true,
            },
        );
        assert_eq!(r.tasks_maybe_resolved.len(), 1);
        assert!(r.tasks_maybe_resolved[0].contains("src/app.rs"));
        assert!(r.tasks_blocked_external.is_empty());

        // Environment: 2 commits，其中 1 筆碰到本 change 檔案（src/app.rs）。
        assert_available(
            &r.dimensions[3],
            &DriftDimension {
                kind: "Environment".to_string(),
                status: "2 commits (1 touching this change's files)".to_string(),
                score: 0,
                contributes_to_total: false,
            },
        );
        assert_eq!(r.commits_since_created, 2);

        // Time: git 可用（無 "git unavailable" 後綴）；score 由 days_old 決定。
        let days = crate::preflight::days_old(change.meta.created.as_deref());
        let expected_time_score = if days > 14 { 3 } else if days > 5 { 1 } else { 0 };
        match &r.dimensions[0] {
            WorkspaceDimension::Available(d) => {
                assert_eq!(d.kind, "Time");
                assert!(!d.status.contains("git unavailable"), "git available: {}", d.status);
                assert_eq!(d.score, expected_time_score);
            }
            other => panic!("Time must be available, got {other:?}"),
        }
    }

    // --- Task 3: merge_drift_reports（單一 merger、coverage、stale）---

    fn digests(spec: &str, tasks: &str, policy: &str) -> crate::tasks::BasisDigests {
        crate::tasks::BasisDigests {
            spec: spec.to_string(),
            tasks: tasks.to_string(),
            policy: policy.to_string(),
        }
    }

    #[test]
    fn full_coverage_merge_equals_current_drift_report_field_for_field() {
        // 以非 git 臨時 workspace 對照：現行 analyze 的整份 DriftReport，應與
        // compute_spec_drift + compute_workspace_drift(git 不可用 facts) + merger
        // 逐欄（並逐位元 JSON）一致；coverage 與 stale 於 full/非 stale 時不出現。
        let tmp = std::env::temp_dir()
            .join(format!("speclink-drift-mergeparity-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let ws = Workspace { root: tmp.clone(), spec_dir_name: "openspec".to_string() };

        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "design.md", "Uses `DriftReport` and helper_fn.");
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## MODIFIED Requirements\n\n### Requirement: Gone\n",
        );
        store.write_canonical_spec("auth", "## Purpose\n\n### Requirement: Other\n").unwrap();
        let change = store.find_change("demo").unwrap();

        let expected = analyze(&ws, &store, &change);

        let facts = WorkspaceFacts::default(); // git 不可用、checkout 存在
        let spec = compute_spec_drift(&store, &change);
        let workspace = compute_workspace_drift(&store, &change, Some(&facts));
        let combined = merge_drift_reports(&change, spec, workspace, None);

        assert_eq!(combined.coverage, Coverage::Full, "facts present → full coverage");
        assert!(combined.stale.is_none(), "no basis → not stale");

        let got = serde_json::to_value(&combined).unwrap();
        let want = serde_json::to_value(&expected).unwrap();
        assert_eq!(got, want, "combined full report is byte-identical to current DriftReport");

        // 選填欄位不得洩漏到 full 路徑輸出。
        let obj = got.as_object().unwrap();
        assert!(!obj.contains_key("coverage"), "coverage omitted on full path");
        assert!(!obj.contains_key("stale"), "stale omitted when not stale");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn workspace_absent_merges_to_spec_only_keeping_unavailable_dimensions() {
        let store = TestStore::with_meta("demo", META);
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## ADDED Requirements\n\n### Requirement: Login\n",
        );
        store.write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n").unwrap();
        let change = store.find_change("demo").unwrap();

        let spec = compute_spec_drift(&store, &change);
        let workspace = compute_workspace_drift(&store, &change, None);
        let combined = merge_drift_reports(&change, spec, workspace, None);

        assert_eq!(combined.coverage, Coverage::SpecOnly);
        // 五維度序仍為 Time, Structure, Tasks, Specs, Environment；四工作區維度標 unavailable。
        let dims = &combined.report.dimensions;
        assert_eq!(dims.len(), 5);
        for (i, kind) in ["Time", "Structure", "Tasks"].iter().enumerate() {
            assert_eq!(dims[i].kind, *kind);
            assert_eq!(dims[i].status, "unavailable", "{kind} 標 unavailable");
            assert!(!dims[i].contributes_to_total, "{kind} 不計分");
        }
        assert_eq!(dims[3].kind, "Specs", "Specs 仍在第 4 位且有值");
        assert_eq!(dims[4].kind, "Environment");
        assert_eq!(dims[4].status, "unavailable");
        // total 只含 Specs（缺席四維度不計為零分或任何分數）。
        assert_eq!(combined.report.total_score, combined.report.dimensions[3].score);

        // spec-only 時 coverage 出現於 JSON。
        let obj = serde_json::to_value(&combined).unwrap();
        assert_eq!(obj["coverage"], serde_json::json!("spec-only"));
    }

    #[test]
    fn basis_mismatch_marks_stale_listing_only_the_mismatched_item() {
        let store = TestStore::with_meta("demo", META);
        let change = store.find_change("demo").unwrap();
        let spec = compute_spec_drift(&store, &change);
        let workspace = compute_workspace_drift(&store, &change, Some(&WorkspaceFacts::default()));

        // 僅 tasks digest 不符 → stale 只列 Tasks。
        let basis = DriftBasis {
            expected: digests("sha256:s", "sha256:t-old", "sha256:p"),
            current: digests("sha256:s", "sha256:t-new", "sha256:p"),
        };
        let combined = merge_drift_reports(&change, spec, workspace, Some(&basis));

        let stale = combined.stale.as_ref().expect("basis mismatch marks stale");
        assert_eq!(stale.mismatched, vec![DriftBasisItem::Tasks], "只列不符項");

        let obj = serde_json::to_value(&combined).unwrap();
        assert_eq!(obj["stale"]["mismatched"], serde_json::json!(["tasks"]));
    }

    #[test]
    fn matching_basis_is_not_stale() {
        let store = TestStore::with_meta("demo", META);
        let change = store.find_change("demo").unwrap();
        let spec = compute_spec_drift(&store, &change);
        let workspace = compute_workspace_drift(&store, &change, Some(&WorkspaceFacts::default()));
        let same = digests("sha256:s", "sha256:t", "sha256:p");
        let basis = DriftBasis { expected: same.clone(), current: same };
        let combined = merge_drift_reports(&change, spec, workspace, Some(&basis));
        assert!(combined.stale.is_none(), "相同 basis 不標 stale");
    }
}

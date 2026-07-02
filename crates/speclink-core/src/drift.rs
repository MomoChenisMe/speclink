//! Drift detection between a change and the current codebase.

use crate::model::Change;
use crate::paths::Paths;
use crate::preflight::days_old;
use crate::util;
use regex::Regex;
use serde::Serialize;

const ANCHOR_CAP: usize = 50;

/// Stopwords filtered out of symbol-anchor extraction (probed against Spectra word by word).
// Spectra filters Rust type/keyword names and the GWT keywords, but KEEPS ordinary English
// words ("The", "Also", "Should", …) and — surprisingly — Eq/Ord/PartialEq/PartialOrd.
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

/// A delta-spec operation whose target no longer matches the canonical specs — archiving the
/// change would silently no-op or collide.
#[derive(Debug, Serialize)]
pub struct SpecAssumption {
    pub capability: String,
    pub operation: String,
    pub requirement: String,
    pub reason: String,
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

fn extract_anchors(design: &str) -> Vec<String> {
    let re = Regex::new(r"\b[A-Z][a-zA-Z0-9]+\b").unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for m in re.find_iter(design) {
        let w = m.as_str();
        if STOPWORDS.contains(&w) {
            continue;
        }
        seen.insert(w.to_string());
    }
    // Backticked code spans additionally contribute their LEADING identifier when it is
    // camelCase (lowercase start, at least one internal uppercase): `pressKey(code)` yields
    // pressKey, but `dotted.pathToken` and `under_scoreCamel` yield nothing (matches Spectra).
    let span = Regex::new(r"`([A-Za-z_][A-Za-z0-9_]*)[^`]*`").unwrap();
    let camel = Regex::new(r"^[a-z][a-z0-9]*[A-Z][A-Za-z0-9]*$").unwrap();
    for m in span.captures_iter(design) {
        let ident = &m[1];
        if camel.is_match(ident) && !STOPWORDS.contains(&ident) {
            seen.insert(ident.to_string());
        }
    }
    seen.into_iter().take(ANCHOR_CAP).collect()
}

/// Work-tree contents of tracked `*.md` / `*.txt` documents (via `git ls-files`), excluding
/// the change's own directory so a committed design.md cannot self-satisfy its anchors.
fn tracked_doc_contents(paths: &Paths, exclude_prefix: &str) -> Vec<String> {
    let Some(list) = util::git(&paths.root, &["ls-files"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in list.lines() {
        let f = line.trim();
        if f.starts_with(exclude_prefix) {
            continue;
        }
        if f.ends_with(".md") || f.ends_with(".txt") {
            if let Ok(content) = std::fs::read_to_string(paths.root.join(f)) {
                out.push(content);
            }
        }
    }
    out
}

/// Whether an anchor matches (whole-word, case-sensitive) in the search corpus: the committed
/// content of tracked files (HEAD) plus the work-tree content of tracked markdown/text
/// documents. Deliberate difference from Spectra: the change's own directory is excluded, so
/// broken anchors keep working after the design is committed (Spectra's corpus includes the
/// design itself, making Structure permanently silent post-commit).
fn symbol_found(paths: &Paths, doc_contents: &[String], exclude_prefix: &str, symbol: &str) -> bool {
    // ASCII word boundary, matching `git grep --word-regexp` semantics.
    let re = Regex::new(&format!(r"(?-u:\b){}(?-u:\b)", regex::escape(symbol)));
    if let Ok(re) = re {
        if doc_contents.iter().any(|c| re.is_match(c)) {
            return true;
        }
    }
    let exclude = format!(":(exclude){exclude_prefix}");
    util::git(
        &paths.root,
        &["grep", "-q", "--word-regexp", "--fixed-strings", symbol, "HEAD", "--", &exclude],
    )
    .is_some()
}

/// Path-like backtick references in a task description (`bomberman/index.html` yes,
/// `reset(seed)` no). References are read from the checkbox line only.
fn task_file_refs(desc: &str) -> Vec<String> {
    let re = Regex::new(r"`([^`\s()]+)`").unwrap();
    re.captures_iter(desc)
        .map(|c| c[1].replace('\\', "/"))
        .filter(|s| s.contains('/') || s.contains('.'))
        .collect()
}

/// Parse the `--since` log output into per-commit file lists (paths are repo-relative,
/// forward-slashed, one commit per `COMMIT|` header).
fn parse_commit_files(log: &str) -> Vec<Vec<String>> {
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

pub fn analyze(paths: &Paths, change: &Change) -> DriftReport {
    let design = util::read_opt(&change.dir.join("design.md")).unwrap_or_default();
    let anchors = extract_anchors(&design);
    let total_anchors = anchors.len();

    let git_ok = util::git_available(&paths.root);
    // Deliberate difference from Spectra: the created DATE is anchored to midnight. Spectra
    // passes the bare date, and git's approxidate fills the missing time-of-day from the
    // current clock — making same-day changes always count 0 commits.
    let since_arg = match change.meta.created.as_deref() {
        Some(c) if !c.is_empty() => format!("--since={c} 00:00:00"),
        _ => "--since=".to_string(),
    };
    let since_log = if git_ok {
        util::git(
            &paths.root,
            &["log", &since_arg, "--pretty=format:COMMIT|%H|%at|%s", "--name-only"],
        )
    } else {
        None
    };
    // The log call fails in a repo with no commits — that is what Spectra's
    // "git unavailable" statuses key off, not `git_available`.
    let git_has_commits = since_log.is_some();
    let commit_files = parse_commit_files(since_log.as_deref().unwrap_or(""));
    // Spectra's CLI never populates last_commit (it is an app-side field).
    let last_commit: Option<String> = None;

    let exclude_prefix = format!(
        "{}/changes/{}/",
        paths.spec_dir_name, change.name
    );
    let doc_contents = tracked_doc_contents(paths, &exclude_prefix);
    let mut broken = Vec::new();
    for a in &anchors {
        if !symbol_found(paths, &doc_contents, &exclude_prefix, a) {
            broken.push(BrokenAnchor {
                anchor: a.clone(),
                category: "Symbol".to_string(),
                reason: "symbol not found in repo".to_string(),
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
    let design_present = change.dir.join("design.md").is_file();
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

    // Tasks dimension — real signals (deliberate difference from Spectra, whose arrays are
    // always empty): an unchecked task is "maybe-done" when a file it references was committed
    // in the window and exists; "blocked" when a referenced file was touched and is now gone.
    let mut tasks_maybe_resolved: Vec<String> = Vec::new();
    let mut tasks_blocked_external: Vec<String> = Vec::new();
    let tasks_present = change.dir.join("tasks.md").is_file();
    let task_list = crate::tasks::parse(
        &util::read_opt(&change.dir.join("tasks.md")).unwrap_or_default(),
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
            .any(|r| window_files.contains(r.as_str()) && !paths.root.join(r).exists());
        if vanished {
            tasks_blocked_external.push(t.description.clone());
        } else if refs
            .iter()
            .any(|r| window_files.contains(r.as_str()) && paths.root.join(r).is_file())
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
    // still exist in the canonical specs, and would an ADDED requirement collide? Archive
    // silently skips both cases — drift is where they must surface.
    let delta_caps = crate::model::delta_capabilities(&change.dir);
    let mut spec_assumptions: Vec<SpecAssumption> = Vec::new();
    for cap in &delta_caps {
        let delta_text =
            util::read_opt(&change.dir.join("specs").join(cap).join("spec.md")).unwrap_or_default();
        let reqs = crate::archive::parse_delta(&delta_text);
        let canonical = util::read_opt(&paths.specs_dir().join(cap).join("spec.md"));
        let canonical_names: Option<std::collections::BTreeSet<String>> = canonical
            .as_deref()
            .map(|t| crate::archive::parse_canonical(t).1.into_iter().map(|(n, _)| n).collect());
        for r in &reqs {
            match (r.operation.as_str(), &canonical_names) {
                ("ADDED", Some(names)) if names.contains(&r.name) => {
                    spec_assumptions.push(SpecAssumption {
                        capability: cap.clone(),
                        operation: r.operation.clone(),
                        requirement: r.name.clone(),
                        reason: "already exists in the canonical spec — archive would skip it"
                            .to_string(),
                    });
                }
                ("MODIFIED" | "REMOVED" | "RENAMED", Some(names)) if !names.contains(&r.name) => {
                    spec_assumptions.push(SpecAssumption {
                        capability: cap.clone(),
                        operation: r.operation.clone(),
                        requirement: r.name.clone(),
                        reason: "target requirement no longer exists in the canonical spec"
                            .to_string(),
                    });
                }
                ("MODIFIED" | "REMOVED" | "RENAMED", None) => {
                    spec_assumptions.push(SpecAssumption {
                        capability: cap.clone(),
                        operation: r.operation.clone(),
                        requirement: r.name.clone(),
                        reason: "canonical spec for this capability does not exist".to_string(),
                    });
                }
                _ => {}
            }
        }
    }
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
        crate::tasks::TouchedRecord::load(paths, &change.name)
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

    // Stale delta assumptions always route to ingest first: archiving (with or without
    // --skip-specs) would silently drop or misapply the delta.
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

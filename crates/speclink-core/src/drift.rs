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

#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub change_id: String,
    pub created: Option<String>,
    pub last_commit: Option<String>,
    pub dimensions: Vec<DriftDimension>,
    pub broken_anchors: Vec<BrokenAnchor>,
    pub tasks_maybe_resolved: Vec<String>,
    pub tasks_blocked_external: Vec<String>,
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

/// Work-tree contents of tracked `*.md` / `*.txt` documents (via `git ls-files`).
fn tracked_doc_contents(paths: &Paths) -> Vec<String> {
    let Some(list) = util::git(&paths.root, &["ls-files"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in list.lines() {
        let f = line.trim();
        if f.ends_with(".md") || f.ends_with(".txt") {
            if let Ok(content) = std::fs::read_to_string(paths.root.join(f)) {
                out.push(content);
            }
        }
    }
    out
}

/// Whether an anchor matches (whole-word, case-sensitive) in the search corpus — mirrors
/// Spectra: the committed content of ALL tracked files (HEAD) plus the work-tree content of
/// tracked markdown/text documents. Note this means a committed design.md makes its own
/// anchors trivially "found"; broken anchors only surface while the design is uncommitted.
fn symbol_found(paths: &Paths, doc_contents: &[String], symbol: &str) -> bool {
    // ASCII word boundary, matching `git grep --word-regexp` semantics.
    let re = Regex::new(&format!(r"(?-u:\b){}(?-u:\b)", regex::escape(symbol)));
    if let Ok(re) = re {
        if doc_contents.iter().any(|c| re.is_match(c)) {
            return true;
        }
    }
    util::git(
        &paths.root,
        &["grep", "-q", "--word-regexp", "--fixed-strings", symbol, "HEAD"],
    )
    .is_some()
}

pub fn analyze(paths: &Paths, change: &Change) -> DriftReport {
    let design = util::read_opt(&change.dir.join("design.md")).unwrap_or_default();
    let anchors = extract_anchors(&design);
    let total_anchors = anchors.len();

    let git_ok = util::git_available(&paths.root);
    // Mirrors Spectra's single drift log call verbatim: `git log --since=<created>` with the
    // created DATE string — including git's approxidate quirk where a bare date fills the
    // missing time-of-day from the current clock (so same-day changes count ~0 commits).
    let since_arg = format!("--since={}", change.meta.created.as_deref().unwrap_or(""));
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
    // Spectra's CLI never populates last_commit (it is an app-side field).
    let last_commit: Option<String> = None;

    let doc_contents = tracked_doc_contents(paths);
    let mut broken = Vec::new();
    for a in &anchors {
        if !symbol_found(paths, &doc_contents, a) {
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

    // Tasks dimension
    let tasks_maybe_resolved: Vec<String> = Vec::new();
    let tasks_blocked_external: Vec<String> = Vec::new();
    let tasks_present = change.dir.join("tasks.md").is_file();
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

    // Environment dimension (display only): number of commits in the --since window.
    let commits_since = since_log
        .as_deref()
        .map(|o| o.lines().filter(|l| l.starts_with("COMMIT|")).count() as i64)
        .unwrap_or(0);
    let env_status = format!("{commits_since} commits");

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

    let primary_recommendation = match severity.as_str() {
        "heavy" => format!("speclink archive {} --skip-specs", change.name),
        "medium" => format!("/speclink-ingest {}", change.name),
        _ => format!("/speclink-apply {}", change.name),
    };

    DriftReport {
        change_id: change.name.clone(),
        created: change.meta.created.clone(),
        last_commit,
        dimensions,
        broken_anchors: broken,
        tasks_maybe_resolved,
        tasks_blocked_external,
        commits_since_created: commits_since,
        total_score,
        severity,
        primary_recommendation,
    }
}

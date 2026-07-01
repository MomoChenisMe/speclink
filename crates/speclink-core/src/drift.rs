//! Drift detection between a change and the current codebase.

use crate::model::Change;
use crate::paths::Paths;
use crate::preflight::days_old;
use crate::util;
use regex::Regex;
use serde::Serialize;

const ANCHOR_CAP: usize = 50;

/// Stopwords filtered out of symbol-anchor extraction (common type/keyword names).
// Spectra filters only a narrow set of std Rust type/trait names (plus a few acronyms) — it does
// NOT filter ordinary English words (e.g. "The") or serde derives (e.g. "Serialize").
const STOPWORDS: &[&str] = &[
    "Context", "State", "Result", "Error", "Option", "Vec", "Rust", "JSON", "CLI", "API",
    "Box", "String", "Self", "Ok", "Err", "Some", "None",
    // std traits that double as common words.
    "Display", "Default", "Debug", "Clone", "Copy", "PartialEq", "Eq", "PartialOrd", "Ord",
    "From", "Into", "Iterator", "Send", "Sync", "Sized",
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
    seen.into_iter().take(ANCHOR_CAP).collect()
}

/// Whether a symbol appears anywhere in the project source (excluding spec/work dirs).
fn symbol_in_repo(paths: &Paths, symbol: &str) -> bool {
    let code_exts = [
        "rs", "ts", "tsx", "jsx", "svelte", "js", "py", "go", "java", "c", "cpp", "h", "hpp",
        "html", "css", "vue", "rb",
    ];
    for file in util::walk_files(&paths.root) {
        let s = util::to_slash(&file);
        if s.contains("/.git/") || s.contains("/.speclink/") || s.contains("/openspec/") {
            continue;
        }
        let ext_ok = file
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| code_exts.contains(&e))
            .unwrap_or(false);
        if !ext_ok {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&file) {
            if content.contains(symbol) {
                return true;
            }
        }
    }
    false
}

pub fn analyze(paths: &Paths, change: &Change) -> DriftReport {
    let design = util::read_opt(&change.dir.join("design.md")).unwrap_or_default();
    let anchors = extract_anchors(&design);
    let total_anchors = anchors.len();

    let git_ok = util::git_available(&paths.root);
    // Last commit touching the change directory.
    let last_commit = if git_ok {
        util::git(
            &paths.root,
            &["log", "-1", "--format=%H"],
        )
        .filter(|s| !s.is_empty())
    } else {
        None
    };
    let git_has_commits = last_commit.is_some();

    let mut broken = Vec::new();
    for a in &anchors {
        if !symbol_in_repo(paths, a) {
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
    let time_status = if git_has_commits {
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
    let design_present = util::has_content(&change.dir.join("design.md"));
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
    let tasks_present = util::has_content(&change.dir.join("tasks.md"));
    let tasks_status = if !tasks_present {
        "no tasks.md".to_string()
    } else if git_has_commits {
        "no task collisions".to_string()
    } else {
        "git unavailable".to_string()
    };
    let tasks_score = 0;

    // Environment dimension (display only)
    let commits_since = 0;
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
        tasks_maybe_resolved: Vec::new(),
        tasks_blocked_external: Vec::new(),
        commits_since_created: commits_since,
        total_score,
        severity,
        primary_recommendation,
    }
}

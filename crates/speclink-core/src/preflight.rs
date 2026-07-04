//! Preflight checks run before apply.

use crate::model::Change;
use crate::store::Store;
use crate::workspace::Workspace;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Staleness {
    #[serde(rename = "daysOld")]
    pub days_old: i64,
    #[serde(rename = "isStale")]
    pub is_stale: bool,
}

#[derive(Debug, Serialize)]
pub struct MissingFile {
    pub path: String,
    #[serde(rename = "referencedIn")]
    pub referenced_in: String,
}

#[derive(Debug, Serialize)]
pub struct Preflight {
    pub status: String,
    #[serde(rename = "missingFiles")]
    pub missing_files: Vec<MissingFile>,
    /// Populated only from desktop-app data; always empty in a CLI-only project (probed).
    #[serde(rename = "driftedFiles")]
    pub drifted_files: Vec<String>,
    /// Omitted entirely when `created` is missing or unparseable, matching Spectra.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staleness: Option<Staleness>,
}

/// Days a change has existed, based on `.openspec.yaml` created date.
pub fn days_old(created: Option<&str>) -> i64 {
    let Some(created) = created else { return 0 };
    let Ok(created_date) = chrono::NaiveDate::parse_from_str(created.trim(), "%Y-%m-%d") else {
        return 0;
    };
    let today = chrono::Local::now().date_naive();
    (today - created_date).num_days().max(0)
}

/// Extensions Spectra accepts as "code" references in the proposal's Affected code
/// line (probed against 2.3.1; case-sensitive — `a.JS`/`a.Rs` do not count).
const CODE_EXTS: [&str; 12] = [
    "md", "html", "js", "ts", "tsx", "jsx", "css", "json", "yaml", "rs", "toml", "svelte",
];

/// Backtick file references from the proposal's "Affected code" block: scanning starts
/// at the first line containing "affected code" (case-insensitive) and stops at the
/// next `## ` heading. A path counts when it contains '/', does not end with '/', and
/// its extension is in `CODE_EXTS`. Order-preserving dedup (all probed).
fn proposal_code_refs(proposal: &str) -> Vec<String> {
    let lines: Vec<&str> = proposal.lines().collect();
    let Some(start) = lines
        .iter()
        .position(|l| l.to_lowercase().contains("affected code"))
    else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for (i, line) in lines[start..].iter().enumerate() {
        if i > 0 && line.starts_with("## ") {
            break;
        }
        let mut rest = *line;
        while let Some(open) = rest.find('`') {
            let Some(close_rel) = rest[open + 1..].find('`') else { break };
            let span = &rest[open + 1..open + 1 + close_rel];
            rest = &rest[open + 1 + close_rel + 1..];
            if span.contains('/')
                && !span.ends_with('/')
                && span
                    .rsplit_once('.')
                    .map(|(_, ext)| CODE_EXTS.contains(&ext))
                    .unwrap_or(false)
                && !out.iter().any(|p| p == span)
            {
                out.push(span.to_string());
            }
        }
    }
    out
}

impl Preflight {
    pub fn compute(ws: &Workspace, store: &dyn Store, change: &Change) -> Preflight {
        let staleness = change
            .meta
            .created
            .as_deref()
            .and_then(|c| chrono::NaiveDate::parse_from_str(c.trim(), "%Y-%m-%d").ok())
            .map(|created| {
                let days = (chrono::Local::now().date_naive() - created).num_days().max(0);
                Staleness {
                    days_old: days,
                    is_stale: days > 14,
                }
            });

        let proposal = store.read_artifact(&change.name, "proposal.md").unwrap_or_default();
        let missing_files: Vec<MissingFile> = proposal_code_refs(&proposal)
            .into_iter()
            .filter(|p| !ws.root.join(p).exists())
            .map(|p| MissingFile {
                path: p,
                referenced_in: "proposal".to_string(),
            })
            .collect();

        let drifted_files: Vec<String> = Vec::new();
        let is_stale = staleness.as_ref().map(|s| s.is_stale).unwrap_or(false);
        let status = if !missing_files.is_empty() {
            "critical"
        } else if !drifted_files.is_empty() || is_stale {
            "warnings"
        } else {
            "clean"
        }
        .to_string();
        Preflight {
            status,
            missing_files,
            drifted_files,
            staleness,
        }
    }
}

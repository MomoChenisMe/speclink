//! Preflight checks run before apply.

use crate::model::Change;
use crate::store::Store;
use crate::workspace::Workspace;
use serde::Serialize;

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Staleness {
    #[serde(rename = "daysOld")]
    pub days_old: i64,
    #[serde(rename = "isStale")]
    pub is_stale: bool,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct MissingFile {
    pub path: String,
    #[serde(rename = "referencedIn")]
    pub referenced_in: String,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Preflight {
    pub status: String,
    #[serde(rename = "missingFiles")]
    pub missing_files: Vec<MissingFile>,
    /// Populated only from desktop-app data; always empty in a CLI-only project (probed).
    #[serde(rename = "driftedFiles")]
    pub drifted_files: Vec<String>,
    /// Omitted entirely when `created` is missing or unparseable (frozen output shape).
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

/// Extensions accepted as "code" references in the proposal's Affected code
/// line (frozen list; case-sensitive — `a.JS`/`a.Rs` do not count).
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
        // A run with no workspace has no local file to look for: an empty root
        // would look in the process's working directory instead.
        let missing_files: Vec<MissingFile> = proposal_code_refs(&proposal)
            .into_iter()
            .filter(|p| ws.is_local() && !ws.root.join(p).exists())
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

#[cfg(test)]
mod tests {
    use super::Preflight;
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    const PROPOSAL: &str =
        "## Impact\n\n- Affected code:\n  - `src/present/kept.rs`\n  - `src/nope/missing.rs`\n";

    fn preflight_on(ws: &Workspace) -> Vec<String> {
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        store.put_artifact("demo", "proposal.md", PROPOSAL);
        let change = crate::model::find_change(&store, "demo").unwrap();
        Preflight::compute(ws, &store, &change)
            .missing_files
            .into_iter()
            .map(|m| m.path)
            .collect()
    }

    #[test]
    fn a_local_workspace_reports_the_referenced_files_it_lacks() {
        let root = std::env::temp_dir().join(format!("speclink-preflight-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src/present")).unwrap();
        std::fs::write(root.join("src/present/kept.rs"), "").unwrap();
        let ws = Workspace { root: root.clone(), spec_dir_name: "speclink".to_string() };
        assert_eq!(preflight_on(&ws), ["src/nope/missing.rs"]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn no_workspace_reports_no_missing_file() {
        // 沒有 workspace（root 是空的）時沒有本機檔案可查：不拿程序的工作目錄去判斷。
        let ws = Workspace { root: std::path::PathBuf::new(), spec_dir_name: "speclink".to_string() };
        assert!(preflight_on(&ws).is_empty());
    }
}

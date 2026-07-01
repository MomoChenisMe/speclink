//! Preflight checks run before apply.

use crate::model::Change;
use crate::paths::Paths;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Staleness {
    #[serde(rename = "daysOld")]
    pub days_old: i64,
    #[serde(rename = "isStale")]
    pub is_stale: bool,
}

#[derive(Debug, Serialize)]
pub struct Preflight {
    pub status: String,
    #[serde(rename = "missingFiles")]
    pub missing_files: Vec<String>,
    #[serde(rename = "driftedFiles")]
    pub drifted_files: Vec<String>,
    pub staleness: Staleness,
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

impl Preflight {
    pub fn compute(_paths: &Paths, change: &Change) -> Preflight {
        let days = days_old(change.meta.created.as_deref());
        let is_stale = days > 14;
        let missing_files: Vec<String> = Vec::new();
        let drifted_files: Vec<String> = Vec::new();
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
            staleness: Staleness {
                days_old: days,
                is_stale,
            },
        }
    }
}

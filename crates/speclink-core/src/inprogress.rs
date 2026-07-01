//! In-progress change markers (file-based; replaces Spectra's SQLite table).

use crate::paths::Paths;
use crate::util;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct InProgress {
    #[serde(default)]
    changes: Vec<String>,
}

fn load(paths: &Paths) -> InProgress {
    match util::read_opt(&paths.in_progress_file()) {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => InProgress::default(),
    }
}

fn save(paths: &Paths, ip: &InProgress) -> Result<()> {
    let json = serde_json::to_string_pretty(ip)?;
    util::write_file(&paths.in_progress_file(), &json)?;
    Ok(())
}

/// Mark a change as in-progress (idempotent).
pub fn add(paths: &Paths, name: &str) -> Result<()> {
    let mut ip = load(paths);
    if !ip.changes.iter().any(|c| c == name) {
        ip.changes.push(name.to_string());
        save(paths, &ip)?;
    }
    Ok(())
}

/// Whether a change is marked in-progress.
pub fn is_in_progress(paths: &Paths, name: &str) -> bool {
    load(paths).changes.iter().any(|c| c == name)
}

/// Remove a change from the in-progress set (used on archive).
pub fn remove(paths: &Paths, name: &str) -> Result<()> {
    let mut ip = load(paths);
    let before = ip.changes.len();
    ip.changes.retain(|c| c != name);
    if ip.changes.len() != before {
        save(paths, &ip)?;
    }
    Ok(())
}

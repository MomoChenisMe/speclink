//! Change discovery, metadata, and artifact status.

use crate::paths::Paths;
use crate::schema::{Artifact, Schema};
use crate::util;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// `.openspec.yaml` — per-change metadata.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChangeMeta {
    pub schema: Option<String>,
    pub created: Option<String>,
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_with: Option<String>,
    /// Slug of the discussion this change was promoted from (speclink extension).
    #[serde(default)]
    pub from_discussion: Option<String>,
}

impl ChangeMeta {
    pub fn load(change_dir: &Path) -> ChangeMeta {
        let p = change_dir.join(".openspec.yaml");
        match std::fs::read_to_string(&p) {
            Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
            Err(_) => ChangeMeta::default(),
        }
    }
    pub fn schema_name(&self) -> String {
        self.schema
            .clone()
            .unwrap_or_else(|| "spec-driven".to_string())
    }
}

/// A discovered change.
#[derive(Debug, Clone)]
pub struct Change {
    pub name: String,
    pub dir: PathBuf,
    pub meta: ChangeMeta,
}

/// List active changes (directories under changes/, excluding `archive`).
pub fn list_changes(paths: &Paths) -> Vec<Change> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(paths.changes_dir()) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "archive" {
            continue;
        }
        out.push(Change {
            name: name.clone(),
            meta: ChangeMeta::load(&path),
            dir: path,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Find an active change by name.
pub fn find_change(paths: &Paths, name: &str) -> Option<Change> {
    let dir = paths.change_dir(name);
    if dir.is_dir() {
        Some(Change {
            name: name.to_string(),
            meta: ChangeMeta::load(&dir),
            dir,
        })
    } else {
        None
    }
}

/// Whether an artifact's output exists and has content.
pub fn artifact_done(change_dir: &Path, artifact: &Artifact) -> bool {
    // Done-ness is EXISTS-based — an empty file counts (matches Spectra). A glob-style output
    // (e.g. "specs/**/*.md") is done when any matching file exists.
    if artifact.output_path.contains("**") {
        return !spec_files(change_dir).is_empty();
    }
    change_dir.join(&artifact.output_path).is_file()
}

/// Delta spec files of a change: exactly `specs/<capability>/spec.md`, one level deep
/// (matches Spectra — nested or differently-named .md files under specs/ do not count).
pub fn spec_files(change_dir: &Path) -> Vec<PathBuf> {
    let specs = change_dir.join("specs");
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&specs) {
        for e in entries.flatten() {
            let spec = e.path().join("spec.md");
            if e.path().is_dir() && spec.is_file() {
                out.push(spec);
            }
        }
    }
    out.sort();
    out
}

/// Newest file mtime inside a change directory (recursive), truncated to whole seconds —
/// Spectra's sort key for "most recently modified" ordering everywhere a change list is
/// ordered (list, validate --all, multi-change candidate lists).
pub fn newest_mtime_secs(dir: &Path) -> u64 {
    util::walk_files(dir)
        .into_iter()
        .filter_map(|p| std::fs::metadata(&p).and_then(|m| m.modified()).ok())
        .filter_map(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .max()
        .unwrap_or(0)
}

/// Number of `### Requirement:` declarations under an ADDED/MODIFIED/REMOVED section. A RENAMED
/// section (FROM:/TO:) and empty operation headers contribute zero — matching Spectra's rule that
/// a delta spec must contain at least one applied operation.
pub fn op_requirement_count(text: &str) -> usize {
    let mut op = "";
    let mut count = 0;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                op = rest.split_whitespace().next().unwrap_or("");
            }
        } else if t.starts_with("### Requirement:")
            && matches!(op, "ADDED" | "MODIFIED" | "REMOVED")
        {
            count += 1;
        }
    }
    count
}

/// A line-start `### Requirement:` that is not under an ADDED/MODIFIED/REMOVED section (a malformed
/// delta — requirement declared with no operation heading).
pub fn has_orphan_requirement(text: &str) -> bool {
    let mut op = "";
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                op = rest.split_whitespace().next().unwrap_or("");
            }
        } else if t.starts_with("### Requirement:") && !matches!(op, "ADDED" | "MODIFIED" | "REMOVED") {
            return true;
        }
    }
    false
}

/// Whether a delta spec body has an applicable operation (ADDED/MODIFIED/REMOVED with a requirement).
pub fn has_delta_operation(text: &str) -> bool {
    // Speclink divergence #4: a RENAMED section with at least one valid FROM/TO pair
    // counts as an operation, so a pure-rename delta validates and archives. (Spectra
    // documents RENAMED but treats rename-only deltas as invalid and never applies
    // renames at all.)
    op_requirement_count(text) > 0 || !rename_pairs(text).is_empty()
}

/// Rename pairs from `## RENAMED Requirements` sections (speclink divergence #4 —
/// Spectra parses but never applies renames). Both documented syntaxes:
/// - bullet form: `- FROM: `### Requirement: Old`` / `- TO: `### Requirement: New``
///   (bold markers and bare names accepted)
/// - header form: `### Requirement: Old` followed by a `TO: New` line
pub fn rename_pairs(text: &str) -> Vec<(String, String)> {
    fn req_name(raw: &str) -> String {
        let s = raw.trim().trim_matches('`').trim();
        s.strip_prefix("### Requirement:").map(str::trim).unwrap_or(s).to_string()
    }
    let mut out = Vec::new();
    let mut in_renamed = false;
    let mut from: Option<String> = None;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            if rest.trim_end().ends_with("Requirements") {
                in_renamed = rest.split_whitespace().next() == Some("RENAMED");
                from = None;
                continue;
            }
        }
        if !in_renamed {
            continue;
        }
        if let Some(name) = t.strip_prefix("### Requirement:") {
            from = Some(name.trim().to_string());
            continue;
        }
        let norm = t.trim().trim_start_matches("- ").replace("**", "");
        if let Some(v) = norm.strip_prefix("FROM:") {
            let v = req_name(v);
            if !v.is_empty() {
                from = Some(v);
            }
        } else if let Some(v) = norm.strip_prefix("TO:") {
            let v = req_name(v);
            if let (Some(f), false) = (from.take(), v.is_empty()) {
                out.push((f, v));
            }
        }
    }
    out
}

/// Capability names present as delta specs — directory names under specs/ whose spec.md FILE
/// exists (empty or op-less files count, matching Spectra's show/archive listing).
pub fn delta_capabilities(change_dir: &Path) -> Vec<String> {
    let specs = change_dir.join("specs");
    let mut caps = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&specs) {
        for entry in entries.flatten() {
            if entry.path().is_dir() && entry.path().join("spec.md").is_file() {
                caps.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    caps.sort();
    caps
}

/// DAG status of a single artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStatus {
    Done,
    Ready,
    Blocked,
}

impl ArtifactStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArtifactStatus::Done => "done",
            ArtifactStatus::Ready => "ready",
            ArtifactStatus::Blocked => "blocked",
        }
    }
}

/// Compute the status of every artifact in the schema for a change.
pub fn artifact_statuses(schema: &Schema, change_dir: &Path) -> Vec<(String, ArtifactStatus)> {
    // First pass: done-ness.
    let mut done: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
    for a in &schema.artifacts {
        done.insert(a.id.as_str(), artifact_done(change_dir, a));
    }
    // Second pass: ready/blocked based on requires.
    let mut out = Vec::new();
    for a in &schema.artifacts {
        let status = if *done.get(a.id.as_str()).unwrap_or(&false) {
            ArtifactStatus::Done
        } else if a.requires.iter().all(|r| *done.get(r.as_str()).unwrap_or(&false)) {
            ArtifactStatus::Ready
        } else {
            ArtifactStatus::Blocked
        };
        out.push((a.id.clone(), status));
    }
    out
}

/// Which artifact ids block a given artifact (unmet requires).
pub fn blocked_by(schema: &Schema, change_dir: &Path, id: &str) -> Vec<String> {
    let Some(a) = schema.artifact(id) else {
        return Vec::new();
    };
    a.requires
        .iter()
        .filter(|r| {
            schema
                .artifact(r)
                .map(|ra| !artifact_done(change_dir, ra))
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .collect()
}

/// Whether EVERY artifact in the schema is done (matches Spectra — an absent optional artifact
/// such as `design` keeps the change incomplete).
pub fn is_complete(schema: &Schema, change_dir: &Path) -> bool {
    schema
        .artifacts
        .iter()
        .all(|a| artifact_done(change_dir, a))
}

/// Artifacts in the schema that are not yet done (used for analyze "Missing" reporting).
pub fn missing_artifacts(schema: &Schema, change_dir: &Path) -> Vec<String> {
    schema
        .artifacts
        .iter()
        .filter(|a| !artifact_done(change_dir, a))
        .map(|a| a.id.to_string())
        .collect()
}

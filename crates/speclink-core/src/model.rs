//! Change discovery, metadata, and artifact status.

use crate::schema::{Artifact, Schema};
use crate::store::Store;
use serde::Deserialize;
use std::path::PathBuf;

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
    /// Parse a raw metadata document. A missing document or a parse error
    /// yields the defaults (a corrupt `.openspec.yaml` never breaks listing).
    pub fn from_text(text: Option<&str>) -> ChangeMeta {
        match text {
            Some(s) => serde_yaml::from_str(s).unwrap_or_default(),
            None => ChangeMeta::default(),
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
    /// Display location of the change's documents (rendered in payloads and
    /// human output; content access goes through the [`Store`]).
    pub dir: PathBuf,
    pub meta: ChangeMeta,
}

/// List active changes, sorted by name.
pub fn list_changes(store: &dyn Store) -> Vec<Change> {
    store.list_changes()
}

/// Find an active change by name.
pub fn find_change(store: &dyn Store, name: &str) -> Option<Change> {
    store.find_change(name)
}

/// Whether an artifact's output exists and has content.
pub fn artifact_done(store: &dyn Store, change: &str, artifact: &Artifact) -> bool {
    // Done-ness is EXISTS-based — an empty file counts (matches Spectra). A glob-style output
    // (e.g. "specs/**/*.md") is done when any matching delta spec exists.
    if artifact.output_path.contains("**") {
        return !store.delta_capabilities(change).is_empty();
    }
    store.artifact_exists(change, &artifact.output_path)
}

/// The artifact identifier of a capability's delta spec inside a change.
pub fn delta_spec_artifact(cap: &str) -> String {
    format!("specs/{cap}/spec.md")
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
pub fn artifact_statuses(schema: &Schema, store: &dyn Store, change: &str) -> Vec<(String, ArtifactStatus)> {
    // First pass: done-ness.
    let mut done: std::collections::HashMap<&str, bool> = std::collections::HashMap::new();
    for a in &schema.artifacts {
        done.insert(a.id.as_str(), artifact_done(store, change, a));
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
pub fn blocked_by(schema: &Schema, store: &dyn Store, change: &str, id: &str) -> Vec<String> {
    let Some(a) = schema.artifact(id) else {
        return Vec::new();
    };
    a.requires
        .iter()
        .filter(|r| {
            schema
                .artifact(r)
                .map(|ra| !artifact_done(store, change, ra))
                .unwrap_or(false)
        })
        .map(|s| s.to_string())
        .collect()
}

/// Whether EVERY artifact in the schema is done (matches Spectra — an absent optional artifact
/// such as `design` keeps the change incomplete).
pub fn is_complete(schema: &Schema, store: &dyn Store, change: &str) -> bool {
    schema
        .artifacts
        .iter()
        .all(|a| artifact_done(store, change, a))
}

/// Artifacts in the schema that are not yet done (used for analyze "Missing" reporting).
pub fn missing_artifacts(schema: &Schema, store: &dyn Store, change: &str) -> Vec<String> {
    schema
        .artifacts
        .iter()
        .filter(|a| !artifact_done(store, change, a))
        .map(|a| a.id.to_string())
        .collect()
}

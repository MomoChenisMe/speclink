//! `new change` and `new artifact`.

use crate::model::{self, Change};
use crate::paths::Paths;
use crate::schema::Schema;
use crate::util;
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Create a new change directory with `.openspec.yaml`.
pub fn new_change(
    paths: &Paths,
    name: &str,
    _description: Option<&str>,
    schema: &str,
) -> Result<PathBuf> {
    let dir = paths.change_dir(name);
    if dir.exists() {
        bail!("change '{name}' already exists");
    }
    let created = util::today();
    let mut meta = format!("schema: {schema}\ncreated: {created}\n");
    if let Some(id) = util::git_identity(&paths.root) {
        meta.push_str(&format!("created_by: {id}\n"));
    }
    util::write_file(&dir.join(".openspec.yaml"), &meta)?;
    Ok(dir)
}

/// Resolve the artifact type token to (artifact_id, relative_output_path).
fn resolve_output(kind: &str, capability: Option<&str>) -> Result<(String, String)> {
    match kind {
        "proposal" => Ok(("proposal".into(), "proposal.md".into())),
        "design" => Ok(("design".into(), "design.md".into())),
        "tasks" => Ok(("tasks".into(), "tasks.md".into())),
        "spec" => {
            let cap = capability.ok_or_else(|| {
                anyhow::anyhow!("capability name is required for spec artifacts")
            })?;
            Ok(("specs".into(), format!("specs/{cap}/spec.md")))
        }
        other => bail!("unknown artifact type: {other}"),
    }
}

/// Create (write) an artifact for a change.
pub fn new_artifact(
    change: &Change,
    schema: &Schema,
    kind: &str,
    capability: Option<&str>,
    content: Option<&str>,
    force: bool,
) -> Result<(String, PathBuf)> {
    let (artifact_id, rel) = resolve_output(kind, capability)?;
    // Join component-by-component so the native path separator is used throughout.
    let out_path = rel.split('/').fold(change.dir.clone(), |p, c| p.join(c));
    if out_path.exists() && !force {
        bail!("{rel} already exists (use --force to overwrite)");
    }

    let body = match content {
        Some(c) => c.to_string(),
        None => {
            // Empty template from schema.
            let art = schema
                .artifact(&artifact_id)
                .ok_or_else(|| anyhow::anyhow!("no template for {artifact_id}"))?;
            art.template.to_string()
        }
    };

    // Validate supplied content structurally (only when content is provided).
    if content.is_some() {
        validate_artifact_content(&artifact_id, &rel, &body)?;
    }

    util::write_file(&out_path, &body)?;
    Ok((artifact_id, out_path))
}

fn validate_artifact_content(artifact_id: &str, rel: &str, body: &str) -> Result<()> {
    match artifact_id {
        "proposal" => {
            let ok = ["## Why", "## Problem", "## Summary"]
                .iter()
                .any(|h| body.lines().any(|l| l.trim_end() == *h || l.trim_start().starts_with(&format!("{h} "))));
            if !ok {
                bail!("proposal must contain one of: ## Why, ## Problem, ## Summary");
            }
        }
        "design" => {
            if !body.contains("## Context") {
                bail!("design must contain ## Context");
            }
        }
        "tasks" => {
            let ok = body.lines().any(|l| {
                let t = l.trim_start();
                t.starts_with("- [ ] ") || t.starts_with("- [x] ") || t.starts_with("- [X] ")
            });
            if !ok {
                bail!("tasks must contain at least one - [ ] checkbox");
            }
        }
        "specs" => {
            let has_op = ["## ADDED Requirements", "## MODIFIED Requirements", "## REMOVED Requirements", "## RENAMED Requirements"]
                .iter()
                .any(|op| body.contains(op));
            if !has_op {
                bail!("{rel}: delta spec must contain at least one operation (ADDED/MODIFIED/REMOVED/RENAMED Requirements)");
            }
        }
        _ => {}
    }
    Ok(())
}

/// Convenience: find or error for an active change by name.
pub fn require_change(paths: &Paths, name: &str) -> Result<Change> {
    model::find_change(paths, name).ok_or_else(|| anyhow::anyhow!("change '{name}' not found"))
}

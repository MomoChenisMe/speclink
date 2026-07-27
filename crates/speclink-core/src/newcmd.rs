//! `new change` and `new artifact`.

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
use crate::util;
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Create a new change with its metadata document. `actor` is the
/// Host-resolved display identity — None (anonymous) stamps no created_by.
pub fn new_change(
    store: &dyn Store,
    name: &str,
    _description: Option<&str>,
    schema: &str,
    agent: Option<&str>,
    from_discussion: Option<&str>,
    actor: Option<&str>,
) -> Result<PathBuf> {
    if !is_kebab_case(name) {
        bail!("Invalid change name '{name}'. Must be kebab-case (e.g., 'add-feature').");
    }
    if store.change_exists(name) {
        bail!("Change '{name}' already exists.");
    }
    let created = util::today();
    let mut meta = format!("schema: {schema}\ncreated: {created}\n");
    if let Some(id) = actor {
        meta.push_str(&format!("created_by: {id}\n"));
    }
    if let Some(agent) = agent {
        meta.push_str(&format!("created_with: {agent}\n"));
    }
    if let Some(slug) = from_discussion {
        meta.push_str(&format!("from_discussion: {slug}\n"));
    }
    store.create_change(name, &meta)
}

/// Resolve the artifact type token to (artifact_id, relative_output_path).
fn resolve_output(kind: &str, capability: Option<&str>) -> Result<(String, String)> {
    match kind {
        "proposal" => Ok(("proposal".into(), "proposal.md".into())),
        "design" => Ok(("design".into(), "design.md".into())),
        "tasks" => Ok(("tasks".into(), "tasks.md".into())),
        "spec" => {
            let cap = capability.ok_or_else(|| {
                anyhow::anyhow!(
                    "Capability name is required for spec type. Usage: speclink new artifact spec <capability> --change <name>"
                )
            })?;
            Ok(("specs".into(), format!("specs/{cap}/spec.md")))
        }
        other => bail!("Unknown artifact type '{other}'. Valid types: proposal, design, tasks, spec"),
    }
}

/// Create (write) an artifact for a change.
pub fn new_artifact(
    store: &dyn Store,
    change: &Change,
    schema: &Schema,
    kind: &str,
    capability: Option<&str>,
    content: Option<&str>,
    force: bool,
) -> Result<(String, PathBuf)> {
    // Fail-closed gate: corrupt metadata must not be read as the default
    // schema and produce an artifact from its templates.
    crate::model::require_valid_meta(change)?;
    let (artifact_id, rel) = resolve_output(kind, capability)?;
    if store.artifact_exists(&change.name, &rel) && !force {
        // The display path is joined component-by-component so the native
        // separator is used throughout (matches the created file's path).
        let out_path = rel.split('/').fold(change.dir.clone(), |p, c| p.join(c));
        bail!("Artifact already exists: {}. Use --force to overwrite", out_path.to_string_lossy());
    }

    let body = match content {
        Some(c) => c.to_string(),
        // Template from the schema; a missing template file (or an artifact the schema doesn't
        // define) yields an empty file (frozen behavior).
        None => schema
            .artifact(&artifact_id)
            .and_then(|a| a.template.clone())
            .unwrap_or_default(),
    };

    // Validate supplied content structurally (only when content is provided).
    if content.is_some() {
        validate_artifact_content(&artifact_id, &rel, &body)?;
    }

    // Engine-produced tasks carry stable IDs on every task line (spec task-identity).
    let body = if artifact_id == "tasks" { crate::tasks::stamp_all(&body) } else { body };

    let out_path = store.write_artifact(&change.name, &rel, &body)?;
    Ok((artifact_id, out_path))
}

fn validate_artifact_content(artifact_id: &str, rel: &str, body: &str) -> Result<()> {
    match artifact_id {
        "proposal" => {
            let ok = ["## Why", "## Problem", "## Summary"]
                .iter()
                .any(|h| body.lines().any(|l| l.trim_end() == *h || l.trim_start().starts_with(&format!("{h} "))));
            if !ok {
                bail!("Proposal must contain a ## Why, ## Problem, or ## Summary section");
            }
        }
        "design" => {
            if !body.contains("## Context") {
                bail!("Design must contain a ## Context section");
            }
        }
        "tasks" => {
            // At least one INCOMPLETE checkbox is required.
            let ok = body
                .lines()
                .any(|l| l.trim_start().starts_with("- [ ] "));
            if !ok {
                bail!("Tasks must contain at least one checkbox (- [ ])");
            }
        }
        "specs" => {
            let _ = rel;
            if !model::has_delta_operation(body) {
                bail!("Delta spec parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)");
            }
        }
        _ => {}
    }
    Ok(())
}

/// Whether a change name is valid kebab-case (lowercase alphanumerics with single hyphens).
fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Convenience: find or error for an active change by name.
pub fn require_change(store: &dyn Store, name: &str) -> Result<Change> {
    model::find_change(store, name).ok_or_else(|| anyhow::anyhow!("Change '{name}' not found."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teststore::TestStore;

    // --- ExecutionContext 由 Host 解析且不可覆寫：new change 收明確 actor ---

    #[test]
    fn new_change_stamps_the_injected_actor_only() {
        let store = TestStore::default();
        new_change(&store, "with-actor", None, "spec-driven", None, None, Some("Alice <a@example.com>"))
            .expect("new change succeeds");
        assert!(
            store.meta("with-actor").contains("created_by: Alice <a@example.com>\n"),
            "created_by is the injected actor, meta: {}",
            store.meta("with-actor")
        );
    }

    #[test]
    fn new_change_without_actor_stamps_no_created_by() {
        // 無身分：沿用現行無章行為（同今日無 git／未設 user.name）。
        let store = TestStore::default();
        new_change(&store, "anon", None, "spec-driven", None, None, None)
            .expect("new change succeeds");
        assert!(
            !store.meta("anon").contains("created_by:"),
            "anonymous stays unstamped, meta: {}",
            store.meta("anon")
        );
    }
}

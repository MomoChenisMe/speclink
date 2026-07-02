//! Build `instructions` payloads (per-artifact and apply mode) with config injection.

use crate::config::{AppConfig, WorkflowConfig};
use crate::model::{self, Change};
use crate::paths::Paths;
use crate::preflight::Preflight;
use crate::schema::Schema;
use crate::tasks::{self, Task};
use serde::Serialize;
use std::path::Path;

fn join_display(base: &Path, rel: &str) -> String {
    base.join(rel).to_string_lossy().to_string()
}

#[derive(Debug, Serialize)]
pub struct Dependency {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

/// Per-artifact instructions payload (field order matches Spectra).
#[derive(Debug, Serialize)]
pub struct ArtifactInstructions {
    #[serde(rename = "changeName")]
    pub change_name: String,
    #[serde(rename = "artifactId")]
    pub artifact_id: String,
    #[serde(rename = "schemaName")]
    pub schema_name: String,
    #[serde(rename = "changeDir")]
    pub change_dir: String,
    #[serde(rename = "outputPath")]
    pub output_path: String,
    pub description: String,
    /// Omitted entirely when the (custom) schema has no instruction, matching Spectra.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    pub locale: String,
    pub template: String,
    pub dependencies: Vec<Dependency>,
    pub unlocks: Vec<String>,
}

/// Build per-artifact instructions.
pub fn build_artifact(
    paths: &Paths,
    change: &Change,
    schema: &Schema,
    artifact_id: &str,
) -> Option<ArtifactInstructions> {
    let artifact = schema.artifact(artifact_id)?;
    let app = AppConfig::load(&paths.app_config());
    let wf = WorkflowConfig::load(&paths.workflow_config());

    let dependencies = artifact
        .requires
        .iter()
        .filter_map(|dep_id| {
            let da = schema.artifact(dep_id)?;
            Some(Dependency {
                id: da.id.to_string(),
                done: model::artifact_done(&change.dir, da),
                path: da.output_path.to_string(),
                description: da.description.to_string(),
            })
        })
        .collect();

    // `unlocks` = downstream artifacts for which THIS artifact is the last unmet dependency:
    // not yet done, list this artifact in their `requires`, and have every other requirement done.
    // Empty once this artifact itself is done (it has already unlocked its dependents).
    let self_done = model::artifact_done(&change.dir, artifact);
    let unlocks: Vec<String> = if self_done {
        Vec::new()
    } else {
        // Listed in display order (topological tier, alphabetical tiebreak), matching Spectra.
        crate::status::display_order(schema)
            .into_iter()
            .filter(|y| y.id != artifact.id)
            .filter(|y| y.requires.iter().any(|r| *r == artifact.id))
            .filter(|y| !model::artifact_done(&change.dir, y))
            .filter(|y| {
                y.requires.iter().all(|d| {
                    *d == artifact.id
                        || schema
                            .artifact(d)
                            .map(|da| model::artifact_done(&change.dir, da))
                            .unwrap_or(false)
                })
            })
            .map(|y| y.id.to_string())
            .collect()
    };

    Some(ArtifactInstructions {
        change_name: change.name.clone(),
        artifact_id: artifact.id.clone(),
        schema_name: schema.display_name.clone(),
        change_dir: change.dir.to_string_lossy().to_string(),
        output_path: artifact.output_path.clone(),
        description: artifact.description.clone(),
        instruction: artifact.instruction.clone(),
        context: wf.context_text(),
        rules: wf.rules_for(&artifact.id),
        locale: crate::config::resolve_locale(&app, &wf),
        // Spectra fills the payload template by looking the artifact up in the BUILT-IN schema
        // matching the yaml display name — never from the custom templates/ file (which only
        // `new artifact` reads). A custom display name therefore yields an empty template.
        template: if schema.is_builtin() {
            artifact.template.clone().unwrap_or_default()
        } else {
            crate::schema::builtin_template(&schema.display_name, &artifact.id).unwrap_or_default()
        },
        dependencies,
        unlocks,
    })
}

#[derive(Debug, Serialize)]
pub struct Progress {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
}

#[derive(Debug, Serialize)]
pub struct TaskJson {
    pub id: String,
    pub description: String,
    pub done: bool,
    pub parallel: bool,
}

impl From<&Task> for TaskJson {
    fn from(t: &Task) -> Self {
        TaskJson {
            id: t.id.to_string(),
            description: t.description.clone(),
            done: t.done,
            parallel: t.parallel,
        }
    }
}

/// Apply-mode instructions payload (field order matches Spectra).
#[derive(Debug, Serialize)]
pub struct ApplyInstructions {
    #[serde(rename = "changeName")]
    pub change_name: String,
    #[serde(rename = "changeDir")]
    pub change_dir: String,
    #[serde(rename = "schemaName")]
    pub schema_name: String,
    #[serde(rename = "contextFiles")]
    pub context_files: std::collections::BTreeMap<String, String>,
    pub progress: Progress,
    pub tasks: Vec<TaskJson>,
    pub state: String,
    #[serde(rename = "missingArtifacts", skip_serializing_if = "Option::is_none")]
    pub missing_artifacts: Option<Vec<String>>,
    pub locale: String,
    /// Omitted when the (custom) schema defines no apply instruction, matching Spectra.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
}

/// Compute apply state: blocked | ready | all_done.
pub fn apply_state(schema: &Schema, change: &Change, tasks: &[Task]) -> String {
    let tasks_artifact = schema.artifact("tasks");
    let tasks_done = tasks_artifact
        .map(|a| model::artifact_done(&change.dir, a))
        .unwrap_or(false);
    if !tasks_done || tasks.is_empty() {
        return "blocked".to_string();
    }
    let (total, complete, _) = tasks::progress(tasks);
    if total > 0 && complete == total {
        "all_done".to_string()
    } else {
        "ready".to_string()
    }
}

pub fn build_apply(paths: &Paths, change: &Change, schema: &Schema) -> ApplyInstructions {
    let app = AppConfig::load(&paths.app_config());
    let wf = WorkflowConfig::load(&paths.workflow_config());
    let tasks_md = std::fs::read_to_string(change.dir.join("tasks.md")).unwrap_or_default();
    let parsed = tasks::parse(&tasks_md);
    let (total, complete, remaining) = tasks::progress(&parsed);

    // contextFiles includes artifacts whose files exist (empty files count, matching Spectra).
    let mut context_files = std::collections::BTreeMap::new();
    if change.dir.join("proposal.md").is_file() {
        context_files.insert("proposal".to_string(), join_display(&change.dir, "proposal.md"));
    }
    if !model::spec_files(&change.dir).is_empty() {
        context_files.insert("specs".to_string(), join_display(&change.dir, "specs/**/*.md"));
    }
    if change.dir.join("design.md").is_file() {
        context_files.insert("design".to_string(), join_display(&change.dir, "design.md"));
    }
    if change.dir.join("tasks.md").is_file() {
        context_files.insert("tasks".to_string(), join_display(&change.dir, "tasks.md"));
    }

    let state = apply_state(schema, change, &parsed);
    let blocked = state == "blocked";

    let missing_artifacts = if blocked {
        Some(
            schema
                .apply_requires
                .iter()
                .filter(|id| {
                    schema
                        .artifact(id)
                        .map(|a| !model::artifact_done(&change.dir, a))
                        .unwrap_or(true)
                })
                .map(|s| s.to_string())
                .collect(),
        )
    } else {
        None
    };
    let preflight = if blocked {
        None
    } else {
        Some(Preflight::compute(paths, change))
    };

    ApplyInstructions {
        change_name: change.name.clone(),
        change_dir: change.dir.to_string_lossy().to_string(),
        schema_name: schema.display_name.clone(),
        context_files,
        progress: Progress {
            total,
            complete,
            remaining,
        },
        tasks: parsed.iter().map(TaskJson::from).collect(),
        state,
        missing_artifacts,
        locale: crate::config::resolve_locale(&app, &wf),
        instruction: schema.apply_instruction.clone(),
        preflight,
    }
}

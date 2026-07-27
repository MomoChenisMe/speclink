//! Build `instructions` payloads (per-artifact and apply mode) with config injection.

use crate::config::{AppConfig, WorkflowConfig};
use crate::model::{self, Change};
use crate::preflight::Preflight;
use crate::schema::Schema;
use crate::store::Store;
use crate::tasks::{self, Task};
use crate::workspace::Workspace;
use serde::Serialize;
use std::path::Path;

fn join_display(base: &Path, rel: &str) -> String {
    base.join(rel).to_string_lossy().to_string()
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Dependency {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

/// Per-artifact instructions payload (frozen field order).
#[derive(Debug, Serialize, serde::Deserialize)]
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
    /// Omitted entirely when the (custom) schema has no instruction (frozen output shape).
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

/// Build per-artifact instructions. `Ok(None)` = unknown artifact id; `Err` =
/// a config file exists but cannot be parsed (fail-closed, no default policy).
pub fn build_artifact(
    ws: &Workspace,
    store: &dyn Store,
    env: &crate::config::EnvOverrides,
    change: &Change,
    schema: &Schema,
    artifact_id: &str,
) -> Result<Option<ArtifactInstructions>, crate::config::ConfigError> {
    let Some(artifact) = schema.artifact(artifact_id) else {
        return Ok(None);
    };
    let app = AppConfig::load(&ws.app_config())?;
    let wf = WorkflowConfig::from_text(store.read_workflow_config().as_deref())?;
    // Policy values come from the four-layer resolution (env > legacy app key >
    // config.yaml > default) — never from one config file alone. The env layer
    // arrives injected from the Host boundary.
    let policy = crate::config::resolve_policy(env, &app, &wf);

    let dependencies = artifact
        .requires
        .iter()
        .filter_map(|dep_id| {
            let da = schema.artifact(dep_id)?;
            Some(Dependency {
                id: da.id.to_string(),
                done: model::artifact_done(store, &change.name, da),
                path: da.output_path.to_string(),
                description: da.description.to_string(),
            })
        })
        .collect();

    // `unlocks` = downstream artifacts for which THIS artifact is the last unmet dependency:
    // not yet done, list this artifact in their `requires`, and have every other requirement done.
    // Empty once this artifact itself is done (it has already unlocked its dependents).
    let self_done = model::artifact_done(store, &change.name, artifact);
    let unlocks: Vec<String> = if self_done {
        Vec::new()
    } else {
        // Listed in display order (topological tier, alphabetical tiebreak) — frozen output shape.
        crate::status::display_order(schema)
            .into_iter()
            .filter(|y| y.id != artifact.id)
            .filter(|y| y.requires.contains(&artifact.id))
            .filter(|y| !model::artifact_done(store, &change.name, y))
            .filter(|y| {
                y.requires.iter().all(|d| {
                    *d == artifact.id
                        || schema
                            .artifact(d)
                            .map(|da| model::artifact_done(store, &change.name, da))
                            .unwrap_or(false)
                })
            })
            .map(|y| y.id.to_string())
            .collect()
    };

    // Make the configured spec language bite without the agent having to read the config:
    // when spec_locale resolves to a non-English language, the specs instruction states it
    // concretely. Unset (the default) leaves the payload byte-identical.
    let mut instruction = artifact.instruction.clone();
    if artifact.id == "specs" {
        if let Some(lang) = policy.spec_locale.as_deref() {
            let display = crate::config::locale_display(Some(lang));
            let lower = lang.to_ascii_lowercase();
            let cjk_note = if lower == "tw" || lower.starts_with("zh") {
                " Vague Chinese wording (應該、可能、也許、或許、大概、考慮、盡量、待定) is \
flagged by the analyzer just like should/may/TBD — state requirements with SHALL/MUST."
            } else {
                ""
            };
            let note = format!(
                "This project sets `spec_locale: {lang}` — write spec prose in {display}. \
Structural markers (`## ADDED/MODIFIED/REMOVED/RENAMED Requirements`, `### Requirement:`, \
`#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST) still stay in English.{cjk_note}"
            );
            match instruction.as_mut() {
                Some(s) => {
                    s.push_str("\n\n");
                    s.push_str(&note);
                }
                None => instruction = Some(note),
            }
        }
    }
    // Same pattern for the workflow toggles: when the RESOLVED policy enables them, the
    // tasks instruction states the discipline concretely — no new `--json` fields.
    if artifact.id == "tasks" {
        let mut notes = Vec::new();
        if policy.tdd {
            notes.push(
                "This project enables TDD — structure implementation tasks in \
Red-Green-Refactor order: each unit of work starts with a failing-test task, then the \
implementation task that turns it green (fetch the full discipline with `speclink \
instructions --skill tdd`).",
            );
        }
        if policy.audit {
            notes.push(
                "This project enables the audit discipline — when tasks add APIs, \
configuration options, or parameter handling, include a step to apply the sharp-edges \
audit checklist (fetch it with `speclink instructions --skill audit`).",
            );
        }
        for note in notes {
            match instruction.as_mut() {
                Some(s) => {
                    s.push_str("\n\n");
                    s.push_str(note);
                }
                None => instruction = Some(note.to_string()),
            }
        }
    }

    Ok(Some(ArtifactInstructions {
        change_name: change.name.clone(),
        artifact_id: artifact.id.clone(),
        schema_name: schema.display_name.clone(),
        change_dir: change.dir.to_string_lossy().to_string(),
        output_path: artifact.output_path.clone(),
        description: artifact.description.clone(),
        instruction,
        context: wf.context_text(),
        rules: wf.rules_for(&artifact.id),
        locale: policy.locale,
        // The payload template is looked up in the BUILT-IN schema matching the yaml display
        // name — never in the custom templates/ file (which only
        // `new artifact` reads). A custom display name therefore yields an empty template.
        template: if schema.is_builtin() {
            artifact.template.clone().unwrap_or_default()
        } else {
            crate::schema::builtin_template(&schema.display_name, &artifact.id).unwrap_or_default()
        },
        dependencies,
        unlocks,
    }))
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Progress {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
}

#[derive(Debug, Serialize, serde::Deserialize)]
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

/// Apply-mode instructions payload (frozen field order).
#[derive(Debug, Serialize, serde::Deserialize)]
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
    /// Omitted when the (custom) schema defines no apply instruction (frozen output shape).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
}

/// Remote mode retargets apply `contextFiles` onto the Context Projection
/// mirror (platform architecture §7): the key set and the server's
/// collection logic stay untouched — every value becomes the corresponding
/// path under the projection's spec-root mirror, so skills read the
/// read-only projection instead of unreachable server paths. Never called
/// on the local fs path, whose output stays byte-identical.
pub fn project_context_files(
    context_files: &mut std::collections::BTreeMap<String, String>,
    projection_spec_root: &Path,
    change_name: &str,
) {
    let change_dir = projection_spec_root.join("changes").join(change_name);
    for (key, value) in context_files.iter_mut() {
        *value = match key.as_str() {
            // The spec-driven artifact locations build_apply emits.
            "proposal" => join_display(&change_dir, "proposal.md"),
            "design" => join_display(&change_dir, "design.md"),
            "tasks" => join_display(&change_dir, "tasks.md"),
            "specs" => join_display(&change_dir, "specs/**/*.md"),
            // Unknown keys keep the server's spec-root-relative value,
            // re-rooted under the projection mirror.
            _ => join_display(projection_spec_root, value),
        };
    }
}

/// Compute apply state: blocked | ready | all_done.
pub fn apply_state(schema: &Schema, store: &dyn Store, change: &Change, tasks: &[Task]) -> String {
    let tasks_artifact = schema.artifact("tasks");
    let tasks_done = tasks_artifact
        .map(|a| model::artifact_done(store, &change.name, a))
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

pub fn build_apply(
    ws: &Workspace,
    store: &dyn Store,
    env: &crate::config::EnvOverrides,
    change: &Change,
    schema: &Schema,
) -> Result<ApplyInstructions, crate::config::ConfigError> {
    let app = AppConfig::load(&ws.app_config())?;
    let wf = WorkflowConfig::from_text(store.read_workflow_config().as_deref())?;
    let policy = crate::config::resolve_policy(env, &app, &wf);
    let tasks_md = store.read_artifact(&change.name, "tasks.md").unwrap_or_default();
    let parsed = tasks::parse(&tasks_md);
    let (total, complete, remaining) = tasks::progress(&parsed);

    // contextFiles includes artifacts whose files exist (empty files count) — frozen output shape.
    let mut context_files = std::collections::BTreeMap::new();
    if store.artifact_exists(&change.name, "proposal.md") {
        context_files.insert("proposal".to_string(), join_display(&change.dir, "proposal.md"));
    }
    if !store.delta_capabilities(&change.name).is_empty() {
        context_files.insert("specs".to_string(), join_display(&change.dir, "specs/**/*.md"));
    }
    if store.artifact_exists(&change.name, "design.md") {
        context_files.insert("design".to_string(), join_display(&change.dir, "design.md"));
    }
    if store.artifact_exists(&change.name, "tasks.md") {
        context_files.insert("tasks".to_string(), join_display(&change.dir, "tasks.md"));
    }

    let state = apply_state(schema, store, change, &parsed);
    let blocked = state == "blocked";

    let missing_artifacts = if blocked {
        let missing: Vec<String> = schema
            .apply_requires
            .iter()
            .filter(|id| {
                schema
                    .artifact(id)
                    .map(|a| !model::artifact_done(store, &change.name, a))
                    .unwrap_or(true)
            })
            .map(|s| s.to_string())
            .collect();
        // Blocked by zero checkboxes rather than a missing artifact → the key is
        // omitted entirely (frozen output shape: no empty missingArtifacts array).
        if missing.is_empty() {
            None
        } else {
            Some(missing)
        }
    } else {
        None
    };
    // Preflight only in the ready state — the field drops out again once all tasks are done.
    let preflight = if state == "ready" {
        Some(Preflight::compute(ws, store, change))
    } else {
        None
    };

    Ok(ApplyInstructions {
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
        locale: policy.locale,
        instruction: schema.apply_instruction.clone(),
        preflight,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- remote contextFiles 重定向：key 與集合邏輯不變、值指向投影鏡像 ---

    #[test]
    fn project_context_files_retargets_values_and_keeps_keys() {
        let mut files = std::collections::BTreeMap::from([
            ("design".to_string(), "changes/demo/design.md".to_string()),
            ("proposal".to_string(), "changes/demo/proposal.md".to_string()),
            ("specs".to_string(), "changes/demo/specs/**/*.md".to_string()),
            ("tasks".to_string(), "changes/demo/tasks.md".to_string()),
            ("extra".to_string(), "changes/demo/extra.md".to_string()),
        ]);
        let root = Path::new("/ws/.speclink/context/openspec");
        project_context_files(&mut files, root, "demo");

        // key 集合不變。
        let keys: Vec<&str> = files.keys().map(String::as_str).collect();
        assert_eq!(keys, ["design", "extra", "proposal", "specs", "tasks"]);

        // 已知 key 的值 = 投影鏡像下該 change 的對應路徑。
        let change_dir = root.join("changes").join("demo");
        assert_eq!(files["proposal"], change_dir.join("proposal.md").to_string_lossy());
        assert_eq!(files["design"], change_dir.join("design.md").to_string_lossy());
        assert_eq!(files["tasks"], change_dir.join("tasks.md").to_string_lossy());
        assert_eq!(files["specs"], change_dir.join("specs/**/*.md").to_string_lossy());
        // 未知 key 保留 server 相對值、改掛投影鏡像根下。
        assert_eq!(files["extra"], root.join("changes/demo/extra.md").to_string_lossy());

        // 每個值都在投影下。
        for v in files.values() {
            assert!(v.starts_with(&root.to_string_lossy().to_string()), "{v} under projection");
        }
    }
}

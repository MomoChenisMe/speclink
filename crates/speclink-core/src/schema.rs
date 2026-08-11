//! Workflow schemas: the embedded `spec-driven` schema plus custom schemas in the OpenSpec
//! format (`openspec/schemas/<name>/schema.yaml` + `templates/`).
//! Resolution order: project → user → built-in.

use crate::workspace::Workspace;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A single artifact definition within a schema.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    /// Output path pattern relative to the change directory (`generates`).
    pub output_path: String,
    /// Template file name (shown by `templates`).
    pub template_name: String,
    pub description: String,
    /// Artifact ids that must be done before this one is ready.
    pub requires: Vec<String>,
    /// Guidance text; custom schemas may omit it.
    pub instruction: Option<String>,
    /// Template content; None when the template file is missing.
    pub template: Option<String>,
}

/// A workflow schema.
#[derive(Debug, Clone)]
pub struct Schema {
    /// Resolution key — for custom schemas this is the directory name.
    pub name: String,
    /// The yaml `name:` field — what payloads and `status` display as the schema name
    /// (a pristine fork still reports "spec-driven").
    pub display_name: String,
    /// Display description. Custom schemas show none (frozen output shape).
    pub description: Option<String>,
    /// "package" | "project" | "user"
    pub source: String,
    pub artifacts: Vec<Artifact>,
    /// Artifact ids required before `apply`.
    pub apply_requires: Vec<String>,
    pub apply_tracks: Option<String>,
    pub apply_instruction: Option<String>,
}

impl Schema {
    pub fn artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }
    pub fn artifact_ids(&self) -> Vec<String> {
        self.artifacts.iter().map(|a| a.id.clone()).collect()
    }
    pub fn is_builtin(&self) -> bool {
        self.source == "package"
    }
}

const A_PROPOSAL_INSTR: &str =
    include_str!("../assets/schema/spec-driven/proposal.instruction.md");
const A_PROPOSAL_TMPL: &str = include_str!("../assets/schema/spec-driven/proposal.template.md");
const A_SPECS_INSTR: &str = include_str!("../assets/schema/spec-driven/specs.instruction.md");
const A_SPECS_TMPL: &str = include_str!("../assets/schema/spec-driven/specs.template.md");
const A_DESIGN_INSTR: &str = include_str!("../assets/schema/spec-driven/design.instruction.md");
const A_DESIGN_TMPL: &str = include_str!("../assets/schema/spec-driven/design.template.md");
const A_TASKS_INSTR: &str = include_str!("../assets/schema/spec-driven/tasks.instruction.md");
const A_TASKS_TMPL: &str = include_str!("../assets/schema/spec-driven/tasks.template.md");
const A_APPLY_INSTR: &str = include_str!("../assets/schema/spec-driven/apply.instruction.md");
/// The YAML dump `schema fork spec-driven` produces, shipped verbatim.
pub const FORK_SCHEMA_YAML: &str =
    include_str!("../assets/schema/spec-driven/fork.schema.yaml");

/// The built-in `spec-driven` schema.
pub fn spec_driven() -> Schema {
    let art = |id: &str, out: &str, tmpl: &str, desc: &str, req: &[&str], instr: &str, body: &str| Artifact {
        id: id.to_string(),
        output_path: out.to_string(),
        template_name: tmpl.to_string(),
        description: desc.to_string(),
        requires: req.iter().map(|s| s.to_string()).collect(),
        instruction: Some(instr.to_string()),
        template: Some(body.to_string()),
    };
    Schema {
        name: "spec-driven".to_string(),
        display_name: "spec-driven".to_string(),
        description: Some("Default OpenSpec workflow - proposal → specs → design → tasks".to_string()),
        source: "package".to_string(),
        apply_requires: vec!["tasks".to_string()],
        apply_tracks: Some("tasks.md".to_string()),
        apply_instruction: Some(A_APPLY_INSTR.to_string()),
        artifacts: vec![
            art("proposal", "proposal.md", "proposal.md", "Initial proposal document outlining the change", &[], A_PROPOSAL_INSTR, A_PROPOSAL_TMPL),
            art("specs", "specs/**/*.md", "spec.md", "Detailed specifications for the change", &["proposal"], A_SPECS_INSTR, A_SPECS_TMPL),
            art("design", "design.md", "design.md", "Technical design document with implementation details", &["proposal"], A_DESIGN_INSTR, A_DESIGN_TMPL),
            art("tasks", "tasks.md", "tasks.md", "Implementation checklist with trackable tasks", &["specs"], A_TASKS_INSTR, A_TASKS_TMPL),
        ],
    }
}

// --- YAML schema files ---

#[derive(Deserialize)]
struct SchemaYaml {
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    version: Option<serde_yaml::Value>,
    #[serde(default)]
    description: Option<String>,
    artifacts: Vec<ArtifactYaml>,
    #[serde(default)]
    apply: Option<ApplyYaml>,
}

#[derive(Deserialize)]
struct ArtifactYaml {
    id: String,
    generates: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    instruction: Option<String>,
    #[serde(default)]
    requires: Vec<String>,
}

#[derive(Deserialize)]
struct ApplyYaml {
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    tracks: Option<String>,
    #[serde(default)]
    instruction: Option<String>,
}

/// DFS cycle check over the artifact `requires` graph.
fn has_cycle(arts: &[ArtifactYaml]) -> bool {
    fn visit(id: &str, arts: &[ArtifactYaml], path: &mut Vec<String>, done: &mut Vec<String>) -> bool {
        if done.iter().any(|d| d == id) {
            return false;
        }
        if path.iter().any(|p| p == id) {
            return true;
        }
        path.push(id.to_string());
        if let Some(a) = arts.iter().find(|a| a.id == id) {
            for r in &a.requires {
                if visit(r, arts, path, done) {
                    return true;
                }
            }
        }
        path.pop();
        done.push(id.to_string());
        false
    }
    let mut done = Vec::new();
    arts.iter().any(|a| visit(&a.id, arts, &mut Vec::new(), &mut done))
}

/// Load a custom schema from its directory. `name` is the directory name (the resolution key —
/// the yaml `name:` field is ignored for resolution).
pub fn load_dir(dir: &Path, name: &str, source: &str) -> Result<Schema, String> {
    let text = std::fs::read_to_string(dir.join("schema.yaml"))
        .map_err(|e| format!("Schema parse error: {e}"))?;
    let y: SchemaYaml =
        serde_yaml::from_str(&text).map_err(|e| format!("Schema parse error: {e}"))?;
    if has_cycle(&y.artifacts) {
        return Err("Invalid schema: Schema contains circular dependencies".to_string());
    }
    let _ = &y.description; // custom schemas display no description (frozen output shape)
    let artifacts = y
        .artifacts
        .into_iter()
        .map(|a| {
            let template_name = a.template.unwrap_or_else(|| format!("{}.md", a.id));
            let template = crate::util::read_opt(&dir.join("templates").join(&template_name));
            Artifact {
                output_path: a.generates,
                template_name,
                description: a.description.unwrap_or_default(),
                requires: a.requires,
                instruction: a.instruction,
                template,
                id: a.id,
            }
        })
        .collect();
    let apply = y.apply;
    Ok(Schema {
        name: name.to_string(),
        display_name: y.name,
        description: None,
        source: source.to_string(),
        artifacts,
        apply_requires: apply.as_ref().map(|a| a.requires.clone()).unwrap_or_default(),
        apply_tracks: apply.as_ref().and_then(|a| a.tracks.clone()),
        apply_instruction: apply.and_then(|a| a.instruction),
    })
}

// --- resolution ---

/// Schema search directories in resolution order: project then user.
/// `user_dir` is the Host-resolved machine-level speclink directory
/// (speclink-host's `global_config_dir`) — None skips the user location.
pub fn schema_dirs(
    ws: Option<&Workspace>,
    user_dir: Option<&Path>,
) -> Vec<(PathBuf, &'static str)> {
    let mut v = Vec::new();
    if let Some(w) = ws {
        v.push((w.spec_dir().join("schemas"), "project"));
    }
    if let Some(dir) = user_dir {
        v.push((dir.join("schemas"), "user"));
    }
    v
}

/// One place a schema name resolves to. `path` is None for the built-in.
pub struct SchemaSource {
    pub path: Option<PathBuf>,
    pub source: &'static str,
}

/// Every location where `name` exists, in resolution order.
pub fn sources(ws: Option<&Workspace>, user_dir: Option<&Path>, name: &str) -> Vec<SchemaSource> {
    let mut out = Vec::new();
    for (dir, src) in schema_dirs(ws, user_dir) {
        let y = dir.join(name).join("schema.yaml");
        if y.is_file() {
            out.push(SchemaSource { path: Some(y), source: src });
        }
    }
    if name == "spec-driven" {
        out.push(SchemaSource { path: None, source: "built-in" });
    }
    out
}

/// Resolve a schema: None = not found; Some(Err) = found but invalid (parse error / cycle).
pub fn resolve_with(
    ws: Option<&Workspace>,
    user_dir: Option<&Path>,
    name: &str,
) -> Option<Result<Schema, String>> {
    for (dir, src) in schema_dirs(ws, user_dir) {
        let d = dir.join(name);
        if d.join("schema.yaml").is_file() {
            return Some(load_dir(&d, name, src));
        }
    }
    if name == "spec-driven" {
        return Some(Ok(spec_driven()));
    }
    None
}

/// The not-found message for a schema name.
pub fn not_found_msg(name: &str) -> String {
    format!("Schema not found: Schema '{name}' not found in project, user, or built-in locations")
}

/// The built-in template for an artifact id, looked up by a schema's DISPLAY name — the
/// instructions payload template is filled this way (custom display names get an empty template).
pub fn builtin_template(display_name: &str, artifact_id: &str) -> Option<String> {
    if display_name != "spec-driven" {
        return None;
    }
    spec_driven().artifact(artifact_id).and_then(|a| a.template.clone())
}

/// A schema entry for the `schemas` listing (parse failures tolerated → empty artifacts).
pub struct ListedSchema {
    pub name: String,
    pub source: &'static str,
    pub description: Option<String>,
    pub artifact_ids: Vec<String>,
}

/// All schemas: built-in first, then project (alphabetical), then user (alphabetical).
pub fn list_all(ws: Option<&Workspace>, user_dir: Option<&Path>) -> Vec<ListedSchema> {
    let b = spec_driven();
    let mut out = vec![ListedSchema {
        name: b.name.clone(),
        source: "package",
        description: b.description.clone(),
        artifact_ids: b.artifact_ids(),
    }];
    for (dir, src) in schema_dirs(ws, user_dir) {
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .map(|it| {
                it.flatten()
                    .filter(|e| e.path().join("schema.yaml").is_file())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        for n in names {
            let ids = std::fs::read_to_string(dir.join(&n).join("schema.yaml"))
                .ok()
                .and_then(|t| serde_yaml::from_str::<SchemaYaml>(&t).ok())
                .map(|y| y.artifacts.iter().map(|a| a.id.clone()).collect())
                .unwrap_or_default();
            out.push(ListedSchema { name: n, source: src, description: None, artifact_ids: ids });
        }
    }
    out
}

// --- fork / init ---

/// Fork (copy) a schema into the project's `openspec/schemas/`. Returns the new name.
pub fn fork(
    ws: &Workspace,
    user_dir: Option<&Path>,
    source: &str,
    name: Option<&str>,
    force: bool,
) -> Result<String, String> {
    let new_name = name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{source}-custom"));
    let target = ws.spec_dir().join("schemas").join(&new_name);
    if target.join("schema.yaml").is_file() && !force {
        return Err(format!("Schema '{new_name}' already exists. Use --force to overwrite."));
    }
    let srcs = sources(Some(ws), user_dir, source);
    let Some(first) = srcs.first() else {
        return Err(not_found_msg(source));
    };
    let write = |rel: &str, body: &str| -> Result<(), String> {
        crate::util::write_file(&target.join(rel), body).map_err(|e| e.to_string())
    };
    match &first.path {
        None => {
            // Built-in: ship the verbatim fork dump + the four templates.
            write("schema.yaml", FORK_SCHEMA_YAML)?;
            write("templates/proposal.md", A_PROPOSAL_TMPL)?;
            write("templates/spec.md", A_SPECS_TMPL)?;
            write("templates/design.md", A_DESIGN_TMPL)?;
            write("templates/tasks.md", A_TASKS_TMPL)?;
        }
        Some(yaml_path) => {
            let src_dir = yaml_path.parent().unwrap_or(Path::new("."));
            let body = std::fs::read_to_string(yaml_path).map_err(|e| e.to_string())?;
            write("schema.yaml", &body)?;
            if let Ok(entries) = std::fs::read_dir(src_dir.join("templates")) {
                for e in entries.flatten() {
                    if e.path().is_file() {
                        let fname = e.file_name().to_string_lossy().to_string();
                        let content = std::fs::read_to_string(e.path()).map_err(|e| e.to_string())?;
                        write(&format!("templates/{fname}"), &content)?;
                    }
                }
            }
        }
    }
    Ok(new_name)
}

/// Create a new custom schema skeleton. Returns its directory.
pub fn init_schema(
    ws: &Workspace,
    name: &str,
    artifacts: Option<&str>,
    description: Option<&str>,
    force: bool,
) -> Result<PathBuf, String> {
    let target = ws.spec_dir().join("schemas").join(name);
    if target.join("schema.yaml").is_file() && !force {
        return Err(format!("Schema '{name}' already exists. Use --force to overwrite."));
    }
    let ids: Vec<String> = artifacts
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
        .filter(|v: &Vec<String>| !v.is_empty())
        .unwrap_or_else(|| vec!["plan".to_string(), "tasks".to_string()]);
    let desc = description
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Custom schema: {name}"));

    let mut y = String::new();
    y.push_str(&format!("name: {name}\n"));
    y.push_str("version: 1\n");
    y.push_str(&format!("description: {desc}\n"));
    y.push('\n');
    y.push_str("artifacts:\n");
    for (i, id) in ids.iter().enumerate() {
        y.push_str(&format!("  - id: {id}\n"));
        y.push_str(&format!("    generates: \"{id}.md\"\n"));
        y.push_str(&format!("    description: \"The {id} artifact\"\n"));
        y.push_str(&format!("    template: \"{id}.md\"\n"));
        if i > 0 {
            y.push_str("    requires:\n");
            y.push_str(&format!("      - {}\n", ids[i - 1]));
        }
    }
    y.push('\n');
    y.push_str("apply:\n");
    y.push_str("  requires:\n");
    y.push_str(&format!("    - {}\n", ids[ids.len() - 1]));
    y.push_str(&format!("  tracks: \"{}.md\"\n", ids[ids.len() - 1]));

    crate::util::write_file(&target.join("schema.yaml"), &y).map_err(|e| e.to_string())?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::{A_TASKS_INSTR, FORK_SCHEMA_YAML};

    #[test]
    fn tasks_drafting_guidance_names_the_manual_marker_only() {
        // `[P]` 語意已移除——翻譯保留規則點名的是唯一承載語意的 `[M]`；起草指引
        // 與 fork schema 內嵌的同一段規則必須同步，否則 fork 出去的專案落差。
        for (label, asset) in [("tasks.instruction.md", A_TASKS_INSTR), ("fork.schema.yaml", FORK_SCHEMA_YAML)] {
            assert!(asset.contains("`[M]` markers"), "{label} 須點名 `[M]` markers");
            // 斷言面只釘翻譯保留規則那一句的舊 token——全文否定「[P]」會把
            // 日後任何合法提及(如遷移說明)一併打死。
            assert!(!asset.contains("`[P]` markers"), "{label} 不得再點名 `[P]` markers");
        }
    }
}

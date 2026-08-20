//! Workflow schemas: the embedded `spec-driven` schema plus custom schemas in the OpenSpec
//! format (`openspec/schemas/<name>/schema.yaml` + `templates/`).
//! Resolution order: project → user → built-in.

use crate::workspace::Workspace;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

const A_PROPOSAL_TMPL: &str = include_str!("../assets/schema/spec-driven/proposal.template.md");
const A_SPECS_TMPL: &str = include_str!("../assets/schema/spec-driven/specs.template.md");
const A_DESIGN_TMPL: &str = include_str!("../assets/schema/spec-driven/design.template.md");
const A_TASKS_TMPL: &str = include_str!("../assets/schema/spec-driven/tasks.template.md");
/// The single canonical definition of the built-in `spec-driven` schema: the engine
/// parses it at load, and `schema fork spec-driven` ships it byte for byte.
pub const FORK_SCHEMA_YAML: &str =
    include_str!("../assets/schema/spec-driven/fork.schema.yaml");

/// Template content for the built-in schema, keyed by the `template:` file name the
/// canonical YAML declares. This is the built-in's counterpart to a custom schema's
/// `templates/` directory — the same lookup, served from the binary.
fn builtin_template_asset(file_name: &str) -> Option<String> {
    let body = match file_name {
        "proposal.md" => A_PROPOSAL_TMPL,
        "spec.md" => A_SPECS_TMPL,
        "design.md" => A_DESIGN_TMPL,
        "tasks.md" => A_TASKS_TMPL,
        _ => return None,
    };
    Some(body.to_string())
}

/// The built-in `spec-driven` schema, parsed from [`FORK_SCHEMA_YAML`] through the same
/// path a custom schema takes. Parsed once per process; the canonical document is
/// compiled in, so a parse or validation failure is a build-time defect, not user input.
pub fn spec_driven() -> Schema {
    static BUILT_IN: OnceLock<Schema> = OnceLock::new();
    BUILT_IN
        .get_or_init(|| {
            parse_schema(FORK_SCHEMA_YAML, "spec-driven", "package", &builtin_template_asset)
                .expect("built-in spec-driven schema parses and validates")
        })
        .clone()
}

// --- YAML schema files ---

#[derive(Deserialize)]
struct SchemaYaml {
    name: String,
    /// Kept as a raw value so a non-integer reads as a schema error naming the field,
    /// not as a serde type error naming a Rust type.
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

/// DFS over the artifact `requires` graph; returns the first cycle as `a → b → a`.
fn cycle_path(arts: &[ArtifactYaml]) -> Option<String> {
    fn visit(
        id: &str,
        arts: &[ArtifactYaml],
        path: &mut Vec<String>,
        done: &mut Vec<String>,
    ) -> Option<String> {
        if done.iter().any(|d| d == id) {
            return None;
        }
        if let Some(at) = path.iter().position(|p| p == id) {
            let mut cycle: Vec<&str> = path[at..].iter().map(String::as_str).collect();
            cycle.push(id);
            return Some(cycle.join(" → "));
        }
        path.push(id.to_string());
        if let Some(a) = arts.iter().find(|a| a.id == id) {
            for r in &a.requires {
                if let Some(cycle) = visit(r, arts, path, done) {
                    return Some(cycle);
                }
            }
        }
        path.pop();
        done.push(id.to_string());
        None
    }
    let mut done = Vec::new();
    arts.iter()
        .find_map(|a| visit(&a.id, arts, &mut Vec::new(), &mut done))
}

/// Every structural check a schema document must pass before any verb may use it.
/// Mirrors the OpenSpec zod schema: required fields first, then graph integrity.
fn validate_yaml(y: &SchemaYaml) -> Result<(), String> {
    let invalid = |detail: String| Err(format!("Invalid schema: {detail}"));

    match &y.version {
        None => return invalid("version is required".to_string()),
        Some(v) if v.as_u64().is_none_or(|n| n == 0) => {
            return invalid("version must be a positive integer".to_string())
        }
        Some(_) => {}
    }

    let mut seen: Vec<&str> = Vec::with_capacity(y.artifacts.len());
    for a in &y.artifacts {
        if seen.contains(&a.id.as_str()) {
            return invalid(format!("Duplicate artifact ID: {}", a.id));
        }
        seen.push(&a.id);
        if a.description.is_none() {
            return invalid(format!("artifact '{}': description is required", a.id));
        }
        if a.template.as_deref().unwrap_or_default().is_empty() {
            return invalid(format!("artifact '{}': template is required", a.id));
        }
    }

    for a in &y.artifacts {
        for r in &a.requires {
            if !seen.contains(&r.as_str()) {
                return invalid(format!(
                    "Invalid dependency reference in artifact '{}': '{r}' does not exist",
                    a.id
                ));
            }
        }
    }

    match cycle_path(&y.artifacts) {
        Some(path) => invalid(format!("Cyclic dependency detected: {path}")),
        None => Ok(()),
    }
}

/// Schema name format, mirroring OpenSpec's `isValidSchemaName`: lowercase kebab-case —
/// a letter, then lowercase alphanumerics, with single hyphens between segments.
pub(crate) fn is_valid_schema_name(name: &str) -> bool {
    let mut segments = name.split('-');
    segments.next().is_some_and(|first| {
        first.starts_with(|c: char| c.is_ascii_lowercase())
            && first.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    }) && segments.all(|seg| {
        !seg.is_empty() && seg.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

/// Parse and validate one schema document. `templates` supplies template content by the
/// `template:` file name an artifact declares — a directory read for custom schemas, an
/// embedded lookup for the built-in.
fn parse_schema(
    text: &str,
    name: &str,
    source: &str,
    templates: &dyn Fn(&str) -> Option<String>,
) -> Result<Schema, String> {
    let y: SchemaYaml =
        serde_yaml::from_str(text).map_err(|e| format!("Schema parse error: {e}"))?;
    validate_yaml(&y)?;
    let artifacts = y
        .artifacts
        .into_iter()
        .map(|a| {
            // `validate_yaml` already refused an absent or empty template name.
            let template_name = a.template.unwrap_or_default();
            let template = templates(&template_name);
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
        // Custom schemas display no description (frozen output shape); the built-in shows
        // the canonical document's own.
        description: if source == "package" { y.description } else { None },
        source: source.to_string(),
        artifacts,
        apply_requires: apply.as_ref().map(|a| a.requires.clone()).unwrap_or_default(),
        apply_tracks: apply.as_ref().and_then(|a| a.tracks.clone()),
        apply_instruction: apply.and_then(|a| a.instruction),
    })
}

/// Load a custom schema from its directory. `name` is the directory name (the resolution key —
/// the yaml `name:` field is ignored for resolution).
pub fn load_dir(dir: &Path, name: &str, source: &str) -> Result<Schema, String> {
    let text = std::fs::read_to_string(dir.join("schema.yaml"))
        .map_err(|e| format!("Schema parse error: {e}"))?;
    let templates_dir = dir.join("templates");
    parse_schema(&text, name, source, &|file_name| {
        crate::util::read_opt(&templates_dir.join(file_name))
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

/// Template files the schema references but cannot supply — the content lookup came back
/// empty (a missing or unreadable file for a custom schema, an absent embedded asset for
/// the built-in). This is `schema validate`'s file-existence check; load deliberately
/// tolerates a missing template FILE (design D3).
pub fn missing_templates(schema: &Schema) -> Vec<String> {
    schema
        .artifacts
        .iter()
        .filter(|a| a.template.is_none())
        .map(|a| a.template_name.clone())
        .collect()
}

/// The rejection message for a schema name that is not lowercase kebab-case.
fn invalid_name_msg(name: &str) -> String {
    format!("Invalid schema name '{name}': must be lowercase kebab-case (e.g. my-flow)")
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
    if !is_valid_schema_name(&new_name) {
        return Err(invalid_name_msg(&new_name));
    }
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
///
/// The document is serialized as one YAML value, never string-concatenated — any scalar
/// (a description carrying `: ` or newlines) stays parseable. Artifact ids double as
/// template file names on disk, so they carry the same kebab-case rule as schema names —
/// that is also what keeps them from escaping the schema directory. The generated
/// document must pass its own load checks before anything reaches the disk: init never
/// leaves a skeleton that its own `schema validate` refuses.
pub fn init_schema(
    ws: &Workspace,
    name: &str,
    artifacts: Option<&str>,
    description: Option<&str>,
    force: bool,
) -> Result<PathBuf, String> {
    if !is_valid_schema_name(name) {
        return Err(invalid_name_msg(name));
    }
    let target = ws.spec_dir().join("schemas").join(name);
    if target.join("schema.yaml").is_file() && !force {
        return Err(format!("Schema '{name}' already exists. Use --force to overwrite."));
    }
    let ids: Vec<String> = artifacts
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
        .filter(|v: &Vec<String>| !v.is_empty())
        .unwrap_or_else(|| vec!["plan".to_string(), "tasks".to_string()]);
    for id in &ids {
        if !is_valid_schema_name(id) {
            return Err(format!(
                "Invalid artifact id '{id}': must be lowercase kebab-case (e.g. plan)"
            ));
        }
    }
    let desc = description
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Custom schema: {name}"));

    let mut doc = serde_yaml::Mapping::new();
    doc.insert("name".into(), name.into());
    doc.insert("version".into(), 1.into());
    doc.insert("description".into(), desc.into());
    let arts: Vec<serde_yaml::Value> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let mut a = serde_yaml::Mapping::new();
            a.insert("id".into(), id.as_str().into());
            a.insert("generates".into(), format!("{id}.md").into());
            a.insert("description".into(), format!("The {id} artifact").into());
            a.insert("template".into(), format!("{id}.md").into());
            if i > 0 {
                a.insert(
                    "requires".into(),
                    serde_yaml::Value::Sequence(vec![ids[i - 1].as_str().into()]),
                );
            }
            serde_yaml::Value::Mapping(a)
        })
        .collect();
    doc.insert("artifacts".into(), serde_yaml::Value::Sequence(arts));
    let last = &ids[ids.len() - 1];
    let mut apply = serde_yaml::Mapping::new();
    apply.insert("requires".into(), serde_yaml::Value::Sequence(vec![last.as_str().into()]));
    apply.insert("tracks".into(), format!("{last}.md").into());
    doc.insert("apply".into(), serde_yaml::Value::Mapping(apply));
    let y = serde_yaml::to_string(&serde_yaml::Value::Mapping(doc)).map_err(|e| e.to_string())?;

    // Self-check first (duplicate ids land here), write only a cleared document.
    parse_schema(&y, name, "project", &|_| Some(String::new()))?;

    crate::util::write_file(&target.join("schema.yaml"), &y).map_err(|e| e.to_string())?;
    for id in &ids {
        crate::util::write_file(
            &target.join("templates").join(format!("{id}.md")),
            &format!("# {id}\n"),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::{
        is_valid_schema_name, parse_schema, spec_driven, SchemaYaml, A_DESIGN_TMPL,
        A_PROPOSAL_TMPL, A_SPECS_TMPL, A_TASKS_TMPL, FORK_SCHEMA_YAML,
    };

    /// 內建 schema 的每個欄位都必須等於正典 YAML 的字面——手寫第二份定義會在這裡紅燈。
    #[test]
    fn builtin_schema_matches_the_canonical_yaml() {
        let y: SchemaYaml = serde_yaml::from_str(FORK_SCHEMA_YAML).expect("正典 YAML 可解析");
        let s = spec_driven();

        assert_eq!(s.display_name, y.name, "display_name");
        assert_eq!(s.description, y.description, "description");

        let apply = y.apply.as_ref().expect("正典 YAML 帶 apply 區塊");
        assert_eq!(s.apply_requires, apply.requires, "apply.requires");
        assert_eq!(s.apply_tracks, apply.tracks, "apply.tracks");
        assert_eq!(s.apply_instruction, apply.instruction, "apply.instruction");

        assert_eq!(s.artifacts.len(), y.artifacts.len(), "artifact 數量");
        for (got, want) in s.artifacts.iter().zip(&y.artifacts) {
            assert_eq!(got.id, want.id, "artifact id");
            assert_eq!(got.output_path, want.generates, "{}: generates", want.id);
            assert_eq!(
                Some(&got.template_name),
                want.template.as_ref(),
                "{}: template 檔名",
                want.id
            );
            assert_eq!(
                Some(&got.description),
                want.description.as_ref(),
                "{}: description",
                want.id
            );
            assert_eq!(got.instruction, want.instruction, "{}: instruction", want.id);
            assert_eq!(got.requires, want.requires, "{}: requires", want.id);
        }
    }

    /// 正典每個 artifact 宣告的 template 檔名都必須在內嵌資產表有對應內容——
    /// 日後在正典新增 artifact 而忘了補表，在這裡紅燈，而不是 schema validate 執行期。
    #[test]
    fn every_canonical_artifact_attaches_an_embedded_template() {
        for art in spec_driven().artifacts {
            assert!(
                art.template.as_deref().is_some_and(|t| !t.is_empty()),
                "{}: 正典宣告的 template {} 必須在內嵌資產表有對應內容",
                art.id,
                art.template_name
            );
        }
    }

    /// specs 的起草指引承載現行規則——正典 YAML 是 instruction 的唯一來源，落後版本在這裡紅燈。
    #[test]
    fn specs_instruction_carries_the_current_drafting_rules() {
        let specs = spec_driven();
        let instruction = specs
            .artifact("specs")
            .and_then(|a| a.instruction.as_deref())
            .expect("specs artifact 帶 instruction");
        for marker in [
            "Purpose section (new capabilities only)",
            "<!-- BEFORE:",
            "<!-- REMOVED-SCENARIO:",
        ] {
            assert!(
                instruction.contains(marker),
                "specs instruction 須點名 {marker}"
            );
        }
    }

    /// template 內容由內嵌資產附掛，逐字相同。
    #[test]
    fn builtin_templates_come_from_the_embedded_assets() {
        let s = spec_driven();
        for (id, asset) in [
            ("proposal", A_PROPOSAL_TMPL),
            ("specs", A_SPECS_TMPL),
            ("design", A_DESIGN_TMPL),
            ("tasks", A_TASKS_TMPL),
        ] {
            let art = s.artifact(id).unwrap_or_else(|| panic!("內建 schema 有 {id}"));
            assert_eq!(art.template.as_deref(), Some(asset), "{id}: template 內容");
        }
    }

    /// 內建正典必須通過全部載入檢查——引擎自帶的文件不得是唯一的例外。
    #[test]
    fn the_canonical_document_passes_every_check() {
        parse_schema(FORK_SCHEMA_YAML, "spec-driven", "package", &|_| Some(String::new()))
            .expect("正典 YAML 通過全部載入檢查");
    }

    /// 載入檢查的邊界表：每列一個 schema 文件與預期的錯誤片語（None ＝ 必須通過）。
    #[test]
    fn load_rejects_invalid_schema_documents() {
        // 一份合法的最小文件，各列在其上改動一處。
        let ok = "\
name: t
version: 1
artifacts:
- id: plan
  generates: plan.md
  description: The plan
  template: plan.md
";
        let cases: [(&str, &str, Option<&str>); 11] = [
            ("合法最小文件", ok, None),
            (
                "重複 artifact id",
                "name: t\nversion: 1\nartifacts:\n- id: plan\n  generates: a.md\n  description: A\n  template: a.md\n- id: plan\n  generates: b.md\n  description: B\n  template: b.md\n",
                Some("Duplicate artifact ID: plan"),
            ),
            (
                "懸空 requires",
                "name: t\nversion: 1\nartifacts:\n- id: plan\n  generates: a.md\n  description: A\n  template: a.md\n  requires:\n  - ghost\n",
                Some("artifact 'plan'"),
            ),
            (
                "循環相依",
                "name: t\nversion: 1\nartifacts:\n- id: a\n  generates: a.md\n  description: A\n  template: a.md\n  requires:\n  - b\n- id: b\n  generates: b.md\n  description: B\n  template: b.md\n  requires:\n  - a\n",
                Some("a → b → a"),
            ),
            ("version 鍵缺席", &ok.replace("version: 1\n", ""), Some("version is required")),
            ("version 為 0", &ok.replace("version: 1", "version: 0"), Some("positive integer")),
            ("version 為小數", &ok.replace("version: 1", "version: 1.5"), Some("positive integer")),
            (
                "description 鍵缺席",
                &ok.replace("  description: The plan\n", ""),
                Some("description is required"),
            ),
            ("description 為空字串", &ok.replace("description: The plan", "description: \"\""), None),
            (
                "template 鍵缺席",
                &ok.replace("  template: plan.md\n", ""),
                Some("template is required"),
            ),
            (
                "template 為空字串",
                &ok.replace("template: plan.md", "template: \"\""),
                Some("template is required"),
            ),
        ];

        for (label, doc, want) in cases {
            let got = parse_schema(doc, "t", "project", &|_| Some(String::new()));
            match (want, &got) {
                (None, Err(e)) => panic!("{label}：預期通過，卻回錯誤 {e}"),
                (Some(phrase), Ok(_)) => panic!("{label}：預期錯誤含「{phrase}」，卻通過"),
                (Some(phrase), Err(e)) => assert!(
                    e.contains(phrase),
                    "{label}：錯誤訊息須含「{phrase}」，實得 {e}"
                ),
                (None, Ok(_)) => {}
            }
        }
    }

    /// fork／init 的目的名稱格式（對齊 OpenSpec 的 isValidSchemaName）。
    #[test]
    fn schema_names_must_be_lowercase_kebab_case() {
        for good in ["a", "my-flow", "spec-driven", "spec-driven-custom", "a1-b2"] {
            assert!(is_valid_schema_name(good), "{good} 應為合法名稱");
        }
        for bad in ["", "My_Schema", "My-Schema", "1flow", "-flow", "flow-", "a--b", "a b", "a.b"] {
            assert!(!is_valid_schema_name(bad), "{bad} 應為非法名稱");
        }
    }

    #[test]
    fn tasks_drafting_guidance_names_the_manual_marker_only() {
        // `[P]` 語意已移除——翻譯保留規則點名的是唯一承載語意的 `[M]`。起草指引只有
        // 正典 YAML 一份，fork 出去的專案因此不會落差。
        assert!(
            FORK_SCHEMA_YAML.contains("`[M]` markers"),
            "fork.schema.yaml 須點名 `[M]` markers"
        );
        // 斷言面只釘翻譯保留規則那一句的舊 token——全文否定「[P]」會把
        // 日後任何合法提及(如遷移說明)一併打死。
        assert!(
            !FORK_SCHEMA_YAML.contains("`[P]` markers"),
            "fork.schema.yaml 不得再點名 `[P]` markers"
        );
    }
}

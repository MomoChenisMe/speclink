//! The render API: a direct pass-through of the core render matrix
//! (target × invocation × store) — the same code paths `speclink init`/`update`
//! use, so SDK-rendered content and CLI-generated files cannot drift.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use speclink_core::config::{CustomTool, Invocation};
use speclink_core::init::StoreKind;
use speclink_core::skills::{self, RenderTarget, Tool};

/// One entry of `skills.list()`.
#[napi(object)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

/// Options of `skills.render` / `instructions.render` — the render matrix:
/// target (claude | codex | neutral), invocation (cli | tool-call, default
/// cli), store (fs | remote, default fs), plus the spec directory name and
/// the neutral target's tool name ({{TOOL}} substitution, default "speclink").
#[napi(object)]
pub struct RenderOptions {
    pub target: String,
    pub invocation: Option<String>,
    pub store: Option<String>,
    pub spec_dir: Option<String>,
    pub tool_name: Option<String>,
}

/// The resolved matrix point.
struct Matrix {
    target: TargetKind,
    store: StoreKind,
    spec_dir: String,
}

enum TargetKind {
    Builtin(Tool),
    Neutral(CustomTool),
}

fn resolve_matrix(opts: &RenderOptions) -> Result<Matrix> {
    let invocation = match opts.invocation.as_deref() {
        None | Some("cli") => Invocation::Cli,
        Some("tool-call") => Invocation::ToolCall,
        Some(other) => {
            return Err(Error::from_reason(format!(
                "invocation '{other}' must be 'cli' or 'tool-call'"
            )))
        }
    };
    let target = match opts.target.as_str() {
        "neutral" => TargetKind::Neutral(CustomTool {
            name: opts.tool_name.clone().unwrap_or_else(|| "speclink".to_string()),
            // Render never touches these locations; they exist because the
            // descriptor type also drives generation in the CLI.
            skills_dir: ".skills".to_string(),
            instructions_file: "INSTRUCTIONS.md".to_string(),
            invocation,
        }),
        other => match Tool::parse(other) {
            Some(t) => TargetKind::Builtin(t),
            None => {
                return Err(Error::from_reason(format!(
                    "target '{other}' must be 'claude', 'codex', or 'neutral'"
                )))
            }
        },
    };
    let store = match opts.store.as_deref() {
        None | Some("fs") => StoreKind::Fs,
        Some("remote") => StoreKind::Remote,
        Some(other) => {
            return Err(Error::from_reason(format!(
                "store '{other}' must be 'fs' or 'remote'"
            )))
        }
    };
    Ok(Matrix {
        target,
        store,
        spec_dir: opts.spec_dir.clone().unwrap_or_else(|| "openspec".to_string()),
    })
}

/// The skills that generate SKILL.md files, with their descriptions.
#[napi(js_name = "skillsList")]
pub fn skills_list() -> Vec<SkillInfo> {
    skills::registry()
        .into_iter()
        .map(|s| SkillInfo {
            name: s.name.to_string(),
            description: s.description.to_string(),
        })
        .collect()
}

/// Render one skill's SKILL.md content for a matrix point.
#[napi(js_name = "skillsRender")]
pub fn skills_render(name: String, opts: RenderOptions) -> Result<String> {
    let matrix = resolve_matrix(&opts)?;
    let skill = skills::registry()
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| Error::from_reason(format!("Unknown skill: {name}")))?;
    // Skill bodies are store-mode independent (they reach documents through
    // speclink verbs); the store axis only shapes the instructions block.
    let _ = matrix.store;
    let target = match &matrix.target {
        TargetKind::Builtin(tool) => RenderTarget::Builtin(*tool),
        TargetKind::Neutral(custom) => RenderTarget::Custom(custom),
    };
    Ok(skills::render_skill_file_for(target, &skill, &matrix.spec_dir))
}

/// Render the SPECLINK instructions marker block for a matrix point — the
/// exact block `speclink init` writes into CLAUDE.md / AGENTS.md / a custom
/// descriptor's instructions file.
#[napi(js_name = "instructionsRender")]
pub fn instructions_render(opts: RenderOptions) -> Result<String> {
    let matrix = resolve_matrix(&opts)?;
    Ok(match &matrix.target {
        TargetKind::Builtin(tool) => {
            speclink_core::init::instructions_body(&matrix.spec_dir, *tool, matrix.store)
        }
        TargetKind::Neutral(custom) => {
            speclink_core::init::custom_instructions_body(&matrix.spec_dir, custom, matrix.store)
        }
    })
}

//! The render API: a direct pass-through of the core render matrix
//! (target × invocation × store) — the same code paths `speclink init`/`update`
//! use, so SDK-rendered content and CLI-generated files cannot drift.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use speclink_core::config::{CustomTool, Invocation};
use speclink_core::skills::{self, RenderTarget, Tool};

/// One entry of `skills.list()`.
#[napi(object)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

/// Options of `skills.render` — the render matrix: target (claude | codex |
/// neutral) and invocation (cli | tool-call, default cli), plus the spec
/// directory name and the neutral target's tool name ({{TOOL}} substitution,
/// default "speclink").
#[napi(object)]
pub struct RenderOptions {
    pub target: String,
    pub invocation: Option<String>,
    pub spec_dir: Option<String>,
    pub tool_name: Option<String>,
}

/// The resolved matrix point.
struct Matrix {
    target: TargetKind,
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
            // Render never touches this location; it exists because the
            // descriptor type also drives generation in the CLI.
            skills_dir: ".skills".to_string(),
            instructions_file: None,
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
    Ok(Matrix {
        target,
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
    let target = match &matrix.target {
        TargetKind::Builtin(tool) => RenderTarget::Builtin(*tool),
        TargetKind::Neutral(custom) => RenderTarget::Custom(custom),
    };
    Ok(skills::render_skill_file_for(target, &skill, &matrix.spec_dir))
}

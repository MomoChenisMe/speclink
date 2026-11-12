//! `POST /skills` (design D2, D3): the skills this scope's checkouts install,
//! for the tools the client names.
//!
//! The text is the Engine's own render for a checkout without the spec
//! folder. Three things choose the set, in this order: the tool (some skills
//! are for Claude only), the scope's workflow config (the two worktree skills
//! need the policy on), and the host's skill list. Nothing of the caller goes
//! into the text, so every caller of a scope gets the same answer.

use super::error::HttpError;
use super::routes::Request;
use super::HttpResponse;
use crate::documents;
use speclink_core::config::{CustomTool, ToolDescriptor};
use speclink_core::init::{managed_skills, skill_digest, worktree_skills_on, ASSET_VERSION};
use speclink_core::skills::{RenderTarget, SpecLocation, Tool};
use speclink_core::workspace::DEFAULT_SPEC_DIR;
use speclink_protocol::skills::{SkillEntry, SkillTarget, SkillsRequest, SkillsResponse, TargetSkills};

/// The most targets one request may name. A checkout has the five built-in tools
/// and a few descriptors; each target is a render of the whole skill set, so the
/// number is bounded.
const MAX_TARGETS: usize = 32;

/// A requested tool, checked: a built-in, or a custom tool's name and how it
/// calls speclink.
enum Target {
    Builtin(Tool),
    Custom(CustomTool),
}

impl Target {
    fn of(sent: &SkillTarget) -> Result<Target, HttpError> {
        let Some(custom) = &sent.custom else {
            return Tool::parse(&sent.tool).map(Target::Builtin).ok_or_else(|| {
                HttpError::invalid_argument(format!(
                    "unknown tool '{}' (the built-in tools are: {}; a custom tool needs `custom`)",
                    sent.tool,
                    Tool::supported_names()
                ))
            });
        };
        // The descriptor rules of `.speclink.yaml` decide the name and the
        // invocation. A render reads no skills directory: where the files go
        // is the client's setting, so this one is a stand-in.
        let descriptor = ToolDescriptor {
            name: Some(sent.tool.clone()),
            skills_dir: Some("skills".to_string()),
            instructions_file: None,
            invocation: Some(custom.invocation.clone()),
        };
        descriptor.validate().map(Target::Custom).map_err(HttpError::invalid_argument)
    }

    fn render(&self) -> RenderTarget<'_> {
        match self {
            Target::Builtin(tool) => RenderTarget::Builtin(*tool),
            Target::Custom(custom) => RenderTarget::Custom(custom),
        }
    }
}

/// `POST /skills`. It only reads: one document, the scope's workflow config.
pub(crate) fn skills(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: SkillsRequest = req.body()?;
    if body.targets.is_empty() {
        return Err(HttpError::invalid_argument("targets must name at least one tool"));
    }
    if body.targets.len() > MAX_TARGETS {
        return Err(HttpError::invalid_argument(format!("targets must name {MAX_TARGETS} tools at most")));
    }
    let checked = body.targets.iter().map(Target::of).collect::<Result<Vec<_>, _>>()?;

    let config = documents::config_text(req.port(), req.scope())?;
    let worktree_on = worktree_skills_on(config.as_deref());
    let listed = req.kit.options.skills.as_deref();

    let targets = body
        .targets
        .iter()
        .zip(&checked)
        .map(|(sent, target)| {
            let mut skills: Vec<SkillEntry> =
                managed_skills(target.render(), worktree_on, DEFAULT_SPEC_DIR, SpecLocation::Elsewhere)
                    .into_iter()
                    .filter_map(|(dir, content)| {
                        let name = dir.strip_prefix("speclink-")?;
                        if listed.is_some_and(|listed| !listed.iter().any(|l| l == name)) {
                            return None;
                        }
                        Some(SkillEntry {
                            name: name.to_string(),
                            digest: skill_digest(&content),
                            content: body.contents.then_some(content),
                        })
                    })
                    .collect();
            skills.sort_by(|a, b| a.name.cmp(&b.name));
            TargetSkills { tool: sent.tool.clone(), skills }
        })
        .collect();
    Ok(HttpResponse::json(200, &SkillsResponse { version: ASSET_VERSION.to_string(), targets }))
}

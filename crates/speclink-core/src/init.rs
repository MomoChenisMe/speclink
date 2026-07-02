//! Project initialization and instruction-file updates.

use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub const MARKER_VERSION: &str = "v1.0.0";

const APP_CONFIG_TEMPLATE: &str = "# Speclink application config
# See: https://github.com/speclink-app/speclink

# OpenSpec directory path (relative to project root)
# spec_dir: docs/specs

# Language for AI-generated artifacts
# locale: tw

# Language for spec files (default: English; \"auto\" follows locale)
# spec_locale: auto

# Workflow toggles
# tdd: true
# audit: true

# AI tools to generate instruction files for
# tools:
#   - claude
#   - cursor
";

const WORKFLOW_CONFIG_TEMPLATE: &str = "schema: spec-driven

# Project context (optional)
# This is shown to AI when creating artifacts.
# Add your tech stack, conventions, style guides, domain knowledge, etc.
# Example:
#   context: |
#     Tech stack: TypeScript, React, Node.js
#     We use conventional commits
#     Domain: e-commerce platform

# Per-artifact rules (optional)
# Add custom rules for specific artifacts.
# Example:
#   rules:
#     proposal:
#       - Keep proposals under 500 words
#       - Always include a \"Non-goals\" section
#     tasks:
#       - Break tasks into chunks of max 2 hours
";

const GITIGNORE_BLOCK: &str = "# Speclink app data\n.speclink/\n";
const CLAUDE_SETTINGS: &str = "{\n  \"includeGitInstructions\": false\n}";

fn instructions_body(spec_dir: &str, tool: Tool) -> String {
    // GEMINI.md is byte-identical to CLAUDE.md (matches Spectra); only codex differs.
    let (title_prefix, plan_line) = match tool {
        Tool::Codex => ("$speclink-", "- Requirements change mid-work? `ingest` → resume `apply`"),
        _ => ("/speclink-", "- Requirements change mid-work? Plan mode → `ingest` → resume `apply`"),
    };
    format!(
        "<!-- SPECLINK:START {ver} -->\n\n\
# Speclink Instructions\n\n\
This project uses Speclink for Spec-Driven Development(SDD). Specs live in `{sd}/specs/`, change proposals in `{sd}/changes/`.\n\n\
## Use `{p}*` skills when:\n\n\
- A discussion needs structure before coding → `{p}discuss`\n\
- User wants to plan, propose, or design a change → `{p}propose`\n\
- Tasks are ready to implement → `{p}apply`\n\
- There's an in-progress change to continue → `{p}ingest`\n\
- Implementation is done → `{p}archive`\n\
- Commit only files related to a specific change → `{p}commit`\n\n\
## Workflow\n\n\
discuss? → propose → apply ⇄ ingest → archive\n\n\
- `discuss` is optional — skip if requirements are clear\n\
{plan_line}\n\n\
<!-- SPECLINK:END -->\n",
        ver = MARKER_VERSION,
        sd = spec_dir,
        p = title_prefix,
        plan_line = plan_line,
    )
}

/// Insert or replace the SPECLINK:START..END block in an existing document. When the document
/// has no marker yet, the block is PREPENDED above the user's content (matches Spectra).
fn upsert_marker(existing: Option<String>, block: &str) -> String {
    let start = "<!-- SPECLINK:START";
    let end = "<!-- SPECLINK:END -->";
    match existing {
        Some(text) if text.contains(start) => {
            let before = &text[..text.find(start).unwrap()];
            let after_idx = text.find(end).map(|i| i + end.len()).unwrap_or(text.len());
            let after = &text[after_idx..];
            format!("{before}{}{after}", block.trim_end())
        }
        Some(text) if !text.trim().is_empty() => {
            format!("{}\n{}", block.trim_end(), text)
        }
        _ => block.to_string(),
    }
}

pub struct InitOutcome {
    pub spec_dir_abs: PathBuf,
}

/// Initialize speclink in `root`. `tools` are the raw (normalized) `--tools` entries — unknown
/// names are accepted and simply generate nothing, matching Spectra.
pub fn init(root: &Path, tools: &[String], force: bool, spec_dir: &str) -> Result<InitOutcome> {
    let spec_root = root.join(spec_dir);
    if !force && (spec_root.exists() || root.join(".speclink.yaml").is_file()) {
        bail!("Already initialized. Use --force to reinitialize.");
    }

    // openspec structure
    std::fs::create_dir_all(spec_root.join("specs"))?;
    std::fs::create_dir_all(spec_root.join("changes").join("archive"))?;
    write_if(&spec_root.join("config.yaml"), WORKFLOW_CONFIG_TEMPLATE, force)?;

    // .speclink.yaml
    write_if(&root.join(".speclink.yaml"), APP_CONFIG_TEMPLATE, force)?;

    // .gitignore (append block if missing)
    ensure_gitignore(&root.join(".gitignore"))?;

    // Per-tool artifacts (unknown tool names are tolerated no-ops)
    for t in tools {
        if let Some(tool) = Tool::parse(t) {
            generate_tool(root, tool, spec_dir, force)?;
        }
    }

    Ok(InitOutcome {
        spec_dir_abs: spec_root,
    })
}

/// Regenerate instruction files for the tools whose dot-directories exist, returning the tool
/// names refreshed. Detection is directory-based and codex is excluded — both matching Spectra
/// (`update` never regenerates AGENTS.md).
pub fn update(root: &Path) -> Result<Vec<&'static str>> {
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"));
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let candidates = [
        (".claude", Tool::Claude),
        (".cursor", Tool::Cursor),
        (".windsurf", Tool::Windsurf),
        (".gemini", Tool::Gemini),
    ];
    let mut updated = Vec::new();
    for (dir, tool) in candidates {
        if root.join(dir).is_dir() {
            generate_tool(root, tool, &spec_dir, true)?;
            updated.push(tool.name());
        }
    }
    Ok(updated)
}

fn generate_tool(root: &Path, tool: Tool, spec_dir: &str, force: bool) -> Result<()> {
    // Root instruction file (marker upsert) + tool-specific extras.
    match tool {
        Tool::Claude => {
            write_if(&root.join(".claude").join("settings.json"), CLAUDE_SETTINGS, force)?;
            let md = root.join("CLAUDE.md");
            let merged = upsert_marker(util::read_opt(&md), &instructions_body(spec_dir, tool));
            util::write_file(&md, &merged)?;
        }
        Tool::Codex => {
            let md = root.join("AGENTS.md");
            let merged = upsert_marker(util::read_opt(&md), &instructions_body(spec_dir, tool));
            util::write_file(&md, &merged)?;
        }
        Tool::Gemini => {
            let md = root.join("GEMINI.md");
            let merged = upsert_marker(util::read_opt(&md), &instructions_body(spec_dir, tool));
            util::write_file(&md, &merged)?;
        }
        Tool::Cursor => {
            let rules = root.join(".cursorrules");
            let merged = upsert_marker(util::read_opt(&rules), &skills::render_rules_file(spec_dir));
            util::write_file(&rules, &merged)?;
        }
        Tool::Windsurf => {
            let rules = root.join(".windsurfrules");
            let merged = upsert_marker(util::read_opt(&rules), &skills::render_rules_file(spec_dir));
            util::write_file(&rules, &merged)?;
        }
    }
    // Skills: Claude gets the full registry; every other tool gets the command subset.
    for skill in skills::registry() {
        if tool != Tool::Claude && !skill.for_codex {
            continue;
        }
        let content = skills::render_skill_file(&skill, tool, spec_dir);
        let path = root
            .join(tool.skills_dir())
            .join(format!("speclink-{}", skill.name))
            .join("SKILL.md");
        write_if(&path, &content, force)?;
    }
    // Command files (cursor/gemini/windsurf).
    for skill in skills::registry() {
        if !skill.for_codex {
            continue;
        }
        match tool {
            Tool::Cursor => {
                let path = root
                    .join(".cursor")
                    .join("commands")
                    .join(format!("speclink-{}.md", skill.name));
                write_if(&path, &skills::render_cursor_command(&skill, spec_dir), force)?;
            }
            Tool::Gemini => {
                let path = root
                    .join(".gemini")
                    .join("commands")
                    .join("speclink")
                    .join(format!("{}.toml", skill.name));
                write_if(&path, &skills::render_gemini_toml(&skill, spec_dir), force)?;
            }
            Tool::Windsurf => {
                let path = root
                    .join(".windsurf")
                    .join("workflows")
                    .join(format!("speclink-{}.md", skill.name));
                write_if(&path, &skills::render_windsurf_workflow(&skill, spec_dir), force)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn write_if(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }
    util::write_file(path, content)?;
    Ok(())
}

fn ensure_gitignore(path: &Path) -> Result<()> {
    match util::read_opt(path) {
        Some(text) if text.contains(".speclink/") => Ok(()),
        Some(text) => {
            let mut new = text;
            if !new.ends_with('\n') {
                new.push('\n');
            }
            new.push('\n');
            new.push_str(GITIGNORE_BLOCK);
            util::write_file(path, &new)?;
            Ok(())
        }
        None => {
            util::write_file(path, GITIGNORE_BLOCK)?;
            Ok(())
        }
    }
}

/// Split a comma-separated `--tools` value into normalized names. Unknown names are kept
/// (echoed in the "Generated files for:" line) but generate nothing, matching Spectra.
pub fn parse_tools(spec: &str) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for part in spec.split(',') {
        let part = part.trim().to_ascii_lowercase();
        if !part.is_empty() && !out.contains(&part) {
            out.push(part);
        }
    }
    Ok(out)
}

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
#   - codex
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
    let (title_prefix, plan_line) = match tool {
        Tool::Claude => ("/speclink-", "- Requirements change mid-work? Plan mode → `ingest` → resume `apply`"),
        Tool::Codex => ("$speclink-", "- Requirements change mid-work? `ingest` → resume `apply`"),
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

/// Insert or replace the SPECLINK:START..END block in an existing document.
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
            format!("{}\n\n{block}", text.trim_end())
        }
        _ => block.to_string(),
    }
}

pub struct InitOutcome {
    pub spec_dir_abs: PathBuf,
}

/// Initialize speclink in `root`.
pub fn init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str) -> Result<InitOutcome> {
    let spec_root = root.join(spec_dir);

    // openspec structure
    std::fs::create_dir_all(spec_root.join("specs"))?;
    std::fs::create_dir_all(spec_root.join("changes").join("archive"))?;
    write_if(&spec_root.join("config.yaml"), WORKFLOW_CONFIG_TEMPLATE, force)?;

    // .speclink.yaml
    write_if(&root.join(".speclink.yaml"), APP_CONFIG_TEMPLATE, force)?;

    // .gitignore (append block if missing)
    ensure_gitignore(&root.join(".gitignore"))?;

    // Per-tool artifacts
    for tool in tools {
        generate_tool(root, *tool, spec_dir, force)?;
    }

    Ok(InitOutcome {
        spec_dir_abs: spec_root,
    })
}

/// Regenerate instruction files and skills (refresh markers).
pub fn update(root: &Path, force: bool) -> Result<()> {
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"));
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    // Detect existing tool dirs.
    let mut tools = Vec::new();
    if root.join(".claude").exists() || root.join("CLAUDE.md").exists() {
        tools.push(Tool::Claude);
    }
    if root.join(".agents").exists() || root.join("AGENTS.md").exists() {
        tools.push(Tool::Codex);
    }
    if tools.is_empty() {
        tools.push(Tool::Claude);
    }
    for tool in tools {
        generate_tool(root, tool, &spec_dir, force || true)?;
    }
    Ok(())
}

fn generate_tool(root: &Path, tool: Tool, spec_dir: &str, force: bool) -> Result<()> {
    match tool {
        Tool::Claude => {
            // settings.json
            write_if(&root.join(".claude").join("settings.json"), CLAUDE_SETTINGS, force)?;
            // CLAUDE.md (upsert marker)
            let claude_md = root.join("CLAUDE.md");
            let block = instructions_body(spec_dir, tool);
            let merged = upsert_marker(util::read_opt(&claude_md), &block);
            util::write_file(&claude_md, &merged)?;
        }
        Tool::Codex => {
            let agents_md = root.join("AGENTS.md");
            let block = instructions_body(spec_dir, tool);
            let merged = upsert_marker(util::read_opt(&agents_md), &block);
            util::write_file(&agents_md, &merged)?;
        }
    }
    // Skills
    for skill in skills::registry() {
        if tool == Tool::Codex && !skill.for_codex {
            continue;
        }
        let content = skills::render_skill_file(&skill, tool, spec_dir);
        let path = root
            .join(tool.skills_dir())
            .join(format!("speclink-{}", skill.name))
            .join("SKILL.md");
        write_if(&path, &content, force)?;
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

/// Validate a comma-separated `--tools` value into a tool list.
pub fn parse_tools(spec: &str) -> Result<Vec<Tool>> {
    let mut out = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match Tool::parse(part) {
            Some(t) => {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
            None => bail!("unknown tool: {part} (supported: claude, codex)"),
        }
    }
    Ok(out)
}

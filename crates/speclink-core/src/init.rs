//! Project initialization and instruction-file updates.

use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub const MARKER_VERSION: &str = "v1.1.0";

const APP_CONFIG_TEMPLATE: &str = "# Speclink application config
# See: https://github.com/speclink-app/speclink

# OpenSpec directory path (relative to project root)
# spec_dir: docs/specs

# Language for AI-generated artifacts (default: English)
# locale: tw

# Language for spec files (default: English; \"auto\" follows locale)
# spec_locale: auto

# Workflow toggles (default: off)
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
    // Codex differs: `$speclink-` prefix, no plan mode, and no verify skill (for_codex=false).
    let (p, plan_line) = match tool {
        Tool::Codex => ("$speclink-", "- Requirements change mid-work? `ingest` → resume `apply`"),
        _ => ("/speclink-", "- Requirements change mid-work? Plan mode → `ingest` → resume `apply`"),
    };
    let (done_line, workflow) = match tool {
        Tool::Codex => (
            format!("- Implementation is done → `{p}archive`"),
            "discuss? → propose → apply ⇄ ingest → archive",
        ),
        _ => (
            format!("- Implementation is done → `{p}verify`, then `{p}archive`"),
            "discuss? → propose → apply ⇄ ingest → verify? → archive",
        ),
    };
    format!(
        "<!-- SPECLINK:START {ver} -->\n\n\
# Speclink Instructions\n\n\
This project uses Speclink for Spec-Driven Development(SDD). Specs live in `{sd}/specs/`, change proposals in `{sd}/changes/`, discussion records in `{sd}/discussions/`.\n\n\
## Use `{p}*` skills when:\n\n\
- Requirements are fuzzy or worth debating → `{p}discuss` (recorded as a document; promote turns it into a change)\n\
- User wants to plan, propose, or design a change → `{p}propose` (`--from-discussion <slug>` seeds it from a concluded discussion)\n\
- Adopting Speclink on an existing codebase → `{p}onboard`\n\
- Tasks are ready to implement → `{p}apply`\n\
- Resuming a change that sat idle → run `{p}drift` first\n\
- Requirements change mid-work → `{p}ingest`\n\
{done_line}\n\
- Commit only files related to a specific change → `{p}commit`\n\n\
## Workflow\n\n\
{workflow}\n\n\
- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is \"don't do it\"\n\
- A promoted discussion is archived automatically with its change\n\
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`\n\
{plan_line}\n\n\
<!-- SPECLINK:END -->\n",
        ver = MARKER_VERSION,
        sd = spec_dir,
        p = p,
        done_line = done_line,
        workflow = workflow,
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

/// Initialize speclink in `root`.
pub fn init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str) -> Result<InitOutcome> {
    let spec_root = root.join(spec_dir);
    if !force && (spec_root.exists() || root.join(".speclink.yaml").is_file()) {
        bail!("Already initialized. Use --force to reinitialize.");
    }

    // openspec structure
    std::fs::create_dir_all(spec_root.join("specs"))?;
    std::fs::create_dir_all(spec_root.join("changes").join("archive"))?;
    write_if(&spec_root.join("config.yaml"), WORKFLOW_CONFIG_TEMPLATE, force)?;

    // .speclink.yaml — the template plus the actual tool selection, so `update` can sync
    // (regenerate + prune) against the recorded list later.
    let config_content = if tools.is_empty() {
        APP_CONFIG_TEMPLATE.to_string()
    } else {
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        format!("{APP_CONFIG_TEMPLATE}tools: [{}]\n", names.join(", "))
    };
    write_if(&root.join(".speclink.yaml"), &config_content, force)?;

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

pub struct UpdateOutcome {
    pub updated: Vec<&'static str>,
    pub pruned: Vec<&'static str>,
    pub notes: Vec<String>,
}

/// Refresh generated instruction files.
///
/// When `.speclink.yaml` records a `tools:` list, this is a full sync: every listed tool is
/// regenerated and generated files for tools NOT on the list are pruned (speclink-* skill
/// dirs removed, the SPECLINK marker block stripped from the instruction file). Unknown tool
/// names in the list produce a warning note. Without a recorded list, this falls back to
/// Spectra's behavior: regenerate the tools whose dot-directories exist (codex excluded).
pub fn update(root: &Path) -> Result<UpdateOutcome> {
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"));
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let mut out = UpdateOutcome { updated: Vec::new(), pruned: Vec::new(), notes: Vec::new() };

    if app.tools.is_empty() {
        // Legacy fallback (matches Spectra): directory detection, codex excluded, no prune.
        if root.join(".claude").is_dir() {
            generate_tool(root, Tool::Claude, &spec_dir, true)?;
            out.updated.push(Tool::Claude.name());
        }
        return Ok(out);
    }

    let mut selected = Vec::new();
    for name in &app.tools {
        match Tool::parse(name) {
            Some(t) => {
                if !selected.contains(&t) {
                    selected.push(t);
                }
            }
            None => out.notes.push(format!(
                "unknown tool '{name}' in .speclink.yaml tools list (supported: claude, codex)"
            )),
        }
    }
    for tool in [Tool::Claude, Tool::Codex] {
        if selected.contains(&tool) {
            generate_tool(root, tool, &spec_dir, true)?;
            out.updated.push(tool.name());
        } else if prune_tool(root, tool)? {
            out.pruned.push(tool.name());
        }
    }
    Ok(out)
}

/// Remove the generated artifacts of a deselected tool: speclink-* skill directories and the
/// SPECLINK marker block in its instruction file (user content outside the block survives; a
/// file left empty is deleted). Returns whether anything was removed.
fn prune_tool(root: &Path, tool: Tool) -> Result<bool> {
    let mut removed = false;
    let skills_root = root.join(tool.skills_dir());
    if let Ok(entries) = std::fs::read_dir(&skills_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("speclink-") && entry.path().is_dir() {
                std::fs::remove_dir_all(entry.path())?;
                removed = true;
            }
        }
    }
    let md = root.join(match tool {
        Tool::Claude => "CLAUDE.md",
        Tool::Codex => "AGENTS.md",
    });
    if let Some(text) = util::read_opt(&md) {
        if text.contains("<!-- SPECLINK:START") {
            let stripped = strip_marker(&text);
            if stripped.trim().is_empty() {
                std::fs::remove_file(&md)?;
            } else {
                util::write_file(&md, &stripped)?;
            }
            removed = true;
        }
    }
    Ok(removed)
}

/// Remove the SPECLINK:START..END block (plus the blank line it was separated by).
fn strip_marker(text: &str) -> String {
    let start = "<!-- SPECLINK:START";
    let end = "<!-- SPECLINK:END -->";
    let Some(s) = text.find(start) else {
        return text.to_string();
    };
    let e = text.find(end).map(|i| i + end.len()).unwrap_or(text.len());
    let before = &text[..s];
    let after = text[e..].trim_start_matches('\n');
    format!("{before}{after}")
}

/// Detect installed AI tools by their footprints (deliberate difference from Spectra, which
/// generates nothing when --tools is omitted). Defaults to claude when nothing is found.
pub fn detect_tools(root: &Path) -> Vec<Tool> {
    let mut out = Vec::new();
    if root.join(".claude").is_dir() {
        out.push(Tool::Claude);
    }
    if root.join(".agents").is_dir() || root.join("AGENTS.md").is_file() {
        out.push(Tool::Codex);
    }
    if out.is_empty() {
        out.push(Tool::Claude);
    }
    out
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
    }
    // Skills: Claude gets the full registry; codex gets the command subset.
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

/// Validate a comma-separated `--tools` value into a tool list. Speclink deliberately scopes
/// the supported tools to claude + codex.
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

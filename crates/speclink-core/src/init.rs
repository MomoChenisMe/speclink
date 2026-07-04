//! Project initialization and instruction-file updates.

use crate::config::{CustomTool, ToolEntry};
use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

pub const MARKER_VERSION: &str = "v1.2.0";

const APP_CONFIG_TEMPLATE: &str = "# Speclink application config
# See: https://github.com/speclink-app/speclink

# OpenSpec directory path (relative to project root)
# spec_dir: docs/specs

# AI tools to generate instruction files for
# tools:
#   - claude
#   - codex
";

const WORKFLOW_CONFIG_TEMPLATE: &str = "schema: spec-driven

# Workflow policy (optional)
# Personal/CI overrides: SPECLINK_LOCALE, SPECLINK_SPEC_LOCALE, SPECLINK_TDD, SPECLINK_AUDIT
#
# Language for AI-generated artifacts (default: English)
# locale: tw
#
# Language for spec files (default: English; \"auto\" follows locale)
# spec_locale: auto
#
# Workflow toggles (default: off)
# tdd: true
# audit: true

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
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)\n\
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

/// Instructions body for a custom tool's marker block — the neutral wording: skills are
/// referenced by their generated names (`speclink-<verb>`, no slash prefix), there is no
/// plan-mode line, and an invocation sentence states how verbs are executed.
fn custom_instructions_body(spec_dir: &str, tool: &CustomTool) -> String {
    let invocation_line = match tool.invocation {
        crate::config::Invocation::Cli => {
            "Speclink verbs are shell commands: run `speclink <verb> [arguments]`."
        }
        crate::config::Invocation::ToolCall => {
            "Speclink verbs are executed by calling the speclink tool with an argv array \
(e.g. [\"apply\", \"add-auth\"])."
        }
    };
    format!(
        "<!-- SPECLINK:START {ver} -->\n\n\
# Speclink Instructions\n\n\
This project uses Speclink for Spec-Driven Development(SDD). Specs live in `{sd}/specs/`, change proposals in `{sd}/changes/`, discussion records in `{sd}/discussions/`.\n\n\
{invocation_line}\n\n\
## Use the `speclink-*` skills when:\n\n\
- Requirements are fuzzy or worth debating → `speclink-discuss` (recorded as a document; promote turns it into a change)\n\
- User wants to plan, propose, or design a change → `speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)\n\
- Adopting Speclink on an existing codebase → `speclink-onboard`\n\
- Tasks are ready to implement → `speclink-apply`\n\
- Resuming a change that sat idle → run `speclink-drift` first\n\
- Requirements change mid-work → `speclink-ingest`\n\
- Implementation is done → `speclink-archive`\n\
- Commit only files related to a specific change → `speclink-commit`\n\n\
## Workflow\n\n\
discuss? → propose → apply ⇄ ingest → archive\n\n\
- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is \"don't do it\"\n\
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)\n\
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`\n\
- Requirements change mid-work? `ingest` → resume `apply`\n\n\
<!-- SPECLINK:END -->\n",
        ver = MARKER_VERSION,
        sd = spec_dir,
        invocation_line = invocation_line,
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

/// Initialize speclink in `root` — store init (spec-document tree) followed by
/// workspace init (host-side files). Externally one command; the split is the seam a
/// remote storage backend plugs into (workspace init always runs locally, store init
/// only for filesystem-backed storage).
pub fn init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str) -> Result<InitOutcome> {
    let spec_root = root.join(spec_dir);
    if !force && (spec_root.exists() || root.join(".speclink.yaml").is_file()) {
        bail!("Already initialized. Use --force to reinitialize.");
    }

    store_init(&spec_root, force)?;
    workspace_init(root, tools, force, spec_dir)?;

    Ok(InitOutcome {
        spec_dir_abs: spec_root,
    })
}

/// Store init: the spec-document tree (`openspec/` skeleton) and the workflow-config
/// template — the canonical home of the policy fields.
fn store_init(spec_root: &Path, force: bool) -> Result<()> {
    std::fs::create_dir_all(spec_root.join("specs"))?;
    std::fs::create_dir_all(spec_root.join("changes").join("archive"))?;
    write_if(&spec_root.join("config.yaml"), WORKFLOW_CONFIG_TEMPLATE, force)
}

/// Workspace init: host-side files that stay local no matter where spec documents live —
/// `.speclink.yaml`, instruction-file markers, skills, settings, `.gitignore`.
fn workspace_init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str) -> Result<()> {
    // .speclink.yaml — the template plus the actual tool selection, so `update` can sync
    // (regenerate + prune) against the recorded list later. A non-default --dir is
    // persisted as an active spec_dir line (matches Spectra) so later commands find it.
    let mut config_content = if tools.is_empty() {
        APP_CONFIG_TEMPLATE.to_string()
    } else {
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        format!("{APP_CONFIG_TEMPLATE}tools: [{}]\n", names.join(", "))
    };
    if spec_dir != "openspec" {
        config_content = config_content.replace(
            "# spec_dir: docs/specs",
            &format!("spec_dir: {spec_dir}"),
        );
    }
    write_if(&root.join(".speclink.yaml"), &config_content, force)?;

    // .gitignore (append block if missing)
    ensure_gitignore(&root.join(".gitignore"))?;

    // Per-tool artifacts
    for tool in tools {
        generate_tool(root, *tool, spec_dir, force)?;
    }
    Ok(())
}

pub struct UpdateOutcome {
    pub updated: Vec<String>,
    pub pruned: Vec<String>,
    pub notes: Vec<String>,
}

/// Refresh generated instruction files.
///
/// When `.speclink.yaml` records a `tools:` list, this is a full sync: every listed tool
/// (built-in name or custom descriptor) is regenerated and generated files for tools NOT on
/// the list are pruned (speclink-* skill dirs removed, the SPECLINK marker block stripped
/// from the instruction file). Unknown built-in names produce a warning note; an invalid
/// descriptor is an error. Without a recorded list, built-ins fall back to Spectra's
/// behavior: regenerate the tools whose dot-directories exist (codex excluded).
pub fn update(root: &Path) -> Result<UpdateOutcome> {
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"));
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let mut out = UpdateOutcome { updated: Vec::new(), pruned: Vec::new(), notes: Vec::new() };

    // Sort entries: built-in name strings vs custom descriptors. Descriptors validate
    // up front — nothing is generated or pruned when any descriptor is invalid.
    let mut selected = Vec::new();
    let mut customs: Vec<CustomTool> = Vec::new();
    for entry in &app.tools {
        match entry {
            ToolEntry::Builtin(name) => match Tool::parse(name) {
                Some(t) => {
                    if !selected.contains(&t) {
                        selected.push(t);
                    }
                }
                None => out.notes.push(format!(
                    "unknown tool '{name}' in .speclink.yaml tools list (supported: claude, codex)"
                )),
            },
            ToolEntry::Descriptor(d) => {
                let custom = d.validate().map_err(|e| anyhow::anyhow!(e))?;
                if customs.iter().any(|c| c.name == custom.name) {
                    bail!("tool descriptor: duplicate name '{}'", custom.name);
                }
                customs.push(custom);
            }
        }
    }

    if app.tools.is_empty() {
        // Legacy fallback (matches Spectra): directory detection, codex excluded, no
        // built-in prune. Custom footprints recorded by an earlier update are still
        // synced below — an emptied tools list must not strand them.
        if root.join(".claude").is_dir() {
            generate_tool(root, Tool::Claude, &spec_dir, true)?;
            out.updated.push(Tool::Claude.name().to_string());
        }
    } else {
        for tool in [Tool::Claude, Tool::Codex] {
            if selected.contains(&tool) {
                generate_tool(root, tool, &spec_dir, true)?;
                out.updated.push(tool.name().to_string());
            } else if prune_tool(root, tool)? {
                out.pruned.push(tool.name().to_string());
            }
        }
    }

    // Custom descriptors share the built-in lifecycle: prune the footprints that fell off
    // the list first (a descriptor may have moved its paths), then (re)generate the
    // current ones and record them for the next sync.
    let previous = load_custom_state(root);
    for old in &previous {
        let still_current = customs.iter().any(|c| {
            c.name == old.name
                && c.skills_dir == old.skills_dir
                && c.instructions_file == old.instructions_file
        });
        if !still_current && prune_custom(root, old, &mut out.notes)? {
            out.pruned.push(old.name.clone());
        }
    }
    for custom in &customs {
        generate_custom(root, custom, &spec_dir)?;
        out.updated.push(custom.name.clone());
    }
    save_custom_state(root, &customs)?;

    Ok(out)
}

/// Recorded footprint of a generated custom tool — what a later update needs in order to
/// clean up after the descriptor disappears from `.speclink.yaml`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CustomFootprint {
    name: String,
    skills_dir: String,
    instructions_file: String,
}

/// Host-side record of generated custom-tool footprints (`.speclink/` is gitignored work
/// data). Missing or unreadable state simply means "nothing to prune".
fn custom_state_path(root: &Path) -> PathBuf {
    root.join(".speclink").join("generated-tools.yaml")
}

fn load_custom_state(root: &Path) -> Vec<CustomFootprint> {
    let Some(text) = util::read_opt(&custom_state_path(root)) else {
        return Vec::new();
    };
    serde_yaml::from_str(&text).unwrap_or_default()
}

fn save_custom_state(root: &Path, customs: &[CustomTool]) -> Result<()> {
    let footprints: Vec<CustomFootprint> = customs
        .iter()
        .map(|c| CustomFootprint {
            name: c.name.clone(),
            skills_dir: c.skills_dir.clone(),
            instructions_file: c.instructions_file.clone(),
        })
        .collect();
    if footprints.is_empty() {
        // No footprints → no state file (and drop .speclink/ if that leaves it empty).
        let path = custom_state_path(root);
        if path.exists() {
            std::fs::remove_file(&path)?;
            if let Some(parent) = path.parent() {
                let _ = std::fs::remove_dir(parent);
            }
        }
        return Ok(());
    }
    util::write_file(&custom_state_path(root), &serde_yaml::to_string(&footprints)?)?;
    Ok(())
}

/// Generate a custom tool's artifacts: skills under `<skills_dir>/speclink-*/SKILL.md`
/// (the non-Claude command subset) and the SPECLINK marker block in its instructions file.
fn generate_custom(root: &Path, tool: &CustomTool, spec_dir: &str) -> Result<()> {
    let md = root.join(&tool.instructions_file);
    let merged = upsert_marker(util::read_opt(&md), &custom_instructions_body(spec_dir, tool));
    util::write_file(&md, &merged)?;
    for skill in skills::registry() {
        if !skill.for_codex {
            continue;
        }
        let content = skills::render_skill_file_for(skills::RenderTarget::Custom(tool), &skill, spec_dir);
        let path = root
            .join(&tool.skills_dir)
            .join(format!("speclink-{}", skill.name))
            .join("SKILL.md");
        util::write_file(&path, &content)?;
    }
    Ok(())
}

/// Prune a recorded custom footprint. Paths are re-checked against the project root before
/// any removal — a tampered state file must not be able to delete outside the project.
fn prune_custom(root: &Path, fp: &CustomFootprint, notes: &mut Vec<String>) -> Result<bool> {
    if !crate::config::is_project_relative(&fp.skills_dir)
        || !crate::config::is_project_relative(&fp.instructions_file)
    {
        notes.push(format!(
            "skipped pruning tool '{}': recorded paths escape the project root",
            fp.name
        ));
        return Ok(false);
    }
    prune_footprint(&root.join(&fp.skills_dir), &root.join(&fp.instructions_file))
}

/// Remove the generated artifacts of a deselected built-in tool.
fn prune_tool(root: &Path, tool: Tool) -> Result<bool> {
    let md = root.join(match tool {
        Tool::Claude => "CLAUDE.md",
        Tool::Codex => "AGENTS.md",
    });
    prune_footprint(&root.join(tool.skills_dir()), &md)
}

/// Remove a generated footprint: speclink-* skill directories and the SPECLINK marker
/// block in the instruction file (user content outside the block survives; a file left
/// empty is deleted). Returns whether anything was removed.
fn prune_footprint(skills_root: &Path, md: &Path) -> Result<bool> {
    let mut removed = false;
    if let Ok(entries) = std::fs::read_dir(skills_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("speclink-") && entry.path().is_dir() {
                std::fs::remove_dir_all(entry.path())?;
                removed = true;
            }
        }
    }
    // A deselected tool should leave no footprint: drop the skills dir and its parent
    // when (and only when) they are now empty — user files keep them alive.
    let _ = std::fs::remove_dir(skills_root);
    if let Some(parent) = skills_root.parent() {
        let _ = std::fs::remove_dir(parent);
    }
    if let Some(text) = util::read_opt(md) {
        if text.contains("<!-- SPECLINK:START") {
            let stripped = strip_marker(&text);
            if stripped.trim().is_empty() {
                std::fs::remove_file(md)?;
            } else {
                util::write_file(md, &stripped)?;
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
        let content = skills::render_skill_file_for(skills::RenderTarget::Builtin(tool), &skill, spec_dir);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Throwaway project root, removed on drop.
    struct TempRoot {
        dir: PathBuf,
    }

    impl TempRoot {
        fn new(tag: &str) -> TempRoot {
            let dir = std::env::temp_dir().join(format!(
                "speclink-init-test-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempRoot { dir }
        }

        fn read(&self, rel: &str) -> String {
            std::fs::read_to_string(self.dir.join(rel.split('/').collect::<PathBuf>())).unwrap()
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    // Spec requirement: init 範本的政策寫入位置 — policy examples live in the
    // openspec/config.yaml template; the .speclink.yaml template carries none.

    #[test]
    fn init_workflow_config_template_has_commented_policy_examples() {
        let root = TempRoot::new("wf-template");
        init(&root.dir, &[], false, "openspec").unwrap();
        let wf = root.read("openspec/config.yaml");
        for example in ["# locale:", "# spec_locale:", "# tdd:", "# audit:"] {
            assert!(
                wf.contains(example),
                "config.yaml template must show a commented {example} example:\n{wf}"
            );
        }
    }

    #[test]
    fn init_app_config_template_has_no_policy_keys() {
        let root = TempRoot::new("app-template");
        init(&root.dir, &[], false, "openspec").unwrap();
        let app = root.read(".speclink.yaml");
        // Not even commented examples: the workspace file must not teach policy keys.
        // ("locale" also catches "spec_locale".)
        for word in ["locale", "tdd", "audit"] {
            assert!(
                !app.contains(word),
                ".speclink.yaml template must not mention policy key {word}:\n{app}"
            );
        }
        // Its actual concerns stay: workspace binding (tools) and spec_dir.
        assert!(app.contains("tools"));
        assert!(app.contains("spec_dir"));
    }

    #[test]
    fn init_with_tools_records_selection_without_policy_keys() {
        let root = TempRoot::new("app-tools");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        let app = root.read(".speclink.yaml");
        assert!(app.contains("tools: [claude, codex]"));
        for word in ["locale", "tdd", "audit"] {
            assert!(!app.contains(word), "no policy key {word} expected:\n{app}");
        }
    }
}

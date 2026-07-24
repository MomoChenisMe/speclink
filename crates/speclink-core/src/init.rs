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

/// Where the spec documents live — the second axis of the marker rendering
/// matrix (tool target) × (fs | remote).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreKind {
    Fs,
    Remote,
}

/// The marker's opening paragraph: fs mode names the local paths; remote mode
/// must not (they don't exist) — documents are reached through speclink verbs.
fn store_paragraph(spec_dir: &str, store: StoreKind) -> String {
    match store {
        StoreKind::Fs => format!(
            "This project uses Speclink for Spec-Driven Development(SDD). Specs live in `{spec_dir}/specs/`, change proposals in `{spec_dir}/changes/`, discussion records in `{spec_dir}/discussions/`."
        ),
        StoreKind::Remote => "This project uses Speclink for Spec-Driven Development(SDD). Specs, change proposals, and discussion records live in the team system's spec store — always access them through `speclink` verbs; never read or write spec documents as local files.".to_string(),
    }
}

/// The SPECLINK marker block for a built-in tool — pub so the SDK's
/// `instructions.render` shares this exact generation path with `init`/`update`.
pub fn instructions_body(spec_dir: &str, tool: Tool, store: StoreKind) -> String {
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
{store_paragraph}\n\n\
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
        store_paragraph = store_paragraph(spec_dir, store),
        p = p,
        done_line = done_line,
        workflow = workflow,
        plan_line = plan_line,
    )
}

/// Instructions body for a custom tool's marker block — the neutral wording: skills are
/// referenced by their generated names (`speclink-<verb>`, no slash prefix), there is no
/// plan-mode line, and an invocation sentence states how verbs are executed.
pub fn custom_instructions_body(spec_dir: &str, tool: &CustomTool, store: StoreKind) -> String {
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
{store_paragraph}\n\n\
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
        store_paragraph = store_paragraph(spec_dir, store),
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
    workspace_init(root, tools, force, spec_dir, StoreKind::Fs)?;

    Ok(InitOutcome {
        spec_dir_abs: spec_root,
    })
}

/// Remote-store initialization: workspace init plus the `remote:` section in
/// `.speclink.yaml` — deliberately NO store init (the spec-document tree lives
/// on the server, so no `openspec/` skeleton and no local workflow-config
/// template).
pub fn init_remote(
    root: &Path,
    tools: &[Tool],
    force: bool,
    url: &str,
    repo: Option<&str>,
) -> Result<()> {
    if !force && root.join(".speclink.yaml").is_file() {
        bail!("Already initialized. Use --force to reinitialize.");
    }
    workspace_init(root, tools, force, "openspec", StoreKind::Remote)?;
    write_remote_section(root, url, repo)
}

/// Write or replace the `remote:` section of `.speclink.yaml` via
/// read–modify–write: other fields keep their values (comments do not survive
/// re-serialization — a documented limitation). A missing file is created.
pub fn write_remote_section(root: &Path, url: &str, repo: Option<&str>) -> Result<()> {
    let path = root.join(".speclink.yaml");
    let mut doc = load_app_yaml_doc(&path)?;
    let mut section = serde_yaml::Mapping::new();
    section.insert("url".into(), url.into());
    if let Some(r) = repo {
        section.insert("repo".into(), r.into());
    }
    doc.insert("remote".into(), serde_yaml::Value::Mapping(section));
    util::write_file(&path, &serde_yaml::to_string(&doc)?)?;
    Ok(())
}

/// Remove the `remote:` section of `.speclink.yaml`, keeping every other
/// field. `Ok(true)` when a section was removed; `Ok(false)` when there was
/// nothing to remove (missing file included).
pub fn remove_remote_section(root: &Path) -> Result<bool> {
    let path = root.join(".speclink.yaml");
    if !path.is_file() {
        return Ok(false);
    }
    let mut doc = load_app_yaml_doc(&path)?;
    if doc.remove("remote").is_none() {
        return Ok(false);
    }
    util::write_file(&path, &serde_yaml::to_string(&doc)?)?;
    Ok(true)
}

/// Load `.speclink.yaml` as a raw mapping for read–modify–write. Unlike
/// `AppConfig::load` (read-only, defaults on error), a malformed file here is
/// a loud error — rewriting it would silently destroy the user's content.
fn load_app_yaml_doc(path: &Path) -> Result<serde_yaml::Mapping> {
    let Some(text) = util::read_opt(path) else {
        return Ok(serde_yaml::Mapping::new());
    };
    let value: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|e| anyhow::anyhow!("invalid .speclink.yaml: {e}"))?;
    match value {
        serde_yaml::Value::Mapping(m) => Ok(m),
        serde_yaml::Value::Null => Ok(serde_yaml::Mapping::new()),
        _ => bail!("invalid .speclink.yaml: expected a mapping at the top level"),
    }
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
fn workspace_init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str, store: StoreKind) -> Result<()> {
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
        generate_tool(root, *tool, spec_dir, force, store)?;
    }
    Ok(())
}

#[derive(Debug)]
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
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"))?;
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    // The remote section's presence is the mode signal — regenerated markers
    // keep the wording of the mode the workspace is actually in.
    let store = if app.remote.is_some() {
        StoreKind::Remote
    } else {
        StoreKind::Fs
    };
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
            generate_tool(root, Tool::Claude, &spec_dir, true, store)?;
            out.updated.push(Tool::Claude.name().to_string());
        }
    } else {
        for tool in [Tool::Claude, Tool::Codex] {
            if selected.contains(&tool) {
                generate_tool(root, tool, &spec_dir, true, store)?;
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
        generate_custom(root, custom, &spec_dir, store)?;
        out.updated.push(custom.name.clone());
    }
    save_custom_state(root, &customs)?;

    Ok(out)
}

/// Converge a workspace on `tools` as the COMPLETE desired state of its built-ins —
/// the single entry point CLI init, remote init and the desktop share.
///
/// Two steps, both existing behavior: `.speclink.yaml`'s claude/codex entries are
/// rewritten to match the selection (custom descriptors, remote, spec_dir and unknown
/// keys carry over untouched), then [`update`] generates the selected tools' skills and
/// marker blocks and prunes the deselected ones. Marker wording follows the store mode
/// recorded in the config, so a remote checkout is never given a local spec tree.
///
/// Empty selections and malformed configs fail before anything is written. Beyond that
/// there is no rollback: every managed write is idempotent, so the same selection can be
/// submitted again to converge (design: "失敗不開啟 Workspace並以可重試收斂取代跨檔回滾").
pub fn reconcile_builtin_tools(root: &Path, tools: &[Tool]) -> Result<UpdateOutcome> {
    if tools.is_empty() {
        bail!("no tools selected (supported: claude, codex)");
    }
    let path = root.join(".speclink.yaml");
    let original = util::read_opt(&path).unwrap_or_default();
    let rewritten = crate::config::update_app_config_tools_text(&original, tools)?;
    util::write_file(&path, &rewritten)?;
    update(root)
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
fn generate_custom(root: &Path, tool: &CustomTool, spec_dir: &str, store: StoreKind) -> Result<()> {
    let md = root.join(&tool.instructions_file);
    let merged = upsert_marker(util::read_opt(&md), &custom_instructions_body(spec_dir, tool, store));
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

/// Detect installed AI tools by their footprints, WITHOUT any fallback — an empty result
/// means "no footprint found", which desktop checkout preselection needs (it must not
/// invent a Claude default). [`detect_tools`] layers the claude fallback on top.
pub fn detect_footprint_tools(root: &Path) -> Vec<Tool> {
    let mut out = Vec::new();
    if root.join(".claude").is_dir() {
        out.push(Tool::Claude);
    }
    if root.join(".agents").is_dir() || root.join("AGENTS.md").is_file() {
        out.push(Tool::Codex);
    }
    out
}

/// Detect installed AI tools by their footprints (deliberate difference from Spectra, which
/// generates nothing when --tools is omitted). Defaults to claude when nothing is found.
pub fn detect_tools(root: &Path) -> Vec<Tool> {
    let mut out = detect_footprint_tools(root);
    if out.is_empty() {
        out.push(Tool::Claude);
    }
    out
}

fn generate_tool(root: &Path, tool: Tool, spec_dir: &str, force: bool, store: StoreKind) -> Result<()> {
    // Root instruction file (marker upsert) + tool-specific extras.
    match tool {
        Tool::Claude => {
            write_if(&root.join(".claude").join("settings.json"), CLAUDE_SETTINGS, force)?;
            let md = root.join("CLAUDE.md");
            let merged = upsert_marker(util::read_opt(&md), &instructions_body(spec_dir, tool, store));
            util::write_file(&md, &merged)?;
        }
        Tool::Codex => {
            let md = root.join("AGENTS.md");
            let merged = upsert_marker(util::read_opt(&md), &instructions_body(spec_dir, tool, store));
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

/// Ensure `.gitignore` covers the `.speclink/` work directory, appending the
/// standard block when it does not. Returns whether the file was amended —
/// the context materializer turns that into a warning (never a silent write
/// of an unignored projection).
pub fn ensure_gitignore(path: &Path) -> Result<bool> {
    match util::read_opt(path) {
        Some(text) if text.contains(".speclink/") => Ok(false),
        Some(text) => {
            let mut new = text;
            if !new.ends_with('\n') {
                new.push('\n');
            }
            new.push('\n');
            new.push_str(GITIGNORE_BLOCK);
            util::write_file(path, &new)?;
            Ok(true)
        }
        None => {
            util::write_file(path, GITIGNORE_BLOCK)?;
            Ok(true)
        }
    }
}

/// Validate a comma-separated `--tools` value into a tool list. Speclink deliberately scopes
/// the supported tools to claude + codex.
pub fn parse_tools(spec: &str) -> Result<Vec<Tool>> {
    parse_tool_names(&spec.split(',').collect::<Vec<&str>>())
}

/// Validate built-in tool NAMES into a deduplicated selection: blank entries are skipped,
/// an unknown name is a loud error. The CLI arrives here through [`parse_tools`] with a
/// comma-separated value, the desktop with a list — one rule, one message for both.
pub fn parse_tool_names<S: AsRef<str>>(names: &[S]) -> Result<Vec<Tool>> {
    let mut out = Vec::new();
    for name in names {
        let name = name.as_ref().trim();
        if name.is_empty() {
            continue;
        }
        match Tool::parse(name) {
            Some(t) => {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
            None => bail!("unknown tool: {name} (supported: claude, codex)"),
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

        fn at(&self, rel: &str) -> PathBuf {
            self.dir.join(rel.split('/').collect::<PathBuf>())
        }

        fn write(&self, rel: &str, content: &str) {
            let path = self.at(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, content).unwrap();
        }

        fn exists(&self, rel: &str) -> bool {
            self.at(rel).exists()
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

    // --- 共用 built-in tools reconciliation ---
    // Spec requirement: 「built-in tools 權威收斂」＋ design「Core 單一 Workspace 工具同步入口」
    // ／「Built-in 選擇收斂且保留自訂描述子」與 Implementation Contract 的
    // 「Core and configuration contract」。

    const CUSTOM_DESCRIPTOR: &str = "  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n";
    const REMOTE_URL: &str = "https://team.example.test/api/speclink/v1/projects/acme";
    const CLAUDE_USER_TEXT: &str = "使用者寫在 CLAUDE.md 的段落";
    const CODEX_USER_TEXT: &str = "使用者寫在 AGENTS.md 的段落";

    fn instructions_file(tool: Tool) -> &'static str {
        match tool {
            Tool::Claude => "CLAUDE.md",
            Tool::Codex => "AGENTS.md",
        }
    }

    fn user_text(tool: Tool) -> &'static str {
        match tool {
            Tool::Claude => CLAUDE_USER_TEXT,
            Tool::Codex => CODEX_USER_TEXT,
        }
    }

    fn propose_skill(tool: Tool) -> String {
        format!("{}/speclink-propose/SKILL.md", tool.skills_dir())
    }

    /// Remote 模式 workspace，其 `.speclink.yaml` 除 built-in 選集外還帶 custom
    /// descriptor、remote section 與未知頂層鍵；兩份指令檔在 marker 之外先有使用者文字。
    fn seed_remote_workspace(root: &TempRoot, builtins: &[Tool]) {
        root.write("CLAUDE.md", &format!("{CLAUDE_USER_TEXT}\n"));
        root.write("AGENTS.md", &format!("{CODEX_USER_TEXT}\n"));
        let listed: String = builtins.iter().map(|t| format!("  - {}\n", t.name())).collect();
        root.write(
            ".speclink.yaml",
            &format!(
                "tools:\n{listed}{CUSTOM_DESCRIPTOR}remote:\n  url: {REMOTE_URL}\n  repo: desktop\nfuture_top_level: keep me\n"
            ),
        );
        update(&root.dir).expect("seed update");
    }

    fn builtin_names(root: &TempRoot) -> Vec<String> {
        let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
        let mut names: Vec<String> = app
            .tools
            .iter()
            .filter_map(|e| match e {
                ToolEntry::Builtin(n) => Some(n.clone()),
                ToolEntry::Descriptor(_) => None,
            })
            .collect();
        names.sort();
        names
    }

    fn marker_count(text: &str) -> usize {
        text.matches("<!-- SPECLINK:START").count()
    }

    /// 目錄快照（檔案內容與目錄項目），供「零寫入」斷言逐位元組比對。
    fn snapshot(root: &TempRoot) -> Vec<(String, Vec<u8>)> {
        fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
                if entry.path().is_dir() {
                    out.push((format!("{rel}/"), Vec::new()));
                    walk(&entry.path(), &rel, out);
                } else {
                    out.push((rel, std::fs::read(entry.path()).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(&root.dir, "", &mut out);
        out
    }

    /// Spec example「built-in 選集轉換」逐列：轉換後 built-in 集合等於請求，
    /// custom descriptor／未知頂層鍵／remote 原值保留，marker 不重複，使用者文字不動。
    #[test]
    fn reconcile_converts_builtin_selection_row_by_row() {
        let rows: [(&[Tool], &[Tool]); 3] = [
            (&[Tool::Claude], &[Tool::Codex]),
            (&[Tool::Claude, Tool::Codex], &[Tool::Claude]),
            (&[Tool::Codex], &[Tool::Claude, Tool::Codex]),
        ];
        for (i, (from, to)) in rows.iter().enumerate() {
            let root = TempRoot::new(&format!("reconcile-row-{i}"));
            seed_remote_workspace(&root, from);

            reconcile_builtin_tools(&root.dir, to).expect("reconcile succeeds");

            let want: Vec<String> = {
                let mut v: Vec<String> = to.iter().map(|t| t.name().to_string()).collect();
                v.sort();
                v
            };
            assert_eq!(builtin_names(&root), want, "row {i}: built-in 集合須等於請求");

            let app_text = root.read(".speclink.yaml");
            assert!(app_text.contains("wad-harness"), "row {i}: custom descriptor 須保留:\n{app_text}");
            assert!(app_text.contains("keep me"), "row {i}: 未知頂層鍵須保留:\n{app_text}");
            let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
            let remote = app.remote.as_ref().expect("remote section 須保留");
            assert_eq!(remote.url.as_deref(), Some(REMOTE_URL), "row {i}");
            assert_eq!(remote.repo.as_deref(), Some("desktop"), "row {i}");

            for tool in [Tool::Claude, Tool::Codex] {
                let md = instructions_file(tool);
                let text = root.read(md);
                let skill = propose_skill(tool);
                if to.contains(&tool) {
                    assert_eq!(marker_count(&text), 1, "row {i}: {md} 應恰有一個 marker:\n{text}");
                    assert!(root.exists(&skill), "row {i}: {skill} 應被補齊");
                } else {
                    assert_eq!(marker_count(&text), 0, "row {i}: {md} 的 marker 應被移除:\n{text}");
                    assert!(!root.exists(&skill), "row {i}: {skill} 應被清理");
                }
                assert!(text.contains(user_text(tool)), "row {i}: {md} 的使用者文字須保留:\n{text}");
            }
            assert!(!root.exists("openspec"), "row {i}: remote 模式不得建立 openspec/");
        }
    }

    /// 既有選集缺少產物時自動補齊，其他使用者檔案不受影響。
    #[test]
    fn reconcile_backfills_missing_managed_artifacts() {
        let root = TempRoot::new("reconcile-backfill");
        seed_remote_workspace(&root, &[Tool::Codex]);
        root.write("AGENTS.md", &format!("{CODEX_USER_TEXT}\n"));
        std::fs::remove_dir_all(root.at(".agents/skills/speclink-propose")).unwrap();
        root.write("docs/notes.md", "使用者檔案\n");

        reconcile_builtin_tools(&root.dir, &[Tool::Codex]).expect("reconcile succeeds");

        let text = root.read("AGENTS.md");
        assert_eq!(marker_count(&text), 1, "缺席的 marker 應補齊:\n{text}");
        assert!(text.contains(CODEX_USER_TEXT), "使用者文字須保留:\n{text}");
        assert!(root.exists(".agents/skills/speclink-propose/SKILL.md"), "缺席的 Skill 應補齊");
        assert_eq!(root.read("docs/notes.md"), "使用者檔案\n");
    }

    /// 空選集在任何寫入之前被拒（build-in 選集是非空契約）。
    #[test]
    fn reconcile_rejects_an_empty_selection_without_writing() {
        let root = TempRoot::new("reconcile-empty");
        seed_remote_workspace(&root, &[Tool::Codex]);
        let before = snapshot(&root);

        let err = reconcile_builtin_tools(&root.dir, &[]).expect_err("空選集必須失敗");

        let message = err.to_string();
        assert!(message.contains("claude") && message.contains("codex"), "{message}");
        assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
    }

    /// 壞 YAML 以單行錯誤失敗，設定與受管產物逐位元組不變。
    #[test]
    fn reconcile_bad_config_fails_loud_with_zero_writes() {
        let root = TempRoot::new("reconcile-bad-yaml");
        seed_remote_workspace(&root, &[Tool::Codex]);
        root.write(".speclink.yaml", "tools: [unclosed\n");
        let before = snapshot(&root);

        let err = reconcile_builtin_tools(&root.dir, &[Tool::Claude]).expect_err("壞 YAML 必須失敗");

        let message = err.to_string();
        assert!(message.contains(".speclink.yaml"), "錯誤須指名檔案：{message}");
        assert_eq!(message.lines().count(), 1, "錯誤須為單行：{message}");
        assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
    }

    /// Remote 模式沿用 remote 指令措辭，且不建立本機規格樹。
    #[test]
    fn reconcile_in_remote_mode_keeps_remote_wording_and_creates_no_spec_tree() {
        let root = TempRoot::new("reconcile-remote-wording");
        seed_remote_workspace(&root, &[Tool::Claude]);

        reconcile_builtin_tools(&root.dir, &[Tool::Claude, Tool::Codex]).expect("reconcile succeeds");

        for md in ["CLAUDE.md", "AGENTS.md"] {
            let text = root.read(md);
            assert!(text.contains("team system's spec store"), "{md} 須用 remote 措辭:\n{text}");
            assert!(!text.contains("openspec/specs/"), "{md} 不得出現本機規格路徑:\n{text}");
        }
        assert!(!root.exists("openspec"), "remote 模式不得建立 openspec/");
    }

    /// 同一選集下，既有 Workspace 收斂的受管產物與 filesystem init 的產物相同。
    #[test]
    fn reconcile_matches_init_output_for_the_same_selection() {
        let both = [Tool::Claude, Tool::Codex];
        let fresh = TempRoot::new("reconcile-parity-init");
        init(&fresh.dir, &both, false, "openspec").unwrap();

        let converged = TempRoot::new("reconcile-parity-converged");
        init(&converged.dir, &[Tool::Claude], false, "openspec").unwrap();
        reconcile_builtin_tools(&converged.dir, &both).expect("reconcile succeeds");

        for tool in both {
            let md = instructions_file(tool);
            assert_eq!(converged.read(md), fresh.read(md), "{md} 受管內容須與 init 相同");
            let skill = propose_skill(tool);
            assert_eq!(converged.read(&skill), fresh.read(&skill), "{skill} 須與 init 相同");
        }
        assert!(converged.exists("openspec/specs"), "既有 filesystem 規格樹須保留");
    }
}

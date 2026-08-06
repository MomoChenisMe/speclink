//! Project initialization and instruction-file updates.

use crate::config::{CustomTool, ToolEntry};
use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// 產物層的唯一版號：指令檔 SPECLINK 標記與技能檔 frontmatter 的 version 同源於此。
/// 僅在內嵌資產（assets/skills）或 marker 模板的 render 內容變動時遞增——與 app／CLI
/// 的發版號無關；`assets.lock` 鎖定測試把這條紀律變成紅燈。
pub const MARKER_VERSION: &str = "v1.14.0";

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
# Personal/CI overrides: SPECLINK_LOCALE, SPECLINK_SPEC_LOCALE, SPECLINK_TDD, SPECLINK_AUDIT, SPECLINK_WORKTREE
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
# worktree: true

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
///
/// `worktree` mirrors the generation gate: the two worktree skill lines appear only
/// when the policy is on, because a marker that points at a skill the policy just
/// pruned tells the agent to invoke something that no longer exists. Same principle
/// as the codex target omitting `verify` (that skill is not generated for codex).
pub fn instructions_body(spec_dir: &str, tool: Tool, store: StoreKind, worktree: bool) -> String {
    // Codex differs: `$speclink-` prefix, no plan mode, and no verify skill (for_codex=false).
    let (p, plan_line) = match tool {
        Tool::Codex => ("$speclink-", "- Requirements change mid-work? `ingest` → resume `apply`"),
        _ => ("/speclink-", "- Requirements change mid-work? Plan mode → `ingest` → resume `apply`"),
    };
    // 並行品質站（spec review-skill「審查技能的生成與正典化」）：實作完成、封存
    // 之前，由使用者判斷是否對高風險 change 執行審查；codex 無 verify（for_codex
    // =false），workflow 行只帶 review?。
    let (done_line, workflow) = match tool {
        Tool::Codex => (
            format!(
                "- Implementation is done, before archiving → optionally `{p}review` (craft quality; user's call), then `{p}archive`"
            ),
            "discuss? → propose → apply ⇄ ingest → review? → archive",
        ),
        _ => (
            format!(
                "- Implementation is done, before archiving → optional quality stations `{p}review` (craft quality) ∥ `{p}verify` (spec compliance; user's call), then `{p}archive`"
            ),
            "discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive",
        ),
    };
    // 兩行 worktree 指引隨政策進出；其餘內容不受影響。
    let worktree_lines = if worktree {
        format!(
            "- Implementing several independent changes at once → `{p}apply-with-worktree` (one git worktree per change)\n\
- A worktree change is committed and ready to land → `{p}worktree-merge` (merge back, then clean up)\n"
        )
    } else {
        String::new()
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
{worktree_lines}\
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
        worktree_lines = worktree_lines,
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
- Implementation is done, before archiving → optionally `speclink-review` (craft quality; user's call), then `speclink-archive`\n\
- Commit only files related to a specific change → `speclink-commit`\n\n\
## Workflow\n\n\
discuss? → propose → apply ⇄ ingest → review? → archive\n\n\
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
/// has no marker yet, the block is PREPENDED above the user's content (frozen behavior).
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
    // persisted as an active spec_dir line so later commands find it.
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
/// descriptor is an error. Without a recorded list, built-ins fall back to legacy
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
        // Legacy fallback: directory detection, codex excluded, no
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

/// Adopt speclink in a directory that already has an `openspec/` tree but no
/// `.speclink.yaml` — the workspace backfill entry (change: desktop-enable-speclink-prompt,
/// 決策 2). Composes [`store_init`]'s idempotent skeleton fill (directories via
/// create_dir_all; the workflow-config template only when config.yaml is absent — an
/// existing file with user policy is never touched) with [`reconcile_builtin_tools`]
/// (tools recorded in `.speclink.yaml`, managed skills and marker blocks regenerated).
/// Deliberately NOT behind `init`'s "Already initialized" guard; spec_dir is fixed to
/// `openspec` — without a `.speclink.yaml`, discovery's fallback is exactly that.
/// An empty selection is rejected before anything is written.
///
/// `.gitignore` is covered here explicitly: the `reconcile_builtin_tools` → `update`
/// path does not touch it (only `init`'s `workspace_init` does), so without this the
/// gitignored work directory would surface as untracked files in the user's repo.
pub fn adopt(root: &Path, tools: &[Tool]) -> Result<UpdateOutcome> {
    if tools.is_empty() {
        bail!("no tools selected (supported: claude, codex)");
    }
    store_init(&root.join("openspec"), false)?;
    ensure_gitignore(&root.join(".gitignore"))?;
    reconcile_builtin_tools(root, tools)
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

/// Whether worktree-gated skills belong in the generation set.
///
/// Reads `openspec/config.yaml`'s `worktree` key DIRECTLY rather than through the
/// four-layer policy resolution: injection is the project's persistent state, while
/// `SPECLINK_WORKTREE` is a per-run escape hatch for the skill's own runtime check.
///
/// A config that exists but cannot be parsed keeps the skills. Pruning is the
/// irreversible direction — a user who broke their config mid-flight must not lose the
/// merge skill their open worktree depends on; the skill's runtime check still refuses
/// to run under an off policy.
fn worktree_skills_enabled(root: &Path, spec_dir: &str) -> bool {
    let text = util::read_opt(&root.join(spec_dir).join("config.yaml"));
    match crate::config::WorkflowConfig::from_text(text.as_deref()) {
        Ok(cfg) => cfg.worktree.unwrap_or(false),
        Err(_) => true,
    }
}

/// Apply the worktree gate to one skill's target directory: `Ok(true)` means "skip it",
/// and any directory a previous policy-on generation left behind is removed first, so
/// flipping the policy off converges in a single run.
fn skip_gated_skill(skill: &skills::Skill, worktree_on: bool, dir: &Path) -> Result<bool> {
    if !skill.worktree_gated || worktree_on {
        return Ok(false);
    }
    if dir.is_dir() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(true)
}

/// Generate a custom tool's artifacts: skills under `<skills_dir>/speclink-*/SKILL.md`
/// (the non-Claude command subset) and the SPECLINK marker block in its instructions file.
fn generate_custom(root: &Path, tool: &CustomTool, spec_dir: &str, store: StoreKind) -> Result<()> {
    let md = root.join(&tool.instructions_file);
    let merged = upsert_marker(util::read_opt(&md), &custom_instructions_body(spec_dir, tool, store));
    util::write_file(&md, &merged)?;
    let worktree_on = worktree_skills_enabled(root, spec_dir);
    for skill in skills::registry() {
        if !skill.for_codex {
            continue;
        }
        let dir = root.join(&tool.skills_dir).join(format!("speclink-{}", skill.name));
        if skip_gated_skill(&skill, worktree_on, &dir)? {
            continue;
        }
        let content = skills::render_skill_file_for(skills::RenderTarget::Custom(tool), &skill, spec_dir);
        util::write_file(&dir.join("SKILL.md"), &content)?;
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
    let md = root.join(instructions_path(tool));
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

/// Detect installed AI tools by their footprints (legacy fallback path only — `init`
/// itself requires an explicit tool selection). Defaults to claude when nothing is found.
pub fn detect_tools(root: &Path) -> Vec<Tool> {
    let mut out = detect_footprint_tools(root);
    if out.is_empty() {
        out.push(Tool::Claude);
    }
    out
}

fn generate_tool(root: &Path, tool: Tool, spec_dir: &str, force: bool, store: StoreKind) -> Result<()> {
    // Root instruction file (marker upsert). No other tool-level files: the tool's own
    // user settings (e.g. .claude/settings.json) are the user's data, never generated
    // (spec: 工具檔生成不寫入 AI 工具的使用者設定檔).
    let worktree_on = worktree_skills_enabled(root, spec_dir);
    let md = root.join(instructions_path(tool));
    let merged = upsert_marker(
        util::read_opt(&md),
        &instructions_body(spec_dir, tool, store, worktree_on),
    );
    util::write_file(&md, &merged)?;
    // Skills: Claude gets the full registry; codex gets the command subset.
    for skill in skills::registry() {
        if tool != Tool::Claude && !skill.for_codex {
            continue;
        }
        let dir = root.join(tool.skills_dir()).join(format!("speclink-{}", skill.name));
        if skip_gated_skill(&skill, worktree_on, &dir)? {
            continue;
        }
        let content = skills::render_skill_file_for(skills::RenderTarget::Builtin(tool), &skill, spec_dir);
        write_if(&dir.join("SKILL.md"), &content, force)?;
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

/// 指令檔過期探測的整體判定（規格「指令檔過期探測」五態）。聚合優先序
/// 較新 > 缺失 > 過期 > 現版：較新排最前，只要有任何檔案領先引擎，就不提供
/// 任何會改寫它的動作；缺失優先於過期，因為「從未安裝」與「裝了但舊了」是不同
/// 的使用者情境，提示文案據此分流。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstructionStatus {
    /// tools 清單宣告的指令檔不存在＝從未安裝（如 clone 後指令檔未進版控）。
    Missing,
    /// 任一工具的標記版號與現版不等且不領先現版。
    Stale,
    /// 任一工具的標記版號數值新於現版＝工作區檔案領先引擎（本體是舊版）。
    Newer,
    Current,
    /// 設定解析失敗或指令檔存在但讀取錯誤——不得與現版混同。
    Unknown,
}

/// 單一內建工具的探測結果。`workspaceVersion` 為 None 代表檔案不存在或標記已被
/// 移除；兩者由 `missing` 區分（決策 2：退出受管與從未安裝意圖完全不同）。
/// `stale` 與 `newer` 互斥：方向由引擎判定，消費端不重算。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInstructionState {
    pub tool: String,
    pub workspace_version: Option<String>,
    pub stale: bool,
    pub newer: bool,
    pub missing: bool,
}

/// 探測回報（決策 3）：目前引擎版本、逐工具狀態，以及「更新將新建或改寫且內容
/// 與現版 render 不同」的受管檔清單（專案根相對路徑）。清單不區分「過期」與
/// 「使用者自訂」——系統無歷史 render，無從分辨。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionProbe {
    pub status: InstructionStatus,
    pub current_version: String,
    pub tools: Vec<ToolInstructionState>,
    pub differing_files: Vec<String>,
}

/// 讀取指令檔的 SPECLINK 標記版號；無標記回 None（＝使用者已退出受管）。
fn marker_version_of(text: &str) -> Option<&str> {
    let start = text.find("<!-- SPECLINK:START")? + "<!-- SPECLINK:START".len();
    let rest = &text[start..];
    let end = rest.find("-->")?;
    let version = rest[..end].trim();
    (!version.is_empty()).then_some(version)
}

/// 比對前正規化換行：Windows checkout（core.autocrlf）的 CRLF 檔案不得因換行
/// 形式被誤報為內容有異。
fn eol_normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// 版號拆段：去 v 前綴、以點號拆段、逐段解析為數字；任一段非純數字即回 None
///（不排序無法解析的版號）。
fn version_parts(version: &str) -> Option<Vec<u64>> {
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|seg| seg.parse::<u64>().ok())
        .collect()
}

/// 工作區標記版號是否數值領先引擎版號（決策 3）：段數不足補零後逐段比較。
/// 任一邊無法完整解析為數字段時回 false——手改壞的標記寧可誤報過期（改寫即恢復
/// 受管狀態），不可誤報較新（那會封鎖 update）。
fn workspace_is_newer(workspace: &str, engine: &str) -> bool {
    let (Some(a), Some(b)) = (version_parts(workspace), version_parts(engine)) else {
        return false;
    };
    for i in 0..a.len().max(b.len()) {
        let (l, r) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if l != r {
            return l > r;
        }
    }
    false
}

/// 唯讀的指令檔過期探測（決策 2、3）：依 `.speclink.yaml` 的 tools 清單（與
/// [`update`] 同一資料源）讀各內建工具的指令檔，以標記版號對 [`MARKER_VERSION`]
/// 判方向——數值領先現版為較新，其餘退回字串相等判定（不等即過期）。方向是唯一
/// 判準來源：desktop 與 CLI 共用同一裁決，領先時兩個出口都不提供改寫動作。
///
/// 零寫入：desktop 開專案時搭載，探測失敗不得阻斷開啟。自訂描述子第一版不涵蓋
/// （回報結構已預留 tool 名欄位，納入時不需改形狀）。
pub fn probe_instructions(root: &Path) -> InstructionProbe {
    let unknown = || InstructionProbe {
        status: InstructionStatus::Unknown,
        current_version: MARKER_VERSION.to_string(),
        tools: Vec::new(),
        differing_files: Vec::new(),
    };
    let Ok(app) = crate::config::AppConfig::load(&root.join(".speclink.yaml")) else {
        return unknown();
    };
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let store = if app.remote.is_some() {
        StoreKind::Remote
    } else {
        StoreKind::Fs
    };

    let mut selected: Vec<Tool> = Vec::new();
    for entry in &app.tools {
        // 未知內建名與自訂描述子皆跳過：前者 update() 只給警告，後者不在第一版範圍。
        if let ToolEntry::Builtin(name) = entry {
            if let Some(tool) = Tool::parse(name) {
                if !selected.contains(&tool) {
                    selected.push(tool);
                }
            }
        }
    }

    let mut tools = Vec::new();
    for tool in &selected {
        let md = root.join(instructions_path(*tool));
        if !md.is_file() {
            tools.push(ToolInstructionState {
                tool: tool.name().to_string(),
                workspace_version: None,
                stale: false,
                newer: false,
                missing: true,
            });
            continue;
        }
        // 檔案在但讀不出來＝無法判定（權限、編碼）；與「不存在」是不同的狀態。
        let Ok(text) = std::fs::read_to_string(&md) else {
            return unknown();
        };
        let version = marker_version_of(&text).map(str::to_string);
        // 方向優先於相等判定：領先現版的標記是「較新」，不得再算成過期。
        let newer = version
            .as_deref()
            .is_some_and(|v| workspace_is_newer(v, MARKER_VERSION));
        tools.push(ToolInstructionState {
            tool: tool.name().to_string(),
            stale: !newer && version.as_deref().is_some_and(|v| v != MARKER_VERSION),
            newer,
            missing: false,
            workspace_version: version,
        });
    }

    let status = if tools.iter().any(|t| t.newer) {
        InstructionStatus::Newer
    } else if tools.iter().any(|t| t.missing) {
        InstructionStatus::Missing
    } else if tools.iter().any(|t| t.stale) {
        InstructionStatus::Stale
    } else {
        InstructionStatus::Current
    };
    let differing_files = match status {
        InstructionStatus::Missing | InstructionStatus::Stale | InstructionStatus::Newer => {
            differing_managed_files(root, &selected, &spec_dir, store)
        }
        _ => Vec::new(),
    };

    InstructionProbe {
        status,
        current_version: MARKER_VERSION.to_string(),
        tools,
        differing_files,
    }
}

/// 更新將新建或改寫、且內容與現版 render 不同的受管檔（專案根相對路徑）。
/// 指令檔的期望內容走與 [`generate_tool`] 相同的 marker upsert——使用者寫在標記
/// 之外的內容原樣保留，不得因此被誤列為差異。不存在的檔案內容視為空、必列入。
fn differing_managed_files(
    root: &Path,
    tools: &[Tool],
    spec_dir: &str,
    store: StoreKind,
) -> Vec<String> {
    let mut differing = Vec::new();
    let mut compare = |rel: String, expected: &str| {
        let actual = std::fs::read_to_string(root.join(rel.split('/').collect::<PathBuf>()))
            .unwrap_or_default();
        if eol_normalized(&actual) != eol_normalized(expected) {
            differing.push(rel);
        }
    };
    // 被政策排除的技能不屬於預期生成集合——否則政策關閉的專案會永遠被報成
    // 「檔案缺失」而過期；marker 的兩行 worktree 指引同理。
    let worktree_on = worktree_skills_enabled(root, spec_dir);
    for tool in tools {
        let rel = instructions_path(*tool);
        let existing = util::read_opt(&root.join(rel));
        let expected = upsert_marker(
            existing,
            &instructions_body(spec_dir, *tool, store, worktree_on),
        );
        compare(rel.to_string(), &expected);
        for skill in skills::registry() {
            if *tool != Tool::Claude && !skill.for_codex {
                continue;
            }
            if skill.worktree_gated && !worktree_on {
                continue;
            }
            let expected =
                skills::render_skill_file_for(skills::RenderTarget::Builtin(*tool), &skill, spec_dir);
            compare(
                format!("{}/speclink-{}/SKILL.md", tool.skills_dir(), skill.name),
                &expected,
            );
        }
    }
    differing
}

/// 內建工具的指令檔路徑（專案根相對）。
fn instructions_path(tool: Tool) -> &'static str {
    match tool {
        Tool::Claude => "CLAUDE.md",
        Tool::Codex => "AGENTS.md",
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
        instructions_path(tool)
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

    // --- adopt：工作區補齊入口 ---
    // Spec requirement:「工作區補齊入口」（desktop-enable-speclink-prompt）——
    // 冪等補骨架缺件、寫 tools、生成受管檔，既有 openspec/ 內容零觸碰。

    const CUSTOM_WORKFLOW_CONFIG: &str =
        "schema: spec-driven\nlocale: tw\nrules:\n  proposal:\n    - 保持提案精簡\n";

    /// 未啟用目錄：openspec/ 內有規格文件、變更、討論與自訂 config.yaml，
    /// 但專案根無 .speclink.yaml。
    fn seed_unadopted(root: &TempRoot) {
        root.write("openspec/specs/auth/spec.md", "## Purpose\n既有規格文件。\n");
        root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");
        root.write("openspec/discussions/auth-scope.md", "# Discussion\n既有討論。\n");
        root.write("openspec/config.yaml", CUSTOM_WORKFLOW_CONFIG);
    }

    /// Spec Scenario「補齊工作區檔且既有內容零觸碰」：工作區檔補齊，
    /// 既有 openspec/ 文件（含自訂 config.yaml）位元級不變。
    #[test]
    fn adopt_fills_workspace_files_with_existing_content_untouched() {
        let root = TempRoot::new("adopt-fill");
        seed_unadopted(&root);
        let before = snapshot(&root);

        adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

        let app = root.read(".speclink.yaml");
        assert!(app.contains("claude"), "tools 須記錄 claude：{app}");
        assert!(root.read("CLAUDE.md").contains("<!-- SPECLINK:START"));
        assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"));

        let after = snapshot(&root);
        for entry in before.iter().filter(|(rel, _)| !rel.ends_with('/')) {
            assert!(after.contains(entry), "既有文件必須位元級不變：{}", entry.0);
        }
        assert_eq!(root.read("openspec/config.yaml"), CUSTOM_WORKFLOW_CONFIG);
    }

    /// Spec Scenario「骨架缺件補齊」：缺 specs/ 與 config.yaml 時補齊目錄與範本。
    #[test]
    fn adopt_backfills_missing_skeleton() {
        let root = TempRoot::new("adopt-skeleton");
        root.write("openspec/changes/add-auth/proposal.md", "## Why\n既有變更。\n");

        adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

        assert!(root.at("openspec/specs").is_dir());
        assert!(root.at("openspec/changes/archive").is_dir());
        assert_eq!(root.read("openspec/config.yaml"), WORKFLOW_CONFIG_TEMPLATE);
    }

    /// Spec Scenario「工作資料夾納入版控忽略」：.gitignore 缺席時建立並涵蓋 `.speclink/`。
    #[test]
    fn adopt_creates_gitignore_covering_the_work_dir() {
        let root = TempRoot::new("adopt-gitignore-new");
        seed_unadopted(&root);

        adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

        assert!(root.read(".gitignore").contains(".speclink/"));
    }

    /// Spec Example「既有 .gitignore 追加而非覆寫」逐值：原有兩行保留，多出 `.speclink/`。
    #[test]
    fn adopt_appends_to_an_existing_gitignore_without_overwriting() {
        let root = TempRoot::new("adopt-gitignore-append");
        seed_unadopted(&root);
        root.write(".gitignore", "node_modules/\ndist/\n");

        adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");

        let text = root.read(".gitignore");
        for line in ["node_modules/", "dist/", ".speclink/"] {
            assert!(text.contains(line), "{line} 須存在於 .gitignore：\n{text}");
        }
    }

    /// 已涵蓋時重跑不重複追加（檔案位元級不變）。
    #[test]
    fn adopt_does_not_duplicate_an_existing_work_dir_entry() {
        let root = TempRoot::new("adopt-gitignore-idem");
        seed_unadopted(&root);
        adopt(&root.dir, &[Tool::Claude]).expect("first adopt");
        let first = root.read(".gitignore");

        adopt(&root.dir, &[Tool::Claude]).expect("second adopt");

        assert_eq!(root.read(".gitignore"), first, "重跑不得重複追加");
        assert_eq!(first.matches(".speclink/").count(), 1, "條目須恰有一筆：\n{first}");
    }

    // --- 工具檔生成不寫入 AI 工具的使用者設定檔 ---
    // Spec requirement:「工具檔生成不寫入 AI 工具的使用者設定檔」
    // （remove-claude-settings-write）——settings.json 屬使用者資料，
    // 任何生成路徑不得建立或改寫。

    const USER_SETTINGS: &str =
        "{\"enabledPlugins\":{\"frontend-design\":true},\"includeGitInstructions\":false}";

    /// Spec Scenario「init 不產生使用者設定檔」。
    #[test]
    fn init_does_not_create_the_user_settings_file() {
        let root = TempRoot::new("no-settings-init");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"), "技能檔照常生成");
        assert!(!root.exists(".claude/settings.json"), "不得產生使用者設定檔");
    }

    /// Spec Scenario「既有使用者設定檔在工具同步後位元級不變」
    /// ＋ Example「自訂外掛設定不被清空」逐值。
    #[test]
    fn update_leaves_an_existing_user_settings_file_untouched() {
        let root = TempRoot::new("no-settings-update");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".claude/settings.json", USER_SETTINGS);
        std::fs::remove_dir_all(root.at(".claude/skills/speclink-propose")).unwrap();

        update(&root.dir).expect("update succeeds");

        assert_eq!(root.read(".claude/settings.json"), USER_SETTINGS, "使用者設定檔位元級不變");
        assert!(root.exists(".claude/skills/speclink-propose/SKILL.md"), "受管技能檔照常再生");
    }

    /// Spec Scenario「工作區補齊不產生使用者設定檔」。
    #[test]
    fn adopt_does_not_create_the_user_settings_file() {
        let root = TempRoot::new("no-settings-adopt");
        seed_unadopted(&root);
        adopt(&root.dir, &[Tool::Claude]).expect("adopt succeeds");
        assert!(!root.exists(".claude/settings.json"), "不得產生使用者設定檔");
    }

    /// Spec Scenario「重複執行冪等」：相同 tools 連續執行兩次，全樹位元級相同。
    #[test]
    fn adopt_twice_with_same_tools_is_idempotent() {
        let root = TempRoot::new("adopt-idem");
        seed_unadopted(&root);
        adopt(&root.dir, &[Tool::Claude, Tool::Codex]).expect("first adopt");
        let first = snapshot(&root);

        adopt(&root.dir, &[Tool::Claude, Tool::Codex]).expect("second adopt");

        assert_eq!(snapshot(&root), first, "重複執行必須收斂於相同結果");
    }

    /// Spec Scenario「tools 空清單拒絕」：回單行錯誤且目錄零寫入。
    #[test]
    fn adopt_rejects_empty_tools_with_zero_writes() {
        let root = TempRoot::new("adopt-empty");
        seed_unadopted(&root);
        let before = snapshot(&root);

        let err = adopt(&root.dir, &[]).expect_err("空 tools 必須失敗");

        let message = err.to_string();
        assert!(message.contains("claude") && message.contains("codex"), "{message}");
        assert_eq!(message.lines().count(), 1, "錯誤須為單行：{message}");
        assert_eq!(snapshot(&root), before, "失敗不得留下任何寫入");
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

    // --- 指令檔過期探測（規格「指令檔過期探測」；決策 2、3） ---

    /// 把工作區的 marker 版號改成指定值（模擬以別版引擎生成的工作區——舊值模擬
    /// 落後、新值模擬領先）。
    fn set_marker(root: &TempRoot, tool: Tool, version: &str) {
        let file = instructions_file(tool);
        let text = root.read(file).replace(MARKER_VERSION, version);
        root.write(file, &text);
    }

    /// 比現版領先一個主版號的標記版號：工作區檔案由更新的引擎生成的情境。
    fn ahead_of_current() -> String {
        let major: u64 = MARKER_VERSION
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .expect("MARKER_VERSION 主版號可解析");
        format!("v{}.0.0", major + 1)
    }

    #[test]
    fn probe_reports_stale_and_lists_differing_files() {
        // Scenario「舊版工作區判過期並列差異檔」：標記版號不等即過期，並列出
        // 內容與現版 render 不同的受管檔（指令檔與技能檔皆可能在列）。
        let root = TempRoot::new("probe-stale");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_marker(&root, Tool::Claude, "v0.9.0");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Stale, "{probe:?}");
        assert_eq!(probe.current_version, MARKER_VERSION);
        assert_eq!(probe.tools.len(), 1);
        assert_eq!(probe.tools[0].tool, "claude");
        assert_eq!(probe.tools[0].workspace_version.as_deref(), Some("v0.9.0"));
        assert!(probe.tools[0].stale && !probe.tools[0].missing);
        assert!(
            probe.differing_files.contains(&"CLAUDE.md".to_string()),
            "改動過的指令檔須列入差異清單：{:?}",
            probe.differing_files
        );
        assert!(!probe.tools[0].newer, "落後的工作區不得判較新");
    }

    #[test]
    fn workspace_version_direction_only_orders_parsable_versions() {
        // 決策 3：去 v 前綴、以點拆段、逐段數值比較、段數不足補零；任一邊無法
        // 完整解析為數字段時不排序方向——寧可誤報過期，不可誤報較新（會封鎖 update）。
        // spec Example「引擎 v1.11.0 探測 v1.14.0 工作區」的字面值：
        assert!(workspace_is_newer("v1.14.0", "v1.11.0"), "工作區領先引擎");
        assert!(!workspace_is_newer("v1.11.0", "v1.14.0"), "工作區落後引擎");
        assert!(!workspace_is_newer("v1.14.0", "v1.14.0"), "同版不算領先");
        // 逐段數值（非字典序）：v1.9.0 < v1.10.0
        assert!(workspace_is_newer("v1.10.0", "v1.9.0"), "以數值而非字典序比較");
        // 段數不足補零
        assert!(workspace_is_newer("v1.14.1", "v1.14"), "缺段視為 0");
        assert!(!workspace_is_newer("v1.14", "v1.14.0"), "補零後相等不算領先");
        // 無法解析：兩個方向都不判較新
        assert!(!workspace_is_newer("bogus", "v1.14.0"), "無法解析不得判較新");
        assert!(!workspace_is_newer("v1.14.0-beta", "v1.14.0"), "非純數字段不得判較新");
        assert!(!workspace_is_newer("v1.14.0", "bogus"), "引擎端無法解析亦不判較新");
    }

    #[test]
    fn probe_reports_newer_when_the_workspace_leads_the_engine() {
        // Scenario「工作區檔案領先引擎判較新」＋ Example「引擎 v1.11.0 探測 v1.14.0
        // 工作區」：2026-08-05 事故情境——舊判準回報「過期」，按「更新」即降級。
        let root = TempRoot::new("probe-newer");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let ahead = ahead_of_current();
        set_marker(&root, Tool::Claude, &ahead);

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Newer, "{probe:?}");
        assert_eq!(probe.current_version, MARKER_VERSION);
        assert_eq!(probe.tools[0].workspace_version.as_deref(), Some(ahead.as_str()));
        assert!(probe.tools[0].newer && !probe.tools[0].stale && !probe.tools[0].missing);
        assert!(
            probe.differing_files.contains(&"CLAUDE.md".to_string()),
            "較新時仍須回報差異檔清單：{:?}",
            probe.differing_files
        );
    }

    #[test]
    fn probe_prefers_newer_over_missing_and_stale() {
        // Scenario「較新優先於缺失與過期」：任一工具領先即整體較新——任何會改寫
        // 領先檔案的動作都不該被提供。
        let root = TempRoot::new("probe-newer-wins");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        set_marker(&root, Tool::Claude, &ahead_of_current());
        std::fs::remove_file(root.at("AGENTS.md")).unwrap();

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Newer, "{probe:?}");
        let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
        assert!(codex.missing && !codex.newer, "缺失的工具不得被標成較新：{codex:?}");
    }

    #[test]
    fn probe_falls_back_to_equality_for_an_unparsable_marker_version() {
        // Scenario「無法解析的版號退回相等判定」：手改壞的標記判過期（改寫即恢復
        // 受管狀態），絕不判較新（那會封鎖 update）。
        let root = TempRoot::new("probe-unparsable");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_marker(&root, Tool::Claude, "v-not-a-version");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Stale, "{probe:?}");
        assert!(probe.tools[0].stale && !probe.tools[0].newer, "{:?}", probe.tools[0]);
    }

    #[test]
    fn probe_reports_current_for_a_freshly_generated_workspace() {
        // Scenario「現版工作區不過期」：差異清單為空。
        let root = TempRoot::new("probe-current");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Current, "{probe:?}");
        assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
        assert!(probe.tools.iter().all(|t| !t.stale && !t.missing));
    }

    #[test]
    fn probe_treats_a_removed_marker_as_opted_out() {
        // Scenario「標記移除視為退出受管」：檔案在但整塊標記被移除＝表達過移除
        // 意圖，回報現版、不列差異檔——提示層不得引導使用者重新植入。
        let root = TempRoot::new("probe-optout");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write("CLAUDE.md", "只剩使用者自己的內容。\n");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Current, "{probe:?}");
        assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
        assert!(!probe.tools[0].stale && !probe.tools[0].missing);
        assert_eq!(probe.tools[0].workspace_version, None);
    }

    #[test]
    fn probe_reports_missing_when_an_instruction_file_does_not_exist() {
        // Scenario「指令檔不存在判缺失」：一工具現版、另一工具檔案不存在
        //（clone 後指令檔未進版控）→ 缺失優先於過期，且不與退出受管或無法判定混同。
        let root = TempRoot::new("probe-missing");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        std::fs::remove_file(root.at("AGENTS.md")).unwrap();
        // 另一支同時過期：缺失仍須勝出。
        set_marker(&root, Tool::Claude, "v0.9.0");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Missing, "{probe:?}");
        let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
        assert!(codex.missing && !codex.stale, "{codex:?}");
        assert_eq!(codex.workspace_version, None);
        assert!(
            probe.differing_files.contains(&"AGENTS.md".to_string()),
            "不存在的受管檔須列入（內容視為空）：{:?}",
            probe.differing_files
        );
    }

    #[test]
    fn probe_reports_unknown_for_a_malformed_config() {
        // Scenario「設定損壞回報無法判定」：不得與現版或過期混同。
        let root = TempRoot::new("probe-badconfig");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", "tools: [\n");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Unknown, "{probe:?}");
        assert!(probe.tools.is_empty(), "{:?}", probe.tools);
        assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
    }

    #[test]
    fn probe_ignores_line_ending_differences() {
        // Scenario「換行差異不誤報」：CRLF 工作區（Windows core.autocrlf）僅換行
        // 形式不同的檔案不得列入差異清單。
        let root = TempRoot::new("probe-crlf");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let skill = propose_skill(Tool::Claude);
        let crlf = root.read(&skill).replace('\n', "\r\n");
        root.write(&skill, &crlf);
        set_marker(&root, Tool::Claude, "v0.9.0");

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Stale, "{probe:?}");
        assert!(
            !probe.differing_files.contains(&skill),
            "僅換行形式不同的檔案不得列入：{:?}",
            probe.differing_files
        );
    }

    #[test]
    fn probe_with_an_empty_tools_list_reports_current_and_writes_nothing() {
        // tools 空清單＝沒有受管工具可查：回報現版、零差異；且探測全程零寫入。
        let root = TempRoot::new("probe-empty-tools");
        init(&root.dir, &[], false, "openspec").unwrap();
        let before = snapshot(&root);

        let probe = probe_instructions(&root.dir);
        assert_eq!(probe.status, InstructionStatus::Current, "{probe:?}");
        assert!(probe.tools.is_empty());
        assert_eq!(snapshot(&root), before, "探測不得寫入任何檔案");
    }

    // --- worktree 政策閘：生成集合隨 openspec/config.yaml 的 worktree 檔值 ---
    // Spec requirement「worktree 技能的政策條件式生成」。

    /// 兩顆受閘控技能於某工具 skills 目錄下的相對路徑。
    fn worktree_skill_dirs(tool: Tool) -> [String; 2] {
        [
            format!("{}/speclink-apply-with-worktree", tool.skills_dir()),
            format!("{}/speclink-worktree-merge", tool.skills_dir()),
        ]
    }

    /// 覆寫 workflow config 的 worktree 政策（其餘欄位不留，測試只關心這一鍵）。
    fn set_worktree_policy(root: &TempRoot, on: bool) {
        root.write("openspec/config.yaml", &format!("schema: spec-driven\nworktree: {on}\n"));
    }

    #[test]
    fn generation_omits_worktree_skills_when_the_policy_key_is_absent() {
        // Scenario「政策關閉時生成集合不含 worktree 技能」的鍵缺席分支：init 範本
        // 只留註解示例，等同未設＝關。
        let root = TempRoot::new("gate-absent");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();

        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(!root.exists(&dir), "政策未設時不得生成 {dir}");
        }
        // 其餘技能照常生成。
        assert!(root.exists(".claude/skills/speclink-apply"), "非閘控技能須照常生成");
    }

    #[test]
    fn generation_omits_worktree_skills_when_the_policy_is_false() {
        let root = TempRoot::new("gate-false");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
        set_worktree_policy(&root, false);

        update(&root.dir).unwrap();

        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(!root.exists(&dir), "政策為 false 時不得生成 {dir}");
        }
        assert!(root.exists(".claude/skills/speclink-apply"));
    }

    #[test]
    fn generation_includes_worktree_skills_when_the_policy_is_on() {
        // Scenario「政策開啟時注入兩顆技能」。
        let root = TempRoot::new("gate-on");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
        set_worktree_policy(&root, true);

        update(&root.dir).unwrap();

        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(root.exists(&format!("{dir}/SKILL.md")), "政策為 true 時須生成 {dir}");
        }
    }

    #[test]
    fn the_gate_applies_to_codex_and_custom_descriptors_alike() {
        // 需求句「此過濾對 claude、codex 與自訂描述子工具一視同仁」。
        let root = TempRoot::new("gate-all-targets");
        init(&root.dir, &[Tool::Claude, Tool::Codex], true, "openspec").unwrap();
        root.write(
            ".speclink.yaml",
            &format!("tools:\n  - claude\n  - codex\n{CUSTOM_DESCRIPTOR}"),
        );

        update(&root.dir).unwrap();
        for dir in worktree_skill_dirs(Tool::Codex) {
            assert!(!root.exists(&dir), "政策關閉時 codex 不得生成 {dir}");
        }
        assert!(!root.exists(".wad/skills/speclink-apply-with-worktree"), "描述子亦受閘控");
        assert!(root.exists(".wad/skills/speclink-apply"), "描述子的非閘控技能照常生成");

        set_worktree_policy(&root, true);
        update(&root.dir).unwrap();
        for dir in worktree_skill_dirs(Tool::Codex) {
            assert!(root.exists(&dir), "政策開啟時 codex 須生成 {dir}");
        }
        assert!(root.exists(".wad/skills/speclink-apply-with-worktree/SKILL.md"));
    }

    #[test]
    fn the_marker_lists_worktree_skills_only_when_the_policy_is_on() {
        // Spec requirement「marker 技能指引跟隨 worktree 政策」：技能檔被政策清掉而
        // marker 仍指路，等於叫代理呼叫不存在的技能。
        let root = TempRoot::new("marker-gate");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
        let off = root.read("CLAUDE.md");
        assert!(!off.contains("apply-with-worktree"), "政策關閉時 marker 不得提 apply-with-worktree:\n{off}");
        assert!(!off.contains("worktree-merge"), "政策關閉時 marker 不得提 worktree-merge:\n{off}");
        assert!(off.contains("/speclink-apply`"), "其餘技能指引須照舊:\n{off}");

        set_worktree_policy(&root, true);
        update(&root.dir).unwrap();
        let on = root.read("CLAUDE.md");

        let added: Vec<&str> = on.lines().filter(|l| !off.lines().any(|o| o == *l)).collect();
        assert_eq!(added.len(), 2, "兩版 marker 應僅差兩行 worktree 指引，實得：{added:?}");
        assert!(added.iter().any(|l| l.contains("apply-with-worktree")));
        assert!(added.iter().any(|l| l.contains("worktree-merge")));
    }

    #[test]
    fn an_unparseable_workflow_config_keeps_the_worktree_skills() {
        // 刪除是不可逆方向：政策讀不出來時（使用者手改壞了 config.yaml）一律保留
        // 技能，由技能內的執行期政策檢查兜底，絕不以「讀不到＝關」為由清掉檔案。
        let root = TempRoot::new("gate-broken-config");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
        set_worktree_policy(&root, true);
        update(&root.dir).unwrap();

        root.write("openspec/config.yaml", "schema: [unterminated\n");
        update(&root.dir).unwrap();

        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(root.exists(&dir), "政策文件壞掉時不得清掉 {dir}");
        }
    }
}

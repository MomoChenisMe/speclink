//! Project initialization and instruction-file updates.

use crate::config::{CustomTool, ToolEntry};
use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// 產物層的唯一版號：技能檔 frontmatter 的 version 同源於此，也是過期探測與
/// 降級守門的比對基準。僅在內嵌資產（assets/skills）的 render 內容變動時遞增——
/// 與 app／CLI 的發版號無關；`assets.lock` 鎖定測試把這條紀律變成紅燈。
pub const ASSET_VERSION: &str = "v1.30.0";

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

pub struct InitOutcome {
    pub spec_dir_abs: PathBuf,
}

/// Initialize speclink in `root`: the spec-document tree (`openspec/` skeleton and
/// workflow-config template), then the host-side files — `.speclink.yaml`, the
/// `.gitignore` entry and the selected tools' skills. Instruction files (`CLAUDE.md`,
/// `AGENTS.md`) are NOT part of the managed set (change: remove-marker-injection);
/// routing lives in the skills themselves.
pub fn init(root: &Path, tools: &[Tool], force: bool, spec_dir: &str) -> Result<InitOutcome> {
    let spec_root = root.join(spec_dir);
    if !force && (spec_root.exists() || root.join(".speclink.yaml").is_file()) {
        bail!("Already initialized. Use --force to reinitialize.");
    }
    // 守門在任何寫入之前：檢查面就是這次要寫的 skills 目錄。
    SyncPlan::resolve(root, ToolSelection::builtins_only(tools), spec_dir).guard()?;

    store_init(&spec_root, force)?;
    write_if(&root.join(".speclink.yaml"), &app_config_text(tools, spec_dir), force)?;
    ensure_gitignore(&root.join(".gitignore"))?;
    // 遺留剝除（design D2）：re-init 蓋過舊工作區時才有東西可剝，新專案是 no-op。
    for tool in tools {
        strip_legacy_marker(&root.join(instructions_path(*tool)))?;
    }
    // 政策以 store_init 之後的 config.yaml 為準——`--force` 會把它寫回範本。
    SyncPlan::resolve(root, ToolSelection::builtins_only(tools), spec_dir).write_skills(force)?;

    Ok(InitOutcome {
        spec_dir_abs: spec_root,
    })
}

/// Remote-store initialization: the same host-side files as [`init`] plus the
/// `remote:` section in `.speclink.yaml` — deliberately NO spec-document tree (it
/// lives on the server, so no `openspec/` skeleton and no local workflow-config
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
    let plan = SyncPlan::resolve(root, ToolSelection::builtins_only(tools), "openspec");
    plan.guard()?;

    write_if(&root.join(".speclink.yaml"), &app_config_text(tools, "openspec"), force)?;
    ensure_gitignore(&root.join(".gitignore"))?;
    for tool in tools {
        strip_legacy_marker(&root.join(instructions_path(*tool)))?;
    }
    plan.write_skills(force)?;
    crate::config::write_remote_section(root, url, repo)
}

/// The initial `.speclink.yaml`: the template plus the actual tool selection (so `update`
/// can sync against the recorded list later). A non-default spec_dir is persisted as an
/// active `spec_dir` line so later commands find it.
fn app_config_text(tools: &[Tool], spec_dir: &str) -> String {
    let mut content = if tools.is_empty() {
        APP_CONFIG_TEMPLATE.to_string()
    } else {
        let names: Vec<&str> = tools.iter().map(|t| t.name()).collect();
        format!("{APP_CONFIG_TEMPLATE}tools: [{}]\n", names.join(", "))
    };
    if spec_dir != "openspec" {
        content = content.replace("# spec_dir: docs/specs", &format!("spec_dir: {spec_dir}"));
    }
    content
}

/// Store init: the spec-document tree (`openspec/` skeleton) and the workflow-config
/// template — the canonical home of the policy fields.
fn store_init(spec_root: &Path, force: bool) -> Result<()> {
    std::fs::create_dir_all(spec_root.join("specs"))?;
    std::fs::create_dir_all(spec_root.join("changes").join("archive"))?;
    write_if(&spec_root.join("config.yaml"), WORKFLOW_CONFIG_TEMPLATE, force)
}

#[derive(Debug)]
pub struct UpdateOutcome {
    pub updated: Vec<String>,
    pub pruned: Vec<String>,
    /// 專案根相對路徑：這次同步剝除掉遺留 SPECLINK 區塊的指令檔（design D2）。
    pub stripped: Vec<String>,
    /// 棄用提示（design D3）：非錯誤、不影響 exit code，CLI 走 stderr。
    pub deprecations: Vec<String>,
    pub notes: Vec<String>,
}

/// The tool selection `.speclink.yaml` records, resolved ONCE: built-in names
/// deduplicated, descriptors validated, and the legacy "no tools list" fallback
/// decided. Every consumer of the tools list reads this — `update`, `probe_assets`,
/// `reconcile_builtin_tools` and the desktop checkout preselection.
///
/// A descriptor problem is carried on the value instead of raised: `update` and
/// `reconcile` turn it into an error before any write (bad descriptor ⇒ zero writes),
/// while `probe_assets` reads built-ins only and must not change its verdict because
/// a descriptor is broken.
#[derive(Debug)]
pub struct ToolSelection {
    /// Built-in tools in list order, deduplicated.
    pub builtins: Vec<Tool>,
    /// Validated descriptors in list order.
    pub customs: Vec<CustomTool>,
    /// Warnings that are not errors: unknown built-in names.
    pub notes: Vec<String>,
    /// The first invalid or duplicate descriptor, as a single-line message.
    pub descriptor_error: Option<String>,
    /// True when `.speclink.yaml` records no tools list at all — `builtins` then comes
    /// from directory detection, not from the file.
    pub legacy_fallback: bool,
}

impl ToolSelection {
    /// Resolve the selection from a loaded `.speclink.yaml`. Without a tools list the
    /// legacy rule applies: regenerate Claude when `.claude` exists, codex excluded.
    pub fn resolve(root: &Path, app: &crate::config::AppConfig) -> ToolSelection {
        let mut sel = ToolSelection {
            builtins: Vec::new(),
            customs: Vec::new(),
            notes: Vec::new(),
            descriptor_error: None,
            legacy_fallback: app.tools.is_empty(),
        };
        for entry in &app.tools {
            match entry {
                ToolEntry::Builtin(name) => match Tool::parse(name) {
                    Some(t) => {
                        if !sel.builtins.contains(&t) {
                            sel.builtins.push(t);
                        }
                    }
                    None => sel.notes.push(format!(
                        "unknown tool '{name}' in .speclink.yaml tools list (supported: claude, codex)"
                    )),
                },
                ToolEntry::Descriptor(d) => {
                    // Keep the FIRST problem: it is the one today's update reports.
                    if sel.descriptor_error.is_some() {
                        continue;
                    }
                    match d.validate() {
                        Ok(custom) => {
                            if sel.customs.iter().any(|c| c.name == custom.name) {
                                sel.descriptor_error =
                                    Some(format!("tool descriptor: duplicate name '{}'", custom.name));
                            } else {
                                sel.customs.push(custom);
                            }
                        }
                        Err(message) => sel.descriptor_error = Some(message),
                    }
                }
            }
        }
        if sel.legacy_fallback && root.join(".claude").is_dir() {
            sel.builtins.push(Tool::Claude);
        }
        sel
    }

    /// Build a selection straight from an in-memory built-in list — the entry `init`,
    /// `init_remote` and `reconcile_builtin_tools` use, where the selection is the
    /// caller's argument rather than a file.
    pub fn builtins_only(tools: &[Tool]) -> ToolSelection {
        ToolSelection {
            builtins: tools.to_vec(),
            customs: Vec::new(),
            notes: Vec::new(),
            descriptor_error: None,
            legacy_fallback: false,
        }
    }
}

/// The managed skill set of ONE render target: `speclink-<name>` directory → SKILL.md
/// content. This is the only producer of that set — generation, orphan cleanup, the
/// staleness diff and the guard all read it through [`SyncPlan`].
///
/// Non-Claude targets get the `for_codex` subset; worktree-gated skills need the policy on.
pub(crate) fn managed_skills(
    target: skills::RenderTarget,
    worktree_on: bool,
    spec_dir: &str,
) -> Vec<(String, String)> {
    let codex_subset = !matches!(target, skills::RenderTarget::Builtin(Tool::Claude));
    skills::registry()
        .into_iter()
        .filter(|s| !codex_subset || s.for_codex)
        .filter(|s| !s.worktree_gated || worktree_on)
        .map(|s| {
            let content = skills::render_skill_file_for(target, &s, spec_dir);
            (format!("speclink-{}", s.name), content)
        })
        .collect()
}

/// What a sync target is: a built-in tool, or a descriptor named by its `name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SyncTargetKind {
    Builtin(Tool),
    Custom(String),
}

/// One tool's slice of a sync: where its skills live, and what should be there.
#[derive(Debug)]
pub(crate) struct SyncTarget {
    pub(crate) kind: SyncTargetKind,
    /// The name that appears in [`UpdateOutcome`]'s lists and the CLI output.
    pub(crate) label: String,
    /// Project-root-relative skills directory, `/`-joined (`.claude/skills`,
    /// a descriptor's `skills_dir`) — the form the probe reports paths in.
    pub(crate) skills_dir: String,
    /// The same directory as an absolute path — what the guard and the writers use.
    pub(crate) skills_root: PathBuf,
    /// `speclink-<name>` directory → SKILL.md content.
    pub(crate) files: Vec<(String, String)>,
}

/// One workspace sync, resolved once and consumed by every station: the downgrade guard,
/// generation, pruning and the staleness probe all read THIS. The guard's target set and
/// the write set are the same `targets` field, so they cannot drift apart.
///
/// Sharp edges (audit, change workspace-sync-plan):
/// - A descriptor whose `skills_dir` escapes the project root never becomes a target —
///   `ToolDescriptor::validate` rejects it before [`ToolSelection`] carries it.
/// - `apply`'s orphan cleanup removes `speclink-` prefixed directories only, so a user's
///   own skill directories under the same root survive.
/// - `guard` has exactly one bypass, `update`'s explicit `--allow-downgrade`; every other
///   regeneration path calls it.
pub(crate) struct SyncPlan {
    /// Ordered claude, codex, then descriptors in list order.
    pub(crate) targets: Vec<SyncTarget>,
    /// Built-ins that are NOT selected and lose their footprint. Empty on the legacy
    /// fallback: without a tools list, nothing was ever "deselected".
    pub(crate) deselected_builtins: Vec<Tool>,
    /// Descriptors that still declare the deprecated `instructions_file`: the file to
    /// strip a legacy marker from, and the deprecation notice to report.
    pub(crate) custom_strip_targets: Vec<(String, String)>,
    pub(crate) selection: ToolSelection,
    /// The worktree policy this plan was built under (read once, in `resolve`).
    worktree_on: bool,
}

impl SyncPlan {
    /// Build the plan. This is the ONE place a sync reads the worktree policy.
    pub(crate) fn resolve(root: &Path, selection: ToolSelection, spec_dir: &str) -> SyncPlan {
        let worktree_on = worktree_skills_enabled(root, spec_dir);
        let mut targets = Vec::new();
        for tool in [Tool::Claude, Tool::Codex] {
            if selection.builtins.contains(&tool) {
                targets.push(SyncTarget {
                    kind: SyncTargetKind::Builtin(tool),
                    label: tool.name().to_string(),
                    skills_dir: tool.skills_dir().to_string(),
                    skills_root: root.join(tool.skills_dir()),
                    files: managed_skills(
                        skills::RenderTarget::Builtin(tool),
                        worktree_on,
                        spec_dir,
                    ),
                });
            }
        }
        let mut custom_strip_targets = Vec::new();
        for custom in &selection.customs {
            targets.push(SyncTarget {
                kind: SyncTargetKind::Custom(custom.name.clone()),
                label: custom.name.clone(),
                skills_dir: custom.skills_dir.clone(),
                skills_root: root.join(&custom.skills_dir),
                files: managed_skills(
                    skills::RenderTarget::Custom(custom),
                    worktree_on,
                    spec_dir,
                ),
            });
            if let Some(file) = custom.instructions_file.as_deref() {
                custom_strip_targets.push((
                    file.to_string(),
                    format!(
                        "tool descriptor '{}': instructions_file is deprecated and no longer generates anything — remove it from .speclink.yaml",
                        custom.name
                    ),
                ));
            }
        }
        let deselected_builtins = if selection.legacy_fallback {
            Vec::new()
        } else {
            [Tool::Claude, Tool::Codex]
                .into_iter()
                .filter(|t| !selection.builtins.contains(t))
                .collect()
        };
        SyncPlan {
            targets,
            deselected_builtins,
            custom_strip_targets,
            selection,
            worktree_on,
        }
    }

    /// Downgrade guard: refuse when any target's skills directory leads this engine.
    /// The checked set IS the write set.
    pub(crate) fn guard(&self) -> Result<()> {
        let dirs: Vec<PathBuf> = self.targets.iter().map(|t| t.skills_root.clone()).collect();
        refuse_downgrade(&dirs)
    }

    /// The built-in targets' managed files that are absent or differ from the current
    /// render (line endings normalized), as project-root-relative `/`-joined paths.
    /// Built-ins only because the staleness probe — its one consumer — does not cover
    /// descriptors yet.
    pub(crate) fn differing_files(&self) -> Vec<String> {
        let mut differing = Vec::new();
        for target in &self.targets {
            if !matches!(target.kind, SyncTargetKind::Builtin(_)) {
                continue;
            }
            for (dir, expected) in &target.files {
                let actual =
                    std::fs::read_to_string(target.skills_root.join(dir).join("SKILL.md"))
                        .unwrap_or_default();
                if eol_normalized(&actual) != eol_normalized(expected) {
                    differing.push(format!("{}/{dir}/SKILL.md", target.skills_dir));
                }
            }
        }
        differing
    }

    /// `update`'s writer. Order (any step's `Err` stops the run; what is written stays —
    /// every step is idempotent, so a rerun converges):
    ///
    /// 1. Strip legacy `SPECLINK:START..END` blocks from the two built-in instruction
    ///    files and from every descriptor that still declares `instructions_file`.
    /// 2. Delete the footprints of descriptors that fell off the list. This happens
    ///    BEFORE generation on purpose: a descriptor that only changed its NAME keeps
    ///    the same `skills_dir`, and deleting after writing would take the fresh files
    ///    with it. Reporting stays late so `pruned` keeps its built-ins-first order.
    /// 3. Per target (claude, codex, then descriptors): write every managed file and
    ///    remove the `speclink-` directories that are not in the set.
    /// 4. Prune the deselected built-ins.
    /// 5. Record the current descriptors as the footprint for the next sync.
    pub(crate) fn apply(&self, root: &Path) -> Result<UpdateOutcome> {
        let mut out = UpdateOutcome {
            updated: Vec::new(),
            pruned: Vec::new(),
            stripped: Vec::new(),
            deprecations: Vec::new(),
            notes: self.selection.notes.clone(),
        };

        // 1. 遺留剝除——內建工具唯一的剝除點（prune_tool 只清技能足跡）。
        for tool in [Tool::Claude, Tool::Codex] {
            let rel = instructions_path(tool);
            if strip_legacy_marker(&root.join(rel))? {
                out.stripped.push(rel.to_string());
            }
        }
        for (file, deprecation) in &self.custom_strip_targets {
            out.deprecations.push(deprecation.clone());
            if strip_legacy_marker(&root.join(file))? {
                out.stripped.push(file.clone());
            }
        }

        // 2. 舊足跡。判定只看 name 與 skills_dir：instructions_file 已棄用，照提示把它
        //    移除不得被誤判「已下架」。
        let mut pruned_customs = Vec::new();
        for old in load_custom_state(root) {
            let still_current = self
                .selection
                .customs
                .iter()
                .any(|c| c.name == old.name && c.skills_dir == old.skills_dir);
            if !still_current && prune_custom(root, &old, &mut out.notes)? {
                pruned_customs.push(old.name.clone());
            }
        }

        for target in &self.targets {
            for (dir, content) in &target.files {
                util::write_file(&target.skills_root.join(dir).join("SKILL.md"), content)?;
            }
            let expected: Vec<String> = target.files.iter().map(|(dir, _)| dir.clone()).collect();
            prune_orphan_skills(&target.skills_root, &expected)?;
            out.updated.push(target.label.clone());
        }

        for tool in &self.deselected_builtins {
            if prune_tool(root, *tool)? {
                out.pruned.push(tool.name().to_string());
            }
        }

        out.pruned.extend(pruned_customs);
        save_custom_state(root, &self.selection.customs)?;

        Ok(out)
    }

    /// `init`'s writer: every target's files, no descriptor state touched, and no orphan
    /// cleanup — `init` stays conservative over an existing workspace. The one removal it
    /// does make: under an OFF worktree policy, the two gated skill directories a
    /// previous policy-on generation left behind (`init --force` resets the policy to
    /// the template, and the tool must not keep loading a skill the policy disabled).
    pub(crate) fn write_skills(&self, force: bool) -> Result<()> {
        for target in &self.targets {
            for (dir, content) in &target.files {
                write_if(&target.skills_root.join(dir).join("SKILL.md"), content, force)?;
            }
            if self.worktree_on {
                continue;
            }
            for skill in skills::registry().iter().filter(|s| s.worktree_gated) {
                let dir = target.skills_root.join(format!("speclink-{}", skill.name));
                if dir.is_dir() {
                    std::fs::remove_dir_all(dir)?;
                }
            }
        }
        Ok(())
    }
}

/// Refresh generated skill files and strip legacy instruction-file markers.
///
/// When `.speclink.yaml` records a `tools:` list, this is a full sync: every listed tool
/// (built-in name or custom descriptor) has its skills regenerated and generated files for
/// tools NOT on the list are pruned (speclink-* skill dirs removed). Unknown built-in names
/// produce a warning note; an invalid descriptor is an error. Without a recorded list,
/// built-ins fall back to legacy behavior: regenerate the tools whose dot-directories exist
/// (codex excluded).
///
/// Legacy stripping (change remove-marker-injection, design D2): instruction files are no
/// longer part of the managed set, so every sync strips the `SPECLINK:START..END` block an
/// older engine injected — user content outside the block survives, a file left empty is
/// deleted, a file without a block is not touched at all.
///
/// Downgrade guard (change instruction-downgrade-guard): before any write, the skill files
/// this call is ABOUT to regenerate are checked for direction — a frontmatter version
/// leading this engine means regenerating would silently rewrite them back to older content,
/// the 2026-08-05 incident. The check is sourced from the write set itself (tools list,
/// legacy directory detection, custom descriptors), not from the builtin-only probe, so no
/// regeneration corner escapes it. Every regeneration path (CLI update, workflow-config sync
/// on both CLI and desktop, the desktop update entry, tool reconciliation) funnels through
/// here, so the guard lives here and nowhere else; `allow_downgrade` is the single explicit
/// override.
pub fn update(root: &Path, allow_downgrade: bool) -> Result<UpdateOutcome> {
    let app = crate::config::AppConfig::load(&root.join(".speclink.yaml"))?;
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let selection = ToolSelection::resolve(root, &app);
    // 壞描述子＝零寫入：錯誤在任何檔案動作之前轉出來。
    if let Some(message) = &selection.descriptor_error {
        bail!("{message}");
    }
    let plan = SyncPlan::resolve(root, selection, &spec_dir);
    if !allow_downgrade {
        plan.guard()?;
    }
    plan.apply(root)
}

/// Converge a workspace on `tools` as the COMPLETE desired state of its built-ins —
/// the single entry point CLI init, remote init and the desktop share.
///
/// Two steps, both existing behavior: `.speclink.yaml`'s claude/codex entries are
/// rewritten to match the selection (custom descriptors, remote, spec_dir and unknown
/// keys carry over untouched), then [`update`] generates the selected tools' skills,
/// prunes the deselected ones and strips any legacy instruction-file marker.
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
    // 選集與計畫都由改寫後的文字建立：守門目標與其後的寫入集同源，拒絕＝整體
    // 零寫入，不留「config 已改、受管檔未同步」的半狀態。
    let app: crate::config::AppConfig = crate::config::parse_lenient_or_reason(&rewritten)
        .map_err(|reason| anyhow::anyhow!("invalid .speclink.yaml: {reason}"))?;
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let selection = ToolSelection::resolve(root, &app);
    if let Some(message) = &selection.descriptor_error {
        bail!("{message}");
    }
    let plan = SyncPlan::resolve(root, selection, &spec_dir);
    plan.guard()?;
    util::write_file(&path, &rewritten)?;
    plan.apply(root)
}

/// Adopt speclink in a directory that already has an `openspec/` tree but no
/// `.speclink.yaml` — the workspace backfill entry (change: desktop-enable-speclink-prompt,
/// 決策 2). Composes [`store_init`]'s idempotent skeleton fill (directories via
/// create_dir_all; the workflow-config template only when config.yaml is absent — an
/// existing file with user policy is never touched) with [`reconcile_builtin_tools`]
/// (tools recorded in `.speclink.yaml`, managed skills regenerated).
/// Deliberately NOT behind `init`'s "Already initialized" guard; spec_dir is fixed to
/// `openspec` — without a `.speclink.yaml`, discovery's fallback is exactly that.
/// An empty selection is rejected before anything is written.
///
/// `.gitignore` is covered here explicitly: the `reconcile_builtin_tools` → `update`
/// path does not touch it (only `init` itself does), so without this the
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
    /// Deprecated and optional — kept so a descriptor that still names one keeps a
    /// prune/strip target after it falls off the tools list.
    #[serde(default)]
    instructions_file: Option<String>,
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
/// three-layer policy resolution: injection is the project's persistent state, while
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

/// Prune a recorded custom footprint. Paths are re-checked against the project root before
/// any removal — a tampered state file must not be able to delete outside the project.
fn prune_custom(root: &Path, fp: &CustomFootprint, notes: &mut Vec<String>) -> Result<bool> {
    if !crate::config::is_project_relative(&fp.skills_dir)
        || fp
            .instructions_file
            .as_deref()
            .is_some_and(|f| !crate::config::is_project_relative(f))
    {
        notes.push(format!(
            "skipped pruning tool '{}': recorded paths escape the project root",
            fp.name
        ));
        return Ok(false);
    }
    prune_footprint(
        &root.join(&fp.skills_dir),
        fp.instructions_file.as_ref().map(|f| root.join(f)).as_deref(),
    )
}

/// update 的孤兒清理（spec: update 清除孤兒技能目錄）：清掉 skills 目錄下
/// speclink- 前綴、不在本次應生成集合的目錄——改名或下架的技能不留舊目錄。
/// 前綴即所有權，與 prune_footprint 同一判準；非前綴的使用者目錄不動。
/// 只掛 update：init 對既有工作區維持保守，不清理。
fn prune_orphan_skills(skills_root: &Path, expected: &[String]) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(skills_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("speclink-")
                && !expected.iter().any(|e| e == &name)
                && entry.path().is_dir()
            {
                std::fs::remove_dir_all(entry.path())?;
            }
        }
    }
    Ok(())
}

/// Remove the generated artifacts of a deselected built-in tool.
fn prune_tool(root: &Path, tool: Tool) -> Result<bool> {
    // 指令檔的遺留剝除已由 update() 對兩個內建工具無條件跑過（選取與否都剝），
    // 這裡只清技能足跡——同一檔案不需要第二個剝除點。
    prune_footprint(&root.join(tool.skills_dir()), None)
}

/// Remove a generated footprint: speclink-* skill directories and any legacy SPECLINK
/// marker block left in the instruction file. Returns whether anything was removed.
fn prune_footprint(skills_root: &Path, md: Option<&Path>) -> Result<bool> {
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
    if let Some(md) = md {
        if strip_legacy_marker(md)? {
            removed = true;
        }
    }
    Ok(removed)
}

/// Strip a legacy `SPECLINK:START..END` block from an instruction file (design D2):
/// only the block and its separating blank line go, user content outside it survives,
/// a file left empty is deleted, and a file WITHOUT a block is not written at all
/// (byte-identical). Returns whether anything was stripped.
fn strip_legacy_marker(md: &Path) -> Result<bool> {
    let Some(text) = util::read_opt(md) else {
        return Ok(false);
    };
    if !text.contains("<!-- SPECLINK:START") {
        return Ok(false);
    }
    let stripped = strip_marker(&text);
    if stripped == text {
        // 不成對的 START：strip_marker 原樣退回，不動檔案也不列入剝除摘要。
        return Ok(false);
    }
    if stripped.trim().is_empty() {
        std::fs::remove_file(md)?;
    } else {
        util::write_file(md, &stripped)?;
    }
    Ok(true)
}

/// Remove every paired SPECLINK:START..END block (plus the blank line each was
/// separated by). An unpaired START — the END line hand-deleted or mangled by a
/// merge — returns the text unchanged: eating everything after START would be
/// silent data loss, and a stray block of dead text is the safer failure.
fn strip_marker(text: &str) -> String {
    let start = "<!-- SPECLINK:START";
    let end = "<!-- SPECLINK:END -->";
    let mut out = text.to_string();
    while let Some(s) = out.find(start) {
        let Some(e) = out[s..].find(end).map(|i| s + i + end.len()) else {
            // 不成對：整段放棄，已剝掉的前段（若有）保留——每一段都是獨立成對判定。
            break;
        };
        let after = out[e..].trim_start_matches(|c| c == '\n' || c == '\r');
        out = format!("{}{}", &out[..s], after);
    }
    out
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

/// 技能檔過期探測的整體判定（規格「技能檔過期探測」五態）。聚合優先序
/// 較新 > 缺失 > 過期 > 現版：較新排最前，只要有任何檔案領先引擎，就不提供
/// 任何會改寫它的動作；缺失優先於過期，因為「從未安裝」與「裝了但舊了」是不同
/// 的使用者情境，提示文案據此分流。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetStatus {
    /// tools 清單宣告的工具其 skills 目錄下無任何 speclink- 技能檔＝從未安裝
    /// 或整組移除（如 clone 後技能未進版控）。
    Missing,
    /// 任一工具的技能版號與現版不等且不領先現版。
    Stale,
    /// 任一工具的技能版號數值新於現版＝工作區檔案領先引擎（本體是舊版）。
    Newer,
    Current,
    /// 設定解析失敗或技能檔存在但讀取錯誤——不得與現版混同。
    Unknown,
}

/// 單一內建工具的探測結果。`workspaceVersion` 為 None 代表技能整組不在（`missing`
/// 為真），或技能檔在但 frontmatter 讀不到版本行。`stale` 與 `newer` 互斥：方向
/// 由引擎判定，消費端不重算。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAssetState {
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
pub struct AssetProbe {
    pub status: AssetStatus,
    pub current_version: String,
    pub tools: Vec<ToolAssetState>,
    pub differing_files: Vec<String>,
}

/// 讀取技能檔 frontmatter 的版本欄位（`  version: "v1.2.3"`）；讀不到回 None。
/// 搜尋範圍限定 frontmatter 本體：第一行的 `---` 到下一個 `---` 之間，body 裡
/// 恰好叫 version: 的內文行不會被誤認。
fn skill_version_of(text: &str) -> Option<&str> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let line = lines
        .take_while(|l| l.trim() != "---")
        .find(|l| l.trim_start().starts_with("version:"))?;
    let value = line.split_once("version:")?.1.trim().trim_matches('"');
    (!value.is_empty()).then_some(value)
}

/// 一個工具的 skills 目錄探測結果——三態，因為「整組不在」「檔在讀不出」
/// 「讀到了」對應到三種不同的回報狀態，混同任兩者都會誤導提示層。
enum SkillsProbe {
    /// 目錄下無任何 speclink- 技能檔（目錄不存在也算：從未安裝或整組移除）。
    Absent,
    /// 找到技能檔且讀得到 frontmatter 版號。
    Found(String),
    /// 技能檔存在但讀不出版號（IO 失敗、目錄不可列舉、frontmatter 版本行遺失）
    /// ——「無法判定」，不得與現版或缺失混同。
    Unreadable,
}

/// 取一個 skills 目錄的產物層版號。同一次生成的所有技能檔帶相同版號（規格
///「產物層版本戳同源」），但工作區可能是半手動狀態，所以整個目錄逐份掃描而不
/// 抽樣一份：守門的契約是「即將被改寫的技能檔中任一檔領先即拒絕」，抽樣會漏。
/// 回傳最能代表方向的版號——任一檔領先引擎即回該檔（領先優先），否則回第一個
/// 與現版不等的版號，全部現版時回現版。目錄項目排序後掃描，結果與檔案系統的
/// 列舉順序無關。
fn probe_skills_dir(skills_root: &Path) -> SkillsProbe {
    let entries = match std::fs::read_dir(skills_root) {
        Ok(entries) => entries,
        // 目錄不存在＝從未安裝；其他失敗（權限、路徑是檔案）＝無法判定。
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return SkillsProbe::Absent,
        Err(_) => return SkillsProbe::Unreadable,
    };
    let mut names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("speclink-"))
        .collect();
    names.sort();
    let mut found_any = false;
    let mut differing: Option<String> = None;
    let mut current: Option<String> = None;
    for name in names {
        let file = skills_root.join(&name).join("SKILL.md");
        if !file.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            return SkillsProbe::Unreadable;
        };
        // 檔在但版本行讀不到＝壞 frontmatter：回報成現版會讓它永遠不被修復。
        let Some(v) = skill_version_of(&text).map(str::to_string) else {
            return SkillsProbe::Unreadable;
        };
        found_any = true;
        if workspace_is_newer(&v, ASSET_VERSION) {
            return SkillsProbe::Found(v);
        }
        if v != ASSET_VERSION {
            differing.get_or_insert(v);
        } else {
            current.get_or_insert(v);
        }
    }
    if !found_any {
        return SkillsProbe::Absent;
    }
    SkillsProbe::Found(differing.or(current).expect("found_any implies a version"))
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

/// 工作區版號是否數值領先引擎版號（決策 3）：段數不足補零後逐段比較。
/// 任一邊無法完整解析為數字段時回 false——手改壞的 frontmatter 寧可誤報過期
///（改寫即恢復受管狀態），不可誤報較新（那會封鎖 update）。
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

/// 降級守門的拒絕判定：`dirs` 中任一 skills 目錄的技能版號數值領先引擎時，以單行
/// 英文錯誤拒絕（含兩邊版號）。技能不在、或版號讀不出來的目錄不擋——守門只在
/// 「可證明較新」時觸發（決策 3 的安全預設）。
fn refuse_downgrade(dirs: &[PathBuf]) -> Result<()> {
    for dir in dirs {
        if let SkillsProbe::Found(v) = probe_skills_dir(dir) {
            if workspace_is_newer(&v, ASSET_VERSION) {
                bail!(
                    "Workspace skill files ({v}) are newer than this engine ({ASSET_VERSION}). \
                     Update Speclink first, or run `speclink update --allow-downgrade` to rewrite them anyway."
                );
            }
        }
    }
    Ok(())
}

/// 唯讀的技能檔過期探測（規格「技能檔過期探測」）：依 `.speclink.yaml` 的 tools
/// 清單（與 [`update`] 同一資料源）讀各內建工具 skills 目錄下技能檔 frontmatter 的
/// 版本欄位，對 [`ASSET_VERSION`] 判方向——數值領先現版為較新，其餘退回字串相等
/// 判定（不等即過期）。方向是唯一判準來源：desktop 與 CLI 共用同一裁決，領先時
/// 兩個出口都不提供改寫動作。
///
/// 零寫入：desktop 開專案時搭載，探測失敗不得阻斷開啟。自訂描述子第一版不涵蓋
/// （回報結構已預留 tool 名欄位，納入時不需改形狀）。
pub fn probe_assets(root: &Path) -> AssetProbe {
    let without_tools = |status: AssetStatus| AssetProbe {
        status,
        current_version: ASSET_VERSION.to_string(),
        tools: Vec::new(),
        differing_files: Vec::new(),
    };
    let Ok(app) = crate::config::AppConfig::load(&root.join(".speclink.yaml")) else {
        return without_tools(AssetStatus::Unknown);
    };
    let spec_dir = app.spec_dir.clone().unwrap_or_else(|| "openspec".to_string());
    let selection = ToolSelection::resolve(root, &app);
    // 探測只讀 tools 清單宣告的內建工具：沒有清單就沒有受管工具可查（目錄偵測的回退
    // 是 update 的再生規則，不是探測的判定面）。描述子錯誤同樣不影響結果。
    if selection.legacy_fallback {
        return without_tools(AssetStatus::Current);
    }
    let plan = SyncPlan::resolve(root, selection, &spec_dir);

    let mut tools = Vec::new();
    for target in &plan.targets {
        let SyncTargetKind::Builtin(tool) = &target.kind else {
            continue;
        };
        let version = match probe_skills_dir(&target.skills_root) {
            SkillsProbe::Absent => {
                tools.push(ToolAssetState {
                    tool: tool.name().to_string(),
                    workspace_version: None,
                    stale: false,
                    newer: false,
                    missing: true,
                });
                continue;
            }
            // 技能檔在但讀不出版號（IO、壞 frontmatter）＝無法判定；
            // 與「不存在」是不同的狀態。
            SkillsProbe::Unreadable => return without_tools(AssetStatus::Unknown),
            SkillsProbe::Found(version) => version,
        };
        // 方向優先於相等判定：領先現版的版號是「較新」，不得再算成過期。
        let newer = workspace_is_newer(&version, ASSET_VERSION);
        tools.push(ToolAssetState {
            tool: tool.name().to_string(),
            stale: !newer && version != ASSET_VERSION,
            newer,
            missing: false,
            workspace_version: Some(version),
        });
    }

    let status = if tools.iter().any(|t| t.newer) {
        AssetStatus::Newer
    } else if tools.iter().any(|t| t.missing) {
        AssetStatus::Missing
    } else if tools.iter().any(|t| t.stale) {
        AssetStatus::Stale
    } else {
        AssetStatus::Current
    };
    let differing_files = match status {
        AssetStatus::Missing | AssetStatus::Stale | AssetStatus::Newer => plan.differing_files(),
        _ => Vec::new(),
    };

    AssetProbe {
        status,
        current_version: ASSET_VERSION.to_string(),
        tools,
        differing_files,
    }
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
    /// descriptor、remote section 與未知頂層鍵；兩份指令檔先有使用者自有文字。
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
        update(&root.dir, false).expect("seed update");
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
            // Spec Scenario 的 GIVEN：指令檔「同時含遺留 Speclink 區塊和使用者文字」
            // ——seed 的 update 已把區塊剝掉，reconcile 前重新注入，讓收斂路徑自己剝。
            for tool in [Tool::Claude, Tool::Codex] {
                root.write(
                    instructions_file(tool),
                    &format!(
                        "<!-- SPECLINK:START v1.0.0 -->\n舊路由表\n<!-- SPECLINK:END -->\n{}\n",
                        user_text(tool)
                    ),
                );
            }

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
                    assert!(root.exists(&skill), "row {i}: {skill} 應被補齊");
                } else {
                    assert!(!root.exists(&skill), "row {i}: {skill} 應被清理");
                }
                // 指令檔已退出受管：不論選取與否都只剩使用者自己的內容。
                assert_eq!(marker_count(&text), 0, "row {i}: {md} 不得帶受管區塊:\n{text}");
                assert_eq!(text, format!("{}\n", user_text(tool)), "row {i}: {md} 須位元級不變");
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
        assert_eq!(text, format!("{CODEX_USER_TEXT}\n"), "指令檔須位元級不變:\n{text}");
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

    /// Remote 模式不生成任何指令檔內容，也不建立本機規格樹。
    #[test]
    fn reconcile_in_remote_mode_writes_no_instruction_file_and_no_spec_tree() {
        let root = TempRoot::new("reconcile-remote-wording");
        seed_remote_workspace(&root, &[Tool::Claude]);

        reconcile_builtin_tools(&root.dir, &[Tool::Claude, Tool::Codex]).expect("reconcile succeeds");

        for tool in [Tool::Claude, Tool::Codex] {
            let md = instructions_file(tool);
            let text = root.read(md);
            assert_eq!(text, format!("{}\n", user_text(tool)), "{md} 須位元級不變:\n{text}");
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
        assert!(!root.exists("CLAUDE.md"), "工作區補齊不得產生指令檔");
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

        update(&root.dir, false).expect("update succeeds");

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
            let skill = propose_skill(tool);
            assert_eq!(converged.read(&skill), fresh.read(&skill), "{skill} 須與 init 相同");
        }
        assert!(converged.exists("openspec/specs"), "既有 filesystem 規格樹須保留");
    }

    // --- 遺留 marker 剝除（design D2；規格「built-in tools 權威收斂」
    // 「描述子的同步與清理生命週期」） ---

    /// 舊版引擎注入過的指令檔：marker 區塊在上，使用者段落在下。
    fn legacy_marker_file(user_text: &str) -> String {
        format!(
            "<!-- SPECLINK:START v1.0.0 -->\n\n# Speclink Instructions\n\n舊版注入的路由表。\n\n<!-- SPECLINK:END -->\n{user_text}"
        )
    }

    #[test]
    fn update_strips_a_legacy_marker_and_keeps_user_content() {
        // Scenario「更新時剝除內建工具的遺留 marker」：區塊消失、使用者段落原樣
        // 保留，摘要列出被剝除的檔案，技能檔照常再生。
        let root = TempRoot::new("strip-keeps-user");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write("CLAUDE.md", &legacy_marker_file(CLAUDE_USER_TEXT));

        let out = update(&root.dir, false).expect("update succeeds");

        let text = root.read("CLAUDE.md");
        assert!(!text.contains("<!-- SPECLINK:START"), "區塊須被剝除:\n{text}");
        assert_eq!(text, CLAUDE_USER_TEXT, "使用者段落須原樣保留:\n{text}");
        assert_eq!(out.stripped, vec!["CLAUDE.md".to_string()], "摘要須列出剝除的檔案");
        assert!(root.exists(&propose_skill(Tool::Claude)), "技能檔照常再生");
    }

    #[test]
    fn update_deletes_an_instruction_file_that_was_only_a_marker() {
        // 剝除後全空的檔案整份刪除——不留一個空殼在專案根。
        let root = TempRoot::new("strip-deletes-empty");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        root.write("CLAUDE.md", &legacy_marker_file(""));
        root.write("AGENTS.md", &legacy_marker_file(""));

        let out = update(&root.dir, false).expect("update succeeds");

        assert!(!root.exists("CLAUDE.md"), "純 marker 檔須刪除");
        assert!(!root.exists("AGENTS.md"), "純 marker 檔須刪除");
        assert_eq!(out.stripped, vec!["CLAUDE.md".to_string(), "AGENTS.md".to_string()]);
    }

    #[test]
    fn update_leaves_an_instruction_file_without_a_marker_byte_identical() {
        // 無區塊＝零觸碰：使用者自己寫的 CLAUDE.md 不得被 update 改動一個位元組。
        let root = TempRoot::new("strip-untouched");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let user_only = "# 我自己的 CLAUDE.md\n\n沒有任何受管區塊。\n";
        root.write("CLAUDE.md", user_only);

        let out = update(&root.dir, false).expect("update succeeds");

        assert_eq!(root.read("CLAUDE.md"), user_only, "無區塊的檔案須位元級不變");
        assert!(out.stripped.is_empty(), "沒剝除任何東西時摘要須為空：{:?}", out.stripped);
    }

    #[test]
    fn update_strips_a_descriptors_legacy_marker() {
        // Scenario「更新時剝除描述子的遺留 marker」：仍帶 instructions_file 欄位的
        // 描述子，其指令檔同受剝除語意。
        let root = TempRoot::new("strip-descriptor");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
        root.write("WAD.md", &legacy_marker_file("使用者寫在 WAD.md 的段落\n"));

        let out = update(&root.dir, false).expect("update succeeds");

        let text = root.read("WAD.md");
        assert!(!text.contains("<!-- SPECLINK:START"), "描述子的區塊須被剝除:\n{text}");
        assert_eq!(text, "使用者寫在 WAD.md 的段落\n", "使用者段落須原樣保留:\n{text}");
        assert_eq!(out.stripped, vec!["WAD.md".to_string()]);
        assert!(root.exists(".wad/skills/speclink-apply/SKILL.md"), "描述子技能檔照常生成");
    }

    #[test]
    fn init_force_over_a_legacy_workspace_strips_the_marker() {
        // design D2：對既有 marker 的專案 re-init（--force）同樣走剝除。
        let root = TempRoot::new("strip-init-force");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write("CLAUDE.md", &legacy_marker_file(CLAUDE_USER_TEXT));

        init(&root.dir, &[Tool::Claude], true, "openspec").expect("re-init succeeds");

        assert_eq!(root.read("CLAUDE.md"), CLAUDE_USER_TEXT, "區塊須被剝除、使用者段落保留");
    }

    #[test]
    fn init_on_a_fresh_project_writes_no_instruction_file() {
        // Scenario「指令檔零受管區塊」：全新目錄以兩個工具 init 後，專案根不存在
        // 任何指令檔，技能檔照常生成。
        let root = TempRoot::new("fresh-no-instructions");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();

        assert!(!root.exists("CLAUDE.md"), "不得生成 CLAUDE.md");
        assert!(!root.exists("AGENTS.md"), "不得生成 AGENTS.md");
        for tool in [Tool::Claude, Tool::Codex] {
            assert!(root.exists(&propose_skill(tool)), "{} 技能檔照常生成", tool.name());
        }
    }

    #[test]
    fn strip_leaves_a_file_with_an_unpaired_start_untouched() {
        // R1：END 行被手動刪掉或 merge conflict 弄壞時，剝除不得把 START 之後的
        // 內容吞掉——不成對就整檔不動（位元級不變），寧可留一塊死文字。
        let root = TempRoot::new("strip-unpaired");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let broken = "<!-- SPECLINK:START v1.0.0 -->\n\n舊路由表。\n\n使用者的重要內容\n";
        root.write("CLAUDE.md", broken);

        let out = update(&root.dir, false).expect("update succeeds");

        assert_eq!(root.read("CLAUDE.md"), broken, "不成對的檔案必須位元級不變");
        assert!(out.stripped.is_empty(), "不成對不算剝除：{:?}", out.stripped);
    }

    #[test]
    fn strip_removes_every_legacy_block_in_one_run() {
        // R2：多個遺留區塊（壞 merge 疊出來的）一次 update 全剝乾淨，不用跑第二次。
        let root = TempRoot::new("strip-multi");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(
            "CLAUDE.md",
            "<!-- SPECLINK:START v1.0.0 -->\nA\n<!-- SPECLINK:END -->\n中間的使用者文字\n<!-- SPECLINK:START v1.1.0 -->\nB\n<!-- SPECLINK:END -->\n結尾文字\n",
        );

        let out = update(&root.dir, false).expect("update succeeds");

        let text = root.read("CLAUDE.md");
        assert!(!text.contains("SPECLINK:START"), "兩個區塊都要剝掉:\n{text}");
        assert!(text.contains("中間的使用者文字") && text.contains("結尾文字"), "{text}");
        assert_eq!(out.stripped, vec!["CLAUDE.md".to_string()]);
    }

    #[test]
    fn strip_handles_crlf_files_without_leaving_blank_lines() {
        // R3：Windows checkout（CRLF）剝除後不得留下前導空行。
        let root = TempRoot::new("strip-crlf");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(
            "CLAUDE.md",
            "<!-- SPECLINK:START v1.0.0 -->\r\n\r\n舊路由表。\r\n\r\n<!-- SPECLINK:END -->\r\n使用者段落\r\n",
        );

        update(&root.dir, false).expect("update succeeds");

        assert_eq!(root.read("CLAUDE.md"), "使用者段落\r\n", "CRLF 分隔空行須一併消失");
    }

    // --- 技能檔過期探測（規格「技能檔過期探測」；design D6） ---

    /// 比現版領先一個主版號的版號：工作區檔案由更新的引擎生成的情境。
    fn ahead_of_current() -> String {
        let major: u64 = ASSET_VERSION
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .expect("ASSET_VERSION 主版號可解析");
        format!("v{}.0.0", major + 1)
    }

    /// 把某工具 skills 目錄下每份技能檔的 frontmatter 版號改成指定值
    ///（模擬以別版引擎生成的工作區——舊值模擬落後、新值模擬領先）。
    fn set_skill_version(root: &TempRoot, tool: Tool, version: &str) {
        crate::testkit::set_skill_version(&root.at(tool.skills_dir()), version);
    }

    #[test]
    fn skill_probe_reports_stale_and_lists_differing_files() {
        // Scenario「舊版工作區判過期並列差異檔」：技能版號舊於現版即過期。
        let root = TempRoot::new("skill-probe-stale");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_skill_version(&root, Tool::Claude, "v0.9.0");

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
        assert_eq!(probe.current_version, ASSET_VERSION);
        assert_eq!(probe.tools.len(), 1);
        assert_eq!(probe.tools[0].tool, "claude");
        assert_eq!(probe.tools[0].workspace_version.as_deref(), Some("v0.9.0"));
        assert!(probe.tools[0].stale && !probe.tools[0].missing && !probe.tools[0].newer);
        assert!(
            probe.differing_files.contains(&propose_skill(Tool::Claude)),
            "改動過的技能檔須列入差異清單：{:?}",
            probe.differing_files
        );
    }

    #[test]
    fn skill_probe_reports_newer_when_the_workspace_leads_the_engine() {
        // Scenario「工作區檔案領先引擎判較新」。
        let root = TempRoot::new("skill-probe-newer");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let ahead = ahead_of_current();
        set_skill_version(&root, Tool::Claude, &ahead);

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Newer, "{probe:?}");
        assert_eq!(probe.tools[0].workspace_version.as_deref(), Some(ahead.as_str()));
        assert!(probe.tools[0].newer && !probe.tools[0].stale && !probe.tools[0].missing);
        assert!(!probe.differing_files.is_empty(), "較新時仍須回報差異檔清單");
    }

    #[test]
    fn skill_probe_reports_missing_when_the_skills_dir_has_no_speclink_skill() {
        // Scenario「技能目錄缺少判缺失」：整組技能不在（clone 後技能未進版控），
        // 且缺失勝過另一支的過期。
        let root = TempRoot::new("skill-probe-missing");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        std::fs::remove_dir_all(root.at(Tool::Codex.skills_dir())).unwrap();
        set_skill_version(&root, Tool::Claude, "v0.9.0");

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Missing, "{probe:?}");
        let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
        assert!(codex.missing && !codex.stale, "{codex:?}");
        assert_eq!(codex.workspace_version, None);
        assert!(
            probe.differing_files.contains(&propose_skill(Tool::Codex)),
            "不存在的受管檔須列入（內容視為空）：{:?}",
            probe.differing_files
        );
    }

    #[test]
    fn skill_probe_prefers_newer_over_missing_and_stale() {
        // Scenario「較新優先於缺失與過期」。
        let root = TempRoot::new("skill-probe-newer-wins");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        set_skill_version(&root, Tool::Claude, &ahead_of_current());
        std::fs::remove_dir_all(root.at(Tool::Codex.skills_dir())).unwrap();

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Newer, "{probe:?}");
        let codex = probe.tools.iter().find(|t| t.tool == "codex").expect("codex 在列");
        assert!(codex.missing && !codex.newer, "缺失的工具不得被標成較新：{codex:?}");
    }

    #[test]
    fn skill_probe_falls_back_to_equality_for_an_unparsable_version() {
        // Scenario「無法解析的版號退回相等判定」。
        let root = TempRoot::new("skill-probe-unparsable");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_skill_version(&root, Tool::Claude, "v-not-a-version");

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
        assert!(probe.tools[0].stale && !probe.tools[0].newer, "{:?}", probe.tools[0]);
    }

    #[test]
    fn dropping_the_deprecated_instructions_file_field_does_not_prune_the_descriptor() {
        // R6：使用者照棄用提示把 instructions_file 從描述子移除——同一工具不得被
        // 誤判「已下架」而整組刪掉重建（pruned 與 updated 同列一個工具的自相矛盾）。
        let root = TempRoot::new("descriptor-drop-deprecated-field");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
        update(&root.dir, false).expect("先以帶欄位的描述子生成足跡");

        // 移除 instructions_file 欄位（skills_dir 不變）。
        root.write(
            ".speclink.yaml",
            "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n",
        );
        // 在技能目錄放一個使用者自己的檔案：整組 prune 重建會讓它消失。
        root.write(".wad/skills/my-note.md", "使用者自己的檔案\n");

        let out = update(&root.dir, false).expect("update succeeds");

        assert!(
            !out.pruned.contains(&"wad-harness".to_string()),
            "移除棄用欄位不得觸發 prune：{:?}",
            out.pruned
        );
        assert!(out.updated.contains(&"wad-harness".to_string()));
        assert_eq!(root.read(".wad/skills/my-note.md"), "使用者自己的檔案\n");
    }

    #[test]
    fn skill_version_parsing_stays_inside_the_frontmatter() {
        // skill_version_of 只認 frontmatter：body 裡恰好叫 version: 的內文行不算，
        // body 的 ---- 分隔線也不會提前截斷 frontmatter 搜尋。
        assert_eq!(
            skill_version_of("---\nname: x\nmetadata:\n  version: \"v9.9.9\"\n---\n\nbody version: \"v0.0.1\"\n"),
            Some("v9.9.9")
        );
        assert_eq!(
            skill_version_of("---\nname: x\n---\n\n----\n\nversion: \"v0.0.1\"\n"),
            None,
            "frontmatter 沒有版本行時，body 的 version 行不得被撿走"
        );
        assert_eq!(skill_version_of("no frontmatter\nversion: \"v1\"\n"), None);
    }

    #[test]
    fn skill_probe_reports_unknown_when_the_version_line_is_gone() {
        // R4：SKILL.md 在但 frontmatter 版本行遺失（手改壞）＝「技能檔存在但讀取
        // 錯誤」→ 無法判定，絕不可回報現版——那會讓壞檔永遠不被提示修復。
        let root = TempRoot::new("skill-probe-no-version");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        for entry in std::fs::read_dir(root.at(Tool::Claude.skills_dir())).unwrap().flatten() {
            let file = entry.path().join("SKILL.md");
            if file.is_file() {
                let text: String = std::fs::read_to_string(&file)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("version:"))
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&file, text).unwrap();
            }
        }

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
    }

    #[test]
    fn skill_probe_reports_unknown_when_the_skills_dir_is_unreadable() {
        // R5：read_dir 失敗（路徑是檔案、權限）不得與「從未安裝」混同——回無法
        // 判定，否則 desktop 會給一個按下去必然失敗的「安裝」動作。
        let root = TempRoot::new("skill-probe-dir-is-file");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        std::fs::remove_dir_all(root.at(".claude/skills")).unwrap();
        root.write(".claude/skills", "這是一個檔案不是目錄");

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
    }

    #[test]
    fn skill_update_refuses_a_workspace_whose_skills_lead_the_engine() {
        // 降級守門的版本來源同步改基準：領先的技能檔不得被任何再生路徑改寫。
        let root = TempRoot::new("skill-guard-refuse");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let ahead = ahead_of_current();
        set_skill_version(&root, Tool::Claude, &ahead);
        let before = snapshot(&root);

        let err = update(&root.dir, false).expect_err("領先的工作區必須被拒");
        let msg = err.to_string();
        assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "訊息須含兩版號：{msg}");
        assert_eq!(msg.lines().count(), 1, "單行錯誤：{msg}");
        assert_eq!(snapshot(&root), before, "拒絕＝零寫入");

        update(&root.dir, true).expect("明示越過後照常再生");
        assert!(
            root.read(&propose_skill(Tool::Claude)).contains(ASSET_VERSION),
            "受管檔須再生為引擎現版"
        );
    }

    #[test]
    fn workspace_version_direction_only_orders_parsable_versions() {
        // 規格「技能檔過期探測」的數值比較規則：去 v 前綴、以點拆段、逐段數值比較、
        // 段數不足補零；任一邊無法完整解析為數字段時不排序方向——寧可誤報過期，
        // 不可誤報較新（會封鎖 update）。
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
    fn init_force_refuses_a_workspace_that_leads_the_engine() {
        // `--force` 的語意是「覆蓋既有檔案」，不是「同意降級」——重新初始化同樣
        // 不得把領先的技能檔改寫回舊內容。
        let root = TempRoot::new("init-force-guard");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let ahead = ahead_of_current();
        set_skill_version(&root, Tool::Claude, &ahead);
        let before = snapshot(&root);

        let Err(err) = init(&root.dir, &[Tool::Claude], true, "openspec") else {
            panic!("領先的工作區不得被 --force 重建改寫");
        };
        let msg = err.to_string();
        assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "訊息須含兩版號：{msg}");
        assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
    }

    #[test]
    fn the_guard_covers_the_legacy_fallback_without_a_tools_list() {
        // 守門與寫入集同源：無 tools: 鍵的工作區走 .claude/ 目錄偵測再生，
        // 探測卻因選集為空判現版——這類 legacy 工作區同樣必須拒絕降級。
        let root = TempRoot::new("guard-legacy");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        // 模擬 legacy 工作區：config 沒有 tools 清單，但 .claude/ 目錄在。
        root.write(".speclink.yaml", "# Speclink application config\n");
        let ahead = ahead_of_current();
        set_skill_version(&root, Tool::Claude, &ahead);
        let before = snapshot(&root);

        let err = update(&root.dir, false).expect_err("legacy fallback 同樣必須被拒");
        let msg = err.to_string();
        assert!(msg.contains(&ahead) && msg.contains(ASSET_VERSION), "{msg}");
        assert_eq!(snapshot(&root), before, "拒絕＝零寫入");

        update(&root.dir, true).expect("明示越過照常再生");
        assert!(root.read(&propose_skill(Tool::Claude)).contains(ASSET_VERSION));
    }

    #[test]
    fn the_guard_covers_custom_descriptor_skill_files() {
        // Scenario「自訂描述子的技能檔同受守門」：只用描述子的工作區不得因判定面
        // 只收 builtin 而被降級。
        let root = TempRoot::new("guard-descriptor");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
        update(&root.dir, false).expect("先生成描述子受管檔");
        let ahead = ahead_of_current();
        let skill = root.at(".wad/skills/speclink-propose/SKILL.md");
        let text = std::fs::read_to_string(&skill).unwrap().replace(ASSET_VERSION, &ahead);
        std::fs::write(&skill, text).unwrap();
        let before = snapshot(&root);

        let err = update(&root.dir, false).expect_err("領先的描述子技能檔必須被拒");
        assert!(err.to_string().contains(&ahead), "{err}");
        assert_eq!(snapshot(&root), before, "拒絕＝零寫入");
    }

    #[test]
    fn reconcile_refuses_a_leading_workspace_before_touching_the_config() {
        // 方向檢查在 .speclink.yaml 寫入之前：拒絕＝整體零寫入，不留
        // 「config 已改、受管檔未同步」的半狀態。
        let root = TempRoot::new("reconcile-guard");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_skill_version(&root, Tool::Claude, &ahead_of_current());
        let before = snapshot(&root);

        let err = reconcile_builtin_tools(&root.dir, &[Tool::Claude, Tool::Codex])
            .expect_err("領先的工作區必須在寫入 config 前被拒");
        assert!(err.to_string().contains(ASSET_VERSION), "{err}");
        assert_eq!(snapshot(&root), before, "拒絕＝零寫入（含 .speclink.yaml）");
    }

    #[test]
    fn probe_reports_current_for_a_freshly_generated_workspace() {
        // Scenario「現版工作區不過期」：差異清單為空。
        let root = TempRoot::new("probe-current");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
        assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
        assert!(probe.tools.iter().all(|t| !t.stale && !t.missing));
    }

    #[test]
    fn probe_reports_unknown_for_a_malformed_config() {
        // Scenario「設定損壞回報無法判定」：不得與現版或過期混同。
        let root = TempRoot::new("probe-badconfig");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", "tools: [\n");

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Unknown, "{probe:?}");
        assert!(probe.tools.is_empty(), "{:?}", probe.tools);
        assert!(probe.differing_files.is_empty(), "{:?}", probe.differing_files);
    }

    #[test]
    fn probe_ignores_line_ending_differences() {
        // Scenario「換行差異不誤報」：CRLF 工作區（Windows core.autocrlf）僅換行
        // 形式不同的檔案不得列入差異清單。
        let root = TempRoot::new("probe-crlf");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        // 只讓一份技能檔落後（探測依此判過期），另一份維持現版內容但改成 CRLF。
        let stale = ".claude/skills/speclink-analyze/SKILL.md";
        root.write(stale, &root.read(stale).replace(ASSET_VERSION, "v0.9.0"));
        let skill = propose_skill(Tool::Claude);
        let crlf = root.read(&skill).replace('\n', "\r\n");
        root.write(&skill, &crlf);

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Stale, "{probe:?}");
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

        let probe = probe_assets(&root.dir);
        assert_eq!(probe.status, AssetStatus::Current, "{probe:?}");
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

        update(&root.dir, false).unwrap();

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

        update(&root.dir, false).unwrap();

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

        update(&root.dir, false).unwrap();
        for dir in worktree_skill_dirs(Tool::Codex) {
            assert!(!root.exists(&dir), "政策關閉時 codex 不得生成 {dir}");
        }
        assert!(!root.exists(".wad/skills/speclink-apply-with-worktree"), "描述子亦受閘控");
        assert!(root.exists(".wad/skills/speclink-apply"), "描述子的非閘控技能照常生成");

        set_worktree_policy(&root, true);
        update(&root.dir, false).unwrap();
        for dir in worktree_skill_dirs(Tool::Codex) {
            assert!(root.exists(&dir), "政策開啟時 codex 須生成 {dir}");
        }
        assert!(root.exists(".wad/skills/speclink-apply-with-worktree/SKILL.md"));
    }

    #[test]
    fn an_unparseable_workflow_config_keeps_the_worktree_skills() {
        // 刪除是不可逆方向：政策讀不出來時（使用者手改壞了 config.yaml）一律保留
        // 技能，由技能內的執行期政策檢查兜底，絕不以「讀不到＝關」為由清掉檔案。
        let root = TempRoot::new("gate-broken-config");
        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
        set_worktree_policy(&root, true);
        update(&root.dir, false).unwrap();

        root.write("openspec/config.yaml", "schema: [unterminated\n");
        update(&root.dir, false).unwrap();

        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(root.exists(&dir), "政策文件壞掉時不得清掉 {dir}");
        }
    }

    // --- update 清除孤兒技能目錄 ---
    // Spec requirement: workspace-tools「update 清除孤兒技能目錄」——speclink- 前綴
    // 且不在本次應生成集合的目錄於 update 時清除；非前綴目錄不動。

    #[test]
    fn update_prunes_renamed_skill_directory() {
        let root = TempRoot::new("prune-renamed");
        init(&root.dir, &[Tool::Claude, Tool::Codex], false, "openspec").unwrap();
        // 舊版生成的目錄：registry 已無此技能名（onboard → baseline 改名遷移）。
        root.write(".claude/skills/speclink-onboard/SKILL.md", "old\n");
        root.write(".agents/skills/speclink-onboard/SKILL.md", "old\n");
        update(&root.dir, false).unwrap();
        assert!(!root.exists(".claude/skills/speclink-onboard"), "舊目錄須被清除");
        assert!(!root.exists(".agents/skills/speclink-onboard"), "舊目錄須被清除");
        assert!(root.exists(".claude/skills/speclink-baseline/SKILL.md"));
        assert!(root.exists(".agents/skills/speclink-baseline/SKILL.md"));
    }

    #[test]
    fn update_keeps_user_skill_directories_without_prefix() {
        let root = TempRoot::new("prune-user-dir");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        let content = "user skill\n";
        root.write(".claude/skills/conventional-commit/SKILL.md", content);
        update(&root.dir, false).unwrap();
        assert_eq!(
            root.read(".claude/skills/conventional-commit/SKILL.md"),
            content,
            "非 speclink- 前綴的使用者技能不受清理影響"
        );
    }

    #[test]
    fn update_prunes_prefixed_directories_not_in_registry() {
        let root = TempRoot::new("prune-prefixed");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".claude/skills/speclink-myown/SKILL.md", "mine\n");
        update(&root.dir, false).unwrap();
        // speclink- 前綴保留給生成物：非 registry 的前綴目錄一律清除。
        assert!(!root.exists(".claude/skills/speclink-myown"));
    }

    fn app_config(yaml: &str) -> crate::config::AppConfig {
        serde_yaml::from_str(yaml).expect("app config parses")
    }

    /// 工具選集是 `.speclink.yaml` tools 清單的唯一解析點：內建名去重、順序固定
    /// claude → codex，沒有清單時才是 legacy 回退。
    #[test]
    fn tool_selection_resolves_builtins_without_duplicates() {
        let root = TempRoot::new("tool-selection-builtins");
        let sel = ToolSelection::resolve(&root.dir, &app_config("tools: [claude, codex, claude]\n"));
        assert_eq!(sel.builtins, vec![Tool::Claude, Tool::Codex]);
        assert!(!sel.legacy_fallback, "a non-empty list is not the legacy path");
        assert!(sel.customs.is_empty());
        assert!(sel.notes.is_empty());
        assert_eq!(sel.descriptor_error, None);
    }

    /// 合法描述子與內建名混在同一份清單：兩邊都解析出來，沒有錯誤。
    #[test]
    fn tool_selection_carries_a_valid_descriptor_beside_the_builtins() {
        let root = TempRoot::new("tool-selection-descriptor");
        let sel = ToolSelection::resolve(
            &root.dir,
            &app_config(&format!("tools:\n  - claude\n{CUSTOM_DESCRIPTOR}")),
        );
        assert_eq!(sel.builtins, vec![Tool::Claude]);
        assert_eq!(sel.customs.len(), 1, "the descriptor resolves: {:?}", sel.customs);
        assert_eq!(sel.customs[0].name, "wad-harness");
        assert_eq!(sel.customs[0].skills_dir, ".wad/skills");
        assert_eq!(sel.descriptor_error, None);
    }

    /// 壞描述子不在解析時就變成 `Err`：錯誤留在值上由消費端裁決（update 轉錯誤、
    /// probe 忽略），內建名照樣解析出來。
    #[test]
    fn tool_selection_defers_a_bad_descriptor_to_its_consumer() {
        let root = TempRoot::new("tool-selection-bad-descriptor");
        let sel = ToolSelection::resolve(
            &root.dir,
            &app_config("tools:\n  - codex\n  - name: broken-harness\n"),
        );
        assert_eq!(sel.builtins, vec![Tool::Codex], "the builtins still resolve");
        assert!(sel.customs.is_empty(), "an invalid descriptor never reaches the filesystem");
        let message = sel.descriptor_error.expect("the descriptor error is carried on the value");
        assert_eq!(message, "tool descriptor: missing required field 'skills_dir'");
    }

    /// 空 tools 清單＝legacy 回退：只有 `.claude` 目錄存在時才把 Claude 算進選集。
    #[test]
    fn tool_selection_falls_back_to_the_claude_footprint_for_an_empty_list() {
        let root = TempRoot::new("tool-selection-empty");
        let app = app_config("tools: []\n");
        let sel = ToolSelection::resolve(&root.dir, &app);
        assert!(sel.builtins.is_empty(), "no .claude directory means nothing to regenerate");
        assert!(sel.legacy_fallback);

        std::fs::create_dir_all(root.at(".claude")).unwrap();
        let sel = ToolSelection::resolve(&root.dir, &app);
        assert_eq!(sel.builtins, vec![Tool::Claude]);
        assert!(sel.legacy_fallback);
    }

    /// 未知內建名只是警告：訊息字面與今天 `update` 的一模一樣。
    #[test]
    fn tool_selection_notes_an_unknown_builtin_name() {
        let root = TempRoot::new("tool-selection-unknown");
        let sel = ToolSelection::resolve(&root.dir, &app_config("tools: [claude, cursor]\n"));
        assert_eq!(sel.builtins, vec![Tool::Claude]);
        assert_eq!(
            sel.notes,
            vec![
                "unknown tool 'cursor' in .speclink.yaml tools list (supported: claude, codex)"
                    .to_string()
            ]
        );
        assert!(!sel.legacy_fallback);
    }

    /// `builtins_only` 是 init／reconcile 的記憶體入口：選集逐字帶過，沒有描述子、
    /// 沒有回退、沒有警告。
    #[test]
    fn tool_selection_builtins_only_takes_the_selection_verbatim() {
        let sel = ToolSelection::builtins_only(&[Tool::Codex]);
        assert_eq!(sel.builtins, vec![Tool::Codex]);
        assert!(sel.customs.is_empty());
        assert!(sel.notes.is_empty());
        assert!(!sel.legacy_fallback);
        assert_eq!(sel.descriptor_error, None);
    }

    /// 與 `CUSTOM_DESCRIPTOR` 同一個描述子的已驗證形式。
    fn custom_target() -> CustomTool {
        CustomTool {
            name: "wad-harness".to_string(),
            skills_dir: ".wad/skills".to_string(),
            instructions_file: Some("WAD.md".to_string()),
            invocation: crate::config::Invocation::Cli,
        }
    }

    fn dir_names(set: &[(String, String)]) -> Vec<String> {
        set.iter().map(|(dir, _)| dir.clone()).collect()
    }

    /// registry 依 `codex_subset` 過濾後的 `speclink-<name>` 目錄名（順序同 registry）。
    fn registry_dirs(codex_subset: bool) -> Vec<String> {
        skills::registry()
            .iter()
            .filter(|s| !codex_subset || s.for_codex)
            .map(|s| format!("speclink-{}", s.name))
            .collect()
    }

    /// 受管技能集合只有一個擁有者：Claude 目標拿 registry 全集，非 Claude 目標拿
    /// `for_codex` 子集，每一筆內容逐字等於同參數的 render。
    #[test]
    fn managed_skills_covers_the_registry_per_target() {
        let custom = custom_target();
        let claude = managed_skills(skills::RenderTarget::Builtin(Tool::Claude), true, "openspec");
        let codex = managed_skills(skills::RenderTarget::Builtin(Tool::Codex), true, "openspec");
        let neutral = managed_skills(skills::RenderTarget::Custom(&custom), true, "openspec");

        assert_eq!(dir_names(&claude), registry_dirs(false));
        assert_eq!(dir_names(&codex), registry_dirs(true));
        assert_eq!(dir_names(&neutral), registry_dirs(true));

        let registry = skills::registry();
        for (target, set) in [
            (skills::RenderTarget::Builtin(Tool::Claude), &claude),
            (skills::RenderTarget::Builtin(Tool::Codex), &codex),
            (skills::RenderTarget::Custom(&custom), &neutral),
        ] {
            // 集合裡的每一筆都比內容——不是「找得到才比」。
            for (dir, content) in set {
                let skill = registry
                    .iter()
                    .find(|s| format!("speclink-{}", s.name) == *dir)
                    .unwrap_or_else(|| panic!("{dir} 不在 registry"));
                assert_eq!(
                    content,
                    &skills::render_skill_file_for(target, skill, "openspec"),
                    "{dir} 的內容必須等於同參數的 render"
                );
            }
        }
    }

    /// worktree 政策關閉時，三個目標的受管集合都不含兩顆 worktree 技能。
    #[test]
    fn managed_skills_drops_the_gated_skills_when_the_policy_is_off() {
        let custom = custom_target();
        for set in [
            managed_skills(skills::RenderTarget::Builtin(Tool::Claude), false, "openspec"),
            managed_skills(skills::RenderTarget::Builtin(Tool::Codex), false, "openspec"),
            managed_skills(skills::RenderTarget::Custom(&custom), false, "openspec"),
        ] {
            for gated in ["speclink-apply-with-worktree", "speclink-worktree-merge"] {
                assert!(
                    !set.iter().any(|(dir, _)| dir == gated),
                    "政策關閉時受管集合不得含 {gated}"
                );
            }
        }
    }

    /// 計畫的每個 target 帶自己的 skills_root；未選中的內建進 `deselected_builtins`。
    #[test]
    fn sync_plan_builds_one_target_per_selected_tool() {
        let root = TempRoot::new("sync-plan-targets");
        let plan =
            SyncPlan::resolve(&root.dir, ToolSelection::builtins_only(&[Tool::Codex]), "openspec");
        assert_eq!(plan.targets.len(), 1, "只選 codex 就只有一個 target");
        assert!(matches!(plan.targets[0].kind, SyncTargetKind::Builtin(Tool::Codex)));
        assert_eq!(plan.targets[0].label, "codex");
        assert_eq!(plan.targets[0].skills_root, root.at(".agents/skills"));
        assert_eq!(plan.deselected_builtins, vec![Tool::Claude]);
    }

    /// legacy 回退（沒有 tools 清單）不下架任何內建工具。
    #[test]
    fn sync_plan_prunes_nothing_on_the_legacy_fallback() {
        let root = TempRoot::new("sync-plan-legacy");
        std::fs::create_dir_all(root.at(".claude")).unwrap();
        let selection = ToolSelection::resolve(&root.dir, &app_config("tools: []\n"));
        let plan = SyncPlan::resolve(&root.dir, selection, "openspec");
        assert_eq!(plan.targets.len(), 1);
        assert_eq!(plan.targets[0].label, "claude");
        assert!(plan.deselected_builtins.is_empty(), "沒有清單就沒有「下架」這回事");
    }

    /// 描述子各自成為一個 target，skills_root 就是描述子宣告的目錄。
    #[test]
    fn sync_plan_adds_a_target_for_each_descriptor() {
        let root = TempRoot::new("sync-plan-descriptor");
        let selection = ToolSelection::resolve(
            &root.dir,
            &app_config(&format!("tools:\n  - claude\n{CUSTOM_DESCRIPTOR}")),
        );
        let plan = SyncPlan::resolve(&root.dir, selection, "openspec");
        assert_eq!(plan.targets.len(), 2);
        assert_eq!(plan.targets[0].label, "claude");
        assert!(matches!(&plan.targets[1].kind, SyncTargetKind::Custom(name) if name == "wad-harness"));
        assert_eq!(plan.targets[1].label, "wad-harness");
        assert_eq!(plan.targets[1].skills_root, root.at(".wad/skills"));
        assert_eq!(plan.deselected_builtins, vec![Tool::Codex]);
    }

    /// 守門的檢查面就是 targets 的 skills_root 集合：只有描述子目錄領先版本時
    /// 一樣被拒，訊息含工作區與引擎兩個版號。
    #[test]
    fn sync_plan_guard_checks_every_targets_skills_root() {
        let root = TempRoot::new("sync-plan-guard");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        root.write(".speclink.yaml", &format!("tools:\n{CUSTOM_DESCRIPTOR}"));
        update(&root.dir, false).expect("先生成描述子受管檔");
        let ahead = ahead_of_current();
        crate::testkit::set_skill_version(&root.at(".wad/skills"), &ahead);

        let app = crate::config::AppConfig::load(&root.at(".speclink.yaml")).expect("config parses");
        let plan = SyncPlan::resolve(&root.dir, ToolSelection::resolve(&root.dir, &app), "openspec");
        assert_eq!(
            plan.targets.iter().map(|t| t.skills_root.clone()).collect::<Vec<PathBuf>>(),
            vec![root.at(".wad/skills")],
            "守門的檢查面等於 targets 的 skills_root"
        );

        let message = plan.guard().expect_err("只有描述子目錄領先也必須被拒").to_string();
        assert!(message.contains(&ahead), "訊息須含工作區版號：{message}");
        assert!(message.contains(ASSET_VERSION), "訊息須含引擎版號：{message}");
    }

    /// `init --force` 會把 `openspec/config.yaml` 寫回範本（worktree 政策關閉）：上一次
    /// 政策開啟留下的兩顆 worktree 技能目錄必須跟著消失（舊 `skip_gated_skill` 的行為），
    /// 其餘技能照常在。
    #[test]
    fn init_force_removes_gated_skill_directories_the_reset_policy_no_longer_allows() {
        let root = TempRoot::new("init-force-gated");
        init(&root.dir, &[Tool::Claude], false, "openspec").unwrap();
        set_worktree_policy(&root, true);
        update(&root.dir, false).unwrap();
        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(root.exists(&format!("{dir}/SKILL.md")), "前置：政策開啟時 {dir} 存在");
        }

        init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();

        assert!(
            !root.read("openspec/config.yaml").contains("\nworktree: true"),
            "--force 把 config.yaml 寫回範本"
        );
        for dir in worktree_skill_dirs(Tool::Claude) {
            assert!(!root.exists(&dir), "政策被重置為關閉後 {dir} 不得留下");
        }
        assert!(root.exists(".claude/skills/speclink-apply/SKILL.md"));
    }
}

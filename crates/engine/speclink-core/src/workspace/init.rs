//! Project initialization and instruction-file updates.

use crate::config::{CustomTool, ToolEntry};
use crate::skills::{self, Tool};
use crate::util;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// 產物層的唯一版號：技能檔 frontmatter 的 version 同源於此，也是過期探測與
/// 降級守門的比對基準。僅在內嵌資產（assets/skills）的 render 內容變動時遞增——
/// 與 app／CLI 的發版號無關；`assets.lock` 鎖定測試把這條紀律變成紅燈。
pub const ASSET_VERSION: &str = "v1.38.0";

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
///
/// The skills go through [`SyncPlan::apply`], the same writer every other entry uses
/// (change: workspace-sync-entrypoints). So `tools` is the COMPLETE desired state here
/// too, and `force` only decides whether an initialized workspace is accepted at all —
/// every init that reaches `apply` (a `--force` re-init, or a first init over residual
/// skill files with no `.speclink.yaml`) drops the deselected built-ins' skill
/// footprints, removes `speclink-` directories outside the current set, and resets the
/// recorded descriptor footprints — matching the `.speclink.yaml` write, which is the
/// template. Of a tool's own content only `speclink-` prefixed directories are removed;
/// a skills directory (and its parent) that this leaves empty goes too, as does an
/// emptied `.speclink/`.
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
    // 第二次解析是刻意的：政策以 store_init 之後的 config.yaml 為準——`--force` 會把
    // 它寫回範本，寫入用的計畫要在那之後才解析才會排除被政策關掉的技能。守門面只取
    // 決於選集，所以兩次的目錄集合相同。apply 的回報（pruned／stripped）在 init 不
    // 呈現：init 的 stdout 維持既有兩行（proposal Non-Goal）。
    SyncPlan::resolve(root, ToolSelection::builtins_only(tools), spec_dir).apply(root)?;

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
    // Remote init 不寫 openspec/，政策檔不會被重寫，所以一份計畫同時服務守門與寫入。
    let plan = SyncPlan::resolve(root, ToolSelection::builtins_only(tools), "openspec");
    plan.guard()?;

    write_if(&root.join(".speclink.yaml"), &app_config_text(tools, "openspec"), force)?;
    ensure_gitignore(&root.join(".gitignore"))?;
    // 回報同 `init`：不呈現。
    plan.apply(root)?;
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
/// while `probe_assets` never reads it — an invalid descriptor simply never becomes a
/// target, and the probe's verdict must not change because one entry is broken.
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
                    // Keep the FIRST problem (it is the one update reports), but keep
                    // collecting the valid descriptors after it: the probe reads
                    // `customs` and must not lose a tool because an earlier entry is broken.
                    let problem = match d.validate() {
                        Ok(custom) => {
                            if sel.customs.iter().any(|c| c.name == custom.name) {
                                Some(format!("tool descriptor: duplicate name '{}'", custom.name))
                            } else {
                                sel.customs.push(custom);
                                None
                            }
                        }
                        Err(message) => Some(message),
                    };
                    if sel.descriptor_error.is_none() {
                        sel.descriptor_error = problem;
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

/// One tool's slice of a sync: where its skills live, and what should be there.
#[derive(Debug)]
pub(crate) struct SyncTarget {
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
}

impl SyncPlan {
    /// Build the plan. This is the ONE place a sync reads the worktree policy.
    pub(crate) fn resolve(root: &Path, selection: ToolSelection, spec_dir: &str) -> SyncPlan {
        let worktree_on = worktree_skills_enabled(root, spec_dir);
        let mut targets = Vec::new();
        for tool in [Tool::Claude, Tool::Codex] {
            if selection.builtins.contains(&tool) {
                targets.push(SyncTarget {
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
            // `skills_dir` 已在 `ToolDescriptor::validate` 正規化（削結尾分隔符、
            // 拒絕削完等於專案根或撞上內建目錄者），這裡直接信任欄位。
            targets.push(SyncTarget {
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
        }
    }

    /// Downgrade guard: refuse when any target's skills directory leads this engine.
    /// The checked set IS the write set.
    pub(crate) fn guard(&self) -> Result<()> {
        let dirs: Vec<PathBuf> = self.targets.iter().map(|t| t.skills_root.clone()).collect();
        refuse_downgrade(&dirs)
    }

    /// Every target's managed files that are absent or differ from the current render
    /// (line endings normalized), as project-root-relative `/`-joined paths. Descriptors
    /// are included: their skill files are managed exactly like a built-in's, so the
    /// staleness probe must see them too.
    pub(crate) fn differing_files(&self) -> Vec<String> {
        let mut differing = Vec::new();
        for target in &self.targets {
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

    /// The ONE writer behind every regeneration entry — [`init`], [`init_remote`],
    /// [`adopt`], [`reconcile_builtin_tools`] and [`update`]; each entry adds only its
    /// own precondition on top. Order (any step's `Err` stops the run; what is written
    /// stays — every step is idempotent, so a rerun converges):
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
        // 足跡比對讀的是 `validate` 正規化後的 skills_dir。舊版引擎記下的可能是未削
        // 的拼法（`.cursor/skills/`），升級後第一次同步會判它已下架、先剪再重寫同一
        // 個目錄，並在 `pruned` 報一次該工具——一次性的誤報，之後兩邊同形就消失。
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
/// the desktop settings page, checkout binding and [`adopt`] share this entry.
///
/// Two steps: `.speclink.yaml`'s claude/codex entries are rewritten to match the
/// selection (custom descriptors, remote, spec_dir and unknown keys carry over
/// untouched), then [`SyncPlan::apply`] — the writer every entry shares — generates the
/// selected tools' skills, prunes the deselected ones and strips any legacy
/// instruction-file marker.
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
/// 掛在 `SyncPlan::apply`，所以每個再生入口（含 init）都清。
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
/// 判定面為 tools 清單宣告的內建工具與通過驗證的自訂描述子——受管檔集合即判定面。
/// 兩條分界：(1) 無效描述子不成為 target，探測結果不受它影響——探測是唯讀提示面，
/// 「設定裡有個壞描述子」不是技能檔過期的訊號，由 update 的錯誤負責告知；(2) 有效
/// target 的技能檔存在但讀不出（目錄不可讀、frontmatter 壞掉）＝無法判定，內建與
/// 描述子同一規則（規格「技能檔過期探測」的第五態）。
///
/// 零寫入：desktop 開專案時搭載，探測失敗不得阻斷開啟。
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
    // 探測只讀 tools 清單宣告的工具：沒有清單就沒有受管工具可查（目錄偵測的回退
    // 是 update 的再生規則，不是探測的判定面）。
    if selection.legacy_fallback {
        return without_tools(AssetStatus::Current);
    }
    let plan = SyncPlan::resolve(root, selection, &spec_dir);

    let mut tools = Vec::new();
    for target in &plan.targets {
        let version = match probe_skills_dir(&target.skills_root) {
            SkillsProbe::Absent => {
                tools.push(ToolAssetState {
                    tool: target.label.clone(),
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
            tool: target.label.clone(),
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
mod tests;

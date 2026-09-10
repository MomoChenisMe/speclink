//! Settings verbs: the user-level `config` and the project `workflow-config`.
//!
//! `config` is ModeFree (a user-level file). `workflow-config` is Dual, and its
//! stdin is consumed once at the argv layer — before the mode is resolved — so
//! both modes normalize identically.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;
use std::path::PathBuf;

use crate::color;
use crate::common::{print_json, read_stdin, require_workspace};
use crate::dual;
use crate::remote_base::RemoteCtx;
use core::workspace::Workspace;

#[derive(Args)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}
#[derive(Subcommand)]
enum ConfigCommands {
    /// Show config file path
    Path,
    /// List all settings
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
        /// Treat value as string
        #[arg(long)]
        string: bool,
        /// Allow unknown keys
        #[arg(long = "allow-unknown")]
        allow_unknown: bool,
    },
    /// Remove a config key
    Unset {
        /// Config key
        key: String,
    },
    /// Reset config
    Reset {
        /// Reset all settings
        #[arg(long)]
        all: bool,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Edit config in $EDITOR
    Edit,
}
#[derive(Args)]
pub(crate) struct WorkflowConfigArgs {
    #[command(subcommand)]
    command: WorkflowConfigCommands,
}
#[derive(Subcommand)]
enum WorkflowConfigCommands {
    /// Show the canonical workflow config (policy fields, context, rules)
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    #[command(about = SET_ABOUT.as_str())]
    Set {
        /// Policy key
        key: String,
        // 布林鍵子集刻意維持字面（design D1：程式裡沒有這個子集的常數，單一消費者
        // 不值得再立一個真相來源）；由 set_help_value_argument_names_every_boolean_key 釘住。
        /// Policy value (tdd/audit/worktree take true or false)
        value: String,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Replace the project context from stdin (blank input removes it)
    Context {
        /// Read the content from stdin
        #[arg(long)]
        stdin: bool,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Replace one artifact's rule section from stdin (empty input removes it)
    Rules {
        /// Artifact id of the active schema (proposal, design, specs, tasks, ...)
        artifact: String,
        /// Read the rules from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}
fn global_config_path() -> PathBuf {
    speclink_host::context::global_config_dir().join("config.yaml")
}
pub(crate) fn cmd_config(a: ConfigArgs) -> Result<()> {
    let path = global_config_path();
    match a.command {
        ConfigCommands::Path => println!("{}", core::util::to_slash(&path)),
        ConfigCommands::List { json } => {
            // The stored file keeps insertion order, but list output is sorted by key.
            let cfg = load_global_map(&path);
            let mut entries: Vec<(String, serde_yaml::Value)> = cfg
                .into_iter()
                .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            if json {
                let mut sorted = serde_yaml::Mapping::new();
                for (k, v) in entries {
                    sorted.insert(serde_yaml::Value::String(k), v);
                }
                return print_json(&sorted);
            }
            if entries.is_empty() {
                println!("No configuration set.");
            }
            for (k, v) in &entries {
                println!("{k} = {}", scalar_str(v));
            }
        }
        ConfigCommands::Get { key } => {
            let cfg = load_global_map(&path);
            match cfg.get(serde_yaml::Value::String(key.clone())) {
                Some(v) => println!("{}", scalar_str(v)),
                None => bail!("Key '{key}' not found."),
            }
        }
        ConfigCommands::Set { key, value, string, allow_unknown: _ } => {
            let mut cfg = load_global_map(&path);
            // Values parse to native YAML scalars (1 → int, true → bool); --string forces
            // string storage (frozen behavior).
            let stored = if string {
                serde_yaml::Value::String(value.clone())
            } else {
                serde_yaml::from_str(&value)
                    .unwrap_or_else(|_| serde_yaml::Value::String(value.clone()))
            };
            cfg.insert(serde_yaml::Value::String(key.clone()), stored);
            save_global_map(&path, &cfg)?;
            println!("{} {key} = {value}", color::green("✓"));
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = load_global_map(&path);
            cfg.remove(serde_yaml::Value::String(key.clone()));
            save_global_map(&path, &cfg)?;
            // Printed whether or not the key existed (frozen behavior).
            println!("{} Removed key: {key}", color::green("✓"));
        }
        ConfigCommands::Reset { all: _, yes: _ } => {
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            println!("{} Config reset.", color::green("✓"));
        }
        ConfigCommands::Edit => {
            // VISUAL wins over EDITOR; the vi fallback and the failure message when no
            // editor can be spawned are both frozen.
            let editor = std::env::var("VISUAL")
                .or_else(|_| std::env::var("EDITOR"))
                .unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(&editor).arg(&path).status();
            match status {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    bail!("Failed to open editor '{editor}': program not found")
                }
                Err(e) => bail!("Failed to open editor '{editor}': {e}"),
            }
        }
    }
    Ok(())
}
/// Bare display form of a YAML scalar (strings unquoted, numbers/bools as literals).
fn scalar_str(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim_end().to_string(),
    }
}
fn load_global_map(path: &std::path::Path) -> serde_yaml::Mapping {
    match core::util::read_opt(path) {
        Some(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        None => Default::default(),
    }
}
fn save_global_map(path: &std::path::Path, map: &serde_yaml::Mapping) -> Result<()> {
    let yaml = serde_yaml::to_string(map)?;
    core::util::write_file(path, &yaml)?;
    Ok(())
}
/// The policy keys `workflow-config set` accepts, in canonical order.
const POLICY_KEYS: [&str; 5] = ["locale", "spec_locale", "tdd", "audit", "worktree"];
/// `set`'s clap description, grown from `POLICY_KEYS` so help can never advertise
/// a different key set than the verb accepts. A doc comment is a compile-time
/// literal and cannot interpolate; the derive attribute takes any expression, the
/// same way `main.rs` builds `version` from a `LazyLock<String>`.
static SET_ABOUT: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("Set a policy field: {}", POLICY_KEYS.join(", ")));
/// `workflow-config show --json` payload. camelCase field names are the contract;
/// the values are CANONICAL (what the document says), never the three-layer
/// resolution — effective policy is the instructions payload's job.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowConfigJson {
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
    worktree: bool,
    context: Option<String>,
    rules: std::collections::BTreeMap<String, Vec<String>>,
}
/// A write subcommand once argv and stdin are resolved — the shape both the fs
/// and the remote branch hand to the shared rewrite.
enum WorkflowConfigWrite {
    Policy { key: String, value: String },
    Context(String),
    Rules { artifact: String, body: String },
}
/// The rewrite a write subcommand produces: the complete new document plus the
/// line printed on a successful write.
struct WorkflowConfigEdit {
    new_text: String,
    summary: String,
}
/// dispatch 正規化後、兩臂共同消費的 workflow-config 執行計畫：`write` 為
/// 解析完的寫入意圖與其 `--dry-run` 旗標（None＝show），`json` 只屬 show。
struct WorkflowConfigPlan {
    write: Option<(WorkflowConfigWrite, bool)>,
    json: bool,
}
/// workflow-config 的本機臂：執行計畫作用於 `<spec_dir>/config.yaml`。
fn workflow_config_fs(plan: WorkflowConfigPlan) -> Result<()> {
    let WorkflowConfigPlan { write, json } = plan;
    let ws = require_workspace()?;
    let label = format!("{}/config.yaml", ws.spec_dir_name);
    let path = ws.spec_dir().join("config.yaml");
    let original = core::util::read_opt(&path).unwrap_or_default();
    let Some((write, dry_run)) = write else {
        return print_workflow_config(&original, &label, json);
    };
    let edit = plan_workflow_config_edit(&write, &original, &label, Some(&ws))?;
    if dry_run {
        print!("{}", unified_diff(&label, &original, &edit.new_text));
        return Ok(());
    }
    // worktree 的寫入牽動技能足跡，故三步有序（design D2）：先擋、再寫、後同步。
    // 擋下時整體不動；同步失敗時 config 已是正典，錯誤浮出並指向重跑 update。
    // 擋下只在「由 true 改 false」（政策已關時技能本就不在，殘留的 worktree
    // 沒有收尾工具被抽走的風險，那個 no-op 寫入不該被拒）。
    let worktree_target = worktree_write_target(&write);
    let worktree_was_on = serde_yaml::from_str::<core::config::WorkflowConfig>(&original)
        .map(|c| c.worktree.unwrap_or(false))
        .unwrap_or(false);
    if worktree_target == Some(false) && worktree_was_on {
        refuse_teardown_with_active_worktrees(&ws)?;
    }
    core::util::write_file(&path, &edit.new_text)
        .map_err(|e| anyhow::anyhow!("{label}: write failed: {e}"))?;
    println!("{} {}", color::green("✓"), edit.summary);
    if worktree_target.is_some() {
        let outcome = core::init::update(&ws.root, false).map_err(|e| {
            anyhow::anyhow!("{label} written, but the skill footprint did not sync: {e} — fix the cause above, then re-run `speclink update` to rebuild it")
        })?;
        println!(
            "{} skills synced ({})",
            color::green("✓"),
            if outcome.updated.is_empty() { "no tools configured".to_string() } else { outcome.updated.join(", ") }
        );
    }
    Ok(())
}
/// The `worktree` value a write is steering towards, or None when the write does
/// not touch that key — the trigger for both the teardown check and the sync.
fn worktree_write_target(write: &WorkflowConfigWrite) -> Option<bool> {
    match write {
        WorkflowConfigWrite::Policy { key, value } if key == "worktree" => {
            Some(matches!(value.trim(), "true"))
        }
        _ => None,
    }
}
/// Refuse to turn the policy off while linked worktrees are still open: doing so
/// would retire the merge skill they depend on. Fail-open cases (no git) list
/// nothing and let the write through.
fn refuse_teardown_with_active_worktrees(ws: &core::workspace::Workspace) -> Result<()> {
    let store = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
    let blockers = speclink_host::worktree::teardown_blockers(ws, &store);
    if blockers.is_empty() {
        return Ok(());
    }
    let list: String = blockers
        .iter()
        .map(|b| format!("\n  - {} ({}) at {}", b.change, b.branch, b.path.display()))
        .collect();
    bail!(
        "worktree is still in use — turning the policy off would remove the merge skill these worktrees need:{list}\n\
Wrap each one up with `speclink-worktree-merge` first, then set worktree false."
    )
}
/// Split the parsed subcommand into `show` (None) or a resolved write plus its
/// `--dry-run` flag. stdin is consumed here, at the argv layer, so the rewrite
/// itself stays a pure text→text step shared by both modes.
fn workflow_config_write(
    cmd: &WorkflowConfigCommands,
) -> Result<Option<(WorkflowConfigWrite, bool)>> {
    Ok(match cmd {
        WorkflowConfigCommands::Show { .. } => None,
        WorkflowConfigCommands::Set { key, value, dry_run } => Some((
            WorkflowConfigWrite::Policy { key: key.clone(), value: value.clone() },
            *dry_run,
        )),
        WorkflowConfigCommands::Context { stdin, dry_run } => {
            require_stdin_flag(*stdin, "context --stdin")?;
            Some((WorkflowConfigWrite::Context(read_stdin()), *dry_run))
        }
        WorkflowConfigCommands::Rules { artifact, stdin, dry_run } => {
            require_stdin_flag(*stdin, &format!("rules {artifact} --stdin"))?;
            Some((
                WorkflowConfigWrite::Rules { artifact: artifact.clone(), body: read_stdin() },
                *dry_run,
            ))
        }
    })
}
/// Content-taking subcommands require the flag explicitly: without it the
/// command would silently write an empty document from an interactive terminal.
fn require_stdin_flag(flag: bool, usage: &str) -> Result<()> {
    if flag {
        return Ok(());
    }
    bail!("content is read from stdin — run: speclink workflow-config {usage}")
}
/// Render the canonical view of a workflow-config document.
fn print_workflow_config(original: &str, label: &str, json: bool) -> Result<()> {
    let cfg = core::config::WorkflowConfig::from_text(Some(original))
        .map_err(|e| anyhow::anyhow!("invalid {label}: {}", e.reason))?;
    if json {
        return print_json(&WorkflowConfigJson {
            locale: cfg.locale.clone(),
            spec_locale: cfg.spec_locale.clone(),
            tdd: cfg.tdd.unwrap_or(false),
            audit: cfg.audit.unwrap_or(false),
            worktree: cfg.worktree.unwrap_or(false),
            context: cfg.context_text(),
            rules: cfg.rules.clone(),
        });
    }
    println!("{} {label}", color::bold("Workflow config:"));
    println!();
    let locale = match cfg.locale.as_deref() {
        Some(v) => v.to_string(),
        None => "unset (English)".to_string(),
    };
    let spec_locale = match cfg.spec_locale.as_deref() {
        Some(v) => v.to_string(),
        None => "unset (specs in English)".to_string(),
    };
    println!("  {:<13}{locale}", "locale");
    println!("  {:<13}{spec_locale}", "spec_locale");
    println!("  {:<13}{}", "tdd", toggle_display(cfg.tdd));
    println!("  {:<13}{}", "audit", toggle_display(cfg.audit));
    println!("  {:<13}{}", "worktree", toggle_display(cfg.worktree));
    let context = match cfg.context_text() {
        Some(text) => format!("{} lines", text.lines().count()),
        None => "none".to_string(),
    };
    println!("  {:<13}{context}", "context");
    let rules: Vec<String> = cfg
        .rules
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .map(|(artifact, entries)| format!("{artifact} {}", entries.len()))
        .collect();
    let rules = if rules.is_empty() { "none".to_string() } else { rules.join(", ") };
    println!("  {:<13}{rules}", "rules");
    Ok(())
}
/// A toggle's canonical display: `false` is never stored, so "not set" and
/// "set to false" are the same state — name the default so it reads as one.
fn toggle_display(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "on",
        Some(false) | None => "unset (off)",
    }
}
/// Apply one write to `original` through the core rewrite seam — the single
/// place fs and remote share, so their write semantics can never diverge.
/// Fails closed on an unparseable document: the read-modify-write would
/// otherwise destroy the user's content.
fn plan_workflow_config_edit(
    write: &WorkflowConfigWrite,
    original: &str,
    label: &str,
    ws: Option<&Workspace>,
) -> Result<WorkflowConfigEdit> {
    let current = core::config::WorkflowConfig::from_text(Some(original))
        .map_err(|e| anyhow::anyhow!("invalid {label}: {} — write refused", e.reason))?;
    // The seam takes the COMPLETE target state of the four policy keys, so the
    // current values are read back first and only the edited key moves.
    let mut fields = core::config::WorkflowPolicyFields {
        locale: current.locale.clone(),
        spec_locale: current.spec_locale.clone(),
        tdd: current.tdd.unwrap_or(false),
        audit: current.audit.unwrap_or(false),
        worktree: current.worktree.unwrap_or(false),
    };
    let mut context = core::config::ContextEdit::Keep;
    let mut rules: Option<Vec<(String, Vec<String>)>> = None;
    let summary = match write {
        WorkflowConfigWrite::Policy { key, value } => {
            set_policy_field(&mut fields, key, value)?;
            format!("{key} = {value}")
        }
        WorkflowConfigWrite::Context(text) => {
            let summary = if text.trim().is_empty() {
                "context removed".to_string()
            } else {
                format!("context set ({} lines)", text.trim_end().lines().count())
            };
            context = core::config::ContextEdit::Set(text.clone());
            summary
        }
        WorkflowConfigWrite::Rules { artifact, body } => {
            let artifacts = workflow_schema_artifacts(ws, &current);
            if artifacts.is_empty() {
                bail!("{}", core::schema::not_found_msg(&current.schema_name()));
            }
            if !artifacts.iter().any(|id| id == artifact) {
                bail!(
                    "Unknown artifact '{artifact}' for schema '{}'. Use one of: {}",
                    current.schema_name(),
                    artifacts.join(", ")
                );
            }
            let entries: Vec<String> = body
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            let summary = if entries.is_empty() {
                format!("rules.{artifact} removed")
            } else {
                format!("rules.{artifact} = {} entries", entries.len())
            };
            rules = Some(merged_rules(&current.rules, &artifacts, artifact, entries));
            summary
        }
    };
    let new_text =
        core::config::update_workflow_config_text(original, &fields, &context, rules.as_deref())?;
    Ok(WorkflowConfigEdit { new_text, summary })
}
/// Map one `set <key> <value>` onto the complete-target-state fields. An empty
/// locale value and `false` both mean "back to default" — the seam then removes
/// the key, keeping unset-means-default intact.
fn set_policy_field(
    fields: &mut core::config::WorkflowPolicyFields,
    key: &str,
    value: &str,
) -> Result<()> {
    let text = || {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    };
    match key {
        "locale" => fields.locale = text(),
        "spec_locale" => fields.spec_locale = text(),
        "tdd" => fields.tdd = policy_bool(key, value)?,
        "audit" => fields.audit = policy_bool(key, value)?,
        "worktree" => fields.worktree = policy_bool(key, value)?,
        _ => bail!("Unknown key '{key}'. Use one of: {}", POLICY_KEYS.join(", ")),
    }
    Ok(())
}
/// Toggles accept only `true`/`false` — "1", "yes" and friends are refused
/// rather than guessed at.
fn policy_bool(key: &str, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("Value for '{key}' must be true or false (got '{value}')"),
    }
}
/// The active schema's artifact ids in display order — both the accepted keys
/// for `rules <artifact>` and the section order written back. Empty when the
/// schema cannot be resolved (the caller turns that into the not-found error).
fn workflow_schema_artifacts(
    ws: Option<&Workspace>,
    cfg: &core::config::WorkflowConfig,
) -> Vec<String> {
    let user_dir = speclink_host::context::global_config_dir();
    match core::schema::resolve_with(ws, Some(&user_dir), &cfg.schema_name()) {
        Some(Ok(schema)) => core::status::display_order(&schema)
            .into_iter()
            .map(|a| a.id.clone())
            .collect(),
        _ => Vec::new(),
    }
}
/// The complete rules map the seam replaces wholesale: the target section
/// swapped in, every other section carried over. Ordered by the schema's
/// artifact display order (the layout the desktop settings page also writes),
/// with any section outside the schema appended after so nothing is dropped.
/// Empty sections are removed by the seam.
fn merged_rules(
    current: &std::collections::BTreeMap<String, Vec<String>>,
    artifacts: &[String],
    target: &str,
    entries: Vec<String>,
) -> Vec<(String, Vec<String>)> {
    let mut keys: Vec<String> = artifacts.to_vec();
    for key in current.keys() {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        let section = if key == target {
            entries.clone()
        } else {
            current.get(&key).cloned().unwrap_or_default()
        };
        out.push((key, section));
    }
    out
}
/// Unified diff over lines, generated here rather than shelled out to a system
/// `diff` (Windows has none): the changed span between the common prefix and
/// suffix, with up to three lines of context on each side. Empty when the two
/// texts are identical.
fn unified_diff(label: &str, old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }
    let o: Vec<&str> = old.lines().collect();
    let n: Vec<&str> = new.lines().collect();
    let mut pre = 0;
    while pre < o.len() && pre < n.len() && o[pre] == n[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < o.len() - pre && suf < n.len() - pre && o[o.len() - 1 - suf] == n[n.len() - 1 - suf]
    {
        suf += 1;
    }
    const CTX: usize = 3;
    let start = pre - pre.min(CTX);
    let o_end = o.len() - suf + suf.min(CTX);
    let n_end = n.len() - suf + suf.min(CTX);
    let mut out = format!("--- a/{label}\n+++ b/{label}\n");
    out.push_str(&format!(
        "@@ -{} +{} @@\n",
        hunk_range(start, o_end - start),
        hunk_range(start, n_end - start)
    ));
    for line in &o[start..pre] {
        out.push_str(&format!(" {line}\n"));
    }
    for line in &o[pre..o.len() - suf] {
        out.push_str(&format!("-{line}\n"));
    }
    for line in &n[pre..n.len() - suf] {
        out.push_str(&format!("+{line}\n"));
    }
    for line in &o[o.len() - suf..o_end] {
        out.push_str(&format!(" {line}\n"));
    }
    out
}
/// One side of a hunk header. An empty side is `0,0` by unified-diff convention
/// (there is no line 1 to point at).
fn hunk_range(start: usize, count: usize) -> String {
    if count == 0 {
        "0,0".to_string()
    } else {
        format!("{},{count}", start + 1)
    }
}
/// The document label in remote mode: the server holds one workflow-config
/// document per scope, with no local path to name.
const REMOTE_CONFIG_LABEL: &str = "config.yaml";
/// Remote workflow-config: read the server document (content plus the scope
/// revision), apply the SAME core rewrite fs mode uses, write back guarded by
/// the revision just read. The revision never reaches the command surface — a
/// CAS refusal simply means someone else wrote in the read→write window, and
/// re-running the command is the whole fix.
fn remote_workflow_config(
    ctx: &RemoteCtx,
    write: Option<(WorkflowConfigWrite, bool)>,
    json: bool,
) -> Result<()> {
    let current = ctx.client.config()?;
    let original = current.content.unwrap_or_default();
    let Some((write, dry_run)) = write else {
        return print_workflow_config(&original, REMOTE_CONFIG_LABEL, json);
    };
    let edit = plan_workflow_config_edit(&write, &original, REMOTE_CONFIG_LABEL, None)?;
    if dry_run {
        print!("{}", unified_diff(REMOTE_CONFIG_LABEL, &original, &edit.new_text));
        return Ok(());
    }
    ctx.client
        .put_config(&edit.new_text, current.revision)
        .map_err(remote_config_write_error)?;
    println!("{} {}", color::green("✓"), edit.summary);
    Ok(())
}
/// The CAS refusal restated in this verb's own terms — the generic
/// "re-read and re-apply" wording does not say what the user should do with a
/// single-shot command. Every other failure keeps its translated message.
fn remote_config_write_error(e: speclink_remote::RemoteError) -> anyhow::Error {
    if e.reason.as_deref() == Some("revision_conflict") {
        return anyhow::anyhow!(
            "the workflow config was updated by someone else — re-run this command to apply your change on top"
        );
    }
    anyhow::Error::new(e)
}

/// workflow-config 的家族入口：stdin 於 argv 層一次消費（兩模式共用的正規化），
/// 先於模式解析——凍結行為；雙臂宣告在尾端的 `dual`。
pub(crate) fn cmd_workflow_config(a: WorkflowConfigArgs) -> Result<()> {
    let plan = WorkflowConfigPlan {
        json: matches!(a.command, WorkflowConfigCommands::Show { json: true }),
        write: workflow_config_write(&a.command)?,
    };
    dual(plan, workflow_config_fs, |ctx, p| remote_workflow_config(ctx, p.write, p.json))
}

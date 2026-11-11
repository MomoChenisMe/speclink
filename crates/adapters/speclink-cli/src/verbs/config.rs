//! Settings verbs: the user-level `config` and the project `workflow-config`.
//!
//! `config` is ModeFree (a user-level file). `workflow-config` is Dual: its
//! clap types, its conversion to an engine command and its output live in
//! speclink-verbs; this file runs the command and does what needs a checkout
//! before and after it. stdin is consumed once at the argv layer — before the
//! mode is resolved — so both modes normalize identically.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;
use std::path::PathBuf;

use crate::color;
use crate::common::{
    emit, engine_command, open_project, print_json, render_options, run, stdin_for,
};
use crate::dual;
use crate::remote_base::RemoteCtx;
use core::command::{Command, WorkflowConfigShowOutcome, WorkflowConfigWriteOutcome};
use core::config::{ChangedKey, PolicyKey, WorkflowConfig, WorkflowConfigEdit};
use speclink_verbs::families::workflow_config::WorkflowConfigArgs;
use speclink_verbs::render::workflow_config as render;
use speclink_verbs::{ToCommand, VerbInputs};

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
/// workflow-config 的家族入口：stdin 於 argv 層一次消費（兩模式共用的正規化），
/// 先於模式解析——凍結行為；雙臂宣告在尾端的 `dual`。
pub(crate) fn cmd_workflow_config(a: WorkflowConfigArgs) -> Result<()> {
    // 只看 argv 就能拒絕的（少了 --stdin、工具名不合法）先於讀 stdin。
    a.to_command(VerbInputs::default())?;
    let inputs = VerbInputs { stdin: stdin_for(a.stdin_policy()), ..Default::default() };
    let cmd = engine_command(a.to_command(inputs))?;
    dual((a, cmd), workflow_config_fs, remote_workflow_config)
}

/// workflow-config 的本機臂：選儲存 → 前置檢查 → 執行命令 → 印出共用的輸出 →
/// 後置處理（workflow-config-commands D5）。讀寫都在引擎命令裡；這裡只做需要
/// checkout 的事。
fn workflow_config_fs((a, cmd): (WorkflowConfigArgs, Command)) -> Result<()> {
    let (ws, store) = open_project()?;
    let Command::WorkflowConfigEdit { edit, dry_run } = cmd else {
        let shown: WorkflowConfigShowOutcome = run(&store, Some(&ws), cmd)?;
        emit(render::show(&shown, render_options(a.json())));
        return Ok(());
    };
    // worktree 的寫入牽動技能足跡，故三步有序：先擋、再寫、後同步。擋下時整體
    // 不動；同步失敗時 config 已是正典，錯誤浮出並指向重跑 update。
    if !dry_run && turns_worktree_off(&store, &edit) {
        refuse_teardown_with_active_worktrees(&ws)?;
    }
    let done: WorkflowConfigWriteOutcome =
        run(&store, Some(&ws), Command::WorkflowConfigEdit { edit, dry_run })?;
    emit(render::written(&a.command, &done, render_options(false)));
    warn_if_station_skills_stale(&ws.root, &done);
    // 只同步技能檔：規格資料夾的遷移只走 `speclink update`（desktop-spec-dir-migration D6）。
    if done.written && done.changed_keys.contains(&ChangedKey::Policy(PolicyKey::Worktree)) {
        let outcome = core::init::sync_skills(&ws.root).map_err(|e| {
            anyhow::anyhow!("{} written, but the skill footprint did not sync: {e} — fix the cause above, then re-run `speclink update` to rebuild it", done.file)
        })?;
        println!(
            "{} skills synced ({})",
            color::green("✓"),
            if outcome.updated.is_empty() { "no tools configured".to_string() } else { outcome.updated.join(", ") }
        );
    }
    Ok(())
}

/// Whether this edit turns the worktree policy from on to off — the one write
/// the teardown check guards. With the policy already off the skills are not
/// there, so a leftover worktree has no merge tool to lose and that write is
/// let through. A document that does not parse is the command's to refuse.
fn turns_worktree_off(store: &dyn core::store::Store, edit: &WorkflowConfigEdit) -> bool {
    let WorkflowConfigEdit::Policy { key, value } = edit else {
        return false;
    };
    PolicyKey::parse(key) == Some(PolicyKey::Worktree)
        && value == "false"
        && WorkflowConfig::from_store(store).is_ok_and(|cfg| cfg.worktree.unwrap_or(false))
}

/// Review rules and sub-agent models reach the quality stations only through
/// their skill files (every other field is injected by the CLI). So after a
/// write that changed one of them — in either mode — the local skill files are
/// probed and a stale tool earns a stderr hint. A write that changed neither,
/// a dry run and a write that did not happen probe nothing.
/// No hint when the workspace is newer (update would downgrade) or the probe
/// cannot tell; a tool whose skills are missing altogether is not stale.
fn warn_if_station_skills_stale(root: &std::path::Path, done: &WorkflowConfigWriteOutcome) {
    let changed = |is: fn(&ChangedKey) -> bool| done.written && done.changed_keys.iter().any(is);
    let consequence = if changed(|k| matches!(k, ChangedKey::ReviewBaseline | ChangedKey::ReviewList(_))) {
        "an older review skill may not apply these rules"
    } else if changed(|k| matches!(k, ChangedKey::SubagentModel(_) | ChangedKey::SubagentChoices)) {
        "an older quality-station skill may not apply this sub-agent model"
    } else {
        return;
    };
    let probe = core::init::probe_assets(root);
    if matches!(probe.status, core::init::AssetStatus::Newer | core::init::AssetStatus::Unknown) {
        return;
    }
    let stale: Vec<String> = probe
        .tools
        .iter()
        .filter(|t| t.stale)
        .map(|t| format!("{} {}", t.tool, t.workspace_version.as_deref().unwrap_or_default()))
        .collect();
    if stale.is_empty() {
        return;
    }
    eprintln!(
        "speclink: warning: skill files are older than this engine ({}; engine {}) — {consequence}; run `speclink update`",
        stale.join(", "),
        probe.current_version
    );
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

/// The document label in remote mode: the server holds one workflow-config
/// document per scope, with no local path to name.
const REMOTE_CONFIG_LABEL: &str = "config.yaml";

/// Remote workflow-config: read the server document (content plus the scope
/// revision), plan the SAME engine edit fs mode runs, write the planned
/// document back guarded by the revision just read. The revision never
/// reaches the command surface — a CAS refusal simply means someone else
/// wrote in the read→write window, and re-running the command is the whole fix.
fn remote_workflow_config(ctx: &RemoteCtx, (a, cmd): (WorkflowConfigArgs, Command)) -> Result<()> {
    let current = ctx.client.config()?;
    let Command::WorkflowConfigEdit { edit, dry_run } = cmd else {
        let config = WorkflowConfig::from_text(current.content.as_deref())
            .map_err(|e| anyhow::anyhow!("invalid {REMOTE_CONFIG_LABEL}: {}", e.reason))?;
        let shown = WorkflowConfigShowOutcome {
            file: REMOTE_CONFIG_LABEL.to_string(),
            config,
            exists: current.content.is_some(),
        };
        emit(render::show(&shown, render_options(a.json())));
        return Ok(());
    };
    let user_dir = speclink_host::context::global_config_dir();
    let mut done = core::command::plan_workflow_config_edit(
        &edit,
        REMOTE_CONFIG_LABEL,
        current.content,
        None,
        Some(&user_dir),
    )?;
    if !dry_run {
        ctx.client.put_config(&done.after, current.revision).map_err(remote_config_write_error)?;
        done.written = true;
    }
    emit(render::written(&a.command, &done, render_options(false)));
    // 技能檔永遠在本機 checkout：remote 只把 config 存到 server。
    warn_if_station_skills_stale(&ctx.ws.root, &done);
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

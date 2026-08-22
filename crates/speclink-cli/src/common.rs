//! Cross-family plumbing: the pieces two or more verb families share, plus
//! the dispatch-level preamble main.rs runs before any family is entered
//! (warn_leftover_remote_file).
//!
//! Admission rule (design D3): a symbol lands here when two or more families
//! use it, or when dispatch itself does. Single-family helpers stay private
//! to their family file.

use anyhow::Result;
use speclink_core as core;
use std::io::{IsTerminal, Read};

use core::store::Store;
use core::workspace::Workspace;

/// Migration signal for the abolished connection file: exactly one fixed-prefix stderr
/// line per invocation while `.speclink.remote.yaml` lingers in the project root. The
/// file is never parsed and never affects the store mode — an unmigrated project runs
/// in fs mode until the fields move. stdout and exit codes stay untouched.
pub(crate) fn warn_leftover_remote_file() {
    let Ok(Some(ws)) = Workspace::discover_cwd() else {
        return;
    };
    if !ws.has_leftover_remote_file() {
        return;
    }
    eprintln!(
        "speclink: warning: .speclink.remote.yaml is no longer read and does not affect the store mode — move its url/repo into the `remote:` section of .speclink.yaml and delete the old file"
    );
}

/// Route a command through the engine runtime. The typed error converts into
/// the anyhow error main() prints — `Error: {message}`, text frozen by the
/// regression baseline. Events are not consumed by the CLI (yet).
pub(crate) fn run_command(
    store: &dyn Store,
    ws: Option<&core::workspace::Workspace>,
    cmd: core::command::Command,
) -> Result<core::command::CommandOutcome> {
    // Host boundary: identity and the SPECLINK_* env layer are resolved here
    // and injected — the engine runtime only ever consumes the context.
    let ctx = core::command::ExecutionContext {
        actor: ws.and_then(|w| speclink_host::context::git_identity(&w.root)),
        repo: Some(speclink_host::binding::local_default_binding().repo.as_str().to_string()),
        env: speclink_host::policy::process_env_overrides(),
        workspace: ws.cloned(),
        user_config_dir: Some(speclink_host::context::global_config_dir()),
    };
    let (outcome, _events) = core::command::execute(store, &ctx, cmd).map_err(anyhow::Error::new)?;
    Ok(outcome)
}

/// For read/analysis commands: when no change name is given and no changes exist, print the
/// informational message and signal exit-0 (returns true = handled).
pub(crate) fn info_if_no_changes(store: &dyn Store, name: Option<&str>) -> bool {
    if name.is_none() && core::model::list_changes(store).is_empty() {
        println!("No active changes. Create one with: speclink new change <name>");
        true
    } else {
        false
    }
}

pub(crate) fn read_stdin() -> String {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    buf
}

/// Read stdin for a content-taking discuss verb. Reads when the caller passed `--stdin`
/// OR stdin is piped/redirected (not an interactive terminal) — so a forgotten `--stdin`
/// with piped content still lands instead of silently becoming empty. An interactive
/// terminal with no pipe yields an empty string, which the core content guard rejects
/// with a helpful message. `--stdin` is kept for back-compat but is no longer required.
pub(crate) fn read_stdin_content(flag: bool) -> String {
    if flag || !std::io::stdin().is_terminal() {
        read_stdin()
    } else {
        String::new()
    }
}

pub(crate) fn print_json<T: serde::Serialize>(v: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

/// Discover the host workspace, or fail with the standard not-initialized error.
/// A `.speclink.yaml` that exists but cannot parse is its own (fail-closed) error.
pub(crate) fn require_workspace() -> Result<core::workspace::Workspace> {
    core::workspace::Workspace::discover_cwd()?
        .ok_or_else(|| anyhow::anyhow!("Not initialized. Run 'speclink init' to initialize."))
}

/// The CLI assembly point: discover the workspace and build the filesystem
/// storage adapter for it. Core flows receive the store as `&dyn Store`.
pub(crate) fn open_project() -> Result<(core::workspace::Workspace, speclink_fs::FsStore)> {
    let ws = require_workspace()?;
    let store = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
    Ok((ws, store))
}

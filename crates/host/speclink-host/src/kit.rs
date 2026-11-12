//! The host kit (design D1): what a host that embeds speclink builds once and
//! shares between every user. It holds the store and the hooks; each call
//! brings who acts and on which scope.
//!
//! The kit takes Engine commands, not verb text: every door (the Agent tool
//! of the Node SDK today, the HTTP door later) turns its input into a
//! [`Command`] and calls [`HostKit::run`], so the hooks see the same thing
//! from every door.

use crate::port_exec::{execute_with, BeforeCommit, ExecError, PortExecution};
use speclink_core::command::{Command, CommandError, CommandOutcome, ErrorCode, ExecutionContext};
use speclink_store_port::{Actor, Event, ScopeId, StorePort, Version};
use std::sync::Arc;

/// Who acts: an id that stays when the user is renamed, and the name that
/// documents are stamped with. Neither is empty or only white space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    actor: Actor,
}

impl Identity {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Result<Identity, InvalidIdentity> {
        let (id, display_name) = (id.into(), display_name.into());
        if id.trim().is_empty() {
            return Err(InvalidIdentity { field: "id" });
        }
        if display_name.trim().is_empty() {
            return Err(InvalidIdentity { field: "display name" });
        }
        let actor = Actor::new(id, display_name).expect("a non-blank id is a valid actor id");
        Ok(Identity { actor })
    }

    /// The id the commit record carries.
    pub fn id(&self) -> &str {
        self.actor.id()
    }

    /// The name the commit record and the document stamps carry.
    pub fn display_name(&self) -> &str {
        self.actor.display_name()
    }
}

/// An identity field that is empty or only white space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidIdentity {
    field: &'static str,
}

impl InvalidIdentity {
    /// `id` or `display name`.
    pub fn field(&self) -> &'static str {
        self.field
    }
}

impl std::fmt::Display for InvalidIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "identity {} must not be empty or blank", self.field)
    }
}

impl std::error::Error for InvalidIdentity {}

/// One call's who and where.
#[derive(Debug, Clone)]
pub struct Caller {
    pub identity: Identity,
    pub scope: ScopeId,
}

/// What the before-command hook is asked about (design D4).
#[derive(Debug, Clone)]
pub struct BeforeCommand {
    pub caller: Caller,
    /// [`Command::name`].
    pub command: &'static str,
    /// [`Command::params`].
    pub params: serde_json::Value,
    /// The keys the write changes; only the two workflow-config write
    /// commands carry them (empty when the write fails or changes nothing).
    pub changed_keys: Option<Vec<String>>,
}

/// The before-command hook's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// The call ends as `refused` with this reason.
    Deny { reason: String },
}

/// What the after-commit hook watches (design D5).
#[derive(Debug, Clone)]
pub struct AfterCommit {
    pub caller: Caller,
    pub command: &'static str,
    pub scope_version: Version,
    /// The events of the commit, as the store's commit received them.
    pub events: Vec<Event>,
}

pub type BeforeCommandHook = Box<dyn Fn(&BeforeCommand) -> Decision + Send + Sync>;
pub type AfterCommitHook = Box<dyn Fn(&AfterCommit) + Send + Sync>;

/// The hooks of a kit; each one is optional.
#[derive(Default)]
pub struct Hooks {
    /// Asked exactly once per call, before anything reaches the store's
    /// commit. Absent: every call is allowed.
    pub before_command: Option<BeforeCommandHook>,
    /// Called once after the store's commit succeeded; it cannot change the
    /// result, and a panic in it is caught.
    pub after_commit: Option<AfterCommitHook>,
}

/// The fixed text of a refusal that gives no reason.
const NO_REASON: &str = "refused by host";

/// One kit for every user: the store and the hooks, with no state of its
/// own, so one value serves every request.
pub struct HostKit {
    store: Arc<dyn StorePort>,
    hooks: Hooks,
}

impl HostKit {
    pub fn new(store: Arc<dyn StorePort>, hooks: Hooks) -> HostKit {
        HostKit { store, hooks }
    }

    /// Run `cmd` for `caller`. Without hooks the result is what
    /// [`crate::port_exec::execute`] gives for the same command; the commit
    /// records the caller's identity and the command's name.
    pub fn run(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError> {
        let command = cmd.name();
        let ask = |params: &serde_json::Value, changed_keys: Option<Vec<String>>| -> Result<(), ExecError> {
            let Some(hook) = &self.hooks.before_command else {
                return Ok(());
            };
            let info = BeforeCommand {
                caller: caller.clone(),
                command,
                params: params.clone(),
                changed_keys,
            };
            match hook(&info) {
                Decision::Allow => Ok(()),
                Decision::Deny { reason } => {
                    let reason = if reason.trim().is_empty() { NO_REASON.to_string() } else { reason };
                    Err(ExecError::Command(CommandError::new(ErrorCode::Refused, reason)))
                }
            }
        };
        let params = match &self.hooks.before_command {
            Some(_) => cmd.params(),
            None => serde_json::Value::Null,
        };
        let cmd = without_local_facts(cmd);
        // The kit reads no process environment and no checkout: the Engine
        // gets the caller's name and identifier, and nothing else.
        let engine_ctx = ExecutionContext {
            actor: Some(caller.identity.display_name().to_string()),
            actor_id: Some(caller.identity.id().to_string()),
            ..Default::default()
        };
        let (store, scope, actor) = (self.store.as_ref(), &caller.scope, &caller.identity.actor);

        let execution = if is_workflow_config_write(&cmd) {
            // Asked once the Engine computed the write, before the commit: the
            // keys it sees are the keys that are committed.
            let asked = std::cell::Cell::new(false);
            let check: BeforeCommit<'_> = &|outcome| {
                asked.set(true);
                ask(&params, Some(changed_keys(outcome)))
            };
            let result = execute_with(store, scope, &engine_ctx, actor, cmd, Some(check));
            if result.is_err() && !asked.get() {
                // The write failed before the check: still asked exactly once,
                // and a refusal wins over the failure.
                ask(&params, Some(Vec::new()))?;
            }
            result?
        } else {
            // Asked before the Engine runs: a refused caller cannot tell
            // "not found" from "refused", and the store sees no call.
            ask(&params, None)?;
            execute_with(store, scope, &engine_ctx, actor, cmd, None)?
        };

        if let (Some(hook), Some(scope_version)) = (&self.hooks.after_commit, execution.scope_version) {
            let info = AfterCommit {
                caller: caller.clone(),
                command,
                scope_version,
                events: execution.commit_events.clone(),
            };
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hook(&info)));
        }
        Ok(execution)
    }
}

/// A task completion that names no changed files records none. Without this,
/// the Engine probes a local workspace, and with no workspace that is the
/// process's working directory: a host running in a git checkout would record
/// that checkout's dirty files. The hook still sees the params as the caller
/// sent them.
fn without_local_facts(cmd: Command) -> Command {
    match cmd {
        Command::TaskDone { task_id, change, touched_files: None, head_commit } => {
            Command::TaskDone { task_id, change, touched_files: Some(Vec::new()), head_commit }
        }
        other => other,
    }
}

fn is_workflow_config_write(cmd: &Command) -> bool {
    matches!(cmd, Command::WorkflowConfigEdit { .. } | Command::WorkflowConfigReplace { .. })
}

fn changed_keys(outcome: &CommandOutcome) -> Vec<String> {
    match outcome {
        CommandOutcome::WorkflowConfigEdit(write) | CommandOutcome::WorkflowConfigReplace(write) => {
            write.changed_keys.iter().map(ToString::to_string).collect()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_done(touched_files: Option<Vec<String>>) -> Command {
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files,
            head_commit: None,
        }
    }

    #[test]
    fn a_completion_without_files_names_none_instead_of_probing() {
        assert_eq!(without_local_facts(task_done(None)), task_done(Some(Vec::new())));
        let named = task_done(Some(vec!["src/app.rs".to_string()]));
        assert_eq!(without_local_facts(named.clone()), named);
        assert_eq!(without_local_facts(Command::LanguageShow), Command::LanguageShow);
    }
}

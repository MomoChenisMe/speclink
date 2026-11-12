//! The host kit (design D1): what a host that embeds speclink builds once and
//! shares between every user. It holds the store and the hooks; each call
//! brings who acts and on which scope.
//!
//! The kit takes Engine commands, not verb text: every door (the Agent tool
//! of the Node SDK, and the HTTP door in [`crate::http`]) turns its input
//! into a [`Command`] and calls [`HostKit::run`], so the hooks see the same
//! thing from every door.

use crate::documents::scope_version;
use crate::port_exec::{at_version, execute_with, AtVersion, BeforeCommit, ExecError, PortExecution, WriteAt};
use speclink_core::command::{
    Command, CommandError, CommandOutcome, ErrorCode, ExecutionContext, WorkflowConfigWriteOutcome,
};
use speclink_core::config::ChangedKey;
use speclink_protocol::auth::AuthMethodsResponse;
use speclink_protocol::events::EventTransport;
use speclink_store_port::{Actor, DocKey, Event, Expected, ScopeId, StorePort, Version};
use std::sync::Arc;

/// Who acts: an id that stays when the user is renamed, and the name that
/// documents are stamped with. Neither is empty or only white space.
///
/// A host can also give a shown name ([`Identity::shown_as`]): the name the
/// HTTP door gives back to the client (the handshake's actor, the identity
/// queries). It never reaches a commit or a stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    actor: Actor,
    shown: Option<String>,
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
        Ok(Identity { actor, shown: None })
    }

    /// This identity with a shown name for the HTTP door. Empty or only
    /// white space is refused.
    pub fn shown_as(self, name: impl Into<String>) -> Result<Identity, InvalidIdentity> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(InvalidIdentity { field: "shown name" });
        }
        Ok(Identity { shown: Some(name), ..self })
    }

    /// The name the HTTP door shows: the shown name, else the display name.
    pub fn shown_name(&self) -> &str {
        self.shown.as_deref().unwrap_or(self.display_name())
    }

    /// The id the commit record carries.
    pub fn id(&self) -> &str {
        self.actor.id()
    }

    /// The name the commit record and the document stamps carry.
    pub fn display_name(&self) -> &str {
        self.actor.display_name()
    }

    /// The commit record's actor.
    pub(crate) fn actor(&self) -> &Actor {
        &self.actor
    }
}

/// An identity field that is empty or only white space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidIdentity {
    field: &'static str,
}

impl InvalidIdentity {
    /// `id`, `display name` or `shown name`.
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

/// A project a caller may use, as the host's scope function gives it
/// (design D2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    /// The key the URL names the project by.
    pub key: String,
    pub name: String,
    pub repos: Vec<RepoEntry>,
}

/// A repo of a [`ProjectEntry`]: the store scope it reads and writes, and
/// what the caller may do there. Both switches are stated by the host; there
/// is no default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoEntry {
    /// The key the `X-Speclink-Repo` header names the repo by.
    pub key: String,
    pub name: String,
    /// The only scope a request for this repo reaches.
    pub scope: ScopeId,
    /// False: every request that is not a read is refused with 403.
    pub can_write: bool,
    /// False: a workflow-config write that changes a policy key is refused
    /// with 403.
    pub can_write_policy: bool,
}

/// The host's function from an identity to the projects it may use. Asked
/// once per HTTP request and never cached by the kit. An `Err` answers the
/// request with 503; its text stays inside the host.
pub type ScopesFn = Box<dyn Fn(&Identity) -> Result<Vec<ProjectEntry>, String> + Send + Sync>;

/// What the HTTP door needs besides the store and the hooks (design D2).
/// Every field is optional: a kit built without them answers 404 on the
/// paths that need them.
#[derive(Default)]
pub struct HostKitOptions {
    /// Absent: the scope list and every project path are 404.
    pub scopes: Option<ScopesFn>,
    /// The login methods the host supports. Absent: `auth-methods` is 404.
    pub auth_methods: Option<AuthMethodsResponse>,
    /// The push streams the host serves itself; the handshake declares them
    /// before the polling fallback. Empty: polling only.
    pub push_transports: Vec<EventTransport>,
}

/// The fixed text of a refusal that gives no reason.
const NO_REASON: &str = "refused by host";

/// One kit for every user: the store, the hooks and the door's options,
/// with no state of its own, so one value serves every request.
pub struct HostKit {
    pub(crate) store: Arc<dyn StorePort>,
    hooks: Hooks,
    pub(crate) options: HostKitOptions,
}

impl HostKit {
    pub fn new(store: Arc<dyn StorePort>, hooks: Hooks) -> HostKit {
        HostKit::with_options(store, hooks, HostKitOptions::default())
    }

    /// A kit whose HTTP door ([`HostKit::handle`]) also has `options`.
    pub fn with_options(store: Arc<dyn StorePort>, hooks: Hooks, options: HostKitOptions) -> HostKit {
        HostKit { store, hooks, options }
    }

    /// Run `cmd` for `caller`. Without hooks the result is what
    /// [`crate::port_exec::execute`] gives for the same command; the commit
    /// records the caller's identity and the command's name.
    pub fn run(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError> {
        match self.run_with(caller, cmd, &RunOptions::default())?.at {
            AtVersion::Done(execution) => Ok(execution),
            AtVersion::Moved { .. } => unreachable!("a run with no expected version never sees the scope move"),
        }
    }

    /// [`HostKit::run`] with what the HTTP door adds; see [`RunOptions`].
    pub(crate) fn run_with(&self, caller: &Caller, cmd: Command, options: &RunOptions) -> Result<Ran, ExecError> {
        let command = cmd.name();
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
            repo: options.repo.clone(),
            ..Default::default()
        };
        let (scope, actor) = (&caller.scope, &caller.identity.actor);
        let write_guard = options.write_at.as_ref().map(|(key, expected)| WriteAt {
            port: self.store.as_ref(),
            key: key.clone(),
            expected: *expected,
        });
        let store: &dyn StorePort = match &write_guard {
            Some(guard) => guard,
            None => self.store.as_ref(),
        };
        let run = |cmd: Command, check: Option<BeforeCommit<'_>>| match options.expected {
            None => execute_with(store, scope, &engine_ctx, actor, cmd, check).map(AtVersion::Done),
            Some(expected) => {
                at_version(store, scope, expected, |port| execute_with(port, scope, &engine_ctx, actor, cmd, check))
            }
        };

        let (at, before) = if is_workflow_config_write(&cmd) {
            // Checked once the Engine computed the write, before the commit:
            // the keys seen are the keys that are committed. The caller's
            // permission first, then the hook.
            let checked = std::cell::Cell::new(false);
            let check: BeforeCommit<'_> = &|outcome| {
                checked.set(true);
                let keys = config_write(outcome).map(|write| write.changed_keys.as_slice()).unwrap_or_default();
                if options.policy_locked {
                    refuse_policy_change(keys)?;
                }
                self.ask(caller, command, &params, Some(keys.iter().map(ToString::to_string).collect()))
            };
            let result = run(cmd, Some(check));
            if !checked.get() {
                // The write ended before the check — it failed, or the scope
                // had already left the expected version: still asked exactly
                // once, and a refusal wins over that ending.
                self.ask(caller, command, &params, Some(Vec::new()))?;
            }
            (result?, None)
        } else {
            // Asked before the Engine runs: a refused caller cannot tell
            // "not found" from "refused", and the store sees no call.
            self.ask(caller, command, &params, None)?;
            let before = match options.version_first {
                true => Some(scope_version(store, scope).map_err(ExecError::Store)?),
                false => None,
            };
            (run(cmd, None)?, before)
        };

        if let AtVersion::Done(execution) = &at {
            if let Some(scope_version) = execution.scope_version {
                self.watch(caller, command, scope_version, execution.commit_events.clone());
            }
        }
        Ok(Ran { at, before })
    }

    /// Ask the before hook about `command`; a refusal is the `refused`
    /// error with the hook's reason.
    pub(crate) fn ask(
        &self,
        caller: &Caller,
        command: &'static str,
        params: &serde_json::Value,
        changed_keys: Option<Vec<String>>,
    ) -> Result<(), ExecError> {
        let Some(hook) = &self.hooks.before_command else {
            return Ok(());
        };
        let info = BeforeCommand { caller: caller.clone(), command, params: params.clone(), changed_keys };
        match hook(&info) {
            Decision::Allow => Ok(()),
            Decision::Deny { reason } => {
                let reason = if reason.trim().is_empty() { NO_REASON.to_string() } else { reason };
                Err(ExecError::Command(CommandError::new(ErrorCode::Refused, reason)))
            }
        }
    }

    /// Tell the after hook about a commit that landed. A panic in the hook
    /// is caught: the commit stands.
    pub(crate) fn watch(&self, caller: &Caller, command: &'static str, scope_version: Version, events: Vec<Event>) {
        if let Some(hook) = &self.hooks.after_commit {
            let info = AfterCommit { caller: caller.clone(), command, scope_version, events };
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| hook(&info)));
        }
    }
}

/// What a run adds for the HTTP door. The default is a plain run.
#[derive(Default)]
pub(crate) struct RunOptions {
    /// The repo key a task completion records in its evidence.
    pub repo: Option<String>,
    /// Commit only while the scope stays at this version.
    pub expected: Option<Version>,
    /// A workflow-config write that changes a policy key is refused.
    pub policy_locked: bool,
    /// Commit a write of this document only over this version of it: the
    /// one the client's `If-Match` named.
    pub write_at: Option<(DocKey, Expected)>,
    /// Take the scope version before the Engine reads anything
    /// ([`Ran::before`]), for an ETag that is never newer than the content.
    pub version_first: bool,
}

/// What [`HostKit::run_with`] did.
pub(crate) struct Ran {
    pub at: AtVersion,
    /// The scope version before the Engine read anything, when
    /// [`RunOptions::version_first`] asked for it. A workflow-config write
    /// has none: it compares its own expected version.
    pub before: Option<Version>,
}

/// The source of the refusal of a policy change by a caller who may not
/// make one; the HTTP door answers it with 403.
#[derive(Debug)]
pub(crate) struct PolicyChangeRefused;

impl std::fmt::Display for PolicyChangeRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the caller may not change workflow policy")
    }
}

impl std::error::Error for PolicyChangeRefused {}

/// Refuse a workflow-config write whose changed keys hold a policy key: the
/// schema or one of the policy fields.
fn refuse_policy_change(changed: &[ChangedKey]) -> Result<(), ExecError> {
    let policy: Vec<String> = changed
        .iter()
        .filter(|key| matches!(key, ChangedKey::Schema | ChangedKey::Policy(_)))
        .map(ToString::to_string)
        .collect();
    if policy.is_empty() {
        return Ok(());
    }
    Err(ExecError::Command(CommandError {
        code: ErrorCode::Refused,
        message: format!(
            "this write changes workflow policy ({}); your access to this repo does not allow policy changes",
            policy.join(", ")
        ),
        source: Some(anyhow::Error::new(PolicyChangeRefused)),
    }))
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

/// The write of a workflow-config command's outcome.
fn config_write(outcome: &CommandOutcome) -> Option<&WorkflowConfigWriteOutcome> {
    match outcome {
        CommandOutcome::WorkflowConfigEdit(write) | CommandOutcome::WorkflowConfigReplace(write) => Some(write),
        _ => None,
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

//! The host kit (design D1): what a host that embeds speclink builds once and
//! shares between every user. It holds the store and the hooks; each call
//! brings who acts and on which scope.
//!
//! The kit takes Engine commands, not verb text: every door (the Agent tool
//! of the Node SDK, and the HTTP door in [`crate::http`]) turns its input
//! into a [`Command`] and calls [`HostKit::run`], so the hooks see the same
//! thing from every door. A host can attach free fields to a call
//! ([`HostKit::run_with_fields`], [`HostKit::handle_with_fields`]).

use crate::documents::scope_version;
use crate::port_exec::{at_version, execute_with, now, AtVersion, BeforeCommit, ExecError, PortExecution, WriteAt};
use crate::read_only::is_read;
use crate::transfer::{self, ImportOptions, ImportReport, ScopeBundle, TransferError};
use speclink_core::command::{
    Command, CommandError, CommandOutcome, ErrorCode, ExecutionContext, WorkflowConfigWriteOutcome,
};
use speclink_core::config::ChangedKey;
use speclink_core::fields::Fields;
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

/// What the before-command hook reads with: it runs commands that write
/// nothing ([`HostKit::read`]), as this call's caller on this call's scope.
/// Only the kit builds one, so a hook cannot read as another identity or on
/// another scope.
///
/// A read here and the read of the command the hook is asked about are two
/// calls to the store: a commit between them makes them differ. The Engine's
/// own checks judge by what the command reads.
pub struct Reader<'a> {
    kit: &'a HostKit,
    caller: &'a Caller,
}

impl Reader<'_> {
    /// [`HostKit::read`] for this call's caller.
    pub fn read(&self, cmd: Command) -> Result<PortExecution, ExecError> {
        self.kit.read(self.caller, cmd)
    }
}

/// The source of [`HostKit::read`]'s refusal of a command that writes. A door
/// knows the refusal by this type, not by the message text, and can name the
/// verb its caller wrote in place of the command.
#[derive(Debug)]
pub struct NotARead;

impl std::fmt::Display for NotARead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the command is not of a kind that only reads")
    }
}

impl std::error::Error for NotARead {}

pub type BeforeCommandHook = Box<dyn Fn(&BeforeCommand, &Reader<'_>) -> Decision + Send + Sync>;
pub type AfterCommitHook = Box<dyn Fn(&AfterCommit) + Send + Sync>;

/// The hooks of a kit; each one is optional.
#[derive(Default)]
pub struct Hooks {
    /// Asked exactly once per call, before anything reaches the store's
    /// commit. Absent: every call is allowed. What the hook reads with its
    /// [`Reader`] does not ask it again.
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
    /// The skills the door gives a client, by name (`apply`). Absent: every
    /// skill of the Engine. A host chooses which skills; it has no way to
    /// change their text. An empty list, or a name the Engine does not have,
    /// builds no kit ([`InvalidSkills`]).
    pub skills: Option<Vec<String>>,
    /// The most an import takes, in bytes. Absent:
    /// [`DEFAULT_IMPORT_LIMIT_BYTES`]. The HTTP door compares it with the
    /// request body before it parses anything (413);
    /// [`HostKit::import_scope`] compares it with the documents' content.
    ///
    /// The door gets a body the host's web framework already read, so the
    /// framework's own body limit for the import route must be at least this
    /// value. The kit neither compresses nor decompresses: that is the
    /// framework's route layer. An export has no limit.
    pub import_limit_bytes: Option<usize>,
}

/// The import limit of a kit whose host gave none: 32 MiB.
pub const DEFAULT_IMPORT_LIMIT_BYTES: usize = 32 * 1024 * 1024;

/// A [`HostKitOptions::skills`] list no kit is built with: it names skills the
/// Engine does not have, or it is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSkills {
    names: Vec<String>,
}

impl InvalidSkills {
    /// The names the Engine does not have, in the order the host gave them;
    /// none when the list was empty.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Check a skill list as [`HostKit::with_options`] does: a host that
    /// builds its kit later (one per request) refuses a wrong list at once.
    ///
    /// An empty list is refused too. A kit that gives no skill would have
    /// every checkout that updates from it lose the skills it has; a host
    /// that wants every skill gives no list.
    pub fn check(listed: Option<&[String]>) -> Result<(), InvalidSkills> {
        let Some(listed) = listed else {
            return Ok(());
        };
        let registry = speclink_core::skills::registry();
        let names: Vec<String> =
            listed.iter().filter(|name| !registry.iter().any(|skill| skill.name == name.as_str())).cloned().collect();
        match names.is_empty() && !listed.is_empty() {
            true => Ok(()),
            false => Err(InvalidSkills { names }),
        }
    }
}

impl std::fmt::Display for InvalidSkills {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.names.is_empty() {
            return f.write_str("the skill list is empty (leave it out to give every skill)");
        }
        let unknown: Vec<String> = self.names.iter().map(|name| format!("'{name}'")).collect();
        let known: Vec<&str> = speclink_core::skills::registry().iter().map(|skill| skill.name).collect();
        write!(f, "unknown skill name: {} (the skills are: {})", unknown.join(", "), known.join(", "))
    }
}

impl std::error::Error for InvalidSkills {}

/// The fixed text of a refusal that gives no reason.
const NO_REASON: &str = "refused by host";

/// Every command name a host's before hook and its store's commit can see,
/// once each: the Engine's ([`speclink_core::command::COMMAND_NAMES`]), then
/// the kit's own export and import, then the HTTP door's write of the board
/// order (spec node-sdk: SDK 匯出命令名稱清單). The order is no contract.
pub fn command_names() -> Vec<&'static str> {
    let mut names = speclink_core::command::COMMAND_NAMES.to_vec();
    names.extend([transfer::EXPORT_COMMAND, transfer::IMPORT_COMMAND, crate::documents::PUT_BOARD_ORDER_COMMAND]);
    names
}

/// One kit for every user: the store, the hooks and the door's options,
/// with no state of its own, so one value serves every request.
pub struct HostKit {
    pub(crate) store: Arc<dyn StorePort>,
    hooks: Hooks,
    pub(crate) options: HostKitOptions,
}

impl HostKit {
    pub fn new(store: Arc<dyn StorePort>, hooks: Hooks) -> HostKit {
        HostKit { store, hooks, options: HostKitOptions::default() }
    }

    /// A kit whose HTTP door ([`HostKit::handle`]) also has `options`. A
    /// skill list that is empty, or names a skill the Engine does not have,
    /// builds no kit.
    pub fn with_options(
        store: Arc<dyn StorePort>,
        hooks: Hooks,
        options: HostKitOptions,
    ) -> Result<HostKit, InvalidSkills> {
        InvalidSkills::check(options.skills.as_deref())?;
        Ok(HostKit { store, hooks, options })
    }

    /// Run `cmd` for `caller`. Without hooks the result is what
    /// [`crate::port_exec::execute`] gives for the same command; the commit
    /// records the caller's identity and the command's name.
    pub fn run(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError> {
        self.run_with_fields(caller, cmd, &Fields::new())
    }

    /// [`HostKit::run`] with free fields the host attaches to this call, from
    /// its own program: for example where the call came from. They are
    /// written only when the call creates a change or a discussion (new
    /// change, new discussion, promote), in the commit that creates it, and
    /// they win over a field of the same key that the verb text or the HTTP
    /// request gave. Every other call ignores them, so a host can attach the
    /// same fields to every call. Attached fields that break the rules of
    /// free fields are the host's own defect: a call that would write them
    /// fails as `invalid_argv` before the hook is asked, with nothing written.
    pub fn run_with_fields(&self, caller: &Caller, cmd: Command, fields: &Fields) -> Result<PortExecution, ExecError> {
        match self.run_with(caller, cmd, &RunOptions { fields: fields.clone(), ..RunOptions::default() })?.at {
            AtVersion::Done(execution) => Ok(execution),
            AtVersion::Moved { .. } => unreachable!("a run with no expected version never sees the scope move"),
        }
    }

    /// [`HostKit::run`] with what the HTTP door adds; see [`RunOptions`].
    pub(crate) fn run_with(&self, caller: &Caller, cmd: Command, options: &RunOptions) -> Result<Ran, ExecError> {
        // Before the hook: it sees the fields that will be written.
        let cmd = attach_fields(cmd, &options.fields)?;
        let command = cmd.name();
        let params = match &self.hooks.before_command {
            Some(_) => cmd.params(),
            None => serde_json::Value::Null,
        };
        let cmd = without_local_facts(cmd);
        let engine_ctx = engine_context(caller, options.repo.clone());
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

    /// Run `cmd` for `caller` when it is of a kind that writes nothing
    /// ([`crate::read_only::is_read`]); any other command is `invalid_argv`
    /// before the store is called. The result is what [`HostKit::run`] gives
    /// for the same command, and no hook is asked or told: this is what the
    /// before-command hook reads with ([`Reader`]).
    pub fn read(&self, caller: &Caller, cmd: Command) -> Result<PortExecution, ExecError> {
        if !is_read(&cmd) {
            return Err(ExecError::Command(CommandError {
                code: ErrorCode::InvalidArgv,
                message: format!("read accepts only commands that write nothing; '{}' is not one of them", cmd.name()),
                source: Some(anyhow::Error::new(NotARead)),
            }));
        }
        let engine_ctx = engine_context(caller, None);
        let cmd = without_local_facts(cmd);
        execute_with(self.store.as_ref(), &caller.scope, &engine_ctx, caller.identity.actor(), cmd, None)
    }

    /// The most an import takes, in bytes ([`HostKitOptions::import_limit_bytes`]).
    pub fn import_limit_bytes(&self) -> usize {
        self.options.import_limit_bytes.unwrap_or(DEFAULT_IMPORT_LIMIT_BYTES)
    }

    /// Every document of the caller's scope ([`transfer::export`]). The
    /// before hook is asked first, as `scope-export` with no params; a
    /// refusal ends the call as `refused` before the store is called. The
    /// kit does not ask who may export: the host decides who calls this (the
    /// HTTP door asks for the write switch).
    pub fn export_scope(&self, caller: &Caller) -> Result<ScopeBundle, ExecError> {
        Ok(self.export_asked(caller)??)
    }

    /// [`Self::export_scope`], with the export's own failure apart from the
    /// hook's refusal: the HTTP door tells a client why an export found no
    /// still moment, which the store's failures do not say.
    pub(crate) fn export_asked(&self, caller: &Caller) -> Result<Result<ScopeBundle, TransferError>, ExecError> {
        self.ask(caller, transfer::EXPORT_COMMAND, &serde_json::json!({}), None)?;
        Ok(transfer::export(self.store.as_ref(), &caller.scope))
    }

    /// Write `bundle` into the caller's scope in the mode `options` names
    /// ([`transfer::import_with`]): the default takes an empty scope only,
    /// the replace mode makes the scope equal the bundle, and a dry run of
    /// either commits nothing. Content over the kit's import limit is not
    /// valid, in both modes. The before hook is asked next, once, as
    /// `scope-import` with the number of documents, the bytes of their
    /// content, the mode (`create` or `replace`) and `dryRun`, and never the
    /// content; a refusal ends the call as `refused` before the store is
    /// called. A dry run is asked about too: its report names the keys of
    /// the scope. The after hook watches the commit, when there is one.
    pub fn import_scope(
        &self,
        caller: &Caller,
        bundle: ScopeBundle,
        options: ImportOptions,
    ) -> Result<ImportReport, ExecError> {
        let bytes: usize = bundle.documents.iter().map(|doc| doc.content.len()).sum();
        let limit = self.import_limit_bytes();
        if bytes > limit {
            return Err(TransferError::TooLarge(format!(
                "the bundle's content is {bytes} bytes, over the {limit}-byte import limit"
            ))
            .into());
        }
        let params = serde_json::json!({
            "documents": bundle.documents.len(),
            "bytes": bytes,
            "mode": options.mode.as_str(),
            "dryRun": options.dry_run,
        });
        self.ask(caller, transfer::IMPORT_COMMAND, &params, None)?;
        let at = now().map_err(ExecError::Store)?;
        let (store, actor) = (self.store.as_ref(), caller.identity.actor());
        let (report, events) = transfer::import_with(store, &caller.scope, actor, bundle, at, options)?;
        if !events.is_empty() {
            self.watch(caller, transfer::IMPORT_COMMAND, report.scope_version, events);
        }
        Ok(report)
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
        match hook(&info, &Reader { kit: self, caller }) {
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
    /// The free fields the host attached to this call
    /// ([`HostKit::run_with_fields`]).
    pub fields: Fields,
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

/// The Engine's context for `caller`. The kit reads no process environment
/// and no checkout: the Engine gets the caller's name and identifier, and
/// nothing else.
fn engine_context(caller: &Caller, repo: Option<String>) -> ExecutionContext {
    ExecutionContext {
        actor: Some(caller.identity.display_name().to_string()),
        actor_id: Some(caller.identity.id().to_string()),
        repo,
        ..Default::default()
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

/// `cmd` with the host's attached fields: merged into a command that creates
/// a change or a discussion, the host's value over the command's own; any
/// other command comes back as it was. This is the one merge, for both doors.
/// The host can add its fields and nothing else: no other part of the command
/// changes. A merged set outside the rules fails here, as `invalid_argv`.
fn attach_fields(mut cmd: Command, attached: &Fields) -> Result<Command, ExecError> {
    if attached.is_empty() {
        return Ok(cmd);
    }
    let Some(own) = cmd.creation_fields_mut() else {
        return Ok(cmd);
    };
    own.extend(attached.iter().map(|(key, value)| (key.clone(), value.clone())));
    speclink_core::fields::check(own)
        .map_err(|e| ExecError::Command(CommandError::new(ErrorCode::InvalidArgv, e.to_string())))?;
    Ok(cmd)
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

    /// spec node-sdk「SDK 匯出命令名稱清單」: the Engine's names and the
    /// kit's own three, each once.
    #[test]
    fn command_names_are_the_engines_and_the_kits_three_each_once() {
        let engine = speclink_core::command::COMMAND_NAMES;
        let names = command_names();
        assert_eq!(names.len(), engine.len() + 3, "{names:?}");
        let unique: std::collections::BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len(), "a name appears twice: {names:?}");
        for name in engine.iter().chain(&["scope-export", "scope-import", "put-board-order"]) {
            assert!(unique.contains(name), "{name} is missing");
        }
    }

    // --- the reason of an import failure (sdk-command-names-and-transfer-reasons) ---

    use crate::projection::content_digest;
    use crate::transfer::{BundleDocument, ImportMode, ImportOptions, TransferReason};

    const CREATE: ImportOptions = ImportOptions { mode: ImportMode::Create, dry_run: false };
    const REPLACE: ImportOptions = ImportOptions { mode: ImportMode::Replace, dry_run: false };

    fn kit_with(limit: Option<usize>, before_command: Option<BeforeCommandHook>) -> HostKit {
        let options = HostKitOptions { import_limit_bytes: limit, ..HostKitOptions::default() };
        let hooks = Hooks { before_command, after_commit: None };
        HostKit::with_options(Arc::new(speclink_store_port::MemoryStorePort::new()), hooks, options).unwrap()
    }

    fn alice() -> Caller {
        Caller { identity: Identity::new("u_42", "Alice").unwrap(), scope: ScopeId::new("default/main").unwrap() }
    }

    fn bundle_of(content: &str) -> ScopeBundle {
        let document =
            BundleDocument { key: "config.yaml".to_string(), content: content.to_string(), digest: content_digest(content) };
        ScopeBundle { format_version: transfer::FORMAT_VERSION, scope_version: None, documents: vec![document] }
    }

    /// The code, the message and the reason of a failed import.
    fn failure(result: Result<ImportReport, ExecError>) -> (ErrorCode, String, Option<TransferReason>) {
        match result {
            Err(ExecError::Command(error)) => {
                let reason = transfer::reason_of(&error);
                (error.code, error.message, reason)
            }
            other => panic!("expected a command failure, got {other:?}"),
        }
    }

    #[test]
    fn an_import_over_the_limit_names_bundle_too_large() {
        let kit = kit_with(Some(1_024), None);
        let (code, message, reason) = failure(kit.import_scope(&alice(), bundle_of(&"x".repeat(2_000)), CREATE));
        assert_eq!(code, ErrorCode::InvalidArgv);
        assert_eq!(message, "the bundle's content is 2000 bytes, over the 1024-byte import limit");
        assert_eq!(reason, Some(TransferReason::BundleTooLarge));
    }

    #[test]
    fn an_import_into_a_scope_that_holds_documents_names_scope_not_empty() {
        let kit = kit_with(None, None);
        kit.import_scope(&alice(), bundle_of("schema: spec-driven\n"), CREATE).expect("the first import");
        let (code, message, reason) = failure(kit.import_scope(&alice(), bundle_of("schema: spec-driven\n"), CREATE));
        assert_eq!(code, ErrorCode::Refused);
        assert_eq!(message, "import (create-new): the target scope already holds documents");
        assert_eq!(reason, Some(TransferReason::ScopeNotEmpty));
    }

    #[test]
    fn an_import_the_hook_refuses_has_no_reason() {
        let deny: BeforeCommandHook = Box::new(|_, _| Decision::Deny { reason: "not allowed".to_string() });
        let kit = kit_with(None, Some(deny));
        let (code, message, reason) = failure(kit.import_scope(&alice(), bundle_of("schema: spec-driven\n"), CREATE));
        assert_eq!((code, message.as_str(), reason), (ErrorCode::Refused, "not allowed", None));
    }

    // --- a replace with no documents (import-replace-mode D3); the scope that
    // changed during a replace is in tests/transfer.rs, on the shared recorder ---

    #[test]
    fn a_replace_with_no_documents_names_empty_bundle() {
        let kit = kit_with(None, None);
        kit.import_scope(&alice(), bundle_of("schema: spec-driven\n"), CREATE).expect("the first import");
        let empty = ScopeBundle { documents: Vec::new(), ..bundle_of("") };
        let (code, message, reason) = failure(kit.import_scope(&alice(), empty, REPLACE));
        assert_eq!(code, ErrorCode::InvalidArgv);
        assert_eq!(
            message,
            "import (replace): the bundle holds no documents — a replace with it would remove every document of the scope"
        );
        assert_eq!(reason, Some(TransferReason::EmptyBundle));
    }
}

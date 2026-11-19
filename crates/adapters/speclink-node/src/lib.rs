//! speclink-node: napi-rs bindings for the speclink engine (@speclink/engine).
//!
//! The engine core stays synchronous; every `dispatch` runs on a dedicated
//! worker thread and resolves a JS Promise, so the JS event loop is never
//! blocked. Results are returned as an `{ok, value | code+message}` envelope
//! that the JS wrapper (`index.js`) unwraps into a return value or a thrown
//! `Error` — this keeps error shaping (the `code` property) in JS, where it
//! is idiomatic, while the semantic message and code are produced here.

mod host;
mod js_bridge;
mod port_bridge;
pub mod render;

use napi::bindgen_prelude::*;
use napi::{JsFunction, JsObject};
use napi_derive::napi;
use speclink_core as core;
use std::path::PathBuf;
use std::sync::Arc;

/// A dispatch failure: `message` is the same semantic text the CLI prints,
/// `code` classifies it (exit-code / 409-reason category — see docs/sdk-node.md).
pub(crate) struct DispatchError {
    code: String,
    message: String,
}

impl DispatchError {
    pub(crate) fn new(code: &str, message: impl Into<String>) -> DispatchError {
        DispatchError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for DispatchError {
    fn from(e: anyhow::Error) -> DispatchError {
        DispatchError::new("error", format!("{e}"))
    }
}

impl From<core::command::CommandError> for DispatchError {
    fn from(e: core::command::CommandError) -> DispatchError {
        DispatchError::new(e.code.as_str(), e.message)
    }
}

pub(crate) type DispatchResult = std::result::Result<serde_json::Value, DispatchError>;

/// Run `work` on a background thread and resolve the returned Promise with
/// its value: the JS event loop never waits for it.
pub(crate) fn on_worker(env: Env, work: impl FnOnce() -> serde_json::Value + Send + 'static) -> Result<JsObject> {
    let (deferred, promise) = env.create_deferred()?;
    std::thread::spawn(move || {
        let value = work();
        deferred.resolve(move |_| Ok(value));
    });
    Ok(promise)
}

/// CLI parity: `status` with no name in a project with no change at all is a
/// notice, not an error — the same text the CLI prints with exit code 0. Both
/// doors read it off the command's own failure, by its type.
pub(crate) fn status_notice(status_without_name: bool, e: &core::command::CommandError) -> Option<serde_json::Value> {
    (status_without_name && e.is_no_active_changes())
        .then(|| serde_json::json!({ "output": speclink_verbs::render::no_active_changes().stdout }))
}

/// The envelope a dispatch Promise resolves to; index.js unwraps it.
pub(crate) fn envelope(result: DispatchResult) -> serde_json::Value {
    match result {
        Ok(value) => serde_json::json!({ "ok": true, "value": value }),
        Err(e) => serde_json::json!({ "ok": false, "code": e.code, "message": e.message }),
    }
}

/// Route a typed command through the engine runtime on the project's files.
/// Events are not surfaced through dispatch (yet) — the envelope shape is
/// frozen; they will ride the outbox once event persistence lands.
fn run_engine(
    workspace: &core::workspace::Workspace,
    actor: Option<&str>,
    cmd: core::command::Command,
) -> std::result::Result<core::command::CommandOutcome, core::command::CommandError> {
    let store = speclink_fs::FsStore::new(&workspace.root, &workspace.spec_dir_name);
    // Host boundary: identity and the SPECLINK_* env layer are resolved here
    // and injected — the engine runtime only ever consumes the context. An
    // actor bound at construction wins outright; without one, the identity
    // is the project's git identity.
    let ctx = core::command::ExecutionContext {
        actor: actor
            .map(str::to_string)
            .or_else(|| speclink_host::context::git_identity(&workspace.root)),
        // The engine's own door binds one identity per instance and has no
        // stable identifier: a claim judges its holder by the name.
        actor_id: None,
        // The repo binding completion evidence records (`task done`): the
        // local zero-configuration case, so it takes the binding the CLI takes.
        repo: Some(speclink_host::binding::local_default_binding().repo.as_str().to_string()),
        env: speclink_host::policy::process_env_overrides(),
        workspace: Some(workspace.clone()),
        user_config_dir: Some(speclink_host::context::global_config_dir()),
    };
    let (outcome, _events) = core::command::execute(&store, &ctx, cmd)?;
    Ok(outcome)
}

#[napi]
pub struct Engine {
    /// The local project: its root and the name of its spec folder. The
    /// built-in filesystem store over it is the engine's one storage.
    workspace: core::workspace::Workspace,
    /// The operator identity, bound once at construction — one instance, one
    /// identity. dispatch carries no identity parameter, so call-time forgery
    /// has no surface. A host that serves several users uses the host kit,
    /// where every call brings its identity.
    actor: Option<Arc<str>>,
    /// A stamp rewrites the whole change metadata, and a workflow-config write
    /// rewrites the whole config document: both are read-modify-write with no
    /// version check. Two of them at once on one instance would clobber each
    /// other, so they serialize here. The lock covers this instance only.
    rewrite_lock: Arc<std::sync::Mutex<()>>,
}

/// Construction-time normalization: a blank actor reads as "not given".
fn normalize_actor(actor: Option<String>) -> Option<Arc<str>> {
    let a = actor?;
    let t = a.trim();
    if t.is_empty() {
        None
    } else {
        Some(Arc::from(t))
    }
}

#[napi]
impl Engine {
    /// argv dispatch on a background worker thread. Resolves to the envelope
    /// unwrapped by the JS wrapper; never blocks the JS event loop.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn dispatch(&self, env: Env, argv: Vec<String>, stdin: Option<String>) -> Result<JsObject> {
        let workspace = self.workspace.clone();
        let actor = self.actor.clone();
        let rewrite_lock = self.rewrite_lock.clone();
        on_worker(env, move || {
            caught(|| run_dispatch(&workspace, actor.as_deref(), &rewrite_lock, &argv, stdin.as_deref()))
        })
    }

    /// Every document of the local project as a bundle by key, on a
    /// background worker thread. Resolves to the envelope index.js unwraps:
    /// the JSON text of the bundle, which index.js parses — a large bundle
    /// crosses as one string, as the host's export does.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn export_scope(&self, env: Env) -> Result<JsObject> {
        let workspace = self.workspace.clone();
        on_worker(env, move || caught(|| export_local(&workspace)))
    }
}

/// The envelope of `work`. A panic in it is an Error, never a process abort.
fn caught(work: impl FnOnce() -> DispatchResult) -> serde_json::Value {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    envelope(outcome.unwrap_or_else(|payload| {
        Err(DispatchError::new("panic", format!("speclink engine panicked: {}", panic_message(&payload))))
    }))
}

/// The local project as the JSON text of a bundle by key, its documents in
/// byte order of the keys and with no scope version: a local project has
/// none (engine-export-scope D2). The documents are the ones Speclink
/// Desktop's move to a server sends. Any failure is the whole answer.
fn export_local(workspace: &core::workspace::Workspace) -> DispatchResult {
    use speclink_host::transfer;
    use speclink_protocol::query::{ImportBundle, ImportScope};
    let failed = |message: String| DispatchError::new("error", message);
    let store = speclink_fs::FsStore::new(&workspace.root, &workspace.spec_dir_name);
    let documents = transfer::local_documents(workspace, &store, &speclink_host::context::global_config_dir())
        .map_err(failed)?;
    // The conversion does not read a first version's target: a local
    // project has none, so it stays empty.
    let first = ImportBundle {
        format_version: transfer::FIRST_VERSION,
        scope: ImportScope { project: String::new(), repo: String::new() },
        project_revision: 0,
        documents,
    };
    let mut bundle = transfer::from_first_version(first).map_err(|e| failed(e.to_string()))?;
    bundle.documents.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(serde_json::Value::String(serde_json::to_string(&bundle).expect("a bundle serializes to JSON")))
}

pub(crate) fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Build an engine over a local project folder.
#[napi(js_name = "engineFromFs")]
pub fn engine_from_fs(
    root: String,
    spec_dir: Option<String>,
    actor: Option<String>,
) -> Result<Engine> {
    if root.trim().is_empty() {
        return Err(Error::from_reason(
            "createEngine: store.root must be a non-empty path",
        ));
    }
    let root = PathBuf::from(root);
    let spec_dir = match spec_dir {
        Some(dir) => dir,
        None => {
            // The same resolution the CLI runs: an explicit `spec_dir` first.
            let explicit = core::config::AppConfig::load(&root.join(".speclink.yaml"))
                .map_err(|e| Error::from_reason(format!("createEngine: {e}")))?
                .spec_dir;
            core::workspace::resolve_spec_dir(&root, explicit.as_deref()).0
        }
    };
    Ok(Engine {
        workspace: core::workspace::Workspace { root, spec_dir_name: spec_dir },
        actor: normalize_actor(actor),
        rewrite_lock: Arc::new(std::sync::Mutex::new(())),
    })
}

/// Run the store contract's conformance checks on a host's JS store port, on
/// a background thread. `invoker` is index.js's port invoker over the store's
/// three functions plus the harness's `newScope` and `recordedEvents`;
/// `keeps_events` says whether `recordedEvents` was given.
#[napi(js_name = "runStoreConformance", ts_return_type = "Promise<unknown>")]
pub fn run_store_conformance(env: Env, invoker: JsFunction, keeps_events: bool) -> Result<JsObject> {
    let bridge = js_bridge::create_bridge(env, invoker)?;
    on_worker(env, move || port_bridge::run_conformance(bridge, keeps_events))
}

// --- argv router ---

/// argv → the parsing the CLI uses → the engine command → the output
/// definition the CLI uses → the envelope value. Every verb with an engine
/// command takes this one path.
fn run_dispatch(
    workspace: &core::workspace::Workspace,
    actor: Option<&str>,
    rewrite_lock: &std::sync::Mutex<()>,
    argv: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let parsed = parse_verb(argv)?;
    let cmd = command_of(&parsed, stdin)?;
    run_verb(workspace, actor, rewrite_lock, &parsed, cmd)
}

/// Parse one verb's argv with the CLI's vocabulary. A usage or help text is
/// an `invalid_argv` failure carrying clap's text.
pub(crate) fn parse_verb(argv: &[String]) -> std::result::Result<speclink_verbs::ParsedVerb, DispatchError> {
    speclink_verbs::parse(argv).map_err(|e| DispatchError::new("invalid_argv", e.message))
}

/// The engine command of a parsed verb, under the stdin rules every door
/// shares. A verb with no engine command is refused with `invalid_argv`.
pub(crate) fn command_of(
    parsed: &speclink_verbs::ParsedVerb,
    stdin: Option<&str>,
) -> std::result::Result<core::command::Command, DispatchError> {
    use speclink_verbs::ToCommand;
    let name = parsed.name();
    let policy = parsed.stdin_policy();
    // Content passed without --stdin to a verb that reads stdin only with the
    // flag is refused: a forgotten flag must not drop the content silently.
    if stdin.is_some() && matches!(policy, speclink_verbs::StdinPolicy::WhenFlagged { flagged: false }) {
        return Err(DispatchError::new(
            "invalid_argv",
            format!("'{name}' reads stdin only with --stdin — add --stdin, or pass no stdin"),
        ));
    }
    // A host has no terminal: stdin content it passed counts as piped.
    let inputs = speclink_verbs::VerbInputs {
        stdin: if policy.reads(stdin.is_some()) { stdin.map(str::to_string) } else { None },
        ..Default::default()
    };
    match parsed.to_command(inputs)? {
        speclink_verbs::CommandTarget::Command(cmd) => {
            let flagged_without_stdin = stdin.is_none()
                && matches!(policy, speclink_verbs::StdinPolicy::WhenFlagged { flagged: true });
            // `--stdin` in argv but no stdin content: the body would read as
            // empty and clear the value, so a forgotten `options.stdin` is an
            // argv error (an empty string is the way to clear on purpose).
            if flagged_without_stdin && matches!(cmd, core::command::Command::WorkflowConfigEdit { .. }) {
                return Err(DispatchError::new(
                    "invalid_argv",
                    format!(
                        "'{name}' was given --stdin but no stdin content — pass options.stdin \
                         (an empty string clears the value)"
                    ),
                ));
            }
            Ok(cmd)
        }
        speclink_verbs::CommandTarget::LocalOnly { verb, reason } => Err(local_only(verb, reason)),
    }
}

/// A verb with no engine command runs only in the CLI; the message says why.
fn local_only(verb: &str, reason: speclink_verbs::LocalOnlyReason) -> DispatchError {
    let message = match reason {
        speclink_verbs::LocalOnlyReason::CliSkillText => {
            format!("'{verb}' prints skill text built into the CLI — dispatch does not carry it")
        }
        speclink_verbs::LocalOnlyReason::NeedsCheckout => {
            format!("'{verb}' only runs where a checkout exists — dispatch has none")
        }
    };
    DispatchError::new("invalid_argv", message)
}

/// Whether `cmd` writes the workflow config document: an edit that is not a
/// dry run, or a whole-document replace.
fn writes_workflow_config(cmd: &core::command::Command) -> bool {
    use core::command::Command;
    matches!(
        cmd,
        Command::WorkflowConfigEdit { dry_run: false, .. } | Command::WorkflowConfigReplace { .. }
    )
}

/// Run one verb's command and shape its envelope value: the verb's
/// structured value when it has one, else `{ output }` with its stdout text
/// (no color), plus `stderr` when the verb also has stderr text. `status`
/// with no name in a project with no change answers with the CLI's notice.
/// `claim` gets the engine's own refusal for a local project, as in the CLI.
fn run_verb(
    workspace: &core::workspace::Workspace,
    actor: Option<&str>,
    rewrite_lock: &std::sync::Mutex<()>,
    parsed: &speclink_verbs::ParsedVerb,
    cmd: core::command::Command,
) -> DispatchResult {
    use core::command::Command;
    let status_without_name = matches!(cmd, Command::Status { change: None, .. });
    // A stamp and a workflow-config write each rewrite one whole document
    // with no version check, so they serialize within this instance. A poisoned lock
    // only means another one panicked; the document itself is no more suspect
    // than after any failed dispatch, so proceed.
    let rewrites = parsed.is_stamp() || writes_workflow_config(&cmd);
    let _rewrite_guard = rewrites.then(|| rewrite_lock.lock().unwrap_or_else(|e| e.into_inner()));
    match run_engine(workspace, actor, cmd) {
        Ok(outcome) => value_of(parsed, &outcome),
        Err(e) => status_notice(status_without_name, &e).map_or_else(|| Err(DispatchError::from(e)), Ok),
    }
}

/// A verb's envelope value: its structured value when it has one, else
/// `{ output }` with its stdout text (no color), plus `stderr` when the verb
/// also has stderr text — the output definition the CLI uses. For an engine
/// on a local folder: a path in the text is a real one.
pub(crate) fn value_of(
    parsed: &speclink_verbs::ParsedVerb,
    outcome: &core::command::CommandOutcome,
) -> DispatchResult {
    Ok(envelope_value(speclink_verbs::render::render(parsed, outcome, TEXT)?))
}

/// [`value_of`] for a host: its documents are in a store, so the text of
/// `new change`, `discuss new` and `discuss promote` has no `Path:` line. A
/// structured value is the same one, its `path` key included.
pub(crate) fn embedded_value_of(
    parsed: &speclink_verbs::ParsedVerb,
    outcome: &core::command::CommandOutcome,
) -> DispatchResult {
    Ok(envelope_value(speclink_verbs::render::render_embedded(parsed, outcome, TEXT)?))
}

/// The human form with no color: a verb's structured value comes with it.
const TEXT: speclink_verbs::render::RenderOptions =
    speclink_verbs::render::RenderOptions { json: false, color: false };

fn envelope_value(rendered: speclink_verbs::render::Rendered) -> serde_json::Value {
    match rendered.json {
        Some(value) => value,
        None if rendered.stderr.is_empty() => serde_json::json!({ "output": rendered.stdout }),
        // A note the CLI prints to stderr (archive's zero-evidence note).
        None => serde_json::json!({ "output": rendered.stdout, "stderr": rendered.stderr }),
    }
}

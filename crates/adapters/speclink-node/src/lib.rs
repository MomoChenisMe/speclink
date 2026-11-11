//! speclink-node: napi-rs bindings for the speclink engine (@speclink/engine).
//!
//! The engine core stays synchronous; every `dispatch` runs on a dedicated
//! worker thread and resolves a JS Promise, so the JS event loop is never
//! blocked. Results are returned as an `{ok, value | code+message}` envelope
//! that the JS wrapper (`index.js`) unwraps into a return value or a thrown
//! `Error` — this keeps error shaping (the `code` property) in JS, where it
//! is idiomatic, while the semantic message and code are produced here.

pub mod render;
mod store_bridge;

use napi::bindgen_prelude::*;
use napi::{JsFunction, JsObject};
use napi_derive::napi;
use speclink_core as core;
use speclink_core::store::Store;
use std::path::PathBuf;
use std::sync::Arc;
use store_bridge::{BridgeFailure, JsStoreBridge};

/// A dispatch failure: `message` is the same semantic text the CLI prints,
/// `code` classifies it (exit-code / 409-reason category — see docs/sdk-node.md).
struct DispatchError {
    code: String,
    message: String,
}

impl DispatchError {
    fn new(code: &str, message: impl Into<String>) -> DispatchError {
        DispatchError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for DispatchError {
    fn from(e: anyhow::Error) -> DispatchError {
        // A store-method failure keeps its JS code and the method-name prefix.
        if let Some(f) = e.downcast_ref::<BridgeFailure>() {
            return DispatchError::from(f.clone());
        }
        DispatchError::new("error", format!("{e}"))
    }
}

impl From<BridgeFailure> for DispatchError {
    fn from(f: BridgeFailure) -> DispatchError {
        DispatchError {
            code: f.code.clone().unwrap_or_else(|| "store_error".to_string()),
            message: f.to_string(),
        }
    }
}

impl From<core::command::CommandError> for DispatchError {
    fn from(e: core::command::CommandError) -> DispatchError {
        // A store-bridge failure keeps its JS code and method-name prefix —
        // that taxonomy belongs to the envelope layer, not the command layer.
        if let Some(f) = e
            .source
            .as_ref()
            .and_then(|s| s.downcast_ref::<BridgeFailure>())
        {
            return DispatchError::from(f.clone());
        }
        DispatchError::new(e.code.as_str(), e.message)
    }
}

type DispatchResult = std::result::Result<serde_json::Value, DispatchError>;

/// Route a typed command through the engine runtime against this backend.
/// Events are not surfaced through dispatch (yet) — the envelope shape is
/// frozen; they will ride the outbox once event persistence lands.
fn run_engine(
    backend: &Backend,
    actor: Option<&str>,
    cmd: core::command::Command,
) -> std::result::Result<core::command::CommandOutcome, DispatchError> {
    let store = backend.store();
    // Host boundary: identity and the SPECLINK_* env layer are resolved here
    // and injected — the engine runtime only ever consumes the context. The
    // JS host is that Host: an actor bound at construction wins outright.
    // Without one, a filesystem backend falls back to the workspace's git
    // identity; a host-store backend has no local workspace, hence no
    // identity at all (anonymous, as before).
    let workspace = backend.workspace();
    let ctx = core::command::ExecutionContext {
        actor: actor.map(str::to_string).or_else(|| {
            workspace
                .as_ref()
                .and_then(|ws| speclink_host::context::git_identity(&ws.root))
        }),
        // The repo binding completion evidence records (`task done`): the fs
        // form is the local zero-configuration case, so it takes the binding
        // the CLI takes; a host store has none resolved here.
        repo: workspace
            .as_ref()
            .map(|_| speclink_host::binding::local_default_binding().repo.as_str().to_string()),
        env: speclink_host::policy::process_env_overrides(),
        workspace,
        user_config_dir: Some(speclink_host::context::global_config_dir()),
    };
    let (outcome, _events) =
        core::command::execute(store.as_ref(), &ctx, cmd).map_err(DispatchError::from)?;
    Ok(outcome)
}

/// Storage assembly of an engine instance.
enum Backend {
    /// Built-in filesystem store over a project root (zero bridging cost).
    Fs { root: PathBuf, spec_dir: String },
    /// Host-implemented Store object, bridged from JavaScript.
    /// `missing_stamp_methods` lists the optional store methods stamping
    /// needs (deleteArtifact / readChangeMeta / writeChangeMeta) that the
    /// store did not implement — captured at construction so the stamp verbs
    /// can refuse BEFORE any destructive step (the engine deletes the ticket
    /// before it rewrites the metadata).
    Js { bridge: JsStoreBridge, missing_stamp_methods: Vec<&'static str> },
}

impl Backend {
    fn store(&self) -> Box<dyn Store> {
        match self {
            Backend::Fs { root, spec_dir } => Box::new(speclink_fs::FsStore::new(root, spec_dir)),
            Backend::Js { bridge, .. } => Box::new(bridge.clone()),
        }
    }

    /// The host workspace, when the backend has a local filesystem one —
    /// drives schema resolution (project → user → built-in) and config files.
    /// A host store has no local workspace: built-in schemas only.
    fn workspace(&self) -> Option<core::workspace::Workspace> {
        match self {
            Backend::Fs { root, spec_dir } => Some(core::workspace::Workspace {
                root: root.clone(),
                spec_dir_name: spec_dir.clone(),
            }),
            Backend::Js { .. } => None,
        }
    }
}

#[napi]
pub struct Engine {
    backend: Arc<Backend>,
    /// The Host-resolved operator identity, bound once at construction —
    /// one instance, one identity. A multi-tenant host builds one engine per
    /// request (or per identity); dispatch carries no identity parameter, so
    /// call-time forgery has no surface.
    actor: Option<Arc<str>>,
    /// Stamping rewrites the whole change metadata (read-modify-write) with
    /// no CAS in the Store contract — concurrent stamps on one instance would
    /// clobber each other's stamp, so they serialize here. Coordination
    /// ACROSS engine instances (or processes) belongs to the host store.
    stamp_lock: Arc<std::sync::Mutex<()>>,
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
        let backend = self.backend.clone();
        let actor = self.actor.clone();
        let stamp_lock = self.stamp_lock.clone();
        let (deferred, promise) = env.create_deferred()?;
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_dispatch(&backend, actor.as_deref(), &stamp_lock, &argv, stdin.as_deref())
            }));
            let envelope = match outcome {
                Ok(Ok(value)) => serde_json::json!({ "ok": true, "value": value }),
                Ok(Err(e)) => {
                    serde_json::json!({ "ok": false, "code": e.code, "message": e.message })
                }
                // A store-method failure unwinds to here from non-Result trait
                // methods; anything else is a genuine panic — either way an
                // Error, never a process abort.
                Err(payload) => match payload.downcast_ref::<BridgeFailure>() {
                    Some(f) => {
                        let e = DispatchError::from(f.clone());
                        serde_json::json!({ "ok": false, "code": e.code, "message": e.message })
                    }
                    None => serde_json::json!({
                        "ok": false,
                        "code": "panic",
                        "message": format!("speclink engine panicked: {}", panic_message(&payload)),
                    }),
                },
            };
            deferred.resolve(move |_| Ok(envelope));
        });
        Ok(promise)
    }
}

fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

/// Build an engine over the built-in filesystem store.
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
        backend: Arc::new(Backend::Fs { root, spec_dir }),
        actor: normalize_actor(actor),
        stamp_lock: Arc::new(std::sync::Mutex::new(())),
    })
}

/// Build an engine over a host-implemented Store object. `invoker` is the JS
/// closure `(method, args, settle) => void` created by index.js with the store
/// bound; the store object itself is passed for construction-time validation.
#[napi(js_name = "engineFromStore")]
pub fn engine_from_store(
    env: Env,
    store: JsObject,
    invoker: JsFunction,
    actor: Option<String>,
) -> Result<Engine> {
    store_bridge::validate_store_methods(&store)?;
    let missing_stamp_methods = store_bridge::missing_stamp_methods(&store);
    let bridge = store_bridge::create_bridge(env, invoker)?;
    Ok(Engine {
        backend: Arc::new(Backend::Js { bridge, missing_stamp_methods }),
        actor: normalize_actor(actor),
        stamp_lock: Arc::new(std::sync::Mutex::new(())),
    })
}

// --- argv router ---

/// argv → the parsing the CLI uses → the engine command → the output
/// definition the CLI uses → the envelope value. Every verb with an engine
/// command takes this one path.
fn run_dispatch(
    backend: &Backend,
    actor: Option<&str>,
    stamp_lock: &std::sync::Mutex<()>,
    argv: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    use speclink_verbs::ToCommand;
    let parsed = speclink_verbs::parse(argv)
        .map_err(|e| DispatchError::new("invalid_argv", e.message))?;
    let name = parsed.name();
    // Before the stdin JSON is read: a store that cannot finish a stamp is
    // the first thing to report.
    if parsed.is_stamp() {
        refuse_unfinishable_stamp(backend, name)?;
    }
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
        speclink_verbs::CommandTarget::Command(cmd) => run_verb(backend, actor, stamp_lock, &parsed, cmd),
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

/// A stamp deletes the ticket before it rewrites the metadata, so a store
/// that cannot finish one is refused before anything else runs — never
/// mid-way with the ticket already gone.
fn refuse_unfinishable_stamp(backend: &Backend, name: &str) -> std::result::Result<(), DispatchError> {
    if let Backend::Js { missing_stamp_methods, .. } = backend {
        if !missing_stamp_methods.is_empty() {
            return Err(DispatchError::new(
                "error",
                format!(
                    "{name} requires store methods this store does not implement: {} — \
                     stamping rewrites the change metadata and deletes the ticket",
                    missing_stamp_methods.join(", ")
                ),
            ));
        }
    }
    Ok(())
}

/// Run one verb's command and shape its envelope value: the verb's
/// structured value when it has one, else `{ output }` with its stdout text
/// (no color). Two verbs keep an answer of their own in front of the engine:
/// `claim`, and `status` when the project has no change at all.
fn run_verb(
    backend: &Backend,
    actor: Option<&str>,
    stamp_lock: &std::sync::Mutex<()>,
    parsed: &speclink_verbs::ParsedVerb,
    cmd: core::command::Command,
) -> DispatchResult {
    use core::command::Command;
    use speclink_verbs::render::{render, RenderOptions};
    match &cmd {
        Command::Claim { name } => return claim(backend, actor, name),
        // CLI parity: with no name and no changes at all, status is informational,
        // not an error (envelope-level presentation, same as the CLI's exit-0 line).
        Command::Status { change: None, .. }
            if core::model::list_changes(backend.store().as_ref()).is_empty() =>
        {
            return Ok(serde_json::json!({
                "output": "No active changes. Create one with: speclink new change <name>"
            }));
        }
        _ => {}
    }
    // Stamping rewrites the whole metadata with no CAS, so stamps serialize
    // within this instance. A poisoned lock only means another stamp
    // panicked; the metadata itself is no more suspect than after any failed
    // dispatch, so proceed.
    let _stamp_guard =
        parsed.is_stamp().then(|| stamp_lock.lock().unwrap_or_else(|e| e.into_inner()));
    let outcome = run_engine(backend, actor, cmd)?;
    let rendered = render(parsed, &outcome, RenderOptions { json: false, color: false })?;
    Ok(match rendered.json {
        Some(value) => value,
        None => serde_json::json!({ "output": rendered.stdout }),
    })
}

/// `claim <name>` — ownership is a team-system concept: the fs store fails
/// loud like the CLI; a host store may implement the optional `claim` method
/// and adjudicate (conflicts reject with their semantic message and code).
fn claim(backend: &Backend, actor: Option<&str>, name: &str) -> DispatchResult {
    match backend {
        // The plain-store refusal comes from the runtime's Claim branch (the
        // frozen text shared with the CLI).
        Backend::Fs { .. } => {
            run_engine(backend, actor, core::command::Command::Claim { name: name.to_string() })?;
            unreachable!("claim on a plain store always refuses");
        }
        // Ownership adjudication is a host-store capability — the optional
        // `claim` bridge method stays at the envelope layer (決策三).
        Backend::Js { bridge, .. } => match bridge.claim(name) {
            Ok(v) => Ok(v),
            Err(f) if f.code.as_deref() == Some("__missing__") => Err(DispatchError::new(
                "error",
                "claim requires a store with claim support — this store does not implement claim",
            )),
            Err(f) => Err(DispatchError::from(f)),
        },
    }
}

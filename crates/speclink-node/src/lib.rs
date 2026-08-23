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
        // Node dispatch carries no task verbs (list/status/new/claim), so no
        // completion evidence is recorded through this path.
        repo: None,
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
    Js(JsStoreBridge),
}

impl Backend {
    fn store(&self) -> Box<dyn Store> {
        match self {
            Backend::Fs { root, spec_dir } => Box::new(speclink_fs::FsStore::new(root, spec_dir)),
            Backend::Js(bridge) => Box::new(bridge.clone()),
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
            Backend::Js(_) => None,
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
        let (deferred, promise) = env.create_deferred()?;
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_dispatch(&backend, actor.as_deref(), &argv, stdin.as_deref())
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
    Ok(Engine {
        backend: Arc::new(Backend::Fs {
            root: PathBuf::from(root),
            spec_dir: spec_dir.unwrap_or_else(|| "openspec".to_string()),
        }),
        actor: normalize_actor(actor),
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
    let bridge = store_bridge::create_bridge(env, invoker)?;
    Ok(Engine {
        backend: Arc::new(Backend::Js(bridge)),
        actor: normalize_actor(actor),
    })
}

// --- argv router ---

/// Minimal argv scanner: collects positionals and `--flag [value]` options for
/// one verb invocation. Option names that take a value are declared by the verb.
struct Argv<'a> {
    positionals: Vec<&'a str>,
    options: std::collections::HashMap<&'a str, &'a str>,
    flags: std::collections::HashSet<&'a str>,
}

fn parse_argv<'a>(
    args: &'a [String],
    value_options: &[&str],
) -> std::result::Result<Argv<'a>, DispatchError> {
    let mut out = Argv {
        positionals: Vec::new(),
        options: std::collections::HashMap::new(),
        flags: std::collections::HashSet::new(),
    };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if let Some(name) = a.strip_prefix("--") {
            if value_options.contains(&name) {
                let Some(v) = args.get(i + 1) else {
                    return Err(DispatchError::new(
                        "invalid_argv",
                        format!("option '--{name}' requires a value"),
                    ));
                };
                out.options.insert(name, v.as_str());
                i += 2;
                continue;
            }
            out.flags.insert(name);
        } else {
            out.positionals.push(a);
        }
        i += 1;
    }
    Ok(out)
}

fn run_dispatch(
    backend: &Backend,
    actor: Option<&str>,
    argv: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let Some(verb) = argv.first().map(|s| s.as_str()) else {
        return Err(DispatchError::new(
            "invalid_argv",
            "dispatch requires at least a verb (e.g. ['list', '--json'])",
        ));
    };
    let rest = &argv[1..];
    match verb {
        "list" => verb_list(backend, actor, rest),
        "status" => verb_status(backend, actor, rest),
        "new" => verb_new(backend, actor, rest, stdin),
        "claim" => verb_claim(backend, actor, rest),
        "review" => verb_station(backend, actor, StationKind::Review, rest, stdin),
        "verify" => verb_station(backend, actor, StationKind::Verify, rest, stdin),
        _ => Err(DispatchError::new(
            "invalid_argv",
            format!("Unknown or unsupported verb '{verb}'"),
        )),
    }
}

/// `list [--specs] [--changes] [--sort <key>]` — always the `--json` shape.
fn verb_list(backend: &Backend, actor: Option<&str>, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["sort"])?;
    let outcome = run_engine(
        backend,
        actor,
        core::command::Command::List {
            sort: a.options.get("sort").copied().unwrap_or("modified").to_string(),
            specs: a.flags.contains("specs"),
            changes: a.flags.contains("changes"),
            worktrees: Default::default(),
        },
    )?;
    let core::command::CommandOutcome::List(list) = outcome else {
        unreachable!("list command yields a list outcome");
    };
    let err = |e: serde_json::Error| DispatchError::new("error", format!("{e}"));
    match (list.changes, list.specs) {
        (None, Some(specs)) => Ok(serde_json::json!({ "specs": specs })),
        (Some(items), Some(specs)) => Ok(serde_json::json!({
            "changes": serde_json::to_value(items).map_err(err)?,
            "specs": specs,
        })),
        (Some(items), None) => Ok(serde_json::json!({
            "changes": serde_json::to_value(items).map_err(err)?,
        })),
        (None, None) => unreachable!("list yields at least one section"),
    }
}

/// `new change <name> …` / `new artifact <type> [capability] …` — stdin comes
/// from dispatch's second parameter (the CLI's `--stdin` content).
fn verb_new(
    backend: &Backend,
    actor: Option<&str>,
    args: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    match args.first().map(|s| s.as_str()) {
        Some("artifact") => verb_new_artifact(backend, actor, &args[1..], stdin),
        Some("change") => verb_new_change(backend, actor, &args[1..]),
        _ => Err(DispatchError::new(
            "invalid_argv",
            "new requires a subcommand: 'change' or 'artifact'",
        )),
    }
}

fn verb_new_artifact(
    backend: &Backend,
    actor: Option<&str>,
    args: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let a = parse_argv(args, &["change"])?;
    let Some(kind) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "new artifact requires a type: proposal, design, tasks, spec",
        ));
    };
    // The CLI's `--stdin` content arrives as dispatch's second parameter.
    let content = if a.flags.contains("stdin") {
        Some(stdin.unwrap_or_default().to_string())
    } else {
        None
    };
    let outcome = run_engine(
        backend,
        actor,
        core::command::Command::NewArtifact {
            kind: kind.to_string(),
            capability: a.positionals.get(1).map(|s| s.to_string()),
            change: a.options.get("change").map(|s| s.to_string()),
            content,
            force: a.flags.contains("force"),
            new_capability: a.flags.contains("new"),
        },
    )?;
    let core::command::CommandOutcome::NewArtifact(o) = outcome else {
        unreachable!("new artifact yields a new-artifact outcome");
    };
    Ok(serde_json::json!({
        "artifact": kind,
        "change": o.change,
        "path": o.path.to_string_lossy(),
        "status": "created",
        "validated": o.had_content,
        "warnings": [],
    }))
}

fn verb_new_change(backend: &Backend, actor: Option<&str>, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["description", "schema", "agent", "from-discussion"])?;
    let Some(name) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "new change requires a name (kebab-case)",
        ));
    };
    // Default-schema fail-closed, from_discussion guard, and mark_promoted all
    // live in the runtime — this layer only shapes the envelope.
    let outcome = run_engine(
        backend,
        actor,
        core::command::Command::NewChange {
            name: name.to_string(),
            description: a.options.get("description").map(|s| s.to_string()),
            schema: a.options.get("schema").map(|s| s.to_string()),
            agent: a.options.get("agent").map(|s| s.to_string()),
            from_discussion: a.options.get("from-discussion").map(|s| s.to_string()),
        },
    )?;
    let core::command::CommandOutcome::NewChange(o) = outcome else {
        unreachable!("new change yields a new-change outcome");
    };
    let mut output = format!(
        "✓ Created change: {}\n  Path: {}\n  Schema: {}",
        o.name,
        o.dir.to_string_lossy(),
        o.schema
    );
    if let Some(slug) = a.options.get("from-discussion") {
        output.push_str(&format!("\n  From discussion: {slug}"));
    }
    // `new change` has no --json form in the CLI → the {output} shape.
    Ok(serde_json::json!({ "output": output }))
}

/// `claim <name>` — ownership is a team-system concept: the fs store fails
/// loud like the CLI; a host store may implement the optional `claim` method
/// and adjudicate (conflicts reject with their semantic message and code).
fn verb_claim(backend: &Backend, actor: Option<&str>, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &[])?;
    let Some(name) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "claim requires a change name",
        ));
    };
    match backend {
        // The plain-store refusal comes from the runtime's Claim branch (the
        // frozen text shared with the CLI).
        Backend::Fs { .. } => {
            run_engine(backend, actor, core::command::Command::Claim { name: name.to_string() })?;
            unreachable!("claim on a plain store always refuses");
        }
        // Ownership adjudication is a host-store capability — the optional
        // `claim` bridge method stays at the envelope layer (決策三).
        Backend::Js(bridge) => match bridge.claim(name) {
            Ok(v) => Ok(v),
            Err(f) if f.code.as_deref() == Some("__missing__") => Err(DispatchError::new(
                "error",
                "claim requires a store with claim support — this store does not implement claim",
            )),
            Err(f) => Err(DispatchError::from(f)),
        },
    }
}

/// `status [--change <name>] [--schema <name>]` — the `--json` report.
fn verb_status(backend: &Backend, actor: Option<&str>, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["change", "schema"])?;
    // CLI parity: with no name and no changes at all, status is informational,
    // not an error (envelope-level presentation, same as the CLI's exit-0 line).
    if !a.options.contains_key("change") {
        let store = backend.store();
        if core::model::list_changes(store.as_ref()).is_empty() {
            return Ok(serde_json::json!({
                "output": "No active changes. Create one with: speclink new change <name>"
            }));
        }
    }
    let outcome = run_engine(
        backend,
        actor,
        core::command::Command::Status {
            change: a.options.get("change").map(|s| s.to_string()),
            schema: a.options.get("schema").map(|s| s.to_string()),
        },
    )?;
    let core::command::CommandOutcome::Status(report) = outcome else {
        unreachable!("status command yields a status outcome");
    };
    serde_json::to_value(&report).map_err(|e| DispatchError::new("error", format!("{e}")))
}

// --- quality stations ---

/// Which stamping station a `review …` / `verify …` argv addresses. The two
/// stations share one argv grammar and one payload shape; only the Command
/// variants differ.
#[derive(Clone, Copy)]
enum StationKind {
    Review,
    Verify,
}

impl StationKind {
    fn noun(self) -> &'static str {
        match self {
            StationKind::Review => "review",
            StationKind::Verify => "verify",
        }
    }
}

/// `review stamp` / `verify stamp` payload, carried by dispatch's stdin
/// parameter because argv cannot hold a fingerprint list. Same shape as the
/// server's stamp request body: the work-tree holder pre-computes
/// `(path, hash)` and declares which union paths are gone. Both fields
/// default to empty, so a host with an empty ticket scope can omit stdin.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StampPayload {
    #[serde(default)]
    scope: Vec<StampScopeEntry>,
    #[serde(default)]
    missing: Vec<String>,
}

#[derive(serde::Deserialize)]
struct StampScopeEntry {
    path: String,
    hash: String,
}

/// `<station> add-round <change> --stdin` and
/// `<station> stamp <change> [--accept] [--agent <tool>] [--stdin]`.
/// These are the identity-bearing verbs: the `_by` field of the stamp is the
/// engine's construction-time actor, resolved the same way `created_by` is.
fn verb_station(
    backend: &Backend,
    actor: Option<&str>,
    station: StationKind,
    args: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let noun = station.noun();
    match args.first().map(|s| s.as_str()) {
        Some("add-round") => verb_station_add_round(backend, actor, station, &args[1..], stdin),
        Some("stamp") => verb_station_stamp(backend, actor, station, &args[1..], stdin),
        _ => Err(DispatchError::new(
            "invalid_argv",
            format!("{noun} requires a subcommand: 'add-round' or 'stamp'"),
        )),
    }
}

fn verb_station_add_round(
    backend: &Backend,
    actor: Option<&str>,
    station: StationKind,
    args: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let noun = station.noun();
    let a = parse_argv(args, &[])?;
    let Some(change) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            format!("{noun} add-round requires a change name"),
        ));
    };
    if !a.flags.contains("stdin") {
        return Err(DispatchError::new(
            "invalid_argv",
            format!("{noun} add-round requires --stdin — the round content comes from stdin"),
        ));
    }
    let content = stdin.unwrap_or_default().to_string();
    let change = change.to_string();
    let cmd = match station {
        StationKind::Review => core::command::Command::ReviewAddRound { change, content },
        StationKind::Verify => core::command::Command::VerifyAddRound { change, content },
    };
    let outcome = run_engine(backend, actor, cmd)?;
    let (core::command::CommandOutcome::ReviewAddRound(o)
    | core::command::CommandOutcome::VerifyAddRound(o)) = outcome
    else {
        unreachable!("add-round yields a round outcome");
    };
    Ok(serde_json::json!({ "change": o.change, "round": o.round }))
}

fn verb_station_stamp(
    backend: &Backend,
    actor: Option<&str>,
    station: StationKind,
    args: &[String],
    stdin: Option<&str>,
) -> DispatchResult {
    let noun = station.noun();
    let a = parse_argv(args, &["agent"])?;
    let Some(change) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            format!("{noun} stamp requires a change name"),
        ));
    };
    let payload: StampPayload = if a.flags.contains("stdin") {
        serde_json::from_str(stdin.unwrap_or_default()).map_err(|e| {
            DispatchError::new(
                "invalid_argv",
                format!(
                    "{noun} stamp --stdin expects {{\"scope\": [{{\"path\", \"hash\"}}], \"missing\": []}}: {e}"
                ),
            )
        })?
    } else {
        StampPayload::default()
    };
    let scope = payload
        .scope
        .into_iter()
        .map(|e| core::model::ReviewedScopeEntry { path: e.path, hash: e.hash })
        .collect();
    let change = change.to_string();
    let accept = a.flags.contains("accept");
    let tool = a.options.get("agent").map(|s| s.to_string());
    let missing = payload.missing;
    let cmd = match station {
        StationKind::Review => {
            core::command::Command::ReviewStamp { change, accept, tool, scope, missing }
        }
        StationKind::Verify => {
            core::command::Command::VerifyStamp { change, accept, tool, scope, missing }
        }
    };
    let outcome = run_engine(backend, actor, cmd)?;
    let (core::command::CommandOutcome::ReviewStamp(o)
    | core::command::CommandOutcome::VerifyStamp(o)) = outcome
    else {
        unreachable!("stamp yields a subject outcome");
    };
    Ok(serde_json::json!({ "change": o.change }))
}

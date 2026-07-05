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

type DispatchResult = std::result::Result<serde_json::Value, DispatchError>;

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
}

#[napi]
impl Engine {
    /// argv dispatch on a background worker thread. Resolves to the envelope
    /// unwrapped by the JS wrapper; never blocks the JS event loop.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn dispatch(&self, env: Env, argv: Vec<String>, stdin: Option<String>) -> Result<JsObject> {
        let backend = self.backend.clone();
        let (deferred, promise) = env.create_deferred()?;
        std::thread::spawn(move || {
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_dispatch(&backend, &argv, stdin.as_deref())
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
pub fn engine_from_fs(root: String, spec_dir: Option<String>) -> Result<Engine> {
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
    })
}

/// Build an engine over a host-implemented Store object. `invoker` is the JS
/// closure `(method, args, settle) => void` created by index.js with the store
/// bound; the store object itself is passed for construction-time validation.
#[napi(js_name = "engineFromStore")]
pub fn engine_from_store(env: Env, store: JsObject, invoker: JsFunction) -> Result<Engine> {
    store_bridge::validate_store_methods(&store)?;
    let bridge = store_bridge::create_bridge(env, invoker)?;
    Ok(Engine {
        backend: Arc::new(Backend::Js(bridge)),
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

/// Resolve a change like the CLI: explicit name → lookup; otherwise auto-select
/// the only active change, or fail listing candidates (most recently modified first).
fn resolve_change(
    store: &dyn Store,
    name: Option<&str>,
    specify: &str,
) -> std::result::Result<core::model::Change, DispatchError> {
    if let Some(n) = name {
        return core::model::find_change(store, n)
            .ok_or_else(|| DispatchError::new("not_found", format!("Change '{n}' not found.")));
    }
    let mut changes = core::model::list_changes(store);
    match changes.len() {
        0 => Err(DispatchError::new(
            "not_found",
            "No active changes. Create one with: speclink new change <name>",
        )),
        1 => Ok(changes.remove(0)),
        _ => {
            core::listing::sort_changes(store, &mut changes, "modified");
            let names: Vec<&str> = changes.iter().map(|c| c.name.as_str()).collect();
            Err(DispatchError::new(
                "invalid_argv",
                format!("Multiple changes found. {specify} {}", names.join(", ")),
            ))
        }
    }
}

fn resolve_schema(
    ws: Option<&core::workspace::Workspace>,
    name: &str,
) -> std::result::Result<core::schema::Schema, DispatchError> {
    match core::schema::resolve_with(ws, name) {
        Some(Ok(s)) => Ok(s),
        Some(Err(e)) => Err(DispatchError::new("error", e)),
        None => Err(DispatchError::new(
            "not_found",
            core::schema::not_found_msg(name),
        )),
    }
}

fn run_dispatch(backend: &Backend, argv: &[String], stdin: Option<&str>) -> DispatchResult {
    let Some(verb) = argv.first().map(|s| s.as_str()) else {
        return Err(DispatchError::new(
            "invalid_argv",
            "dispatch requires at least a verb (e.g. ['list', '--json'])",
        ));
    };
    let rest = &argv[1..];
    match verb {
        "list" => verb_list(backend, rest),
        "status" => verb_status(backend, rest),
        "new" => verb_new(backend, rest, stdin),
        "claim" => verb_claim(backend, rest),
        _ => Err(DispatchError::new(
            "invalid_argv",
            format!("Unknown or unsupported verb '{verb}'"),
        )),
    }
}

/// `list [--specs] [--changes] [--sort <key>]` — always the `--json` shape.
fn verb_list(backend: &Backend, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["sort"])?;
    let store = backend.store();
    let store: &dyn Store = store.as_ref();
    let specs = a.flags.contains("specs");
    let changes_flag = a.flags.contains("changes");
    if specs && !changes_flag {
        return Ok(serde_json::json!({ "specs": core::listing::specs_json_items(store) }));
    }
    let mut changes = core::model::list_changes(store);
    core::listing::sort_changes(store, &mut changes, a.options.get("sort").copied().unwrap_or("modified"));
    let items = core::listing::changes_json(store, &changes);
    if specs {
        return Ok(serde_json::json!({
            "changes": items,
            "specs": core::listing::specs_json_items(store),
        }));
    }
    Ok(serde_json::json!({ "changes": items }))
}

/// `new change <name> …` / `new artifact <type> [capability] …` — stdin comes
/// from dispatch's second parameter (the CLI's `--stdin` content).
fn verb_new(backend: &Backend, args: &[String], stdin: Option<&str>) -> DispatchResult {
    match args.first().map(|s| s.as_str()) {
        Some("artifact") => verb_new_artifact(backend, &args[1..], stdin),
        Some("change") => verb_new_change(backend, &args[1..]),
        _ => Err(DispatchError::new(
            "invalid_argv",
            "new requires a subcommand: 'change' or 'artifact'",
        )),
    }
}

fn verb_new_artifact(backend: &Backend, args: &[String], stdin: Option<&str>) -> DispatchResult {
    let a = parse_argv(args, &["change"])?;
    let store = backend.store();
    let store: &dyn Store = store.as_ref();
    let Some(kind) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "new artifact requires a type: proposal, design, tasks, spec",
        ));
    };
    let capability = a.positionals.get(1).copied();
    let type_ok = ["proposal", "design", "tasks", "spec"].contains(&kind);
    let type_err = || {
        DispatchError::new(
            "invalid_argv",
            format!("Unknown artifact type '{kind}'. Valid types: proposal, design, tasks, spec"),
        )
    };
    // CLI parity: with an explicit --change, validate the type before existence;
    // when auto-detecting, resolve the change first. Change-not-found without
    // a trailing period, matching the CLI.
    let change = match a.options.get("change").copied() {
        Some(name) => {
            if !type_ok {
                return Err(type_err());
            }
            core::model::find_change(store, name).ok_or_else(|| {
                DispatchError::new("not_found", format!("Change '{name}' not found"))
            })?
        }
        None => {
            let c = resolve_change(store, None, "Use --change to specify one:")?;
            if !type_ok {
                return Err(type_err());
            }
            c
        }
    };
    // Best-effort schema resolution: an unresolvable/broken schema still
    // creates the artifact (empty template), matching the CLI.
    let ws = backend.workspace();
    let schema = match core::schema::resolve_with(ws.as_ref(), &change.meta.schema_name()) {
        Some(Ok(s)) => s,
        _ => core::schema::Schema {
            name: change.meta.schema_name(),
            display_name: change.meta.schema_name(),
            description: None,
            source: "project".to_string(),
            artifacts: Vec::new(),
            apply_requires: Vec::new(),
            apply_tracks: None,
            apply_instruction: None,
        },
    };
    // The CLI's `--stdin` content arrives as dispatch's second parameter.
    let content = if a.flags.contains("stdin") {
        Some(stdin.unwrap_or_default())
    } else {
        None
    };
    let had_content = content.is_some();
    let (_artifact_id, path) = core::newcmd::new_artifact(
        store,
        &change,
        &schema,
        kind,
        capability,
        content,
        a.flags.contains("force"),
    )?;
    Ok(serde_json::json!({
        "artifact": kind,
        "change": change.name,
        "path": path.to_string_lossy(),
        "status": "created",
        "validated": had_content,
        "warnings": [],
    }))
}

fn verb_new_change(backend: &Backend, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["description", "schema", "agent", "from-discussion"])?;
    let store = backend.store();
    let store: &dyn Store = store.as_ref();
    let Some(name) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "new change requires a name (kebab-case)",
        ));
    };
    let schema = a
        .options
        .get("schema")
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| {
            core::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())
                .schema_name()
        });
    let from_discussion = a.options.get("from-discussion").copied();
    if let Some(slug) = from_discussion {
        if core::discuss::info(store, slug).is_none() {
            return Err(DispatchError::new(
                "not_found",
                format!("discussion '{slug}' not found — run `speclink discuss new` first"),
            ));
        }
    }
    // A host store has no local workspace; the synthetic one only skips the
    // git-identity lookup (`created_by`) inside new_change.
    let ws = backend.workspace().unwrap_or(core::workspace::Workspace {
        root: PathBuf::new(),
        spec_dir_name: "openspec".to_string(),
    });
    let dir = core::newcmd::new_change(
        &ws,
        store,
        name,
        a.options.get("description").copied(),
        &schema,
        a.options.get("agent").copied(),
        from_discussion,
    )?;
    let mut output = format!(
        "✓ Created change: {name}\n  Path: {}\n  Schema: {schema}",
        dir.to_string_lossy()
    );
    if let Some(slug) = from_discussion {
        core::discuss::mark_promoted(store, slug, name)?;
        output.push_str(&format!("\n  From discussion: {slug}"));
    }
    // `new change` has no --json form in the CLI → the {output} shape.
    Ok(serde_json::json!({ "output": output }))
}

/// `claim <name>` — ownership is a team-system concept: the fs store fails
/// loud like the CLI; a host store may implement the optional `claim` method
/// and adjudicate (conflicts reject with their semantic message and code).
fn verb_claim(backend: &Backend, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &[])?;
    let Some(name) = a.positionals.first().copied() else {
        return Err(DispatchError::new(
            "invalid_argv",
            "claim requires a change name",
        ));
    };
    match backend {
        Backend::Fs { .. } => Err(DispatchError::new(
            "error",
            "claim requires a remote store — this project uses the local fs store",
        )),
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
fn verb_status(backend: &Backend, args: &[String]) -> DispatchResult {
    let a = parse_argv(args, &["change", "schema"])?;
    let store = backend.store();
    let store: &dyn Store = store.as_ref();
    let ws = backend.workspace();
    // CLI parity: with no name and no changes at all, status is informational, not an error.
    if !a.options.contains_key("change") && core::model::list_changes(store).is_empty() {
        return Ok(serde_json::json!({
            "output": "No active changes. Create one with: speclink new change <name>"
        }));
    }
    let change = resolve_change(store, a.options.get("change").copied(), "Use --change to specify one:")?;
    let schema_name = match a.options.get("schema") {
        Some(s) => (*s).to_string(),
        None => change.meta.schema_name(),
    };
    let schema = resolve_schema(ws.as_ref(), &schema_name)?;
    let report = core::status::build(store, &change, &schema);
    serde_json::to_value(&report).map_err(|e| DispatchError::new("error", format!("{e}")))
}

//! `createHost`, `host.dispatch`, `host.handleRequest` and the export and the
//! import of a scope: the host kit's two doors. A host is built once over a JS store port, its hooks and the HTTP
//! door's options. `dispatch` is the door for the host's own Agent tool: each
//! call brings who acts and on which scope, then takes the same verb parsing,
//! stdin rules and output as `engine.dispatch`, and runs the command through
//! `speclink_host::kit::HostKit`. `handleRequest` is the door for the host's
//! one HTTP route (host-kit-client-door D7): the original CLI and Speclink
//! Desktop talk to it. Both doors share the store and the hooks.

use crate::port_bridge::{events_json, person_json, settle_call, JsPort};
use crate::js_bridge::{create_bridge, JsBridge};
use crate::{
    command_of, embedded_value_of, envelope, on_worker, panic_message, parse_verb, status_notice, DispatchError,
    DispatchResult,
};
use napi::bindgen_prelude::*;
use napi::{JsFunction, JsObject};
use napi_derive::napi;
use speclink_core::command::Command;
use speclink_core::fields::Fields;
use speclink_host::http::{HttpError, HttpRequest, Method, API_ROOT};
use speclink_host::kit::{
    AfterCommit, AfterCommitHook, BeforeCommand, BeforeCommandHook, Caller, Decision, Hooks, HostKit,
    HostKitOptions, Identity, InvalidSkills, NotARead, ProjectEntry, Reader, RepoEntry, ScopesFn,
};
use speclink_host::port_exec::{ExecError, PortExecution};
use speclink_host::transfer::{self, ImportMode, ImportOptions};
use speclink_protocol::auth::AuthMethodsResponse;
use speclink_protocol::events::EventTransport;
use speclink_store_port::{PortError, ScopeId, StorePort};
use std::sync::{Arc, Mutex};

/// What `createHost` set up: the store port, the bridge over the hooks and
/// the scope function, which of them the host gave, and the HTTP door's two
/// declarations.
struct HostSetup {
    store: Arc<dyn StorePort>,
    /// The bridge over `{ beforeCommand, afterCommit, scopes }` (index.js).
    hooks: JsBridge,
    has_before: bool,
    has_after: bool,
    has_scopes: bool,
    auth_methods: Option<AuthMethodsResponse>,
    push_transports: Vec<EventTransport>,
    /// The skills the door gives a client, checked at `createHost`. Absent:
    /// every skill of the Engine.
    skills: Option<Vec<String>>,
    /// The most an import takes, in bytes. Absent: the kit's default.
    import_limit_bytes: Option<usize>,
}

/// The HTTP door's declarations as index.js passes them, in the wire's own
/// camelCase shape: an unknown field or a wrong type is refused at
/// `createHost`.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DoorOptions {
    #[serde(default)]
    auth_methods: Option<AuthMethodsResponse>,
    #[serde(default)]
    push_transports: Vec<EventTransport>,
    #[serde(default)]
    skills: Option<Vec<String>>,
    #[serde(default)]
    limits: DoorLimits,
}

/// `createHost`'s `limits`. index.js lets an object through only, and checks
/// the number: JSON cannot carry every wrong one. A limit this does not know
/// is refused here.
#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DoorLimits {
    #[serde(default)]
    import_bytes: Option<usize>,
}

#[napi]
pub struct Host {
    setup: Arc<HostSetup>,
}

/// Build a host over a JS store port. `port_invoker` and `hook_invoker` are
/// index.js's invokers over the store's three functions and over the two
/// hooks; the flags say which hooks the host gave.
/// `door_options` is the JSON text of `{ authMethods, pushTransports, skills, limits }`;
/// `has_scopes` says whether the host gave a scope function. An empty skill
/// list, or a name the Engine does not have, throws here: `createHost` gives
/// no host.
#[napi(js_name = "hostFromPort")]
pub fn host_from_port(
    env: Env,
    port_invoker: JsFunction,
    hook_invoker: JsFunction,
    has_before: bool,
    has_after: bool,
    has_scopes: bool,
    door_options: String,
) -> Result<Host> {
    let door: DoorOptions = serde_json::from_str(&door_options).map_err(|e| {
        Error::new(Status::InvalidArg, format!("createHost: authMethods, pushTransports, skills or limits: {e}"))
    })?;
    InvalidSkills::check(door.skills.as_deref())
        .map_err(|e| Error::new(Status::InvalidArg, format!("createHost: skills: {e}")))?;
    let store: Arc<dyn StorePort> = Arc::new(JsPort::new(create_bridge(env, port_invoker)?));
    let hooks = create_bridge(env, hook_invoker)?;
    Ok(Host {
        setup: Arc::new(HostSetup {
            store,
            hooks,
            has_before,
            has_after,
            has_scopes,
            auth_methods: door.auth_methods,
            push_transports: door.push_transports,
            skills: door.skills,
            import_limit_bytes: door.limits.import_bytes,
        }),
    })
}

/// Every command name the hooks and the store's commits can see
/// ([`speclink_host::kit::command_names`]): index.js freezes it once as `commandNames`.
#[napi(js_name = "commandNames")]
pub fn command_names() -> Vec<&'static str> {
    speclink_host::kit::command_names()
}

/// The `speclink` Agent tool (design D9), shaped for JS:
/// `{ name, description, inputSchema }`.
#[napi(js_name = "agentToolDefinition")]
pub fn agent_tool_definition() -> serde_json::Value {
    let tool = speclink_verbs::tool::tool_definition();
    serde_json::json!({ "name": tool.name, "description": tool.description, "inputSchema": tool.input_schema })
}

/// The `speclink` Agent tool's description without the verbs `exclude` names
/// ([`speclink_verbs::tool::tool_description`]): index.js checks the options.
#[napi(js_name = "agentToolDescription")]
pub fn agent_tool_description(exclude: Vec<String>) -> Result<String> {
    speclink_verbs::tool::tool_description(&exclude).map_err(Error::from_reason)
}

#[napi]
impl Host {
    /// One verb for `identity` on `scope`, on a background thread. Resolves
    /// to the envelope index.js unwraps; never blocks the JS event loop.
    /// `fields` is the JSON text of the free fields the host attaches to
    /// this call, absent when it attaches none.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn dispatch(
        &self,
        env: Env,
        id: String,
        display_name: String,
        scope: String,
        argv: Vec<String>,
        stdin: Option<String>,
        fields: Option<String>,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        on_worker(env, move || {
            guarded(|| {
                let caller = caller_of(id, display_name, scope)?;
                let fields =
                    fields_of(fields.as_deref()).map_err(|message| DispatchError::new("invalid_argv", message))?;
                run(&setup, &caller, &argv, stdin.as_deref(), &fields)
            })
        })
    }

    /// One verb that writes nothing, for `identity` on `scope`, on a
    /// background thread: the `read` index.js gives `beforeCommand`. It is
    /// not on the public `Host` object. The result and the errors are those
    /// of `dispatch`; no hook is asked.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn read(
        &self,
        env: Env,
        id: String,
        display_name: String,
        scope: String,
        argv: Vec<String>,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        on_worker(env, move || guarded(|| read(&setup, &caller_of(id, display_name, scope)?, &argv)))
    }

    /// One HTTP request for `identity` (none when `id` is absent), on a
    /// background thread. `request` is the JSON text of
    /// `{ method, path, query, headers }`. Resolves to the envelope index.js
    /// unwraps: the response `{ status, headers, body }` with the body as
    /// text, or null when the path is not the door's. `fields` is the JSON
    /// text of the free fields the host attaches, as for `dispatch`.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn handle_request(
        &self,
        env: Env,
        request: String,
        body: Buffer,
        id: Option<String>,
        display_name: Option<String>,
        fields: Option<String>,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        let body = body.to_vec();
        on_worker(env, move || guarded(|| handle(&setup, &request, body, id.zip(display_name), fields.as_deref())))
    }

    /// Every document of `scope` for `identity`, on a background thread.
    /// Resolves to the envelope index.js unwraps; its value is the JSON text
    /// of the keyed bundle, which index.js parses: a large bundle crosses as
    /// one string.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn export_scope(&self, env: Env, id: String, display_name: String, scope: String) -> Result<JsObject> {
        let setup = self.setup.clone();
        on_worker(env, move || {
            guarded(|| {
                let caller = caller_of(id, display_name, scope)?;
                let bundle = with_kit(&setup, |kit| kit.export_scope(&caller))?.map_err(exec_error)?;
                let text = serde_json::to_string(&bundle).expect("a bundle serializes to JSON");
                Ok(serde_json::Value::String(text))
            })
        })
    }

    /// Write a bundle into `scope` for `identity`, on a background thread.
    /// `bundle` is the JSON text of a bundle of either version. `mode` is
    /// the text of an import mode (`create` or `replace`), and `dry_run`
    /// commits nothing: index.js checked both before the call. Resolves to the
    /// envelope index.js unwraps:
    /// `{ scopeVersion, documents, added, changed, removed }`.
    #[napi(ts_return_type = "Promise<unknown>")]
    #[allow(clippy::too_many_arguments)]
    pub fn import_scope(
        &self,
        env: Env,
        id: String,
        display_name: String,
        scope: String,
        bundle: String,
        mode: String,
        dry_run: bool,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        on_worker(env, move || {
            guarded(|| {
                let caller = caller_of(id, display_name, scope)?;
                let invalid = |message: String| DispatchError::new("invalid_argv", message);
                let mode = ImportMode::parse(&mode)
                    .ok_or_else(|| invalid("importScope: mode must be 'create' or 'replace'".to_string()))?;
                let options = ImportOptions { mode, dry_run };
                let bundle = transfer::read_bundle(bundle.as_bytes())
                    .map_err(|unread| invalid(unread.to_string()))?
                    .into_keyed()
                    .map_err(|e| invalid(e.to_string()))?;
                let report = with_kit(&setup, |kit| kit.import_scope(&caller, bundle, options))?.map_err(exec_error)?;
                Ok(serde_json::json!({
                    "scopeVersion": report.scope_version.get(),
                    "documents": report.documents,
                    "added": report.added,
                    "changed": report.changed,
                    "removed": report.removed,
                }))
            })
        })
    }
}

/// One call through a kit over this host's store, hooks and options. A hook,
/// or the scope function, that broke during the call ends it as `internal`,
/// whatever the kit answered.
fn with_kit<T>(setup: &HostSetup, call: impl FnOnce(&HostKit) -> T) -> std::result::Result<T, DispatchError> {
    let broken_hook: Arc<Mutex<Option<String>>> = Arc::default();
    let options = HostKitOptions {
        scopes: setup.has_scopes.then(|| scopes_for(setup, &broken_hook)),
        auth_methods: setup.auth_methods.clone(),
        push_transports: setup.push_transports.clone(),
        skills: setup.skills.clone(),
        import_limit_bytes: setup.import_limit_bytes,
    };
    let kit = HostKit::with_options(setup.store.clone(), hooks_for(setup, &broken_hook), options)
        .expect("the skill list was checked at createHost");
    let result = call(&kit);
    hooks_held(&broken_hook)?;
    Ok(result)
}

/// A kit call's failure, by the codes of a dispatch. A command failure whose
/// source is an import's failure keeps that failure's code; no other source
/// gives one.
fn exec_error(e: ExecError) -> DispatchError {
    match e {
        ExecError::Command(e) => {
            let reason = transfer::reason_of(&e);
            DispatchError::new(e.code.as_str(), e.message).with_reason(reason)
        }
        ExecError::Store(e) => store_error(e),
    }
}

/// The envelope of `work`; a panic in it is an `internal` error, not a
/// Promise that never settles.
fn guarded(work: impl FnOnce() -> DispatchResult) -> serde_json::Value {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|payload| {
        Err(DispatchError::new("internal", format!("speclink host panicked: {}", panic_message(&payload))))
    });
    envelope(result)
}

/// A JS hook, or the scope function, that threw or answered in a shape it
/// does not know ends the call as `internal`: the Rust side can only deny,
/// so it denies and leaves the reason in `broken_hook` for this check.
fn hooks_held(broken_hook: &Mutex<Option<String>>) -> std::result::Result<(), DispatchError> {
    match broken_hook.lock().unwrap_or_else(|e| e.into_inner()).take() {
        Some(message) => Err(DispatchError::new("internal", message)),
        None => Ok(()),
    }
}

/// The free fields a host attached to one call, from their JSON text: an
/// object whose values are all strings. Absent text is no field. Any other
/// shape is refused: a number or a nested object is the host's own bug, and
/// writing its text form would hide it.
fn fields_of(json: Option<&str>) -> std::result::Result<Fields, String> {
    let Some(json) = json else {
        return Ok(Fields::new());
    };
    serde_json::from_str(json).map_err(|_| "fields: expected an object whose values are all strings".to_string())
}

/// The methods the door takes; any other is answered 405 before the door.
const DOOR_METHODS: &str = "GET, HEAD, POST, PUT, DELETE";

/// The request's fields other than the body, as index.js passes them.
#[derive(serde::Deserialize)]
struct DoorRequest {
    method: String,
    path: String,
    #[serde(default)]
    query: Option<String>,
    headers: serde_json::Map<String, serde_json::Value>,
}

/// `who` is the identity's id and display name, from the host's own code.
fn handle(
    setup: &HostSetup,
    request: &str,
    body: Vec<u8>,
    who: Option<(String, String)>,
    fields: Option<&str>,
) -> DispatchResult {
    let request: DoorRequest = serde_json::from_str(request)
        .map_err(|e| DispatchError::new("invalid_argv", format!("handleRequest: request: {e}")))?;
    let Some(method) = Method::from_name(&request.method) else {
        return Ok(serde_json::json!({ "status": 405, "headers": { "allow": DOOR_METHODS }, "body": "" }));
    };
    let identity = who
        .map(|(id, display_name)| Identity::new(id, display_name))
        .transpose()
        .map_err(|e| DispatchError::new("invalid_argv", e.to_string()))?;
    let headers =
        request.headers.into_iter().filter_map(|(name, value)| Some((name, value.as_str()?.to_string()))).collect();
    let request = HttpRequest { method, path: request.path, query: request.query, headers, body };
    // The door's answer for the host's own mistake is a 400, like any request
    // it cannot read; a path outside the door's prefix stays the host's.
    let attached = match fields_of(fields) {
        Ok(fields) => fields,
        Err(_) if !request.path.starts_with(API_ROOT) => return Ok(serde_json::Value::Null),
        Err(message) => return Ok(response_json(HttpError::invalid_argument(message).into_response())),
    };

    let response = with_kit(setup, |kit| kit.handle_with_fields(&request, identity.as_ref(), &attached))?;
    Ok(response.map_or(serde_json::Value::Null, response_json))
}

/// A door response as index.js takes it: `{ status, headers, body }`.
fn response_json(response: speclink_host::http::HttpResponse) -> serde_json::Value {
    let headers: serde_json::Map<String, serde_json::Value> =
        response.headers.into_iter().map(|(name, value)| (name, value.into())).collect();
    // The door writes JSON or nothing: the body is always text.
    let body = String::from_utf8(response.body).expect("the door writes UTF-8 JSON");
    serde_json::json!({ "status": response.status, "headers": headers, "body": body })
}

/// One project the scope function gives, in the SDK's camelCase shape. A
/// switch has no default: the host states both.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct JsProject {
    key: String,
    name: String,
    repos: Vec<JsRepo>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JsRepo {
    key: String,
    name: String,
    scope: String,
    can_write: bool,
    can_write_policy: bool,
}

/// The host's `scopes` through the hook bridge. A throw or a rejected
/// Promise is the door's 503; an answer of another shape is the host's bug,
/// so the call ends with `internal` naming it, like a broken hook.
fn scopes_for(setup: &HostSetup, broken_hook: &Arc<Mutex<Option<String>>>) -> ScopesFn {
    let bridge = setup.hooks.clone();
    let broken_hook = broken_hook.clone();
    Box::new(move |identity: &Identity| {
        let answer = settle_call(&bridge, "scopes", serde_json::json!([identity_json(identity)]))
            .map_err(|failure| failure.to_string())?;
        projects_of(answer).map_err(|message| {
            *broken_hook.lock().unwrap_or_else(|e| e.into_inner()) = Some(message.clone());
            message
        })
    })
}

fn projects_of(answer: serde_json::Value) -> std::result::Result<Vec<ProjectEntry>, String> {
    let shape = "scopes: expected [{ key, name, repos: [{ key, name, scope, canWrite, canWritePolicy }] }]";
    let projects: Vec<JsProject> = serde_json::from_value(answer).map_err(|e| format!("{shape}: {e}"))?;
    projects
        .into_iter()
        .map(|project| {
            let repos = project
                .repos
                .into_iter()
                .map(|repo| {
                    let scope = ScopeId::new(repo.scope).map_err(|e| format!("{shape}: invalid scope: {e}"))?;
                    Ok(RepoEntry {
                        key: repo.key,
                        name: repo.name,
                        scope,
                        can_write: repo.can_write,
                        can_write_policy: repo.can_write_policy,
                    })
                })
                .collect::<std::result::Result<Vec<_>, String>>()?;
            Ok(ProjectEntry { key: project.key, name: project.name, repos })
        })
        .collect()
}

/// Who and where, from the host's own code — never from argv or stdin.
fn caller_of(id: String, display_name: String, scope: String) -> std::result::Result<Caller, DispatchError> {
    let identity = Identity::new(id, display_name).map_err(|e| DispatchError::new("invalid_argv", e.to_string()))?;
    let scope = ScopeId::new(scope).map_err(|e| DispatchError::new("invalid_argv", format!("invalid scope: {e}")))?;
    Ok(Caller { identity, scope })
}

fn run(setup: &HostSetup, caller: &Caller, argv: &[String], stdin: Option<&str>, fields: &Fields) -> DispatchResult {
    let parsed = parse_verb(argv)?;
    let cmd = command_of(&parsed, stdin)?;
    through_kit(setup, &parsed, cmd, |kit, cmd| kit.run_with_fields(caller, cmd, fields))
}

/// [`run`] for the hook's `read`: the same parsing and the same output, for a
/// verb that writes nothing, through the kit's reader. The kit refuses any
/// other verb, and the refusal names the verb as the caller wrote it.
fn read(setup: &HostSetup, caller: &Caller, argv: &[String]) -> DispatchResult {
    let parsed = parse_verb(argv)?;
    // `read` has no content to give, and a verb that takes content converts
    // only with it: refused by its name here, not for the content it lacks.
    if parsed.stdin_policy() != speclink_verbs::StdinPolicy::Never {
        return Err(DispatchError::new("invalid_argv", not_a_read(parsed.name())));
    }
    let cmd = command_of(&parsed, None)?;
    through_kit(setup, &parsed, cmd, |kit, cmd| kit.read(caller, cmd).map_err(|e| by_verb(e, parsed.name())))
}

/// The message of `read`'s refusal of `verb`.
fn not_a_read(verb: &str) -> String {
    format!("read accepts only verbs that write nothing; '{verb}' is not one of them")
}

/// The kit's refusal of a command that writes, with the verb text in place of
/// the command's name; any other failure as it is.
fn by_verb(e: ExecError, verb: &str) -> ExecError {
    match e {
        ExecError::Command(mut e) if e.source.as_ref().is_some_and(|source| source.is::<NotARead>()) => {
            e.message = not_a_read(verb);
            ExecError::Command(e)
        }
        other => other,
    }
}

/// The envelope value of `cmd` run by `call` on this host's kit: `dispatch`
/// and the hook's `read` both end here, so both give the output of a store
/// with no local folder.
fn through_kit(
    setup: &HostSetup,
    parsed: &speclink_verbs::ParsedVerb,
    cmd: Command,
    call: impl FnOnce(&HostKit, Command) -> std::result::Result<PortExecution, ExecError>,
) -> DispatchResult {
    let status_without_name = matches!(cmd, Command::Status { change: None, .. });

    match with_kit(setup, |kit| call(kit, cmd))? {
        Ok(execution) => embedded_value_of(parsed, &execution.outcome),
        Err(ExecError::Command(e)) => match status_notice(status_without_name, &e) {
            Some(notice) => Ok(notice),
            None => Err(exec_error(ExecError::Command(e))),
        },
        Err(e) => Err(exec_error(e)),
    }
}

/// The store layer's failures, by the wire protocol's names.
fn store_error(e: PortError) -> DispatchError {
    let code = match e {
        PortError::Conflict { .. } => "revision_conflict",
        PortError::Unavailable { .. } => "unavailable",
        PortError::Backend { .. } => "internal",
    };
    DispatchError::new(code, e.to_string())
}

/// This call's hooks: each one calls its JS function through the hook bridge.
fn hooks_for(setup: &HostSetup, broken_hook: &Arc<Mutex<Option<String>>>) -> Hooks {
    let before_command = setup.has_before.then(|| {
        let bridge = setup.hooks.clone();
        let broken_hook = broken_hook.clone();
        Box::new(move |info: &BeforeCommand, _: &Reader<'_>| decide(&bridge, info, &broken_hook)) as BeforeCommandHook
    });
    let after_commit = setup.has_after.then(|| {
        let bridge = setup.hooks.clone();
        // Not waited for: the dispatch resolves whatever the hook does.
        Box::new(move |info: &AfterCommit| bridge.notify("afterCommit", serde_json::json!([after_json(info)])))
            as AfterCommitHook
    });
    Hooks { before_command, after_commit }
}

fn identity_json(identity: &Identity) -> serde_json::Value {
    person_json(identity.id(), identity.display_name())
}

fn before_json(info: &BeforeCommand) -> serde_json::Value {
    let mut value = serde_json::json!({
        "identity": identity_json(&info.caller.identity),
        "scope": info.caller.scope.as_str(),
        "command": info.command,
        "params": info.params,
    });
    if let Some(keys) = &info.changed_keys {
        value["changedKeys"] = serde_json::json!(keys);
    }
    value
}

fn after_json(info: &AfterCommit) -> serde_json::Value {
    serde_json::json!({
        "identity": identity_json(&info.caller.identity),
        "scope": info.caller.scope.as_str(),
        "command": info.command,
        "scopeVersion": info.scope_version.get(),
        "events": events_json(&info.events),
    })
}

/// The JS answer as a decision. index.js hands over one of three shapes —
/// `{ allow: true }`, `{ allow: false, reason }`, `{ invalid }` — so a throw,
/// a rejected Promise or any other answer does not allow: it is recorded as a
/// broken hook.
fn decide(bridge: &JsBridge, info: &BeforeCommand, broken_hook: &Mutex<Option<String>>) -> Decision {
    let broken = |message: String| {
        *broken_hook.lock().unwrap_or_else(|e| e.into_inner()) = Some(message);
        Decision::Deny { reason: "beforeCommand failed".to_string() }
    };
    let answer = match settle_call(bridge, "beforeCommand", serde_json::json!([before_json(info)])) {
        Ok(answer) => answer,
        Err(failure) => return broken(failure.to_string()),
    };
    match (answer.get("allow"), answer.get("reason")) {
        (Some(serde_json::Value::Bool(true)), _) => Decision::Allow,
        (Some(serde_json::Value::Bool(false)), Some(serde_json::Value::String(reason))) => {
            Decision::Deny { reason: reason.clone() }
        }
        _ => broken(format!(
            "beforeCommand: expected undefined, {{ allow: true }} or {{ allow: false, reason }}, got {}",
            answer.get("invalid").and_then(|v| v.as_str()).unwrap_or("an answer of another shape")
        )),
    }
}

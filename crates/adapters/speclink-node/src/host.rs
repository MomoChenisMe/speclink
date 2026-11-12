//! `createHost`, `host.dispatch` and `host.handleRequest`: the host kit's two
//! doors. A host is built once over a JS store port, its hooks and the HTTP
//! door's options. `dispatch` is the door for the host's own Agent tool: each
//! call brings who acts and on which scope, then takes the same verb parsing,
//! stdin rules and output as `engine.dispatch`, and runs the command through
//! `speclink_host::kit::HostKit`. `handleRequest` is the door for the host's
//! one HTTP route (host-kit-client-door D7): the original CLI and Speclink
//! Desktop talk to it. Both doors share the store and the hooks.

use crate::port_bridge::{events_json, person_json, settle_call, JsPort};
use crate::store_bridge::{create_bridge, JsStoreBridge};
use crate::{
    command_of, envelope, on_worker, panic_message, parse_verb, status_notice, value_of, DispatchError, DispatchResult,
};
use napi::bindgen_prelude::*;
use napi::{JsFunction, JsObject};
use napi_derive::napi;
use speclink_core::command::Command;
use speclink_host::http::{HttpRequest, Method};
use speclink_host::kit::{
    AfterCommit, AfterCommitHook, BeforeCommand, BeforeCommandHook, Caller, Decision, Hooks, HostKit,
    HostKitOptions, Identity, ProjectEntry, RepoEntry, ScopesFn, InvalidSkills,
};
use speclink_host::port_exec::ExecError;
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
    hooks: JsStoreBridge,
    has_before: bool,
    has_after: bool,
    has_scopes: bool,
    auth_methods: Option<AuthMethodsResponse>,
    push_transports: Vec<EventTransport>,
    /// The skills the door gives a client, checked at `createHost`. Absent:
    /// every skill of the Engine.
    skills: Option<Vec<String>>,
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
}

#[napi]
pub struct Host {
    setup: Arc<HostSetup>,
}

/// Build a host over a JS store port. `port_invoker` and `hook_invoker` are
/// index.js's invokers over the store's three functions and over the two
/// hooks; the flags say which hooks the host gave.
/// `door_options` is the JSON text of `{ authMethods, pushTransports, skills }`;
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
        Error::new(Status::InvalidArg, format!("createHost: authMethods, pushTransports or skills: {e}"))
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
        }),
    })
}

/// The `speclink` Agent tool (design D9), shaped for JS:
/// `{ name, description, inputSchema }`.
#[napi(js_name = "agentToolDefinition")]
pub fn agent_tool_definition() -> serde_json::Value {
    let tool = speclink_verbs::tool::tool_definition();
    serde_json::json!({ "name": tool.name, "description": tool.description, "inputSchema": tool.input_schema })
}

#[napi]
impl Host {
    /// One verb for `identity` on `scope`, on a background thread. Resolves
    /// to the envelope index.js unwraps; never blocks the JS event loop.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn dispatch(
        &self,
        env: Env,
        id: String,
        display_name: String,
        scope: String,
        argv: Vec<String>,
        stdin: Option<String>,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        on_worker(env, move || {
            guarded(|| caller_of(id, display_name, scope).and_then(|caller| run(&setup, &caller, &argv, stdin.as_deref())))
        })
    }

    /// One HTTP request for `identity` (none when `id` is absent), on a
    /// background thread. `request` is the JSON text of
    /// `{ method, path, query, headers }`. Resolves to the envelope index.js
    /// unwraps: the response `{ status, headers, body }` with the body as
    /// text, or null when the path is not the door's.
    #[napi(ts_return_type = "Promise<unknown>")]
    pub fn handle_request(
        &self,
        env: Env,
        request: String,
        body: Buffer,
        id: Option<String>,
        display_name: Option<String>,
    ) -> Result<JsObject> {
        let setup = self.setup.clone();
        let body = body.to_vec();
        on_worker(env, move || guarded(|| handle(&setup, &request, body, id.zip(display_name))))
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
fn handle(setup: &HostSetup, request: &str, body: Vec<u8>, who: Option<(String, String)>) -> DispatchResult {
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

    let broken_hook: Arc<Mutex<Option<String>>> = Arc::default();
    let options = HostKitOptions {
        scopes: setup.has_scopes.then(|| scopes_for(setup, &broken_hook)),
        auth_methods: setup.auth_methods.clone(),
        push_transports: setup.push_transports.clone(),
        skills: setup.skills.clone(),
    };
    let kit = HostKit::with_options(setup.store.clone(), hooks_for(setup, &broken_hook), options)
        .expect("the skill list was checked at createHost");
    let response = kit.handle(&request, identity.as_ref());
    hooks_held(&broken_hook)?;
    let Some(response) = response else {
        return Ok(serde_json::Value::Null);
    };
    let headers: serde_json::Map<String, serde_json::Value> =
        response.headers.into_iter().map(|(name, value)| (name, value.into())).collect();
    // The door writes JSON or nothing: the body is always text.
    let body = String::from_utf8(response.body).expect("the door writes UTF-8 JSON");
    Ok(serde_json::json!({ "status": response.status, "headers": headers, "body": body }))
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

fn run(setup: &HostSetup, caller: &Caller, argv: &[String], stdin: Option<&str>) -> DispatchResult {
    let parsed = parse_verb(argv)?;
    let cmd = command_of(&parsed, stdin)?;
    let status_without_name = matches!(cmd, Command::Status { change: None, .. });

    let broken_hook: Arc<Mutex<Option<String>>> = Arc::default();
    let kit = HostKit::new(setup.store.clone(), hooks_for(setup, &broken_hook));
    let result = kit.run(caller, cmd);
    hooks_held(&broken_hook)?;
    match result {
        Ok(execution) => value_of(&parsed, &execution.outcome),
        Err(ExecError::Command(e)) => status_notice(status_without_name, &e)
            .map_or_else(|| Err(DispatchError::new(e.code.as_str(), e.message)), Ok),
        Err(ExecError::Store(e)) => Err(store_error(e)),
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
        Box::new(move |info: &BeforeCommand| decide(&bridge, info, &broken_hook)) as BeforeCommandHook
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
fn decide(bridge: &JsStoreBridge, info: &BeforeCommand, broken_hook: &Mutex<Option<String>>) -> Decision {
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

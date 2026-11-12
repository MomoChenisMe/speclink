//! `createHost` and `host.dispatch` (design D6): the host kit's door for a
//! host's own Agent tool. A host is built once over a JS store port and its
//! hooks; each dispatch brings who acts and on which scope, then takes the
//! same verb parsing, stdin rules and output as `engine.dispatch`, and runs
//! the command through `speclink_host::kit::HostKit`.

use crate::port_bridge::{events_json, person_json, settle_call, JsPort};
use crate::store_bridge::{create_bridge, JsStoreBridge};
use crate::{
    command_of, envelope, on_worker, panic_message, parse_verb, status_notice, value_of, DispatchError, DispatchResult,
};
use napi::bindgen_prelude::*;
use napi::{JsFunction, JsObject};
use napi_derive::napi;
use speclink_core::command::Command;
use speclink_host::kit::{
    AfterCommit, AfterCommitHook, BeforeCommand, BeforeCommandHook, Caller, Decision, Hooks, HostKit, Identity,
};
use speclink_host::port_exec::ExecError;
use speclink_store_port::{PortError, ScopeId, StorePort};
use std::sync::{Arc, Mutex};

/// What `createHost` set up: the store port, the bridge over the hooks, and
/// which hooks the host gave.
struct HostSetup {
    store: Arc<dyn StorePort>,
    /// The bridge over `{ beforeCommand, afterCommit }` (index.js).
    hooks: JsStoreBridge,
    has_before: bool,
    has_after: bool,
}

#[napi]
pub struct Host {
    setup: Arc<HostSetup>,
}

/// Build a host over a JS store port. `port_invoker` and `hook_invoker` are
/// index.js's invokers over the store's three functions and over the two
/// hooks; the flags say which hooks the host gave.
#[napi(js_name = "hostFromPort")]
pub fn host_from_port(
    env: Env,
    port_invoker: JsFunction,
    hook_invoker: JsFunction,
    has_before: bool,
    has_after: bool,
) -> Result<Host> {
    let store: Arc<dyn StorePort> = Arc::new(JsPort::new(create_bridge(env, port_invoker)?));
    let hooks = create_bridge(env, hook_invoker)?;
    Ok(Host { setup: Arc::new(HostSetup { store, hooks, has_before, has_after }) })
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
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                caller_of(id, display_name, scope).and_then(|caller| run(&setup, &caller, &argv, stdin.as_deref()))
            }))
            .unwrap_or_else(|payload| {
                Err(DispatchError::new("internal", format!("speclink host panicked: {}", panic_message(&payload))))
            });
            envelope(result)
        })
    }
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

    // A JS hook that throws or answers in a shape it does not know has to end
    // the call as `internal`; the Rust hook can only allow or deny, so it
    // denies and leaves the reason here.
    let broken_hook: Arc<Mutex<Option<String>>> = Arc::default();
    let kit = HostKit::new(setup.store.clone(), hooks_for(setup, &broken_hook));
    let result = kit.run(caller, cmd);
    if let Some(message) = broken_hook.lock().unwrap_or_else(|e| e.into_inner()).take() {
        return Err(DispatchError::new("internal", message));
    }
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

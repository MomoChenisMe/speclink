//! The host kit's HTTP door on the in-memory store port behind the shared
//! recorder, for the tests that send requests through it: project `alpha`,
//! one repo `web` on scope `s1`, and the caller Alice.

use serde_json::Value;
use speclink_host::http::{HttpRequest, HttpResponse, Method};
use speclink_host::kit::{Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, RepoEntry};
use speclink_store_port::{DocKey, MemoryStorePort, ScopeId, StorePort};
use std::sync::Arc;

pub const API: &str = "/api/speclink/v1";

pub struct Door {
    pub kit: HostKit,
    pub store: Arc<super::Recorder>,
}

pub fn scope_s1() -> ScopeId {
    ScopeId::new("s1").unwrap()
}

pub fn alice() -> Identity {
    Identity::new("u_42", "Alice").unwrap()
}

/// Project alpha, one repo web on scope s1, with this write switch.
pub fn door_with(can_write: bool, hooks: Hooks) -> Door {
    let repo = RepoEntry {
        key: "web".to_string(),
        name: "WEB".to_string(),
        scope: scope_s1(),
        can_write,
        can_write_policy: can_write,
    };
    let projects = vec![ProjectEntry { key: "alpha".to_string(), name: "Project alpha".to_string(), repos: vec![repo] }];
    let options = HostKitOptions {
        scopes: Some(Box::new(move |_identity: &Identity| Ok(projects.clone()))),
        ..Default::default()
    };
    let store = Arc::new(super::Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::with_options(store.clone(), hooks, options).expect("the options name no unknown skill");
    Door { kit, store }
}

pub fn door() -> Door {
    door_with(true, Hooks::default())
}

/// Seed `docs` into scope s1, then forget the calls.
pub fn seed(door: &Door, docs: &[(&str, &str)]) {
    super::write_in(&door.store.inner, &scope_s1(), docs);
    door.store.take();
}

pub fn content(door: &Door, key: &str) -> Option<String> {
    door.store.inner.read(&scope_s1(), &[DocKey::new(key).unwrap()]).unwrap().remove(0).map(|d| d.content)
}

pub fn request(method: Method, tail: &str, body: Option<Value>) -> HttpRequest {
    HttpRequest {
        method,
        path: format!("{API}/projects/alpha/{tail}"),
        query: None,
        headers: vec![("x-speclink-api-version".to_string(), speclink_protocol::API_VERSION.to_string())],
        body: body.map(|b| serde_json::to_vec(&b).unwrap()).unwrap_or_default(),
    }
}

pub fn answer(door: &Door, req: &HttpRequest) -> HttpResponse {
    door.kit
        .handle(req, Some(&alice()))
        .unwrap_or_else(|| panic!("{} {} is the door's path", req.method.as_str(), req.path))
}

pub fn body_of(response: &HttpResponse) -> Value {
    serde_json::from_slice(&response.body)
        .unwrap_or_else(|e| panic!("a JSON body ({e}): {:?}", String::from_utf8_lossy(&response.body)))
}

/// GET `tail` as JSON; the request must succeed.
pub fn get(door: &Door, tail: &str) -> Value {
    let response = answer(door, &request(Method::Get, tail, None));
    assert_eq!(response.status, 200, "GET {tail}: {:?}", String::from_utf8_lossy(&response.body));
    body_of(&response)
}

pub fn post(door: &Door, tail: &str, body: Value) -> HttpResponse {
    answer(door, &request(Method::Post, tail, Some(body)))
}

/// The error envelope's status and reason.
pub fn error(response: &HttpResponse) -> (u16, String) {
    let body = body_of(response);
    (response.status, body["reason"].as_str().expect("a reason").to_string())
}

pub fn etag_of(response: &HttpResponse) -> u64 {
    let (_, value) = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("etag"))
        .unwrap_or_else(|| panic!("an ETag on {}", response.status));
    value.trim_matches('"').parse().expect("the ETag is a quoted number")
}

/// The item named `name` of the change list.
pub fn listed(door: &Door, name: &str) -> Value {
    let list = get(door, "changes");
    list["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("'{name}' in {list}"))
        .clone()
}

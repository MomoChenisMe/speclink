//! The HTTP door of the host kit (spec host-kit「HTTP 入口處理原版用戶端的請求」
//! and the requirements after it): a request with the host's identity goes in,
//! a response — or "not the door's path" — comes out, with no HTTP framework.
//! Every scenario runs on the in-memory store port behind the shared recorder,
//! so a test can say what the store received.

use speclink_host::http::{match_route, HttpRequest, HttpResponse, Method};
use speclink_host::kit::{Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, RepoEntry};
use speclink_store_port::{MemoryStorePort, ScopeId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

mod common;

const API: &str = "/api/speclink/v1";

// --- what the host gives the door ---

fn scope_of(id: &str) -> ScopeId {
    ScopeId::new(id).unwrap()
}

fn alice() -> Identity {
    Identity::new("u_42", "Alice").unwrap()
}

fn repo(key: &str, scope: &str) -> RepoEntry {
    RepoEntry {
        key: key.to_string(),
        name: key.to_uppercase(),
        scope: scope_of(scope),
        can_write: true,
        can_write_policy: true,
    }
}

fn project(key: &str, repos: Vec<RepoEntry>) -> ProjectEntry {
    ProjectEntry { key: key.to_string(), name: format!("Project {key}"), repos }
}

/// Options whose scope function answers `projects` for every identity.
fn scopes(projects: Vec<ProjectEntry>) -> HostKitOptions {
    HostKitOptions {
        scopes: Some(Box::new(move |_identity: &Identity| Ok(projects.clone()))),
        ..Default::default()
    }
}

/// Project alpha with one repo, web, on scope s1.
fn one_repo() -> HostKitOptions {
    scopes(vec![project("alpha", vec![repo("web", "s1")])])
}

/// Project alpha with two repos: web on s1, api on s2.
fn two_repos() -> HostKitOptions {
    scopes(vec![project("alpha", vec![repo("web", "s1"), repo("api", "s2")])])
}

struct Door {
    kit: HostKit,
    store: Arc<common::Recorder>,
}

fn door(options: HostKitOptions) -> Door {
    door_with(options, Hooks::default())
}

fn door_with(options: HostKitOptions, hooks: Hooks) -> Door {
    let store = Arc::new(common::Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::with_options(store.clone(), hooks, options).expect("the options name no unknown skill");
    Door { kit, store }
}

// --- requests and responses ---

fn request(method: Method, path: &str) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_string(),
        query: None,
        headers: vec![("x-speclink-api-version".to_string(), speclink_protocol::API_VERSION.to_string())],
        body: Vec::new(),
    }
}

fn get(path: &str) -> HttpRequest {
    request(Method::Get, path)
}

/// `path` under project alpha.
fn alpha(tail: &str) -> String {
    format!("{API}/projects/alpha/{tail}")
}

trait With {
    fn header(self, name: &str, value: &str) -> Self;
    fn without(self, name: &str) -> Self;
}

impl With for HttpRequest {
    fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(name));
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    fn without(mut self, name: &str) -> Self {
        self.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(name));
        self
    }
}

fn answer(door: &Door, req: &HttpRequest, identity: Option<&Identity>) -> HttpResponse {
    door.kit
        .handle(req, identity)
        .unwrap_or_else(|| panic!("{} {} is the door's path", req.method.as_str(), req.path))
}

fn json(response: &HttpResponse) -> serde_json::Value {
    serde_json::from_slice(&response.body)
        .unwrap_or_else(|e| panic!("a JSON body ({e}): {:?}", String::from_utf8_lossy(&response.body)))
}

/// The status, the reason and the message of an error response.
fn error(response: &HttpResponse) -> (u16, String, String) {
    let body = json(response);
    assert_eq!(body["status"], response.status, "the envelope repeats the status: {body}");
    (
        response.status,
        body["reason"].as_str().expect("a reason").to_string(),
        body["message"].as_str().expect("a message").to_string(),
    )
}

fn header<'r>(response: &'r HttpResponse, name: &str) -> Option<&'r str> {
    response.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
}

// --- 需要身分的路徑沒有身分時回 401 ---

#[test]
fn without_an_identity_a_project_request_is_401_and_the_store_sees_nothing() {
    let asked = Arc::new(AtomicUsize::new(0));
    let counted = asked.clone();
    let options = HostKitOptions {
        scopes: Some(Box::new(move |_: &Identity| {
            counted.fetch_add(1, Ordering::SeqCst);
            Ok(vec![project("alpha", vec![repo("web", "s1")])])
        })),
        ..Default::default()
    };
    let door = door(options);

    let response = answer(&door, &get(&alpha("changes")), None);

    let (status, reason, _) = error(&response);
    assert_eq!((status, reason.as_str()), (401, "permission_denied"));
    assert!(door.store.calls().is_empty(), "the store was not called: {:?}", door.store.calls());
    assert_eq!(asked.load(Ordering::SeqCst), 0, "the scope function was not asked");
}

#[test]
fn without_an_identity_the_api_root_paths_that_need_one_are_401() {
    let door = door(one_repo());
    for path in [format!("{API}/scopes"), format!("{API}/whoami")] {
        let (status, reason, _) = error(&answer(&door, &get(&path), None));
        assert_eq!((status, reason.as_str()), (401, "permission_denied"), "{path}");
    }
}

// --- API 版本不符 ---

#[test]
fn a_wrong_or_missing_api_version_is_refused_before_the_store() {
    let door = door(one_repo());

    let wrong = answer(&door, &get(&alpha("changes")).header("x-speclink-api-version", "0"), Some(&alice()));
    let (status, reason, message) = error(&wrong);
    assert_eq!((status, reason.as_str()), (409, "refused"));
    assert_eq!(message, "incompatible api version — this server speaks version '1', client sent '0'");

    let missing = answer(&door, &get(&alpha("changes")).without("x-speclink-api-version"), Some(&alice()));
    let (status, reason, message) = error(&missing);
    assert_eq!((status, reason.as_str()), (409, "refused"));
    assert_eq!(message, "incompatible api version — this server speaks version '1'");

    assert!(door.store.calls().is_empty(), "the store was not called");
}

// --- 範圍由宿主的函式決定 ---

#[test]
fn a_project_not_in_the_callers_list_is_404_before_the_store() {
    let door = door(one_repo());

    let response = answer(&door, &get(&format!("{API}/projects/beta/changes")), Some(&alice()));

    let (status, reason, message) = error(&response);
    assert_eq!((status, reason.as_str()), (404, "not_found"));
    assert_eq!(message, "project 'beta' not found");
    assert!(door.store.calls().is_empty(), "the store was not called");
}

#[test]
fn a_named_repo_not_in_the_projects_list_is_404() {
    let door = door(two_repos());

    let response = answer(&door, &get(&alpha("changes")).header("x-speclink-repo", "mobile"), Some(&alice()));

    let (status, reason, message) = error(&response);
    assert_eq!((status, reason.as_str()), (404, "not_found"));
    assert_eq!(message, "repo 'mobile' is not registered in project 'alpha'");
    assert!(door.store.calls().is_empty(), "the store was not called");
}

#[test]
fn several_repos_without_a_named_one_are_refused_and_listed() {
    let door = door(two_repos());

    let response = answer(&door, &get(&alpha("changes")), Some(&alice()));

    let (status, reason, message) = error(&response);
    assert_eq!((status, reason.as_str()), (409, "refused"));
    assert_eq!(
        message,
        "project 'alpha' registers multiple repos — specify one with X-Speclink-Repo (candidates: alpha/web, alpha/api)"
    );
    assert!(door.store.calls().is_empty(), "no scope was read");
}

#[test]
fn a_project_without_repos_is_404() {
    let door = door(scopes(vec![project("alpha", Vec::new())]));

    let (status, reason, message) = error(&answer(&door, &get(&alpha("changes")), Some(&alice())));

    assert_eq!((status, reason.as_str()), (404, "not_found"));
    assert_eq!(message, "project 'alpha' has no registered repo");
}

#[test]
fn without_a_scope_function_the_scope_list_and_every_project_path_are_404() {
    let store = Arc::new(common::Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::new(store.clone(), Hooks::default());

    for path in [format!("{API}/scopes"), alpha("changes"), alpha("binding")] {
        let response = kit.handle(&get(&path), Some(&alice())).expect("the door's path");
        let (status, reason, _) = error(&response);
        assert_eq!((status, reason.as_str()), (404, "not_found"), "{path}");
    }
    assert!(store.calls().is_empty(), "the store was not called");
}

#[test]
fn a_failing_scope_function_is_503_and_names_no_host_detail() {
    let options = HostKitOptions {
        scopes: Some(Box::new(|_: &Identity| Err("db at 10.0.0.7 refused the connection".to_string()))),
        ..Default::default()
    };
    let door = door(options);

    for path in [alpha("changes"), format!("{API}/scopes")] {
        let (status, reason, message) = error(&answer(&door, &get(&path), Some(&alice())));
        assert_eq!((status, reason.as_str()), (503, "unavailable"), "{path}");
        assert!(!message.contains("10.0.0.7"), "the host's text stays inside: {message}");
    }
    assert!(door.store.calls().is_empty(), "the store was not called");
}

// --- 入口不認得的路徑交還宿主 ---

#[test]
fn a_path_the_door_does_not_know_goes_back_to_the_host() {
    let door = door(one_repo());
    let unknown = [
        get(&alpha("nonsense")),
        get(&alpha("changes/demo/nonsense")),
        // The two routes the demo server keeps for itself.
        request(Method::Post, &alpha("import")),
        get(&alpha("events")),
        // A known path with a method it does not have.
        request(Method::Post, &alpha("changes/demo/drift")),
        get("/healthz"),
        get("/auth/device"),
        get(&format!("{API}/web/session")),
        get(&format!("{API}/admin/users")),
        get(&format!("{API}/projects/alpha")),
    ];
    for req in unknown {
        assert!(
            door.kit.handle(&req, Some(&alice())).is_none(),
            "{} {} is not the door's path",
            req.method.as_str(),
            req.path
        );
    }
    // A path outside the door needs no identity to be handed back.
    assert!(door.kit.handle(&get("/healthz"), None).is_none());
    assert!(door.store.calls().is_empty(), "the store was not called");
}

// --- 標頭名稱不分大小寫 ---

#[test]
fn header_names_match_in_any_case() {
    let door = door(two_repos());

    let repo_named = get(&alpha("changes"))
        .without("x-speclink-api-version")
        .header("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .header("X-SPECLINK-REPO", "mobile");
    let (status, _, message) = error(&answer(&door, &repo_named, Some(&alice())));
    assert_eq!(status, 404, "the version and the repo header were both read: {message}");
    assert!(message.contains("'mobile'"), "{message}");
}

// --- 路徑比對與今天示範 server 的行為相同 ---

/// The pattern and the params (still percent-encoded) `path` matches for
/// `method`, or none.
fn matched(method: Method, path: &str) -> Option<(&'static str, Vec<String>)> {
    match_route(method, path).map(|m| (m.pattern, m.params))
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| v.to_string()).collect()
}

#[test]
fn a_path_matches_one_route_with_its_params() {
    assert_eq!(matched(Method::Get, &alpha("changes")), Some(("/changes", strings(&["alpha"]))));
    assert_eq!(matched(Method::Post, &alpha("changes")), Some(("/changes", strings(&["alpha"]))));
    assert_eq!(
        matched(Method::Get, &alpha("changes/demo/drift")),
        Some(("/changes/{name}/drift", strings(&["alpha", "demo"])))
    );
    assert_eq!(
        matched(Method::Post, &alpha("changes/demo/tasks/1.1/done")),
        Some(("/changes/{name}/tasks/{task_id}/done", strings(&["alpha", "demo", "1.1"])))
    );
}

#[test]
fn a_trailing_slash_or_an_empty_segment_matches_nothing() {
    for path in [
        alpha("changes/"),
        alpha("changes/demo/"),
        alpha("changes//drift"),
        alpha("changes/demo/artifacts/"),
        format!("{API}/projects//changes"),
        format!("{API}/projects/alpha/"),
    ] {
        assert_eq!(matched(Method::Get, &path), None, "{path}");
    }
}

#[test]
fn an_encoded_slash_stays_inside_its_segment() {
    assert_eq!(
        matched(Method::Get, &alpha("changes/a%2Fb/drift")),
        Some(("/changes/{name}/drift", strings(&["alpha", "a%2Fb"])))
    );
    assert_eq!(matched(Method::Get, &alpha("changes/a%2Fb%2Fdrift")), Some(("/changes/{name}", strings(&["alpha", "a%2Fb%2Fdrift"]))));
}

#[test]
fn the_rest_of_the_path_segment_takes_every_segment_after_it() {
    assert_eq!(
        matched(Method::Get, &alpha("changes/demo/artifacts/specs/auth")),
        Some(("/changes/{name}/artifacts/{*artifact}", strings(&["alpha", "demo", "specs/auth"])))
    );
    assert_eq!(
        matched(Method::Put, &alpha("changes/demo/artifacts/proposal")),
        Some(("/changes/{name}/artifacts/{*artifact}", strings(&["alpha", "demo", "proposal"])))
    );
    assert_eq!(
        matched(Method::Get, &alpha("changes/demo/instructions/apply")),
        Some(("/changes/{name}/instructions/{*artifact}", strings(&["alpha", "demo", "apply"])))
    );
    assert_eq!(
        matched(Method::Get, &alpha("archived/2026-01-01-demo/artifacts/specs/auth/spec.md")),
        Some((
            "/archived/{dated_name}/artifacts/{*artifact}",
            strings(&["alpha", "2026-01-01-demo", "specs/auth/spec.md"])
        ))
    );
}

#[test]
fn a_fixed_segment_wins_over_a_named_one_without_falling_back() {
    assert_eq!(matched(Method::Get, &alpha("discussions/search")), Some(("/discussions/search", strings(&["alpha"]))));
    // DELETE is not a method of the fixed route: no fall back to {slug}.
    assert_eq!(matched(Method::Delete, &alpha("discussions/search")), None);
    assert_eq!(
        matched(Method::Delete, &alpha("discussions/talk")),
        Some(("/discussions/{slug}", strings(&["alpha", "talk"])))
    );
    assert_eq!(
        matched(Method::Post, &alpha("changes/demo/tasks/move")),
        Some(("/changes/{name}/tasks/move", strings(&["alpha", "demo"])))
    );
}

#[test]
fn head_matches_the_get_routes() {
    assert_eq!(matched(Method::Head, &alpha("changes")), Some(("/changes", strings(&["alpha"]))));
    assert_eq!(matched(Method::Head, &alpha("changes/demo/archive")), None);
}

#[test]
fn the_method_names_are_the_five_the_door_takes() {
    for (name, method) in [
        ("GET", Method::Get),
        ("HEAD", Method::Head),
        ("POST", Method::Post),
        ("PUT", Method::Put),
        ("DELETE", Method::Delete),
    ] {
        assert_eq!(Method::from_name(name), Some(method));
        assert_eq!(method.as_str(), name);
    }
    for other in ["PATCH", "OPTIONS", "get", ""] {
        assert_eq!(Method::from_name(other), None, "{other:?}");
    }
}

// ======================================================================
// The routes (design D4, D5): handshake, polling, identity, scope list,
// login methods, hooks and the two capability switches.
// ======================================================================

use serde_json::{json, Value};
use speclink_host::kit::{AfterCommit, BeforeCommand, Decision};
use speclink_protocol::auth::{AuthMethodsResponse, BrowserLogin};
use speclink_protocol::events::{EventTransport, TransportKind};
use std::sync::Mutex;

const META: &str = "schema: spec-driven\n";
const OPEN_TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n";
const CONFIG: &str = "schema: spec-driven\ntdd: false\n";

/// Seed `docs` into scope `scope` of the door's store, then forget the calls.
fn seed(door: &Door, scope: &str, docs: &[(&str, &str)]) {
    common::write_in(&door.store.inner, &scope_of(scope), docs);
    door.store.take();
}

/// A store with change `demo` (one open task) and a config, on scope s1.
fn with_demo(door: &Door) {
    seed(
        door,
        "s1",
        &[("changes/demo/.meta.yaml", META), ("changes/demo/tasks.md", OPEN_TASKS), ("config.yaml", CONFIG)],
    );
}

fn content(door: &Door, scope: &str, key: &str) -> Option<String> {
    use speclink_store_port::{DocKey, StorePort};
    door.store.inner.read(&scope_of(scope), &[DocKey::new(key).unwrap()]).unwrap().remove(0).map(|d| d.content)
}

fn send(method: Method, path: &str, body: Value) -> HttpRequest {
    let mut req = request(method, path);
    req.body = serde_json::to_vec(&body).unwrap();
    req
}

fn etag_of(response: &HttpResponse) -> u64 {
    header(response, "etag")
        .unwrap_or_else(|| panic!("an ETag on {}", response.status))
        .trim_matches('"')
        .parse()
        .expect("the ETag is a quoted number")
}

/// A JSON object's keys, sorted.
fn keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap_or_else(|| panic!("an object: {value}")).keys().cloned().collect();
    keys.sort();
    keys
}

/// What the hooks saw: (command, params) asked before, and (command, event
/// names) watched after.
#[derive(Default)]
struct Seen {
    asked: Mutex<Vec<(String, Value)>>,
    watched: Mutex<Vec<(String, Vec<String>)>>,
}

/// Hooks that record into `seen` and answer the before hook with `decide`.
fn recording(seen: &Arc<Seen>, decide: impl Fn(&BeforeCommand) -> Decision + Send + Sync + 'static) -> Hooks {
    let (before, after) = (seen.clone(), seen.clone());
    Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand| {
            before.asked.lock().unwrap().push((info.command.to_string(), info.params.clone()));
            decide(info)
        })),
        after_commit: Some(Box::new(move |info: &AfterCommit| {
            let events = info.events.iter().map(|e| e.name().to_string()).collect();
            after.watched.lock().unwrap().push((info.command.to_string(), events));
        })),
    }
}

fn deny(command: &'static str, reason: &'static str) -> impl Fn(&BeforeCommand) -> Decision + Send + Sync {
    move |info: &BeforeCommand| match info.command == command {
        true => Decision::Deny { reason: reason.to_string() },
        false => Decision::Allow,
    }
}

fn allow(_: &BeforeCommand) -> Decision {
    Decision::Allow
}

/// Options with project alpha, one repo web on s1, with these switches.
fn switches(can_write: bool, can_write_policy: bool) -> HostKitOptions {
    let mut web = repo("web", "s1");
    web.can_write = can_write;
    web.can_write_policy = can_write_policy;
    scopes(vec![project("alpha", vec![web])])
}

// --- 不用 HTTP 框架完成一次握手與寫入 ---

#[test]
fn a_handshake_a_create_and_a_list_need_no_framework() {
    let door = door(one_repo());
    let me = alice();

    let handshake = answer(&door, &get(&alpha("binding")), Some(&me));
    assert_eq!(handshake.status, 200);
    assert_eq!(header(&handshake, "content-type"), Some("application/json"));
    let body = json(&handshake);
    assert_eq!(keys(&body), ["actor", "apiVersion", "capabilities", "engineVersion", "project", "repo"]);
    assert_eq!(body["actor"], json!({ "id": "u_42", "name": "Alice" }));
    assert_eq!(body["project"], json!({ "id": "prj_alpha", "key": "alpha", "name": "Project alpha" }));
    assert_eq!(body["repo"], json!({ "id": "repo_web", "key": "web", "name": "WEB" }));
    assert_eq!(body["apiVersion"], speclink_protocol::API_VERSION);
    assert!(body["engineVersion"].as_str().is_some_and(|v| !v.is_empty()), "{body}");
    let at_handshake = etag_of(&answer(&door, &get(&alpha("sync-state")), Some(&me)));

    let created = answer(&door, &send(Method::Post, &alpha("changes"), json!({ "name": "demo" })), Some(&me));
    assert_eq!(created.status, 200, "{:?}", String::from_utf8_lossy(&created.body));
    let body = json(&created);
    assert_eq!(body["name"], "demo");
    assert_eq!(body["repo"], "web");
    assert!(body["schema"].is_string(), "{body}");
    assert!(etag_of(&created) > at_handshake, "the create's ETag is past the handshake's version");

    let listed = answer(&door, &get(&alpha("changes")), Some(&me));
    assert_eq!(listed.status, 200);
    let body = json(&listed);
    let names: Vec<&str> = body["changes"].as_array().expect("changes").iter().filter_map(|c| c["name"].as_str()).collect();
    assert_eq!(names, ["demo"]);
    let first = &body["changes"][0];
    assert!(first["completedTasks"].is_u64() && first["totalTasks"].is_u64(), "camelCase counts: {first}");
    assert_eq!(etag_of(&listed), etag_of(&created), "no commit between the two");
}

#[test]
fn an_archive_through_the_door_moves_the_change_and_reports_it() {
    let door = door(one_repo());
    seed(
        &door,
        "s1",
        &[
            ("changes/demo/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-01\n"),
            ("changes/demo/tasks.md", "- [x] 1.1 done\n"),
            (
                "changes/demo/specs/user-auth/spec.md",
                "## Purpose\n\n本 capability 是測試用的示範能力，涵蓋一個可觀察行為與其成功路徑，專供 HTTP 入口的測試取用。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
            ),
        ],
    );

    let response = answer(&door, &request(Method::Post, &alpha("changes/demo/archive")), Some(&alice()));

    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    let body = json(&response);
    assert!(body["datedName"].as_str().is_some_and(|name| name.ends_with("-demo")), "{body}");
    assert_eq!(body["specs"][0]["capability"], "user-auth");
    assert_eq!(content(&door, "s1", "changes/demo/.meta.yaml"), None, "the change left the active changes");
    assert!(content(&door, "s1", "specs/user-auth/spec.md").is_some(), "the delta landed in the specs");
}

#[test]
fn head_answers_like_get_without_a_body() {
    let door = door(one_repo());
    with_demo(&door);
    let got = answer(&door, &get(&alpha("changes")), Some(&alice()));
    let head = answer(&door, &request(Method::Head, &alpha("changes")), Some(&alice()));
    assert_eq!(head.status, got.status);
    assert_eq!(header(&head, "etag"), header(&got, "etag"));
    assert!(head.body.is_empty());
}

#[test]
fn a_body_that_is_not_json_is_400_and_nothing_is_committed() {
    let door = door(one_repo());
    let mut req = request(Method::Post, &alpha("changes"));
    req.body = b"{name: demo".to_vec();

    let (status, reason, _) = error(&answer(&door, &req, Some(&alice())));

    assert_eq!((status, reason.as_str()), (400, "invalid_argument"));
    assert!(door.store.calls().commits.is_empty(), "no commit");

    let missing = answer(&door, &send(Method::Post, &alpha("changes"), json!({ "description": "x" })), Some(&alice()));
    assert_eq!(error(&missing).0, 400, "a missing field is the same 400");
}

#[test]
fn a_path_param_reaches_the_command_decoded() {
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, |_| Decision::Deny { reason: "no".into() }));

    answer(&door, &request(Method::Post, &alpha("changes/a%20b/claim")), Some(&alice()));

    let asked = seen.asked.lock().unwrap().clone();
    assert_eq!(asked, vec![("claim".to_string(), json!({ "name": "a b" }))]);
}

// --- 掛鉤經入口照樣生效 ---

#[test]
fn a_hook_refusal_through_the_door_is_409_with_the_hosts_reason() {
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, deny("archive", "程式碼還沒合併")));
    with_demo(&door);

    let response = answer(&door, &request(Method::Post, &alpha("changes/demo/archive")), Some(&alice()));

    let (status, reason, message) = error(&response);
    assert_eq!((status, reason.as_str()), (409, "refused"));
    assert_eq!(message, "程式碼還沒合併");
    assert_eq!(content(&door, "s1", "changes/demo/.meta.yaml").as_deref(), Some(META), "the change is still active");
    assert!(door.store.calls().commits.is_empty(), "no commit");
}

#[test]
fn a_board_order_write_goes_through_the_hooks() {
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, deny("put-board-order", "看板凍結中")));
    with_demo(&door);
    let revision = etag_of(&answer(&door, &get(&alpha("board-order")), Some(&alice())));

    let put = send(Method::Put, &alpha("board-order"), json!({ "content": "{\"changes\":{}}" }))
        .header("if-match", &format!("\"{revision}\""));
    let (status, reason, message) = error(&answer(&door, &put, Some(&alice())));

    assert_eq!((status, reason.as_str(), message.as_str()), (409, "refused", "看板凍結中"));
    assert_eq!(content(&door, "s1", "board/order"), None, "the board order is unchanged");
    assert_eq!(seen.asked.lock().unwrap().clone(), vec![("put-board-order".to_string(), json!({}))]);
    assert!(seen.watched.lock().unwrap().is_empty());

    // Allowed, the write lands and the after hook watches it.
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, allow));
    let put = send(Method::Put, &alpha("board-order"), json!({ "content": "{\"changes\":{}}" })).header("if-match", "\"0\"");
    let response = answer(&door, &put, Some(&alice()));
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert_eq!(json(&response)["revision"], etag_of(&response));
    assert_eq!(
        seen.watched.lock().unwrap().clone(),
        vec![("put-board-order".to_string(), vec!["board-order-updated".to_string()])]
    );
}

#[test]
fn the_after_hook_watches_a_commit_through_the_door() {
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, allow));
    with_demo(&door);

    let response = answer(&door, &send(Method::Post, &alpha("changes/demo/tasks/1/done"), json!({})), Some(&alice()));

    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert_eq!(json(&response)["alreadyDone"], false);
    assert_eq!(
        seen.watched.lock().unwrap().clone(),
        vec![("task-done".to_string(), vec!["task-completed".to_string()])]
    );
}

// --- 能力開關的把關 ---

#[test]
fn without_write_a_write_is_403_and_a_read_and_a_context_snapshot_still_work() {
    let door = door(switches(false, false));
    with_demo(&door);

    let (status, reason, message) =
        error(&answer(&door, &send(Method::Post, &alpha("changes"), json!({ "name": "other" })), Some(&alice())));
    assert_eq!((status, reason.as_str()), (403, "permission_denied"));
    assert_eq!(message, "your role in project 'alpha' is reader; this action needs the editor role");
    assert!(door.store.calls().commits.is_empty(), "no commit");

    assert_eq!(answer(&door, &get(&alpha("changes")), Some(&alice())).status, 200);
    let snapshot = answer(&door, &send(Method::Post, &alpha("context"), json!({})), Some(&alice()));
    assert_eq!(snapshot.status, 200, "{:?}", String::from_utf8_lossy(&snapshot.body));
    assert!(json(&snapshot)["snapshotId"].is_string());
}

#[test]
fn without_policy_write_a_policy_key_change_is_403_and_a_context_change_lands() {
    let door = door(switches(true, false));
    with_demo(&door);
    let revision = json(&answer(&door, &get(&alpha("config")), Some(&alice())))["revision"].as_u64().unwrap();

    let tdd = send(
        Method::Put,
        &alpha("config"),
        json!({ "content": "schema: spec-driven\ntdd: true\n", "expectedRevision": revision }),
    );
    let (status, reason, _) = error(&answer(&door, &tdd, Some(&alice())));
    assert_eq!((status, reason.as_str()), (403, "permission_denied"));
    assert_eq!(content(&door, "s1", "config.yaml").as_deref(), Some(CONFIG), "the config is unchanged");

    let context = send(
        Method::Put,
        &alpha("config"),
        json!({ "content": "schema: spec-driven\ntdd: false\ncontext: our stack\n", "expectedRevision": revision }),
    );
    let response = answer(&door, &context, Some(&alice()));
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert!(json(&response)["revision"].as_u64().unwrap() > revision);
}

#[test]
fn the_handshake_declares_what_the_switches_allow() {
    let caps = |options| {
        let door = door(options);
        json(&answer(&door, &get(&alpha("binding")), Some(&alice())))["capabilities"].clone()
    };

    let reader = caps(switches(false, true));
    for field in ["policyWrite", "deleteChange", "moveTask"] {
        assert_eq!(reader[field], false, "{field}: {reader}");
    }
    for field in ["contextSnapshots", "validate", "analyze"] {
        assert_eq!(reader[field], true, "{field}: {reader}");
    }

    let writer = caps(switches(true, false));
    assert_eq!((writer["policyWrite"].clone(), writer["deleteChange"].clone()), (json!(false), json!(true)));

    let editor = caps(switches(true, true));
    assert_eq!(keys(&editor), [
        "analyze", "authentication", "contextSnapshots", "deleteChange", "events", "moveTask", "policyWrite", "skills",
        "validate"
    ]);
    assert_eq!(editor["policyWrite"], true);
}

// --- 握手、輪詢與範圍清單 ---

#[test]
fn by_default_the_handshake_declares_polling_only() {
    let door = door(one_repo());
    let events = &json(&answer(&door, &get(&alpha("binding")), Some(&alice())))["capabilities"]["events"];
    assert_eq!(events["transports"], json!([]));
    assert_eq!(events["polling"], json!({ "url": "/sync-state", "etag": true }));
}

#[test]
fn a_push_stream_the_host_declares_is_in_the_handshake() {
    let mut options = one_repo();
    options.push_transports =
        vec![EventTransport { kind: TransportKind::Sse, url: "https://host.example/events".into(), resume: true }];
    let door = door(options);
    let events = &json(&answer(&door, &get(&alpha("binding")), Some(&alice())))["capabilities"]["events"];
    assert_eq!(events["transports"], json!([{ "type": "sse", "url": "https://host.example/events", "resume": true }]));
    assert_eq!(events["polling"]["url"], "/sync-state");
}

#[test]
fn polling_an_unchanged_scope_is_304_and_reads_no_document() {
    let door = door(one_repo());
    with_demo(&door);
    let tag = header(&answer(&door, &get(&alpha("changes")), Some(&alice())), "etag").unwrap().to_string();
    door.store.take();

    let poll = answer(&door, &get(&alpha("sync-state")).header("if-none-match", &tag), Some(&alice()));

    assert_eq!(poll.status, 304);
    assert_eq!(header(&poll, "etag"), Some(tag.as_str()));
    assert!(poll.body.is_empty());
    assert!(door.store.calls().reads.is_empty(), "no document was read: {:?}", door.store.calls().reads);
}

#[test]
fn polling_after_a_commit_is_200_with_the_new_etag() {
    let door = door(one_repo());
    with_demo(&door);
    let old = header(&answer(&door, &get(&alpha("changes")), Some(&alice())), "etag").unwrap().to_string();
    answer(&door, &send(Method::Post, &alpha("changes"), json!({ "name": "next" })), Some(&alice()));

    let poll = answer(&door, &get(&alpha("sync-state")).header("if-none-match", &old), Some(&alice()));

    assert_eq!(poll.status, 200);
    assert_ne!(header(&poll, "etag"), Some(old.as_str()));
    assert_eq!(json(&poll), json!({}));
}

#[test]
fn the_scope_list_has_the_projects_and_repos_without_scope_ids_or_switches() {
    let door = door(two_repos());

    let response = answer(&door, &get(&format!("{API}/scopes")), Some(&alice()));

    assert_eq!(response.status, 200);
    assert_eq!(
        json(&response),
        json!({ "projects": [{
            "id": "prj_alpha", "key": "alpha", "name": "Project alpha",
            "repos": [
                { "id": "repo_web", "key": "web", "name": "WEB" },
                { "id": "repo_api", "key": "api", "name": "API" },
            ],
        }] })
    );
}

#[test]
fn the_project_whoami_names_the_identity_and_the_projects_repos() {
    let door = door(two_repos());

    let response = answer(&door, &get(&alpha("whoami")).header("x-speclink-repo", "api"), Some(&alice()));

    assert_eq!(response.status, 200);
    assert_eq!(
        json(&response),
        json!({
            "user": { "name": "Alice", "handle": "u_42" },
            "repos": [{ "name": "web", "gitUrl": "" }, { "name": "api", "gitUrl": "" }],
        })
    );
    etag_of(&response);
}

#[test]
fn the_api_root_whoami_names_the_identity_and_needs_no_project_or_version() {
    // A user in no project: the scope function lists nothing.
    let door = door(scopes(Vec::new()));
    let req = get(&format!("{API}/whoami")).without("x-speclink-api-version");

    let response = answer(&door, &req, Some(&alice()));

    assert_eq!(response.status, 200);
    assert_eq!(json(&response), json!({ "user": { "name": "Alice", "handle": "u_42" } }));
}

// --- 登入方式由宿主宣告 ---

#[test]
fn declared_login_methods_are_answered_without_an_identity_or_a_version() {
    let mut options = one_repo();
    options.auth_methods = Some(AuthMethodsResponse {
        methods: vec!["browser".into(), "token".into()],
        browser: Some(BrowserLogin {
            issuer: "https://id.example.com".into(),
            client_id: "speclink".into(),
            scopes: Vec::new(),
        }),
    });
    let door = door(options);
    let req = get(&format!("{API}/auth-methods")).without("x-speclink-api-version");

    let response = answer(&door, &req, None);

    assert_eq!(response.status, 200);
    let body = json(&response);
    assert_eq!(body["methods"], json!(["browser", "token"]));
    assert_eq!(body["browser"]["issuer"], "https://id.example.com");
    assert_eq!(body["browser"]["clientId"], "speclink");
}

#[test]
fn without_a_declaration_auth_methods_is_404() {
    let door = door(one_repo());
    let (status, reason, _) = error(&answer(&door, &get(&format!("{API}/auth-methods")), None));
    assert_eq!((status, reason.as_str()), (404, "not_found"));
}

// ======================================================================
// 每個引擎命令都有線上的歸屬 (design D6). Run with the `coverage` filter.
// ======================================================================

use speclink_core::command::Command;
use speclink_core::config::WorkflowConfigEdit;
use speclink_host::http::{routes, wire_coverage, WireCoverage};

/// One command of every kind. The match below has no catch-all arm, so a
/// new kind does not compile until it is listed here too.
fn every_command() -> Vec<Command> {
    fn listed(cmd: &Command) {
        match cmd {
            Command::List { .. }
            | Command::Show { .. }
            | Command::Status { .. }
            | Command::Instructions { .. }
            | Command::Validate { .. }
            | Command::Analyze { .. }
            | Command::Trace { .. }
            | Command::Plan { .. }
            | Command::PlanStrict
            | Command::ArtifactCat { .. }
            | Command::EvidenceShow { .. }
            | Command::LanguageShow
            | Command::DiscussList { .. }
            | Command::DiscussShow { .. }
            | Command::DiscussSearch { .. }
            | Command::NewChange { .. }
            | Command::NewArtifact { .. }
            | Command::TaskDone { .. }
            | Command::TaskUndone { .. }
            | Command::TaskMove { .. }
            | Command::Claim { .. }
            | Command::InProgressAdd { .. }
            | Command::InProgressRemove { .. }
            | Command::ChangeDepends { .. }
            | Command::ChangeSchema { .. }
            | Command::ChangeRank { .. }
            | Command::ChangeLanded { .. }
            | Command::Archive { .. }
            | Command::Discard { .. }
            | Command::DiscussNew { .. }
            | Command::DiscussContext { .. }
            | Command::DiscussAddRound { .. }
            | Command::DiscussConclude { .. }
            | Command::DiscussPromote { .. }
            | Command::DiscussLink { .. }
            | Command::DiscussSeal { .. }
            | Command::DiscussArchive { .. }
            | Command::DiscussDiscard { .. }
            | Command::ReviewAddRound { .. }
            | Command::ReviewShow { .. }
            | Command::ReviewStamp { .. }
            | Command::ReviewDiscard { .. }
            | Command::VerifyAddRound { .. }
            | Command::VerifyShow { .. }
            | Command::VerifyStamp { .. }
            | Command::VerifyDiscard { .. }
            | Command::WorkflowConfigShow
            | Command::WorkflowConfigEdit { .. }
            | Command::WorkflowConfigReplace { .. } => {}
        }
    }
    let s = |text: &str| text.to_string();
    let list = |specs| Command::List { sort: s("name"), specs, changes: false, worktrees: Default::default() };
    let commands = vec![
        list(false),
        list(true),
        Command::Show { item: Some(s("demo")), item_type: None },
        Command::Status { change: Some(s("demo")), schema: None },
        Command::Instructions { artifact: Some(s("apply")), change: Some(s("demo")), schema: None },
        Command::Validate { item: Some(s("demo")), all: false, changes: false, specs: false, strict: false },
        Command::Analyze { change: Some(s("demo")) },
        Command::Trace { capability: s("auth") },
        Command::Plan { ranks: None },
        Command::PlanStrict,
        Command::ArtifactCat { artifact: s("proposal"), change: Some(s("demo")) },
        Command::EvidenceShow { change: s("demo") },
        Command::LanguageShow,
        Command::DiscussList { archived: false },
        Command::DiscussShow { slug: s("talk") },
        Command::DiscussSearch { terms: vec![s("auth")] },
        Command::NewChange { name: s("demo"), description: None, schema: None, agent: None, from_discussion: None, last: false },
        Command::NewArtifact {
            kind: s("proposal"),
            capability: None,
            change: Some(s("demo")),
            content: Some(s("## Why\n")),
            force: true,
            new_capability: true,
        },
        Command::TaskDone { task_id: s("1"), change: Some(s("demo")), touched_files: None, head_commit: None },
        Command::TaskUndone { task_id: s("1"), change: Some(s("demo")) },
        Command::TaskMove { change: s("demo"), from: 1, to: 2, before: None },
        Command::Claim { name: s("demo") },
        Command::InProgressAdd { name: s("demo"), branch: None },
        Command::InProgressRemove { name: s("demo") },
        Command::ChangeDepends { name: s("demo"), on: vec![s("base")], remove: false },
        Command::ChangeSchema { name: s("demo"), schema: s("spec-driven") },
        Command::ChangeRank {
            name: s("demo"),
            anchor: s("base"),
            position: speclink_core::plan::Side::parse("before").unwrap(),
            force: false,
        },
        Command::ChangeLanded { name: s("demo"), commit: s("0123456789ab"), branch: None },
        Command::Archive {
            change: Some(s("demo")),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
        Command::Discard { change: s("demo"), force: false },
        Command::DiscussNew { topic: s("Talk"), slug: None, kind: None },
        Command::DiscussContext { slug: s("talk"), content: s("context") },
        Command::DiscussAddRound { slug: s("talk"), mode: s("round"), content: s("text") },
        Command::DiscussConclude { slug: s("talk"), content: s("done"), hold: false },
        Command::DiscussPromote { slug: s("talk"), name: Some(s("demo")), last: false },
        Command::DiscussLink { slug: s("talk"), change: s("demo") },
        Command::DiscussSeal { slug: s("talk"), change: s("demo"), last: false },
        Command::DiscussArchive { slug: s("talk") },
        Command::DiscussDiscard { slug: s("talk"), force: false },
        Command::ReviewAddRound { change: s("demo"), content: s("round") },
        Command::ReviewShow { change: s("demo") },
        Command::ReviewStamp { change: s("demo"), accept: false, tool: None, head_commit: None, scope: Vec::new(), missing: Vec::new() },
        Command::ReviewDiscard { change: s("demo") },
        Command::VerifyAddRound { change: s("demo"), content: s("round") },
        Command::VerifyShow { change: s("demo") },
        Command::VerifyStamp { change: s("demo"), accept: false, tool: None, head_commit: None, scope: Vec::new(), missing: Vec::new() },
        Command::VerifyDiscard { change: s("demo") },
        Command::WorkflowConfigShow,
        Command::WorkflowConfigEdit {
            edit: WorkflowConfigEdit::Policy { key: s("tdd"), value: s("true") },
            dry_run: false,
        },
        Command::WorkflowConfigReplace { content: s(CONFIG) },
    ];
    commands.iter().for_each(listed);
    commands
}

/// A request to `pattern` that gets as far as the route's handler: the
/// params filled in, a body of the route's shape, and the headers it needs.
fn sample_request(method: Method, pattern: &str) -> HttpRequest {
    let path = pattern
        .replace("{name}", "demo")
        .replace("{task_id}", "1")
        .replace("{slug}", "talk")
        .replace("{capability}", "auth")
        .replace("{dated_name}", "2026-01-01-demo")
        .replace("{*artifact}", "proposal");
    let body = match (method, pattern) {
        (Method::Post, "/changes") => json!({ "name": "demo" }),
        (Method::Post, "/changes/{name}/tasks/move") => json!({ "from": 1, "to": 2 }),
        (Method::Put, "/changes/{name}/artifacts/{*artifact}") => json!({ "content": "## Why\n" }),
        (Method::Post, "/changes/{name}/review/rounds" | "/changes/{name}/verify/rounds") => json!({ "content": "x" }),
        (Method::Post, "/changes/{name}/review/stamp" | "/changes/{name}/verify/stamp") => json!({ "scope": [] }),
        (Method::Post, "/changes/{name}/depends") => json!({ "on": ["base"] }),
        (Method::Post, "/changes/{name}/schema") => json!({ "schema": "spec-driven" }),
        (Method::Post, "/changes/{name}/rank") => json!({ "anchor": "base", "position": "before" }),
        (Method::Post, "/changes/{name}/landed") => json!({ "commit": "0123456789ab" }),
        (Method::Post, "/discussions") => json!({ "topic": "Talk" }),
        (Method::Post, "/discussions/{slug}/link" | "/discussions/{slug}/seal") => json!({ "change": "demo" }),
        (Method::Put, "/discussions/{slug}/context") => json!({ "content": "x" }),
        (Method::Post, "/discussions/{slug}/rounds") => json!({ "mode": "round", "content": "x" }),
        (Method::Post, "/discussions/{slug}/conclude") => json!({ "content": "x" }),
        (Method::Post, "/discussions/{slug}/promote") => json!({}),
        (Method::Put, "/config") => json!({ "content": "schema: spec-driven\ntdd: true\n", "expectedRevision": 0 }),
        (Method::Put, "/board-order") => json!({ "content": "{}" }),
        (Method::Post, _) => json!({}),
        _ => Value::Null,
    };
    let mut req = request(method, &format!("{API}/projects/alpha{path}"));
    if !body.is_null() {
        req.body = serde_json::to_vec(&body).unwrap();
    }
    req.header("if-match", "0")
}

/// The command names the hook was asked about for one request to `route`.
fn asked_by(method: Method, pattern: &str) -> Vec<String> {
    asked_with(sample_request(method, pattern))
}

/// The command names the hook was asked about for `req`.
fn asked_with(req: HttpRequest) -> Vec<String> {
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, |_| Decision::Deny { reason: "watching".into() }));
    answer(&door, &req, Some(&alice()));
    let asked = seen.asked.lock().unwrap().iter().map(|(command, _)| command.clone()).collect();
    asked
}

/// The sample request that runs `cmd` on its route: the route's sample, with
/// the query that picks the command where one route runs two.
fn request_for(cmd: &Command, method: Method, pattern: &str) -> HttpRequest {
    let mut req = sample_request(method, pattern);
    if matches!(cmd, Command::PlanStrict) {
        req.query = Some("strictOverlap=true".to_string());
    }
    req
}

#[test]
fn coverage_answers_every_command_with_a_route_or_a_reason() {
    for cmd in every_command() {
        match wire_coverage(&cmd) {
            WireCoverage::Route { .. } => {}
            WireCoverage::NotOnWire { reason } => assert!(!reason.trim().is_empty(), "{}: an empty reason", cmd.name()),
        }
    }
}

#[test]
fn coverage_routes_are_in_the_table_and_run_their_command() {
    let table: Vec<(Method, &str)> = routes().collect();
    for cmd in every_command() {
        let WireCoverage::Route { method, pattern } = wire_coverage(&cmd) else {
            continue;
        };
        assert!(table.contains(&(method, pattern)), "{}: {} {pattern} is not a route", cmd.name(), method.as_str());
        let asked = asked_with(request_for(&cmd, method, pattern));
        assert_eq!(asked, [cmd.name()], "{} {pattern} runs {}", method.as_str(), cmd.name());
    }
}

#[test]
fn coverage_points_at_every_route_that_runs_a_command() {
    let commands = every_command();
    let names: Vec<&str> = commands.iter().map(Command::name).collect();
    let pointed: Vec<(Method, &str)> = commands
        .iter()
        .filter_map(|cmd| match wire_coverage(cmd) {
            WireCoverage::Route { method, pattern } => Some((method, pattern)),
            WireCoverage::NotOnWire { .. } => None,
        })
        .collect();
    let mut running = 0;
    for (method, pattern) in routes() {
        let asked = asked_by(method, pattern);
        if asked.iter().any(|name| names.contains(&name.as_str())) {
            running += 1;
            assert!(pointed.contains(&(method, pattern)), "no command points at {} {pattern}", method.as_str());
        }
    }
    assert!(running >= 40, "the sample requests reach the commands: {running}");
}

#[test]
fn coverage_puts_create_change_on_the_post_of_changes() {
    let create = Command::NewChange {
        name: "demo".into(),
        description: None,
        schema: None,
        agent: None,
        from_discussion: None,
        last: false,
    };
    assert_eq!(wire_coverage(&create), WireCoverage::Route { method: Method::Post, pattern: "/changes" });
}

// --- 顯示用名稱只用在入口回給用戶端的名稱 ---

#[test]
fn a_shown_name_is_what_the_door_shows_and_the_display_name_is_what_it_stamps() {
    let door = door(one_repo());
    let bob = Identity::new("u_7", "Bob <bob@example.com>").unwrap().shown_as("Bob").unwrap();

    let handshake = json(&answer(&door, &get(&alpha("binding")), Some(&bob)));
    assert_eq!(handshake["actor"], json!({ "id": "u_7", "name": "Bob" }));
    let whoami = json(&answer(&door, &get(&alpha("whoami")), Some(&bob)));
    assert_eq!(whoami["user"], json!({ "name": "Bob", "handle": "u_7" }));
    let root = json(&answer(&door, &get(&format!("{API}/whoami")), Some(&bob)));
    assert_eq!(root["user"], json!({ "name": "Bob", "handle": "u_7" }));

    answer(&door, &send(Method::Post, &alpha("changes"), json!({ "name": "demo" })), Some(&bob));
    let meta = content(&door, "s1", "changes/demo/.meta.yaml").expect("the change's meta");
    assert!(meta.contains("created_by: Bob <bob@example.com>"), "{meta}");
    let commits = door.store.calls().commits;
    assert_eq!(commits[0].actor().display_name(), "Bob <bob@example.com>");
}

#[test]
fn a_blank_shown_name_is_refused() {
    for blank in ["", "  "] {
        let err = Identity::new("u_7", "Bob").unwrap().shown_as(blank).unwrap_err();
        assert_eq!(err.field(), "shown name");
    }
}

// ======================================================================
// 稽核 (task 6.1): the sharp edges of the door, route by route.
// ======================================================================

use speclink_store_port::{Commit, DocKey, Document, KeyPrefix, Listing, PortError, StorePort, Version as PortVersion};

#[test]
fn audit_without_an_identity_every_route_is_401_and_nothing_is_read() {
    let asked = Arc::new(AtomicUsize::new(0));
    let counted = asked.clone();
    let options = HostKitOptions {
        scopes: Some(Box::new(move |_: &Identity| {
            counted.fetch_add(1, Ordering::SeqCst);
            Ok(vec![project("alpha", vec![repo("web", "s1")])])
        })),
        ..Default::default()
    };
    let door = door(options);
    for (method, pattern) in routes() {
        let response = answer(&door, &sample_request(method, pattern), None);
        assert_eq!(response.status, 401, "{} {pattern}", method.as_str());
    }
    assert!(door.store.calls().is_empty(), "the store was not called");
    assert_eq!(asked.load(Ordering::SeqCst), 0, "the scope function was not asked");
}

#[test]
fn audit_a_request_cannot_name_a_scope_itself() {
    let door = door(two_repos());
    // A scope id where a repo key belongs is no repo of the project.
    for named in ["s1", "s2", "alpha/web"] {
        let (status, _, _) = error(&answer(&door, &get(&alpha("changes")).header("x-speclink-repo", named), Some(&alice())));
        assert_eq!(status, 404, "{named}");
    }
    assert!(door.store.calls().is_empty(), "no scope was reached");
}

#[test]
fn audit_names_from_the_url_reach_no_other_kind_of_document() {
    let door = door(one_repo());
    with_demo(&door);
    seed(&door, "s1", &[("discussions/secret.md", "# SECRET-TALK\n"), ("specs/auth/spec.md", "# Auth\n")]);
    let before = door.store.inner.list(&scope_of("s1"), &KeyPrefix::new("").unwrap()).unwrap();

    let crafted = [
        "changes/..%2Fdiscussions%2Fsecret.md/evidence",
        "changes/..%2F..%2Fconfig.yaml/evidence",
        "changes/demo/artifacts/..%2F..%2Fdiscussions%2Fsecret.md",
        "changes/demo/artifacts/specs%2F..%2F..%2F..%2Fdiscussions%2Fsecret",
        "changes/demo/artifacts/specs/..%2F..%2F..%2Fdiscussions%2Fsecret",
        "changes/%2E%2E/artifacts/proposal",
        "specs/..%2F..%2Fdiscussions%2Fsecret.md/document",
        "specs/auth%2F..%2F..%2Fdiscussions%2Fsecret/document",
        "archived/..%2F..%2Fdiscussions/artifacts/secret.md",
        "archived/x/artifacts/..%2F..%2F..%2Fdiscussions%2Fsecret.md",
        "archived/..%2Fdiscussions/capabilities",
        "changes/..%2Fdiscussions%2Fsecret.md",
    ];
    for tail in crafted {
        let response = answer(&door, &get(&alpha(tail)), Some(&alice()));
        let body = String::from_utf8_lossy(&response.body);
        assert!(!body.contains("SECRET-TALK"), "GET {tail} read the discussion: {body}");
    }
    let writes = [
        send(Method::Put, &alpha("changes/..%2Fdiscussions/artifacts/proposal"), json!({ "content": "## Why\n" })),
        send(Method::Put, &alpha("changes/demo/artifacts/specs%2F..%2F..%2Fconfig"), json!({ "content": "x" })),
        send(Method::Put, &alpha("changes/demo/artifacts/..%2F..%2Fconfig.yaml"), json!({ "content": "x" })),
        send(Method::Post, &alpha("changes/..%2F..%2Fconfig.yaml/tasks/1/done"), json!({})),
        send(Method::Delete, &alpha("discussions/..%2Fsecret"), json!(null)),
        send(Method::Delete, &alpha("changes/..%2Fdiscussions%2Fsecret.md"), json!(null)),
    ];
    for req in writes {
        let req = req.header("if-match", "0");
        let response = answer(&door, &req, Some(&alice()));
        assert!(response.status >= 400, "{} {} wrote: {}", req.method.as_str(), req.path, response.status);
    }
    let after = door.store.inner.list(&scope_of("s1"), &KeyPrefix::new("").unwrap()).unwrap();
    assert_eq!(after.entries, before.entries, "no document was written or removed");
}

#[test]
fn audit_every_route_that_writes_needs_can_write_and_no_read_commits() {
    let door = door(switches(false, false));
    with_demo(&door);
    for (method, pattern) in routes() {
        let response = answer(&door, &sample_request(method, pattern), Some(&alice()));
        let writes = !matches!(method, Method::Get | Method::Head) && !matches!(pattern, "/context" | "/skills");
        if writes {
            assert_eq!(response.status, 403, "{} {pattern}", method.as_str());
        } else {
            assert_ne!(response.status, 403, "{} {pattern} only reads", method.as_str());
        }
    }
    assert!(door.store.calls().commits.is_empty(), "no route committed: {:?}", door.store.calls().commits);
}

#[test]
fn audit_a_refused_hook_leaves_no_commit_on_any_route() {
    let door = door_with(one_repo(), recording(&Arc::new(Seen::default()), |_| Decision::Deny { reason: "no".into() }));
    with_demo(&door);
    for (method, pattern) in routes() {
        answer(&door, &sample_request(method, pattern), Some(&alice()));
    }
    assert!(door.store.calls().commits.is_empty(), "nothing was committed: {:?}", door.store.calls().commits);
}

#[test]
fn audit_the_door_reads_no_authorization_header() {
    let door = door(one_repo());
    let with_token = get(&alpha("changes")).header("authorization", "Bearer spk_pat_anything");
    assert_eq!(answer(&door, &with_token, None).status, 401, "a token is not an identity");
    let garbage = get(&alpha("changes")).header("authorization", "garbage");
    assert_eq!(answer(&door, &garbage, Some(&alice())).status, 200, "the host's identity decides");
}

/// A store whose every call fails the way a host's database driver would
/// report it.
struct Failing(PortError);

impl StorePort for Failing {
    fn list(&self, _: &ScopeId, _: &KeyPrefix) -> Result<Listing, PortError> {
        Err(self.0.clone())
    }
    fn read(&self, _: &ScopeId, _: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        Err(self.0.clone())
    }
    fn commit(&self, _: &ScopeId, _: &Commit) -> Result<PortVersion, PortError> {
        Err(self.0.clone())
    }
}

#[test]
fn audit_a_store_failure_names_no_host_detail() {
    let detail = "connect ECONNREFUSED 10.0.0.7:5432 (/var/lib/host/db.sock)";
    for (failure, status) in [
        (PortError::Backend { message: detail.to_string() }, 500),
        (PortError::Unavailable { message: detail.to_string() }, 503),
    ] {
        let kit = HostKit::with_options(Arc::new(Failing(failure)), Hooks::default(), one_repo()).unwrap();
        for path in [alpha("changes"), alpha("config"), alpha("sync-state")] {
            let response = kit.handle(&get(&path), Some(&alice())).expect("the door's path");
            let (got, _, message) = error(&response);
            assert_eq!(got, status, "{path}");
            assert!(!message.contains("10.0.0.7") && !message.contains("/var/lib"), "{path}: {message}");
        }
    }
}

// ======================================================================
// 品質關卡第 1 輪的修正：競態、版本與掛鉤的次數。
// ======================================================================

/// A memory store where another writer lands just before the `nth` read
/// that names `key` (every read when `key` is empty).
struct Meddled {
    inner: MemoryStorePort,
    key: &'static str,
    reads_left: AtomicUsize,
    meddle: Mutex<Option<Box<dyn FnOnce(&MemoryStorePort) + Send>>>,
}

impl Meddled {
    fn new(key: &'static str, nth: usize, meddle: impl FnOnce(&MemoryStorePort) + Send + 'static) -> Meddled {
        Meddled {
            inner: MemoryStorePort::new(),
            key,
            reads_left: AtomicUsize::new(nth),
            meddle: Mutex::new(Some(Box::new(meddle))),
        }
    }
}

impl StorePort for Meddled {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.inner.list(scope, prefix)
    }
    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        if (self.key.is_empty() || keys.iter().any(|k| k.as_str() == self.key))
            && self.reads_left.fetch_sub(1, Ordering::SeqCst) == 1
        {
            if let Some(meddle) = self.meddle.lock().unwrap().take() {
                meddle(&self.inner);
            }
        }
        self.inner.read(scope, keys)
    }
    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<PortVersion, PortError> {
        self.inner.commit(scope, commit)
    }
}

fn version_of(port: &MemoryStorePort, scope: &str) -> u64 {
    port.list(&scope_of(scope), &KeyPrefix::new("").unwrap()).unwrap().version.get()
}

#[test]
fn a_named_repo_decides_the_scope_a_write_lands_in() {
    let door = door(two_repos());

    let create = send(Method::Post, &alpha("changes"), json!({ "name": "demo" })).header("x-speclink-repo", "api");
    assert_eq!(answer(&door, &create, Some(&alice())).status, 200);

    assert!(content(&door, "s2", "changes/demo/.meta.yaml").is_some(), "demo is in the scope of api");
    assert_eq!(content(&door, "s1", "changes/demo/.meta.yaml"), None, "and not in the scope of web");
}

#[test]
fn a_stale_revision_through_the_door_is_409_revision_conflict() {
    let door = door(one_repo());
    with_demo(&door);
    let revision = json(&answer(&door, &get(&alpha("config")), Some(&alice())))["revision"].as_u64().unwrap();
    answer(&door, &send(Method::Post, &alpha("changes"), json!({ "name": "other" })), Some(&alice()));

    let stale = send(
        Method::Put,
        &alpha("config"),
        json!({ "content": "schema: spec-driven\ntdd: true\n", "expectedRevision": revision }),
    );
    let (status, reason, message) = error(&answer(&door, &stale, Some(&alice())));

    assert_eq!((status, reason.as_str()), (409, "revision_conflict"));
    assert!(message.contains(&format!("expected scope revision {revision}")), "{message}");
    assert_eq!(content(&door, "s1", "config.yaml").as_deref(), Some(CONFIG), "the config is unchanged");
}

#[test]
fn a_stale_config_write_still_asks_the_hook_exactly_once() {
    for refuse in [false, true] {
        let seen = Arc::new(Seen::default());
        let hooks = recording(&seen, move |_| match refuse {
            true => Decision::Deny { reason: "frozen".into() },
            false => Decision::Allow,
        });
        let door = door_with(one_repo(), hooks);
        with_demo(&door);
        let stale = send(
            Method::Put,
            &alpha("config"),
            json!({ "content": "schema: spec-driven\ntdd: true\n", "expectedRevision": 0 }),
        );

        let (status, reason, _) = error(&answer(&door, &stale, Some(&alice())));

        let asked: Vec<String> = seen.asked.lock().unwrap().iter().map(|(command, _)| command.clone()).collect();
        assert_eq!(asked, ["workflow-config-replace"], "asked once (refuse: {refuse})");
        // A refusal wins over the stale revision.
        let expected = if refuse { "refused" } else { "revision_conflict" };
        assert_eq!((status, reason.as_str()), (409, expected));
    }
}

#[test]
fn an_artifact_write_never_lands_over_a_version_the_client_did_not_see() {
    const KEY: &str = "changes/demo/proposal.md";
    // The first read of the artifact is the If-Match check; the second is
    // the command's own. Another writer lands in between.
    let store = Arc::new(Meddled::new(KEY, 2, |port| {
        common::write_in(port, &scope_of("s1"), &[(KEY, "## Why\n\nthe other writer's text\n")]);
    }));
    common::write_in(
        &store.inner,
        &scope_of("s1"),
        &[("changes/demo/.meta.yaml", META), (KEY, "## Why\n\nthe text the client read\n")],
    );
    let kit = HostKit::with_options(store.clone(), Hooks::default(), one_repo()).unwrap();
    use speclink_store_port::StorePort as _;
    // The version the client read, taken beside the door so the door's own
    // reads start with the write.
    let seen = store.inner.read(&scope_of("s1"), &[DocKey::new(KEY).unwrap()]).unwrap().remove(0).unwrap().version;

    let put = send(Method::Put, &alpha("changes/demo/artifacts/proposal"), json!({ "content": "## Why\n\nmine\n" }))
        .header("if-match", &seen.get().to_string());
    let response = kit.handle(&put, Some(&alice())).unwrap();

    let (status, reason, _) = error(&response);
    assert_eq!((status, reason.as_str()), (409, "revision_conflict"));
    let kept = store.inner.read(&scope_of("s1"), &[DocKey::new(KEY).unwrap()]).unwrap().remove(0).unwrap();
    assert_eq!(kept.content, "## Why\n\nthe other writer's text\n", "the other writer's text is not overwritten");
}

#[test]
fn a_create_only_artifact_write_does_not_replace_a_document_that_just_appeared() {
    const KEY: &str = "changes/demo/design.md";
    // The If-Match check reads nothing there; the document appears before
    // the command reads it.
    let store = Arc::new(Meddled::new(KEY, 2, |port| {
        common::write_in(port, &scope_of("s1"), &[(KEY, "## Context\n\nfirst writer\n")]);
    }));
    common::write_in(&store.inner, &scope_of("s1"), &[("changes/demo/.meta.yaml", META)]);
    let kit = HostKit::with_options(store.clone(), Hooks::default(), one_repo()).unwrap();

    let put = send(Method::Put, &alpha("changes/demo/artifacts/design"), json!({ "content": "## Context\n\nsecond\n" }))
        .header("if-match", "0");
    let (status, reason, _) = error(&kit.handle(&put, Some(&alice())).unwrap());

    assert_eq!((status, reason.as_str()), (409, "revision_conflict"));
    use speclink_store_port::StorePort as _;
    let kept = store.inner.read(&scope_of("s1"), &[DocKey::new(KEY).unwrap()]).unwrap().remove(0).unwrap();
    assert_eq!(kept.content, "## Context\n\nfirst writer\n");
}

#[test]
fn a_reads_etag_is_never_newer_than_what_it_read() {
    // Another writer lands when the command starts to read.
    let store = Arc::new(Meddled::new("", 1, |port| {
        common::write_in(port, &scope_of("s1"), &[("changes/late/.meta.yaml", META)]);
    }));
    common::write_in(&store.inner, &scope_of("s1"), &[("changes/demo/.meta.yaml", META), ("config.yaml", CONFIG)]);
    let before = version_of(&store.inner, "s1");
    let kit = HostKit::with_options(store.clone(), Hooks::default(), one_repo()).unwrap();

    let response = kit.handle(&get(&alpha("changes/demo")), Some(&alice())).unwrap();

    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert!(version_of(&store.inner, "s1") > before, "the other writer landed during the request");
    assert_eq!(etag_of(&response), before, "the ETag is the version taken before the reads");
}

#[test]
fn a_refused_discussion_write_gets_its_status_from_the_kind_of_refusal() {
    let door = door(one_repo());
    let me = alice();
    let create = |body: Value| answer(&door, &send(Method::Post, &alpha("discussions"), body), Some(&me));

    assert_eq!(create(json!({ "topic": "Scope talk", "slug": "talk" })).status, 200);
    let (status, reason, message) = error(&create(json!({ "topic": "Scope talk", "slug": "talk" })));
    assert_eq!((status, reason.as_str()), (409, "refused"), "it exists already: {message}");
    let (status, reason, message) = error(&create(json!({ "topic": "x", "slug": "Not Kebab" })));
    assert_eq!((status, reason.as_str()), (400, "invalid_argument"), "{message}");
    assert!(message.starts_with("invalid slug 'Not Kebab'"), "the Engine's text travels as it is: {message}");
    // A topic that quotes another refusal's words stays what it is.
    let (status, _, _) = error(&create(json!({ "topic": "it's ' not found\nkind: x" })));
    assert_eq!(status, 400);

    let missing = answer(&door, &request(Method::Delete, &alpha("discussions/nowhere")), Some(&me));
    let (status, reason, message) = error(&missing);
    assert_eq!((status, reason.as_str(), message.as_str()), (404, "not_found", "discussion 'nowhere' not found"));
    let unlinked = send(Method::Post, &alpha("discussions/talk/link"), json!({ "change": "ghost" }));
    let (status, reason, _) = error(&answer(&door, &unlinked, Some(&me)));
    assert_eq!((status, reason.as_str()), (404, "not_found"));
}

// ======================================================================
// Change remote-read-verbs: the reads that need no checkout.
// ======================================================================

// --- artifact cat 讀取已封存變更的文件（經儲存接口） ---

const ARCHIVED_PROPOSAL: &str = "## Why\n\nArchived.\n";

/// `demo` archived as 2026-11-12-demo with a proposal and a delta spec for auth.
fn with_archived_demo(door: &Door) {
    seed(
        door,
        "s1",
        &[
            ("archive/changes/2026-11-12-demo/.meta.yaml", META),
            ("archive/changes/2026-11-12-demo/proposal.md", ARCHIVED_PROPOSAL),
            ("archive/changes/2026-11-12-demo/specs/auth/spec.md", "## ADDED Requirements\n"),
            ("config.yaml", CONFIG),
        ],
    );
}

#[test]
fn an_artifact_of_an_archived_change_reads_by_either_name() {
    let door = door(one_repo());
    with_archived_demo(&door);
    let me = alice();
    for name in ["demo", "2026-11-12-demo"] {
        let proposal = answer(&door, &get(&alpha(&format!("changes/{name}/artifacts/proposal"))), Some(&me));
        assert_eq!(proposal.status, 200, "{name}: {:?}", String::from_utf8_lossy(&proposal.body));
        let body = json(&proposal);
        assert_eq!(body["content"], ARCHIVED_PROPOSAL, "{name}");
        assert_eq!(body["artifact"], "proposal", "{name}");
        // The version is the archived document's own.
        assert!(body["version"].as_u64().unwrap() > 0, "{name}: {body}");
        let spec = answer(&door, &get(&alpha(&format!("changes/{name}/artifacts/specs/auth"))), Some(&me));
        assert_eq!(json(&spec)["content"], "## ADDED Requirements\n", "{name}");
    }
    assert!(door.store.calls().commits.is_empty(), "a read commits nothing");
}

#[test]
fn an_archived_artifacts_version_is_that_documents_own() {
    let door = door(one_repo());
    with_archived_demo(&door);
    // A later write to another document moves the scope, not this document.
    let proposal = json(&answer(&door, &get(&alpha("changes/demo/artifacts/proposal")), Some(&alice())));
    seed(&door, "s1", &[("archive/changes/2026-11-12-demo/design.md", "## Context\n")]);
    let again = json(&answer(&door, &get(&alpha("changes/demo/artifacts/proposal")), Some(&alice())));
    assert_eq!(again["version"], proposal["version"]);
    let design = json(&answer(&door, &get(&alpha("changes/demo/artifacts/design")), Some(&alice())));
    assert!(design["version"].as_u64().unwrap() > proposal["version"].as_u64().unwrap(), "{design} vs {proposal}");
    // The read asks the store for the archive's names once, for the command.
    door.store.take();
    answer(&door, &get(&alpha("changes/demo/artifacts/proposal")), Some(&alice()));
    let archive_lists = door.store.calls().lists.iter().filter(|prefix| prefix.starts_with("archive/changes")).count();
    assert_eq!(archive_lists, 1, "{:?}", door.store.calls().lists);
}

#[test]
fn an_artifact_of_an_active_change_wins_over_an_archived_one_of_its_name() {
    let door = door(one_repo());
    with_archived_demo(&door);
    seed(&door, "s1", &[("changes/demo/.meta.yaml", META), ("changes/demo/proposal.md", "## Why\n\nActive.\n")]);
    let me = alice();
    let proposal = answer(&door, &get(&alpha("changes/demo/artifacts/proposal")), Some(&me));
    assert_eq!(json(&proposal)["content"], "## Why\n\nActive.\n");
    // The active change has no delta spec: the archive is not a second chance.
    let spec = answer(&door, &get(&alpha("changes/demo/artifacts/specs/auth")), Some(&me));
    assert_eq!(
        error(&spec),
        (404, "not_found".to_string(), "artifact 'specs/auth' not found for change 'demo'".to_string())
    );
}

#[test]
fn an_artifact_of_a_name_in_neither_place_keeps_its_message() {
    let door = door(one_repo());
    with_archived_demo(&door);
    let me = alice();
    let ghost = answer(&door, &get(&alpha("changes/ghost/artifacts/proposal")), Some(&me));
    assert_eq!(error(&ghost), (404, "not_found".to_string(), "Change 'ghost' not found.".to_string()));
    let design = answer(&door, &get(&alpha("changes/demo/artifacts/design")), Some(&me));
    assert_eq!(
        error(&design),
        (404, "not_found".to_string(), "artifact 'design' not found for change 'demo'".to_string())
    );
}

// --- 溯源與嚴格重疊的唯讀端點 ---

fn get_with(path: &str, query: &str) -> HttpRequest {
    let mut req = get(path);
    req.query = Some(query.to_string());
    req
}

const AUTH_SPEC: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: Session\n\nIt SHALL work.\n\n<!-- @trace\nsource: beta\nupdated: 2026-11-02\n-->\n";

const ARCHIVED_EVIDENCE: &str = r#"{"version":2,"change":"alpha","entries":[
    {"taskId":"tsk_a","taskDesc":"1.1 First","touchedFiles":["src/a.rs","src/b.rs"],"recordedAt":"2026-11-01T03:00:00Z"},
    {"taskId":"tsk_b","taskDesc":"1.2 Second","actor":"Alice","touchedFiles":["src/b.rs"],"recordedAt":"2026-11-01T04:00:00Z"}]}"#;

/// Capability auth, changed by two archived changes: alpha (from a
/// discussion, with evidence) then beta.
fn with_traced_auth(door: &Door) {
    seed(
        door,
        "s1",
        &[
            ("specs/auth/spec.md", AUTH_SPEC),
            (
                "archive/changes/2026-11-01-alpha/.meta.yaml",
                "schema: spec-driven\ncreated: 2026-10-30\nfrom_discussion: origin-talk\n",
            ),
            ("archive/changes/2026-11-01-alpha/specs/auth/spec.md", "## MODIFIED Requirements\n"),
            ("archive/changes/2026-11-01-alpha/.evidence.json", ARCHIVED_EVIDENCE),
            ("archive/changes/2026-11-02-beta/.meta.yaml", META),
            ("archive/changes/2026-11-02-beta/specs/auth/spec.md", "## MODIFIED Requirements\n"),
            ("config.yaml", CONFIG),
        ],
    );
}

#[test]
fn the_trace_route_answers_the_chain_oldest_first_with_an_etag() {
    let door = door(switches(false, false));
    with_traced_auth(&door);
    let response = answer(&door, &get(&alpha("trace/auth")), Some(&alice()));
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    assert_eq!(etag_of(&response), common::scope_version_in(&door.store.inner, &scope_of("s1")));
    let body = json(&response);
    assert_eq!(keys(&body), ["capability", "changes", "discussions", "requirements"]);
    assert_eq!(body["capability"], "auth");
    assert_eq!(body["requirements"], json!([{ "name": "Session", "source": "beta" }]));
    assert_eq!(
        body["changes"],
        json!([
            {
                "name": "alpha",
                "archivedDir": "2026-11-01-alpha",
                "fromDiscussion": "origin-talk",
                "evidence": [
                    { "taskId": "tsk_a", "files": ["src/a.rs", "src/b.rs"] },
                    { "taskId": "tsk_b", "files": ["src/b.rs"] },
                ],
            },
            { "name": "beta", "archivedDir": "2026-11-02-beta", "fromDiscussion": null, "evidence": null },
        ])
    );
    assert!(body["discussions"].is_array());
    assert!(door.store.calls().commits.is_empty(), "the trace commits nothing");
}

#[test]
fn the_trace_route_of_an_unknown_capability_is_404_with_the_engines_suggestion() {
    let door = door(one_repo());
    with_traced_auth(&door);
    // The Engine suggests names that share a word with the one asked for.
    let (status, reason, message) = error(&answer(&door, &get(&alpha("trace/auth-flow")), Some(&alice())));
    assert_eq!((status, reason.as_str()), (404, "not_found"));
    assert!(message.starts_with("Capability 'auth-flow' is not in the canonical specs.\n"), "{message}");
    assert!(message.contains("\n  - auth") || message.contains(" auth\n"), "the near name is suggested: {message}");
    assert!(message.ends_with("Re-run with one of these exact names."), "{message}");
}

fn modified(requirement: &str) -> String {
    format!("## MODIFIED Requirements\n\n### Requirement: {requirement}\n\nbody\n")
}

/// Changes a and b: each has a delta for desktop-app, on different requirements.
fn with_shared_capability(door: &Door, meta_a: &str, meta_b: &str) {
    seed(
        door,
        "s1",
        &[
            ("changes/a/.meta.yaml", meta_a),
            ("changes/a/specs/desktop-app/spec.md", &modified("Run rewind point")),
            ("changes/b/.meta.yaml", meta_b),
            ("changes/b/specs/desktop-app/spec.md", &modified("Conversation pin")),
            ("config.yaml", CONFIG),
        ],
    );
}

const META_A: &str = "schema: spec-driven\ncreated: 2026-09-01\n";
const META_B: &str = "schema: spec-driven\ncreated: 2026-09-02\n";

#[test]
fn a_strict_overlap_plan_puts_a_shared_capability_in_two_waves() {
    let door = door(switches(false, false));
    with_shared_capability(&door, META_A, META_B);
    let me = alice();
    let plain = answer(&door, &get(&alpha("plan")), Some(&me));
    assert_eq!(plain.status, 200);
    assert_eq!(json(&plain)["waves"], json!([{ "index": 1, "changes": ["a", "b"] }]));

    let strict = answer(&door, &get_with(&alpha("plan"), "strictOverlap=true"), Some(&me));
    assert_eq!(strict.status, 200, "{:?}", String::from_utf8_lossy(&strict.body));
    let body = json(&strict);
    assert_eq!(body["waves"], json!([{ "index": 1, "changes": ["a"] }, { "index": 2, "changes": ["b"] }]));
    assert_eq!(body["changes"][1]["blockedBy"], json!(["a"]));
    assert_eq!(body["changes"][1]["ready"], false);
    // The strict plan says that it is one; the rest of the shape is the same.
    assert_eq!(body["strictOverlap"], true);
    assert!(json(&plain).get("strictOverlap").is_none(), "the plain plan does not carry the key");
    let mut shape = keys(&json(&plain));
    shape.push("strictOverlap".to_string());
    shape.sort();
    assert_eq!(keys(&body), shape);
    assert_eq!(keys(&body["changes"][1]), keys(&json(&plain)["changes"][1]));
    assert!(header(&strict, "etag").is_some());

    // `false` and any other parameter leave the plan as it was.
    for query in ["strictOverlap=false", "strict=true", "foo=bar&x=1"] {
        let same = answer(&door, &get_with(&alpha("plan"), query), Some(&me));
        assert_eq!(json(&same), json(&plain), "{query}");
    }
    assert!(door.store.calls().commits.is_empty(), "a plan commits nothing");
}

#[test]
fn the_strict_plan_orders_by_the_same_board_order_as_the_plain_one() {
    // 「嚴格重疊的計算 SHALL 使用與一般 plan 相同的順序來源」：兩個互不相干的變更，
    // 依建立日期是 a 在前；看板順序把 b 排到前面。兩種計畫都要照看板順序。
    let door = door(one_repo());
    seed(
        &door,
        "s1",
        &[
            ("changes/a/.meta.yaml", META_A),
            ("changes/b/.meta.yaml", META_B),
            ("config.yaml", CONFIG),
        ],
    );
    let me = alice();
    let order = |query: Option<&str>| -> Vec<String> {
        let mut req = get(&alpha("plan"));
        req.query = query.map(str::to_string);
        let body = json(&answer(&door, &req, Some(&me)));
        body["changes"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect()
    };
    assert_eq!(order(None), ["a", "b"], "with no board order, the older change is first");
    assert_eq!(order(Some("strictOverlap=true")), ["a", "b"]);

    seed(&door, "s1", &[("board/order", r#"{"changes":{"b":"a0","a":"a1"}}"#)]);
    assert_eq!(order(None), ["b", "a"], "the board order ranks b first");
    assert_eq!(order(Some("strictOverlap=true")), ["b", "a"], "the strict plan reads the same ranks");
}

#[test]
fn a_strict_overlap_value_that_is_not_a_boolean_is_400() {
    let door = door(one_repo());
    with_shared_capability(&door, META_A, META_B);
    for query in ["strictOverlap=maybe", "strictOverlap=", "strictOverlap=1"] {
        let (status, reason, _) = error(&answer(&door, &get_with(&alpha("plan"), query), Some(&alice())));
        assert_eq!((status, reason.as_str()), (400, "invalid_argument"), "{query}");
    }
}

#[test]
fn a_dependency_cycle_refuses_the_strict_plan_like_the_plain_one() {
    let door = door(one_repo());
    with_shared_capability(&door, &format!("{META_A}depends_on: b\n"), &format!("{META_B}depends_on: a\n"));
    let me = alice();
    let plain = error(&answer(&door, &get(&alpha("plan")), Some(&me)));
    let strict = error(&answer(&door, &get_with(&alpha("plan"), "strictOverlap=true"), Some(&me)));
    assert_eq!((strict.0, strict.1.as_str()), (409, "refused"));
    assert!(strict.2.starts_with("dependency cycle: "), "{}", strict.2);
    assert_eq!(strict, plain);
}

// --- evidence 端點以引擎命令回答 ---

const DEMO_EVIDENCE: &str = r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_a","taskDesc":"1.1 First","actor":"Alice","touchedFiles":["src/a.rs"],"recordedAt":"2026-11-12T03:00:00Z"}]}"#;

#[test]
fn the_evidence_route_reads_an_active_change_without_naming_an_archive() {
    let door = door(switches(false, false));
    with_demo(&door);
    seed(&door, "s1", &[("changes/demo/.evidence.json", DEMO_EVIDENCE)]);
    let response = answer(&door, &get(&alpha("changes/demo/evidence")), Some(&alice()));
    assert_eq!(response.status, 200);
    assert!(header(&response, "etag").is_some());
    let body = json(&response);
    assert_eq!(keys(&body), ["change", "entries"], "no archivedAs for an active change: {body}");
    assert_eq!(body["change"], "demo");
    assert_eq!(
        body["entries"],
        json!([{
            "taskId": "tsk_a",
            "taskDesc": "1.1 First",
            "actor": "Alice",
            "touchedFiles": ["src/a.rs"],
            "recordedAt": "2026-11-12T03:00:00Z",
        }])
    );
    assert!(door.store.calls().commits.is_empty(), "reading evidence commits nothing");
}

#[test]
fn the_evidence_route_reads_back_what_task_done_recorded() {
    let door = door(one_repo());
    with_demo(&door);
    let me = alice();
    let done = send(Method::Post, &alpha("changes/demo/tasks/1/done"), json!({ "touchedFiles": ["src/app.rs"] }));
    assert_eq!(answer(&door, &done, Some(&me)).status, 200);
    let body = json(&answer(&door, &get(&alpha("changes/demo/evidence")), Some(&me)));
    assert_eq!(body["entries"].as_array().unwrap().len(), 1, "{body}");
    assert_eq!(body["entries"][0]["touchedFiles"], json!(["src/app.rs"]));
    assert!(body["entries"][0]["taskId"].is_string() && body["entries"][0]["recordedAt"].is_string());
}

#[test]
fn the_evidence_route_reads_an_archived_change_and_names_it() {
    let door = door(one_repo());
    seed(
        &door,
        "s1",
        &[
            ("archive/changes/2026-11-12-demo/.meta.yaml", META),
            ("archive/changes/2026-11-12-demo/.evidence.json", DEMO_EVIDENCE),
            ("config.yaml", CONFIG),
        ],
    );
    for name in ["demo", "2026-11-12-demo"] {
        let response = answer(&door, &get(&alpha(&format!("changes/{name}/evidence"))), Some(&alice()));
        assert_eq!(response.status, 200, "{name}");
        let body = json(&response);
        assert_eq!(keys(&body), ["archivedAs", "change", "entries"], "{name}");
        assert_eq!(body["archivedAs"], "2026-11-12-demo", "{name}");
        assert_eq!(body["change"], "demo", "the change's own name, whichever name was asked: {name}");
        assert_eq!(body["entries"][0]["taskId"], "tsk_a", "{name}");
    }
}

#[test]
fn a_change_with_no_evidence_is_an_empty_set_and_an_unknown_name_is_404() {
    let door = door(one_repo());
    with_demo(&door);
    let me = alice();
    let none = answer(&door, &get(&alpha("changes/demo/evidence")), Some(&me));
    assert_eq!(none.status, 200);
    assert_eq!(json(&none), json!({ "change": "demo", "entries": [] }));
    for name in ["ghost", "..", "archive"] {
        let missing = answer(&door, &get(&alpha(&format!("changes/{name}/evidence"))), Some(&me));
        assert_eq!(
            error(&missing),
            (404, "not_found".to_string(), format!("Change '{name}' not found.")),
            "{name}"
        );
    }
}

#[test]
fn a_broken_evidence_record_is_500_and_its_content_stays_out_of_the_message() {
    let door = door(one_repo());
    with_demo(&door);
    seed(&door, "s1", &[("changes/demo/.evidence.json", "SECRET-CONTENT not json")]);
    let (status, reason, message) = error(&answer(&door, &get(&alpha("changes/demo/evidence")), Some(&alice())));
    assert_eq!((status, reason.as_str()), (500, "internal"));
    assert!(message.starts_with("evidence record of 'demo' cannot be parsed: "), "{message}");
    assert!(!message.contains("SECRET-CONTENT"), "{message}");
}

// --- 變更清單的最後修改時間欄位 ---

/// Seed `docs` in scope s1 with a commit at `at_millis`.
fn seed_at(door: &Door, at_millis: u64, docs: &[(&str, &str)]) {
    use speclink_store_port::{Actor, Commit, DocKey, Expected, Op, StorePort, Timestamp};
    let ops = docs
        .iter()
        .map(|(key, content)| Op::Put {
            key: DocKey::new(*key).unwrap(),
            content: content.to_string(),
            expected: Expected::Absent,
        })
        .collect();
    let commit =
        Commit::new(Actor::new("seed", "seed").unwrap(), "seed", Timestamp::new(at_millis).unwrap(), ops, Vec::new())
            .unwrap();
    door.store.inner.commit(&scope_of("s1"), &commit).expect("seed commit");
    door.store.take();
}

const DATED_META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";

#[test]
fn a_listed_change_carries_created_and_the_stores_last_modified_time() {
    let door = door(switches(false, false));
    // 2026-11-12T03:00:00Z, then a later write to one document of the change.
    seed_at(&door, 1_794_452_400_000, &[("changes/demo/.meta.yaml", DATED_META), ("config.yaml", CONFIG)]);
    seed_at(&door, 1_794_456_000_999, &[("changes/demo/tasks.md", OPEN_TASKS)]);
    let body = json(&answer(&door, &get(&alpha("changes")), Some(&alice())));
    let demo = &body["changes"][0];
    assert_eq!(demo["name"], "demo");
    assert_eq!(demo["created"], "2026-11-01");
    assert_eq!(demo["modifiedAt"], "2026-11-12T04:00:00Z", "the newest document, in whole seconds");
    assert!(demo.get("modified_at").is_none());
}

#[test]
fn a_store_that_reports_no_time_gives_no_modified_at_key() {
    let door = door(one_repo());
    seed_at(&door, 0, &[("changes/demo/.meta.yaml", DATED_META), ("changes/demo/tasks.md", OPEN_TASKS)]);
    let body = json(&answer(&door, &get(&alpha("changes")), Some(&alice())));
    let demo = &body["changes"][0];
    assert_eq!(demo["created"], "2026-11-01");
    assert!(!keys(demo).contains(&"modifiedAt".to_string()), "{demo}");
}

// --- 線上歸屬表：三個讀取都有路由 ---

#[test]
fn coverage_puts_the_three_reads_on_their_routes() {
    let s = |text: &str| text.to_string();
    let route = |pattern| WireCoverage::Route { method: Method::Get, pattern };
    assert_eq!(wire_coverage(&Command::Trace { capability: s("auth") }), route("/trace/{capability}"));
    assert_eq!(wire_coverage(&Command::PlanStrict), route("/plan"));
    assert_eq!(wire_coverage(&Command::EvidenceShow { change: s("demo") }), route("/changes/{name}/evidence"));
}

// --- sharp-edges audit of the three reads ---

#[test]
fn audit_a_crafted_name_reaches_no_other_document_through_the_reads() {
    let door = door(one_repo());
    with_demo(&door);
    with_traced_auth(&door);
    seed(
        &door,
        "s1",
        &[
            ("discussions/secret.md", "# SECRET-TALK\n"),
            ("changes/demo/.evidence.json", DEMO_EVIDENCE),
            ("archive/changes/2026-11-01-alpha/proposal.md", "## Why\n\nSECRET-ARCHIVED\n"),
        ],
    );
    let me = alice();
    for tail in [
        // An archived change is not an active one under another name.
        "changes/archive%2F2026-11-01-alpha/evidence",
        "changes/archive%2F2026-11-01-alpha/artifacts/proposal",
        "changes/demo%2F..%2Fdemo/evidence",
        "changes/..%2Fdiscussions%2Fsecret.md/evidence",
        "changes/%2E%2E/evidence",
        "changes/./evidence",
        "trace/..%2Fdiscussions%2Fsecret",
        "trace/auth%2F..%2F..%2Fdiscussions%2Fsecret",
        "trace/..%2F..%2Fchanges%2Fdemo%2F.evidence.json",
    ] {
        let response = answer(&door, &get(&alpha(tail)), Some(&me));
        let body = String::from_utf8_lossy(&response.body);
        assert_eq!(response.status, 404, "GET {tail}: {body}");
        assert!(!body.contains("SECRET") && !body.contains("tsk_a"), "GET {tail} read a document: {body}");
    }
    assert!(door.store.calls().commits.is_empty(), "no read commits");
}

#[test]
fn an_archived_change_without_a_date_in_its_name_is_traced_and_read_like_any_other() {
    // design Open Question: an older archive can hold a change under its
    // bare name. The store view reads its documents by that name, so the
    // trace finds its evidence and `evidence show` reads it.
    let door = door(one_repo());
    with_traced_auth(&door);
    seed(
        &door,
        "s1",
        &[
            ("archive/changes/legacy/.meta.yaml", META),
            ("archive/changes/legacy/specs/auth/spec.md", "## MODIFIED Requirements\n"),
            ("archive/changes/legacy/.evidence.json", DEMO_EVIDENCE),
        ],
    );
    let me = alice();
    let trace = json(&answer(&door, &get(&alpha("trace/auth")), Some(&me)));
    let legacy = trace["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["archivedDir"] == "legacy")
        .unwrap_or_else(|| panic!("legacy is in the chain: {trace}"));
    assert_eq!(legacy["name"], "legacy");
    assert_eq!(legacy["evidence"], json!([{ "taskId": "tsk_a", "files": ["src/a.rs"] }]));

    let evidence = json(&answer(&door, &get(&alpha("changes/legacy/evidence")), Some(&me)));
    assert_eq!(evidence["archivedAs"], "legacy");
    assert_eq!(evidence["change"], "legacy");
    assert_eq!(evidence["entries"][0]["taskId"], "tsk_a");
}

#[test]
fn audit_the_reads_ask_the_before_hook_and_a_refusal_reads_nothing() {
    // A host that refuses a read gets its refusal, and no document is read.
    let seen = Arc::new(Seen::default());
    let door = door_with(one_repo(), recording(&seen, |_| Decision::Deny { reason: "not today".into() }));
    with_demo(&door);
    for (tail, query, command) in [
        ("trace/auth", None, "trace"),
        ("plan", Some("strictOverlap=true"), "plan-strict"),
        ("changes/demo/evidence", None, "evidence-show"),
    ] {
        let mut req = get(&alpha(tail));
        req.query = query.map(str::to_string);
        let (status, reason, message) = error(&answer(&door, &req, Some(&alice())));
        assert_eq!((status, reason.as_str(), message.as_str()), (409, "refused", "not today"), "{tail}");
        assert_eq!(seen.asked.lock().unwrap().last().unwrap().0, command, "{tail}");
    }
    assert!(door.store.calls().reads.is_empty(), "a refused read reads no document");
}


// ======================================================================
// 入口提供這個範圍的技能 (spec host-kit; design D2, D3)
// ======================================================================

use speclink_core::config::{CustomTool, Invocation};
use speclink_core::init::{managed_skills, ASSET_VERSION};
use speclink_core::skills::{RenderTarget, SpecLocation, Tool};
use speclink_protocol::skills::{SkillEntry, SkillsResponse};

const WORKTREE_ON: &str = "schema: spec-driven\nworktree: true\n";
const WORKTREE_SKILLS: [&str; 2] = ["apply-with-worktree", "worktree-merge"];

/// A door on project alpha whose scope s1 holds `config` as its workflow config.
fn door_with_config(options: HostKitOptions, config: Option<&str>) -> Door {
    let door = door(options);
    if let Some(config) = config {
        seed(&door, "s1", &[("config.yaml", config)]);
    }
    door
}

/// The door's answer to a skills request with `body`, which must be a 200.
fn skills_of(door: &Door, body: Value) -> SkillsResponse {
    let response = answer(door, &send(Method::Post, &alpha("skills"), body), Some(&alice()));
    assert_eq!(response.status, 200, "{:?}", String::from_utf8_lossy(&response.body));
    serde_json::from_slice(&response.body).expect("a skills response")
}

fn names(skills: &[SkillEntry]) -> Vec<&str> {
    skills.iter().map(|s| s.name.as_str()).collect()
}

/// What the Engine generates for `target` in a checkout without the spec
/// folder: (skill name, SKILL.md text), sorted by name.
fn engine_skills(target: RenderTarget, worktree_on: bool) -> Vec<(String, String)> {
    let mut skills: Vec<(String, String)> =
        managed_skills(target, worktree_on, speclink_core::workspace::DEFAULT_SPEC_DIR, SpecLocation::Elsewhere)
            .into_iter()
            .map(|(dir, content)| (dir.trim_start_matches("speclink-").to_string(), content))
            .collect();
    skills.sort();
    skills
}

fn sha256_of_lf(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(text.replace("\r\n", "\n").as_bytes()))
}

#[test]
fn skills_with_the_worktree_policy_on_have_the_two_worktree_skills() {
    let door = door_with_config(one_repo(), Some(WORKTREE_ON));
    let response = skills_of(&door, json!({ "targets": [{ "tool": "claude" }] }));
    let got = names(&response.targets[0].skills);
    for skill in WORKTREE_SKILLS {
        assert!(got.contains(&skill), "{skill}: {got:?}");
    }
}

#[test]
fn skills_without_a_workflow_config_have_every_skill_but_the_two_worktree_ones() {
    let door = door(one_repo());
    let response = skills_of(&door, json!({ "targets": [{ "tool": "claude" }] }));
    assert_eq!(response.targets.len(), 1);
    assert_eq!(response.targets[0].tool, "claude");
    let got = names(&response.targets[0].skills);
    let expected = engine_skills(RenderTarget::Builtin(Tool::Claude), false);
    assert_eq!(got, expected.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), "sorted by name");
    for skill in WORKTREE_SKILLS {
        assert!(!got.contains(&skill), "{skill}: {got:?}");
    }
    assert!(got.contains(&"apply") && got.contains(&"discuss"), "{got:?}");
}

#[test]
fn skills_keep_the_two_worktree_skills_when_the_workflow_config_cannot_be_parsed() {
    let door = door_with_config(one_repo(), Some("schema: [unclosed\n"));
    let got = skills_of(&door, json!({ "targets": [{ "tool": "claude" }] }));
    for skill in WORKTREE_SKILLS {
        assert!(names(&got.targets[0].skills).contains(&skill), "{skill}");
    }
}

#[test]
fn skills_follow_the_scopes_config_and_not_the_process_environment() {
    // The engine's own rule: generation never reads SPECLINK_WORKTREE.
    // Two scopes of one door answer by their own config.
    let door = door(two_repos());
    seed(&door, "s2", &[("config.yaml", WORKTREE_ON)]);
    let ask = |repo: &str| {
        let req = send(Method::Post, &alpha("skills"), json!({ "targets": [{ "tool": "claude" }] }))
            .header("x-speclink-repo", repo);
        let response: SkillsResponse = serde_json::from_slice(&answer(&door, &req, Some(&alice())).body).unwrap();
        response.targets[0].skills.iter().any(|s| s.name == "worktree-merge")
    };
    assert!(!ask("web"), "s1 has no config");
    assert!(ask("api"), "s2 turned the policy on");
}

#[test]
fn skills_a_host_lists_are_the_only_ones_given() {
    let options = HostKitOptions { skills: Some(vec!["discuss".into(), "propose".into()]), ..one_repo() };
    let door = door_with_config(options, Some(WORKTREE_ON));
    let got = skills_of(&door, json!({ "targets": [{ "tool": "claude" }] }));
    assert_eq!(names(&got.targets[0].skills), ["discuss", "propose"]);
}

#[test]
fn skills_a_host_lists_still_follow_the_tool_and_the_worktree_policy() {
    let listed = vec!["analyze".to_string(), "discuss".to_string(), "worktree-merge".to_string()];
    let door = door(HostKitOptions { skills: Some(listed), ..one_repo() });
    let got = skills_of(&door, json!({ "targets": [{ "tool": "claude" }, { "tool": "codex" }] }));
    assert_eq!(names(&got.targets[0].skills), ["analyze", "discuss"], "the policy is off");
    assert_eq!(names(&got.targets[1].skills), ["discuss"], "analyze is for Claude only");
}

#[test]
fn skills_for_claude_only_are_not_given_to_another_tool() {
    let door = door(one_repo());
    let got = skills_of(&door, json!({ "targets": [{ "tool": "codex" }, { "tool": "claude" }] }));
    assert_eq!(
        got.targets.iter().map(|t| t.tool.as_str()).collect::<Vec<_>>(),
        ["codex", "claude"],
        "the targets keep the request's order"
    );
    assert!(!names(&got.targets[0].skills).contains(&"analyze"));
    assert!(names(&got.targets[1].skills).contains(&"analyze"));
}

#[test]
fn skills_without_contents_carry_a_name_and_a_digest_only() {
    let door = door(one_repo());
    let response = answer(&door, &send(Method::Post, &alpha("skills"), json!({ "targets": [{ "tool": "claude" }] })), Some(&alice()));
    let body = json(&response);
    assert_eq!(body["version"], ASSET_VERSION);
    let skills = body["targets"][0]["skills"].as_array().expect("skills");
    assert!(!skills.is_empty());
    for skill in skills {
        assert_eq!(keys(skill), ["digest", "name"], "{skill}");
    }
    let expected = engine_skills(RenderTarget::Builtin(Tool::Claude), false);
    for (skill, (name, content)) in skills.iter().zip(&expected) {
        assert_eq!(skill["name"], name.as_str());
        assert_eq!(skill["digest"], sha256_of_lf(content), "{name}");
    }
}

#[test]
fn skills_with_contents_are_the_engines_render_for_a_checkout_without_the_spec_folder() {
    let door = door_with_config(one_repo(), Some(WORKTREE_ON));
    let got = skills_of(&door, json!({ "targets": [{ "tool": "claude" }], "contents": true }));
    assert_eq!(got.version, ASSET_VERSION);
    let declared = json(&answer(&door, &get(&alpha("binding")), Some(&alice())))["capabilities"]["skills"]["version"]
        .as_str()
        .expect("the handshake declares the version")
        .to_string();
    let expected = engine_skills(RenderTarget::Builtin(Tool::Claude), true);
    assert_eq!(got.targets[0].skills.len(), expected.len());
    for (skill, (name, content)) in got.targets[0].skills.iter().zip(&expected) {
        let text = skill.content.as_deref().unwrap_or_else(|| panic!("{name} has its text"));
        assert_eq!((skill.name.as_str(), text), (name.as_str(), content.as_str()));
        assert_eq!(skill.digest, sha256_of_lf(text), "{name}: the digest is of the text sent");
        assert!(text.contains(&format!("  version: \"{declared}\"\n")), "{name}: the frontmatter version");
    }
    let apply = got.targets[0].skills.iter().find(|s| s.name == "apply").unwrap().content.as_deref().unwrap();
    assert!(!apply.contains("speclink/changes/"), "apply names no path in the spec folder");
}

#[test]
fn skills_of_a_custom_tool_are_the_neutral_tool_call_render() {
    let door = door(one_repo());
    let target = json!({ "tool": "my-agent", "custom": { "invocation": "tool-call" } });
    let got = skills_of(&door, json!({ "targets": [target], "contents": true }));
    assert_eq!(got.targets[0].tool, "my-agent");
    let custom = CustomTool {
        name: "my-agent".into(),
        skills_dir: String::new(),
        instructions_file: None,
        invocation: Invocation::ToolCall,
    };
    let expected = engine_skills(RenderTarget::Custom(&custom), false);
    let texts: Vec<(&str, &str)> =
        got.targets[0].skills.iter().map(|s| (s.name.as_str(), s.content.as_deref().unwrap())).collect();
    assert_eq!(texts, expected.iter().map(|(n, c)| (n.as_str(), c.as_str())).collect::<Vec<_>>());
    assert!(!names(&got.targets[0].skills).contains(&"analyze"), "a custom tool is not Claude");

    let apply = texts.iter().find(|(name, _)| *name == "apply").unwrap().1;
    let after_frontmatter = apply.splitn(3, "---\n").nth(2).expect("a frontmatter").trim_start();
    assert!(
        after_frontmatter.starts_with("## Invocation\n\nThis harness executes speclink verbs by calling the speclink tool"),
        "the text opens with the tool-call note: {:?}",
        &after_frontmatter[..120.min(after_frontmatter.len())]
    );
    assert!(apply.contains("speclink-archive") && !apply.contains("/speclink-archive"), "another skill is named `speclink-<name>`");

    let cli = skills_of(&door, json!({ "targets": [{ "tool": "my-agent", "custom": { "invocation": "cli" } }], "contents": true }));
    let cli_apply = cli.targets[0].skills.iter().find(|s| s.name == "apply").unwrap().content.clone().unwrap();
    assert!(cli_apply.contains("executes speclink verbs as shell commands"), "the cli note");
}

#[test]
fn skills_are_given_to_a_caller_who_cannot_write_and_nothing_is_committed() {
    let door = door(switches(false, false));
    with_demo(&door);
    let got = skills_of(&door, json!({ "targets": [{ "tool": "claude" }], "contents": true }));
    assert!(!got.targets[0].skills.is_empty());
    assert!(door.store.calls().commits.is_empty(), "no commit");
}

#[test]
fn skills_are_the_same_for_every_caller_of_one_scope() {
    let door = door_with_config(one_repo(), Some(WORKTREE_ON));
    let body = json!({ "targets": [{ "tool": "claude" }, { "tool": "codex" }], "contents": true });
    let bob = Identity::new("u_7", "Bob").unwrap();
    let as_alice = answer(&door, &send(Method::Post, &alpha("skills"), body.clone()), Some(&alice()));
    let as_bob = answer(&door, &send(Method::Post, &alpha("skills"), body), Some(&bob));
    assert_eq!(as_alice.body, as_bob.body);
}

#[test]
fn skills_a_host_lists_must_be_ones_the_engine_has() {
    let store = Arc::new(common::Recorder::new(MemoryStorePort::new()));
    let options = HostKitOptions { skills: Some(vec!["discuss".into(), "no-such-skill".into()]), ..one_repo() };
    let Err(error) = HostKit::with_options(store, Hooks::default(), options) else {
        panic!("a kit with an unknown skill name is not built");
    };
    assert_eq!(error.names(), ["no-such-skill"]);
    let message = error.to_string();
    assert!(message.contains("no-such-skill"), "{message}");
    assert!(!message.contains("'discuss'"), "a known name is not listed as unknown: {message}");
}

#[test]
fn the_handshake_declares_the_skill_template_version() {
    let door = door(one_repo());
    let caps = json(&answer(&door, &get(&alpha("binding")), Some(&alice())))["capabilities"].clone();
    assert_eq!(caps["skills"], json!({ "version": ASSET_VERSION }), "{caps}");
}

#[test]
fn skills_requests_that_name_nothing_the_door_can_render_are_400() {
    let door = door(one_repo());
    for (body, names_it) in [
        (json!({ "targets": [] }), "targets"),
        (json!({ "targets": [{ "tool": "emacs" }] }), "emacs"),
        (json!({ "targets": [{ "tool": "claude" }, { "tool": "emacs" }] }), "emacs"),
        (json!({ "targets": [{ "tool": "my-agent", "custom": { "invocation": "telepathy" } }] }), "telepathy"),
        (json!({ "targets": [{ "tool": "claude", "custom": { "invocation": "cli" } }] }), "claude"),
        (json!({ "targets": [{ "tool": "Not A Name", "custom": { "invocation": "cli" } }] }), "Not A Name"),
        (json!({}), "targets"),
    ] {
        let (status, reason, message) =
            error(&answer(&door, &send(Method::Post, &alpha("skills"), body.clone()), Some(&alice())));
        assert_eq!((status, reason.as_str()), (400, "invalid_argument"), "{body}");
        assert!(message.contains(names_it), "{body}: {message}");
    }
    assert!(door.store.calls().commits.is_empty());
}

// --- sharp-edges audit of the skills route ---

#[test]
fn audit_a_skills_request_cannot_ask_for_more_targets_than_a_checkout_can_have() {
    // Every target is a render of the whole skill set: the number a caller can ask for is bounded.
    let door = door(one_repo());
    let target = |i: usize| json!({ "tool": format!("agent-{i}"), "custom": { "invocation": "cli" } });
    let at_the_limit: Vec<Value> = (0..32).map(target).collect();
    assert_eq!(skills_of(&door, json!({ "targets": at_the_limit })).targets.len(), 32);

    let over: Vec<Value> = (0..33).map(target).collect();
    let (status, reason, message) =
        error(&answer(&door, &send(Method::Post, &alpha("skills"), json!({ "targets": over })), Some(&alice())));
    assert_eq!((status, reason.as_str()), (400, "invalid_argument"));
    assert!(message.contains("32"), "{message}");
}

#[test]
fn audit_an_empty_skill_list_builds_no_kit() {
    // A kit that gives no skill would have every checkout that updates from it lose its
    // skills. A host that wants every skill leaves the list out.
    let store = Arc::new(common::Recorder::new(MemoryStorePort::new()));
    let options = HostKitOptions { skills: Some(Vec::new()), ..one_repo() };
    let Err(error) = HostKit::with_options(store, Hooks::default(), options) else {
        panic!("a kit with an empty skill list is not built");
    };
    assert!(error.names().is_empty());
    assert!(error.to_string().contains("empty"), "{error}");
}

#[test]
fn audit_the_skills_route_reads_the_callers_scope_only_and_one_document() {
    let door = door(two_repos());
    seed(&door, "s2", &[("config.yaml", WORKTREE_ON)]);
    let req = send(Method::Post, &alpha("skills"), json!({ "targets": [{ "tool": "claude" }], "contents": true }))
        .header("x-speclink-repo", "web");
    let response = answer(&door, &req, Some(&alice()));
    assert_eq!(response.status, 200);
    assert!(!String::from_utf8_lossy(&response.body).contains("speclink-worktree-merge\n"), "s2's policy is not s1's");
    let calls = door.store.calls();
    assert!(calls.lists.is_empty() && calls.commits.is_empty(), "{calls:?}");
    let read: Vec<&str> = calls.reads.iter().flatten().map(|(key, _)| key.as_str()).collect();
    assert_eq!(read, ["config.yaml"], "one read, of the workflow config");
}

#[test]
fn audit_a_project_that_is_not_the_callers_gives_no_skills() {
    let door = door(one_repo());
    let req = send(Method::Post, &format!("{API}/projects/beta/skills"), json!({ "targets": [{ "tool": "claude" }] }));
    assert_eq!(answer(&door, &req, Some(&alice())).status, 404);
    assert!(door.store.calls().is_empty(), "the store was not called");
}

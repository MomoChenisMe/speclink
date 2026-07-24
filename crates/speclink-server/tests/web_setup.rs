//! Browser JSON setup API（server-setup spec「setup 流程完成開箱四要素」, 設計決策
//! D2／D3／D8 第三階段）。
//!
//! `/api/speclink/v1/web/setup*` 是 bootstrap-token 門禁的 same-origin JSON 流程：
//! GET 唯讀回目前步驟與 Store 狀態；兩個提交節點分別建立第一位 Admin 與第一組
//! Project／Repo。最後（registry）節點在既有 setup 交易邊界內耗用 token、記錄 audit
//! 並建立第一位 Admin 的 Web session，成功回 `destination:"/admin?welcome=1"` 與
//! `connection:{publicUrl,projectKey,repoKey}`。流程冪等可續作，全部 mutation 先驗
//! 同源；已耗用 token 回既有不可區分的無效結果。

mod common;

use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore};
use speclink_server::setup::setup_token_ttl;
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// The origin matching `demo_config().public_url`.
const SAME_ORIGIN: &str = "http://127.0.0.1";

/// Start a fresh server with no admin and one live bootstrap setup token.
/// Returns the base URL, the plaintext token, and the identity store.
fn start_setup() -> (String, String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    let token = identity
        .create_setup_token(setup_token_ttl())
        .expect("setup token");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), token, identity)
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn get(base: &str, path: &str) -> Result<ureq::Response, ureq::Error> {
    agent().get(&format!("{base}{path}")).call()
}

fn get_with_cookie(base: &str, path: &str, session: &str) -> Result<ureq::Response, ureq::Error> {
    agent()
        .get(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .call()
}

fn post(
    base: &str,
    path: &str,
    body: Value,
    origin: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let mut req = agent().post(&format!("{base}{path}"));
    if let Some(o) = origin {
        req = req.set("Origin", o);
    }
    req.send_json(body)
}

fn json_of(result: Result<ureq::Response, ureq::Error>) -> (u16, Value) {
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_json().unwrap_or(Value::Null))
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_json().unwrap_or(Value::Null)),
        Err(e) => panic!("transport error: {e}"),
    }
}

/// The session id from a response's Set-Cookie header, if present.
fn session_cookie(resp: &ureq::Response) -> Option<String> {
    resp.header("set-cookie")?
        .split(';')
        .next()?
        .trim()
        .strip_prefix("speclink_session=")
        .map(str::to_string)
}

/// Create the admin, then register the project/repo (setup completion). Returns
/// the completion response so a caller can inspect its cookie and body.
fn complete_setup(base: &str, token: &str) -> ureq::Response {
    let (status, _b) = json_of(post(
        base,
        &format!("/api/speclink/v1/web/setup/admin?token={token}"),
        json!({ "email": "root@example.com", "display": "Root", "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    ));
    assert_eq!(status, 200, "admin creation succeeds");
    post(
        base,
        &format!("/api/speclink/v1/web/setup/registry?token={token}"),
        json!({ "projectKey": "demo", "projectName": "Demo", "repoKey": "backend", "repoName": "Backend" }),
        Some(SAME_ORIGIN),
    )
    .expect("registry completion")
}

#[test]
fn setup_state_reports_the_admin_step_and_store_status() {
    let (base, token, _id) = start_setup();
    let (status, body) = json_of(get(
        &base,
        &format!("/api/speclink/v1/web/setup?token={token}"),
    ));
    assert_eq!(status, 200, "a live token yields the current setup state");
    assert_eq!(body["data"]["step"], json!("admin"), "no admin yet → the admin step");
    // 四要素之二：顯示 Store 狀態（driver、health、identity schema version）。
    assert!(
        body["data"]["store"]["driver"].is_string(),
        "store status carries the driver: {body}"
    );
    assert!(body["data"]["store"]["healthy"].is_boolean());
    assert!(
        body["data"]["store"]["identitySchemaVersion"].is_number(),
        "the identity schema version is shown (camelCase)"
    );
}

#[test]
fn completing_setup_creates_the_admin_registry_and_logs_in() {
    let (base, token, identity) = start_setup();

    // Node 1: create the first admin, advancing to the registry step.
    let (status, body) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/setup/admin?token={token}"),
        json!({ "email": "root@example.com", "display": "Root", "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    ));
    assert_eq!(status, 200, "admin creation succeeds: {body}");
    assert_eq!(body["data"]["step"], json!("registry"), "advances to the registry step");
    assert!(identity.has_admin().unwrap(), "an admin now exists");

    // Node 2: register the first project/repo — this completes setup.
    let resp = post(
        &base,
        &format!("/api/speclink/v1/web/setup/registry?token={token}"),
        json!({ "projectKey": "demo", "projectName": "Demo", "repoKey": "backend", "repoName": "Backend" }),
        Some(SAME_ORIGIN),
    )
    .expect("registry completion");
    let session = session_cookie(&resp).expect("setup completion sets a session cookie");
    let body: Value = resp.into_json().unwrap();
    assert_eq!(
        body["data"]["destination"], json!("/admin?welcome=1"),
        "completion lands on the admin welcome"
    );
    // The public URL is the deployment config's, never written by setup.
    assert_eq!(body["data"]["connection"]["publicUrl"], json!(SAME_ORIGIN));
    assert_eq!(body["data"]["connection"]["projectKey"], json!("demo"));
    assert_eq!(body["data"]["connection"]["repoKey"], json!("backend"));

    // The session cookie authenticates as the newly-created admin.
    let (s, session_body) = json_of(get_with_cookie(
        &base,
        "/api/speclink/v1/web/session",
        &session,
    ));
    assert_eq!(s, 200);
    assert_eq!(
        session_body["data"]["authenticated"], json!(true),
        "setup completion logs the first admin in"
    );
    assert_eq!(session_body["data"]["user"]["admin"], json!(true));
    assert_eq!(session_body["data"]["user"]["email"], json!("root@example.com"));

    // The registry was written.
    assert_eq!(identity.list_projects().unwrap().len(), 1, "one project registered");
}

#[test]
fn resuming_with_the_same_token_keeps_the_admin() {
    let (base, token, identity) = start_setup();
    let (status, _b) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/setup/admin?token={token}"),
        json!({ "email": "root@example.com", "display": "Root", "password": "hunter2password" }),
        Some(SAME_ORIGIN),
    ));
    assert_eq!(status, 200);

    // Re-entering with the same live token resumes at the registry step and does
    // not rebuild the admin.
    let (status, body) = json_of(get(
        &base,
        &format!("/api/speclink/v1/web/setup?token={token}"),
    ));
    assert_eq!(status, 200, "the token is still live after the admin node");
    assert_eq!(body["data"]["step"], json!("registry"), "resumes at the registry step");
    assert_eq!(
        identity.list_users().unwrap().iter().filter(|u| u.admin).count(),
        1,
        "exactly one admin exists"
    );
}

#[test]
fn a_repeated_completion_does_not_create_a_second_project() {
    let (base, token, identity) = start_setup();
    let _ = complete_setup(&base, &token);

    // The completion request is retried with the same (now consumed) token.
    let (status, _b) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/setup/registry?token={token}"),
        json!({ "projectKey": "other", "projectName": "Other", "repoKey": "x", "repoName": "X" }),
        Some(SAME_ORIGIN),
    ));
    assert!(
        status == 404 || status == 401,
        "a consumed token is the invalid/closed result, got {status}"
    );
    assert_eq!(
        identity.list_projects().unwrap().len(),
        1,
        "no second project is created by a repeated completion"
    );
}

#[test]
fn a_cross_origin_setup_mutation_is_refused_and_creates_nothing() {
    let (base, token, identity) = start_setup();
    let (status, _b) = json_of(post(
        &base,
        &format!("/api/speclink/v1/web/setup/admin?token={token}"),
        json!({ "email": "root@example.com", "display": "Root", "password": "hunter2password" }),
        Some("http://evil.example"),
    ));
    assert_eq!(status, 403, "a foreign-origin setup mutation is refused");
    assert!(
        !identity.has_admin().unwrap(),
        "the refused request created no admin"
    );
}

#[test]
fn an_invalid_setup_token_is_indistinguishable() {
    let (base, _token, _id) = start_setup();
    // A missing and an unknown token both get the invalid response, never a state.
    let (missing, _b) = json_of(get(&base, "/api/speclink/v1/web/setup"));
    let (unknown, _b2) = json_of(get(&base, "/api/speclink/v1/web/setup?token=nope"));
    assert_eq!(missing, 401, "a missing token is invalid");
    assert_eq!(unknown, 401, "an unknown token is invalid, indistinguishably");
}

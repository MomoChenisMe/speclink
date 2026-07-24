//! First-run bootstrap (決策 3/4): mint a one-time setup token when no admin
//! exists, and serve the token-gated `/setup` flow. The flow is a single page
//! that advances through four sections — create the first admin, show the store
//! status, register the first project/repo, show the connection info — and is
//! idempotent so a resumed token continues where it left off. Setup closes for
//! good once it completes: after that /setup is 404 whatever token is presented,
//! and a missing/unknown/expired/consumed token gets one invalid response with
//! the reason never distinguished.

use crate::audit::{AuditAction, AuditActor, AuditSource};
use crate::identity::{IdentityError, IdentityStore};
use crate::state::AppState;
use crate::web;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Duration;
use serde::{Deserialize, Serialize};

/// The default lifetime of a bootstrap setup token (決策 3): long enough for an
/// operator to finish, short enough to keep the window narrow.
pub fn setup_token_ttl() -> Duration {
    Duration::hours(24)
}

/// At startup, mint a bootstrap setup token when setup is still open (no admin)
/// and no live token is already outstanding. Returns the one-time plaintext to
/// print once on stdout, or `None` when setup is closed (an admin exists) or a
/// live token already stands (its plaintext was printed on an earlier start).
pub fn ensure_setup_token(identity: &dyn IdentityStore) -> Result<Option<String>, IdentityError> {
    if identity.has_admin()? {
        return Ok(None);
    }
    if identity.has_valid_setup_token()? {
        return Ok(None);
    }
    Ok(Some(identity.create_setup_token(setup_token_ttl())?))
}

/// The `?token=` query on `GET`/`POST /setup`.
#[derive(Deserialize)]
pub struct SetupQuery {
    #[serde(default)]
    pub token: Option<String>,
}

/// The `/setup` form body. One shape carries both the admin section's fields and
/// the project/repo section's; the handler reads the pair the current step needs.
#[derive(Deserialize)]
pub struct SetupForm {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub display: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub project_key: String,
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub repo_key: String,
    #[serde(default)]
    pub repo_name: String,
}

/// `GET /setup` — the first-run flow, token-gated. Closed with a 404 once setup
/// completes; a missing/invalid/expired/consumed token gets one invalid
/// response; a valid token renders the current section.
pub async fn setup_page(State(state): State<AppState>, Query(q): Query<SetupQuery>) -> Response {
    match gate(&state, token_of(&q)) {
        Gate::Closed => setup_closed(),
        Gate::Invalid => setup_invalid(),
        Gate::Open => render_flow(&state, q.token.as_deref().unwrap_or_default()),
    }
}

/// `POST /setup` — advance one section (same-origin, token-gated). The step is
/// the admin form while no admin exists, otherwise the project/repo form. A
/// validation or duplicate-key failure re-renders that section with an error and
/// leaves the token unconsumed.
pub async fn setup_submit(
    State(state): State<AppState>,
    Query(q): Query<SetupQuery>,
    headers: HeaderMap,
    Form(form): Form<SetupForm>,
) -> Response {
    if let Err(refused) = web::check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let token = q.token.clone().unwrap_or_default();
    match gate(&state, token_of(&q)) {
        Gate::Closed => return setup_closed(),
        Gate::Invalid => return setup_invalid(),
        Gate::Open => {}
    }

    if !state.identity.has_admin().unwrap_or(false) {
        return submit_admin(&state, &token, &form);
    }
    submit_project(&state, &token, &form)
}

/// Section 1 — create the first admin, then advance.
fn submit_admin(state: &AppState, token: &str, form: &SetupForm) -> Response {
    let email = form.email.trim();
    let display = form.display.trim();
    if email.is_empty() || display.is_empty() || form.password.is_empty() {
        return Html(admin_form(token, Some("email、顯示名稱與密碼皆為必填"))).into_response();
    }
    match state
        .identity
        .create_admin_user(email, display, &form.password)
    {
        Ok(_) => render_flow(state, token),
        Err(IdentityError::Duplicate(msg)) => Html(admin_form(token, Some(&msg))).into_response(),
        Err(_) => internal_error(),
    }
}

/// Section 3 — register the first project and repo, then advance. A duplicate key
/// re-renders the form with the error; nothing is consumed.
fn submit_project(state: &AppState, token: &str, form: &SetupForm) -> Response {
    let project_key = form.project_key.trim();
    let repo_key = form.repo_key.trim();
    if project_key.is_empty() || repo_key.is_empty() {
        return Html(project_form(
            state,
            token,
            Some("project key 與 repo key 皆為必填"),
        ))
        .into_response();
    }
    let project_name = non_empty_or(&form.project_name, project_key);
    let repo_name = non_empty_or(&form.repo_name, repo_key);
    if let Err(e) = state.identity.create_project(project_key, project_name) {
        return match e {
            IdentityError::Duplicate(msg) => {
                Html(project_form(state, token, Some(&msg))).into_response()
            }
            _ => internal_error(),
        };
    }
    if let Err(e) = state.identity.create_repo(project_key, repo_key, repo_name) {
        return match e {
            IdentityError::Duplicate(msg) => {
                Html(project_form(state, token, Some(&msg))).into_response()
            }
            _ => internal_error(),
        };
    }
    render_flow(state, token)
}

/// Render the section the current state calls for: the admin form while no admin
/// exists, then the store status + project/repo form while no project exists,
/// then — once both stand — the connection info, consuming the token (setup is
/// complete).
fn render_flow(state: &AppState, token: &str) -> Response {
    if !state.identity.has_admin().unwrap_or(false) {
        return Html(admin_form(token, None)).into_response();
    }
    let projects = state.identity.list_projects().unwrap_or_default();
    let Some(project) = projects.first() else {
        return Html(project_form(state, token, None)).into_response();
    };
    // Admin + project stand: setup is complete. Consume the token (idempotent)
    // and show the connection info once. Record the completion as an audit (源
    // web, operator the first admin) — this branch runs once, since the consumed
    // token then gates every further /setup request closed (決策 3).
    let _ = state.identity.consume_setup_token(token);
    if let Ok(users) = state.identity.list_users() {
        if let Some(admin) = users.iter().find(|u| u.admin) {
            let actor = AuditActor::user(admin.id.clone(), AuditSource::Web);
            let _ = state
                .identity
                .record_audit(&actor, AuditAction::SetupCompleted, &project.key);
        }
    }
    let repos = state.identity.list_repos(&project.key).unwrap_or_default();
    let repo_key = repos.first().map(|r| r.key.as_str()).unwrap_or("");
    Html(connection_info(
        &state.config.public_url,
        &project.key,
        repo_key,
    ))
    .into_response()
}

/// The gate decision for a /setup request.
enum Gate {
    /// Setup completed (an admin exists and no live token stands) — closed for
    /// good.
    Closed,
    /// The token is missing, unknown, expired or consumed.
    Invalid,
    /// A valid token on an open (or resumable) setup.
    Open,
}

/// Decide the gate, fail closed. Setup is closed once an admin exists and no
/// live token remains — completion consumes the token, so a finished server has
/// both. A live token keeps the flow open (resumable) even after the admin is
/// created. A store error reads as closed/invalid rather than opening setup.
fn gate(state: &AppState, token: Option<&str>) -> Gate {
    let has_admin = state.identity.has_admin().unwrap_or(true);
    let has_live_token = state.identity.has_valid_setup_token().unwrap_or(false);
    if has_admin && !has_live_token {
        return Gate::Closed;
    }
    match token {
        Some(t) if state.identity.is_valid_setup_token(t).unwrap_or(false) => Gate::Open,
        _ => Gate::Invalid,
    }
}

/// The `?token=` value, treating an empty string as absent.
fn token_of(q: &SetupQuery) -> Option<&str> {
    q.token.as_deref().filter(|t| !t.is_empty())
}

// --- browser JSON setup API (決策 D2／D3／D8 第三階段)：/api/speclink/v1/web/setup ---
//
// bootstrap-token 門禁的 same-origin JSON 流程。GET 唯讀回目前步驟與 Store 狀態；兩個
// 提交節點分別建立第一位 Admin 與第一組 Project／Repo。registry 節點在既有 setup 交易
// 邊界內耗用 token、記錄 audit 並建立第一位 Admin 的 Web session，回
// `destination:"/admin?welcome=1"` 與 `connection`。gate 沿用 HTML 流程（closed→404、
// invalid→401），已耗用 token 因此不可區分。

/// `/api/speclink/v1/web` 下的 setup routes（app.rs 併入 [`crate::web::api_router`]）。
pub fn web_router() -> Router<AppState> {
    Router::new()
        .route("/setup", get(api_setup_state))
        .route("/setup/admin", post(api_setup_admin))
        .route("/setup/registry", post(api_setup_registry))
}

/// The store-status panel's fields (四要素之二)。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StoreStatusPayload {
    driver: String,
    contract_version: u32,
    level: String,
    capabilities: Vec<String>,
    healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    health_error: Option<String>,
    identity_schema_version: u32,
}

/// The current setup step and store status (GET state / after the admin node).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupStatePayload {
    step: &'static str,
    store: StoreStatusPayload,
}

/// The initial connection info returned on completion (源 = 部署組態的 public URL)。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionPayload {
    public_url: String,
    project_key: String,
    repo_key: String,
}

/// The completion response: the welcome destination plus connection info.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetupCompletePayload {
    destination: &'static str,
    connection: ConnectionPayload,
}

/// The create-admin node body (camelCase).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminBody {
    #[serde(default)]
    email: String,
    #[serde(default)]
    display: String,
    #[serde(default)]
    password: String,
}

/// The create-project/repo node body (camelCase).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryBody {
    #[serde(default)]
    project_key: String,
    #[serde(default)]
    project_name: String,
    #[serde(default)]
    repo_key: String,
    #[serde(default)]
    repo_name: String,
}

/// `GET /setup?token=` — the current step and store status. Read-only: it never
/// completes setup or mutates. Token-gated (closed → 404, invalid → 401).
pub async fn api_setup_state(State(state): State<AppState>, Query(q): Query<SetupQuery>) -> Response {
    match gate(&state, token_of(&q)) {
        Gate::Closed => web_setup_closed(),
        Gate::Invalid => web_setup_invalid(),
        Gate::Open => web::web_ok(SetupStatePayload {
            step: current_step(&state),
            store: store_payload(&state),
        }),
    }
}

/// `POST /setup/admin?token=` — create the first admin (same-origin, token-gated),
/// advancing to the registry step. Idempotent: once an admin exists a repeat is a
/// no-op that reports the registry step, never a second admin.
pub async fn api_setup_admin(
    State(state): State<AppState>,
    Query(q): Query<SetupQuery>,
    headers: HeaderMap,
    body: Result<Json<AdminBody>, JsonRejection>,
) -> Response {
    if !web::is_same_origin(&headers, &state.config.public_url) {
        return web::web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        );
    }
    match gate(&state, token_of(&q)) {
        Gate::Closed => return web_setup_closed(),
        Gate::Invalid => return web_setup_invalid(),
        Gate::Open => {}
    }
    // An admin already stands (resumed flow): do not create a second — report the
    // registry step.
    if state.identity.has_admin().unwrap_or(false) {
        return web::web_ok(SetupStatePayload {
            step: "registry",
            store: store_payload(&state),
        });
    }
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let email = body.email.trim();
    let display = body.display.trim();
    if email.is_empty() || display.is_empty() || body.password.is_empty() {
        return web::web_err(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "email、顯示名稱與密碼皆為必填",
        );
    }
    match state
        .identity
        .create_admin_user(email, display, &body.password)
    {
        Ok(_) => web::web_ok(SetupStatePayload {
            step: "registry",
            store: store_payload(&state),
        }),
        Err(IdentityError::Duplicate(msg)) => {
            web::web_field_err(StatusCode::CONFLICT, "duplicate", msg, "email", "此 email 已被使用")
        }
        Err(_) => internal_error_json(),
    }
}

/// `POST /setup/registry?token=` — register the first project/repo, then complete
/// setup (the last node): consume the token, record the audit, open the admin's
/// session, and return the welcome destination and connection info. Idempotent on
/// resume: a project that already stands is not recreated.
pub async fn api_setup_registry(
    State(state): State<AppState>,
    Query(q): Query<SetupQuery>,
    headers: HeaderMap,
    body: Result<Json<RegistryBody>, JsonRejection>,
) -> Response {
    if !web::is_same_origin(&headers, &state.config.public_url) {
        return web::web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        );
    }
    let token = q.token.clone().unwrap_or_default();
    match gate(&state, token_of(&q)) {
        Gate::Closed => return web_setup_closed(),
        Gate::Invalid => return web_setup_invalid(),
        Gate::Open => {}
    }
    // The admin node precedes the registry node.
    if !state.identity.has_admin().unwrap_or(false) {
        return web::web_err(StatusCode::CONFLICT, "admin_required", "請先建立管理員");
    }
    // Register the project/repo if none stands yet (idempotent on resume).
    if state.identity.list_projects().unwrap_or_default().is_empty() {
        let Ok(Json(body)) = body else {
            return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
        };
        let project_key = body.project_key.trim();
        let repo_key = body.repo_key.trim();
        if project_key.is_empty() || repo_key.is_empty() {
            return web::web_err(
                StatusCode::BAD_REQUEST,
                "validation_error",
                "project key 與 repo key 皆為必填",
            );
        }
        let project_name = non_empty_or(&body.project_name, project_key);
        let repo_name = non_empty_or(&body.repo_name, repo_key);
        if let Err(e) = state.identity.create_project(project_key, project_name) {
            return registry_error(e);
        }
        if let Err(e) = state.identity.create_repo(project_key, repo_key, repo_name) {
            return registry_error(e);
        }
    }
    complete_setup_json(&state, &token)
}

/// The current step: the admin form while no admin exists, otherwise the registry
/// form.
fn current_step(state: &AppState) -> &'static str {
    if state.identity.has_admin().unwrap_or(false) {
        "registry"
    } else {
        "admin"
    }
}

/// Complete setup in the existing transaction boundary: consume the token, record
/// the `SetupCompleted` audit (源 web, operator the first admin), and open the
/// admin's Web session. A session failure is a retryable recovery — the created
/// admin is never presented as logged in.
fn complete_setup_json(state: &AppState, token: &str) -> Response {
    let projects = state.identity.list_projects().unwrap_or_default();
    let Some(project) = projects.first() else {
        return internal_error_json();
    };
    let Some(admin) = state
        .identity
        .list_users()
        .unwrap_or_default()
        .into_iter()
        .find(|u| u.admin)
    else {
        return internal_error_json();
    };
    // Consume the token (idempotent) and record the completion audit once — the
    // consumed token then gates every further /setup request closed.
    let _ = state.identity.consume_setup_token(token);
    let actor = AuditActor::user(admin.id.clone(), AuditSource::Web);
    let _ = state
        .identity
        .record_audit(&actor, AuditAction::SetupCompleted, &project.key);
    let session = match web::open_session(state, &admin.id) {
        Ok(session) => session,
        Err(recovery) => return recovery,
    };
    let repos = state.identity.list_repos(&project.key).unwrap_or_default();
    let repo_key = repos.first().map(|r| r.key.clone()).unwrap_or_default();
    web::with_session_cookie(
        web::web_ok(SetupCompletePayload {
            destination: "/admin?welcome=1",
            connection: ConnectionPayload {
                public_url: state.config.public_url.clone(),
                project_key: project.key.clone(),
                repo_key,
            },
        }),
        &session,
    )
}

/// The store manifest, health and identity schema version as a JSON payload.
fn store_payload(state: &AppState) -> StoreStatusPayload {
    let manifest = state.store.manifest();
    let (healthy, health_error) = match state.store.health() {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    StoreStatusPayload {
        driver: manifest.driver.clone(),
        contract_version: manifest.contract_version,
        level: manifest.level.as_str().to_string(),
        capabilities: manifest
            .capabilities
            .iter()
            .map(|c| c.as_str().to_string())
            .collect(),
        healthy,
        health_error,
        identity_schema_version: state.identity.schema_version().unwrap_or(0),
    }
}

/// A registry-creation error as JSON: a duplicate key is a 409, anything else 500.
fn registry_error(e: IdentityError) -> Response {
    match e {
        IdentityError::Duplicate(msg) => web::web_err(StatusCode::CONFLICT, "duplicate", msg),
        _ => internal_error_json(),
    }
}

/// The JSON closed result — a 404 once setup completes (any token).
fn web_setup_closed() -> Response {
    web::web_err(StatusCode::NOT_FOUND, "setup_closed", "找不到頁面")
}

/// The JSON invalid-token result — 401, byte-identical for a missing, unknown,
/// expired or consumed token so the reason is never distinguished.
fn web_setup_invalid() -> Response {
    web::web_err(StatusCode::UNAUTHORIZED, "invalid_setup_token", "設定連結無效")
}

fn internal_error_json() -> Response {
    web::web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤")
}

// --- rendering (embedded, no external resources; reuses web.rs's shell) ---

/// Section 1: the create-admin form.
fn admin_form(token: &str, error: Option<&str>) -> String {
    let body = format!(
        "<h1>Speclink 初始設定</h1>\n<h2>1. 建立管理員帳號</h2>\n{error}<form method=\"post\" action=\"/setup?token={token}\">\n<label>Email <input type=\"email\" name=\"email\" required></label>\n<label>顯示名稱 <input type=\"text\" name=\"display\" required></label>\n<label>密碼 <input type=\"password\" name=\"password\" required></label>\n<button type=\"submit\">建立管理員</button>\n</form>\n",
        error = error_block(error),
        token = web::escape(token),
    );
    web::page("初始設定", &body)
}

/// Section 2 + 3: the store status panel and the create-project/repo form.
fn project_form(state: &AppState, token: &str, error: Option<&str>) -> String {
    let body = format!(
        "<h1>Speclink 初始設定</h1>\n<p>管理員已建立。</p>\n<h2>2. Store 狀態</h2>\n{status}<h2>3. 建立第一組 Project 與 Repo</h2>\n{error}<form method=\"post\" action=\"/setup?token={token}\">\n<label>Project key <input type=\"text\" name=\"project_key\" required></label>\n<label>Project 名稱 <input type=\"text\" name=\"project_name\"></label>\n<label>Repo key <input type=\"text\" name=\"repo_key\" required></label>\n<label>Repo 名稱 <input type=\"text\" name=\"repo_name\"></label>\n<button type=\"submit\">建立 Project 與 Repo</button>\n</form>\n",
        status = store_status(state),
        error = error_block(error),
        token = web::escape(token),
    );
    web::page("初始設定", &body)
}

/// The store manifest, health, and identity schema version panel (決策 4).
fn store_status(state: &AppState) -> String {
    let manifest = state.store.manifest();
    let caps = manifest
        .capabilities
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let health = match state.store.health() {
        Ok(()) => "正常".to_string(),
        Err(e) => format!("異常：{e}"),
    };
    let schema = state
        .identity
        .schema_version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "未知".to_string());
    format!(
        "<ul>\n<li>Store driver：{driver}</li>\n<li>Contract 版本：{contract}</li>\n<li>能力等級：{level}</li>\n<li>能力：{caps}</li>\n<li>健康檢查：{health}</li>\n<li>Identity schema 版本：{schema}</li>\n</ul>\n",
        driver = web::escape(&manifest.driver),
        contract = manifest.contract_version,
        level = manifest.level.as_str(),
        caps = web::escape(&caps),
        health = web::escape(&health),
        schema = web::escape(&schema),
    )
}

/// Section 4: the initial connection info (决策 4). The public url is the
/// deployment config's, never written by setup.
fn connection_info(public_url: &str, project_key: &str, repo_key: &str) -> String {
    let body = format!(
        "<h1>Speclink 初始設定完成</h1>\n<h2>4. 連線資訊</h2>\n<ul>\n<li>Public URL：{url}</li>\n<li>Project：{project}</li>\n<li>Repo：{repo}</li>\n</ul>\n<p>用 invite 子命令或後續 admin 介面邀請成員後即可連線。</p>\n",
        url = web::escape(public_url),
        project = web::escape(project_key),
        repo = web::escape(repo_key),
    );
    web::page("初始設定完成", &body)
}

/// An optional error paragraph.
fn error_block(error: Option<&str>) -> String {
    error
        .map(|e| format!("<p class=\"error\">{}</p>\n", web::escape(e)))
        .unwrap_or_default()
}

/// A trimmed value, or `fallback` when it is empty.
fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

/// `/setup` is closed once setup completes — a bare 404 for any token.
fn setup_closed() -> Response {
    (
        StatusCode::NOT_FOUND,
        Html(web::page("找不到頁面", "<h1>找不到頁面</h1>\n")),
    )
        .into_response()
}

/// The single invalid-token response — byte-identical for a missing, unknown,
/// expired or consumed token so the reason is never distinguished.
fn setup_invalid() -> Response {
    let body = "<h1>設定連結無效</h1>\n<p>這個初始設定連結無法使用。請使用 server 啟動時輸出的最新連結。</p>\n";
    (
        StatusCode::UNAUTHORIZED,
        Html(web::page("設定連結無效", body)),
    )
        .into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(web::page("錯誤", "<h1>發生錯誤</h1>\n")),
    )
        .into_response()
}

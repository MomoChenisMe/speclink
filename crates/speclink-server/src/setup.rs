//! First-run bootstrap (決策 3/4): mint a one-time setup token when no admin
//! exists, and serve the token-gated `/setup` flow. The flow is a single page
//! that advances through four sections — create the first admin, show the store
//! status, register the first project/repo, show the connection info — and is
//! idempotent so a resumed token continues where it left off. Setup closes for
//! good once it completes: after that /setup is 404 whatever token is presented,
//! and a missing/unknown/expired/consumed token gets one invalid response with
//! the reason never distinguished.

use crate::identity::{IdentityError, IdentityStore};
use crate::state::AppState;
use crate::web;
use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use chrono::Duration;
use serde::Deserialize;

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
    match state.identity.create_admin_user(email, display, &form.password) {
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
        return Html(project_form(state, token, Some("project key 與 repo key 皆為必填"))).into_response();
    }
    let project_name = non_empty_or(&form.project_name, project_key);
    let repo_name = non_empty_or(&form.repo_name, repo_key);
    if let Err(e) = state.identity.create_project(project_key, project_name) {
        return match e {
            IdentityError::Duplicate(msg) => Html(project_form(state, token, Some(&msg))).into_response(),
            _ => internal_error(),
        };
    }
    if let Err(e) = state.identity.create_repo(project_key, repo_key, repo_name) {
        return match e {
            IdentityError::Duplicate(msg) => Html(project_form(state, token, Some(&msg))).into_response(),
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
    // and show the connection info once.
    let _ = state.identity.consume_setup_token(token);
    let repos = state.identity.list_repos(&project.key).unwrap_or_default();
    let repo_key = repos.first().map(|r| r.key.as_str()).unwrap_or("");
    Html(connection_info(&state.config.public_url, &project.key, repo_key)).into_response()
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
    (StatusCode::NOT_FOUND, Html(web::page("找不到頁面", "<h1>找不到頁面</h1>\n"))).into_response()
}

/// The single invalid-token response — byte-identical for a missing, unknown,
/// expired or consumed token so the reason is never distinguished.
fn setup_invalid() -> Response {
    let body = "<h1>設定連結無效</h1>\n<p>這個初始設定連結無法使用。請使用 server 啟動時輸出的最新連結。</p>\n";
    (StatusCode::UNAUTHORIZED, Html(web::page("設定連結無效", body))).into_response()
}

fn internal_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Html(web::page("錯誤", "<h1>發生錯誤</h1>\n"))).into_response()
}

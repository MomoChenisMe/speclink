//! The management面: the admin gate, the admin JSON API and the /admin
//! server-rendered pages (server-admin spec, 決策 1). The gate is a flag check
//! layered on the existing authentication — the admin API resolves a bearer
//! token per-request (so suspension and revocation are immediate) and the /admin
//! pages resolve the session cookie, and both then require the user's admin flag;
//! a non-admin is 403 permission_denied and no action runs. Every state-changing
//! action is a single function that writes the identity mutation and its audit
//! record in one transaction (決策 2/3); the API handler, the /admin form and the
//! CLI subcommand all call the same path.

use crate::audit::{AuditAction, AuditActor, AuditSource};
use crate::auth::{bearer_token, header};
use crate::error::ApiError;
use crate::identity::{IdentityError, NewInvitation, User};
use crate::state::{AppState, SharedStore};
use crate::web;
use axum::extract::{Form, FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use speclink_protocol::API_VERSION;
use speclink_store::{OutboxCursor, ProjectId, RepoId, Scope, StoreError, CONTRACT_VERSION};
use std::collections::HashMap;

/// An acknowledgment body for admin API actions whose response the caller ignores.
#[derive(Serialize)]
struct Ack {}

/// A resolved admin API request: the authenticated administrator and the audit
/// actor (source `api`) its actions record under. The extractor is the admin API
/// gate — a request that reaches a handler has already passed authentication, the
/// admin-flag check and the API version check.
#[derive(Debug, Clone)]
pub struct AdminApi {
    pub user: User,
    pub actor: AuditActor,
}

impl FromRequestParts<AppState> for AdminApi {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        // 1. bearer token → user, resolved per-request against the identity store
        //    (決策 1): `spk_at_` is a device access token, anything else a PAT,
        //    into the same check-list (hash-match, unrevoked, unexpired, owning
        //    user active). No cache means suspension and revocation are immediate.
        let token = bearer_token(parts)
            .ok_or_else(|| ApiError::permission_denied("missing or malformed bearer token"))?;
        let (user, touch_pat_id): (User, Option<String>) = if token.starts_with("spk_at_") {
            let user = state
                .identity
                .authenticate_access_token(&token)
                .map_err(|_| ApiError::internal("identity store unavailable"))?
                .ok_or_else(|| ApiError::permission_denied("invalid token"))?;
            (user, None)
        } else {
            let (pat, user) = state
                .identity
                .authenticate_pat(&token)
                .map_err(|_| ApiError::internal("identity store unavailable"))?
                .ok_or_else(|| ApiError::permission_denied("invalid token"))?;
            (user, Some(pat.id))
        };

        // 2. admin flag — a valid but non-admin token is 403, the same reason as
        //    the non-member 403 (SHALL NOT 新增 wire reason).
        if !user.admin {
            return Err(ApiError::forbidden("actor is not an administrator"));
        }

        // 3. API version compatibility — the same check every other API route runs.
        let version = header(parts, "x-speclink-api-version");
        if version.as_deref() != Some(API_VERSION) {
            let sent = version.map(|v| format!(", client sent '{v}'")).unwrap_or_default();
            return Err(ApiError::refused(format!(
                "incompatible api version — this server speaks version '{API_VERSION}'{sent}"
            )));
        }

        // Past the gate: advance a PAT's last-used best-effort (an access token
        // keeps none — it is short-lived and rotates).
        if let Some(pat_id) = &touch_pat_id {
            let _ = state.identity.touch_pat(pat_id);
        }

        let actor = AuditActor::user(user.id.clone(), AuditSource::Api);
        Ok(AdminApi { user, actor })
    }
}

/// The /admin page gate: the session cookie must resolve to an active admin. A
/// logged-in non-admin is 403; an unauthenticated (or suspended, whose session
/// no longer authenticates) visit redirects to login. On success the audit actor
/// records source `web`.
pub(crate) fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(User, AuditActor), Response> {
    match web::current_user(state, headers) {
        Some(user) if user.admin => {
            let actor = AuditActor::user(user.id.clone(), AuditSource::Web);
            Ok((user, actor))
        }
        Some(_) => Err(forbidden_page()),
        None => Err(Redirect::to("/login").into_response()),
    }
}

/// The 403 page for a logged-in non-admin. Reuses the shared HTML shell.
fn forbidden_page() -> Response {
    (
        axum::http::StatusCode::FORBIDDEN,
        axum::response::Html(web::page("拒絕存取", "<h1>需要管理員權限</h1>\n<p>你的帳號沒有管理權限。</p>\n")),
    )
        .into_response()
}

// --- admin JSON API ---

/// Pagination for the audit list: newest first, `limit` per page from `offset`.
#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_limit() -> u32 {
    50
}

/// One audit record on the wire: the five-tuple, the time as RFC3339.
#[derive(Serialize)]
struct AuditEntryDto {
    id: String,
    actor_id: String,
    action: String,
    subject: String,
    source: String,
    created_at: String,
}

/// A page of audit records.
#[derive(Serialize)]
struct AuditListResponse {
    entries: Vec<AuditEntryDto>,
}

/// `GET /api/speclink/v1/admin/audit` — a read-only, newest-first page of the
/// management audit log.
pub async fn list_audit(
    State(state): State<AppState>,
    _admin: AdminApi,
    Query(query): Query<AuditQuery>,
) -> Result<Response, ApiError> {
    let entries = state
        .identity
        .list_audit(query.limit, query.offset)
        .map_err(|_| ApiError::internal("identity store unavailable"))?;
    let dto = AuditListResponse {
        entries: entries
            .into_iter()
            .map(|e| AuditEntryDto {
                id: e.id,
                actor_id: e.actor_id,
                action: e.action,
                subject: e.subject,
                source: e.source,
                created_at: e.created_at.to_rfc3339(),
            })
            .collect(),
    };
    Ok(Json(dto).into_response())
}

// --- admin API mutating actions (源 api; each calls the single-point admin_* fn) ---

/// The create-project / create-repo JSON body. `name` defaults to the key.
#[derive(Deserialize)]
pub struct CreateProjectBody {
    key: String,
    #[serde(default)]
    name: Option<String>,
}

/// The create-repo JSON body.
#[derive(Deserialize)]
pub struct CreateRepoBody {
    project_key: String,
    key: String,
    #[serde(default)]
    name: Option<String>,
}

/// A trimmed override, or the key itself when the name is empty.
fn name_or_key<'a>(name: &'a Option<String>, key: &'a str) -> &'a str {
    name.as_deref().map(str::trim).filter(|n| !n.is_empty()).unwrap_or(key)
}

/// `POST /api/speclink/v1/admin/users/{id}/suspend`
pub async fn api_suspend_user(
    State(state): State<AppState>,
    admin: AdminApi,
    Path(user_id): Path<String>,
) -> Result<Response, ApiError> {
    state.identity.admin_set_user_suspended(&admin.actor, &user_id, true)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/users/{id}/reactivate`
pub async fn api_reactivate_user(
    State(state): State<AppState>,
    admin: AdminApi,
    Path(user_id): Path<String>,
) -> Result<Response, ApiError> {
    state.identity.admin_set_user_suspended(&admin.actor, &user_id, false)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/tokens/{id}/revoke`
pub async fn api_revoke_token(
    State(state): State<AppState>,
    admin: AdminApi,
    Path(pat_id): Path<String>,
) -> Result<Response, ApiError> {
    state.identity.admin_revoke_pat(&admin.actor, &pat_id)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/projects`
pub async fn api_create_project(
    State(state): State<AppState>,
    admin: AdminApi,
    Json(body): Json<CreateProjectBody>,
) -> Result<Response, ApiError> {
    let name = name_or_key(&body.name, &body.key).to_string();
    state.identity.admin_create_project(&admin.actor, &body.key, &name)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/repos`
pub async fn api_create_repo(
    State(state): State<AppState>,
    admin: AdminApi,
    Json(body): Json<CreateRepoBody>,
) -> Result<Response, ApiError> {
    let name = name_or_key(&body.name, &body.key).to_string();
    state.identity.admin_create_repo(&admin.actor, &body.project_key, &body.key, &name)?;
    Ok(Json(Ack {}).into_response())
}

/// The admin JSON API sub-router, nested under `/api/speclink/v1/admin`. Every
/// route's handler takes the [`AdminApi`] gate. These mirror the CLI subcommands
/// (決策 2: 與 API 同函式) and the /admin forms — one function, three entries.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/audit", get(list_audit))
        .route("/system", get(api_system))
        .route("/users/{id}/suspend", post(api_suspend_user))
        .route("/users/{id}/reactivate", post(api_reactivate_user))
        .route("/tokens/{id}/revoke", post(api_revoke_token))
        .route("/projects", post(api_create_project))
        .route("/repos", post(api_create_repo))
}

// --- /admin pages ---

/// `GET /admin` — the management home: a nav to the page組 and nothing else. It
/// deliberately links only to installation/administration pages — never to any
/// spec content (changes、specs、discussions), which the /admin UI does not serve.
pub async fn admin_home(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let (_user, _actor) = match require_admin(&state, &headers) {
        Ok(admin) => admin,
        Err(refused) => return refused,
    };
    let body = "<h1>管理面</h1>\n<nav>\n<ul>\n\
        <li><a href=\"/admin/users\">使用者</a></li>\n\
        <li><a href=\"/admin/registry\">Registry（Project／Repo）</a></li>\n\
        <li><a href=\"/admin/credentials\">憑證</a></li>\n\
        <li><a href=\"/admin/data\">資料操作</a></li>\n\
        <li><a href=\"/admin/system\">系統狀態</a></li>\n\
        <li><a href=\"/admin/audit\">Audit log</a></li>\n\
        </ul>\n</nav>\n";
    Html(web::page("管理面", body)).into_response()
}

// --- /admin form actions (源 web; session + same-origin, then the single-point fn) ---

/// `POST /admin/users/{id}/suspend` — a /admin form suspends a user; the acting
/// admin's audit records source web. A refused action (last active admin) shows
/// the reason.
pub async fn web_suspend_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Response {
    set_user_suspended_via_form(&state, &headers, &user_id, true)
}

/// `POST /admin/users/{id}/reactivate` — the reactivate counterpart.
pub async fn web_reactivate_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Response {
    set_user_suspended_via_form(&state, &headers, &user_id, false)
}

/// Shared body for the suspend / reactivate form handlers: same-origin, admin
/// gate, then the single-point action under a web-sourced actor.
fn set_user_suspended_via_form(
    state: &AppState,
    headers: &HeaderMap,
    user_id: &str,
    suspended: bool,
) -> Response {
    let actor = match guard_web(state, headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_set_user_suspended(&actor, user_id, suspended) {
        Ok(()) => Redirect::to("/admin/users").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// The /admin form gate: same-origin then the admin session, yielding the
/// web-sourced audit actor. Every form POST runs it before its single-point call.
fn guard_web(state: &AppState, headers: &HeaderMap) -> Result<AuditActor, Response> {
    if let Err(refused) = web::check_origin(headers, &state.config.public_url) {
        return Err(refused);
    }
    require_admin(state, headers).map(|(_user, actor)| actor)
}

/// Render a management-action failure as a page carrying its reason (e.g. the
/// last active admin cannot be suspended). Not-found is 404, other refusals 409.
fn admin_action_error(err: &IdentityError) -> Response {
    let status = match err {
        IdentityError::NotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::CONFLICT,
    };
    let body = format!(
        "<h1>操作未完成</h1>\n<p>{}</p>\n<p><a href=\"/admin\">回到管理面</a></p>\n",
        web::escape(&err.to_string())
    );
    (status, Html(web::page("操作未完成", &body))).into_response()
}

/// Format a timestamp for display (date and minute, UTC), matching the account page.
fn fmt_ts(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M UTC").to_string()
}

// --- /admin users page (決策 2) ---

/// The invite form submission: email, optional display name, comma-separated
/// initial project memberships, and the admin checkbox (present when checked).
#[derive(Deserialize)]
pub struct InviteForm {
    email: String,
    #[serde(default)]
    display: String,
    #[serde(default)]
    projects: String,
    #[serde(default)]
    admin: Option<String>,
}

/// The membership form: a project key and whether to grant or revoke it.
#[derive(Deserialize)]
pub struct MembershipForm {
    project_key: String,
    #[serde(default)]
    action: String,
}

/// The admin-flag toggle: the next value as a string ("true"/"false").
#[derive(Deserialize)]
pub struct AdminFlagForm {
    #[serde(default)]
    admin: String,
}

/// `GET /admin/users` — the user list with the invite/suspend/reactivate/
/// membership/admin-flag forms.
pub async fn users_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_users(&state, None, None)).into_response()
}

/// `POST /admin/users/invite` — mint an invitation; the acceptance URL is shown
/// once on the re-rendered page.
pub async fn web_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<InviteForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let email = form.email.trim();
    if email.is_empty() {
        return Html(render_users(&state, None, Some("email 為必填"))).into_response();
    }
    let display = if form.display.trim().is_empty() {
        email.to_string()
    } else {
        form.display.trim().to_string()
    };
    let memberships: Vec<String> = form
        .projects
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    let req = NewInvitation {
        email: email.to_string(),
        display,
        memberships,
        admin: form.admin.is_some(),
        expires_at: Utc::now() + Duration::days(7),
    };
    match state.identity.admin_create_invitation(&actor, req) {
        Ok(token) => {
            let url = format!("{}/invite/{token}", state.config.public_url.trim_end_matches('/'));
            Html(render_users(&state, Some(&url), None)).into_response()
        }
        Err(e) => Html(render_users(&state, None, Some(&e.to_string()))).into_response(),
    }
}

/// `POST /admin/users/{id}/membership` — grant or revoke a project membership.
pub async fn web_set_membership(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Form(form): Form<MembershipForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let member = form.action == "grant";
    match state.identity.admin_set_membership(&actor, &user_id, form.project_key.trim(), member) {
        Ok(()) => Redirect::to("/admin/users").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// `POST /admin/users/{id}/admin-flag` — set or clear the admin flag.
pub async fn web_set_admin_flag(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    Form(form): Form<AdminFlagForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_set_admin_flag(&actor, &user_id, form.admin == "true") {
        Ok(()) => Redirect::to("/admin/users").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// Render the user-management page. `flash` shows a fresh invite URL once;
/// `error` shows an action failure inline.
fn render_users(state: &AppState, flash: Option<&str>, error: Option<&str>) -> String {
    let users = state.identity.list_users().unwrap_or_default();
    let projects = state.identity.list_projects().unwrap_or_default();
    let options: String = projects
        .iter()
        .map(|p| format!("<option value=\"{k}\">{k}</option>", k = web::escape(&p.key)))
        .collect();

    let flash_html = flash
        .map(|url| format!("<div class=\"flash\"><p>邀請連結（只顯示一次）：</p>\n<code>{}</code></div>\n", web::escape(url)))
        .unwrap_or_default();
    let error_html = error
        .map(|e| format!("<p class=\"error\">{}</p>\n", web::escape(e)))
        .unwrap_or_default();

    let mut rows = String::new();
    for u in &users {
        let id = web::escape(&u.id);
        let memberships = state.identity.list_memberships(&u.id).unwrap_or_default();
        let mut mem_html = String::new();
        for m in &memberships {
            mem_html.push_str(&format!(
                "<form method=\"post\" action=\"/admin/users/{id}/membership\" class=\"inline\"><input type=\"hidden\" name=\"project_key\" value=\"{k}\"><input type=\"hidden\" name=\"action\" value=\"revoke\"><button type=\"submit\">{k} ✕</button></form> ",
                k = web::escape(m)
            ));
        }
        let add_mem = if projects.is_empty() {
            String::new()
        } else {
            format!(
                "<form method=\"post\" action=\"/admin/users/{id}/membership\" class=\"inline\"><input type=\"hidden\" name=\"action\" value=\"grant\"><select name=\"project_key\">{options}</select><button type=\"submit\">加入</button></form>"
            )
        };
        let active_ctrl = if u.active {
            format!("<form method=\"post\" action=\"/admin/users/{id}/suspend\" class=\"inline\"><button type=\"submit\">停權</button></form>")
        } else {
            format!("<form method=\"post\" action=\"/admin/users/{id}/reactivate\" class=\"inline\"><button type=\"submit\">復權</button></form>")
        };
        let admin_ctrl = format!(
            "<form method=\"post\" action=\"/admin/users/{id}/admin-flag\" class=\"inline\"><input type=\"hidden\" name=\"admin\" value=\"{next}\"><button type=\"submit\">{label}</button></form>",
            next = if u.admin { "false" } else { "true" },
            label = if u.admin { "撤除管理權" } else { "設為管理員" }
        );
        rows.push_str(&format!(
            "<li><strong>{email}</strong>{admin_badge}{status}<br>成員：{mem}{add_mem}<br>{active_ctrl} {admin_ctrl}</li>\n",
            email = web::escape(&u.email),
            admin_badge = if u.admin { "（管理員）" } else { "" },
            status = if u.active { "" } else { "（已停權）" },
            mem = mem_html,
        ));
    }

    let body = format!(
        "<h1>使用者管理</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <h2>邀請成員</h2>\n{flash_html}{error_html}\
         <form method=\"post\" action=\"/admin/users/invite\">\n\
         <label>Email <input type=\"email\" name=\"email\" required></label>\n\
         <label>顯示名稱 <input type=\"text\" name=\"display\"></label>\n\
         <label>加入 Project（逗號分隔，可留空） <input type=\"text\" name=\"projects\"></label>\n\
         <label><input type=\"checkbox\" name=\"admin\"> 設為管理員</label>\n\
         <button type=\"submit\">建立邀請</button>\n</form>\n\
         <h2>使用者</h2>\n<ul>\n{rows}</ul>\n"
    );
    web::page("使用者管理", &body)
}

// --- /admin registry page ---

/// The create-project / create-repo form submissions.
#[derive(Deserialize)]
pub struct CreateProjectForm {
    key: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
pub struct CreateRepoForm {
    project_key: String,
    key: String,
    #[serde(default)]
    name: String,
}

/// A rename form: just the new display name (the key is in the path or a hidden field).
#[derive(Deserialize)]
pub struct RenameForm {
    name: String,
}

#[derive(Deserialize)]
pub struct RenameRepoForm {
    project_key: String,
    key: String,
    name: String,
}

/// `GET /admin/registry` — the project/repo list with create and rename forms.
pub async fn registry_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_registry(&state)).into_response()
}

/// `POST /admin/registry/projects` — register a project.
pub async fn web_create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateProjectForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let key = form.key.trim();
    let name = non_empty(&form.name).unwrap_or(key);
    match state.identity.admin_create_project(&actor, key, name) {
        Ok(()) => Redirect::to("/admin/registry").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// `POST /admin/registry/projects/{key}/rename` — change a project's display name.
pub async fn web_rename_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    Form(form): Form<RenameForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_rename_project(&actor, &key, form.name.trim()) {
        Ok(()) => Redirect::to("/admin/registry").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// `POST /admin/registry/repos` — register a repo within a project.
pub async fn web_create_repo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreateRepoForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let key = form.key.trim();
    let name = non_empty(&form.name).unwrap_or(key);
    match state.identity.admin_create_repo(&actor, form.project_key.trim(), key, name) {
        Ok(()) => Redirect::to("/admin/registry").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// `POST /admin/registry/repos/rename` — change a repo's display name.
pub async fn web_rename_repo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RenameRepoForm>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_rename_repo(&actor, form.project_key.trim(), form.key.trim(), form.name.trim()) {
        Ok(()) => Redirect::to("/admin/registry").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// Render the registry page.
fn render_registry(state: &AppState) -> String {
    let projects = state.identity.list_projects().unwrap_or_default();
    let mut project_html = String::new();
    for p in &projects {
        let key = web::escape(&p.key);
        let repos = state.identity.list_repos(&p.key).unwrap_or_default();
        let mut repo_html = String::new();
        for r in &repos {
            repo_html.push_str(&format!(
                "<li><code>{rk}</code> {rn}\n<form method=\"post\" action=\"/admin/registry/repos/rename\" class=\"inline\"><input type=\"hidden\" name=\"project_key\" value=\"{key}\"><input type=\"hidden\" name=\"key\" value=\"{rk}\"><input type=\"text\" name=\"name\" placeholder=\"新顯示名\" required><button type=\"submit\">改名</button></form></li>\n",
                rk = web::escape(&r.key),
                rn = web::escape(&r.name),
            ));
        }
        project_html.push_str(&format!(
            "<li><strong>{name}</strong> <code>{key}</code>\n\
             <form method=\"post\" action=\"/admin/registry/projects/{key}/rename\" class=\"inline\"><input type=\"text\" name=\"name\" placeholder=\"新顯示名\" required><button type=\"submit\">改名</button></form>\n\
             <ul>\n{repo_html}</ul>\n\
             <form method=\"post\" action=\"/admin/registry/repos\" class=\"inline\"><input type=\"hidden\" name=\"project_key\" value=\"{key}\"><input type=\"text\" name=\"key\" placeholder=\"repo key\" required><input type=\"text\" name=\"name\" placeholder=\"顯示名（可留空）\"><button type=\"submit\">新增 Repo</button></form>\n</li>\n",
            name = web::escape(&p.name),
        ));
    }
    let body = format!(
        "<h1>Registry 管理</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <h2>新增 Project</h2>\n\
         <form method=\"post\" action=\"/admin/registry/projects\">\n\
         <label>Key <input type=\"text\" name=\"key\" required></label>\n\
         <label>顯示名（可留空） <input type=\"text\" name=\"name\"></label>\n\
         <button type=\"submit\">新增 Project</button>\n</form>\n\
         <h2>Project 與 Repo</h2>\n<ul>\n{project_html}</ul>\n\
         <p>Key 為穩定識別，不可變更；只能修改顯示名。</p>\n"
    );
    web::page("Registry 管理", &body)
}

// --- /admin credentials page (決策 4) ---

/// `GET /admin/credentials` — the site-wide PAT and device-family metadata, each
/// with a force-revoke button. Metadata only — no secret is ever shown.
pub async fn credentials_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_credentials(&state)).into_response()
}

/// `POST /admin/credentials/tokens/{id}/revoke` — force-revoke a PAT.
pub async fn web_revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(pat_id): Path<String>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_revoke_pat(&actor, &pat_id) {
        Ok(()) => Redirect::to("/admin/credentials").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// `POST /admin/credentials/families/{id}/revoke` — force-revoke a device family.
pub async fn web_revoke_family(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(family_id): Path<String>,
) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    match state.identity.admin_revoke_family(&actor, &family_id) {
        Ok(()) => Redirect::to("/admin/credentials").into_response(),
        Err(e) => admin_action_error(&e),
    }
}

/// Render the credential-oversight page: PAT and device-family metadata only.
fn render_credentials(state: &AppState) -> String {
    let emails: HashMap<String, String> = state
        .identity
        .list_users()
        .unwrap_or_default()
        .into_iter()
        .map(|u| (u.id, u.email))
        .collect();
    let owner = |uid: &str| emails.get(uid).cloned().unwrap_or_else(|| uid.to_string());

    let pats = state.identity.list_all_pats().unwrap_or_default();
    let mut pat_rows = String::new();
    for p in &pats {
        let revoke = if p.revoked_at.is_none() {
            format!(
                "<form method=\"post\" action=\"/admin/credentials/tokens/{id}/revoke\" class=\"inline\"><button type=\"submit\">撤銷</button></form>",
                id = web::escape(&p.id)
            )
        } else {
            "（已撤銷）".to_string()
        };
        pat_rows.push_str(&format!(
            "<tr><td><code>{prefix}</code></td><td>{name}</td><td>{owner}</td><td>{expires}</td><td>{last_used}</td><td>{created}</td><td>{revoke}</td></tr>\n",
            prefix = web::escape(&p.prefix),
            name = web::escape(&p.name),
            owner = web::escape(&owner(&p.user_id)),
            expires = p.expires_at.map(fmt_ts).unwrap_or_else(|| "永久".to_string()),
            last_used = p.last_used_at.map(fmt_ts).unwrap_or_else(|| "從未".to_string()),
            created = fmt_ts(p.created_at),
        ));
    }

    let families = state.identity.list_all_device_families().unwrap_or_default();
    let mut fam_rows = String::new();
    for (uid, f) in &families {
        let revoke = if f.revoked_at.is_none() {
            format!(
                "<form method=\"post\" action=\"/admin/credentials/families/{id}/revoke\" class=\"inline\"><button type=\"submit\">撤銷</button></form>",
                id = web::escape(&f.id)
            )
        } else {
            "（已撤銷）".to_string()
        };
        fam_rows.push_str(&format!(
            "<tr><td>{owner}</td><td>{source}</td><td>{created}</td><td>{last}</td><td>{revoke}</td></tr>\n",
            owner = web::escape(&owner(uid)),
            source = web::escape(&f.source),
            created = fmt_ts(f.created_at),
            last = fmt_ts(f.last_refresh_at),
        ));
    }

    let body = format!(
        "<h1>憑證監督</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <p>僅顯示 metadata；明文與 hash 一律不可讀回。</p>\n\
         <h2>Personal Access Tokens</h2>\n\
         <table>\n<thead><tr><th>prefix</th><th>名稱</th><th>所屬</th><th>到期</th><th>last-used</th><th>建立</th><th></th></tr></thead>\n<tbody>\n{pat_rows}</tbody>\n</table>\n\
         <h2>裝置憑證 Families</h2>\n\
         <table>\n<thead><tr><th>所屬</th><th>來源</th><th>建立</th><th>最近 refresh</th><th></th></tr></thead>\n<tbody>\n{fam_rows}</tbody>\n</table>\n"
    );
    web::page("憑證監督", &body)
}

// --- /admin audit page (決策 3) ---

/// The audit page's `?page=` query.
#[derive(Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    page: u32,
}

/// How many audit records a page shows.
const AUDIT_PAGE: u32 = 50;

/// `GET /admin/audit?page=N` — the read-only, newest-first, paginated audit view.
pub async fn audit_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<PageQuery>,
) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_audit(&state, q.page)).into_response()
}

/// Render the audit view. There are no edit or delete controls — the log is
/// append-only and this page only reads it.
fn render_audit(state: &AppState, page: u32) -> String {
    let offset = page.saturating_mul(AUDIT_PAGE);
    let entries = state.identity.list_audit(AUDIT_PAGE, offset).unwrap_or_default();
    let mut rows = String::new();
    for e in &entries {
        rows.push_str(&format!(
            "<tr><td>{time}</td><td>{source}</td><td>{actor}</td><td>{action}</td><td>{subject}</td></tr>\n",
            time = fmt_ts(e.created_at),
            source = web::escape(&e.source),
            actor = web::escape(&e.actor_id),
            action = web::escape(&e.action),
            subject = web::escape(&e.subject),
        ));
    }
    let prev = if page > 0 {
        format!("<a href=\"/admin/audit?page={}\">← 上一頁</a> ", page - 1)
    } else {
        String::new()
    };
    // A full page hints there may be more; the last page shows no next link.
    let next = if entries.len() as u32 == AUDIT_PAGE {
        format!("<a href=\"/admin/audit?page={}\">下一頁 →</a>", page + 1)
    } else {
        String::new()
    };
    let body = format!(
        "<h1>Audit log</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <p>唯讀，倒序（最新在上）。</p>\n\
         <table>\n<thead><tr><th>時間 (UTC)</th><th>來源</th><th>操作者</th><th>動作</th><th>對象</th></tr></thead>\n<tbody>\n{rows}</tbody>\n</table>\n\
         <p>{prev}{next}</p>\n"
    );
    web::page("Audit log", &body)
}

/// A trimmed value, or `None` when empty.
fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

// --- /admin system status (決策 5) ---

/// One scope's outbox backlog: newest sequence minus the acked cursor. `backlog`
/// is `None` when the store could not be read (e.g. it is unavailable).
#[derive(Serialize)]
struct ScopeBacklog {
    project: String,
    repo: String,
    backlog: Option<u64>,
}

/// The read-only system aggregation (決策 5): engine/API versions, the store
/// manifest and live health, the identity schema version, and per-scope outbox
/// backlog. Every field comes from an existing interface; nothing here is a new
/// probe. A store failure surfaces as a health error, not a 500.
#[derive(Serialize)]
pub struct SystemInfo {
    engine_version: String,
    api_version: String,
    identity_schema_version: Option<u32>,
    store_driver: String,
    store_contract_version: u32,
    store_level: String,
    store_capabilities: Vec<String>,
    store_healthy: bool,
    store_health_error: Option<String>,
    outbox_backlogs: Vec<ScopeBacklog>,
}

/// One scope's backlog: newest outbox sequence minus the acked cursor.
fn scope_backlog(store: &SharedStore, scope: &Scope) -> Result<u64, StoreError> {
    let acked = store.outbox_acked(scope)?.0;
    let tail = store.read_outbox(scope, OutboxCursor(acked))?;
    let newest = tail.last().map(|e| e.seq).unwrap_or(acked);
    Ok(newest.saturating_sub(acked))
}

/// Aggregate the system view. The manifest, versions and identity schema are read
/// even when the store backend is down; a store health failure and unreadable
/// backlogs are reported as such rather than failing the whole view — the
/// identity-side management面 stays up (決策 5).
fn gather_system(state: &AppState) -> SystemInfo {
    let manifest = state.store.manifest();
    let (store_healthy, store_health_error) = match state.store.health() {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    let mut outbox_backlogs = Vec::new();
    for project in state.identity.list_projects().unwrap_or_default() {
        for repo in state.identity.list_repos(&project.key).unwrap_or_default() {
            let scope = Scope::new(ProjectId::new(project.key.clone()), RepoId::new(repo.key.clone()));
            outbox_backlogs.push(ScopeBacklog {
                project: project.key.clone(),
                repo: repo.key,
                backlog: scope_backlog(&state.store, &scope).ok(),
            });
        }
    }
    SystemInfo {
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        api_version: API_VERSION.to_string(),
        identity_schema_version: state.identity.schema_version().ok(),
        store_driver: manifest.driver,
        store_contract_version: manifest.contract_version,
        store_level: manifest.level.as_str().to_string(),
        store_capabilities: manifest.capabilities.iter().map(|c| c.as_str().to_string()).collect(),
        store_healthy,
        store_health_error,
        outbox_backlogs,
    }
}

/// `GET /api/speclink/v1/admin/system` — the system aggregation as JSON.
pub async fn api_system(State(state): State<AppState>, _admin: AdminApi) -> Response {
    Json(gather_system(&state)).into_response()
}

/// `GET /admin/system` — the system status page.
pub async fn system_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_system(&gather_system(&state))).into_response()
}

// --- data operations (决策 5): scope export download ---

/// `GET /admin/data` — the data-operations page: a scope export-download link per
/// registered scope.
pub async fn data_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_admin(&state, &headers) {
        return refused;
    }
    Html(render_data_page(&state)).into_response()
}

/// Render the data-operations page: recent backup info, one export-download link
/// per registered scope, and the store-migration trigger (决策 5).
fn render_data_page(state: &AppState) -> String {
    let mut scope_rows = String::new();
    for project in state.identity.list_projects().unwrap_or_default() {
        for repo in state.identity.list_repos(&project.key).unwrap_or_default() {
            scope_rows.push_str(&format!(
                "<tr><td>{p}</td><td>{r}</td>\
                 <td><a href=\"/admin/data/export/{p}/{r}\">下載 export bundle</a></td></tr>\n",
                p = web::escape(&project.key),
                r = web::escape(&repo.key),
            ));
        }
    }

    // Most recent backup/verify summary, if any has been recorded.
    let backup_info = match state.identity.latest_backup() {
        Ok(Some(rec)) => format!(
            "<ul>\n<li>類型：{kind}</li>\n<li>建立時間：{created}</li>\n\
             <li>格式版本：{fmt}</li>\n<li>摘要：{detail}</li>\n\
             <li>結果：{result}</li>\n</ul>\n",
            kind = web::escape(&rec.kind),
            created = web::escape(&rec.created_at.to_rfc3339()),
            fmt = rec.format_version,
            detail = web::escape(&rec.detail),
            result = if rec.ok { "通過" } else { "失敗" },
        ),
        _ => "<p>尚無備份記錄。</p>\n".to_string(),
    };

    let body = format!(
        "<h1>資料操作</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <h2>最近備份資訊</h2>\n{backup_info}\
         <h2>Scope export 下載</h2>\n\
         <table>\n<thead><tr><th>Project</th><th>Repo</th><th></th></tr></thead>\n\
         <tbody>\n{scope_rows}</tbody>\n</table>\n\
         <h2>Store 遷移</h2>\n\
         <p>觸發 TeamStore migrate（前置 health 檢查通過才執行）。</p>\n\
         <form method=\"post\" action=\"/admin/data/migrate\">\n\
         <button type=\"submit\">觸發遷移</button>\n</form>\n"
    );
    web::page("資料操作", &body)
}

/// `POST /admin/data/migrate` — trigger a store migration to the current contract
/// version (决策 5). A pre-flight health check must pass first: an unhealthy store
/// is refused with the reason and no `store-migrated` audit is written. A
/// successful migration records `store-migrated`.
pub async fn web_migrate_store(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let actor = match guard_web(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    // Pre-flight: an unhealthy backend is not migrated, and nothing is recorded.
    if let Err(e) = state.store.health() {
        return (
            StatusCode::CONFLICT,
            Html(web::page(
                "遷移未執行",
                &format!("<h1>Store health 檢查未通過，遷移未執行</h1>\n<p>{}</p>\n", web::escape(&e.to_string())),
            )),
        )
            .into_response();
    }
    match state.store.migrate(CONTRACT_VERSION) {
        Ok(()) => {
            let _ = state.identity.record_audit(
                &actor,
                AuditAction::StoreMigrated,
                &format!("contract version {CONTRACT_VERSION}"),
            );
            Redirect::to("/admin/data").into_response()
        }
        Err(e) => (
            StatusCode::CONFLICT,
            Html(web::page(
                "遷移失敗",
                &format!("<h1>遷移失敗</h1>\n<p>{}</p>\n", web::escape(&e.to_string())),
            )),
        )
            .into_response(),
    }
}

/// `GET /admin/data/export/{project}/{repo}` — download a scope's export bundle
/// (决策 5). Admin-gated; an unregistered scope is 404; a scope-exported audit is
/// recorded. The download reuses the backup bundle shape, so it passes the same
/// structure and digest verification a backup member does.
pub async fn web_export_scope(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project, repo)): Path<(String, String)>,
) -> Response {
    let (_user, actor) = match require_admin(&state, &headers) {
        Ok(admin) => admin,
        Err(refused) => return refused,
    };
    // Export on an unknown scope would silently yield an empty bundle, so gate on
    // the registry and 404 anything unregistered.
    if !scope_is_registered(&state, &project, &repo) {
        return (StatusCode::NOT_FOUND, Html(web::page("找不到", "<h1>找不到該 scope</h1>\n")))
            .into_response();
    }
    let scope = Scope::new(ProjectId::new(&project), RepoId::new(&repo));
    let bytes = match crate::backup::export_bundle_json(state.store.as_ref(), &scope) {
        Ok(bytes) => bytes,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let _ =
        state.identity.record_audit(&actor, AuditAction::ScopeExported, &format!("{project}/{repo}"));
    let filename = format!("{project}__{repo}.bundle.json");
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/json".to_string()),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        bytes,
    )
        .into_response()
}

/// Whether `(project, repo)` names a registered scope.
fn scope_is_registered(state: &AppState, project: &str, repo: &str) -> bool {
    if !matches!(state.identity.get_project(project), Ok(Some(_))) {
        return false;
    }
    state
        .identity
        .list_repos(project)
        .map(|repos| repos.iter().any(|r| r.key == repo))
        .unwrap_or(false)
}

/// Render the system status page.
fn render_system(info: &SystemInfo) -> String {
    let health = match &info.store_health_error {
        None => "正常".to_string(),
        Some(e) => format!("異常：{}", web::escape(e)),
    };
    let schema = info
        .identity_schema_version
        .map(|v| v.to_string())
        .unwrap_or_else(|| "未知".to_string());
    let mut backlog_rows = String::new();
    for b in &info.outbox_backlogs {
        backlog_rows.push_str(&format!(
            "<tr><td>{project}</td><td>{repo}</td><td>{backlog}</td></tr>\n",
            project = web::escape(&b.project),
            repo = web::escape(&b.repo),
            backlog = b.backlog.map(|n| n.to_string()).unwrap_or_else(|| "無法讀取".to_string()),
        ));
    }
    let body = format!(
        "<h1>系統狀態</h1>\n<p><a href=\"/admin\">← 管理面</a></p>\n\
         <ul>\n\
         <li>Engine 版本：{engine}</li>\n\
         <li>API 版本：{api}</li>\n\
         <li>Identity schema 版本：{schema}</li>\n\
         <li>Store driver：{driver}</li>\n\
         <li>Store contract 版本：{contract}</li>\n\
         <li>Store 能力等級：{level}</li>\n\
         <li>Store 能力：{caps}</li>\n\
         <li>Store 健康檢查：{health}</li>\n\
         </ul>\n\
         <h2>各 scope outbox 積壓</h2>\n\
         <table>\n<thead><tr><th>Project</th><th>Repo</th><th>積壓</th></tr></thead>\n<tbody>\n{backlog_rows}</tbody>\n</table>\n",
        engine = web::escape(&info.engine_version),
        api = web::escape(&info.api_version),
        driver = web::escape(&info.store_driver),
        contract = info.store_contract_version,
        level = web::escape(&info.store_level),
        caps = web::escape(&info.store_capabilities.join(", ")),
    );
    web::page("系統狀態", &body)
}

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
use crate::identity::{
    DeviceFamily, IdentityError, MembershipRole, NewInvitation, Pat, Project, Repo, User,
};
use crate::state::{AppState, SharedStore};
use crate::web;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use speclink_protocol::API_VERSION;
use speclink_store::{OutboxCursor, ProjectId, RepoId, Scope, StoreError, CONTRACT_VERSION};

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

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. bearer token → user, resolved per-request against the identity store
        //    (決策 1): `spk_at_` is a device access token, anything else a PAT,
        //    into the same check-list (hash-match, unrevoked, unexpired, owning
        //    user active). No cache means suspension and revocation are immediate.
        let token = bearer_token(&parts.headers)
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
            let sent = version
                .map(|v| format!(", client sent '{v}'"))
                .unwrap_or_default();
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
pub(crate) fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(User, AuditActor), Response> {
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
        axum::response::Html(web::page(
            "拒絕存取",
            "<h1>需要管理員權限</h1>\n<p>你的帳號沒有管理權限。</p>\n",
        )),
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
    name.as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or(key)
}

/// `POST /api/speclink/v1/admin/users/{id}/suspend`
pub async fn api_suspend_user(
    State(state): State<AppState>,
    admin: AdminApi,
    Path(user_id): Path<String>,
) -> Result<Response, ApiError> {
    state
        .identity
        .admin_set_user_suspended(&admin.actor, &user_id, true)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/users/{id}/reactivate`
pub async fn api_reactivate_user(
    State(state): State<AppState>,
    admin: AdminApi,
    Path(user_id): Path<String>,
) -> Result<Response, ApiError> {
    state
        .identity
        .admin_set_user_suspended(&admin.actor, &user_id, false)?;
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
    state
        .identity
        .admin_create_project(&admin.actor, &body.key, &name)?;
    Ok(Json(Ack {}).into_response())
}

/// `POST /api/speclink/v1/admin/repos`
pub async fn api_create_repo(
    State(state): State<AppState>,
    admin: AdminApi,
    Json(body): Json<CreateRepoBody>,
) -> Result<Response, ApiError> {
    let name = name_or_key(&body.name, &body.key).to_string();
    state
        .identity
        .admin_create_repo(&admin.actor, &body.project_key, &body.key, &name)?;
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

// --- /admin system status (決策 5) ---

/// One scope's outbox backlog: newest sequence minus the acked cursor. `backlog`
/// is `None` when the store could not be read (e.g. it is unavailable).
#[derive(Serialize)]
pub(crate) struct ScopeBacklog {
    pub(crate) project: String,
    pub(crate) repo: String,
    pub(crate) backlog: Option<u64>,
}

/// The read-only system aggregation (決策 5): engine/API versions, the store
/// manifest and live health, the identity schema version, and per-scope outbox
/// backlog. Every field comes from an existing interface; nothing here is a new
/// probe. A store failure surfaces as a health error, not a 500.
#[derive(Serialize)]
pub struct SystemInfo {
    pub(crate) engine_version: String,
    pub(crate) api_version: String,
    pub(crate) identity_schema_version: Option<u32>,
    pub(crate) store_driver: String,
    pub(crate) store_contract_version: u32,
    pub(crate) store_level: String,
    pub(crate) store_capabilities: Vec<String>,
    pub(crate) store_healthy: bool,
    pub(crate) store_health_error: Option<String>,
    pub(crate) outbox_backlogs: Vec<ScopeBacklog>,
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
            let scope = Scope::new(
                ProjectId::new(project.key.clone()),
                RepoId::new(repo.key.clone()),
            );
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
        store_capabilities: manifest
            .capabilities
            .iter()
            .map(|c| c.as_str().to_string())
            .collect(),
        store_healthy,
        store_health_error,
        outbox_backlogs,
    }
}

/// `GET /api/speclink/v1/admin/system` — the system aggregation as JSON.
pub async fn api_system(State(state): State<AppState>, _admin: AdminApi) -> Response {
    Json(gather_system(&state)).into_response()
}

// --- data operations (决策 5): scope export download ---

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
        return (
            StatusCode::NOT_FOUND,
            Html(web::page("找不到", "<h1>找不到該 scope</h1>\n")),
        )
            .into_response();
    }
    let scope = Scope::new(ProjectId::new(&project), RepoId::new(&repo));
    let bytes = match crate::backup::export_bundle_json(state.store.as_ref(), &scope) {
        Ok(bytes) => bytes,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let _ = state.identity.record_audit(
        &actor,
        AuditAction::ScopeExported,
        &format!("{project}/{repo}"),
    );
    let filename = format!("{project}__{repo}.bundle.json");
    (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/json".to_string(),
            ),
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

// --- browser JSON admin API (server-admin spec, D4)：/api/speclink/v1/web/admin ---
//
// SPA 專用的 session-cookie admin API：先驗同源（mutation）、active session 與 admin
// 旗標；未登入 401、非 admin 403 `permission_denied`（不新增 wire reason）。七個獨立
// view model 只回頁面所需 metadata（絕不含 hash／plaintext／refresh credential／token），
// mutation 呼叫與 bearer API／CLI 相同的單點 `admin_*` 函式，audit source `web`。

/// The browser admin gate: an active session whose user carries the admin flag.
/// Returns the admin and a web-source audit actor, or a JSON 401/403.
fn require_web_admin(state: &AppState, headers: &HeaderMap) -> Result<(User, AuditActor), Response> {
    match web::current_user(state, headers) {
        Some(user) if user.admin => {
            let actor = AuditActor::user(user.id.clone(), AuditSource::Web);
            Ok((user, actor))
        }
        Some(_) => Err(web::web_err(
            StatusCode::FORBIDDEN,
            "permission_denied",
            "沒有管理權限",
        )),
        None => Err(web::web_err(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "請先登入",
        )),
    }
}

/// A mutation gate: same-origin first (before the permission decision), then the
/// admin session. Returns the web-source audit actor.
fn web_admin_mutation(state: &AppState, headers: &HeaderMap) -> Result<AuditActor, Response> {
    if !web::is_same_origin(headers, &state.config.public_url) {
        return Err(web::web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        ));
    }
    let (_user, actor) = require_web_admin(state, headers)?;
    Ok(actor)
}

/// A mutation acknowledgement (no view model).
#[derive(Serialize)]
struct WebAck {
    ok: bool,
}

/// Map a single-point admin action's result to a JSON response: a refused guard
/// (e.g. the last active admin) is 409 with its reason, an unknown subject 404,
/// a duplicate key 409, anything else a 500 without internal detail.
fn admin_result(result: Result<(), IdentityError>) -> Response {
    match result {
        Ok(()) => web::web_ok(WebAck { ok: true }),
        Err(IdentityError::Refused(msg)) => web::web_err(StatusCode::CONFLICT, "refused", msg),
        Err(IdentityError::NotFound(msg)) => web::web_err(StatusCode::NOT_FOUND, "not_found", msg),
        Err(IdentityError::Duplicate(msg)) => web::web_err(StatusCode::CONFLICT, "duplicate", msg),
        Err(_) => web::web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
    }
}

fn iso_opt(dt: Option<DateTime<Utc>>) -> Option<String> {
    dt.map(|t| t.to_rfc3339())
}

// --- view models (read; secrets never appear) ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebConnection {
    public_url: String,
    project_key: String,
    repo_key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebOverview {
    active_users: usize,
    suspended_users: usize,
    projects: usize,
    repos: usize,
    active_credentials: usize,
    store_healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    store_health_error: Option<String>,
    identity_schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    connection: Option<WebConnection>,
}

/// `GET /admin/overview` — the low-cost summary the management nav needs.
pub async fn web_admin_overview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let users = state.identity.list_users().unwrap_or_default();
    let projects = state.identity.list_projects().unwrap_or_default();
    let repos: usize = projects
        .iter()
        .map(|p| state.identity.list_repos(&p.key).map(|r| r.len()).unwrap_or(0))
        .sum();
    let active_pats = state
        .identity
        .list_all_pats()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.revoked_at.is_none())
        .count();
    let active_families = state
        .identity
        .list_all_device_families()
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, f)| f.revoked_at.is_none())
        .count();
    let (store_healthy, store_health_error) = match state.store.health() {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    let connection = projects.first().map(|p| WebConnection {
        public_url: state.config.public_url.clone(),
        project_key: p.key.clone(),
        repo_key: state
            .identity
            .list_repos(&p.key)
            .ok()
            .and_then(|r| r.first().map(|r| r.key.clone()))
            .unwrap_or_default(),
    });
    web::web_ok(WebOverview {
        active_users: users.iter().filter(|u| u.active).count(),
        suspended_users: users.iter().filter(|u| !u.active).count(),
        projects: projects.len(),
        repos,
        active_credentials: active_pats + active_families,
        store_healthy,
        store_health_error,
        identity_schema_version: state.identity.schema_version().unwrap_or(0),
        connection,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebMembership {
    project_key: String,
    role: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebUser {
    id: String,
    email: String,
    display: String,
    admin: bool,
    active: bool,
    memberships: Vec<WebMembership>,
    can_suspend: bool,
    can_remove_admin: bool,
}

#[derive(Serialize)]
struct WebUsers {
    users: Vec<WebUser>,
}

/// `GET /admin/users` — the users view model with per-row action eligibility.
pub async fn web_admin_users(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let users = state.identity.list_users().unwrap_or_default();
    let active_admins = users.iter().filter(|u| u.active && u.admin).count();
    let rows = users
        .iter()
        .map(|u| {
            let last_active_admin = u.active && u.admin && active_admins <= 1;
            let memberships = state
                .identity
                .list_memberships(&u.id)
                .unwrap_or_default()
                .into_iter()
                .map(|key| {
                    let role = state
                        .identity
                        .membership_role(&u.id, &key)
                        .ok()
                        .flatten()
                        .map(|r| r.as_str().to_string())
                        .unwrap_or_default();
                    WebMembership { project_key: key, role }
                })
                .collect();
            WebUser {
                id: u.id.clone(),
                email: u.email.clone(),
                display: u.display.clone(),
                admin: u.admin,
                active: u.active,
                memberships,
                can_suspend: u.active && !last_active_admin,
                can_remove_admin: u.admin && !last_active_admin,
            }
        })
        .collect();
    web::web_ok(WebUsers { users: rows })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebRepo {
    key: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebProject {
    key: String,
    name: String,
    repos: Vec<WebRepo>,
}

#[derive(Serialize)]
struct WebRegistry {
    projects: Vec<WebProject>,
}

/// `GET /admin/registry` — projects and their repos (keys are stable, no rename).
pub async fn web_admin_registry(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let projects = state
        .identity
        .list_projects()
        .unwrap_or_default()
        .into_iter()
        .map(|p: Project| {
            let repos = state
                .identity
                .list_repos(&p.key)
                .unwrap_or_default()
                .into_iter()
                .map(|r: Repo| WebRepo { key: r.key, name: r.name })
                .collect();
            WebProject { key: p.key, name: p.name, repos }
        })
        .collect();
    web::web_ok(WebRegistry { projects })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebPat {
    id: String,
    user_id: String,
    prefix: String,
    name: String,
    created_at: String,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    revoked_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebCredFamily {
    id: String,
    user_id: String,
    source: String,
    created_at: String,
    last_refresh_at: String,
    revoked_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebCredentials {
    pats: Vec<WebPat>,
    device_families: Vec<WebCredFamily>,
}

/// `GET /admin/credentials` — every credential's metadata (no secret value).
pub async fn web_admin_credentials(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let pats = state
        .identity
        .list_all_pats()
        .unwrap_or_default()
        .into_iter()
        .map(|p: Pat| WebPat {
            id: p.id,
            user_id: p.user_id,
            prefix: p.prefix,
            name: p.name,
            created_at: p.created_at.to_rfc3339(),
            expires_at: iso_opt(p.expires_at),
            last_used_at: iso_opt(p.last_used_at),
            revoked_at: iso_opt(p.revoked_at),
        })
        .collect();
    let device_families = state
        .identity
        .list_all_device_families()
        .unwrap_or_default()
        .into_iter()
        .map(|(user_id, f): (String, DeviceFamily)| WebCredFamily {
            id: f.id,
            user_id,
            source: f.source,
            created_at: f.created_at.to_rfc3339(),
            last_refresh_at: f.last_refresh_at.to_rfc3339(),
            revoked_at: iso_opt(f.revoked_at),
        })
        .collect();
    web::web_ok(WebCredentials { pats, device_families })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebScope {
    project: String,
    repo: String,
    export_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebData {
    scopes: Vec<WebScope>,
    store_healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    store_health_error: Option<String>,
}

/// `GET /admin/data` — the registered scopes (each with its export-download path)
/// and store health. Store failure degrades this view but keeps identity management up.
pub async fn web_admin_data(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let mut scopes = Vec::new();
    for project in state.identity.list_projects().unwrap_or_default() {
        for repo in state.identity.list_repos(&project.key).unwrap_or_default() {
            scopes.push(WebScope {
                export_path: format!("/admin/data/export/{}/{}", project.key, repo.key),
                project: project.key.clone(),
                repo: repo.key,
            });
        }
    }
    let (store_healthy, store_health_error) = match state.store.health() {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    web::web_ok(WebData { scopes, store_healthy, store_health_error })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebBacklog {
    project: String,
    repo: String,
    backlog: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebSystem {
    engine_version: String,
    api_version: String,
    identity_schema_version: Option<u32>,
    store_driver: String,
    store_contract_version: u32,
    store_level: String,
    store_capabilities: Vec<String>,
    store_healthy: bool,
    store_health_error: Option<String>,
    outbox_backlogs: Vec<WebBacklog>,
}

/// `GET /admin/system` — the system aggregation (camelCase). Reuses [`gather_system`].
pub async fn web_admin_system(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let info = gather_system(&state);
    web::web_ok(WebSystem {
        engine_version: info.engine_version,
        api_version: info.api_version,
        identity_schema_version: info.identity_schema_version,
        store_driver: info.store_driver,
        store_contract_version: info.store_contract_version,
        store_level: info.store_level,
        store_capabilities: info.store_capabilities,
        store_healthy: info.store_healthy,
        store_health_error: info.store_health_error,
        outbox_backlogs: info
            .outbox_backlogs
            .into_iter()
            .map(|b| WebBacklog { project: b.project, repo: b.repo, backlog: b.backlog })
            .collect(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebAuditEntry {
    id: String,
    actor_id: String,
    action: String,
    subject: String,
    source: String,
    created_at: String,
}

#[derive(Serialize)]
struct WebAudit {
    entries: Vec<WebAuditEntry>,
}

/// `GET /admin/audit` — the newest-first management audit log page.
pub async fn web_admin_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Response {
    if let Err(refused) = require_web_admin(&state, &headers) {
        return refused;
    }
    let entries = state
        .identity
        .list_audit(query.limit, query.offset)
        .unwrap_or_default()
        .into_iter()
        .map(|e| WebAuditEntry {
            id: e.id,
            actor_id: e.actor_id,
            action: e.action,
            subject: e.subject,
            source: e.source,
            created_at: e.created_at.to_rfc3339(),
        })
        .collect();
    web::web_ok(WebAudit { entries })
}

// --- mutations (源 web; each calls the single-point admin_* fn) ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebInviteBody {
    email: String,
    display: String,
    #[serde(default)]
    memberships: Vec<String>,
    #[serde(default)]
    admin: bool,
}

#[derive(Serialize)]
struct WebInviteResult {
    token: String,
}

/// `POST /admin/users/invite` — mint an invitation via the single-point action.
pub async fn web_admin_invite(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebInviteBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let email = body.email.trim();
    let display = body.display.trim();
    if email.is_empty() || display.is_empty() {
        return web::web_err(StatusCode::BAD_REQUEST, "validation_error", "email 與顯示名稱皆為必填");
    }
    match state.identity.admin_create_invitation(
        &actor,
        NewInvitation {
            email: email.to_string(),
            display: display.to_string(),
            memberships: body.memberships,
            admin: body.admin,
            expires_at: Utc::now() + Duration::days(7),
        },
    ) {
        Ok(token) => web::web_ok(WebInviteResult { token }),
        Err(IdentityError::Duplicate(msg)) => web::web_err(StatusCode::CONFLICT, "duplicate", msg),
        Err(_) => web::web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
    }
}

/// `POST /admin/users/{id}/suspend`
pub async fn web_admin_suspend(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Response {
    match web_admin_mutation(&state, &headers) {
        Ok(actor) => admin_result(state.identity.admin_set_user_suspended(&actor, &user_id, true)),
        Err(refused) => refused,
    }
}

/// `POST /admin/users/{id}/reactivate`
pub async fn web_admin_reactivate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Response {
    match web_admin_mutation(&state, &headers) {
        Ok(actor) => admin_result(state.identity.admin_set_user_suspended(&actor, &user_id, false)),
        Err(refused) => refused,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebMembershipBody {
    project_key: String,
    #[serde(default)]
    role: String,
    member: bool,
}

/// `POST /admin/users/{id}/membership`
pub async fn web_admin_membership(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    body: Result<Json<WebMembershipBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let role = if body.role.is_empty() {
        MembershipRole::default()
    } else {
        match MembershipRole::parse(&body.role) {
            Ok(role) => role,
            Err(_) => return web::web_err(StatusCode::BAD_REQUEST, "validation_error", "未知的角色"),
        }
    };
    admin_result(
        state
            .identity
            .admin_set_membership(&actor, &user_id, &body.project_key, role, body.member),
    )
}

#[derive(Deserialize)]
pub struct WebAdminFlagBody {
    admin: bool,
}

/// `POST /admin/users/{id}/admin-flag`
pub async fn web_admin_flag(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
    body: Result<Json<WebAdminFlagBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    admin_result(state.identity.admin_set_admin_flag(&actor, &user_id, body.admin))
}

#[derive(Deserialize)]
pub struct WebCreateProjectBody {
    key: String,
    #[serde(default)]
    name: Option<String>,
}

/// `POST /admin/registry/projects`
pub async fn web_admin_create_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebCreateProjectBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let key = body.key.trim();
    if key.is_empty() {
        return web::web_err(StatusCode::BAD_REQUEST, "validation_error", "project key 為必填");
    }
    let name = name_or_key(&body.name, key).to_string();
    admin_result(state.identity.admin_create_project(&actor, key, &name))
}

#[derive(Deserialize)]
pub struct WebRenameBody {
    name: String,
}

/// `POST /admin/registry/projects/{key}/rename` — the display name only; the key
/// is the stable identifier and has no change interface.
pub async fn web_admin_rename_project(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    body: Result<Json<WebRenameBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    admin_result(state.identity.admin_rename_project(&actor, &key, body.name.trim()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebCreateRepoBody {
    project_key: String,
    key: String,
    #[serde(default)]
    name: Option<String>,
}

/// `POST /admin/registry/repos`
pub async fn web_admin_create_repo(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebCreateRepoBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let key = body.key.trim();
    if key.is_empty() || body.project_key.trim().is_empty() {
        return web::web_err(StatusCode::BAD_REQUEST, "validation_error", "project 與 repo key 皆為必填");
    }
    let name = name_or_key(&body.name, key).to_string();
    admin_result(
        state
            .identity
            .admin_create_repo(&actor, body.project_key.trim(), key, &name),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebRenameRepoBody {
    project_key: String,
    key: String,
    name: String,
}

/// `POST /admin/registry/repos/rename` — the repo display name only; keys stable.
pub async fn web_admin_rename_repo(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebRenameRepoBody>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web::web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    admin_result(
        state
            .identity
            .admin_rename_repo(&actor, body.project_key.trim(), body.key.trim(), body.name.trim()),
    )
}

/// `POST /admin/credentials/tokens/{id}/revoke`
pub async fn web_admin_revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(pat_id): Path<String>,
) -> Response {
    match web_admin_mutation(&state, &headers) {
        Ok(actor) => admin_result(state.identity.admin_revoke_pat(&actor, &pat_id)),
        Err(refused) => refused,
    }
}

/// `POST /admin/credentials/families/{id}/revoke`
pub async fn web_admin_revoke_family(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(family_id): Path<String>,
) -> Response {
    match web_admin_mutation(&state, &headers) {
        Ok(actor) => admin_result(state.identity.admin_revoke_family(&actor, &family_id)),
        Err(refused) => refused,
    }
}

/// `POST /admin/data/migrate` — migrate the store to the current contract version.
/// An unhealthy backend is not migrated (409) and nothing is recorded.
pub async fn web_admin_migrate(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let actor = match web_admin_mutation(&state, &headers) {
        Ok(actor) => actor,
        Err(refused) => return refused,
    };
    if let Err(e) = state.store.health() {
        return web::web_err(
            StatusCode::CONFLICT,
            "store_unhealthy",
            format!("Store health 檢查未通過，遷移未執行：{e}"),
        );
    }
    match state.store.migrate(CONTRACT_VERSION) {
        Ok(()) => {
            let _ = state.identity.record_audit(
                &actor,
                AuditAction::StoreMigrated,
                &format!("contract version {CONTRACT_VERSION}"),
            );
            web::web_ok(WebAck { ok: true })
        }
        Err(e) => web::web_err(StatusCode::CONFLICT, "migrate_failed", e.to_string()),
    }
}

/// The browser admin API sub-router, merged into the `/api/speclink/v1/web` nest.
/// Every handler runs the session + admin gate; mutations additionally verify
/// same-origin before the permission decision.
pub fn web_router() -> Router<AppState> {
    Router::new()
        .route("/admin/overview", get(web_admin_overview))
        .route("/admin/users", get(web_admin_users))
        .route("/admin/users/invite", post(web_admin_invite))
        .route("/admin/users/{id}/suspend", post(web_admin_suspend))
        .route("/admin/users/{id}/reactivate", post(web_admin_reactivate))
        .route("/admin/users/{id}/membership", post(web_admin_membership))
        .route("/admin/users/{id}/admin-flag", post(web_admin_flag))
        .route("/admin/registry", get(web_admin_registry))
        .route("/admin/registry/projects", post(web_admin_create_project))
        .route("/admin/registry/projects/{key}/rename", post(web_admin_rename_project))
        .route("/admin/registry/repos", post(web_admin_create_repo))
        .route("/admin/registry/repos/rename", post(web_admin_rename_repo))
        .route("/admin/credentials", get(web_admin_credentials))
        .route("/admin/credentials/tokens/{id}/revoke", post(web_admin_revoke_token))
        .route("/admin/credentials/families/{id}/revoke", post(web_admin_revoke_family))
        .route("/admin/data", get(web_admin_data))
        .route("/admin/data/migrate", post(web_admin_migrate))
        .route("/admin/system", get(web_admin_system))
        .route("/admin/audit", get(web_admin_audit))
}

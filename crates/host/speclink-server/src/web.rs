//! The minimal web entry (決策 4): server-rendered HTML forms embedded in the
//! binary, no JS framework and no external resources. This knife starts with the
//! invitation acceptance page; login, logout and the account page are layered on
//! in the following tasks.

use crate::identity::{DeviceFamily, IdentityError, Pat, SessionInfo, User};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The session cookie name.
const SESSION_COOKIE: &str = "speclink_session";

/// How long a session lives.
fn session_ttl() -> Duration {
    Duration::days(7)
}

/// Accept only the exact `XXXX-XXXX` alphabet generated for device user codes.
/// The validated characters are URL-safe, so redirects can use a fixed path
/// without accepting any caller-controlled destination.
fn validated_user_code(input: &str) -> Option<&str> {
    let bytes = input.as_bytes();
    let valid = bytes.len() == 9
        && bytes[4] == b'-'
        && bytes.iter().enumerate().all(|(index, byte)| {
            index == 4
                || matches!(
                    byte,
                    b'A'..=b'H' | b'J'..=b'K' | b'M'..=b'N' | b'P'..=b'Z' | b'2'..=b'9'
                )
        });
    valid.then_some(input)
}

fn activation_location(user_code: &str) -> String {
    format!("/activate?user_code={user_code}")
}

/// Parse the optional expiry field. Empty means a permanent token; a date is
/// interpreted as the end of that day, UTC.
fn parse_expiry(input: &str) -> Result<Option<chrono::DateTime<chrono::Utc>>, ()> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let date = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").map_err(|_| ())?;
    let naive = date.and_hms_opt(23, 59, 59).ok_or(())?;
    Ok(Some(chrono::DateTime::from_naive_utc_and_offset(
        naive,
        chrono::Utc,
    )))
}

// --- session cookie + origin (決策 4) ---

/// Attach a fresh session cookie to a response.
pub(crate) fn with_session_cookie(mut resp: Response, session: &str) -> Response {
    let cookie = format!("{SESSION_COOKIE}={session}; HttpOnly; Secure; SameSite=Strict; Path=/");
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, value);
    }
    resp
}

/// Clear the session cookie on a response.
fn with_cleared_cookie(mut resp: Response) -> Response {
    let cookie = format!("{SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0");
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        resp.headers_mut().insert(SET_COOKIE, value);
    }
    resp
}

/// Read the session id from the request's Cookie header.
fn read_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for pair in cookies.split(';') {
        if let Some(value) = pair.trim().strip_prefix(&format!("{SESSION_COOKIE}=")) {
            return Some(value.to_string());
        }
    }
    None
}

/// The authenticated user for this request, if the session cookie is live.
pub(crate) fn current_user(state: &AppState, headers: &HeaderMap) -> Option<User> {
    let session = read_session_cookie(headers)?;
    state.identity.authenticate_session(&session).ok().flatten()
}

/// True if a change-making request is same-origin with `public_url`, or carries
/// neither Origin nor Referer (a non-browser client is not a CSRF vector). A
/// present-but-foreign origin is not same-origin.
pub(crate) fn is_same_origin(headers: &HeaderMap, public_url: &str) -> bool {
    match header_str(headers, "origin").or_else(|| header_str(headers, "referer")) {
        None => true,
        Some(value) => origin_of(&value) == origin_of(public_url),
    }
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name)?.to_str().ok().map(str::to_string)
}

/// The `scheme://authority` origin of a URL, dropping any path.
fn origin_of(url: &str) -> &str {
    match url.split_once("://") {
        Some((scheme, rest)) => {
            let authority = rest.split('/').next().unwrap_or(rest);
            &url[..scheme.len() + 3 + authority.len()]
        }
        None => url,
    }
}

// --- browser JSON API (決策 D2／D3)：/api/speclink/v1/web ---
//
// bundled SPA 專用的 same-origin、session-cookie API。成功回 `{data: T}`、失敗回
// `{error: {code, message, fieldErrors?}}`，欄位 camelCase。mutation 先驗同源再解析
// session。登入 destination 由 Server 裁決：有效 device userCode → 通過白名單的
// returnTo → 角色 home；一般成員的 `/admin` destination 回 403 不降級。

/// `/api/speclink/v1/web` 下的 browser session／invite routes。setup routes 由
/// [`crate::setup::web_router`] 併入（app.rs）。
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/session", get(api_session))
        .route("/login", post(api_login))
        .route("/logout", post(api_logout))
        .route(
            "/invite/{token}",
            get(api_invite_summary).post(api_invite_accept),
        )
        .route("/account", get(api_account))
        .route("/account/tokens", post(api_create_pat))
        .route("/account/tokens/{id}/revoke", post(api_revoke_pat))
        .route("/account/devices/{id}/revoke", post(api_revoke_device))
        .route("/activate", post(api_activate))
}

#[derive(Serialize)]
struct DataEnvelope<T: Serialize> {
    data: T,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: WebError,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WebError {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_errors: Option<BTreeMap<String, String>>,
}

/// 200 `{data}`.
pub(crate) fn web_ok<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(DataEnvelope { data })).into_response()
}

/// A `{error}` envelope at `status`.
pub(crate) fn web_err(status: StatusCode, code: &'static str, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorEnvelope {
            error: WebError {
                code,
                message: message.into(),
                field_errors: None,
            },
        }),
    )
        .into_response()
}

/// A `{error}` envelope carrying one `fieldErrors` entry, so the SPA can place the
/// message next to the offending form field.
pub(crate) fn web_field_err(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
    field: &str,
    field_message: impl Into<String>,
) -> Response {
    let mut fields = BTreeMap::new();
    fields.insert(field.to_string(), field_message.into());
    (
        status,
        Json(ErrorEnvelope {
            error: WebError {
                code,
                message: message.into(),
                field_errors: Some(fields),
            },
        }),
    )
        .into_response()
}

/// Open a Web session for a just-committed account, returning the plaintext cookie
/// value. A creation failure is a retryable recovery error (500) — a committed
/// account or setup is never presented as logged in when the session cannot open.
pub(crate) fn open_session(state: &AppState, user_id: &str) -> Result<String, Response> {
    state
        .identity
        .create_session(user_id, session_ttl())
        .map_err(|_| {
            web_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "無法建立 session，請重試登入",
            )
        })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionPayload {
    authenticated: bool,
    user: Option<UserPayload>,
    home: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserPayload {
    id: String,
    email: String,
    display: String,
    admin: bool,
}

impl UserPayload {
    fn of(user: &User) -> UserPayload {
        UserPayload {
            id: user.id.clone(),
            email: user.email.clone(),
            display: user.display.clone(),
            admin: user.admin,
        }
    }
}

/// 預設的角色 home。
fn role_home(admin: bool) -> &'static str {
    if admin {
        "/admin"
    } else {
        "/account"
    }
}

/// `GET /session` — 呼叫者身分與其角色 home。永不回錯：沒有或失效的 session 只是
/// `authenticated: false`。
pub async fn api_session(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match current_user(&state, &headers) {
        Some(user) => web_ok(SessionPayload {
            authenticated: true,
            user: Some(UserPayload::of(&user)),
            home: role_home(user.admin).to_string(),
        }),
        None => web_ok(SessionPayload {
            authenticated: false,
            user: None,
            home: "/login".to_string(),
        }),
    }
}

/// The browser login body (camelCase).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginBody {
    email: String,
    password: String,
    #[serde(default)]
    user_code: Option<String>,
    #[serde(default)]
    return_to: Option<String>,
}

/// The login success destination the SPA navigates to.
#[derive(Serialize)]
struct Destination {
    destination: String,
}

/// A whitelisted in-site return path and its first segment. Accepts only a
/// single leading `/`, no scheme/authority (rejecting `//…` and `/\…`), no `..`
/// traversal (so a first segment cannot be walked past the whitelist), with a
/// first segment of `account`, `activate` or `admin`.
fn classify_return_to(path: &str) -> Option<(&str, &str)> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.starts_with("/\\")
        || path.contains("..")
    {
        return None;
    }
    let first = path[1..]
        .split(|c| c == '/' || c == '\\' || c == '?' || c == '#')
        .next()
        .unwrap_or("");
    match first {
        "account" | "activate" | "admin" => Some((path, first)),
        _ => None,
    }
}

/// Adjudicate the post-login destination: a valid device `userCode`, then a
/// whitelisted `returnTo`, then the role home. A member's `/admin` destination
/// is refused (403), not downgraded.
fn compute_destination(
    user: &User,
    user_code: Option<&str>,
    return_to: Option<&str>,
) -> Result<String, Response> {
    if let Some(code) = user_code.and_then(validated_user_code) {
        return Ok(activation_location(code));
    }
    if let Some(path) = return_to.filter(|r| !r.is_empty()) {
        if let Some((safe, first)) = classify_return_to(path) {
            if first == "admin" && !user.admin {
                return Err(web_err(
                    StatusCode::FORBIDDEN,
                    "permission_denied",
                    "沒有管理權限",
                ));
            }
            return Ok(safe.to_string());
        }
        // 不安全的 returnTo 直接忽略——落到角色 home。
    }
    Ok(role_home(user.admin).to_string())
}

/// `POST /login` — 以 argon2 驗證 email＋密碼、建立 session 並回傳 Server 裁決的
/// destination。先檢查同源；失敗為統一 401，永不洩漏 email 是否存在。
pub async fn api_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<LoginBody>, JsonRejection>,
) -> Response {
    if !is_same_origin(&headers, &state.config.public_url) {
        return web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        );
    }
    let Ok(Json(body)) = body else {
        return web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    match state
        .identity
        .authenticate_password(&body.email, &body.password)
    {
        Ok(Some(user)) => {
            let destination = match compute_destination(
                &user,
                body.user_code.as_deref(),
                body.return_to.as_deref(),
            ) {
                Ok(destination) => destination,
                Err(refused) => return refused,
            };
            match state.identity.create_session(&user.id, session_ttl()) {
                Ok(session) => with_session_cookie(web_ok(Destination { destination }), &session),
                Err(_) => web_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "無法建立 session，請重試登入",
                ),
            }
        }
        Ok(None) => web_err(
            StatusCode::UNAUTHORIZED,
            "invalid_credentials",
            "帳號或密碼不正確",
        ),
        Err(_) => web_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "登入時發生錯誤",
        ),
    }
}

/// `POST /logout` — 撤銷 server 端 session 並清除 cookie。
pub async fn api_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if !is_same_origin(&headers, &state.config.public_url) {
        return web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        );
    }
    if let Some(session) = read_session_cookie(&headers) {
        let _ = state.identity.revoke_session(&session);
    }
    with_cleared_cookie(web_ok(Destination {
        destination: "/login".to_string(),
    }))
}

/// The non-secret invitation summary the set-password form needs.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InvitationSummary {
    email: String,
    display: String,
    admin: bool,
}

/// The browser invite acceptance body (camelCase).
#[derive(Deserialize)]
pub struct AcceptBody {
    password: String,
}

/// The single "邀請無效" JSON result for used, expired or unknown tokens — the
/// reason is never distinguished (mirrors the HTML flow's 404).
fn invalid_invitation_json() -> Response {
    web_err(StatusCode::NOT_FOUND, "invalid_invitation", "邀請無效")
}

/// `GET /api/speclink/v1/web/invite/{token}` — the non-secret summary for the
/// set-password form. A used, expired or unknown token is one indistinguishable 404.
pub async fn api_invite_summary(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Response {
    match state.identity.find_valid_invitation(&token) {
        Ok(Some(inv)) => web_ok(InvitationSummary {
            email: inv.email,
            display: inv.display,
            admin: inv.admin,
        }),
        Ok(None) => invalid_invitation_json(),
        Err(_) => web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
    }
}

/// `POST /api/speclink/v1/web/invite/{token}` — accept the invitation: same-origin,
/// atomically create the active user with the invited memberships and consume the
/// invitation, then open the user's Web session and return the Server-adjudicated
/// destination (admin → `/admin`, member → `/account`). A no-longer-valid token is
/// the same 404 and creates no session.
pub async fn api_invite_accept(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: HeaderMap,
    body: Result<Json<AcceptBody>, JsonRejection>,
) -> Response {
    if !is_same_origin(&headers, &state.config.public_url) {
        return web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        );
    }
    let Ok(Json(body)) = body else {
        return web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    if body.password.is_empty() {
        return web_field_err(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "請輸入密碼",
            "password",
            "請輸入密碼",
        );
    }
    match state.identity.accept_invitation(&token, &body.password) {
        Ok(user_id) => open_session_destination(&state, &user_id),
        Err(IdentityError::InvalidInvitation) => invalid_invitation_json(),
        Err(_) => web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
    }
}

/// Open the just-created user's session and return its role-home destination with
/// the cookie set. A session-creation failure yields the retryable recovery error.
fn open_session_destination(state: &AppState, user_id: &str) -> Response {
    let Some(user) = state.identity.get_user(user_id).ok().flatten() else {
        return web_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "無法建立 session，請重試登入",
        );
    };
    match open_session(state, user_id) {
        Ok(session) => with_session_cookie(
            web_ok(Destination {
                destination: role_home(user.admin).to_string(),
            }),
            &session,
        ),
        Err(recovery) => recovery,
    }
}

// --- browser JSON account + activation API (server-identity「帳號 browser API 保持
// 憑證祕密邊界」與 server-device-auth「核准頁 session 保護且明確確認」, D2／D3／D4) ---
//
// account read／mutation 與 activation decision 皆先驗同源與 active session。read
// payload 只回呈現與 eligibility 所需 metadata，絕不含 hash／refresh credential／
// password／可重播 session secret；PAT 明文只在建立回應出現一次。

/// The authenticated user for a browser API request, or a 401 recovery response.
fn require_user(state: &AppState, headers: &HeaderMap) -> Result<User, Response> {
    current_user(state, headers)
        .ok_or_else(|| web_err(StatusCode::UNAUTHORIZED, "unauthenticated", "請先登入"))
}

/// Refuse a change-making request whose Origin/Referer is foreign (403).
fn require_same_origin(headers: &HeaderMap, public_url: &str) -> Result<(), Response> {
    if is_same_origin(headers, public_url) {
        Ok(())
    } else {
        Err(web_err(
            StatusCode::FORBIDDEN,
            "same_origin_required",
            "跨來源請求被拒絕",
        ))
    }
}

/// A generic `{data:{ok:true}}` acknowledgement for a mutation with no view model.
#[derive(Serialize)]
struct OkPayload {
    ok: bool,
}

fn ok_ack() -> Response {
    web_ok(OkPayload { ok: true })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountPayload {
    user: UserPayload,
    memberships: Vec<MembershipPayload>,
    pats: Vec<PatPayload>,
    sessions: Vec<SessionMetaPayload>,
    device_families: Vec<DeviceFamilyPayload>,
}

/// One of the caller's own project memberships: the project's key, its display
/// name and the caller's role there. The registry is not exposed — only the
/// projects the caller belongs to appear, for an admin as for a member.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipPayload {
    project_key: String,
    project_name: String,
    role: String,
}

/// A PAT's non-secret metadata (prefix, not the plaintext or hash).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PatPayload {
    id: String,
    prefix: String,
    name: String,
    created_at: String,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    revoked_at: Option<String>,
}

impl PatPayload {
    fn of(p: &Pat) -> PatPayload {
        PatPayload {
            id: p.id.clone(),
            prefix: p.prefix.clone(),
            name: p.name.clone(),
            created_at: p.created_at.to_rfc3339(),
            expires_at: p.expires_at.map(|t| t.to_rfc3339()),
            last_used_at: p.last_used_at.map(|t| t.to_rfc3339()),
            revoked_at: p.revoked_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// A Web session's metadata (the id is a metadata id, never the cookie secret).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionMetaPayload {
    id: String,
    created_at: String,
    expires_at: String,
    revoked_at: Option<String>,
}

impl SessionMetaPayload {
    fn of(s: &SessionInfo) -> SessionMetaPayload {
        SessionMetaPayload {
            id: s.id.clone(),
            created_at: s.created_at.to_rfc3339(),
            expires_at: s.expires_at.to_rfc3339(),
            revoked_at: s.revoked_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// A device credential family's metadata (no refresh credential is ever included).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceFamilyPayload {
    id: String,
    source: String,
    created_at: String,
    last_refresh_at: String,
    revoked_at: Option<String>,
}

impl DeviceFamilyPayload {
    fn of(f: &DeviceFamily) -> DeviceFamilyPayload {
        DeviceFamilyPayload {
            id: f.id.clone(),
            source: f.source.clone(),
            created_at: f.created_at.to_rfc3339(),
            last_refresh_at: f.last_refresh_at.to_rfc3339(),
            revoked_at: f.revoked_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// The PAT-create response: the metadata plus the one-time plaintext.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PatCreatedPayload {
    pat: PatPayload,
    plaintext: String,
}

/// A user's project memberships as `(project key, role)` pairs — the single
/// query path both the account summary and the admin users view read from.
/// A membership whose role row is unreadable degrades to an empty role rather
/// than dropping the project from the list.
pub(crate) fn membership_roles(state: &AppState, user_id: &str) -> Vec<(String, String)> {
    state
        .identity
        .list_memberships(user_id)
        .unwrap_or_default()
        .into_iter()
        .map(|key| {
            let role = state
                .identity
                .membership_role(user_id, &key)
                .ok()
                .flatten()
                .map(|r| r.as_str().to_string())
                .unwrap_or_default();
            (key, role)
        })
        .collect()
}

/// `GET /account` — the caller's own user, own project memberships, PAT
/// metadata, Web sessions and device families. Session-protected; secrets never
/// appear.
pub async fn api_account(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match require_user(&state, &headers) {
        Ok(user) => user,
        Err(refused) => return refused,
    };
    let memberships = membership_roles(&state, &user.id)
        .into_iter()
        .map(|(project_key, role)| {
            let project_name = state
                .identity
                .get_project(&project_key)
                .ok()
                .flatten()
                .map(|p| p.name)
                .unwrap_or_default();
            MembershipPayload {
                project_key,
                project_name,
                role,
            }
        })
        .collect();
    let pats = state.identity.list_pats(&user.id).unwrap_or_default();
    let sessions = state.identity.list_sessions(&user.id).unwrap_or_default();
    let families = state
        .identity
        .list_device_families(&user.id)
        .unwrap_or_default();
    web_ok(AccountPayload {
        user: UserPayload::of(&user),
        memberships,
        pats: pats.iter().map(PatPayload::of).collect(),
        sessions: sessions.iter().map(SessionMetaPayload::of).collect(),
        device_families: families.iter().map(DeviceFamilyPayload::of).collect(),
    })
}

/// The create-PAT body (camelCase).
#[derive(Deserialize)]
pub struct CreatePatBody {
    name: String,
    #[serde(default)]
    expires: Option<String>,
}

/// `POST /account/tokens` — create a PAT for the logged-in user. The response is
/// the only place the plaintext appears.
pub async fn api_create_pat(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<CreatePatBody>, JsonRejection>,
) -> Response {
    if let Err(refused) = require_same_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let user = match require_user(&state, &headers) {
        Ok(user) => user,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let name = body.name.trim();
    if name.is_empty() {
        return web_field_err(
            StatusCode::BAD_REQUEST,
            "validation_error",
            "請輸入名稱",
            "name",
            "請輸入名稱",
        );
    }
    let expires_at = match parse_expiry(body.expires.as_deref().unwrap_or("")) {
        Ok(expires_at) => expires_at,
        Err(()) => {
            return web_field_err(
                StatusCode::BAD_REQUEST,
                "validation_error",
                "到期日格式須為 YYYY-MM-DD",
                "expires",
                "格式須為 YYYY-MM-DD",
            )
        }
    };
    match state.identity.create_pat(&user.id, name, expires_at) {
        Ok((pat, plaintext)) => web_ok(PatCreatedPayload {
            pat: PatPayload::of(&pat),
            plaintext,
        }),
        Err(_) => web_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "建立 PAT 時發生錯誤",
        ),
    }
}

/// `POST /account/tokens/{id}/revoke` — revoke one of the caller's own PATs.
/// Immediate for the next API request. Idempotent.
pub async fn api_revoke_pat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(pat_id): Path<String>,
) -> Response {
    if let Err(refused) = require_same_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let user = match require_user(&state, &headers) {
        Ok(user) => user,
        Err(refused) => return refused,
    };
    let _ = state.identity.revoke_pat(&user.id, &pat_id);
    ok_ack()
}

/// `POST /account/devices/{id}/revoke` — revoke one of the caller's own device
/// credential families; its access token and refresh credential die at once.
pub async fn api_revoke_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(family_id): Path<String>,
) -> Response {
    if let Err(refused) = require_same_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let user = match require_user(&state, &headers) {
        Ok(user) => user,
        Err(refused) => return refused,
    };
    let _ = state.identity.revoke_family(&user.id, &family_id);
    ok_ack()
}

/// The activation body: the user code and an optional decision.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateBody {
    #[serde(default)]
    user_code: String,
    #[serde(default)]
    action: Option<String>,
}

/// The activation outcome the SPA reflects.
#[derive(Serialize)]
struct ActivateStatus {
    status: &'static str,
}

/// The single invalid-code JSON result for unknown, used or expired user codes.
fn activate_invalid_json() -> Response {
    web_err(
        StatusCode::NOT_FOUND,
        "invalid_user_code",
        "這個裝置代碼無法使用",
    )
}

/// `POST /activate` — session-protected device approval. No `action` is the
/// explicit confirm step (checks the code without deciding); `approve`/`deny`
/// record the decision against the acting user. Same-origin; unknown, used and
/// expired codes are one indistinguishable 404. GET never queries state, so there
/// is no GET endpoint here.
pub async fn api_activate(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<ActivateBody>, JsonRejection>,
) -> Response {
    if let Err(refused) = require_same_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let user = match require_user(&state, &headers) {
        Ok(user) => user,
        Err(refused) => return refused,
    };
    let Ok(Json(body)) = body else {
        return web_err(StatusCode::BAD_REQUEST, "invalid_request", "請求格式不正確");
    };
    let user_code = body.user_code.trim();
    match body.action.as_deref() {
        Some("approve") => match state.identity.approve_device(user_code, &user.id) {
            Ok(true) => web_ok(ActivateStatus { status: "approved" }),
            Ok(false) => activate_invalid_json(),
            Err(_) => web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
        },
        Some("deny") => match state.identity.deny_device(user_code, &user.id) {
            Ok(true) => web_ok(ActivateStatus { status: "denied" }),
            Ok(false) => activate_invalid_json(),
            Err(_) => web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
        },
        // No decision yet: the explicit confirm step validates the code only.
        _ => match state.identity.device_is_pending(user_code) {
            Ok(true) => web_ok(ActivateStatus { status: "pending" }),
            Ok(false) => activate_invalid_json(),
            Err(_) => web_err(StatusCode::INTERNAL_SERVER_ERROR, "internal", "發生錯誤"),
        },
    }
}

// --- rendering (embedded, no external resources) ---

/// The full HTML document shell. A single inline stylesheet keeps the pages
/// self-contained (CSP can be strict).
pub(crate) fn page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"zh-Hant\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n</head>\n<body>\n<main>\n{body}\n</main>\n</body>\n</html>\n"
    )
}

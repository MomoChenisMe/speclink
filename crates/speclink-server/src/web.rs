//! The minimal web entry (決策 4): server-rendered HTML forms embedded in the
//! binary, no JS framework and no external resources. This knife starts with the
//! invitation acceptance page; login, logout and the account page are layered on
//! in the following tasks.

use crate::identity::{IdentityError, User};
use crate::state::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Form, Path, Query, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
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

fn login_location(user_code: Option<&str>) -> String {
    user_code
        .map(|code| format!("/login?user_code={code}"))
        .unwrap_or_else(|| "/login".to_string())
}

/// The set-password form submission.
#[derive(Deserialize)]
pub struct AcceptForm {
    pub password: String,
}

/// The login form submission.
#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub user_code: String,
}

/// The optional device user code carried by login and activation GET requests.
#[derive(Deserialize)]
pub struct UserCodeQuery {
    #[serde(default)]
    pub user_code: String,
}

/// The create-PAT form submission.
#[derive(Deserialize)]
pub struct CreatePatForm {
    pub name: String,
    #[serde(default)]
    pub expires: String,
}

/// The `/activate` form submission: the user code, plus an optional decision
/// (`approve`/`deny`) once the confirm step is shown.
#[derive(Deserialize)]
pub struct ActivateForm {
    #[serde(default)]
    pub user_code: String,
    #[serde(default)]
    pub action: String,
}

/// `GET /invite/{token}` — a valid invitation shows the set-password form; a
/// used, expired or unknown token yields the same invalid page.
pub async fn invite_page(State(state): State<AppState>, Path(token): Path<String>) -> Response {
    match state.identity.find_valid_invitation(&token) {
        Ok(Some(invitation)) => Html(invite_form(&token, &invitation.email, None)).into_response(),
        Ok(None) => invalid_invitation(),
        Err(_) => internal_error(),
    }
}

/// `POST /invite/{token}` — atomically create the active user with the invited
/// memberships and consume the invitation. A no-longer-valid token yields the
/// same invalid page as the GET.
pub async fn accept_invite(
    State(state): State<AppState>,
    Path(token): Path<String>,
    headers: HeaderMap,
    Form(form): Form<AcceptForm>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    if form.password.is_empty() {
        // A password is required; re-show the form rather than create an account
        // with an empty credential.
        let email = match state.identity.find_valid_invitation(&token) {
            Ok(Some(inv)) => inv.email,
            Ok(None) => return invalid_invitation(),
            Err(_) => return internal_error(),
        };
        return (
            StatusCode::BAD_REQUEST,
            Html(invite_form(&token, &email, Some("請輸入密碼"))),
        )
            .into_response();
    }
    match state.identity.accept_invitation(&token, &form.password) {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(IdentityError::InvalidInvitation) => invalid_invitation(),
        Err(_) => internal_error(),
    }
}

/// `GET /login` — the login form, optionally carrying one validated device
/// user code back to the activation page after authentication.
pub async fn login_page(Query(query): Query<UserCodeQuery>) -> Response {
    Html(login_form(None, validated_user_code(&query.user_code))).into_response()
}

/// `POST /login` — verify the password with argon2 and open a session. A
/// failure re-renders the form with a uniform message and no submitted values,
/// so an unknown email and a wrong password are byte-identical.
pub async fn do_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let user_code = validated_user_code(&form.user_code);
    match state
        .identity
        .authenticate_password(&form.email, &form.password)
    {
        Ok(Some(user)) => match state.identity.create_session(&user.id, session_ttl()) {
            Ok(session) => {
                let destination = user_code
                    .map(activation_location)
                    .unwrap_or_else(|| "/account".to_string());
                with_session_cookie(Redirect::to(&destination).into_response(), &session)
            }
            Err(_) => internal_error(),
        },
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Html(login_form(Some("帳號或密碼不正確"), user_code)),
        )
            .into_response(),
        Err(_) => internal_error(),
    }
}

/// `POST /logout` — revoke the server-side session (the cookie clear is only a
/// courtesy; the database is the authority) and return to the login page.
pub async fn do_logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    if let Some(session) = read_session_cookie(&headers) {
        let _ = state.identity.revoke_session(&session);
    }
    with_cleared_cookie(Redirect::to("/login").into_response())
}

/// `GET /account` — the account page. An unauthenticated visit redirects to
/// login.
pub async fn account_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(user) = current_user(&state, &headers) else {
        return Redirect::to("/login").into_response();
    };
    let sessions = state.identity.list_sessions(&user.id).unwrap_or_default();
    let pats = state.identity.list_pats(&user.id).unwrap_or_default();
    let families = state
        .identity
        .list_device_families(&user.id)
        .unwrap_or_default();
    Html(account_html(&user, &sessions, &pats, &families, None, None)).into_response()
}

/// `POST /account/tokens` — create a PAT for the logged-in user. The response
/// page shows the plaintext exactly once; the store keeps only prefix, hash and
/// metadata.
pub async fn create_pat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CreatePatForm>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let Some(user) = current_user(&state, &headers) else {
        return Redirect::to("/login").into_response();
    };
    let expires_at = match parse_expiry(&form.expires) {
        Ok(expires_at) => expires_at,
        Err(()) => {
            let sessions = state.identity.list_sessions(&user.id).unwrap_or_default();
            let pats = state.identity.list_pats(&user.id).unwrap_or_default();
            let families = state
                .identity
                .list_device_families(&user.id)
                .unwrap_or_default();
            return (
                StatusCode::BAD_REQUEST,
                Html(account_html(
                    &user,
                    &sessions,
                    &pats,
                    &families,
                    None,
                    Some("到期日格式須為 YYYY-MM-DD"),
                )),
            )
                .into_response();
        }
    };
    match state.identity.create_pat(&user.id, &form.name, expires_at) {
        Ok((_, plaintext)) => {
            let sessions = state.identity.list_sessions(&user.id).unwrap_or_default();
            let pats = state.identity.list_pats(&user.id).unwrap_or_default();
            let families = state
                .identity
                .list_device_families(&user.id)
                .unwrap_or_default();
            Html(account_html(
                &user,
                &sessions,
                &pats,
                &families,
                Some(&plaintext),
                None,
            ))
            .into_response()
        }
        Err(_) => internal_error(),
    }
}

/// `POST /account/tokens/{id}/revoke` — revoke one of the user's own PATs; the
/// effect is immediate for the next API request.
pub async fn revoke_pat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(pat_id): Path<String>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let Some(user) = current_user(&state, &headers) else {
        return Redirect::to("/login").into_response();
    };
    let _ = state.identity.revoke_pat(&user.id, &pat_id);
    Redirect::to("/account").into_response()
}

/// `POST /account/device/{id}/revoke` — revoke one of the user's own device
/// credential families; its access token and refresh credential die at once,
/// leaving other families and PATs untouched.
pub async fn revoke_device_family(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(family_id): Path<String>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let Some(user) = current_user(&state, &headers) else {
        return Redirect::to("/login").into_response();
    };
    let _ = state.identity.revoke_family(&user.id, &family_id);
    Redirect::to("/account").into_response()
}

/// `GET /activate` — the device approval page. A valid user-code query is
/// carried through login and prefilled, but GET never checks or changes device
/// authorization state.
pub async fn activate_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UserCodeQuery>,
) -> Response {
    let user_code = validated_user_code(&query.user_code);
    if current_user(&state, &headers).is_none() {
        return Redirect::to(&login_location(user_code)).into_response();
    }
    Html(activate_form(None, user_code)).into_response()
}

/// `POST /activate` — enter a user code to reach the explicit confirm step, or
/// confirm the approve/deny. Session-protected (an unauthenticated POST leaves
/// the request unapproved) and same-origin; the acting user is recorded on the
/// decision. Unknown, used and expired user codes all get one invalid page.
pub async fn activate_submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ActivateForm>,
) -> Response {
    if let Err(refused) = check_origin(&headers, &state.config.public_url) {
        return refused;
    }
    let Some(user) = current_user(&state, &headers) else {
        return Redirect::to("/login").into_response();
    };
    let user_code = form.user_code.trim();
    match form.action.as_str() {
        "approve" => match state.identity.approve_device(user_code, &user.id) {
            Ok(true) => Html(activate_result("已核准。你可以回到裝置繼續登入。")).into_response(),
            Ok(false) => activate_invalid(),
            Err(_) => internal_error(),
        },
        "deny" => match state.identity.deny_device(user_code, &user.id) {
            Ok(true) => Html(activate_result("已拒絕這個裝置的登入請求。")).into_response(),
            Ok(false) => activate_invalid(),
            Err(_) => internal_error(),
        },
        // No decision yet: validate the code and show the explicit confirm step.
        _ => match state.identity.device_is_pending(user_code) {
            Ok(true) => Html(activate_confirm(user_code)).into_response(),
            Ok(false) => activate_invalid(),
            Err(_) => internal_error(),
        },
    }
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
fn with_session_cookie(mut resp: Response, session: &str) -> Response {
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

/// Validate that a change-making POST is same-origin with the configured public
/// URL (HTML forms). A request with neither Origin nor Referer (a non-browser
/// client) is allowed; a present-but-foreign origin is refused with 403.
pub(crate) fn check_origin(headers: &HeaderMap, public_url: &str) -> Result<(), Response> {
    if is_same_origin(headers, public_url) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Html(page("拒絕", "<h1>跨來源請求被拒絕</h1>\n")),
        )
            .into_response())
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

/// `/api/speclink/v1/web` 下的 browser session routes。
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/session", get(api_session))
        .route("/login", post(api_login))
        .route("/logout", post(api_logout))
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
fn web_ok<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(DataEnvelope { data })).into_response()
}

/// A `{error}` envelope at `status`.
fn web_err(status: StatusCode, code: &'static str, message: impl Into<String>) -> Response {
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
/// single leading `/`, no scheme/authority (rejecting `//…` and `/\…`), with a
/// first segment of `account`, `activate` or `admin`.
fn classify_return_to(path: &str) -> Option<(&str, &str)> {
    if !path.starts_with('/') || path.starts_with("//") || path.starts_with("/\\") {
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

// --- rendering (embedded, no external resources) ---

/// The full HTML document shell. A single inline stylesheet keeps the pages
/// self-contained (CSP can be strict).
pub(crate) fn page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"zh-Hant\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n</head>\n<body>\n<main>\n{body}\n</main>\n</body>\n</html>\n"
    )
}

/// The set-password form for an invitation.
fn invite_form(token: &str, email: &str, error: Option<&str>) -> String {
    let error = error
        .map(|e| format!("<p class=\"error\">{}</p>", escape(e)))
        .unwrap_or_default();
    let body = format!(
        "<h1>接受邀請</h1>\n<p>為 {email} 設定登入密碼。</p>\n{error}<form method=\"post\" action=\"/invite/{token}\">\n<label>密碼 <input type=\"password\" name=\"password\" required></label>\n<button type=\"submit\">建立帳號</button>\n</form>\n",
        email = escape(email),
        token = escape(token),
    );
    page("接受邀請", &body)
}

/// The login form. On failure it carries a uniform message and no submitted
/// values, so the response never leaks whether an email exists.
fn login_form(error: Option<&str>, user_code: Option<&str>) -> String {
    let error = error
        .map(|e| format!("<p class=\"error\">{}</p>", escape(e)))
        .unwrap_or_default();
    let user_code = user_code
        .map(|code| {
            format!(
                "<input type=\"hidden\" name=\"user_code\" value=\"{}\">\n",
                escape(code)
            )
        })
        .unwrap_or_default();
    let body = format!(
        "<h1>登入</h1>\n{error}<form method=\"post\" action=\"/login\">\n{user_code}<label>Email <input type=\"email\" name=\"email\" required></label>\n<label>密碼 <input type=\"password\" name=\"password\" required></label>\n<button type=\"submit\">登入</button>\n</form>\n"
    );
    page("登入", &body)
}

/// The account page: the user's sessions and PATs, plus the forms to create and
/// revoke a PAT. `flash` carries a freshly-created PAT's plaintext, shown once.
fn account_html(
    user: &User,
    sessions: &[crate::identity::SessionInfo],
    pats: &[crate::identity::Pat],
    families: &[crate::identity::DeviceFamily],
    flash: Option<&str>,
    error: Option<&str>,
) -> String {
    let flash = flash
        .map(|token| {
            format!(
                "<div class=\"flash\">\n<p>新的 PAT，只會顯示這一次，請立即複製保存：</p>\n<code>{}</code>\n</div>\n",
                escape(token)
            )
        })
        .unwrap_or_default();
    let error = error
        .map(|e| format!("<p class=\"error\">{}</p>\n", escape(e)))
        .unwrap_or_default();

    let mut pat_rows = String::new();
    for pat in pats {
        let expires = pat
            .expires_at
            .map(fmt_ts)
            .unwrap_or_else(|| "永久".to_string());
        let last_used = pat
            .last_used_at
            .map(fmt_ts)
            .unwrap_or_else(|| "從未".to_string());
        let status = if pat.revoked_at.is_some() {
            "（已撤銷）"
        } else {
            ""
        };
        let revoke = if pat.revoked_at.is_none() {
            format!(
                "<form method=\"post\" action=\"/account/tokens/{id}/revoke\"><button type=\"submit\">撤銷</button></form>",
                id = escape(&pat.id)
            )
        } else {
            String::new()
        };
        pat_rows.push_str(&format!(
            "<li><code>{prefix}</code> {name}{status} — 到期 {expires}，last-used {last_used} {revoke}</li>\n",
            prefix = escape(&pat.prefix),
            name = escape(&pat.name),
        ));
    }

    let mut session_rows = String::new();
    for s in sessions {
        let status = if s.revoked_at.is_some() {
            "（已撤銷）"
        } else {
            ""
        };
        session_rows.push_str(&format!(
            "<li>建立於 {created}，到期 {expires}{status}</li>\n",
            created = fmt_ts(s.created_at),
            expires = fmt_ts(s.expires_at),
        ));
    }

    let mut family_rows = String::new();
    for f in families {
        let status = if f.revoked_at.is_some() {
            "（已撤銷）"
        } else {
            ""
        };
        let revoke = if f.revoked_at.is_none() {
            format!(
                "<form method=\"post\" action=\"/account/device/{id}/revoke\"><button type=\"submit\">撤銷</button></form>",
                id = escape(&f.id)
            )
        } else {
            String::new()
        };
        family_rows.push_str(&format!(
            "<li>{source}{status} — 建立於 {created}，最近 refresh {last} {revoke}</li>\n",
            source = escape(&f.source),
            created = fmt_ts(f.created_at),
            last = fmt_ts(f.last_refresh_at),
        ));
    }

    let body = format!(
        "<h1>帳號</h1>\n<p>{email}</p>\n<form method=\"post\" action=\"/logout\"><button type=\"submit\">登出</button></form>\n{flash}{error}\n<h2>Personal Access Tokens</h2>\n<ul>\n{pat_rows}</ul>\n<form method=\"post\" action=\"/account/tokens\">\n<label>名稱 <input type=\"text\" name=\"name\" required></label>\n<label>到期日（YYYY-MM-DD，留空為永久） <input type=\"text\" name=\"expires\"></label>\n<button type=\"submit\">建立 PAT</button>\n</form>\n<h2>裝置登入 Sessions</h2>\n<ul>\n{family_rows}</ul>\n<h2>Sessions</h2>\n<ul>\n{session_rows}</ul>\n",
        email = escape(&user.email),
    );
    page("帳號", &body)
}

/// The device-code entry form (the first step of `/activate`).
fn activate_form(error: Option<&str>, user_code: Option<&str>) -> String {
    let error = error
        .map(|e| format!("<p class=\"error\">{}</p>", escape(e)))
        .unwrap_or_default();
    let value = user_code
        .map(|code| format!(" value=\"{}\"", escape(code)))
        .unwrap_or_default();
    let body = format!(
        "<h1>裝置登入</h1>\n<p>輸入裝置上顯示的代碼以核准登入。</p>\n{error}<form method=\"post\" action=\"/activate\">\n<label>裝置代碼 <input type=\"text\" name=\"user_code\"{value} required></label>\n<button type=\"submit\">下一步</button>\n</form>\n"
    );
    page("裝置登入", &body)
}

/// The explicit confirm step: the user code plus approve/deny buttons.
fn activate_confirm(user_code: &str) -> String {
    let code = escape(user_code);
    let body = format!(
        "<h1>確認裝置登入</h1>\n<p>代碼 <code>{code}</code> 的裝置要求以你的身分登入。</p>\n<form method=\"post\" action=\"/activate\">\n<input type=\"hidden\" name=\"user_code\" value=\"{code}\">\n<button type=\"submit\" name=\"action\" value=\"approve\">核准</button>\n<button type=\"submit\" name=\"action\" value=\"deny\">拒絕</button>\n</form>\n"
    );
    page("確認裝置登入", &body)
}

/// The result page after an approve or deny decision.
fn activate_result(message: &str) -> String {
    page(
        "裝置登入",
        &format!("<h1>裝置登入</h1>\n<p>{}</p>\n", escape(message)),
    )
}

/// The single invalid-code page returned for unknown, used or expired user
/// codes — the reason is never distinguished.
fn activate_invalid() -> Response {
    let body = "<h1>裝置代碼無效</h1>\n<p>這個裝置代碼無法使用。請確認代碼，或在裝置上重新開始登入。</p>\n";
    (StatusCode::NOT_FOUND, Html(page("裝置代碼無效", body))).into_response()
}

/// Format a timestamp for display (date and minute, UTC).
fn fmt_ts(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M UTC").to_string()
}

/// The single "邀請無效" page returned for used, expired or unknown tokens —
/// the reason is never distinguished.
fn invalid_invitation() -> Response {
    let body = "<h1>邀請無效</h1>\n<p>這個邀請連結無法使用。請向邀請你的人索取新的連結。</p>\n";
    (StatusCode::NOT_FOUND, Html(page("邀請無效", body))).into_response()
}

fn internal_error() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(page("錯誤", "<h1>發生錯誤</h1>\n")),
    )
        .into_response()
}

/// Minimal HTML-attribute/text escaping for the values we interpolate.
pub(crate) fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

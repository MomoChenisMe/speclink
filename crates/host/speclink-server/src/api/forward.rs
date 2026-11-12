//! The demo server's one route into the host kit's HTTP door (design D8).
//!
//! The server keeps what is its own: it turns the bearer token into a user,
//! answers the two refusals the door cannot tell apart (a suspended account
//! asking for its scopes, a project the user is not a member of), and hands
//! the request with that user to the door on the blocking pool. A path the
//! door does not take ends in the same 404 any unknown path gets; a method
//! the door does not take is a 405.

use crate::assets;
use crate::auth::{bearer_token, resolve_bearer};
use crate::error::{into_axum, ApiError};
use crate::identity::User;
use crate::state::AppState;
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::{Path, Request, State};
use axum::http::header::ALLOW;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use speclink_host::http::{match_route, HttpError, HttpRequest, Method};
use speclink_host::kit::Identity;

/// The most a request body is read before anything is answered: the import's
/// allowance, so an early answer never cuts a body off mid-upload.
pub(crate) const READ_LIMIT_BYTES: usize = 32 * 1024 * 1024;

/// The board order's allowance. Its handler still refuses content over its
/// own, smaller cap with a clean 413.
const BOARD_ORDER_BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

/// Every other route's allowance: axum's default.
const BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;

const SCOPES: &str = "/api/speclink/v1/scopes";
const WHOAMI: &str = "/api/speclink/v1/whoami";

/// A user the token resolved to, and the PAT to mark as used once the
/// server's own checks accept the request.
struct Resolved {
    user: User,
    pat_id: Option<String>,
}

impl Resolved {
    /// The request passed the server's checks: a PAT's last use advances,
    /// best-effort — a metering failure never fails the request.
    fn accepted(self, state: &AppState) -> User {
        if let Some(pat_id) = &self.pat_id {
            let _ = state.identity.touch_pat(pat_id);
        }
        self.user
    }
}

/// `/api/speclink/v1/auth-methods`, `/whoami` and `/scopes`.
pub async fn root(State(state): State<AppState>, req: Request) -> Result<Response, ApiError> {
    let (parts, body) = req.into_parts();
    let body = read_whole(body).await?;
    within(&body, BODY_LIMIT_BYTES)?;
    let Some(method) = Method::from_name(parts.method.as_str()) else {
        return Ok(method_not_allowed());
    };
    // The login-method declaration needs no credential, so none is looked at.
    let user = match parts.uri.path() {
        SCOPES => scope_list_user(&state, &parts)?,
        WHOAMI => token_user(&state, &parts)?,
        _ => None,
    };
    through_door(state.clone(), method, parts, body, user.map(|found| found.accepted(&state))).await
}

/// `/api/speclink/v1/projects/{key}/{*rest}`.
pub async fn project(
    State(state): State<AppState>,
    Path((key, _rest)): Path<(String, String)>,
    req: Request,
) -> Result<Response, ApiError> {
    let (parts, body) = req.into_parts();
    let body = read_whole(body).await?;
    let Some(method) = Method::from_name(parts.method.as_str()) else {
        return Ok(method_not_allowed());
    };
    let Some(route) = match_route(method, parts.uri.path()) else {
        return Ok(assets::spa_fallback(parts.method, parts.uri).await);
    };
    within(&body, if route.pattern == "/board-order" { BOARD_ORDER_BODY_LIMIT_BYTES } else { BODY_LIMIT_BYTES })?;
    let user = member_user(&state, &parts, &key)?;
    through_door(state, method, parts, body, user).await
}

/// The whole body, read before anything is answered: an answer sent while a
/// body is still arriving makes the connection reset, and the client then
/// reads an empty response.
pub(crate) async fn read_whole(body: Body) -> Result<Bytes, ApiError> {
    Ok(to_bytes(body, READ_LIMIT_BYTES)
        .await
        .map_err(|e| HttpError::payload_too_large(format!("request body unreadable: {e}")))?)
}

fn within(body: &Bytes, limit: usize) -> Result<(), ApiError> {
    if body.len() > limit {
        return Err(HttpError::payload_too_large(format!("request body exceeds the {limit}-byte limit")).into());
    }
    Ok(())
}

/// A method the door does not take (design D1).
fn method_not_allowed() -> Response {
    (StatusCode::METHOD_NOT_ALLOWED, [(ALLOW, "GET, HEAD, POST, PUT, DELETE")]).into_response()
}

/// The user behind the bearer, or none when there is no valid one: the door
/// answers that with 401. A suspended user's token is not valid here.
fn token_user(state: &AppState, parts: &Parts) -> Result<Option<Resolved>, ApiError> {
    let Some(token) = bearer_token(&parts.headers) else {
        return Ok(None);
    };
    match resolve_bearer(state, &token) {
        Ok((user, pat_id)) => Ok(Some(Resolved { user, pat_id })),
        Err(e) if e.0.status == 401 => Ok(None),
        Err(e) => Err(e),
    }
}

/// The scope list's user: a suspended account is told so with 403, as the
/// scope list always did, instead of the 401 of a token that is not valid.
fn scope_list_user(state: &AppState, parts: &Parts) -> Result<Option<Resolved>, ApiError> {
    let Some(token) = bearer_token(&parts.headers) else {
        return Ok(None);
    };
    let unavailable = |_| HttpError::internal("identity store unavailable");
    let found = if token.starts_with("spk_at_") {
        let user = state.identity.authenticate_access_token_allow_suspended(&token).map_err(unavailable)?;
        user.map(|user| Resolved { user, pat_id: None })
    } else {
        let found = state.identity.authenticate_pat_allow_suspended(&token).map_err(unavailable)?;
        found.map(|(pat, user)| Resolved { user, pat_id: Some(pat.id) })
    };
    match found {
        Some(found) if !found.user.active => Err(HttpError::forbidden("account is suspended").into()),
        found => Ok(found),
    }
}

/// A project route's user. The door answers a project outside the user's
/// list with 404; a project that exists but the user is not a member of
/// stays 403, as it always was — and such a request does not count as a use
/// of the PAT.
fn member_user(state: &AppState, parts: &Parts, key: &str) -> Result<Option<User>, ApiError> {
    let Some(found) = token_user(state, parts)? else {
        return Ok(None);
    };
    let unavailable = |_| HttpError::internal("identity store unavailable");
    if state.identity.get_project(key).map_err(unavailable)?.is_some()
        && state.identity.membership_role(&found.user.id, key).map_err(unavailable)?.is_none()
    {
        return Err(HttpError::forbidden(format!("actor is not a member of project '{key}'")).into());
    }
    Ok(Some(found.accepted(state)))
}

/// The door's identity of a user: the stamp is "Name <email>" — a bare name
/// is not unique, the identity store keys on email — and the name the client
/// is shown stays the user's display.
fn identity_of(user: &User) -> Result<Identity, ApiError> {
    let invalid = |e: speclink_host::kit::InvalidIdentity| HttpError::internal(e.to_string());
    let identity = Identity::new(user.id.clone(), format!("{} <{}>", user.display, user.email)).map_err(invalid)?;
    if user.display.trim().is_empty() {
        return Ok(identity);
    }
    Ok(identity.shown_as(user.display.clone()).map_err(invalid)?)
}

/// Hand the request to the door on the blocking pool, with this thread's
/// store reads pinned to one load per scope.
async fn through_door(
    state: AppState,
    method: Method,
    parts: Parts,
    body: Bytes,
    user: Option<User>,
) -> Result<Response, ApiError> {
    let identity = user.as_ref().map(identity_of).transpose()?;
    let request = HttpRequest {
        method,
        path: parts.uri.path().to_string(),
        query: parts.uri.query().map(str::to_string),
        headers: parts
            .headers
            .iter()
            .filter_map(|(name, value)| Some((name.as_str().to_string(), value.to_str().ok()?.to_string())))
            .collect(),
        body: body.to_vec(),
    };
    let (kit, port) = (state.kit.clone(), state.port.clone());
    let answer = tokio::task::spawn_blocking(move || {
        let _pin = port.pin_thread();
        kit.handle(&request, identity.as_ref())
    })
    .await
    .map_err(|e| HttpError::internal(format!("blocking task failed: {e}")))?;
    Ok(match answer {
        Some(response) => into_axum(response),
        // A path the door does not take: what any unknown path gets.
        None => assets::spa_fallback(parts.method, parts.uri).await,
    })
}

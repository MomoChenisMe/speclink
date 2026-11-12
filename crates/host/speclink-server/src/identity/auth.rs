//! Authentication and binding precondition, fail closed (design 決策四).
//!
//! Every project-scoped route runs the same precondition before any verb: the
//! bearer token resolves to an actor, the URL project key resolves to a
//! registered project, the API version must be compatible, and the repo is
//! adjudicated — an explicit `X-Speclink-Repo` must name a registered repo, and
//! an absent header binds the sole repo or is refused as ambiguous (reusing the
//! Host's `resolve_binding`, which never auto-picks). Any step's failure stops
//! the request before the verb runs.

use crate::config::ActorConfig;
use crate::error::ApiError;
use speclink_host::http::{check_api_version, choose_repo, HttpError};
use crate::identity::{MembershipRole, Project, User};
use crate::state::AppState;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use speclink_protocol::query::{AuthWhoamiResponse, WhoamiUser};
use std::collections::HashMap;

/// A resolved request identity: the authenticated actor, the registered
/// project (from the registry), and the bound repo key.
#[derive(Debug, Clone)]
pub struct Binding {
    pub actor: ActorConfig,
    pub project: Project,
    pub repo: String,
    /// The membership role is Editor — the gate every project-scope write
    /// shares, enforced once by the router's write guard (app.rs).
    pub editor: bool,
}

impl FromRequestParts<AppState> for Binding {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 0. 寫入檢查中介層已解析過的請求直接取用（app.rs `require_editor`）：
        //    同一請求不查兩次 identity store、不重複推進 PAT 的最後使用時間。
        if let Some(binding) = parts.extensions.get::<Binding>() {
            return Ok(binding.clone());
        }
        // 1. bearer token → actor, resolved per-request against the identity
        //    store (決策 4): split by prefix — `spk_at_` is a device access
        //    token, anything else a PAT — into the same check-list (hash-match,
        //    unrevoked, unexpired, owning user active). Any failure is the same
        //    401 permission_denied so the cause is never probed; no cache means
        //    suspension and revocation are immediate. A PAT's last-used is
        //    advanced once every check passes (`touch_pat_id`).
        let token = bearer_token(&parts.headers)
            .ok_or_else(|| HttpError::unauthenticated("missing or malformed bearer token"))?;
        let (user, touch_pat_id) = resolve_bearer(state, &token)?;

        // 2. project key → registered project, read from the registry
        //    (unregistered → not found; a store failure → internal).
        let params = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| HttpError::not_found("project not found"))?;
        let key = params
            .get("key")
            .cloned()
            .ok_or_else(|| HttpError::not_found("project not found"))?;
        let project = state
            .identity
            .get_project(&key)
            .map_err(|_| HttpError::internal("identity store unavailable"))?
            .ok_or_else(|| HttpError::not_found(format!("project '{key}' not found")))?;

        // 3. membership: a valid token whose user is not a member of the URL
        //    project is 403, distinct from the 401 of an invalid token.
        let role = state
            .identity
            .membership_role(&user.id, &project.key)
            .map_err(|_| HttpError::internal("identity store unavailable"))?;
        let Some(role) = role else {
            return Err(HttpError::forbidden(format!(
                "actor is not a member of project '{}'",
                project.key
            )).into());
        };

        // 4. API version compatibility (incompatible → refused with reason),
        //    by the same check the host kit's door runs.
        check_api_version(header(parts, "x-speclink-api-version").as_deref())?;

        // 5. repo adjudication (explicit must be registered; absent binds the
        //    sole repo or refuses as ambiguous), by the door's own rule. The
        //    candidate repos come from the registry.
        let repos = state
            .identity
            .list_repos(&project.key)
            .map_err(|_| HttpError::internal("identity store unavailable"))?;
        let named = header(parts, "x-speclink-repo");
        let repo = choose_repo(&project.key, &repos, |r| &r.key, named.as_deref())?.key.clone();

        // The request authenticated. A PAT's last-used advances best-effort — a
        // metering write failure never fails the request; an access token keeps
        // no last-used (it is short-lived and rotates).
        if let Some(pat_id) = &touch_pat_id {
            let _ = state.identity.touch_pat(pat_id);
        }

        let actor = ActorConfig {
            id: user.id,
            display: user.display,
            email: user.email,
        };
        let editor = role == MembershipRole::Editor;
        Ok(Binding {
            actor,
            project,
            repo,
            editor,
        })
    }
}

/// Resolve a bearer to its user — the Binding precondition's first step,
/// shared with the root-level `/auth/whoami`: `spk_at_` is a device access
/// token, anything else a PAT, into the same check-list (hash-match,
/// unrevoked, unexpired, owning user active). Any failure is the same 401 so
/// the cause is never probed. A PAT hit returns its id for the caller to
/// touch once the request is otherwise accepted.
pub(crate) fn resolve_bearer(
    state: &AppState,
    token: &str,
) -> Result<(User, Option<String>), ApiError> {
    if token.starts_with("spk_at_") {
        let user = state
            .identity
            .authenticate_access_token(token)
            .map_err(|_| HttpError::internal("identity store unavailable"))?
            .ok_or_else(|| HttpError::unauthenticated("invalid token"))?;
        Ok((user, None))
    } else {
        let (pat, user) = state
            .identity
            .authenticate_pat(token)
            .map_err(|_| HttpError::internal("identity store unavailable"))?
            .ok_or_else(|| HttpError::unauthenticated("invalid token"))?;
        Ok((user, Some(pat.id)))
    }
}

/// `GET /auth/whoami` — the identity behind a bearer, at the server root
/// (connection-registry-keychain 決策 8). No project scope, no API-version or
/// repo header: this is what a client shows right after logging in, before
/// any project is chosen. Resolution is exactly the Binding's first step; a
/// PAT hit advances its last-used best-effort like any authenticated call.
pub async fn auth_whoami(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<AuthWhoamiResponse>, ApiError> {
    let token = bearer_token(&headers)
        .ok_or_else(|| HttpError::unauthenticated("missing or malformed bearer token"))?;
    let (user, touch_pat_id) = resolve_bearer(&state, &token)?;
    if let Some(pat_id) = &touch_pat_id {
        let _ = state.identity.touch_pat(pat_id);
    }
    Ok(axum::Json(AuthWhoamiResponse {
        user: WhoamiUser {
            name: user.display,
            handle: user.id,
        },
    }))
}

/// Extract the bearer token from the Authorization header.
pub(crate) fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(axum::http::header::AUTHORIZATION)?;
    let text = value.to_str().ok()?;
    text.strip_prefix("Bearer ").map(|t| t.trim().to_string())
}

/// Read a request header as a string.
pub(crate) fn header(parts: &Parts, name: &str) -> Option<String> {
    parts.headers.get(name)?.to_str().ok().map(str::to_string)
}

//! Authentication and binding precondition, fail closed (design 決策四).
//!
//! Every project-scoped route runs the same precondition before any verb: the
//! bearer token resolves to an actor, the URL project key resolves to a
//! registered project, the API version must be compatible, and the repo is
//! adjudicated — an explicit `X-Speclink-Repo` must name a registered repo, and
//! an absent header binds the sole repo or is refused as ambiguous (reusing the
//! Host's `resolve_binding`, which never auto-picks). Any step's failure stops
//! the request before the verb runs.

use crate::config::{ActorConfig, ProjectConfig};
use crate::error::ApiError;
use crate::identity::User;
use crate::state::AppState;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use speclink_core::config::ResolvedPolicy;
use speclink_host::binding::{resolve_binding, BindingCandidate, BindingError};
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_protocol::binding::{Actor as BindingActor, BindingResponse, Capabilities, ScopeRef};
use speclink_protocol::events::{EventTransport, EventsDeclaration, PollingDeclaration, TransportKind};
use speclink_protocol::API_VERSION;
use speclink_store::{ProjectId, RepoId};
use std::collections::HashMap;

/// The engine version the server advertises at handshake.
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A resolved request identity: the authenticated actor, the registered
/// project, and the bound repo key.
#[derive(Debug, Clone)]
pub struct Binding {
    pub actor: ActorConfig,
    pub project: ProjectConfig,
    pub repo: String,
}

impl Binding {
    /// The Host execution context this binding runs verbs under. The policy is
    /// a placeholder: server-mode policy is resolved from the store's workflow
    /// config, not from this field.
    pub fn execution_context(&self) -> SpeclinkExecutionContext {
        SpeclinkExecutionContext {
            actor: Actor::Identified {
                display: self.actor.display.clone(),
                source: ActorSource::Explicit,
            },
            project: ProjectId::new(self.project.key.clone()),
            repo: RepoId::new(self.repo.clone()),
            mode: ExecutionMode::SharedStore,
            policy: placeholder_policy(),
        }
    }

    /// The `/binding` handshake response: identity, versions, and the
    /// capability declaration — the sse push transport at `/events` (resume
    /// supported) alongside the unchanged polling fallback over `/sync-state`
    /// with ETag. The declared urls match the served routes.
    pub fn to_response(&self) -> BindingResponse {
        BindingResponse {
            actor: BindingActor {
                id: self.actor.id.clone(),
                name: self.actor.display.clone(),
            },
            project: ScopeRef {
                id: format!("prj_{}", self.project.key),
                key: self.project.key.clone(),
                name: self.project.name.clone(),
            },
            repo: ScopeRef {
                id: format!("repo_{}", self.repo),
                key: self.repo.clone(),
                name: self.repo.clone(),
            },
            api_version: API_VERSION.to_string(),
            engine_version: ENGINE_VERSION.to_string(),
            capabilities: Capabilities {
                context_snapshots: false,
                authentication: Vec::new(),
                events: EventsDeclaration {
                    transports: vec![EventTransport {
                        kind: TransportKind::Sse,
                        url: "/events".to_string(),
                        resume: true,
                    }],
                    polling: Some(PollingDeclaration {
                        url: "/sync-state".to_string(),
                        etag: true,
                    }),
                },
            },
        }
    }
}

impl FromRequestParts<AppState> for Binding {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. bearer token → actor, resolved per-request against the identity
        //    store (決策 4): split by prefix — `spk_at_` is a device access
        //    token, anything else a PAT — into the same check-list (hash-match,
        //    unrevoked, unexpired, owning user active). Any failure is the same
        //    401 permission_denied so the cause is never probed; no cache means
        //    suspension and revocation are immediate. A PAT's last-used is
        //    advanced once every check passes (`touch_pat_id`).
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

        // 2. project key → registered project (unregistered → not found)
        let params = Path::<HashMap<String, String>>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::not_found("project not found"))?;
        let key = params
            .get("key")
            .cloned()
            .ok_or_else(|| ApiError::not_found("project not found"))?;
        let project = state
            .config
            .projects
            .iter()
            .find(|p| p.key == key)
            .ok_or_else(|| ApiError::not_found(format!("project '{key}' not found")))?
            .clone();

        // 3. membership: a valid token whose user is not a member of the URL
        //    project is 403, distinct from the 401 of an invalid token.
        let member = state
            .identity
            .is_member(&user.id, &project.key)
            .map_err(|_| ApiError::internal("identity store unavailable"))?;
        if !member {
            return Err(ApiError::forbidden(format!(
                "actor is not a member of project '{}'",
                project.key
            )));
        }

        // 4. API version compatibility (incompatible → refused with reason)
        let version = header(parts, "x-speclink-api-version");
        if version.as_deref() != Some(API_VERSION) {
            let sent = version
                .map(|v| format!(", client sent '{v}'"))
                .unwrap_or_default();
            return Err(ApiError::refused(format!(
                "incompatible api version — this server speaks version '{API_VERSION}'{sent}"
            )));
        }

        // 5. repo adjudication (explicit must be registered; absent binds the
        //    sole repo or refuses as ambiguous)
        let repo = resolve_repo(&project, header(parts, "x-speclink-repo"))?;

        // The request authenticated. A PAT's last-used advances best-effort — a
        // metering write failure never fails the request; an access token keeps
        // no last-used (it is short-lived and rotates).
        if let Some(pat_id) = &touch_pat_id {
            let _ = state.identity.touch_pat(pat_id);
        }

        let actor = ActorConfig { id: user.id, display: user.display };
        Ok(Binding { actor, project, repo })
    }
}

/// Adjudicate the repo for a request, fail closed. Reuses the Host's
/// `resolve_binding` for the no-header case so ambiguity never auto-picks.
fn resolve_repo(project: &ProjectConfig, repo_header: Option<String>) -> Result<String, ApiError> {
    if let Some(repo) = repo_header {
        return if project.repos.iter().any(|r| *r == repo) {
            Ok(repo)
        } else {
            Err(ApiError::not_found(format!(
                "repo '{repo}' is not registered in project '{}'",
                project.key
            )))
        };
    }
    let candidates: Vec<BindingCandidate> = project
        .repos
        .iter()
        .map(|r| BindingCandidate {
            project: ProjectId::new(project.key.clone()),
            repo: RepoId::new(r.clone()),
            project_key: project.key.clone(),
            repo_key: r.clone(),
            permitted: true,
        })
        .collect();
    match resolve_binding(candidates) {
        Ok(binding) => Ok(binding.repo.as_str().to_string()),
        Err(BindingError::Missing) => Err(ApiError::not_found(format!(
            "project '{}' has no registered repo",
            project.key
        ))),
        Err(BindingError::Ambiguous { candidates }) => Err(ApiError::refused(format!(
            "project '{}' registers multiple repos — specify one with X-Speclink-Repo (candidates: {})",
            project.key,
            candidates.join(", ")
        ))),
        Err(BindingError::PermissionDenied { .. }) => Err(ApiError::permission_denied(format!(
            "no permitted repo in project '{}'",
            project.key
        ))),
    }
}

/// Extract the bearer token from the Authorization header.
fn bearer_token(parts: &Parts) -> Option<String> {
    let value = parts.headers.get(axum::http::header::AUTHORIZATION)?;
    let text = value.to_str().ok()?;
    text.strip_prefix("Bearer ").map(|t| t.trim().to_string())
}

/// Read a request header as a string.
fn header(parts: &Parts, name: &str) -> Option<String> {
    parts.headers.get(name)?.to_str().ok().map(str::to_string)
}

/// A placeholder policy for the Host context. Unused by the bridge (server-mode
/// policy comes from the store's workflow config), so its values are inert.
fn placeholder_policy() -> EffectiveWorkflowPolicy {
    EffectiveWorkflowPolicy::new(
        ResolvedPolicy {
            locale: "English".to_string(),
            spec_locale: None,
            tdd: false,
            audit: false,
        },
        "",
    )
}

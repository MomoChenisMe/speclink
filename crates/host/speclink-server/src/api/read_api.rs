//! Read-only server surfaces that are not Engine verbs: scope discovery,
//! canonical/archived documents, and desktop-aligned workspace search. The
//! document reads are `speclink_host::documents` on the shared store port.

use crate::auth::{Binding, IdentityOnly};
use crate::error::ApiError;
use crate::state::AppState;
use crate::verb;
use axum::extract::{Path, Query, State};
use axum::http::header::ETAG;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use speclink_host::documents;
use speclink_protocol::binding::ScopeRef;
use speclink_protocol::query::{ProjectScope, ScopesResponse};
use speclink_store_port::Version;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    #[serde(default)]
    q: String,
}

fn ok<T: Serialize>(dto: T, version: Version) -> Response {
    ([(ETAG, documents::etag(version))], Json(dto)).into_response()
}

/// `GET /api/speclink/v1/scopes` — membership-filtered identity scope, before
/// project/repo binding exists.
pub async fn scopes(
    State(state): State<AppState>,
    identity: IdentityOnly,
) -> Result<Json<ScopesResponse>, ApiError> {
    let memberships: HashSet<String> = state
        .identity
        .list_memberships(&identity.user.id)
        .map_err(|_| ApiError::internal("identity store unavailable"))?
        .into_iter()
        .collect();
    let projects = state
        .identity
        .list_projects()
        .map_err(|_| ApiError::internal("identity store unavailable"))?;
    let mut visible = Vec::new();
    for project in projects {
        if !memberships.contains(&project.key) {
            continue;
        }
        let repos = state
            .identity
            .list_repos(&project.key)
            .map_err(|_| ApiError::internal("identity store unavailable"))?
            .into_iter()
            .map(|repo| ScopeRef {
                id: format!("repo_{}", repo.key),
                key: repo.key,
                name: repo.name,
            })
            .collect();
        visible.push(ProjectScope {
            id: format!("prj_{}", project.key),
            key: project.key,
            name: project.name,
            repos,
        });
    }
    Ok(Json(ScopesResponse { projects: visible }))
}

/// `GET /specs/{capability}/document`.
pub async fn spec_document(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, capability)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let (dto, version) =
        verb::on_port(&state, &binding, move |port, scope| Ok(documents::spec_document(port, scope, &capability)?)).await?;
    Ok(ok(dto, version))
}

/// `GET /archived`.
pub async fn archived_list(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let (dto, version) = verb::on_port(&state, &binding, move |port, scope| Ok(documents::archived_list(port, scope)?)).await?;
    Ok(ok(dto, version))
}

/// `GET /archived/{datedName}/artifacts/{*artifact}`.
pub async fn archived_artifact(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, dated_name, artifact)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let (dto, version) = verb::on_port(&state, &binding, move |port, scope| {
        Ok(documents::archived_artifact(port, scope, &dated_name, &artifact)?)
    })
    .await?;
    Ok(ok(dto, version))
}

/// `GET /archived/{datedName}/capabilities`.
pub async fn archived_capabilities(
    State(state): State<AppState>,
    binding: Binding,
    Path((_project, dated_name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let (capabilities, version) = verb::on_port(&state, &binding, move |port, scope| {
        Ok(documents::archived_capabilities(port, scope, &dated_name)?)
    })
    .await?;
    Ok(ok(capabilities, version))
}

/// `GET /search?q=...` — the desktop D6 algorithm, including fixed artifact
/// order, one hit per card, char-safe snippets and live discussions only.
pub(crate) async fn search(
    State(state): State<AppState>,
    binding: Binding,
    Query(query): Query<SearchQuery>,
) -> Result<Response, ApiError> {
    let (dto, version) =
        verb::on_port(&state, &binding, move |port, scope| Ok(documents::search(port, scope, &query.q)?)).await?;
    Ok(ok(dto, version))
}

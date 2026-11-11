//! Context snapshot endpoint (server-context-api spec「一致快照端點」/「change 縮小
//! 與 flow 透傳」).
//!
//! One `POST /context` returns a [`ContextSnapshot`] built by
//! `speclink_host::documents::context_snapshot` on the shared store port. The
//! snapshot id is the scope state token (any commit advances it), so
//! `If-None-Match` returns 304 while the scope is unchanged. Document
//! narrowing is by change only; the `flow` field is passed through and
//! narrowed by the materializer, never here (design 決策三).
//!
//! [`ContextSnapshot`]: speclink_protocol::context::ContextSnapshot

use crate::auth::Binding;
use crate::error::ApiError;
use crate::state::AppState;
use crate::verb;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use speclink_host::documents::{self, ContextOutcome};
use speclink_protocol::context::ContextSnapshotRequest;

/// `POST /context` — one context snapshot of the binding's scope.
pub async fn snapshot(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
    Json(request): Json<ContextSnapshotRequest>,
) -> Result<Response, ApiError> {
    let if_none_match = headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let outcome = verb::on_port(&state, &binding, move |port, scope| {
        Ok(documents::context_snapshot(
            port,
            scope,
            &request,
            if_none_match.as_deref(),
        )?)
    })
    .await?;
    match outcome {
        ContextOutcome::NotModified(version) => {
            Ok((StatusCode::NOT_MODIFIED, [(ETAG, documents::etag(version))]).into_response())
        }
        ContextOutcome::Snapshot(snapshot) => {
            Ok(([(ETAG, snapshot.snapshot_id.clone())], Json(snapshot)).into_response())
        }
    }
}

//! The project routes the demo server keeps for itself: the import and the
//! event stream. Every other project route is the host kit's HTTP door
//! (`forward`).

use crate::auth::Binding;
use crate::error::ApiError;
use speclink_host::http::HttpError;
use crate::events::Subscription;
use crate::state::AppState;
use axum::http::header::ETAG;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use speclink_protocol::events::InvalidationEvent;
use speclink_protocol::query::{
    ImportBundle, ImportDocumentId, ImportDocumentOutcome, ImportReportResponse, ImportedDocument,
};
use speclink_store::{
    content_digest, Bundle, BundleDoc, DocumentId, ImportMode, ImportOutcome, ProjectId, RepoId, Revision, Scope,
    StoreError, BUNDLE_FORMAT_VERSION,
};
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;

/// A JSON response carrying the scope ETag.
fn ok<T: Serialize>(dto: T, etag: &str) -> Response {
    ([(ETAG, etag.to_string())], Json(dto)).into_response()
}

/// The store scope a binding addresses.
fn scope_of(binding: &Binding) -> Scope {
    Scope::new(
        ProjectId::new(binding.project.key.clone()),
        RepoId::new(binding.repo.clone()),
    )
}

/// `POST /import` — local-to-remote migration into one empty bound scope.
/// The wire has no operation selector; the handler always invokes CreateNew.
pub async fn import_bundle(
    State(state): State<AppState>,
    binding: Binding,
    Json(request): Json<ImportBundle>,
) -> Result<Response, ApiError> {
    if request.format_version != BUNDLE_FORMAT_VERSION {
        return Err(HttpError::refused(format!(
            "unsupported bundle format version {} (supported: {})",
            request.format_version, BUNDLE_FORMAT_VERSION
        )).into());
    }

    let scope = scope_of(&binding);
    if request.scope.project != scope.project.as_str() || request.scope.repo != scope.repo.as_str()
    {
        return Err(HttpError::refused(format!(
            "bundle scope {}/{} does not match authenticated binding {}/{}",
            request.scope.project,
            request.scope.repo,
            scope.project.as_str(),
            scope.repo.as_str()
        )).into());
    }

    let mut documents = Vec::with_capacity(request.documents.len());
    for document in request.documents {
        let actual_digest = content_digest(&document.content);
        if actual_digest != document.digest {
            return Err(HttpError::invalid_argument(format!(
                "bundle digest mismatch for {:?}",
                document.document
            )).into());
        }
        documents.push(BundleDoc {
            doc: import_document_id(document.document),
            content: document.content,
            digest: document.digest,
        });
    }

    let bundle = Bundle {
        format_version: request.format_version,
        scope: scope.clone(),
        project_revision: Revision(request.project_revision),
        documents,
    };
    let store = state.store.clone();
    let report = tokio::task::spawn_blocking(move || {
        store
            .import(bundle, ImportMode::CreateNew)
            .map_err(import_store_error)
    })
    .await
    .map_err(|error| HttpError::internal(format!("blocking task failed: {error}")))??;

    let project_revision = report.project_revision.0;
    let documents = report
        .documents
        .into_iter()
        .map(|document| {
            if document.outcome != ImportOutcome::Created {
                return Err(HttpError::internal(
                    "CreateNew import returned an impossible overwrite outcome",
                ).into());
            }
            Ok(ImportedDocument {
                document: store_document_id(document.doc),
                outcome: ImportDocumentOutcome::Created,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    state.events.notify(&scope);
    Ok(ok(
        ImportReportResponse {
            project_revision,
            documents,
        },
        &format!("\"{project_revision}\""),
    ))
}

fn import_store_error(error: StoreError) -> ApiError {
    match error {
        StoreError::Backend { source } if source.contains("import (create-new)") => {
            HttpError::refused(source).into()
        }
        other => ApiError::from(other),
    }
}

fn import_document_id(document: ImportDocumentId) -> DocumentId {
    match document {
        ImportDocumentId::ChangeMeta { change } => DocumentId::ChangeMeta { change },
        ImportDocumentId::ChangeArtifact { change, artifact } => {
            DocumentId::ChangeArtifact { change, artifact }
        }
        ImportDocumentId::ChangeEvidence { change } => DocumentId::ChangeEvidence { change },
        ImportDocumentId::CanonicalSpec { capability } => DocumentId::CanonicalSpec { capability },
        ImportDocumentId::Discussion { slug, archived } => {
            DocumentId::Discussion { slug, archived }
        }
        ImportDocumentId::WorkflowConfig => DocumentId::WorkflowConfig,
        ImportDocumentId::ArchivedChange { change, doc } => {
            DocumentId::ArchivedChange { change, doc }
        }
        ImportDocumentId::Language => DocumentId::Language,
        ImportDocumentId::BoardOrder => DocumentId::BoardOrder,
    }
}

fn store_document_id(document: DocumentId) -> ImportDocumentId {
    match document {
        DocumentId::ChangeMeta { change } => ImportDocumentId::ChangeMeta { change },
        DocumentId::ChangeArtifact { change, artifact } => {
            ImportDocumentId::ChangeArtifact { change, artifact }
        }
        DocumentId::ChangeEvidence { change } => ImportDocumentId::ChangeEvidence { change },
        DocumentId::CanonicalSpec { capability } => ImportDocumentId::CanonicalSpec { capability },
        DocumentId::Discussion { slug, archived } => {
            ImportDocumentId::Discussion { slug, archived }
        }
        DocumentId::WorkflowConfig => ImportDocumentId::WorkflowConfig,
        DocumentId::ArchivedChange { change, doc } => {
            ImportDocumentId::ArchivedChange { change, doc }
        }
        DocumentId::Language => ImportDocumentId::Language,
        DocumentId::BoardOrder => ImportDocumentId::BoardOrder,
    }
}

/// `GET /events` — the project-scoped SSE invalidation stream (server-event-stream
/// spec). The `Binding` extractor has already run the same bearer/binding
/// precondition as every route, so an unauthenticated or non-member request
/// never reaches here. A `Last-Event-ID` below the cleaned floor gets a reset
/// frame first; a resumable one backfills the gap; an idle stream is kept alive
/// by comment heartbeats. Each hint carries only the outbox sequence, scope,
/// resource id, and revision — never document content.
pub async fn events(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let heartbeat = state.events.settings().heartbeat;
    let scope = scope_of(&binding);
    let Subscription { mut rx, plan } = state.events.subscribe(&scope, last_event_id)?;

    let stream = async_stream::stream! {
        if plan.reset {
            yield Ok::<Event, Infallible>(reset_event());
        }
        // Everything at or below `last` is already delivered (backfill or
        // history), so the live tail skips it — no gap, no repeat.
        let mut last = plan.cursor;
        for hint in plan.backfill {
            if let Ok(seq) = hint.event_id.parse::<u64>() {
                last = seq;
            }
            yield Ok(invalidation_event(&hint));
        }
        loop {
            match rx.recv().await {
                Ok(item) => {
                    if item.seq > last {
                        last = item.seq;
                        yield Ok(invalidation_event(&item.event));
                    }
                }
                // A slow consumer that overflowed its buffer is dropped; it
                // reconnects with Last-Event-ID and resumes.
                Err(RecvError::Lagged(_)) | Err(RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(heartbeat).text("heartbeat"))
        .into_response())
}

/// One invalidation hint as an SSE event: `id` is the outbox sequence (so a
/// client's `Last-Event-ID` resumes from it), the event type is `invalidate`,
/// and the data is the DTO JSON.
fn invalidation_event(hint: &InvalidationEvent) -> Event {
    Event::default()
        .id(hint.event_id.clone())
        .event("invalidate")
        .json_data(hint)
        .unwrap_or_else(|_| Event::default().comment("invalidation serialize failed"))
}

/// The reset signal — a distinct SSE event type telling the client its cursor
/// is cleaned; it converges by re-reading through Query + ETag.
fn reset_event() -> Event {
    Event::default()
        .event("reset")
        .data("cursor expired; re-read via query")
}

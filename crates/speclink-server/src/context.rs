//! Context snapshot endpoint (server-context-api spec「一致快照端點」/「change 縮小
//! 與 flow 透傳」).
//!
//! One `POST /context` returns a [`ContextSnapshot`] read from a single
//! consistent store snapshot: the snapshot fixes the scope state token and
//! revision base, `export` enumerates the scope's documents (the contract's only
//! enumeration seam), and each is read back through the same snapshot so content
//! and revision agree at one project revision — mirroring the bridge's
//! consistent read. The snapshot id is the scope state token (any commit
//! advances it), so `If-None-Match` returns 304 while the scope is unchanged.
//! Document narrowing is by change only; the `flow` field is passed through and
//! narrowed by the materializer, never here (design 決策三).

use crate::auth::Binding;
use crate::error::ApiError;
use crate::state::AppState;
use crate::verb;
use axum::extract::State;
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use speclink_protocol::context::{ContextDocument, ContextSnapshot, ContextSnapshotRequest};
use speclink_store::{content_digest, DocumentId, Scope, TeamStore};

/// The openspec projection path of the workflow config — the policy-revision
/// source and the config mirror path.
const CONFIG_PATH: &str = "openspec/config.yaml";

/// `POST /context` — one consistent context snapshot of the binding's scope.
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
    let scope = verb::scope_of(&binding);
    let store = state.store.clone();
    let outcome =
        tokio::task::spawn_blocking(move || build(store.as_ref(), &scope, &request, if_none_match))
            .await
            .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    match outcome {
        BuildOutcome::NotModified(etag) => {
            Ok((StatusCode::NOT_MODIFIED, [(ETAG, etag)]).into_response())
        }
        BuildOutcome::Snapshot { etag, snapshot } => {
            Ok(([(ETAG, etag)], Json(snapshot)).into_response())
        }
    }
}

/// A built snapshot, or the not-modified short-circuit when the caller's
/// `If-None-Match` still matches the scope state token.
enum BuildOutcome {
    NotModified(String),
    Snapshot {
        etag: String,
        snapshot: ContextSnapshot,
    },
}

/// Build the snapshot on the blocking pool. A single `snapshot` fixes the scope
/// state token (the ETag / snapshot id) and the revision each document is read
/// at; when the caller's `If-None-Match` matches, the export is skipped
/// entirely. Otherwise `export` enumerates the scope and each selected document
/// is read back through the same snapshot for content and revision at one state.
fn build(
    store: &dyn TeamStore,
    scope: &Scope,
    request: &ContextSnapshotRequest,
    if_none_match: Option<String>,
) -> Result<BuildOutcome, ApiError> {
    let snapshot = store.snapshot(scope)?;
    let etag = format!("\"{}\"", snapshot.revision().0);
    if if_none_match.as_deref() == Some(etag.as_str()) {
        return Ok(BuildOutcome::NotModified(etag));
    }
    let bundle = store.export(scope)?;

    // Change-narrowing existence check: an addressed change with no metadata and
    // no artifacts does not exist.
    if let Some(change) = request.change.as_deref() {
        let exists = bundle.documents.iter().any(|b| match &b.doc {
            DocumentId::ChangeMeta { change: c } => c == change,
            DocumentId::ChangeArtifact { change: c, .. } => c == change,
            _ => false,
        });
        if !exists {
            return Err(ApiError::not_found(format!("change '{change}' not found")));
        }
    }

    let mut documents = Vec::new();
    for entry in &bundle.documents {
        let Some(path) = select_path(&entry.doc, request.change.as_deref()) else {
            continue;
        };
        // Read back through the fixed-point snapshot: a document `export` lists
        // but the snapshot does not hold was written after it — outside this view.
        let Some(doc) = snapshot.read(&entry.doc)? else {
            continue;
        };
        documents.push(ContextDocument {
            path,
            digest: content_digest(&doc.content),
            content: doc.content,
            revision: Some(doc.revision.0),
        });
    }
    documents.sort_by(|a, b| a.path.cmp(&b.path));

    // Policy revision is the workflow config document's revision at this
    // snapshot, absent when the scope has no config (design 決策二).
    let policy_revision = documents
        .iter()
        .find(|d| d.path == CONFIG_PATH)
        .and_then(|d| d.revision);
    let combined: Vec<&str> = documents.iter().map(|d| d.digest.as_str()).collect();
    let digest = content_digest(&combined.join("\n"));
    let snapshot = ContextSnapshot {
        snapshot_id: etag.clone(),
        policy_revision,
        digest,
        documents,
    };
    Ok(BuildOutcome::Snapshot { etag, snapshot })
}

/// The openspec projection path of a document when the request selects it, or
/// `None` to exclude it. `change = Some(A)` narrows to A's artifacts (proposal /
/// design / tasks and delta specs), all canonical specs, config and LANGUAGE;
/// `change = None` is the full mirror — every change's artifacts plus all
/// canonical specs, config, LANGUAGE and live discussions. Internal change
/// metadata and archived documents are never part of the readable mirror.
fn select_path(doc: &DocumentId, change: Option<&str>) -> Option<String> {
    match doc {
        DocumentId::ChangeArtifact {
            change: c,
            artifact,
        } => change
            .map_or(true, |n| n == c)
            .then(|| format!("openspec/changes/{c}/{artifact}")),
        DocumentId::CanonicalSpec { capability } => {
            Some(format!("openspec/specs/{capability}/spec.md"))
        }
        DocumentId::WorkflowConfig => Some(CONFIG_PATH.to_string()),
        DocumentId::Language => Some("openspec/LANGUAGE.md".to_string()),
        DocumentId::Discussion {
            slug,
            archived: false,
        } if change.is_none() => Some(format!("openspec/discussions/{slug}.md")),
        _ => None,
    }
}

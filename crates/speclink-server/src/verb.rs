//! Running one Engine verb over the store through the bridge, off the async
//! runtime (design 決策二): the synchronous Host/Engine/Store path runs on the
//! blocking pool, and the scope's ETag is read in the same hop so a command's
//! response carries its post-commit state token.

use crate::auth::Binding;
use crate::error::ApiError;
use crate::state::AppState;
use speclink_core::command::Command;
use speclink_host::bridge::{self, BridgeExecution};
use speclink_store::{Document, DocumentId, ProjectId, RepoId, Scope, StoreError, TeamStore};

/// The outcome of running a verb plus the scope's current ETag.
pub struct VerbResult {
    pub execution: BridgeExecution,
    pub etag: String,
}

/// The store scope a binding addresses.
pub fn scope_of(binding: &Binding) -> Scope {
    Scope::new(
        ProjectId::new(binding.project.key.clone()),
        RepoId::new(binding.repo.clone()),
    )
}

/// The scope's ETag: the project revision, which advances on every commit
/// (design 決策五). Monotonic, so any change invalidates it.
pub fn scope_token(store: &dyn TeamStore, scope: &Scope) -> Result<String, StoreError> {
    Ok(format!("\"{}\"", store.snapshot(scope)?.revision().0))
}

/// Run `cmd` over the binding's scope and read the resulting ETag, on the
/// blocking pool. On success the scope's event broadcaster is notified so any
/// commit's outbox events reach subscribers (决策 2).
pub async fn run(state: &AppState, binding: &Binding, cmd: Command) -> Result<VerbResult, ApiError> {
    let store = state.store.clone();
    let ctx = binding.execution_context();
    let scope = scope_of(binding);
    let result = tokio::task::spawn_blocking(move || -> Result<VerbResult, ApiError> {
        let execution = bridge::execute(store.as_ref(), &ctx, cmd)?;
        let etag = scope_token(store.as_ref(), &scope)?;
        Ok(VerbResult { execution, etag })
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?;
    if result.is_ok() {
        state.events.notify(&scope_of(binding));
    }
    result
}

/// The scope's ETag, read on the blocking pool. For query routes that do not
/// run a verb but still declare the scope state token.
pub async fn scope_etag(state: &AppState, binding: &Binding) -> Result<String, ApiError> {
    let store = state.store.clone();
    let scope = scope_of(binding);
    tokio::task::spawn_blocking(move || scope_token(store.as_ref(), &scope).map_err(ApiError::from))
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?
}

/// Run a write verb guarded by an `If-Match` precondition against `doc`, on the
/// blocking pool. The precondition catches a stale client read (the doc moved
/// since the version the client holds); the bridge's own CAS at commit catches
/// a concurrent write during the request. Returns the doc's new version and the
/// scope ETag. Both are read after the commit so the response reflects it.
pub async fn run_write_with_if_match(
    state: &AppState,
    binding: &Binding,
    doc: DocumentId,
    if_match: u64,
    cmd: Command,
) -> Result<(u64, String), ApiError> {
    let store = state.store.clone();
    let ctx = binding.execution_context();
    let scope = scope_of(binding);
    let result = tokio::task::spawn_blocking(move || -> Result<(u64, String), ApiError> {
        let current = store
            .snapshot(&scope)
            .map_err(ApiError::from)?
            .read(&doc)
            .map_err(ApiError::from)?
            .map(|d| d.revision.0);
        check_if_match(if_match, current)?;
        bridge::execute(store.as_ref(), &ctx, cmd).map_err(ApiError::from)?;
        let version = store
            .snapshot(&scope)
            .map_err(ApiError::from)?
            .read(&doc)
            .map_err(ApiError::from)?
            .map(|d| d.revision.0)
            .unwrap_or(0);
        let etag = scope_token(store.as_ref(), &scope)?;
        Ok((version, etag))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?;
    if result.is_ok() {
        state.events.notify(&scope_of(binding));
    }
    result
}

/// Enforce the `If-Match` precondition: `0` is create-only (the document must
/// be absent), a positive value requires the document at that exact revision. A
/// mismatch is a revision conflict naming expected and actual.
fn check_if_match(if_match: u64, current: Option<u64>) -> Result<(), ApiError> {
    let satisfied = match (if_match, current) {
        (0, None) => true,
        (0, Some(_)) => false,
        (n, Some(c)) => n == c,
        (_, None) => false,
    };
    if satisfied {
        Ok(())
    } else {
        let actual = current.map(|c| c.to_string()).unwrap_or_else(|| "absent".to_string());
        Err(ApiError::revision_conflict(format!(
            "expected version {if_match}, actual {actual}"
        )))
    }
}

/// Read one document (content + revision) at the scope's current snapshot, on
/// the blocking pool. Used to stamp an artifact read's version (so a later
/// If-Match write can CAS against it) and to read the workflow config.
pub async fn read_doc(
    state: &AppState,
    binding: &Binding,
    doc: DocumentId,
) -> Result<Option<Document>, ApiError> {
    let store = state.store.clone();
    let scope = scope_of(binding);
    tokio::task::spawn_blocking(move || -> Result<Option<Document>, ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        snapshot.read(&doc).map_err(ApiError::from)
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?
}

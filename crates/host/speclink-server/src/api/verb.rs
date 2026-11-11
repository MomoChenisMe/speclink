//! Running one Engine verb over the store port, off the async runtime (design
//! 決策二): the synchronous Host/Engine path runs on the blocking pool, and the
//! scope's ETag is read in the same hop so a command's response carries its
//! post-commit state token.

use crate::auth::Binding;
use crate::error::ApiError;
use crate::state::AppState;
use speclink_core::command::Command;
use speclink_host::documents;
use speclink_host::port_exec::{self, PortExecution};
use speclink_host::teamstore_port;
use speclink_store::{ProjectId, RepoId, Scope};
use speclink_store_port::ScopeId;

/// The outcome of running a verb plus the scope's current ETag.
pub struct VerbResult {
    pub execution: PortExecution,
    pub etag: String,
}

/// The store scope a binding addresses.
pub fn scope_of(binding: &Binding) -> Scope {
    Scope::new(
        ProjectId::new(binding.project.key.clone()),
        RepoId::new(binding.repo.clone()),
    )
}

/// The store-port scope id a binding addresses: `<project>/<repo>`. A
/// binding that cannot make one is a server bug.
pub fn scope_id_of(binding: &Binding) -> Result<ScopeId, ApiError> {
    Ok(teamstore_port::scope_id(&scope_of(binding))?)
}

/// Run `work` on the blocking pool.
pub async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?
}

/// Run `cmd` over the binding's scope and read the resulting ETag, on the
/// blocking pool. On success the scope's event broadcaster is notified so any
/// commit's outbox events reach subscribers (决策 2).
pub async fn run(
    state: &AppState,
    binding: &Binding,
    cmd: Command,
) -> Result<VerbResult, ApiError> {
    let port = state.port.clone();
    let ctx = binding.execution_context();
    let scope = scope_id_of(binding)?;
    let result = blocking(move || {
        let execution = port_exec::execute(port.as_ref(), &scope, &ctx, cmd)?;
        let etag = documents::etag(documents::scope_version(port.as_ref(), &scope)?);
        Ok(VerbResult { execution, etag })
    })
    .await;
    if result.is_ok() {
        state.events.notify(&scope_of(binding));
    }
    result
}

/// The scope's ETag, read on the blocking pool. For query routes that do not
/// run a verb but still declare the scope state token. The scope version
/// changes on every commit (design 決策五), so any change invalidates it.
pub async fn scope_etag(state: &AppState, binding: &Binding) -> Result<String, ApiError> {
    let port = state.port.clone();
    let scope = scope_id_of(binding)?;
    blocking(move || Ok(documents::etag(documents::scope_version(port.as_ref(), &scope)?))).await
}

/// Run a write verb guarded by an `If-Match` precondition against one
/// artifact of `change`, on the blocking pool. The precondition catches a
/// stale client read (the document moved since the version the client holds);
/// the commit's own version check catches a concurrent write during the
/// request. Returns the document's new version and the scope ETag, both read
/// after the commit so the response reflects it.
pub async fn run_write_with_if_match(
    state: &AppState,
    binding: &Binding,
    change: String,
    artifact: String,
    if_match: u64,
    cmd: Command,
) -> Result<(u64, String), ApiError> {
    let port = state.port.clone();
    let ctx = binding.execution_context();
    let scope = scope_id_of(binding)?;
    let result = blocking(move || {
        let port = port.as_ref();
        let version = || {
            documents::document_version(port, &scope, &change, &artifact)
                .map(|v| v.map(|v| v.get()))
        };
        check_if_match(if_match, version()?)?;
        port_exec::execute(port, &scope, &ctx, cmd)?;
        let written = version()?.unwrap_or(0);
        let etag = documents::etag(documents::scope_version(port, &scope)?);
        Ok((written, etag))
    })
    .await;
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
        let actual = current
            .map(|c| c.to_string())
            .unwrap_or_else(|| "absent".to_string());
        Err(ApiError::revision_conflict(format!(
            "expected version {if_match}, actual {actual}"
        )))
    }
}

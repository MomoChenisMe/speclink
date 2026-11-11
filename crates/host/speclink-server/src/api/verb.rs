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
use speclink_store_port::{ScopeId, StorePort, Version};

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

/// Run `work` on the blocking pool over a view of the binding's scope pinned
/// for this request: every list and read of the request comes from one load
/// of the scope (design D6).
pub async fn on_port<T: Send + 'static>(
    state: &AppState,
    binding: &Binding,
    work: impl FnOnce(&dyn StorePort, &ScopeId) -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    let port = state.port.clone();
    let scope = teamstore_port::scope_id(&scope_of(binding))?;
    tokio::task::spawn_blocking(move || work(&port.pinned(&scope), &scope))
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?
}

/// The scope's ETag: its version, which advances on every commit (design
/// 決策五), so any change invalidates it.
pub fn etag_now(port: &dyn StorePort, scope: &ScopeId) -> Result<String, ApiError> {
    Ok(documents::etag(documents::scope_version(port, scope)?))
}

/// The ETag after a command: the version its commit landed at, or — when it
/// committed nothing — the scope's version.
fn etag_after(
    port: &dyn StorePort,
    scope: &ScopeId,
    execution: &PortExecution,
) -> Result<String, ApiError> {
    match execution.scope_version {
        Some(version) => Ok(documents::etag(version)),
        None => etag_now(port, scope),
    }
}

/// The scope version a client says it last saw. A number past the largest
/// version names no version any scope can be at, so it is stale.
pub fn expected_version(
    port: &dyn StorePort,
    scope: &ScopeId,
    sent: u64,
) -> Result<Version, ApiError> {
    Version::new(sent).map_err(|_| match documents::scope_version(port, scope) {
        Ok(actual) => ApiError::revision_conflict(documents::scope_moved(sent, actual)),
        Err(e) => ApiError::from(e),
    })
}

/// Run `cmd` over the binding's scope and read the resulting ETag, on the
/// blocking pool. On success the scope's event broadcaster is notified so any
/// commit's outbox events reach subscribers (决策 2).
pub async fn run(
    state: &AppState,
    binding: &Binding,
    cmd: Command,
) -> Result<VerbResult, ApiError> {
    let ctx = binding.execution_context();
    let result = on_port(state, binding, move |port, scope| {
        let execution = port_exec::execute(port, scope, &ctx, cmd)?;
        let etag = etag_after(port, scope, &execution)?;
        Ok(VerbResult { execution, etag })
    })
    .await;
    if result.is_ok() {
        state.events.notify(&scope_of(binding));
    }
    result
}

/// The scope's ETag, read on the blocking pool. For query routes that do not
/// run a verb but still declare the scope state token.
pub async fn scope_etag(state: &AppState, binding: &Binding) -> Result<String, ApiError> {
    on_port(state, binding, etag_now).await
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
    let ctx = binding.execution_context();
    let result = on_port(state, binding, move |port, scope| {
        let version = || {
            documents::document_version(port, scope, &change, &artifact)
                .map(|v| v.map(|v| v.get()))
        };
        check_if_match(if_match, version()?)?;
        let execution = port_exec::execute(port, scope, &ctx, cmd)?;
        let written = version()?.unwrap_or(0);
        Ok((written, etag_after(port, scope, &execution)?))
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

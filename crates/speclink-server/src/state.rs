//! Shared application state handed to every route handler.

use crate::config::ServerConfig;
use crate::identity::IdentityStore;
use speclink_store::TeamStore;
use std::sync::Arc;

/// The store backend, shared across async handlers. `Send + Sync` so a handler
/// can move a clone into `spawn_blocking` to run the synchronous bridge.
pub type SharedStore = Arc<dyn TeamStore + Send + Sync>;

/// The identity store, shared across handlers. The API auth precondition and the
/// web entry both resolve identity through it.
pub type SharedIdentity = Arc<dyn IdentityStore>;

/// State every handler receives: the store backend, the identity store, and the
/// validated configuration (Project/Repo registry and public origin).
#[derive(Clone)]
pub struct AppState {
    pub store: SharedStore,
    pub identity: SharedIdentity,
    pub config: Arc<ServerConfig>,
}

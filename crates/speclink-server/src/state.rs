//! Shared application state handed to every route handler.

use crate::config::ServerConfig;
use speclink_store::TeamStore;
use std::sync::Arc;

/// The store backend, shared across async handlers. `Send + Sync` so a handler
/// can move a clone into `spawn_blocking` to run the synchronous bridge.
pub type SharedStore = Arc<dyn TeamStore + Send + Sync>;

/// State every handler receives: the store backend and the validated
/// configuration (Project/Repo registry and token → actor mapping).
#[derive(Clone)]
pub struct AppState {
    pub store: SharedStore,
    pub config: Arc<ServerConfig>,
}

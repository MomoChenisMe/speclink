//! Shared application state handed to every route handler.

use crate::config::ServerConfig;
use crate::events::EventHub;
use crate::identity::IdentityStore;
use speclink_host::teamstore_port::TeamStorePort;
use speclink_store::TeamStore;
use speclink_store_port::StorePort;
use std::sync::Arc;

/// The store backend, shared across async handlers. `Send + Sync` so a handler
/// can move a clone into `spawn_blocking` to run the synchronous Host path.
pub type SharedStore = Arc<dyn TeamStore + Send + Sync>;

/// The store backend as a store port: the one adapter every request shares,
/// so a run of requests with no commit in between loads a scope once.
pub type SharedPort = Arc<dyn StorePort>;

/// The identity store, shared across handlers. The API auth precondition and the
/// web entry both resolve identity through it.
pub type SharedIdentity = Arc<dyn IdentityStore>;

/// State every handler receives: the store backend and its store-port
/// adapter, the identity store, the validated configuration (Project/Repo
/// registry and public origin), and the per-scope event broadcaster fed from
/// the store's outbox.
///
/// Project routes read and write through `port`. `store` is left for what
/// the store port has no operation for: the import, the event stream's
/// outbox, the admin surfaces and the backups.
#[derive(Clone)]
pub struct AppState {
    pub store: SharedStore,
    pub port: SharedPort,
    pub identity: SharedIdentity,
    pub config: Arc<ServerConfig>,
    pub events: Arc<EventHub>,
}

impl AppState {
    /// The state over `store`, with the one store-port adapter built here.
    pub fn new(
        store: SharedStore,
        identity: SharedIdentity,
        config: Arc<ServerConfig>,
        events: Arc<EventHub>,
    ) -> AppState {
        let port: SharedPort = Arc::new(TeamStorePort::new(store.clone()));
        AppState {
            store,
            port,
            identity,
            config,
            events,
        }
    }
}

//! speclink-server: the official Speclink HTTP server.
//!
//! An axum adapter that serves the Client Protocol over the canonical
//! Host → Engine → TeamStore path. The async boundary lives entirely in this
//! crate (design 決策二): handlers run the synchronous bridge on the blocking
//! pool, and the Engine/Host/Store crates stay runtime-free.

pub mod admin;
pub mod app;
pub mod audit;
pub mod auth;
pub mod backup;
pub mod config;
pub mod context;
pub mod device;
pub mod error;
pub mod events;
pub mod identity;
pub mod identity_sqlite;
pub mod read_api;
pub mod routes;
pub mod setup;
pub mod state;
pub mod verb;
pub mod web;

use config::{IdentityConfig, StoreConfig};
use identity::IdentitySqlite;
use state::{AppState, SharedIdentity, SharedStore};
use std::sync::Arc;

/// Build the store backend the configuration declares, fail closed: an open
/// failure (unreadable path, corrupt or incompatible data, a directory already
/// held by another server) propagates rather than starting on a broken store.
pub fn build_store(store: &StoreConfig) -> anyhow::Result<SharedStore> {
    match store {
        StoreConfig::Memory => Ok(Arc::new(speclink_store::memory::MemoryStore::new())),
        StoreConfig::Sqlite { path } => {
            let store = speclink_store_sqlite::SqliteTeamStore::open(path).map_err(|e| {
                anyhow::anyhow!("cannot open sqlite store at '{}': {e}", path.display())
            })?;
            Ok(Arc::new(store))
        }
        StoreConfig::ServerFs { path } => {
            let store = speclink_store_fs::FsTeamStore::open(path).map_err(|e| {
                anyhow::anyhow!("cannot open serverfs store at '{}': {e}", path.display())
            })?;
            Ok(Arc::new(store))
        }
        StoreConfig::Postgres { url } => {
            // A password in the config file is accepted — some deployments have
            // nowhere else to put it — but it is a secret at rest in a file that
            // gets copied, diffed and pasted, so say so once at startup. The
            // URL itself is never echoed: that would print the very secret this
            // warns about.
            if speclink_store_postgres::url_embeds_password(url).unwrap_or(false) {
                eprintln!(
                    "warning: the postgres store url carries a password; \
                     prefer leaving it out and setting {} instead",
                    speclink_store_postgres::PASSWORD_VAR
                );
            }
            let store = speclink_store_postgres::PostgresTeamStore::connect(url)
                .map_err(|e| anyhow::anyhow!("cannot open postgres store: {e}"))?;
            Ok(Arc::new(store))
        }
    }
}

/// Build the identity store the configuration declares, fail closed: a foreign
/// or newer identity database is refused with its bytes untouched.
pub fn build_identity(identity: &IdentityConfig) -> anyhow::Result<SharedIdentity> {
    let store = match identity {
        IdentityConfig::Memory => IdentitySqlite::open_memory(),
        IdentityConfig::Sqlite { path } => IdentitySqlite::open(path),
    }
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(Arc::new(store))
}

/// Bind `addr` and serve until the process is signalled. The router is built
/// from the shared application state.
pub async fn serve(addr: &str, state: AppState) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app::router(state)).await?;
    Ok(())
}

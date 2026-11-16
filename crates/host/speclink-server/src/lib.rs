//! speclink-server: the official Speclink HTTP server.
//!
//! An axum adapter that serves the Client Protocol over the canonical
//! Host → Engine → store port path. The async boundary lives entirely in this
//! crate (design 決策二): handlers run the synchronous bridge on the blocking
//! pool, and the Engine/Host/Store crates stay runtime-free.

pub mod admin;
mod api;
pub mod app;
pub mod config;
pub mod identity;
pub mod state;
pub mod storage;
pub mod web;

// 四個面的子模組在根層掛回原本的名字：crate 內外既有的 `speclink_server::<模組>`
// 與 `crate::<模組>` 路徑一字不改。`api` 沒有自己的主模組，資料夾私有即可。
pub use admin::{audit, backup};
pub use api::{error, events, forward, routes};
pub use identity::{auth, device, setup};
pub use web::assets;

pub use storage::open as build_store;

use config::IdentityConfig;
use identity::IdentitySqlite;
use state::{AppState, SharedIdentity};
use std::sync::Arc;

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

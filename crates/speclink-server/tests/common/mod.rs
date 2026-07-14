//! Shared test harness: start the server in-process on a random port and
//! return its base URL. Used by every route test.

#![allow(dead_code)]

use speclink_server::config::{ActorConfig, ProjectConfig, ServerConfig, StoreConfig, TokenConfig};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::memory::MemoryStore;
use std::net::TcpListener as StdListener;
use std::sync::Arc;

/// Start the server over `state` on a free loopback port; returns the base URL
/// (`http://127.0.0.1:<port>`). The server runs on a detached runtime thread
/// that lives for the rest of the test process.
pub fn start(state: AppState) -> String {
    let listener = StdListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    listener.set_nonblocking(true).expect("nonblocking");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
            axum::serve(listener, speclink_server::app::router(state))
                .await
                .expect("serve");
        });
    });
    format!("http://{addr}")
}

/// A single-project, single-repo, single-token configuration for tests.
pub fn demo_config() -> ServerConfig {
    ServerConfig {
        store: StoreConfig::Memory,
        projects: vec![ProjectConfig {
            key: "demo".to_string(),
            name: "Demo".to_string(),
            repos: vec!["backend".to_string()],
        }],
        tokens: vec![TokenConfig {
            token: "secret".to_string(),
            actor: ActorConfig {
                id: "u_1".to_string(),
                display: "Tester <tester@example.com>".to_string(),
            },
        }],
    }
}

/// Build an [`AppState`] over `store` and the demo configuration.
pub fn state_with(store: SharedStore) -> AppState {
    AppState {
        store,
        config: Arc::new(demo_config()),
    }
}

/// Build an [`AppState`] over a fresh in-memory store and `config`.
pub fn state_with_config(config: ServerConfig) -> AppState {
    AppState {
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(config),
    }
}

/// The demo configuration with an additional two-repo project `multi`.
pub fn config_with_dual_repo_project() -> ServerConfig {
    let mut config = demo_config();
    config.projects.push(ProjectConfig {
        key: "multi".to_string(),
        name: "Multi".to_string(),
        repos: vec!["web".to_string(), "api".to_string()],
    });
    config
}

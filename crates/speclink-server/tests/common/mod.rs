//! Shared test harness: start the server in-process on a random port and
//! return its base URL. Used by every route test.

#![allow(dead_code)]

use chrono::{Duration, Utc};
use speclink_server::config::{IdentityConfig, ProjectConfig, ServerConfig, StoreConfig};
use speclink_server::identity::{IdentitySqlite, NewInvitation};
use speclink_server::state::{AppState, SharedIdentity, SharedStore};
use speclink_store::memory::MemoryStore;
use std::net::TcpListener as StdListener;
use std::sync::Arc;

/// The display identity the seeded test user authenticates as.
pub const SEED_DISPLAY: &str = "Tester <tester@example.com>";

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
        identity: IdentityConfig::Memory,
        public_url: "http://127.0.0.1".to_string(),
        projects: vec![ProjectConfig {
            key: "demo".to_string(),
            name: "Demo".to_string(),
            repos: vec!["backend".to_string()],
        }],
    }
}

/// A fresh, empty in-memory identity store for tests that do not authenticate
/// (health/readiness) or that seed their own PAT afterwards.
pub fn empty_identity() -> SharedIdentity {
    Arc::new(IdentitySqlite::open_memory().expect("in-memory identity store"))
}

/// Seed a user (display [`SEED_DISPLAY`]) that is a member of every project in
/// `projects`, plus a PAT. Returns the PAT plaintext (the bearer to send) and
/// the new user's id (the actor id to expect). Call on `state.identity` before
/// [`start`] moves the state.
pub fn seed_pat(identity: &SharedIdentity, projects: &[&str]) -> (String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: "tester@example.com".to_string(),
            display: SEED_DISPLAY.to_string(),
            memberships: projects.iter().map(|p| p.to_string()).collect(),
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("seed invitation");
    let user_id = identity.accept_invitation(&token, "seed-password").expect("seed accept");
    let (_, pat) = identity.create_pat(&user_id, "test", None).expect("seed pat");
    (pat, user_id)
}

/// Build an [`AppState`] over `store` and the demo configuration.
pub fn state_with(store: SharedStore) -> AppState {
    AppState {
        store,
        identity: empty_identity(),
        config: Arc::new(demo_config()),
    }
}

/// Build an [`AppState`] over a fresh in-memory store and `config`.
pub fn state_with_config(config: ServerConfig) -> AppState {
    AppState {
        store: Arc::new(MemoryStore::new()),
        identity: empty_identity(),
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

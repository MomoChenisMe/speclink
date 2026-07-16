//! Shared test harness: start the server in-process on a random port and
//! return its base URL. Used by every route test.

#![allow(dead_code)]

pub mod subscriber;

use chrono::{Duration, Utc};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentityStore, IdentitySqlite, NewInvitation};
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

/// A minimal in-memory configuration for tests. The Project/Repo registry is
/// seeded into the identity store separately (see [`seed_demo_registry`]).
pub fn demo_config() -> ServerConfig {
    ServerConfig {
        store: StoreConfig::Memory,
        identity: IdentityConfig::Memory,
        public_url: "http://127.0.0.1".to_string(),
        events: EventSettings::default(),
    }
}

/// A fresh, empty in-memory identity store for tests that do not authenticate
/// (health/readiness) or that seed their own PAT afterwards.
pub fn empty_identity() -> SharedIdentity {
    Arc::new(IdentitySqlite::open_memory().expect("in-memory identity store"))
}

/// An event hub over a throwaway store, for tests that build [`AppState`]
/// directly but never exercise the event stream (auth, web, device flows).
pub fn detached_events() -> Arc<EventHub> {
    EventHub::new(Arc::new(MemoryStore::new()), EventSettings::default())
}

/// Seed a user (display [`SEED_DISPLAY`]) that is a member of every project in
/// `projects`, plus a PAT. Returns the PAT plaintext (the bearer to send) and
/// the new user's id (the actor id to expect). Call on `state.identity` before
/// [`start`] moves the state.
pub fn seed_pat(identity: &SharedIdentity, projects: &[&str]) -> (String, String) {
    seed_named_pat(identity, "tester@example.com", SEED_DISPLAY, projects)
}

/// Seed a user with `email`/`display` (a member of `projects`) plus a PAT, so a
/// test can hold two distinct identities. Returns the PAT plaintext and user id.
pub fn seed_named_pat(
    identity: &SharedIdentity,
    email: &str,
    display: &str,
    projects: &[&str],
) -> (String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: display.to_string(),
            memberships: projects.iter().map(|p| p.to_string()).collect(),
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("seed invitation");
    let user_id = identity.accept_invitation(&token, "seed-password").expect("seed accept");
    let (_, pat) = identity.create_pat(&user_id, "test", None).expect("seed pat");
    (pat, user_id)
}

/// Build an [`AppState`] over `store` and the demo configuration, with the demo
/// project seeded into the registry.
pub fn state_with(store: SharedStore) -> AppState {
    let events = EventHub::new(store.clone(), EventSettings::default());
    let identity = empty_identity();
    seed_demo_registry(&*identity);
    AppState {
        store,
        identity,
        config: Arc::new(demo_config()),
        events,
    }
}

/// Build an [`AppState`] over `store`, the demo configuration, and explicit
/// event settings (short heartbeat / small buffer for stream tests). The demo
/// project is seeded into the registry.
pub fn state_with_event_settings(store: SharedStore, settings: EventSettings) -> AppState {
    let events = EventHub::new(store.clone(), settings);
    let identity = empty_identity();
    seed_demo_registry(&*identity);
    AppState {
        store,
        identity,
        config: Arc::new(demo_config()),
        events,
    }
}

/// Build an [`AppState`] over a fresh in-memory store and `config`, with the
/// demo project seeded into the registry.
pub fn state_with_config(config: ServerConfig) -> AppState {
    let store: SharedStore = Arc::new(MemoryStore::new());
    let events = EventHub::new(store.clone(), config.events.clone());
    let identity = empty_identity();
    seed_demo_registry(&*identity);
    AppState {
        store,
        identity,
        config: Arc::new(config),
        events,
    }
}

/// Seed the default demo project (repo `backend`) into the registry — the
/// registry equivalent of what `demo_config` used to declare. The state builders
/// call this, so binding-exercising tests need no per-test registry seeding.
pub fn seed_demo_registry(identity: &dyn IdentityStore) {
    identity.create_project("demo", "Demo").expect("seed demo project");
    identity.create_repo("demo", "backend", "backend").expect("seed demo repo");
}

/// Additionally register the two-repo `multi` project (repos `web`, `api`) — for
/// the ambiguous-repo and membership tests that need a second project.
pub fn seed_multi_project(identity: &dyn IdentityStore) {
    identity.create_project("multi", "Multi").expect("seed multi project");
    identity.create_repo("multi", "web", "web").expect("seed multi web repo");
    identity.create_repo("multi", "api", "api").expect("seed multi api repo");
}

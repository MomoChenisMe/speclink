//! Shared test harness: start the server in-process on a random port and
//! return its base URL. Used by every route test.

#![allow(dead_code)]

pub mod subscriber;

use chrono::{Duration, Utc};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedIdentity, SharedStore};
use speclink_store::memory::MemoryStore;
use std::net::TcpListener as StdListener;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// 重量級測試閘門：spawn 真實子程序（server／CLI binary）的測試取此鎖互斥
/// 執行，壓低合併單 binary 後 loopback 埠競爭、free_port 的 TOCTOU 窗口與
/// healthz／setup-token 就緒時限在高負載下被擠壓的風險；in-process 測試不取，
/// 維持全並發（樣式同 desktop tests/it/common 的 HARNESS_GATE）。
///
/// 代價：it 全程約 275s（gate 前約 100s）。嫌慢的退場路徑＝縮小射程至帶硬
/// 時限的 admin_e2e、e2e_cli、startup 三檔（依據與門檻見封存 change
/// merge-integration-test-binaries 的 task 10.1 與 design Decisions 5）。
static PROCESS_GATE: Mutex<()> = Mutex::new(());

pub fn acquire_process_gate() -> MutexGuard<'static, ()> {
    // 前一個測試 panic 只代表該測試失敗，poison 不該連坐後續測試。
    PROCESS_GATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Sends `body`, or no body when `body` is null. Pass null to a route that never reads the
/// body: the server can answer and close before a late body arrives, and the reset loses the
/// response (macOS: EINVAL from `set_read_timeout`; Windows: connection aborted).
pub fn send_json_or_empty(
    req: ureq::Request,
    body: serde_json::Value,
) -> Result<ureq::Response, ureq::Error> {
    if body.is_null() {
        req.call()
    } else {
        req.send_json(body)
    }
}

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
    let user_id = identity
        .accept_invitation(&token, "seed-password")
        .expect("seed accept");
    let (_, pat) = identity
        .create_pat(&user_id, "test", None)
        .expect("seed pat");
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
    identity
        .create_project("demo", "Demo")
        .expect("seed demo project");
    identity
        .create_repo("demo", "backend", "backend")
        .expect("seed demo repo");
}

/// Additionally register the two-repo `multi` project (repos `web`, `api`) — for
/// the ambiguous-repo and membership tests that need a second project.
pub fn seed_multi_project(identity: &dyn IdentityStore) {
    identity
        .create_project("multi", "Multi")
        .expect("seed multi project");
    identity
        .create_repo("multi", "web", "web")
        .expect("seed multi web repo");
    identity
        .create_repo("multi", "api", "api")
        .expect("seed multi api repo");
}

/// A store that lets one other writer in while a verb runs: the writer runs
/// once, when the verb opens its unit of work — or earlier, at the first
/// snapshot taken after `snapshots_first` snapshots, when that is set.
pub struct Racing {
    inner: Arc<MemoryStore>,
    other_writer: Mutex<Option<OtherWriter>>,
    snapshots_first: Option<usize>,
    snapshots: std::sync::atomic::AtomicUsize,
}

type OtherWriter = Box<dyn FnOnce(&MemoryStore) + Send>;

impl Racing {
    /// The writer lands between the verb's reads and its commit.
    pub fn at_unit_of_work(
        inner: Arc<MemoryStore>,
        other_writer: impl FnOnce(&MemoryStore) + Send + 'static,
    ) -> Racing {
        Racing {
            inner,
            other_writer: Mutex::new(Some(Box::new(other_writer))),
            snapshots_first: None,
            snapshots: Default::default(),
        }
    }

    /// The writer lands right after the first snapshot: a route that reads
    /// the scope twice sees the writer's content the second time.
    pub fn after_the_first_snapshot(
        inner: Arc<MemoryStore>,
        other_writer: impl FnOnce(&MemoryStore) + Send + 'static,
    ) -> Racing {
        Racing { snapshots_first: Some(1), ..Racing::at_unit_of_work(inner, other_writer) }
    }

    fn let_the_writer_in(&self) {
        if let Some(other_writer) = self.other_writer.lock().unwrap().take() {
            other_writer(&self.inner);
        }
    }
}

impl speclink_store::TeamStore for Racing {
    fn manifest(&self) -> speclink_store::Manifest {
        self.inner.manifest()
    }
    fn health(&self) -> Result<(), speclink_store::StoreError> {
        self.inner.health()
    }
    fn migrate(&self, target_version: u32) -> Result<(), speclink_store::StoreError> {
        self.inner.migrate(target_version)
    }
    fn snapshot<'a>(
        &'a self,
        scope: &speclink_store::Scope,
    ) -> Result<Box<dyn speclink_store::Snapshot + 'a>, speclink_store::StoreError> {
        let taken = self.snapshots.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if self.snapshots_first.is_some_and(|first| taken >= first) {
            self.let_the_writer_in();
        }
        self.inner.snapshot(scope)
    }
    fn begin_unit_of_work(
        &self,
        scope: &speclink_store::Scope,
        ctx: speclink_store::CommandContext,
    ) -> Result<speclink_store::UnitOfWork, speclink_store::StoreError> {
        self.let_the_writer_in();
        self.inner.begin_unit_of_work(scope, ctx)
    }
    fn commit(
        &self,
        uow: speclink_store::UnitOfWork,
        events: Vec<speclink_store::EventRecord>,
    ) -> Result<speclink_store::Revision, speclink_store::StoreError> {
        self.inner.commit(uow, events)
    }
    fn rollback(&self, uow: speclink_store::UnitOfWork) -> Result<(), speclink_store::StoreError> {
        self.inner.rollback(uow)
    }
    fn export(
        &self,
        scope: &speclink_store::Scope,
    ) -> Result<speclink_store::Bundle, speclink_store::StoreError> {
        self.inner.export(scope)
    }
    fn import(
        &self,
        bundle: speclink_store::Bundle,
        mode: speclink_store::ImportMode,
    ) -> Result<speclink_store::ImportReport, speclink_store::StoreError> {
        self.inner.import(bundle, mode)
    }
    fn read_outbox(
        &self,
        scope: &speclink_store::Scope,
        from: speclink_store::OutboxCursor,
    ) -> Result<Vec<speclink_store::OutboxEntry>, speclink_store::StoreError> {
        self.inner.read_outbox(scope, from)
    }
    fn ack_outbox(
        &self,
        scope: &speclink_store::Scope,
        up_to: speclink_store::OutboxCursor,
    ) -> Result<(), speclink_store::StoreError> {
        self.inner.ack_outbox(scope, up_to)
    }
    fn outbox_acked(
        &self,
        scope: &speclink_store::Scope,
    ) -> Result<speclink_store::OutboxCursor, speclink_store::StoreError> {
        self.inner.outbox_acked(scope)
    }
}

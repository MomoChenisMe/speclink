//! Shared test harness: start the server in-process on a random port and
//! return its base URL. Used by every route test.

#![allow(dead_code)]

pub mod subscriber;

use chrono::{Duration, Utc};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_host::scope::{scope_id, ProjectId, RepoId};
use speclink_server::state::{AppState, SharedIdentity, SharedStore};
use speclink_server::storage::{Memory, ServerStore};
use speclink_store_port::{
    Actor, Commit, DocKey, Document, Expected, KeyPrefix, Listing, Op, PortError, RecordedEvent, ScopeId,
    StorePort, Timestamp, Version,
};
use std::net::TcpListener as StdListener;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    EventHub::new(TestStore::memory().shared(), EventSettings::default())
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
    AppState::new(
        store,
        identity,
        Arc::new(demo_config()),
        events,
    )
}

/// Build an [`AppState`] over `store`, the demo configuration, and explicit
/// event settings (short heartbeat / small buffer for stream tests). The demo
/// project is seeded into the registry.
pub fn state_with_event_settings(store: SharedStore, settings: EventSettings) -> AppState {
    let events = EventHub::new(store.clone(), settings);
    let identity = empty_identity();
    seed_demo_registry(&*identity);
    AppState::new(
        store,
        identity,
        Arc::new(demo_config()),
        events,
    )
}

/// Build an [`AppState`] over a fresh in-memory store and `config`, with the
/// demo project seeded into the registry.
pub fn state_with_config(config: ServerConfig) -> AppState {
    let store = TestStore::memory().shared();
    let events = EventHub::new(store.clone(), config.events.clone());
    let identity = empty_identity();
    seed_demo_registry(&*identity);
    AppState::new(
        store,
        identity,
        Arc::new(config),
        events,
    )
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

// --- the store a test server runs on ---

/// One document as a test reads it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDoc {
    pub content: String,
    /// The scope version of the commit that last wrote the document.
    pub version: u64,
}

/// One event a store kept, with its sequence in the scope.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredEvent {
    pub seq: u64,
    pub name: String,
    pub payload: serde_json::Value,
    /// The scope version of the commit the event came with.
    pub version: u64,
}

/// A test's handle on the store a server runs on. A test fills the store,
/// reads documents back and reads the kept events through it; a document is
/// named by its store-port key (`changes/demo/tasks.md`, `config.yaml`).
/// The store type is not part of a test.
#[derive(Clone)]
pub struct TestStore {
    /// The store as the server state takes it.
    shared: SharedStore,
    /// The switch and the counter of the in-memory store.
    memory: Option<Arc<Switched>>,
}

impl TestStore {
    /// A new, empty store in memory.
    pub fn memory() -> TestStore {
        let memory = Arc::new(Switched { inner: Memory::new(), down: AtomicBool::new(false), reads: AtomicUsize::new(0) });
        TestStore { shared: memory.clone(), memory: Some(memory) }
    }

    /// The SQLite store at `path`: a second connection when a server has it
    /// open.
    pub fn sqlite(path: impl AsRef<Path>) -> TestStore {
        let opened = speclink_server::build_store(&StoreConfig::Sqlite { path: path.as_ref().to_path_buf() });
        let shared = opened.unwrap_or_else(|e| panic!("open the sqlite store: {e}"));
        TestStore { shared, memory: None }
    }

    /// The store as the server state takes it.
    pub fn shared(&self) -> SharedStore {
        self.shared.clone()
    }

    /// Write `docs` (key, content) into the scope in one commit: a key with
    /// no document is created, one with a document is written over. Returns
    /// the scope version after the commit.
    pub fn put(&self, project: &str, repo: &str, docs: &[(&str, &str)]) -> u64 {
        let scope = scope_of(project, repo);
        let keys: Vec<DocKey> = docs.iter().map(|(key, _)| doc_key(key)).collect();
        let current = self.shared.read(&scope, &keys).expect("read the current versions");
        let ops = keys
            .into_iter()
            .zip(docs)
            .zip(current)
            .map(|((key, (_, content)), current)| Op::Put {
                key,
                content: content.to_string(),
                expected: current.map_or(Expected::Absent, |doc| Expected::At(doc.version)),
            })
            .collect();
        let commit = Commit::new(
            Actor::new("seed", "seed").expect("an actor"),
            "seed",
            Timestamp::new(0).expect("a time"),
            ops,
            Vec::new(),
        )
        .expect("a commit");
        self.shared.commit(&scope, &commit).expect("the test's write commits").get()
    }

    /// The document at `key`, or `None` when the scope has none there.
    pub fn doc(&self, project: &str, repo: &str, key: &str) -> Option<StoredDoc> {
        let read = self.shared.read(&scope_of(project, repo), &[doc_key(key)]).expect("read the document");
        read.into_iter()
            .next()
            .flatten()
            .map(|doc| StoredDoc { content: doc.content, version: doc.version.get() })
    }

    /// The content of the document at `key`.
    pub fn content(&self, project: &str, repo: &str, key: &str) -> Option<String> {
        self.doc(project, repo, key).map(|doc| doc.content)
    }

    /// The scope's current version; 0 when nothing was written.
    pub fn version(&self, project: &str, repo: &str) -> u64 {
        let everything = KeyPrefix::new("").expect("the empty prefix");
        self.shared.list(&scope_of(project, repo), &everything).expect("list the scope").version.get()
    }

    /// The key of every document of the scope, in key order.
    pub fn keys(&self, project: &str, repo: &str) -> Vec<String> {
        let everything = KeyPrefix::new("").expect("the empty prefix");
        let listing = self.shared.list(&scope_of(project, repo), &everything).expect("list the scope");
        listing.entries.into_iter().map(|entry| entry.key.as_str().to_string()).collect()
    }

    /// The events the store keeps for the scope, oldest first.
    pub fn events(&self, project: &str, repo: &str) -> Vec<StoredEvent> {
        self.shared
            .events_after(&scope_of(project, repo), 0, usize::MAX)
            .expect("read the events")
            .into_iter()
            .map(|(seq, event)| StoredEvent {
                seq,
                name: event.name,
                payload: event.payload,
                version: event.version.get(),
            })
            .collect()
    }

    /// The names of the scope's kept events, oldest first.
    pub fn event_names(&self, project: &str, repo: &str) -> Vec<String> {
        self.events(project, repo).into_iter().map(|event| event.name).collect()
    }

    /// The largest event sequence the server removed from the scope; 0 when
    /// it removed none.
    pub fn event_floor(&self, project: &str, repo: &str) -> u64 {
        self.shared.event_floor(&scope_of(project, repo)).expect("read the event floor")
    }

    /// Break the store: from now on its health check fails and it serves
    /// nothing. In-memory store only.
    pub fn break_down(&self) {
        let memory = self.memory.as_ref().expect("only the in-memory store can be broken");
        memory.down.store(true, Ordering::SeqCst);
        assert!(self.shared.probe().is_err(), "the store is unhealthy after the break");
    }

    /// How many times documents were read from the store (a list reads
    /// none). In-memory store only.
    pub fn loads(&self) -> usize {
        self.memory.as_ref().expect("only the in-memory store counts its reads").reads.load(Ordering::SeqCst)
    }
}

fn scope_of(project: &str, repo: &str) -> ScopeId {
    scope_id(&ProjectId::new(project), &RepoId::new(repo)).expect("a scope id")
}

fn doc_key(key: &str) -> DocKey {
    DocKey::new(key).unwrap_or_else(|e| panic!("{key}: {e}"))
}

/// The in-memory store with a switch: once it is down, every operation is
/// the store's temporary failure. It also counts the document reads.
struct Switched {
    inner: Memory,
    down: AtomicBool,
    reads: AtomicUsize,
}

impl Switched {
    fn up(&self) -> Result<(), PortError> {
        if self.down.load(Ordering::SeqCst) {
            return Err(PortError::Unavailable { message: "the test broke the store".into() });
        }
        Ok(())
    }
}

impl StorePort for Switched {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.up()?;
        self.inner.list(scope, prefix)
    }
    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.up()?;
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.inner.read(scope, keys)
    }
    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.up()?;
        self.inner.commit(scope, commit)
    }
}

impl ServerStore for Switched {
    fn kind(&self) -> &'static str {
        self.inner.kind()
    }
    fn events_after(&self, scope: &ScopeId, after: u64, limit: usize) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        self.up()?;
        self.inner.events_after(scope, after, limit)
    }
    fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        self.up()?;
        self.inner.event_floor(scope)
    }
    fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        self.up()?;
        self.inner.trim_events(scope, through)
    }
    fn probe(&self) -> Result<(), PortError> {
        self.up()?;
        self.inner.probe()
    }
}

/// A store that lets one other writer in while a verb runs: the writer runs
/// once, right before the verb's commit — or earlier, before the first list
/// or read after `reads_first` of them, when that is set.
pub struct Racing {
    inner: TestStore,
    other_writer: Mutex<Option<OtherWriter>>,
    reads_first: Option<usize>,
    reads: AtomicUsize,
}

type OtherWriter = Box<dyn FnOnce(&TestStore) + Send>;

impl Racing {
    fn new(inner: &TestStore, other_writer: OtherWriter, reads_first: Option<usize>) -> Arc<Racing> {
        Arc::new(Racing {
            inner: inner.clone(),
            other_writer: Mutex::new(Some(other_writer)),
            reads_first,
            reads: Default::default(),
        })
    }

    /// The writer lands between the verb's reads and its commit.
    pub fn before_the_commit(
        inner: &TestStore,
        other_writer: impl FnOnce(&TestStore) + Send + 'static,
    ) -> Arc<Racing> {
        Racing::new(inner, Box::new(other_writer), None)
    }

    /// The writer lands right after the first list or read: a route that
    /// reads the scope twice sees the writer's content the second time.
    pub fn after_the_first_read(
        inner: &TestStore,
        other_writer: impl FnOnce(&TestStore) + Send + 'static,
    ) -> Arc<Racing> {
        Racing::new(inner, Box::new(other_writer), Some(1))
    }

    /// The store as the server state takes it.
    pub fn shared(self: &Arc<Self>) -> SharedStore {
        self.clone()
    }

    fn let_the_writer_in(&self) {
        if let Some(other_writer) = self.other_writer.lock().unwrap().take() {
            other_writer(&self.inner);
        }
    }

    /// One more list or read begins.
    fn reading(&self) {
        let taken = self.reads.fetch_add(1, Ordering::SeqCst);
        if self.reads_first.is_some_and(|first| taken >= first) {
            self.let_the_writer_in();
        }
    }
}

impl StorePort for Racing {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.reading();
        self.inner.shared.list(scope, prefix)
    }
    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.reading();
        self.inner.shared.read(scope, keys)
    }
    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.let_the_writer_in();
        self.inner.shared.commit(scope, commit)
    }
}

impl ServerStore for Racing {
    fn kind(&self) -> &'static str {
        self.inner.shared.kind()
    }
    fn events_after(&self, scope: &ScopeId, after: u64, limit: usize) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
        self.inner.shared.events_after(scope, after, limit)
    }
    fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
        self.inner.shared.event_floor(scope)
    }
    fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
        self.inner.shared.trim_events(scope, through)
    }
    fn probe(&self) -> Result<(), PortError> {
        self.inner.shared.probe()
    }
}

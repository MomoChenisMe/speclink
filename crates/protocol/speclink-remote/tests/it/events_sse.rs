//! Client-side SSE consumption contract tests (remote-workspace-data spec):
//! the `events` module subscribes to a project's `/events` stream and yields
//! typed events — invalidation hints (id = outbox seq, scope, resource) and
//! the reset signal — supports `Last-Event-ID` resume, and exposes an abort
//! handle that unblocks the blocking read.
//!
//! These run against a real in-process `speclink-server` (memory store +
//! memory identity), mirroring the harness in the server's own route tests.

use speclink_protocol::events::{InvalidationEvent, InvalidationScope};
use speclink_remote::client::Client;
use speclink_remote::events::{self, RemoteEvent};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, NewInvitation};
use speclink_server::state::{AppState, SharedIdentity, SharedStore};
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

/// Seed change `demo` (schema + four tasks) so `task done` commits land
/// task-completed events in the outbox.
fn seed_change(store: &dyn TeamStore) {
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n- [ ] 1.2 Second\n- [ ] 1.3 Third\n- [ ] 1.4 Fourth\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
}

/// An [`AppState`] over a seeded memory store with the given outbox retention
/// (short heartbeat so idle streams stay lively in tests).
fn state_with_retention(retention: u64) -> AppState {
    let store: SharedStore = {
        let store = Arc::new(MemoryStore::new());
        seed_change(store.as_ref());
        store
    };
    let settings = EventSettings { retention, buffer: 64, heartbeat: Duration::from_millis(100) };
    let events = EventHub::new(store.clone(), settings);
    let identity: SharedIdentity =
        Arc::new(IdentitySqlite::open_memory().expect("in-memory identity store"));
    identity.create_project("demo", "Demo").expect("seed demo project");
    identity.create_repo("demo", "backend", "backend").expect("seed demo repo");
    AppState {
        store,
        identity,
        config: Arc::new(ServerConfig {
            store: StoreConfig::Memory,
            identity: IdentityConfig::Memory,
            public_url: "http://127.0.0.1".to_string(),
            events: EventSettings::default(),
        }),
        events,
    }
}

/// Seed a member user plus a PAT; returns the PAT plaintext to send as bearer.
fn seed_pat(identity: &SharedIdentity) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: "tester@example.com".to_string(),
            display: "Tester <tester@example.com>".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: chrono::Utc::now() + chrono::Duration::days(1),
        })
        .expect("seed invitation");
    let user_id = identity.accept_invitation(&token, "seed-password").expect("seed accept");
    let (_, pat) = identity.create_pat(&user_id, "test", None).expect("seed pat");
    pat
}

/// Start the server on a free loopback port; returns `http://127.0.0.1:<port>`.
fn start(state: AppState) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    listener.set_nonblocking(true).expect("nonblocking");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
            axum::serve(listener, speclink_server::app::router(state)).await.expect("serve");
        });
    });
    format!("http://{addr}")
}

/// The project-scoped base URL the client and the events module both take.
fn project_base(base: &str) -> String {
    format!("{base}/api/speclink/v1/projects/demo")
}

fn client(base: &str, token: &str) -> Client {
    Client::new(&project_base(base), token, Some("backend"))
}

/// What the reader thread fed back: a typed event, a clean end of stream
/// (`next()` returned `Ok(None)`), or a stream failure.
#[derive(Debug)]
enum Fed {
    Event(RemoteEvent),
    End,
    Failed(String),
}

/// A subscription driven on its own thread — the shape the desktop event
/// manager consumes it in: the thread owns the blocking stream, the test holds
/// the abort handle.
struct Feed {
    rx: mpsc::Receiver<Fed>,
    abort: events::AbortHandle,
}

fn feed(stream: events::EventStream) -> Feed {
    let abort = stream.abort_handle();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut stream = stream;
        loop {
            match stream.next() {
                Ok(Some(event)) => {
                    if tx.send(Fed::Event(event)).is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = tx.send(Fed::End);
                    break;
                }
                Err(e) => {
                    let _ = tx.send(Fed::Failed(e.message));
                    break;
                }
            }
        }
    });
    Feed { rx, abort }
}

impl Feed {
    /// The next typed event within `timeout`; panics on end/failure.
    fn next_event(&self, timeout: Duration) -> Option<RemoteEvent> {
        match self.rx.recv_timeout(timeout) {
            Ok(Fed::Event(event)) => Some(event),
            Ok(other) => panic!("expected an event, got {other:?}"),
            Err(_) => None,
        }
    }

    /// The next event, asserted to be an invalidation hint.
    fn expect_invalidate(&self, why: &str) -> InvalidationEvent {
        match self.next_event(Duration::from_secs(3)) {
            Some(RemoteEvent::Invalidate(hint)) => hint,
            other => panic!("{why}: expected an invalidate, got {other:?}"),
        }
    }

    /// End the stream and wait for the reader thread to observe `Ok(None)`.
    fn abort_and_join(&self, why: &str) {
        self.abort.abort();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(left) {
                Ok(Fed::End) => return,
                Ok(Fed::Event(_)) => continue, // drain events already in flight
                Ok(Fed::Failed(msg)) => panic!("{why}: abort surfaced an error: {msg}"),
                Err(_) => panic!("{why}: the blocking read did not unblock after abort"),
            }
        }
    }
}

fn subscribe(base: &str, token: &str, last_event_id: Option<u64>) -> Feed {
    let stream = events::subscribe(&project_base(base), token, Some("backend"), last_event_id)
        .expect("subscribe");
    feed(stream)
}

#[test]
fn a_subscription_yields_typed_invalidations_carrying_the_outbox_sequence() {
    let state = state_with_retention(1024);
    let pat = seed_pat(&state.identity);
    let base = start(state);
    let writer = client(&base, &pat);

    let sub = subscribe(&base, &pat, None);
    std::thread::sleep(Duration::from_millis(80));
    writer.task_done("demo", "1", &[], None).expect("task 1");

    let first = sub.expect_invalidate("a write pushes a typed invalidation");
    assert_eq!(first.scope, InvalidationScope::Change, "a task completion invalidates the change");
    assert_eq!(first.resource_id, "demo", "the hint names the resource");
    assert!(first.revision > 0, "a revision is carried");
    let first_seq: u64 =
        first.event_id.parse().expect("the event id is the numeric outbox sequence");

    writer.task_done("demo", "2", &[], None).expect("task 2");
    let second = sub.expect_invalidate("the next write pushes the next hint");
    assert_eq!(
        second.event_id.parse::<u64>().expect("numeric id"),
        first_seq + 1,
        "event ids advance with the outbox sequence"
    );
}

#[test]
fn a_resume_with_last_event_id_backfills_missed_events_in_order() {
    let state = state_with_retention(1024);
    let pat = seed_pat(&state.identity);
    let base = start(state);
    let writer = client(&base, &pat);

    let sub = subscribe(&base, &pat, None);
    std::thread::sleep(Duration::from_millis(80));
    writer.task_done("demo", "1", &[], None).expect("task 1");
    let first = sub.expect_invalidate("the pre-disconnect event arrives");
    let first_seq: u64 = first.event_id.parse().expect("numeric id");
    sub.abort_and_join("disconnect before the missed writes");

    // Three writes happen while disconnected.
    writer.task_done("demo", "2", &[], None).expect("task 2");
    writer.task_done("demo", "3", &[], None).expect("task 3");
    writer.task_done("demo", "4", &[], None).expect("task 4");

    // Resume from the last seen id: exactly the three missed hints, in order.
    let resumed = subscribe(&base, &pat, Some(first_seq));
    let mut seqs = Vec::new();
    for _ in 0..3 {
        let hint = resumed.expect_invalidate("a backfilled event arrives");
        seqs.push(hint.event_id.parse::<u64>().expect("numeric id"));
    }
    let want: Vec<u64> = (1..=3).map(|d| first_seq + d).collect();
    assert_eq!(seqs, want, "the gap is backfilled in order, no loss, no repeat");
    assert!(
        resumed.next_event(Duration::from_millis(400)).is_none(),
        "no extra replay beyond the gap"
    );
}

#[test]
fn a_resume_below_the_server_retention_yields_a_reset_signal_first() {
    let state = state_with_retention(2);
    let pat = seed_pat(&state.identity);
    let base = start(state);
    let writer = client(&base, &pat);

    // An active subscriber drives retention: four writes ack the floor past 1.
    let sub = subscribe(&base, &pat, None);
    std::thread::sleep(Duration::from_millis(80));
    for task in ["1", "2", "3", "4"] {
        writer.task_done("demo", task, &[], None).expect("task done");
    }
    for _ in 0..4 {
        sub.expect_invalidate("a live event arrives");
    }
    sub.abort_and_join("disconnect after the floor advanced");

    // Resume from a cleaned cursor: reset is the first typed event.
    let resumed = subscribe(&base, &pat, Some(1));
    match resumed.next_event(Duration::from_secs(3)) {
        Some(RemoteEvent::Reset) => {}
        other => panic!("a cleaned cursor yields a reset signal first, got {other:?}"),
    }

    // After the reset the stream keeps yielding new writes.
    writer.task_undone("demo", "1").expect("a fresh write");
    resumed.expect_invalidate("new writes stream after the reset");
}

#[test]
fn an_abort_unblocks_the_blocking_read_and_ends_the_stream() {
    let state = state_with_retention(1024);
    let pat = seed_pat(&state.identity);
    let base = start(state);

    let sub = subscribe(&base, &pat, None);
    // Idle stream: heartbeat comments are consumed internally, never surfaced
    // as typed events.
    assert!(
        sub.next_event(Duration::from_millis(400)).is_none(),
        "heartbeats are not surfaced as events"
    );
    // Abort from another thread unblocks the read; `next()` returns `Ok(None)`.
    sub.abort_and_join("abort unblocks an idle blocking read");
}

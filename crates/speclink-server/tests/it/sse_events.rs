//! `/events` SSE endpoint (server-event-stream spec). A project-scoped stream
//! that reuses the bearer/binding precondition, pushes outbox invalidation
//! hints (SSE id = outbox seq, data = the DTO, no document content), resumes
//! from `Last-Event-ID`, signals reset for a cleaned cursor, and heartbeats.

use crate::common;

use speclink_remote::client::Client;
use speclink_server::events::EventSettings;
use speclink_server::state::SharedStore;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use speclink_store_sqlite::SqliteTeamStore;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

const EVENTS_PATH: &str = "/api/speclink/v1/projects/demo/events";

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

/// Seed change `demo` (schema + four tasks) into `store` so `task done` commits
/// land task-completed events in the outbox.
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

/// A memory store seeded with change `demo`.
fn seeded_store() -> Arc<MemoryStore> {
    let store = Arc::new(MemoryStore::new());
    seed_change(store.as_ref());
    store
}

/// The scope's current ETag from `/sync-state` (the polling convergence bedrock).
fn sync_state_etag(base: &str, token: &str) -> String {
    ureq::get(&format!("{base}/api/speclink/v1/projects/demo/sync-state"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", "1")
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("sync-state")
        .header("etag")
        .unwrap_or_default()
        .to_string()
}

fn client(base: &str, token: &str) -> Client {
    Client::new(&format!("{base}/api/speclink/v1/projects/demo"), token, Some("backend"))
}

/// Fast test event settings: short heartbeat, ample live buffer.
fn fast_events() -> EventSettings {
    EventSettings { retention: 1024, buffer: 64, heartbeat: Duration::from_millis(100) }
}

/// One parsed SSE frame: a comment (heartbeat) or a dispatched event.
#[derive(Debug, Clone)]
enum Frame {
    Comment(String),
    Event { id: Option<String>, event: Option<String>, data: String },
}

/// A live SSE subscription. A reader thread parses frames off the chunked body
/// (ureq decodes the transfer encoding) into a channel the test pulls with a
/// timeout; the thread exits on its own once the connection or receiver drops.
struct Sub {
    rx: mpsc::Receiver<Frame>,
}

impl Sub {
    /// Open `/events`. `Err(status)` on a non-2xx precondition failure.
    fn open(base: &str, token: &str, last_event_id: Option<&str>) -> Result<Sub, u16> {
        let url = format!("{base}{EVENTS_PATH}");
        let mut req = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("X-Speclink-Api-Version", "1")
            .set("X-Speclink-Repo", "backend")
            .set("Accept", "text/event-stream");
        if let Some(id) = last_event_id {
            req = req.set("Last-Event-ID", id);
        }
        match req.call() {
            Ok(resp) => {
                let reader = BufReader::new(resp.into_reader());
                let (tx, rx) = mpsc::channel();
                std::thread::spawn(move || parse_frames(reader, tx));
                Ok(Sub { rx })
            }
            Err(ureq::Error::Status(code, _)) => Err(code),
            Err(e) => panic!("transport error: {e}"),
        }
    }

    /// The next frame of any kind within `timeout`.
    fn next(&self, timeout: Duration) -> Option<Frame> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// The next dispatched event within `timeout`, skipping heartbeat comments.
    fn next_event(&self, timeout: Duration) -> Option<Frame> {
        let deadline = Instant::now() + timeout;
        while let Ok(frame) = self.rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            if let Frame::Event { .. } = frame {
                return Some(frame);
            }
        }
        None
    }
}

/// Parse the SSE body line by line, sending each comment and each blank-line
/// terminated event to `tx`. Ends on EOF, a read error, or a dropped receiver.
fn parse_frames<R: BufRead>(mut reader: R, tx: mpsc::Sender<Frame>) {
    let mut id = None;
    let mut event = None;
    let mut data: Vec<String> = Vec::new();
    let mut have_fields = false;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let text = line.trim_end_matches(['\r', '\n']);
        if text.is_empty() {
            if have_fields {
                let frame = Frame::Event { id: id.take(), event: event.take(), data: data.join("\n") };
                data.clear();
                have_fields = false;
                if tx.send(frame).is_err() {
                    break;
                }
            }
            continue;
        }
        if let Some(comment) = text.strip_prefix(':') {
            if tx.send(Frame::Comment(comment.trim_start().to_string())).is_err() {
                break;
            }
            continue;
        }
        let (key, value) = match text.split_once(':') {
            Some((k, v)) => (k, v.strip_prefix(' ').unwrap_or(v)),
            None => (text, ""),
        };
        have_fields = true;
        match key {
            "id" => id = Some(value.to_string()),
            "event" => event = Some(value.to_string()),
            "data" => data.push(value.to_string()),
            _ => {}
        }
    }
}

/// Destructure an event frame into (id, event type, data).
fn parts(frame: Frame) -> (Option<String>, Option<String>, String) {
    match frame {
        Frame::Event { id, event, data } => (id, event, data),
        Frame::Comment(c) => panic!("expected an event, got a comment: {c}"),
    }
}

#[test]
fn events_runs_the_same_auth_precondition_as_every_route() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (member_pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let (stranger_pat, _) =
        common::seed_named_pat(&state.identity, "stranger@example.com", "Stranger", &["other"]);
    let base = common::start(state);

    assert_eq!(Sub::open(&base, "wrong-token", None).err(), Some(401), "an invalid token is 401");
    assert_eq!(Sub::open(&base, &stranger_pat, None).err(), Some(403), "a non-member is 403");
    assert!(Sub::open(&base, &member_pat, None).is_ok(), "a member's subscription opens");
}

#[test]
fn a_write_pushes_an_invalidation_hint_carrying_no_document_content() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let sub = Sub::open(&base, &pat, None).expect("subscribe");
    std::thread::sleep(Duration::from_millis(80));
    client(&base, &pat).task_done("demo", "1", &[]).expect("task done");

    let (id, event, data) = parts(sub.next_event(Duration::from_secs(3)).expect("an invalidation arrives"));
    let id = id.expect("the SSE id is the outbox sequence");
    assert_eq!(event.as_deref(), Some("invalidate"), "the event type is invalidate");
    let dto: serde_json::Value = serde_json::from_str(&data).expect("data is the JSON DTO");
    assert_eq!(dto["eventId"], id, "the DTO event id echoes the SSE id");
    assert_eq!(dto["scope"], "change", "a task completion invalidates the change");
    assert_eq!(dto["resourceId"], "demo");
    assert!(dto.get("revision").and_then(|r| r.as_u64()).is_some(), "a revision is carried");
    assert!(
        !data.contains("First") && !data.contains("tasks"),
        "the hint carries no document content: {data}"
    );
}

#[test]
fn every_subscriber_receives_the_write_exactly_once() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let a = Sub::open(&base, &pat, None).expect("subscribe a");
    let b = Sub::open(&base, &pat, None).expect("subscribe b");
    std::thread::sleep(Duration::from_millis(80));
    client(&base, &pat).task_done("demo", "1", &[]).expect("task done");

    let (id_a, _, _) = parts(a.next_event(Duration::from_secs(3)).expect("a receives the write"));
    let (id_b, _, _) = parts(b.next_event(Duration::from_secs(3)).expect("b receives the write"));
    assert_eq!(id_a, id_b, "both see the same outbox sequence");
    assert!(a.next_event(Duration::from_millis(400)).is_none(), "a receives it exactly once");
    assert!(b.next_event(Duration::from_millis(400)).is_none(), "b receives it exactly once");
}

#[test]
fn a_reconnect_with_last_event_id_backfills_the_gap_without_loss_or_repeat() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let writer = client(&base, &pat);

    let sub = Sub::open(&base, &pat, None).expect("subscribe");
    std::thread::sleep(Duration::from_millis(80));
    writer.task_done("demo", "1", &[]).expect("task 1");
    let first_id = parts(sub.next_event(Duration::from_secs(3)).expect("first event"))
        .0
        .expect("the first event carries an id");
    drop(sub); // disconnect

    // Three writes happen while disconnected.
    writer.task_done("demo", "2", &[]).expect("task 2");
    writer.task_done("demo", "3", &[]).expect("task 3");
    writer.task_done("demo", "4", &[]).expect("task 4");

    // Reconnect from the last seen id: the three missed events arrive in order.
    let resumed = Sub::open(&base, &pat, Some(&first_id)).expect("resubscribe");
    let mut ids = Vec::new();
    for _ in 0..3 {
        let (id, _, _) = parts(resumed.next_event(Duration::from_secs(3)).expect("a backfilled event"));
        ids.push(id.unwrap());
    }
    assert!(resumed.next_event(Duration::from_millis(400)).is_none(), "no extra replay");
    let base_seq: u64 = first_id.parse().unwrap();
    let want: Vec<String> = (1..=3).map(|d| (base_seq + d).to_string()).collect();
    assert_eq!(ids, want, "the gap is backfilled in order, no loss, no repeat");
}

#[test]
fn an_illegal_last_event_id_is_treated_as_absent() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let writer = client(&base, &pat);

    // Produce history first so a genuine resume would replay something.
    writer.task_done("demo", "1", &[]).expect("task 1");

    let sub = Sub::open(&base, &pat, Some("not-a-number")).expect("subscribe");
    std::thread::sleep(Duration::from_millis(120));
    // No backfill and no reset — an illegal id starts fresh from the newest.
    assert!(sub.next_event(Duration::from_millis(300)).is_none(), "no history is replayed");

    // A new write still arrives.
    writer.task_done("demo", "2", &[]).expect("task 2");
    let (_, event, _) = parts(sub.next_event(Duration::from_secs(3)).expect("a new write arrives"));
    assert_eq!(event.as_deref(), Some("invalidate"));
}

#[test]
fn a_reconnect_below_the_cleaned_floor_gets_a_reset_first() {
    let store = seeded_store();
    let settings = EventSettings { retention: 2, buffer: 64, heartbeat: Duration::from_millis(100) };
    let state = common::state_with_event_settings(store, settings);
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let writer = client(&base, &pat);

    // An active subscriber drives retention: four writes, keep-only-2 acks to 2.
    let sub = Sub::open(&base, &pat, None).expect("subscribe");
    std::thread::sleep(Duration::from_millis(80));
    for t in ["1", "2", "3", "4"] {
        writer.task_done("demo", t, &[]).expect("task done");
    }
    for _ in 0..4 {
        sub.next_event(Duration::from_secs(3)).expect("live event");
    }

    // Reconnect from an id below the cleaned floor: reset is the first frame.
    let resumed = Sub::open(&base, &pat, Some("1")).expect("resubscribe");
    let (_, event, _) = parts(resumed.next_event(Duration::from_secs(3)).expect("a frame"));
    assert_eq!(event.as_deref(), Some("reset"), "a cleaned cursor gets a reset signal first");

    // After reset, a new write is pushed as usual.
    writer.task_done("demo", "4", &[]).expect("task 4 idempotent flip is fine as a new write");
}

#[test]
fn an_idle_stream_sends_heartbeat_comments() {
    let store = seeded_store();
    let state = common::state_with_event_settings(store, fast_events());
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let sub = Sub::open(&base, &pat, None).expect("subscribe");
    // No writes: within a few heartbeat intervals a comment must arrive.
    let mut saw_comment = false;
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Some(Frame::Comment(_)) = sub.next(Duration::from_millis(300)) {
            saw_comment = true;
            break;
        }
    }
    assert!(saw_comment, "an idle subscription is kept alive by comment heartbeats");
}

#[test]
fn dropped_events_converge_via_reset_and_full_reread_on_a_sqlite_server() {
    // A real SQLite-backed server: a fresh file initializes its own schema.
    let dir = tempfile::tempdir().expect("tempdir");
    let store: SharedStore =
        Arc::new(SqliteTeamStore::open(dir.path().join("store.db")).expect("open sqlite store"));
    seed_change(store.as_ref());
    // Retention 2: after four writes the earliest sequences are acked-cleaned.
    let settings = EventSettings { retention: 2, buffer: 64, heartbeat: Duration::from_millis(100) };
    let state = common::state_with_event_settings(store, settings);
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let writer = client(&base, &pat);

    // Subscribe, then drive four writes; the active subscriber advances the
    // acked floor past the earliest sequences.
    let sub = Sub::open(&base, &pat, None).expect("subscribe");
    std::thread::sleep(Duration::from_millis(80));
    for task in ["1", "2", "3", "4"] {
        writer.task_done("demo", task, &[]).expect("task done");
    }
    for _ in 0..4 {
        sub.next_event(Duration::from_secs(3)).expect("live event");
    }
    drop(sub); // disconnect

    // Reconnect from a cleaned sequence → reset is the first frame.
    let resumed = Sub::open(&base, &pat, Some("1")).expect("resubscribe");
    let (_, event, _) = parts(resumed.next_event(Duration::from_secs(3)).expect("a frame"));
    assert_eq!(event.as_deref(), Some("reset"), "a cleaned cursor gets a reset signal");

    // Converge by the polling bedrock: /sync-state advertises an ETag and a
    // query-route full re-read reflects every committed change.
    assert!(!sync_state_etag(&base, &pat).is_empty(), "sync-state advertises an ETag");
    let tasks = writer.get_artifact("demo", "tasks").expect("re-read tasks").content;
    assert_eq!(
        tasks.matches("- [x]").count(),
        4,
        "the full re-read reflects all four committed completions: {tasks}"
    );

    // Re-subscribe from the newest sequence and keep hearing new writes.
    let tail = Sub::open(&base, &pat, None).expect("resubscribe fresh");
    std::thread::sleep(Duration::from_millis(80));
    writer.task_undone("demo", "1").expect("a fresh write");
    let (_, event, _) = parts(tail.next_event(Duration::from_secs(3)).expect("a new event"));
    assert_eq!(event.as_deref(), Some("invalidate"), "new writes stream from the newest sequence");
}

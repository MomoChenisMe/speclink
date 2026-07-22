//! The SSE event stream: outbox → invalidation broadcast (design 決策 1–4).
//!
//! One mapping single-point turns a stored domain event into the protocol's
//! invalidation hint (决策 1). An event whose name this server does not
//! recognize is emitted under the `unknown` category rather than swallowed —
//! a spurious re-read is safe, a missed invalidation is not.

use crate::state::SharedStore;
use serde_json::Value;
use speclink_protocol::events::{InvalidationEvent, InvalidationScope};
use speclink_store::{OutboxCursor, OutboxEntry, Scope, StoreError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{broadcast, Notify};

/// Map one outbox entry to its invalidation hint. The event id is the scope's
/// outbox sequence (so a client's `Last-Event-ID` resumes from it), the scope
/// category and resource id come from the event name and payload, and the
/// revision is the commit that produced it.
pub fn invalidation_of(entry: &OutboxEntry) -> InvalidationEvent {
    let (scope, resource_id) = classify(&entry.record.name, &entry.record.payload);
    InvalidationEvent {
        event_id: entry.seq.to_string(),
        scope,
        resource_id,
        revision: entry.revision.0,
    }
}

/// The single point that classifies a domain event name into a resource
/// category and identity. `change-archived` points at the specs it promoted;
/// every other change-family event at the change; every discussion-family
/// event at the discussion. An unrecognized name is `unknown` with no
/// identity — the client re-reads everything, never misses.
fn classify(name: &str, payload: &Value) -> (InvalidationScope, String) {
    match name {
        "change-created"
        | "artifact-created"
        | "task-completed"
        | "task-uncompleted"
        | "change-claimed"
        | "change-marked-in-progress"
        | "change-discarded" => (InvalidationScope::Change, str_field(payload, "change")),
        "change-archived" => (InvalidationScope::Spec, str_field(payload, "change")),
        "discussion-created"
        | "discussion-context-set"
        | "discussion-round-added"
        | "discussion-concluded"
        | "discussion-promoted"
        | "discussion-linked"
        | "discussion-sealed"
        | "discussion-archived"
        | "discussion-discarded" => (InvalidationScope::Discussion, str_field(payload, "slug")),
        _ => (
            InvalidationScope::Unknown("unknown".to_string()),
            String::new(),
        ),
    }
}

/// Read a string field from an event payload, empty string if absent.
fn str_field(payload: &Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// Tunables for the event stream. An absent config section uses these defaults
/// (决策 3–4); a malformed one fails startup like any other config error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSettings {
    /// How many most-recent outbox entries each scope keeps resumable; older
    /// ones are acked so the driver can clean them (决策 3).
    pub retention: u64,
    /// Per-connection live buffer. A subscriber that falls this far behind is
    /// dropped rather than backing up memory (慢消費者有界處置, 决策 4).
    pub buffer: usize,
    /// Interval between SSE comment heartbeats.
    pub heartbeat: Duration,
}

impl Default for EventSettings {
    fn default() -> Self {
        EventSettings {
            retention: 1024,
            buffer: 256,
            heartbeat: Duration::from_secs(15),
        }
    }
}

/// One event on the live push channel: the invalidation hint plus its outbox
/// sequence, the dedup key between a resume backfill and the live tail.
#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub seq: u64,
    pub event: InvalidationEvent,
}

/// How a (re)subscription starts: an optional reset signal, the entries to
/// backfill before tailing, and the sequence at or below which a live event is
/// a duplicate of the backfill (or of already-delivered history).
#[derive(Debug, Clone)]
pub struct ResumePlan {
    pub reset: bool,
    pub backfill: Vec<InvalidationEvent>,
    pub cursor: u64,
}

/// A joined subscription: the live receiver plus the start plan, taken from one
/// consistent outbox snapshot so the tail never gaps or duplicates the
/// backfill.
pub struct Subscription {
    pub rx: broadcast::Receiver<StreamEvent>,
    pub plan: ResumePlan,
}

/// The live-broadcast channel and commit signal for one active scope.
struct ScopeChannel {
    tx: broadcast::Sender<StreamEvent>,
    notify: Arc<Notify>,
}

/// Per-scope outbox → SSE broadcaster registry (决策 2). Each active scope has
/// one broadcaster fed only from its outbox — never from in-memory events — so
/// the live tail and a resume read the same sequence. The write path notifies
/// on commit; an unsubscribed scope has no broadcaster and stays idle.
pub struct EventHub {
    store: SharedStore,
    settings: EventSettings,
    scopes: Mutex<HashMap<Scope, ScopeChannel>>,
}

impl EventHub {
    pub fn new(store: SharedStore, settings: EventSettings) -> Arc<Self> {
        Arc::new(EventHub {
            store,
            settings,
            scopes: Mutex::new(HashMap::new()),
        })
    }

    /// The stream tunables (heartbeat interval and buffer size the route needs).
    pub fn settings(&self) -> &EventSettings {
        &self.settings
    }

    /// Signal that `scope` committed. Its broadcaster (if a subscription created
    /// one) wakes and drains the new outbox entries. No broadcaster means no
    /// subscribers — the events wait in the outbox for a future resume.
    pub fn notify(&self, scope: &Scope) {
        if let Some(ch) = self.scopes.lock().unwrap().get(scope) {
            ch.notify.notify_one();
        }
    }

    /// Join `scope`'s live push channel and compute the start plan from one
    /// outbox snapshot. Creates and starts the scope's broadcaster on first
    /// subscription. The receiver is taken before the snapshot is read, so any
    /// event past the plan's cursor is guaranteed to reach the tail.
    ///
    /// Synchronous (it reads the store); the route runs it on the blocking pool.
    pub fn subscribe(
        self: &Arc<Self>,
        scope: &Scope,
        last_event_id: Option<u64>,
    ) -> Result<Subscription, StoreError> {
        let (tx, notify, is_new) = {
            let mut scopes = self.scopes.lock().unwrap();
            match scopes.get(scope) {
                Some(ch) => (ch.tx.clone(), ch.notify.clone(), false),
                None => {
                    let (tx, _) = broadcast::channel(self.settings.buffer);
                    let notify = Arc::new(Notify::new());
                    scopes.insert(
                        scope.clone(),
                        ScopeChannel {
                            tx: tx.clone(),
                            notify: notify.clone(),
                        },
                    );
                    (tx, notify, true)
                }
            }
        };
        let rx = tx.subscribe();

        // One snapshot drives both the plan and (for a new scope) the pump's
        // start cursor, so the two never disagree on where "new" begins.
        let acked = self.store.outbox_acked(scope)?.0;
        let newest = self.newest_seq(scope, acked)?;
        let plan = match last_event_id {
            Some(l) if l < acked => ResumePlan {
                reset: true,
                backfill: Vec::new(),
                cursor: newest,
            },
            Some(l) => {
                let backfill = self
                    .store
                    .read_outbox(scope, OutboxCursor(l))?
                    .iter()
                    .map(invalidation_of)
                    .collect();
                ResumePlan {
                    reset: false,
                    backfill,
                    cursor: l,
                }
            }
            None => ResumePlan {
                reset: false,
                backfill: Vec::new(),
                cursor: newest,
            },
        };

        if is_new {
            self.clone().spawn_pump(scope.clone(), tx, notify, newest);
        }
        Ok(Subscription { rx, plan })
    }

    /// The newest outbox sequence for `scope`, given its acked floor. Reads only
    /// the retained tail, so it stays bounded by the retention window.
    fn newest_seq(&self, scope: &Scope, acked: u64) -> Result<u64, StoreError> {
        let tail = self.store.read_outbox(scope, OutboxCursor(acked))?;
        Ok(tail.last().map(|e| e.seq).unwrap_or(acked))
    }

    /// Spawn the scope's pump: idle until notified, then drain new outbox
    /// entries to the broadcast channel (决策 2) and ack past the retention
    /// window so the driver can clean older entries (决策 3).
    fn spawn_pump(
        self: Arc<Self>,
        scope: Scope,
        tx: broadcast::Sender<StreamEvent>,
        notify: Arc<Notify>,
        start: u64,
    ) {
        tokio::spawn(async move {
            let mut cursor = start;
            loop {
                notify.notified().await;
                let entries = match self.read_from(&scope, cursor).await {
                    Ok(entries) => entries,
                    Err(_) => continue, // a transient read failure retries on the next notify
                };
                for entry in &entries {
                    cursor = entry.seq;
                    let _ = tx.send(StreamEvent {
                        seq: entry.seq,
                        event: invalidation_of(entry),
                    });
                }
                if cursor > self.settings.retention {
                    self.ack(&scope, cursor - self.settings.retention).await;
                }
            }
        });
    }

    /// Read outbox entries after `from` on the blocking pool.
    async fn read_from(&self, scope: &Scope, from: u64) -> Result<Vec<OutboxEntry>, StoreError> {
        let store = self.store.clone();
        let scope = scope.clone();
        tokio::task::spawn_blocking(move || store.read_outbox(&scope, OutboxCursor(from)))
            .await
            .expect("outbox read task")
    }

    /// Ack the outbox up to `up_to` on the blocking pool, best-effort.
    async fn ack(&self, scope: &Scope, up_to: u64) {
        let store = self.store.clone();
        let scope = scope.clone();
        let _ = tokio::task::spawn_blocking(move || store.ack_outbox(&scope, OutboxCursor(up_to)))
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use speclink_store::{EventRecord, Revision};

    /// An outbox entry (seq 7, revision 42) carrying `name` and `payload`.
    fn entry(name: &str, payload: Value) -> OutboxEntry {
        OutboxEntry {
            seq: 7,
            revision: Revision(42),
            record: EventRecord {
                name: name.to_string(),
                payload,
                actor: "Tester".to_string(),
                at: DateTime::from_timestamp(0, 0).unwrap(),
            },
        }
    }

    #[test]
    fn every_domain_event_name_maps_to_a_fixed_category_and_resource() {
        use serde_json::json;
        // (event name, payload, expected category, expected resource id)
        let cases: &[(&str, Value, InvalidationScope, &str)] = &[
            (
                "change-created",
                json!({ "change": "add-auth" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "artifact-created",
                json!({ "change": "add-auth", "artifact": "proposal.md" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "task-completed",
                json!({ "change": "add-auth", "taskId": "1" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "task-uncompleted",
                json!({ "change": "add-auth", "taskId": "1" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-claimed",
                json!({ "change": "add-auth" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-marked-in-progress",
                json!({ "change": "add-auth" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-discarded",
                json!({ "change": "add-auth" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-archived",
                json!({ "change": "add-auth", "datedName": "2026-07-14-add-auth" }),
                InvalidationScope::Spec,
                "add-auth",
            ),
            (
                "discussion-created",
                json!({ "slug": "topic" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-context-set",
                json!({ "slug": "topic" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-round-added",
                json!({ "slug": "topic", "round": 2 }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-concluded",
                json!({ "slug": "topic" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-promoted",
                json!({ "slug": "topic", "change": "add-auth" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-linked",
                json!({ "slug": "topic", "change": "add-auth" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-sealed",
                json!({ "slug": "topic", "change": "add-auth" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-archived",
                json!({ "slug": "topic" }),
                InvalidationScope::Discussion,
                "topic",
            ),
            (
                "discussion-discarded",
                json!({ "slug": "topic" }),
                InvalidationScope::Discussion,
                "topic",
            ),
        ];
        for (name, payload, want_scope, want_resource) in cases {
            let ev = invalidation_of(&entry(name, payload.clone()));
            assert_eq!(&ev.scope, want_scope, "{name} maps to the wrong category");
            assert_eq!(
                ev.resource_id, *want_resource,
                "{name} maps to the wrong resource"
            );
            assert_eq!(
                ev.event_id, "7",
                "{name} carries its outbox sequence as the event id"
            );
            assert_eq!(ev.revision, 42, "{name} carries the commit revision");
        }
    }

    #[test]
    fn an_unrecognized_event_name_is_emitted_under_the_unknown_category() {
        // Not swallowed — a future event name still invalidates; the client
        // over-reads rather than missing it.
        let ev = invalidation_of(&entry(
            "widget-frobbed",
            serde_json::json!({ "widget": "x" }),
        ));
        assert_eq!(ev.scope, InvalidationScope::Unknown("unknown".to_string()));
        assert_eq!(
            ev.resource_id, "",
            "an unmapped event has no resource identity"
        );
        assert_eq!(ev.event_id, "7");
    }

    // --- broadcaster layer (design 決策 2–3): driven directly, no HTTP ---

    use crate::state::SharedStore;
    use serde_json::json;
    use speclink_store::memory::MemoryStore;
    use speclink_store::{
        CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast;

    fn scope() -> Scope {
        Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
    }

    /// Commit one event `name`/`payload` (seq `n`, with a unique doc write so
    /// repeated commits do not collide) straight to the store's outbox.
    fn commit_event(store: &MemoryStore, n: u64, name: &str, payload: Value) {
        let mut uow = store
            .begin_unit_of_work(
                &scope(),
                CommandContext {
                    command: "t".into(),
                    actor: "t".into(),
                },
            )
            .expect("begin uow");
        uow.create(
            DocumentId::ChangeArtifact {
                change: "demo".into(),
                artifact: format!("a{n}.md"),
            },
            "x",
        );
        store
            .commit(
                uow,
                vec![EventRecord {
                    name: name.into(),
                    payload,
                    actor: "t".into(),
                    at: DateTime::from_timestamp(0, 0).unwrap(),
                }],
            )
            .expect("commit");
    }

    /// Event settings with an explicit retention and small, fast test bounds.
    fn settings(retention: u64) -> EventSettings {
        EventSettings {
            retention,
            buffer: 64,
            heartbeat: Duration::from_millis(50),
        }
    }

    /// Receive one broadcast event, failing if none arrives within 2s.
    async fn recv(rx: &mut broadcast::Receiver<StreamEvent>) -> StreamEvent {
        tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("an event arrives within 2s")
            .expect("the channel stays open")
    }

    /// Poll `cond` until true, failing after ~2s.
    async fn eventually(mut cond: impl FnMut() -> bool) {
        for _ in 0..200 {
            if cond() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("condition not met within timeout");
    }

    #[tokio::test]
    async fn a_commit_notification_pushes_outbox_events_in_monotonic_sequence() {
        let store = Arc::new(MemoryStore::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(1024));
        let mut rx = hub.subscribe(&scope(), None).expect("subscribe").rx;

        commit_event(
            &store,
            1,
            "task-completed",
            json!({ "change": "demo", "taskId": "1" }),
        );
        hub.notify(&scope());
        commit_event(
            &store,
            2,
            "task-completed",
            json!({ "change": "demo", "taskId": "2" }),
        );
        hub.notify(&scope());

        let first = recv(&mut rx).await;
        let second = recv(&mut rx).await;
        assert!(
            second.seq > first.seq,
            "the sequence is monotonic: {} then {}",
            first.seq,
            second.seq
        );
        // The pushed event is what read_outbox → invalidation_of yields.
        assert_eq!(first.event.scope, InvalidationScope::Change);
        assert_eq!(first.event.resource_id, "demo");
        assert_eq!(
            first.event.event_id,
            first.seq.to_string(),
            "the event id is the outbox seq"
        );
    }

    #[tokio::test]
    async fn retention_acks_everything_older_than_the_kept_window() {
        let store = Arc::new(MemoryStore::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(3));
        let mut rx = hub.subscribe(&scope(), None).expect("subscribe").rx;

        for i in 1..=5 {
            commit_event(
                &store,
                i,
                "task-completed",
                json!({ "change": "demo", "taskId": i.to_string() }),
            );
        }
        hub.notify(&scope());
        for _ in 0..5 {
            recv(&mut rx).await;
        }
        // retention = 3, newest seq = 5 → everything up to 5 - 3 = 2 is acked.
        let s = store.clone();
        eventually(move || s.outbox_acked(&scope()).unwrap() == OutboxCursor(2)).await;
    }

    #[tokio::test]
    async fn a_cursor_below_the_acked_floor_resumes_with_a_reset() {
        let store = Arc::new(MemoryStore::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(3));
        // Start the pump so retention advances.
        let _sub = hub.subscribe(&scope(), None).expect("subscribe");

        for i in 1..=5 {
            commit_event(
                &store,
                i,
                "task-completed",
                json!({ "change": "demo", "taskId": i.to_string() }),
            );
        }
        hub.notify(&scope());
        let s = store.clone();
        eventually(move || s.outbox_acked(&scope()).unwrap() == OutboxCursor(2)).await;

        // A Last-Event-ID below the acked floor (2) can no longer resume: reset.
        let reset_plan = hub.subscribe(&scope(), Some(1)).expect("subscribe").plan;
        assert!(
            reset_plan.reset,
            "a cursor below the acked floor gets a reset"
        );
        assert!(
            reset_plan.backfill.is_empty(),
            "reset does not resend cleaned entries"
        );

        // A Last-Event-ID at or above the floor resumes, backfilling what follows.
        let resume_plan = hub.subscribe(&scope(), Some(3)).expect("subscribe").plan;
        assert!(
            !resume_plan.reset,
            "a cursor within the kept window resumes without reset"
        );
        assert_eq!(
            resume_plan.backfill.len(),
            2,
            "entries 4 and 5 are backfilled"
        );
        assert_eq!(resume_plan.backfill[0].event_id, "4");
        assert_eq!(resume_plan.backfill[1].event_id, "5");
    }

    #[tokio::test]
    async fn a_slow_subscriber_overflows_and_is_dropped_while_a_fast_one_keeps_up() {
        let store = Arc::new(MemoryStore::new());
        let shared: SharedStore = store.clone();
        // A live buffer of 4: a receiver more than 4 events behind is lagged.
        let hub = EventHub::new(
            shared,
            EventSettings {
                retention: 1024,
                buffer: 4,
                heartbeat: Duration::from_millis(50),
            },
        );
        let mut fast = hub.subscribe(&scope(), None).expect("subscribe fast").rx;
        let mut slow = hub.subscribe(&scope(), None).expect("subscribe slow").rx;

        // Twenty writes. The fast subscriber drains each as it lands (staying
        // within its buffer); the slow one never reads.
        for i in 1..=20u64 {
            commit_event(
                &store,
                i,
                "task-completed",
                json!({ "change": "demo", "taskId": i.to_string() }),
            );
            hub.notify(&scope());
            let ev = recv(&mut fast).await;
            assert_eq!(
                ev.seq, i,
                "the fast subscriber keeps up, gapless and in order"
            );
        }

        // The slow subscriber overflowed its bounded buffer: it is lagged
        // (dropped), never backing memory up unboundedly.
        let dropped = loop {
            match slow.recv().await {
                Ok(_) => continue,
                Err(broadcast::error::RecvError::Lagged(_)) => break true,
                Err(broadcast::error::RecvError::Closed) => break false,
            }
        };
        assert!(
            dropped,
            "the slow subscriber is dropped once its buffer overflows"
        );

        // A dropped subscriber recovers by reconnecting with its Last-Event-ID
        // (retention 1024 keeps everything, so it resumes rather than resets).
        let plan = hub.subscribe(&scope(), Some(2)).expect("resubscribe").plan;
        assert!(!plan.reset, "within retention the reconnect resumes");
        assert!(
            !plan.backfill.is_empty(),
            "the gap since its last id is backfilled"
        );
    }
}

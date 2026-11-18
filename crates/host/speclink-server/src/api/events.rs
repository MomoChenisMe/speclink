//! The SSE event stream: event log → invalidation broadcast (design 決策 1–4).
//!
//! One mapping single-point turns a stored domain event into the protocol's
//! invalidation hint (决策 1). An event whose name this server does not
//! recognize is emitted under the `unknown` category rather than swallowed —
//! a spurious re-read is safe, a missed invalidation is not.

use crate::state::SharedStore;
use crate::storage::{each_event_batch, newest_event};
use speclink_protocol::events::InvalidationEvent;
use speclink_store_port::{PortError, RecordedEvent, ScopeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{broadcast, Notify};

/// Map one kept event to its invalidation hint. The event id is the event's
/// sequence in its scope (so a client's `Last-Event-ID` resumes from it), the
/// scope category and resource id come from the event name and payload, and
/// the revision is the scope version of the commit that produced it.
pub fn invalidation_of(seq: u64, event: &RecordedEvent) -> InvalidationEvent {
    // The one table of event kind → resource is the host kit's.
    let (scope, resource_id) = speclink_host::commit::event_resource(&event.name, &event.payload);
    InvalidationEvent {
        event_id: seq.to_string(),
        scope,
        resource_id,
        revision: event.version.get(),
    }
}

/// Tunables for the event stream. An absent config section uses these defaults
/// (决策 3–4); a malformed one fails startup like any other config error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSettings {
    /// How many most-recent events each scope keeps resumable; older ones
    /// are removed from the store (决策 3).
    pub retention: u64,
    /// Per-connection live buffer. A subscriber that falls this far behind is
    /// dropped rather than backing up memory (慢消費者有界處置, 决策 4). Also
    /// how many events one read of the store asks for.
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

/// One event on the live push channel: the invalidation hint plus its
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
/// consistent read of the event log so the tail never gaps or duplicates the
/// backfill.
pub struct Subscription {
    pub rx: broadcast::Receiver<StreamEvent>,
    pub plan: ResumePlan,
}

/// The live-broadcast channel and commit signal for one active scope.
struct ScopeChannel {
    tx: broadcast::Sender<StreamEvent>,
    notify: Arc<Notify>,
    /// False until a subscription's reads succeeded and started the pump.
    pump_started: bool,
}

/// Per-scope event log → SSE broadcaster registry (决策 2). Each active scope
/// has one broadcaster fed only from the store's event log — never from
/// in-memory events — so the live tail and a resume read the same sequence.
/// The write path notifies on commit; an unsubscribed scope has no
/// broadcaster and stays idle.
pub struct EventHub {
    store: SharedStore,
    settings: EventSettings,
    scopes: Mutex<HashMap<ScopeId, ScopeChannel>>,
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
    /// one) wakes and drains the new events. No broadcaster means no
    /// subscribers — the events wait in the store for a future resume.
    pub fn notify(&self, scope: &ScopeId) {
        if let Some(ch) = self.scopes.lock().unwrap().get(scope) {
            ch.notify.notify_one();
        }
    }

    /// Join `scope`'s live push channel and compute the start plan from one
    /// read of the event log. Creates the scope's broadcaster on first
    /// subscription, and starts its pump once a subscription's reads
    /// succeeded: a subscription that fails leaves no pump behind, and the
    /// next one starts it. The receiver is taken before the log is read, so
    /// any event past the plan's cursor is guaranteed to reach the tail.
    ///
    /// A `last_event_id` below the floor (its events were removed) or above
    /// the newest sequence (the event log started again, as after a restore)
    /// cannot resume: the plan is a reset, and the tail starts after the
    /// newest sequence.
    ///
    /// Synchronous: it reads the store. A route runs it on the blocking pool.
    pub fn subscribe(
        self: &Arc<Self>,
        scope: &ScopeId,
        last_event_id: Option<u64>,
    ) -> Result<Subscription, PortError> {
        let (tx, notify) = {
            let mut scopes = self.scopes.lock().unwrap();
            let channel = scopes.entry(scope.clone()).or_insert_with(|| ScopeChannel {
                tx: broadcast::channel(self.settings.buffer).0,
                notify: Arc::new(Notify::new()),
                pump_started: false,
            });
            (channel.tx.clone(), channel.notify.clone())
        };
        let rx = tx.subscribe();

        // One read drives both the plan and (for a new scope) the pump's
        // start cursor, so the two never disagree on where "new" begins.
        let floor = self.store.event_floor(scope)?;
        let newest = newest_event(&*self.store, scope, floor, self.settings.buffer)?;
        let from_newest = |reset| ResumePlan { reset, backfill: Vec::new(), cursor: newest };
        let plan = match last_event_id {
            Some(last) if last < floor || last > newest => from_newest(true),
            Some(last) => ResumePlan { reset: false, backfill: self.backfill(scope, last)?, cursor: last },
            None => from_newest(false),
        };

        let start_pump = {
            let mut scopes = self.scopes.lock().unwrap();
            let channel = scopes.get_mut(scope).expect("the channel was made above and is never removed");
            !std::mem::replace(&mut channel.pump_started, true)
        };
        if start_pump {
            self.clone().spawn_pump(scope.clone(), tx, notify, newest);
        }
        Ok(Subscription { rx, plan })
    }

    /// Every event of `scope` after `last`, read in batches.
    fn backfill(&self, scope: &ScopeId, last: u64) -> Result<Vec<InvalidationEvent>, PortError> {
        let mut hints = Vec::new();
        each_event_batch(&*self.store, scope, last, self.settings.buffer, |events| {
            hints.extend(events.iter().map(|(seq, event)| invalidation_of(*seq, event)));
        })?;
        Ok(hints)
    }

    /// Spawn the scope's pump: idle until notified, then drain the new
    /// events to the broadcast channel in batches (决策 2) and remove the
    /// events older than the retention window from the store (决策 3).
    fn spawn_pump(
        self: Arc<Self>,
        scope: ScopeId,
        tx: broadcast::Sender<StreamEvent>,
        notify: Arc<Notify>,
        start: u64,
    ) {
        tokio::spawn(async move {
            let mut cursor = start;
            loop {
                notify.notified().await;
                cursor = self.drain(&scope, cursor, &tx).await;
                if cursor > self.settings.retention {
                    self.trim(&scope, cursor - self.settings.retention).await;
                }
            }
        });
    }

    /// Send every event after `from` to the channel, on the blocking pool,
    /// and return the sequence of the last one sent. A read failure ends the
    /// drain; the next notify continues from the returned sequence.
    async fn drain(&self, scope: &ScopeId, from: u64, tx: &broadcast::Sender<StreamEvent>) -> u64 {
        let (store, scope, tx, batch) = (self.store.clone(), scope.clone(), tx.clone(), self.settings.buffer);
        tokio::task::spawn_blocking(move || {
            let mut cursor = from;
            let _ = each_event_batch(&*store, &scope, from, batch, |events| {
                for (seq, event) in events {
                    cursor = *seq;
                    let _ = tx.send(StreamEvent { seq: *seq, event: invalidation_of(*seq, event) });
                }
            });
            cursor
        })
        .await
        .expect("event read task")
    }

    /// Remove the events up to `through` on the blocking pool, best-effort.
    async fn trim(&self, scope: &ScopeId, through: u64) {
        let store = self.store.clone();
        let scope = scope.clone();
        let _ = tokio::task::spawn_blocking(move || store.trim_events(&scope, through)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use speclink_protocol::events::InvalidationScope;
    use speclink_store_port::Version;

    /// The hint of an event at sequence 7, from the commit of scope version
    /// 42, carrying `name` and `payload`.
    fn hint(name: &str, payload: Value) -> InvalidationEvent {
        let event = RecordedEvent { name: name.to_string(), payload, version: Version::new(42).unwrap() };
        invalidation_of(7, &event)
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
                "change-claim-released",
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
                "change-in-progress-removed",
                json!({ "change": "add-auth" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-depends-changed",
                json!({ "change": "add-auth", "dependsOn": ["add-base"] }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-schema-changed",
                json!({ "change": "add-auth", "schema": "no-spec-change" }),
                InvalidationScope::Change,
                "add-auth",
            ),
            (
                "change-ranked",
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
                "discussion-restored",
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
            // The two quality stations and the landing report point at their change.
            ("review-round-added", json!({ "change": "add-auth", "round": 2 }), InvalidationScope::Change, "add-auth"),
            ("review-stamped", json!({ "change": "add-auth" }), InvalidationScope::Change, "add-auth"),
            ("review-discarded", json!({ "change": "add-auth" }), InvalidationScope::Change, "add-auth"),
            ("verify-round-added", json!({ "change": "add-auth", "round": 1 }), InvalidationScope::Change, "add-auth"),
            ("verify-stamped", json!({ "change": "add-auth" }), InvalidationScope::Change, "add-auth"),
            ("verify-discarded", json!({ "change": "add-auth" }), InvalidationScope::Change, "add-auth"),
            (
                "change-landed",
                json!({ "change": "add-auth", "commit": "0123456789ab" }),
                InvalidationScope::Change,
                "add-auth",
            ),
        ];
        for (name, payload, want_scope, want_resource) in cases {
            let ev = hint(name, payload.clone());
            assert_eq!(&ev.scope, want_scope, "{name} maps to the wrong category");
            assert_eq!(
                ev.resource_id, *want_resource,
                "{name} maps to the wrong resource"
            );
            assert_eq!(
                ev.event_id, "7",
                "{name} carries its sequence as the event id"
            );
            assert_eq!(ev.revision, 42, "{name} carries the commit revision");
        }
    }

    #[test]
    fn an_unrecognized_event_name_is_emitted_under_the_unknown_category() {
        // Not swallowed — a future event name still invalidates; the client
        // over-reads rather than missing it.
        let ev = hint("widget-frobbed", serde_json::json!({ "widget": "x" }));
        assert_eq!(ev.scope, InvalidationScope::Unknown("unknown".to_string()));
        assert_eq!(
            ev.resource_id, "",
            "an unmapped event has no resource identity"
        );
        assert_eq!(ev.event_id, "7");
    }

    // --- broadcaster layer (design 決策 2–3): driven directly, no HTTP ---

    use crate::storage::{Memory, ServerStore};
    use serde_json::json;
    use speclink_store_port::{Actor, Commit, Event, StorePort, Timestamp};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast;

    fn scope() -> ScopeId {
        ScopeId::new("demo/backend").unwrap()
    }

    /// Commit one event `name`/`payload` straight to the store's event log.
    fn commit_event(store: &Memory, name: &str, payload: Value) {
        let commit = Commit::new(
            Actor::new("t", "t").unwrap(),
            "t",
            Timestamp::new(0).unwrap(),
            Vec::new(),
            vec![Event::new(name, payload).unwrap()],
        )
        .unwrap();
        store.commit(&scope(), &commit).expect("commit");
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
        let store = Arc::new(Memory::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(1024));
        let mut rx = hub.subscribe(&scope(), None).expect("subscribe").rx;

        commit_event(&store, "task-completed",
            json!({ "change": "demo", "taskId": "1" }),
        );
        hub.notify(&scope());
        commit_event(&store, "task-completed",
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
        // The pushed event is what the event log → invalidation_of yields.
        assert_eq!(first.event.scope, InvalidationScope::Change);
        assert_eq!(first.event.resource_id, "demo");
        assert_eq!(
            first.event.event_id,
            first.seq.to_string(),
            "the event id is the event's sequence"
        );
    }

    #[tokio::test]
    async fn retention_removes_everything_older_than_the_kept_window() {
        let store = Arc::new(Memory::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(3));
        let mut rx = hub.subscribe(&scope(), None).expect("subscribe").rx;

        for i in 1..=5 {
            commit_event(&store, "task-completed",
                json!({ "change": "demo", "taskId": i.to_string() }),
            );
        }
        hub.notify(&scope());
        for _ in 0..5 {
            recv(&mut rx).await;
        }
        // retention = 3, newest seq = 5 → everything up to 5 - 3 = 2 is removed.
        let s = store.clone();
        eventually(move || s.event_floor(&scope()).unwrap() == 2).await;
    }

    #[tokio::test]
    async fn a_cursor_below_the_floor_resumes_with_a_reset() {
        let store = Arc::new(Memory::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(3));
        // Start the pump so retention advances.
        let _sub = hub.subscribe(&scope(), None).expect("subscribe");

        for i in 1..=5 {
            commit_event(&store, "task-completed",
                json!({ "change": "demo", "taskId": i.to_string() }),
            );
        }
        hub.notify(&scope());
        let s = store.clone();
        eventually(move || s.event_floor(&scope()).unwrap() == 2).await;

        // A Last-Event-ID below the floor (2) can no longer resume: reset.
        let reset_plan = hub.subscribe(&scope(), Some(1)).expect("subscribe").plan;
        assert!(
            reset_plan.reset,
            "a cursor below the floor gets a reset"
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

    /// The in-memory store with a switch: while it is on, a read of the
    /// event floor is the store's temporary failure.
    struct Failing {
        inner: Memory,
        fail: std::sync::atomic::AtomicBool,
    }

    impl StorePort for Failing {
        fn list(
            &self,
            scope: &ScopeId,
            prefix: &speclink_store_port::KeyPrefix,
        ) -> Result<speclink_store_port::Listing, PortError> {
            self.inner.list(scope, prefix)
        }
        fn read(
            &self,
            scope: &ScopeId,
            keys: &[speclink_store_port::DocKey],
        ) -> Result<Vec<Option<speclink_store_port::Document>>, PortError> {
            self.inner.read(scope, keys)
        }
        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            self.inner.commit(scope, commit)
        }
    }

    impl ServerStore for Failing {
        fn kind(&self) -> &'static str {
            self.inner.kind()
        }
        fn events_after(&self, scope: &ScopeId, after: u64, limit: usize) -> Result<Vec<(u64, RecordedEvent)>, PortError> {
            self.inner.events_after(scope, after, limit)
        }
        fn event_floor(&self, scope: &ScopeId) -> Result<u64, PortError> {
            if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(PortError::Unavailable { message: "the test's failure".into() });
            }
            self.inner.event_floor(scope)
        }
        fn trim_events(&self, scope: &ScopeId, through: u64) -> Result<(), PortError> {
            self.inner.trim_events(scope, through)
        }
        fn probe(&self) -> Result<(), PortError> {
            self.inner.probe()
        }
    }

    #[tokio::test]
    async fn a_subscription_that_fails_leaves_the_scope_able_to_push_later() {
        let store = Arc::new(Failing { inner: Memory::new(), fail: true.into() });
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(1024));

        // The first subscription of the scope fails on its read of the store.
        assert!(hub.subscribe(&scope(), None).is_err(), "the store's failure reaches the subscriber");

        // The store is back: the next subscription gets the live events.
        store.fail.store(false, std::sync::atomic::Ordering::SeqCst);
        let mut rx = hub.subscribe(&scope(), None).expect("subscribe").rx;
        commit_event(&store.inner, "task-completed", json!({ "change": "demo", "taskId": "1" }));
        hub.notify(&scope());
        assert_eq!(recv(&mut rx).await.seq, 1, "the pump runs for the scope");
    }

    #[tokio::test]
    async fn a_cursor_above_the_newest_sequence_resumes_with_a_reset_from_the_newest() {
        let store = Arc::new(Memory::new());
        let shared: SharedStore = store.clone();
        let hub = EventHub::new(shared, settings(1024));
        for i in 1..=3 {
            commit_event(&store, "task-completed", json!({ "change": "demo", "taskId": i.to_string() }));
        }

        // The client's id is from an event log that started again since.
        let plan = hub.subscribe(&scope(), Some(500)).expect("subscribe").plan;
        assert!(plan.reset, "an id above the newest sequence gets a reset");
        assert!(plan.backfill.is_empty());
        assert_eq!(plan.cursor, 3, "the tail starts after the newest sequence, not after the client's id");

        // The newest sequence itself resumes, with nothing to backfill.
        let plan = hub.subscribe(&scope(), Some(3)).expect("subscribe").plan;
        assert!(!plan.reset, "the newest sequence is a cursor the server knows");
        assert!(plan.backfill.is_empty());
    }

    #[tokio::test]
    async fn a_slow_subscriber_overflows_and_is_dropped_while_a_fast_one_keeps_up() {
        let store = Arc::new(Memory::new());
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
            commit_event(&store, "task-completed",
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

//! The conformance suite: one entry point that runs the same checks
//! against any [`StorePort`] implementation and returns a report.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Barrier;
use std::thread;

use crate::{
    Actor, Commit, DocKey, Document, Event, Expected, KeyPrefix, Listing, Op, PortError,
    RecordedEvent, ScopeId, StorePort, Timestamp, Version,
};

/// What the implementer of a store plugs into the suite.
pub trait PortHarness {
    /// The store under test.
    fn store(&self) -> &dyn StorePort;

    /// A new, empty scope that the store can use at once. The harness
    /// gives it because a host often has to create a row for a scope first.
    fn new_scope(&self) -> Result<ScopeId, PortError>;

    /// The events the store kept for `scope`, oldest first. `Ok(None)`
    /// means the store does not keep events; the event checks then report
    /// "skipped".
    fn recorded_events(&self, scope: &ScopeId) -> Result<Option<Vec<RecordedEvent>>, PortError>;
}

/// The result of one check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    Passed,
    /// The store did something the contract does not allow.
    Failed(String),
    /// The check does not apply to this store.
    Skipped(String),
}

/// One check by its stable name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub outcome: CheckOutcome,
}

/// The verdict of the suite for one store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    pub checks: Vec<CheckResult>,
}

impl ConformanceReport {
    /// True when no check failed.
    #[must_use]
    pub fn passed(&self) -> bool {
        !self
            .checks
            .iter()
            .any(|check| matches!(check.outcome, CheckOutcome::Failed(_)))
    }

    /// The outcome of the check with this name.
    pub fn outcome(&self, name: &str) -> Option<&CheckOutcome> {
        self.checks
            .iter()
            .find(|check| check.name == name)
            .map(|check| &check.outcome)
    }
}

type CheckFn = fn(&dyn PortHarness) -> Check;

/// Every check in report order, by its stable name.
const CHECKS: [(&str, CheckFn); 16] = [
    ("empty-scope", empty_scope),
    ("put-then-read", put_then_read),
    ("list-prefix-order", list_prefix_order),
    ("read-alignment", read_alignment),
    ("update-advances-version", update_advances_version),
    ("delete-then-recreate", delete_then_recreate),
    ("conflict-stale-version", conflict_stale_version),
    ("conflict-absent-but-exists", conflict_absent_but_exists),
    ("conflict-missing-document", conflict_missing_document),
    (
        "conflict-rejects-whole-commit",
        conflict_rejects_whole_commit,
    ),
    ("events-with-commit", events_with_commit),
    ("events-only-commit", events_only_commit),
    ("scope-isolation", scope_isolation),
    ("concurrent-single-winner", concurrent_single_winner),
    ("batch-read-atomic", batch_read_atomic),
    ("content-roundtrip", content_roundtrip),
];

/// Run every check against the harness's store. A check that fails or
/// panics does not stop the others: the report always lists all sixteen.
#[must_use]
pub fn run(harness: &dyn PortHarness) -> ConformanceReport {
    let checks = CHECKS
        .iter()
        .map(|(name, check)| {
            // Each check works in its own new scope, so a panic leaves
            // nothing that the next check reads.
            let outcome = match panic::catch_unwind(AssertUnwindSafe(|| check(harness))) {
                Ok(Ok(())) => CheckOutcome::Passed,
                Ok(Err(Stop::Fail(detail))) => CheckOutcome::Failed(detail),
                Ok(Err(Stop::Skip(reason))) => CheckOutcome::Skipped(reason),
                Err(payload) => CheckOutcome::Failed(format!(
                    "the check panicked: {}",
                    panic_message(payload.as_ref())
                )),
            };
            CheckResult { name, outcome }
        })
        .collect();
    ConformanceReport { checks }
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    if let Some(text) = payload.downcast_ref::<&str>() {
        text
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text
    } else {
        "a panic with no message"
    }
}

/// Why a check ended early.
enum Stop {
    Fail(String),
    Skip(String),
}

impl From<String> for Stop {
    fn from(detail: String) -> Self {
        Stop::Fail(detail)
    }
}

type Check = Result<(), Stop>;

fn ensure(condition: bool, detail: impl FnOnce() -> String) -> Check {
    if condition {
        Ok(())
    } else {
        Err(Stop::Fail(detail()))
    }
}

/// The commit times the checks use, in milliseconds.
const T1: u64 = 1_700_000_000_000;
const T2: u64 = T1 + 1_000;
const T3: u64 = T1 + 2_000;

// The suite's own fixed values are always valid, so building them uses
// `expect`, the same as `keys::fixed`.
fn key(text: &str) -> DocKey {
    DocKey::new(text).expect("the suite's fixed keys follow the key format")
}

fn event(name: &str, payload: serde_json::Value) -> Event {
    Event::new(name, payload).expect("the suite's event names are not empty")
}

fn put(key: &DocKey, content: &str, expected: Expected) -> Op {
    Op::Put {
        key: key.clone(),
        content: content.to_string(),
        expected,
    }
}

fn delete(key: &DocKey, expected: Version) -> Op {
    Op::Delete {
        key: key.clone(),
        expected,
    }
}

fn commit_at(at: u64, ops: Vec<Op>, events: Vec<Event>) -> Commit {
    Commit::new(
        Actor::new("conformance", "Conformance suite").expect("a fixed actor id"),
        "conformance",
        Timestamp::new(at).expect("the suite's times are within the range"),
        ops,
        events,
    )
    .expect("the suite's commits have no repeated key and are not empty")
}

/// One scope of the store under test.
struct Ctx<'a> {
    harness: &'a dyn PortHarness,
    scope: ScopeId,
}

impl<'a> Ctx<'a> {
    fn new(harness: &'a dyn PortHarness) -> Result<Self, Stop> {
        let scope = harness
            .new_scope()
            .map_err(|err| format!("the harness could not give a new scope: {err}"))?;
        Ok(Self { harness, scope })
    }

    fn store(&self) -> &dyn StorePort {
        self.harness.store()
    }

    fn list(&self, prefix: &str) -> Result<Listing, Stop> {
        let prefix = KeyPrefix::new(prefix).expect("the suite's prefixes follow the prefix format");
        Ok(self
            .store()
            .list(&self.scope, &prefix)
            .map_err(|err| format!("list \"{}\" failed: {err}", prefix.as_str()))?)
    }

    /// A read whose result has one entry per key, so callers can index it.
    fn read(&self, keys: &[DocKey]) -> Result<Vec<Option<Document>>, Stop> {
        let found = self
            .store()
            .read(&self.scope, keys)
            .map_err(|err| format!("read failed: {err}"))?;
        ensure(found.len() == keys.len(), || {
            format!(
                "read of {} keys returned {} results",
                keys.len(),
                found.len()
            )
        })?;
        Ok(found)
    }

    /// A read of one key.
    fn read_one(&self, key: &DocKey) -> Result<Option<Document>, Stop> {
        Ok(self.read(slice::from_ref(key))?.remove(0))
    }

    /// The events the store kept in this scope, or `None` when it keeps
    /// none.
    fn events(&self) -> Result<Option<Vec<RecordedEvent>>, Stop> {
        Ok(self
            .harness
            .recorded_events(&self.scope)
            .map_err(|err| format!("the harness could not read the kept events: {err}"))?)
    }

    /// The kept events of a store that reported at the start that it keeps
    /// events.
    fn kept_events(&self) -> Result<Vec<RecordedEvent>, Stop> {
        self.events()?.ok_or_else(|| {
            Stop::Fail("the harness first reported kept events, then reported none".into())
        })
    }

    /// A commit that must succeed.
    fn commit(&self, commit: &Commit) -> Result<Version, Stop> {
        Ok(self
            .store()
            .commit(&self.scope, commit)
            .map_err(|err| format!("a valid commit failed: {err}"))?)
    }
}

fn expect_doc(
    found: &Option<Document>,
    key: &DocKey,
    content: &str,
    version: Version,
    at: u64,
) -> Check {
    let Some(doc) = found else {
        return Err(Stop::Fail(format!(
            "read of {} found no document",
            key.as_str()
        )));
    };
    ensure(doc.content == content, || {
        format!(
            "read of {}: the content differs from what was written ({} bytes written, {} bytes read)",
            key.as_str(),
            content.len(),
            doc.content.len()
        )
    })?;
    ensure(doc.version == version, || {
        format!(
            "read of {}: version {}, expected {version}",
            key.as_str(),
            doc.version
        )
    })?;
    ensure(doc.modified_at.get() == at, || {
        format!(
            "read of {}: modified at {}, expected {at}",
            key.as_str(),
            doc.modified_at.get()
        )
    })
}

fn expect_version(listing: &Listing, want: Version) -> Check {
    ensure(listing.version == want, || {
        format!(
            "the scope lists version {}, expected {want}",
            listing.version
        )
    })
}

/// The listing holds exactly `expected` (key, version, time), in order.
fn expect_entries(listing: &Listing, expected: &[(&DocKey, Version, u64)]) -> Check {
    let found: Vec<(&str, u64, u64)> = listing
        .entries
        .iter()
        .map(|entry| {
            (
                entry.key.as_str(),
                entry.version.get(),
                entry.modified_at.get(),
            )
        })
        .collect();
    let wanted: Vec<(&str, u64, u64)> = expected
        .iter()
        .map(|(key, version, at)| (key.as_str(), version.get(), *at))
        .collect();
    ensure(found == wanted, || {
        format!("the listing is {found:?}, expected {wanted:?}")
    })
}

fn empty_scope(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let listing = cx.list("")?;
    expect_version(&listing, Version::ZERO)?;
    expect_entries(&listing, &[])?;
    let found = cx.read(&[key("config.yaml"), key("specs/auth/spec.md")])?;
    ensure(found.iter().all(Option::is_none), || {
        "a read in a new scope found a document".into()
    })
}

fn put_then_read(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let spec = key("specs/auth/spec.md");
    let config = key("config.yaml");
    let v = cx.commit(&commit_at(
        T1,
        vec![
            put(&spec, "spec", Expected::Absent),
            put(&config, "config", Expected::Absent),
        ],
        Vec::new(),
    ))?;
    ensure(v > Version::ZERO, || {
        "the first commit of a scope returned version 0".into()
    })?;
    let found = cx.read(&[spec.clone(), config.clone()])?;
    expect_doc(&found[0], &spec, "spec", v, T1)?;
    expect_doc(&found[1], &config, "config", v, T1)?;
    let listing = cx.list("")?;
    expect_version(&listing, v)?;
    expect_entries(&listing, &[(&config, v, T1), (&spec, v, T1)])
}

fn list_prefix_order(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let spec = key("specs/auth/spec.md");
    let addendum = key("changes/addendum/tasks.md");
    let language = key("LANGUAGE.md");
    let add_x = key("changes/add-x/proposal.md");
    let config = key("config.yaml");
    // The operations are out of order on purpose.
    let ops = [&spec, &addendum, &language, &add_x, &config]
        .into_iter()
        .map(|key| put(key, "x", Expected::Absent))
        .collect();
    let v = cx.commit(&commit_at(T1, ops, Vec::new()))?;

    // A prefix matches bytes, not segments.
    let listing = cx.list("changes/add")?;
    expect_version(&listing, v)?;
    expect_entries(&listing, &[(&add_x, v, T1), (&addendum, v, T1)])?;
    expect_entries(&cx.list("changes/add-x/")?, &[(&add_x, v, T1)])?;
    // Byte order puts upper case before lower case.
    expect_entries(
        &cx.list("")?,
        &[
            (&language, v, T1),
            (&add_x, v, T1),
            (&addendum, v, T1),
            (&config, v, T1),
            (&spec, v, T1),
        ],
    )?;
    let none = cx.list("discussions/")?;
    expect_version(&none, v)?;
    expect_entries(&none, &[])
}

fn read_alignment(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let config = key("config.yaml");
    let language = key("LANGUAGE.md");
    let v = cx.commit(&commit_at(
        T1,
        vec![put(&config, "a", Expected::Absent)],
        Vec::new(),
    ))?;
    let found = cx.read(&[config.clone(), language.clone(), config.clone()])?;
    expect_doc(&found[0], &config, "a", v, T1)?;
    ensure(found[1].is_none(), || {
        "a read of a key with no document found one".into()
    })?;
    ensure(found[2] == found[0], || {
        "the same key read twice gave two different results".into()
    })?;
    ensure(cx.read(&[])?.is_empty(), || {
        "a read of no keys returned results".into()
    })
}

fn update_advances_version(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let edited = key("specs/auth/spec.md");
    let untouched = key("config.yaml");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![
            put(&edited, "first", Expected::Absent),
            put(&untouched, "kept", Expected::Absent),
        ],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(
        T2,
        vec![put(&edited, "second", Expected::At(v1))],
        Vec::new(),
    ))?;
    ensure(v2 > v1, || {
        format!("a commit after version {v1} returned version {v2}")
    })?;
    let found = cx.read(&[edited.clone(), untouched.clone()])?;
    expect_doc(&found[0], &edited, "second", v2, T2)?;
    expect_doc(&found[1], &untouched, "kept", v1, T1)?;
    let listing = cx.list("")?;
    expect_version(&listing, v2)?;
    expect_entries(&listing, &[(&untouched, v1, T1), (&edited, v2, T2)])
}

fn delete_then_recreate(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let doc = key("discussions/board-search.md");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "first", Expected::Absent)],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(T2, vec![delete(&doc, v1)], Vec::new()))?;
    ensure(v2 > v1, || {
        format!("a delete after version {v1} returned version {v2}")
    })?;
    ensure(cx.read_one(&doc)?.is_none(), || {
        "a read after the delete still found the document".into()
    })?;
    let listing = cx.list("")?;
    expect_version(&listing, v2)?;
    expect_entries(&listing, &[])?;
    let v3 = cx.commit(&commit_at(
        T3,
        vec![put(&doc, "again", Expected::Absent)],
        Vec::new(),
    ))?;
    ensure(v3 > v2, || {
        format!("the commit that recreated the document returned version {v3}, not more than {v2}")
    })?;
    expect_version(&cx.list("")?, v3)?;
    expect_doc(&cx.read_one(&doc)?, &doc, "again", v3, T3)
}

/// The commit must fail with exactly this conflict.
fn expect_conflict(
    cx: &Ctx,
    commit: &Commit,
    key: &DocKey,
    expected: Expected,
    actual: Option<Version>,
) -> Check {
    let want = PortError::Conflict {
        key: key.clone(),
        expected,
        actual,
    };
    match cx.store().commit(&cx.scope, commit) {
        Err(err) if err == want => Ok(()),
        Err(err @ PortError::Conflict { .. }) => Err(Stop::Fail(format!(
            "the store reported \"{err}\", expected \"{want}\""
        ))),
        Err(err) => Err(Stop::Fail(format!(
            "the store reported \"{err}\" instead of the conflict \"{want}\""
        ))),
        Ok(version) => Err(Stop::Fail(format!(
            "a commit that should end in \"{want}\" succeeded with version {version}"
        ))),
    }
}

fn conflict_stale_version(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let doc = key("specs/auth/spec.md");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "first", Expected::Absent)],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(
        T2,
        vec![put(&doc, "second", Expected::At(v1))],
        Vec::new(),
    ))?;
    // v1 is stale now.
    expect_conflict(
        &cx,
        &commit_at(T3, vec![put(&doc, "stale", Expected::At(v1))], Vec::new()),
        &doc,
        Expected::At(v1),
        Some(v2),
    )?;
    expect_conflict(
        &cx,
        &commit_at(T3, vec![delete(&doc, v1)], Vec::new()),
        &doc,
        Expected::At(v1),
        Some(v2),
    )?;
    expect_version(&cx.list("")?, v2)?;
    expect_doc(&cx.read_one(&doc)?, &doc, "second", v2, T2)
}

fn conflict_absent_but_exists(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let doc = key("config.yaml");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "first", Expected::Absent)],
        Vec::new(),
    ))?;
    expect_conflict(
        &cx,
        &commit_at(T2, vec![put(&doc, "again", Expected::Absent)], Vec::new()),
        &doc,
        Expected::Absent,
        Some(v1),
    )?;
    expect_version(&cx.list("")?, v1)?;
    expect_doc(&cx.read_one(&doc)?, &doc, "first", v1, T1)
}

fn conflict_missing_document(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let never = key("specs/missing/spec.md");
    let gone = key("discussions/gone.md");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&gone, "first", Expected::Absent)],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(T2, vec![delete(&gone, v1)], Vec::new()))?;
    let five = Version::new(5).expect("5 is within the version range");
    // A key that never had a document.
    expect_conflict(
        &cx,
        &commit_at(T3, vec![put(&never, "x", Expected::At(five))], Vec::new()),
        &never,
        Expected::At(five),
        None,
    )?;
    expect_conflict(
        &cx,
        &commit_at(T3, vec![delete(&never, five)], Vec::new()),
        &never,
        Expected::At(five),
        None,
    )?;
    // A key whose document was deleted.
    expect_conflict(
        &cx,
        &commit_at(T3, vec![put(&gone, "x", Expected::At(v1))], Vec::new()),
        &gone,
        Expected::At(v1),
        None,
    )?;
    expect_version(&cx.list("")?, v2)?;
    ensure(
        cx.read(&[never.clone(), gone.clone()])?
            .iter()
            .all(Option::is_none),
        || "a rejected commit created a document".into(),
    )
}

fn conflict_rejects_whole_commit(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let target = key("config.yaml");
    let first = key("changes/add-x/proposal.md");
    let second = key("changes/add-x/tasks.md");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&target, "first", Expected::Absent)],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(
        T2,
        vec![put(&target, "second", Expected::At(v1))],
        Vec::new(),
    ))?;
    // Two operations that hold, then one that does not.
    let rejected = commit_at(
        T3,
        vec![
            put(&first, "proposal", Expected::Absent),
            put(&second, "tasks", Expected::Absent),
            put(&target, "third", Expected::At(v1)),
        ],
        vec![event("whole.rejected", serde_json::Value::Null)],
    );
    expect_conflict(&cx, &rejected, &target, Expected::At(v1), Some(v2))?;
    ensure(
        cx.read(&[first.clone(), second.clone()])?
            .iter()
            .all(Option::is_none),
        || "a rejected commit wrote the operations before the conflict".into(),
    )?;
    expect_version(&cx.list("")?, v2)?;
    expect_doc(&cx.read_one(&target)?, &target, "second", v2, T2)?;
    if let Some(events) = cx.events()? {
        ensure(
            events.iter().all(|event| event.name != "whole.rejected"),
            || "a rejected commit left its event".into(),
        )?;
    }
    Ok(())
}

/// `name@version` for each kept event, to show in a failure.
fn event_list(events: &[RecordedEvent]) -> Vec<String> {
    events
        .iter()
        .map(|event| format!("{}@{}", event.name, event.version))
        .collect()
}

fn events_with_commit(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let Some(before) = cx.events()? else {
        return Err(Stop::Skip(
            "the harness reports that the store keeps no events".into(),
        ));
    };
    ensure(before.is_empty(), || {
        format!("a new scope already kept {:?}", event_list(&before))
    })?;
    let doc = key("changes/add-x/tasks.md");
    let first = event("task.done", serde_json::json!({ "task": 1 }));
    let second = event("change.updated", serde_json::json!({ "name": "add-x" }));
    let v = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "tasks", Expected::Absent)],
        vec![first.clone(), second.clone()],
    ))?;
    let want: Vec<RecordedEvent> = [first, second]
        .into_iter()
        .map(|event| RecordedEvent {
            name: event.name().to_string(),
            payload: event.payload().clone(),
            version: v,
        })
        .collect();
    let kept = cx.kept_events()?;
    ensure(kept == want, || {
        format!(
            "the store kept {:?}, expected {:?} with the same payloads",
            event_list(&kept),
            event_list(&want)
        )
    })?;

    expect_conflict(
        &cx,
        &commit_at(
            T2,
            vec![put(&doc, "again", Expected::Absent)],
            vec![event("task.rejected", serde_json::Value::Null)],
        ),
        &doc,
        Expected::Absent,
        Some(v),
    )?;
    let kept = cx.kept_events()?;
    ensure(kept == want, || {
        format!(
            "after a rejected commit the store kept {:?}, expected {:?}",
            event_list(&kept),
            event_list(&want)
        )
    })
}

fn events_only_commit(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let doc = key("board/order");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "add-x", Expected::Absent)],
        Vec::new(),
    ))?;
    let v2 = cx.commit(&commit_at(
        T2,
        Vec::new(),
        vec![event("board.viewed", serde_json::Value::Null)],
    ))?;
    ensure(v2 > v1, || {
        format!("a commit with only events after version {v1} returned version {v2}")
    })?;
    expect_doc(&cx.read_one(&doc)?, &doc, "add-x", v1, T1)?;
    let listing = cx.list("")?;
    expect_version(&listing, v2)?;
    expect_entries(&listing, &[(&doc, v1, T1)])?;
    if let Some(events) = cx.events()? {
        ensure(
            events
                .iter()
                .any(|event| event.name == "board.viewed" && event.version == v2),
            || {
                format!(
                    "the store kept {:?}, which lacks board.viewed@{v2}",
                    event_list(&events)
                )
            },
        )?;
    }
    Ok(())
}

fn scope_isolation(harness: &dyn PortHarness) -> Check {
    let first = Ctx::new(harness)?;
    let second = Ctx::new(harness)?;
    ensure(first.scope != second.scope, || {
        format!("the harness gave the scope {} twice", first.scope.as_str())
    })?;
    let config = key("config.yaml");
    let v1 = first.commit(&commit_at(
        T1,
        vec![put(&config, "first", Expected::Absent)],
        vec![event("isolation.first", serde_json::Value::Null)],
    ))?;

    let listing = second.list("")?;
    ensure(listing.version == Version::ZERO, || {
        format!(
            "another scope's commit moved this scope to version {}",
            listing.version
        )
    })?;
    expect_entries(&listing, &[])?;
    ensure(second.read_one(&config)?.is_none(), || {
        "a read found a document another scope wrote".into()
    })?;

    let w1 = second.commit(&commit_at(
        T2,
        vec![put(&config, "second", Expected::Absent)],
        Vec::new(),
    ))?;
    ensure(w1 > Version::ZERO, || {
        "the first commit of a scope returned version 0".into()
    })?;
    let listing = first.list("")?;
    ensure(listing.version == v1, || {
        format!(
            "another scope's commit moved this scope from version {v1} to {}",
            listing.version
        )
    })?;
    expect_doc(&first.read_one(&config)?, &config, "first", v1, T1)?;
    expect_doc(&second.read_one(&config)?, &config, "second", w1, T2)?;

    if let Some(events) = second.events()? {
        ensure(
            events.iter().all(|event| event.name != "isolation.first"),
            || "a scope kept an event that another scope's commit carried".into(),
        )?;
    }
    Ok(())
}

/// How many threads write at the same time in `concurrent-single-winner`.
const WRITERS: usize = 8;

/// How many commits the writer makes in `batch-read-atomic`.
const ROUNDS: u64 = 200;

fn concurrent_single_winner(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let doc = key("specs/auth/spec.md");
    let v1 = cx.commit(&commit_at(
        T1,
        vec![put(&doc, "start", Expected::Absent)],
        Vec::new(),
    ))?;
    let commits = (0..WRITERS)
        .map(|writer| {
            commit_at(
                T2,
                vec![put(&doc, &format!("writer {writer}"), Expected::At(v1))],
                Vec::new(),
            )
        })
        .collect::<Vec<_>>();

    let store = cx.store();
    let scope = &cx.scope;
    let barrier = Barrier::new(WRITERS);
    let results: Vec<Result<Result<Version, PortError>, String>> = thread::scope(|threads| {
        let handles: Vec<_> = commits
            .iter()
            .map(|commit| {
                let barrier = &barrier;
                threads.spawn(move || {
                    barrier.wait();
                    store.commit(scope, commit)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "a writer thread panicked".to_string())
            })
            .collect()
    });

    let mut winners = Vec::new();
    for (writer, result) in results.into_iter().enumerate() {
        match result? {
            Ok(version) => winners.push((writer, version)),
            Err(PortError::Conflict { key: k, .. }) if k == doc => {}
            Err(err) => {
                return Err(Stop::Fail(format!(
                    "writer {writer} got \"{err}\". The contract requires every writer that loses to report a conflict on {}: a host that sees a busy or serialization error must retry inside its own transaction, then report the conflict",
                    doc.as_str()
                )))
            }
        }
    }
    let [(writer, version)] = winners[..] else {
        return Err(Stop::Fail(format!(
            "{} of {WRITERS} writers with the same expected version succeeded, expected exactly 1",
            winners.len()
        )));
    };
    expect_doc(
        &cx.read_one(&doc)?,
        &doc,
        &format!("writer {writer}"),
        version,
        T2,
    )
}

/// The two documents of one batch read come from the same commit.
fn same_commit(
    found: &[Option<Document>],
    keys: &[DocKey; 2],
) -> Result<(Version, String), String> {
    let [Some(first), Some(second)] = found else {
        return Err(format!(
            "a batch read of {} and {} did not find both documents",
            keys[0].as_str(),
            keys[1].as_str()
        ));
    };
    if first.version != second.version || first.content != second.content {
        return Err(format!(
            "a batch read returned {} at version {} (\"{}\") and {} at version {} (\"{}\")",
            keys[0].as_str(),
            first.version,
            first.content,
            keys[1].as_str(),
            second.version,
            second.content
        ));
    }
    Ok((first.version, first.content.clone()))
}

/// Sets the flag when it is dropped, so the reader of `batch-read-atomic`
/// stops even when the writer panics.
struct SetOnDrop<'a>(&'a AtomicBool);

impl Drop for SetOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn batch_read_atomic(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let keys = [
        key("changes/add-x/proposal.md"),
        key("changes/add-x/tasks.md"),
    ];
    let start = cx.commit(&commit_at(
        T1,
        keys.iter()
            .map(|key| put(key, "0", Expected::Absent))
            .collect(),
        Vec::new(),
    ))?;

    let store = cx.store();
    let scope = &cx.scope;
    let keys = &keys;
    let done = AtomicBool::new(false);
    let (written, reads) = thread::scope(|threads| {
        let writer = threads.spawn(|| {
            let _done = SetOnDrop(&done);
            let mut version = start;
            for round in 1..=ROUNDS {
                let content = round.to_string();
                let commit = commit_at(
                    T1 + round,
                    keys.iter()
                        .map(|key| put(key, &content, Expected::At(version)))
                        .collect(),
                    Vec::new(),
                );
                version = store
                    .commit(scope, &commit)
                    .map_err(|err| format!("commit {round} of the writer failed: {err}"))?;
            }
            Ok::<Version, String>(version)
        });
        let reader = threads.spawn(|| {
            loop {
                // Load the flag before the read: once it is set, this read
                // starts after the writer's last commit.
                let finished = done.load(Ordering::SeqCst);
                let found = store
                    .read(scope, keys)
                    .map_err(|err| format!("a batch read failed: {err}"))?;
                same_commit(&found, keys)?;
                if finished {
                    return Ok::<(), String>(());
                }
                thread::yield_now();
            }
        });
        (
            writer
                .join()
                .map_err(|_| "the writer thread panicked".to_string()),
            reader
                .join()
                .map_err(|_| "the reader thread panicked".to_string()),
        )
    });
    let last = written??;
    reads??;

    let (version, content) = same_commit(&cx.read(keys)?, keys)?;
    ensure(version == last && content == ROUNDS.to_string(), || {
        format!("after {ROUNDS} commits the documents are at version {version}, expected {last}")
    })
}

fn content_roundtrip(harness: &dyn PortHarness) -> Check {
    let cx = Ctx::new(harness)?;
    let large = "0123456789abcdef".repeat(65_536);
    let contents = [
        ("roundtrip/empty", ""),
        ("roundtrip/multibyte", "規格 ✓ — émoji 🚀"),
        ("roundtrip/crlf", "line one\r\nline two\r\n"),
        ("roundtrip/one-mebibyte", large.as_str()),
    ];
    let keys: Vec<DocKey> = contents.iter().map(|(text, _)| key(text)).collect();
    let ops = keys
        .iter()
        .zip(contents)
        .map(|(key, (_, content))| put(key, content, Expected::Absent))
        .collect();
    let v = cx.commit(&commit_at(T1, ops, Vec::new()))?;
    let found = cx.read(&keys)?;
    for ((key, (_, content)), found) in keys.iter().zip(contents).zip(&found) {
        expect_doc(found, key, content, v, T1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU64;
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;

    use super::*;
    use crate::{InvalidValue, MemoryStorePort};

    const NAMES: [&str; 16] = [
        "empty-scope",
        "put-then-read",
        "list-prefix-order",
        "read-alignment",
        "update-advances-version",
        "delete-then-recreate",
        "conflict-stale-version",
        "conflict-absent-but-exists",
        "conflict-missing-document",
        "conflict-rejects-whole-commit",
        "events-with-commit",
        "events-only-commit",
        "scope-isolation",
        "concurrent-single-winner",
        "batch-read-atomic",
        "content-roundtrip",
    ];

    /// A harness over any store. `memory` is the in-memory store whose
    /// events the harness reports; `None` makes `recorded_events` fail.
    struct Harness {
        store: Arc<dyn StorePort>,
        memory: Option<Arc<MemoryStorePort>>,
        keeps_events: bool,
        next: AtomicU64,
    }

    impl Harness {
        fn new(store: Arc<dyn StorePort>, memory: Option<Arc<MemoryStorePort>>) -> Self {
            Self {
                store,
                memory,
                keeps_events: true,
                next: AtomicU64::new(0),
            }
        }
    }

    impl PortHarness for Harness {
        fn store(&self) -> &dyn StorePort {
            self.store.as_ref()
        }

        fn new_scope(&self) -> Result<ScopeId, PortError> {
            let n = self.next.fetch_add(1, Ordering::SeqCst);
            Ok(ScopeId::new(format!("scope-{n}")).expect("a valid scope id"))
        }

        fn recorded_events(
            &self,
            scope: &ScopeId,
        ) -> Result<Option<Vec<RecordedEvent>>, PortError> {
            if !self.keeps_events {
                return Ok(None);
            }
            match &self.memory {
                Some(memory) => Ok(Some(memory.recorded_events(scope))),
                None => Err(PortError::Backend {
                    message: "no event table".into(),
                }),
            }
        }
    }

    fn memory_harness() -> Harness {
        let memory = Arc::new(MemoryStorePort::new());
        Harness::new(memory.clone(), Some(memory))
    }

    fn names(report: &ConformanceReport) -> Vec<&'static str> {
        report.checks.iter().map(|check| check.name).collect()
    }

    fn failures(report: &ConformanceReport) -> Vec<String> {
        report
            .checks
            .iter()
            .filter_map(|check| match &check.outcome {
                CheckOutcome::Failed(detail) => Some(format!("{}: {detail}", check.name)),
                _ => None,
            })
            .collect()
    }

    /// A store whose three operations always fail.
    struct Broken;

    impl StorePort for Broken {
        fn list(&self, _: &ScopeId, _: &KeyPrefix) -> Result<Listing, PortError> {
            Err(PortError::Backend {
                message: "broken".into(),
            })
        }

        fn read(&self, _: &ScopeId, _: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
            Err(PortError::Backend {
                message: "broken".into(),
            })
        }

        fn commit(&self, _: &ScopeId, _: &Commit) -> Result<Version, PortError> {
            Err(PortError::Backend {
                message: "broken".into(),
            })
        }
    }

    /// A store whose three operations panic.
    struct Panics;

    impl StorePort for Panics {
        fn list(&self, _: &ScopeId, _: &KeyPrefix) -> Result<Listing, PortError> {
            panic!("the store crashed")
        }

        fn read(&self, _: &ScopeId, _: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
            panic!("the store crashed")
        }

        fn commit(&self, _: &ScopeId, _: &Commit) -> Result<Version, PortError> {
            panic!("the store crashed")
        }
    }

    fn backend(err: InvalidValue) -> PortError {
        PortError::Backend {
            message: err.to_string(),
        }
    }

    fn rebuilt(commit: &Commit, ops: Vec<Op>, events: Vec<Event>) -> Result<Commit, PortError> {
        Commit::new(
            commit.actor().clone(),
            commit.command(),
            commit.at(),
            ops,
            events,
        )
        .map_err(backend)
    }

    type CommitFn = fn(&MemoryStorePort, &ScopeId, &Commit) -> Result<Version, PortError>;

    /// Lists and reads through the memory store, but commits its own way.
    struct CommitWith {
        inner: Arc<MemoryStorePort>,
        commit: CommitFn,
    }

    impl StorePort for CommitWith {
        fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
            self.inner.list(scope, prefix)
        }

        fn read(
            &self,
            scope: &ScopeId,
            keys: &[DocKey],
        ) -> Result<Vec<Option<Document>>, PortError> {
            self.inner.read(scope, keys)
        }

        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            (self.commit)(&self.inner, scope, commit)
        }
    }

    fn commit_with(commit: CommitFn) -> impl FnOnce(Arc<MemoryStorePort>) -> Arc<dyn StorePort> {
        move |inner| Arc::new(CommitWith { inner, commit })
    }

    /// Wrong on purpose: checks one operation, writes it at once, then
    /// checks the next one. A late conflict leaves the earlier writes.
    fn writes_while_checking(
        inner: &MemoryStorePort,
        scope: &ScopeId,
        commit: &Commit,
    ) -> Result<Version, PortError> {
        let mut version = None;
        for op in commit.ops() {
            version = Some(inner.commit(scope, &rebuilt(commit, vec![op.clone()], Vec::new())?)?);
        }
        if !commit.events().is_empty() {
            version = Some(inner.commit(
                scope,
                &rebuilt(commit, Vec::new(), commit.events().to_vec())?,
            )?);
        }
        Ok(version.expect("a valid commit has an operation or an event"))
    }

    /// Wrong on purpose: replaces every expectation with the document's
    /// current state, so every write goes through.
    fn ignores_expected(
        inner: &MemoryStorePort,
        scope: &ScopeId,
        commit: &Commit,
    ) -> Result<Version, PortError> {
        let mut ops = Vec::new();
        for op in commit.ops() {
            let current = inner.read(scope, slice::from_ref(op.key()))?.remove(0);
            match (op, current) {
                (Op::Put { key, content, .. }, current) => ops.push(Op::Put {
                    key: key.clone(),
                    content: content.clone(),
                    expected: current.map_or(Expected::Absent, |doc| Expected::At(doc.version)),
                }),
                (Op::Delete { key, .. }, Some(doc)) => ops.push(Op::Delete {
                    key: key.clone(),
                    expected: doc.version,
                }),
                (Op::Delete { .. }, None) => {}
            }
        }
        if ops.is_empty() && commit.events().is_empty() {
            return Ok(inner
                .list(scope, &KeyPrefix::new("").map_err(backend)?)?
                .version);
        }
        inner.commit(scope, &rebuilt(commit, ops, commit.events().to_vec())?)
    }

    /// Wrong on purpose: reports a busy database instead of a conflict.
    fn busy_instead_of_conflict(
        inner: &MemoryStorePort,
        scope: &ScopeId,
        commit: &Commit,
    ) -> Result<Version, PortError> {
        inner.commit(scope, commit).map_err(|err| match err {
            PortError::Conflict { .. } => PortError::Unavailable {
                message: "database is busy".into(),
            },
            other => other,
        })
    }

    /// Wrong on purpose: crashes on the first commit of the batch-read-atomic
    /// writer, the only commit in the suite that writes the content "1".
    fn panics_on_round_one(
        inner: &MemoryStorePort,
        scope: &ScopeId,
        commit: &Commit,
    ) -> Result<Version, PortError> {
        let round_one = commit
            .ops()
            .iter()
            .any(|op| matches!(op, Op::Put { content, .. } if content == "1"));
        if round_one {
            panic!("the store crashed");
        }
        inner.commit(scope, commit)
    }

    /// Wrong on purpose: writes the events to its event log before it
    /// checks the expectations, so a rejected commit still leaves its
    /// events. The documents themselves commit as a whole.
    struct EventsBeforeCheck {
        docs: MemoryStorePort,
        log: Arc<MemoryStorePort>,
    }

    impl StorePort for EventsBeforeCheck {
        fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
            self.docs.list(scope, prefix)
        }

        fn read(
            &self,
            scope: &ScopeId,
            keys: &[DocKey],
        ) -> Result<Vec<Option<Document>>, PortError> {
            self.docs.read(scope, keys)
        }

        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            if !commit.events().is_empty() {
                self.log.commit(
                    scope,
                    &rebuilt(commit, Vec::new(), commit.events().to_vec())?,
                )?;
            }
            self.docs.commit(scope, commit)
        }
    }

    /// Wrong on purpose: in a read of two or more keys, an existing second
    /// document comes from the commit before the one that last wrote it.
    struct StaleSecondRead {
        inner: Arc<MemoryStorePort>,
        previous: Mutex<HashMap<(ScopeId, DocKey), Document>>,
    }

    impl StorePort for StaleSecondRead {
        fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
            self.inner.list(scope, prefix)
        }

        fn read(
            &self,
            scope: &ScopeId,
            keys: &[DocKey],
        ) -> Result<Vec<Option<Document>>, PortError> {
            let mut found = self.inner.read(scope, keys)?;
            if keys.len() >= 2 && found[1].is_some() {
                let previous = self.previous.lock().unwrap();
                if let Some(doc) = previous.get(&(scope.clone(), keys[1].clone())) {
                    found[1] = Some(doc.clone());
                }
            }
            Ok(found)
        }

        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            let keys: Vec<DocKey> = commit.ops().iter().map(|op| op.key().clone()).collect();
            let before = self.inner.read(scope, &keys)?;
            let version = self.inner.commit(scope, commit)?;
            let mut previous = self.previous.lock().unwrap();
            for (key, doc) in keys.into_iter().zip(before) {
                if let Some(doc) = doc {
                    previous.insert((scope.clone(), key), doc);
                }
            }
            Ok(version)
        }
    }

    /// Runs the suite on a wrong store and on the memory store. Each named
    /// check must fail for the wrong store and pass for the memory store,
    /// so the check really tells the two apart. The wrong store gets the
    /// memory store that the harness reads the kept events from.
    fn assert_catches(
        wrong: impl FnOnce(Arc<MemoryStorePort>) -> Arc<dyn StorePort>,
        checks: &[&str],
    ) -> ConformanceReport {
        let memory = Arc::new(MemoryStorePort::new());
        let report = run(&Harness::new(wrong(memory.clone()), Some(memory)));
        let reference = run(&memory_harness());
        assert_eq!(names(&report), NAMES);
        assert!(!report.passed());
        for name in checks {
            assert!(
                matches!(report.outcome(name), Some(CheckOutcome::Failed(_))),
                "{name} did not fail: {:?}",
                report.outcome(name)
            );
            assert_eq!(
                reference.outcome(name),
                Some(&CheckOutcome::Passed),
                "the memory store does not pass {name}"
            );
        }
        report
    }

    #[test]
    fn a_store_that_writes_while_it_checks_is_caught() {
        assert_catches(
            commit_with(writes_while_checking),
            &["conflict-rejects-whole-commit"],
        );
    }

    #[test]
    fn a_store_that_ignores_expectations_is_caught() {
        assert_catches(
            commit_with(ignores_expected),
            &[
                "conflict-stale-version",
                "conflict-absent-but-exists",
                "conflict-missing-document",
            ],
        );
    }

    #[test]
    fn a_store_that_keeps_events_before_the_check_is_caught() {
        let report = assert_catches(
            |log| {
                Arc::new(EventsBeforeCheck {
                    docs: MemoryStorePort::new(),
                    log,
                })
            },
            &["conflict-rejects-whole-commit"],
        );
        // The documents and the scope version stay right, so only the
        // event assertion can catch this store.
        assert_eq!(
            report.outcome("conflict-rejects-whole-commit"),
            Some(&CheckOutcome::Failed(
                "a rejected commit left its event".into()
            ))
        );
    }

    #[test]
    fn a_store_whose_batch_read_mixes_commits_is_caught() {
        let report = assert_catches(
            |memory| {
                Arc::new(StaleSecondRead {
                    inner: memory,
                    previous: Default::default(),
                })
            },
            &["batch-read-atomic"],
        );
        // 「一項失敗不影響其他檢查」：其餘十五項照各自的實際行為記為通過。
        for check in &report.checks {
            if check.name != "batch-read-atomic" {
                assert_eq!(check.outcome, CheckOutcome::Passed, "{}", check.name);
            }
        }
    }

    #[test]
    fn a_store_that_reports_busy_instead_of_a_conflict_is_told_what_to_do() {
        let report = assert_catches(
            commit_with(busy_instead_of_conflict),
            &["concurrent-single-winner"],
        );
        let Some(CheckOutcome::Failed(detail)) = report.outcome("concurrent-single-winner") else {
            panic!("concurrent-single-winner did not fail");
        };
        assert!(
            detail.contains("must retry inside its own transaction"),
            "{detail}"
        );
    }

    #[test]
    fn a_store_that_keeps_no_events_passes_with_one_skip() {
        let mut harness = memory_harness();
        harness.keeps_events = false;
        let report = run(&harness);
        assert!(report.passed(), "{:?}", failures(&report));
        for check in &report.checks {
            let expected = if check.name == "events-with-commit" {
                CheckOutcome::Skipped("the harness reports that the store keeps no events".into())
            } else {
                CheckOutcome::Passed
            };
            assert_eq!(check.outcome, expected, "{}", check.name);
        }
    }

    #[test]
    fn the_memory_store_passes_every_check_without_skips() {
        let report = run(&memory_harness());
        assert_eq!(names(&report), NAMES);
        assert_eq!(failures(&report), Vec::<String>::new());
        for check in &report.checks {
            assert_eq!(check.outcome, CheckOutcome::Passed, "{}", check.name);
        }
        assert!(report.passed());
    }

    #[test]
    fn a_store_that_always_fails_still_gets_all_sixteen_results() {
        let report = run(&Harness::new(Arc::new(Broken), None));
        assert_eq!(names(&report), NAMES);
        assert!(!report.passed());
        for check in &report.checks {
            assert!(
                matches!(check.outcome, CheckOutcome::Failed(_)),
                "{} did not fail",
                check.name
            );
        }
    }

    #[test]
    fn a_store_that_panics_still_gets_all_sixteen_results() {
        let report = run(&Harness::new(Arc::new(Panics), None));
        assert_eq!(names(&report), NAMES);
        for check in &report.checks {
            assert!(
                matches!(check.outcome, CheckOutcome::Failed(_)),
                "{} did not fail",
                check.name
            );
        }
        assert_eq!(
            report.outcome("empty-scope"),
            Some(&CheckOutcome::Failed(
                "the check panicked: the store crashed".into()
            ))
        );
    }

    #[test]
    fn a_writer_that_panics_does_not_hang_batch_read_atomic() {
        let memory = Arc::new(MemoryStorePort::new());
        let harness = Harness::new(
            commit_with(panics_on_round_one)(memory.clone()),
            Some(memory),
        );
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || sender.send(run(&harness)));
        let report = receiver
            .recv_timeout(Duration::from_secs(60))
            .expect("the suite hangs when the writer panics");
        assert_eq!(
            report.outcome("batch-read-atomic"),
            Some(&CheckOutcome::Failed("the writer thread panicked".into()))
        );
    }
}

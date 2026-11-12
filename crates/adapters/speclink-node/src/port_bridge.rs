//! The JS store port (design D7): a host's three JS functions — `list`, `read`,
//! `commit` — as a Rust `StorePort`. The functions may return values or
//! Promises; each call goes through the store bridge's threadsafe function and
//! the calling worker thread waits for it to settle.
//!
//! The JS side (`makeInvoker` over `asJson` functions in index.js) settles
//! with the result as JSON text, or with the thrown object's `code`,
//! `message`, `key` and `actual`. Everything the JS side returns is checked
//! here before the engine sees it.

use crate::store_bridge::{BridgeFailure, JsStoreBridge};
use speclink_store_port::conformance::{self, CheckOutcome, PortHarness};
use speclink_store_port::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError, RecordedEvent,
    ScopeId, StorePort, Timestamp, Version, MAX_VERSION,
};

/// A JS object with `list`, `read` and `commit` (and, for a conformance run,
/// `newScope` and `recordedEvents`), reached through the bridge.
#[derive(Clone)]
pub struct JsPort {
    bridge: JsStoreBridge,
}

impl JsPort {
    pub fn new(bridge: JsStoreBridge) -> JsPort {
        JsPort { bridge }
    }

    fn call(&self, method: &'static str, args: serde_json::Value) -> Result<serde_json::Value, BridgeFailure> {
        settle_call(&self.bridge, method, args)
    }
}

/// Call one JS function whose result crosses as JSON text, and wait for it to
/// settle. A throw comes back as a [`BridgeFailure`] carrying its fields.
pub(crate) fn settle_call(
    bridge: &JsStoreBridge,
    method: &'static str,
    args: serde_json::Value,
) -> Result<serde_json::Value, BridgeFailure> {
    let text = bridge.call(method, args)?;
    let text = text.as_str().ok_or_else(|| BridgeFailure::new(method, "returned no JSON text"))?;
    serde_json::from_str(text).map_err(|e| BridgeFailure::new(method, format!("returned text that is not JSON: {e}")))
}

/// `unavailable` is a temporary failure; every other throw outside a commit's
/// conflict is a backend failure. The message names the function.
fn port_error(f: BridgeFailure) -> PortError {
    match f.code.as_deref() {
        Some("unavailable") => PortError::Unavailable { message: f.to_string() },
        _ => PortError::Backend { message: f.to_string() },
    }
}

/// `{ id, displayName }`, as the JS side reads an actor or an identity.
pub(crate) fn person_json(id: &str, display_name: &str) -> serde_json::Value {
    serde_json::json!({ "id": id, "displayName": display_name })
}

/// `[{ name, payload }]`, as the JS side reads a commit's events.
pub(crate) fn events_json(events: &[speclink_store_port::Event]) -> serde_json::Value {
    events.iter().map(|e| serde_json::json!({ "name": e.name(), "payload": e.payload() })).collect()
}

/// A returned value that breaks the contract: a backend failure that names
/// the function and the field.
fn malformed(method: &str, field: &str, rule: impl std::fmt::Display) -> PortError {
    PortError::Backend { message: format!("{method}: {field} {rule}") }
}

/// An integer from 0 to 2^53 − 1. A JS number arrives as an integer or a
/// float; a fraction, a negative number or a larger one is rejected, never
/// rounded or cut.
fn integer(method: &str, field: &str, value: Option<&serde_json::Value>) -> Result<u64, PortError> {
    let rule = format!("must be an integer from 0 to {MAX_VERSION}");
    let number = value.and_then(|v| v.as_number()).ok_or_else(|| malformed(method, field, &rule))?;
    let n = match (number.as_u64(), number.as_f64()) {
        (Some(n), _) => Some(n),
        (None, Some(f)) if f.fract() == 0.0 && (0.0..=MAX_VERSION as f64).contains(&f) => Some(f as u64),
        _ => None,
    };
    match n {
        Some(n) if n <= MAX_VERSION => Ok(n),
        _ => Err(malformed(method, field, format!("{rule}, got {number}"))),
    }
}

fn version(method: &str, field: &str, value: Option<&serde_json::Value>) -> Result<Version, PortError> {
    Version::new(integer(method, field, value)?).map_err(|e| malformed(method, field, e))
}

fn timestamp(method: &str, field: &str, value: Option<&serde_json::Value>) -> Result<Timestamp, PortError> {
    Timestamp::new(integer(method, field, value)?).map_err(|e| malformed(method, field, e))
}

fn commit_json(commit: &Commit) -> serde_json::Value {
    let ops: Vec<serde_json::Value> = commit
        .ops()
        .iter()
        .map(|op| match op {
            Op::Put { key, content, expected } => serde_json::json!({
                "op": "put",
                "key": key.as_str(),
                "content": content,
                "expected": match expected {
                    Expected::Absent => serde_json::Value::Null,
                    Expected::At(v) => serde_json::json!(v.get()),
                },
            }),
            Op::Delete { key, expected } => {
                serde_json::json!({ "op": "delete", "key": key.as_str(), "expected": expected.get() })
            }
        })
        .collect();
    serde_json::json!({
        "actor": person_json(commit.actor().id(), commit.actor().display_name()),
        "command": commit.command(),
        "at": commit.at().get(),
        "ops": ops,
        "events": events_json(commit.events()),
    })
}

/// The expectation the commit's operation on `key` stated.
fn expected_of(commit: &Commit, key: &DocKey) -> Option<Expected> {
    commit.ops().iter().find(|op| op.key() == key).map(|op| match op {
        Op::Put { expected, .. } => *expected,
        Op::Delete { expected, .. } => Expected::At(*expected),
    })
}

impl StorePort for JsPort {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        const M: &str = "list";
        let value = self
            .call(M, serde_json::json!([scope.as_str(), prefix.as_str()]))
            .map_err(port_error)?;
        let scope_version = version(M, "version", value.get("version"))?;
        let raw = value
            .get("entries")
            .and_then(|e| e.as_array())
            .ok_or_else(|| malformed(M, "entries", "must be an array"))?;
        let mut entries: Vec<ListEntry> = Vec::with_capacity(raw.len());
        for (i, entry) in raw.iter().enumerate() {
            let field = |name: &str| format!("entries[{i}].{name}");
            let key = entry
                .get("key")
                .and_then(|k| k.as_str())
                .ok_or_else(|| malformed(M, &field("key"), "must be a string"))?;
            let key = DocKey::new(key).map_err(|e| malformed(M, &field("key"), e))?;
            if !key.as_str().starts_with(prefix.as_str()) {
                return Err(malformed(M, &field("key"), format!("must start with the prefix '{}'", prefix.as_str())));
            }
            // Byte order, each key once: the engine reads the listing as given.
            if entries.last().is_some_and(|before| key <= before.key) {
                return Err(malformed(M, &field("key"), "must come after the key before it, in byte order"));
            }
            entries.push(ListEntry {
                key,
                version: version(M, &field("version"), entry.get("version"))?,
                modified_at: timestamp(M, &field("modifiedAt"), entry.get("modifiedAt"))?,
            });
        }
        Ok(Listing { version: scope_version, entries })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        const M: &str = "read";
        let names: Vec<&str> = keys.iter().map(DocKey::as_str).collect();
        let value = self
            .call(M, serde_json::json!([scope.as_str(), names]))
            .map_err(port_error)?;
        let docs = value.as_array().ok_or_else(|| malformed(M, "result", "must be an array"))?;
        if docs.len() != keys.len() {
            return Err(malformed(
                M,
                "result",
                format!("array length must equal the number of keys ({}), got {}", keys.len(), docs.len()),
            ));
        }
        docs.iter()
            .enumerate()
            .map(|(i, doc)| {
                if doc.is_null() {
                    return Ok(None);
                }
                let field = |name: &str| format!("[{i}].{name}");
                let content = doc
                    .get("content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| malformed(M, &field("content"), "must be a string"))?;
                Ok(Some(Document {
                    content: content.to_string(),
                    version: version(M, &field("version"), doc.get("version"))?,
                    modified_at: timestamp(M, &field("modifiedAt"), doc.get("modifiedAt"))?,
                }))
            })
            .collect()
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        const M: &str = "commit";
        match self.call(M, serde_json::json!([scope.as_str(), commit_json(commit)])) {
            Ok(value) => version(M, "result", Some(&value)),
            Err(thrown) if thrown.code.as_deref() == Some("conflict") => {
                // A conflict names one key of this commit; any other key is not
                // this commit's conflict, so it reads as a backend failure.
                let named = thrown.key.as_deref().and_then(|k| DocKey::new(k).ok());
                match named.and_then(|key| expected_of(commit, &key).map(|expected| (key, expected))) {
                    Some((key, expected)) => Err(PortError::Conflict {
                        key,
                        expected,
                        // `null` (or no `actual` at all) reads as "no document";
                        // a value of another type is refused, never read as one.
                        actual: match &thrown.actual {
                            serde_json::Value::Null => None,
                            other => Some(version(M, "error.actual", Some(other))?),
                        },
                    }),
                    None => Err(PortError::Backend {
                        message: format!(
                            "{thrown} (a conflict on a key that is not in this commit: {})",
                            thrown.key.as_deref().unwrap_or("none given")
                        ),
                    }),
                }
            }
            Err(thrown) => Err(port_error(thrown)),
        }
    }
}

// --- the conformance run (design D8) ---

struct JsHarness {
    port: JsPort,
    keeps_events: bool,
}

impl PortHarness for JsHarness {
    fn store(&self) -> &dyn StorePort {
        &self.port
    }

    fn new_scope(&self) -> Result<ScopeId, PortError> {
        const M: &str = "newScope";
        let value = self.port.call(M, serde_json::json!([])).map_err(port_error)?;
        let scope = value.as_str().ok_or_else(|| malformed(M, "result", "must be a string"))?;
        ScopeId::new(scope).map_err(|e| malformed(M, "result", e))
    }

    fn recorded_events(&self, scope: &ScopeId) -> Result<Option<Vec<RecordedEvent>>, PortError> {
        const M: &str = "recordedEvents";
        if !self.keeps_events {
            return Ok(None);
        }
        let value = self
            .port
            .call(M, serde_json::json!([scope.as_str()]))
            .map_err(port_error)?;
        let events = value.as_array().ok_or_else(|| malformed(M, "result", "must be an array"))?;
        events
            .iter()
            .enumerate()
            .map(|(i, event)| {
                let name = event
                    .get("name")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| malformed(M, &format!("[{i}].name"), "must be a string"))?;
                Ok(RecordedEvent {
                    name: name.to_string(),
                    payload: event.get("payload").cloned().unwrap_or_default(),
                    version: version(M, &format!("[{i}].version"), event.get("version"))?,
                })
            })
            .collect::<Result<Vec<_>, PortError>>()
            .map(Some)
    }
}

/// Run the store contract's checks on the JS functions behind `bridge` and
/// shape the report for JS: `{ passed, checks: [{ name, outcome, detail? }] }`.
pub fn run_conformance(bridge: JsStoreBridge, keeps_events: bool) -> serde_json::Value {
    let harness = JsHarness { port: JsPort::new(bridge), keeps_events };
    let report = conformance::run(&harness);
    let checks: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|check| match &check.outcome {
            CheckOutcome::Passed => serde_json::json!({ "name": check.name, "outcome": "passed" }),
            CheckOutcome::Failed(detail) => {
                serde_json::json!({ "name": check.name, "outcome": "failed", "detail": detail })
            }
            CheckOutcome::Skipped(reason) => {
                // The suite's reason speaks of a harness; a JS host gave no
                // recordedEvents function.
                let detail = if keeps_events {
                    reason.clone()
                } else {
                    "skipped: no recordedEvents function was given, so the events the store keeps cannot be read back"
                        .to_string()
                };
                serde_json::json!({ "name": check.name, "outcome": "skipped", "detail": detail })
            }
        })
        .collect();
    serde_json::json!({ "passed": report.passed(), "checks": checks })
}

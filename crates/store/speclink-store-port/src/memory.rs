//! The in-memory reference implementation of the store port.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError, ScopeId,
    StorePort, Version,
};

/// One event a store kept, with the scope version of its commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedEvent {
    pub name: String,
    pub payload: serde_json::Value,
    pub version: Version,
}

#[derive(Debug)]
struct ScopeState {
    version: Version,
    /// A `BTreeMap` keeps the keys in byte order.
    docs: BTreeMap<DocKey, Document>,
    events: Vec<RecordedEvent>,
}

impl ScopeState {
    fn new() -> Self {
        Self {
            version: Version::ZERO,
            docs: BTreeMap::new(),
            events: Vec::new(),
        }
    }
}

/// Keeps every scope in memory behind one lock. A commit holds the lock for
/// its whole run, so a read or a list never sees half of a commit.
///
/// It reports no failure other than a conflict, with one exception: a
/// commit on a scope that is already at [`crate::MAX_VERSION`] returns
/// [`PortError::Backend`] and changes nothing.
#[derive(Debug, Default)]
pub struct MemoryStorePort {
    scopes: Mutex<HashMap<ScopeId, ScopeState>>,
}

impl MemoryStorePort {
    pub fn new() -> Self {
        Self::default()
    }

    /// No code panics while it holds the lock, so a poisoned lock still
    /// guards a whole state.
    fn scopes(&self) -> MutexGuard<'_, HashMap<ScopeId, ScopeState>> {
        self.scopes.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The events this store kept for `scope`, oldest first. Not part of
    /// the contract: it exists for the conformance harness and for tests.
    pub fn recorded_events(&self, scope: &ScopeId) -> Vec<RecordedEvent> {
        self.scopes()
            .get(scope)
            .map(|state| state.events.clone())
            .unwrap_or_default()
    }
}

impl StorePort for MemoryStorePort {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        let scopes = self.scopes();
        let Some(state) = scopes.get(scope) else {
            return Ok(Listing {
                version: Version::ZERO,
                entries: Vec::new(),
            });
        };
        let entries = state
            .docs
            .iter()
            .filter(|(key, _)| prefix.matches(key))
            .map(|(key, doc)| ListEntry {
                key: key.clone(),
                version: doc.version,
                modified_at: doc.modified_at,
            })
            .collect();
        Ok(Listing {
            version: state.version,
            entries,
        })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        let scopes = self.scopes();
        let state = scopes.get(scope);
        Ok(keys
            .iter()
            .map(|key| state.and_then(|state| state.docs.get(key).cloned()))
            .collect())
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        let mut scopes = self.scopes();
        let state = scopes.entry(scope.clone()).or_insert_with(ScopeState::new);
        // Check every expectation before the first change.
        for op in commit.ops() {
            let actual = state.docs.get(op.key()).map(|doc| doc.version);
            let expected = match op {
                Op::Put { expected, .. } => *expected,
                Op::Delete { expected, .. } => Expected::At(*expected),
            };
            let holds = match expected {
                Expected::Absent => actual.is_none(),
                Expected::At(version) => actual == Some(version),
            };
            if !holds {
                return Err(PortError::Conflict {
                    key: op.key().clone(),
                    expected,
                    actual,
                });
            }
        }
        // The current version is at most MAX_VERSION, so the sum cannot
        // overflow; `Version::new` rejects the one value past the limit.
        let version = Version::new(state.version.get() + 1).map_err(|_| PortError::Backend {
            message: "the scope version reached its limit".into(),
        })?;
        for op in commit.ops() {
            match op {
                Op::Put { key, content, .. } => {
                    state.docs.insert(
                        key.clone(),
                        Document {
                            content: content.clone(),
                            version,
                            modified_at: commit.at(),
                        },
                    );
                }
                Op::Delete { key, .. } => {
                    state.docs.remove(key);
                }
            }
        }
        state
            .events
            .extend(commit.events().iter().map(|event| RecordedEvent {
                name: event.name().to_string(),
                payload: event.payload().clone(),
                version,
            }));
        state.version = version;
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, Event, Timestamp};

    fn commit(ops: Vec<Op>, events: Vec<Event>) -> Commit {
        Commit::new(
            Actor::new("u-1", "Momo").unwrap(),
            "test",
            Timestamp::new(1).unwrap(),
            ops,
            events,
        )
        .unwrap()
    }

    fn event(name: &str) -> Event {
        Event::new(name, serde_json::json!({ "name": name })).unwrap()
    }

    #[test]
    fn keeps_the_events_of_a_commit_in_order_with_its_version() {
        let store = MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let doc = DocKey::new("config.yaml").unwrap();
        let version = store
            .commit(
                &scope,
                &commit(
                    vec![Op::Put {
                        key: doc,
                        content: "a".into(),
                        expected: Expected::Absent,
                    }],
                    vec![event("first"), event("second")],
                ),
            )
            .unwrap();
        let kept: Vec<(String, u64)> = store
            .recorded_events(&scope)
            .into_iter()
            .map(|event| (event.name, event.version.get()))
            .collect();
        assert_eq!(
            kept,
            [
                ("first".to_string(), version.get()),
                ("second".to_string(), version.get())
            ]
        );
        assert_eq!(
            store.recorded_events(&scope)[0].payload,
            serde_json::json!({ "name": "first" })
        );
    }

    #[test]
    fn a_rejected_commit_keeps_no_event() {
        let store = MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let rejected = commit(
            vec![Op::Delete {
                key: DocKey::new("config.yaml").unwrap(),
                expected: Version::new(1).unwrap(),
            }],
            vec![event("lost")],
        );
        assert!(matches!(
            store.commit(&scope, &rejected),
            Err(PortError::Conflict { actual: None, .. })
        ));
        assert!(store.recorded_events(&scope).is_empty());
        assert_eq!(
            store
                .list(&scope, &KeyPrefix::new("").unwrap())
                .unwrap()
                .version,
            Version::ZERO
        );
    }

    #[test]
    fn a_scope_at_the_version_limit_rejects_the_next_commit_without_change() {
        let store = MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let mut state = ScopeState::new();
        state.version = Version::new(crate::MAX_VERSION).unwrap();
        store.scopes().insert(scope.clone(), state);
        let doc = DocKey::new("config.yaml").unwrap();
        let result = store.commit(
            &scope,
            &commit(
                vec![Op::Put {
                    key: doc.clone(),
                    content: "a".into(),
                    expected: Expected::Absent,
                }],
                vec![event("lost")],
            ),
        );
        assert_eq!(
            result,
            Err(PortError::Backend {
                message: "the scope version reached its limit".into()
            })
        );
        assert_eq!(store.read(&scope, &[doc]).unwrap(), [None]);
        assert!(store.recorded_events(&scope).is_empty());
        assert_eq!(
            store
                .list(&scope, &KeyPrefix::new("").unwrap())
                .unwrap()
                .version
                .get(),
            crate::MAX_VERSION
        );
    }

    #[test]
    fn conflicts_report_the_current_version_from_the_spec_table() {
        // 規格範例「衝突的對照」：文件在版本 7，或不存在。
        let store = MemoryStorePort::new();
        let scope = ScopeId::new("s").unwrap();
        let doc = DocKey::new("config.yaml").unwrap();
        let missing = DocKey::new("LANGUAGE.md").unwrap();
        let mut version = Version::ZERO;
        for round in 1..=7 {
            let expected = if round == 1 {
                Expected::Absent
            } else {
                Expected::At(version)
            };
            let put = Op::Put {
                key: doc.clone(),
                content: round.to_string(),
                expected,
            };
            version = store
                .commit(&scope, &commit(vec![put], Vec::new()))
                .unwrap();
        }
        let seven = Version::new(7).unwrap();
        let five = Version::new(5).unwrap();
        assert_eq!(version, seven);
        let put = |key: &DocKey, expected| Op::Put {
            key: key.clone(),
            content: "x".into(),
            expected,
        };
        let delete = |key: &DocKey| Op::Delete {
            key: key.clone(),
            expected: five,
        };
        let rows = [
            (
                put(&doc, Expected::At(five)),
                Expected::At(five),
                Some(seven),
            ),
            (delete(&doc), Expected::At(five), Some(seven)),
            (put(&doc, Expected::Absent), Expected::Absent, Some(seven)),
            (put(&missing, Expected::At(five)), Expected::At(five), None),
            (delete(&missing), Expected::At(five), None),
        ];
        for (op, expected, actual) in rows {
            let key = op.key().clone();
            assert_eq!(
                store.commit(&scope, &commit(vec![op], Vec::new())),
                Err(PortError::Conflict {
                    key,
                    expected,
                    actual
                })
            );
        }
        assert_eq!(
            store
                .list(&scope, &KeyPrefix::new("").unwrap())
                .unwrap()
                .version,
            seven
        );
    }
}

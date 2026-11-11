//! A TeamStore scope as a store port (design D5). Transitional.
//!
//! The reference server keeps its three TeamStore drivers while the Engine
//! moves to the store port. [`TeamStorePort`] wraps one scope of a TeamStore
//! as a [`StorePort`], so the server runs every verb on the same path as a
//! host that brings its own store. The adapter goes away with the TeamStore.
//!
//! It does not pass the store port's conformance suite and does not have to:
//! it reports 0 as every last-modified time, and the scopes of one project
//! share their version counter.

use crate::port_view::{parse_key, KeyKind};
use speclink_store::{
    CommandContext, DocumentId, EventRecord, ExpectedRevision, Revision, Scope, StoreError,
    TeamStore,
};
use speclink_store_port::{
    Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError, ScopeId,
    StorePort, Timestamp, Version,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The TeamStore document a key names, or `None` when the key names none of
/// the nine document kinds.
fn doc_of(key: &DocKey) -> Option<DocumentId> {
    Some(match parse_key(key.as_str())? {
        KeyKind::ChangeMeta { change } => DocumentId::ChangeMeta {
            change: change.to_string(),
        },
        KeyKind::ChangeEvidence { change } => DocumentId::ChangeEvidence {
            change: change.to_string(),
        },
        KeyKind::ChangeArtifact { change, artifact } => DocumentId::ChangeArtifact {
            change: change.to_string(),
            artifact: artifact.to_string(),
        },
        KeyKind::ArchivedChangeDoc { dated_name, doc } => DocumentId::ArchivedChange {
            change: dated_name.to_string(),
            doc: doc.to_string(),
        },
        KeyKind::Spec { capability } => DocumentId::CanonicalSpec {
            capability: capability.to_string(),
        },
        KeyKind::Discussion { slug } => DocumentId::Discussion {
            slug: slug.to_string(),
            archived: false,
        },
        // The stored name is the slug, as it is: a record the Engine archives
        // through the port carries its creation date, an older one does not.
        KeyKind::ArchivedDiscussion { stem } => DocumentId::Discussion {
            slug: stem.to_string(),
            archived: true,
        },
        KeyKind::Config => DocumentId::WorkflowConfig,
        KeyKind::Language => DocumentId::Language,
        KeyKind::BoardOrder => DocumentId::BoardOrder,
    })
}

/// The key of a TeamStore document, or `None` when its names cannot make a
/// key that reads back as the same document (a name with a `/`, for one).
fn key_of(doc: &DocumentId) -> Option<DocKey> {
    let text = match doc {
        DocumentId::ChangeMeta { change } => format!("changes/{change}/.meta.yaml"),
        DocumentId::ChangeEvidence { change } => format!("changes/{change}/.evidence.json"),
        DocumentId::ChangeArtifact { change, artifact } => format!("changes/{change}/{artifact}"),
        DocumentId::ArchivedChange { change, doc } => format!("archive/changes/{change}/{doc}"),
        DocumentId::CanonicalSpec { capability } => format!("specs/{capability}/spec.md"),
        DocumentId::Discussion {
            slug,
            archived: false,
        } => format!("discussions/{slug}.md"),
        DocumentId::Discussion {
            slug,
            archived: true,
        } => format!("archive/discussions/{slug}.md"),
        DocumentId::WorkflowConfig => "config.yaml".to_string(),
        DocumentId::Language => "LANGUAGE.md".to_string(),
        DocumentId::BoardOrder => "board/order".to_string(),
    };
    let key = DocKey::new(text).ok()?;
    (doc_of(&key).as_ref() == Some(doc)).then_some(key)
}

fn backend(message: impl Into<String>) -> PortError {
    PortError::Backend {
        message: message.into(),
    }
}

fn version_of(revision: Revision) -> Result<Version, PortError> {
    Version::new(revision.0).map_err(|e| backend(e.to_string()))
}

/// The six TeamStore failures as the three of the store port. A conflict
/// keeps its document, expectation and current revision; "unavailable" stays
/// a temporary failure; the rest is a backend failure with the TeamStore's
/// own text.
fn port_error(e: StoreError) -> PortError {
    match e {
        StoreError::RevisionConflict {
            doc,
            expected,
            actual,
        } => {
            let converted = key_of(&doc.doc).zip(
                match expected {
                    ExpectedRevision::Absent => Ok(Expected::Absent),
                    ExpectedRevision::At(revision) => version_of(revision).map(Expected::At),
                }
                .ok(),
            );
            match (converted, actual.map(version_of).transpose()) {
                (Some((key, expected)), Ok(actual)) => PortError::Conflict {
                    key,
                    expected,
                    actual,
                },
                _ => backend(format!(
                    "revision_conflict: {:?} expected {expected:?}, actual {actual:?}",
                    doc.doc
                )),
            }
        }
        StoreError::Unavailable => PortError::Unavailable {
            message: "the store backend is temporarily unavailable".to_string(),
        },
        other => backend(other.to_string()),
    }
}

/// The scope as one fixed point: the project revision and every document
/// that has a key, with its content and revision.
struct Loaded {
    revision: Revision,
    docs: BTreeMap<DocKey, (String, Revision)>,
}

/// One TeamStore scope as a store port.
///
/// Build one adapter for each command. The first list or read loads the
/// scope once (one snapshot, one export) and every later list and read of the
/// adapter answers from that; the adapter's own commit drops it.
pub struct TeamStorePort<'a> {
    store: &'a (dyn TeamStore + Sync),
    scope: Scope,
    scope_id: ScopeId,
    loaded: Mutex<Option<Loaded>>,
}

impl<'a> TeamStorePort<'a> {
    /// Wrap `scope` of `store`. Fails when the project and repo names cannot
    /// make a scope id.
    pub fn new(store: &'a (dyn TeamStore + Sync), scope: Scope) -> Result<Self, PortError> {
        let scope_id = ScopeId::new(format!(
            "{}/{}",
            scope.project.as_str(),
            scope.repo.as_str()
        ))
        .map_err(|e| backend(e.to_string()))?;
        Ok(TeamStorePort {
            store,
            scope,
            scope_id,
            loaded: Mutex::new(None),
        })
    }

    /// The id of the one scope this adapter serves: the project and the repo
    /// joined by `/`. Every call with another id is a backend error.
    pub fn scope_id(&self) -> &ScopeId {
        &self.scope_id
    }

    fn check_scope(&self, scope: &ScopeId) -> Result<(), PortError> {
        if scope == &self.scope_id {
            return Ok(());
        }
        Err(backend(format!(
            "this adapter serves the scope {} only",
            self.scope_id.as_str()
        )))
    }

    fn slot(&self) -> MutexGuard<'_, Option<Loaded>> {
        self.loaded.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Answer from the loaded scope, loading it first when needed.
    ///
    /// A TeamStore lists documents only through `export`, and gives document
    /// revisions only through a snapshot read. So a load is one snapshot, one
    /// export, and one snapshot read for each exported document.
    fn with_loaded<R>(
        &self,
        answer: impl FnOnce(&Loaded) -> Result<R, PortError>,
    ) -> Result<R, PortError> {
        let mut slot = self.slot();
        if slot.is_none() {
            let snapshot = self.store.snapshot(&self.scope).map_err(port_error)?;
            let bundle = self.store.export(&self.scope).map_err(port_error)?;
            let mut docs = BTreeMap::new();
            for entry in bundle.documents {
                let Some(key) = key_of(&entry.doc) else {
                    continue;
                };
                // A document the export lists but the snapshot does not hold
                // was written after the snapshot: it is outside this view.
                if let Some(doc) = snapshot.read(&entry.doc).map_err(port_error)? {
                    docs.insert(key, (doc.content, doc.revision));
                }
            }
            *slot = Some(Loaded {
                revision: snapshot.revision(),
                docs,
            });
        }
        answer(slot.as_ref().expect("the scope was just loaded"))
    }
}

/// A TeamStore has no cheap last-modified time, so the adapter reports 0.
fn no_time() -> Timestamp {
    Timestamp::new(0).expect("0 is a timestamp")
}

impl StorePort for TeamStorePort<'_> {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.check_scope(scope)?;
        self.with_loaded(|loaded| {
            let entries = loaded
                .docs
                .iter()
                .filter(|(key, _)| prefix.matches(key))
                .map(|(key, (_, revision))| {
                    Ok(ListEntry {
                        key: key.clone(),
                        version: version_of(*revision)?,
                        modified_at: no_time(),
                    })
                })
                .collect::<Result<Vec<ListEntry>, PortError>>()?;
            Ok(Listing {
                version: version_of(loaded.revision)?,
                entries,
            })
        })
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.check_scope(scope)?;
        self.with_loaded(|loaded| {
            keys.iter()
                .map(|key| {
                    loaded
                        .docs
                        .get(key)
                        .map(|(content, revision)| {
                            Ok(Document {
                                content: content.clone(),
                                version: version_of(*revision)?,
                                modified_at: no_time(),
                            })
                        })
                        .transpose()
                })
                .collect()
        })
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.check_scope(scope)?;
        // Every key is checked before the TeamStore sees anything.
        let docs = commit
            .ops()
            .iter()
            .map(|op| {
                doc_of(op.key()).ok_or_else(|| {
                    backend(format!(
                        "the key {} names no TeamStore document kind",
                        op.key().as_str()
                    ))
                })
            })
            .collect::<Result<Vec<DocumentId>, PortError>>()?;
        let at = i64::try_from(commit.at().get())
            .ok()
            .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_millis)
            .ok_or_else(|| backend("the commit time is out of range"))?;
        let actor = match commit.actor().display_name() {
            "" => commit.actor().id(),
            name => name,
        };

        let mut uow = self
            .store
            .begin_unit_of_work(
                &self.scope,
                CommandContext {
                    command: commit.command().to_string(),
                    actor: actor.to_string(),
                },
            )
            .map_err(port_error)?;
        for (op, doc) in commit.ops().iter().zip(docs) {
            match op {
                Op::Put {
                    content, expected, ..
                } => uow.put(
                    doc,
                    content.clone(),
                    match expected {
                        Expected::Absent => ExpectedRevision::Absent,
                        Expected::At(version) => ExpectedRevision::At(Revision(version.get())),
                    },
                ),
                Op::Delete { expected, .. } => uow.delete(doc, Revision(expected.get())),
            }
        }
        let records = commit
            .events()
            .iter()
            .map(|event| EventRecord {
                name: event.name().to_string(),
                payload: event.payload().clone(),
                actor: actor.to_string(),
                at,
            })
            .collect();
        let result = self.store.commit(uow, records);
        // What this adapter loaded is now out of date, also after a failure:
        // a conflict means another writer changed the scope.
        *self.slot() = None;
        version_of(result.map_err(port_error)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::local_default_binding;
    use speclink_store::memory::MemoryStore;
    use speclink_store::{
        Bundle, CommandContext, EventRecord, ImportMode, ImportReport, Manifest, OutboxCursor,
        OutboxEntry, Snapshot, UnitOfWork,
    };
    use speclink_store_port::{
        Actor, Commit, Event, Expected, KeyPrefix, Op, Timestamp, Version,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    const AT: u64 = 1_800_000_000_000;

    /// A TeamStore that counts its exports and snapshots.
    #[derive(Default)]
    struct Counting {
        inner: MemoryStore,
        exports: AtomicUsize,
        snapshots: AtomicUsize,
    }

    impl TeamStore for Counting {
        fn manifest(&self) -> Manifest {
            self.inner.manifest()
        }
        fn health(&self) -> Result<(), StoreError> {
            self.inner.health()
        }
        fn migrate(&self, target_version: u32) -> Result<(), StoreError> {
            self.inner.migrate(target_version)
        }
        fn snapshot<'a>(&'a self, scope: &Scope) -> Result<Box<dyn Snapshot + 'a>, StoreError> {
            self.snapshots.fetch_add(1, Ordering::SeqCst);
            self.inner.snapshot(scope)
        }
        fn begin_unit_of_work(
            &self,
            scope: &Scope,
            ctx: CommandContext,
        ) -> Result<UnitOfWork, StoreError> {
            self.inner.begin_unit_of_work(scope, ctx)
        }
        fn commit(&self, uow: UnitOfWork, events: Vec<EventRecord>) -> Result<Revision, StoreError> {
            self.inner.commit(uow, events)
        }
        fn rollback(&self, uow: UnitOfWork) -> Result<(), StoreError> {
            self.inner.rollback(uow)
        }
        fn export(&self, scope: &Scope) -> Result<Bundle, StoreError> {
            self.exports.fetch_add(1, Ordering::SeqCst);
            self.inner.export(scope)
        }
        fn import(&self, bundle: Bundle, mode: ImportMode) -> Result<ImportReport, StoreError> {
            self.inner.import(bundle, mode)
        }
        fn read_outbox(
            &self,
            scope: &Scope,
            from: OutboxCursor,
        ) -> Result<Vec<OutboxEntry>, StoreError> {
            self.inner.read_outbox(scope, from)
        }
        fn ack_outbox(&self, scope: &Scope, up_to: OutboxCursor) -> Result<(), StoreError> {
            self.inner.ack_outbox(scope, up_to)
        }
        fn outbox_acked(&self, scope: &Scope) -> Result<OutboxCursor, StoreError> {
            self.inner.outbox_acked(scope)
        }
    }

    fn scope() -> Scope {
        let binding = local_default_binding();
        Scope::new(binding.project, binding.repo)
    }

    fn other_scope() -> Scope {
        Scope::new(scope().project, speclink_store::RepoId::new("other"))
    }

    /// Write `docs` into `scope` of `store` in one unit of work.
    fn seed(store: &dyn TeamStore, scope: &Scope, docs: &[(DocumentId, &str)]) -> Revision {
        let mut uow = store
            .begin_unit_of_work(
                scope,
                CommandContext { command: "seed".to_string(), actor: "seed".to_string() },
            )
            .expect("begin seed unit of work");
        for (doc, content) in docs {
            uow.create(doc.clone(), *content);
        }
        store.commit(uow, Vec::new()).expect("commit seed data")
    }

    fn key(text: &str) -> DocKey {
        DocKey::new(text).unwrap()
    }

    fn prefix(text: &str) -> KeyPrefix {
        KeyPrefix::new(text).unwrap()
    }

    fn v(n: u64) -> Version {
        Version::new(n).unwrap()
    }

    fn commit_of(ops: Vec<Op>, events: Vec<Event>) -> Commit {
        Commit::new(
            Actor::new("Alice <alice@example.com>", "Alice <alice@example.com>").unwrap(),
            "new-change",
            Timestamp::new(AT).unwrap(),
            ops,
            events,
        )
        .unwrap()
    }

    fn put(k: &str, content: &str, expected: Expected) -> Op {
        Op::Put { key: key(k), content: content.to_string(), expected }
    }

    fn change_meta(change: &str) -> DocumentId {
        DocumentId::ChangeMeta { change: change.to_string() }
    }

    // --- 鍵與文件種類雙向對應 ---

    #[test]
    fn the_nine_document_kinds_map_to_keys_and_back() {
        // 規格範例「對應」的八列，加上進行中與已封存的討論。
        let archived_change = |change: &str, doc: &str| DocumentId::ArchivedChange {
            change: change.to_string(),
            doc: doc.to_string(),
        };
        let discussion = |slug: &str, archived| DocumentId::Discussion { slug: slug.to_string(), archived };
        let rows = [
            (change_meta("add-x"), "changes/add-x/.meta.yaml"),
            (
                DocumentId::ChangeArtifact {
                    change: "add-x".to_string(),
                    artifact: "specs/auth/spec.md".to_string(),
                },
                "changes/add-x/specs/auth/spec.md",
            ),
            (
                DocumentId::ChangeEvidence { change: "add-x".to_string() },
                "changes/add-x/.evidence.json",
            ),
            (
                archived_change("2026-11-10-add-x", "proposal.md"),
                "archive/changes/2026-11-10-add-x/proposal.md",
            ),
            (
                DocumentId::CanonicalSpec { capability: "auth".to_string() },
                "specs/auth/spec.md",
            ),
            (DocumentId::WorkflowConfig, "config.yaml"),
            (DocumentId::Language, "LANGUAGE.md"),
            (DocumentId::BoardOrder, "board/order"),
            (discussion("board-search", false), "discussions/board-search.md"),
            // 已封存討論的名稱段原樣對應：新寫入的帶建立日，既有的不帶。
            (
                discussion("2026-11-10-board-search", true),
                "archive/discussions/2026-11-10-board-search.md",
            ),
            (discussion("reuse", true), "archive/discussions/reuse.md"),
        ];
        for (doc, text) in rows {
            assert_eq!(key_of(&doc), Some(key(text)), "{doc:?}");
            assert_eq!(doc_of(&key(text)), Some(doc), "{text}");
        }
    }

    #[test]
    fn a_document_whose_name_cannot_be_a_key_has_no_key() {
        // `a/b` as a change name would read back as change `a`.
        assert_eq!(key_of(&change_meta("a/b")), None);
        assert_eq!(key_of(&change_meta("")), None);
        assert_eq!(
            key_of(&DocumentId::CanonicalSpec { capability: "a/b".to_string() }),
            None
        );
    }

    // --- 列出與讀取 ---

    fn seeded() -> Counting {
        let store = Counting::default();
        seed(
            &store,
            &scope(),
            &[
                (change_meta("add-x"), "schema: spec-driven\n"),
                (
                    DocumentId::ChangeArtifact {
                        change: "add-x".to_string(),
                        artifact: "tasks.md".to_string(),
                    },
                    "- [ ] 1.1 a\n",
                ),
                (DocumentId::Discussion { slug: "talk".to_string(), archived: false }, "talk"),
                (DocumentId::WorkflowConfig, "schema: spec-driven\n"),
            ],
        );
        seed(&store, &other_scope(), &[(change_meta("elsewhere"), "schema: spec-driven\n")]);
        store
    }

    #[test]
    fn listing_gives_document_revisions_the_project_revision_and_time_zero() {
        let store = seeded();
        // A later write, so document revisions differ from the project revision.
        seed(&store, &scope(), &[(DocumentId::Language, "# Vocabulary\n")]);
        let port = TeamStorePort::new(&store, scope()).unwrap();
        assert_eq!(port.scope_id().as_str(), "default/main");

        let listing = port.list(port.scope_id(), &prefix("changes/")).unwrap();
        let project = store.inner.snapshot(&scope()).unwrap().revision().0;
        assert_eq!(listing.version, v(project), "the scope version is the project revision");
        let rows: Vec<(&str, u64, u64)> = listing
            .entries
            .iter()
            .map(|e| (e.key.as_str(), e.version.get(), e.modified_at.get()))
            .collect();
        assert_eq!(
            rows,
            [("changes/add-x/.meta.yaml", 1, 0), ("changes/add-x/tasks.md", 1, 0)],
            "document versions are document revisions; the last-modified time is 0"
        );

        let docs = port
            .read(port.scope_id(), &[key("LANGUAGE.md"), key("config.yaml"), key("specs/nope/spec.md")])
            .unwrap();
        let language = docs[0].as_ref().expect("the vocabulary");
        assert_eq!((language.content.as_str(), language.modified_at.get()), ("# Vocabulary\n", 0));
        assert!(language.version > docs[1].as_ref().unwrap().version);
        assert!(docs[2].is_none());
    }

    #[test]
    fn one_execution_exports_the_scope_once() {
        // 「一次執行只匯出一次」：列出兩個前綴，讀取多份文件。
        let store = seeded();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        port.list(port.scope_id(), &prefix("changes/")).unwrap();
        port.list(port.scope_id(), &prefix("discussions/")).unwrap();
        port.read(port.scope_id(), &[key("changes/add-x/.meta.yaml"), key("changes/add-x/tasks.md")])
            .unwrap();
        port.read(port.scope_id(), &[key("config.yaml")]).unwrap();
        assert_eq!(store.exports.load(Ordering::SeqCst), 1);
        assert_eq!(store.snapshots.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn its_own_commit_makes_the_adapter_load_again() {
        let store = seeded();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        port.list(port.scope_id(), &prefix("changes/")).unwrap();
        port.commit(
            port.scope_id(),
            &commit_of(vec![put("changes/add-y/.meta.yaml", "schema: spec-driven\n", Expected::Absent)], Vec::new()),
        )
        .unwrap();
        let keys: Vec<String> = port
            .list(port.scope_id(), &prefix("changes/"))
            .unwrap()
            .entries
            .into_iter()
            .map(|e| e.key.as_str().to_string())
            .collect();
        assert!(keys.contains(&"changes/add-y/.meta.yaml".to_string()), "{keys:?}");
        assert_eq!(store.exports.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn another_scope_is_a_backend_error_and_its_documents_never_show() {
        let store = seeded();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        let elsewhere = ScopeId::new("default/other").unwrap();
        let backend = |result: Result<(), PortError>| match result {
            Err(PortError::Backend { message }) => assert!(message.contains("default/main"), "{message}"),
            other => panic!("expected a backend error, got {other:?}"),
        };
        backend(port.list(&elsewhere, &prefix("")).map(|_| ()));
        backend(port.read(&elsewhere, &[key("config.yaml")]).map(|_| ()));
        backend(
            port.commit(&elsewhere, &commit_of(vec![put("config.yaml", "x", Expected::Absent)], Vec::new()))
                .map(|_| ()),
        );
        // The bound scope lists only its own documents.
        let all = port.list(port.scope_id(), &prefix("")).unwrap();
        assert!(all.entries.iter().all(|e| !e.key.as_str().contains("elsewhere")));
        assert_eq!(all.entries.len(), 4);
    }

    // --- 提交：一個 unit of work，事件進 outbox ---

    #[test]
    fn a_commit_lands_the_document_and_its_event_together() {
        // 「commit 後 outbox 含對應事件」
        let store = Counting::default();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        let event = Event::new("change-created", serde_json::json!({ "change": "add-auth" })).unwrap();

        let version = port
            .commit(
                port.scope_id(),
                &commit_of(
                    vec![put("changes/add-auth/.meta.yaml", "schema: spec-driven\n", Expected::Absent)],
                    vec![event],
                ),
            )
            .expect("commit succeeds");

        let entries = store.inner.read_outbox(&scope(), OutboxCursor(0)).expect("outbox reads");
        assert_eq!(entries.len(), 1, "exactly one event record landed");
        let record = &entries[0].record;
        assert_eq!(record.name, "change-created");
        assert_eq!(record.payload, serde_json::json!({ "change": "add-auth" }));
        assert_eq!(record.actor, "Alice <alice@example.com>");
        assert_eq!(record.at.timestamp_millis(), AT as i64, "the time of the commit");
        assert_eq!(entries[0].revision.0, version.get(), "the event rides its commit");

        let snapshot = store.inner.snapshot(&scope()).expect("snapshot");
        let doc = snapshot.read(&change_meta("add-auth")).expect("read").expect("visible");
        assert_eq!(doc.revision.0, version.get(), "document and event share the commit revision");
        let history = snapshot.history(&change_meta("add-auth")).expect("history");
        assert_eq!((history[0].command.as_str(), history[0].actor.as_str()), ("new-change", "Alice <alice@example.com>"));
    }

    #[test]
    fn a_commit_is_one_unit_of_work_with_writes_and_deletes() {
        let store = seeded();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        let before = store.inner.snapshot(&scope()).unwrap().revision().0;
        port.commit(
            port.scope_id(),
            &commit_of(
                vec![
                    put("archive/changes/2026-11-10-add-x/.meta.yaml", "schema: spec-driven\n", Expected::Absent),
                    Op::Delete { key: key("changes/add-x/.meta.yaml"), expected: v(1) },
                    put("changes/add-x/tasks.md", "- [x] 1.1 a\n", Expected::At(v(1))),
                ],
                Vec::new(),
            ),
        )
        .expect("commit");
        let snapshot = store.inner.snapshot(&scope()).unwrap();
        assert_eq!(snapshot.revision().0, before + 1, "one commit, one revision");
        assert!(snapshot.read(&change_meta("add-x")).unwrap().is_none());
        assert!(snapshot
            .read(&DocumentId::ArchivedChange {
                change: "2026-11-10-add-x".to_string(),
                doc: ".meta.yaml".to_string()
            })
            .unwrap()
            .is_some());
    }

    #[test]
    fn the_loser_of_two_commits_on_one_document_gets_the_conflict() {
        // 「CAS 衝突原樣傳遞」
        let store = Counting::default();
        let base = seed(&store, &scope(), &[(change_meta("contested"), "v1")]);
        let port = TeamStorePort::new(&store, scope()).unwrap();
        let write = |content: &str| {
            port.commit(
                port.scope_id(),
                &commit_of(
                    vec![put("changes/contested/.meta.yaml", content, Expected::At(v(base.0)))],
                    vec![Event::new("change-created", serde_json::json!({})).unwrap()],
                ),
            )
        };
        let after = write("v2 — winner").expect("the winner commits");
        assert_eq!(
            write("v2 — loser"),
            Err(PortError::Conflict {
                key: key("changes/contested/.meta.yaml"),
                expected: Expected::At(v(base.0)),
                actual: Some(after),
            })
        );
        assert_eq!(
            store.inner.read_outbox(&scope(), OutboxCursor(0)).unwrap().len(),
            1,
            "the loser's event did not land"
        );
    }

    #[test]
    fn a_key_of_no_document_kind_reads_as_nothing_and_cannot_be_written() {
        // 「對不到文件種類的鍵」
        let store = seeded();
        let port = TeamStorePort::new(&store, scope()).unwrap();
        let before = store.inner.snapshot(&scope()).unwrap().revision();

        assert_eq!(port.read(port.scope_id(), &[key("notes/today.md")]).unwrap(), [None]);
        let err = port
            .commit(
                port.scope_id(),
                &commit_of(
                    vec![
                        put("changes/add-y/.meta.yaml", "schema: spec-driven\n", Expected::Absent),
                        put("notes/today.md", "x", Expected::Absent),
                    ],
                    Vec::new(),
                ),
            )
            .expect_err("no document kind");
        match err {
            PortError::Backend { message } => assert!(message.contains("notes/today.md"), "{message}"),
            other => panic!("expected a backend error, got {other:?}"),
        }
        let snapshot = store.inner.snapshot(&scope()).unwrap();
        assert_eq!(snapshot.revision(), before, "the TeamStore did not change");
        assert!(snapshot.read(&change_meta("add-y")).unwrap().is_none());
    }

    #[test]
    fn an_unavailable_store_stays_a_temporary_failure() {
        assert!(matches!(port_error(StoreError::Unavailable), PortError::Unavailable { .. }));
        for other in [
            StoreError::NotFound,
            StoreError::PermissionDenied,
            StoreError::Corrupt { reason: "truncated".to_string() },
            StoreError::Backend { source: "io: disk full".to_string() },
        ] {
            let text = other.to_string();
            match port_error(other) {
                PortError::Backend { message } => assert_eq!(message, text),
                mapped => panic!("expected a backend error, got {mapped:?}"),
            }
        }
    }
}

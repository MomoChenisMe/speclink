//! A TeamStore as a store port (design D5, D6). Transitional.
//!
//! The reference server keeps its three TeamStore drivers while the Engine
//! moves to the store port. [`TeamStorePort`] wraps a TeamStore as a
//! [`StorePort`], so the server runs every verb and every document read on
//! the same path as a host that brings its own store. The adapter goes away
//! with the TeamStore.
//!
//! It does not pass the store port's conformance suite and does not have to:
//! it reports 0 as every last-modified time, and the scopes of one project
//! share their version counter.

use crate::port_view::{parse_key, KeyKind};
use speclink_store::{
    CommandContext, DocumentId, EventRecord, ExpectedRevision, ProjectId, RepoId, Revision, Scope,
    StoreError, TeamStore,
};
use speclink_store_port::{
    keys, Commit, DocKey, Document, Expected, KeyPrefix, ListEntry, Listing, Op, PortError,
    ScopeId, StorePort, Timestamp, Version,
};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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
    let key = match doc {
        DocumentId::ChangeMeta { change } => keys::change_meta(change),
        DocumentId::ChangeEvidence { change } => keys::change_evidence(change),
        DocumentId::ChangeArtifact { change, artifact } => keys::change_artifact(change, artifact),
        DocumentId::CanonicalSpec { capability } => keys::spec(capability),
        DocumentId::Discussion {
            slug,
            archived: false,
        } => keys::discussion(slug),
        // The contract's builders for the two archived kinds take dated names
        // only. A TeamStore holds older records without a date, so these keep
        // the stored name as it is.
        DocumentId::ArchivedChange { change, doc } => {
            DocKey::new(format!("{}{change}/{doc}", keys::ARCHIVED_CHANGES))
        }
        DocumentId::Discussion {
            slug,
            archived: true,
        } => DocKey::new(format!("{}{slug}.md", keys::ARCHIVED_DISCUSSIONS)),
        DocumentId::WorkflowConfig => Ok(keys::config()),
        DocumentId::Language => Ok(keys::language()),
        DocumentId::BoardOrder => Ok(keys::board_order()),
    }
    .ok()?;
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
            message: "the TeamStore is temporarily unavailable".to_string(),
        },
        other => backend(other.to_string()),
    }
}

/// The store-port scope id of a TeamStore scope: the project and the repo
/// joined by `/`. Fails when the names cannot make a scope id.
pub fn scope_id(scope: &Scope) -> Result<ScopeId, PortError> {
    ScopeId::new(format!(
        "{}/{}",
        scope.project.as_str(),
        scope.repo.as_str()
    ))
    .map_err(|e| backend(e.to_string()))
}

/// The TeamStore scope a scope id names. A scope id that is not
/// `<project>/<repo>` is a backend error, before the TeamStore sees anything.
fn teamstore_scope(scope: &ScopeId) -> Result<Scope, PortError> {
    match scope.as_str().split_once('/') {
        Some((project, repo)) if !project.is_empty() && !repo.is_empty() => {
            Ok(Scope::new(ProjectId::new(project), RepoId::new(repo)))
        }
        _ => Err(backend(format!(
            "the scope id {} is not <project>/<repo>",
            scope.as_str()
        ))),
    }
}

/// A scope as one fixed point: the project revision and every document that
/// has a key, with its content and revision.
struct Loaded {
    revision: Revision,
    docs: BTreeMap<DocKey, (String, Revision)>,
}

/// A TeamStore as a store port, for any number of scopes.
///
/// One adapter can live as long as its store and serve every request. Each
/// list and read takes a snapshot of the scope to learn the project revision.
/// When the adapter holds a load of the scope at that revision, it answers
/// from it; otherwise it loads the scope again (one export, and one snapshot
/// read for each exported document). So a run of calls with no commit in
/// between exports once, and an answer is never older than the revision at
/// the start of its call. The adapter's own commit drops the scope's load.
/// Each scope keeps at most one load.
///
/// `S` is how the adapter holds its store: a reference for one command, or
/// an `Arc` for an adapter that lives as long as the server.
pub struct TeamStorePort<S> {
    store: S,
    loaded: Mutex<BTreeMap<ScopeId, Arc<Loaded>>>,
}

impl<S> TeamStorePort<S>
where
    S: Deref,
    S::Target: TeamStore,
{
    /// Wrap `store`. The adapter is bound to no scope: every call names its
    /// own.
    pub fn new(store: S) -> Self {
        TeamStorePort {
            store,
            loaded: Mutex::new(BTreeMap::new()),
        }
    }

    fn slots(&self) -> MutexGuard<'_, BTreeMap<ScopeId, Arc<Loaded>>> {
        self.loaded.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The scope at the current project revision, loading it when the
    /// adapter holds no load at that revision.
    ///
    /// A TeamStore lists documents only through `export`, and gives document
    /// revisions only through a snapshot read. So a load is one export and
    /// one snapshot read for each exported document. The lock is not held
    /// while the TeamStore works: another scope never waits for this one.
    fn loaded(&self, scope_id: &ScopeId) -> Result<Arc<Loaded>, PortError> {
        let scope = teamstore_scope(scope_id)?;
        let snapshot = self.store.snapshot(&scope).map_err(port_error)?;
        let revision = snapshot.revision();
        if let Some(loaded) = self.slots().get(scope_id) {
            if loaded.revision == revision {
                return Ok(Arc::clone(loaded));
            }
        }
        let bundle = self.store.export(&scope).map_err(port_error)?;
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
        let loaded = Arc::new(Loaded { revision, docs });
        self.slots().insert(scope_id.clone(), Arc::clone(&loaded));
        Ok(loaded)
    }
}

/// A TeamStore has no cheap last-modified time, so the adapter reports 0.
fn no_time() -> Timestamp {
    Timestamp::new(0).expect("0 is a timestamp")
}

impl<S> StorePort for TeamStorePort<S>
where
    S: Deref + Send + Sync,
    S::Target: TeamStore,
{
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        let loaded = self.loaded(scope)?;
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
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        let loaded = self.loaded(scope)?;
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
    }

    fn commit(&self, scope_id: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        let scope = teamstore_scope(scope_id)?;
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
                &scope,
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
        self.slots().remove(scope_id);
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
    struct ExportCounter {
        inner: MemoryStore,
        exports: AtomicUsize,
        snapshots: AtomicUsize,
    }

    impl TeamStore for ExportCounter {
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
        // The key naming rules keep `archive` for the archive folder.
        assert_eq!(key_of(&change_meta("archive")), None);
        assert_eq!(
            key_of(&DocumentId::CanonicalSpec { capability: "a/b".to_string() }),
            None
        );
    }

    // --- 列出與讀取 ---

    fn sid() -> ScopeId {
        scope_id(&scope()).unwrap()
    }

    fn seeded() -> ExportCounter {
        let store = ExportCounter::default();
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

    fn exports(store: &ExportCounter) -> usize {
        store.exports.load(Ordering::SeqCst)
    }

    #[test]
    fn the_scope_id_is_the_project_and_the_repo() {
        assert_eq!(sid().as_str(), "default/main");
    }

    #[test]
    fn listing_gives_document_revisions_the_project_revision_and_time_zero() {
        let store = seeded();
        // A later write, so document revisions differ from the project revision.
        seed(&store, &scope(), &[(DocumentId::Language, "# Vocabulary\n")]);
        let port = TeamStorePort::new(&store);

        let listing = port.list(&sid(), &prefix("changes/")).unwrap();
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
            .read(&sid(), &[key("LANGUAGE.md"), key("config.yaml"), key("specs/nope/spec.md")])
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
        let port = TeamStorePort::new(&store);
        port.list(&sid(), &prefix("changes/")).unwrap();
        port.list(&sid(), &prefix("discussions/")).unwrap();
        port.read(&sid(), &[key("changes/add-x/.meta.yaml"), key("changes/add-x/tasks.md")])
            .unwrap();
        port.read(&sid(), &[key("config.yaml")]).unwrap();
        assert_eq!(exports(&store), 1);
    }

    #[test]
    fn one_adapter_serves_two_scopes_without_mixing_them() {
        // 「一個轉接服務兩個範圍」
        let store = ExportCounter::default();
        let p1 = Scope::new(speclink_store::ProjectId::new("p1"), speclink_store::RepoId::new("main"));
        let p2 = Scope::new(speclink_store::ProjectId::new("p2"), speclink_store::RepoId::new("main"));
        seed(&store, &p1, &[(change_meta("only-in-p1"), "schema: spec-driven\n")]);
        seed(&store, &p2, &[(change_meta("only-in-p2"), "schema: spec-driven\n")]);
        let port = TeamStorePort::new(&store);
        let changes = |scope: &str| -> Vec<String> {
            port.list(&ScopeId::new(scope).unwrap(), &prefix("changes/"))
                .unwrap()
                .entries
                .into_iter()
                .map(|e| e.key.as_str().to_string())
                .collect()
        };
        assert_eq!(changes("p1/main"), ["changes/only-in-p1/.meta.yaml"]);
        assert_eq!(changes("p2/main"), ["changes/only-in-p2/.meta.yaml"]);
        // Each scope keeps its own load: asking again exports nothing.
        assert_eq!(changes("p1/main"), ["changes/only-in-p1/.meta.yaml"]);
        assert_eq!(exports(&store), 2);
    }

    #[test]
    fn an_unchanged_revision_answers_without_another_export() {
        // 「revision 沒變時不再匯出」
        let store = seeded();
        let port = TeamStorePort::new(&store);
        let first = port.list(&sid(), &prefix("config.yaml")).unwrap().version;
        let second = port.list(&sid(), &prefix("config.yaml")).unwrap().version;
        assert_eq!(first, second);
        assert_eq!(exports(&store), 1, "the second call did not export");
    }

    #[test]
    fn a_commit_made_straight_to_the_teamstore_is_read_back() {
        // 「別人提交後讀到新內容」
        let store = seeded();
        let port = TeamStorePort::new(&store);
        let before = port.read(&sid(), &[key("config.yaml")]).unwrap()[0].clone().unwrap();

        let mut uow = store
            .begin_unit_of_work(
                &scope(),
                CommandContext { command: "edit".to_string(), actor: "someone".to_string() },
            )
            .unwrap();
        uow.update(DocumentId::WorkflowConfig, "schema: other\n", Revision(before.version.get()));
        let landed = store.commit(uow, Vec::new()).unwrap();

        let after = port.read(&sid(), &[key("config.yaml")]).unwrap()[0].clone().unwrap();
        assert_eq!(after.content, "schema: other\n");
        assert_eq!(after.version, v(landed.0), "the new document version");
        assert_eq!(exports(&store), 2, "the moved revision loaded the scope again");
    }

    #[test]
    fn its_own_commit_is_read_back() {
        // 「自己提交後讀到新內容」
        let store = seeded();
        let port = TeamStorePort::new(&store);
        port.list(&sid(), &prefix("changes/")).unwrap();
        let version = port
            .commit(
                &sid(),
                &commit_of(vec![put("changes/add-y/.meta.yaml", "schema: spec-driven\n", Expected::Absent)], Vec::new()),
            )
            .unwrap();
        let doc = port.read(&sid(), &[key("changes/add-y/.meta.yaml")]).unwrap()[0].clone();
        let doc = doc.expect("the document it just wrote");
        assert_eq!((doc.content.as_str(), doc.version), ("schema: spec-driven\n", version));
        assert_eq!(exports(&store), 2);
    }

    #[test]
    fn a_scope_id_without_a_slash_is_a_backend_error_before_the_teamstore_is_called() {
        // 「範圍識別的形式不合」
        let store = seeded();
        let (exports_before, snapshots_before) =
            (exports(&store), store.snapshots.load(Ordering::SeqCst));
        let port = TeamStorePort::new(&store);
        let bad = ScopeId::new("no-slash").unwrap();
        let backend = |result: Result<(), PortError>| match result {
            Err(PortError::Backend { message }) => assert!(message.contains("no-slash"), "{message}"),
            other => panic!("expected a backend error, got {other:?}"),
        };
        backend(port.list(&bad, &prefix("")).map(|_| ()));
        backend(port.read(&bad, &[key("config.yaml")]).map(|_| ()));
        backend(
            port.commit(&bad, &commit_of(vec![put("config.yaml", "x", Expected::Absent)], Vec::new()))
                .map(|_| ()),
        );
        assert_eq!(exports(&store), exports_before);
        assert_eq!(store.snapshots.load(Ordering::SeqCst), snapshots_before, "no snapshot either");
    }

    #[test]
    fn a_scope_lists_only_its_own_documents() {
        let store = seeded();
        let port = TeamStorePort::new(&store);
        let all = port.list(&sid(), &prefix("")).unwrap();
        assert!(all.entries.iter().all(|e| !e.key.as_str().contains("elsewhere")));
        assert_eq!(all.entries.len(), 4);
    }

    #[test]
    fn the_adapter_is_send_and_sync_and_can_be_shared() {
        fn shared<T: Send + Sync + 'static>(_: &T) {}
        let store: std::sync::Arc<dyn TeamStore + Send + Sync> = std::sync::Arc::new(MemoryStore::default());
        let port = std::sync::Arc::new(TeamStorePort::new(store));
        shared(&port);
        let as_port: std::sync::Arc<dyn StorePort> = port;
        assert_eq!(as_port.list(&sid(), &prefix("")).unwrap().version, Version::ZERO);
    }

    // --- 提交：一個 unit of work，事件進 outbox ---

    #[test]
    fn a_commit_lands_the_document_and_its_event_together() {
        // 「commit 後 outbox 含對應事件」
        let store = ExportCounter::default();
        let port = TeamStorePort::new(&store);
        let event = Event::new("change-created", serde_json::json!({ "change": "add-auth" })).unwrap();

        let version = port
            .commit(
                &sid(),
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
    fn a_commit_with_events_only_lands_the_events() {
        // The store contract allows a commit with no operation. It becomes a
        // unit of work with no staged write.
        let store = seeded();
        let port = TeamStorePort::new(&store);
        let before = store.inner.snapshot(&scope()).unwrap().revision().0;
        let event = Event::new("task-moved", serde_json::json!({ "change": "add-x" })).unwrap();
        let version = port
            .commit(&sid(), &commit_of(Vec::new(), vec![event]))
            .expect("an events-only commit");
        assert_eq!(version.get(), before + 1);
        let entries = store.inner.read_outbox(&scope(), OutboxCursor(0)).expect("outbox");
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].record.name.as_str(), entries[0].revision.0), ("task-moved", before + 1));
    }

    #[test]
    fn a_verb_run_through_the_bridge_exports_the_scope_once() {
        // 「一次執行只匯出一次」，經橋接執行一個會寫入的動詞。
        let store = seeded();
        let done = crate::bridge::execute(
            &store,
            &crate::port_view::testing::host_ctx(),
            speclink_core::command::Command::TaskDone {
                task_id: "1".to_string(),
                change: Some("add-x".to_string()),
                touched_files: Some(vec!["src/app.rs".to_string()]),
                head_commit: None,
            },
        )
        .expect("task done through the bridge");
        assert!(done.revision.is_some(), "the verb committed");
        assert_eq!(exports(&store), 1);
    }

    #[test]
    fn a_commit_is_one_unit_of_work_with_writes_and_deletes() {
        let store = seeded();
        let port = TeamStorePort::new(&store);
        let before = store.inner.snapshot(&scope()).unwrap().revision().0;
        port.commit(
            &sid(),
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
        let store = ExportCounter::default();
        let base = seed(&store, &scope(), &[(change_meta("contested"), "v1")]);
        let port = TeamStorePort::new(&store);
        let write = |content: &str| {
            port.commit(
                &sid(),
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
        let port = TeamStorePort::new(&store);
        let before = store.inner.snapshot(&scope()).unwrap().revision();

        assert_eq!(port.read(&sid(), &[key("notes/today.md")]).unwrap(), [None]);
        let err = port
            .commit(
                &sid(),
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

//! The export and the import of a scope on the three operations of a store
//! port (design D1–D3): what the two functions give back, what they refuse,
//! and what they ask of the store.

use speclink_host::documents::BOARD_ORDER_LIMIT;
use speclink_host::transfer::{self, BundleDocument, ImportMode, ImportOptions, ScopeBundle, TransferError};
use speclink_protocol::query::{ImportBundle, ImportBundleDocument, ImportDocumentId, ImportScope};
use speclink_host::projection::content_digest;
use speclink_store_port::{
    Actor, Commit, DocKey, Document, Expected, KeyPrefix, Listing, MemoryStorePort, Op, PortError, ScopeId, StorePort,
    Timestamp, Version,
};

mod common;
use common::{scope, seed_in, spec_folder_docs, write_in, Recorder};

const AT: u64 = 1_800_000_000_000;

/// One document of every kind the Engine writes.
const EVERY_KIND: &[(&str, &str)] = &[
    ("changes/add-x/.meta.yaml", "schema: spec-driven\ncreated: 2026-11-10\n"),
    ("changes/add-x/proposal.md", "## Why\n\nA change in progress.\n"),
    ("changes/add-x/specs/auth/spec.md", "## ADDED Requirements\n"),
    ("changes/add-x/.evidence.json", "{\"entries\":[]}\n"),
    ("specs/auth/spec.md", "# auth Specification\n"),
    ("discussions/talk.md", "---\ntopic: Talk\nslug: talk\nstatus: open\ncreated: 2026-11-10\n---\n"),
    ("archive/discussions/2026-11-09-old-talk.md", "---\ntopic: Old\nslug: old-talk\nstatus: concluded\n---\n"),
    ("archive/changes/2026-11-09-add-w/proposal.md", "## Why\n\nAn archived change.\n"),
    ("archive/changes/2026-11-09-add-w/.evidence.json", "{\"entries\":[]}\n"),
    ("config.yaml", "schema: spec-driven\n"),
    ("LANGUAGE.md", "TeamStore: 團隊儲存\n"),
    ("board/order", "{\"changes\":{\"add-x\":\"a0\"},\"discussions\":{}}"),
];

fn target() -> ScopeId {
    ScopeId::new("default/copy").unwrap()
}

fn actor() -> Actor {
    Actor::new("u_42", "Alice").unwrap()
}

fn at() -> Timestamp {
    Timestamp::new(AT).unwrap()
}

fn everything() -> KeyPrefix {
    KeyPrefix::new("").unwrap()
}

fn document(key: &str, content: &str) -> BundleDocument {
    BundleDocument { key: key.to_string(), content: content.to_string(), digest: content_digest(content) }
}

fn bundle(documents: Vec<BundleDocument>) -> ScopeBundle {
    ScopeBundle { format_version: 2, scope_version: None, documents }
}

fn bundle_of(docs: &[(&str, &str)]) -> ScopeBundle {
    bundle(docs.iter().map(|(key, content)| document(key, content)).collect())
}

/// Every key of `scope` with its content, in key order.
fn contents(port: &dyn StorePort, scope: &ScopeId) -> Vec<(String, String)> {
    let keys: Vec<DocKey> = port.list(scope, &everything()).unwrap().entries.into_iter().map(|e| e.key).collect();
    let docs = port.read(scope, &keys).unwrap();
    keys.iter()
        .zip(docs)
        .map(|(key, doc)| (key.as_str().to_string(), doc.expect("a listed document").content))
        .collect()
}

fn import(port: &dyn StorePort, scope: &ScopeId, bundle: ScopeBundle) -> Result<transfer::ImportReport, TransferError> {
    transfer::import(port, scope, &actor(), bundle, at())
}

/// The text of the refusal of a bundle that is not valid as a whole.
fn invalid(result: Result<transfer::ImportReport, TransferError>) -> String {
    match result {
        Err(TransferError::InvalidBundle(message)) => message,
        other => panic!("expected an invalid bundle, got {other:?}"),
    }
}

/// The text of the refusal of a bundle one of whose documents breaks a rule.
fn invalid_document(result: Result<transfer::ImportReport, TransferError>) -> String {
    match result {
        Err(TransferError::InvalidDocument(message)) => message,
        other => panic!("expected a document that is not valid, got {other:?}"),
    }
}

// --- the export ---

#[test]
fn an_export_imported_into_an_empty_scope_gives_the_same_keys_and_contents() {
    let port = MemoryStorePort::new();
    write_in(&port, &scope(), EVERY_KIND);

    let exported = transfer::export(&port, &scope()).expect("the export");
    let report = import(&port, &target(), exported).expect("the import");

    assert_eq!(report.documents, EVERY_KIND.len());
    assert_eq!(contents(&port, &target()), contents(&port, &scope()));
    assert_eq!(contents(&port, &target()).len(), EVERY_KIND.len());
}

#[test]
fn an_export_is_in_key_order_with_each_digest_and_the_scope_version() {
    let port = MemoryStorePort::new();
    write_in(&port, &scope(), EVERY_KIND);
    write_in(&port, &scope(), &[("config.yaml", "schema: spec-driven\nlocale: tw\n")]);
    let version = port.list(&scope(), &everything()).unwrap().version;

    let exported = transfer::export(&port, &scope()).expect("the export");

    assert_eq!(exported.format_version, 2);
    assert_eq!(exported.scope_version, Some(version.get()));
    let keys: Vec<&str> = exported.documents.iter().map(|doc| doc.key.as_str()).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
    assert_eq!(keys.len(), EVERY_KIND.len());
    for doc in &exported.documents {
        assert_eq!(doc.digest, content_digest(&doc.content), "{}", doc.key);
    }
    let config = exported.documents.iter().find(|doc| doc.key == "config.yaml").unwrap();
    assert_eq!(config.content, "schema: spec-driven\nlocale: tw\n");
}

#[test]
fn an_empty_scope_exports_a_bundle_with_no_documents() {
    let exported = transfer::export(&MemoryStorePort::new(), &scope()).expect("an empty scope is no failure");
    assert_eq!(exported.format_version, 2);
    assert_eq!(exported.scope_version, Some(0));
    assert!(exported.documents.is_empty());
}

#[test]
fn an_export_reads_in_batches_of_200_keys_at_most() {
    let store = Recorder::new(MemoryStorePort::new());
    let docs = (0..450)
        .map(|i| (DocKey::new(format!("specs/cap-{i:03}/spec.md")).unwrap(), format!("# cap {i}\n")))
        .collect();
    seed_in(&store.inner, &scope(), docs);

    let exported = transfer::export(&store, &scope()).expect("the export");

    assert_eq!(exported.documents.len(), 450);
    let calls = store.take();
    let sizes: Vec<usize> = calls.reads.iter().map(Vec::len).collect();
    assert_eq!(sizes, [200, 200, 50]);
    assert_eq!(calls.lists.len(), 2, "one list before the reads and one after: {:?}", calls.lists);
    assert!(calls.commits.is_empty());
}

#[test]
fn an_export_starts_again_when_the_scope_changes_during_it() {
    let store = Recorder::new(MemoryStorePort::new())
        .before_first_read(|inner| write_in(inner, &scope(), &[("config.yaml", "schema: spec-driven\nlocale: ja\n")]));
    write_in(&store.inner, &scope(), EVERY_KIND);
    store.take();

    let exported = transfer::export(&store, &scope()).expect("the second try is stable");

    let now = store.inner.list(&scope(), &everything()).unwrap().version;
    assert_eq!(exported.scope_version, Some(now.get()));
    let config = exported.documents.iter().find(|doc| doc.key == "config.yaml").unwrap();
    assert_eq!(config.content, "schema: spec-driven\nlocale: ja\n");
    assert_eq!(store.take().lists.len(), 4, "two tries, two lists each");
}

#[test]
fn an_export_of_a_scope_that_never_stops_changing_ends_as_unavailable_after_three_tries() {
    let store = Recorder::never_still();
    write_in(&store.inner, &scope(), EVERY_KIND);

    let result = transfer::export(&store, &scope());

    assert!(matches!(result, Err(TransferError::Unstable)), "{result:?}");
    assert_eq!(store.take().reads.len(), 3, "three tries, one batch each");
    assert!(TransferError::Unstable.to_string().contains("changed"), "{}", TransferError::Unstable);
}

/// A store whose read gives one document fewer than it was asked for.
struct ShortRead(MemoryStorePort);

impl StorePort for ShortRead {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.0.list(scope, prefix)
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        let mut docs = self.0.read(scope, keys)?;
        docs.pop();
        Ok(docs)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.0.commit(scope, commit)
    }
}

#[test]
fn an_export_from_a_store_that_reads_fewer_documents_than_asked_is_a_store_failure() {
    // A bundle with a document missing is no backup: the export fails.
    let store = ShortRead(MemoryStorePort::new());
    write_in(&store.0, &scope(), EVERY_KIND);

    let result = transfer::export(&store, &scope());

    match result {
        Err(TransferError::Store(PortError::Backend { message })) => {
            assert!(message.contains("11") && message.contains("12"), "{message}")
        }
        other => panic!("expected a store failure, got {other:?}"),
    }
}

// --- the import ---

#[test]
fn an_import_is_one_commit_of_writes_that_expect_no_document_and_one_event() {
    let store = Recorder::new(MemoryStorePort::new());
    let docs: Vec<BundleDocument> =
        (0..300).map(|i| document(&format!("specs/cap-{i:03}/spec.md"), &format!("# cap {i}\n"))).collect();

    let report = import(&store, &scope(), bundle(docs)).expect("the import");

    let calls = store.take();
    assert_eq!(calls.commits.len(), 1);
    let commit = &calls.commits[0];
    assert_eq!(commit.ops().len(), 300);
    assert!(commit.ops().iter().all(|op| matches!(op, Op::Put { expected: Expected::Absent, .. })));
    let events: Vec<(&str, &serde_json::Value)> = commit.events().iter().map(|e| (e.name(), e.payload())).collect();
    assert_eq!(events, [("scope-imported", &serde_json::json!({ "documents": 300 }))]);
    assert_eq!(commit.command(), "scope-import");
    assert_eq!((commit.actor().id(), commit.actor().display_name()), ("u_42", "Alice"));
    assert_eq!(commit.at(), at());

    assert_eq!(report.documents, 300);
    assert_eq!(report.scope_version, store.inner.list(&scope(), &everything()).unwrap().version);
    assert_eq!(contents(&store.inner, &scope()).len(), 300);
}

#[test]
fn an_import_into_a_scope_that_holds_a_document_is_refused_with_no_commit() {
    let store = Recorder::new(MemoryStorePort::new());
    write_in(&store.inner, &scope(), &[("config.yaml", "schema: spec-driven\n")]);
    let before = store.inner.list(&scope(), &everything()).unwrap().version;
    store.take();

    let result = import(&store, &scope(), bundle_of(EVERY_KIND));

    assert!(matches!(result, Err(TransferError::NotEmpty)), "{result:?}");
    assert!(store.take().commits.is_empty());
    assert_eq!(contents(&store.inner, &scope()), [("config.yaml".to_string(), "schema: spec-driven\n".to_string())]);
    assert_eq!(store.inner.list(&scope(), &everything()).unwrap().version, before);
}

#[test]
fn a_writer_that_lands_between_the_empty_check_and_the_commit_refuses_the_import() {
    // Two imports of the same project at the same moment: the other one's
    // config is there when this one commits.
    let store = Recorder::new(MemoryStorePort::new())
        .before_first_commit(|inner| write_in(inner, &scope(), &[("config.yaml", "schema: other\n")]));

    let result = import(&store, &scope(), bundle_of(EVERY_KIND));

    assert!(matches!(result, Err(TransferError::NotEmpty)), "{result:?}");
    assert_eq!(
        contents(&store.inner, &scope()),
        [("config.yaml".to_string(), "schema: other\n".to_string())],
        "no document of the bundle was written"
    );
}

#[test]
fn a_writer_of_a_key_the_bundle_does_not_hold_is_not_seen_and_both_stay() {
    // A commit can state what it expects of its own keys only: the import
    // lands whole, next to the other writer's document.
    let store = Recorder::new(MemoryStorePort::new())
        .before_first_commit(|inner| write_in(inner, &scope(), &[("specs/other/spec.md", "# other\n")]));

    let report = import(&store, &scope(), bundle_of(EVERY_KIND)).expect("the import");

    assert_eq!(report.documents, EVERY_KIND.len());
    let mut expected: Vec<(String, String)> =
        EVERY_KIND.iter().map(|(key, content)| (key.to_string(), content.to_string())).collect();
    expected.push(("specs/other/spec.md".to_string(), "# other\n".to_string()));
    expected.sort();
    assert_eq!(contents(&store.inner, &scope()), expected);
}

#[test]
fn a_bundle_that_is_not_valid_is_refused_by_key_before_the_store_is_called() {
    let wrong_digest = BundleDocument {
        key: "specs/auth/spec.md".to_string(),
        content: "# auth\n".to_string(),
        digest: content_digest("# other\n"),
    };
    let large_order = "x".repeat(BOARD_ORDER_LIMIT + 1);
    let rows: Vec<(&str, Vec<BundleDocument>, &str)> = vec![
        // The write route of the board order refuses this size: an import
        // must not leave an order that route can never write again.
        ("a board order over its limit", vec![document("board/order", &large_order)], "board/order"),
        ("a digest that is not the content's", vec![document("config.yaml", "a\n"), wrong_digest], "specs/auth/spec.md"),
        ("a key outside the Engine's naming rules", vec![document("secrets/token", "x")], "secrets/token"),
        // The naming rules keep `archive` for the archive folder: the Engine
        // never writes a change of that name, so a bundle cannot either.
        (
            "a change named 'archive'",
            vec![document("changes/archive/2026-01-01-x/proposal.md", "x")],
            "changes/archive/2026-01-01-x/proposal.md",
        ),
        ("a spec file that is not spec.md", vec![document("specs/auth/notes.md", "x")], "specs/auth/notes.md"),
        ("a key with a '..' segment", vec![document("changes/../config.yaml", "x")], "changes/../config.yaml"),
        ("a key that starts with '/'", vec![document("/config.yaml", "x")], "/config.yaml"),
        ("a key with a backslash", vec![document("changes\\add-x\\proposal.md", "x")], "changes"),
        ("an empty key", vec![document("", "x")], "document key"),
        (
            "a key that appears twice",
            vec![document("config.yaml", "a\n"), document("LANGUAGE.md", "b\n"), document("config.yaml", "c\n")],
            "config.yaml",
        ),
    ];
    for (case, documents, named) in rows {
        let store = Recorder::new(MemoryStorePort::new());
        let message = invalid_document(import(&store, &scope(), bundle(documents)));
        assert!(message.contains(named), "{case}: {message}");
        assert!(store.take().is_empty(), "{case}: the store was called");
    }
}

#[test]
fn a_refusal_names_the_first_document_that_is_wrong_and_shows_no_content() {
    let store = Recorder::new(MemoryStorePort::new());
    let secret = "the content of a document never goes into an error";
    let documents = vec![document("config.yaml", "a\n"), document("secrets/token", secret), document("other/x", secret)];
    let message = invalid_document(import(&store, &scope(), bundle(documents)));
    assert!(message.contains("secrets/token") && !message.contains("other/x"), "{message}");
    assert!(!message.contains(secret), "{message}");
}

#[test]
fn a_refusal_never_grows_with_what_the_bundle_holds() {
    // A name or a key of any length is cut in the message.
    let store = Recorder::new(MemoryStorePort::new());
    let long_key = format!("secrets/{}", "k".repeat(400));
    let message = invalid_document(import(&store, &scope(), bundle(vec![document(&long_key, "x")])));
    assert!(message.len() < 400, "{} bytes", message.len());

    let long_name = "n/".repeat(100_000);
    let first = first_version(vec![(ImportDocumentId::ChangeMeta { change: long_name }, "x")]);
    match transfer::from_first_version(first) {
        Err(TransferError::InvalidBundle(message)) => assert!(message.len() < 400, "{} bytes", message.len()),
        other => panic!("expected an invalid bundle, got {other:?}"),
    }

    // A value of the wrong type is not written back whole.
    let text = format!("{{\"formatVersion\":\"{}\"}}", "v".repeat(100_000));
    let unread = transfer::read_bundle(text.as_bytes()).expect_err("not a bundle");
    assert!(unread.to_string().len() < 400, "{} bytes", unread.to_string().len());
}

#[test]
fn a_bundle_of_another_format_version_is_not_valid() {
    for version in [0, 1, 3] {
        let store = Recorder::new(MemoryStorePort::new());
        let wrong = ScopeBundle { format_version: version, ..bundle_of(EVERY_KIND) };
        let message = invalid(import(&store, &scope(), wrong));
        assert!(message.contains("format version"), "{message}");
        assert!(store.take().is_empty());
    }
}

#[test]
fn a_bundle_with_no_documents_commits_nothing() {
    let store = Recorder::new(MemoryStorePort::new());
    let report = import(&store, &scope(), bundle(Vec::new())).expect("an empty bundle is no failure");
    assert_eq!((report.documents, report.scope_version), (0, Version::ZERO));
    assert!(store.take().commits.is_empty());
}

#[test]
fn a_store_failure_comes_back_as_it_is() {
    let result = import(&Recorder::unavailable(), &scope(), bundle_of(EVERY_KIND));
    assert!(matches!(result, Err(TransferError::Store(PortError::Unavailable { .. }))), "{result:?}");
}

// --- the report and the dry run of the create mode (import-replace-mode D4) ---

const CREATE_DRY_RUN: ImportOptions = ImportOptions { mode: ImportMode::Create, dry_run: true };

#[test]
fn a_create_reports_every_key_of_the_bundle_as_added_in_byte_order() {
    // spec host-kit「建立模式的回報列出全部的鍵」.
    let store = Recorder::new(MemoryStorePort::new());
    let sent = bundle_of(&[("specs/auth/spec.md", "# auth\n"), ("LANGUAGE.md", "TeamStore: 團隊儲存\n")]);

    let report = import(&store, &scope(), sent).expect("the import");

    assert_eq!(report.documents, 2);
    assert_eq!(report.added, ["LANGUAGE.md", "specs/auth/spec.md"]);
    assert!(report.changed.is_empty() && report.removed.is_empty(), "{report:?}");
    let commits = store.take().commits;
    let events: Vec<(&str, &serde_json::Value)> = commits[0].events().iter().map(|e| (e.name(), e.payload())).collect();
    assert_eq!(events, [("scope-imported", &serde_json::json!({ "documents": 2 }))], "a create has no `removed`");
}

#[test]
fn a_dry_run_of_a_create_into_an_empty_scope_reports_and_commits_nothing() {
    let store = Recorder::new(MemoryStorePort::new());

    let report = import_with(&store, &scope(), bundle_of(EVERY_KIND), CREATE_DRY_RUN).expect("the dry run");

    let mut keys: Vec<&str> = EVERY_KIND.iter().map(|(key, _)| *key).collect();
    keys.sort_unstable();
    assert_eq!(report.added, keys);
    assert!(report.changed.is_empty() && report.removed.is_empty(), "{report:?}");
    assert_eq!((report.documents, report.scope_version), (EVERY_KIND.len(), Version::ZERO));
    assert!(store.take().commits.is_empty());
    assert!(contents(&store.inner, &scope()).is_empty());
}

#[test]
fn a_dry_run_of_a_create_into_a_scope_that_holds_a_document_is_refused_like_the_import() {
    let store = Recorder::new(MemoryStorePort::new());
    write_in(&store.inner, &scope(), &[("config.yaml", "schema: spec-driven\n")]);
    store.take();

    let result = import_with(&store, &scope(), bundle_of(EVERY_KIND), CREATE_DRY_RUN);

    assert!(matches!(result, Err(TransferError::NotEmpty)), "{result:?}");
    assert!(store.take().commits.is_empty());
}

#[test]
fn a_create_with_no_documents_reports_three_empty_lists() {
    let store = Recorder::new(MemoryStorePort::new());
    let report = import(&store, &scope(), bundle(Vec::new())).expect("an empty bundle is no failure");
    assert_eq!((report.documents, report.scope_version), (0, Version::ZERO));
    assert!(report.added.is_empty() && report.changed.is_empty() && report.removed.is_empty(), "{report:?}");
    assert!(store.take().commits.is_empty());
}

// --- the replace mode and its dry run (import-replace-mode D2, D3) ---

const REPLACE: ImportOptions = ImportOptions { mode: ImportMode::Replace, dry_run: false };
const REPLACE_DRY_RUN: ImportOptions = ImportOptions { mode: ImportMode::Replace, dry_run: true };

/// The scope of the design's example, before the replace.
const SCOPE_BEFORE: &[(&str, &str)] = &[
    ("LANGUAGE.md", "TeamStore: 團隊儲存\n"),
    ("changes/add-x/proposal.md", "## Why\n\nMade in the host.\n"),
    ("config.yaml", "schema: spec-driven\ntdd: true\n"),
    ("discussions/d1.md", "---\ntopic: D1\nslug: d1\nstatus: open\ncreated: 2026-11-10\n---\n"),
    ("specs/auth/spec.md", "# auth Specification\n\nThe old content.\n"),
];

/// The bundle of the design's example: `LANGUAGE.md` is the same as in the
/// scope.
const REPLACEMENT: &[(&str, &str)] = &[
    ("LANGUAGE.md", "TeamStore: 團隊儲存\n"),
    ("config.yaml", "schema: spec-driven\n"),
    ("specs/auth/spec.md", "# auth Specification\n\nThe new content.\n"),
    ("specs/billing/spec.md", "# billing Specification\n"),
];

fn import_with(
    port: &dyn StorePort,
    scope: &ScopeId,
    bundle: ScopeBundle,
    options: ImportOptions,
) -> Result<transfer::ImportReport, TransferError> {
    transfer::import_with(port, scope, &actor(), bundle, at(), options).map(|(report, _)| report)
}

/// A store that holds [`SCOPE_BEFORE`], with no call recorded yet.
fn store_before() -> Recorder {
    let store = Recorder::new(MemoryStorePort::new());
    write_in(&store.inner, &scope(), SCOPE_BEFORE);
    store.take();
    store
}

fn owned(docs: &[(&str, &str)]) -> Vec<(String, String)> {
    docs.iter().map(|(key, content)| (key.to_string(), content.to_string())).collect()
}

/// The version of each key of `scope`.
fn versions(port: &dyn StorePort, scope: &ScopeId) -> std::collections::BTreeMap<String, Version> {
    let entries = port.list(scope, &everything()).unwrap().entries;
    entries.into_iter().map(|entry| (entry.key.as_str().to_string(), entry.version)).collect()
}

fn version_now(port: &dyn StorePort) -> Version {
    port.list(&scope(), &everything()).unwrap().version
}

fn read_one(port: &dyn StorePort, key: &str) -> Option<Document> {
    port.read(&scope(), &[DocKey::new(key).unwrap()]).unwrap().remove(0)
}

/// Another writer deletes `key`.
fn delete_in(port: &dyn StorePort, key: &str) {
    let version = read_one(port, key).expect("a document to delete").version;
    let op = Op::Delete { key: DocKey::new(key).unwrap(), expected: version };
    let other = Actor::new("u_7", "Bob").unwrap();
    let commit = Commit::new(other, "other-writer", Timestamp::new(1).unwrap(), vec![op], Vec::new()).unwrap();
    port.commit(&scope(), &commit).expect("the other writer's delete");
}

fn assert_example_lists(report: &transfer::ImportReport) {
    assert_eq!(report.documents, 3);
    assert_eq!(report.added, ["specs/billing/spec.md"]);
    assert_eq!(report.changed, ["config.yaml", "specs/auth/spec.md"]);
    assert_eq!(report.removed, ["changes/add-x/proposal.md", "discussions/d1.md"]);
}

#[test]
fn a_replace_is_one_commit_that_makes_the_scope_equal_the_bundle() {
    // spec host-kit「取代讓範圍等於包」and its example.
    let store = store_before();
    let listed = versions(&store.inner, &scope());
    let language = read_one(&store.inner, "LANGUAGE.md").unwrap();

    let report = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");

    let calls = store.take();
    assert_eq!(calls.commits.len(), 1);
    let commit = &calls.commits[0];
    let mut ops: Vec<(&str, String)> = commit
        .ops()
        .iter()
        .map(|op| match op {
            Op::Put { key, expected: Expected::Absent, .. } => (key.as_str(), "put, expects no document".to_string()),
            Op::Put { key, expected: Expected::At(version), .. } => (key.as_str(), format!("put at {}", version.get())),
            Op::Delete { key, expected } => (key.as_str(), format!("delete at {}", expected.get())),
        })
        .collect();
    ops.sort();
    let at_listed = |key: &str| listed[key].get();
    assert_eq!(
        ops,
        [
            ("changes/add-x/proposal.md", format!("delete at {}", at_listed("changes/add-x/proposal.md"))),
            ("config.yaml", format!("put at {}", at_listed("config.yaml"))),
            ("discussions/d1.md", format!("delete at {}", at_listed("discussions/d1.md"))),
            ("specs/auth/spec.md", format!("put at {}", at_listed("specs/auth/spec.md"))),
            ("specs/billing/spec.md", "put, expects no document".to_string()),
        ]
    );
    let events: Vec<(&str, &serde_json::Value)> = commit.events().iter().map(|e| (e.name(), e.payload())).collect();
    assert_eq!(events, [("scope-imported", &serde_json::json!({ "documents": 3, "removed": 2 }))]);
    assert_eq!(commit.command(), "scope-import");

    assert_eq!(contents(&store.inner, &scope()), owned(REPLACEMENT));
    assert_eq!(read_one(&store.inner, "LANGUAGE.md"), Some(language), "the same content is not written again");

    assert_example_lists(&report);
    assert_eq!(report.scope_version, version_now(&store.inner));
}

#[test]
fn a_second_replace_with_the_same_bundle_commits_nothing() {
    let store = store_before();
    let first = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");
    store.take();

    let second = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the second replace");

    assert!(store.take().commits.is_empty());
    assert_eq!(second.documents, 0);
    assert!(second.added.is_empty() && second.changed.is_empty() && second.removed.is_empty(), "{second:?}");
    assert_eq!(second.scope_version, first.scope_version);
    assert_eq!(version_now(&store.inner), first.scope_version);
}

#[test]
fn a_dry_run_of_a_replace_reports_the_same_lists_and_commits_nothing() {
    let store = store_before();
    let before = version_now(&store.inner);

    let dry = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE_DRY_RUN).expect("the dry run");

    assert!(store.take().commits.is_empty(), "a dry run never commits");
    assert_eq!(dry.scope_version, before);
    assert_eq!(contents(&store.inner, &scope()), owned(SCOPE_BEFORE));
    assert_example_lists(&dry);

    let real = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");
    assert_eq!((&dry.added, &dry.changed, &dry.removed), (&real.added, &real.changed, &real.removed));
    assert_eq!(dry.documents, real.documents);
}

#[test]
fn a_replace_with_no_documents_is_refused_before_the_store_is_called() {
    // One call must not be able to empty a scope.
    for options in [REPLACE, REPLACE_DRY_RUN] {
        let store = store_before();

        let result = import_with(&store, &scope(), bundle(Vec::new()), options);

        assert!(matches!(result, Err(TransferError::EmptyBundle)), "{result:?}");
        assert!(store.take().is_empty(), "the store was called");
        assert_eq!(contents(&store.inner, &scope()), owned(SCOPE_BEFORE));
    }
}

#[test]
fn a_writer_of_a_key_the_replace_updates_refuses_it_whole() {
    let other = "# auth Specification\n\nThe other writer's content.\n";
    let store = store_before();
    let store = store.before_first_commit(move |inner| write_in(inner, &scope(), &[("specs/auth/spec.md", other)]));

    let result = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE);

    assert!(matches!(result, Err(TransferError::ScopeChanged)), "{result:?}");
    let mut expected = owned(SCOPE_BEFORE);
    expected.last_mut().unwrap().1 = other.to_string();
    assert_eq!(contents(&store.inner, &scope()), expected, "nothing of the bundle was written, nothing was deleted");
}

#[test]
fn a_writer_that_adds_a_key_only_the_bundle_held_refuses_the_replace() {
    // The write of a new key expects no document: the other writer's is there.
    let store = store_before()
        .before_first_commit(|inner| write_in(inner, &scope(), &[("specs/billing/spec.md", "# billing, by another writer\n")]));

    let result = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE);

    assert!(matches!(result, Err(TransferError::ScopeChanged)), "{result:?}");
    let mut expected = owned(SCOPE_BEFORE);
    expected.push(("specs/billing/spec.md".to_string(), "# billing, by another writer\n".to_string()));
    assert_eq!(contents(&store.inner, &scope()), expected, "nothing of the bundle was written, nothing was deleted");
}

#[test]
fn a_writer_that_deletes_a_key_both_sides_hold_refuses_the_replace() {
    // The read of a listed key finds no document: the scope changed, and
    // the replace does not start again.
    let store = store_before().before_first_read(|inner| delete_in(inner, "specs/auth/spec.md"));

    let result = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE);

    assert!(matches!(result, Err(TransferError::ScopeChanged)), "{result:?}");
    let calls = store.take();
    assert!(calls.commits.is_empty());
    assert_eq!(calls.lists.len(), 1, "one list and no second try: {:?}", calls.lists);
    let expected: Vec<(String, String)> =
        owned(SCOPE_BEFORE).into_iter().filter(|(key, _)| key != "specs/auth/spec.md").collect();
    assert_eq!(contents(&store.inner, &scope()), expected);
}

#[test]
fn a_writer_of_a_key_neither_side_held_is_not_seen_and_its_document_stays() {
    let store = store_before()
        .before_first_commit(|inner| write_in(inner, &scope(), &[("discussions/d2.md", "---\ntopic: D2\n---\n")]));

    let report = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");

    assert_example_lists(&report);
    let mut expected = owned(REPLACEMENT);
    expected.push(("discussions/d2.md".to_string(), "---\ntopic: D2\n---\n".to_string()));
    expected.sort();
    assert_eq!(contents(&store.inner, &scope()), expected);
}

#[test]
fn a_writer_of_a_key_with_the_same_content_is_not_seen_and_its_content_stays() {
    // `LANGUAGE.md` is the same on both sides, so the replace has no
    // operation for it and states nothing about it.
    let store = store_before()
        .before_first_commit(|inner| write_in(inner, &scope(), &[("LANGUAGE.md", "TeamStore: the other writer's\n")]));

    let report = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");

    assert_example_lists(&report);
    let mut expected = owned(REPLACEMENT);
    expected[0].1 = "TeamStore: the other writer's\n".to_string();
    assert_eq!(contents(&store.inner, &scope()), expected);
}

#[test]
fn a_replace_removes_a_key_outside_the_naming_rules_and_lists_it() {
    // The rule is "the scope equals the bundle": a document the host put
    // into the same scope goes too, and a dry run shows it first.
    let store = store_before();
    write_in(&store.inner, &scope(), &[("notes/a.md", "the host's own note\n")]);

    let dry = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE_DRY_RUN).expect("the dry run");
    let report = import_with(&store, &scope(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");

    let removed = ["changes/add-x/proposal.md", "discussions/d1.md", "notes/a.md"];
    assert_eq!(dry.removed, removed);
    assert_eq!(report.removed, removed);
    assert_eq!(contents(&store.inner, &scope()), owned(REPLACEMENT));
}

#[test]
fn a_replace_reads_the_keys_both_sides_hold_in_batches_of_200_at_most() {
    let store = Recorder::new(MemoryStorePort::new());
    let docs: Vec<(String, String)> = (0..450).map(|i| (format!("specs/cap-{i:03}/spec.md"), format!("# cap {i}\n"))).collect();
    let borrowed: Vec<(&str, &str)> = docs.iter().map(|(key, content)| (key.as_str(), content.as_str())).collect();
    write_in(&store.inner, &scope(), &borrowed);
    write_in(&store.inner, &scope(), &[("discussions/d1.md", "only in the scope: never read\n")]);
    store.take();

    let report = import_with(&store, &scope(), bundle_of(&borrowed), REPLACE).expect("the replace");

    assert_eq!((report.documents, report.removed.as_slice()), (0, &["discussions/d1.md".to_string()][..]));
    let calls = store.take();
    let sizes: Vec<usize> = calls.reads.iter().map(Vec::len).collect();
    assert_eq!(sizes, [200, 200, 50]);
    assert_eq!(calls.commits.len(), 1);
}

// --- the first version of the bundle ---

fn first_version(documents: Vec<(ImportDocumentId, &str)>) -> ImportBundle {
    ImportBundle {
        format_version: 1,
        scope: ImportScope { project: "default".to_string(), repo: "main".to_string() },
        project_revision: 27,
        documents: documents
            .into_iter()
            .map(|(document, content)| ImportBundleDocument {
                document,
                content: content.to_string(),
                digest: content_digest(content),
            })
            .collect(),
    }
}

/// [`EVERY_KIND`] as the first version names its documents.
fn every_kind_by_document() -> Vec<(ImportDocumentId, &'static str)> {
    let change = || "add-x".to_string();
    let archived = || "2026-11-09-add-w".to_string();
    let ids = vec![
        ImportDocumentId::ChangeMeta { change: change() },
        ImportDocumentId::ChangeArtifact { change: change(), artifact: "proposal.md".to_string() },
        ImportDocumentId::ChangeArtifact { change: change(), artifact: "specs/auth/spec.md".to_string() },
        ImportDocumentId::ChangeEvidence { change: change() },
        ImportDocumentId::CanonicalSpec { capability: "auth".to_string() },
        ImportDocumentId::Discussion { slug: "talk".to_string(), archived: false },
        ImportDocumentId::Discussion { slug: "2026-11-09-old-talk".to_string(), archived: true },
        ImportDocumentId::ArchivedChange { change: archived(), doc: "proposal.md".to_string() },
        ImportDocumentId::ArchivedChange { change: archived(), doc: ".evidence.json".to_string() },
        ImportDocumentId::WorkflowConfig,
        ImportDocumentId::Language,
        ImportDocumentId::BoardOrder,
    ];
    ids.into_iter().zip(EVERY_KIND).map(|(id, (_, content))| (id, *content)).collect()
}

#[test]
fn every_document_kind_of_the_first_version_has_the_key_of_the_naming_rules() {
    for ((id, _), (key, _)) in every_kind_by_document().iter().zip(EVERY_KIND) {
        assert_eq!(transfer::key_of_document(id).as_ref().map(DocKey::as_str), Some(*key), "{id:?}");
    }
}

#[test]
fn a_first_version_bundle_imports_like_the_same_documents_by_key() {
    let port = MemoryStorePort::new();

    let keyed = transfer::from_first_version(first_version(every_kind_by_document())).expect("every document has a key");
    assert_eq!(keyed.format_version, 2);
    import(&port, &scope(), keyed).expect("the import");
    import(&port, &target(), bundle_of(EVERY_KIND)).expect("the import by key");

    assert_eq!(contents(&port, &scope()), contents(&port, &target()));
    assert_eq!(contents(&port, &scope()).len(), EVERY_KIND.len());
}

#[test]
fn a_first_version_document_that_has_no_key_is_refused_by_name() {
    let rows = [
        // `a/b` as a change name would read back as change `a`.
        (ImportDocumentId::ChangeMeta { change: "a/b".to_string() }, "a/b"),
        // The refusal shows the empty name as it is.
        (ImportDocumentId::ChangeMeta { change: String::new() }, "change: \"\""),
        (ImportDocumentId::ChangeMeta { change: "archive".to_string() }, "archive"),
        (ImportDocumentId::CanonicalSpec { capability: "..".to_string() }, ".."),
        (ImportDocumentId::CanonicalSpec { capability: "a/b".to_string() }, "a/b"),
        // The key naming rules keep `archive` for the archive folder.
        (ImportDocumentId::ChangeArtifact { change: "archive".to_string(), artifact: "x/proposal.md".to_string() }, "archive"),
    ];
    for (id, named) in rows {
        assert_eq!(transfer::key_of_document(&id), None, "{id:?}");
        let documents = vec![(ImportDocumentId::WorkflowConfig, "schema: spec-driven\n"), (id.clone(), "x")];
        match transfer::from_first_version(first_version(documents)) {
            Err(TransferError::InvalidBundle(message)) => assert!(message.contains(named), "{id:?}: {message}"),
            other => panic!("{id:?}: expected an invalid bundle, got {other:?}"),
        }
    }
}

#[test]
fn the_first_version_constant_is_1_and_a_host_can_name_it() {
    assert_eq!(transfer::FIRST_VERSION, 1);
    assert_eq!(first_version(Vec::new()).format_version, transfer::FIRST_VERSION);
}

#[test]
fn each_of_the_nine_document_kinds_goes_to_its_key_and_back() {
    // 規格範例「對應」的八列，加上進行中與已封存的討論。
    let change = || "add-x".to_string();
    let archived = |doc: &str| ImportDocumentId::ArchivedChange { change: "2026-11-10-add-x".to_string(), doc: doc.to_string() };
    let discussion = |slug: &str, archived| ImportDocumentId::Discussion { slug: slug.to_string(), archived };
    let rows = [
        (ImportDocumentId::ChangeMeta { change: change() }, "changes/add-x/.meta.yaml"),
        (
            ImportDocumentId::ChangeArtifact { change: change(), artifact: "specs/auth/spec.md".to_string() },
            "changes/add-x/specs/auth/spec.md",
        ),
        (ImportDocumentId::ChangeEvidence { change: change() }, "changes/add-x/.evidence.json"),
        (archived("proposal.md"), "archive/changes/2026-11-10-add-x/proposal.md"),
        (ImportDocumentId::CanonicalSpec { capability: "auth".to_string() }, "specs/auth/spec.md"),
        (ImportDocumentId::WorkflowConfig, "config.yaml"),
        (ImportDocumentId::Language, "LANGUAGE.md"),
        (ImportDocumentId::BoardOrder, "board/order"),
        (discussion("board-search", false), "discussions/board-search.md"),
        // The name of an archived discussion is kept as it is: a new record
        // carries its creation date, an older one does not.
        (discussion("2026-11-10-board-search", true), "archive/discussions/2026-11-10-board-search.md"),
        (discussion("reuse", true), "archive/discussions/reuse.md"),
    ];
    for (document, key) in rows {
        let made = transfer::key_of_document(&document).unwrap_or_else(|| panic!("{document:?} has a key"));
        assert_eq!(made.as_str(), key, "{document:?}");
        assert_eq!(transfer::document_of_key(&made), Some(document), "{key}");
    }
}

#[test]
fn a_key_that_follows_no_naming_rule_names_no_document_kind() {
    for key in ["notes/today.md", "specs/auth/notes.md", "changes/add-x"] {
        assert_eq!(transfer::document_of_key(&DocKey::new(key).unwrap()), None, "{key}");
    }
}

#[test]
fn a_first_version_bundle_with_a_kind_outside_the_nine_is_not_a_bundle() {
    let digest = content_digest("x");
    let json = format!(
        r#"{{"formatVersion":1,"scope":{{"project":"default","repo":"main"}},"projectRevision":1,"documents":[{{"document":{{"type":"mystery"}},"content":"x","digest":"{digest}"}}]}}"#
    );
    match transfer::read_bundle(json.as_bytes()) {
        Err(unread @ transfer::UnreadBundle::WrongShape(_)) => {
            let message = unread.to_string();
            assert!(message.starts_with("the bundle is not valid"), "{message}");
            assert!(message.contains("mystery"), "{message}");
        }
        other => panic!("expected a bundle of the wrong shape, got {other:?}"),
    }
}

// --- this project's own spec folder ---

#[test]
fn this_projects_spec_folder_goes_through_an_export_and_an_import_unchanged() {
    let port = MemoryStorePort::new();
    let docs = spec_folder_docs();
    let documents = docs.len();
    seed_in(&port, &scope(), docs);

    let exported = transfer::export(&port, &scope()).expect("the export");
    assert_eq!(exported.documents.len(), documents);
    let report = import(&port, &target(), exported).expect("the import");

    assert_eq!(report.documents, documents);
    assert_eq!(contents(&port, &target()), contents(&port, &scope()));
}

// ======================================================================
// The host kit's two calls (design D4, D5): the hooks and the size limit.
// ======================================================================

use speclink_core::command::ErrorCode;
use speclink_host::kit::{
    AfterCommit, BeforeCommand, Caller, Decision, Hooks, HostKit, HostKitOptions, Identity, Reader,
};
use speclink_host::port_exec::ExecError;
use std::sync::{Arc, Mutex};

fn alice() -> Caller {
    Caller { identity: Identity::new("u_42", "Alice").unwrap(), scope: scope() }
}

/// What the two hooks saw: (command, params) before, and (command, scope
/// version, event names) after.
#[derive(Default)]
struct Seen {
    asked: Mutex<Vec<(String, serde_json::Value)>>,
    watched: Mutex<Vec<(String, u64, Vec<String>)>>,
}

fn recording(seen: &Arc<Seen>, decide: impl Fn(&BeforeCommand) -> Decision + Send + Sync + 'static) -> Hooks {
    let (before, after) = (seen.clone(), seen.clone());
    Hooks {
        before_command: Some(Box::new(move |info: &BeforeCommand, _: &Reader<'_>| {
            before.asked.lock().unwrap().push((info.command.to_string(), info.params.clone()));
            decide(info)
        })),
        after_commit: Some(Box::new(move |info: &AfterCommit| {
            let events = info.events.iter().map(|e| e.name().to_string()).collect();
            after.watched.lock().unwrap().push((info.command.to_string(), info.scope_version.get(), events));
        })),
    }
}

fn allow(_: &BeforeCommand) -> Decision {
    Decision::Allow
}

/// Three documents, 1200 bytes of content together.
fn three_of_400_bytes() -> ScopeBundle {
    let text = |fill: &str| fill.repeat(400);
    bundle(vec![
        document("config.yaml", &text("a")),
        document("LANGUAGE.md", &text("b")),
        document("specs/auth/spec.md", &text("c")),
    ])
}

/// The code and the message of a command-layer failure.
fn command_error(error: ExecError) -> (ErrorCode, String) {
    match error {
        ExecError::Command(e) => (e.code, e.message),
        ExecError::Store(e) => panic!("expected a command error, got the store's {e:?}"),
    }
}

#[test]
fn the_before_hook_is_asked_about_an_import_with_its_count_and_bytes_and_no_content() {
    let seen = Arc::new(Seen::default());
    let store = Arc::new(Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::new(store.clone(), recording(&seen, allow));

    let report = kit.import_scope(&alice(), three_of_400_bytes(), ImportOptions::default()).expect("the import");

    assert_eq!(report.documents, 3);
    let asked = seen.asked.lock().unwrap().clone();
    let params = serde_json::json!({ "documents": 3, "bytes": 1200, "mode": "create", "dryRun": false });
    assert_eq!(asked, [("scope-import".to_string(), params)]);
    let watched = seen.watched.lock().unwrap().clone();
    assert_eq!(watched, [("scope-import".to_string(), report.scope_version.get(), vec!["scope-imported".to_string()])]);
    let commits = store.take().commits;
    assert_eq!(commits.len(), 1);
    assert_eq!((commits[0].actor().id(), commits[0].actor().display_name()), ("u_42", "Alice"));
}

#[test]
fn an_import_the_before_hook_refuses_ends_as_refused_and_the_store_is_not_called() {
    let seen = Arc::new(Seen::default());
    let store = Arc::new(Recorder::new(MemoryStorePort::new()));
    let hooks = recording(&seen, |_| Decision::Deny { reason: "搬移凍結中".to_string() });
    let kit = HostKit::new(store.clone(), hooks);

    let error = kit.import_scope(&alice(), three_of_400_bytes(), ImportOptions::default()).expect_err("refused");

    assert_eq!(command_error(error), (ErrorCode::Refused, "搬移凍結中".to_string()));
    assert!(store.take().is_empty(), "the store was called");
    assert!(seen.watched.lock().unwrap().is_empty());
}

#[test]
fn the_before_hook_is_asked_about_an_export_and_a_refusal_gives_no_bundle() {
    let seen = Arc::new(Seen::default());
    let store = Arc::new(Recorder::new(MemoryStorePort::new()));
    write_in(&store.inner, &scope(), EVERY_KIND);
    store.take();
    let kit = HostKit::new(store.clone(), recording(&seen, allow));

    let exported = kit.export_scope(&alice()).expect("the export");

    assert_eq!(exported.documents.len(), EVERY_KIND.len());
    assert_eq!(seen.asked.lock().unwrap().clone(), [("scope-export".to_string(), serde_json::json!({}))]);
    assert!(seen.watched.lock().unwrap().is_empty(), "an export commits nothing");

    let refusing = HostKit::new(store.clone(), recording(&seen, |_| Decision::Deny { reason: "no export".to_string() }));
    store.take();
    let error = refusing.export_scope(&alice()).expect_err("refused");
    assert_eq!(command_error(error), (ErrorCode::Refused, "no export".to_string()));
    assert!(store.take().is_empty(), "the store was called");
}

#[test]
fn a_direct_import_over_the_limit_is_not_valid_and_the_store_is_not_called() {
    let store = Arc::new(Recorder::new(MemoryStorePort::new()));
    let options = HostKitOptions { import_limit_bytes: Some(1199), ..Default::default() };
    let kit = HostKit::with_options(store.clone(), Hooks::default(), options).expect("the kit");

    let (code, message) = command_error(kit.import_scope(&alice(), three_of_400_bytes(), ImportOptions::default()).expect_err("over the limit"));

    assert_eq!(code, ErrorCode::InvalidArgv);
    assert!(message.contains("1199"), "{message}");
    assert!(store.take().is_empty(), "the store was called");

    // At the limit, it imports.
    let options = HostKitOptions { import_limit_bytes: Some(1200), ..Default::default() };
    let kit = HostKit::with_options(store.clone(), Hooks::default(), options).expect("the kit");
    assert_eq!(kit.import_scope(&alice(), three_of_400_bytes(), ImportOptions::default()).expect("at the limit").documents, 3);
}

#[test]
fn the_failures_of_a_direct_call_keep_the_two_vocabularies() {
    let store = Arc::new(Recorder::new(MemoryStorePort::new()));
    let kit = HostKit::new(store.clone(), Hooks::default());

    let (code, message) = command_error(kit.import_scope(&alice(), bundle_of(&[("secrets/token", "x")]), ImportOptions::default()).unwrap_err());
    assert_eq!(code, ErrorCode::InvalidArgv);
    assert!(message.contains("secrets/token"), "{message}");

    write_in(&store.inner, &scope(), &[("config.yaml", "schema: spec-driven\n")]);
    let (code, message) = command_error(kit.import_scope(&alice(), three_of_400_bytes(), ImportOptions::default()).unwrap_err());
    assert_eq!(code, ErrorCode::Refused);
    assert!(message.contains("create-new"), "{message}");

    let never_still = Arc::new(Recorder::never_still());
    write_in(&never_still.inner, &scope(), EVERY_KIND);
    let kit = HostKit::new(never_still, Hooks::default());
    match kit.export_scope(&alice()) {
        Err(ExecError::Store(PortError::Unavailable { message })) => assert!(message.contains("changed"), "{message}"),
        other => panic!("expected a temporary failure, got {other:?}"),
    }
}

// --- the replace mode and the dry run through the kit (import-replace-mode D5) ---

/// A kit over a store that holds [`SCOPE_BEFORE`], with no call recorded yet.
fn kit_before(hooks: Hooks) -> (HostKit, Arc<Recorder>) {
    let store = Arc::new(store_before());
    (HostKit::new(store.clone(), hooks), store)
}

#[test]
fn the_before_hook_is_asked_once_about_a_dry_run_of_a_replace_with_its_mode_and_no_content() {
    // spec host-kit「掛鉤看得到取代與試跑」.
    let seen = Arc::new(Seen::default());
    let (kit, store) = kit_before(recording(&seen, allow));

    let report = kit.import_scope(&alice(), three_of_400_bytes(), REPLACE_DRY_RUN).expect("the dry run");

    let params = serde_json::json!({ "documents": 3, "bytes": 1200, "mode": "replace", "dryRun": true });
    assert_eq!(seen.asked.lock().unwrap().clone(), [("scope-import".to_string(), params)]);
    assert!(seen.watched.lock().unwrap().is_empty(), "a dry run commits nothing, so the after hook is not told");
    assert!(store.take().commits.is_empty());
    assert_eq!(report.removed, ["changes/add-x/proposal.md", "discussions/d1.md"]);
}

#[test]
fn a_replace_the_before_hook_refuses_ends_as_refused_before_the_store_is_listed() {
    let seen = Arc::new(Seen::default());
    let refuse_replace = |info: &BeforeCommand| match info.params["mode"] == "replace" {
        true => Decision::Deny { reason: "只有管理者可以取代".to_string() },
        false => Decision::Allow,
    };
    let (kit, store) = kit_before(recording(&seen, refuse_replace));

    for options in [REPLACE, REPLACE_DRY_RUN] {
        let error = kit.import_scope(&alice(), bundle_of(REPLACEMENT), options).expect_err("refused");
        assert_eq!(command_error(error), (ErrorCode::Refused, "只有管理者可以取代".to_string()));
    }

    assert!(store.take().is_empty(), "the store was called");
    assert!(seen.watched.lock().unwrap().is_empty());
    assert_eq!(contents(&store.inner, &scope()), owned(SCOPE_BEFORE));
}

#[test]
fn a_replace_of_a_scope_that_changed_during_it_is_refused_with_its_reason_and_its_message() {
    // import-replace-mode D3: `refused` and `scope_changed`, through the kit.
    let other = "# auth Specification\n\nThe other writer's content.\n";
    let store = store_before().before_first_commit(move |inner| write_in(inner, &scope(), &[("specs/auth/spec.md", other)]));
    let kit = HostKit::new(Arc::new(store), Hooks::default());

    let error = match kit.import_scope(&alice(), bundle_of(REPLACEMENT), REPLACE) {
        Err(ExecError::Command(error)) => error,
        other => panic!("expected a command failure, got {other:?}"),
    };

    assert_eq!(error.code, ErrorCode::Refused);
    assert_eq!(error.message, "import (replace): the target scope changed during the import — run it again");
    assert_eq!(transfer::reason_of(&error), Some(transfer::TransferReason::ScopeChanged));
}

#[test]
fn the_after_hook_is_told_about_a_replace_only_when_it_committed() {
    let seen = Arc::new(Seen::default());
    let (kit, store) = kit_before(recording(&seen, allow));

    let report = kit.import_scope(&alice(), bundle_of(REPLACEMENT), REPLACE).expect("the replace");
    let watched = seen.watched.lock().unwrap().clone();
    assert_eq!(watched, [("scope-import".to_string(), report.scope_version.get(), vec!["scope-imported".to_string()])]);
    assert_eq!(store.take().commits.len(), 1);

    // The same replace again has nothing to write or delete: no commit, and
    // the after hook is not told. The before hook was asked each time.
    let again = kit.import_scope(&alice(), bundle_of(REPLACEMENT), REPLACE).expect("the second replace");
    assert_eq!((again.documents, again.scope_version), (0, report.scope_version));
    assert_eq!(seen.watched.lock().unwrap().len(), 1);
    assert_eq!(seen.asked.lock().unwrap().len(), 2);
    assert!(store.take().commits.is_empty());
}

#[test]
fn a_replace_over_the_limit_is_not_valid_and_the_store_is_not_called() {
    // spec host-kit「取代也受大小上限限制」: 2,000 bytes against a limit of 1,024.
    let store = Arc::new(store_before());
    let options = HostKitOptions { import_limit_bytes: Some(1_024), ..Default::default() };
    let kit = HostKit::with_options(store.clone(), Hooks::default(), options).expect("the kit");
    let large = bundle_of(&[("specs/auth/spec.md", &"x".repeat(2_000))]);

    for options in [REPLACE, REPLACE_DRY_RUN] {
        let error = kit.import_scope(&alice(), large.clone(), options).expect_err("over the limit");
        let (code, message) = command_error(error);
        assert_eq!(code, ErrorCode::InvalidArgv);
        assert!(message.contains("1024"), "{message}");
    }

    assert!(store.take().is_empty(), "the store was called");
    assert_eq!(contents(&store.inner, &scope()), owned(SCOPE_BEFORE));
}

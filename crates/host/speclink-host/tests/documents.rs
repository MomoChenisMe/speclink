//! The document reads and writes that are not Engine commands (design D1–D4),
//! on a memory store and on a store that counts its calls.
//!
//! Each test follows a scenario of 「不經引擎命令的文件讀寫只用儲存接口」.

use speclink_core::workspace::DEFAULT_SPEC_DIR;
use speclink_host::documents::{self, ContextOutcome, DocumentsError};
use speclink_protocol::context::ContextSnapshotRequest;
use speclink_store::content_digest;
use speclink_store_port::{
    Actor, DocKey, Expected, MemoryStorePort, Op, PortError, StorePort, Timestamp, Version,
};
use std::sync::Mutex;

mod common;
use common::{scope, scope_version, write, Recorder};

// --- helpers ---

fn key(text: &str) -> DocKey {
    DocKey::new(text).unwrap()
}

fn at() -> Timestamp {
    Timestamp::new(1_800_000_000_000).unwrap()
}

fn doc_version(port: &dyn StorePort, k: &str) -> u64 {
    port.read(&scope(), &[key(k)]).unwrap()[0]
        .as_ref()
        .unwrap()
        .version
        .get()
}

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";

// --- 範圍版本號 ---

#[test]
fn the_scope_version_is_one_narrow_list() {
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/add-x/.meta.yaml", META)]);
    let version = documents::scope_version(&store, &scope()).unwrap();
    assert_eq!(version.get(), scope_version(&store.inner));
    assert_eq!(store.calls().lists, ["config.yaml"]);
    assert!(store.calls().read_keys().is_empty());
    assert_eq!(documents::etag(version), format!("\"{}\"", version.get()));
}

// --- 流程設定 ---

#[test]
fn reading_the_workflow_config_reads_one_document() {
    // 「流程設定的讀取只讀一份文件」：165 個已封存變更也不碰。
    let store = Recorder::new(MemoryStorePort::new());
    let archived: Vec<(String, String)> = (0..165)
        .map(|n| {
            (
                format!("archive/changes/2026-01-01-c{n}/.meta.yaml"),
                META.to_string(),
            )
        })
        .collect();
    let archived: Vec<(&str, &str)> = archived
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    write(&store.inner, &archived);
    write(
        &store.inner,
        &[("config.yaml", "schema: spec-driven\nlocale: ja\n")],
    );

    let (config, version) = documents::config_document(&store, &scope()).unwrap();
    assert_eq!(
        config.content.as_deref(),
        Some("schema: spec-driven\nlocale: ja\n")
    );
    assert_eq!(config.schema, "spec-driven");
    assert_eq!(
        config.revision,
        scope_version(&store.inner),
        "the revision is the scope version"
    );
    assert_eq!(version.get(), config.revision, "the ETag's version");
    assert_eq!(store.calls().read_keys(), [["config.yaml"]]);
    assert!(
        store.calls().lists.iter().all(|p| !p.starts_with("archive/")),
        "{:?}",
        store.calls().lists
    );
}

#[test]
fn a_missing_workflow_config_reads_as_absent() {
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/add-x/.meta.yaml", META)]);
    let (config, _) = documents::config_document(&store, &scope()).unwrap();
    assert_eq!(config.content, None);
    assert_eq!(config.schema, "spec-driven", "the default schema");
    assert_eq!(config.revision, 1);
}

#[test]
fn a_broken_workflow_config_is_an_invalid_config() {
    // 「壞的流程設定」
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("config.yaml", "schema: [unclosed\n")]);
    match documents::config_document(&store, &scope()) {
        Err(DocumentsError::InvalidConfig(message)) => assert!(!message.is_empty()),
        other => panic!("expected an invalid config, got {other:?}"),
    }
}

#[test]
fn the_version_is_taken_before_the_content() {
    // 「回報的版本不比內容新」：取得版本之後、讀取內容之前有人提交了新的流程設定。
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("config.yaml", "schema: spec-driven\n")]);
    let old = scope_version(&store.inner);
    let store = store.before_first_read(|inner| {
        write(
            inner,
            &[("config.yaml", "schema: spec-driven\nlocale: ja\n")],
        );
    });

    let (config, version) = documents::config_document(&store, &scope()).unwrap();
    assert_eq!(
        config.content.as_deref(),
        Some("schema: spec-driven\nlocale: ja\n"),
        "the new content"
    );
    assert_eq!((config.revision, version.get()), (old, old), "the old version");
    assert!(
        scope_version(&store.inner) > old,
        "polling with that version shows the scope moved"
    );
}

#[test]
fn an_unavailable_store_stays_unavailable() {
    // 「儲存暫時無法使用」
    let store = Recorder::unavailable();
    match documents::config_document(&store, &scope()) {
        Err(DocumentsError::Store(PortError::Unavailable { .. })) => {}
        other => panic!("expected the store's temporary failure, got {other:?}"),
    }
}

// --- 看板順序 ---

#[test]
fn reading_the_board_order() {
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/add-x/.meta.yaml", META)]);
    let (absent, _) = documents::board_order(&store, &scope()).unwrap();
    assert_eq!((absent.content, absent.revision), (None, 1));

    write(&store.inner, &[("board/order", "{\"changes\":{}}")]);
    let (present, version) = documents::board_order(&store, &scope()).unwrap();
    assert_eq!(present.content.as_deref(), Some("{\"changes\":{}}"));
    assert_eq!((present.revision, version.get()), (2, 2));
}

fn v(n: u64) -> Version {
    Version::new(n).unwrap()
}

fn alice() -> Actor {
    Actor::new("alice", "").unwrap()
}

/// A scope at version `n`, with no board order.
fn at_version(n: u64) -> Recorder {
    let store = Recorder::new(MemoryStorePort::new());
    for i in 0..n {
        write(&store.inner, &[(&format!("changes/c{i}/.meta.yaml"), META)]);
    }
    assert_eq!(scope_version(&store.inner), n);
    store
}

#[test]
fn writing_the_board_order_is_one_commit_with_its_event() {
    // 「看板順序寫入成功」
    let store = at_version(7);
    let content = "x".repeat(200);
    let version =
        documents::put_board_order(&store, &scope(), &alice(), content.clone(), v(7), at()).unwrap();
    assert!(version.get() > 7);

    let commits = store.calls().commits;
    assert_eq!(commits.len(), 1, "exactly one commit");
    let commit = &commits[0];
    match commit.ops() {
        [Op::Put {
            key: k,
            content: c,
            expected: Expected::Absent,
        }] => {
            assert_eq!((k.as_str(), c.as_str()), ("board/order", content.as_str()));
        }
        other => panic!("expected one create of board/order, got {other:?}"),
    }
    let events: Vec<(&str, &serde_json::Value)> = commit
        .events()
        .iter()
        .map(|e| (e.name(), e.payload()))
        .collect();
    assert_eq!(events, [("board-order-updated", &serde_json::json!({}))]);
    assert_eq!(commit.command(), "put-board-order");
    assert_eq!(commit.actor().id(), "alice");
    assert_eq!(commit.at(), at());
}

#[test]
fn writing_over_an_existing_board_order_expects_its_version() {
    let store = at_version(3);
    write(&store.inner, &[("board/order", "{}")]);
    let doc = doc_version(&store.inner, "board/order");
    documents::put_board_order(&store, &scope(), &alice(), "{\"a\":1}".to_string(), v(4), at())
        .unwrap();
    match store.calls().commits[0].ops() {
        [Op::Put { expected, .. }] => {
            assert_eq!(*expected, Expected::At(Version::new(doc).unwrap()))
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_stale_board_order_write_commits_nothing() {
    // 「看板順序的預期版本過期」
    let store = at_version(8);
    match documents::put_board_order(&store, &scope(), &alice(), "{}".to_string(), v(7), at()) {
        Err(error @ DocumentsError::ScopeMoved { .. }) => {
            assert_eq!(error.to_string(), "expected scope revision 7, actual 8");
        }
        other => panic!("expected the scope to have moved, got {other:?}"),
    }
    assert!(store.calls().commits.is_empty());
}

#[test]
fn an_oversized_board_order_never_reaches_the_store() {
    // 「看板順序內容過大」
    let store = at_version(1);
    let content = "x".repeat(documents::BOARD_ORDER_LIMIT + 1);
    match documents::put_board_order(&store, &scope(), &alice(), content, v(1), at()) {
        Err(error @ DocumentsError::TooLarge { .. }) => {
            assert_eq!(
                error.to_string(),
                "board order content exceeds the 1048576-byte cap"
            );
        }
        other => panic!("expected too large, got {other:?}"),
    }
    assert!(store.calls().is_empty(), "no call at all");
}

#[test]
fn a_board_order_written_by_another_writer_meanwhile_is_a_conflict() {
    // 第三種拒絕：比對範圍版本之後、提交之前別人寫了看板順序。提交回報衝突，別人的內容留著。
    let store = at_version(2).before_first_commit(|inner| {
        write(inner, &[("board/order", "{\"theirs\":1}")]);
    });
    match documents::put_board_order(
        &store,
        &scope(),
        &alice(),
        "{\"mine\":1}".to_string(),
        v(2),
        at(),
    ) {
        Err(DocumentsError::Store(PortError::Conflict {
            key: k,
            expected: Expected::Absent,
            ..
        })) => {
            assert_eq!(k.as_str(), "board/order");
        }
        other => panic!("expected a conflict, got {other:?}"),
    }
    let doc = store.inner.read(&scope(), &[key("board/order")]).unwrap()[0]
        .clone()
        .unwrap();
    assert_eq!(doc.content, "{\"theirs\":1}");
    assert_eq!(
        scope_version(&store.inner),
        3,
        "only the other writer's commit landed"
    );
    assert!(
        store.inner.recorded_events(&scope()).is_empty(),
        "no board-order event landed"
    );
}

// --- evidence ---

const EVIDENCE: &str = r#"{"version":2,"change":"add-x","touched":["src/a.rs"],"entries":[{"taskId":"1","taskDesc":"1.1 a","actor":"Alice","touchedFiles":["src/a.rs"],"recordedAt":"2026-11-01T00:00:00Z"}]}"#;

#[test]
fn a_change_evidence_reads_its_entries() {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("changes/add-x/.meta.yaml", META),
            ("changes/add-x/.evidence.json", EVIDENCE),
        ],
    );
    let (evidence, version) = documents::change_evidence(&store, &scope(), "add-x").unwrap();
    assert_eq!(version.get(), scope_version(&store.inner));
    assert_eq!(evidence.entries.len(), 1);
    assert_eq!(evidence.entries[0].task_id, "1");
    assert_eq!(evidence.entries[0].touched_files, ["src/a.rs"]);
    assert_eq!(store.calls().read_keys(), [["changes/add-x/.evidence.json"]]);
}

#[test]
fn a_missing_evidence_is_an_empty_set() {
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/add-x/.meta.yaml", META)]);
    let (evidence, _) = documents::change_evidence(&store, &scope(), "nope").unwrap();
    assert!(evidence.entries.is_empty());
}

#[test]
fn a_broken_evidence_is_corrupt() {
    // 「壞的 evidence」
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[("changes/add-x/.evidence.json", "not json")],
    );
    match documents::change_evidence(&store, &scope(), "add-x") {
        Err(DocumentsError::Corrupt(message)) => {
            assert!(
                message.starts_with("evidence record is unreadable: "),
                "{message}"
            );
        }
        other => panic!("expected corrupt, got {other:?}"),
    }
}

// --- 正式規格與已封存 ---

#[test]
fn a_canonical_spec_reads_one_document() {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("specs/auth/spec.md", "# Auth\n"),
            ("specs/billing/spec.md", "# Billing\n"),
        ],
    );
    let (spec, version) = documents::spec_document(&store, &scope(), "auth").unwrap();
    assert_eq!(
        (spec.content.as_str(), version.get()),
        ("# Auth\n", scope_version(&store.inner))
    );
    assert_eq!(store.calls().read_keys(), [["specs/auth/spec.md"]]);
}

#[test]
fn a_missing_canonical_spec_is_not_found() {
    // 「不存在的正式規格」
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("specs/auth/spec.md", "# Auth\n")]);
    match documents::spec_document(&store, &scope(), "nope") {
        Err(DocumentsError::NotFound(message)) => assert_eq!(message, "Spec 'nope' not found."),
        other => panic!("expected not found, got {other:?}"),
    }
}

fn archive() -> Recorder {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            (
                "archive/changes/2026-11-01-alpha/tasks.md",
                "- [x] 1.1 a\n- [ ] 1.2 b\n",
            ),
            (
                "archive/changes/2026-11-01-alpha/.meta.yaml",
                "schema: spec-driven\ncreated_by: Alice\n",
            ),
            ("archive/changes/2026-11-01-alpha/proposal.md", "# alpha\n"),
            (
                "archive/changes/2026-11-01-alpha/specs/auth/spec.md",
                "## ADDED Requirements\n",
            ),
            (
                "archive/changes/2026-11-01-alpha/specs/billing/spec.md",
                "## ADDED Requirements\n",
            ),
            // Only the data file's name from before the rename.
            (
                "archive/changes/2026-11-03-beta/.openspec.yaml",
                "schema: spec-driven\ncreated_by: Bob\nfrom_discussion: talk\n",
            ),
            ("archive/changes/2026-11-02-gamma/tasks.md", "- [x] 1.1 a\n"),
            ("changes/live/.meta.yaml", META),
        ],
    );
    store
}

#[test]
fn the_archived_list_is_one_list_and_one_read() {
    // 「已封存清單一次讀取」
    let store = archive();
    let (list, version) = documents::archived_list(&store, &scope()).unwrap();
    assert_eq!(version.get(), scope_version(&store.inner));
    assert_eq!(store.calls().lists, ["archive/changes/"]);
    let reads = store.calls().read_keys();
    assert_eq!(reads.len(), 1, "one read");
    let mut read = reads[0].clone();
    read.sort();
    assert_eq!(
        read,
        [
            "archive/changes/2026-11-01-alpha/.meta.yaml",
            "archive/changes/2026-11-01-alpha/tasks.md",
            "archive/changes/2026-11-02-gamma/tasks.md",
            "archive/changes/2026-11-03-beta/.openspec.yaml",
        ]
    );

    let rows: Vec<(
        &str,
        &str,
        &str,
        Option<usize>,
        Option<usize>,
        usize,
        Option<&str>,
        Vec<String>,
    )> = list
        .archived
        .iter()
        .map(|a| {
            (
                a.dated_name.as_str(),
                a.date.as_str(),
                a.name.as_str(),
                a.tasks_total,
                a.tasks_done,
                a.spec_count,
                a.created_by.as_deref(),
                a.from_discussions.clone(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (
                "2026-11-03-beta",
                "2026-11-03",
                "beta",
                None,
                None,
                0,
                Some("Bob"),
                vec!["talk".to_string()]
            ),
            (
                "2026-11-02-gamma",
                "2026-11-02",
                "gamma",
                Some(1),
                Some(1),
                0,
                None,
                vec![]
            ),
            (
                "2026-11-01-alpha",
                "2026-11-01",
                "alpha",
                Some(2),
                Some(1),
                2,
                Some("Alice"),
                vec![]
            ),
        ],
        "newest first"
    );
}

#[test]
fn an_archived_document_and_its_capabilities() {
    let store = archive();
    let (doc, _) =
        documents::archived_artifact(&store, &scope(), "2026-11-01-alpha", "specs/auth/spec.md")
            .unwrap();
    assert_eq!(doc.content, "## ADDED Requirements\n");

    let (capabilities, version) =
        documents::archived_capabilities(&store, &scope(), "2026-11-01-alpha").unwrap();
    assert_eq!(capabilities, ["auth", "billing"]);
    assert_eq!(version.get(), scope_version(&store.inner));
}

#[test]
fn a_missing_archived_document_is_not_found() {
    let store = archive();
    match documents::archived_artifact(&store, &scope(), "2026-11-01-alpha", "design.md") {
        Err(DocumentsError::NotFound(message)) => {
            assert_eq!(
                message,
                "Archived artifact 'design.md' not found in '2026-11-01-alpha'."
            );
        }
        other => panic!("expected not found, got {other:?}"),
    }
}

#[test]
fn a_missing_archived_change_has_no_capabilities() {
    let store = archive();
    let (capabilities, _) =
        documents::archived_capabilities(&store, &scope(), "2026-01-01-nope").unwrap();
    assert!(capabilities.is_empty());
}

// --- 變更清單與單筆變更的補充欄位 ---

#[test]
fn the_delta_capabilities_list_one_prefix_and_read_nothing() {
    // 「單筆變更的 delta capability 只列出該變更的前綴」
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("changes/add-x/.meta.yaml", META),
            ("changes/add-x/specs/billing/spec.md", "d"),
            ("changes/add-x/specs/auth/spec.md", "d"),
            ("changes/add-xy/specs/other/spec.md", "d"),
        ],
    );
    let capabilities = documents::change_delta_capabilities(&store, &scope(), "add-x").unwrap();
    assert_eq!(capabilities, ["auth", "billing"]);
    assert_eq!(store.calls().lists, ["changes/add-x/specs/"]);
    assert!(store.calls().read_keys().is_empty());
}

#[test]
fn the_change_extras_are_one_read_of_the_metadata() {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            (
                "changes/add-x/.meta.yaml",
                "schema: spec-driven\ncreated_by: Alice\n",
            ),
            ("changes/broken/.meta.yaml", "schema: [unclosed\n"),
        ],
    );
    let names = ["add-x", "broken", "missing"].map(String::from);
    let extras = documents::change_extras(&store, &scope(), &names).unwrap();
    assert_eq!(store.calls().read_keys().len(), 1, "one read for every name");
    assert_eq!(
        extras.len(),
        1,
        "a broken or missing metadata file has no extras"
    );
    assert_eq!(extras["add-x"].created_by.as_deref(), Some("Alice"));
}

#[test]
fn a_document_version_is_one_read() {
    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/add-x/tasks.md", "- [ ] 1.1 a\n")]);
    let version = documents::document_version(&store, &scope(), "add-x", "tasks.md").unwrap();
    assert_eq!(
        version.map(Version::get),
        Some(doc_version(&store.inner, "changes/add-x/tasks.md"))
    );
    assert_eq!(
        documents::document_version(&store, &scope(), "add-x", "design.md").unwrap(),
        None
    );
    assert_eq!(store.calls().read_keys().len(), 2);
    assert!(store.calls().lists.is_empty());
}

// --- 搜尋 ---

fn searchable() -> Recorder {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("changes/add-search/.meta.yaml", META),
            ("changes/add-search/proposal.md", "nothing here"),
            ("changes/add-search/design.md", "the MagicToken\nlives here"),
            ("changes/add-search/tasks.md", "MagicToken again"),
            ("changes/b-change/.meta.yaml", META),
            (
                "changes/b-change/specs/auth/spec.md",
                "a magictoken in a delta",
            ),
            // An artifact with no metadata file is not a change the search finds.
            ("changes/no-meta/proposal.md", "MagicToken"),
            ("discussions/talk.md", "we said MAGICTOKEN"),
            ("archive/discussions/2026-01-01-old.md", "MagicToken"),
            ("archive/changes/2026-01-01-old/proposal.md", "MagicToken"),
        ],
    );
    store
}

#[test]
fn the_search_follows_the_fixed_order_and_one_hit_per_card() {
    let store = searchable();
    let (result, version) = documents::search(&store, &scope(), "magictoken").unwrap();
    assert_eq!(version.get(), scope_version(&store.inner));
    let hits: Vec<(&str, &str, &str, &str)> = result
        .hits
        .iter()
        .map(|h| {
            (
                h.kind.as_str(),
                h.id.as_str(),
                h.artifact.as_str(),
                h.snippet.as_str(),
            )
        })
        .collect();
    assert_eq!(
        hits,
        [
            (
                "change",
                "add-search",
                "design.md",
                "the MagicToken lives here"
            ),
            (
                "change",
                "b-change",
                "specs/auth/spec.md",
                "a magictoken in a delta"
            ),
            ("discussion", "talk", "talk.md", "we said MAGICTOKEN"),
        ]
    );
    let mut lists = store.calls().lists;
    lists.sort();
    assert_eq!(lists, ["changes/", "discussions/"]);
    assert_eq!(
        store.calls().read_keys().len(),
        1,
        "one read for every document it scans"
    );
}

#[test]
fn an_empty_query_has_no_hits() {
    let store = searchable();
    let (result, version) = documents::search(&store, &scope(), "   ").unwrap();
    assert!(result.hits.is_empty());
    assert_eq!(version.get(), scope_version(&store.inner));
    assert!(store.calls().read_keys().is_empty());
}

// --- Context 快照 ---

fn contextual() -> Recorder {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("changes/add-x/.meta.yaml", META),
            ("changes/add-x/.evidence.json", EVIDENCE),
            ("changes/add-x/proposal.md", "# add-x\n"),
            ("changes/add-x/tasks.md", "- [ ] 1.1 a\n"),
            (
                "changes/add-x/specs/auth/spec.md",
                "## ADDED Requirements\n",
            ),
            ("changes/other/proposal.md", "# other\n"),
            ("specs/auth/spec.md", "# Auth\n"),
            ("discussions/talk.md", "# talk\n"),
            ("archive/discussions/2026-01-01-old.md", "# old\n"),
            ("archive/changes/2026-01-01-old/proposal.md", "# old\n"),
        ],
    );
    write(&store.inner, &[("config.yaml", "schema: spec-driven\n")]);
    write(&store.inner, &[("LANGUAGE.md", "# Vocabulary\n")]);
    store
}

fn request(change: Option<&str>) -> ContextSnapshotRequest {
    ContextSnapshotRequest {
        change: change.map(str::to_string),
        flow: None,
    }
}

fn snapshot_of(outcome: ContextOutcome) -> speclink_protocol::context::ContextSnapshot {
    match outcome {
        ContextOutcome::Snapshot(snapshot) => snapshot,
        other => panic!("expected a snapshot, got {other:?}"),
    }
}

#[test]
fn the_full_context_selects_the_readable_mirror() {
    let store = contextual();
    let snapshot =
        snapshot_of(documents::context_snapshot(&store, &scope(), &request(None), None).unwrap());
    let paths: Vec<&str> = snapshot.documents.iter().map(|d| d.path.as_str()).collect();
    let d = DEFAULT_SPEC_DIR;
    assert_eq!(
        paths,
        [
            format!("{d}/LANGUAGE.md"),
            format!("{d}/changes/add-x/proposal.md"),
            format!("{d}/changes/add-x/specs/auth/spec.md"),
            format!("{d}/changes/add-x/tasks.md"),
            format!("{d}/changes/other/proposal.md"),
            format!("{d}/config.yaml"),
            format!("{d}/discussions/talk.md"),
            format!("{d}/specs/auth/spec.md"),
        ]
    );
    for doc in &snapshot.documents {
        let k = doc.path.strip_prefix(&format!("{d}/")).unwrap();
        assert_eq!(doc.revision, Some(doc_version(&store.inner, k)), "{k}");
        assert_eq!(doc.digest, content_digest(&doc.content), "{k}");
    }
    assert_eq!(
        snapshot.snapshot_id,
        documents::etag(Version::new(scope_version(&store.inner)).unwrap())
    );
    assert_eq!(
        snapshot.policy_revision,
        Some(doc_version(&store.inner, "config.yaml"))
    );
    let combined: Vec<&str> = snapshot
        .documents
        .iter()
        .map(|d| d.digest.as_str())
        .collect();
    assert_eq!(snapshot.digest, content_digest(&combined.join("\n")));
    assert_eq!(
        store.calls().read_keys().len(),
        1,
        "one read for every selected document"
    );
}

#[test]
fn a_change_context_narrows_to_that_change() {
    let store = contextual();
    let snapshot = snapshot_of(
        documents::context_snapshot(&store, &scope(), &request(Some("add-x")), None).unwrap(),
    );
    let paths: Vec<&str> = snapshot.documents.iter().map(|d| d.path.as_str()).collect();
    let d = DEFAULT_SPEC_DIR;
    assert_eq!(
        paths,
        [
            format!("{d}/LANGUAGE.md"),
            format!("{d}/changes/add-x/proposal.md"),
            format!("{d}/changes/add-x/specs/auth/spec.md"),
            format!("{d}/changes/add-x/tasks.md"),
            format!("{d}/config.yaml"),
            format!("{d}/specs/auth/spec.md"),
        ]
    );
    assert!(
        !store.calls().lists.iter().any(|p| p == "discussions/"),
        "{:?}",
        store.calls().lists
    );
}

#[test]
fn an_unchanged_context_reads_nothing() {
    // 「Context 快照沒有變時不讀內容」
    let store = contextual();
    let etag = documents::etag(Version::new(scope_version(&store.inner)).unwrap());
    match documents::context_snapshot(&store, &scope(), &request(None), Some(&etag)).unwrap() {
        ContextOutcome::NotModified(version) => assert_eq!(documents::etag(version), etag),
        other => panic!("expected not modified, got {other:?}"),
    }
    assert_eq!(store.calls().lists.len(), 1, "one list");
    assert!(store.calls().read_keys().is_empty(), "no read");
}

#[test]
fn a_context_for_a_missing_change_is_not_found() {
    // 「Context 快照指定不存在的變更」
    let store = contextual();
    match documents::context_snapshot(&store, &scope(), &request(Some("nope")), None) {
        Err(DocumentsError::NotFound(message)) => assert_eq!(message, "change 'nope' not found"),
        other => panic!("expected not found, got {other:?}"),
    }
}

// --- 沒有 workspace 時封存不寫入本機檔案 ---

/// Only the tests that switch the working directory take this lock; the
/// other tests in this file still run beside them. That is safe because no
/// other test here reads or writes a relative path. A new test that does
/// must take this lock too, or `archive_writes_no_local_files` can find a
/// file of its in the temp folder.
static WORKING_DIRECTORY: Mutex<()> = Mutex::new(());

/// Puts the working directory back when dropped — before the temp folder is
/// removed, which Windows refuses while it is a working directory.
struct BackTo(std::path::PathBuf);

impl Drop for BackTo {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

const CANON_R1: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
const ADDED_R2: &str = "## ADDED Requirements\n\n### Requirement: R2\n\nIt SHALL also work.\n\n#### Scenario: fine\n\n- **WHEN** used\n- **THEN** fine\n";

#[test]
fn archive_writes_no_local_files() {
    // 「經 Host 封存不在程序目錄寫檔」
    let _serial = WORKING_DIRECTORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let folder = tempfile::tempdir().expect("temp folder");
    let _back = BackTo(std::env::current_dir().expect("working directory"));
    std::env::set_current_dir(folder.path()).expect("enter the temp folder");

    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("changes/demo/.meta.yaml", META),
            ("changes/demo/tasks.md", "- [x] 1.1 done\n"),
            ("changes/demo/specs/auth/spec.md", ADDED_R2),
            ("specs/auth/spec.md", CANON_R1),
        ],
    );
    let done = speclink_host::port_exec::execute(
        &store,
        &scope(),
        &common::host_ctx(),
        speclink_core::command::Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: true,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
    )
    .expect("the archive succeeds");

    match &done.outcome {
        speclink_core::command::CommandOutcome::Archive(outcome) => {
            assert!(!outcome.snapshot_created, "no backup was made");
        }
        other => panic!("expected an archive outcome, got {other:?}"),
    }
    let left: Vec<_> = std::fs::read_dir(folder.path())
        .expect("read the temp folder")
        .map(|entry| entry.expect("entry").file_name())
        .collect();
    assert!(
        left.is_empty(),
        "the working directory stays empty: {left:?}"
    );
    assert_eq!(store.calls().commits.len(), 1, "exactly one commit");
    let canonical = store
        .inner
        .read(&scope(), &[key("specs/auth/spec.md")])
        .unwrap()[0]
        .clone()
        .expect("the canonical spec");
    assert!(
        canonical.content.contains("### Requirement: R2"),
        "the delta was merged"
    );
}

// --- 稽核：來自網址的輸入只組得出自己那一類的鍵 ---

#[test]
fn request_names_cannot_reach_another_kind_of_document() {
    let store = Recorder::new(MemoryStorePort::new());
    write(
        &store.inner,
        &[
            ("config.yaml", "schema: spec-driven\n"),
            ("changes/add-x/.meta.yaml", META),
            ("changes/add-x/.evidence.json", EVIDENCE),
            ("archive/changes/2026-11-01-alpha/proposal.md", "# alpha\n"),
            ("specs/auth/spec.md", "# Auth\n"),
        ],
    );
    let not_found = |result: Result<
        (speclink_protocol::query::SpecDocumentResponse, Version),
        DocumentsError,
    >| {
        assert!(
            matches!(result, Err(DocumentsError::NotFound(_))),
            "{result:?}"
        );
    };
    // A path that climbs out, or a name with a `/`, makes no key at all.
    not_found(documents::archived_artifact(
        &store,
        &scope(),
        "2026-11-01-alpha",
        "../../config.yaml",
    ));
    not_found(documents::archived_artifact(
        &store,
        &scope(),
        "2026-11-01-alpha/proposal.md",
        "x",
    ));
    not_found(documents::spec_document(&store, &scope(), "../config.yaml"));
    not_found(documents::spec_document(&store, &scope(), "auth/spec.md"));
    assert!(
        store.calls().is_empty(),
        "no store call for a name that makes no key"
    );

    assert!(
        documents::archived_capabilities(&store, &scope(), "2026-11-01-alpha/..")
            .unwrap()
            .0
            .is_empty()
    );
    assert!(
        documents::change_delta_capabilities(&store, &scope(), "archive")
            .unwrap()
            .is_empty()
    );
    assert!(
        documents::change_delta_capabilities(&store, &scope(), "add-x/specs")
            .unwrap()
            .is_empty()
    );
    assert!(documents::change_evidence(&store, &scope(), "..")
        .unwrap()
        .0
        .entries
        .is_empty());
    // An artifact path that names the data file or the evidence is no artifact.
    for artifact in [".meta.yaml", ".evidence.json"] {
        assert_eq!(
            documents::document_version(&store, &scope(), "add-x", artifact).unwrap(),
            None,
            "{artifact}"
        );
    }
    let reads: Vec<String> = store.calls().read_keys().into_iter().flatten().collect();
    assert!(
        reads.iter().all(|k| k != "config.yaml"
            && !k.ends_with(".meta.yaml")
            && !k.ends_with(".evidence.json")),
        "{reads:?}"
    );
}

#[test]
fn discard_removes_no_local_file() {
    // 和封存同一類：沒有 workspace 時，discard 也不碰程序目錄裡的 touched 記錄。
    let _serial = WORKING_DIRECTORY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let folder = tempfile::tempdir().expect("temp folder");
    let _back = BackTo(std::env::current_dir().expect("working directory"));
    std::env::set_current_dir(folder.path()).expect("enter the temp folder");
    let local = folder.path().join(".speclink").join("touched").join("demo.json");
    std::fs::create_dir_all(local.parent().unwrap()).unwrap();
    std::fs::write(&local, "{}").unwrap();

    let store = Recorder::new(MemoryStorePort::new());
    write(&store.inner, &[("changes/demo/.meta.yaml", META), ("changes/demo/proposal.md", "# demo\n")]);
    speclink_host::port_exec::execute(
        &store,
        &scope(),
        &common::host_ctx(),
        speclink_core::command::Command::Discard {
            change: "demo".to_string(),
            force: true,
        },
    )
    .expect("the discard succeeds");

    assert!(local.is_file(), "a file in the working directory is not the change's");
    assert!(common::keys_under(&store.inner, "changes/").is_empty(), "the change is gone");
}

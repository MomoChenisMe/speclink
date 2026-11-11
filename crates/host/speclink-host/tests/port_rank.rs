//! `change rank` on a store port (skills-verbs-first-wave D4; change-plan
//! 「change rank 動詞寫入看板順序鍵」的「內嵌宿主經引擎命令排序」). On a store
//! port the board ranks live in the scope's board-order document (`board/order`),
//! never in a change's metadata, and one command lands in one commit.

use serde_json::{json, Value};
use speclink_core::command::{Command, CommandOutcome, ErrorCode};
use speclink_core::plan::Side;
use speclink_host::port_exec::{self, ExecError};
use speclink_protocol::query::BoardOrderDoc;
use speclink_store_port::{
    keys, Commit, DocKey, Document, Expected, KeyPrefix, Listing, MemoryStorePort, Op, PortError,
    ScopeId, StorePort, Timestamp, Version,
};
use std::sync::Mutex;

mod common;
use common::{host_ctx, scope, scope_version, seed};

const A: &str = "schema: spec-driven\ncreated: 2026-09-01\n";
const B: &str = "schema: spec-driven\ncreated: 2026-09-02\n";
const C: &str = "schema: spec-driven\ncreated: 2026-09-03\n";

type Writer = Box<dyn FnOnce(&MemoryStorePort) + Send>;

/// A store that keeps a copy of every commit it is asked to apply, and can let
/// another writer in right before the first one.
struct Recording {
    inner: MemoryStorePort,
    commits: Mutex<Vec<Commit>>,
    before_commit: Mutex<Option<Writer>>,
}

impl StorePort for Recording {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.inner.list(scope, prefix)
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.inner.read(scope, keys)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        if let Some(other_writer) = self.before_commit.lock().unwrap().take() {
            other_writer(&self.inner);
        }
        self.commits.lock().unwrap().push(commit.clone());
        self.inner.commit(scope, commit)
    }
}

/// The changes (name, metadata) and, when given, the board-order document.
fn seeded(changes: &[(&str, &str)], board: Option<&str>) -> Recording {
    let inner = MemoryStorePort::new();
    let mut docs: Vec<(DocKey, String)> = changes
        .iter()
        .map(|(name, meta)| (keys::change_meta(name).unwrap(), meta.to_string()))
        .collect();
    if let Some(board) = board {
        docs.push((keys::board_order(), board.to_string()));
    }
    seed(&inner, docs);
    Recording { inner, commits: Mutex::new(Vec::new()), before_commit: Mutex::new(None) }
}

fn read(port: &dyn StorePort, key: &DocKey) -> Option<Document> {
    port.read(&scope(), std::slice::from_ref(key)).unwrap().pop().flatten()
}

fn board_text(port: &dyn StorePort) -> String {
    read(port, &keys::board_order()).expect("the board document exists").content
}

fn board(port: &dyn StorePort) -> BoardOrderDoc {
    // Strict parse: what the command writes must be the document's real shape.
    serde_json::from_str(&board_text(port)).expect("the board document is a board-order document")
}

/// Another party replaces a document, expecting the version it has now.
fn overwrite(port: &MemoryStorePort, key: &DocKey, content: &str) {
    let current = read(port, key).map(|doc| doc.version);
    let commit = Commit::new(
        speclink_store_port::Actor::new("desktop", "desktop").unwrap(),
        "board-order",
        Timestamp::new(1_800_000_000_000).unwrap(),
        vec![Op::Put {
            key: key.clone(),
            content: content.to_string(),
            expected: current.map_or(Expected::Absent, Expected::At),
        }],
        Vec::new(),
    )
    .unwrap();
    port.commit(&scope(), &commit).expect("the other writer's commit");
}

fn rank(port: &dyn StorePort, name: &str, anchor: &str, side: Side, force: bool) -> Result<port_exec::PortExecution, ExecError> {
    let cmd = Command::ChangeRank { name: name.into(), anchor: anchor.into(), side, force };
    port_exec::execute(port, &scope(), &host_ctx(), cmd)
}

fn plan_order(port: &dyn StorePort) -> Vec<String> {
    let ranks = BoardOrderDoc::from_content(Some(&board_text(port))).changes;
    let run = port_exec::execute(port, &scope(), &host_ctx(), Command::Plan { ranks: Some(ranks) }).expect("plan");
    let CommandOutcome::Plan(plan) = run.outcome else { panic!("a plan") };
    plan.changes.into_iter().map(|c| c.name).collect()
}

#[test]
fn one_commit_carries_the_board_write_and_the_event() {
    let port = seeded(&[("a", A), ("c", C)], Some(r#"{"changes":{"a":"n"}}"#));
    let run = rank(&port, "c", "a", Side::Before, false).expect("the move succeeds");
    let CommandOutcome::ChangeRank(outcome) = &run.outcome else { panic!("a ChangeRank outcome") };
    assert_eq!((outcome.change.as_str(), outcome.anchor.as_str(), outcome.position), ("c", "a", Side::Before));

    let commits = port.commits.lock().unwrap().clone();
    assert_eq!(commits.len(), 1, "exactly one commit");
    let written: Vec<&str> = commits[0].ops().iter().map(|op| op.key().as_str()).collect();
    assert_eq!(written, ["board/order"], "one write, and no change metadata");
    let events: Vec<(&str, &Value)> = commits[0].events().iter().map(|e| (e.name(), e.payload())).collect();
    assert_eq!(events, [("change-ranked", &json!({ "change": "c" }))]);

    assert_eq!(read(&port, &keys::change_meta("a").unwrap()).unwrap().content, A);
    assert_eq!(read(&port, &keys::change_meta("c").unwrap()).unwrap().content, C);
    let doc = board(&port);
    assert_eq!(doc.changes["a"], "n");
    assert!(doc.changes["c"].as_str() < "n", "{:?}", doc.changes);
    assert_eq!(plan_order(&port), ["c", "a"]);
}

#[test]
fn a_column_without_ranks_is_stamped_in_the_same_commit() {
    let port = seeded(&[("a", A), ("b", B), ("c", C)], None);
    let before = scope_version(&port);
    rank(&port, "c", "b", Side::Before, false).expect("the move succeeds");
    assert_eq!(scope_version(&port), before + 1, "the document moves one version");
    let commits = port.commits.lock().unwrap().clone();
    assert_eq!(commits.len(), 1);
    assert!(
        matches!(&commits[0].ops()[..], [Op::Put { key, expected: Expected::Absent, .. }] if key.as_str() == "board/order"),
        "one new board document: {:?}",
        commits[0].ops()
    );
    let ranks = board(&port).changes;
    assert!(ranks["a"] < ranks["c"] && ranks["c"] < ranks["b"], "{ranks:?}");
    assert_eq!(plan_order(&port), ["a", "c", "b"]);
}

#[test]
fn the_discussion_table_is_kept() {
    let port = seeded(&[("a", A), ("c", C)], Some(r#"{"changes":{"a":"n"},"discussions":{"topic":"f","other":"t"}}"#));
    rank(&port, "c", "a", Side::After, false).expect("the move succeeds");
    let text: Value = serde_json::from_str(&board_text(&port)).unwrap();
    assert_eq!(text["discussions"], json!({ "topic": "f", "other": "t" }));
}

#[test]
fn unusable_board_content_reads_as_no_ranks_and_is_rewritten() {
    for content in ["not json", "[1, 2]", r#"{"changes":5}"#, r#"{"changes":{"a":"n"},"discussions":7}"#] {
        let port = seeded(&[("a", A), ("c", C)], Some(content));
        rank(&port, "c", "a", Side::Before, false).unwrap_or_else(|e| panic!("{content}: {e:?}"));
        let doc = board(&port);
        assert!(doc.changes["c"] < doc.changes["a"], "{content}: {:?}", doc.changes);
    }
    // A wrong-typed change table does not take the discussion table with it.
    let port = seeded(&[("a", A), ("c", C)], Some(r#"{"changes":5,"discussions":{"topic":"f"}}"#));
    rank(&port, "c", "a", Side::Before, false).expect("the move succeeds");
    assert_eq!(board(&port).discussions.into_iter().collect::<Vec<_>>(), [("topic".to_string(), "f".to_string())]);
}

#[test]
fn a_refused_move_commits_nothing() {
    let content = r#"{"changes":{"a":"n","c":"t"}}"#;
    let port = seeded(&[("a", A), ("c", C)], Some(content));
    let before = scope_version(&port);
    let err = match rank(&port, "c", "a", Side::After, false) {
        Err(ExecError::Command(e)) => e,
        other => panic!("a command refusal, got {other:?}"),
    };
    assert_eq!(err.code, ErrorCode::Refused);
    assert!(err.message.contains("already has a board rank") && err.message.contains("--force"), "{}", err.message);
    assert!(port.commits.lock().unwrap().is_empty(), "no commit");
    assert_eq!(scope_version(&port), before);
    assert_eq!(board_text(&port), content);
}

#[test]
fn a_board_rewritten_after_the_read_is_a_conflict() {
    let port = seeded(&[("a", A), ("c", C)], Some(r#"{"changes":{"a":"n"}}"#));
    let theirs = r#"{"changes":{"a":"n","c":"b"}}"#;
    *port.before_commit.lock().unwrap() =
        Some(Box::new(move |inner: &MemoryStorePort| overwrite(inner, &keys::board_order(), theirs)));
    match rank(&port, "c", "a", Side::Before, false) {
        Err(ExecError::Store(PortError::Conflict { key, .. })) => assert_eq!(key.as_str(), "board/order"),
        other => panic!("a version conflict, got {other:?}"),
    }
    assert_eq!(board_text(&port), theirs, "the other party's write stands");
}

#[test]
fn a_name_that_leaves_the_changes_folder_writes_nothing() {
    // 稽核（skills-verbs-first-wave 4.1）：名稱與錨點都只能是作用中的變更名稱；帶
    // 路徑字元的名稱在任何寫入之前被拒，順序鍵也只會落在 board/order 這一個鍵。
    let content = r#"{"changes":{"a":"n"}}"#;
    let port = seeded(&[("a", A), ("c", C)], Some(content));
    for (name, anchor, code) in [
        ("../a", "a", ErrorCode::NotFound),
        ("c/../a", "a", ErrorCode::NotFound),
        ("c", "../a", ErrorCode::Refused),
        ("c", "a/x", ErrorCode::Refused),
    ] {
        match rank(&port, name, anchor, Side::Before, false) {
            Err(ExecError::Command(e)) => assert_eq!(e.code, code, "{name} --before {anchor}: {}", e.message),
            other => panic!("{name} --before {anchor}: a refusal, got {other:?}"),
        }
    }
    assert!(port.commits.lock().unwrap().is_empty(), "no commit");
    assert_eq!(board_text(&port), content);
}

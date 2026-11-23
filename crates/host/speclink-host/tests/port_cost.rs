//! What a command costs a store port (design D6).
//!
//! This project's own `speclink/` folder is loaded into a `MemoryStorePort`.
//! A counting wrapper then records, for a set of representative commands, how
//! many times each one lists, reads and commits, and how many keys and bytes
//! it reads. The test prints the numbers and fails when a read-type command
//! touches the archive, when a command lists a prefix or reads a key twice,
//! or when a write-type command does not commit exactly once.
//!
//! Run with `cargo test -p speclink-host --test port_cost -- --nocapture` to
//! see the table.

use speclink_core::command::{Command, CommandOutcome, ListOutcome};
use speclink_core::station::StationProgress;
use speclink_host::port_exec;
use speclink_store_port::{DocKey, Expected, MemoryStorePort, Op};

mod common;
use common::{host_ctx, scope, seed, spec_folder_docs, Calls, Recorder};
use std::collections::BTreeSet;

const CHANGE: &str = "cost-probe";
const TALK: &str = "cost-probe-talk";

fn doc(key: &str, content: &str) -> (DocKey, String) {
    (DocKey::new(key).unwrap(), content.to_string())
}

/// This project's spec folder as a store, plus the documents the measured
/// commands work on. Returns the store and the bytes it holds.
fn loaded_store() -> (MemoryStorePort, usize, usize) {
    let docs = spec_folder_docs();
    let documents = docs.len();
    let bytes = docs.iter().map(|(_, content)| content.len()).sum();
    let port = MemoryStorePort::new();
    seed(&port, docs);
    // The commands that need one change or one discussion use these, so the
    // test does not depend on which changes are active in the repo today.
    seed(
        &port,
        vec![
            doc(
                &format!("changes/{CHANGE}/.meta.yaml"),
                "schema: spec-driven\ncreated: 2026-11-11\n",
            ),
            doc(
                &format!("changes/{CHANGE}/proposal.md"),
                "## Why\n\nMeasure what a command costs a store.\n",
            ),
            doc(
                &format!("changes/{CHANGE}/tasks.md"),
                "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n",
            ),
            doc(
                &format!("changes/{CHANGE}/specs/cost-probe-cap/spec.md"),
                "## Purpose\n\n本 capability 只存在於量測測試裡，用來讓封存有一份新的正式規格可以寫入，內容沒有其他用途。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
            ),
            doc(
                &format!("discussions/{TALK}.md"),
                "---\ntopic: Cost probe\nslug: cost-probe-talk\nstatus: open\ncreated: 2026-11-11\n---\n\n## Round 1\n\nThe cost-probe keyword.\n",
            ),
        ],
    );
    (port, documents, bytes)
}

// --- counting ---

/// The numbers of one command.
struct Row {
    name: &'static str,
    writes: bool,
    calls: Calls,
}

impl Row {
    fn read_keys(&self) -> impl Iterator<Item = &(String, Option<usize>)> {
        self.calls.reads.iter().flatten()
    }

    fn store_calls(&self) -> usize {
        self.calls.lists.len() + self.calls.reads.len() + self.calls.commits.len()
    }

    /// Keys read as "no document" and then created by the commit: the reads
    /// that only gave a write its expected version.
    fn read_before_create(&self) -> Vec<&str> {
        let created: BTreeSet<&str> = self
            .calls
            .commits
            .iter()
            .flat_map(|commit| commit.ops())
            .filter_map(|op| match op {
                Op::Put {
                    key,
                    expected: Expected::Absent,
                    ..
                } => Some(key.as_str()),
                _ => None,
            })
            .collect();
        self.read_keys()
            .filter(|(key, size)| size.is_none() && created.contains(key.as_str()))
            .map(|(key, _)| key.as_str())
            .collect()
    }
}

fn measure(store: &Recorder, name: &'static str, writes: bool, cmd: Command) -> Row {
    store.take();
    port_exec::execute(store, &scope(), &host_ctx(), cmd)
        .unwrap_or_else(|e| panic!("{name} failed: {e:?}"));
    Row {
        name,
        writes,
        calls: store.take(),
    }
}

#[test]
fn commands_read_only_what_they_use() {
    let (inner, documents, bytes) = loaded_store();
    let store = Recorder::new(inner);
    let change = || Some(CHANGE.to_string());

    let rows = [
        measure(
            &store,
            "list changes",
            false,
            Command::List {
                sort: "modified".to_string(),
                specs: false,
                changes: false,
                worktrees: Default::default(),
                detail: false,
                archived: false,
            },
        ),
        measure(
            &store,
            "status of one change",
            false,
            Command::Status {
                change: change(),
                schema: None,
            },
        ),
        measure(&store, "plan", false, Command::Plan { ranks: None }),
        measure(
            &store,
            "list discussions",
            false,
            Command::DiscussList { archived: false },
        ),
        measure(
            &store,
            "new artifact",
            true,
            Command::NewArtifact {
                kind: "design".to_string(),
                capability: None,
                change: change(),
                content: Some("## Context\n\nA design.\n".to_string()),
                force: false,
                new_capability: false,
            },
        ),
        measure(
            &store,
            "task done",
            true,
            Command::TaskDone {
                task_id: "1".to_string(),
                change: change(),
                touched_files: Some(vec!["src/app.rs".to_string()]),
                head_commit: None,
            },
        ),
        measure(
            &store,
            "archive a change",
            true,
            Command::Archive {
                change: change(),
                skip_specs: false,
                no_validate: true,
                mark_tasks_complete: true,
                carry_review: false,
                carry_verify: false,
            },
        ),
        measure(
            &store,
            "search discussions",
            false,
            Command::DiscussSearch {
                terms: vec!["cost-probe".to_string()],
            },
        ),
    ];

    println!("store: {documents} documents, {bytes} bytes");
    println!(
        "{:<22} {:>5} {:>5} {:>9} {:>10} {:>7} {:>11} {:>18}",
        "command", "lists", "reads", "keys read", "bytes read", "commits", "store calls", "read before create"
    );
    for row in &rows {
        println!(
            "{:<22} {:>5} {:>5} {:>9} {:>10} {:>7} {:>11} {:>18}",
            row.name,
            row.calls.lists.len(),
            row.calls.reads.len(),
            row.read_keys().count(),
            row.read_keys().filter_map(|(_, size)| *size).sum::<usize>(),
            row.calls.commits.len(),
            row.store_calls(),
            row.read_before_create().len(),
        );
    }
    for row in &rows {
        println!("{}: lists {:?}", row.name, row.calls.lists);
        let created = row.read_before_create();
        if !created.is_empty() {
            println!("{}: read before create {created:?}", row.name);
        }
    }

    for row in &rows {
        // Every command: a prefix is listed at most once, a key is read at
        // most once.
        let mut prefixes = BTreeSet::new();
        for prefix in &row.calls.lists {
            assert!(prefixes.insert(prefix), "{}: {prefix} listed twice", row.name);
        }
        let mut keys = BTreeSet::new();
        for (key, _) in row.read_keys() {
            assert!(keys.insert(key), "{}: {key} read twice", row.name);
        }
        // A write-type command commits exactly once; a read-type one never.
        let commits = if row.writes { 1 } else { 0 };
        assert_eq!(row.calls.commits.len(), commits, "{}: commits", row.name);
    }

    // The three read-type commands stay away from the archive.
    for row in rows
        .iter()
        .filter(|row| ["list changes", "status of one change", "list discussions"].contains(&row.name))
    {
        assert!(
            !row.calls.lists.iter().any(|prefix| prefix == "archive/changes/"),
            "{}: lists archive/changes/",
            row.name
        );
        for (key, _) in row.read_keys() {
            assert!(!key.starts_with("archive/"), "{}: reads {key}", row.name);
        }
    }
}

/// What the archived list costs on this project's own archive
/// (archived-list-and-detail-reads, the store's prefetch hint): one list of
/// the archive and one read of every archived change's data file, task list
/// and tickets, through the one implementation every entry point shares. The
/// numbers are printed for the design.
#[test]
fn the_archived_list_is_one_list_and_one_read() {
    let (inner, _, _) = loaded_store();
    let store = Recorder::new(inner);
    let cmd = Command::List {
        sort: "name".to_string(),
        specs: false,
        changes: false,
        worktrees: Default::default(),
        detail: false,
        archived: true,
    };
    store.take();
    let started = std::time::Instant::now();
    let execution = port_exec::execute(&store, &scope(), &host_ctx(), cmd).expect("the archived list");
    let elapsed = started.elapsed();
    let calls = store.take();
    let CommandOutcome::List(ListOutcome::Archived(archived)) = execution.outcome else {
        panic!("expected an archived list outcome");
    };
    let keys: usize = calls.reads.iter().map(Vec::len).sum();
    let bytes: usize = calls.reads.iter().flatten().filter_map(|(_, size)| *size).sum();
    println!(
        "archived list: {} archived changes; {} list, {} read, {keys} keys, {bytes} bytes, {elapsed:?}",
        archived.len(),
        calls.lists.len(),
        calls.reads.len()
    );
    assert_eq!(calls.lists, ["archive/changes/"]);
    assert_eq!(calls.reads.len(), 1);
    assert!(calls.commits.is_empty());
    // A key the listing does not have is not read.
    for (key, size) in calls.reads.iter().flatten() {
        assert!(size.is_some(), "a document the listing does not have was read: {key}");
    }
    assert!(!archived.is_empty(), "this project has archived changes");
}

/// What `list --json --detail` costs (archived-list-and-detail-reads,
/// revising verb-output-for-embedded D3): with open tickets, the calls of a
/// list without it and one read more, which holds every open ticket at once —
/// their rounds are in them; with none, the calls of a list without it. The
/// claim, the branch and the landing are in the data file the list already
/// reads, and the listing of `changes/` says which tickets exist.
#[test]
fn a_detailed_list_costs_one_read_more_only_for_open_tickets() {
    const PROBE: &str = "cost-probe-detail";
    const OTHER: &str = "cost-probe-detail-two";
    let run = |store: &Recorder, detail| {
        let cmd = Command::List {
            sort: "modified".to_string(),
            specs: false,
            changes: false,
            worktrees: Default::default(),
            detail,
            archived: false,
        };
        store.take();
        let execution = port_exec::execute(store, &scope(), &host_ctx(), cmd).expect("list");
        (execution.outcome, store.take())
    };

    // Two changes with an open ticket: one more read, for both tickets.
    let (inner, _, _) = loaded_store();
    seed(
        &inner,
        vec![
            doc(
                &format!("changes/{PROBE}/.meta.yaml"),
                "schema: spec-driven\ncreated: 2026-11-18\nbranch: speclink/cost-probe-detail\n",
            ),
            doc(&format!("changes/{PROBE}/tasks.md"), "- [ ] 1.1 first task\n"),
            doc(
                &format!("changes/{PROBE}/review.md"),
                "# Review — cost-probe-detail\n\n## Round 1\n\n**Scope**: src/app.rs\n",
            ),
            doc(&format!("changes/{OTHER}/.meta.yaml"), "schema: spec-driven\n"),
            doc(
                &format!("changes/{OTHER}/verify.md"),
                "# Verify — two\n\n## Round 1\n\n**Scope**: src/app.rs\n\n## Round 2\n\n**Scope**: src/app.rs\n",
            ),
        ],
    );
    let store = Recorder::new(inner);
    let (_, plain) = run(&store, false);
    let (outcome, detailed) = run(&store, true);

    // The detail is there: the open tickets with their rounds, the recorded branch.
    let CommandOutcome::List(ListOutcome::Sections { changes: Some(changes), .. }) = outcome else {
        panic!("expected a list outcome with the changes section");
    };
    let probe = changes.iter().find(|change| change.name == PROBE).expect("the probe change");
    let review = probe.review.as_ref().expect("an open review");
    assert_eq!((review.progress, review.rounds), (StationProgress::InProgress, Some(1)));
    assert_eq!(probe.branch.as_deref(), Some("speclink/cost-probe-detail"));
    let other = changes.iter().find(|change| change.name == OTHER).expect("the second change");
    assert_eq!(other.verify.as_ref().and_then(|status| status.rounds), Some(2));

    println!(
        "list changes --detail: {} list, {} read, {} keys (plain: {} list, {} read, {} keys)",
        detailed.lists.len(),
        detailed.reads.len(),
        detailed.reads.iter().map(Vec::len).sum::<usize>(),
        plain.lists.len(),
        plain.reads.len(),
        plain.reads.iter().map(Vec::len).sum::<usize>(),
    );
    assert_eq!(detailed.lists, plain.lists, "no list call more");
    assert_eq!(detailed.reads.len(), plain.reads.len() + 1, "one read call more");
    assert!(detailed.commits.is_empty());
    let tickets: Vec<&(String, Option<usize>)> = detailed
        .reads
        .iter()
        .flatten()
        .filter(|(key, _)| key.ends_with("/review.md") || key.ends_with("/verify.md"))
        .collect();
    for key in [format!("changes/{PROBE}/review.md"), format!("changes/{OTHER}/verify.md")] {
        assert_eq!(tickets.iter().filter(|(k, _)| *k == key).count(), 1, "{key} is read once: {tickets:?}");
    }
    assert!(tickets.iter().all(|(_, size)| size.is_some()), "a ticket the listing does not have is not read");

    // No open ticket: the calls of a plain list, not one more.
    let docs = spec_folder_docs()
        .into_iter()
        .filter(|(key, _)| {
            let key = key.as_str();
            !(key.starts_with("changes/") && (key.ends_with("/review.md") || key.ends_with("/verify.md")))
        })
        .collect();
    let inner = MemoryStorePort::new();
    seed(&inner, docs);
    let store = Recorder::new(inner);
    let (_, plain) = run(&store, false);
    let (_, detailed) = run(&store, true);
    assert_eq!(detailed.lists, plain.lists, "no list call more");
    assert_eq!(detailed.read_keys(), plain.read_keys(), "no read call and no key more");
}

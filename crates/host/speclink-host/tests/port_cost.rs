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

use speclink_core::command::Command;
use speclink_host::port_exec;
use speclink_store_port::{DocKey, Expected, MemoryStorePort, Op};

mod common;
use common::{host_ctx, scope, seed, Calls, Recorder};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CHANGE: &str = "cost-probe";
const TALK: &str = "cost-probe-talk";

// --- loading the folder ---

/// The key of a file under `speclink/`, from its path segments. The archive
/// moves to the top-level `archive/` (the key naming rules); a file that is
/// no speclink document (the manual, for one) has no key.
fn key_of(segments: &[String]) -> Option<String> {
    let parts: Vec<&str> = segments.iter().map(String::as_str).collect();
    match parts.as_slice() {
        ["changes", "archive", dated, rest @ ..] if !rest.is_empty() => {
            Some(format!("archive/changes/{dated}/{}", rest.join("/")))
        }
        ["discussions", "archive", file] => Some(format!("archive/discussions/{file}")),
        ["changes", _, _, ..] | ["specs", _, _] | ["discussions", _] => Some(parts.join("/")),
        ["config.yaml"] | ["LANGUAGE.md"] => Some(parts.join("/")),
        _ => None,
    }
}

/// Every file under `dir` as its path segments from the root. Segments, not
/// a path string: the key separator is `/` on every platform.
fn walk(dir: &Path, segments: &mut Vec<String>, out: &mut Vec<(Vec<String>, PathBuf)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("directory entry"))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        segments.push(entry.file_name().to_string_lossy().into_owned());
        let path = entry.path();
        if path.is_dir() {
            walk(&path, segments, out);
        } else {
            out.push((segments.clone(), path));
        }
        segments.pop();
    }
}

fn doc(key: &str, content: &str) -> (DocKey, String) {
    (DocKey::new(key).unwrap(), content.to_string())
}

/// This project's spec folder as a store, plus the documents the measured
/// commands work on. Returns the store and the bytes it holds.
fn loaded_store() -> (MemoryStorePort, usize, usize) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("speclink");
    let mut files = Vec::new();
    walk(&root, &mut Vec::new(), &mut files);
    let mut docs = Vec::new();
    let mut bytes = 0;
    for (segments, path) in files {
        let Some(key) = key_of(&segments) else {
            continue;
        };
        // A file that is not text, or a name the key format rejects, is not
        // a speclink document.
        let (Ok(content), Ok(key)) = (std::fs::read_to_string(&path), DocKey::new(key)) else {
            continue;
        };
        bytes += content.len();
        docs.push((key, content));
    }
    let documents = docs.len();
    assert!(documents > 100, "the spec folder was found at {}", root.display());
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

/// What the archived list costs on this project's own archive (design Open
/// Questions): one list of the archive and one read of every archived
/// change's task list and data file. The numbers are printed for the design.
#[test]
fn the_archived_list_is_one_list_and_one_read() {
    let (inner, _, _) = loaded_store();
    let store = Recorder::new(inner);
    let started = std::time::Instant::now();
    let (list, _) = speclink_host::documents::archived_list(&store, &scope()).expect("the archived list");
    let elapsed = started.elapsed();
    let calls = store.take();
    let keys: usize = calls.reads.iter().map(Vec::len).sum();
    let bytes: usize = calls.reads.iter().flatten().filter_map(|(_, size)| *size).sum();
    println!(
        "archived list: {} archived changes; {} list, {} read, {keys} keys, {bytes} bytes, {elapsed:?}",
        list.archived.len(),
        calls.lists.len(),
        calls.reads.len()
    );
    assert_eq!(calls.lists, ["archive/changes/"]);
    assert_eq!(calls.reads.len(), 1);
    assert!(!list.archived.is_empty(), "this project has archived changes");
}

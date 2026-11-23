//! The events of a commit cover every active change and discussion it wrote
//! (spec command-runtime「一次提交的事件涵蓋它寫到的每個變更與討論」). Each row
//! runs one command through the kit on the in-memory store; the resources
//! come from the commit's keys (`changes/<name>/…`, `discussions/<slug>.md`,
//! per store-port-contract), and each one needs an event that
//! [`event_resource`] points at it. `change-archived` points at the specs
//! the archive promoted; its identity is the change, so it counts for the
//! change. Other keys (the archive, specs, the workflow config, LANGUAGE,
//! the board order) are outside the rule.
//!
//! The rows walk the Engine's command names: a command that writes and has
//! no row fails [`every_engine_command_that_writes_has_a_row`]. The one
//! write left out is the kit's scope import, which is not an Engine
//! command: it replaces the whole scope, and its one `scope-imported` event
//! means exactly that.

use speclink_core::command::{Command, COMMAND_NAMES};
use speclink_core::config::WorkflowConfigEdit;
use speclink_host::commit::event_resource;
use speclink_host::kit::{Caller, Hooks, HostKit, Identity};
use speclink_host::read_only::is_read;
use speclink_protocol::events::InvalidationScope;
use speclink_store_port::{Commit, MemoryStorePort};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod common;

const META: &str = "schema: spec-driven\n";
const OPEN_TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n";
const DONE_TASKS: &str = "## 1. Section\n\n- [x] 1.1 first\n";
const SCOPE: &str = "**Scope**: src/lib.rs\n";

/// A discussion record with `status` and `promoted_to`, concluded when
/// `concluded` (else the conclusion is still the placeholder).
fn record(slug: &str, status: &str, promoted_to: Option<&str>, concluded: bool) -> String {
    let promoted = promoted_to.map(|c| format!("promoted_to: {c}\n")).unwrap_or_default();
    let conclusion = if concluded { "**Decision**: x" } else { "<!-- Written by `speclink discuss conclude` -->" };
    format!(
        "---\ntopic: {slug}\nslug: {slug}\nstatus: {status}\n{promoted}created: 2026-11-01\n---\n\n\
         ## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n{conclusion}\n"
    )
}

/// What every row starts from: changes `demo` (open tasks) and `base`, and
/// the open discussion `talk`.
fn base() -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        ("config.yaml", META.to_string()),
        ("changes/demo/.meta.yaml", META.to_string()),
        ("changes/demo/tasks.md", OPEN_TASKS.to_string()),
        ("changes/base/.meta.yaml", META.to_string()),
        ("discussions/talk.md", record("talk", "open", None, false)),
    ])
}

struct Row {
    /// What the row is, for the failure message.
    label: &'static str,
    /// Documents written over [`base`] before the row runs.
    seed: Vec<(&'static str, String)>,
    /// Commands run before the one the row checks.
    before: Vec<Command>,
    command: Command,
}

fn row(label: &'static str, command: Command) -> Row {
    Row { label, seed: Vec::new(), before: Vec::new(), command }
}

impl Row {
    fn seed(mut self, docs: &[(&'static str, &str)]) -> Row {
        self.seed.extend(docs.iter().map(|(key, text)| (*key, text.to_string())));
        self
    }

    fn after(mut self, before: Command) -> Row {
        self.before.push(before);
        self
    }
}

fn s(text: &str) -> String {
    text.to_string()
}

fn new_change(name: &str, from_discussion: Option<&str>) -> Command {
    Command::NewChange {
        name: s(name),
        description: None,
        schema: None,
        agent: None,
        from_discussion: from_discussion.map(s),
        last: false,
        fields: Default::default(),
    }
}

fn archive(name: &str) -> Command {
    Command::Archive {
        change: Some(s(name)),
        skip_specs: false,
        no_validate: false,
        mark_tasks_complete: false,
        carry_review: false,
        carry_verify: false,
    }
}

fn conclude(slug: &str) -> Command {
    Command::DiscussConclude { slug: s(slug), content: s("結論：做。"), hold: false }
}

/// The reviewed scope a station stamp of `demo` carries.
fn stamp_scope() -> Vec<speclink_core::model::ReviewedScopeEntry> {
    vec![speclink_core::model::ReviewedScopeEntry {
        path: s("src/lib.rs"),
        hash: speclink_core::station::content_fingerprint("fn lib() {}\n"),
    }]
}

/// One row per command that writes, and one per case where a commit writes
/// a resource besides the command's own (design: the seven cross-resource
/// cases).
fn rows() -> Vec<Row> {
    let two_concluded = record("two", "concluded", None, true);
    let two_promoted = record("two", "promoted", Some("add-two"), true);
    let from_two = "schema: spec-driven\nfrom_discussion: two\n";
    let demo_done: &[(&str, &str)] = &[("changes/demo/tasks.md", DONE_TASKS)];
    vec![
        row("new change", new_change("solo", None)),
        row("new change --from-discussion", new_change("add-two", Some("two"))).seed(&[("discussions/two.md", two_concluded.as_str())]),
        row(
            "new artifact",
            Command::NewArtifact {
                kind: s("proposal"),
                capability: None,
                change: Some(s("demo")),
                content: Some(s("## Why\n\nx\n")),
                force: true,
                new_capability: false,
            },
        ),
        row("task done", Command::TaskDone { task_id: s("1"), change: Some(s("demo")), touched_files: None, head_commit: None }),
        row("task undone", Command::TaskUndone { task_id: s("1"), change: Some(s("demo")) }).seed(demo_done),
        row("task move", Command::TaskMove { change: s("demo"), from: 1, to: 2, before: None }),
        row("claim", Command::Claim { name: s("demo") }),
        row("claim release", Command::ClaimRelease { name: s("demo") }).after(Command::Claim { name: s("demo") }),
        row("in-progress add", Command::InProgressAdd { name: s("demo"), branch: None }),
        row("in-progress remove", Command::InProgressRemove { name: s("demo") })
            .after(Command::InProgressAdd { name: s("demo"), branch: None }),
        row("change depends", Command::ChangeDepends { name: s("demo"), on: vec![s("base")], remove: false }),
        row("change schema", Command::ChangeSchema { name: s("demo"), schema: s("no-spec-change") }),
        row(
            "change rank",
            Command::ChangeRank {
                name: s("demo"),
                anchor: s("base"),
                position: speclink_core::plan::Side::parse("before").unwrap(),
                force: false,
            },
        ),
        row("change landed", Command::ChangeLanded { name: s("demo"), commit: s("0123456789ab"), branch: None }),
        row(
            "change field set",
            Command::ChangeFieldsEdit { name: s("demo"), set: [(s("ticket"), s("PROJ-1"))].into(), unset: vec![] },
        ),
        row(
            "discuss field set",
            Command::DiscussFieldsEdit { slug: s("talk"), set: [(s("owner"), s("alice"))].into(), unset: vec![] },
        ),
        row("archive", archive("demo")).seed(demo_done),
        row("archive, the source discussion archived along", archive("add-two")).seed(&[
            ("changes/add-two/.meta.yaml", from_two),
            ("changes/add-two/tasks.md", DONE_TASKS),
            ("discussions/two.md", two_promoted.as_str()),
        ]),
        row("discard", Command::Discard { change: s("demo"), force: false }),
        row("discard, the source discussion unlinked", Command::Discard { change: s("add-two"), force: false })
            .seed(&[("changes/add-two/.meta.yaml", from_two), ("discussions/two.md", two_promoted.as_str())]),
        row("discuss new", Command::DiscussNew { topic: s("Talk two"), slug: None, kind: None, fields: Default::default() }),
        row("discuss context", Command::DiscussContext { slug: s("talk"), content: s("context") }),
        row("discuss add-round", Command::DiscussAddRound { slug: s("talk"), mode: s("round"), content: s("text") }),
        row("discuss conclude", conclude("talk")),
        row("discuss conclude, the record archived along", conclude("two"))
            .seed(&[("discussions/two.md", record("two", "promoted", Some("gone"), true).as_str())]),
        row("discuss conclude again, a reflected change flagged", conclude("two"))
            .seed(&[("changes/add-two/.meta.yaml", from_two), ("discussions/two.md", two_promoted.as_str())]),
        row(
            "discuss promote",
            Command::DiscussPromote { slug: s("two"), name: Some(s("add-two")), last: false, fields: Default::default() },
        )
        .seed(&[("discussions/two.md", two_concluded.as_str())]),
        row("discuss link", Command::DiscussLink { slug: s("talk"), change: s("demo") }),
        row("discuss seal, the change's flag cleared", Command::DiscussSeal { slug: s("two"), change: s("demo"), last: false })
            .seed(&[
                ("changes/demo/.meta.yaml", "schema: spec-driven\nfrom_discussion: two\nrestale_from: two\n"),
                ("discussions/two.md", two_promoted.as_str()),
            ]),
        row("discuss archive", Command::DiscussArchive { slug: s("talk") }),
        row("discuss restore", Command::DiscussRestore { slug: s("old") })
            .seed(&[("archive/discussions/2026-11-01-old.md", record("old", "concluded", None, true).as_str())]),
        row("discuss discard", Command::DiscussDiscard { slug: s("talk"), force: false }),
        row("language replace", Command::LanguageReplace { content: s("# Language\n") }),
        row("review add-round", Command::ReviewAddRound { change: s("demo"), content: s(SCOPE) }),
        row(
            "review stamp",
            Command::ReviewStamp {
                change: s("demo"),
                accept: false,
                tool: None,
                head_commit: None,
                scope: stamp_scope(),
                missing: vec![],
            },
        )
            .seed(demo_done)
            .after(Command::ReviewAddRound { change: s("demo"), content: s(SCOPE) }),
        row("review discard", Command::ReviewDiscard { change: s("demo") })
            .after(Command::ReviewAddRound { change: s("demo"), content: s(SCOPE) }),
        row("verify add-round", Command::VerifyAddRound { change: s("demo"), content: s(SCOPE) }).seed(demo_done),
        row(
            "verify stamp",
            Command::VerifyStamp {
                change: s("demo"),
                accept: false,
                tool: None,
                head_commit: None,
                scope: stamp_scope(),
                missing: vec![],
            },
        )
            .seed(demo_done)
            .after(Command::VerifyAddRound { change: s("demo"), content: s(SCOPE) }),
        row("verify discard", Command::VerifyDiscard { change: s("demo") })
            .seed(demo_done)
            .after(Command::VerifyAddRound { change: s("demo"), content: s(SCOPE) }),
        row(
            "workflow-config edit",
            Command::WorkflowConfigEdit { edit: WorkflowConfigEdit::Policy { key: s("tdd"), value: s("true") }, dry_run: false },
        ),
        row("workflow-config replace", Command::WorkflowConfigReplace { content: s("schema: spec-driven\ntdd: false\n") }),
    ]
}

fn caller() -> Caller {
    Caller { identity: Identity::new("u_1", "Alice").unwrap(), scope: common::scope() }
}

/// The commit of the row's command, on a store seeded for the row.
fn commit_of(row: &Row) -> Commit {
    let mut docs = base();
    docs.extend(row.seed.iter().cloned());
    let port = MemoryStorePort::new();
    common::write(&port, &docs.iter().map(|(key, text)| (*key, text.as_str())).collect::<Vec<_>>());
    let recorder = Arc::new(common::Recorder::new(port));
    let kit = HostKit::new(recorder.clone(), Hooks::default());
    for cmd in &row.before {
        kit.run(&caller(), cmd.clone()).unwrap_or_else(|e| panic!("{}: {} before it: {e:?}", row.label, cmd.name()));
    }
    recorder.take();
    kit.run(&caller(), row.command.clone()).unwrap_or_else(|e| panic!("{}: {e:?}", row.label));
    let mut commits = recorder.take().commits;
    assert_eq!(commits.len(), 1, "{}: one commit", row.label);
    commits.remove(0)
}

/// An active change or an active discussion, by its name or its slug.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Resource {
    Change(String),
    Discussion(String),
}

/// The active changes and discussions whose documents the commit wrote or deleted.
fn written(commit: &Commit) -> BTreeSet<Resource> {
    let resource = |key: &str| {
        if let Some(rest) = key.strip_prefix("changes/") {
            return rest.split_once('/').map(|(name, _)| Resource::Change(name.to_string()));
        }
        let slug = key.strip_prefix("discussions/")?.strip_suffix(".md")?;
        (!slug.contains('/')).then(|| Resource::Discussion(slug.to_string()))
    };
    commit.ops().iter().filter_map(|op| resource(op.key().as_str())).collect()
}

/// The changes and discussions the commit's events point at.
fn pointed(commit: &Commit) -> BTreeSet<Resource> {
    let resource = |name: &str, payload| match event_resource(name, payload) {
        (InvalidationScope::Change, id) => Some(Resource::Change(id)),
        (InvalidationScope::Spec, id) if name == "change-archived" => Some(Resource::Change(id)),
        (InvalidationScope::Discussion, id) => Some(Resource::Discussion(id)),
        _ => None,
    };
    commit.events().iter().filter_map(|e| resource(e.name(), e.payload())).collect()
}

#[test]
fn every_change_and_discussion_a_commit_writes_has_an_event() {
    for row in rows() {
        let commit = commit_of(&row);
        assert!(!commit.ops().is_empty(), "{}: the row writes nothing, so it checks nothing", row.label);
        let missing: Vec<_> = written(&commit).difference(&pointed(&commit)).cloned().collect();
        let events: Vec<&str> = commit.events().iter().map(|e| e.name()).collect();
        assert!(missing.is_empty(), "{}: no event points at {missing:?}; the events are {events:?}", row.label);
    }
}

#[test]
fn the_cross_resource_rows_write_what_they_are_about() {
    // A row that stops writing the other resource would pass the coverage
    // check without checking it.
    let change = |name: &str| Resource::Change(s(name));
    let discussion = |slug: &str| Resource::Discussion(s(slug));
    let cases = [
        ("new change --from-discussion", vec![change("add-two"), discussion("two")]),
        ("archive, the source discussion archived along", vec![change("add-two"), discussion("two")]),
        ("discard, the source discussion unlinked", vec![change("add-two"), discussion("two")]),
        ("discuss conclude, the record archived along", vec![discussion("two")]),
        ("discuss conclude again, a reflected change flagged", vec![change("add-two"), discussion("two")]),
        ("discuss link", vec![change("demo")]),
        ("discuss seal, the change's flag cleared", vec![change("demo"), discussion("two")]),
    ];
    for (label, want) in cases {
        let row = rows().into_iter().find(|r| r.label == label).unwrap();
        assert_eq!(written(&commit_of(&row)), want.into_iter().collect(), "{label}");
    }
}

#[test]
fn every_engine_command_that_writes_has_a_row() {
    let reads: BTreeMap<&str, bool> = common::every_command().iter().map(|c| (c.name(), is_read(c))).collect();
    let rows: BTreeSet<&str> = rows().iter().map(|r| r.command.name()).collect();
    let missing: Vec<&str> = COMMAND_NAMES
        .iter()
        .copied()
        .filter(|name| !reads.get(name).copied().unwrap_or(false) && !rows.contains(name))
        .collect();
    assert!(missing.is_empty(), "commands that write and have no row: {missing:?}");
}

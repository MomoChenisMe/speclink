//! Shared by the integration tests that run the Engine on a store port.

// Each test target uses its own part of this module.
#![allow(dead_code)]

use speclink_core::command::{Command, CommandOutcome, ShowChange, ShowOutcome};
use speclink_core::config::{ResolvedPolicy, WorkflowConfigEdit};
use speclink_host::binding::local_default_binding;
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::kit::Reader;
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_host::port_exec::ExecError;
use speclink_store_port::{
    Actor as PortActor, Commit, DocKey, Document, Event, Expected, KeyPrefix, Listing, MemoryStorePort,
    Op, PortError, ScopeId, StorePort, Timestamp, Version,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub mod door;

pub const ACTOR: &str = "Alice <alice@example.com>";

pub fn scope() -> ScopeId {
    ScopeId::new("default/main").unwrap()
}

/// The Host context of an identified actor on the default binding.
pub fn host_ctx() -> SpeclinkExecutionContext {
    let binding = local_default_binding();
    SpeclinkExecutionContext {
        actor: Actor::Identified {
            display: ACTOR.to_string(),
            source: ActorSource::Explicit,
        },
        project: binding.project,
        repo: binding.repo,
        mode: ExecutionMode::SharedStore,
        policy: EffectiveWorkflowPolicy::new(
            ResolvedPolicy {
                locale: "English".to_string(),
                spec_locale: None,
                tdd: false,
                audit: false,
                worktree: false,
            },
            "",
        ),
    }
}

/// Write `docs` into `port` in one commit: each document is created, or
/// replaced at its current version.
pub fn seed(port: &dyn StorePort, docs: Vec<(DocKey, String)>) {
    seed_in(port, &scope(), docs);
}

/// [`seed`] on another scope.
pub fn seed_in(port: &dyn StorePort, scope: &ScopeId, docs: Vec<(DocKey, String)>) {
    let keys: Vec<DocKey> = docs.iter().map(|(key, _)| key.clone()).collect();
    let current = port.read(scope, &keys).expect("read before the seed");
    let ops = docs
        .into_iter()
        .zip(current)
        .map(|((key, content), doc)| Op::Put {
            key,
            content,
            expected: doc.map_or(Expected::Absent, |d| Expected::At(d.version)),
        })
        .collect();
    let commit = Commit::new(
        PortActor::new("seed", "seed").unwrap(),
        "seed",
        Timestamp::new(1_700_000_000_000).unwrap(),
        ops,
        Vec::new(),
    )
    .unwrap();
    port.commit(scope, &commit).expect("seed commit");
}

pub fn scope_version(port: &dyn StorePort) -> u64 {
    scope_version_in(port, &scope())
}

/// [`scope_version`] of another scope.
pub fn scope_version_in(port: &dyn StorePort, scope: &ScopeId) -> u64 {
    port.list(scope, &KeyPrefix::new("").unwrap())
        .expect("list")
        .version
        .get()
}

/// The keys that start with `prefix`, in byte order.
pub fn keys_under(port: &dyn StorePort, prefix: &str) -> Vec<String> {
    port.list(&scope(), &KeyPrefix::new(prefix).unwrap())
        .expect("list")
        .entries
        .into_iter()
        .map(|entry| entry.key.as_str().to_string())
        .collect()
}

/// [`seed`], from key and content text.
pub fn write(port: &dyn StorePort, docs: &[(&str, &str)]) {
    write_in(port, &scope(), docs);
}

/// [`write`] on another scope.
pub fn write_in(port: &dyn StorePort, scope: &ScopeId, docs: &[(&str, &str)]) {
    seed_in(
        port,
        scope,
        docs.iter()
            .map(|(key, content)| (DocKey::new(*key).unwrap(), content.to_string()))
            .collect(),
    );
}

// --- recording what a command asks of a store ---

/// Every call a [`Recorder`] saw, with what came back.
#[derive(Debug, Default, Clone)]
pub struct Calls {
    /// The prefix of every list call.
    pub lists: Vec<String>,
    /// Every read call: each key with the size of what came back.
    pub reads: Vec<Vec<(String, Option<usize>)>>,
    pub commits: Vec<Commit>,
}

impl Calls {
    pub fn is_empty(&self) -> bool {
        self.lists.is_empty() && self.reads.is_empty() && self.commits.is_empty()
    }

    /// The keys of every read call.
    pub fn read_keys(&self) -> Vec<Vec<String>> {
        self.reads
            .iter()
            .map(|read| read.iter().map(|(key, _)| key.clone()).collect())
            .collect()
    }
}

type Hook = Box<dyn FnOnce(&MemoryStorePort) + Send>;

/// A memory store port that records every call with what came back.
/// `before_first_read` and `before_first_commit` each run a hook once, just
/// before the first read or commit reaches the store; `unavailable` makes
/// every call a temporary failure; `never_still` lets another writer commit
/// just before every read.
#[derive(Default)]
pub struct Recorder {
    pub inner: MemoryStorePort,
    calls: Mutex<Calls>,
    before_read: Mutex<Option<Hook>>,
    before_commit: Mutex<Option<Hook>>,
    unavailable: bool,
    never_still: bool,
}

impl Recorder {
    pub fn new(inner: MemoryStorePort) -> Self {
        Recorder {
            inner,
            ..Default::default()
        }
    }

    /// A recorder whose every call is a temporary failure.
    pub fn unavailable() -> Self {
        Recorder {
            unavailable: true,
            ..Default::default()
        }
    }

    /// A recorder whose scope version moves just before every read: another
    /// writer commits one event there.
    pub fn never_still() -> Self {
        Recorder {
            never_still: true,
            ..Default::default()
        }
    }

    pub fn before_first_read(self, hook: impl FnOnce(&MemoryStorePort) + Send + 'static) -> Self {
        *self.before_read.lock().unwrap() = Some(Box::new(hook));
        self
    }

    pub fn before_first_commit(self, hook: impl FnOnce(&MemoryStorePort) + Send + 'static) -> Self {
        *self.before_commit.lock().unwrap() = Some(Box::new(hook));
        self
    }

    /// The calls so far.
    pub fn calls(&self) -> Calls {
        self.calls.lock().unwrap().clone()
    }

    /// The calls so far, which are then forgotten.
    pub fn take(&self) -> Calls {
        std::mem::take(&mut *self.calls.lock().unwrap())
    }

    fn fail(&self) -> Result<(), PortError> {
        if self.unavailable {
            return Err(PortError::Unavailable {
                message: "connection reset".to_string(),
            });
        }
        Ok(())
    }
}

impl StorePort for Recorder {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.calls.lock().unwrap().lists.push(prefix.as_str().to_string());
        self.fail()?;
        self.inner.list(scope, prefix)
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.fail()?;
        if let Some(hook) = self.before_read.lock().unwrap().take() {
            hook(&self.inner);
        }
        if self.never_still {
            let event = Event::new("board.viewed", serde_json::json!({})).unwrap();
            let other = PortActor::new("u_7", "Bob").unwrap();
            let commit = Commit::new(other, "other-writer", Timestamp::new(1).unwrap(), Vec::new(), vec![event]).unwrap();
            self.inner.commit(scope, &commit)?;
        }
        let docs = self.inner.read(scope, keys)?;
        let call = keys
            .iter()
            .zip(&docs)
            .map(|(key, doc)| (key.as_str().to_string(), doc.as_ref().map(|d| d.content.len())))
            .collect();
        self.calls.lock().unwrap().reads.push(call);
        Ok(docs)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        self.calls.lock().unwrap().commits.push(commit.clone());
        self.fail()?;
        if let Some(hook) = self.before_commit.lock().unwrap().take() {
            hook(&self.inner);
        }
        self.inner.commit(scope, commit)
    }
}

// --- one command of every kind ---

/// One command of every kind. The match below has no catch-all arm, so a
/// new kind does not compile until it is listed here too.
pub fn every_command() -> Vec<Command> {
    fn listed(cmd: &Command) {
        match cmd {
            Command::List { .. }
            | Command::Show { .. }
            | Command::Status { .. }
            | Command::Instructions { .. }
            | Command::Validate { .. }
            | Command::Analyze { .. }
            | Command::Trace { .. }
            | Command::Plan { .. }
            | Command::PlanStrict
            | Command::ArtifactCat { .. }
            | Command::EvidenceShow { .. }
            | Command::LanguageShow
            | Command::DiscussList { .. }
            | Command::DiscussShow { .. }
            | Command::DiscussSearch { .. }
            | Command::NewChange { .. }
            | Command::NewArtifact { .. }
            | Command::TaskDone { .. }
            | Command::TaskUndone { .. }
            | Command::TaskMove { .. }
            | Command::Claim { .. }
            | Command::ClaimRelease { .. }
            | Command::InProgressAdd { .. }
            | Command::InProgressRemove { .. }
            | Command::ChangeDepends { .. }
            | Command::ChangeSchema { .. }
            | Command::ChangeRank { .. }
            | Command::ChangeLanded { .. }
            | Command::ChangeFieldsEdit { .. }
            | Command::DiscussFieldsEdit { .. }
            | Command::Archive { .. }
            | Command::Discard { .. }
            | Command::DiscussNew { .. }
            | Command::DiscussContext { .. }
            | Command::DiscussAddRound { .. }
            | Command::DiscussConclude { .. }
            | Command::DiscussPromote { .. }
            | Command::DiscussLink { .. }
            | Command::DiscussSeal { .. }
            | Command::DiscussArchive { .. }
            | Command::DiscussDiscard { .. }
            | Command::ReviewAddRound { .. }
            | Command::ReviewShow { .. }
            | Command::ReviewStamp { .. }
            | Command::ReviewDiscard { .. }
            | Command::VerifyAddRound { .. }
            | Command::VerifyShow { .. }
            | Command::VerifyStamp { .. }
            | Command::VerifyDiscard { .. }
            | Command::WorkflowConfigShow
            | Command::WorkflowConfigEdit { .. }
            | Command::WorkflowConfigReplace { .. } => {}
        }
    }
    let s = |text: &str| text.to_string();
    let list = |specs| Command::List { sort: s("name"), specs, changes: false, worktrees: Default::default(), detail: false };
    let commands = vec![
        list(false),
        list(true),
        Command::Show { item: Some(s("demo")), item_type: None },
        Command::Status { change: Some(s("demo")), schema: None },
        Command::Instructions { artifact: Some(s("apply")), change: Some(s("demo")), schema: None },
        Command::Validate { item: Some(s("demo")), all: false, changes: false, specs: false, strict: false },
        Command::Analyze { change: Some(s("demo")) },
        Command::Trace { capability: s("auth") },
        Command::Plan { ranks: None },
        Command::PlanStrict,
        Command::ArtifactCat { artifact: s("proposal"), change: Some(s("demo")) },
        Command::EvidenceShow { change: s("demo") },
        Command::LanguageShow,
        Command::DiscussList { archived: false },
        Command::DiscussShow { slug: s("talk") },
        Command::DiscussSearch { terms: vec![s("auth")] },
        Command::NewChange { name: s("demo"), description: None, schema: None, agent: None, from_discussion: None, last: false, fields: Default::default() },
        Command::NewArtifact {
            kind: s("proposal"),
            capability: None,
            change: Some(s("demo")),
            content: Some(s("## Why\n")),
            force: true,
            new_capability: true,
        },
        Command::TaskDone { task_id: s("1"), change: Some(s("demo")), touched_files: None, head_commit: None },
        Command::TaskUndone { task_id: s("1"), change: Some(s("demo")) },
        Command::TaskMove { change: s("demo"), from: 1, to: 2, before: None },
        Command::Claim { name: s("demo") },
        Command::ClaimRelease { name: s("demo") },
        Command::InProgressAdd { name: s("demo"), branch: None },
        Command::InProgressRemove { name: s("demo") },
        Command::ChangeDepends { name: s("demo"), on: vec![s("base")], remove: false },
        Command::ChangeSchema { name: s("demo"), schema: s("spec-driven") },
        Command::ChangeRank {
            name: s("demo"),
            anchor: s("base"),
            position: speclink_core::plan::Side::parse("before").unwrap(),
            force: false,
        },
        Command::ChangeLanded { name: s("demo"), commit: s("0123456789ab"), branch: None },
        Command::ChangeFieldsEdit { name: s("demo"), set: [(s("ticket"), s("PROJ-1"))].into(), unset: vec![] },
        Command::DiscussFieldsEdit { slug: s("talk"), set: [(s("owner"), s("alice"))].into(), unset: vec![] },
        Command::Archive {
            change: Some(s("demo")),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
        Command::Discard { change: s("demo"), force: false },
        Command::DiscussNew { topic: s("Talk"), slug: None, kind: None, fields: Default::default() },
        Command::DiscussContext { slug: s("talk"), content: s("context") },
        Command::DiscussAddRound { slug: s("talk"), mode: s("round"), content: s("text") },
        Command::DiscussConclude { slug: s("talk"), content: s("done"), hold: false },
        Command::DiscussPromote { slug: s("talk"), name: Some(s("demo")), last: false, fields: Default::default() },
        Command::DiscussLink { slug: s("talk"), change: s("demo") },
        Command::DiscussSeal { slug: s("talk"), change: s("demo"), last: false },
        Command::DiscussArchive { slug: s("talk") },
        Command::DiscussDiscard { slug: s("talk"), force: false },
        Command::ReviewAddRound { change: s("demo"), content: s("round") },
        Command::ReviewShow { change: s("demo") },
        Command::ReviewStamp { change: s("demo"), accept: false, tool: None, head_commit: None, scope: Vec::new(), missing: Vec::new() },
        Command::ReviewDiscard { change: s("demo") },
        Command::VerifyAddRound { change: s("demo"), content: s("round") },
        Command::VerifyShow { change: s("demo") },
        Command::VerifyStamp { change: s("demo"), accept: false, tool: None, head_commit: None, scope: Vec::new(), missing: Vec::new() },
        Command::VerifyDiscard { change: s("demo") },
        Command::WorkflowConfigShow,
        Command::WorkflowConfigEdit {
            edit: WorkflowConfigEdit::Policy { key: s("tdd"), value: s("true") },
            dry_run: false,
        },
        Command::WorkflowConfigReplace { content: s("schema: spec-driven\ntdd: false\n") },
    ];
    commands.iter().for_each(listed);
    commands
}

// --- what a before-command hook reads ---

pub fn show(item: &str) -> Command {
    Command::Show { item: Some(item.to_string()), item_type: None }
}

/// What the reader shows of change `name`.
pub fn shown(reader: &Reader<'_>, name: &str) -> Result<ShowChange, ExecError> {
    match reader.read(show(name))?.outcome {
        CommandOutcome::Show(ShowOutcome::Change(change)) => Ok(change),
        other => panic!("expected a shown change, got {other:?}"),
    }
}

// --- this project's own spec folder ---

/// The key of a file under `speclink/`, from its path segments. The archive
/// moves to the top-level `archive/` (the key naming rules); a file that is
/// no speclink document (the manual, or a stray file in a folder of
/// documents) has no key.
fn key_of(segments: &[String]) -> Option<String> {
    let parts: Vec<&str> = segments.iter().map(String::as_str).collect();
    let markdown = |file: &str| file.ends_with(".md");
    match parts.as_slice() {
        ["changes", "archive", dated, rest @ ..] if !rest.is_empty() => {
            Some(format!("archive/changes/{dated}/{}", rest.join("/")))
        }
        ["discussions", "archive", file] if markdown(file) => Some(format!("archive/discussions/{file}")),
        // A file right in an archive folder is no archived record.
        ["changes" | "discussions", "archive", ..] => None,
        ["changes", _, _, ..] | ["specs", _, "spec.md"] => Some(parts.join("/")),
        ["discussions", file] if markdown(file) => Some(parts.join("/")),
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

/// This project's own `speclink/` folder as documents: each file that is a
/// speclink document, with its key.
pub fn spec_folder_docs() -> Vec<(DocKey, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("speclink");
    let mut files = Vec::new();
    walk(&root, &mut Vec::new(), &mut files);
    let mut docs = Vec::new();
    for (segments, path) in files {
        let Some(key) = key_of(&segments) else {
            continue;
        };
        // A file that is not text, or a name the key format rejects, is not
        // a speclink document.
        let (Ok(content), Ok(key)) = (std::fs::read_to_string(&path), DocKey::new(key)) else {
            continue;
        };
        docs.push((key, content));
    }
    assert!(docs.len() > 100, "the spec folder was found at {}", root.display());
    docs
}

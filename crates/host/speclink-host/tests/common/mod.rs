//! Shared by the integration tests that run the Engine on a store port.

// Each test target uses its own part of this module.
#![allow(dead_code)]

use speclink_core::config::ResolvedPolicy;
use speclink_host::binding::local_default_binding;
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_store_port::{
    Actor as PortActor, Commit, DocKey, Document, Expected, KeyPrefix, Listing, MemoryStorePort,
    Op, PortError, ScopeId, StorePort, Timestamp, Version,
};
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
/// every call a temporary failure.
#[derive(Default)]
pub struct Recorder {
    pub inner: MemoryStorePort,
    calls: Mutex<Calls>,
    before_read: Mutex<Option<Hook>>,
    before_commit: Mutex<Option<Hook>>,
    unavailable: bool,
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

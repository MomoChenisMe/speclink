//! Shared by the integration tests that run the Engine on a store port.

// Each test target uses its own part of this module.
#![allow(dead_code)]

use speclink_core::config::ResolvedPolicy;
use speclink_host::binding::local_default_binding;
use speclink_host::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
use speclink_host::policy::EffectiveWorkflowPolicy;
use speclink_store_port::{
    Actor as PortActor, Commit, DocKey, Expected, KeyPrefix, Op, ScopeId, StorePort, Timestamp,
};

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

/// Write `docs` into `port` in one commit. Each document must not exist yet.
pub fn seed(port: &dyn StorePort, docs: Vec<(DocKey, String)>) {
    let ops = docs
        .into_iter()
        .map(|(key, content)| Op::Put {
            key,
            content,
            expected: Expected::Absent,
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
    port.commit(&scope(), &commit).expect("seed commit");
}

pub fn scope_version(port: &dyn StorePort) -> u64 {
    port.list(&scope(), &KeyPrefix::new("").unwrap())
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

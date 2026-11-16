//! Fill the store of an in-process server before a test starts it.

use speclink_store_port::{Actor, Commit, DocKey, Expected, Op, ScopeId, StorePort, Timestamp};

/// Write `docs` (store-port key, content) into `demo/backend` in one commit.
pub fn seed(store: &dyn StorePort, docs: &[(&str, &str)]) {
    let ops = docs
        .iter()
        .map(|(key, content)| Op::Put {
            key: DocKey::new(*key).expect("a document key"),
            content: content.to_string(),
            expected: Expected::Absent,
        })
        .collect();
    let commit = Commit::new(
        Actor::new("seed", "seed").expect("an actor"),
        "seed",
        Timestamp::new(0).expect("a time"),
        ops,
        Vec::new(),
    )
    .expect("a commit");
    store.commit(&ScopeId::new("demo/backend").expect("a scope id"), &commit).expect("seed commit");
}

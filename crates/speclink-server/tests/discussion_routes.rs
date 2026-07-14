//! Discussion routes over the same bridge and commit path (reference-server
//! spec). Promoting a discussion returns the new change name and lands both a
//! discussion-promoted and a change-created event in the scope outbox.

mod common;

use speclink_remote::client::Client;
use speclink_store::memory::MemoryStore;
use speclink_store::{OutboxCursor, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn client(base: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        "secret",
        Some("backend"),
    )
}

#[test]
fn create_and_show_round_trip_a_discussion() {
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let base = common::start(common::state_with(store));
    let client = client(&base);

    let created = client.new_discussion("Rate limiting").expect("create discussion");
    assert!(!created.slug.is_empty(), "a slug is derived from the topic");

    let shown = client.show_discussion(&created.slug).expect("show discussion");
    assert_eq!(shown.info.slug, created.slug);
    assert_eq!(shown.info.topic, "Rate limiting");

    let listed = client.list_discussions(false).expect("list discussions");
    assert!(listed.discussions.iter().any(|d| d.slug == created.slug), "the discussion is listed");
}

#[test]
fn promote_returns_the_change_and_lands_both_events() {
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let base = common::start(common::state_with(store.clone()));
    let client = client(&base);

    let created = client.new_discussion("Auth scope").expect("create discussion");
    let promoted = client
        .discussion_promote(&created.slug, None)
        .expect("promote discussion");
    assert!(!promoted.change.is_empty(), "promote returns the new change name");

    let entries = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox");
    let kinds: Vec<&str> = entries.iter().map(|e| e.record.name.as_str()).collect();
    assert!(
        kinds.contains(&"discussion-promoted"),
        "a discussion-promoted event landed: {kinds:?}"
    );
    assert!(
        kinds.contains(&"change-created"),
        "a change-created event landed in the same promote: {kinds:?}"
    );
}

//! What a commit carries besides its documents: the actor name and the
//! events. Every commit path of the Host uses these two mappings, so a store
//! sees the same actor string and the same event payload whichever store it
//! is.

use crate::context::Actor;
use speclink_core::command::DomainEvent;

/// The actor string recorded in store history and event records. A store
/// needs a non-empty actor, so anonymity maps to the stable "anonymous"
/// placeholder instead of an unattributable empty string.
pub(crate) fn actor_name(actor: &Actor) -> String {
    actor.display().unwrap_or("anonymous").to_string()
}

/// The camelCase subject payload of a domain event. With the event's stable
/// kind string it is all a store keeps of the event; no store type is
/// involved, so every commit path shares this one mapping.
pub fn event_payload(event: &DomainEvent) -> serde_json::Value {
    use serde_json::json;
    match event {
        DomainEvent::ChangeCreated { change, .. } => json!({ "change": change }),
        DomainEvent::ArtifactCreated { change, artifact, .. } => json!({ "change": change, "artifact": artifact }),
        DomainEvent::TaskCompleted { change, task_id, touched_files, .. } => {
            // Additive: consumers that never read touchedFiles keep working,
            // and an empty list is the honest answer when nothing was
            // attributable — never a guess.
            json!({ "change": change, "taskId": task_id, "touchedFiles": touched_files })
        }
        DomainEvent::TaskUncompleted { change, task_id, .. } => json!({ "change": change, "taskId": task_id }),
        DomainEvent::TaskMoved { change, .. } => json!({ "change": change }),
        DomainEvent::ChangeClaimed { change, .. } => json!({ "change": change }),
        DomainEvent::ChangeMarkedInProgress { change, .. } => json!({ "change": change }),
        DomainEvent::ChangeInProgressRemoved { change, .. } => json!({ "change": change }),
        DomainEvent::ChangeDependsChanged { change, depends_on, .. } => json!({ "change": change, "dependsOn": depends_on }),
        DomainEvent::ChangeSchemaChanged { change, schema, .. } => json!({ "change": change, "schema": schema }),
        DomainEvent::ChangeArchived { change, dated_name, .. } => json!({ "change": change, "datedName": dated_name }),
        DomainEvent::ChangeDiscarded { change, .. } => json!({ "change": change }),
        DomainEvent::DiscussionCreated { slug, .. } => json!({ "slug": slug }),
        DomainEvent::DiscussionContextSet { slug, .. } => json!({ "slug": slug }),
        DomainEvent::DiscussionRoundAdded { slug, round, .. } => json!({ "slug": slug, "round": round }),
        DomainEvent::DiscussionConcluded { slug, .. } => json!({ "slug": slug }),
        DomainEvent::DiscussionPromoted { slug, change, .. } => json!({ "slug": slug, "change": change }),
        DomainEvent::DiscussionLinked { slug, change, .. } => json!({ "slug": slug, "change": change }),
        DomainEvent::DiscussionSealed { slug, change, .. } => json!({ "slug": slug, "change": change }),
        DomainEvent::DiscussionArchived { slug, .. } => json!({ "slug": slug }),
        DomainEvent::DiscussionDiscarded { slug, .. } => json!({ "slug": slug }),
        DomainEvent::ReviewRoundAdded { change, round, .. } => json!({ "change": change, "round": round }),
        DomainEvent::ReviewStamped { change, .. } => json!({ "change": change }),
        DomainEvent::ReviewDiscarded { change, .. } => json!({ "change": change }),
        DomainEvent::VerifyRoundAdded { change, round, .. } => json!({ "change": change, "round": round }),
        DomainEvent::VerifyStamped { change, .. } => json!({ "change": change }),
        DomainEvent::VerifyDiscarded { change, .. } => json!({ "change": change }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ActorSource;
    use serde_json::json;

    fn at() -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp_millis(1_800_000_000_000).unwrap()
    }

    #[test]
    fn payloads_are_camel_case_and_leave_the_event_time_out() {
        let rows = [
            (
                DomainEvent::TaskCompleted {
                    change: "add-x".into(),
                    task_id: "tsk_1".into(),
                    touched_files: vec!["src/a.rs".into()],
                    occurred_at: at(),
                },
                json!({ "change": "add-x", "taskId": "tsk_1", "touchedFiles": ["src/a.rs"] }),
            ),
            (
                DomainEvent::ChangeDependsChanged {
                    change: "add-x".into(),
                    depends_on: vec!["add-w".into()],
                    occurred_at: at(),
                },
                json!({ "change": "add-x", "dependsOn": ["add-w"] }),
            ),
            (
                DomainEvent::ChangeArchived {
                    change: "add-x".into(),
                    dated_name: "2026-11-10-add-x".into(),
                    occurred_at: at(),
                },
                json!({ "change": "add-x", "datedName": "2026-11-10-add-x" }),
            ),
            (
                DomainEvent::DiscussionRoundAdded { slug: "talk".into(), round: 2, occurred_at: at() },
                json!({ "slug": "talk", "round": 2 }),
            ),
        ];
        for (event, payload) in rows {
            assert_eq!(event_payload(&event), payload, "{}", event.kind());
        }
    }

    #[test]
    fn an_anonymous_actor_has_the_stable_placeholder_name() {
        assert_eq!(actor_name(&Actor::Anonymous), "anonymous");
        let alice = Actor::Identified {
            display: "Alice <alice@example.com>".to_string(),
            source: ActorSource::GitConfig,
        };
        assert_eq!(actor_name(&alice), "Alice <alice@example.com>");
    }
}

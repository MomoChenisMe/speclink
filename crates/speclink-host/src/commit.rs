//! The Host's TeamStore composition path: open a unit of work on behalf of
//! an ExecutionContext, map domain events one-way into store event records,
//! and commit both atomically (documents + outbox). This is the §3.7
//! UoW/event-commit responsibility of the Host, proven against the
//! in-memory reference — no existing CLI flow is wired through it (the
//! local mode keeps the current Store seam until the server work lands).

use crate::context::{Actor, SpeclinkExecutionContext};
use speclink_core::command::DomainEvent;
use speclink_store::{
    CommandContext, EventRecord, Revision, Scope, StoreError, TeamStore, UnitOfWork,
};

/// The actor string recorded in store history and event records. The store
/// contract requires a non-empty actor, so anonymity maps to the stable
/// "anonymous" placeholder instead of an unattributable empty string.
fn actor_name(actor: &Actor) -> String {
    actor.display().unwrap_or("anonymous").to_string()
}

/// Open a unit of work in the context's scope on behalf of its actor.
pub fn begin_unit_of_work(
    store: &dyn TeamStore,
    ctx: &SpeclinkExecutionContext,
    command: &str,
) -> Result<UnitOfWork, StoreError> {
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    store.begin_unit_of_work(
        &scope,
        CommandContext {
            command: command.to_string(),
            actor: actor_name(&ctx.actor),
        },
    )
}

/// Commit a unit of work together with its domain events: documents and
/// outbox records land atomically. A TeamStore failure — including
/// revision_conflict with its expected/actual detail — passes through
/// verbatim as the Host error.
pub fn commit_with_events(
    store: &dyn TeamStore,
    ctx: &SpeclinkExecutionContext,
    uow: UnitOfWork,
    events: &[DomainEvent],
) -> Result<Revision, StoreError> {
    let records = events.iter().map(|e| event_record_of(e, &ctx.actor)).collect();
    store.commit(uow, records)
}

/// The one-way mapping: core typed event → store event record. Canonical
/// event semantics stay in the engine; the record carries the stable kind
/// string, the camelCase subject payload, the acting identity, and the
/// event's own timestamp. Nothing maps back.
pub fn event_record_of(event: &DomainEvent, actor: &Actor) -> EventRecord {
    use serde_json::json;
    let (payload, at) = match event {
        DomainEvent::ChangeCreated { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ArtifactCreated { change, artifact, occurred_at } => {
            (json!({ "change": change, "artifact": artifact }), *occurred_at)
        }
        DomainEvent::TaskCompleted { change, task_id, occurred_at } => {
            (json!({ "change": change, "taskId": task_id }), *occurred_at)
        }
        DomainEvent::TaskUncompleted { change, task_id, occurred_at } => {
            (json!({ "change": change, "taskId": task_id }), *occurred_at)
        }
        DomainEvent::TaskMoved { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ChangeClaimed { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ChangeMarkedInProgress { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ChangeInProgressRemoved { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ChangeArchived { change, dated_name, occurred_at } => {
            (json!({ "change": change, "datedName": dated_name }), *occurred_at)
        }
        DomainEvent::ChangeDiscarded { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::DiscussionCreated { slug, occurred_at } => {
            (json!({ "slug": slug }), *occurred_at)
        }
        DomainEvent::DiscussionContextSet { slug, occurred_at } => {
            (json!({ "slug": slug }), *occurred_at)
        }
        DomainEvent::DiscussionRoundAdded { slug, round, occurred_at } => {
            (json!({ "slug": slug, "round": round }), *occurred_at)
        }
        DomainEvent::DiscussionConcluded { slug, occurred_at } => {
            (json!({ "slug": slug }), *occurred_at)
        }
        DomainEvent::DiscussionPromoted { slug, change, occurred_at } => {
            (json!({ "slug": slug, "change": change }), *occurred_at)
        }
        DomainEvent::DiscussionLinked { slug, change, occurred_at } => {
            (json!({ "slug": slug, "change": change }), *occurred_at)
        }
        DomainEvent::DiscussionSealed { slug, change, occurred_at } => {
            (json!({ "slug": slug, "change": change }), *occurred_at)
        }
        DomainEvent::DiscussionArchived { slug, occurred_at } => {
            (json!({ "slug": slug }), *occurred_at)
        }
        DomainEvent::DiscussionDiscarded { slug, occurred_at } => {
            (json!({ "slug": slug }), *occurred_at)
        }
        DomainEvent::ReviewRoundAdded { change, round, occurred_at } => {
            (json!({ "change": change, "round": round }), *occurred_at)
        }
        DomainEvent::ReviewStamped { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::ReviewDiscarded { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::VerifyRoundAdded { change, round, occurred_at } => {
            (json!({ "change": change, "round": round }), *occurred_at)
        }
        DomainEvent::VerifyStamped { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
        DomainEvent::VerifyDiscarded { change, occurred_at } => {
            (json!({ "change": change }), *occurred_at)
        }
    };
    EventRecord {
        name: event.kind().to_string(),
        payload,
        actor: actor_name(actor),
        at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::local_default_binding;
    use crate::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
    use crate::policy::EffectiveWorkflowPolicy;
    use speclink_core::command::DomainEvent;
    use speclink_core::config::ResolvedPolicy;
    use speclink_store::memory::MemoryStore;
    use speclink_store::{DocumentId, ExpectedRevision, OutboxCursor, Scope, StoreError, TeamStore};

    fn test_context() -> SpeclinkExecutionContext {
        let binding = local_default_binding();
        SpeclinkExecutionContext {
            actor: Actor::Identified {
                display: "Alice <alice@example.com>".to_string(),
                source: ActorSource::GitConfig,
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

    fn scope_of(ctx: &SpeclinkExecutionContext) -> Scope {
        Scope::new(ctx.project.clone(), ctx.repo.clone())
    }

    // --- Host 承擔 TeamStore 的 UoW 與 event commit ---

    #[test]
    fn commit_lands_document_and_event_atomically() {
        // 以 ExecutionContext 開 UoW、寫一份文件帶一筆領域事件 commit：
        // 自 cursor 0 重讀 outbox 得恰一筆含 actor 與事件名的 record；
        // 文件與事件同 commit 可見。
        let store = MemoryStore::new();
        let ctx = test_context();

        let mut uow = begin_unit_of_work(&store, &ctx, "new-change").expect("uow opens");
        uow.create(
            DocumentId::ChangeMeta { change: "add-auth".into() },
            "schema: spec-driven\n",
        );
        let event = DomainEvent::ChangeCreated {
            change: "add-auth".into(),
            occurred_at: chrono::Utc::now(),
        };
        let revision =
            commit_with_events(&store, &ctx, uow, &[event]).expect("commit succeeds");

        let entries = store
            .read_outbox(&scope_of(&ctx), OutboxCursor(0))
            .expect("outbox reads");
        assert_eq!(entries.len(), 1, "exactly one event record landed");
        assert_eq!(entries[0].record.name, "change-created");
        assert_eq!(entries[0].record.actor, "Alice <alice@example.com>");
        assert_eq!(entries[0].revision, revision, "the event rides its commit");

        let snapshot = store.snapshot(&scope_of(&ctx)).expect("snapshot");
        let doc = snapshot
            .read(&DocumentId::ChangeMeta { change: "add-auth".into() })
            .expect("read")
            .expect("document visible in the same commit");
        assert_eq!(doc.revision, revision, "document and event share the commit revision");
    }

    #[test]
    fn revision_conflict_passes_through_verbatim() {
        // 兩個 Host commit 以相同 expected revision 競寫同一文件：
        // 敗方錯誤保留 revision_conflict 分類與 expected/actual 詳情。
        let store = MemoryStore::new();
        let ctx = test_context();
        let doc = DocumentId::ChangeMeta { change: "contested".into() };

        let mut seed = begin_unit_of_work(&store, &ctx, "new-change").expect("uow opens");
        seed.create(doc.clone(), "v1");
        let base = commit_with_events(&store, &ctx, seed, &[]).expect("seed commit");

        let mut winner = begin_unit_of_work(&store, &ctx, "edit").expect("uow opens");
        winner.update(doc.clone(), "v2 — winner", base);
        let after = commit_with_events(&store, &ctx, winner, &[]).expect("winner commits");

        let mut loser = begin_unit_of_work(&store, &ctx, "edit").expect("uow opens");
        loser.update(doc.clone(), "v2 — loser", base);
        let err = commit_with_events(&store, &ctx, loser, &[])
            .expect_err("stale expected revision must conflict");
        match err {
            StoreError::RevisionConflict { doc: conflicted, expected, actual } => {
                assert_eq!(conflicted.doc, doc);
                assert_eq!(expected, ExpectedRevision::At(base));
                assert_eq!(actual, Some(after), "actual names the winner's revision");
            }
            other => panic!("expected a verbatim revision_conflict, got {other:?}"),
        }
    }

    #[test]
    fn anonymous_actor_maps_to_the_stable_placeholder() {
        // EventRecord.actor 是必填字串：匿名映射為固定 "anonymous"，
        // 不得為空字串（空值下游無法歸因）。
        let store = MemoryStore::new();
        let mut ctx = test_context();
        ctx.actor = Actor::Anonymous;

        let mut uow = begin_unit_of_work(&store, &ctx, "new-change").expect("uow opens");
        uow.create(DocumentId::ChangeMeta { change: "anon".into() }, "schema: spec-driven\n");
        let event = DomainEvent::ChangeCreated {
            change: "anon".into(),
            occurred_at: chrono::Utc::now(),
        };
        commit_with_events(&store, &ctx, uow, &[event]).expect("commit succeeds");

        let entries = store.read_outbox(&scope_of(&ctx), OutboxCursor(0)).expect("outbox");
        assert_eq!(entries[0].record.actor, "anonymous");
    }
}

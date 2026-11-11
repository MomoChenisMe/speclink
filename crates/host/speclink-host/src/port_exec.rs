//! Run one Engine command on a store port (design D2).
//!
//! [`execute`] is the only way from a store port to the Engine. The store
//! view it runs the Engine on is not part of the public interface: a view
//! keeps its writes in memory, and only this entry commits them.
//!
//! ```compile_fail
//! use speclink_host::port_view::PortView;
//! ```

use crate::commit::{actor_name, event_payload};
use crate::context::SpeclinkExecutionContext;
use crate::port_view::PortView;
use speclink_core::command::{
    execute as engine_execute, Command, CommandError, CommandOutcome, DomainEvent, ExecutionContext,
};
use speclink_store_port::{
    Actor, Commit, Event, InvalidValue, Op, PortError, ScopeId, StorePort, Timestamp, Version,
};

/// Why an execution on a store port failed. The two vocabularies stay
/// apart: the Engine command layer's typed codes, or the store's failures.
#[derive(Debug)]
pub enum ExecError {
    /// The Engine command layer refused before or during the flow.
    Command(CommandError),
    /// The store reported a failure, during a read or at the commit.
    Store(PortError),
}

/// The result of one command on a store port: the typed outcome, the domain
/// events, and the new scope version when the command committed (`None` for
/// a query and for a command that wrote nothing).
#[derive(Debug)]
pub struct PortExecution {
    pub outcome: CommandOutcome,
    pub events: Vec<DomainEvent>,
    pub scope_version: Option<Version>,
}

/// The Engine execution context for a Host-resolved context. Identity comes
/// from the Host; there is no local workspace or user schema directory in
/// this mode (built-in schemas resolve without one), and env overrides do
/// not apply: the Host already resolved the policy.
pub(crate) fn engine_context(ctx: &SpeclinkExecutionContext) -> ExecutionContext {
    ExecutionContext {
        actor: ctx.actor.display().map(str::to_string),
        repo: Some(ctx.repo.as_str().to_string()),
        ..Default::default()
    }
}

/// A short, stable command label for the commit's audit record. Derived from
/// the outcome, so it names the verb that produced the writes.
pub(crate) fn command_label(outcome: &CommandOutcome) -> String {
    match outcome {
        CommandOutcome::NewChange(_) => "new-change",
        CommandOutcome::NewArtifact(_) => "new-artifact",
        CommandOutcome::TaskDone(_) => "task-done",
        CommandOutcome::TaskUndone(_) => "task-undone",
        CommandOutcome::Claim(_) => "claim",
        CommandOutcome::InProgressAdd(_) => "in-progress-add",
        CommandOutcome::InProgressRemove(_) => "in-progress-remove",
        CommandOutcome::ChangeDepends(_) => "change-depends",
        CommandOutcome::ChangeSchema(_) => "change-schema",
        CommandOutcome::Archive(_) => "archive",
        CommandOutcome::Discard(_) => "discard",
        CommandOutcome::DiscussNew(_) => "discuss-new",
        CommandOutcome::DiscussContext(_) => "discuss-context",
        CommandOutcome::DiscussAddRound(_) => "discuss-add-round",
        CommandOutcome::DiscussConclude(_) => "discuss-conclude",
        CommandOutcome::DiscussPromote(_) => "discuss-promote",
        CommandOutcome::DiscussLink(_) => "discuss-link",
        CommandOutcome::DiscussSeal(_) => "discuss-seal",
        CommandOutcome::DiscussArchive(_) => "discuss-archive",
        CommandOutcome::DiscussDiscard(_) => "discuss-discard",
        CommandOutcome::ReviewAddRound(_) => "review-add-round",
        CommandOutcome::ReviewStamp(_) => "review-stamp",
        CommandOutcome::ReviewDiscard(_) => "review-discard",
        _ => "command",
    }
    .to_string()
}

/// Run one Engine command on `scope` of `port`.
///
/// The command reads only the documents it uses. Its writes, its deletes and
/// its domain events go to the store in exactly one commit; a query, and a
/// command that wrote nothing and produced no event, does not commit.
///
/// `scope` alone selects the documents. `ctx` gives who acts; its project and
/// repo are not used to address the store.
///
/// After a failure the store is unchanged:
/// - the Engine refused: [`ExecError::Command`], and the staged writes are
///   dropped;
/// - the store failed during a read: [`ExecError::Store`], whatever the
///   Engine made of the missing document, and nothing is committed;
/// - the store rejected the commit (another writer changed a document this
///   command saw, or the store failed): [`ExecError::Store`]. The store
///   contract applies a commit whole or not at all.
pub fn execute(
    port: &dyn StorePort,
    scope: &ScopeId,
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
) -> Result<PortExecution, ExecError> {
    // The Engine does not read the clock: the Host gives the commit time.
    let millis = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let at = Timestamp::new(millis).map_err(invalid)?;
    execute_at(port, scope, ctx, cmd, at)
}

/// [`execute`] with the commit time given by the caller.
pub(crate) fn execute_at(
    port: &dyn StorePort,
    scope: &ScopeId,
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
    at: Timestamp,
) -> Result<PortExecution, ExecError> {
    let view = PortView::new(port, scope.clone());
    let result = engine_execute(&view, &engine_context(ctx), cmd);
    // A store failure during a read made the Engine see "no document": its
    // result means nothing then, so the store failure is checked first.
    let ops = view.into_ops().map_err(ExecError::Store)?;
    let (outcome, events) = result.map_err(ExecError::Command)?;
    if ops.is_empty() && events.is_empty() {
        return Ok(PortExecution {
            outcome,
            events,
            scope_version: None,
        });
    }
    let commit = commit_of(ctx, &outcome, &events, ops, at).map_err(invalid)?;
    let version = port.commit(scope, &commit).map_err(ExecError::Store)?;
    Ok(PortExecution {
        outcome,
        events,
        scope_version: Some(version),
    })
}

/// The one commit of a command: its writes in key order, its events in the
/// order the Engine produced them.
fn commit_of(
    ctx: &SpeclinkExecutionContext,
    outcome: &CommandOutcome,
    events: &[DomainEvent],
    ops: Vec<Op>,
    at: Timestamp,
) -> Result<Commit, InvalidValue> {
    // The execution context has a display name only. It is the actor id too,
    // until an entry point that carries a stable identity exists.
    let name = actor_name(&ctx.actor);
    let actor = Actor::new(name.clone(), name)?;
    let events = events
        .iter()
        .map(|event| Event::new(event.kind(), event_payload(event)))
        .collect::<Result<Vec<Event>, InvalidValue>>()?;
    Commit::new(actor, command_label(outcome), at, ops, events)
}

/// A value the store contract rejects never reaches the store; the caller
/// sees a backend failure that names the rule.
fn invalid(e: InvalidValue) -> ExecError {
    ExecError::Store(PortError::Backend {
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::local_default_binding;
    use crate::context::{Actor, ActorSource, ExecutionMode};
    use crate::policy::EffectiveWorkflowPolicy;
    use crate::port_view::testing::{key, overwrite, scope, seed, Probe};
    use speclink_core::command::ErrorCode;
    use speclink_core::config::ResolvedPolicy;
    use speclink_store_port::{Expected, KeyPrefix, MemoryStorePort, Op, Timestamp};

    const ACTOR: &str = "Alice <alice@example.com>";
    const AT: u64 = 1_800_000_000_000;
    const META: &str = "schema: spec-driven\n";
    const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";
    const DELTA: &str = "## Purpose\n\n本 capability 負責身分驗證的簽發與撤銷，涵蓋權杖生命週期各階段的可觀察行為、失敗處置與稽核紀錄。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

    fn ctx() -> SpeclinkExecutionContext {
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

    /// One change that can be worked on and archived.
    fn seeded() -> MemoryStorePort {
        let port = MemoryStorePort::new();
        seed(
            &port,
            1_000,
            &[
                ("changes/demo/.meta.yaml", META),
                ("changes/demo/tasks.md", TASKS),
                ("changes/demo/specs/auth/spec.md", DELTA),
            ],
        );
        port
    }

    fn at() -> Timestamp {
        Timestamp::new(AT).unwrap()
    }

    fn version(port: &dyn StorePort) -> u64 {
        port.list(&scope(), &KeyPrefix::new("").unwrap())
            .unwrap()
            .version
            .get()
    }

    fn keys_under(port: &dyn StorePort, prefix: &str) -> Vec<String> {
        port.list(&scope(), &KeyPrefix::new(prefix).unwrap())
            .unwrap()
            .entries
            .into_iter()
            .map(|entry| entry.key.as_str().to_string())
            .collect()
    }

    fn task_done() -> Command {
        Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("demo".to_string()),
            touched_files: Some(vec!["src/app.rs".to_string()]),
            head_commit: None,
        }
    }

    fn list_changes() -> Command {
        Command::List {
            sort: "name".to_string(),
            specs: false,
            changes: false,
            worktrees: Default::default(),
        }
    }

    fn archive() -> Command {
        Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: true,
            mark_tasks_complete: true,
            carry_review: false,
            carry_verify: false,
        }
    }

    fn put_content<'c>(commit: &'c speclink_store_port::Commit, k: &str) -> Option<&'c str> {
        commit.ops().iter().find_map(|op| match op {
            Op::Put { key, content, .. } if key.as_str() == k => Some(content.as_str()),
            _ => None,
        })
    }

    #[test]
    fn task_done_is_exactly_one_commit_with_its_event() {
        // 「寫入的動詞一次提交」
        let probe = Probe::new(seeded());
        let before = version(&probe.inner);

        let done = execute_at(&probe, &scope(), &ctx(), task_done(), at()).expect("task done");

        let commits = probe.commits();
        assert_eq!(commits.len(), 1, "exactly one commit");
        let commit = &commits[0];
        assert_eq!(commit.command(), "task-done");
        assert_eq!((commit.actor().id(), commit.actor().display_name()), (ACTOR, ACTOR));
        assert_eq!(commit.at(), at(), "the time the Host gave");
        let tasks = put_content(commit, "changes/demo/tasks.md").expect("the flipped tasks.md");
        assert!(tasks.contains("- [x] 1.1"), "{tasks}");
        let evidence = put_content(commit, "changes/demo/.evidence.json").expect("the evidence");
        assert!(evidence.contains("src/app.rs"), "{evidence}");
        let mut sorted: Vec<&str> = commit.ops().iter().map(|op| op.key().as_str()).collect();
        let in_commit_order = sorted.clone();
        sorted.sort_unstable();
        assert_eq!(in_commit_order, sorted, "operations are in key order");

        assert_eq!(commit.events().len(), 1);
        assert_eq!(commit.events()[0].name(), "task-completed");
        assert_eq!(commit.events()[0].payload()["change"], "demo");
        assert_eq!(commit.events()[0].payload()["touchedFiles"], serde_json::json!(["src/app.rs"]));
        assert_eq!(done.events.len(), 1);

        let after = version(&probe.inner);
        assert!(after > before);
        assert_eq!(done.scope_version.map(Version::get), Some(after));
        let kept = probe.inner.recorded_events(&scope());
        assert_eq!(kept.len(), 1);
        assert_eq!((kept[0].name.as_str(), kept[0].version.get()), ("task-completed", after));
    }

    #[test]
    fn a_query_commits_nothing() {
        // 「查詢不提交」
        let probe = Probe::new(seeded());
        let before = version(&probe.inner);
        let listed = execute_at(&probe, &scope(), &ctx(), list_changes(), at()).expect("list");
        assert!(probe.commits().is_empty());
        assert_eq!(listed.scope_version, None);
        assert!(listed.events.is_empty());
        assert_eq!(version(&probe.inner), before);
    }

    #[test]
    fn archive_is_one_commit_that_moves_the_change() {
        // 「封存是一次提交裡的搬移」
        let probe = Probe::new(seeded());
        let archived = execute_at(&probe, &scope(), &ctx(), archive(), at()).expect("archive");
        let dated = match &archived.outcome {
            CommandOutcome::Archive(o) => o.dated_name.clone(),
            other => panic!("expected an archive outcome, got {other:?}"),
        };

        let commits = probe.commits();
        assert_eq!(commits.len(), 1, "exactly one commit");
        let commit = &commits[0];
        assert_eq!(commit.command(), "archive");
        for doc in [".meta.yaml", "tasks.md", "specs/auth/spec.md"] {
            assert!(
                put_content(commit, &format!("archive/changes/{dated}/{doc}")).is_some(),
                "{doc} is written to its archived key"
            );
            assert!(
                commit.ops().iter().any(|op| matches!(
                    op,
                    Op::Delete { key, .. } if key.as_str() == format!("changes/demo/{doc}")
                )),
                "{doc} is deleted from its active key"
            );
        }
        let canonical = put_content(commit, "specs/auth/spec.md").expect("the canonical spec");
        assert!(canonical.contains("### Requirement: R1"), "{canonical}");
        assert!(
            put_content(commit, &format!("archive/changes/{dated}/tasks.md"))
                .is_some_and(|tasks| tasks.contains("- [x] 1.2")),
            "the pre-write of the same flow travels into the archive"
        );
        assert_eq!(commit.events().len(), 1);
        assert_eq!(commit.events()[0].name(), "change-archived");

        // 提交之後以 `changes/` 為前綴列出時不含該變更。
        assert_eq!(keys_under(&probe.inner, "changes/"), Vec::<String>::new());
        assert_eq!(keys_under(&probe.inner, "specs/"), ["specs/auth/spec.md"]);
    }

    #[test]
    fn a_concurrent_write_is_a_conflict_and_nothing_lands() {
        // 「別人先寫入時回報衝突且不留半套」：動詞讀過 tasks.md 之後、提交
        // 之前，另一方改寫了它。
        let probe = Probe::new(seeded()).before_commit(|store| {
            overwrite(store, 2_000, "changes/demo/tasks.md", "## 1. Section\n\n- [ ] 1.1 changed\n");
        });

        let err = execute_at(&probe, &scope(), &ctx(), task_done(), at()).expect_err("conflict");

        match err {
            ExecError::Store(PortError::Conflict { key: k, expected, actual }) => {
                assert_eq!(k, key("changes/demo/tasks.md"));
                assert_eq!(expected, Expected::At(Version::new(1).unwrap()));
                assert_eq!(actual, Some(Version::new(2).unwrap()));
            }
            other => panic!("expected a store conflict, got {other:?}"),
        }
        // The other writer's content stays; nothing of this verb landed.
        let docs = probe
            .inner
            .read(&scope(), &[key("changes/demo/tasks.md"), key("changes/demo/.evidence.json")])
            .unwrap();
        assert!(docs[0].as_ref().unwrap().content.contains("1.1 changed"));
        assert!(docs[1].is_none(), "the evidence write did not land");
        assert!(probe.inner.recorded_events(&scope()).is_empty(), "no event landed");
        assert_eq!(version(&probe.inner), 2);
    }

    #[test]
    fn a_refused_command_leaves_the_store_alone() {
        // 「命令層拒絕時儲存不變」
        let probe = Probe::new(seeded());
        let missing = Command::TaskDone {
            task_id: "1".to_string(),
            change: Some("nonexistent".to_string()),
            touched_files: None,
            head_commit: None,
        };
        match execute_at(&probe, &scope(), &ctx(), missing, at()) {
            Err(ExecError::Command(e)) => assert_eq!(e.code, ErrorCode::NotFound),
            other => panic!("expected a command not_found, got {other:?}"),
        }
        assert!(probe.commits().is_empty());
        assert_eq!(version(&probe.inner), 1);
    }

    #[test]
    fn a_store_failure_at_any_call_is_a_store_error_and_writes_nothing() {
        // 「讀取途中儲存失敗時不提交」：對每一次儲存呼叫各失敗一次。
        let clean = Probe::new(seeded());
        execute_at(&clean, &scope(), &ctx(), task_done(), at()).expect("the clean run");
        let reads = clean.lists().len() + clean.reads().len();
        assert!(reads >= 2, "the verb reads before it commits");

        for n in 1..=reads + 1 {
            let probe = Probe::new(seeded()).failing_at(n);
            match execute_at(&probe, &scope(), &ctx(), task_done(), at()) {
                Err(ExecError::Store(PortError::Unavailable { .. })) => {}
                other => panic!("call {n}: expected a store failure, got {other:?}"),
            }
            if n <= reads {
                assert!(probe.commits().is_empty(), "call {n}: no commit after a failed read");
            }
            assert_eq!(version(&probe.inner), 1, "call {n}: the store did not change");
        }
    }

    #[test]
    fn an_anonymous_actor_commits_under_the_stable_placeholder() {
        let probe = Probe::new(seeded());
        let mut anonymous = ctx();
        anonymous.actor = Actor::Anonymous;
        execute_at(&probe, &scope(), &anonymous, task_done(), at()).expect("task done");
        let commit = &probe.commits()[0];
        assert_eq!((commit.actor().id(), commit.actor().display_name()), ("anonymous", "anonymous"));
    }
}

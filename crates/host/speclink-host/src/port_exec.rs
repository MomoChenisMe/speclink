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
    keys, Actor, Commit, DocKey, Document, Event, InvalidValue, KeyPrefix, Listing, Op, PortError,
    ScopeId, StorePort, Timestamp, Version,
};
use std::sync::{Mutex, PoisonError};

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
fn engine_context(ctx: &SpeclinkExecutionContext) -> ExecutionContext {
    ExecutionContext {
        actor: ctx.actor.display().map(str::to_string),
        repo: Some(ctx.repo.as_str().to_string()),
        ..Default::default()
    }
}

/// A short, stable command label for the commit's audit record. Derived from
/// the outcome, so it names the verb that produced the writes.
fn command_label(outcome: &CommandOutcome) -> String {
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
        CommandOutcome::ChangeRank(_) => "change-rank",
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
        CommandOutcome::WorkflowConfigEdit(_) => "workflow-config-edit",
        CommandOutcome::WorkflowConfigReplace(_) => "workflow-config-replace",
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
    execute_at(port, scope, ctx, cmd, now().map_err(ExecError::Store)?)
}

/// The current time as a commit time.
pub fn now() -> Result<Timestamp, PortError> {
    let millis = chrono::Utc::now().timestamp_millis().max(0) as u64;
    Timestamp::new(millis).map_err(invalid_value)
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

/// The scope's current version, from the narrowest list there is: the key of
/// the workflow config. Every list returns the scope version, and this one
/// returns one document at most.
pub fn scope_version(port: &dyn StorePort, scope: &ScopeId) -> Result<Version, PortError> {
    let prefix = KeyPrefix::new(keys::config().as_str()).expect("a key is a prefix");
    Ok(port.list(scope, &prefix)?.version)
}

/// What [`execute_at_version`] did.
#[derive(Debug)]
pub enum AtVersion {
    /// The scope stayed at the expected version and the command ran.
    Done(PortExecution),
    /// The scope is at another version; nothing was committed.
    Moved { actual: Version },
}

/// [`execute`], but only while the scope stays at `expected` — for a write
/// whose caller states the scope version it last saw.
///
/// The scope version is compared before the command runs, and again after
/// its reads: just before its commit, or at the end when it commits nothing.
/// A version that moved in between means another writer landed while the
/// command read, so nothing is committed and the answer is
/// [`AtVersion::Moved`]. The command thus never writes over content its
/// caller did not see. A writer that lands between the second comparison and
/// the commit meets the commit's own version check.
pub fn execute_at_version(
    port: &dyn StorePort,
    scope: &ScopeId,
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
    expected: Version,
) -> Result<AtVersion, ExecError> {
    let actual = scope_version(port, scope).map_err(ExecError::Store)?;
    if actual != expected {
        return Ok(AtVersion::Moved { actual });
    }
    let guard = AtExpected {
        port,
        expected,
        moved: Mutex::new(None),
    };
    let result = execute(&guard, scope, ctx, cmd);
    if let Some(actual) = *guard.moved.lock().unwrap_or_else(PoisonError::into_inner) {
        return Ok(AtVersion::Moved { actual });
    }
    let done = result?;
    if done.scope_version.is_none() {
        let actual = scope_version(port, scope).map_err(ExecError::Store)?;
        if actual != expected {
            return Ok(AtVersion::Moved { actual });
        }
    }
    Ok(AtVersion::Done(done))
}

/// A store port that refuses to commit once the scope has left `expected`.
struct AtExpected<'p> {
    port: &'p dyn StorePort,
    expected: Version,
    moved: Mutex<Option<Version>>,
}

impl StorePort for AtExpected<'_> {
    fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
        self.port.list(scope, prefix)
    }

    fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
        self.port.read(scope, keys)
    }

    fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
        let actual = scope_version(self.port, scope)?;
        if actual != self.expected {
            *self.moved.lock().unwrap_or_else(PoisonError::into_inner) = Some(actual);
            return Err(PortError::Backend {
                message: format!(
                    "the scope moved from version {} to {actual} during the command",
                    self.expected
                ),
            });
        }
        self.port.commit(scope, commit)
    }
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
    ExecError::Store(invalid_value(e))
}

/// [`invalid`] as the store's own failure.
pub(crate) fn invalid_value(e: InvalidValue) -> PortError {
    PortError::Backend {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Actor;
    use crate::port_view::testing::{host_ctx as ctx, key, overwrite, scope, seed, Probe, ACTOR};
    use speclink_core::command::ErrorCode;
    use speclink_store_port::{Expected, KeyPrefix, MemoryStorePort, Op, Timestamp};

    const AT: u64 = 1_800_000_000_000;
    const META: &str = "schema: spec-driven\n";
    const TASKS: &str = "## 1. Section\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";
    const DELTA: &str = "## Purpose\n\n本 capability 負責身分驗證的簽發與撤銷，涵蓋權杖生命週期各階段的可觀察行為、失敗處置與稽核紀錄。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

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

    // --- 流程設定的寫入（workflow-config-commands D1／D3）---

    const CONFIG: &str = "schema: spec-driven\ntdd: false\n";

    fn with_config_of(text: &str) -> MemoryStorePort {
        let port = seeded();
        seed(&port, 1_500, &[("config.yaml", text)]);
        port
    }

    fn with_config() -> MemoryStorePort {
        with_config_of(CONFIG)
    }

    fn set_tdd(value: &str, dry_run: bool) -> Command {
        Command::WorkflowConfigEdit {
            edit: speclink_core::config::WorkflowConfigEdit::Policy {
                key: "tdd".to_string(),
                value: value.to_string(),
            },
            dry_run,
        }
    }

    #[test]
    fn a_workflow_config_edit_is_one_commit_of_the_config_document() {
        let probe = Probe::new(with_config());
        let read_version = version(&probe.inner);

        let done = execute_at(&probe, &scope(), &ctx(), set_tdd("true", false), at()).expect("edit");

        let commits = probe.commits();
        assert_eq!(commits.len(), 1, "exactly one commit");
        let commit = &commits[0];
        assert_eq!(commit.command(), "workflow-config-edit");
        match commit.ops() {
            [Op::Put { key: k, content, expected }] => {
                assert_eq!(k, &key("config.yaml"));
                assert_eq!(content, "schema: spec-driven\ntdd: true\n");
                assert_eq!(
                    expected,
                    &Expected::At(Version::new(read_version).unwrap()),
                    "the write expects the version the command read"
                );
            }
            other => panic!("expected one write of config.yaml, got {other:?}"),
        }
        assert!(commit.events().is_empty(), "a workflow-config write has no domain event");
        assert_eq!(done.scope_version.map(Version::get), Some(version(&probe.inner)));
        assert_eq!(probe.reads(), [["config.yaml"]], "the command reads only the config document");
        assert!(probe.lists().is_empty());
    }

    #[test]
    fn a_workflow_config_write_that_changes_nothing_commits_nothing() {
        // 寫明的 `false` 會被改寫函式拿掉（false 不落鍵），所以這裡用沒有它的文件。
        const PLAIN: &str = "schema: spec-driven\n";
        for cmd in [
            set_tdd("false", false),
            set_tdd("true", true),
            Command::WorkflowConfigReplace { content: PLAIN.to_string() },
            Command::WorkflowConfigShow,
        ] {
            let probe = Probe::new(with_config_of(PLAIN));
            let before = version(&probe.inner);
            let done = execute_at(&probe, &scope(), &ctx(), cmd.clone(), at()).expect("executes");
            assert!(probe.commits().is_empty(), "{cmd:?}");
            assert_eq!(done.scope_version, None, "{cmd:?}");
            assert_eq!(version(&probe.inner), before, "{cmd:?}");
        }
    }

    #[test]
    fn a_workflow_config_replace_creates_a_missing_document() {
        let probe = Probe::new(seeded());
        let content = "schema: spec-driven\nlocale: ja\n";
        execute_at(
            &probe,
            &scope(),
            &ctx(),
            Command::WorkflowConfigReplace { content: content.to_string() },
            at(),
        )
        .expect("replace");
        let commits = probe.commits();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].command(), "workflow-config-replace");
        assert!(
            matches!(
                commits[0].ops(),
                [Op::Put { key: k, content: c, expected: Expected::Absent }]
                    if k == &key("config.yaml") && c == content
            ),
            "{:?}",
            commits[0].ops()
        );
    }

    #[test]
    fn a_workflow_config_edit_after_another_writer_is_a_conflict() {
        // 別人在讀取之後、提交之前改過流程設定：提交回報衝突，別人的內容留著。
        let probe = Probe::new(with_config()).before_commit(|store| {
            overwrite(store, 2_000, "config.yaml", "schema: spec-driven\nlocale: ja\n");
        });
        let read_version = version(&probe.inner);

        let err = execute_at(&probe, &scope(), &ctx(), set_tdd("true", false), at()).expect_err("conflict");

        match err {
            ExecError::Store(PortError::Conflict { key: k, expected, actual }) => {
                assert_eq!(k, key("config.yaml"));
                assert_eq!(expected, Expected::At(Version::new(read_version).unwrap()));
                assert_eq!(actual, Some(Version::new(read_version + 1).unwrap()));
            }
            other => panic!("expected a store conflict, got {other:?}"),
        }
        let doc = probe.inner.read(&scope(), &[key("config.yaml")]).unwrap();
        assert_eq!(doc[0].as_ref().unwrap().content, "schema: spec-driven\nlocale: ja\n");
    }

    /// A store where another writer replaces the config just before the
    /// command's first read: after the version comparison, so the command
    /// reads that writer's content.
    struct WriterBeforeFirstRead {
        inner: MemoryStorePort,
        pending: std::sync::Mutex<Option<&'static str>>,
    }

    impl StorePort for WriterBeforeFirstRead {
        fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
            self.inner.list(scope, prefix)
        }
        fn read(&self, scope: &ScopeId, keys: &[DocKey]) -> Result<Vec<Option<Document>>, PortError> {
            if let Some(text) = self.pending.lock().unwrap().take() {
                overwrite(&self.inner, 2_000, "config.yaml", text);
            }
            self.inner.read(scope, keys)
        }
        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            self.inner.commit(scope, commit)
        }
    }

    fn config_text(port: &dyn StorePort) -> String {
        port.read(&scope(), &[key("config.yaml")]).unwrap()[0].clone().unwrap().content
    }

    fn replace_with(text: &str) -> Command {
        Command::WorkflowConfigReplace { content: text.to_string() }
    }

    #[test]
    fn execute_at_version_runs_the_command_while_the_scope_is_at_that_version() {
        let port = with_config();
        let expected = Version::new(version(&port)).unwrap();
        match execute_at_version(&port, &scope(), &ctx(), replace_with("locale: ja\n"), expected) {
            Ok(AtVersion::Done(done)) => assert_eq!(done.scope_version.map(Version::get), Some(version(&port))),
            other => panic!("expected the command to run, got {other:?}"),
        }
        assert_eq!(config_text(&port), "locale: ja\n");
    }

    #[test]
    fn execute_at_version_runs_nothing_on_a_stale_version() {
        let probe = Probe::new(with_config());
        let expected = Version::new(version(&probe.inner) - 1).unwrap();
        match execute_at_version(&probe, &scope(), &ctx(), replace_with("locale: ja\n"), expected) {
            Ok(AtVersion::Moved { actual }) => assert_eq!(actual.get(), version(&probe.inner)),
            other => panic!("expected moved, got {other:?}"),
        }
        assert!(probe.reads().is_empty(), "the command did not run");
        assert!(probe.commits().is_empty());
    }

    #[test]
    fn a_writer_that_lands_before_the_command_reads_is_never_written_over() {
        // 比對版本之後、命令讀取之前有人改了流程設定：命令讀到的是別人的內容，
        // 提交前的第二次比對發現版本前進，什麼都不寫。
        let port = WriterBeforeFirstRead {
            inner: with_config(),
            pending: std::sync::Mutex::new(Some("schema: spec-driven\nlocale: ja\n")),
        };
        let expected = version(&port.inner);
        match execute_at_version(&port, &scope(), &ctx(), replace_with("locale: en\n"), Version::new(expected).unwrap()) {
            Ok(AtVersion::Moved { actual }) => assert_eq!(actual.get(), expected + 1),
            other => panic!("expected moved, got {other:?}"),
        }
        assert_eq!(config_text(&port.inner), "schema: spec-driven\nlocale: ja\n", "the other writer's content stays");
        assert_eq!(version(&port.inner), expected + 1, "nothing else was committed");
    }

    #[test]
    fn a_command_that_commits_nothing_still_reports_a_moved_version() {
        // 別人寫入的內容剛好等於請求的內容：命令沒有東西可寫，但版本已經不是呼叫端看到的那個。
        let port = WriterBeforeFirstRead {
            inner: with_config(),
            pending: std::sync::Mutex::new(Some("locale: ja\n")),
        };
        let expected = version(&port.inner);
        match execute_at_version(&port, &scope(), &ctx(), replace_with("locale: ja\n"), Version::new(expected).unwrap()) {
            Ok(AtVersion::Moved { actual }) => assert_eq!(actual.get(), expected + 1),
            other => panic!("expected moved, got {other:?}"),
        }
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

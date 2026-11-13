//! The `change landed` command (station-status-and-landing D5): record which
//! commit a change's code landed on.
//!
//! The engine stores what the caller reports — the commit, optionally the
//! branch — with the time and the reporter. It reads no git and decides
//! nothing by these values: a rule such as "landed before archive" is the
//! host's, in its before-command hook.

use super::{classify, CommandError, CommandOutcome, ErrorCode};
use crate::store::Store;

/// The metadata keys of one landing report. A repeated report overwrites them.
const LANDED_COMMIT: &str = "landed_commit";
const LANDED_AT: &str = "landed_at";
const LANDED_BY: &str = "landed_by";

/// `change landed` outcome: what `--json` prints. `commit` is the full value
/// as recorded; `branch` is present only when this call gave one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLandedOutcome {
    pub change: String,
    pub commit: String,
    pub branch: Option<String>,
}

/// The landing record a change's metadata holds, as `show --json` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandedRecord {
    pub commit: String,
    /// The report time as written: RFC 3339.
    pub at: String,
    pub by: Option<String>,
}

impl LandedRecord {
    /// The record from its three stored values — a change's metadata, or a
    /// server's single-change response. `None` when no landing was reported
    /// (no commit). A commit with no time is a hand-edited record: the time
    /// reads as empty.
    pub fn new(commit: Option<String>, at: Option<String>, by: Option<String>) -> Option<LandedRecord> {
        Some(LandedRecord { commit: commit?, at: at.unwrap_or_default(), by })
    }
}

/// Every guard runs before the write, and the write is one edit of the
/// change's metadata: a refusal leaves the document as it was.
pub(super) fn run(
    store: &dyn Store,
    actor: Option<&str>,
    name: String,
    commit: &str,
    branch: Option<String>,
) -> Result<CommandOutcome, CommandError> {
    let commit = crate::model::checked_commit(commit).map_err(classify)?;
    if let Some(branch) = &branch {
        crate::model::checked_branch(branch).map_err(classify)?;
    }
    let not_found = || CommandError::new(ErrorCode::NotFound, format!("Change '{name}' not found."));
    if crate::model::find_change(store, &name).is_none() {
        return Err(not_found());
    }
    // The stage is not a guard: code can land before the change is started.
    crate::model::edit_meta(store, &name, |m| {
        m.set(LANDED_COMMIT, &crate::util::yaml_roundtrip_scalar(&commit))?;
        m.set(LANDED_AT, &crate::util::now_rfc3339())?;
        // The reporter of this report only: an anonymous report does not keep
        // the name of whoever reported before it.
        match actor {
            Some(actor) => m.set(LANDED_BY, &crate::util::yaml_scalar(actor))?,
            None => m.remove(LANDED_BY),
        }
        if let Some(branch) = &branch {
            m.set(crate::inprogress::BRANCH_KEY, &crate::util::yaml_roundtrip_scalar(branch))?;
        }
        // The edit only sees a key written as `key:` in the first column. A
        // hand-written `key : value` would get a second line with the same
        // key, and the document would no longer parse. Refuse unless the
        // result reads back as this report.
        let written = crate::model::check_meta_text(store, &name, Some(&m.text())).ok();
        let reads_back = written.is_some_and(|meta| {
            meta.landed_commit.as_deref() == Some(commit.as_str())
                && branch.as_ref().is_none_or(|branch| meta.branch.as_deref() == Some(branch.as_str()))
        });
        if !reads_back {
            anyhow::bail!(crate::command::Refusal(format!(
                "cannot record the landing of '{name}': {} has a `landed_commit:` or `branch:` line this \
                 command cannot rewrite — make it a plain `key: value` line first",
                store.change_meta_display(&name)
            )));
        }
        Ok(())
    })
    .map_err(classify)?
    // A change the roster lists without a metadata document has nowhere to
    // keep the record.
    .ok_or_else(not_found)?;
    Ok(CommandOutcome::ChangeLanded(ChangeLandedOutcome { change: name, commit, branch }))
}

#[cfg(test)]
mod tests {
    use crate::command::{execute, Command, CommandError, CommandOutcome, DomainEvent, ErrorCode, ExecutionContext};
    use crate::model::{ChangeMeta, Stage};
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";

    fn landed(name: &str, commit: &str, branch: Option<&str>) -> Command {
        Command::ChangeLanded { name: name.into(), commit: commit.into(), branch: branch.map(str::to_string) }
    }

    fn ctx(actor: Option<&str>) -> ExecutionContext {
        ExecutionContext { actor: actor.map(str::to_string), ..Default::default() }
    }

    fn run(
        store: &TestStore,
        actor: Option<&str>,
        cmd: Command,
    ) -> Result<(CommandOutcome, Vec<DomainEvent>), CommandError> {
        execute(store, &ctx(actor), cmd)
    }

    fn meta_of(store: &TestStore) -> ChangeMeta {
        ChangeMeta::from_text(Some(&store.meta("demo"))).expect("meta parses")
    }

    fn lines_of(store: &TestStore, key: &str) -> usize {
        store.meta("demo").lines().filter(|l| l.starts_with(&format!("{key}:"))).count()
    }

    #[test]
    fn a_report_records_the_commit_the_time_and_the_reporter_in_one_write() {
        // Scenario「以目前的 HEAD 回報」的引擎面：完整 commit、回報時間、回報的人。
        let store = TestStore::with_meta("demo", META);
        let (outcome, events) = run(&store, Some("Ann <ann@example.com>"), landed("demo", SHA, None)).unwrap();
        let CommandOutcome::ChangeLanded(o) = outcome else { panic!("a change-landed outcome") };
        assert_eq!((o.change.as_str(), o.commit.as_str(), o.branch), ("demo", SHA, None));
        let [DomainEvent::ChangeLanded { change, commit, branch, .. }] = &events[..] else {
            panic!("one change-landed event, got {events:?}");
        };
        assert_eq!((change.as_str(), commit.as_str(), branch), ("demo", SHA, &None));
        assert_eq!(events[0].kind(), "change-landed");

        let raw = store.meta("demo");
        assert!(raw.starts_with(META), "existing fields kept: {raw}");
        let meta = meta_of(&store);
        assert_eq!(meta.landed_commit.as_deref(), Some(SHA));
        assert_eq!(meta.landed_by.as_deref(), Some("Ann <ann@example.com>"));
        let at = meta.landed_at.expect("a report time");
        assert!(crate::util::is_offset_stamp(&at), "RFC 3339 with an offset: {at}");
        assert!(meta.branch.is_none(), "no branch was given");
        assert_eq!(*store.meta_writes.borrow(), 1, "one metadata write");
    }

    #[test]
    fn an_anonymous_report_records_no_reporter() {
        let store = TestStore::with_meta("demo", META);
        run(&store, None, landed("demo", SHA, None)).unwrap();
        assert_eq!(lines_of(&store, "landed_by"), 0);
        assert_eq!(meta_of(&store).landed_commit.as_deref(), Some(SHA));
    }

    #[test]
    fn a_given_branch_overwrites_the_recorded_one_and_no_branch_leaves_it() {
        // Scenario「指定 commit 與分支」。
        let store = TestStore::with_meta("demo", &format!("{META}branch: feature/x\n"));
        run(&store, None, landed("demo", SHA, None)).unwrap();
        assert_eq!(meta_of(&store).branch.as_deref(), Some("feature/x"), "no --branch keeps the branch");

        let (outcome, events) = run(&store, None, landed("demo", SHA, Some("speclink/demo"))).unwrap();
        assert_eq!(meta_of(&store).branch.as_deref(), Some("speclink/demo"));
        assert_eq!(lines_of(&store, "branch"), 1);
        let CommandOutcome::ChangeLanded(o) = outcome else { panic!("a change-landed outcome") };
        assert_eq!(o.branch.as_deref(), Some("speclink/demo"));
        let [DomainEvent::ChangeLanded { branch, .. }] = &events[..] else { panic!("one event") };
        assert_eq!(branch.as_deref(), Some("speclink/demo"));
    }

    #[test]
    fn a_repeated_report_overwrites_the_earlier_one() {
        // Scenario「重複回報覆寫」：只留新的 commit、新的回報時間、這一次回報的人。
        let earlier =
            format!("{META}landed_commit: \"{SHA}\"\nlanded_at: 2026-11-01T10:00:00+08:00\nlanded_by: Ann <ann@example.com>\n");
        let store = TestStore::with_meta("demo", &earlier);
        run(&store, None, landed("demo", OTHER, None)).unwrap();
        let meta = meta_of(&store);
        assert_eq!(meta.landed_commit.as_deref(), Some(OTHER));
        assert_ne!(meta.landed_at.as_deref(), Some("2026-11-01T10:00:00+08:00"));
        assert!(meta.landed_by.is_none(), "the earlier reporter does not stay on an anonymous report");
        for key in ["landed_commit", "landed_at"] {
            assert_eq!(lines_of(&store, key), 1, "{key}");
        }
    }

    #[test]
    fn the_commit_is_recorded_without_its_surrounding_whitespace() {
        let store = TestStore::with_meta("demo", META);
        let (outcome, _) = run(&store, None, landed("demo", "  1234567\n", None)).unwrap();
        let CommandOutcome::ChangeLanded(o) = outcome else { panic!("a change-landed outcome") };
        assert_eq!(o.commit, "1234567");
        assert_eq!(meta_of(&store).landed_commit.as_deref(), Some("1234567"), "digits stay text");
    }

    #[test]
    fn an_unknown_change_is_not_found() {
        // Scenario「變更不存在」。
        let store = TestStore::with_meta("demo", META);
        for name in ["ghost", "../demo", "archive"] {
            let err = run(&store, None, landed(name, "0123456", None)).expect_err(name);
            assert_eq!(err.code, ErrorCode::NotFound, "{name}");
            assert_eq!(err.message, format!("Change '{name}' not found."));
        }
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_corrupt_meta_refuses_without_writing() {
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_meta("demo", BAD);
        let err = run(&store, None, landed("demo", SHA, None)).expect_err("corrupt meta");
        assert_eq!(err.code, ErrorCode::InvalidConfig);
        assert_eq!(store.meta("demo"), BAD);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_malformed_commit_or_branch_is_invalid_argv_with_zero_writes() {
        // Scenario「commit 的形狀不合」。
        let store = TestStore::with_meta("demo", META);
        let too_long = "a".repeat(65);
        for commit in ["not-a-sha", "012345", "", too_long.as_str(), "0123456\nlanded_by: forged"] {
            let err = run(&store, None, landed("demo", commit, None)).expect_err(commit);
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{commit:?}");
        }
        for branch in ["", "has space", "a\nb"] {
            let err = run(&store, None, landed("demo", SHA, Some(branch))).expect_err(branch);
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{branch:?}");
        }
        assert_eq!(store.meta("demo"), META);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_proposed_change_can_report_and_stays_proposed() {
        // Scenario「提案中的變更也能回報」：不看階段，也不改階段。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [ ] 1.1 open\n");
        run(&store, None, landed("demo", "0123456", None)).unwrap();
        let change = crate::model::find_change(&store, "demo").expect("the change");
        assert_eq!(crate::model::stage(&store, &change), Stage::Proposed);
        assert!(meta_of(&store).started_at.is_none());
    }

    #[test]
    fn a_reporter_name_with_yaml_marks_adds_no_key() {
        let store = TestStore::with_meta("demo", META);
        run(&store, Some("Eve\nbranch: forged # x: y"), landed("demo", SHA, None)).unwrap();
        let meta = meta_of(&store);
        assert_eq!(meta.landed_by.as_deref(), Some("Eve branch: forged # x: y"));
        assert!(meta.branch.is_none(), "{}", store.meta("demo"));
    }

    #[test]
    fn a_key_the_edit_cannot_reach_refuses_without_writing() {
        // sharp-edges 稽核：手改的 meta 把鍵寫成 `landed_commit : x`（冒號前有空白）或值放在
        // 下一行時，文字手術看不到那個鍵，會補出第二個同名鍵——整份 meta 從此解析失敗。
        // 寫入前以解析結果把關：讀不回這次的值就拒絕，零寫入。
        for meta in [
            "schema: spec-driven\nlanded_commit : \"0000000\"\n",
            "schema: spec-driven\nbranch : old\n",
        ] {
            let store = TestStore::with_meta("demo", meta);
            let err = run(&store, None, landed("demo", SHA, Some("speclink/demo"))).expect_err(meta);
            assert_eq!(err.code, ErrorCode::Refused, "{meta:?}: {}", err.message);
            assert!(err.message.contains(".meta.yaml"), "{}", err.message);
            assert_eq!(store.meta("demo"), meta);
            assert_eq!(*store.meta_writes.borrow(), 0);
        }
    }

    #[test]
    fn the_record_stays_in_the_archived_metadata() {
        // Scenario「封存後記錄仍在」。
        let store = TestStore::with_meta("demo", META);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        run(&store, Some("Ann <ann@example.com>"), landed("demo", SHA, Some("speclink/demo"))).unwrap();
        let ghost = Workspace {
            root: std::env::temp_dir().join("speclink-landing-test-ghost-root"),
            spec_dir_name: "speclink".to_string(),
        };
        let archive = Command::Archive {
            change: Some("demo".to_string()),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        };
        execute(&store, &ExecutionContext { workspace: Some(ghost), ..Default::default() }, archive).expect("archive");
        let archived = store.archived_metas.borrow();
        let [text] = &archived.values().collect::<Vec<_>>()[..] else { panic!("one archived change") };
        let meta = ChangeMeta::from_text(Some(text)).expect("archived meta parses");
        assert_eq!(meta.landed_commit.as_deref(), Some(SHA));
        assert_eq!(meta.branch.as_deref(), Some("speclink/demo"));
        assert!(meta.landed_at.is_some() && meta.landed_by.is_some());
    }

    #[test]
    fn the_command_has_its_name_and_its_params() {
        let cmd = landed("demo", SHA, Some("speclink/demo"));
        assert_eq!(cmd.name(), "change-landed");
        assert_eq!(
            cmd.params(),
            serde_json::json!({ "name": "demo", "commit": SHA, "branch": "speclink/demo" })
        );
    }
}

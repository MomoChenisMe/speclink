//! The `change rank` command (skills-verbs-first-wave D3, D4).
//!
//! The guards, the column order, the column stamp, the midpoint and the
//! dependency check are [`crate::plan::set_rank`]'s, unchanged. The command
//! only decides where the ranks are read from and written to: the store's
//! [`Store::board_ranks`] and [`Store::write_board_ranks`]. The local CLI keeps
//! calling `set_rank` itself with a writer that picks each change's own copy
//! (D5); a remote scope and an embedded host come here.

use super::{classify, CommandError, CommandOutcome, ErrorCode};
use crate::plan::Side;
use crate::store::Store;
use std::cell::RefCell;

/// `change rank` outcome: the three fields `--json` prints. The rank value
/// itself stays out — it is an internal order key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRankOutcome {
    pub change: String,
    pub anchor: String,
    pub position: Side,
}

pub(super) fn run(
    store: &dyn Store,
    name: String,
    anchor: String,
    side: Side,
    force: bool,
) -> Result<CommandOutcome, CommandError> {
    // Loud not-found under the stable code before the flow; every other guard
    // is a typed Refusal inside `set_rank`, a corrupt meta its MetaError.
    if crate::model::find_change(store, &name).is_none() {
        return Err(CommandError::new(ErrorCode::NotFound, format!("Change '{name}' not found.")));
    }
    // `set_rank` checks everything before its first write, so collecting its
    // writes and handing them to the store at once changes nothing but where
    // they land: one batch, which a one-document backend commits together.
    let writes: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
    let collect = |change: &str, key: &str| {
        writes.borrow_mut().push((change.to_string(), key.to_string()));
        Ok(())
    };
    crate::plan::set_rank(store, &collect, &name, &anchor, side, force).map_err(classify)?;
    store.write_board_ranks(&writes.into_inner()).map_err(classify)?;
    Ok(CommandOutcome::ChangeRank(ChangeRankOutcome { change: name, anchor, position: side }))
}

#[cfg(test)]
mod tests {
    use crate::command::{execute, Command, CommandOutcome, DomainEvent, ErrorCode, ExecutionContext};
    use crate::model;
    use crate::plan::{self, Side};
    use crate::store::Store;
    use crate::teststore::TestStore;

    fn store_of(changes: &[(&str, &str)]) -> TestStore {
        let store = TestStore::default();
        for (name, meta) in changes {
            store.metas.borrow_mut().insert(name.to_string(), meta.to_string());
        }
        store
    }

    fn rank_cmd(name: &str, anchor: &str, side: Side, force: bool) -> Command {
        Command::ChangeRank { name: name.into(), anchor: anchor.into(), side, force }
    }

    /// One scenario on two copies of the same store: `set_rank` with the meta
    /// writer on the first, the command on the second. Both must write the
    /// same metas in the same order and fail with the same message.
    fn same_as_set_rank(
        changes: &[(&str, &str)],
        missing_meta: Option<&str>,
        name: &str,
        anchor: &str,
        side: Side,
        force: bool,
    ) -> (TestStore, Result<(CommandOutcome, Vec<DomainEvent>), crate::command::CommandError>) {
        let direct = store_of(changes);
        let commanded = store_of(changes);
        *direct.missing_meta.borrow_mut() = missing_meta.map(str::to_string);
        *commanded.missing_meta.borrow_mut() = missing_meta.map(str::to_string);
        let writer = |change: &str, key: &str| model::set_board_rank(&direct, change, key);
        let expected = plan::set_rank(&direct, &writer, name, anchor, side, force);
        let got = execute(&commanded, &ExecutionContext::default(), rank_cmd(name, anchor, side, force));
        match (&expected, &got) {
            (Ok(()), Ok(_)) => {}
            (Err(want), Err(have)) => assert_eq!(have.message, want.to_string(), "same message"),
            (want, have) => panic!("set_rank gave {want:?}, the command gave {have:?}"),
        }
        assert_eq!(*commanded.metas.borrow(), *direct.metas.borrow(), "same metas after the run");
        assert_eq!(*commanded.meta_write_log.borrow(), *direct.meta_write_log.borrow(), "same write order");
        (commanded, got)
    }

    fn rank_of(store: &TestStore, name: &str) -> Option<String> {
        model::ChangeMeta::from_text(Some(&store.meta(name))).expect("meta parses").board_rank
    }

    fn plan_names(store: &TestStore) -> Vec<String> {
        plan::compute(store).unwrap().changes.into_iter().map(|c| c.name).collect()
    }

    const PROPOSED: &str = "schema: spec-driven\ncreated: 2026-09-01\n";

    #[test]
    fn an_unranked_change_goes_before_a_ranked_one() {
        // Scenario 無 rank 的 change 排到另一個之前.
        let (store, got) = same_as_set_rank(
            &[("a", "schema: spec-driven\nboard_rank: n\n"), ("c", PROPOSED)],
            None,
            "c",
            "a",
            Side::Before,
            false,
        );
        let (outcome, events) = got.expect("the move succeeds");
        let CommandOutcome::ChangeRank(o) = outcome else { panic!("a ChangeRank outcome") };
        assert_eq!((o.change.as_str(), o.anchor.as_str(), o.position), ("c", "a", Side::Before));
        assert_eq!(events.len(), 1, "one event: {events:?}");
        assert!(matches!(&events[0], DomainEvent::ChangeRanked { change, .. } if change == "c"));
        assert_eq!(events[0].kind(), "change-ranked");
        assert!(rank_of(&store, "c").unwrap().as_str() < "n");
        assert_eq!(*store.meta_write_log.borrow(), ["c"]);
        assert_eq!(plan_names(&store), ["c", "a"]);
    }

    #[test]
    fn a_ranked_change_without_force_is_refused() {
        // Scenario 已有 rank 未帶 --force 被拒.
        let (store, got) = same_as_set_rank(
            &[("a", "schema: spec-driven\nboard_rank: n\n"), ("c", "schema: spec-driven\nboard_rank: t\n")],
            None,
            "c",
            "a",
            Side::After,
            false,
        );
        let err = got.expect_err("refused");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(err.message.contains("already has a board rank") && err.message.contains("--force"), "{}", err.message);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn force_overwrites_a_rank() {
        // Scenario 帶 --force 覆寫既有 rank.
        let (store, got) = same_as_set_rank(
            &[("a", "schema: spec-driven\nboard_rank: n\n"), ("c", "schema: spec-driven\nboard_rank: b\n")],
            None,
            "c",
            "a",
            Side::After,
            true,
        );
        got.expect("forced move succeeds");
        assert!(rank_of(&store, "c").unwrap().as_str() > "n");
        assert_eq!(plan_names(&store), ["a", "c"]);
    }

    #[test]
    fn a_move_across_a_declared_dependency_is_refused() {
        // Scenario 跨宣告依賴被拒.
        let (store, got) = same_as_set_rank(
            &[("a", PROPOSED), ("c", "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n")],
            None,
            "c",
            "a",
            Side::Before,
            false,
        );
        let err = got.expect_err("refused");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(err.message.contains("cannot move 'c' there") && err.message.contains("it depends on a"), "{}", err.message);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_column_without_ranks_is_stamped_first() {
        // Scenario 缺 rank 的欄先整欄補章.
        let (store, got) = same_as_set_rank(
            &[
                ("a", "schema: spec-driven\ncreated: 2026-09-01\n"),
                ("b", "schema: spec-driven\ncreated: 2026-09-02\n"),
                ("c", "schema: spec-driven\ncreated: 2026-09-03\n"),
            ],
            None,
            "c",
            "b",
            Side::Before,
            false,
        );
        got.expect("the move succeeds");
        let rank = |name: &str| rank_of(&store, name).unwrap();
        assert!(rank("a") < rank("c") && rank("c") < rank("b"));
        assert_eq!(*store.meta_write_log.borrow(), ["a", "b", "c"]);
        assert_eq!(plan_names(&store), ["a", "c", "b"]);
    }

    #[test]
    fn a_different_stage_is_refused() {
        // Scenario 不同階段被拒.
        let (store, got) = same_as_set_rank(
            &[("a", "schema: spec-driven\nstarted_at: 2026-09-06\n"), ("c", PROPOSED)],
            None,
            "c",
            "a",
            Side::Before,
            false,
        );
        let err = got.expect_err("refused");
        assert_eq!(err.code, ErrorCode::Refused);
        assert!(err.message.contains("proposed") && err.message.contains("in-progress"), "{}", err.message);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn a_change_without_meta_is_refused_before_any_write() {
        // Scenario 缺 .meta.yaml 的 change 零寫入拒絕.
        let changes = [("p1", PROPOSED), ("p2", PROPOSED), ("p3", PROPOSED)];
        for name in ["p2", "p3"] {
            let (store, got) = same_as_set_rank(&changes, Some("p2"), name, "p1", Side::Before, false);
            let err = got.expect_err("refused");
            assert_eq!(err.code, ErrorCode::Refused);
            assert!(err.message.contains("speclink/changes/p2/.meta.yaml is missing"), "{}", err.message);
            assert_eq!(*store.meta_writes.borrow(), 0);
        }
    }

    #[test]
    fn an_unknown_change_is_not_found() {
        let store = store_of(&[("a", PROPOSED)]);
        let err = execute(&store, &ExecutionContext::default(), rank_cmd("ghost", "a", Side::Before, false))
            .expect_err("not found");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "Change 'ghost' not found.");
    }

    // --- the two Store methods' defaults: the change metas (D4) ---

    #[test]
    fn board_ranks_defaults_to_each_meta_board_rank() {
        let store = store_of(&[
            ("a", "schema: spec-driven\nboard_rank: n\n"),
            ("c", PROPOSED),
            ("bad", ": : :\n\t bad yaml [unclosed\n"),
        ]);
        let ranks = store.board_ranks();
        assert_eq!(ranks.into_iter().collect::<Vec<_>>(), [("a".to_string(), "n".to_string())]);
    }

    #[test]
    fn write_board_ranks_defaults_to_meta_writes_in_order() {
        let store = store_of(&[("a", PROPOSED), ("b", "schema: spec-driven\nboard_rank: x\n")]);
        store
            .write_board_ranks(&[("b".to_string(), "f".to_string()), ("a".to_string(), "t".to_string())])
            .expect("writes");
        assert_eq!(*store.meta_write_log.borrow(), ["b", "a"]);
        assert_eq!(store.meta("a"), format!("{PROPOSED}board_rank: t\n"), "other bytes kept, one line added");
        assert_eq!(store.meta("b"), "schema: spec-driven\nboard_rank: f\n", "the line is replaced in place");
    }

    #[test]
    fn write_board_ranks_refuses_a_key_that_is_not_a_plain_scalar() {
        let store = store_of(&[("a", PROPOSED)]);
        let err = store
            .write_board_ranks(&[("a".to_string(), "x\nstarted_at: now".to_string())])
            .expect_err("a key that would add a line is refused");
        assert!(err.to_string().contains("invalid board rank"), "{err}");
        assert_eq!(store.meta("a"), PROPOSED);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }
}

//! What a command needs beyond argv, and what the conversion returns.

use std::collections::BTreeMap;

use speclink_core::command::Command;
use speclink_core::listing::ListWorktreeJson;

/// Facts the caller collects for a command — argv cannot carry them, and
/// this crate never reads them itself. Every field defaults to "not given".
///
/// Where each command field comes from (checked against the CLI's `cmd_*`):
/// every field of the 45 commands the conversion returns comes from argv,
/// except these.
///
/// | Command field | Input | What the CLI passes |
/// |---|---|---|
/// | `List.worktrees` | `worktrees` | the main checkout's worktree facts (`common::worktree_overlay`) |
/// | `NewArtifact.content` | `stdin` (with `--stdin`) | `common::stdin_for` |
/// | `TaskDone.touched_files`, `TaskDone.head_commit` | `touched_files`, `head_commit` | nothing: the engine probes the workspace (the remote arm sends its own request) |
/// | `Plan.ranks` | `ranks` | nothing: each change's own board rank |
/// | `DiscussContext`／`DiscussAddRound`／`DiscussConclude` `.content` | `stdin` (empty when unread) | `common::stdin_for` |
/// | `ReviewAddRound`／`VerifyAddRound` `.content` | `stdin` (empty when unread) | — (the CLI's stations run `station_fs`) |
/// | `ReviewStamp`／`VerifyStamp` `.scope`, `.missing` | `stdin` JSON (with `--stdin`) | — (the CLI computes the scope and refuses `--stdin`) |
/// | `ReviewStamp`／`VerifyStamp` `.head_commit` | `stdin` JSON `headCommit`, else `head_commit` | — (the CLI's stations pass the checkout's HEAD themselves) |
/// | `InProgressAdd.branch` | `branch` | the checkout's branch (`common::git_branch`); none on a detached HEAD |
/// | `ChangeLanded.commit` | `--commit`, else `head_commit` | the checkout's HEAD |
/// | `WorkflowConfigEdit.edit` of `context`／`rules`／`review <list>`／`subagent-model choices` | `stdin` (with `--stdin`; empty when unread) | `common::stdin_for` |
///
/// The execution context (actor, repo, policy overrides) is not part of a
/// command and does not come through here.
#[derive(Debug, Default)]
pub struct VerbInputs {
    /// The stdin content, when the caller read it (see [`StdinPolicy`]).
    pub stdin: Option<String>,
    /// The checkout's change → worktree facts for `list` (empty: none).
    pub worktrees: BTreeMap<String, ListWorktreeJson>,
    /// The touched-file candidates for `task done`, when the caller resolved
    /// them; absent lets the engine probe the local workspace.
    pub touched_files: Option<Vec<String>>,
    /// The commit the caller's checkout is on: the one the touched files were
    /// observed on for `task done`, the stamped one for the two stamps, and
    /// the landing commit of `change landed` without `--commit`.
    pub head_commit: Option<String>,
    /// The branch the caller's checkout is on, for `in-progress add`; absent
    /// on a detached HEAD or without a checkout.
    pub branch: Option<String>,
    /// An external board rank table for `plan`; absent reads each change's
    /// own rank.
    pub ranks: Option<BTreeMap<String, String>>,
}

/// Whether a verb reads stdin. `flagged` records whether argv carried
/// `--stdin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdinPolicy {
    /// The verb never reads stdin.
    Never,
    /// The verb reads stdin only with `--stdin`.
    WhenFlagged { flagged: bool },
    /// The verb reads stdin with `--stdin`, or when stdin is piped.
    FlagOrPiped { flagged: bool },
}

impl StdinPolicy {
    /// Whether the caller should read stdin. `piped` is the caller's own fact:
    /// the CLI passes "stdin is not a terminal", a host passes "stdin content
    /// was given".
    pub fn reads(self, piped: bool) -> bool {
        match self {
            StdinPolicy::Never => false,
            StdinPolicy::WhenFlagged { flagged } => flagged,
            StdinPolicy::FlagOrPiped { flagged } => flagged || piped,
        }
    }
}

/// What a verb converts to.
#[derive(Debug, PartialEq)]
pub enum CommandTarget {
    /// The engine command this verb stands for.
    Command(Command),
    /// The verb has no engine command, so only the CLI runs it; `reason` says
    /// why.
    LocalOnly { verb: &'static str, reason: LocalOnlyReason },
}

/// Why a verb has no engine command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalOnlyReason {
    /// It works on facts only a checkout holds: git state, worktree copies,
    /// the stations' snapshots.
    NeedsCheckout,
    /// It prints skill text built into the CLI.
    CliSkillText,
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::target_of;
    use crate::{CommandTarget, VerbInputs};

    /// One argv per command the conversion returns: 45 distinct commands, so
    /// the table above speaks for every one of them (TaskMove and
    /// WorkflowConfigReplace have no argv).
    #[test]
    fn the_conversion_reaches_45_commands() {
        let argvs: &[&[&str]] = &[
            &["list"], &["show", "x"], &["status"], &["validate"], &["analyze"],
            &["artifact", "cat", "tasks"], &["language", "show"], &["trace", "x"], &["instructions"],
            &["new", "change", "x"], &["new", "artifact", "tasks"],
            &["task", "done", "1"], &["task", "undone", "1"],
            &["in-progress", "add", "x"], &["in-progress", "remove", "x"],
            &["archive"], &["discard", "x"], &["claim", "x"],
            &["plan"], &["plan", "--strict-overlap"],
            &["change", "depends", "x", "--on", "y"], &["change", "schema", "x", "y"],
            &["change", "landed", "x", "--commit", "0123456"],
            &["discuss", "new", "t"], &["discuss", "list"], &["discuss", "show", "t"],
            &["discuss", "search", "k"], &["discuss", "context", "t"], &["discuss", "add-round", "t"],
            &["discuss", "conclude", "t"], &["discuss", "archive", "t"], &["discuss", "discard", "t"],
            &["discuss", "promote", "t"], &["discuss", "link", "t", "c"], &["discuss", "seal", "t", "c"],
            &["review", "add-round", "c"], &["review", "show", "c"], &["review", "stamp", "c"],
            &["review", "discard", "c"],
            &["verify", "add-round", "c"], &["verify", "show", "c"], &["verify", "stamp", "c"],
            &["verify", "discard", "c"],
            &["workflow-config", "show"], &["workflow-config", "set", "tdd", "true"],
        ];
        let kinds: std::collections::HashSet<_> = argvs
            .iter()
            .map(|argv| match target_of(argv, VerbInputs::default()) {
                Ok(CommandTarget::Command(cmd)) => std::mem::discriminant(&cmd),
                other => panic!("{argv:?} must convert to a command, got {other:?}"),
            })
            .collect();
        assert_eq!(kinds.len(), 45);
    }
}

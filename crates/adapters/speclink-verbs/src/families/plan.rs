//! Execution-order verbs: `plan`, `change depends`, `change schema`, `change rank`;
//! and `change landed`, the landing report of the same `change` family. The
//! family's `field` verbs are in [`super::fields`].

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError, ErrorCode};
use speclink_core::plan::Side;

use super::fields::FieldKind;
use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct PlanArgs {
    /// Let a delta capability shared with an earlier change push the wave and block it again (the rule before requirement-level overlap)
    #[arg(long)]
    pub strict_overlap: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ChangeVerbArgs {
    #[command(subcommand)]
    pub command: ChangeCommands,
}

#[derive(Subcommand, Debug)]
pub enum ChangeCommands {
    /// Declare (or with --remove, drop) prerequisite changes in a change's metadata
    Depends {
        /// Change name
        name: String,
        /// Prerequisite change name(s)
        #[arg(long, num_args = 1.., required = true, value_name = "OTHER")]
        on: Vec<String>,
        /// Remove the given prerequisites instead of adding them
        #[arg(long)]
        remove: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Switch a change's workflow schema in its metadata
    Schema {
        /// Change name
        name: String,
        /// Schema name to switch to
        schema: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Move a change right before or after another change of its stage by rewriting its board rank
    #[command(group(clap::ArgGroup::new("side").required(true).args(["before", "after"])))]
    Rank {
        /// Change name
        name: String,
        /// Place it right before this change
        #[arg(long, value_name = "OTHER")]
        before: Option<String>,
        /// Place it right after this change
        #[arg(long, value_name = "OTHER")]
        after: Option<String>,
        /// Move it even when it already has a board rank (someone ordered it by hand)
        #[arg(long)]
        force: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Report the commit a change's code landed on (the checkout's HEAD when --commit is not given)
    Landed {
        /// Change name
        name: String,
        /// The commit the code landed on
        #[arg(long, value_name = "SHA")]
        commit: Option<String>,
        /// The change's branch; replaces the recorded one
        #[arg(long, value_name = "NAME")]
        branch: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Free fields of a change: your own key/value pairs, stored and never read by speclink
    Field(super::fields::FieldArgs),
}

impl ToCommand for PlanArgs {
    /// `plan` takes the caller's rank table; `--strict-overlap` plans on each
    /// change's own rank.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(if self.strict_overlap {
            Command::PlanStrict
        } else {
            Command::Plan { ranks: inputs.ranks }
        }))
    }
}

/// What `change landed` answers when there is no commit to report: no
/// `--commit`, and the caller has no HEAD to give. A host without a work tree
/// reads this too, so it names the flag first.
pub const NO_COMMIT_TO_REPORT: &str = "no commit to report: pass --commit <sha>, or run this inside a git work tree";

/// The anchor of `change rank` and the side it names. clap makes exactly one
/// of `--before` and `--after` present.
pub fn rank_anchor(before: &Option<String>, after: &Option<String>) -> (String, Side) {
    match (before, after) {
        (Some(anchor), _) => (anchor.clone(), Side::Before),
        (None, Some(anchor)) => (anchor.clone(), Side::After),
        (None, None) => unreachable!("clap requires --before or --after"),
    }
}

impl ChangeVerbArgs {
    pub fn name(&self) -> &'static str {
        match &self.command {
            ChangeCommands::Depends { .. } => "change depends",
            ChangeCommands::Schema { .. } => "change schema",
            ChangeCommands::Rank { .. } => "change rank",
            ChangeCommands::Landed { .. } => "change landed",
            ChangeCommands::Field(a) => a.name(FieldKind::Change),
        }
    }
}

impl ToCommand for ChangeVerbArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(match &self.command {
            ChangeCommands::Depends { name, on, remove, .. } => {
                CommandTarget::Command(Command::ChangeDepends {
                    name: name.clone(),
                    on: on.clone(),
                    remove: *remove,
                })
            }
            ChangeCommands::Schema { name, schema, .. } => {
                CommandTarget::Command(Command::ChangeSchema { name: name.clone(), schema: schema.clone() })
            }
            ChangeCommands::Rank { name, before, after, force, .. } => {
                let (anchor, side) = rank_anchor(before, after);
                CommandTarget::Command(Command::ChangeRank { name: name.clone(), anchor, position: side, force: *force })
            }
            // The branch comes from `--branch` only: after a merge the caller's
            // checkout is on the main branch, which is not the change's branch.
            ChangeCommands::Landed { name, commit, branch, .. } => {
                let Some(commit) = commit.clone().or(inputs.head_commit) else {
                    return Err(CommandError::new(ErrorCode::InvalidArgv, NO_COMMIT_TO_REPORT));
                };
                CommandTarget::Command(Command::ChangeLanded { name: name.clone(), commit, branch: branch.clone() })
            }
            ChangeCommands::Field(a) => CommandTarget::Command(a.command(FieldKind::Change)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};
    use speclink_core::plan::Side;
    use std::collections::BTreeMap;

    #[test]
    fn plan_carries_the_callers_ranks() {
        let ranks = BTreeMap::from([("demo".to_string(), "a0".to_string())]);
        assert_eq!(
            command_of(&["plan", "--json"], VerbInputs { ranks: Some(ranks.clone()), ..Default::default() }),
            Command::Plan { ranks: Some(ranks) }
        );
        assert_eq!(command_of(&["plan"], VerbInputs::default()), Command::Plan { ranks: None });
        assert_eq!(command_of(&["plan", "--strict-overlap"], VerbInputs::default()), Command::PlanStrict);
    }

    #[test]
    fn change_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["change", "depends", "demo", "--on", "a", "b", "--remove", "--json"],
                Command::ChangeDepends { name: "demo".into(), on: vec!["a".into(), "b".into()], remove: true },
            ),
            (
                &["change", "depends", "demo", "--on=a"],
                Command::ChangeDepends { name: "demo".into(), on: vec!["a".into()], remove: false },
            ),
            (
                &["change", "schema", "demo", "spec-driven"],
                Command::ChangeSchema { name: "demo".into(), schema: "spec-driven".into() },
            ),
            (
                &["change", "rank", "demo", "--before", "other"],
                Command::ChangeRank { name: "demo".into(), anchor: "other".into(), position: Side::Before, force: false },
            ),
            (
                &["change", "rank", "demo", "--after=other", "--force", "--json"],
                Command::ChangeRank { name: "demo".into(), anchor: "other".into(), position: Side::After, force: true },
            ),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

    fn with_head(commit: &str) -> VerbInputs {
        VerbInputs { head_commit: Some(commit.into()), ..Default::default() }
    }

    #[test]
    fn change_landed_takes_the_commit_from_the_flag_then_from_the_callers_head() {
        let landed = |commit: &str, branch: Option<&str>| Command::ChangeLanded {
            name: "demo".into(),
            commit: commit.into(),
            branch: branch.map(str::to_string),
        };
        assert_eq!(
            command_of(&["change", "landed", "demo", "--commit", SHA, "--branch", "speclink/demo", "--json"], VerbInputs::default()),
            landed(SHA, Some("speclink/demo"))
        );
        assert_eq!(command_of(&["change", "landed", "demo"], with_head("abc1234")), landed("abc1234", None));
        assert_eq!(
            command_of(&["change", "landed", "demo", "--commit=0123456"], with_head("abc1234")),
            landed("0123456", None),
            "the flag wins over the caller's HEAD"
        );
    }

    #[test]
    fn change_landed_never_takes_the_branch_from_the_caller() {
        // After a merge the checkout is on the main branch: that is not the
        // change's branch, so only `--branch` names one.
        let inputs = VerbInputs { branch: Some("main".into()), ..with_head("abc1234") };
        assert_eq!(
            command_of(&["change", "landed", "demo"], inputs),
            Command::ChangeLanded { name: "demo".into(), commit: "abc1234".into(), branch: None }
        );
    }

    #[test]
    fn change_landed_without_a_commit_is_invalid_argv_and_says_how_to_give_one() {
        let err = target_of(&["change", "landed", "demo"], VerbInputs::default()).expect_err("no commit");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
        assert!(err.message.contains("--commit") && err.message.contains("git work tree"), "{}", err.message);
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [
            (&["plan"][..], "plan"),
            (&["change", "depends", "x", "--on", "y"], "change depends"),
            (&["change", "schema", "x", "y"], "change schema"),
            (&["change", "rank", "x", "--after", "y"], "change rank"),
            (&["change", "landed", "x", "--commit", "0123456"], "change landed"),
        ] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

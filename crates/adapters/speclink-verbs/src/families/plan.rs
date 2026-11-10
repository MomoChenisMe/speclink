//! Execution-order verbs: `plan`, `change depends`, `change schema`, `change rank`.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, VerbInputs};

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
}

impl PlanArgs {
    /// `plan` takes the caller's rank table; `--strict-overlap` plans on each
    /// change's own rank.
    pub fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(if self.strict_overlap {
            Command::PlanStrict
        } else {
            Command::Plan { ranks: inputs.ranks }
        }))
    }
}

impl ChangeVerbArgs {
    /// `change rank` — the one member of the family without an engine command.
    pub fn is_rank(&self) -> bool {
        matches!(self.command, ChangeCommands::Rank { .. })
    }

    pub fn name(&self) -> &'static str {
        match self.command {
            ChangeCommands::Depends { .. } => "change depends",
            ChangeCommands::Schema { .. } => "change schema",
            ChangeCommands::Rank { .. } => "change rank",
        }
    }

    /// `change rank` rewrites board ranks across the checkout's worktree
    /// copies, so it has no engine command.
    pub fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
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
            ChangeCommands::Rank { .. } => CommandTarget::LocalOnly { verb: "change rank" },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, local_only_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;
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
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn change_rank_is_local_only() {
        assert_eq!(local_only_of(&["change", "rank", "demo", "--before", "other"]), Some("change rank"));
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [
            (&["plan"][..], "plan"),
            (&["change", "depends", "x", "--on", "y"], "change depends"),
            (&["change", "schema", "x", "y"], "change schema"),
            (&["change", "rank", "x", "--after", "y"], "change rank"),
        ] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

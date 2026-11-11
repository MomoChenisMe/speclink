//! Read verbs: list, show, status.

use clap::Args;
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show only specs
    #[arg(long)]
    pub specs: bool,
    /// Show only changes
    #[arg(long)]
    pub changes: bool,
    /// Sort by: name, modified, created
    #[arg(long, default_value = "modified")]
    pub sort: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Item to show (change or spec name)
    pub item: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
    /// Item type: change, spec
    #[arg(long = "item-type", value_name = "type")]
    pub item_type: Option<String>,
    /// Show only delta specs
    #[arg(long = "deltas-only")]
    pub deltas_only: bool,
    /// Show requirements
    #[arg(short = 'r', long)]
    pub requirements: bool,
}
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Change name
    #[arg(long)]
    pub change: Option<String>,
    /// Schema name
    #[arg(long)]
    pub schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

impl ToCommand for ListArgs {
    /// `list` — the caller's worktree facts ride along.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::List {
            sort: self.sort.clone(),
            specs: self.specs,
            changes: self.changes,
            worktrees: inputs.worktrees,
        }))
    }
}

impl ToCommand for ShowArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Show {
            item: self.item.clone(),
            item_type: self.item_type.clone(),
        }))
    }
}

impl ToCommand for StatusArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Status {
            change: self.change.clone(),
            schema: self.schema.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, local_only_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;
    use speclink_core::listing::ListWorktreeJson;
    use std::collections::BTreeMap;

    fn list(sort: &str, specs: bool, changes: bool) -> Command {
        Command::List { sort: sort.into(), specs, changes, worktrees: BTreeMap::new() }
    }

    #[test]
    fn list_commands() {
        let cases: &[(&[&str], Command)] = &[
            (&["list"], list("modified", false, false)),
            (&["list", "--json"], list("modified", false, false)),
            (&["list", "--specs", "--changes", "--sort", "name"], list("name", true, true)),
            (&["list", "--sort=name"], list("name", false, false)),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn list_carries_the_callers_worktrees() {
        let facts = BTreeMap::from([(
            "alpha".to_string(),
            ListWorktreeJson { path: "/w/alpha".into(), branch: "speclink/alpha".into() },
        )]);
        let got = command_of(&["list"], VerbInputs { worktrees: facts.clone(), ..Default::default() });
        assert_eq!(
            got,
            Command::List { sort: "modified".into(), specs: false, changes: false, worktrees: facts }
        );
    }

    #[test]
    fn show_and_status_commands() {
        let cases: &[(&[&str], Command)] = &[
            (&["show"], Command::Show { item: None, item_type: None }),
            (
                &["show", "alpha", "--item-type", "change", "--json", "-r", "--deltas-only"],
                Command::Show { item: Some("alpha".into()), item_type: Some("change".into()) },
            ),
            (&["status"], Command::Status { change: None, schema: None }),
            (
                &["status", "--change", "alpha", "--schema", "spec-driven", "--json"],
                Command::Status { change: Some("alpha".into()), schema: Some("spec-driven".into()) },
            ),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [(&["list"][..], "list"), (&["show", "x"], "show"), (&["status"], "status")] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
            assert!(local_only_of(argv).is_none(), "{argv:?}");
        }
    }
}

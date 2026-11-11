//! Read-only check verbs: validate, analyze, drift.

use clap::Args;
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Item to validate
    pub item: Option<String>,
    /// Validate all items
    #[arg(long)]
    pub all: bool,
    /// Validate only changes
    #[arg(long)]
    pub changes: bool,
    /// Validate only specs
    #[arg(long)]
    pub specs: bool,
    /// Strict mode
    #[arg(long)]
    pub strict: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
// The argv of `analyze` and `drift`.
#[derive(Args, Debug)]
pub struct ChangeArg {
    /// Change name (auto-detects if only one exists)
    pub change: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

impl ToCommand for ValidateArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Validate {
            item: self.item.clone(),
            all: self.all,
            changes: self.changes,
            specs: self.specs,
            strict: self.strict,
        }))
    }
}

impl ChangeArg {
    /// `analyze [change]`. (`drift` shares these args and has no command.)
    pub fn analyze_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Analyze { change: self.change.clone() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, local_only_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    #[test]
    fn validate_and_analyze_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["validate"],
                Command::Validate { item: None, all: false, changes: false, specs: false, strict: false },
            ),
            (
                &["validate", "alpha", "--all", "--changes", "--specs", "--strict", "--json"],
                Command::Validate {
                    item: Some("alpha".into()),
                    all: true,
                    changes: true,
                    specs: true,
                    strict: true,
                },
            ),
            (&["analyze"], Command::Analyze { change: None }),
            (&["analyze", "alpha", "--json"], Command::Analyze { change: Some("alpha".into()) }),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn drift_is_local_only() {
        assert_eq!(local_only_of(&["drift", "alpha"]), Some("drift"));
        assert_eq!(local_only_of(&["drift"]), Some("drift"));
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [(&["validate"][..], "validate"), (&["analyze"], "analyze"), (&["drift"], "drift")] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

//! `evidence show` — one change's completion evidence.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct EvidenceArgs {
    #[command(subcommand)]
    pub command: EvidenceCommands,
}
#[derive(Subcommand, Debug)]
pub enum EvidenceCommands {
    /// Show the files each completed task of a change touched (an archived change by its archived name or its own name)
    Show {
        /// Change name
        change: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

impl EvidenceArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            EvidenceCommands::Show { .. } => "evidence show",
        }
    }
}

impl ToCommand for EvidenceArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match &self.command {
            EvidenceCommands::Show { change, .. } => {
                Ok(CommandTarget::Command(Command::EvidenceShow { change: change.clone() }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    #[test]
    fn evidence_show_command() {
        for argv in [
            &["evidence", "show", "demo"][..],
            &["evidence", "show", "demo", "--json"],
            &["evidence", "show", "demo", "--no-color"],
            &["--no-color", "evidence", "show", "demo", "--json"],
        ] {
            assert_eq!(
                command_of(argv, VerbInputs::default()),
                Command::EvidenceShow { change: "demo".into() },
                "{argv:?}"
            );
        }
        // An archived name is a name like any other.
        assert_eq!(
            command_of(&["evidence", "show", "2026-11-12-demo"], VerbInputs::default()),
            Command::EvidenceShow { change: "2026-11-12-demo".into() }
        );
    }

    #[test]
    fn name_and_stdin_policy() {
        assert_eq!(name_of(&["evidence", "show", "demo"]), "evidence show");
        assert_eq!(policy_of(&["evidence", "show", "demo", "--json"]), StdinPolicy::Never);
    }

    #[test]
    fn the_json_flag_is_read_from_argv() {
        use crate::families::evidence::EvidenceCommands;
        use crate::SpecVerb;
        for (argv, want) in [(&["evidence", "show", "demo"][..], false), (&["evidence", "show", "demo", "--json"], true)] {
            let parsed = crate::parse(argv.iter().copied()).expect("parses");
            match parsed.spec() {
                SpecVerb::Evidence(a) => match &a.command {
                    EvidenceCommands::Show { change, json } => {
                        assert_eq!(change, "demo");
                        assert_eq!(*json, want, "{argv:?}");
                    }
                },
                other => panic!("expected the evidence family, got {other:?}"),
            }
        }
    }

    #[test]
    fn a_missing_change_name_is_a_usage_error() {
        let err = crate::parse(["evidence", "show"]).err().expect("the change name is required");
        assert_eq!(err.kind, crate::ParseErrorKind::Usage);
    }
}

//! The instructions verb: artifact guidance and the apply payload.

use clap::Args;
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, LocalOnlyReason, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct InstructionsArgs {
    /// Artifact ID or "apply"
    pub artifact: Option<String>,
    /// Change name
    #[arg(long)]
    pub change: Option<String>,
    /// Schema name
    #[arg(long)]
    pub schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
    /// Embedded skill name (outputs skill body directly)
    #[arg(long)]
    pub skill: Option<String>,
}

impl ToCommand for InstructionsArgs {
    /// `--skill` prints a constant skill body that lives outside the store,
    /// so it has no engine command.
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        if self.skill.is_some() {
            return Ok(CommandTarget::LocalOnly {
                verb: "instructions --skill",
                reason: LocalOnlyReason::CliSkillText,
            });
        }
        Ok(CommandTarget::Command(Command::Instructions {
            artifact: self.artifact.clone(),
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

    #[test]
    fn instructions_commands() {
        let cases: &[(&[&str], Command)] = &[
            (&["instructions"], Command::Instructions { artifact: None, change: None, schema: None }),
            (
                &["instructions", "apply", "--change", "alpha", "--json"],
                Command::Instructions {
                    artifact: Some("apply".into()),
                    change: Some("alpha".into()),
                    schema: None,
                },
            ),
            (
                &["instructions", "design", "--change", "alpha", "--schema", "spec-driven"],
                Command::Instructions {
                    artifact: Some("design".into()),
                    change: Some("alpha".into()),
                    schema: Some("spec-driven".into()),
                },
            ),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn skill_is_local_only() {
        assert_eq!(local_only_of(&["instructions", "--skill", "tdd"]), Some("instructions --skill"));
        assert!(local_only_of(&["instructions", "apply"]).is_none());
    }

    #[test]
    fn name_and_stdin_policy() {
        assert_eq!(name_of(&["instructions"]), "instructions");
        assert_eq!(policy_of(&["instructions", "--skill", "tdd"]), StdinPolicy::Never);
    }
}

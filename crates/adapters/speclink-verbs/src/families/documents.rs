//! Document-read verbs: `artifact cat` and `language show`.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, VerbInputs};

#[derive(Args, Debug)]
pub struct ArtifactArgs {
    #[command(subcommand)]
    pub command: ArtifactCommands,
}
#[derive(Subcommand, Debug)]
pub enum ArtifactCommands {
    /// Print an artifact's content (proposal | design | tasks | specs/<capability>)
    Cat {
        /// Artifact id
        artifact: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
    },
}
#[derive(Args, Debug)]
pub struct LanguageArgs {
    #[command(subcommand)]
    pub command: LanguageCommands,
}
#[derive(Subcommand, Debug)]
pub enum LanguageCommands {
    /// Print the project's shared vocabulary (LANGUAGE document)
    Show,
}

impl ArtifactArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            ArtifactCommands::Cat { .. } => "artifact cat",
        }
    }

    pub fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match &self.command {
            ArtifactCommands::Cat { artifact, change } => {
                Ok(CommandTarget::Command(Command::ArtifactCat {
                    artifact: artifact.clone(),
                    change: change.clone(),
                }))
            }
        }
    }
}

impl LanguageArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            LanguageCommands::Show => "language show",
        }
    }

    pub fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match self.command {
            LanguageCommands::Show => Ok(CommandTarget::Command(Command::LanguageShow)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    #[test]
    fn artifact_and_language_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["artifact", "cat", "proposal"],
                Command::ArtifactCat { artifact: "proposal".into(), change: None },
            ),
            (
                &["artifact", "cat", "specs/auth", "--change", "alpha"],
                Command::ArtifactCat { artifact: "specs/auth".into(), change: Some("alpha".into()) },
            ),
            (&["language", "show"], Command::LanguageShow),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [(&["artifact", "cat", "tasks"][..], "artifact cat"), (&["language", "show"], "language show")] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

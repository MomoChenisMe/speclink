//! Document verbs: `artifact cat`, `language show` and `language replace`.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, StdinPolicy, ToCommand, VerbInputs};

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
    /// Replace the whole LANGUAGE document with the content from stdin (creates it when absent)
    Replace {
        /// Read the whole document from stdin
        #[arg(long, required = true)]
        stdin: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

impl ArtifactArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            ArtifactCommands::Cat { .. } => "artifact cat",
        }
    }
}

impl ToCommand for ArtifactArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
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
            LanguageCommands::Replace { .. } => "language replace",
        }
    }

    pub fn stdin_policy(&self) -> StdinPolicy {
        match self.command {
            LanguageCommands::Show => StdinPolicy::Never,
            LanguageCommands::Replace { stdin, .. } => StdinPolicy::WhenFlagged { flagged: stdin },
        }
    }
}

impl ToCommand for LanguageArgs {
    /// Unread stdin is empty content; the engine refuses an empty document.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(match self.command {
            LanguageCommands::Show => Command::LanguageShow,
            LanguageCommands::Replace { .. } => {
                Command::LanguageReplace { content: inputs.stdin.unwrap_or_default() }
            }
        }))
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

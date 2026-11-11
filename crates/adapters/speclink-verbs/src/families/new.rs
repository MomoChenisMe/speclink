//! Creation verbs: `new change` and `new artifact`.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError, ErrorCode};

use crate::{CommandTarget, StdinPolicy, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct NewArgs {
    #[command(subcommand)]
    pub command: NewCommands,
}
#[derive(Subcommand, Debug)]
pub enum NewCommands {
    /// Create a new change
    Change(NewChangeArgs),
    /// Create a new artifact file for a change
    Artifact(NewArtifactArgs),
}
#[derive(Args, Debug)]
pub struct NewChangeArgs {
    /// Change name (kebab-case)
    pub name: String,
    /// Description
    #[arg(long)]
    pub description: Option<String>,
    /// Workflow schema to use
    #[arg(long)]
    pub schema: Option<String>,
    /// AI agent that created this change (e.g., claude, codex, gemini)
    #[arg(long)]
    pub agent: Option<String>,
    /// Link this change to a discussion document (writes from_discussion metadata)
    #[arg(long = "from-discussion")]
    pub from_discussion: Option<String>,
    /// This change is the last cut the discussion's conclusion planned: release the
    /// record's hold so the final archive co-archives the discussion
    #[arg(long, requires = "from_discussion")]
    pub last: bool,
}
#[derive(Args, Debug)]
pub struct NewArtifactArgs {
    /// Artifact type: proposal, design, tasks, spec
    #[arg(name = "TYPE")]
    pub artifact_type: String,
    /// Capability name (required for spec type)
    pub capability: Option<String>,
    /// Change name
    #[arg(long)]
    pub change: Option<String>,
    /// Read content from stdin instead of using empty template
    #[arg(long)]
    pub stdin: bool,
    /// Overwrite existing artifact
    #[arg(long)]
    pub force: bool,
    /// Declare a new capability (required when the spec capability is not in the canonical specs)
    #[arg(long)]
    pub new: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

impl NewArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            NewCommands::Change(_) => "new change",
            NewCommands::Artifact(_) => "new artifact",
        }
    }

    pub fn stdin_policy(&self) -> StdinPolicy {
        match &self.command {
            NewCommands::Change(_) => StdinPolicy::Never,
            NewCommands::Artifact(a) => a.stdin_policy(),
        }
    }
}

impl ToCommand for NewArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match &self.command {
            NewCommands::Change(a) => a.to_command(inputs),
            NewCommands::Artifact(a) => a.to_command(inputs),
        }
    }
}

impl ToCommand for NewChangeArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::NewChange {
            name: self.name.clone(),
            description: self.description.clone(),
            schema: self.schema.clone(),
            agent: self.agent.clone(),
            from_discussion: self.from_discussion.clone(),
            last: self.last,
        }))
    }
}

impl NewArtifactArgs {
    pub fn stdin_policy(&self) -> StdinPolicy {
        StdinPolicy::WhenFlagged { flagged: self.stdin }
    }
}

impl ToCommand for NewArtifactArgs {
    /// With `--stdin` the caller's stdin becomes the content; without it the
    /// schema template is written and any given stdin is ignored.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        let content = match (self.stdin, inputs.stdin) {
            (false, _) => None,
            (true, Some(content)) => Some(content),
            (true, None) => {
                return Err(CommandError::new(
                    ErrorCode::InvalidArgv,
                    "new artifact --stdin needs the content from stdin, and none was given",
                ))
            }
        };
        Ok(CommandTarget::Command(Command::NewArtifact {
            kind: self.artifact_type.clone(),
            capability: self.capability.clone(),
            change: self.change.clone(),
            content,
            force: self.force,
            new_capability: self.new,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};

    fn with_stdin(content: &str) -> VerbInputs {
        VerbInputs { stdin: Some(content.into()), ..Default::default() }
    }

    #[test]
    fn new_change_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["new", "change", "demo"],
                Command::NewChange {
                    name: "demo".into(),
                    description: None,
                    schema: None,
                    agent: None,
                    from_discussion: None,
                    last: false,
                },
            ),
            (
                &[
                    "new", "change", "demo", "--description", "d", "--schema=spec-driven", "--agent",
                    "claude", "--from-discussion", "topic", "--last",
                ],
                Command::NewChange {
                    name: "demo".into(),
                    description: Some("d".into()),
                    schema: Some("spec-driven".into()),
                    agent: Some("claude".into()),
                    from_discussion: Some("topic".into()),
                    last: true,
                },
            ),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
        assert!(crate::parse(["new", "change", "demo", "--last"]).is_err(), "--last needs --from-discussion");
    }

    #[test]
    fn new_artifact_takes_the_callers_stdin_only_when_flagged() {
        assert_eq!(
            command_of(&["new", "artifact", "proposal", "--change", "demo", "--stdin"], with_stdin("body")),
            Command::NewArtifact {
                kind: "proposal".into(),
                capability: None,
                change: Some("demo".into()),
                content: Some("body".into()),
                force: false,
                new_capability: false,
            }
        );
        assert_eq!(
            command_of(&["new", "artifact", "spec", "auth", "--change", "demo", "--force", "--new", "--json"], with_stdin("ignored")),
            Command::NewArtifact {
                kind: "spec".into(),
                capability: Some("auth".into()),
                change: Some("demo".into()),
                content: None,
                force: true,
                new_capability: true,
            }
        );
    }

    #[test]
    fn new_artifact_with_stdin_flag_but_no_content_is_invalid_argv() {
        let err = target_of(&["new", "artifact", "proposal", "--change", "demo", "--stdin"], VerbInputs::default())
            .expect_err("--stdin without content");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
    }

    #[test]
    fn names_and_stdin_policies() {
        assert_eq!(name_of(&["new", "change", "x"]), "new change");
        assert_eq!(name_of(&["new", "artifact", "tasks"]), "new artifact");
        assert_eq!(policy_of(&["new", "change", "x"]), StdinPolicy::Never);
        assert_eq!(
            policy_of(&["new", "artifact", "proposal", "--change", "demo", "--stdin"]),
            StdinPolicy::WhenFlagged { flagged: true }
        );
        assert_eq!(policy_of(&["new", "artifact", "tasks"]), StdinPolicy::WhenFlagged { flagged: false });
    }
}

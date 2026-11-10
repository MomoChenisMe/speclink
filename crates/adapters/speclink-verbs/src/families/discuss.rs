//! The discuss family: every discussion subcommand.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, StdinPolicy, VerbInputs};

#[derive(Args, Debug)]
pub struct DiscussArgs {
    #[command(subcommand)]
    pub command: DiscussCommands,
}
#[derive(Subcommand, Debug)]
pub enum DiscussCommands {
    /// Create a new discussion document
    New {
        topic: String,
        /// Override the record slug (ASCII kebab-case); the topic stays verbatim
        #[arg(long)]
        slug: Option<String>,
        /// Mark the record's type (only: improve)
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List discussions
    List {
        /// Show archived discussions instead of live ones
        #[arg(long)]
        archived: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a discussion document
    Show { slug: String, #[arg(long)] json: bool },
    /// Search live and archived discussions by keyword (topic, slug and decision lines)
    Search {
        /// Keywords (space-split); any one matching counts, case-insensitive
        #[arg(required = true, value_name = "KEYWORD")]
        terms: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Set the discussion's Context section (content from stdin)
    Context { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Append a round to a discussion (content from stdin)
    #[command(name = "add-round")]
    AddRound { slug: String, #[arg(long, default_value = "interview")] mode: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Conclude a discussion (content from stdin)
    Conclude {
        slug: String,
        #[arg(long)]
        stdin: bool,
        /// Keep the record live: its conclusion plans a further change to spin out later
        #[arg(long)]
        hold: bool,
        #[arg(long)]
        json: bool,
    },
    /// Archive a discussion (move to discussions/archive/<created>-<slug>.md)
    Archive { slug: String, #[arg(long)] json: bool },
    /// Discard a live discussion (delete the file; --force required once rounds exist)
    Discard {
        slug: String,
        /// Delete even when the discussion has recorded rounds
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Promote a discussion into a change scaffold (proposal prefilled from the conclusion)
    Promote {
        slug: String,
        /// Change name (defaults to the discussion slug)
        #[arg(long)]
        name: Option<String>,
        /// This is the last cut the conclusion planned: release the record's hold so the
        /// final archive co-archives the discussion
        #[arg(long)]
        last: bool,
        #[arg(long)]
        json: bool,
    },
    /// Link a discussion to an existing change (forges the from_discussion chain)
    Link {
        slug: String,
        /// Existing change name to link the discussion to
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Seal a discussion→change reflection: mark the discussion promoted once content has landed
    Seal {
        slug: String,
        /// Change whose from_discussion already includes this discussion
        change: String,
        /// This is the last cut the conclusion planned: release the record's hold so the
        /// final archive co-archives the discussion
        #[arg(long)]
        last: bool,
        #[arg(long)]
        json: bool,
    },
}

impl DiscussArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            DiscussCommands::New { .. } => "discuss new",
            DiscussCommands::List { .. } => "discuss list",
            DiscussCommands::Show { .. } => "discuss show",
            DiscussCommands::Search { .. } => "discuss search",
            DiscussCommands::Context { .. } => "discuss context",
            DiscussCommands::AddRound { .. } => "discuss add-round",
            DiscussCommands::Conclude { .. } => "discuss conclude",
            DiscussCommands::Archive { .. } => "discuss archive",
            DiscussCommands::Discard { .. } => "discuss discard",
            DiscussCommands::Promote { .. } => "discuss promote",
            DiscussCommands::Link { .. } => "discuss link",
            DiscussCommands::Seal { .. } => "discuss seal",
        }
    }

    /// The three content verbs read stdin with `--stdin` or when it is piped.
    pub fn stdin_policy(&self) -> StdinPolicy {
        match self.command {
            DiscussCommands::Context { stdin, .. }
            | DiscussCommands::AddRound { stdin, .. }
            | DiscussCommands::Conclude { stdin, .. } => StdinPolicy::FlagOrPiped { flagged: stdin },
            DiscussCommands::New { .. }
            | DiscussCommands::List { .. }
            | DiscussCommands::Show { .. }
            | DiscussCommands::Search { .. }
            | DiscussCommands::Archive { .. }
            | DiscussCommands::Discard { .. }
            | DiscussCommands::Promote { .. }
            | DiscussCommands::Link { .. }
            | DiscussCommands::Seal { .. } => StdinPolicy::Never,
        }
    }

    /// Unread stdin is empty content; the engine's content guard rejects it.
    pub fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        let content = || inputs.stdin.clone().unwrap_or_default();
        Ok(CommandTarget::Command(match &self.command {
            DiscussCommands::New { topic, slug, kind, .. } => Command::DiscussNew {
                topic: topic.clone(),
                slug: slug.clone(),
                kind: kind.clone(),
            },
            DiscussCommands::List { archived, .. } => Command::DiscussList { archived: *archived },
            DiscussCommands::Show { slug, .. } => Command::DiscussShow { slug: slug.clone() },
            DiscussCommands::Search { terms, .. } => Command::DiscussSearch { terms: terms.clone() },
            DiscussCommands::Context { slug, .. } => {
                Command::DiscussContext { slug: slug.clone(), content: content() }
            }
            DiscussCommands::AddRound { slug, mode, .. } => Command::DiscussAddRound {
                slug: slug.clone(),
                mode: mode.clone(),
                content: content(),
            },
            DiscussCommands::Conclude { slug, hold, .. } => Command::DiscussConclude {
                slug: slug.clone(),
                content: content(),
                hold: *hold,
            },
            DiscussCommands::Archive { slug, .. } => Command::DiscussArchive { slug: slug.clone() },
            DiscussCommands::Discard { slug, force, .. } => {
                Command::DiscussDiscard { slug: slug.clone(), force: *force }
            }
            DiscussCommands::Promote { slug, name, last, .. } => Command::DiscussPromote {
                slug: slug.clone(),
                name: name.clone(),
                last: *last,
            },
            DiscussCommands::Link { slug, change, .. } => {
                Command::DiscussLink { slug: slug.clone(), change: change.clone() }
            }
            DiscussCommands::Seal { slug, change, last, .. } => Command::DiscussSeal {
                slug: slug.clone(),
                change: change.clone(),
                last: *last,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    fn with_stdin(content: &str) -> VerbInputs {
        VerbInputs { stdin: Some(content.into()), ..Default::default() }
    }

    #[test]
    fn every_discuss_terminal_converts() {
        let cases: &[(&[&str], VerbInputs, Command)] = &[
            (
                &["discuss", "new", "Rate limits", "--slug", "rate-limits", "--kind", "improve", "--json"],
                VerbInputs::default(),
                Command::DiscussNew {
                    topic: "Rate limits".into(),
                    slug: Some("rate-limits".into()),
                    kind: Some("improve".into()),
                },
            ),
            (&["discuss", "list", "--archived"], VerbInputs::default(), Command::DiscussList { archived: true }),
            (&["discuss", "show", "topic"], VerbInputs::default(), Command::DiscussShow { slug: "topic".into() }),
            (
                &["discuss", "search", "auth", "quota"],
                VerbInputs::default(),
                Command::DiscussSearch { terms: vec!["auth".into(), "quota".into()] },
            ),
            (
                &["discuss", "context", "topic", "--stdin"],
                with_stdin("ctx"),
                Command::DiscussContext { slug: "topic".into(), content: "ctx".into() },
            ),
            (
                &["discuss", "add-round", "topic"],
                with_stdin("round"),
                Command::DiscussAddRound { slug: "topic".into(), mode: "interview".into(), content: "round".into() },
            ),
            (
                &["discuss", "add-round", "topic", "--mode", "debate", "--stdin"],
                with_stdin("round"),
                Command::DiscussAddRound { slug: "topic".into(), mode: "debate".into(), content: "round".into() },
            ),
            (
                &["discuss", "conclude", "topic", "--hold"],
                with_stdin("done"),
                Command::DiscussConclude { slug: "topic".into(), content: "done".into(), hold: true },
            ),
            (&["discuss", "archive", "topic"], VerbInputs::default(), Command::DiscussArchive { slug: "topic".into() }),
            (
                &["discuss", "discard", "topic", "--force"],
                VerbInputs::default(),
                Command::DiscussDiscard { slug: "topic".into(), force: true },
            ),
            (
                &["discuss", "promote", "topic", "--name", "add-x", "--last"],
                VerbInputs::default(),
                Command::DiscussPromote { slug: "topic".into(), name: Some("add-x".into()), last: true },
            ),
            (
                &["discuss", "link", "topic", "add-x"],
                VerbInputs::default(),
                Command::DiscussLink { slug: "topic".into(), change: "add-x".into() },
            ),
            (
                &["discuss", "seal", "topic", "add-x", "--last"],
                VerbInputs::default(),
                Command::DiscussSeal { slug: "topic".into(), change: "add-x".into(), last: true },
            ),
        ];
        for (argv, inputs, want) in cases {
            let inputs = VerbInputs { stdin: inputs.stdin.clone(), ..Default::default() };
            assert_eq!(&command_of(argv, inputs), want, "{argv:?}");
        }
    }

    /// Unread stdin reads as empty content: the engine's content guard
    /// rejects it with its own message, as the CLI on a terminal does today.
    #[test]
    fn content_verbs_without_stdin_carry_empty_content() {
        assert_eq!(
            command_of(&["discuss", "context", "topic"], VerbInputs::default()),
            Command::DiscussContext { slug: "topic".into(), content: String::new() }
        );
    }

    #[test]
    fn stdin_policies() {
        assert_eq!(policy_of(&["discuss", "context", "t"]), StdinPolicy::FlagOrPiped { flagged: false });
        assert_eq!(policy_of(&["discuss", "add-round", "t", "--stdin"]), StdinPolicy::FlagOrPiped { flagged: true });
        assert_eq!(policy_of(&["discuss", "conclude", "t"]), StdinPolicy::FlagOrPiped { flagged: false });
        for argv in [&["discuss", "list"][..], &["discuss", "show", "t"], &["discuss", "new", "x"]] {
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }

    #[test]
    fn names() {
        assert_eq!(name_of(&["discuss", "add-round", "t"]), "discuss add-round");
        assert_eq!(name_of(&["discuss", "seal", "t", "c"]), "discuss seal");
    }
}

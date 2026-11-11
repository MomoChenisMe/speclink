//! Workflow config verbs: `workflow-config show` and the ten write verbs
//! (`set`, `context`, `rules`, `review …`, `subagent-model …`).

use clap::{Args, Subcommand, ValueEnum};
use speclink_core::command::{Command, CommandError, ErrorCode};
use speclink_core::config::{
    PolicyKey, ReviewBaseline, ReviewList, SubagentStation, ToolName, WorkflowConfigEdit,
};

use crate::{CommandTarget, StdinPolicy, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct WorkflowConfigArgs {
    #[command(subcommand)]
    pub command: WorkflowConfigCommands,
}
#[derive(Subcommand, Debug)]
pub enum WorkflowConfigCommands {
    /// Show the canonical workflow config (policy fields, context, rules, review)
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    #[command(about = SET_ABOUT.as_str())]
    Set {
        /// Policy key
        key: String,
        // 布林鍵子集刻意維持字面（design D1：程式裡沒有這個子集的常數，單一消費者
        // 不值得再立一個真相來源）；由 set_help_value_argument_names_every_boolean_key 釘住。
        /// Policy value (tdd/audit/worktree take true or false)
        value: String,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Replace the project context from stdin (blank input removes it)
    Context {
        /// Read the content from stdin
        #[arg(long)]
        stdin: bool,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Replace one artifact's rule section from stdin (empty input removes it)
    Rules {
        /// Artifact id of the active schema (proposal, design, specs, tasks, ...)
        artifact: String,
        /// Read the rules from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Edit the project review rules the review station's Standards axis applies
    #[command(subcommand_value_name = REVIEW_COMMANDS.as_str())]
    Review {
        #[command(subcommand)]
        command: ReviewCommands,
    },
    /// Set the model the quality stations' sub-agents use, per station and AI tool
    #[command(subcommand_value_name = SUBAGENT_MODEL_COMMANDS.as_str())]
    SubagentModel {
        #[command(subcommand)]
        command: SubagentModelCommands,
    },
}
#[derive(Subcommand, Debug)]
pub enum SubagentModelCommands {
    /// Set the review station's sub-agent model for one AI tool (an empty model removes it)
    Review(StationModelArgs),
    /// Set the verify station's sub-agent model for one AI tool (an empty model removes it)
    Verify(StationModelArgs),
    /// Replace one AI tool's custom model list from stdin (empty input removes it)
    Choices {
        /// AI tool name in kebab-case (claude, codex, ...)
        tool: String,
        /// Read the model names from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}
#[derive(Args, Debug)]
pub struct StationModelArgs {
    /// AI tool name in kebab-case (claude, codex, ...)
    pub tool: String,
    /// Model name; an empty string removes the setting (the sub-agents follow the main conversation)
    pub model: String,
    /// Print the unified diff instead of writing
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}
#[derive(Subcommand, Debug)]
pub enum ReviewCommands {
    /// Replace the CRITICAL rule list from stdin (empty input removes it)
    Critical(ReviewListArgs),
    /// Replace the WARNING rule list from stdin (empty input removes it)
    Warning(ReviewListArgs),
    /// Replace the SUGGESTION rule list from stdin (empty input removes it)
    Suggestion(ReviewListArgs),
    /// Set whether the built-in smell baseline applies (extend is the default)
    Baseline {
        /// extend: the baseline plus project rules; replace: project rules only
        #[arg(value_enum)]
        mode: BaselineMode,
        /// Print the unified diff instead of writing
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}
#[derive(Args, Debug)]
pub struct ReviewListArgs {
    /// Read the rules from stdin (one per line)
    #[arg(long)]
    pub stdin: bool,
    /// Print the unified diff instead of writing
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BaselineMode {
    Extend,
    Replace,
}

/// What a `review` subcommand edits — the one place its four subcommands map
/// to the engine's terms.
pub enum ReviewTarget<'a> {
    List { list: ReviewList, args: &'a ReviewListArgs },
    Baseline { baseline: ReviewBaseline, dry_run: bool },
}

impl ReviewCommands {
    pub fn target(&self) -> ReviewTarget<'_> {
        let list = |list, args| ReviewTarget::List { list, args };
        match self {
            ReviewCommands::Critical(args) => list(ReviewList::Critical, args),
            ReviewCommands::Warning(args) => list(ReviewList::Warning, args),
            ReviewCommands::Suggestion(args) => list(ReviewList::Suggestion, args),
            ReviewCommands::Baseline { mode, dry_run } => ReviewTarget::Baseline {
                baseline: match mode {
                    BaselineMode::Extend => ReviewBaseline::Extend,
                    BaselineMode::Replace => ReviewBaseline::Replace,
                },
                dry_run: *dry_run,
            },
        }
    }
}

/// What a `subagent-model` subcommand edits, like [`ReviewTarget`].
pub enum SubagentModelTarget<'a> {
    Cell { station: SubagentStation, args: &'a StationModelArgs },
    Choices { tool: &'a str, stdin: bool, dry_run: bool },
}

impl SubagentModelCommands {
    pub fn target(&self) -> SubagentModelTarget<'_> {
        let cell = |station, args| SubagentModelTarget::Cell { station, args };
        match self {
            SubagentModelCommands::Review(args) => cell(SubagentStation::Review, args),
            SubagentModelCommands::Verify(args) => cell(SubagentStation::Verify, args),
            SubagentModelCommands::Choices { tool, stdin, dry_run } => {
                SubagentModelTarget::Choices { tool, stdin: *stdin, dry_run: *dry_run }
            }
        }
    }
}

/// `set`'s clap description, grown from the policy keys so help can never advertise
/// a different key set than the verb accepts. A doc comment is a compile-time
/// literal and cannot interpolate; the derive attribute takes any expression.
static SET_ABOUT: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("Set a policy field: {}", PolicyKey::list()));
/// `review`'s usage placeholder: clap's unknown-subcommand error prints only the
/// usage line, so the legal names must live there — grown from the subcommand
/// definitions, so the usage can never name a different set.
static REVIEW_COMMANDS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    subcommand_names(ReviewCommands::augment_subcommands(clap::Command::new("review")))
});
/// `subagent-model`'s usage placeholder, for the same reason as [`REVIEW_COMMANDS`].
static SUBAGENT_MODEL_COMMANDS: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    subcommand_names(SubagentModelCommands::augment_subcommands(clap::Command::new("subagent-model")))
});
/// A subcommand group's names as one `a|b|c` usage placeholder.
fn subcommand_names(group: clap::Command) -> String {
    group.get_subcommands().map(|c| c.get_name().to_string()).collect::<Vec<_>>().join("|")
}

impl WorkflowConfigArgs {
    pub fn name(&self) -> &'static str {
        use WorkflowConfigCommands as C;
        match &self.command {
            C::Show { .. } => "workflow-config show",
            C::Set { .. } => "workflow-config set",
            C::Context { .. } => "workflow-config context",
            C::Rules { .. } => "workflow-config rules",
            C::Review { command } => match command {
                ReviewCommands::Critical(_) => "workflow-config review critical",
                ReviewCommands::Warning(_) => "workflow-config review warning",
                ReviewCommands::Suggestion(_) => "workflow-config review suggestion",
                ReviewCommands::Baseline { .. } => "workflow-config review baseline",
            },
            C::SubagentModel { command } => match command {
                SubagentModelCommands::Review(_) => "workflow-config subagent-model review",
                SubagentModelCommands::Verify(_) => "workflow-config subagent-model verify",
                SubagentModelCommands::Choices { .. } => "workflow-config subagent-model choices",
            },
        }
    }

    /// A content-taking verb reads stdin only with `--stdin`; the others never.
    pub fn stdin_policy(&self) -> StdinPolicy {
        use WorkflowConfigCommands as C;
        let flagged = match &self.command {
            C::Context { stdin, .. } | C::Rules { stdin, .. } => *stdin,
            C::Review { command } => match command.target() {
                ReviewTarget::List { args, .. } => args.stdin,
                ReviewTarget::Baseline { .. } => return StdinPolicy::Never,
            },
            C::SubagentModel { command } => match command.target() {
                SubagentModelTarget::Choices { stdin, .. } => stdin,
                SubagentModelTarget::Cell { .. } => return StdinPolicy::Never,
            },
            C::Show { .. } | C::Set { .. } => return StdinPolicy::Never,
        };
        StdinPolicy::WhenFlagged { flagged }
    }

    /// Whether `show` was asked for its JSON form (no other verb has one).
    pub fn json(&self) -> bool {
        matches!(self.command, WorkflowConfigCommands::Show { json: true })
    }
}

impl WorkflowConfigCommands {
    /// Whether a write verb carries `--dry-run`; false for `show`.
    pub fn dry_run(&self) -> bool {
        use WorkflowConfigCommands as C;
        match self {
            C::Show { .. } => false,
            C::Set { dry_run, .. } | C::Context { dry_run, .. } | C::Rules { dry_run, .. } => *dry_run,
            C::Review { command } => match command.target() {
                ReviewTarget::List { args, .. } => args.dry_run,
                ReviewTarget::Baseline { dry_run, .. } => dry_run,
            },
            C::SubagentModel { command } => match command.target() {
                SubagentModelTarget::Cell { args, .. } => args.dry_run,
                SubagentModelTarget::Choices { dry_run, .. } => dry_run,
            },
        }
    }
}

/// Content-taking verbs require the flag explicitly: without it the command
/// would silently write an empty document from an interactive terminal.
fn require_stdin_flag(flag: bool, usage: &str) -> Result<(), CommandError> {
    if flag {
        return Ok(());
    }
    Err(CommandError::new(
        ErrorCode::InvalidArgv,
        format!("content is read from stdin — run: speclink workflow-config {usage}"),
    ))
}

/// The tool name, checked with the rule the config parser applies — before
/// the document is read and before stdin is asked for.
fn tool_name(tool: &str) -> Result<ToolName, CommandError> {
    ToolName::parse(tool).map_err(|e| CommandError::new(ErrorCode::InvalidArgv, e))
}

impl ToCommand for WorkflowConfigArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        use WorkflowConfigCommands as C;
        use WorkflowConfigEdit as E;
        let body = || inputs.stdin.clone().unwrap_or_default();
        let edit = match &self.command {
            C::Show { .. } => return Ok(CommandTarget::Command(Command::WorkflowConfigShow)),
            C::Set { key, value, .. } => E::Policy { key: key.clone(), value: value.clone() },
            C::Context { stdin, .. } => {
                require_stdin_flag(*stdin, "context --stdin")?;
                E::Context(body())
            }
            C::Rules { artifact, stdin, .. } => {
                require_stdin_flag(*stdin, &format!("rules {artifact} --stdin"))?;
                E::Rules { artifact: artifact.clone(), body: body() }
            }
            C::Review { command } => match command.target() {
                ReviewTarget::List { list, args } => {
                    require_stdin_flag(args.stdin, &format!("review {} --stdin", list.key()))?;
                    E::ReviewList { list, body: body() }
                }
                ReviewTarget::Baseline { baseline, .. } => E::ReviewBaseline(baseline),
            },
            C::SubagentModel { command } => match command.target() {
                SubagentModelTarget::Cell { station, args } => E::SubagentCell {
                    station,
                    tool: tool_name(&args.tool)?,
                    model: args.model.clone(),
                },
                SubagentModelTarget::Choices { tool, stdin, .. } => {
                    let tool = tool_name(tool)?;
                    require_stdin_flag(stdin, &format!("subagent-model choices {tool} --stdin"))?;
                    E::SubagentChoices { tool, body: body() }
                }
            },
        };
        Ok(CommandTarget::Command(Command::WorkflowConfigEdit {
            edit,
            dry_run: self.command.dry_run(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};
    use speclink_core::config::{
        ReviewBaseline, ReviewList, SubagentStation, ToolName, WorkflowConfigEdit,
    };

    fn stdin(text: &str) -> VerbInputs {
        VerbInputs { stdin: Some(text.to_string()), ..Default::default() }
    }

    fn edit(edit: WorkflowConfigEdit) -> Command {
        Command::WorkflowConfigEdit { edit, dry_run: false }
    }

    fn tool(name: &str) -> ToolName {
        ToolName::parse(name).expect("a kebab-case tool name")
    }

    // The eleven leaves, each with the command it converts to.
    #[test]
    fn every_leaf_converts_to_its_command() {
        use WorkflowConfigEdit as E;
        let rows: Vec<(&[&str], Option<&str>, Command)> = vec![
            (&["workflow-config", "show"], None, Command::WorkflowConfigShow),
            (&["workflow-config", "show", "--json"], None, Command::WorkflowConfigShow),
            (
                &["workflow-config", "set", "tdd", "true"],
                None,
                edit(E::Policy { key: "tdd".into(), value: "true".into() }),
            ),
            (
                &["workflow-config", "context", "--stdin"],
                Some("Line one.\nLine two.\n"),
                edit(E::Context("Line one.\nLine two.\n".into())),
            ),
            (
                &["workflow-config", "rules", "proposal", "--stdin"],
                Some("one\ntwo\n"),
                edit(E::Rules { artifact: "proposal".into(), body: "one\ntwo\n".into() }),
            ),
            (
                &["workflow-config", "review", "critical", "--stdin"],
                Some("c\n"),
                edit(E::ReviewList { list: ReviewList::Critical, body: "c\n".into() }),
            ),
            (
                &["workflow-config", "review", "warning", "--stdin"],
                Some("w\n"),
                edit(E::ReviewList { list: ReviewList::Warning, body: "w\n".into() }),
            ),
            (
                &["workflow-config", "review", "suggestion", "--stdin"],
                Some("s\n"),
                edit(E::ReviewList { list: ReviewList::Suggestion, body: "s\n".into() }),
            ),
            (
                &["workflow-config", "review", "baseline", "replace"],
                None,
                edit(E::ReviewBaseline(ReviewBaseline::Replace)),
            ),
            (
                &["workflow-config", "review", "baseline", "extend"],
                None,
                edit(E::ReviewBaseline(ReviewBaseline::Extend)),
            ),
            (
                &["workflow-config", "subagent-model", "review", "claude", "opus"],
                None,
                edit(E::SubagentCell { station: SubagentStation::Review, tool: tool("claude"), model: "opus".into() }),
            ),
            (
                &["workflow-config", "subagent-model", "verify", "codex", ""],
                None,
                edit(E::SubagentCell { station: SubagentStation::Verify, tool: tool("codex"), model: "".into() }),
            ),
            (
                &["workflow-config", "subagent-model", "choices", "codex", "--stdin"],
                Some("a\nb\n"),
                edit(E::SubagentChoices { tool: tool("codex"), body: "a\nb\n".into() }),
            ),
        ];
        for (argv, text, want) in rows {
            let inputs = text.map(stdin).unwrap_or_default();
            assert_eq!(command_of(argv, inputs), want, "{argv:?}");
        }
    }

    #[test]
    fn dry_run_reaches_the_command_of_every_write_verb() {
        let rows: &[&[&str]] = &[
            &["workflow-config", "set", "tdd", "true", "--dry-run"],
            &["workflow-config", "context", "--stdin", "--dry-run"],
            &["workflow-config", "rules", "tasks", "--stdin", "--dry-run"],
            &["workflow-config", "review", "critical", "--stdin", "--dry-run"],
            &["workflow-config", "review", "warning", "--stdin", "--dry-run"],
            &["workflow-config", "review", "suggestion", "--stdin", "--dry-run"],
            &["workflow-config", "review", "baseline", "replace", "--dry-run"],
            &["workflow-config", "subagent-model", "review", "claude", "opus", "--dry-run"],
            &["workflow-config", "subagent-model", "verify", "claude", "opus", "--dry-run"],
            &["workflow-config", "subagent-model", "choices", "claude", "--stdin", "--dry-run"],
        ];
        for argv in rows {
            match command_of(argv, stdin("x\n")) {
                Command::WorkflowConfigEdit { dry_run, .. } => assert!(dry_run, "{argv:?}"),
                other => panic!("{argv:?} must be an edit, got {other:?}"),
            }
        }
    }

    #[test]
    fn names_are_the_subcommand_words() {
        let rows: &[(&[&str], &str)] = &[
            (&["workflow-config", "show"], "workflow-config show"),
            (&["workflow-config", "set", "tdd", "true"], "workflow-config set"),
            (&["workflow-config", "context", "--stdin"], "workflow-config context"),
            (&["workflow-config", "rules", "tasks", "--stdin"], "workflow-config rules"),
            (&["workflow-config", "review", "critical", "--stdin"], "workflow-config review critical"),
            (&["workflow-config", "review", "warning", "--stdin"], "workflow-config review warning"),
            (&["workflow-config", "review", "suggestion", "--stdin"], "workflow-config review suggestion"),
            (&["workflow-config", "review", "baseline", "extend"], "workflow-config review baseline"),
            (&["workflow-config", "subagent-model", "review", "claude", "x"], "workflow-config subagent-model review"),
            (&["workflow-config", "subagent-model", "verify", "claude", "x"], "workflow-config subagent-model verify"),
            (&["workflow-config", "subagent-model", "choices", "claude"], "workflow-config subagent-model choices"),
        ];
        for (argv, name) in rows {
            assert_eq!(name_of(argv), *name, "{argv:?}");
        }
    }

    // A content-taking verb reads stdin only with `--stdin`; the others never.
    #[test]
    fn stdin_is_read_only_with_the_flag() {
        use StdinPolicy::{Never, WhenFlagged};
        let rows: &[(&[&str], StdinPolicy)] = &[
            (&["workflow-config", "show"], Never),
            (&["workflow-config", "set", "tdd", "true"], Never),
            (&["workflow-config", "review", "baseline", "extend"], Never),
            (&["workflow-config", "subagent-model", "review", "claude", "opus"], Never),
            (&["workflow-config", "subagent-model", "verify", "claude", "opus"], Never),
            (&["workflow-config", "context", "--stdin"], WhenFlagged { flagged: true }),
            (&["workflow-config", "context"], WhenFlagged { flagged: false }),
            (&["workflow-config", "rules", "tasks", "--stdin"], WhenFlagged { flagged: true }),
            (&["workflow-config", "rules", "tasks"], WhenFlagged { flagged: false }),
            (&["workflow-config", "review", "critical", "--stdin"], WhenFlagged { flagged: true }),
            (&["workflow-config", "review", "warning"], WhenFlagged { flagged: false }),
            (&["workflow-config", "review", "suggestion", "--stdin"], WhenFlagged { flagged: true }),
            (&["workflow-config", "subagent-model", "choices", "codex", "--stdin"], WhenFlagged { flagged: true }),
            (&["workflow-config", "subagent-model", "choices", "codex"], WhenFlagged { flagged: false }),
        ];
        for (argv, want) in rows {
            assert_eq!(policy_of(argv), *want, "{argv:?}");
        }
    }

    // Without the flag the verb would write an empty document from a
    // terminal: it is refused with the command to run.
    #[test]
    fn a_content_verb_without_the_flag_is_invalid_argv() {
        let rows: &[(&[&str], &str)] = &[
            (&["workflow-config", "context"], "context --stdin"),
            (&["workflow-config", "rules", "proposal"], "rules proposal --stdin"),
            (&["workflow-config", "review", "critical"], "review critical --stdin"),
            (&["workflow-config", "review", "warning"], "review warning --stdin"),
            (&["workflow-config", "review", "suggestion"], "review suggestion --stdin"),
            (&["workflow-config", "subagent-model", "choices", "codex"], "subagent-model choices codex --stdin"),
        ];
        for (argv, usage) in rows {
            // Content the caller read anyway does not stand in for the flag.
            for inputs in [VerbInputs::default(), stdin("x\n")] {
                let err = target_of(argv, inputs).expect_err("the flag is required");
                assert_eq!(err.code, ErrorCode::InvalidArgv, "{argv:?}");
                assert_eq!(
                    err.message,
                    format!("content is read from stdin — run: speclink workflow-config {usage}"),
                    "{argv:?}"
                );
            }
        }
    }

    #[test]
    fn the_flag_with_no_stdin_content_is_an_empty_body() {
        assert_eq!(
            command_of(&["workflow-config", "context", "--stdin"], VerbInputs::default()),
            edit(WorkflowConfigEdit::Context(String::new()))
        );
    }

    // The tool name is checked before stdin is asked for.
    #[test]
    fn a_tool_name_that_is_not_kebab_case_is_invalid_argv() {
        const MESSAGE: &str = "tool name 'Bad_Tool' is not kebab-case (2-50 lowercase letters, digits or hyphens; no leading, trailing or doubled hyphen)";
        let rows: &[&[&str]] = &[
            &["workflow-config", "subagent-model", "review", "Bad_Tool", "x"],
            &["workflow-config", "subagent-model", "verify", "Bad_Tool", "x"],
            &["workflow-config", "subagent-model", "choices", "Bad_Tool"],
            &["workflow-config", "subagent-model", "choices", "Bad_Tool", "--stdin"],
        ];
        for argv in rows {
            let err = target_of(argv, VerbInputs::default()).expect_err("the tool name is refused");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{argv:?}");
            assert_eq!(err.message, MESSAGE, "{argv:?}");
        }
    }

    #[test]
    fn set_help_names_exactly_the_policy_keys() {
        let err = crate::parse(["workflow-config", "set", "--help"]).err().expect("help text");
        assert!(
            err.message.starts_with("Set a policy field: locale, spec_locale, tdd, audit, worktree\n"),
            "{}",
            err.message
        );
    }
}

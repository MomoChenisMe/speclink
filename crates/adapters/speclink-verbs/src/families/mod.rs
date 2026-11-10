//! The spec verb families: one file per family, the clap types public.

use clap::Subcommand;
use speclink_core::command::CommandError;

use crate::inputs::{CommandTarget, StdinPolicy, VerbInputs};

pub mod checks;
pub mod discuss;
pub mod documents;
pub mod instructions;
pub mod lifecycle;
pub mod new;
pub mod plan;
pub mod progress;
pub mod query;
pub mod station;
pub mod trace;

// Every top-level spec verb; the CLI flattens this into its own command list.
// No doc comments on clap-derived types: clap would print them as help text.
// `display_order` is each verb's position in the CLI's `--help` list, where
// the spec verbs sit between the CLI's own commands.
#[derive(Subcommand, Debug)]
pub enum SpecVerb {
    /// List changes or specs
    #[command(display_order = 2)]
    List(query::ListArgs),
    /// Show change or spec content
    #[command(display_order = 3)]
    Show(query::ShowArgs),
    /// Validate changes and specs
    #[command(display_order = 4)]
    Validate(checks::ValidateArgs),
    /// Analyze change artifacts for consistency and gaps
    #[command(display_order = 5)]
    Analyze(checks::ChangeArg),
    /// Detect drift between a change and the current codebase state
    #[command(display_order = 6)]
    Drift(checks::ChangeArg),
    /// Trace a capability's provenance chain (archived changes, discussions, evidence)
    #[command(display_order = 7)]
    Trace(trace::TraceArgs),
    /// Show the execution order of active changes (waves, blockers, next)
    #[command(display_order = 8)]
    Plan(plan::PlanArgs),
    /// Change metadata operations (declared prerequisites)
    #[command(display_order = 9)]
    Change(plan::ChangeVerbArgs),
    /// Archive a completed change
    #[command(display_order = 10)]
    Archive(lifecycle::ArchiveArgs),
    /// Discard a change (delete it; --force required once work has started)
    #[command(display_order = 11)]
    Discard(lifecycle::DiscardArgs),
    /// Claim a change for implementation (remote mode)
    #[command(display_order = 12)]
    Claim(lifecycle::ClaimArgs),
    /// Read store documents (artifact contents)
    #[command(display_order = 16)]
    Artifact(documents::ArtifactArgs),
    /// Shared vocabulary document
    #[command(display_order = 17)]
    Language(documents::LanguageArgs),
    /// Show artifact DAG status
    #[command(display_order = 18)]
    Status(query::StatusArgs),
    /// Get instructions for an artifact
    #[command(display_order = 19)]
    Instructions(instructions::InstructionsArgs),
    /// Create a new change or resource
    #[command(display_order = 20)]
    New(new::NewArgs),
    /// Task operations
    #[command(display_order = 28)]
    Task(progress::TaskArgs),
    /// Manage in-progress markers
    #[command(name = "in-progress", display_order = 29)]
    InProgress(progress::InProgressArgs),
    /// Discussion documents (record and evolve a discussion)
    #[command(display_order = 31)]
    Discuss(discuss::DiscussArgs),
    /// Review quality station (ticket rounds, stamping)
    #[command(display_order = 32)]
    Review(station::ReviewArgs),
    /// Verify quality station (ticket rounds, stamping)
    #[command(display_order = 33)]
    Verify(station::VerifyArgs),
}

impl SpecVerb {
    /// The verb's stable name: its subcommand words, e.g. `new artifact`.
    pub fn name(&self) -> &'static str {
        match self {
            SpecVerb::List(_) => "list",
            SpecVerb::Show(_) => "show",
            SpecVerb::Validate(_) => "validate",
            SpecVerb::Analyze(_) => "analyze",
            SpecVerb::Drift(_) => "drift",
            SpecVerb::Trace(_) => "trace",
            SpecVerb::Plan(_) => "plan",
            SpecVerb::Change(a) => a.name(),
            SpecVerb::Archive(_) => "archive",
            SpecVerb::Discard(_) => "discard",
            SpecVerb::Claim(_) => "claim",
            SpecVerb::Artifact(a) => a.name(),
            SpecVerb::Language(a) => a.name(),
            SpecVerb::Status(_) => "status",
            SpecVerb::Instructions(_) => "instructions",
            SpecVerb::New(a) => a.name(),
            SpecVerb::Task(a) => a.name(),
            SpecVerb::InProgress(a) => a.name(),
            SpecVerb::Discuss(a) => a.name(),
            SpecVerb::Review(a) => a.name(),
            SpecVerb::Verify(a) => a.name(),
        }
    }

    /// Whether this verb reads stdin.
    pub fn stdin_policy(&self) -> StdinPolicy {
        match self {
            SpecVerb::List(_)
            | SpecVerb::Show(_)
            | SpecVerb::Validate(_)
            | SpecVerb::Analyze(_)
            | SpecVerb::Drift(_)
            | SpecVerb::Trace(_)
            | SpecVerb::Plan(_)
            | SpecVerb::Change(_)
            | SpecVerb::Archive(_)
            | SpecVerb::Discard(_)
            | SpecVerb::Claim(_)
            | SpecVerb::Artifact(_)
            | SpecVerb::Language(_)
            | SpecVerb::Status(_)
            | SpecVerb::Instructions(_)
            | SpecVerb::Task(_)
            | SpecVerb::InProgress(_) => StdinPolicy::Never,
            SpecVerb::New(a) => a.stdin_policy(),
            SpecVerb::Discuss(a) => a.stdin_policy(),
            SpecVerb::Review(a) => a.stdin_policy(),
            SpecVerb::Verify(a) => a.stdin_policy(),
        }
    }

    /// The single conversion from a parsed verb to its engine command. The
    /// match has no wildcard arm: a new verb does not compile until it is
    /// handled here. Each family's own match is exhaustive the same way.
    pub fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match self {
            SpecVerb::List(a) => a.to_command(inputs),
            SpecVerb::Show(a) => a.to_command(inputs),
            SpecVerb::Validate(a) => a.to_command(inputs),
            SpecVerb::Analyze(a) => a.analyze_command(inputs),
            SpecVerb::Drift(_) => Ok(CommandTarget::LocalOnly { verb: "drift" }),
            SpecVerb::Trace(a) => a.to_command(inputs),
            SpecVerb::Plan(a) => a.to_command(inputs),
            SpecVerb::Change(a) => a.to_command(inputs),
            SpecVerb::Archive(a) => a.to_command(inputs),
            SpecVerb::Discard(a) => a.to_command(inputs),
            SpecVerb::Claim(a) => a.to_command(inputs),
            SpecVerb::Artifact(a) => a.to_command(inputs),
            SpecVerb::Language(a) => a.to_command(inputs),
            SpecVerb::Status(a) => a.to_command(inputs),
            SpecVerb::Instructions(a) => a.to_command(inputs),
            SpecVerb::New(a) => a.to_command(inputs),
            SpecVerb::Task(a) => a.to_command(inputs),
            SpecVerb::InProgress(a) => a.to_command(inputs),
            SpecVerb::Discuss(a) => a.to_command(inputs),
            SpecVerb::Review(a) => a.to_command(inputs),
            SpecVerb::Verify(a) => a.to_command(inputs),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::{CommandTarget, StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, CommandError};

    fn parsed(argv: &[&str]) -> crate::ParsedVerb {
        crate::parse(argv.iter().copied())
            .unwrap_or_else(|e| panic!("{argv:?} must parse: {}", e.message))
    }

    /// The conversion result, error included.
    pub(crate) fn target_of(argv: &[&str], inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        parsed(argv).into_command(inputs)
    }

    /// The engine command an argv converts to; panics on LocalOnly or error.
    pub(crate) fn command_of(argv: &[&str], inputs: VerbInputs) -> Command {
        match target_of(argv, inputs) {
            Ok(CommandTarget::Command(cmd)) => cmd,
            other => panic!("{argv:?} must convert to a command, got {other:?}"),
        }
    }

    /// The verb name a LocalOnly argv reports, or None for a command.
    pub(crate) fn local_only_of(argv: &[&str]) -> Option<&'static str> {
        match target_of(argv, VerbInputs::default()) {
            Ok(CommandTarget::LocalOnly { verb }) => Some(verb),
            Ok(CommandTarget::Command(_)) => None,
            Err(e) => panic!("{argv:?} must convert: {}", e.message),
        }
    }

    pub(crate) fn name_of(argv: &[&str]) -> &'static str {
        parsed(argv).name()
    }

    pub(crate) fn policy_of(argv: &[&str]) -> StdinPolicy {
        parsed(argv).stdin_policy()
    }
}

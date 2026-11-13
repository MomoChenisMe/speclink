//! The spec verb families: one file per family, the clap types public.

use clap::Subcommand;
use speclink_core::command::CommandError;

use crate::inputs::{CommandTarget, LocalOnlyReason, StdinPolicy, VerbInputs};

/// A parsed verb, or one family's arguments, converted to its engine command
/// (or `LocalOnly`). `SpecVerb`'s implementation is the single conversion; each
/// family's implementation is one arm of it.
pub trait ToCommand {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError>;
}

pub mod checks;
pub mod discuss;
pub mod documents;
pub mod evidence;
pub mod fields;
pub mod instructions;
pub mod lifecycle;
pub mod new;
pub mod plan;
pub mod progress;
pub mod query;
pub mod station;
pub mod trace;
pub mod workflow_config;

// Every top-level spec verb; the CLI flattens this into its own command list
// and sets the `--help` order there. No doc comments on clap-derived types:
// clap would print them as help text.
#[derive(Subcommand, Debug)]
pub enum SpecVerb {
    /// List changes or specs
    List(query::ListArgs),
    /// Show change or spec content
    Show(query::ShowArgs),
    /// Validate changes and specs
    Validate(checks::ValidateArgs),
    /// Analyze change artifacts for consistency and gaps
    Analyze(checks::ChangeArg),
    /// Detect drift between a change and the current codebase state
    Drift(checks::ChangeArg),
    /// Trace a capability's provenance chain (archived changes, discussions, evidence)
    Trace(trace::TraceArgs),
    /// Show the execution order of active changes (waves, blockers, next)
    Plan(plan::PlanArgs),
    /// Change metadata operations (declared prerequisites)
    Change(plan::ChangeVerbArgs),
    /// Archive a completed change
    Archive(lifecycle::ArchiveArgs),
    /// Discard a change (delete it; --force required once work has started)
    Discard(lifecycle::DiscardArgs),
    /// Claim a change for implementation (remote mode)
    Claim(lifecycle::ClaimArgs),
    /// Read store documents (artifact contents)
    Artifact(documents::ArtifactArgs),
    /// Completion evidence (the files each completed task touched)
    Evidence(evidence::EvidenceArgs),
    /// Shared vocabulary document
    Language(documents::LanguageArgs),
    /// Show artifact DAG status
    Status(query::StatusArgs),
    /// Get instructions for an artifact
    Instructions(instructions::InstructionsArgs),
    /// Create a new change or resource
    New(new::NewArgs),
    /// Task operations
    Task(progress::TaskArgs),
    /// Manage in-progress markers
    #[command(name = "in-progress")]
    InProgress(progress::InProgressArgs),
    /// Discussion documents (record and evolve a discussion)
    Discuss(discuss::DiscussArgs),
    /// Review quality station (ticket rounds, stamping)
    Review(station::ReviewArgs),
    /// Verify quality station (ticket rounds, stamping)
    Verify(station::VerifyArgs),
    /// Workflow config commands (<spec dir>/config.yaml)
    #[command(name = "workflow-config")]
    WorkflowConfig(workflow_config::WorkflowConfigArgs),
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
            SpecVerb::Evidence(a) => a.name(),
            SpecVerb::Language(a) => a.name(),
            SpecVerb::Status(_) => "status",
            SpecVerb::Instructions(_) => "instructions",
            SpecVerb::New(a) => a.name(),
            SpecVerb::Task(a) => a.name(),
            SpecVerb::InProgress(a) => a.name(),
            SpecVerb::Discuss(a) => a.name(),
            SpecVerb::Review(a) => a.name(),
            SpecVerb::Verify(a) => a.name(),
            SpecVerb::WorkflowConfig(a) => a.name(),
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
            | SpecVerb::Evidence(_)
            | SpecVerb::Language(_)
            | SpecVerb::Status(_)
            | SpecVerb::Instructions(_)
            | SpecVerb::Task(_)
            | SpecVerb::InProgress(_) => StdinPolicy::Never,
            SpecVerb::New(a) => a.stdin_policy(),
            SpecVerb::Discuss(a) => a.stdin_policy(),
            SpecVerb::Review(a) => a.stdin_policy(),
            SpecVerb::Verify(a) => a.stdin_policy(),
            SpecVerb::WorkflowConfig(a) => a.stdin_policy(),
        }
    }
}

impl ToCommand for SpecVerb {
    /// The single conversion from a parsed verb to its engine command. The
    /// match has no wildcard arm: a new verb does not compile until it is
    /// handled here. Each family's own match is exhaustive the same way.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        match self {
            SpecVerb::List(a) => a.to_command(inputs),
            SpecVerb::Show(a) => a.to_command(inputs),
            SpecVerb::Validate(a) => a.to_command(inputs),
            SpecVerb::Analyze(a) => a.analyze_command(inputs),
            SpecVerb::Drift(_) => {
                Ok(CommandTarget::LocalOnly { verb: self.name(), reason: LocalOnlyReason::NeedsCheckout })
            }
            SpecVerb::Trace(a) => a.to_command(inputs),
            SpecVerb::Plan(a) => a.to_command(inputs),
            SpecVerb::Change(a) => a.to_command(inputs),
            SpecVerb::Archive(a) => a.to_command(inputs),
            SpecVerb::Discard(a) => a.to_command(inputs),
            SpecVerb::Claim(a) => a.to_command(inputs),
            SpecVerb::Artifact(a) => a.to_command(inputs),
            SpecVerb::Evidence(a) => a.to_command(inputs),
            SpecVerb::Language(a) => a.to_command(inputs),
            SpecVerb::Status(a) => a.to_command(inputs),
            SpecVerb::Instructions(a) => a.to_command(inputs),
            SpecVerb::New(a) => a.to_command(inputs),
            SpecVerb::Task(a) => a.to_command(inputs),
            SpecVerb::InProgress(a) => a.to_command(inputs),
            SpecVerb::Discuss(a) => a.to_command(inputs),
            SpecVerb::Review(a) => a.to_command(inputs),
            SpecVerb::Verify(a) => a.to_command(inputs),
            SpecVerb::WorkflowConfig(a) => a.to_command(inputs),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::{CommandTarget, StdinPolicy, ToCommand, VerbInputs};
    use speclink_core::command::{Command, CommandError};

    fn parsed(argv: &[&str]) -> crate::ParsedVerb {
        crate::parse(argv.iter().copied())
            .unwrap_or_else(|e| panic!("{argv:?} must parse: {}", e.message))
    }

    /// The conversion result, error included.
    pub(crate) fn target_of(argv: &[&str], inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        parsed(argv).to_command(inputs)
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
            Ok(CommandTarget::LocalOnly { verb, .. }) => Some(verb),
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

#[cfg(test)]
mod tests {
    use super::test_support::{policy_of, target_of};
    use crate::{CommandTarget, LocalOnlyReason, StdinPolicy, VerbInputs};

    // command-runtime: a verb with no engine command is reported with why it
    // runs only in the CLI.
    #[test]
    fn local_only_verbs_name_their_reason() {
        use LocalOnlyReason::{CliSkillText, NeedsCheckout};
        let rows: &[(&[&str], LocalOnlyReason)] = &[
            (&["drift", "demo"], NeedsCheckout),
            (&["review", "prepare", "demo"], NeedsCheckout),
            (&["review", "scope", "demo"], NeedsCheckout),
            (&["verify", "scope", "demo"], NeedsCheckout),
            (&["instructions", "--skill", "tdd"], CliSkillText),
        ];
        for (argv, want) in rows {
            match target_of(argv, VerbInputs::default()) {
                Ok(CommandTarget::LocalOnly { reason, .. }) => assert_eq!(reason, *want, "{argv:?}"),
                other => panic!("{argv:?} must be LocalOnly, got {other:?}"),
            }
        }
    }

    // The Example "stdin 的三種規則" of command-runtime's requirement, row by
    // row with the spec's own argv.
    #[test]
    fn stdin_rules_match_the_spec_example_table() {
        let rows: &[(&[&str], StdinPolicy)] = &[
            (&["list"], StdinPolicy::Never),
            (
                &["new", "artifact", "proposal", "--change", "demo", "--stdin"],
                StdinPolicy::WhenFlagged { flagged: true },
            ),
            (&["review", "stamp", "demo", "--stdin"], StdinPolicy::WhenFlagged { flagged: true }),
            (&["discuss", "add-round", "demo"], StdinPolicy::FlagOrPiped { flagged: false }),
            (&["discuss", "conclude", "demo"], StdinPolicy::FlagOrPiped { flagged: false }),
        ];
        for (argv, want) in rows {
            assert_eq!(policy_of(argv), *want, "{argv:?}");
        }
    }
}

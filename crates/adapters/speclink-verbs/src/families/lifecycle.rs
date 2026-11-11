//! Change lifecycle verbs: archive, discard, claim.

use clap::Args;
use speclink_core::command::{Command, CommandError, ErrorCode};

use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct ArchiveArgs {
    /// Changes to archive (several allowed; auto-detects when omitted and only one exists)
    #[arg(value_name = "CHANGE")]
    pub changes: Vec<String>,
    /// Archive every ready change (tasks complete, valid, nothing the merge gate refuses)
    #[arg(long)]
    pub all: bool,
    /// Skip confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
    /// Skip spec updates
    #[arg(long = "skip-specs")]
    pub skip_specs: bool,
    /// Skip validation before archiving
    #[arg(long = "no-validate")]
    pub no_validate: bool,
    /// Mark all incomplete tasks as complete before archiving
    #[arg(long = "mark-tasks-complete")]
    pub mark_tasks_complete: bool,
    /// Archive despite an open review ticket (the ticket travels with the
    /// change and is permanently shown as reviewed-not-passed)
    #[arg(long = "carry-review")]
    pub carry_review: bool,
    /// Archive despite an open verify ticket (the ticket travels with the
    /// change and is permanently shown as verified-not-passed)
    #[arg(long = "carry-verify")]
    pub carry_verify: bool,
}
/// The bulk preflight and `Command::Archive` start from the same flags.
impl From<&ArchiveArgs> for speclink_core::archive::ArchiveOptions {
    fn from(a: &ArchiveArgs) -> Self {
        Self {
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
            carry_review: a.carry_review,
            carry_verify: a.carry_verify,
        }
    }
}

#[derive(Args, Debug)]
pub struct DiscardArgs {
    /// Change to discard
    pub change: String,
    /// Discard even when the change has started work (started_at or checked tasks)
    #[arg(long)]
    pub force: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}
#[derive(Args, Debug)]
pub struct ClaimArgs {
    /// Change name
    pub name: String,
}

impl ArchiveArgs {
    /// One change's archive under these flags — the single command, and each
    /// step of the CLI's bulk loop.
    /// The flags come from the one `ArchiveOptions` mapping above.
    pub fn archive_command(&self, change: Option<String>) -> Command {
        let speclink_core::archive::ArchiveOptions {
            skip_specs,
            no_validate,
            mark_tasks_complete,
            carry_review,
            carry_verify,
        } = self.into();
        Command::Archive { change, skip_specs, no_validate, mark_tasks_complete, carry_review, carry_verify }
    }
}

impl ToCommand for ArchiveArgs {
    /// One engine command archives one change: several names or `--all` run
    /// as the caller's loop of single archives.
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        if self.all || self.changes.len() > 1 {
            return Err(CommandError::new(
                ErrorCode::InvalidArgv,
                "archive converts one change to one command — several changes or --all run one archive per change",
            ));
        }
        Ok(CommandTarget::Command(self.archive_command(self.changes.first().cloned())))
    }
}

impl ToCommand for DiscardArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Discard { change: self.change.clone(), force: self.force }))
    }
}

impl ToCommand for ClaimArgs {
    fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Claim { name: self.name.clone() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};

    #[test]
    fn archive_discard_claim_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["archive"],
                Command::Archive {
                    change: None,
                    skip_specs: false,
                    no_validate: false,
                    mark_tasks_complete: false,
                    carry_review: false,
                    carry_verify: false,
                },
            ),
            (
                &[
                    "archive", "demo", "-y", "--skip-specs", "--no-validate", "--mark-tasks-complete",
                    "--carry-review", "--carry-verify",
                ],
                Command::Archive {
                    change: Some("demo".into()),
                    skip_specs: true,
                    no_validate: true,
                    mark_tasks_complete: true,
                    carry_review: true,
                    carry_verify: true,
                },
            ),
            (&["discard", "demo"], Command::Discard { change: "demo".into(), force: false }),
            (&["discard", "demo", "--force", "--json"], Command::Discard { change: "demo".into(), force: true }),
            (&["claim", "demo"], Command::Claim { name: "demo".into() }),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    /// One engine command archives one change; several names or `--all` run
    /// one archive per change in the caller's loop.
    #[test]
    fn bulk_archive_is_not_one_command() {
        for argv in [&["archive", "a", "b"][..], &["archive", "--all"]] {
            let err = target_of(argv, VerbInputs::default()).expect_err("bulk archive");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{argv:?}");
        }
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [(&["archive"][..], "archive"), (&["discard", "x"], "discard"), (&["claim", "x"], "claim")] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

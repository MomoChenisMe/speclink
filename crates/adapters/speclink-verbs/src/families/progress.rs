//! Work-progress verbs: task done/undone and the in-progress marker.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub command: TaskCommands,
}
#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// Mark a task as done and record touched files
    Done {
        /// Task ID (1-based sequential index)
        task_id: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mark a task as not done (pure state flip, no side effects)
    Undone {
        /// Task ID (1-based sequential index)
        task_id: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args, Debug)]
pub struct InProgressArgs {
    #[command(subcommand)]
    pub command: InProgressCommands,
}
#[derive(Subcommand, Debug)]
pub enum InProgressCommands {
    /// Mark a change as in-progress
    Add {
        /// Change name
        name: String,
    },
    /// Remove the in-progress marker — only when the change carries no work
    /// traces (no checked tasks, no touched records); unknown names error
    Remove {
        /// Change name
        name: String,
    },
}

impl TaskArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            TaskCommands::Done { .. } => "task done",
            TaskCommands::Undone { .. } => "task undone",
        }
    }
}

impl ToCommand for TaskArgs {
    /// `task done` takes the caller's touched files and HEAD commit; without
    /// them the engine probes the local workspace.
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(match &self.command {
            TaskCommands::Done { task_id, change, .. } => Command::TaskDone {
                task_id: task_id.clone(),
                change: change.clone(),
                touched_files: inputs.touched_files,
                head_commit: inputs.head_commit,
            },
            TaskCommands::Undone { task_id, change, .. } => Command::TaskUndone {
                task_id: task_id.clone(),
                change: change.clone(),
            },
        }))
    }
}

impl InProgressArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            InProgressCommands::Add { .. } => "in-progress add",
            InProgressCommands::Remove { .. } => "in-progress remove",
        }
    }
}

impl ToCommand for InProgressArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(match &self.command {
            InProgressCommands::Add { name } => Command::InProgressAdd { name: name.clone(), branch: inputs.branch },
            InProgressCommands::Remove { name } => Command::InProgressRemove { name: name.clone() },
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    #[test]
    fn task_done_carries_the_callers_touched_files_and_head() {
        let inputs = VerbInputs {
            touched_files: Some(vec!["src/a.rs".into()]),
            head_commit: Some("abc123".into()),
            ..Default::default()
        };
        assert_eq!(
            command_of(&["task", "done", "3", "--change", "demo", "--json"], inputs),
            Command::TaskDone {
                task_id: "3".into(),
                change: Some("demo".into()),
                touched_files: Some(vec!["src/a.rs".into()]),
                head_commit: Some("abc123".into()),
            }
        );
        assert_eq!(
            command_of(&["task", "done", "3"], VerbInputs::default()),
            Command::TaskDone { task_id: "3".into(), change: None, touched_files: None, head_commit: None }
        );
    }

    #[test]
    fn task_undone_and_in_progress_commands() {
        let cases: &[(&[&str], Command)] = &[
            (
                &["task", "undone", "01", "--change", "demo"],
                Command::TaskUndone { task_id: "01".into(), change: Some("demo".into()) },
            ),
            (&["in-progress", "add", "demo"], Command::InProgressAdd { name: "demo".into(), branch: None }),
            (&["in-progress", "remove", "demo"], Command::InProgressRemove { name: "demo".into() }),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
    }

    #[test]
    fn names_and_stdin_policies() {
        for (argv, name) in [
            (&["task", "done", "1"][..], "task done"),
            (&["task", "undone", "1"], "task undone"),
            (&["in-progress", "add", "x"], "in-progress add"),
            (&["in-progress", "remove", "x"], "in-progress remove"),
        ] {
            assert_eq!(name_of(argv), name);
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }

    #[test]
    fn in_progress_add_carries_the_callers_branch() {
        // The CLI passes its checkout's branch; a detached HEAD passes none.
        let inputs = VerbInputs { branch: Some("feature/x".into()), ..Default::default() };
        assert_eq!(
            command_of(&["in-progress", "add", "demo"], inputs),
            Command::InProgressAdd { name: "demo".into(), branch: Some("feature/x".into()) }
        );
    }
}

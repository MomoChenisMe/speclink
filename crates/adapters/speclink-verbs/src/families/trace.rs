//! `trace` — one capability's provenance chain.

use clap::Args;
use speclink_core::command::{Command, CommandError};

use crate::{CommandTarget, VerbInputs};

#[derive(Args, Debug)]
pub struct TraceArgs {
    /// Capability name (canonical spec directory name)
    pub capability: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

impl TraceArgs {
    pub fn to_command(&self, _inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(Command::Trace { capability: self.capability.clone() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, name_of, policy_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::Command;

    #[test]
    fn trace_command() {
        for argv in [&["trace", "user-auth"][..], &["trace", "user-auth", "--json"]] {
            assert_eq!(
                command_of(argv, VerbInputs::default()),
                Command::Trace { capability: "user-auth".into() },
                "{argv:?}"
            );
        }
        assert_eq!(name_of(&["trace", "x"]), "trace");
        assert_eq!(policy_of(&["trace", "x"]), StdinPolicy::Never);
    }
}

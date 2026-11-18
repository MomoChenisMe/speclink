//! Which Engine commands only read (design D2 of hook-read-function): what
//! the before-command hook's reader ([`crate::kit::Reader`]) runs. The match
//! has no catch-all arm: a new command kind does not compile until it is
//! judged here.
//!
//! The kind decides, not this call's result: a command that writes is a write
//! also when it finds nothing to change (a task that is already checked), so
//! a hook's author knows which commands the reader takes.
//!
//! # Sharp-edges audit of the reader
//!
//! The interface this module guards was checked against the audit list of
//! `speclink instructions --skill audit`. The conclusions:
//!
//! - **No path to the store's commit.** [`crate::kit::HostKit::read`] is the
//!   one implementation, for the Rust hook and for the Node `read` alike. It
//!   asks [`is_read`] first and refuses any other command before the store
//!   gets a call; it asks no hook and tells no after-commit hook. A kind that
//!   is judged wrongly here is caught by the kit's test that runs every read
//!   kind on a store with data and counts the commits.
//! - **No other identity, no other scope.** [`crate::kit::Reader`] has
//!   private fields and no public constructor, and its `read` takes a command
//!   only: the caller is the one of the call. The Node `read` takes argv
//!   only; index.js takes the identity and the scope before the hook runs, so
//!   a hook that changes its `info` object changes nothing.
//! - **One parsing.** The Node `read` parses argv with the functions of
//!   `dispatch` (`parse_verb`, `command_of`): a usage error, or a verb that
//!   needs a checkout, fails as it does there. There is no stdin and no free
//!   field to attach, so a verb that takes content is refused by its name
//!   before the conversion asks for the content. The Node side does not
//!   judge the kind a second time: it knows the kit's refusal by its type
//!   ([`crate::kit::NotARead`]) and names the verb in the message.
//! - **It ends with the hook.** In Rust the reader borrows the kit and the
//!   caller, so it cannot outlive the hook call. In Node a switch closes when
//!   the hook's value or Promise settles; a `read` kept past that is refused
//!   with `invalid_argv` before the native side.
//! - **Wrong use fails loudly.** A command that writes is an error, never a
//!   silent no-op, and the message names it. A Node hook that lets a read's
//!   rejection out does not allow: the call ends as `internal`.
//! - **The known limit is documented.** What the reader gives and what the
//!   command itself reads are two calls to the store (design D5): the docs of
//!   [`crate::kit::Reader`], of `BeforeCommandTools.read` in index.d.ts and of
//!   docs/sdk-node say so, and that the Engine's own checks judge by what the
//!   command reads.

use speclink_core::command::Command;

/// Whether `command` is of a kind that writes no document.
pub fn is_read(command: &Command) -> bool {
    match command {
        Command::List { .. }
        | Command::Show { .. }
        | Command::Status { .. }
        | Command::Instructions { .. }
        | Command::Validate { .. }
        | Command::Analyze { .. }
        | Command::Trace { .. }
        | Command::Plan { .. }
        | Command::PlanStrict
        | Command::ArtifactCat { .. }
        | Command::EvidenceShow { .. }
        | Command::LanguageShow
        | Command::ReviewShow { .. }
        | Command::VerifyShow { .. }
        | Command::DiscussList { .. }
        | Command::DiscussShow { .. }
        | Command::DiscussSearch { .. }
        | Command::WorkflowConfigShow => true,
        Command::NewChange { .. }
        | Command::NewArtifact { .. }
        | Command::TaskDone { .. }
        | Command::TaskUndone { .. }
        | Command::TaskMove { .. }
        | Command::Claim { .. }
        | Command::InProgressAdd { .. }
        | Command::InProgressRemove { .. }
        | Command::ChangeDepends { .. }
        | Command::ChangeSchema { .. }
        | Command::ChangeRank { .. }
        | Command::ChangeLanded { .. }
        | Command::ChangeFieldsEdit { .. }
        | Command::Archive { .. }
        | Command::Discard { .. }
        | Command::DiscussNew { .. }
        | Command::DiscussContext { .. }
        | Command::DiscussAddRound { .. }
        | Command::DiscussConclude { .. }
        | Command::DiscussPromote { .. }
        | Command::DiscussLink { .. }
        | Command::DiscussSeal { .. }
        | Command::DiscussArchive { .. }
        | Command::DiscussDiscard { .. }
        | Command::DiscussFieldsEdit { .. }
        | Command::ReviewAddRound { .. }
        | Command::ReviewStamp { .. }
        | Command::ReviewDiscard { .. }
        | Command::VerifyAddRound { .. }
        | Command::VerifyStamp { .. }
        | Command::VerifyDiscard { .. }
        | Command::WorkflowConfigEdit { .. }
        | Command::WorkflowConfigReplace { .. } => false,
    }
}

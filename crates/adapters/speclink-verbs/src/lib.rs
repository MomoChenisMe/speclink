//! speclink-verbs: the one definition of the spec verbs' argv vocabulary.
//!
//! The CLI and the Node SDK both parse verb text with these clap types and
//! turn the result into an engine command through one function, so a verb
//! added here reaches both entry points at once. Nothing in this crate exits
//! the process, prints, reads stdin, files or environment variables, or runs
//! git — facts a command needs beyond argv come in from the caller.

pub mod families;
mod inputs;
mod parse;

pub use families::SpecVerb;
pub use inputs::{CommandTarget, StdinPolicy, VerbInputs};
pub use parse::{parse, GlobalArgs, ParseError, ParseErrorKind, ParsedVerb};

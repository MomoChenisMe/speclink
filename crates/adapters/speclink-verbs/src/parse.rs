//! The parse entry: verb text in, a parsed verb or a parse error out.
//!
//! The root parser carries the same global flag the CLI has (`--no-color`)
//! and the bin name `speclink`, so a usage error reads exactly like the CLI's
//! stderr. clap's `try_parse_from` never prints or exits; the rendered text
//! comes back inside [`ParseError`].

use clap::{Args, CommandFactory, FromArgMatches, Parser};

use crate::families::{station, SpecVerb, ToCommand};
use crate::inputs::{CommandTarget, StdinPolicy, VerbInputs};

// The flags every verb accepts — one definition, flattened into the CLI's
// root as well. (A doc comment here would become the root's help text.)
#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Parser)]
#[command(name = "speclink")]
struct VerbLine {
    #[command(flatten)]
    _global: GlobalArgs,
    #[command(subcommand)]
    verb: SpecVerb,
}

/// Why `parse` returned no verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The argv does not fit the vocabulary (unknown verb or flag, missing
    /// argument, extra positional).
    Usage,
    /// The argv asked for help or version text; `message` is that text.
    Help,
}

/// A parse failure: clap's own text, without ANSI color codes or the final
/// newline, plus the exit code clap would have used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub exit_code: i32,
}

/// One parsed spec verb.
#[derive(Debug)]
pub struct ParsedVerb {
    verb: SpecVerb,
    // `Some(flag)` for the two stamps: whether their host-only `--stdin` came.
    stamp_stdin: Option<bool>,
}

impl ParsedVerb {
    /// The verb's stable name: its subcommand words, e.g. `new artifact`.
    pub fn name(&self) -> &'static str {
        self.verb.name()
    }

    /// Whether this verb is `review stamp` or `verify stamp`.
    pub fn is_stamp(&self) -> bool {
        self.stamp_stdin.is_some()
    }

    /// Whether this verb reads stdin — see [`StdinPolicy`]. A stamp reads it
    /// only with its host-only `--stdin`.
    pub fn stdin_policy(&self) -> StdinPolicy {
        match self.stamp_stdin {
            Some(flagged) => StdinPolicy::WhenFlagged { flagged },
            None => self.verb.stdin_policy(),
        }
    }

    /// The engine command this verb stands for (see `SpecVerb`'s [`ToCommand`]).
    pub fn into_command(
        self,
        inputs: VerbInputs,
    ) -> Result<CommandTarget, speclink_core::command::CommandError> {
        // A stamp's stdin is its scope JSON, read under the host-only flag.
        match (&self.verb, self.stamp_stdin) {
            (SpecVerb::Review(a), Some(flagged)) => a.command_with(inputs, flagged),
            (SpecVerb::Verify(a), Some(flagged)) => a.command_with(inputs, flagged),
            (verb, _) => verb.to_command(inputs),
        }
    }
}

/// Parse one verb's argv (without the program name, e.g. `["list", "--json"]`).
pub fn parse<I, T>(argv: I) -> Result<ParsedVerb, ParseError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let line = std::iter::once(std::ffi::OsString::from("speclink"))
        .chain(argv.into_iter().map(Into::into));
    // The CLI's tree plus the stamps' host-only `--stdin`.
    let mut tree = station::with_stamp_stdin(VerbLine::command());
    let matches = tree.try_get_matches_from_mut(line).map_err(parse_error)?;
    let VerbLine { verb, .. } =
        VerbLine::from_arg_matches(&matches).map_err(|e| parse_error(e.format(&mut tree)))?;
    Ok(ParsedVerb { verb, stamp_stdin: station::stamp_stdin_of(&matches) })
}

fn parse_error(e: clap::Error) -> ParseError {
    ParseError {
        kind: if e.use_stderr() { ParseErrorKind::Usage } else { ParseErrorKind::Help },
        message: e.render().to_string().trim_end().to_string(),
        exit_code: e.exit_code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANSI_ESC: char = '\u{1b}';

    #[test]
    fn only_the_two_stamps_are_stamps() {
        for argv in [&["review", "stamp", "x"][..], &["verify", "stamp", "x", "--stdin"]] {
            assert!(parse(argv).expect("parses").is_stamp(), "{argv:?}");
        }
        for argv in [&["review", "add-round", "x"][..], &["verify", "show", "x"], &["list"]] {
            assert!(!parse(argv).expect("parses").is_stamp(), "{argv:?}");
        }
    }

    #[test]
    fn empty_argv_is_a_usage_error() {
        let err = parse(Vec::<String>::new()).err().expect("empty argv must not parse");
        assert_eq!(err.kind, ParseErrorKind::Usage);
        assert_eq!(err.exit_code, 2);
        assert!(!err.message.is_empty());
    }

    #[test]
    fn unknown_subcommand_is_a_usage_error() {
        let err = parse(["no-such-verb"]).err().expect("unknown verb must not parse");
        assert_eq!(err.kind, ParseErrorKind::Usage);
        assert!(err.message.contains("no-such-verb"), "{}", err.message);
    }

    #[test]
    fn help_is_reported_as_help_text() {
        let err = parse(["--help"]).err().expect("--help yields help text, not a verb");
        assert_eq!(err.kind, ParseErrorKind::Help);
        assert_eq!(err.exit_code, 0);
        assert!(err.message.starts_with("Usage: speclink"), "{}", err.message);
    }

    #[test]
    fn messages_carry_no_ansi_and_no_trailing_newline() {
        for argv in [vec![], vec!["no-such-verb"], vec!["--help"], vec!["--no-colr"]] {
            let err = parse(argv.clone()).err().expect("none of these parse");
            assert!(!err.message.contains(ANSI_ESC), "{argv:?}: {:?}", err.message);
            assert_eq!(err.message, err.message.trim_end(), "{argv:?}");
        }
    }

    /// Re-runs this test in a child process: there `parse` gets every probe
    /// argv; the parent checks that the child ran to its end (no exit from
    /// inside parse) and that clap wrote nothing to stdout or stderr.
    #[test]
    fn parse_neither_exits_nor_writes() {
        const PROBE: &str = "SPECLINK_VERBS_PARSE_PROBE";
        if std::env::var_os(PROBE).is_some() {
            for argv in [vec![], vec!["no-such-verb"], vec!["--help"], vec!["-h"], vec!["help"]] {
                let _ = parse(argv);
            }
            return;
        }
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "parse::tests::parse_neither_exits_nor_writes", "--nocapture"])
            .env(PROBE, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{stdout}{stderr}");
        assert!(stdout.contains("1 passed"), "the child stopped early: {stdout}");
        for text in [&stdout, &stderr] {
            assert!(!text.contains("Usage:") && !text.contains("error:"), "{text}");
        }
    }
}

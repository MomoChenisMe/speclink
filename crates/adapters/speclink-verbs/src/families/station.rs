//! The two quality stations: review and verify.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError, ErrorCode};
use speclink_core::model::ReviewedScopeEntry;

use crate::{CommandTarget, LocalOnlyReason, StdinPolicy, ToCommand, VerbInputs};

#[derive(Args, Debug)]
pub struct ReviewArgs {
    #[command(subcommand)]
    pub command: ReviewCommands,
}
#[derive(Subcommand, Debug)]
pub enum ReviewCommands {
    /// Capture the Apply baseline sidecar before the change is marked in-progress
    Prepare {
        change: String,
        /// Reset the baseline to this trusted commit (an ancestor of HEAD)
        #[arg(long)]
        base: Option<String>,
        /// Carry both stations' snapshots over from this worktree (requires --base)
        #[arg(long, requires = "base")]
        from: Option<std::path::PathBuf>,
    },
    /// Resolve and freeze the review scope (--json for the structured payload)
    Scope {
        change: String,
        #[arg(long)]
        json: bool,
        /// Trusted fixed point overriding the Apply baseline
        #[arg(long)]
        base: Option<String>,
        /// Candidate identity a hash-pinned hunk selection is anchored to
        #[arg(long = "candidate-hash")]
        candidate_hash: Option<String>,
        /// Hunk id to include (repeatable; requires --candidate-hash)
        #[arg(long = "include-hunk")]
        include_hunk: Vec<String>,
    },
    /// Append a review round to the change's ticket (content from stdin; creates the ticket on the first round)
    #[command(name = "add-round")]
    AddRound {
        change: String,
        #[arg(long)]
        stdin: bool,
    },
    /// Print the review ticket (--json for the structured payload)
    Show {
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Stamp the review: requires all tasks done and an empty must-fix set in the last round (SUGGESTION never blocks)
    Stamp {
        change: String,
        /// Stamp despite outstanding must-fix (CRITICAL/WARNING) findings in the last round
        #[arg(long)]
        accept: bool,
        /// Tool identity recorded as reviewed_with (mirrors `new change --agent`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Discard the review ticket without stamping
    Discard { change: String },
}
#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[command(subcommand)]
    pub command: VerifyCommands,
}
/// 驗證站沒有 `prepare`：Apply baseline 由 apply 流程一次錄下、兩站共用
/// （design D8），第二個 prepare 只會覆蓋同一份 sidecar。
#[derive(Subcommand, Debug)]
pub enum VerifyCommands {
    /// Resolve and freeze the verify scope (--json for the structured payload)
    Scope {
        change: String,
        #[arg(long)]
        json: bool,
        /// Trusted fixed point overriding the Apply baseline
        #[arg(long)]
        base: Option<String>,
        /// Candidate identity a hash-pinned hunk selection is anchored to
        #[arg(long = "candidate-hash")]
        candidate_hash: Option<String>,
        /// Hunk id to include (repeatable; requires --candidate-hash)
        #[arg(long = "include-hunk")]
        include_hunk: Vec<String>,
    },
    /// Append a verify round to the change's ticket (content from stdin; requires every task done)
    #[command(name = "add-round")]
    AddRound {
        change: String,
        #[arg(long)]
        stdin: bool,
    },
    /// Print the verify ticket (--json for the structured payload)
    Show {
        change: String,
        #[arg(long)]
        json: bool,
    },
    /// Stamp the verification: requires all tasks done and an empty must-fix set in the last round (SUGGESTION never blocks)
    Stamp {
        change: String,
        /// Stamp despite outstanding must-fix (CRITICAL/WARNING) findings in the last round
        #[arg(long)]
        accept: bool,
        /// Tool identity recorded as verified_with (mirrors `new change --agent`)
        #[arg(long)]
        agent: Option<String>,
    },
    /// Discard the verify ticket without stamping
    Discard { change: String },
}

/// The `--stdin` payload of a stamp: the work-tree holder's `(path, hash)`
/// list, the union paths that are gone, and the commit the stamp is for. The
/// lists default to empty and the commit to absent; any other field is refused.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StampPayload {
    #[serde(default)]
    scope: Vec<ReviewedScopeEntry>,
    #[serde(default)]
    missing: Vec<String>,
    #[serde(default)]
    head_commit: Option<String>,
}

/// Parse the stamp payload. The error names only where the JSON broke, never
/// what it said: stdin text stays out of the message.
fn stamp_payload(
    noun: &str,
    flagged: bool,
    stdin: Option<String>,
) -> Result<StampPayload, CommandError> {
    if !flagged {
        return Ok(StampPayload::default());
    }
    let shape = r#"{"scope": [{"path", "hash"}], "missing": [], "headCommit": "<sha>"}"#;
    let Some(text) = stdin else {
        return Err(CommandError::new(
            ErrorCode::InvalidArgv,
            format!("{noun} stamp --stdin expects {shape} on stdin, and none was given"),
        ));
    };
    serde_json::from_str(&text).map_err(|e| {
        let what = match e.classify() {
            serde_json::error::Category::Data => "does not have that shape",
            _ => "is not valid JSON",
        };
        CommandError::new(
            ErrorCode::InvalidArgv,
            format!("{noun} stamp --stdin expects {shape}: the input {what} (line {}, column {})", e.line(), e.column()),
        )
    })
}

/// The id of the stamps' host-only `--stdin`.
const STAMP_STDIN: &str = "stdin";

/// A host without a checkout passes a stamp's scope and missing lists as JSON
/// on stdin, under a hidden `--stdin`. `parse` adds the flag to the two
/// stamps of the tree it parses with; `SpecVerb`, which the CLI flattens,
/// never has it. `mut_subcommands` keeps the subcommand order.
pub(crate) fn with_stamp_stdin(tree: clap::Command) -> clap::Command {
    tree.mut_subcommands(|station| match station.get_name() {
        "review" | "verify" => station.mut_subcommands(|sub| {
            if sub.get_name() != "stamp" {
                return sub;
            }
            sub.arg(clap::Arg::new(STAMP_STDIN).long("stdin").action(clap::ArgAction::SetTrue).hide(true))
        }),
        _ => station,
    })
}

/// For a parsed stamp, whether `--stdin` came with it; `None` for every
/// other verb.
pub(crate) fn stamp_stdin_of(matches: &clap::ArgMatches) -> Option<bool> {
    let ("review" | "verify", station) = matches.subcommand()? else { return None };
    let ("stamp", stamp) = station.subcommand()? else { return None };
    Some(stamp.get_flag(STAMP_STDIN))
}

/// Which station a shared subcommand belongs to: it picks the engine command
/// and the noun in messages.
#[derive(Clone, Copy)]
enum Station {
    Review,
    Verify,
}

impl Station {
    fn noun(self) -> &'static str {
        match self {
            Station::Review => "review",
            Station::Verify => "verify",
        }
    }
}

/// The subcommands both stations share, read off either station's clap
/// value. `prepare` (review only) and `scope` work on the checkout's git
/// state and snapshots, so they have no engine command.
enum StationSub<'a> {
    LocalOnly,
    AddRound { change: &'a str, stdin: bool },
    Show { change: &'a str },
    Stamp { change: &'a str, accept: bool, agent: &'a Option<String> },
    Discard { change: &'a str },
}

impl StationSub<'_> {
    /// A stamp's `--stdin` is host-only: `ParsedVerb` gives a stamp its rule.
    fn stdin_policy(&self) -> StdinPolicy {
        match *self {
            StationSub::AddRound { stdin, .. } => StdinPolicy::FlagOrPiped { flagged: stdin },
            StationSub::LocalOnly | StationSub::Show { .. } | StationSub::Stamp { .. } | StationSub::Discard { .. } => {
                StdinPolicy::Never
            }
        }
    }

    fn to_command(
        self,
        station: Station,
        name: &'static str,
        inputs: VerbInputs,
        stamp_stdin: bool,
    ) -> Result<CommandTarget, CommandError> {
        use Station::{Review, Verify};
        let cmd = match self {
            StationSub::LocalOnly => {
                return Ok(CommandTarget::LocalOnly { verb: name, reason: LocalOnlyReason::NeedsCheckout })
            }
            StationSub::AddRound { change, .. } => {
                let (change, content) = (change.to_string(), inputs.stdin.unwrap_or_default());
                match station {
                    Review => Command::ReviewAddRound { change, content },
                    Verify => Command::VerifyAddRound { change, content },
                }
            }
            StationSub::Show { change } => {
                let change = change.to_string();
                match station {
                    Review => Command::ReviewShow { change },
                    Verify => Command::VerifyShow { change },
                }
            }
            StationSub::Stamp { change, accept, agent } => {
                let StampPayload { scope, missing, head_commit } =
                    stamp_payload(station.noun(), stamp_stdin, inputs.stdin)?;
                // The commit the stdin JSON names, else the caller's HEAD.
                let head_commit = head_commit.or(inputs.head_commit);
                let (change, tool) = (change.to_string(), agent.clone());
                match station {
                    Review => Command::ReviewStamp { change, accept, tool, head_commit, scope, missing },
                    Verify => Command::VerifyStamp { change, accept, tool, head_commit, scope, missing },
                }
            }
            StationSub::Discard { change } => {
                let change = change.to_string();
                match station {
                    Review => Command::ReviewDiscard { change },
                    Verify => Command::VerifyDiscard { change },
                }
            }
        };
        Ok(CommandTarget::Command(cmd))
    }
}

impl ReviewArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            ReviewCommands::Prepare { .. } => "review prepare",
            ReviewCommands::Scope { .. } => "review scope",
            ReviewCommands::AddRound { .. } => "review add-round",
            ReviewCommands::Show { .. } => "review show",
            ReviewCommands::Stamp { .. } => "review stamp",
            ReviewCommands::Discard { .. } => "review discard",
        }
    }

    pub fn stdin_policy(&self) -> StdinPolicy {
        self.sub().stdin_policy()
    }

    /// The engine command with the stamp's host-only `--stdin` applied
    /// (`parse` calls this; `ToCommand` passes `false`).
    pub(crate) fn command_with(&self, inputs: VerbInputs, stamp_stdin: bool) -> Result<CommandTarget, CommandError> {
        self.sub().to_command(Station::Review, self.name(), inputs, stamp_stdin)
    }

    fn sub(&self) -> StationSub<'_> {
        match &self.command {
            ReviewCommands::Prepare { .. } | ReviewCommands::Scope { .. } => StationSub::LocalOnly,
            ReviewCommands::AddRound { change, stdin } => StationSub::AddRound { change, stdin: *stdin },
            ReviewCommands::Show { change, .. } => StationSub::Show { change },
            ReviewCommands::Stamp { change, accept, agent } => StationSub::Stamp { change, accept: *accept, agent },
            ReviewCommands::Discard { change } => StationSub::Discard { change },
        }
    }
}

impl ToCommand for ReviewArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        self.command_with(inputs, false)
    }
}

impl VerifyArgs {
    pub fn name(&self) -> &'static str {
        match self.command {
            VerifyCommands::Scope { .. } => "verify scope",
            VerifyCommands::AddRound { .. } => "verify add-round",
            VerifyCommands::Show { .. } => "verify show",
            VerifyCommands::Stamp { .. } => "verify stamp",
            VerifyCommands::Discard { .. } => "verify discard",
        }
    }

    pub fn stdin_policy(&self) -> StdinPolicy {
        self.sub().stdin_policy()
    }

    /// See `ReviewArgs::command_with`.
    pub(crate) fn command_with(&self, inputs: VerbInputs, stamp_stdin: bool) -> Result<CommandTarget, CommandError> {
        self.sub().to_command(Station::Verify, self.name(), inputs, stamp_stdin)
    }

    fn sub(&self) -> StationSub<'_> {
        match &self.command {
            VerifyCommands::Scope { .. } => StationSub::LocalOnly,
            VerifyCommands::AddRound { change, stdin } => StationSub::AddRound { change, stdin: *stdin },
            VerifyCommands::Show { change, .. } => StationSub::Show { change },
            VerifyCommands::Stamp { change, accept, agent } => StationSub::Stamp { change, accept: *accept, agent },
            VerifyCommands::Discard { change } => StationSub::Discard { change },
        }
    }
}

impl ToCommand for VerifyArgs {
    fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        self.command_with(inputs, false)
    }
}

#[cfg(test)]
mod tests {
    use crate::families::test_support::{command_of, local_only_of, name_of, policy_of, target_of};
    use crate::{StdinPolicy, VerbInputs};
    use speclink_core::command::{Command, ErrorCode};
    use speclink_core::model::ReviewedScopeEntry;

    fn with_stdin(content: &str) -> VerbInputs {
        VerbInputs { stdin: Some(content.into()), ..Default::default() }
    }

    #[test]
    fn review_and_verify_terminals_convert() {
        let cases: &[(&[&str], Command)] = &[
            (&["review", "show", "demo", "--json"], Command::ReviewShow { change: "demo".into() }),
            (&["review", "discard", "demo"], Command::ReviewDiscard { change: "demo".into() }),
            (&["verify", "show", "demo"], Command::VerifyShow { change: "demo".into() }),
            (&["verify", "discard", "demo"], Command::VerifyDiscard { change: "demo".into() }),
            (
                &["review", "stamp", "demo", "--accept", "--agent", "claude"],
                Command::ReviewStamp {
                    change: "demo".into(),
                    accept: true,
                    tool: Some("claude".into()),
                    head_commit: None,
                    scope: vec![],
                    missing: vec![],
                },
            ),
            (
                &["verify", "stamp", "demo"],
                Command::VerifyStamp { change: "demo".into(), accept: false, tool: None, head_commit: None, scope: vec![], missing: vec![] },
            ),
        ];
        for (argv, want) in cases {
            assert_eq!(&command_of(argv, VerbInputs::default()), want, "{argv:?}");
        }
        assert_eq!(
            command_of(&["review", "add-round", "demo", "--stdin"], with_stdin("## Round")),
            Command::ReviewAddRound { change: "demo".into(), content: "## Round".into() }
        );
        assert_eq!(
            command_of(&["verify", "add-round", "demo"], with_stdin("## Round")),
            Command::VerifyAddRound { change: "demo".into(), content: "## Round".into() }
        );
    }

    #[test]
    fn stamp_scope_and_missing_come_from_the_stdin_json() {
        let json = r#"{"scope": [{"path": "src/a.rs", "hash": "h1"}], "missing": ["src/gone.rs"]}"#;
        assert_eq!(
            command_of(&["review", "stamp", "demo", "--stdin"], with_stdin(json)),
            Command::ReviewStamp {
                change: "demo".into(),
                accept: false,
                tool: None,
                head_commit: None,
                scope: vec![ReviewedScopeEntry { path: "src/a.rs".into(), hash: "h1".into() }],
                missing: vec!["src/gone.rs".into()],
            }
        );
        // Both fields absent read as empty lists.
        assert_eq!(
            command_of(&["verify", "stamp", "demo", "--stdin"], with_stdin("{}")),
            Command::VerifyStamp { change: "demo".into(), accept: false, tool: None, head_commit: None, scope: vec![], missing: vec![] }
        );
    }

    #[test]
    fn a_stamp_carries_the_callers_head_commit() {
        // The CLI passes its checkout's HEAD; without one the stamp carries none.
        let head = || VerbInputs { head_commit: Some("abc1234".into()), ..Default::default() };
        assert_eq!(
            command_of(&["review", "stamp", "demo"], head()),
            Command::ReviewStamp {
                change: "demo".into(),
                accept: false,
                tool: None,
                head_commit: Some("abc1234".into()),
                scope: vec![],
                missing: vec![],
            }
        );
        assert_eq!(
            command_of(&["verify", "stamp", "demo", "--accept"], head()),
            Command::VerifyStamp {
                change: "demo".into(),
                accept: true,
                tool: None,
                head_commit: Some("abc1234".into()),
                scope: vec![],
                missing: vec![],
            }
        );
    }

    #[test]
    fn the_stamp_stdin_json_can_name_the_head_commit() {
        // A host without a checkout names the commit itself; it wins over the
        // caller's HEAD input.
        let json = r#"{"scope": [], "headCommit": "0123456789ab"}"#;
        let inputs = VerbInputs { stdin: Some(json.into()), head_commit: Some("abc1234".into()), ..Default::default() };
        assert_eq!(
            command_of(&["review", "stamp", "demo", "--stdin"], inputs),
            Command::ReviewStamp {
                change: "demo".into(),
                accept: false,
                tool: None,
                head_commit: Some("0123456789ab".into()),
                scope: vec![],
                missing: vec![],
            }
        );
        let inputs = VerbInputs { stdin: Some("{}".into()), head_commit: Some("abc1234".into()), ..Default::default() };
        let Command::VerifyStamp { head_commit, .. } = command_of(&["verify", "stamp", "demo", "--stdin"], inputs) else {
            panic!("a verify stamp");
        };
        assert_eq!(head_commit.as_deref(), Some("abc1234"), "absent in the JSON: the caller's HEAD");
        let bad = VerbInputs { stdin: Some(r#"{"headCommit": 7}"#.into()), ..Default::default() };
        let err = target_of(&["review", "stamp", "demo", "--stdin"], bad).expect_err("a number is not a commit");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
    }

    #[test]
    fn bad_stamp_json_is_invalid_argv() {
        for stdin in [Some(r#"{"scope": [], "extra": 1}"#), Some("not json"), None] {
            let inputs = VerbInputs { stdin: stdin.map(str::to_string), ..Default::default() };
            let err = target_of(&["review", "stamp", "demo", "--stdin"], inputs).expect_err("bad stamp stdin");
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{stdin:?}");
        }
    }

    /// The message names where the JSON broke, never what stdin said.
    #[test]
    fn stamp_json_errors_carry_no_stdin_text() {
        for stdin in [
            r#"{"scope": "SECRET-1"}"#,
            r#"{"SECRET-2": 1}"#,
            r#"{"scope": [{"path": "SECRET-3"}]}"#,
            "SECRET-4 is not json",
        ] {
            let inputs = VerbInputs { stdin: Some(stdin.into()), ..Default::default() };
            let err = target_of(&["verify", "stamp", "demo", "--stdin"], inputs).expect_err("bad stamp stdin");
            assert_eq!(err.code, ErrorCode::InvalidArgv);
            assert!(!err.message.contains("SECRET"), "{stdin}: {}", err.message);
            assert!(err.message.contains("line 1, column"), "{}", err.message);
        }
    }

    #[test]
    fn scope_and_prepare_are_local_only() {
        assert_eq!(local_only_of(&["review", "prepare", "demo"]), Some("review prepare"));
        assert_eq!(local_only_of(&["review", "scope", "demo", "--json"]), Some("review scope"));
        assert_eq!(local_only_of(&["verify", "scope", "demo"]), Some("verify scope"));
    }

    #[test]
    fn names_and_stdin_policies() {
        assert_eq!(name_of(&["review", "stamp", "x"]), "review stamp");
        assert_eq!(name_of(&["verify", "add-round", "x"]), "verify add-round");
        assert_eq!(policy_of(&["review", "add-round", "x"]), StdinPolicy::FlagOrPiped { flagged: false });
        assert_eq!(policy_of(&["verify", "add-round", "x", "--stdin"]), StdinPolicy::FlagOrPiped { flagged: true });
        assert_eq!(policy_of(&["review", "stamp", "x", "--stdin"]), StdinPolicy::WhenFlagged { flagged: true });
        assert_eq!(policy_of(&["verify", "stamp", "x"]), StdinPolicy::WhenFlagged { flagged: false });
        assert_eq!(policy_of(&["review", "stamp", "demo"]), StdinPolicy::WhenFlagged { flagged: false });
        for argv in [&["review", "show", "x"][..], &["verify", "discard", "x"], &["review", "prepare", "x"]] {
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }

    /// The CLI flattens `SpecVerb`, so its stamps carry no host-only
    /// `--stdin`; only `parse` adds it.
    #[test]
    fn the_shared_stamps_have_no_stdin_flag() {
        use clap::Subcommand;
        let tree = crate::SpecVerb::augment_subcommands(clap::Command::new("speclink"));
        for station in ["review", "verify"] {
            let stamp = tree
                .find_subcommand(station)
                .and_then(|s| s.find_subcommand("stamp"))
                .expect("the station has a stamp");
            assert!(stamp.get_arguments().all(|a| a.get_id() != "stdin"), "{station} stamp carries --stdin");
        }
    }

    /// The CLI's view of a stamp (`SpecVerb` parsed on its own tree) reads no
    /// stdin; only `ParsedVerb` gives a stamp its `--stdin` rule.
    #[test]
    fn the_cli_view_of_a_stamp_reads_no_stdin() {
        use clap::{FromArgMatches, Subcommand};
        let tree = crate::SpecVerb::augment_subcommands(clap::Command::new("speclink"));
        for station in ["review", "verify"] {
            let matches = tree.clone().get_matches_from(["speclink", station, "stamp", "demo"]);
            let verb = crate::SpecVerb::from_arg_matches(&matches).expect("a stamp");
            assert_eq!(verb.stdin_policy(), StdinPolicy::Never, "{station} stamp");
        }
    }
}

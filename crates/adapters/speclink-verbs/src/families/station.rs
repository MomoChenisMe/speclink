//! The two quality stations: review and verify.

use clap::{Args, Subcommand};
use speclink_core::command::{Command, CommandError, ErrorCode};
use speclink_core::model::ReviewedScopeEntry;

use crate::{CommandTarget, StdinPolicy, VerbInputs};

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
        // A host without a checkout passes the scope and missing lists as
        // JSON on stdin; the CLI computes them itself and refuses the flag.
        #[arg(long, hide = true)]
        stdin: bool,
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
        // See `ReviewCommands::Stamp`.
        #[arg(long, hide = true)]
        stdin: bool,
    },
    /// Discard the verify ticket without stamping
    Discard { change: String },
}

/// The `--stdin` payload of a stamp: the work-tree holder's `(path, hash)`
/// list and the union paths that are gone. Both default to empty; any other
/// field is refused.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StampPayload {
    #[serde(default)]
    scope: Vec<ReviewedScopeEntry>,
    #[serde(default)]
    missing: Vec<String>,
}

/// Parse the stamp payload. The error names only where the JSON broke, never
/// what it said: stdin text stays out of the message.
fn stamp_payload(
    noun: &str,
    flagged: bool,
    stdin: Option<String>,
) -> Result<(Vec<ReviewedScopeEntry>, Vec<String>), CommandError> {
    if !flagged {
        return Ok((Vec::new(), Vec::new()));
    }
    let shape = r#"{"scope": [{"path", "hash"}], "missing": []}"#;
    let Some(text) = stdin else {
        return Err(CommandError::new(
            ErrorCode::InvalidArgv,
            format!("{noun} stamp --stdin expects {shape} on stdin, and none was given"),
        ));
    };
    let payload: StampPayload = serde_json::from_str(&text).map_err(|e| {
        let what = match e.classify() {
            serde_json::error::Category::Data => "does not have that shape",
            _ => "is not valid JSON",
        };
        CommandError::new(
            ErrorCode::InvalidArgv,
            format!("{noun} stamp --stdin expects {shape}: the input {what} (line {}, column {})", e.line(), e.column()),
        )
    })?;
    Ok((payload.scope, payload.missing))
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
        match self.command {
            ReviewCommands::AddRound { stdin, .. } => StdinPolicy::FlagOrPiped { flagged: stdin },
            ReviewCommands::Stamp { stdin, .. } => StdinPolicy::WhenFlagged { flagged: stdin },
            ReviewCommands::Prepare { .. }
            | ReviewCommands::Scope { .. }
            | ReviewCommands::Show { .. }
            | ReviewCommands::Discard { .. } => StdinPolicy::Never,
        }
    }

    /// `prepare` and `scope` work on the checkout's git state and snapshots,
    /// so they have no engine command.
    pub fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(match &self.command {
            ReviewCommands::Prepare { .. } => return Ok(CommandTarget::LocalOnly { verb: "review prepare" }),
            ReviewCommands::Scope { .. } => return Ok(CommandTarget::LocalOnly { verb: "review scope" }),
            ReviewCommands::AddRound { change, .. } => Command::ReviewAddRound {
                change: change.clone(),
                content: inputs.stdin.unwrap_or_default(),
            },
            ReviewCommands::Show { change, .. } => Command::ReviewShow { change: change.clone() },
            ReviewCommands::Stamp { change, accept, agent, stdin } => {
                let (scope, missing) = stamp_payload("review", *stdin, inputs.stdin)?;
                Command::ReviewStamp {
                    change: change.clone(),
                    accept: *accept,
                    tool: agent.clone(),
                    scope,
                    missing,
                }
            }
            ReviewCommands::Discard { change } => Command::ReviewDiscard { change: change.clone() },
        }))
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
        match self.command {
            VerifyCommands::AddRound { stdin, .. } => StdinPolicy::FlagOrPiped { flagged: stdin },
            VerifyCommands::Stamp { stdin, .. } => StdinPolicy::WhenFlagged { flagged: stdin },
            VerifyCommands::Scope { .. } | VerifyCommands::Show { .. } | VerifyCommands::Discard { .. } => {
                StdinPolicy::Never
            }
        }
    }

    pub fn to_command(&self, inputs: VerbInputs) -> Result<CommandTarget, CommandError> {
        Ok(CommandTarget::Command(match &self.command {
            VerifyCommands::Scope { .. } => return Ok(CommandTarget::LocalOnly { verb: "verify scope" }),
            VerifyCommands::AddRound { change, .. } => Command::VerifyAddRound {
                change: change.clone(),
                content: inputs.stdin.unwrap_or_default(),
            },
            VerifyCommands::Show { change, .. } => Command::VerifyShow { change: change.clone() },
            VerifyCommands::Stamp { change, accept, agent, stdin } => {
                let (scope, missing) = stamp_payload("verify", *stdin, inputs.stdin)?;
                Command::VerifyStamp {
                    change: change.clone(),
                    accept: *accept,
                    tool: agent.clone(),
                    scope,
                    missing,
                }
            }
            VerifyCommands::Discard { change } => Command::VerifyDiscard { change: change.clone() },
        }))
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
                    scope: vec![],
                    missing: vec![],
                },
            ),
            (
                &["verify", "stamp", "demo"],
                Command::VerifyStamp { change: "demo".into(), accept: false, tool: None, scope: vec![], missing: vec![] },
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
                scope: vec![ReviewedScopeEntry { path: "src/a.rs".into(), hash: "h1".into() }],
                missing: vec!["src/gone.rs".into()],
            }
        );
        // Both fields absent read as empty lists.
        assert_eq!(
            command_of(&["verify", "stamp", "demo", "--stdin"], with_stdin("{}")),
            Command::VerifyStamp { change: "demo".into(), accept: false, tool: None, scope: vec![], missing: vec![] }
        );
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
        for argv in [&["review", "show", "x"][..], &["verify", "discard", "x"], &["review", "prepare", "x"]] {
            assert_eq!(policy_of(argv), StdinPolicy::Never, "{argv:?}");
        }
    }
}

//! `speclink` 二進位的 clap surface — 頂層 `Cli`、`propose` subcommand 樹、共用 machine
//! interface 旗標。

use clap::{Args, Parser, Subcommand};

/// 頂層 CLI 結構。
#[derive(Debug, Parser)]
#[command(
    name = "speclink",
    version,
    about = "SpecLink — spec-driven development workflow for AI agents",
    propagate_version = true
)]
pub struct Cli {
    /// CLI 子命令。
    #[command(subcommand)]
    pub command: Command,
}

/// 頂層 CLI 子命令。MVP 僅一個。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Propose 工作流子命令樹。
    #[command(subcommand)]
    Propose(ProposeCommand),
}

/// `propose` subcommand 樹。
#[derive(Debug, Subcommand)]
pub enum ProposeCommand {
    /// 建立新 change proposal。
    Create(ProposeCreateArgs),
}

/// `speclink propose create` 的參數。
#[derive(Debug, Args)]
pub struct ProposeCreateArgs {
    /// Change 識別碼（kebab-case）。
    #[arg(long)]
    pub change: String,

    /// 一行摘要（最大 200 字元，非空）。
    #[arg(long, value_parser = parse_summary)]
    pub summary: String,

    /// 共用 machine-interface 旗標。
    #[command(flatten)]
    pub flags: MachineInterfaceFlags,
}

/// Summary 在 clap 層的驗證：拒絕空字串與超過 200 字元的輸入。
fn parse_summary(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("summary must not be empty".to_string());
    }
    if s.chars().count() > runtime::propose::MAX_SUMMARY_LEN {
        return Err(format!(
            "summary exceeds maximum length of {} characters",
            runtime::propose::MAX_SUMMARY_LEN
        ));
    }
    Ok(s.to_string())
}

/// 跨命令共用的 machine-interface 旗標。
#[derive(Debug, Args)]
pub struct MachineInterfaceFlags {
    /// 以單行 JSON envelope 輸出至 stdout。
    #[arg(long)]
    pub json: bool,

    /// 停用 stderr ANSI 色碼。
    #[arg(long)]
    pub no_color: bool,

    /// 抑制 INFO 等級以下的 stderr 輸出。
    #[arg(long)]
    pub quiet: bool,

    /// 從 stdin 讀取 payload；本 change 任何指令收到 `--stdin` 都會回傳 exit 2。
    #[arg(long)]
    pub stdin: bool,
}

#[cfg(test)]
mod tests {
    use crate::cli::{Cli, Command, ProposeCommand};
    use clap::Parser;

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("speclink").chain(args.iter().copied()))
    }

    #[test]
    fn parse_propose_create_full() {
        let cli = parse(&[
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--json",
            "--no-color",
            "--quiet",
        ])
        .expect("parse");
        match cli.command {
            Command::Propose(p) => match p {
                ProposeCommand::Create(args) => {
                    assert_eq!(args.change, "demo");
                    assert_eq!(args.summary, "x");
                    assert!(args.flags.json);
                    assert!(args.flags.no_color);
                    assert!(args.flags.quiet);
                    assert!(!args.flags.stdin);
                }
            },
        }
    }

    #[test]
    fn empty_summary_rejected_by_clap() {
        let err = parse(&["propose", "create", "--change", "demo", "--summary", ""])
            .expect_err("empty summary must be rejected");
        // clap parse error → exit code 2 mapping handled by main()
        let _ = err;
    }

    #[test]
    fn missing_required_flag_errors() {
        let err = parse(&["propose", "create", "--summary", "x"]).expect_err("missing --change");
        let _ = err;
    }

    #[test]
    fn no_subcommand_shows_help_error() {
        // clap 不會把 no-args 視為成功 — 要求 subcommand
        let res = parse(&[]);
        assert!(res.is_err(), "no subcommand must error");
    }

    #[test]
    fn stdin_flag_present_in_machine_interface() {
        let cli = parse(&[
            "propose",
            "create",
            "--change",
            "demo",
            "--summary",
            "x",
            "--stdin",
        ])
        .expect("parse stdin");
        match cli.command {
            Command::Propose(p) => match p {
                ProposeCommand::Create(args) => assert!(args.flags.stdin),
            },
        }
    }
}

//! `speclink` 二進位的 clap surface — 頂層 `Cli`、`propose` / `artifact` / `status`
//! subcommand 樹、共用 machine interface 旗標、共用 value parser。

use clap::{Args, Parser, Subcommand};
use provider_local::storage::is_valid_kebab_id;

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

/// 頂層 CLI 子命令。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Propose 工作流子命令樹。
    #[command(subcommand)]
    Propose(ProposeCommand),
    /// Artifact 工作流子命令樹。
    #[command(subcommand)]
    Artifact(ArtifactCommand),
    /// 觀察 change 進度。
    Status(StatusArgs),
}

/// `artifact` subcommand 樹。
#[derive(Debug, Subcommand)]
pub enum ArtifactCommand {
    /// 寫入 artifact（design / tasks / spec）。
    #[command(subcommand)]
    Write(ArtifactWriteCommand),
}

/// `artifact write` 子命令樹（kind 為 positional subcommand）。
#[derive(Debug, Subcommand)]
pub enum ArtifactWriteCommand {
    /// 寫入 `design.md`。
    Design(ArtifactWriteArgs),
    /// 寫入 `tasks.md`。
    Tasks(ArtifactWriteArgs),
    /// 寫入 `specs/<capability>/spec.md`。
    Spec(ArtifactWriteSpecArgs),
}

/// `artifact write design` / `artifact write tasks` 的參數。
#[derive(Debug, Args)]
pub struct ArtifactWriteArgs {
    /// Change 識別碼（kebab-case）。
    #[arg(long, value_parser = parse_change_id)]
    pub change: String,
    /// 從 stdin 讀取 artifact 內容；REQUIRED。
    #[arg(long, required = true)]
    pub stdin: bool,
    /// 雖然 design / tasks 不接受 `--capability`，仍開放欄位以利 runtime 統一回 `input.invalid`。
    #[arg(long)]
    pub capability: Option<String>,
    /// 以單行 JSON envelope 輸出至 stdout。
    #[arg(long)]
    pub json: bool,
    /// 停用 stderr ANSI 色碼。
    #[arg(long)]
    pub no_color: bool,
    /// 抑制 INFO 等級以下的 stderr 輸出。
    #[arg(long)]
    pub quiet: bool,
}

/// `artifact write spec` 的參數。
#[derive(Debug, Args)]
pub struct ArtifactWriteSpecArgs {
    /// Change 識別碼（kebab-case）。
    #[arg(long, value_parser = parse_change_id)]
    pub change: String,
    /// Capability 名稱（kebab-case；同 change-id 規則）。
    #[arg(long, value_parser = parse_capability_name)]
    pub capability: String,
    /// 從 stdin 讀取 artifact 內容；REQUIRED。
    #[arg(long, required = true)]
    pub stdin: bool,
    /// 以單行 JSON envelope 輸出至 stdout。
    #[arg(long)]
    pub json: bool,
    /// 停用 stderr ANSI 色碼。
    #[arg(long)]
    pub no_color: bool,
    /// 抑制 INFO 等級以下的 stderr 輸出。
    #[arg(long)]
    pub quiet: bool,
}

/// `status` 命令的參數。
#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Change 識別碼（kebab-case）。
    #[arg(long, value_parser = parse_change_id)]
    pub change: String,
    /// 共用 machine-interface 旗標；`status` 不接受 `--stdin`，傳入會在 runtime 回 `input.invalid`。
    #[command(flatten)]
    pub flags: MachineInterfaceFlags,
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
    #[arg(long, value_parser = parse_change_id)]
    pub change: String,

    /// 一行摘要（最大 200 字元，非空）。
    #[arg(long, value_parser = parse_summary)]
    pub summary: String,

    /// 共用 machine-interface 旗標。
    #[command(flatten)]
    pub flags: MachineInterfaceFlags,
}

/// Clap value_parser：kebab-case change-id 驗證。
///
/// 規則：`^[a-z][a-z0-9-]{0,63}$`、不含連續 hyphen、不以 hyphen 結尾。
pub fn parse_change_id(s: &str) -> Result<String, String> {
    if is_valid_kebab_id(s) {
        Ok(s.to_string())
    } else {
        Err(format!("invalid change id: '{s}'"))
    }
}

/// Clap value_parser：kebab-case capability-name 驗證（同 change-id）。
pub fn parse_capability_name(s: &str) -> Result<String, String> {
    if is_valid_kebab_id(s) {
        Ok(s.to_string())
    } else {
        Err(format!("invalid capability name: '{s}'"))
    }
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
    use crate::cli::{
        ArtifactCommand, ArtifactWriteCommand, Cli, Command, ProposeCommand, parse_capability_name,
        parse_change_id,
    };
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
            _ => panic!("expected Propose"),
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
            _ => panic!("expected Propose"),
        }
    }

    #[test]
    fn parse_change_id_accepts_kebab() {
        assert!(parse_change_id("demo").is_ok());
        assert!(parse_change_id("add-feature").is_ok());
    }

    #[test]
    fn parse_change_id_rejects_invalid() {
        assert!(parse_change_id("Add-Feature").is_err());
        assert!(parse_change_id("1bad").is_err());
        assert!(parse_change_id("add--feature").is_err());
        assert!(parse_change_id("add-").is_err());
        assert!(parse_change_id("").is_err());
    }

    #[test]
    fn parse_capability_name_rejects_invalid() {
        assert!(parse_capability_name("auth").is_ok());
        assert!(parse_capability_name("user-auth").is_ok());
        assert!(parse_capability_name("Auth-Module").is_err());
        assert!(parse_capability_name("1bad").is_err());
    }

    #[test]
    fn parse_artifact_write_design() {
        let cli = parse(&[
            "artifact", "write", "design", "--change", "demo", "--stdin", "--json",
        ])
        .expect("parse design");
        match cli.command {
            Command::Artifact(ArtifactCommand::Write(ArtifactWriteCommand::Design(args))) => {
                assert_eq!(args.change, "demo");
                assert!(args.stdin);
                assert!(args.json);
                assert!(args.capability.is_none());
            }
            _ => panic!("expected artifact write design"),
        }
    }

    #[test]
    fn parse_artifact_write_tasks() {
        let cli = parse(&["artifact", "write", "tasks", "--change", "demo", "--stdin"])
            .expect("parse tasks");
        match cli.command {
            Command::Artifact(ArtifactCommand::Write(ArtifactWriteCommand::Tasks(args))) => {
                assert_eq!(args.change, "demo");
                assert!(args.stdin);
            }
            _ => panic!("expected artifact write tasks"),
        }
    }

    #[test]
    fn parse_artifact_write_spec() {
        let cli = parse(&[
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "user-auth",
            "--stdin",
            "--json",
        ])
        .expect("parse spec");
        match cli.command {
            Command::Artifact(ArtifactCommand::Write(ArtifactWriteCommand::Spec(args))) => {
                assert_eq!(args.change, "demo");
                assert_eq!(args.capability, "user-auth");
                assert!(args.stdin);
                assert!(args.json);
            }
            _ => panic!("expected artifact write spec"),
        }
    }

    #[test]
    fn artifact_write_design_missing_stdin_errors() {
        let err = parse(&["artifact", "write", "design", "--change", "demo"])
            .expect_err("missing --stdin");
        let _ = err;
    }

    #[test]
    fn artifact_write_spec_missing_capability_errors() {
        let err = parse(&["artifact", "write", "spec", "--change", "demo", "--stdin"])
            .expect_err("missing --capability");
        let _ = err;
    }

    #[test]
    fn artifact_write_invalid_change_id_errors() {
        let err = parse(&[
            "artifact",
            "write",
            "design",
            "--change",
            "Add-Feature",
            "--stdin",
        ])
        .expect_err("invalid change id");
        let _ = err;
    }

    #[test]
    fn artifact_write_invalid_capability_errors() {
        let err = parse(&[
            "artifact",
            "write",
            "spec",
            "--change",
            "demo",
            "--capability",
            "Bad-Name",
            "--stdin",
        ])
        .expect_err("invalid capability");
        let _ = err;
    }

    #[test]
    fn parse_status_command() {
        let cli = parse(&["status", "--change", "demo", "--json"]).expect("parse status");
        match cli.command {
            Command::Status(args) => {
                assert_eq!(args.change, "demo");
                assert!(args.flags.json);
            }
            _ => panic!("expected status"),
        }
    }
}

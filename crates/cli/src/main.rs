//! SpecLink CLI 入口。
//!
//! 解析 clap subcommand → 呼叫 runtime → 用 JSON envelope 輸出 + 對應 exit code。

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use speclink_cli::commands;
use speclink_cli::output::{Envelope, Warning, error, success};
use speclink_runtime::RuntimeError;

#[derive(Parser, Debug)]
#[command(
    name = "speclink",
    version,
    about = "SpecLink — Spec-Driven Development workflow engine"
)]
struct Cli {
    /// 以 stable JSON envelope 輸出（給 AI / CI / tooling 使用）。
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 在當前目錄初始化 SpecLink project（artifact root + state root）。
    Init {
        #[arg(long)]
        force: bool,
    },
    /// 顯示當前 project 狀態。
    Status,
    /// 把當前目錄綁定到既存 project_id。
    Link { project_id: String },
    /// 移除當前目錄與 SpecLink project 的綁定（不刪 state.db、不刪 schemas）。
    Unlink,

    /// 建立新 change 或 artifact。
    New {
        #[command(subcommand)]
        sub: NewSub,
    },
    /// 顯示 change metadata 與 artifact 清單。
    Show {
        #[command(subcommand)]
        sub: ShowSub,
    },
    /// destructive：刪除 change row + filesystem 目錄。
    Delete {
        #[command(subcommand)]
        sub: DeleteSub,
    },
    /// 列舉資源。
    List(ListArgs),
    /// Artifact 動詞（目前僅 `read`）。
    Artifact {
        #[command(subcommand)]
        sub: ArtifactSub,
    },
}

#[derive(Subcommand, Debug)]
enum NewSub {
    /// 建立新 change。
    Change { name: String },
    /// 寫入 artifact；bytes 從 stdin 讀取。
    Artifact {
        /// `proposal` / `design` / `tasks` / `spec`
        kind: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        capability: Option<String>,
        #[arg(long = "expected-etag")]
        expected_etag: Option<String>,
        /// 從 stdin 讀 bytes 寫入檔案。
        #[arg(long)]
        stdin: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ShowSub {
    /// 顯示 change metadata。
    Change { name: String },
}

#[derive(Subcommand, Debug)]
enum DeleteSub {
    /// 刪除 change（destructive）。
    Change {
        name: String,
        #[arg(long = "confirm-name")]
        confirm_name: Option<String>,
    },
}

#[derive(Args, Debug)]
struct ListArgs {
    /// 列舉所有 change。
    #[arg(long)]
    changes: bool,
    /// 列舉某 change 下的 spec capability id。
    #[arg(long)]
    specs: bool,
    /// `--specs` 用的 change 名稱。
    #[arg(long)]
    change: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ArtifactSub {
    /// 讀 artifact。
    Read {
        kind: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        capability: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let use_json = cli.json;

    let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to start async runtime: {e}");
            return ExitCode::from(1);
        }
    };

    let result: Result<(serde_json::Value, Vec<Warning>), RuntimeError> = rt.block_on(async move {
        match cli.command {
            Commands::Init { force } => commands::init::run(&working_dir, force).await,
            Commands::Status => commands::status::run(&working_dir).await,
            Commands::Link { project_id } => commands::link::run(&working_dir, &project_id).await,
            Commands::Unlink => commands::unlink::run(&working_dir).await,
            Commands::New { sub } => match sub {
                NewSub::Change { name } => commands::new_change::run(&working_dir, &name).await,
                NewSub::Artifact {
                    kind,
                    change,
                    capability,
                    expected_etag,
                    stdin,
                } => {
                    commands::new_artifact::run(
                        &working_dir,
                        &kind,
                        &change,
                        capability.as_deref(),
                        expected_etag.as_deref(),
                        stdin,
                    )
                    .await
                }
            },
            Commands::Show { sub } => match sub {
                ShowSub::Change { name } => commands::show_change::run(&working_dir, &name).await,
            },
            Commands::Delete { sub } => match sub {
                DeleteSub::Change { name, confirm_name } => {
                    commands::delete_change::run(&working_dir, &name, confirm_name.as_deref()).await
                }
            },
            Commands::List(args) => dispatch_list(&working_dir, args).await,
            Commands::Artifact { sub } => match sub {
                ArtifactSub::Read {
                    kind,
                    change,
                    capability,
                } => {
                    commands::artifact_read::run(
                        &working_dir,
                        &kind,
                        &change,
                        capability.as_deref(),
                    )
                    .await
                }
            },
        }
    });

    match result {
        Ok((data, warnings)) => {
            if use_json {
                let env: Envelope<serde_json::Value> = success(data, warnings);
                println!("{}", serde_json::to_string(&env).unwrap_or_default());
            } else {
                print_human_ok(&data);
            }
            ExitCode::from(0)
        }
        Err(err) => {
            let code = err.code();
            let exit = err.exit_code();
            let hint = hint_for(code);
            if use_json {
                let env = error(code, &err.to_string(), hint, err.retryable());
                println!("{}", serde_json::to_string(&env).unwrap_or_default());
            } else {
                eprintln!("error[{code}]: {err}");
                if let Some(h) = hint {
                    eprintln!("hint: {h}");
                }
            }
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let code_u8: u8 = if (0..=255).contains(&exit) {
                exit as u8
            } else {
                1
            };
            ExitCode::from(code_u8)
        }
    }
}

async fn dispatch_list(
    working_dir: &std::path::Path,
    args: ListArgs,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    match (args.changes, args.specs, args.change) {
        (true, false, _) => commands::list_changes::run(working_dir).await,
        (false, true, Some(change)) => commands::list_specs::run(working_dir, &change).await,
        (false, true, None) => Err(RuntimeError::ChangeInvalidName {
            name: String::new(),
            reason: "`speclink list --specs` requires `--change <name>`".into(),
        }),
        (false, false, _) => Err(RuntimeError::Internal(
            "`speclink list` requires `--changes` or `--specs --change <name>`".into(),
        )),
        (true, true, _) => Err(RuntimeError::Internal(
            "`--changes` and `--specs` are mutually exclusive".into(),
        )),
    }
}

fn print_human_ok(data: &serde_json::Value) {
    println!("{}", speclink_cli::human::render_human(data));
}

fn hint_for(code: &str) -> Option<&'static str> {
    match code {
        "project.requires_git" => Some("Run `git init` first, then re-run `speclink init`."),
        "project.already_initialized" => {
            Some("Pass `--force` to re-initialize while preserving state.db.")
        }
        "project.not_initialized" => Some("Run `speclink init` first."),
        "project.link_target_not_found" => {
            Some("Check available project_ids via `speclink status`.")
        }
        "change.not_found" => Some("Verify the change name via `speclink list --changes`."),
        "change.duplicate_name" => {
            Some("Pick a different name or delete the existing change first.")
        }
        "change.invalid_name" => {
            Some("Change names match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with 1-64 byte length.")
        }
        "artifact.kind_invalid" => Some(
            "Use one of `proposal`, `design`, `tasks`, `spec`. Capability ids follow the same kebab-case grammar as change names.",
        ),
        "artifact.capability_required" => {
            Some("`speclink new artifact spec` requires `--capability <id>`.")
        }
        "artifact.not_found" => {
            Some("Confirm the file exists, or omit `--expected-etag` to create a new artifact.")
        }
        "artifact.version_conflict" => Some(
            "Re-read the artifact, supply its current `--expected-etag`, then retry the write.",
        ),
        _ => None,
    }
}

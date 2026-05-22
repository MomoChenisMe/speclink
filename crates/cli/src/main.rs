//! SpecLink CLI 入口。
//!
//! 解析 clap subcommand → 呼叫 runtime → 用 JSON envelope 輸出 + 對應 exit code。

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use speclink_cli::commands;
use speclink_cli::output::{Envelope, error, success};
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
        /// 強制覆寫既有 `.speclink/link.yaml`（保留 state.db）。
        #[arg(long)]
        force: bool,
    },
    /// 顯示當前 project 狀態。
    Status,
    /// 把當前目錄綁定到既存 project_id。
    Link {
        /// 既存 project_id（UUID v4 字串）。
        project_id: String,
    },
    /// 移除當前目錄與 SpecLink project 的綁定（不刪 state.db、不刪 schemas）。
    Unlink,
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

    let result: Result<serde_json::Value, RuntimeError> = rt.block_on(async move {
        match cli.command {
            Commands::Init { force } => commands::init::run(&working_dir, force).await,
            Commands::Status => commands::status::run(&working_dir).await,
            Commands::Link { project_id } => commands::link::run(&working_dir, &project_id).await,
            Commands::Unlink => commands::unlink::run(&working_dir).await,
        }
    });

    match result {
        Ok(data) => {
            if use_json {
                let env: Envelope<serde_json::Value> = success(data, vec![]);
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
                let env = error(code, &err.to_string(), hint, false);
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

fn print_human_ok(data: &serde_json::Value) {
    if data.is_object() && data.as_object().is_some_and(serde_json::Map::is_empty) {
        println!("OK");
        return;
    }
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            let display = match v {
                serde_json::Value::String(s) => s.clone(),
                _ => v.to_string(),
            };
            println!("{k}: {display}");
        }
    } else {
        println!("{data}");
    }
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
        _ => None,
    }
}

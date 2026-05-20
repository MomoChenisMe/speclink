//! `speclink` 二進位入口。

use clap::Parser;
use cli::cli::{ArtifactCommand, Cli, Command, ProposeCommand};
use cli::commands;
use std::process::ExitCode;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    init_tracing();
    let parsed = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // clap parse error；clap 內建會印 usage 到 stderr 並決定 exit code。
            // 對於 InvalidValue / MissingRequired / DisplayHelp / DisplayVersion，
            // 我們把 exit code 統一為 2（InvalidInput）或 0（顯示資訊）。
            let kind = e.kind();
            e.print().ok();
            return match kind {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    ExitCode::from(0)
                }
                _ => ExitCode::from(2),
            };
        }
    };

    let exit = match parsed.command {
        Command::Propose(ProposeCommand::Create(args)) => commands::propose::run(args).await,
        Command::Artifact(ArtifactCommand::Write(cmd)) => commands::artifact::run(cmd).await,
        Command::Status(args) => commands::status::run(args).await,
        Command::Archive(args) => commands::archive::run(args).await,
        Command::Instructions(cmd) => commands::instructions::run(cmd).await,
        Command::Task(cmd) => commands::task::run(cmd).await,
    };
    ExitCode::from(exit.as_u8())
}

fn init_tracing() {
    use cli::tracing_layer::RedactingWriter;
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // RedactingWriter 套在 stderr 之外，所有 tracing 訊息經由 redact 處理才寫出。
    let make_writer = || RedactingWriter::new(std::io::stderr());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(make_writer)
        .with_target(false)
        .try_init();
}

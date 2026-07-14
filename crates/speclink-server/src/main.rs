//! The `speclink-server` binary: load the configuration fail closed, build the
//! store, then bind and serve. A configuration failure prints the reason and
//! exits non-zero — before any port is bound.

use clap::Parser;
use speclink_server::state::AppState;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "speclink-server", about = "The official Speclink HTTP server")]
struct Args {
    /// Path to the server configuration file (YAML).
    #[arg(long)]
    config: PathBuf,
    /// Address to bind.
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Fail closed on configuration before anything else — no port is bound
    // until the config is valid and the store opens.
    let config = match speclink_server::config::load(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let store = match speclink_server::build_store(&config.store) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let state = AppState { store, config: Arc::new(config) };
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("cannot start async runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = runtime.block_on(speclink_server::serve(&args.addr, state)) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

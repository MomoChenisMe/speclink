//! The `speclink-server` binary. With no subcommand it runs the server: load
//! the configuration fail closed, build the store, then bind and serve — a
//! configuration failure prints the reason and exits non-zero before any port
//! is bound. The `invite` subcommand is the headless management entry (決策 3):
//! it mints a one-time invitation against the configured identity store and
//! prints its URL.

use chrono::{Duration, Utc};
use clap::{Args as ClapArgs, Parser, Subcommand};
use speclink_server::config::IdentityConfig;
use speclink_server::events::EventHub;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "speclink-server", about = "The official Speclink HTTP server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[command(flatten)]
    run: RunArgs,
}

/// The run-mode arguments, used when no subcommand is given.
#[derive(ClapArgs)]
struct RunArgs {
    /// Path to the server configuration file (YAML).
    #[arg(long)]
    config: Option<PathBuf>,
    /// Address to bind.
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[derive(Subcommand)]
enum Command {
    /// Create a one-time invitation and print its acceptance URL.
    Invite(InviteArgs),
}

#[derive(ClapArgs)]
struct InviteArgs {
    /// Path to the server configuration file (locates the identity database).
    #[arg(long)]
    config: PathBuf,
    /// The invitee's email (the login identity).
    #[arg(long)]
    email: String,
    /// The invitee's display name.
    #[arg(long)]
    display: String,
    /// A project to grant membership to (repeatable).
    #[arg(long = "project")]
    projects: Vec<String>,
    /// Grant the admin flag.
    #[arg(long, default_value_t = false)]
    admin: bool,
    /// Days until the invitation expires.
    #[arg(long = "expires-in-days", default_value_t = 7)]
    expires_in_days: i64,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Invite(args)) => run_invite(args),
        None => run_server(cli.run),
    }
}

/// Load the config and serve. Fail closed: a config or store failure prints the
/// reason and exits non-zero before any port is bound.
fn run_server(args: RunArgs) -> ExitCode {
    let Some(config_path) = args.config else {
        eprintln!("missing required argument --config");
        return ExitCode::FAILURE;
    };
    let config = match speclink_server::config::load(&config_path) {
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
    let identity = match speclink_server::build_identity(&config.identity) {
        Ok(identity) => identity,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    // First-run bootstrap (決策 3): while no admin exists and no live token
    // stands, mint a one-time setup token and print it once — the only place its
    // plaintext ever appears. An operator opens /setup with it to finish setup.
    match speclink_server::setup::ensure_setup_token(identity.as_ref()) {
        Ok(Some(token)) => {
            let base = config.public_url.trim_end_matches('/');
            println!(
                "Speclink 首次啟動：開啟 {base}/setup?token={token} 完成初始設定（此連結 24 小時內有效，且僅顯示這一次）。"
            );
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    }

    let events = EventHub::new(store.clone(), config.events.clone());
    let state = AppState { store, identity, config: Arc::new(config), events };
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

/// Mint an invitation against the configured identity store and print its URL.
/// A duplicate email or any store failure prints the reason and exits non-zero.
fn run_invite(args: InviteArgs) -> ExitCode {
    let config = match speclink_server::config::load(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let store = match &config.identity {
        IdentityConfig::Sqlite { path } => match IdentitySqlite::open(path) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },
        IdentityConfig::Memory => {
            eprintln!("invite needs a persistent identity store; the config declares an in-memory one");
            return ExitCode::FAILURE;
        }
    };

    // Every --project must name a registered project (決策 5): the registry lives
    // in the identity store now, not the config. An unregistered key is refused
    // non-zero, listing the registered keys.
    let registered: Vec<String> = match store.list_projects() {
        Ok(projects) => projects.into_iter().map(|p| p.key).collect(),
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    for project in &args.projects {
        if !registered.iter().any(|k| k == project) {
            let listed = if registered.is_empty() {
                "(none registered)".to_string()
            } else {
                registered.join(", ")
            };
            eprintln!("project '{project}' is not registered; registered projects: {listed}");
            return ExitCode::FAILURE;
        }
    }

    let invitation = NewInvitation {
        email: args.email,
        display: args.display,
        memberships: args.projects,
        admin: args.admin,
        expires_at: Utc::now() + Duration::days(args.expires_in_days),
    };
    match store.create_invitation(invitation) {
        Ok(token) => {
            let base = config.public_url.trim_end_matches('/');
            println!("{base}/invite/{token}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

//! The `speclink-server` binary. With no subcommand it runs the server: load
//! the configuration fail closed, build the store, then bind and serve — a
//! configuration failure prints the reason and exits non-zero before any port
//! is bound. The `invite` subcommand is the headless management entry (決策 3):
//! it mints a one-time invitation against the configured identity store and
//! prints its URL.

use chrono::{Duration, Utc};
use clap::{Args as ClapArgs, Parser, Subcommand};
use speclink_server::audit::AuditActor;
use speclink_server::config::IdentityConfig;
use speclink_server::events::EventHub;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewBackupRecord, NewInvitation};
use speclink_server::state::AppState;
use std::path::{Path, PathBuf};
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
    /// User management, the headless equivalent of the /admin actions.
    #[command(subcommand)]
    User(UserCommand),
    /// Token management.
    #[command(subcommand)]
    Token(TokenCommand),
    /// Project registry management.
    #[command(subcommand)]
    Project(ProjectCommand),
    /// Repo registry management.
    #[command(subcommand)]
    Repo(RepoCommand),
    /// Produce a single self-describing backup file (offline: the store and
    /// identity must not be under concurrent writes).
    Backup(BackupArgs),
    /// Verify a backup file's integrity without restoring it.
    VerifyBackup(VerifyBackupArgs),
    /// Restore a backup into an empty target, then validate it.
    Restore(RestoreArgs),
}

#[derive(ClapArgs)]
struct RestoreArgs {
    /// Path to the server configuration file (locates the empty target).
    #[arg(long)]
    config: PathBuf,
    /// The backup file to restore.
    #[arg(long)]
    input: PathBuf,
}

#[derive(ClapArgs)]
struct BackupArgs {
    /// Path to the server configuration file (locates the store and identity).
    #[arg(long)]
    config: PathBuf,
    /// Where to write the backup file.
    #[arg(long)]
    output: PathBuf,
}

#[derive(ClapArgs)]
struct VerifyBackupArgs {
    /// The backup file to verify.
    #[arg(long)]
    input: PathBuf,
    /// An optional server configuration; when given, the verify result is
    /// recorded in that identity store's backup log (决策 5).
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum UserCommand {
    /// Suspend a user; they lose access on their next request.
    Suspend(UserTargetArgs),
    /// Reactivate a suspended user.
    Reactivate(UserTargetArgs),
}

#[derive(ClapArgs)]
struct UserTargetArgs {
    /// Path to the server configuration file (locates the identity database).
    #[arg(long)]
    config: PathBuf,
    /// The target user's email.
    #[arg(long)]
    email: String,
}

#[derive(Subcommand)]
enum TokenCommand {
    /// Revoke a PAT by its id; the next use fails at once.
    Revoke(TokenRevokeArgs),
}

#[derive(ClapArgs)]
struct TokenRevokeArgs {
    #[arg(long)]
    config: PathBuf,
    /// The PAT's id (from the credential list, not its plaintext).
    #[arg(long = "token-id")]
    token_id: String,
}

#[derive(Subcommand)]
enum ProjectCommand {
    /// Register a project.
    Create(ProjectCreateArgs),
}

#[derive(ClapArgs)]
struct ProjectCreateArgs {
    #[arg(long)]
    config: PathBuf,
    /// The project's URL key (stable identifier).
    #[arg(long)]
    key: String,
    /// The display name (defaults to the key).
    #[arg(long)]
    name: Option<String>,
}

#[derive(Subcommand)]
enum RepoCommand {
    /// Register a repo within a project.
    Create(RepoCreateArgs),
}

#[derive(ClapArgs)]
struct RepoCreateArgs {
    #[arg(long)]
    config: PathBuf,
    /// The owning project's key.
    #[arg(long)]
    project: String,
    /// The repo's key (unique within the project).
    #[arg(long)]
    key: String,
    /// The display name (defaults to the key).
    #[arg(long)]
    name: Option<String>,
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
        Some(Command::User(UserCommand::Suspend(args))) => run_set_suspended(args, true),
        Some(Command::User(UserCommand::Reactivate(args))) => run_set_suspended(args, false),
        Some(Command::Token(TokenCommand::Revoke(args))) => run_token_revoke(args),
        Some(Command::Project(ProjectCommand::Create(args))) => run_project_create(args),
        Some(Command::Repo(RepoCommand::Create(args))) => run_repo_create(args),
        Some(Command::Backup(args)) => run_backup(args),
        Some(Command::VerifyBackup(args)) => run_verify_backup(args),
        Some(Command::Restore(args)) => run_restore(args),
        None => run_server(cli.run),
    }
}

/// Open the identity store the config declares, for the headless management
/// subcommands (決策 2: the same single-point actions the admin API and /admin
/// forms call). An in-memory config has no persistent store to manage.
fn open_identity(config_path: &Path) -> Result<IdentitySqlite, String> {
    let config = speclink_server::config::load(config_path).map_err(|e| e.to_string())?;
    match &config.identity {
        IdentityConfig::Sqlite { path } => IdentitySqlite::open(path).map_err(|e| e.to_string()),
        IdentityConfig::Memory => Err(
            "this subcommand needs a persistent identity store; the config declares an in-memory one"
                .to_string(),
        ),
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
    // The invite subcommand shares the single-point path with the /admin form
    // (決策 2); the CLI records the host as operator and source cli.
    match store.admin_create_invitation(&AuditActor::system_cli(), invitation) {
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

/// Suspend or reactivate the user named by `--email`. Resolves the email to a
/// user id, then runs the single-point action under a CLI-sourced actor (operator
/// `system`). A refused (last active admin) or unknown email exits non-zero.
fn run_set_suspended(args: UserTargetArgs, suspended: bool) -> ExitCode {
    let store = match open_identity(&args.config) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let user = match store.find_user_by_email(&args.email) {
        Ok(Some(user)) => user,
        Ok(None) => {
            eprintln!("no user with email '{}'", args.email);
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match store.admin_set_user_suspended(&AuditActor::system_cli(), &user.id, suspended) {
        Ok(()) => {
            println!("{} {}", if suspended { "suspended" } else { "reactivated" }, args.email);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Force-revoke a PAT by its id under a CLI-sourced actor.
fn run_token_revoke(args: TokenRevokeArgs) -> ExitCode {
    let store = match open_identity(&args.config) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match store.admin_revoke_pat(&AuditActor::system_cli(), &args.token_id) {
        Ok(()) => {
            println!("revoked {}", args.token_id);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Register a project under a CLI-sourced actor.
fn run_project_create(args: ProjectCreateArgs) -> ExitCode {
    let store = match open_identity(&args.config) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let name = args.name.as_deref().map(str::trim).filter(|n| !n.is_empty()).unwrap_or(&args.key);
    match store.admin_create_project(&AuditActor::system_cli(), &args.key, name) {
        Ok(()) => {
            println!("created project {}", args.key);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Produce a backup of the configured store and identity into `--output`
/// (决策 1/2). Offline: the operator guarantees no concurrent writes. A memory
/// identity has no persistent database to snapshot and is refused.
fn run_backup(args: BackupArgs) -> ExitCode {
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
    let identity = match &config.identity {
        IdentityConfig::Sqlite { path } => match IdentitySqlite::open(path) {
            Ok(store) => store,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },
        IdentityConfig::Memory => {
            eprintln!("backup needs a persistent identity store; the config declares an in-memory one");
            return ExitCode::FAILURE;
        }
    };
    match speclink_server::backup::create(store.as_ref(), &identity, &args.output) {
        Ok(summary) => {
            let detail = format!("{} 個 scope、{} 個成員", summary.scope_count, summary.member_count);
            // Record the run in the identity store's backup log for the admin
            // backup-info view (决策 5).
            let _ = identity.record_backup(
                &AuditActor::system_cli(),
                NewBackupRecord {
                    kind: "backup".to_string(),
                    created_at: summary.created_at,
                    format_version: summary.backup_format_version,
                    scope_count: summary.scope_count,
                    ok: true,
                    detail: detail.clone(),
                },
            );
            println!("備份完成：{detail} → {}", args.output.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Verify a backup file's integrity (决策 4). A digest mismatch, unparseable
/// structure or unknown format version prints the reason and exits non-zero. When
/// `--config` is given, the outcome is recorded in that identity store's backup
/// log (决策 5).
fn run_verify_backup(args: VerifyBackupArgs) -> ExitCode {
    let result = speclink_server::backup::verify(&args.input);
    if let Some(config_path) = &args.config {
        record_verify_result(config_path, &result);
    }
    match &result {
        Ok(report) => {
            println!(
                "備份完整：格式版本 {}、{} 個成員、{} 個 scope",
                report.backup_format_version, report.member_count, report.scope_count
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Record a verify outcome into the config's identity store (best-effort — a
/// config or store problem never changes the verify's own exit code).
fn record_verify_result(
    config_path: &Path,
    result: &Result<speclink_server::backup::VerifyReport, speclink_server::backup::BackupError>,
) {
    let Ok(config) = speclink_server::config::load(config_path) else { return };
    let IdentityConfig::Sqlite { path } = &config.identity else { return };
    let Ok(identity) = IdentitySqlite::open(path) else { return };
    let record = match result {
        Ok(report) => NewBackupRecord {
            kind: "verify".to_string(),
            created_at: Utc::now(),
            format_version: report.backup_format_version,
            scope_count: report.scope_count,
            ok: true,
            detail: format!("{} 個成員、{} 個 scope 驗證通過", report.member_count, report.scope_count),
        },
        Err(e) => NewBackupRecord {
            kind: "verify".to_string(),
            created_at: Utc::now(),
            format_version: 0,
            scope_count: 0,
            ok: false,
            detail: format!("驗證失敗：{e}"),
        },
    };
    let _ = identity.record_backup(&AuditActor::system_cli(), record);
}

/// Restore a backup into the empty target the config declares, then validate
/// (决策 3). A non-empty target, an integrity failure, or a validation mismatch
/// prints the reason and exits non-zero; a mismatch marks the target unusable.
fn run_restore(args: RestoreArgs) -> ExitCode {
    let config = match speclink_server::config::load(&args.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match speclink_server::backup::restore(&config, &args.input) {
        Ok(report) if report.ok => {
            println!("還原完成且驗證通過：{} 個 scope 全數比對一致", report.scopes_checked);
            ExitCode::SUCCESS
        }
        Ok(report) => {
            eprintln!("還原後驗證發現不符，此目標不可投產：");
            for diff in &report.differences {
                eprintln!("  - {diff}");
            }
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

/// Register a repo within a project under a CLI-sourced actor.
fn run_repo_create(args: RepoCreateArgs) -> ExitCode {
    let store = match open_identity(&args.config) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let name = args.name.as_deref().map(str::trim).filter(|n| !n.is_empty()).unwrap_or(&args.key);
    match store.admin_create_repo(&AuditActor::system_cli(), &args.project, &args.key, name) {
        Ok(()) => {
            println!("created repo {}/{}", args.project, args.key);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

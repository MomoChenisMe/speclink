//! Connection management: link, unlink, auth.
//!
//! ModeFree by declaration — these verbs change the store mode rather than
//! consume it, so they resolve the connection themselves instead of going
//! through dispatch's mode shapes.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;
use speclink_protocol::query as protocol_query;
use speclink_remote::auth as remote_auth;
use speclink_remote::client::Client as RemoteClient;
use speclink_remote::credentials as remote_credentials;
use speclink_remote::credentials::CredentialStore as _;
use speclink_remote::login as remote_login;
use std::io::IsTerminal;
use std::process::Stdio;

use crate::color;
use crate::common::{print_json, read_stdin};
use crate::remote_base::{ensure_repo_registered, git_reference_warning};

#[derive(Args)]
pub(crate) struct LinkArgs {
    /// Project-scoped connection URL
    url: String,
    /// This repo's registered name in the remote project
    #[arg(long)]
    repo: Option<String>,
}
#[derive(Args)]
pub(crate) struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommands,
}
#[derive(Subcommand)]
enum AuthCommands {
    /// Log in to the connected server (device authorization by default)
    Login {
        /// Read a personal access token from stdin (CI/scripted use)
        #[arg(long = "token-stdin", conflicts_with = "pat")]
        token_stdin: bool,
        /// Paste a personal access token instead of authorizing this device
        #[arg(long = "pat")]
        pat: bool,
    },
    /// Show the current identity and repo validation result
    Status {
        /// Emit the identity and credential source as JSON
        #[arg(long)]
        json: bool,
    },
    /// Log out: revoke this device's credential family and clear local credentials
    Logout,
}
pub(crate) fn cmd_link(a: LinkArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let origin = remote_auth::origin_of(&a.url);
    // The full ladder, not just the file: a desktop login already covers this
    // origin, and reporting "no credentials yet" at it would be a lie.
    match remote_auth::resolve_credential(&origin).ok().flatten() {
        Some(resolved) => {
            let token = resolved.token;
            let whoami = RemoteClient::new(&a.url, &token, None).whoami()?;
            if let Some(repo) = a.repo.as_deref() {
                ensure_repo_registered(&whoami, repo)?;
                println!("{} Repo '{repo}' is registered in this project", color::green("✓"));
            }
            core::init::write_remote_section(&root, &a.url, a.repo.as_deref())?;
            println!("{} Linked to {}", color::green("✓"), a.url);
            git_reference_warning(&root, a.repo.as_deref(), &whoami);
            Ok(())
        }
        None => {
            core::init::write_remote_section(&root, &a.url, a.repo.as_deref())?;
            println!("{} Linked to {}", color::green("✓"), a.url);
            println!("  No credentials yet — run `speclink auth login` to connect");
            Ok(())
        }
    }
}
pub(crate) fn cmd_unlink() -> Result<()> {
    let root = core::workspace::Workspace::discover_cwd()?
        .map(|ws| ws.root)
        .unwrap_or(std::env::current_dir()?);
    if !core::init::remove_remote_section(&root)? {
        bail!("No remote connection to remove (no `remote:` section in .speclink.yaml)");
    }
    println!(
        "{} Removed the remote section from .speclink.yaml — back to the local fs store",
        color::green("✓")
    );
    Ok(())
}
pub(crate) fn cmd_auth(a: AuthArgs) -> Result<()> {
    // Both auth verbs need the connection (its origin keys the credentials).
    let Some(ws) = core::workspace::Workspace::discover_cwd()? else {
        bail!("Not connected to a remote store — run `speclink link <url>` first");
    };
    let conn = match speclink_host::context::resolve_store_mode(&ws)?.mode {
        core::workspace::StoreMode::Remote(conn) => conn,
        core::workspace::StoreMode::Fs => {
            bail!("Not connected to a remote store — run `speclink link <url>` first")
        }
    };
    let origin = remote_auth::origin_of(&conn.url);
    match a.command {
        AuthCommands::Login { token_stdin, pat } => {
            if token_stdin || pat {
                return login_with_pat(&conn, &origin, token_stdin);
            }
            // Device authorization needs a terminal to show the code in and a
            // keyring to put the credential in. Missing either is a hard stop
            // with the flag that works instead — never a silent downgrade.
            if !std::io::stdin().is_terminal() {
                bail!(
                    "`speclink auth login` needs a terminal to authorize this device — use `--token-stdin` to pipe a personal access token instead"
                );
            }
            login_with_device(&conn, &origin)
        }
        AuthCommands::Status { json } => {
            let resolved = remote_auth::resolve_credential(&origin)
                .map_err(|e| anyhow::anyhow!(e.message()))?;
            let Some(resolved) = resolved else {
                bail!("Not logged in to {origin} — run `speclink auth login`");
            };
            let whoami = RemoteClient::new(&conn.url, &resolved.token, None).whoami()?;
            if json {
                return print_json(&serde_json::json!({
                    "user": {
                        "name": whoami.user.name,
                        "handle": whoami.user.handle,
                    },
                    "credentialSource": resolved.source.as_str(),
                }));
            }
            print_identity(&whoami);
            println!("  Credential from: {}", resolved.source.describe());
            if let Some(repo) = conn.repo.as_deref() {
                match ensure_repo_registered(&whoami, repo) {
                    Ok(()) => println!("Repo '{repo}' is registered in this project"),
                    Err(e) => println!("! {e}"),
                }
            }
            git_reference_warning(&ws.root, conn.repo.as_deref(), &whoami);
            Ok(())
        }
        AuthCommands::Logout => cmd_auth_logout(&origin),
    }
}
/// The PAT tracks, unchanged from before device authorization existed: paste
/// interactively, or pipe from stdin for CI. Both land in the credentials
/// file, and both validate before storing so a rejected token is never
/// written.
fn login_with_pat(
    conn: &core::workspace::RemoteConnection,
    origin: &str,
    token_stdin: bool,
) -> Result<()> {
    if !token_stdin {
        eprint!("Paste your personal access token: ");
    }
    let token = read_stdin().trim().to_string();
    if token.is_empty() {
        bail!("empty token — nothing stored");
    }
    let whoami = RemoteClient::new(&conn.url, &token, None).whoami()?;
    remote_auth::save_token_at(&remote_auth::speclink_config_dir(), origin, &token)?;
    print_identity(&whoami);
    Ok(())
}
/// Terminal, browser and clock for a device login.
struct CliDeviceIo;
impl remote_login::DeviceLoginIo for CliDeviceIo {
    fn announce(&self, verification_uri: &str, user_code: &str) {
        println!("Open {verification_uri} and enter code: {user_code}");
        println!("Waiting for approval…");
    }

    fn open_browser(&self, url: &str) -> bool {
        #[cfg(target_os = "macos")]
        let mut cmd = std::process::Command::new("open");
        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", "start", ""]);
            c
        };
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        let mut cmd = std::process::Command::new("xdg-open");

        cmd.arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn sleep_secs(&self, secs: u64) {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
}
fn login_with_device(conn: &core::workspace::RemoteConnection, origin: &str) -> Result<()> {
    let store = remote_credentials::KeyringCredentialStore;
    // Probe before starting: approving in a browser only to discover there is
    // nowhere to keep the credential wastes the user's time.
    if store.get(origin, remote_credentials::CredentialKind::Refresh).is_err() {
        bail!(
            "no system keychain available on this machine — a device login's refresh credential is never written to a plain file; use `speclink auth login --pat` or set SPECLINK_TOKEN"
        );
    }

    let outcome = remote_login::device_login(
        origin,
        &store,
        &remote_auth::speclink_config_dir(),
        &CliDeviceIo,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    match outcome {
        remote_login::DeviceLoginOutcome::Approved { display } => {
            println!("{} Logged in as {display}", color::green("✓"));
            if let Some(repo) = conn.repo.as_deref() {
                println!("Connected to repo '{repo}'");
            }
            Ok(())
        }
        remote_login::DeviceLoginOutcome::Denied => {
            bail!("authorization denied — nothing was stored")
        }
        remote_login::DeviceLoginOutcome::Expired => {
            bail!("authorization expired before it was approved — run `speclink auth login` again")
        }
        remote_login::DeviceLoginOutcome::Unsupported => {
            bail!("this server does not offer device authorization — use `speclink auth login --pat`")
        }
    }
}
/// Logout revokes the credential family on the server, then clears every local
/// credential for the origin. The family is shared with the desktop app, so
/// this logs the whole machine out — the symmetric cost of one login covering
/// both. The server-side PAT is left alone: it is a separate credential the
/// user manages, possibly in use from another machine.
fn cmd_auth_logout(origin: &str) -> Result<()> {
    let store = remote_credentials::KeyringCredentialStore;
    let dir = remote_auth::speclink_config_dir();
    let refresh = store
        .get(origin, remote_credentials::CredentialKind::Refresh)
        .ok()
        .flatten();
    let keyring_pat = store
        .get(origin, remote_credentials::CredentialKind::Pat)
        .ok()
        .flatten();
    let had_file_token = remote_auth::load_token_at(&dir, origin).is_some();

    if refresh.is_none() && keyring_pat.is_none() && !had_file_token {
        bail!("Not logged in to {origin}");
    }

    // A revoke that cannot reach the server still leaves nothing usable here;
    // the family outliving us is worth a warning, not a failed logout.
    if let Some(refresh) = refresh.as_deref() {
        if let Err(e) = speclink_remote::device::revoke(origin, refresh) {
            eprintln!(
                "speclink: warning: could not revoke the credential family on the server ({e}) — it stays live until it expires"
            );
        }
    }
    remote_login::clear_all_local_credentials(origin, &store);
    remote_auth::remove_token_at(&dir, origin)?;
    println!("{} Logged out of {origin}", color::green("✓"));
    Ok(())
}
fn print_identity(whoami: &protocol_query::WhoamiResponse) {
    if whoami.user.handle.is_empty() {
        println!("{} Logged in as {}", color::green("✓"), whoami.user.name);
    } else {
        println!(
            "{} Logged in as {} (@{})",
            color::green("✓"),
            whoami.user.name,
            whoami.user.handle
        );
    }
}

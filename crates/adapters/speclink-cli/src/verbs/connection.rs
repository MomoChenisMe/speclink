//! Connection management: link, unlink, auth.
//!
//! ModeFree by declaration — these verbs change the store mode rather than
//! consume it, so they resolve the connection themselves instead of going
//! through dispatch's mode shapes.

use anyhow::{anyhow, bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;
use speclink_protocol::query as protocol_query;
use speclink_remote::auth as remote_auth;
use speclink_remote::client::Client as RemoteClient;
use speclink_remote::credentials as remote_credentials;
use speclink_remote::credentials::CredentialStore;
use speclink_remote::login::{self as remote_login, LoginOutcome, LoginPhase, Method, Plan};
use std::cell::RefCell;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
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
    /// Log in to the connected server (the first method it supports of: browser, device, token)
    Login {
        /// Sign in through the server's login service in a browser
        #[arg(long, conflicts_with_all = ["device", "pat", "token_stdin"])]
        browser: bool,
        /// Authorize this device with a code
        #[arg(long, conflicts_with_all = ["pat", "token_stdin"])]
        device: bool,
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
            core::config::write_remote_section(&root, &a.url, a.repo.as_deref())?;
            println!("{} Linked to {}", color::green("✓"), a.url);
            git_reference_warning(&root, a.repo.as_deref(), &whoami);
            Ok(())
        }
        None => {
            core::config::write_remote_section(&root, &a.url, a.repo.as_deref())?;
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
    if !core::config::remove_remote_section(&root)? {
        bail!("No remote connection to remove (no `remote:` section in .speclink.yaml)");
    }
    println!(
        "{} Removed the remote section from .speclink.yaml — back to the local fs store",
        color::green("✓")
    );
    Ok(())
}
pub(crate) fn cmd_auth(a: AuthArgs) -> Result<()> {
    // Every auth verb needs the connection (its origin keys the credentials).
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
    let store = remote_credentials::KeyringCredentialStore;
    let mut stdout = std::io::stdout();
    let sleep_secs = |secs: u64| std::thread::sleep(std::time::Duration::from_secs(secs));
    let env = AuthEnv {
        store: &store,
        config_dir: remote_auth::speclink_config_dir(),
        env_token: std::env::var("SPECLINK_TOKEN").ok(),
        interactive: std::io::stdin().is_terminal(),
        read_token: &read_stdin,
        open_browser: &open_system_browser,
        sleep_secs: &sleep_secs,
        out: RefCell::new(&mut stdout),
    };
    match a.command {
        AuthCommands::Login { browser, device, token_stdin, pat } => {
            let request = if token_stdin || pat {
                LoginRequest::Pat { prompt: pat }
            } else if browser {
                LoginRequest::Browser
            } else if device {
                LoginRequest::Device
            } else {
                LoginRequest::Auto
            };
            auth_login(&conn, &origin, request, &env)
        }
        AuthCommands::Status { json } => auth_status(&ws.root, &conn, &origin, json, &env),
        AuthCommands::Logout => auth_logout(&origin, &env),
    }
}

/// What the auth verbs reach outside the process: the keyring, the user-level
/// config directory, `SPECLINK_TOKEN`, the terminal and the browser.
/// `cmd_auth` builds the real one; the tests build one around an in-memory
/// keyring and a captured stdout.
struct AuthEnv<'a> {
    store: &'a dyn CredentialStore,
    config_dir: PathBuf,
    env_token: Option<String>,
    interactive: bool,
    read_token: &'a dyn Fn() -> String,
    open_browser: &'a dyn Fn(&str) -> bool,
    sleep_secs: &'a dyn Fn(u64),
    out: RefCell<&'a mut dyn Write>,
}

impl remote_login::LoginIo for AuthEnv<'_> {
    fn phase(&self, phase: LoginPhase) {
        let mut out = self.out.borrow_mut();
        match phase {
            LoginPhase::AwaitingDevice { verification_uri, user_code, .. } => {
                let _ = writeln!(out, "Open {verification_uri} and enter code: {user_code}");
                let _ = writeln!(out, "Waiting for approval…");
            }
            LoginPhase::AwaitingBrowser { url, .. } => {
                let _ = writeln!(out, "Open this URL to sign in:");
                let _ = writeln!(out, "  {url}");
                let _ = writeln!(out, "Waiting for the sign-in…");
            }
        }
    }

    fn open_browser(&self, url: &str) -> bool {
        (self.open_browser)(url)
    }

    fn sleep_secs(&self, secs: u64) {
        (self.sleep_secs)(secs)
    }

    /// Ctrl-C ends the process; there is nothing else to cancel with.
    fn cancelled(&self) -> bool {
        false
    }
}

/// Which login `auth login`'s flags asked for.
enum LoginRequest {
    /// No flag: the first method the server supports.
    Auto,
    Browser,
    Device,
    /// `--pat` (prompt for it) or `--token-stdin` (read it from stdin).
    Pat { prompt: bool },
}

fn auth_login(
    conn: &core::workspace::RemoteConnection,
    origin: &str,
    request: LoginRequest,
    env: &AuthEnv,
) -> Result<()> {
    let preference = match request {
        LoginRequest::Pat { prompt } => return login_with_pat(origin, prompt, env),
        // A flagless login needs a terminal to show a code or a URL in. In a
        // pipeline the form that works is the token one — never a silent
        // downgrade.
        LoginRequest::Auto if !env.interactive => bail!(
            "`speclink auth login` needs a terminal to authorize this device — use `--token-stdin` to pipe a personal access token instead"
        ),
        LoginRequest::Auto => None,
        LoginRequest::Browser => Some(Method::Browser),
        LoginRequest::Device => Some(Method::Device),
    };
    let discovery = remote_login::discover(origin).map_err(|e| anyhow!(e.message))?;
    let plan = remote_login::choose(&discovery, preference)?;
    let browser = matches!(plan, Plan::Browser(_));
    let kind = if browser { "browser" } else { "device" };

    match remote_login::login(origin, &plan, env.store, &env.config_dir, env) {
        LoginOutcome::LoggedIn { display, .. } => {
            let mut out = env.out.borrow_mut();
            let _ = writeln!(out, "{} Logged in as {display}", color::green("✓"));
            if let Some(repo) = conn.repo.as_deref() {
                let _ = writeln!(out, "Connected to repo '{repo}'");
            }
            Ok(())
        }
        LoginOutcome::LoggedInWithoutIdentity { message } => {
            let _ = writeln!(env.out.borrow_mut(), "{} Logged in", color::green("✓"));
            eprintln!("speclink: warning: could not read the identity of this login ({message})");
            Ok(())
        }
        LoginOutcome::Denied if browser => {
            bail!("sign-in was denied at the login service — nothing was stored")
        }
        LoginOutcome::Denied => bail!("authorization denied — nothing was stored"),
        LoginOutcome::Expired if browser => bail!(
            "the sign-in timed out — nobody came back from the login service within {} minutes; run `speclink auth login` again",
            remote_login::BROWSER_LOGIN_WINDOW_SECS / 60
        ),
        LoginOutcome::Expired => {
            bail!("authorization expired before it was approved — run `speclink auth login` again")
        }
        LoginOutcome::Cancelled => bail!("login cancelled — nothing was stored"),
        LoginOutcome::Unsupported => {
            bail!("this server does not offer device authorization — use `speclink auth login --pat`")
        }
        LoginOutcome::TokenRequired => bail!(
            "this server only accepts personal access tokens — use `speclink auth login --pat`, or `--token-stdin` to pipe one in"
        ),
        LoginOutcome::KeychainUnavailable => bail!(
            "no system keychain available on this machine — a {kind} login's refresh credential is never written to a plain file; use `speclink auth login --pat` or set SPECLINK_TOKEN"
        ),
        LoginOutcome::Failed(message) => bail!("{message}"),
    }
}

/// The PAT tracks: paste interactively, or pipe from stdin for CI. The token
/// is checked before it is stored, so a rejected one is never written; it goes
/// to the keyring, or to the credentials file when there is no keyring.
fn login_with_pat(origin: &str, prompt: bool, env: &AuthEnv) -> Result<()> {
    if prompt {
        eprint!("Paste your personal access token: ");
    }
    let token = (env.read_token)().trim().to_string();
    if token.is_empty() {
        bail!("empty token — nothing stored");
    }
    match remote_login::pat_login(origin, &token, env.store, &env.config_dir, Some(&env.config_dir)) {
        LoginOutcome::LoggedIn { display, handle } => {
            print_identity(env, &display, &handle);
            Ok(())
        }
        LoginOutcome::Denied => bail!("authentication failed — run `speclink auth login`"),
        LoginOutcome::Failed(message) => bail!("{message}"),
        other => bail!("unexpected login outcome: {other:?}"),
    }
}

fn auth_status(
    root: &Path,
    conn: &core::workspace::RemoteConnection,
    origin: &str,
    json: bool,
    env: &AuthEnv,
) -> Result<()> {
    let resolved =
        remote_auth::resolve_credential_at(&env.config_dir, origin, env.store, env.env_token.clone())
            .map_err(|e| anyhow!(e.message()))?;
    let Some(resolved) = resolved else {
        bail!("Not logged in to {origin} — run `speclink auth login`");
    };
    let whoami = RemoteClient::new(&conn.url, &resolved.token, None).whoami()?;
    if json {
        return print_json(&status_json(&whoami, resolved.source));
    }
    print_identity(env, &whoami.user.name, &whoami.user.handle);
    println!("  Credential from: {}", resolved.source.describe());
    if let Some(repo) = conn.repo.as_deref() {
        match ensure_repo_registered(&whoami, repo) {
            Ok(()) => println!("Repo '{repo}' is registered in this project"),
            Err(e) => println!("! {e}"),
        }
    }
    git_reference_warning(root, conn.repo.as_deref(), &whoami);
    Ok(())
}

/// `auth status --json`: the identity and the ladder layer it came from.
fn status_json(
    whoami: &protocol_query::WhoamiResponse,
    source: remote_auth::CredentialSource,
) -> impl serde::Serialize {
    serde_json::json!({
        "user": {
            "name": whoami.user.name,
            "handle": whoami.user.handle,
        },
        "credentialSource": source.as_str(),
    })
}

/// Logout revokes what can be revoked, then clears every local credential for
/// the origin. A device family is shared with the desktop app, so this logs the
/// whole machine out — the symmetric cost of one login covering both. The
/// server-side PAT is left alone: it is a separate credential the user manages,
/// possibly in use from another machine.
fn auth_logout(origin: &str, env: &AuthEnv) -> Result<()> {
    let outcome = remote_login::logout(origin, env.store, Some(&env.config_dir)).map_err(|e| anyhow!(e))?;
    if !outcome.was_logged_in {
        bail!("Not logged in to {origin}");
    }
    // A revoke that cannot reach anyone still leaves nothing usable here; the
    // credential outliving us is worth a warning, not a failed logout.
    if let Some(warning) = outcome.revoke_warning {
        eprintln!("speclink: warning: {warning}");
    }
    let _ = writeln!(env.out.borrow_mut(), "{} Logged out of {origin}", color::green("✓"));
    Ok(())
}

/// Best-effort launch of the system browser.
fn open_system_browser(url: &str) -> bool {
    browser_command(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The platform's URL opener, with the URL as its last argument. No shell sees
/// the URL: on Windows `cmd /C start` would split an authorization URL at its
/// first `&` and run the rest as commands, so the URL goes to the shell's URL
/// handler directly.
fn browser_command(url: &str) -> std::process::Command {
    #[cfg(target_os = "macos")]
    let mut cmd = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("rundll32");
        c.arg("url.dll,FileProtocolHandler");
        c
    };
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(url);
    cmd
}

fn print_identity(env: &AuthEnv, name: &str, handle: &str) {
    let mut out = env.out.borrow_mut();
    if handle.is_empty() {
        let _ = writeln!(out, "{} Logged in as {name}", color::green("✓"));
    } else {
        let _ = writeln!(out, "{} Logged in as {name} (@{handle})", color::green("✓"));
    }
}

#[cfg(test)]
mod tests {
    //! `auth login` and `auth logout` at the function level
    //! (client-browser-login D8): a fake server, an in-memory keyring, captured
    //! stdout. The binary's own tests (`tests/it/auth_login_methods.rs`) cover
    //! only what needs no keyring.

    use super::*;
    use speclink_remote::credentials::{CredentialKind, MemoryCredentialStore};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    const DISPLAY: &str = "Dev <dev@example.com>";

    /// A server answering the routes it is given and 404 everywhere else.
    struct Fake {
        server: Arc<tiny_http::Server>,
        origin: String,
        routes: Arc<Mutex<HashMap<String, (u16, String)>>>,
        paths: Arc<Mutex<Vec<String>>>,
    }

    impl Drop for Fake {
        fn drop(&mut self) {
            self.server.unblock();
        }
    }

    impl Fake {
        fn new() -> Fake {
            let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
            let origin = format!("http://{}", server.server_addr());
            let routes: Arc<Mutex<HashMap<String, (u16, String)>>> = Arc::default();
            let paths: Arc<Mutex<Vec<String>>> = Arc::default();
            let (looper, table, log) = (server.clone(), routes.clone(), paths.clone());
            std::thread::spawn(move || {
                for req in looper.incoming_requests() {
                    let path = req.url().split('?').next().unwrap_or_default().to_string();
                    let (status, body) = table.lock().unwrap().get(&path).cloned().unwrap_or((
                        404,
                        r#"{"status":404,"reason":"not_found","message":"no route"}"#.to_string(),
                    ));
                    log.lock().unwrap().push(path);
                    let resp = tiny_http::Response::from_string(body).with_status_code(status).with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    );
                    let _ = req.respond(resp);
                }
            });
            Fake { server, origin, routes, paths }
        }

        fn route(&self, path: &str, status: u16, body: &str) -> &Fake {
            self.routes.lock().unwrap().insert(path.to_string(), (status, body.to_string()));
            self
        }

        /// Today's official server: a device flow, `/auth/whoami`, `/auth/revoke`.
        fn device_flow(&self, verdict: &str) -> &Fake {
            self.route(
                "/auth/device",
                200,
                r#"{"deviceCode":"dc_1","userCode":"WDJB-MJHT","verificationUri":"http://approve.example/activate","expiresIn":900,"interval":1}"#,
            )
            .route(
                "/auth/device/token",
                200,
                &format!(r#"{{"status":"{verdict}","accessToken":"access-1","refreshToken":"refresh-1","expiresIn":3600}}"#),
            )
            .route("/auth/whoami", 200, &format!(r#"{{"user":{{"name":"{DISPLAY}","handle":"dev"}}}}"#))
            .route("/auth/revoke", 200, "{}")
        }

        fn paths(&self) -> Vec<String> {
            self.paths.lock().unwrap().clone()
        }

        fn conn(&self, repo: Option<&str>) -> core::workspace::RemoteConnection {
            core::workspace::RemoteConnection {
                url: format!("{}/api/speclink/v1/projects/demo", self.origin),
                repo: repo.map(str::to_string),
            }
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("speclink-cli-auth-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Run `f` against an env with an in-memory keyring and a captured stdout.
    /// Nobody signs in at a browser, and time passes instantly.
    fn with_env<T>(
        store: &MemoryCredentialStore,
        dir: &Path,
        interactive: bool,
        token: &str,
        f: impl FnOnce(&AuthEnv) -> T,
    ) -> (T, String) {
        let mut out = Vec::new();
        let read_token = || token.to_string();
        let open_browser = |_: &str| false;
        let sleep_secs = |_: u64| {};
        let result = {
            let env = AuthEnv {
                store,
                config_dir: dir.to_path_buf(),
                env_token: None,
                interactive,
                read_token: &read_token,
                open_browser: &open_browser,
                sleep_secs: &sleep_secs,
                out: RefCell::new(&mut out),
            };
            f(&env)
        };
        (result, String::from_utf8(out).unwrap())
    }

    /// The device login against a server that declares nothing prints what it
    /// printed before discovery existed, byte for byte. The expected text is
    /// the pre-change CLI's output.
    #[test]
    fn an_undeclared_server_keeps_the_device_login_output_byte_for_byte() {
        let fake = Fake::new();
        fake.device_flow("approved");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("undeclared-ok"));

        let (result, out) = with_env(&store, &dir, true, "", |env| {
            auth_login(&fake.conn(Some("backend")), &fake.origin, LoginRequest::Auto, env)
        });

        result.expect("login");
        assert_eq!(
            out,
            "Open http://approve.example/activate and enter code: WDJB-MJHT\nWaiting for approval…\n✓ Logged in as Dev <dev@example.com>\nConnected to repo 'backend'\n"
        );
        assert!(store.get(&fake.origin, CredentialKind::Refresh).unwrap().is_some());
    }

    #[test]
    fn an_undeclared_server_keeps_the_device_denial_message() {
        let fake = Fake::new();
        fake.device_flow("denied");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("undeclared-denied"));

        let (result, _) = with_env(&store, &dir, true, "", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Auto, env)
        });

        assert_eq!(result.unwrap_err().to_string(), "authorization denied — nothing was stored");
    }

    fn declare_browser_and_device(fake: &Fake) {
        let issuer = format!("{}/idp", fake.origin);
        fake.route(
            "/api/speclink/v1/auth-methods",
            200,
            &format!(r#"{{"methods":["device","browser"],"browser":{{"issuer":"{issuer}","clientId":"speclink","scopes":["openid"]}}}}"#),
        )
        .route(
            "/.well-known/oauth-authorization-server/idp",
            200,
            &format!(r#"{{"issuer":"{issuer}","authorization_endpoint":"{issuer}/authorize","token_endpoint":"{issuer}/token"}}"#),
        )
        .device_flow("approved");
    }

    /// No flag on a server with both: the browser login, with its URL on stdout
    /// and no user code. Nobody signs in here, so it ends in a timeout that
    /// reads differently from a denial.
    #[test]
    fn browser_and_device_declared_go_to_the_browser_and_print_its_url() {
        let fake = Fake::new();
        declare_browser_and_device(&fake);
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("browser-auto"));

        let (result, out) = with_env(&store, &dir, true, "", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Auto, env)
        });

        assert!(out.contains("Open this URL to sign in:"), "{out}");
        assert!(out.contains(&format!("{}/idp/authorize?", fake.origin)), "{out}");
        assert!(!out.contains("enter code"), "{out}");
        assert!(!fake.paths().iter().any(|p| p.starts_with("/auth/device")), "{:?}", fake.paths());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("timed out"), "{err}");
        assert!(store.get(&fake.origin, CredentialKind::Refresh).unwrap().is_none());
    }

    #[test]
    fn device_flag_on_a_browser_and_device_server_prints_the_user_code() {
        let fake = Fake::new();
        declare_browser_and_device(&fake);
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("device-flag"));

        let (result, out) = with_env(&store, &dir, false, "", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Device, env)
        });

        result.expect("login");
        assert!(out.contains("enter code: WDJB-MJHT"), "{out}");
        assert!(!out.contains("Open this URL to sign in"), "{out}");
    }

    #[test]
    fn a_token_only_server_points_at_the_access_token_flags() {
        let fake = Fake::new();
        fake.route("/api/speclink/v1/auth-methods", 200, r#"{"methods":["token"]}"#).device_flow("approved");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("token-only"));

        let (result, _) = with_env(&store, &dir, true, "", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Auto, env)
        });

        let err = result.unwrap_err().to_string();
        assert!(err.contains("--pat") && err.contains("--token-stdin"), "{err}");
        assert_eq!(fake.paths(), vec!["/api/speclink/v1/auth-methods".to_string()], "no device flow");
    }

    #[test]
    fn a_failing_discovery_stops_the_flagless_login() {
        let fake = Fake::new();
        fake.route("/api/speclink/v1/auth-methods", 500, "boom").device_flow("approved");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("discovery-5xx"));

        let (result, _) = with_env(&store, &dir, true, "", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Auto, env)
        });

        assert!(result.unwrap_err().to_string().contains("server unavailable"));
        assert_eq!(fake.paths(), vec!["/api/speclink/v1/auth-methods".to_string()]);
    }

    /// `--token-stdin` prints what it always printed, and the PAT now lives in
    /// the keyring: `auth status --json` reports it as `keychain_pat`.
    #[test]
    fn token_stdin_stores_a_keyring_pat_reported_as_keychain_pat() {
        let fake = Fake::new();
        fake.device_flow("approved");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("token-stdin"));

        let (result, out) = with_env(&store, &dir, false, "pat-abc\n", |env| {
            auth_login(&fake.conn(None), &fake.origin, LoginRequest::Pat { prompt: false }, env)
        });

        result.expect("login");
        assert_eq!(out, "✓ Logged in as Dev <dev@example.com> (@dev)\n");
        assert_eq!(store.get(&fake.origin, CredentialKind::Pat).unwrap(), Some("pat-abc".to_string()));
        assert!(!speclink_remote::auth::credentials_path_in(&dir).exists(), "憑證檔不變");
        assert!(!fake.paths().iter().any(|p| p.ends_with("/auth-methods")), "a PAT login does not discover");

        let resolved = remote_auth::resolve_credential_at(&dir, &fake.origin, &store, None)
            .unwrap()
            .expect("a credential");
        let whoami: protocol_query::WhoamiResponse = serde_json::from_str(
            r#"{"user":{"name":"Dev","handle":"dev"},"project":"demo","repos":[]}"#,
        )
        .unwrap();
        let payload = serde_json::to_value(status_json(&whoami, resolved.source)).unwrap();
        assert_eq!(payload["credentialSource"], serde_json::json!("keychain_pat"));
        assert!(payload["credentialSource"].is_string());
        assert_eq!(payload["user"]["handle"], "dev");
    }

    /// The browser launch gets the URL as one argument and no shell parses it:
    /// an authorization URL always carries `&`, which `cmd.exe` would treat as
    /// a command separator.
    #[test]
    fn the_browser_launch_passes_the_url_untouched_and_without_a_shell() {
        let url = "https://id.example.com/authorize?a=1&b=2%20&c=%PATH%";
        let cmd = browser_command(url);
        assert_ne!(cmd.get_program(), "cmd", "no shell may parse the URL");
        assert_eq!(cmd.get_args().last().and_then(|a| a.to_str()), Some(url));
    }

    #[test]
    fn logout_clears_all_four_local_credentials() {
        let fake = Fake::new();
        fake.device_flow("approved");
        let (store, dir) = (MemoryCredentialStore::new(), temp_dir("logout"));
        store.set(&fake.origin, CredentialKind::Refresh, "spk_rt_live").unwrap();
        store.set(&fake.origin, CredentialKind::Bearer, "{}").unwrap();
        store.set(&fake.origin, CredentialKind::Pat, "keyring-pat").unwrap();
        remote_auth::save_token_at(&dir, &fake.origin, "file-pat").unwrap();

        let (result, out) = with_env(&store, &dir, false, "", |env| auth_logout(&fake.origin, env));

        result.expect("logout");
        assert_eq!(out, format!("✓ Logged out of {}\n", fake.origin));
        for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
            assert_eq!(store.get(&fake.origin, kind).unwrap(), None, "{kind:?}");
        }
        assert_eq!(remote_auth::load_token_at(&dir, &fake.origin), None);
        assert!(fake.paths().contains(&"/auth/revoke".to_string()));
    }
}

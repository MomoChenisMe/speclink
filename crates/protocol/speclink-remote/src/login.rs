//! The one login flow both front ends run (client-browser-login D7): discover
//! what the server supports, choose a method, run it to a verdict, store the
//! credentials.
//!
//! It lives here rather than in a front end so the CLI and the desktop app
//! reach the same outcome and write the same keyring entries — a login on
//! either one is a login for both (cli-desktop-credential-sharing).
//!
//! What differs between the front ends — how a phase is shown, how a browser
//! opens, how time passes, whether the user cancelled — comes in through
//! [`LoginIo`]. Tests use it to make a 300-second wait pass in a second.

pub use crate::auth::normalize_origin;
pub use crate::discovery::{choose, discover, ChooseError, Discovery, Method, Plan};

use crate::auth;
use crate::credentials::{CredentialKind, CredentialStore, OAuthRefresh, RefreshEntry};
use crate::device::{self, InitiateOutcome};
use crate::loopback::{Callback, CallbackListener};
use crate::oauth;
use speclink_protocol::auth::BrowserLogin;
use speclink_protocol::device::DeviceTokenStatus;
use std::path::Path;

/// How much to widen the polling interval when the server asks for room.
const SLOW_DOWN_STEP_SECS: u64 = 5;

/// How long a browser login waits for the login service to send the user back.
pub const BROWSER_LOGIN_WINDOW_SECS: u64 = 300;

/// The terminal, browser, clock and cancel button a login needs.
pub trait LoginIo {
    /// Show what the login is waiting for. Always called, even when a browser
    /// opens: the user may need to copy the URL, or approve from a phone.
    fn phase(&self, phase: LoginPhase);
    /// Best-effort browser launch; `false` simply means the user goes to what
    /// [`LoginIo::phase`] showed.
    fn open_browser(&self, url: &str) -> bool;
    /// The login's clock: every wait goes through here.
    fn sleep_secs(&self, secs: u64);
    /// Checked between waits; `true` ends the login with nothing stored.
    fn cancelled(&self) -> bool;
}

/// What a login is waiting for. Carries what a user needs to act, and never a
/// code verifier, an authorization code or a token.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginPhase {
    /// Waiting for the user to sign in at the login service in a browser.
    AwaitingBrowser { url: String, expires_in: u64 },
    /// Waiting for the user to approve this device on the server's page.
    AwaitingDevice {
        verification_uri: String,
        user_code: String,
        expires_in: u64,
    },
}

/// Where a login acts: the server, the keyring its credentials go to, and the
/// directory that holds the rotation lock.
#[derive(Clone, Copy)]
pub struct LoginTarget<'a> {
    pub origin: &'a str,
    pub credentials: &'a dyn CredentialStore,
    pub lock_dir: &'a Path,
}

/// How a login ended. Denial, expiry and cancellation are verdicts, not
/// errors — each needs its own message.
#[derive(Debug, Clone, PartialEq)]
pub enum LoginOutcome {
    LoggedIn { display: String, handle: String },
    /// The credentials are stored, but the identity lookup failed.
    LoggedInWithoutIdentity { message: String },
    Denied,
    Expired,
    Cancelled,
    /// The server has no device flow — the access-token fallback signal.
    Unsupported,
    /// The server only takes an access token.
    TokenRequired,
    /// There is nowhere safe to keep a refresh credential on this machine.
    KeychainUnavailable,
    Failed(String),
}

/// Run `plan` to a verdict, storing the credentials on success.
pub fn login(target: &LoginTarget, plan: &Plan, io: &dyn LoginIo) -> LoginOutcome {
    let result = match plan {
        Plan::Token => return LoginOutcome::TokenRequired,
        // Probe before starting: signing in only to find there is nowhere to
        // keep the credential wastes the user's time, and a refresh credential
        // never goes to a plain file.
        _ if target.credentials.get(target.origin, CredentialKind::Refresh).is_err() => {
            return LoginOutcome::KeychainUnavailable
        }
        Plan::Browser(browser) => browser_login(target, browser, io),
        Plan::Device => device_login(target, io),
    };
    result.unwrap_or_else(LoginOutcome::Failed)
}

/// Run a device authorization to its verdict, storing the credentials on
/// approval.
///
/// Both credentials are written before the identity lookup: the refresh
/// credential is what survives this process, and the access token beside it is
/// what keeps the next command from spending a rotation.
pub fn device_login(target: &LoginTarget, io: &dyn LoginIo) -> Result<LoginOutcome, String> {
    let origin = target.origin;
    let auth = match device::initiate(origin).map_err(|e| e.to_string())? {
        InitiateOutcome::Supported(auth) => auth,
        InitiateOutcome::Unsupported => return Ok(LoginOutcome::Unsupported),
    };
    if io.cancelled() {
        return Ok(LoginOutcome::Cancelled);
    }

    io.phase(LoginPhase::AwaitingDevice {
        verification_uri: auth.verification_uri.clone(),
        user_code: auth.user_code.clone(),
        expires_in: auth.expires_in,
    });
    let sep = if auth.verification_uri.contains('?') { '&' } else { '?' };
    io.open_browser(&format!(
        "{}{sep}user_code={}",
        auth.verification_uri, auth.user_code
    ));

    let mut interval = auth.interval.max(1);
    let mut waited = 0u64;
    loop {
        if io.cancelled() {
            return Ok(LoginOutcome::Cancelled);
        }
        let resp = device::poll(origin, &auth.device_code).map_err(|e| e.to_string())?;
        match resp.status {
            DeviceTokenStatus::Approved => {
                let access = resp
                    .access_token
                    .ok_or_else(|| "server 回報已核准卻未附 access token".to_string())?;
                let refresh = resp
                    .refresh_token
                    .ok_or_else(|| "server 回報已核准卻未附 refresh credential".to_string())?;
                let entry = RefreshEntry::device(refresh.clone())?;
                // A cancel that lands with the approval still wins: nothing is
                // stored, and the family the server just issued is revoked.
                if io.cancelled() {
                    let _ = device::revoke(origin, &refresh);
                    return Ok(LoginOutcome::Cancelled);
                }
                store_login(target, &entry, &access, resp.expires_in.unwrap_or(0))?;
                return Ok(signed_in(origin, &access));
            }
            DeviceTokenStatus::Denied => return Ok(LoginOutcome::Denied),
            DeviceTokenStatus::Expired => return Ok(LoginOutcome::Expired),
            DeviceTokenStatus::SlowDown => interval += SLOW_DOWN_STEP_SECS,
            DeviceTokenStatus::Pending => {}
        }

        // The server's own expiry is the ceiling; without it a lost approval
        // page would poll until the user gave up.
        waited += interval;
        if waited > auth.expires_in {
            return Ok(LoginOutcome::Expired);
        }
        io.sleep_secs(interval);
    }
}

/// A browser login (D3): metadata, loopback listener, PKCE and state, the
/// browser, the callback, the code exchange, storage, identity. Nothing is
/// stored before the exchange succeeds, and a cancel wins at every step up to
/// the store.
fn browser_login(
    target: &LoginTarget,
    browser: &BrowserLogin,
    io: &dyn LoginIo,
) -> Result<LoginOutcome, String> {
    let endpoints = oauth::discover_endpoints(&browser.issuer)?;
    let port_error = |e: std::io::Error| format!("could not open a local port for the sign-in callback: {e}");
    let listener = CallbackListener::bind().map_err(port_error)?;
    let redirect_uri = listener.redirect_uri().map_err(port_error)?;
    let verifier = oauth::random_token()?;
    let state = oauth::random_token()?;
    let url = oauth::authorization_url(
        &endpoints.authorization,
        &browser.client_id,
        &redirect_uri,
        &oauth::challenge_of(&verifier),
        &state,
        &browser.scopes,
    );
    if io.cancelled() {
        return Ok(LoginOutcome::Cancelled);
    }

    io.phase(LoginPhase::AwaitingBrowser {
        url: url.clone(),
        expires_in: BROWSER_LOGIN_WINDOW_SECS,
    });
    io.open_browser(&url);

    let (code, reply) = match listener.wait(&state, BROWSER_LOGIN_WINDOW_SECS, io) {
        Callback::Code { code, reply } => (code, reply),
        Callback::Denied => return Ok(LoginOutcome::Denied),
        Callback::Expired => return Ok(LoginOutcome::Expired),
        Callback::Cancelled => return Ok(LoginOutcome::Cancelled),
        Callback::Failed(message) => return Err(message),
    };
    // The port closes now: nothing that arrives later can matter.
    drop(listener);
    if io.cancelled() {
        reply.finish(false);
        return Ok(LoginOutcome::Cancelled);
    }

    // `Ok(None)`: the user cancelled while the code was being exchanged.
    let stored = oauth::exchange_code(&endpoints.token, &browser.client_id, &code, &redirect_uri, &verifier)
        .map_err(oauth::GrantFailure::message)
        .and_then(|tokens| {
            if io.cancelled() {
                if let (Some(revocation), Some(refresh)) = (&endpoints.revocation, &tokens.refresh_token) {
                    let _ = oauth::revoke(revocation, &browser.client_id, refresh);
                }
                return Ok(None);
            }
            let entry = RefreshEntry::OAuth(OAuthRefresh {
                refresh_token: tokens.refresh_token,
                token_endpoint: endpoints.token,
                client_id: browser.client_id.clone(),
                revocation_endpoint: endpoints.revocation,
            });
            store_login(
                target,
                &entry,
                &tokens.access_token,
                tokens.expires_in.unwrap_or(oauth::DEFAULT_ACCESS_TOKEN_SECS),
            )?;
            Ok(Some(tokens.access_token))
        });
    reply.finish(matches!(stored, Ok(Some(_))));
    match stored? {
        Some(access) => Ok(signed_in(target.origin, &access)),
        None => Ok(LoginOutcome::Cancelled),
    }
}

/// The verdict of a login whose credentials are stored: an identity lookup
/// that fails does not undo them.
fn signed_in(origin: &str, access: &str) -> LoginOutcome {
    match device::identity(origin, access) {
        Ok(who) => LoginOutcome::LoggedIn { display: who.user.name, handle: who.user.handle },
        Err(e) => LoginOutcome::LoggedInWithoutIdentity { message: e.message },
    }
}

/// Store a login's credentials under the rotation lock: another process
/// mid-rotation would otherwise write back a credential from the family this
/// login just replaced. The refresh entry is the login. A cache that cannot be
/// written costs the next command one rotation, and a PAT entry that cannot be
/// removed loses the ladder to the refresh entry anyway — neither fails it.
fn store_login(
    target: &LoginTarget,
    refresh: &RefreshEntry,
    access: &str,
    expires_in: u64,
) -> Result<(), String> {
    let LoginTarget { origin, credentials, lock_dir } = *target;
    crate::refresh::with_rotation_lock(lock_dir, || {
        credentials.set(origin, CredentialKind::Refresh, &refresh.encode())?;
        let _ = crate::refresh::store_cached_bearer(origin, credentials, access, expires_in);
        let _ = credentials.delete(origin, CredentialKind::Pat);
        Ok(())
    })
}

/// How a personal-access-token login ended.
#[derive(Debug, Clone, PartialEq)]
pub enum PatLoginOutcome {
    LoggedIn { display: String, handle: String },
    /// The server refused the token; nothing was stored.
    Denied,
    /// The keyring cannot take the token and there is no file to fall back to.
    KeychainUnavailable,
    Failed(String),
}

/// Log in with a personal access token: check it with the identity lookup,
/// then store it in the keyring, replacing any other login for the origin.
///
/// When the keyring cannot take it, the token goes to the credentials file in
/// `fallback_dir`; with no `fallback_dir` (the desktop app keeps credentials in
/// the keyring only) that is [`PatLoginOutcome::KeychainUnavailable`].
pub fn pat_login(target: &LoginTarget, token: &str, fallback_dir: Option<&Path>) -> PatLoginOutcome {
    let LoginTarget { origin, credentials, lock_dir } = *target;
    let who = match device::identity(origin, token) {
        Ok(who) => who,
        Err(e) if matches!(e.status, Some(401 | 403)) => return PatLoginOutcome::Denied,
        Err(e) => return PatLoginOutcome::Failed(e.message),
    };
    let logged_in = PatLoginOutcome::LoggedIn { display: who.user.name, handle: who.user.handle };

    // Under the rotation lock: a rotation in flight would otherwise write the
    // refresh entry back after it was cleared, and win the ladder over the PAT.
    let in_keyring = crate::refresh::with_rotation_lock(lock_dir, || {
        Ok(match credentials.set(origin, CredentialKind::Pat, token) {
            Ok(()) => {
                let _ = credentials.delete(origin, CredentialKind::Refresh);
                let _ = credentials.delete(origin, CredentialKind::Bearer);
                true
            }
            Err(_) => false,
        })
    });
    match (in_keyring, fallback_dir) {
        (Err(message), _) => PatLoginOutcome::Failed(message),
        (Ok(true), _) => logged_in,
        (Ok(false), Some(dir)) => match auth::save_token_at(dir, origin, token) {
            Ok(_) => logged_in,
            Err(e) => PatLoginOutcome::Failed(format!("could not write the credentials file: {e}")),
        },
        (Ok(false), None) => PatLoginOutcome::KeychainUnavailable,
    }
}

/// What a logout did.
#[derive(Debug, Default)]
pub struct LogoutOutcome {
    /// Whether the origin held any credential to log out of.
    pub was_logged_in: bool,
    /// A server family or a login-service token was revoked.
    pub revoked: bool,
    /// A revocation was tried and failed; the local cleanup still ran.
    pub revoke_warning: Option<String>,
    /// A keyring PAT was removed. It stays valid on the server until the user
    /// revokes it there.
    pub removed_pat: bool,
}

/// Log out of `origin`: revoke what can be revoked (best effort), then clear
/// every local credential — the keyring's refresh, cache and PAT entries, and
/// the credentials-file entry in `credentials_file_dir` when there is one.
///
/// A keyring that cannot be read is not "logged out". With a credentials file
/// to act on (the CLI on a machine without a keyring) the keyring layer is
/// skipped, as the resolution ladder skips it; without one (the desktop app)
/// the logout fails, because nothing could be checked or cleared. A keyring
/// that reads but will not delete fails the logout for both.
pub fn logout(
    origin: &str,
    credentials: &dyn CredentialStore,
    credentials_file_dir: Option<&Path>,
) -> Result<LogoutOutcome, String> {
    let read = credentials.get(origin, CredentialKind::Refresh).and_then(|refresh| {
        Ok((refresh, credentials.get(origin, CredentialKind::Pat)?.is_some()))
    });
    let (refresh, removed_pat, keyring_reads) = match (read, credentials_file_dir) {
        (Ok((refresh, pat)), _) => (refresh, pat, true),
        (Err(_), Some(_)) => (None, false, false),
        (Err(e), None) => return Err(e),
    };
    let in_file = credentials_file_dir.is_some_and(|dir| auth::load_token_at(dir, origin).is_some());
    let mut outcome = LogoutOutcome {
        was_logged_in: refresh.is_some() || removed_pat || in_file,
        removed_pat,
        ..LogoutOutcome::default()
    };
    // Nothing to log out of: leave the keyring and the file alone.
    if !outcome.was_logged_in {
        return Ok(outcome);
    }

    let revoked = match refresh.as_deref().map(RefreshEntry::parse) {
        Some(RefreshEntry::Device(token)) => Some(device::revoke(origin, &token).map_err(|e| {
            format!("could not revoke the credential family on the server ({e}) — it stays live until it expires")
        })),
        Some(RefreshEntry::OAuth(entry)) => match (&entry.revocation_endpoint, &entry.refresh_token) {
            (Some(endpoint), Some(token)) => Some(oauth::revoke(endpoint, &entry.client_id, token).map_err(|e| {
                format!("could not revoke the sign-in at the login service ({e}) — it stays live until it expires")
            })),
            _ => None,
        },
        None => None,
    };
    match revoked {
        Some(Ok(())) => outcome.revoked = true,
        Some(Err(warning)) => outcome.revoke_warning = Some(warning),
        None => {}
    }

    if keyring_reads {
        for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
            credentials.delete(origin, kind)?;
        }
    }
    if let Some(dir) = credentials_file_dir {
        auth::remove_token_at(dir, origin)
            .map_err(|e| format!("could not update the credentials file: {e}"))?;
    }
    Ok(outcome)
}

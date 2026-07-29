//! Credential storage and the resolution ladder.
//!
//! PATs live in a YAML map (`<url origin> → <token>`) in the user-level config
//! directory — never inside the repo. The device flow's refresh credential and
//! the cached access token live in the OS keyring, shared with the desktop app
//! so one login on a machine covers both.
//!
//! Resolution walks four layers in order: `SPECLINK_TOKEN` → keyring refresh
//! → keyring PAT → credentials file. A layer that cannot answer falls through
//! silently; that is what keeps a headless CI box (no keyring at all) working
//! exactly as it did before the keyring layers existed.

use crate::credentials::{CredentialKind, CredentialStore};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The credentials file inside an explicit base directory (test seam).
pub fn credentials_path_in(dir: &Path) -> PathBuf {
    dir.join("credentials.yaml")
}

/// The credentials file in the user-level config directory.
pub fn credentials_path() -> PathBuf {
    credentials_path_in(&speclink_config_dir())
}

/// The user-level config directory (same base as the global CLI config).
pub fn speclink_config_dir() -> PathBuf {
    speclink_host::context::global_config_dir()
}

/// The origin (`scheme://host[:port]`) of a connection URL — credentials are
/// keyed by origin so one login covers every project on the same server.
pub fn origin_of(url: &str) -> String {
    match url.find("://") {
        Some(scheme_end) => {
            let host_start = scheme_end + 3;
            let end = url[host_start..]
                .find('/')
                .map(|i| host_start + i)
                .unwrap_or(url.len());
            url[..end].to_string()
        }
        None => url.trim_end_matches('/').to_string(),
    }
}

fn read_map(path: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_yaml::from_str(&text).ok())
        .unwrap_or_default()
}

/// Store `token` for `origin` in the credentials file under `dir`,
/// creating the file with owner-only permissions (Unix 0600; Windows relies
/// on the user-profile directory ACL). Other origins in the file survive.
pub fn save_token_at(dir: &Path, origin: &str, token: &str) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = credentials_path_in(dir);
    let mut map = read_map(&path);
    map.insert(origin.to_string(), token.to_string());
    let yaml = serde_yaml::to_string(&map)?;

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        // Create with 0600 so the token is never world-readable, not even
        // briefly; tighten a pre-existing file's mode as well.
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        f.write_all(yaml.as_bytes())?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    std::fs::write(&path, yaml)?;

    Ok(path)
}

/// Read the token stored for `origin`, if any. A missing file or missing
/// entry is simply `None` (treated as "not logged in").
pub fn load_token_at(dir: &Path, origin: &str) -> Option<String> {
    read_map(&credentials_path_in(dir)).remove(origin)
}

/// Drop `origin`'s entry from the credentials file, leaving other origins
/// alone. Returns whether an entry was actually removed. A missing file is not
/// an error — logout must never be blockable by an absent credential.
pub fn remove_token_at(dir: &Path, origin: &str) -> anyhow::Result<bool> {
    let path = credentials_path_in(dir);
    let mut map = read_map(&path);
    if map.remove(origin).is_none() {
        return Ok(false);
    }
    std::fs::write(&path, serde_yaml::to_string(&map)?)?;
    Ok(true)
}

/// Which layer of the ladder a credential came from. Surfaced by
/// `speclink auth status` — with four possible sources, "which credential am I
/// even using?" is the first question when a shared login misbehaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    Env,
    KeychainRefresh,
    KeychainPat,
    CredentialsFile,
}

impl CredentialSource {
    /// The `credentialSource` value in `--json` output. These strings are an
    /// external contract.
    pub fn as_str(self) -> &'static str {
        match self {
            CredentialSource::Env => "env",
            CredentialSource::KeychainRefresh => "keychain_refresh",
            CredentialSource::KeychainPat => "keychain_pat",
            CredentialSource::CredentialsFile => "credentials_file",
        }
    }

    /// The human-readable line for `speclink auth status`.
    pub fn describe(self) -> &'static str {
        match self {
            CredentialSource::Env => "SPECLINK_TOKEN environment variable",
            CredentialSource::KeychainRefresh => "system keychain (device login, shared with the desktop app)",
            CredentialSource::KeychainPat => "system keychain (personal access token)",
            CredentialSource::CredentialsFile => "credentials file",
        }
    }
}

/// A resolved credential: the bearer to send, and which layer produced it.
#[derive(Debug, Clone)]
pub struct ResolvedCredential {
    pub token: String,
    pub source: CredentialSource,
}

/// Walk the ladder against an explicit config dir, credential store and
/// environment value (the test seam).
///
/// A keyring that cannot answer — no service on this platform, access denied,
/// no entry — is skipped silently, which is what keeps a headless CI box and a
/// macOS user who clicked Deny working exactly as before.
///
/// A keyring that *does* hold a refresh credential is a different matter: once
/// that layer owns the request, a failed rotation is an error, not a reason to
/// try the next layer. Falling through would swap identity mid-verb, and the
/// canonical contract forbids silently reaching for another credential source.
/// A revoked family is cleared on the way out so the next invocation resolves
/// to whatever layers remain.
pub fn resolve_credential_at(
    dir: &Path,
    origin: &str,
    credentials: &dyn CredentialStore,
    env_token: Option<String>,
) -> Result<Option<ResolvedCredential>, crate::refresh::RefreshFailure> {
    if let Some(token) = env_token.filter(|t| !t.trim().is_empty()) {
        return Ok(Some(ResolvedCredential { token, source: CredentialSource::Env }));
    }

    let has_refresh = matches!(credentials.get(origin, CredentialKind::Refresh), Ok(Some(_)));
    if has_refresh {
        return match crate::refresh::bearer_for(origin, credentials, dir) {
            Ok(token) => Ok(Some(ResolvedCredential {
                token,
                source: CredentialSource::KeychainRefresh,
            })),
            Err(crate::refresh::RefreshFailure::Rejected(msg)) => {
                clear_keychain_login(origin, credentials);
                Err(crate::refresh::RefreshFailure::Rejected(msg))
            }
            Err(transient) => Err(transient),
        };
    }

    if let Ok(Some(token)) = credentials.get(origin, CredentialKind::Pat) {
        return Ok(Some(ResolvedCredential { token, source: CredentialSource::KeychainPat }));
    }

    Ok(load_token_at(dir, origin).map(|token| ResolvedCredential {
        token,
        source: CredentialSource::CredentialsFile,
    }))
}

/// Drop the device-login credentials for `origin` — the refresh credential and
/// the cached access token. The PAT layers are left alone: they are separate
/// logins that a dead refresh family says nothing about.
pub fn clear_keychain_login(origin: &str, credentials: &dyn CredentialStore) {
    let _ = credentials.delete(origin, CredentialKind::Refresh);
    let _ = credentials.delete(origin, CredentialKind::Bearer);
}

/// Why an authenticated attempt could not be completed.
#[derive(Debug)]
pub enum CredentialError {
    /// No layer of the ladder holds a credential for this origin.
    NotLoggedIn,
    /// The refresh credential could not be rotated.
    Rotation(crate::refresh::RefreshFailure),
    /// The attempt itself failed — including a rejection that a rotation
    /// could not rescue.
    Remote(crate::RemoteError),
}

impl CredentialError {
    /// The single line a user sees.
    pub fn message(self, origin: &str) -> String {
        match self {
            CredentialError::NotLoggedIn => {
                format!("not logged in to {origin} — run `speclink auth login`")
            }
            CredentialError::Rotation(f) => f.message(),
            CredentialError::Remote(e) => e.message,
        }
    }
}

/// What an authenticated attempt produced, plus the layer that authenticated it.
#[derive(Debug)]
pub struct Authenticated<T> {
    pub value: T,
    pub source: CredentialSource,
}

/// Resolve a credential and run `attempt` with it.
///
/// A cached access token can be refused while the refresh credential behind it
/// is still good — it aged out, or the server restarted. That case is worth
/// exactly one rotation and one retry, and the user never sees it. Every other
/// refusal surfaces: a PAT cannot be rotated, and a 403 means the identity was
/// accepted and the permission was not, so rotating would burn a credential
/// for nothing.
pub fn with_credential<T, E>(
    dir: &Path,
    origin: &str,
    credentials: &dyn CredentialStore,
    env_token: Option<String>,
    attempt: impl Fn(&str) -> Result<T, E>,
) -> Result<Authenticated<T>, CredentialError>
where
    E: Into<crate::RemoteError>,
{
    let resolved = resolve_credential_at(dir, origin, credentials, env_token)
        .map_err(CredentialError::Rotation)?
        .ok_or(CredentialError::NotLoggedIn)?;
    let source = resolved.source;

    match attempt(&resolved.token) {
        Ok(value) => Ok(Authenticated { value, source }),
        Err(err) => {
            let err: crate::RemoteError = err.into();
            if source != CredentialSource::KeychainRefresh || err.status != Some(401) {
                return Err(CredentialError::Remote(err));
            }
            // The cached token is spent; a rotation is the one thing that can
            // rescue this verb.
            crate::refresh::clear_cached_bearer(origin, credentials);
            let token = crate::refresh::rotate(origin, credentials, dir).map_err(|failure| {
                if matches!(failure, crate::refresh::RefreshFailure::Rejected(_)) {
                    clear_keychain_login(origin, credentials);
                }
                CredentialError::Rotation(failure)
            })?;
            attempt(&token)
                .map(|value| Authenticated { value, source })
                .map_err(|e| CredentialError::Remote(e.into()))
        }
    }
}

/// [`with_credential`] against the real config dir, the OS keyring and the
/// process environment.
pub fn with_resolved_credential<T, E>(
    origin: &str,
    attempt: impl Fn(&str) -> Result<T, E>,
) -> Result<Authenticated<T>, CredentialError>
where
    E: Into<crate::RemoteError>,
{
    with_credential(
        &speclink_config_dir(),
        origin,
        &crate::credentials::KeyringCredentialStore,
        std::env::var("SPECLINK_TOKEN").ok(),
        attempt,
    )
}

/// [`resolve_credential_at`] against the real config dir, the OS keyring and
/// the process environment.
pub fn resolve_credential(
    origin: &str,
) -> Result<Option<ResolvedCredential>, crate::refresh::RefreshFailure> {
    resolve_credential_at(
        &speclink_config_dir(),
        origin,
        &crate::credentials::KeyringCredentialStore,
        std::env::var("SPECLINK_TOKEN").ok(),
    )
}

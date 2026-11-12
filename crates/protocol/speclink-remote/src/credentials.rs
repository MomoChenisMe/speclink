//! The one way credentials enter and leave a client: the OS keyring, keyed by
//! server origin plus credential kind.
//!
//! Lives here rather than in a front-end so the CLI and the desktop app read
//! and write the *same* entries — one login on a machine covers both (see
//! `openspec/changes/cli-desktop-credential-sharing/design.md`). The keyring
//! service name and account format are frozen: changing either orphans every
//! existing login.
//!
//! CI has no headless keyring, so the trait is the only testable shape —
//! tests and orchestration inject [`MemoryCredentialStore`]. Error strings
//! never carry secret material.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// The keyring service name every entry is filed under (manual check:
/// `security find-generic-password -s speclink-desktop`). Frozen for
/// compatibility with logins written when the desktop app owned this store.
pub const SERVICE: &str = "speclink-desktop";

/// What a stored secret is: the device flow's rotating refresh credential, a
/// long-lived PAT, or the cached short-lived access token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKind {
    Refresh,
    Pat,
    /// The cached access token plus its expiry, as [`CachedBearer`] JSON.
    /// Short-lived processes have no memory to keep a bearer in, so caching it
    /// here keeps every verb from spending a rotation.
    Bearer,
}

impl CredentialKind {
    fn slug(self) -> &'static str {
        match self {
            CredentialKind::Refresh => "refresh",
            CredentialKind::Pat => "pat",
            CredentialKind::Bearer => "bearer",
        }
    }
}

/// The value of a keyring `refresh:<origin>` entry (client-browser-login D4).
/// A device login stores the bare refresh token, as it always has; a browser
/// login stores [`OAuthRefresh`] as JSON in the same entry, so the resolution
/// ladder, the rotation lock and logout cover both without a fifth layer.
#[derive(Clone, PartialEq)]
pub enum RefreshEntry {
    Device(String),
    OAuth(OAuthRefresh),
}

/// Printed without the token, so a `{:?}` in a log line cannot leak it.
impl std::fmt::Debug for RefreshEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefreshEntry::Device(_) => f.write_str("Device(<redacted>)"),
            RefreshEntry::OAuth(oauth) => f.debug_tuple("OAuth").field(oauth).finish(),
        }
    }
}

/// What a browser login needs to refresh and revoke: the login service's
/// endpoints are kept beside the token, so a refresh never re-reads the
/// service's metadata.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthRefresh {
    /// `None` when the login service issued no refresh token: the login lasts
    /// as long as its access token.
    pub refresh_token: Option<String>,
    pub token_endpoint: String,
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,
}

/// Printed without the token, so a `{:?}` in a log line cannot leak it.
impl std::fmt::Debug for OAuthRefresh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthRefresh")
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "<redacted>"))
            .field("token_endpoint", &self.token_endpoint)
            .field("client_id", &self.client_id)
            .field("revocation_endpoint", &self.revocation_endpoint)
            .finish()
    }
}

/// The tagged JSON shape of a browser login's entry: `{"kind":"oauth", …}`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum StoredRefresh {
    Oauth(OAuthRefresh),
}

impl RefreshEntry {
    /// Read an entry. Anything that is not a `kind: oauth` object is a device
    /// login's refresh token — which is every entry written before browser
    /// login existed, so none of them needs migrating.
    pub fn parse(raw: &str) -> RefreshEntry {
        match serde_json::from_str::<StoredRefresh>(raw) {
            Ok(StoredRefresh::Oauth(oauth)) => RefreshEntry::OAuth(oauth),
            Err(_) => RefreshEntry::Device(raw.to_string()),
        }
    }

    /// The entry for a device login's refresh token. The entry's kind is told
    /// from its content, so a token that reads as a browser-login entry is
    /// refused: stored as it is, it would later send refresh and revocation
    /// requests to whatever endpoints the server wrote into it.
    pub fn device(token: String) -> Result<RefreshEntry, String> {
        match RefreshEntry::parse(&token) {
            RefreshEntry::Device(token) => Ok(RefreshEntry::Device(token)),
            RefreshEntry::OAuth(_) => Err("server 回傳的 refresh credential 格式不合法".to_string()),
        }
    }

    /// The entry's stored value.
    pub fn encode(&self) -> String {
        match self {
            RefreshEntry::Device(token) => token.clone(),
            RefreshEntry::OAuth(oauth) => serde_json::to_string(&StoredRefresh::Oauth(oauth.clone()))
                .expect("a struct of strings always serializes"),
        }
    }
}

/// Per-origin, per-kind credential access. Deletion is idempotent by
/// contract: an already-absent entry is not an error, so local cleanup during
/// logout can never be blocked.
pub trait CredentialStore: Send + Sync {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String>;
    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String>;
    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String>;
}

/// in-memory implementation for tests and orchestration tests.
#[derive(Default)]
pub struct MemoryCredentialStore {
    slots: Mutex<HashMap<(String, CredentialKind), String>>,
}

impl MemoryCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
        Ok(self
            .slots
            .lock()
            .expect("credential lock")
            .get(&(origin.to_string(), kind))
            .cloned())
    }

    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
        self.slots
            .lock()
            .expect("credential lock")
            .insert((origin.to_string(), kind), secret.to_string());
        Ok(())
    }

    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
        self.slots
            .lock()
            .expect("credential lock")
            .remove(&(origin.to_string(), kind));
        Ok(())
    }
}

/// keyring-backed production implementation: service [`SERVICE`], account
/// `<kind>:<origin>`.
pub struct KeyringCredentialStore;

/// Whether this platform's keyring keeps what it is given. Where no native
/// store is compiled in (everything but macOS and Windows here), the keyring
/// crate falls back to an in-memory mock whose "successful" writes vanish with
/// the entry — a login or PAT stored there would be lost on the spot. Such a
/// keyring counts as unavailable, so callers fall back or refuse loudly.
fn keyring_persists() -> bool {
    use keyring::credential::CredentialPersistence;
    matches!(
        keyring::default::default_credential_builder().persistence(),
        CredentialPersistence::UntilDelete | CredentialPersistence::UntilReboot
    )
}

fn entry(origin: &str, kind: CredentialKind) -> Result<keyring::Entry, String> {
    if !keyring_persists() {
        return Err("此平台沒有系統 Keychain".to_string());
    }
    keyring::Entry::new(SERVICE, &format!("{}:{origin}", kind.slug()))
        .map_err(|e| format!("無法存取系統 Keychain：{e}"))
}

impl CredentialStore for KeyringCredentialStore {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
        match entry(origin, kind)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("無法讀取系統 Keychain：{e}")),
        }
    }

    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
        entry(origin, kind)?
            .set_password(secret)
            .map_err(|e| format!("無法寫入系統 Keychain：{e}"))
    }

    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
        match entry(origin, kind)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("無法刪除系統 Keychain entry：{e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_returns_none_for_unknown_entry() {
        let store = MemoryCredentialStore::new();
        assert_eq!(
            store.get("https://a.example", CredentialKind::Refresh).unwrap(),
            None
        );
    }

    #[test]
    fn memory_store_isolates_by_kind() {
        let store = MemoryCredentialStore::new();
        store.set("https://a.example", CredentialKind::Refresh, "r").unwrap();
        store.set("https://a.example", CredentialKind::Pat, "p").unwrap();
        store.set("https://a.example", CredentialKind::Bearer, "b").unwrap();

        assert_eq!(
            store.get("https://a.example", CredentialKind::Refresh).unwrap(),
            Some("r".to_string())
        );
        assert_eq!(
            store.get("https://a.example", CredentialKind::Pat).unwrap(),
            Some("p".to_string())
        );
        assert_eq!(
            store.get("https://a.example", CredentialKind::Bearer).unwrap(),
            Some("b".to_string())
        );
    }

    #[test]
    fn memory_store_isolates_by_origin() {
        let store = MemoryCredentialStore::new();
        store.set("https://a.example", CredentialKind::Refresh, "a").unwrap();
        store.set("https://b.example", CredentialKind::Refresh, "b").unwrap();

        assert_eq!(
            store.get("https://a.example", CredentialKind::Refresh).unwrap(),
            Some("a".to_string())
        );
        assert_eq!(
            store.get("https://b.example", CredentialKind::Refresh).unwrap(),
            Some("b".to_string())
        );
    }

    #[test]
    fn memory_store_delete_removes_only_the_named_entry() {
        let store = MemoryCredentialStore::new();
        store.set("https://a.example", CredentialKind::Refresh, "r").unwrap();
        store.set("https://a.example", CredentialKind::Pat, "p").unwrap();

        store.delete("https://a.example", CredentialKind::Refresh).unwrap();

        assert_eq!(
            store.get("https://a.example", CredentialKind::Refresh).unwrap(),
            None
        );
        assert_eq!(
            store.get("https://a.example", CredentialKind::Pat).unwrap(),
            Some("p".to_string())
        );
    }

    /// Logout must not be blockable by an already-absent entry.
    #[test]
    fn memory_store_delete_is_idempotent() {
        let store = MemoryCredentialStore::new();
        assert!(store.delete("https://a.example", CredentialKind::Pat).is_ok());
        assert!(store.delete("https://a.example", CredentialKind::Pat).is_ok());
    }

    #[test]
    fn set_overwrites_the_previous_secret() {
        let store = MemoryCredentialStore::new();
        store.set("https://a.example", CredentialKind::Refresh, "old").unwrap();
        store.set("https://a.example", CredentialKind::Refresh, "new").unwrap();

        assert_eq!(
            store.get("https://a.example", CredentialKind::Refresh).unwrap(),
            Some("new".to_string())
        );
    }

    #[test]
    fn a_bare_refresh_token_reads_as_a_device_login() {
        assert_eq!(
            RefreshEntry::parse("spk_rt_abc"),
            RefreshEntry::Device("spk_rt_abc".to_string())
        );
    }

    /// JSON that is not a `kind: oauth` object, or is broken, falls back to the
    /// old format rather than failing.
    #[test]
    fn unrecognized_json_reads_as_a_device_login() {
        for raw in [r#"{"kind":"other"}"#, r#"{"kind":"oauth""#, r#"{"refreshToken":"x"}"#, "null"] {
            assert_eq!(RefreshEntry::parse(raw), RefreshEntry::Device(raw.to_string()), "{raw}");
        }
    }

    #[test]
    fn a_device_token_shaped_like_a_browser_login_entry_is_refused() {
        assert!(RefreshEntry::device("spk_rt_abc".into()).is_ok());
        let hostile = r#"{"kind":"oauth","refreshToken":"x","tokenEndpoint":"http://evil.example/t","clientId":"c"}"#;
        assert!(RefreshEntry::device(hostile.into()).is_err());
    }

    /// An entry printed with `{:?}` must not carry its refresh token.
    #[test]
    fn a_refresh_entry_debug_print_hides_the_token() {
        let device = format!("{:?}", RefreshEntry::Device("spk_rt_secret".into()));
        let oauth = format!(
            "{:?}",
            RefreshEntry::OAuth(OAuthRefresh {
                refresh_token: Some("rt-secret".into()),
                token_endpoint: "https://id.example.com/token".into(),
                client_id: "speclink".into(),
                revocation_endpoint: None,
            })
        );
        assert!(!device.contains("spk_rt_secret"), "{device}");
        assert!(!oauth.contains("rt-secret"), "{oauth}");
        assert!(oauth.contains("https://id.example.com/token"), "non-secret fields stay readable: {oauth}");
    }

    #[test]
    fn an_oauth_entry_round_trips_in_the_d4_shape() {
        let entry = RefreshEntry::OAuth(OAuthRefresh {
            refresh_token: None,
            token_endpoint: "https://id.example.com/token".into(),
            client_id: "speclink".into(),
            revocation_endpoint: None,
        });
        let raw = entry.encode();
        assert_eq!(
            raw,
            r#"{"kind":"oauth","refreshToken":null,"tokenEndpoint":"https://id.example.com/token","clientId":"speclink"}"#
        );
        assert_eq!(RefreshEntry::parse(&raw), entry);
    }

    /// The native keyrings persist; anything else is the crate's in-memory
    /// mock, which must read as "no keyring".
    #[test]
    fn only_a_native_keyring_counts_as_available() {
        assert_eq!(keyring_persists(), cfg!(any(target_os = "macos", target_os = "windows")));
    }

    /// The account format is a compatibility contract: desktop logins written
    /// before this crate owned the store must still resolve.
    #[test]
    fn account_names_are_stable_per_kind() {
        assert_eq!(CredentialKind::Refresh.slug(), "refresh");
        assert_eq!(CredentialKind::Pat.slug(), "pat");
        assert_eq!(CredentialKind::Bearer.slug(), "bearer");
        assert_eq!(SERVICE, "speclink-desktop");
    }
}

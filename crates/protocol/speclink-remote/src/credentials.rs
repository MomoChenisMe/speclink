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

fn entry(origin: &str, kind: CredentialKind) -> Result<keyring::Entry, String> {
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

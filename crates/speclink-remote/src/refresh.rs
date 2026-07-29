//! Bearer acquisition and refresh rotation, serialized across processes.
//!
//! The desktop app and every CLI process on a machine share one credential
//! family (cli-desktop-credential-sharing). The server spends a refresh
//! credential on use and treats a re-spend as a leak — it tears the family
//! down — so two clients rotating at once would log each other out. A lock
//! file in the user-level config directory serializes the whole read → rotate
//! → write-back sequence.
//!
//! The lock is only half of it: waiting for the lock and *then* rotating would
//! still spend one credential per waiter. So the cached access token is
//! re-read inside the lock — whoever wins rotates, everyone else takes what
//! the winner cached. Short-lived CLI processes have nowhere else to keep a
//! bearer, which is why the cache lives beside the credentials rather than in
//! memory.

use crate::credentials::{CredentialKind, CredentialStore};
use crate::device;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The lock file name in the user-level config directory. Separate from
/// `credentials.yaml` so PAT reads and writes never contend with rotation.
const LOCK_FILE: &str = "refresh.lock";

/// Treat a token expiring within this window as already expired: a bearer that
/// dies mid-request costs a round trip and an error the user sees.
const EXPIRY_SKEW_SECS: i64 = 30;

/// How long to wait for the lock before giving up. Comfortably longer than a
/// rotation round trip, short enough that a wedged holder does not look like a
/// hung command.
const LOCK_WAIT: std::time::Duration = std::time::Duration::from_secs(60);
const LOCK_POLL: std::time::Duration = std::time::Duration::from_millis(50);

/// The cached access token. An internal storage format, not a wire contract —
/// but the field names are still camelCase, matching every other JSON this
/// project writes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedBearer {
    token: String,
    /// Absolute expiry as a Unix timestamp (seconds).
    expires_at: i64,
}

/// Why a rotation failed. The split drives the caller's decision: throw the
/// credential away and demand a new login, or keep it and say "try later".
/// A transient failure must never read as "your credential is dead".
#[derive(Debug)]
pub enum RefreshFailure {
    /// The server refused (`permission_denied`), or there is no credential at
    /// all — this credential is spent; logging in again is the only way out.
    Rejected(String),
    /// The credential's fate is unknown: unreachable server, 5xx, or a local
    /// keyring fault. Keep it and report a connection error.
    Unavailable(String),
}

impl RefreshFailure {
    /// The single line a user sees (both variants are one error line in a UI).
    pub fn message(self) -> String {
        match self {
            RefreshFailure::Rejected(m) | RefreshFailure::Unavailable(m) => m,
        }
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The unexpired cached access token for `origin`, if there is one. An absent
/// entry, unreadable keyring, unparseable value, or expired token are all the
/// same answer — no usable cache — so a corrupt entry degrades to a rotation
/// rather than an error.
pub fn cached_bearer(origin: &str, credentials: &dyn CredentialStore) -> Option<String> {
    let raw = credentials.get(origin, CredentialKind::Bearer).ok()??;
    let cached: CachedBearer = serde_json::from_str(&raw).ok()?;
    (cached.expires_at - EXPIRY_SKEW_SECS > now_secs()).then_some(cached.token)
}

/// Cache `token` for `origin`, expiring `expires_in` seconds from now.
/// A fresh login writes this alongside the refresh credential so the next
/// command costs no rotation.
pub fn store_cached_bearer(
    origin: &str,
    credentials: &dyn CredentialStore,
    token: &str,
    expires_in: u64,
) -> Result<(), String> {
    let cached = CachedBearer {
        token: token.to_string(),
        expires_at: now_secs() + expires_in as i64,
    };
    let encoded = serde_json::to_string(&cached)
        .map_err(|e| format!("無法寫入 access token 快取：{e}"))?;
    credentials.set(origin, CredentialKind::Bearer, &encoded)
}

fn store_bearer(
    origin: &str,
    credentials: &dyn CredentialStore,
    token: &str,
    expires_in: u64,
) -> Result<(), RefreshFailure> {
    store_cached_bearer(origin, credentials, token, expires_in)
        .map_err(RefreshFailure::Unavailable)
}

/// Drop the cached access token. Called when the server rejects it, so the
/// next attempt rotates instead of replaying a token the server already
/// refuses.
pub fn clear_cached_bearer(origin: &str, credentials: &dyn CredentialStore) {
    let _ = credentials.delete(origin, CredentialKind::Bearer);
}

/// Take the rotation lock for the duration of `f`. The lock file lives in
/// `lock_dir` and carries no data — it exists only to serialize.
///
/// Anything that writes the refresh credential takes this, not just rotation:
/// a login landing in the middle of another process's read-rotate-write would
/// otherwise be overwritten by a credential from the family it replaced.
pub fn with_rotation_lock<T>(
    lock_dir: &Path,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    with_rotation_lock_for(lock_dir, LOCK_WAIT, f)
}

/// [`with_rotation_lock`] with an explicit wait budget — tests need a wedged
/// holder to fail in milliseconds rather than a minute.
pub fn with_rotation_lock_for<T>(
    lock_dir: &Path,
    wait: std::time::Duration,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    with_lock(lock_dir, wait, || f().map_err(RefreshFailure::Unavailable))
        .map_err(RefreshFailure::message)
}

fn with_lock<T>(
    lock_dir: &Path,
    wait: std::time::Duration,
    f: impl FnOnce() -> Result<T, RefreshFailure>,
) -> Result<T, RefreshFailure> {
    std::fs::create_dir_all(lock_dir).map_err(|e| {
        RefreshFailure::Unavailable(format!("無法建立設定目錄 {}：{e}", lock_dir.display()))
    })?;
    let path = lock_dir.join(LOCK_FILE);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|e| {
            RefreshFailure::Unavailable(format!("無法開啟換發鎖檔 {}：{e}", path.display()))
        })?;
    acquire(&file, &path, wait)?;
    let outcome = f();
    let _ = fs2::FileExt::unlock(&file);
    outcome
}

/// Wait for the lock, but not forever. Blocking indefinitely would hang a verb
/// with no output and nothing for the user to act on; a bounded wait turns a
/// wedged holder into an error that names the problem. The OS releases the
/// lock when a process dies, so only a live, stuck holder can exhaust this.
fn acquire(
    file: &std::fs::File,
    path: &Path,
    wait: std::time::Duration,
) -> Result<(), RefreshFailure> {
    let deadline = std::time::Instant::now() + wait;
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    return Err(RefreshFailure::Unavailable(format!(
                        "等待換發鎖 {} 逾時——可能有其他 speclink 行程長時間持鎖",
                        path.display()
                    )));
                }
                std::thread::sleep(LOCK_POLL);
            }
            Err(e) => {
                return Err(RefreshFailure::Unavailable(format!(
                    "無法取得換發鎖 {}：{e}",
                    path.display()
                )))
            }
        }
    }
}

/// Rotate inside an already-held lock: re-read the refresh credential (a
/// leader may have replaced it while we waited), spend it, write the new pair
/// back.
fn rotate_held(
    origin: &str,
    credentials: &dyn CredentialStore,
) -> Result<String, RefreshFailure> {
    let Some(current) = credentials
        .get(origin, CredentialKind::Refresh)
        .map_err(RefreshFailure::Unavailable)?
    else {
        return Err(RefreshFailure::Rejected(
            "此連線沒有 refresh credential——請重新登入".to_string(),
        ));
    };
    let rotated = device::refresh(origin, &current).map_err(|e| match e.reason.as_deref() {
        Some("permission_denied") => RefreshFailure::Rejected(e.message),
        _ => RefreshFailure::Unavailable(e.message),
    })?;
    credentials
        .set(origin, CredentialKind::Refresh, &rotated.refresh_token)
        .map_err(RefreshFailure::Unavailable)?;
    store_bearer(origin, credentials, &rotated.access_token, rotated.expires_in)?;
    Ok(rotated.access_token)
}

/// Rotate the credential family, ignoring any cached access token. Use this
/// when the server has rejected the cached bearer; otherwise use
/// [`bearer_for`], which spends a rotation only when it must.
pub fn rotate(
    origin: &str,
    credentials: &dyn CredentialStore,
    lock_dir: &Path,
) -> Result<String, RefreshFailure> {
    with_lock(lock_dir, LOCK_WAIT, || rotate_held(origin, credentials))
}

/// A usable bearer for `origin`: the cached access token while it lives,
/// otherwise one rotation under the lock.
///
/// The cache is checked twice — once without the lock (the common path costs
/// no I/O contention) and again while holding it, because a caller that just
/// waited out a rotation should take the winner's token rather than spend
/// another credential.
pub fn bearer_for(
    origin: &str,
    credentials: &dyn CredentialStore,
    lock_dir: &Path,
) -> Result<String, RefreshFailure> {
    if let Some(cached) = cached_bearer(origin, credentials) {
        return Ok(cached);
    }
    with_lock(lock_dir, LOCK_WAIT, || {
        if let Some(cached) = cached_bearer(origin, credentials) {
            return Ok(cached);
        }
        rotate_held(origin, credentials)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::MemoryCredentialStore;

    #[test]
    fn unexpired_cache_is_returned() {
        let store = MemoryCredentialStore::new();
        let raw = serde_json::to_string(&CachedBearer {
            token: "live".into(),
            expires_at: now_secs() + 600,
        })
        .unwrap();
        store.set("o", CredentialKind::Bearer, &raw).unwrap();

        assert_eq!(cached_bearer("o", &store), Some("live".to_string()));
    }

    #[test]
    fn expired_cache_is_a_miss() {
        let store = MemoryCredentialStore::new();
        let raw = serde_json::to_string(&CachedBearer {
            token: "stale".into(),
            expires_at: now_secs() - 1,
        })
        .unwrap();
        store.set("o", CredentialKind::Bearer, &raw).unwrap();

        assert_eq!(cached_bearer("o", &store), None);
    }

    /// A token that expires inside the skew window is treated as gone: using
    /// it risks the request outliving it.
    #[test]
    fn cache_within_expiry_skew_is_a_miss() {
        let store = MemoryCredentialStore::new();
        let raw = serde_json::to_string(&CachedBearer {
            token: "almost".into(),
            expires_at: now_secs() + EXPIRY_SKEW_SECS - 1,
        })
        .unwrap();
        store.set("o", CredentialKind::Bearer, &raw).unwrap();

        assert_eq!(cached_bearer("o", &store), None);
    }

    /// Anything unparseable degrades to a cache miss — never an error. Older
    /// clients wrote no Bearer entry at all, and a corrupt one must not brick
    /// a login.
    #[test]
    fn unparseable_cache_is_a_miss() {
        let store = MemoryCredentialStore::new();
        store.set("o", CredentialKind::Bearer, "not json").unwrap();

        assert_eq!(cached_bearer("o", &store), None);
    }

    #[test]
    fn absent_cache_is_a_miss() {
        let store = MemoryCredentialStore::new();
        assert_eq!(cached_bearer("o", &store), None);
    }

    #[test]
    fn cached_bearer_is_serialized_with_camel_case_fields() {
        let raw = serde_json::to_string(&CachedBearer {
            token: "t".into(),
            expires_at: 42,
        })
        .unwrap();
        assert_eq!(raw, r#"{"token":"t","expiresAt":42}"#);
    }

    #[test]
    fn clear_cached_bearer_leaves_other_kinds_alone() {
        let store = MemoryCredentialStore::new();
        store.set("o", CredentialKind::Bearer, "{}").unwrap();
        store.set("o", CredentialKind::Refresh, "r").unwrap();

        clear_cached_bearer("o", &store);

        assert_eq!(store.get("o", CredentialKind::Bearer).unwrap(), None);
        assert_eq!(
            store.get("o", CredentialKind::Refresh).unwrap(),
            Some("r".to_string())
        );
    }
}

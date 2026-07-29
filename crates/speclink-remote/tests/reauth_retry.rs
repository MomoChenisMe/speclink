//! What a verb does when the server rejects its credential
//! (cli-desktop-credential-sharing:「憑證失效的處理」).
//!
//! A cached access token can be refused while the refresh credential behind it
//! is still perfectly good — the token simply aged out, or the server
//! restarted. That case buys exactly one rotation and one retry, and the user
//! sees nothing. A refusal of the *rotation* is the real end of the line: the
//! family is gone, the local entries go with it, and the verb fails asking for
//! a fresh login.
//!
//! Everything runs against an in-memory store: a test that reached for the
//! real OS keyring could not run on CI, and would rewrite the developer's own
//! credentials if it did.

use speclink_remote::auth::{self, CredentialError, CredentialSource};
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct Stub {
    base: String,
    rotations: Arc<AtomicUsize>,
    attempts: Arc<AtomicUsize>,
}

/// A server that rotates successfully up to `rotations_allowed` times, then
/// refuses. `/auth/refresh` is the only path it serves; verb attempts are
/// simulated by the closure the test passes in.
fn stub(rotations_allowed: usize) -> Stub {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", server.server_addr());
    let rotations = Arc::new(AtomicUsize::new(0));
    let counter = rotations.clone();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let resp = if n <= rotations_allowed {
                let body = format!(
                    r#"{{"accessToken":"access-{n}","refreshToken":"refresh-{n}","expiresIn":3600}}"#
                );
                tiny_http::Response::from_string(body)
            } else {
                tiny_http::Response::from_string(
                    r#"{"status":401,"reason":"permission_denied","message":"refresh credential is not live"}"#,
                )
                .with_status_code(401)
            };
            let resp = resp.with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("header"),
            );
            let _ = req.respond(resp);
        }
    });
    Stub { base, rotations, attempts: Arc::new(AtomicUsize::new(0)) }
}

fn unauthorized() -> speclink_remote::RemoteError {
    speclink_remote::RemoteError {
        message: "authentication failed".into(),
        reason: Some("permission_denied".into()),
        status: Some(401),
    }
}

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

/// The everyday case: a stale cached token, a healthy family. One rotation,
/// one retry, no user-visible interruption.
#[test]
fn a_rejected_cached_bearer_rotates_once_and_retries() {
    let server = stub(9);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Refresh, "seed").unwrap();
    seed_live_bearer(&store, &server.base, "stale-access");

    let attempts = server.attempts.clone();
    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |bearer| {
        let n = attempts.fetch_add(1, Ordering::SeqCst) + 1;
        if bearer == "stale-access" {
            Err(unauthorized())
        } else {
            Ok(format!("ok:{bearer}:attempt{n}"))
        }
    })
    .expect("換發後重試應成功");

    assert_eq!(outcome.value, "ok:access-1:attempt2");
    assert_eq!(outcome.source, CredentialSource::KeychainRefresh);
    assert_eq!(server.attempts.load(Ordering::SeqCst), 2, "恰重試一次");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 1, "恰換發一次");
}

/// One retry, not a loop. A server that refuses even a freshly minted token
/// must stop the verb, not spin.
#[test]
fn the_retry_happens_at_most_once() {
    let server = stub(9);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Refresh, "seed").unwrap();
    seed_live_bearer(&store, &server.base, "stale-access");

    let attempts = server.attempts.clone();
    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |_| {
        attempts.fetch_add(1, Ordering::SeqCst);
        Err::<(), _>(unauthorized())
    });

    assert!(matches!(outcome, Err(CredentialError::Remote(_))));
    assert_eq!(server.attempts.load(Ordering::SeqCst), 2, "不得重試超過一次");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 1);
}

/// The end of the line: the rotation itself is refused. Local device-login
/// entries are cleared and the verb asks for a new login.
#[test]
fn a_refused_rotation_clears_the_login_and_fails() {
    let server = stub(0);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Refresh, "dead").unwrap();
    seed_live_bearer(&store, &server.base, "stale-access");

    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |_| {
        Err::<(), _>(unauthorized())
    });

    assert!(matches!(outcome, Err(CredentialError::Rotation(_))));
    assert_eq!(store.get(&server.base, CredentialKind::Refresh).unwrap(), None);
    assert_eq!(store.get(&server.base, CredentialKind::Bearer).unwrap(), None);
}

/// A PAT is not rotatable, so a refusal is final — no retry, and the PAT stays
/// where it is for the user to deal with.
#[test]
fn a_rejected_pat_is_not_retried() {
    let server = stub(9);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Pat, "the-pat").unwrap();

    let attempts = server.attempts.clone();
    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |_| {
        attempts.fetch_add(1, Ordering::SeqCst);
        Err::<(), _>(unauthorized())
    });

    assert!(matches!(outcome, Err(CredentialError::Remote(_))));
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1, "PAT 不換發、不重試");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 0);
    assert_eq!(
        store.get(&server.base, CredentialKind::Pat).unwrap(),
        Some("the-pat".to_string()),
        "PAT 不因單次拒絕被清除"
    );
}

/// Only 401 buys a rotation. A 403 means the identity is right and the
/// permission is not — rotating would just burn a credential.
#[test]
fn a_403_does_not_trigger_a_rotation() {
    let server = stub(9);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Refresh, "seed").unwrap();
    seed_live_bearer(&store, &server.base, "live-access");

    let attempts = server.attempts.clone();
    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |_| {
        attempts.fetch_add(1, Ordering::SeqCst);
        Err::<(), _>(speclink_remote::RemoteError {
            message: "access denied".into(),
            reason: Some("permission_denied".into()),
            status: Some(403),
        })
    });

    assert!(matches!(outcome, Err(CredentialError::Remote(_))));
    assert_eq!(server.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(server.rotations.load(Ordering::SeqCst), 0);
}

#[test]
fn no_credential_anywhere_is_not_logged_in() {
    let dir = tempdir();
    let store = MemoryCredentialStore::new();

    let outcome = auth::with_credential(dir.path(), "http://127.0.0.1:1", &store, None, |_| {
        Ok::<_, speclink_remote::RemoteError>(())
    });

    assert!(matches!(outcome, Err(CredentialError::NotLoggedIn)));
}

/// The success path costs nothing extra: one attempt, no rotation.
#[test]
fn a_successful_attempt_does_not_rotate() {
    let server = stub(9);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Refresh, "seed").unwrap();
    seed_live_bearer(&store, &server.base, "live-access");

    let outcome = auth::with_credential(dir.path(), &server.base, &store, None, |bearer| {
        Ok::<_, speclink_remote::RemoteError>(bearer.to_string())
    })
    .expect("成功路徑");

    assert_eq!(outcome.value, "live-access");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 0);
}

fn seed_live_bearer(store: &dyn CredentialStore, origin: &str, token: &str) {
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 3600;
    let raw = format!(r#"{{"token":"{token}","expiresAt":{expires_at}}}"#);
    store.set(origin, CredentialKind::Bearer, &raw).unwrap();
}

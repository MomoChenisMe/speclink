//! Rotation is serialized across processes (cli-desktop-credential-sharing:
//! 「檔案鎖序列化跨行程換發」): the desktop app and any number of CLI
//! processes share ONE credential family, and the server spends a refresh
//! credential on use — two concurrent rotations would trip its reuse
//! detection and tear the whole family down, logging both sides out.
//!
//! The stub server counts `/auth/refresh` hits, which is the assertion that
//! matters: concurrent callers must produce exactly one rotation, with the
//! late arrivals reusing what the leader cached.

use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_remote::refresh;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A stub server counting rotations. Each rotation mints a fresh pair so a
/// second rotation is observable both by the counter and by the stored value.
struct RotationServer {
    base: String,
    rotations: Arc<AtomicUsize>,
}

fn rotation_server(delay_ms: u64) -> RotationServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", server.server_addr());
    let rotations = Arc::new(AtomicUsize::new(0));
    let counter = rotations.clone();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
            // Hold the request open so a racing caller has a real window to
            // arrive while the leader is mid-rotation.
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            let body = format!(
                r#"{{"accessToken":"access-{n}","refreshToken":"refresh-{n}","expiresIn":3600}}"#
            );
            let resp = tiny_http::Response::from_string(body).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("header"),
            );
            let _ = req.respond(resp);
        }
    });
    RotationServer { base, rotations }
}

#[test]
fn concurrent_callers_rotate_exactly_once() {
    let server = rotation_server(120);
    let dir = tempfile::tempdir().expect("tempdir");
    let store: Arc<MemoryCredentialStore> = Arc::new(MemoryCredentialStore::new());
    store
        .set(&server.base, CredentialKind::Refresh, "refresh-seed")
        .expect("seed refresh");

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let origin = server.base.clone();
            let store = store.clone();
            let lock_dir = dir.path().to_path_buf();
            std::thread::spawn(move || {
                refresh::bearer_for(&origin, store.as_ref(), &lock_dir)
                    .expect("每個併發呼叫都要拿到可用 bearer")
            })
        })
        .collect();

    let bearers: Vec<String> = handles.into_iter().map(|h| h.join().expect("join")).collect();

    assert_eq!(
        server.rotations.load(Ordering::SeqCst),
        1,
        "併發換發必須恰打一次 server，否則觸發 reuse 偵測整族撤銷"
    );
    for bearer in &bearers {
        assert_eq!(bearer, &bearers[0], "後到者應複用先行者換得的 bearer");
    }
    assert_eq!(
        store.get(&server.base, CredentialKind::Refresh).unwrap(),
        Some("refresh-1".to_string()),
        "rotation 後的新 refresh credential 必須回寫"
    );
}

#[test]
fn rotation_writes_back_the_new_refresh_credential() {
    let server = rotation_server(0);
    let dir = tempfile::tempdir().expect("tempdir");
    let store = MemoryCredentialStore::new();
    store
        .set(&server.base, CredentialKind::Refresh, "refresh-seed")
        .expect("seed refresh");

    let first = refresh::rotate(&server.base, &store, dir.path()).expect("first rotation");
    assert_eq!(first, "access-1");
    assert_eq!(
        store.get(&server.base, CredentialKind::Refresh).unwrap(),
        Some("refresh-1".to_string())
    );

    // A forced rotation spends the freshly stored credential, never the seed.
    let second = refresh::rotate(&server.base, &store, dir.path()).expect("second rotation");
    assert_eq!(second, "access-2");
    assert_eq!(
        store.get(&server.base, CredentialKind::Refresh).unwrap(),
        Some("refresh-2".to_string())
    );
}

/// The cache is what keeps a verb-per-invocation CLI from spending a
/// rotation on every command.
#[test]
fn a_live_cached_bearer_costs_no_rotation() {
    let server = rotation_server(0);
    let dir = tempfile::tempdir().expect("tempdir");
    let store = MemoryCredentialStore::new();
    store
        .set(&server.base, CredentialKind::Refresh, "refresh-seed")
        .expect("seed refresh");

    // First call rotates and caches.
    let first = refresh::bearer_for(&server.base, &store, dir.path()).expect("first bearer");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 1);

    // Every later call inside the token's life is served from the cache.
    for _ in 0..5 {
        let again = refresh::bearer_for(&server.base, &store, dir.path()).expect("cached bearer");
        assert_eq!(again, first);
    }
    assert_eq!(
        server.rotations.load(Ordering::SeqCst),
        1,
        "快取未到期時不得再打換發端點"
    );
}

/// A corrupt cache entry must degrade to a rotation, never to a hard failure.
#[test]
fn an_unparseable_cache_entry_falls_through_to_rotation() {
    let server = rotation_server(0);
    let dir = tempfile::tempdir().expect("tempdir");
    let store = MemoryCredentialStore::new();
    store
        .set(&server.base, CredentialKind::Refresh, "refresh-seed")
        .expect("seed refresh");
    store
        .set(&server.base, CredentialKind::Bearer, "{not json")
        .expect("seed corrupt cache");

    let bearer = refresh::bearer_for(&server.base, &store, dir.path()).expect("bearer");

    assert_eq!(bearer, "access-1");
    assert_eq!(server.rotations.load(Ordering::SeqCst), 1);
}

/// The lock is held under a bounded wait, not indefinitely: a wedged holder
/// should produce an error naming the problem, never a command that hangs with
/// no output.
#[test]
fn a_held_lock_does_not_block_forever() {
    let dir = tempfile::tempdir().expect("tempdir");
    let outcome = refresh::with_rotation_lock(dir.path(), || {
        // Re-entering from the same thread must not deadlock the test either;
        // this asserts on the nested attempt, which is what a second process
        // would experience.
        Ok(std::thread::scope(|s| {
            s.spawn(|| {
                refresh::with_rotation_lock_for(
                    dir.path(),
                    std::time::Duration::from_millis(150),
                    || Ok(()),
                )
            })
            .join()
            .expect("nested thread")
        }))
    })
    .expect("outer lock");

    assert!(
        outcome.is_err(),
        "鎖被持有時，等待者必須逾時回錯誤而非無限阻塞"
    );
    assert!(
        outcome.unwrap_err().contains("逾時"),
        "訊息需說明是等鎖逾時"
    );
}

#[test]
fn missing_refresh_credential_is_rejected_not_unavailable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = MemoryCredentialStore::new();

    match refresh::bearer_for("http://127.0.0.1:1", &store, dir.path()) {
        Err(refresh::RefreshFailure::Rejected(_)) => {}
        other => panic!("本機沒有 refresh credential 應為 Rejected（要求重登入），得到 {other:?}"),
    }
}

#[test]
fn transport_failure_keeps_the_credential() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = MemoryCredentialStore::new();
    // Port 1 on loopback refuses instantly — a transport failure, not a refusal.
    store
        .set("http://127.0.0.1:1", CredentialKind::Refresh, "refresh-seed")
        .expect("seed refresh");

    match refresh::bearer_for("http://127.0.0.1:1", &store, dir.path()) {
        Err(refresh::RefreshFailure::Unavailable(_)) => {}
        other => panic!("網路不可達應為 Unavailable，得到 {other:?}"),
    }
    assert_eq!(
        store.get("http://127.0.0.1:1", CredentialKind::Refresh).unwrap(),
        Some("refresh-seed".to_string()),
        "暫時性失敗不得清除 credential"
    );
}

//! The four-layer credential resolution ladder
//! (cli-desktop-credential-sharing:「憑證解析階梯」).
//!
//! Order: SPECLINK_TOKEN → keyring refresh (via cache/rotation) → keyring PAT
//! → credentials.yaml PAT. A layer that is *unavailable* — no keyring service,
//! access denied, no entry — falls through silently, because a headless CI box
//! and a macOS user who clicked Deny must both keep working exactly as before.
//!
//! Falling through happens during resolution only. Once a credential is
//! chosen, a server rejection inside that same verb never silently reaches for
//! another layer (the canonical `remote-auth` contract).
//!
//! Every test injects an in-memory store: CI has no keyring, and a test that
//! touched the real one would be unrunnable there.

use speclink_remote::auth::{self, CredentialSource};
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};

const ORIGIN: &str = "https://team.example.com";

fn tempdir(tag: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("speclink-ladder-{tag}-"))
        .tempdir()
        .expect("tempdir")
}

/// A store whose every operation fails — a machine with no Secret Service, or
/// a user who denied keychain access.
struct UnavailableStore;

impl CredentialStore for UnavailableStore {
    fn get(&self, _: &str, _: CredentialKind) -> Result<Option<String>, String> {
        Err("no keyring service on this platform".into())
    }
    fn set(&self, _: &str, _: CredentialKind, _: &str) -> Result<(), String> {
        Err("no keyring service on this platform".into())
    }
    fn delete(&self, _: &str, _: CredentialKind) -> Result<(), String> {
        Err("no keyring service on this platform".into())
    }
}

#[test]
fn env_token_wins_over_every_other_layer() {
    let dir = tempdir("env-wins");
    let store = MemoryCredentialStore::new();
    store.set(ORIGIN, CredentialKind::Pat, "keyring-pat").unwrap();
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();

    let resolved = auth::resolve_credential_at(
        dir.path(),
        ORIGIN,
        &store,
        Some("env-token".to_string()),
    )
    .expect("解析不應失敗")
    .expect("應解析到憑證");

    assert_eq!(resolved.token, "env-token");
    assert_eq!(resolved.source, CredentialSource::Env);
}

/// An empty environment value counts as unset — it must never silently
/// disable the other layers.
#[test]
fn blank_env_token_does_not_shadow_the_file() {
    let dir = tempdir("blank-env");
    let store = MemoryCredentialStore::new();
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();

    let resolved =
        auth::resolve_credential_at(dir.path(), ORIGIN, &store, Some("   ".to_string()))
            .expect("解析不應失敗")
            .expect("空白環境變數應視為未設定");

    assert_eq!(resolved.token, "file-pat");
    assert_eq!(resolved.source, CredentialSource::CredentialsFile);
}

#[test]
fn keyring_pat_beats_the_credentials_file() {
    let dir = tempdir("keyring-pat");
    let store = MemoryCredentialStore::new();
    store.set(ORIGIN, CredentialKind::Pat, "keyring-pat").unwrap();
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();

    let resolved =
        auth::resolve_credential_at(dir.path(), ORIGIN, &store, None).expect("解析不應失敗")
    .expect("應解析到憑證");

    assert_eq!(resolved.token, "keyring-pat");
    assert_eq!(resolved.source, CredentialSource::KeychainPat);
}

/// The whole point of the change: a desktop login (a refresh credential in the
/// keyring) is what the CLI picks up, ahead of any PAT.
#[test]
fn keyring_refresh_beats_both_pat_layers() {
    let dir = tempdir("refresh-first");
    let store = MemoryCredentialStore::new();
    store.set(ORIGIN, CredentialKind::Refresh, "the-refresh").unwrap();
    store.set(ORIGIN, CredentialKind::Pat, "keyring-pat").unwrap();
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();
    // A live cached access token so resolution needs no network.
    seed_live_bearer(&store, ORIGIN, "cached-access");

    let resolved =
        auth::resolve_credential_at(dir.path(), ORIGIN, &store, None).expect("解析不應失敗")
    .expect("應解析到憑證");

    assert_eq!(resolved.token, "cached-access");
    assert_eq!(resolved.source, CredentialSource::KeychainRefresh);
}

#[test]
fn credentials_file_is_the_last_layer() {
    let dir = tempdir("file-last");
    let store = MemoryCredentialStore::new();
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();

    let resolved =
        auth::resolve_credential_at(dir.path(), ORIGIN, &store, None).expect("解析不應失敗")
    .expect("應解析到憑證");

    assert_eq!(resolved.token, "file-pat");
    assert_eq!(resolved.source, CredentialSource::CredentialsFile);
}

/// The zero-regression guarantee for headless CI: no keyring at all still
/// resolves the credentials file, with no error surfacing.
#[test]
fn an_unavailable_keyring_falls_through_to_the_file() {
    let dir = tempdir("no-keyring");
    auth::save_token_at(dir.path(), ORIGIN, "file-pat").unwrap();

    let resolved = auth::resolve_credential_at(dir.path(), ORIGIN, &UnavailableStore, None)
        .expect("金鑰圈不可用不得使解析失敗")
        .expect("應回退到憑證檔");

    assert_eq!(resolved.token, "file-pat");
    assert_eq!(resolved.source, CredentialSource::CredentialsFile);
}

#[test]
fn all_four_layers_empty_resolves_to_nothing() {
    let dir = tempdir("empty");
    let store = MemoryCredentialStore::new();

    assert!(auth::resolve_credential_at(dir.path(), ORIGIN, &store, None)
        .expect("解析不應失敗")
        .is_none());
}

/// Credentials are keyed by origin so one login covers every project on a
/// server — and never leaks across servers.
#[test]
fn resolution_is_scoped_to_the_origin() {
    let dir = tempdir("origin-scope");
    let store = MemoryCredentialStore::new();
    store.set("https://other.example.com", CredentialKind::Pat, "other").unwrap();

    assert!(auth::resolve_credential_at(dir.path(), ORIGIN, &store, None)
        .expect("解析不應失敗")
        .is_none());
}

/// A dead refresh family fails the verb; it does NOT quietly continue as
/// whoever the credentials file belongs to. Swapping identity mid-command is
/// exactly what the canonical contract forbids — the next invocation is where
/// the remaining layers get their turn.
#[test]
fn a_revoked_refresh_family_fails_instead_of_falling_through() {
    let dir = tempdir("revoked-no-fallthrough");
    let origin = refusing_server();
    let store = MemoryCredentialStore::new();
    store.set(&origin, CredentialKind::Refresh, "dead-refresh").unwrap();
    store.set(&origin, CredentialKind::Pat, "keyring-pat").unwrap();
    auth::save_token_at(dir.path(), &origin, "file-pat").unwrap();

    let outcome = auth::resolve_credential_at(dir.path(), &origin, &store, None);

    assert!(
        outcome.is_err(),
        "family 已撤銷時該次執行不得改用其他憑證層"
    );
    assert_eq!(
        store.get(&origin, CredentialKind::Refresh).unwrap(),
        None,
        "撤銷的 refresh credential 必須被清除"
    );
    assert_eq!(
        store.get(&origin, CredentialKind::Bearer).unwrap(),
        None,
        "隨之失效的 access token 快取必須被清除"
    );
    // The PAT layers are untouched — they are separate logins.
    assert_eq!(
        store.get(&origin, CredentialKind::Pat).unwrap(),
        Some("keyring-pat".to_string())
    );

    // The next invocation resolves to what remains.
    let next = auth::resolve_credential_at(dir.path(), &origin, &store, None)
        .expect("清除後解析不應失敗")
        .expect("應解析到金鑰圈 PAT");
    assert_eq!(next.token, "keyring-pat");
    assert_eq!(next.source, CredentialSource::KeychainPat);
}

/// A server that refuses every rotation with `permission_denied` — the shape
/// of a family the server has torn down. Returns its base URL.
fn refusing_server() -> String {
    let server = std::sync::Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", server.server_addr());
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let body = r#"{"status":401,"reason":"permission_denied","message":"refresh credential is not live"}"#;
            let resp = tiny_http::Response::from_string(body)
                .with_status_code(401)
                .with_header(
                    tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"application/json"[..],
                    )
                    .expect("header"),
                );
            let _ = req.respond(resp);
        }
    });
    base
}

#[test]
fn credential_source_labels_are_stable_json_values() {
    // The --json contract: these strings are what `credentialSource` carries.
    assert_eq!(CredentialSource::Env.as_str(), "env");
    assert_eq!(CredentialSource::KeychainRefresh.as_str(), "keychain_refresh");
    assert_eq!(CredentialSource::KeychainPat.as_str(), "keychain_pat");
    assert_eq!(CredentialSource::CredentialsFile.as_str(), "credentials_file");
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

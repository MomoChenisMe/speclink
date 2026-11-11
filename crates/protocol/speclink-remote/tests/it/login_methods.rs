//! Login-method discovery and choice (client-browser-login D1, D2；規格
//!「登入方式的探索與選擇」), plus origin normalization.
//!
//! The order a client picks in is its own — browser, device, token — never the
//! order the server happened to list. A server without the route (404/405) has
//! declared nothing and keeps the pre-discovery behavior; any other failure is
//! an error, never a guess.

use speclink_remote::login::{choose, discover, normalize_origin, ChooseError, Discovery, Method, Plan};
use std::sync::{Arc, Mutex};

/// What the fake server saw: path and whether an Authorization header came.
#[derive(Debug, Clone)]
struct Seen {
    path: String,
    authorized: bool,
    versioned: bool,
}

/// A server answering every request with `status` and `body`.
fn server(status: u16, body: &'static str) -> (String, Arc<Mutex<Vec<Seen>>>) {
    let http = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", http.server_addr());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for req in http.incoming_requests() {
            let has = |name: &'static str| req.headers().iter().any(|h| h.field.equiv(name));
            log.lock().unwrap().push(Seen {
                path: req.url().to_string(),
                authorized: has("Authorization"),
                versioned: has("X-Speclink-Api-Version"),
            });
            let resp = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .expect("header"),
                );
            let _ = req.respond(resp);
        }
    });
    (base, seen)
}

fn declared(body: &'static str) -> Discovery {
    let (base, _) = server(200, body);
    discover(&base).expect("a 200 declaration is a discovery, not an error")
}

const BROWSER_AND_DEVICE: &str = r#"{"methods":["device","browser"],"browser":{"issuer":"https://id.example.com","clientId":"speclink","scopes":["openid"]}}"#;

#[test]
fn discovery_asks_the_api_root_without_credentials() {
    let (base, seen) = server(200, r#"{"methods":["token"]}"#);

    discover(&base).expect("discover");

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].path, "/api/speclink/v1/auth-methods");
    assert!(!seen[0].authorized, "discovery happens before any credential exists");
    assert!(!seen[0].versioned, "discovery needs no version header");
}

/// The client's order wins: the server listing device first does not matter.
#[test]
fn browser_and_device_declared_choose_browser_regardless_of_listing_order() {
    let discovery = declared(BROWSER_AND_DEVICE);

    match choose(&discovery, None).expect("choose") {
        Plan::Browser(browser) => {
            assert_eq!(browser.issuer, "https://id.example.com");
            assert_eq!(browser.client_id, "speclink");
            assert_eq!(browser.scopes, vec!["openid".to_string()]);
        }
        other => panic!("應選瀏覽器登入，得到 {other:?}"),
    }
}

#[test]
fn only_token_declared_asks_for_an_access_token() {
    let discovery = declared(r#"{"methods":["token"]}"#);

    assert!(matches!(choose(&discovery, None).expect("choose"), Plan::Token));
}

/// 404 and 405 mean "this server declares nothing" — the old behavior, not an
/// error: try the device flow first.
#[test]
fn a_404_or_405_is_undeclared_and_keeps_the_device_first_behavior() {
    for status in [404, 405] {
        let (base, _) = server(status, r#"{"status":404,"reason":"not_found","message":"no route"}"#);
        let discovery = discover(&base).expect("a missing route is an outcome, not an error");
        assert!(matches!(discovery, Discovery::Undeclared), "HTTP {status}");
        assert!(matches!(choose(&discovery, None).expect("choose"), Plan::Device), "HTTP {status}");
    }
}

#[test]
fn a_5xx_is_an_error_not_a_guess() {
    let (base, _) = server(500, "boom");
    assert!(discover(&base).is_err(), "a broken server must not read as undeclared");
}

#[test]
fn an_unreachable_server_is_an_error() {
    assert!(discover("http://127.0.0.1:1").is_err());
}

#[test]
fn an_unparseable_declaration_is_an_error() {
    let (base, _) = server(200, "<html>not json</html>");
    assert!(discover(&base).is_err());
}

#[test]
fn a_preferred_method_the_server_does_not_offer_lists_what_it_does() {
    let discovery = declared(r#"{"methods":["device"]}"#);

    match choose(&discovery, Some(Method::Browser)) {
        Err(ChooseError::NotOffered { wanted, supported }) => {
            assert_eq!(wanted, Method::Browser);
            assert_eq!(supported, vec![Method::Device]);
            let line = ChooseError::NotOffered { wanted, supported }.to_string();
            assert!(line.contains("browser") && line.contains("device"), "{line}");
        }
        other => panic!("應回報不支援，得到 {other:?}"),
    }
}

/// Without a declaration there is no issuer, so a browser login has nowhere to go.
#[test]
fn preferring_browser_on_an_undeclared_server_is_an_error() {
    let (base, _) = server(404, "{}");
    let discovery = discover(&base).expect("discover");

    let err = choose(&discovery, Some(Method::Browser)).expect_err("no issuer to log in at");
    assert!(matches!(err, ChooseError::BrowserUndeclared));
    assert!(err.to_string().contains("does not declare browser login"), "{err}");
}

/// A preference for a method that needs no declaration still works on an old server.
#[test]
fn preferring_device_on_an_undeclared_server_uses_the_device_flow() {
    let (base, _) = server(404, "{}");
    let discovery = discover(&base).expect("discover");

    assert!(matches!(choose(&discovery, Some(Method::Device)).expect("choose"), Plan::Device));
}

#[test]
fn a_preferred_method_the_server_offers_is_used() {
    let discovery = declared(BROWSER_AND_DEVICE);

    assert!(matches!(choose(&discovery, Some(Method::Device)).expect("choose"), Plan::Device));
}

#[test]
fn unknown_methods_are_skipped() {
    let discovery = declared(r#"{"methods":["passkey","device"]}"#);

    match &discovery {
        Discovery::Declared { methods, .. } => assert_eq!(methods, &vec![Method::Device]),
        other => panic!("應為 Declared，得到 {other:?}"),
    }
    assert!(matches!(choose(&discovery, None).expect("choose"), Plan::Device));
}

/// A browser entry without somewhere to send the user cannot be used; the rest
/// of the declaration still counts.
#[test]
fn a_browser_declaration_without_issuer_or_client_id_counts_as_no_browser() {
    for body in [
        r#"{"methods":["browser","device"]}"#,
        r#"{"methods":["browser","device"],"browser":{"issuer":"","clientId":"speclink"}}"#,
        r#"{"methods":["browser","device"],"browser":{"issuer":"https://id.example.com","clientId":" "}}"#,
    ] {
        let (base, _) = server(200, Box::leak(body.to_string().into_boxed_str()));
        let discovery = discover(&base).expect("discover");
        match &discovery {
            Discovery::Declared { methods, browser } => {
                assert_eq!(methods, &vec![Method::Device], "{body}");
                assert!(browser.is_none(), "{body}");
            }
            other => panic!("應為 Declared，得到 {other:?}"),
        }
        assert!(matches!(choose(&discovery, None).expect("choose"), Plan::Device), "{body}");
    }
}

/// A declaration listing nothing this client can do is a dead end, said plainly.
#[test]
fn a_declaration_with_no_usable_method_is_an_error() {
    let discovery = declared(r#"{"methods":["passkey"]}"#);

    assert!(matches!(choose(&discovery, None), Err(ChooseError::NoUsableMethod)));
}

#[test]
fn normalize_origin_lowercases_scheme_and_host_and_drops_the_path() {
    assert_eq!(
        normalize_origin("HTTPS://Team.Example.COM:8443/api/speclink/v1/projects/Foo?x=1#y").unwrap(),
        "https://team.example.com:8443"
    );
    assert_eq!(normalize_origin("  http://localhost:8080/  ").unwrap(), "http://localhost:8080");
}

#[test]
fn normalize_origin_accepts_only_http_and_https() {
    assert!(normalize_origin("ftp://files.example.com").is_err());
    assert!(normalize_origin("team.example.com").is_err(), "a scheme is required");
    assert!(normalize_origin("https:///path-only").is_err(), "a host is required");
}

// --- access-token login, logout and the identity lookup (D6, D7；規格
// 「PAT 登入與憑證儲存」「登出」) ---

use crate::browser_login::{idp, tempdir, IdpConfig, CLIENT_ID};
use speclink_remote::auth;
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore, OAuthRefresh, RefreshEntry};
use speclink_remote::device;
use speclink_remote::login::{logout, pat_login, LoginOutcome};

const GOOD_PAT: &str = "pat-good";

/// One request the fake server saw.
#[derive(Debug, Clone)]
struct Hit {
    path: String,
    auth: Option<String>,
    body: String,
}

type Handler = dyn Fn(&str, Option<&str>) -> (u16, String) + Send + Sync;

/// A fake speclink server: `handler(path, authorization)` decides each answer.
fn fake(handler: Box<Handler>) -> (String, Arc<Mutex<Vec<Hit>>>) {
    let http = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", http.server_addr());
    let hits = Arc::new(Mutex::new(Vec::new()));
    let log = hits.clone();
    std::thread::spawn(move || {
        for mut req in http.incoming_requests() {
            let auth = req
                .headers()
                .iter()
                .find(|h| h.field.equiv("Authorization"))
                .map(|h| h.value.as_str().to_string());
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
            let path = req.url().to_string();
            let (status, answer) = handler(&path, auth.as_deref());
            log.lock().unwrap().push(Hit { path, auth, body });
            let resp = tiny_http::Response::from_string(answer).with_status_code(status).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).expect("header"),
            );
            let _ = req.respond(resp);
        }
    });
    (base, hits)
}

const IDENTITY: &str = r#"{"user":{"name":"Dev <dev@example.com>","handle":"dev"}}"#;
const NO_ROUTE: &str = r#"{"status":404,"reason":"not_found","message":"no route"}"#;
const REFUSED: &str = r#"{"status":401,"reason":"permission_denied","message":"bad token"}"#;

/// Today's official server: no API-root whoami, `/auth/whoami` and `/auth/revoke` only.
fn official_server() -> (String, Arc<Mutex<Vec<Hit>>>) {
    fake(Box::new(|path, auth| match path {
        "/auth/whoami" if auth == Some(&format!("Bearer {GOOD_PAT}")[..]) => (200, IDENTITY.into()),
        "/auth/whoami" => (401, REFUSED.into()),
        "/auth/revoke" => (200, "{}".into()),
        _ => (404, NO_ROUTE.into()),
    }))
}

fn paths(hits: &Arc<Mutex<Vec<Hit>>>) -> Vec<String> {
    hits.lock().unwrap().iter().map(|h| h.path.clone()).collect()
}

fn assert_nothing_stored(store: &MemoryCredentialStore, dir: &std::path::Path, origin: &str) {
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
        assert_eq!(store.get(origin, kind).unwrap(), None, "{kind:?}");
    }
    assert_eq!(auth::load_token_at(dir, origin), None, "credentials file entry");
}

#[test]
fn the_identity_asks_the_api_root_first() {
    let (base, hits) = fake(Box::new(|path, _| match path {
        "/api/speclink/v1/whoami" => (200, IDENTITY.into()),
        _ => (404, NO_ROUTE.into()),
    }));

    let who = device::identity(&base, "any").expect("identity");

    assert_eq!(who.user.name, "Dev <dev@example.com>");
    assert_eq!(paths(&hits), vec!["/api/speclink/v1/whoami".to_string()]);
    assert_eq!(hits.lock().unwrap()[0].auth.as_deref(), Some("Bearer any"));
}

#[test]
fn the_identity_falls_back_to_the_old_route_on_a_404() {
    let (base, hits) = official_server();

    let who = device::identity(&base, GOOD_PAT).expect("identity");

    assert_eq!(who.user.handle, "dev");
    assert_eq!(
        paths(&hits),
        vec!["/api/speclink/v1/whoami".to_string(), "/auth/whoami".to_string()]
    );
}

#[test]
fn a_pat_is_validated_then_stored_in_the_keyring_replacing_other_logins() {
    let (base, hits) = official_server();
    let store = MemoryCredentialStore::new();
    store.set(&base, CredentialKind::Refresh, "old-refresh").unwrap();
    store.set(&base, CredentialKind::Bearer, "{}").unwrap();
    let dir = tempdir();

    let outcome = pat_login(&base, GOOD_PAT, &store, dir.path(), Some(dir.path()));

    assert_eq!(
        outcome,
        LoginOutcome::LoggedIn { display: "Dev <dev@example.com>".into(), handle: "dev".into() }
    );
    assert!(paths(&hits).iter().any(|p| p == "/auth/whoami"), "validated before storing");
    assert_eq!(store.get(&base, CredentialKind::Pat).unwrap(), Some(GOOD_PAT.to_string()));
    assert_eq!(store.get(&base, CredentialKind::Refresh).unwrap(), None, "one login per origin");
    assert_eq!(store.get(&base, CredentialKind::Bearer).unwrap(), None);
    assert!(!auth::credentials_path_in(dir.path()).exists(), "憑證檔不變");
}

struct NoKeyring;
impl CredentialStore for NoKeyring {
    fn get(&self, _: &str, _: CredentialKind) -> Result<Option<String>, String> {
        Err("no keyring service on this platform".into())
    }
    fn set(&self, _: &str, _: CredentialKind, _: &str) -> Result<(), String> {
        Err("no keyring service on this platform".into())
    }
    fn delete(&self, _: &str, _: CredentialKind) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn without_a_keyring_a_pat_goes_to_the_credentials_file() {
    let (base, _) = official_server();
    let dir = tempdir();

    let outcome = pat_login(&base, GOOD_PAT, &NoKeyring, dir.path(), Some(dir.path()));

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
    assert_eq!(auth::load_token_at(dir.path(), &base), Some(GOOD_PAT.to_string()));
}

/// The desktop app keeps credentials in the keyring only: no fallback file.
#[test]
fn without_a_keyring_and_no_fallback_a_pat_is_refused() {
    let (base, _) = official_server();
    let dir = tempdir();

    let outcome = pat_login(&base, GOOD_PAT, &NoKeyring, dir.path(), None);

    assert_eq!(outcome, LoginOutcome::KeychainUnavailable);
    assert!(!auth::credentials_path_in(dir.path()).exists());
}

#[test]
fn an_invalid_pat_is_denied_and_stores_nothing() {
    let (base, _) = official_server();
    let store = MemoryCredentialStore::new();
    let dir = tempdir();

    let outcome = pat_login(&base, "pat-bad", &store, dir.path(), Some(dir.path()));

    assert_eq!(outcome, LoginOutcome::Denied);
    assert_nothing_stored(&store, dir.path(), &base);
}

#[test]
fn logout_revokes_a_device_family_and_clears_every_local_credential() {
    let (base, hits) = official_server();
    let store = MemoryCredentialStore::new();
    store.set(&base, CredentialKind::Refresh, "spk_rt_live").unwrap();
    store.set(&base, CredentialKind::Bearer, "{}").unwrap();
    store.set(&base, CredentialKind::Pat, "keyring-pat").unwrap();
    let dir = tempdir();
    auth::save_token_at(dir.path(), &base, "file-pat").unwrap();

    let outcome = logout(&base, &store, Some(dir.path())).expect("logout");

    assert!(outcome.was_logged_in);
    assert!(outcome.revoked && outcome.revoke_warning.is_none(), "{outcome:?}");
    assert!(outcome.removed_pat);
    let revoke = hits.lock().unwrap().iter().find(|h| h.path == "/auth/revoke").cloned().expect("revoked");
    assert!(revoke.body.contains("spk_rt_live"), "{}", revoke.body);
    assert_nothing_stored(&store, dir.path(), &base);
}

fn oauth_entry(token_endpoint: String, revocation: Option<String>) -> String {
    RefreshEntry::OAuth(OAuthRefresh {
        refresh_token: Some("rt-live".into()),
        token_endpoint,
        client_id: CLIENT_ID.into(),
        revocation_endpoint: revocation,
    })
    .encode()
}

#[test]
fn logout_revokes_a_browser_login_at_the_login_service() {
    let (base, hits) = official_server();
    let service = idp(IdpConfig::default());
    let store = MemoryCredentialStore::new();
    store
        .set(&base, CredentialKind::Refresh, &oauth_entry(service.token_endpoint(), Some(format!("{}/revoke", service.issuer))))
        .unwrap();
    store.set(&base, CredentialKind::Bearer, "{}").unwrap();

    let outcome = logout(&base, &store, None).expect("logout");

    assert!(outcome.revoked, "{outcome:?}");
    let revocations = service.revoke.lock().unwrap();
    assert_eq!(revocations.len(), 1);
    assert_eq!(revocations[0]["token"], "rt-live");
    assert_eq!(revocations[0]["token_type_hint"], "refresh_token");
    assert_eq!(revocations[0]["client_id"], CLIENT_ID);
    assert!(!paths(&hits).iter().any(|p| p == "/auth/revoke"), "a browser login is not a server family");
    assert_eq!(store.get(&base, CredentialKind::Refresh).unwrap(), None);
    assert_eq!(store.get(&base, CredentialKind::Bearer).unwrap(), None);
}

#[test]
fn logout_of_a_browser_login_without_a_revocation_endpoint_sends_nothing() {
    let (base, hits) = official_server();
    let service = idp(IdpConfig::default());
    let store = MemoryCredentialStore::new();
    store.set(&base, CredentialKind::Refresh, &oauth_entry(service.token_endpoint(), None)).unwrap();

    let outcome = logout(&base, &store, None).expect("logout");

    assert!(outcome.was_logged_in && !outcome.revoked && outcome.revoke_warning.is_none(), "{outcome:?}");
    assert!(service.revoke.lock().unwrap().is_empty());
    assert!(hits.lock().unwrap().is_empty());
    assert_eq!(store.get(&base, CredentialKind::Refresh).unwrap(), None);
}

/// A revocation that cannot reach anyone is a warning: the local cleanup still runs.
#[test]
fn a_failed_revocation_warns_and_still_clears_locally() {
    let store = MemoryCredentialStore::new();
    let origin = "http://127.0.0.1:1";
    store.set(origin, CredentialKind::Refresh, "spk_rt_live").unwrap();
    store.set(origin, CredentialKind::Bearer, "{}").unwrap();
    let dir = tempdir();

    let outcome = logout(origin, &store, Some(dir.path())).expect("logout");

    assert!(!outcome.revoked);
    let warning = outcome.revoke_warning.expect("a warning");
    assert!(warning.contains("could not revoke"), "{warning}");
    assert_nothing_stored(&store, dir.path(), origin);
}

#[test]
fn logout_with_nothing_stored_reports_not_logged_in() {
    let store = MemoryCredentialStore::new();
    let dir = tempdir();

    let outcome = logout("http://127.0.0.1:1", &store, Some(dir.path())).expect("logout");

    assert!(!outcome.was_logged_in);
}

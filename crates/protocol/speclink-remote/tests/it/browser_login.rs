//! Browser login (client-browser-login D3, D4；規格「瀏覽器登入」).
//!
//! A fake login service publishes its metadata document, an authorization
//! endpoint and a token endpoint; a fake speclink server answers whoami. The
//! test's "browser" asks the authorization endpoint where to go and then hits
//! the loopback callback, the way a real browser follows the redirect. Time is
//! the injected `sleep_secs`, so the 300-second window passes in about a second.
//!
//! What the tests pin: the authorization request carries PKCE S256, a state and
//! a 127.0.0.1 redirect; the token request carries the verifier and no client
//! secret; a callback with the wrong state is refused without ending the login;
//! every failure leaves the keyring as it was; and the page the browser gets
//! back never carries the code or a token.

use base64::Engine as _;
use sha2::Digest as _;
use speclink_protocol::auth::BrowserLogin;
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_remote::login::{self, LoginIo, LoginOutcome, LoginPhase, LoginTarget, Plan};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub(crate) const DISPLAY: &str = "Dev <dev@example.com>";
pub(crate) const CLIENT_ID: &str = "speclink-cli";
const AUTH_CODE: &str = "authcode-1";

// --- query and form helpers ---

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
                out.push(u8::from_str_radix(hex, 16).unwrap());
                i += 2;
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8(out).unwrap()
}

pub(crate) fn pairs(s: &str) -> HashMap<String, String> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn query_of(url: &str) -> HashMap<String, String> {
    url.split_once('?').map(|(_, q)| pairs(q)).unwrap_or_default()
}

fn json_response(code: u16, body: String) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_string(body)
        .with_status_code(code)
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                .expect("header"),
        )
}

// --- the fake speclink server (whoami only) ---

pub(crate) struct Server {
    pub(crate) origin: String,
    pub(crate) bearers: Arc<Mutex<Vec<String>>>,
}

pub(crate) fn speclink_server() -> Server {
    let http = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let origin = format!("http://{}", http.server_addr());
    let bearers = Arc::new(Mutex::new(Vec::new()));
    let log = bearers.clone();
    std::thread::spawn(move || {
        for req in http.incoming_requests() {
            let auth = req
                .headers()
                .iter()
                .find(|h| h.field.equiv("Authorization"))
                .map(|h| h.value.as_str().to_string());
            let resp = match (req.url(), auth) {
                ("/api/speclink/v1/whoami", Some(auth)) => {
                    log.lock().unwrap().push(auth);
                    json_response(200, format!(r#"{{"user":{{"name":"{DISPLAY}","handle":"dev"}}}}"#))
                }
                _ => json_response(404, r#"{"status":404,"reason":"not_found","message":"no route"}"#.into()),
            };
            let _ = req.respond(resp);
        }
    });
    Server { origin, bearers }
}

// --- the fake login service ---

#[derive(Clone)]
pub(crate) struct IdpConfig {
    /// The issuer the metadata document claims, when it differs from the real one.
    pub(crate) claimed_issuer: Option<String>,
    /// The RFC 8414 location answers 404; only the OpenID Connect one exists.
    pub(crate) oidc_only: bool,
    /// `code_challenge_methods_supported`; `None` leaves the field out.
    pub(crate) pkce_methods: Option<Vec<&'static str>>,
    /// The authorization endpoint sends the user back with `access_denied`.
    pub(crate) deny: bool,
    /// The token endpoint issues a refresh token.
    pub(crate) issue_refresh: bool,
    pub(crate) expires_in: Option<u64>,
    pub(crate) revocation: bool,
    /// How the token endpoint answers a refresh grant.
    pub(crate) refresh_reply: RefreshReply,
    /// Hold each refresh grant open this long, so racing callers overlap.
    pub(crate) refresh_delay_ms: u64,
    /// The token endpoint refuses the authorization code.
    pub(crate) refuse_exchange: bool,
    /// Raised when the token endpoint gets the authorization code: the user
    /// cancels while the code is being exchanged.
    pub(crate) cancel_on_exchange: Option<Arc<AtomicBool>>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RefreshReply {
    Rotate,
    KeepRefresh,
    InvalidGrant,
    ServerError,
}

impl Default for IdpConfig {
    fn default() -> Self {
        IdpConfig {
            claimed_issuer: None,
            oidc_only: false,
            pkce_methods: Some(vec!["S256"]),
            deny: false,
            issue_refresh: true,
            expires_in: Some(3600),
            revocation: true,
            refresh_reply: RefreshReply::Rotate,
            refresh_delay_ms: 0,
            refuse_exchange: false,
            cancel_on_exchange: None,
        }
    }
}

pub(crate) struct Idp {
    pub(crate) issuer: String,
    pub(crate) authorize: Arc<Mutex<Vec<HashMap<String, String>>>>,
    pub(crate) token: Arc<Mutex<Vec<HashMap<String, String>>>>,
    pub(crate) revoke: Arc<Mutex<Vec<HashMap<String, String>>>>,
}

impl Idp {
    pub(crate) fn browser_login(&self) -> BrowserLogin {
        BrowserLogin {
            issuer: self.issuer.clone(),
            client_id: CLIENT_ID.to_string(),
            scopes: vec!["openid".into(), "offline_access".into()],
        }
    }

    pub(crate) fn token_endpoint(&self) -> String {
        format!("{}/token", self.issuer)
    }
}

pub(crate) fn idp(config: IdpConfig) -> Idp {
    let http = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let root = format!("http://{}", http.server_addr());
    let issuer = format!("{root}/realms/dev");
    let authorize = Arc::new(Mutex::new(Vec::new()));
    let token = Arc::new(Mutex::new(Vec::new()));
    let revoke = Arc::new(Mutex::new(Vec::new()));
    let (seen_authorize, seen_token, seen_revoke) = (authorize.clone(), token.clone(), revoke.clone());
    let issuer_for_thread = issuer.clone();
    std::thread::spawn(move || {
        let issuer = issuer_for_thread;
        let challenge = Mutex::new(String::new());
        let refreshes = Mutex::new(0u32);
        let metadata = || {
            let mut doc = serde_json::json!({
                "issuer": config.claimed_issuer.clone().unwrap_or_else(|| issuer.clone()),
                "authorization_endpoint": format!("{issuer}/authorize"),
                "token_endpoint": format!("{issuer}/token"),
            });
            if let Some(methods) = &config.pkce_methods {
                doc["code_challenge_methods_supported"] = serde_json::json!(methods);
            }
            if config.revocation {
                doc["revocation_endpoint"] = serde_json::json!(format!("{issuer}/revoke"));
            }
            json_response(200, doc.to_string())
        };
        for mut req in http.incoming_requests() {
            let url = req.url().to_string();
            let path = url.split('?').next().unwrap_or_default().to_string();
            let mut body = String::new();
            let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
            let resp = if path == "/.well-known/oauth-authorization-server/realms/dev" && !config.oidc_only {
                metadata()
            } else if path == "/realms/dev/.well-known/openid-configuration" {
                metadata()
            } else if path == "/realms/dev/authorize" {
                let q = query_of(&url);
                *challenge.lock().unwrap() = q.get("code_challenge").cloned().unwrap_or_default();
                seen_authorize.lock().unwrap().push(q.clone());
                let redirect = q.get("redirect_uri").cloned().unwrap_or_default();
                let state = q.get("state").cloned().unwrap_or_default();
                // The redirect target as the body: the test browser follows it.
                let target = if config.deny {
                    format!("{redirect}?error=access_denied&state={state}")
                } else {
                    format!("{redirect}?code={AUTH_CODE}&state={state}")
                };
                tiny_http::Response::from_string(target).with_status_code(200)
            } else if path == "/realms/dev/token" {
                let form = pairs(&body);
                seen_token.lock().unwrap().push(form.clone());
                let refresh_field = |rt: &str| {
                    if config.issue_refresh { format!(r#","refresh_token":"{rt}""#) } else { String::new() }
                };
                let expires = config.expires_in.map(|s| format!(r#","expires_in":{s}"#)).unwrap_or_default();
                match form.get("grant_type").map(String::as_str) {
                    Some("authorization_code") => {
                        if let Some(cancel) = &config.cancel_on_exchange {
                            cancel.store(true, Ordering::SeqCst);
                        }
                        let verifier = form.get("code_verifier").cloned().unwrap_or_default();
                        let hashed = base64::engine::general_purpose::URL_SAFE_NO_PAD
                            .encode(sha2::Sha256::digest(verifier.as_bytes()));
                        if config.refuse_exchange
                            || form.get("code").map(String::as_str) != Some(AUTH_CODE)
                            || hashed != *challenge.lock().unwrap()
                        {
                            json_response(400, r#"{"error":"invalid_grant"}"#.into())
                        } else {
                            json_response(
                                200,
                                format!(r#"{{"access_token":"at-1","token_type":"Bearer"{expires}{}}}"#, refresh_field("rt-1")),
                            )
                        }
                    }
                    Some("refresh_token") => {
                        std::thread::sleep(std::time::Duration::from_millis(config.refresh_delay_ms));
                        match config.refresh_reply {
                            RefreshReply::InvalidGrant => json_response(400, r#"{"error":"invalid_grant"}"#.into()),
                            RefreshReply::ServerError => json_response(503, "{}".into()),
                            reply => {
                                let mut n = refreshes.lock().unwrap();
                                *n += 1;
                                let rotated = if reply == RefreshReply::Rotate {
                                    format!(r#","refresh_token":"rt-{}""#, *n + 1)
                                } else {
                                    String::new()
                                };
                                json_response(
                                    200,
                                    format!(r#"{{"access_token":"at-{}","token_type":"Bearer","expires_in":3600{rotated}}}"#, *n + 1),
                                )
                            }
                        }
                    }
                    _ => json_response(400, r#"{"error":"unsupported_grant_type"}"#.into()),
                }
            } else if path == "/realms/dev/revoke" {
                seen_revoke.lock().unwrap().push(pairs(&body));
                tiny_http::Response::from_string("").with_status_code(200)
            } else {
                tiny_http::Response::from_string("no route").with_status_code(404)
            };
            let _ = req.respond(resp);
        }
    });
    Idp { issuer, authorize, token, revoke }
}

// --- the test's browser and terminal ---

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Browser {
    /// Nothing opens; nobody signs in.
    Absent,
    /// Follows the authorization endpoint back to the loopback callback.
    Follows,
    /// Hits the callback with a forged state first, then follows for real.
    ForgesStateFirst,
}

pub(crate) struct TestIo {
    browser: Browser,
    pub(crate) phases: Mutex<Vec<LoginPhase>>,
    pub(crate) opened: Mutex<Vec<String>>,
    /// What the loopback answered the browser, request by request.
    pages: Arc<Mutex<Vec<(u16, String)>>>,
    browsing: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// The user's cancel button.
    pub(crate) cancel: Arc<AtomicBool>,
}

pub(crate) fn target<'a>(origin: &'a str, store: &'a dyn CredentialStore, dir: &'a std::path::Path) -> LoginTarget<'a> {
    LoginTarget { origin, credentials: store, lock_dir: dir }
}

impl TestIo {
    pub(crate) fn new(browser: Browser) -> TestIo {
        TestIo {
            browser,
            phases: Mutex::new(Vec::new()),
            opened: Mutex::new(Vec::new()),
            pages: Arc::new(Mutex::new(Vec::new())),
            browsing: Mutex::new(None),
            cancel: Arc::default(),
        }
    }

    /// The pages the browser got, once it is done with them.
    pub(crate) fn pages(&self) -> Vec<(u16, String)> {
        if let Some(browsing) = self.browsing.lock().unwrap().take() {
            browsing.join().expect("the test browser panicked");
        }
        self.pages.lock().unwrap().clone()
    }
}

fn fetch(url: &str) -> (u16, String) {
    match ureq::get(url).call() {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => (0, e.to_string()),
    }
}

impl LoginIo for TestIo {
    fn phase(&self, phase: LoginPhase) {
        self.phases.lock().unwrap().push(phase);
    }

    fn open_browser(&self, url: &str) -> bool {
        self.opened.lock().unwrap().push(url.to_string());
        if self.browser == Browser::Absent {
            return false;
        }
        let (url, forge, pages) = (url.to_string(), self.browser == Browser::ForgesStateFirst, self.pages.clone());
        let browsing = std::thread::spawn(move || {
            let (_, target) = fetch(&url);
            if forge {
                let state = query_of(&target).get("state").cloned().unwrap_or_default();
                let forged = target.replace(&format!("state={state}"), "state=forged-state");
                let page = fetch(&forged);
                pages.lock().unwrap().push(page);
            }
            let page = fetch(&target);
            pages.lock().unwrap().push(page);
        });
        *self.browsing.lock().unwrap() = Some(browsing);
        true
    }

    fn sleep_secs(&self, _secs: u64) {
        // One "second" of the login's clock.
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

pub(crate) fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

pub(crate) fn browser_url(io: &TestIo) -> String {
    match io.phases.lock().unwrap().as_slice() {
        [LoginPhase::AwaitingBrowser { url, .. }] => url.clone(),
        other => panic!("應恰好一個等待瀏覽器登入的階段，得到 {other:?}"),
    }
}

fn assert_keyring_untouched(store: &MemoryCredentialStore, origin: &str) {
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
        assert_eq!(store.get(origin, kind).unwrap(), None, "{kind:?} 不應有新增條目");
    }
}

/// Log in through the browser against a fresh server and login service.
pub(crate) fn browser_login(
    config: IdpConfig,
    browser: Browser,
) -> (Server, Idp, MemoryCredentialStore, tempfile::TempDir, TestIo, LoginOutcome) {
    let server = speclink_server();
    let idp = idp(config);
    let store = MemoryCredentialStore::new();
    let dir = tempdir();
    let io = TestIo::new(browser);
    let outcome = login::login(&target(&server.origin, &store, dir.path()), &Plan::Browser(idp.browser_login()), &io);
    (server, idp, store, dir, io, outcome)
}

#[test]
fn a_full_browser_login_stores_the_oauth_credential_and_reports_the_identity() {
    let server = speclink_server();
    let idp = idp(IdpConfig::default());
    let store = MemoryCredentialStore::new();
    store.set(&server.origin, CredentialKind::Pat, "old-pat").unwrap();
    let dir = tempdir();
    let io = TestIo::new(Browser::Follows);

    let outcome = login::login(&target(&server.origin, &store, dir.path()), &Plan::Browser(idp.browser_login()), &io);

    match outcome {
        LoginOutcome::LoggedIn { display, .. } => assert_eq!(display, DISPLAY),
        other => panic!("應登入成功，得到 {other:?}"),
    }

    // The authorization request: PKCE S256, a state, a 127.0.0.1 callback.
    let url = browser_url(&io);
    assert_eq!(io.opened.lock().unwrap().as_slice(), &[url.clone()], "開的就是宣告的網址");
    assert!(url.starts_with(&format!("{}/authorize?", idp.issuer)), "{url}");
    let q = query_of(&url);
    assert_eq!(q["response_type"], "code");
    assert_eq!(q["client_id"], CLIENT_ID);
    assert_eq!(q["code_challenge_method"], "S256");
    assert_eq!(q["code_challenge"].len(), 43, "base64url of a SHA-256");
    assert!(q["state"].len() >= 43, "a state from 32 random bytes");
    assert_eq!(q["scope"], "openid offline_access");
    let redirect = &q["redirect_uri"];
    assert!(redirect.starts_with("http://127.0.0.1:") && redirect.ends_with("/callback"), "{redirect}");
    match &io.phases.lock().unwrap()[0] {
        LoginPhase::AwaitingBrowser { expires_in, .. } => assert_eq!(*expires_in, 300),
        other => panic!("{other:?}"),
    }

    // The token request: the verifier, no client secret.
    let forms = idp.token.lock().unwrap();
    assert_eq!(forms.len(), 1);
    let form = &forms[0];
    assert_eq!(form["grant_type"], "authorization_code");
    assert_eq!(form["code"], AUTH_CODE);
    assert_eq!(&form["redirect_uri"], redirect);
    assert_eq!(form["client_id"], CLIENT_ID);
    assert_eq!(form["code_verifier"].len(), 43);
    assert!(!form.contains_key("client_secret"), "a public client sends no secret: {form:?}");

    // The keyring: the D4 refresh entry, a cache with an expiry, no PAT left.
    let entry: serde_json::Value = serde_json::from_str(
        &store.get(&server.origin, CredentialKind::Refresh).unwrap().expect("refresh entry"),
    )
    .unwrap();
    assert_eq!(entry["kind"], "oauth");
    assert_eq!(entry["refreshToken"], "rt-1");
    assert_eq!(entry["tokenEndpoint"], idp.token_endpoint());
    assert_eq!(entry["clientId"], CLIENT_ID);
    assert_eq!(entry["revocationEndpoint"], format!("{}/revoke", idp.issuer));
    let cached: serde_json::Value = serde_json::from_str(
        &store.get(&server.origin, CredentialKind::Bearer).unwrap().expect("bearer cache"),
    )
    .unwrap();
    assert_eq!(cached["token"], "at-1");
    assert!(cached["expiresAt"].is_number());
    assert_eq!(store.get(&server.origin, CredentialKind::Pat).unwrap(), None, "一個位址同時只有一種登入");
    assert!(!dir.path().join("credentials.yaml").exists(), "憑證檔不變");

    // The identity was asked with the new access token.
    assert_eq!(server.bearers.lock().unwrap().as_slice(), &["Bearer at-1".to_string()]);

    // `auth status` reports the login as the keyring refresh layer.
    let resolved = speclink_remote::auth::resolve_credential_at(dir.path(), &server.origin, &store, None)
        .expect("resolve")
        .expect("a credential");
    assert_eq!(resolved.source.as_str(), "keychain_refresh");
    assert!(!format!("{resolved:?}").contains("at-1"), "a debug print hides the bearer");
    assert_eq!(resolved.token, "at-1", "the cached access token, no refresh spent");
}

/// The page the browser lands on says the login is done, and nothing else.
#[test]
fn the_page_returned_to_the_browser_carries_no_code_or_token() {
    let (.., io, outcome) = browser_login(IdpConfig::default(), Browser::Follows);
    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");

    let pages = io.pages();
    assert_eq!(pages.len(), 1);
    let (status, body) = &pages[0];
    assert_eq!(*status, 200);
    for secret in [AUTH_CODE, "at-1", "rt-1"] {
        assert!(!body.contains(secret), "page leaks {secret}: {body}");
    }
}

#[test]
fn a_denial_at_the_login_service_is_denied_and_stores_nothing() {
    let (server, _, store, _dir, io, outcome) =
        browser_login(IdpConfig { deny: true, ..IdpConfig::default() }, Browser::Follows);

    assert!(matches!(outcome, LoginOutcome::Denied), "{outcome:?}");
    assert_keyring_untouched(&store, &server.origin);
    let pages = io.pages();
    assert!(pages[0].1.contains("登入沒有完成"), "the browser is told the login did not finish: {}", pages[0].1);
}

/// Nobody comes back within the window: expired, nothing stored, the port shut.
#[test]
fn no_callback_within_the_window_expires_and_closes_the_port() {
    let (server, _, store, _dir, io, outcome) = browser_login(IdpConfig::default(), Browser::Absent);

    assert!(matches!(outcome, LoginOutcome::Expired), "{outcome:?}");
    assert_keyring_untouched(&store, &server.origin);
    let redirect = query_of(&browser_url(&io))["redirect_uri"].clone();
    let port = redirect
        .trim_start_matches("http://127.0.0.1:")
        .trim_end_matches("/callback")
        .to_string();
    assert!(
        std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_err(),
        "the callback port must be closed after the login ends"
    );
}

/// A callback that does not carry our state is someone else's; it gets an
/// error and the login keeps waiting for the real one.
#[test]
fn a_callback_with_the_wrong_state_is_refused_and_the_login_continues() {
    let (_, _, _, _dir, io, outcome) = browser_login(IdpConfig::default(), Browser::ForgesStateFirst);

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
    let pages = io.pages();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].0, 400, "the forged callback is refused");
    let real_state = query_of(&browser_url(&io))["state"].clone();
    assert!(!pages[0].1.contains(&real_state), "the refusal must not leak the expected state");
    assert_eq!(pages[1].0, 200);
}

#[test]
fn an_issuer_mismatch_in_the_metadata_fails_before_the_browser_opens() {
    let (server, _, store, _dir, io, outcome) = browser_login(
        IdpConfig { claimed_issuer: Some("https://evil.example.com".into()), ..IdpConfig::default() },
        Browser::Follows,
    );

    match outcome {
        LoginOutcome::Failed(msg) => assert!(msg.contains("issuer"), "names the item: {msg}"),
        other => panic!("應失敗，得到 {other:?}"),
    }
    assert!(io.opened.lock().unwrap().is_empty(), "沒有開啟瀏覽器");
    assert_keyring_untouched(&store, &server.origin);
}

#[test]
fn a_login_service_with_only_an_openid_configuration_still_works() {
    let (.., outcome) = browser_login(IdpConfig { oidc_only: true, ..IdpConfig::default() }, Browser::Follows);

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
}

/// Listing no PKCE methods at all is common and still gets S256; listing
/// methods without S256 is a refusal.
#[test]
fn a_login_service_listing_only_plain_pkce_is_refused() {
    let (server, _, store, _dir, io, outcome) = browser_login(
        IdpConfig { pkce_methods: Some(vec!["plain"]), ..IdpConfig::default() },
        Browser::Follows,
    );

    match outcome {
        LoginOutcome::Failed(msg) => assert!(msg.contains("S256"), "names the item: {msg}"),
        other => panic!("應失敗，得到 {other:?}"),
    }
    assert!(io.opened.lock().unwrap().is_empty());
    assert_keyring_untouched(&store, &server.origin);
}

#[test]
fn a_login_service_without_a_pkce_list_still_gets_s256() {
    let (_, idp, _, _dir, _, outcome) =
        browser_login(IdpConfig { pkce_methods: None, ..IdpConfig::default() }, Browser::Follows);

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
    assert_eq!(idp.authorize.lock().unwrap()[0]["code_challenge_method"], "S256");
}

#[test]
fn an_http_issuer_off_this_machine_is_refused() {
    let server = speclink_server();
    let store = MemoryCredentialStore::new();
    let dir = tempdir();
    let io = TestIo::new(Browser::Follows);
    let plan = Plan::Browser(BrowserLogin {
        issuer: "http://id.example.com".into(),
        client_id: CLIENT_ID.into(),
        scopes: vec![],
    });

    match login::login(&target(&server.origin, &store, dir.path()), &plan, &io) {
        LoginOutcome::Failed(msg) => assert!(msg.contains("https"), "{msg}"),
        other => panic!("應失敗，得到 {other:?}"),
    }
    assert!(io.opened.lock().unwrap().is_empty());
}

/// A userinfo part must not smuggle a remote host past the local-address rule.
#[test]
fn an_issuer_disguised_as_a_local_address_is_refused() {
    let server = speclink_server();
    let store = MemoryCredentialStore::new();
    let dir = tempdir();
    let io = TestIo::new(Browser::Follows);
    for issuer in ["http://127.0.0.1@id.example.com", "http://localhost.id.example.com"] {
        let plan = Plan::Browser(BrowserLogin { issuer: issuer.into(), client_id: CLIENT_ID.into(), scopes: vec![] });
        assert!(
            matches!(login::login(&target(&server.origin, &store, dir.path()), &plan, &io), LoginOutcome::Failed(_)),
            "{issuer}"
        );
    }
    assert!(io.opened.lock().unwrap().is_empty());
}

/// No keyring, no browser: a refresh credential is never written to a plain file.
#[test]
fn an_unavailable_keyring_refuses_before_opening_the_browser() {
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

    let server = speclink_server();
    let idp = idp(IdpConfig::default());
    let dir = tempdir();
    let io = TestIo::new(Browser::Follows);

    let outcome = login::login(&target(&server.origin, &NoKeyring, dir.path()), &Plan::Browser(idp.browser_login()), &io);

    assert!(matches!(outcome, LoginOutcome::KeychainUnavailable), "{outcome:?}");
    assert!(io.opened.lock().unwrap().is_empty());
    assert!(idp.authorize.lock().unwrap().is_empty());
}

/// A login service that issues no refresh token still logs in; the entry
/// records that there is nothing to refresh with. No `expires_in` reads as 300s.
#[test]
fn a_token_response_without_a_refresh_token_still_logs_in() {
    let (server, _, store, _dir, _, outcome) = browser_login(
        IdpConfig { issue_refresh: false, expires_in: None, ..IdpConfig::default() },
        Browser::Follows,
    );

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
    let entry: serde_json::Value = serde_json::from_str(
        &store.get(&server.origin, CredentialKind::Refresh).unwrap().expect("refresh entry"),
    )
    .unwrap();
    assert_eq!(entry["kind"], "oauth");
    assert!(entry["refreshToken"].is_null(), "{entry}");
    let cached: serde_json::Value = serde_json::from_str(
        &store.get(&server.origin, CredentialKind::Bearer).unwrap().expect("bearer cache"),
    )
    .unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let lifetime = cached["expiresAt"].as_i64().unwrap() - now;
    assert!((290..=310).contains(&lifetime), "no expires_in reads as 300s, got {lifetime}");
}

/// The login service takes the user's sign-in but refuses the code: a failure
/// that reads as one, with nothing stored and the browser told so.
#[test]
fn a_refused_code_exchange_fails_and_stores_nothing() {
    let (server, _, store, _dir, io, outcome) =
        browser_login(IdpConfig { refuse_exchange: true, ..IdpConfig::default() }, Browser::Follows);

    match outcome {
        LoginOutcome::Failed(msg) => assert!(msg.contains("refused"), "{msg}"),
        other => panic!("應失敗，得到 {other:?}"),
    }
    assert_keyring_untouched(&store, &server.origin);
    assert!(io.pages()[0].1.contains("登入沒有完成"));
}

/// A cancel that is already set when the login starts opens no browser.
#[test]
fn a_browser_login_cancelled_before_it_starts_opens_no_browser() {
    let server = speclink_server();
    let idp = idp(IdpConfig::default());
    let store = MemoryCredentialStore::new();
    let dir = tempdir();
    let io = TestIo::new(Browser::Follows);
    io.cancel.store(true, Ordering::SeqCst);

    let outcome = login::login(&target(&server.origin, &store, dir.path()), &Plan::Browser(idp.browser_login()), &io);

    assert!(matches!(outcome, LoginOutcome::Cancelled), "{outcome:?}");
    assert!(io.opened.lock().unwrap().is_empty());
    assert!(io.phases.lock().unwrap().is_empty());
}

/// A cancel while the code is being exchanged still wins: nothing is stored,
/// the token the login service just issued is revoked, and the browser is not
/// told the login finished.
#[test]
fn a_cancel_during_the_code_exchange_stores_nothing_and_revokes() {
    let server = speclink_server();
    let cancel = Arc::new(AtomicBool::new(false));
    let idp = idp(IdpConfig { cancel_on_exchange: Some(cancel.clone()), ..IdpConfig::default() });
    let store = MemoryCredentialStore::new();
    let dir = tempdir();
    let mut io = TestIo::new(Browser::Follows);
    io.cancel = cancel;

    let outcome = login::login(&target(&server.origin, &store, dir.path()), &Plan::Browser(idp.browser_login()), &io);

    assert!(matches!(outcome, LoginOutcome::Cancelled), "{outcome:?}");
    assert_keyring_untouched(&store, &server.origin);
    let revocations = idp.revoke.lock().unwrap();
    assert_eq!(revocations.len(), 1);
    assert_eq!(revocations[0]["token"], "rt-1");
    assert!(io.pages()[0].1.contains("登入沒有完成"));
}

// --- refreshing a browser login (D5；規格「access token 到期的靜默換新」等) ---

use speclink_remote::auth;
use speclink_remote::credentials::{OAuthRefresh, RefreshEntry};
use speclink_remote::refresh::{self, RefreshFailure};

/// A browser login whose access token has run out.
fn expired_browser_login(idp: &Idp, refresh_token: Option<&str>) -> (String, Arc<MemoryCredentialStore>, tempfile::TempDir) {
    let origin = "http://speclink.example.test".to_string();
    let store = Arc::new(MemoryCredentialStore::new());
    let entry = RefreshEntry::OAuth(OAuthRefresh {
        refresh_token: refresh_token.map(str::to_string),
        token_endpoint: idp.token_endpoint(),
        client_id: CLIENT_ID.to_string(),
        revocation_endpoint: None,
    });
    store.set(&origin, CredentialKind::Refresh, &entry.encode()).unwrap();
    store.set(&origin, CredentialKind::Bearer, r#"{"token":"at-1","expiresAt":0}"#).unwrap();
    (origin, store, tempdir())
}

fn refresh_grants(idp: &Idp) -> Vec<HashMap<String, String>> {
    idp.token
        .lock()
        .unwrap()
        .iter()
        .filter(|f| f.get("grant_type").map(String::as_str) == Some("refresh_token"))
        .cloned()
        .collect()
}

fn stored_entry(store: &MemoryCredentialStore, origin: &str) -> RefreshEntry {
    RefreshEntry::parse(&store.get(origin, CredentialKind::Refresh).unwrap().expect("refresh entry"))
}

#[test]
fn an_expired_browser_login_refreshes_at_the_token_endpoint_once() {
    let idp = idp(IdpConfig::default());
    let (origin, store, dir) = expired_browser_login(&idp, Some("rt-1"));

    let bearer = refresh::bearer_for(&origin, store.as_ref(), dir.path()).expect("refresh");

    assert_eq!(bearer, "at-2");
    let grants = refresh_grants(&idp);
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0]["refresh_token"], "rt-1");
    assert_eq!(grants[0]["client_id"], CLIENT_ID);
    assert!(!grants[0].contains_key("client_secret"));
    match stored_entry(&store, &origin) {
        RefreshEntry::OAuth(oauth) => {
            assert_eq!(oauth.refresh_token.as_deref(), Some("rt-2"), "rotated token is written back");
            assert_eq!(oauth.token_endpoint, idp.token_endpoint());
        }
        other => panic!("應仍是瀏覽器登入的條目，得到 {other:?}"),
    }
    assert_eq!(refresh::cached_bearer(&origin, store.as_ref()), Some("at-2".to_string()));
}

/// A token endpoint that does not rotate leaves the old refresh token in place.
#[test]
fn a_refresh_without_a_new_refresh_token_keeps_the_old_one() {
    let idp = idp(IdpConfig { refresh_reply: RefreshReply::KeepRefresh, ..IdpConfig::default() });
    let (origin, store, dir) = expired_browser_login(&idp, Some("rt-1"));

    refresh::bearer_for(&origin, store.as_ref(), dir.path()).expect("refresh");

    match stored_entry(&store, &origin) {
        RefreshEntry::OAuth(oauth) => assert_eq!(oauth.refresh_token.as_deref(), Some("rt-1")),
        other => panic!("{other:?}"),
    }
}

/// Two processes finding the cache expired at once still refresh once: the
/// late one takes what the first one cached.
#[test]
fn concurrent_refreshes_of_a_browser_login_hit_the_token_endpoint_once() {
    let idp = idp(IdpConfig { refresh_delay_ms: 120, ..IdpConfig::default() });
    let (origin, store, dir) = expired_browser_login(&idp, Some("rt-1"));

    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (origin, store, lock_dir) = (origin.clone(), store.clone(), dir.path().to_path_buf());
            std::thread::spawn(move || refresh::bearer_for(&origin, store.as_ref(), &lock_dir).expect("refresh"))
        })
        .collect();
    let bearers: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(refresh_grants(&idp).len(), 1, "the token endpoint must see one refresh");
    assert_eq!(bearers[0], bearers[1]);
}

#[test]
fn an_invalid_grant_clears_the_login_and_asks_for_a_new_one() {
    let idp = idp(IdpConfig { refresh_reply: RefreshReply::InvalidGrant, ..IdpConfig::default() });
    let (origin, store, dir) = expired_browser_login(&idp, Some("rt-1"));

    match auth::resolve_credential_at(dir.path(), &origin, store.as_ref(), None) {
        Err(RefreshFailure::Rejected(msg)) => assert!(msg.contains("speclink auth login"), "{msg}"),
        other => panic!("應被拒絕，得到 {other:?}"),
    }
    assert_eq!(store.get(&origin, CredentialKind::Refresh).unwrap(), None);
    assert_eq!(store.get(&origin, CredentialKind::Bearer).unwrap(), None);
}

/// A login service that is down says nothing about the credential: keep it.
#[test]
fn a_token_endpoint_5xx_keeps_the_credential() {
    let idp = idp(IdpConfig { refresh_reply: RefreshReply::ServerError, ..IdpConfig::default() });
    let (origin, store, dir) = expired_browser_login(&idp, Some("rt-1"));

    match auth::resolve_credential_at(dir.path(), &origin, store.as_ref(), None) {
        Err(RefreshFailure::Unavailable(_)) => {}
        other => panic!("應為暫時無法換新，得到 {other:?}"),
    }
    assert!(store.get(&origin, CredentialKind::Refresh).unwrap().is_some(), "credential kept");
}

/// No refresh token was ever issued: once the access token is gone, so is the login.
#[test]
fn a_login_without_a_refresh_token_is_rejected_once_its_access_token_expires() {
    let idp = idp(IdpConfig::default());
    let (origin, store, dir) = expired_browser_login(&idp, None);

    match refresh::bearer_for(&origin, store.as_ref(), dir.path()) {
        Err(RefreshFailure::Rejected(msg)) => assert!(msg.contains("speclink auth login"), "{msg}"),
        other => panic!("應被拒絕，得到 {other:?}"),
    }
    assert!(refresh_grants(&idp).is_empty(), "nothing to refresh with, so no request");
}

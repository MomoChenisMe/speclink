//! Driving a device authorization to a verdict
//! (cli-desktop-credential-sharing:「裝置授權登入」).
//!
//! The orchestration lives in the library rather than in the CLI because a
//! CLI integration test runs the real binary, which would reach for the real
//! OS keychain — unusable on CI and destructive on a developer's machine. Here
//! the store, the clock and the terminal are all injected.
//!
//! What the tests pin: the URL and code are always announced (a second device
//! can approve even when no browser opens), the poll loop honours the server's
//! interval and slow_down, approval stores both credentials, and denied,
//! expired and unsupported each come back as distinct verdicts rather than as
//! one generic failure.

use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_remote::login::{self, LoginIo, LoginOutcome, LoginPhase, LoginTarget};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct RecordingIo {
    announced: Mutex<Vec<(String, String)>>,
    opened: Mutex<Vec<String>>,
    slept: Mutex<Vec<u64>>,
    /// Set when the environment has no browser to open.
    browser_fails: bool,
    /// The user cancels after the login asked this many times; `None` never.
    cancel_after: Option<usize>,
    asked: AtomicUsize,
}

fn target<'a>(origin: &'a str, store: &'a dyn CredentialStore, dir: &'a std::path::Path) -> LoginTarget<'a> {
    LoginTarget { origin, credentials: store, lock_dir: dir }
}

impl LoginIo for RecordingIo {
    fn phase(&self, phase: LoginPhase) {
        if let LoginPhase::AwaitingDevice { verification_uri, user_code, .. } = phase {
            self.announced.lock().unwrap().push((verification_uri, user_code));
        }
    }
    fn open_browser(&self, url: &str) -> bool {
        self.opened.lock().unwrap().push(url.to_string());
        !self.browser_fails
    }
    fn sleep_secs(&self, secs: u64) {
        self.slept.lock().unwrap().push(secs);
    }
    fn cancelled(&self) -> bool {
        let asked = self.asked.fetch_add(1, Ordering::SeqCst);
        self.cancel_after.is_some_and(|after| asked >= after)
    }
}

/// A scripted device-flow server: `/auth/device` initiates, `/auth/device/token`
/// returns the next scripted status each time it is polled, `/auth/whoami`
/// answers with an identity.
struct Scripted {
    base: String,
    polls: Arc<AtomicUsize>,
    paths: Arc<Mutex<Vec<String>>>,
}

/// What the scripted server answers beyond the poll script.
#[derive(Clone, Copy)]
struct Answers {
    /// The refresh credential an approval carries (a JSON string literal's content).
    refresh_token: &'static str,
    whoami_status: u16,
}

const USUAL: Answers = Answers { refresh_token: "refresh-1", whoami_status: 200 };

fn scripted_server(statuses: Vec<&'static str>, interval: u64) -> Scripted {
    scripted_server_answering(statuses, interval, USUAL)
}

fn scripted_server_answering(statuses: Vec<&'static str>, interval: u64, answers: Answers) -> Scripted {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", server.server_addr());
    let polls = Arc::new(AtomicUsize::new(0));
    let counter = polls.clone();
    let paths: Arc<Mutex<Vec<String>>> = Arc::default();
    let seen = paths.clone();
    let script = Arc::new(statuses);
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            seen.lock().unwrap().push(path.clone());
            let (code, body) = match path.as_str() {
                "/auth/device" => (
                    200,
                    format!(
                        r#"{{"deviceCode":"dc_1","userCode":"WDJB-MJHT","verificationUri":"http://approve.example/activate","expiresIn":900,"interval":{interval}}}"#
                    ),
                ),
                "/auth/device/token" => {
                    let n = counter.fetch_add(1, Ordering::SeqCst);
                    let status = script.get(n).copied().unwrap_or("pending");
                    let body = match status {
                        "approved" => format!(
                            r#"{{"status":"approved","accessToken":"access-1","refreshToken":"{}","expiresIn":3600}}"#,
                            answers.refresh_token
                        ),
                        other => format!(r#"{{"status":"{other}"}}"#),
                    };
                    (200, body)
                }
                "/auth/whoami" => (
                    answers.whoami_status,
                    r#"{"user":{"name":"Dev <dev@example.com>","handle":"dev"}}"#.to_string(),
                ),
                "/auth/revoke" => (200, "{}".to_string()),
                _ => (
                    404,
                    r#"{"status":404,"reason":"not_found","message":"no route"}"#.to_string(),
                ),
            };
            let resp = tiny_http::Response::from_string(body)
                .with_status_code(code)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .expect("header"),
                );
            let _ = req.respond(resp);
        }
    });
    Scripted { base, polls, paths }
}

/// A server with no device flow at all — the PAT fallback signal.
fn unsupported_server() -> String {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}", server.server_addr());
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let resp = tiny_http::Response::from_string(
                r#"{"status":404,"reason":"not_found","message":"no route"}"#,
            )
            .with_status_code(404)
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                    .expect("header"),
            );
            let _ = req.respond(resp);
        }
    });
    base
}

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn approval_stores_both_credentials_and_reports_the_identity() {
    let server = scripted_server(vec!["pending", "approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo::default();

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &io).expect("登入不應失敗");

    match outcome {
        LoginOutcome::LoggedIn { display, .. } => {
            assert_eq!(display, "Dev <dev@example.com>");
        }
        other => panic!("應為 Approved，得到 {other:?}"),
    }
    assert_eq!(
        store.get(&server.base, CredentialKind::Refresh).unwrap(),
        Some("refresh-1".to_string()),
        "refresh credential 必須入金鑰圈"
    );
    let cached = store
        .get(&server.base, CredentialKind::Bearer)
        .unwrap()
        .expect("access token 應同時快取");
    let parsed: serde_json::Value = serde_json::from_str(&cached).unwrap();
    assert_eq!(parsed["token"], "access-1");
    assert!(parsed["expiresAt"].is_number());
}

/// An origin holds one kind of login at a time: approving a device replaces a
/// keyring PAT (client-browser-login D4).
#[test]
fn approval_replaces_a_keyring_pat() {
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    store.set(&server.base, CredentialKind::Pat, "old-pat").unwrap();

    login::device_login(&target(&server.base, &store, dir.path()), &RecordingIo::default()).expect("登入不應失敗");

    assert_eq!(store.get(&server.base, CredentialKind::Pat).unwrap(), None);
}

/// The URL and the code are printed whether or not a browser opened: approving
/// from a phone or another machine is a first-class path, not a fallback.
#[test]
fn the_url_and_code_are_always_announced() {
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo { browser_fails: true, ..Default::default() };

    login::device_login(&target(&server.base, &store, dir.path()), &io).expect("登入不應失敗");

    let announced = io.announced.lock().unwrap();
    assert_eq!(
        announced.as_slice(),
        &[(
            "http://approve.example/activate".to_string(),
            "WDJB-MJHT".to_string()
        )],
        "無法開瀏覽器時仍必須印出網址與裝置碼"
    );
}

/// The browser URL carries the code so the approval page can prefill it.
#[test]
fn the_browser_url_prefills_the_user_code() {
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo::default();

    login::device_login(&target(&server.base, &store, dir.path()), &io).expect("登入不應失敗");

    let opened = io.opened.lock().unwrap();
    assert_eq!(
        opened.as_slice(),
        &["http://approve.example/activate?user_code=WDJB-MJHT".to_string()]
    );
}

#[test]
fn polling_waits_the_interval_the_server_declared() {
    let server = scripted_server(vec!["pending", "pending", "approved"], 7);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo::default();

    login::device_login(&target(&server.base, &store, dir.path()), &io).expect("登入不應失敗");

    assert_eq!(
        io.slept.lock().unwrap().as_slice(),
        &[7, 7],
        "每次 pending 後應等待 server 宣告的間隔"
    );
    assert_eq!(server.polls.load(Ordering::SeqCst), 3);
}

/// slow_down is the server asking for room; ignoring it is how a client gets
/// itself rate-limited.
#[test]
fn slow_down_widens_the_interval() {
    let server = scripted_server(vec!["slow_down", "approved"], 5);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo::default();

    login::device_login(&target(&server.base, &store, dir.path()), &io).expect("登入不應失敗");

    let slept = io.slept.lock().unwrap();
    assert!(
        slept[0] > 5,
        "slow_down 後的等待必須大於原間隔，實得 {}",
        slept[0]
    );
}

#[test]
fn denial_and_expiry_are_distinct_verdicts() {
    let dir = tempdir();

    let denied = scripted_server(vec!["denied"], 1);
    let outcome = login::device_login(&target(&denied.base, &MemoryCredentialStore::new(), dir.path()), &RecordingIo::default())
    .expect("被拒是結果不是錯誤");
    assert!(matches!(outcome, LoginOutcome::Denied));

    let expired = scripted_server(vec!["expired"], 1);
    let outcome = login::device_login(&target(&expired.base, &MemoryCredentialStore::new(), dir.path()), &RecordingIo::default())
    .expect("逾期是結果不是錯誤");
    assert!(matches!(outcome, LoginOutcome::Expired));
}

/// A server without the device flow is the explicit PAT fallback signal — not
/// an error, and never confused with a broken server.
#[test]
fn a_server_without_the_device_flow_is_unsupported() {
    let dir = tempdir();
    let outcome = login::device_login(&target(&unsupported_server(), &MemoryCredentialStore::new(), dir.path()), &RecordingIo::default())
    .expect("不支援是結果不是錯誤");

    assert!(matches!(outcome, LoginOutcome::Unsupported));
}

/// Nothing is written when the user never approves.
#[test]
fn a_denied_login_stores_nothing() {
    let server = scripted_server(vec!["denied"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();

    login::device_login(&target(&server.base, &store, dir.path()), &RecordingIo::default())
        .expect("被拒是結果不是錯誤");

    assert_eq!(store.get(&server.base, CredentialKind::Refresh).unwrap(), None);
    assert_eq!(store.get(&server.base, CredentialKind::Bearer).unwrap(), None);
}

/// A keyring that cannot store the credential must fail the login loudly —
/// silently "succeeding" would leave the user logged in only until the process
/// exited.
#[test]
fn an_unwritable_keyring_fails_the_login() {
    struct ReadOnly;
    impl CredentialStore for ReadOnly {
        fn get(&self, _: &str, _: CredentialKind) -> Result<Option<String>, String> {
            Ok(None)
        }
        fn set(&self, _: &str, _: CredentialKind, _: &str) -> Result<(), String> {
            Err("no keyring service on this platform".into())
        }
        fn delete(&self, _: &str, _: CredentialKind) -> Result<(), String> {
            Ok(())
        }
    }

    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();

    let outcome = login::device_login(&target(&server.base, &ReadOnly, dir.path()), &RecordingIo::default());

    assert!(outcome.is_err(), "無法儲存 credential 時登入必須失敗");
}

/// The credentials are in; an identity lookup that then fails does not turn
/// the login into a failure (the browser login answers the same way).
#[test]
fn an_identity_failure_after_approval_still_reports_the_login() {
    let server = scripted_server_answering(vec!["approved"], 1, Answers { whoami_status: 500, ..USUAL });
    let dir = tempdir();
    let store = MemoryCredentialStore::new();

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &RecordingIo::default())
        .expect("the credentials are stored, so this is not an error");

    assert!(matches!(outcome, LoginOutcome::LoggedInWithoutIdentity { .. }), "{outcome:?}");
    assert_eq!(
        store.get(&server.base, CredentialKind::Refresh).unwrap(),
        Some("refresh-1".to_string())
    );
}

/// The refresh entry is the login; a cache that cannot be written only costs
/// the next command one rotation.
#[test]
fn a_cache_that_cannot_be_written_does_not_fail_the_login() {
    struct NoCache(MemoryCredentialStore);
    impl CredentialStore for NoCache {
        fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
            self.0.get(origin, kind)
        }
        fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
            if kind == CredentialKind::Bearer {
                return Err("cache entry refused".into());
            }
            self.0.set(origin, kind, secret)
        }
        fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
            self.0.delete(origin, kind)
        }
    }
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = NoCache(MemoryCredentialStore::new());

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &RecordingIo::default())
        .expect("登入不應失敗");

    assert!(matches!(outcome, LoginOutcome::LoggedIn { .. }), "{outcome:?}");
    assert_eq!(store.get(&server.base, CredentialKind::Refresh).unwrap(), Some("refresh-1".to_string()));
}

/// A cancel that is already set when the login starts opens nothing and asks
/// the server nothing more.
#[test]
fn a_login_cancelled_before_it_starts_opens_no_browser() {
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    let io = RecordingIo { cancel_after: Some(0), ..Default::default() };

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &io).expect("取消是結果不是錯誤");

    assert!(matches!(outcome, LoginOutcome::Cancelled), "{outcome:?}");
    assert!(io.opened.lock().unwrap().is_empty());
    assert!(io.announced.lock().unwrap().is_empty());
    assert_eq!(server.polls.load(Ordering::SeqCst), 0);
}

/// A cancel that lands together with the approval still wins: nothing is
/// stored, and the family the server just issued is revoked.
#[test]
fn a_cancel_that_lands_with_the_approval_stores_nothing_and_revokes() {
    let server = scripted_server(vec!["approved"], 1);
    let dir = tempdir();
    let store = MemoryCredentialStore::new();
    // Asked before the phase and before the poll: not cancelled. Asked again
    // once the approval is in: cancelled.
    let io = RecordingIo { cancel_after: Some(2), ..Default::default() };

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &io).expect("取消是結果不是錯誤");

    assert!(matches!(outcome, LoginOutcome::Cancelled), "{outcome:?}");
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer] {
        assert_eq!(store.get(&server.base, kind).unwrap(), None, "{kind:?}");
    }
    assert!(server.paths.lock().unwrap().contains(&"/auth/revoke".to_string()));
}

/// A server cannot plant a browser-login entry through the device flow: such a
/// "refresh token" would later send requests to endpoints of its choosing.
#[test]
fn a_refresh_credential_shaped_like_a_browser_login_entry_is_refused() {
    let hostile = r#"{\"kind\":\"oauth\",\"refreshToken\":\"x\",\"tokenEndpoint\":\"http://evil.example/t\",\"clientId\":\"c\"}"#;
    let server = scripted_server_answering(vec!["approved"], 1, Answers { refresh_token: hostile, ..USUAL });
    let dir = tempdir();
    let store = MemoryCredentialStore::new();

    let outcome = login::device_login(&target(&server.base, &store, dir.path()), &RecordingIo::default());

    assert!(outcome.is_err(), "{outcome:?}");
    assert_eq!(store.get(&server.base, CredentialKind::Refresh).unwrap(), None);
}

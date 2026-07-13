//! Contract tests for the request layer: required headers on every request,
//! and the error-translation red line — every non-2xx becomes one semantic
//! line, never a bare status code as the primary output.
//!
//! Error bodies speak the protocol registry (`{ status, reason, message }`);
//! the full reason-by-reason mapping table is asserted in `typed_client.rs`.
//! The mock server is a dev-dependency `tiny_http` one-shot listener; each
//! test spins its own on a random port, so tests stay independent.

use speclink_remote::client::Client;
use std::sync::mpsc;

/// What the mock captured about the single request it served.
struct Captured {
    method: String,
    path: String,
    headers: Vec<(String, String)>, // lowercased field names
}

impl Captured {
    fn header(&self, name: &str) -> Option<&str> {
        let want = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| *k == want)
            .map(|(_, v)| v.as_str())
    }
}

/// Serve exactly one request with the given status/body, capturing it.
/// Returns the project-scoped base URL to point the client at.
fn serve_once(status: u16, body: &'static str) -> (String, mpsc::Receiver<Captured>) {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind mock server");
    let port = server.server_addr().to_ip().expect("ip addr").port();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(req) = server.recv() {
            let cap = Captured {
                method: req.method().to_string(),
                path: req.url().to_string(),
                headers: req
                    .headers()
                    .iter()
                    .map(|h| (h.field.to_string().to_ascii_lowercase(), h.value.to_string()))
                    .collect(),
            };
            let response = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(response);
            let _ = tx.send(cap);
        }
    });
    (
        format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo"),
        rx,
    )
}

fn recv(rx: &mpsc::Receiver<Captured>) -> Captured {
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("mock server captured a request")
}

// --- headers ---

#[test]
fn every_request_carries_auth_version_and_repo_headers() {
    let (base, rx) = serve_once(200, r#"{"changes":[]}"#);
    let client = Client::new(&base, "tok-123", Some("backend"));
    client.list_changes().expect("list ok");
    let cap = recv(&rx);
    assert_eq!(cap.method, "GET");
    assert!(cap.path.ends_with("/changes"), "path was {}", cap.path);
    assert_eq!(cap.header("authorization"), Some("Bearer tok-123"));
    assert_eq!(cap.header("x-speclink-api-version"), Some("1"));
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"));
}

#[test]
fn repo_header_is_omitted_when_no_repo_declared() {
    let (base, rx) = serve_once(200, r#"{"changes":[]}"#);
    let client = Client::new(&base, "tok-123", None);
    client.list_changes().expect("list ok");
    let cap = recv(&rx);
    assert_eq!(cap.header("x-speclink-repo"), None);
}

#[test]
fn artifact_write_sends_if_match_of_the_read_version() {
    let (base, rx) = serve_once(200, r#"{"artifact":"design","version":8}"#);
    let client = Client::new(&base, "tok-123", Some("backend"));
    client
        .put_artifact("add-rate-limit", "design", "content", 7)
        .expect("put ok");
    let cap = recv(&rx);
    assert_eq!(cap.method, "PUT");
    assert!(
        cap.path.ends_with("/changes/add-rate-limit/artifacts/design"),
        "path was {}",
        cap.path
    );
    assert_eq!(cap.header("if-match"), Some("7"));
}

// --- error translation: auth ---

#[test]
fn err_401_points_at_auth_login_without_bare_status() {
    let (base, _rx) = serve_once(
        401,
        r#"{"status":401,"reason":"permission_denied","message":"token expired"}"#,
    );
    let client = Client::new(&base, "tok-old", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("speclink auth login"),
        "message was: {}",
        err.message
    );
    assert!(!err.message.contains("401"), "bare status leaked: {}", err.message);
    assert_eq!(err.reason.as_deref(), Some("permission_denied"));
}

// --- error translation: engine-class refusals relay the server's message ---

#[test]
fn err_404_relays_the_servers_not_found_message() {
    let (base, _rx) = serve_once(
        404,
        r#"{"status":404,"reason":"not_found","message":"Change 'add-rate-limit' not found."}"#,
    );
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.get_change("add-rate-limit").unwrap_err();
    assert_eq!(err.message, "Change 'add-rate-limit' not found.");
    assert!(!err.message.contains("404"), "bare status leaked: {}", err.message);
    assert_eq!(err.reason.as_deref(), Some("not_found"));
}

#[test]
fn err_409_refused_relays_the_servers_precondition_message() {
    let (base, _rx) = serve_once(
        409,
        r#"{"status":409,"reason":"refused","message":"3 task(s) still open — finish them before archiving"}"#,
    );
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.archive("add-rate-limit").unwrap_err();
    assert_eq!(
        err.message,
        "3 task(s) still open — finish them before archiving"
    );
    assert_eq!(err.reason.as_deref(), Some("refused"));
}

// --- error translation: the CAS conflict keeps its suggested action ---

#[test]
fn err_409_revision_conflict_suggests_rereading() {
    let (base, _rx) = serve_once(
        409,
        r#"{"status":409,"reason":"revision_conflict","message":"stale"}"#,
    );
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client
        .put_artifact("add-rate-limit", "design", "content", 3)
        .unwrap_err();
    assert!(err.message.contains("re-read"), "message was: {}", err.message);
    assert!(!err.message.contains("409"), "bare status leaked: {}", err.message);
    assert_eq!(err.reason.as_deref(), Some("revision_conflict"));
}

// --- error translation: server failure and unreachable ---

#[test]
fn err_5xx_translates_to_server_unavailable() {
    let (base, _rx) = serve_once(500, "boom");
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("server unavailable"),
        "message was: {}",
        err.message
    );
    assert!(
        err.message.contains("remote.url") && err.message.contains(".speclink.yaml"),
        "message points at the remote section: {}",
        err.message
    );
    assert!(!err.message.starts_with("500"), "bare status leads: {}", err.message);
}

#[test]
fn err_connection_refused_fails_loud_with_connection_hint() {
    // Grab a port the OS just released — nothing listens there.
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("server unreachable"),
        "message was: {}",
        err.message
    );
    assert!(
        err.message.contains("remote.url") && err.message.contains(".speclink.yaml"),
        "message points at the remote section: {}",
        err.message
    );
    assert!(err.reason.is_none(), "transport failure has no server reason");
}

// --- unknown reasons and bare envelopes fall back generically ---

#[test]
fn err_unknown_reason_uses_generic_fallback() {
    let (base, _rx) = serve_once(
        418,
        r#"{"status":418,"reason":"im_a_teapot","message":"short and stout"}"#,
    );
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("unexpected server response"),
        "message was: {}",
        err.message
    );
    assert_eq!(err.reason.as_deref(), Some("im_a_teapot"));
}

#[test]
fn err_envelope_less_401_and_404_keep_their_status_fallbacks() {
    let (base, _rx) = serve_once(401, "nope");
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("speclink auth login"),
        "message was: {}",
        err.message
    );
    assert!(err.reason.is_none());

    let (base, _rx) = serve_once(404, "gone");
    let client = Client::new(&base, "tok", Some("backend"));
    let err = client.list_changes().unwrap_err();
    assert!(
        err.message.contains("resource not found"),
        "message was: {}",
        err.message
    );
}

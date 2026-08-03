//! Binding handshake contract tests: the handshake is the connection
//! precondition and fails closed — an incompatible API version or an
//! ambiguous binding is a refusal, never an automatic choice, and no other
//! request leaves the client. Capabilities are parsed and kept as
//! declarations only; no event connection is ever opened.
//!
//! The mock server reuses the `tiny_http` capture pattern from
//! `client_errors.rs`, extended to serve any number of requests so "exactly
//! one request left the client" is assertable.

use speclink_remote::client::Client;
use std::sync::{Arc, Mutex};

/// What the mock captured about each request it served.
#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
}

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
    captured: Arc<Mutex<Vec<Captured>>>,
}

/// Serve every incoming request with the same status/body, capturing all of
/// them. Returns the project-scoped base URL to point the client at.
fn serve(status: u16, body: &'static str) -> MockServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip addr").port();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let looper = Arc::clone(&server);
    let sink = Arc::clone(&captured);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: req.url().split('?').next().unwrap_or_default().to_string(),
            });
            let response = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(response);
        }
    });
    MockServer {
        server,
        base: format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo"),
        captured,
    }
}

impl MockServer {
    fn requests(&self) -> Vec<Captured> {
        self.captured.lock().unwrap().clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

const COMPATIBLE_BINDING: &str = r#"{"actor":{"id":"u_42","name":"王小明"},"project":{"id":"prj_01H","key":"demo","name":"Demo"},"repo":{"id":"repo_01H","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"1.4.0","capabilities":{"events":{"transports":[{"type":"sse","url":"/events","resume":true}],"polling":{"url":"/sync-state","etag":true}}}}"#;

const INCOMPATIBLE_BINDING: &str = r#"{"actor":{"id":"u_42","name":"王小明"},"project":{"id":"prj_01H","key":"demo","name":"Demo"},"repo":{"id":"repo_01H","key":"backend","name":"Backend"},"apiVersion":"2","engineVersion":"9.0.0"}"#;

// --- fail closed: version incompatibility is a client-side refusal ---

#[test]
fn handshake_rejects_an_incompatible_api_version_and_sends_nothing_else() {
    let mock = serve(200, INCOMPATIBLE_BINDING);
    let client = Client::new(&mock.base, "tok", Some("backend"));
    let err = client.handshake().unwrap_err();
    assert!(
        err.message.contains("API version"),
        "the refusal names the version incompatibility: {}",
        err.message
    );
    assert!(
        err.message.contains("upgrade"),
        "the refusal keeps the existing upgrade guidance: {}",
        err.message
    );
    let requests = mock.requests();
    assert_eq!(requests.len(), 1, "no follow-up request leaves the client: {requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert!(requests[0].path.ends_with("/binding"), "path was {}", requests[0].path);
}

// --- fail closed: an ambiguous binding is the server's registry refusal ---

#[test]
fn handshake_ambiguous_binding_relays_the_candidate_list() {
    let mock = serve(
        400,
        r#"{"status":400,"reason":"invalid_config","message":"this project has multiple repos — set `remote.repo` in .speclink.yaml (candidates: backend, frontend)"}"#,
    );
    let client = Client::new(&mock.base, "tok", None);
    let err = client.handshake().unwrap_err();
    assert!(
        err.message.contains("backend") && err.message.contains("frontend"),
        "the refusal lists the candidates: {}",
        err.message
    );
    assert_eq!(err.reason.as_deref(), Some("invalid_config"));
    assert_eq!(mock.requests().len(), 1, "ambiguity is not resolved by probing");
}

// --- capabilities are declarations, not connections ---

#[test]
fn handshake_parses_and_keeps_capabilities_without_opening_event_connections() {
    let mock = serve(200, COMPATIBLE_BINDING);
    let client = Client::new(&mock.base, "tok", Some("backend"));
    let binding = client.handshake().expect("handshake succeeds");

    assert_eq!(binding.actor.name, "王小明");
    assert_eq!(binding.project.key, "demo");
    assert_eq!(binding.repo.key, "backend");
    assert_eq!(binding.api_version, "1");
    assert_eq!(binding.engine_version, "1.4.0");

    let events = &binding.capabilities.events;
    assert_eq!(events.transports.len(), 1);
    assert_eq!(
        events.transports[0].kind,
        speclink_protocol::events::TransportKind::Sse
    );
    assert!(events.transports[0].resume);
    let polling = events.polling.as_ref().expect("polling declaration kept");
    assert_eq!(polling.url, "/sync-state");
    assert!(polling.etag);

    let requests = mock.requests();
    assert_eq!(
        requests.len(),
        1,
        "declarations are stored, no event connection is opened: {requests:?}"
    );
    assert!(requests[0].path.ends_with("/binding"));
}

// --- handshake requests carry the contract headers like any other call ---

#[test]
fn handshake_failure_reports_missing_binding_as_the_servers_refusal() {
    let mock = serve(
        404,
        r#"{"status":404,"reason":"not_found","message":"no binding for this project"}"#,
    );
    let client = Client::new(&mock.base, "tok", Some("backend"));
    let err = client.handshake().unwrap_err();
    assert_eq!(err.reason.as_deref(), Some("not_found"));
    assert!(
        err.message.contains("no binding for this project"),
        "the server's refusal message survives: {}",
        err.message
    );
}

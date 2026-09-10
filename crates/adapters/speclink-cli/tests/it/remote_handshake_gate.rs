//! Remote verbs run inside the connection context the binding handshake
//! establishes: a failed handshake (here: an ambiguous binding) stops any
//! verb with a nonzero exit and the candidate list on stderr, and no verb
//! request ever leaves the client. A workspace's existing `.speclink.yaml`
//! remote section works unchanged.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

// --- capturing mock server ---

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

/// Routes: (method, path after the project base — query ignored, status, body).
fn mock_server(routes: Vec<(&'static str, &'static str, u16, String)>) -> MockServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let captured = Arc::new(Mutex::new(Vec::new()));
    let looper = Arc::clone(&server);
    let sink = Arc::clone(&captured);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: path.clone(),
            });
            let hit = routes.iter().find(|(m, suffix, _, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, status, body)) => (*status, body.clone()),
                None => (404, r#"{"status":404,"reason":"not_found","message":"no route"}"#.to_string()),
            };
            let resp = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(resp);
        }
    });
    MockServer { server, base, captured }
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

// --- throwaway remote project ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A remote-mode project with the existing `.speclink.yaml` remote
    /// section shape — url plus optional repo key, nothing new.
    fn remote(tag: &str, url: &str, repo: Option<&str>) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-handshake-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut yaml = format!("remote:\n  url: {url}\n");
        if let Some(r) = repo {
            yaml.push_str(&format!("  repo: {r}\n"));
        }
        std::fs::write(dir.join(".speclink.yaml"), yaml).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env("SPECLINK_TOKEN", "tok");
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// --- fail closed: an ambiguous binding stops every verb ---

#[test]
fn ambiguous_binding_stops_the_verb_with_candidates_and_no_verb_request() {
    let mock = mock_server(vec![(
        "GET",
        "/binding",
        400,
        r#"{"status":400,"reason":"invalid_config","message":"this project has multiple repos — set `remote.repo` in .speclink.yaml (candidates: backend, frontend)"}"#
            .into(),
    )]);
    let p = TempProject::remote("ambiguous", &mock.base, None);

    let out = p.run(&["list"]);
    assert!(!out.status.success(), "an ambiguous binding must be a nonzero exit");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("backend") && stderr.contains("frontend"),
        "stderr lists the candidates: {stderr}"
    );
    assert!(
        stderr.contains("multiple repos"),
        "stderr points at the ambiguity: {stderr}"
    );
    assert!(out.stdout.is_empty(), "no data on stdout when the handshake fails");

    let requests = mock.requests();
    assert_eq!(requests.len(), 1, "no verb request left the client: {requests:?}");
    assert_eq!(requests[0].method, "GET");
    assert!(requests[0].path.ends_with("/binding"));
}

#[test]
fn incompatible_api_version_stops_the_verb_before_any_request() {
    let mock = mock_server(vec![(
        "GET",
        "/binding",
        200,
        r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"2","engineVersion":"9.0.0"}"#
            .into(),
    )]);
    let p = TempProject::remote("version", &mock.base, Some("backend"));

    let out = p.run(&["status", "--change", "demo"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("API version") && stderr.contains("upgrade"),
        "stderr names the version incompatibility: {stderr}"
    );
    assert_eq!(mock.requests().len(), 1, "the verb request never left");
}

// --- the existing connection file works unchanged ---

#[test]
fn existing_speclink_yaml_remote_section_works_without_modification() {
    let mock = mock_server(vec![
        (
            "GET",
            "/binding",
            200,
            r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0"}"#
                .into(),
        ),
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","summary":"Demo change summary","status":"done","completedTasks":2,"totalTasks":2}]}"#
                .into(),
        ),
    ]);
    // The pre-handshake connection file shape: url + repo key, unchanged.
    let p = TempProject::remote("unchanged", &mock.base, Some("backend"));

    let out = p.run(&["list", "--json"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    assert_eq!(payload["changes"][0]["name"], "demo");
}

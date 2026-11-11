//! `speclink auth login`'s method flags against a fake server
//! (client-browser-login D8；規格「登入方式的探索與選擇」).
//!
//! These runs drive the real binary, so every case here ends before a
//! credential could be stored: the binary would reach for the real OS keychain,
//! and on a developer's machine it would open a real browser. The flows that
//! do store a credential are pinned at the function level, in
//! `src/verbs/connection.rs`, against an in-memory keyring.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};

/// A server that answers `auth-methods` as told and 404s everything else,
/// recording every path it was asked for.
struct Fake {
    server: Arc<tiny_http::Server>,
    url: String,
    paths: Arc<Mutex<Vec<String>>>,
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

fn fake(auth_methods: (u16, &'static str)) -> Fake {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let url = format!("http://{}/api/speclink/v1/projects/demo", server.server_addr());
    let paths = Arc::new(Mutex::new(Vec::new()));
    let (looper, log) = (server.clone(), paths.clone());
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().to_string();
            let (status, body) = if path == "/api/speclink/v1/auth-methods" {
                auth_methods
            } else {
                (404, r#"{"status":404,"reason":"not_found","message":"no route"}"#)
            };
            log.lock().unwrap().push(path);
            let resp = tiny_http::Response::from_string(body).with_status_code(status).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = req.respond(resp);
        }
    });
    Fake { server, url, paths }
}

impl Fake {
    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

/// A throwaway project connected to `url`, with a throwaway home.
struct Project {
    dir: PathBuf,
    home: PathBuf,
}

impl Project {
    fn new(tag: &str, url: &str) -> Project {
        let base = std::env::temp_dir().join(format!("speclink-cli-login-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (dir, home) = (base.join("project"), base.join("home"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), format!("remote:\n  url: {url}\n")).unwrap();
        Project { dir, home }
    }

    /// Run with a piped (non-terminal) stdin.
    fn run(&self, args: &[&str], stdin: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_TOKEN")
            .env_remove("SPECLINK_STORE_URL")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink");
        child.stdin.as_mut().unwrap().write_all(stdin.as_bytes()).unwrap();
        child.wait_with_output().expect("wait speclink")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const DEVICE_ONLY: (u16, &str) = (200, r#"{"methods":["device"]}"#);

#[test]
fn any_two_login_flags_conflict_before_any_request() {
    let server = fake(DEVICE_ONLY);
    let project = Project::new("conflict", &server.url);
    let flags = ["--browser", "--device", "--pat", "--token-stdin"];

    for (i, a) in flags.iter().enumerate() {
        for b in &flags[i + 1..] {
            let out = project.run(&["auth", "login", a, b], "");
            assert!(!out.status.success(), "{a} {b} must be refused");
            assert!(stderr_of(&out).contains("cannot be used with"), "{a} {b}: {}", stderr_of(&out));
        }
    }
    assert!(server.paths().is_empty(), "a refused invocation sends nothing: {:?}", server.paths());
}

#[test]
fn browser_on_a_server_without_browser_login_lists_what_it_supports() {
    let server = fake(DEVICE_ONLY);
    let project = Project::new("browser-device-only", &server.url);

    let out = project.run(&["auth", "login", "--browser"], "");

    assert!(!out.status.success());
    let err = stderr_of(&out);
    assert!(err.contains("does not support browser login") && err.contains("device"), "{err}");
    assert_eq!(server.paths(), vec!["/api/speclink/v1/auth-methods".to_string()], "no login starts");
}

#[test]
fn browser_on_a_server_that_declares_nothing_says_so() {
    let server = fake((404, r#"{"status":404,"reason":"not_found","message":"no route"}"#));
    let project = Project::new("browser-undeclared", &server.url);

    let out = project.run(&["auth", "login", "--browser"], "");

    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("does not declare browser login"), "{}", stderr_of(&out));
    assert_eq!(server.paths(), vec!["/api/speclink/v1/auth-methods".to_string()]);
}

#[test]
fn device_on_a_token_only_server_lists_what_it_supports() {
    let server = fake((200, r#"{"methods":["token"]}"#));
    let project = Project::new("device-token-only", &server.url);

    let out = project.run(&["auth", "login", "--device"], "");

    assert!(!out.status.success());
    let err = stderr_of(&out);
    assert!(err.contains("does not support device login") && err.contains("token"), "{err}");
    assert_eq!(server.paths(), vec!["/api/speclink/v1/auth-methods".to_string()], "no device flow starts");
}

/// A broken server is a connection error, never a reason to try something else.
#[test]
fn a_failing_discovery_is_a_connection_error() {
    let server = fake((500, "boom"));
    let project = Project::new("discovery-5xx", &server.url);

    let out = project.run(&["auth", "login", "--browser"], "");

    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("server unavailable"), "{}", stderr_of(&out));
    assert_eq!(server.paths(), vec!["/api/speclink/v1/auth-methods".to_string()]);
}

#[test]
fn no_flag_without_a_terminal_sends_no_request() {
    let server = fake(DEVICE_ONLY);
    let project = Project::new("no-tty", &server.url);

    let out = project.run(&["auth", "login"], "");

    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("--token-stdin"), "{}", stderr_of(&out));
    assert!(server.paths().is_empty(), "{:?}", server.paths());
}

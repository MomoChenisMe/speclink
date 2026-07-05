//! Store-document read verbs: `speclink artifact cat` and `speclink language
//! show` behave identically in fs mode (reads the local file) and remote mode
//! (contract endpoint), and a missing document is a loud, semantic failure.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Arc;

// --- minimal mock server ---

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
}

fn mock_server(routes: Vec<(&'static str, &'static str, u16, String)>) -> MockServer {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            let hit = routes.iter().find(|(m, suffix, _, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, status, body)) => (*status, body.clone()),
                None => (
                    404,
                    r#"{"reason":"not_found","message":"missing","resource":"route","name":"?"}"#
                        .to_string(),
                ),
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
    MockServer { server, base }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

// --- throwaway projects ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-docverbs-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    fn fs_project(tag: &str) -> TempProject {
        let p = TempProject::new(tag);
        let change = p.dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nProposal body\n").unwrap();
        p
    }

    fn remote(tag: &str, url: &str) -> TempProject {
        let p = TempProject::new(tag);
        std::fs::write(
            p.dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: backend\n"),
        )
        .unwrap();
        p
    }

    fn run(&self, args: &[&str], token: Option<&str>) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        if let Some(t) = token {
            cmd.env("SPECLINK_TOKEN", t);
        }
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

// --- artifact cat ---

#[test]
fn fs_artifact_cat_prints_the_document() {
    let p = TempProject::fs_project("cat-fs");
    let out = p.run(&["artifact", "cat", "proposal", "--change", "demo"], None);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "## Why\n\nProposal body\n");
}

#[test]
fn fs_artifact_cat_missing_document_fails_semantically() {
    let p = TempProject::fs_project("cat-fs-missing");
    let out = p.run(&["artifact", "cat", "design", "--change", "demo"], None);
    assert!(!out.status.success(), "missing artifact → non-zero exit");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("design"), "names the artifact: {stderr}");
}

#[test]
fn remote_artifact_cat_prints_the_endpoint_content() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/artifacts/proposal",
        200,
        r###"{"artifact":"proposal","content":"## Why\n\nProposal body\n","version":3}"###.into(),
    )]);
    let p = TempProject::remote("cat-remote", &mock.base);
    let out = p.run(&["artifact", "cat", "proposal", "--change", "demo"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    // Both modes print the same bytes for the same document.
    assert_eq!(stdout_of(&out), "## Why\n\nProposal body\n");
}

#[test]
fn remote_artifact_cat_missing_document_fails_semantically() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/artifacts/design",
        404,
        r#"{"reason":"not_found","message":"missing","resource":"artifact","name":"design"}"#.into(),
    )]);
    let p = TempProject::remote("cat-remote-missing", &mock.base);
    let out = p.run(&["artifact", "cat", "design", "--change", "demo"], Some("tok"));
    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("design"), "names the artifact: {stderr}");
    assert!(!stderr.contains("404"), "no bare status: {stderr}");
}

// --- language show ---

#[test]
fn fs_language_show_prints_the_vocabulary() {
    let p = TempProject::fs_project("lang-fs");
    std::fs::write(
        p.dir.join("openspec").join("LANGUAGE.md"),
        "# Language\n\n- term: 客戶\n",
    )
    .unwrap();
    let out = p.run(&["language", "show"], None);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "# Language\n\n- term: 客戶\n");
}

#[test]
fn fs_language_show_missing_fails_semantically() {
    let p = TempProject::fs_project("lang-fs-missing");
    let out = p.run(&["language", "show"], None);
    assert!(!out.status.success(), "missing vocabulary → non-zero exit");
    let stderr = stderr_of(&out).to_ascii_lowercase();
    assert!(stderr.contains("language"), "names the document: {stderr}");
}

#[test]
fn remote_language_show_prints_the_endpoint_content() {
    let mock = mock_server(vec![(
        "GET",
        "/language",
        200,
        r###"{"content":"# Language\n\n- term: 客戶\n"}"###.into(),
    )]);
    let p = TempProject::remote("lang-remote", &mock.base);
    let out = p.run(&["language", "show"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "# Language\n\n- term: 客戶\n");
}

#[test]
fn remote_language_show_missing_fails_semantically() {
    let mock = mock_server(vec![(
        "GET",
        "/language",
        404,
        r#"{"reason":"not_found","message":"missing","resource":"language","name":"LANGUAGE"}"#.into(),
    )]);
    let p = TempProject::remote("lang-remote-missing", &mock.base);
    let out = p.run(&["language", "show"], Some("tok"));
    assert!(!out.status.success());
    let stderr = stderr_of(&out).to_ascii_lowercase();
    assert!(stderr.contains("language"), "names the document: {stderr}");
}

//! CLI wiring for `discuss new --slug` (change discuss-english-slug): the
//! override names the record file and frontmatter slug while the topic stays
//! verbatim; invalid values must fail loudly without touching the filesystem;
//! the no-flag fallback derivation is unchanged.
//!
//! Remote 分支（change remote-cli-parity）：--slug 隨建立請求上 wire，驗證
//! 單一事實來源在引擎（server 端）——CLI 不預驗、引擎拒絕訊息逐字呈現。

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A throwaway project with an empty discussions directory.
    fn empty(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-discuss-slug-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("discussions")).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            // Plain output must be deterministic regardless of the host shell.
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn discussions(&self) -> PathBuf {
        self.dir.join("openspec").join("discussions")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn discuss_new_with_slug_override_names_file_and_keeps_topic() {
    let p = TempProject::empty("valid");
    let out = p.run(&["discuss", "new", "看板搜尋列", "--slug", "board-search-bar"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("board-search-bar"), "stdout: {stdout}");
    let file = p.discussions().join("board-search-bar.md");
    let text = std::fs::read_to_string(&file).expect("record exists under override slug");
    assert!(text.contains("slug: board-search-bar\n"), "text: {text}");
    assert!(text.contains("topic: 看板搜尋列\n"), "text: {text}");
}

#[test]
fn discuss_new_with_slug_json_reports_override_and_topic() {
    let p = TempProject::empty("json");
    let out = p.run(&["discuss", "new", "看板搜尋列", "--slug", "board-x", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json stdout");
    assert_eq!(v["slug"], "board-x");
    assert_eq!(v["topic"], "看板搜尋列");
}

#[test]
fn discuss_new_rejects_invalid_slug_without_writing() {
    let p = TempProject::empty("invalid");
    let out = p.run(&["discuss", "new", "主題", "--slug", "Bad_Slug"]);
    assert!(!out.status.success(), "invalid slug must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("kebab-case"), "stderr: {stderr}");
    let leftover: Vec<_> = std::fs::read_dir(p.discussions()).unwrap().collect();
    assert!(leftover.is_empty(), "no record may be created: {leftover:?}");
}

#[test]
fn discuss_new_without_slug_derives_from_topic_as_before() {
    let p = TempProject::empty("fallback");
    let out = p.run(&["discuss", "new", "Board Search"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(p.discussions().join("board-search.md").exists(), "derived filename unchanged");
}

// --- remote 分支（spec「討論動詞於 remote 模式與本機同語意」）---

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
    body: String,
}

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
    captured: Arc<Mutex<Vec<Captured>>>,
}

const BINDING_BODY: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;

fn mock_server(mut routes: Vec<(&'static str, &'static str, u16, String)>) -> MockServer {
    if !routes.iter().any(|(_, suffix, _, _)| *suffix == "/binding") {
        routes.push(("GET", "/binding", 200, BINDING_BODY.to_string()));
    }
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let captured = Arc::new(Mutex::new(Vec::new()));
    let looper = Arc::clone(&server);
    let sink = Arc::clone(&captured);
    std::thread::spawn(move || {
        for mut req in looper.incoming_requests() {
            let mut body = String::new();
            let _ = req.as_reader().read_to_string(&mut body);
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: path.clone(),
                body,
            });
            let hit = routes.iter().find(|(m, suffix, _, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, status, body)) => (*status, body.clone()),
                None => (
                    404,
                    r#"{"reason":"not_found","message":"no route"}"#.to_string(),
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
    MockServer { server, base, captured }
}

impl MockServer {
    fn captured_posts(&self, path_suffix: &str) -> Vec<Captured> {
        self.captured
            .lock()
            .unwrap()
            .iter()
            .filter(|c| {
                c.method == "POST"
                    && c.path == format!("/api/speclink/v1/projects/demo{path_suffix}")
            })
            .cloned()
            .collect()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

fn remote_project(tag: &str, url: &str) -> TempProject {
    let dir = std::env::temp_dir().join(format!(
        "speclink-cli-discuss-slug-{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(".speclink.yaml"),
        format!("remote:\n  url: {url}\n  repo: backend\n"),
    )
    .unwrap();
    TempProject { dir }
}

fn run_remote(p: &TempProject, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_speclink"))
        .args(args)
        .current_dir(&p.dir)
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("SPECLINK_STORE_URL")
        .env("SPECLINK_TOKEN", "tok")
        .output()
        .expect("run speclink binary")
}

#[test]
fn remote_discuss_new_with_slug_posts_override_and_matches_fs_output_shape() {
    let mock = mock_server(vec![(
        "POST",
        "/discussions",
        201,
        r#"{"slug":"board-search-bar","topic":"看板搜尋列","path":"discussions/board-search-bar.md"}"#.into(),
    )]);
    let p = remote_project("remote-valid", &mock.base);
    let out = run_remote(&p, &["discuss", "new", "看板搜尋列", "--slug", "board-search-bar"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // fs 模式同形的三行建立訊息。
    assert!(stdout.contains("Created discussion: board-search-bar"), "stdout: {stdout}");
    assert!(stdout.contains("Topic: 看板搜尋列"), "stdout: {stdout}");
    let posts = mock.captured_posts("/discussions");
    assert_eq!(posts.len(), 1, "exactly one create request");
    let body: serde_json::Value = serde_json::from_str(&posts[0].body).unwrap();
    assert_eq!(body["slug"], "board-search-bar", "the override rides the request");
    assert_eq!(body["topic"], "看板搜尋列", "the CJK topic stays verbatim");

    let out = run_remote(&p, &["discuss", "new", "看板搜尋列", "--slug", "board-search-bar", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json stdout");
    assert_eq!(v["slug"], "board-search-bar", "--json slug field matches fs mode's");
    assert_eq!(v["topic"], "看板搜尋列");
}

#[test]
fn remote_discuss_new_invalid_slug_fails_loudly_with_the_engine_message() {
    // D1：驗證單一事實來源在引擎——CLI 不預驗，server 的引擎拒絕逐字上 stderr。
    let mock = mock_server(vec![(
        "POST",
        "/discussions",
        400,
        r#"{"status":400,"reason":"invalid_argument","message":"invalid slug '中文slug' — must be ASCII kebab-case: lowercase letters/digits separated by single hyphens (e.g. board-search-bar)"}"#.into(),
    )]);
    let p = remote_project("remote-invalid", &mock.base);
    let out = run_remote(&p, &["discuss", "new", "看板搜尋列", "--slug", "中文slug"]);
    assert!(!out.status.success(), "invalid slug must exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("kebab-case"), "engine message relayed: {stderr}");
    let posts = mock.captured_posts("/discussions");
    assert_eq!(posts.len(), 1, "no silent slug-less retry after the refusal");
    let body: serde_json::Value = serde_json::from_str(&posts[0].body).unwrap();
    assert_eq!(body["slug"], "中文slug", "the CLI sends the value for the engine to judge");
}

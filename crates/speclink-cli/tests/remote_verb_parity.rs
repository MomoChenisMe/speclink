//! Remote verb parity (remote-verb-parity 刀): validate/analyze 的 remote 分流
//! 與 fs 模式同形輸出、discard 由拒絕改為實作且 guard 訊息與本地一致。
//! 沿 remote_write_path.rs 的 capturing mock server 模式。

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

// --- capturing mock verb-contract server（多一個 query 欄位供 force 斷言）---

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
    query: String,
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
            let mut parts = req.url().splitn(2, '?');
            let path = parts.next().unwrap_or_default().to_string();
            let query = parts.next().unwrap_or_default().to_string();
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: path.clone(),
                query,
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
    fn find(&self, method: &str, path_suffix: &str) -> Captured {
        let caps = self.captured.lock().unwrap();
        caps.iter()
            .find(|c| {
                c.method == method
                    && c.path == format!("/api/speclink/v1/projects/demo{path_suffix}")
            })
            .unwrap_or_else(|| panic!("no captured {method} {path_suffix}; got {caps:?}"))
            .clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn remote(tag: &str, url: &str, repo: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-remote-parity-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: {repo}\n"),
        )
        .unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_STORE_URL")
            .env("SPECLINK_TOKEN", "tok")
            .output()
            .expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

// --- validate：無參數聚合由 client 以逐 change 端點組合（design 決策 2）---

#[test]
fn remote_validate_aggregates_per_change_endpoints_with_fs_parity_json() {
    const ERR: &str = "openspec/changes/beta/specs/auth/spec.md: Parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)";
    let mock = mock_server(vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"alpha","status":"in-progress","completedTasks":0,"totalTasks":2},{"name":"beta","status":"in-progress","completedTasks":0,"totalTasks":1}]}"#.into(),
        ),
        (
            "GET",
            "/changes/alpha/validate",
            200,
            r#"{"change":"alpha","valid":true,"errors":[],"warnings":[]}"#.into(),
        ),
        (
            "GET",
            "/changes/beta/validate",
            200,
            format!(r#"{{"change":"beta","valid":false,"errors":["{ERR}"],"warnings":["No delta specs found"]}}"#),
        ),
    ]);
    let p = TempProject::remote("validate-agg", &mock.base, "backend");
    let out = p.run(&["validate", "--json"]);
    assert!(!out.status.success(), "any invalid change must exit non-zero");
    assert!(
        stderr_of(&out).contains("Validation failed."),
        "fs-parity failure message: {}",
        stderr_of(&out)
    );

    // fs 模式 --json 形狀：頂層陣列、每筆 change/errors/valid/warnings。
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&out).trim()).expect("stdout is JSON");
    assert_eq!(
        payload,
        serde_json::json!([
            { "change": "alpha", "errors": [], "valid": true, "warnings": [] },
            { "change": "beta", "errors": [ERR], "valid": false, "warnings": ["No delta specs found"] },
        ]),
        "aggregate output must match the fs-mode shape per change"
    );

    // client 組合語意：先 list，再逐 change 打單 change 端點。
    mock.find("GET", "/changes");
    mock.find("GET", "/changes/alpha/validate");
    mock.find("GET", "/changes/beta/validate");
}

#[test]
fn remote_validate_all_valid_exits_zero() {
    let mock = mock_server(vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"alpha","status":"in-progress","completedTasks":0,"totalTasks":2}]}"#.into(),
        ),
        (
            "GET",
            "/changes/alpha/validate",
            200,
            r#"{"change":"alpha","valid":true,"errors":[],"warnings":[]}"#.into(),
        ),
    ]);
    let p = TempProject::remote("validate-ok", &mock.base, "backend");
    let out = p.run(&["validate", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&out).trim()).expect("stdout is JSON");
    assert_eq!(payload[0]["valid"], true);
}

// --- analyze：單 change 端點、輸出與 fs 模式同形（snake_case 的引擎序列化）---

#[test]
fn remote_analyze_prints_fs_parity_json() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/analyze",
        200,
        r#"{"changeId":"demo","dimensions":[{"dimension":"Coverage","status":"Clean","findingCount":0},{"dimension":"Ambiguity","status":"1 issue(s) found","findingCount":1}],"findings":[{"id":"AMB-1","dimension":"Ambiguity","severity":"Suggestion","location":"specs/auth/spec.md","summary":"Scenario 'X' has no concrete examples","recommendation":"Add ##### Example:","summaryMsg":{"key":"ambAbstractScenario.summary","params":{"scenario":"X"}},"recommendationMsg":{"key":"ambAbstractScenario.recommendation","params":{"scenario":"X"}}}],"artifactsAnalyzed":["proposal.md"],"artifactsMissing":["design.md"]}"#.into(),
    )]);
    let p = TempProject::remote("analyze", &mock.base, "backend");
    let out = p.run(&["analyze", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    // fs 模式 print_json(&AnalyzeReport)：snake_case 欄位的完整報告。
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&out).trim()).expect("stdout is JSON");
    assert_eq!(
        payload,
        serde_json::json!({
            "change_id": "demo",
            "dimensions": [
                { "dimension": "Coverage", "status": "Clean", "finding_count": 0 },
                { "dimension": "Ambiguity", "status": "1 issue(s) found", "finding_count": 1 },
            ],
            "findings": [{
                "id": "AMB-1",
                "dimension": "Ambiguity",
                "severity": "Suggestion",
                "location": "specs/auth/spec.md",
                "summary": "Scenario 'X' has no concrete examples",
                "recommendation": "Add ##### Example:",
                "summary_msg": { "key": "ambAbstractScenario.summary", "params": { "scenario": "X" } },
                "recommendation_msg": { "key": "ambAbstractScenario.recommendation", "params": { "scenario": "X" } },
            }],
            "artifacts_analyzed": ["proposal.md"],
            "artifacts_missing": ["design.md"],
        }),
        "remote analyze --json must match the fs-mode engine serialization"
    );
}

// --- discard：由 remote 拒絕改為實作；guard 訊息與本地一致 ---

#[test]
fn remote_discard_without_force_translates_the_guard_message() {
    let mock = mock_server(vec![(
        "DELETE",
        "/changes/demo",
        409,
        r#"{"status":409,"reason":"refused","message":"change 'demo' has started work (started_at set or tasks checked) — discard refuses to delete it; pass --force to discard anyway"}"#.into(),
    )]);
    let p = TempProject::remote("discard-guard", &mock.base, "backend");
    let out = p.run(&["discard", "demo"]);
    assert!(!out.status.success(), "guard refusal must exit non-zero");
    let stderr = stderr_of(&out);
    assert!(stderr.contains("started work"), "names the guard: {stderr}");
    assert!(stderr.contains("--force"), "points at --force like fs mode: {stderr}");
    assert!(!stderr.contains("409"), "no bare status: {stderr}");
    assert!(
        !stderr.contains("not available in remote mode"),
        "the remote refusal is gone: {stderr}"
    );

    let cap = mock.find("DELETE", "/changes/demo");
    assert!(
        !cap.query.contains("force=true"),
        "no --force must never send force=true: {}",
        cap.query
    );
}

#[test]
fn remote_discard_with_force_succeeds_with_fs_parity_json() {
    let mock = mock_server(vec![(
        "DELETE",
        "/changes/demo",
        200,
        r#"{"change":"demo","unlinkedDiscussions":[{"slug":"auth","status":"concluded"}]}"#.into(),
    )]);
    let p = TempProject::remote("discard-force", &mock.base, "backend");
    let out = p.run(&["discard", "demo", "--force", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    // fs 模式 cmd_discard 的 --json 形狀。
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&out).trim()).expect("stdout is JSON");
    assert_eq!(
        payload,
        serde_json::json!({
            "change": "demo",
            "unlinkedDiscussions": [{ "slug": "auth", "status": "concluded" }],
        })
    );

    let cap = mock.find("DELETE", "/changes/demo");
    assert!(cap.query.contains("force=true"), "--force rides the query: {}", cap.query);
}

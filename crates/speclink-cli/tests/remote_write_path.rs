//! Remote write-path verbs against a capturing mock server: success shapes,
//! every 409 reason a write verb can hit, the repo-identity chain
//! (X-Speclink-Repo on every request, repo_mismatch naming both repos), and
//! change→repo attribution behaviors.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};

// --- capturing mock verb-contract server ---

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
    headers: Vec<(String, String)>, // lowercased names
    body: String,
}

impl Captured {
    fn header(&self, name: &str) -> Option<&str> {
        let want = name.to_ascii_lowercase();
        self.headers.iter().find(|(k, _)| *k == want).map(|(_, v)| v.as_str())
    }
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
        for mut req in looper.incoming_requests() {
            let mut body = String::new();
            let _ = req.as_reader().read_to_string(&mut body);
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            let cap = Captured {
                method: req.method().to_string(),
                path: path.clone(),
                headers: req
                    .headers()
                    .iter()
                    .map(|h| (h.field.to_string().to_ascii_lowercase(), h.value.to_string()))
                    .collect(),
                body,
            };
            sink.lock().unwrap().push(cap);
            let hit = routes.iter().find(|(m, suffix, _, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, status, body)) => (*status, body.clone()),
                None => (
                    404,
                    r#"{"reason":"not_found","message":"no route","resource":"route","name":"?"}"#
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

// --- throwaway remote project ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn remote(tag: &str, url: &str, repo: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-remote-write-{tag}-{}",
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

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env("SPECLINK_TOKEN", "tok");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], stdin: &str) -> Output {
        let mut child = self
            .cmd(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().expect("wait speclink binary")
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

// --- new change: attribution to the current repo ---

#[test]
fn new_change_posts_with_repo_header_and_name() {
    let mock = mock_server(vec![(
        "POST",
        "/changes",
        201,
        r#"{"name":"demo","schema":"spec-driven","repo":"backend","lifecycle":"drafting"}"#.into(),
    )]);
    let p = TempProject::remote("new-change", &mock.base, "backend");
    let out = p.run(&["new", "change", "demo", "--agent", "claude"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("Created change: demo"), "stdout: {}", stdout_of(&out));

    let cap = mock.find("POST", "/changes");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"), "creation carries the repo identity");
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert_eq!(body["name"], "demo");
    assert_eq!(body["agent"], "claude");
    // No local spec tree appears — the change lives on the server.
    assert!(!p.dir.join("openspec").exists(), "no local openspec/ tree in remote mode");
}

#[test]
fn new_change_conflict_reports_already_exists() {
    let mock = mock_server(vec![(
        "POST",
        "/changes",
        409,
        r#"{"reason":"already_exists","message":"taken","name":"demo"}"#.into(),
    )]);
    let p = TempProject::remote("new-change-conflict", &mock.base, "backend");
    let out = p.run(&["new", "change", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("already in use"), "stderr: {}", stderr_of(&out));
}

// --- list: repo filtering is server-side, keyed by the request header ---

#[test]
fn list_carries_repo_header_and_prints_only_the_servers_changes() {
    // The server (which filters by repo) returns only backend's change; the
    // CLI adds nothing of its own.
    let mock = mock_server(vec![(
        "GET",
        "/changes",
        200,
        r#"{"changes":[{"name":"backend-change","status":"in-progress","completedTasks":0,"totalTasks":2,"repo":"backend","lifecycle":"ready"}]}"#.into(),
    )]);
    let p = TempProject::remote("list-filter", &mock.base, "backend");
    let out = p.run(&["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload: serde_json::Value = serde_json::from_str(&stdout_of(&out)).unwrap();
    let names: Vec<&str> = payload["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["backend-change"], "another repo's change never appears");
    let cap = mock.find("GET", "/changes");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"), "list carries the repo for server-side filtering");
}

// --- new artifact: If-Match create-only write ---

#[test]
fn new_artifact_puts_content_with_if_match_zero() {
    let mock = mock_server(vec![(
        "PUT",
        "/changes/demo/artifacts/design",
        200,
        r#"{"artifact":"design","version":1}"#.into(),
    )]);
    let p = TempProject::remote("new-artifact", &mock.base, "backend");
    let out = p.run_stdin(
        &["new", "artifact", "design", "--change", "demo", "--stdin"],
        "## Context\n\nDesign body\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let cap = mock.find("PUT", "/changes/demo/artifacts/design");
    assert_eq!(cap.header("if-match"), Some("0"), "creation asserts the artifact does not exist yet");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"));
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert!(
        body["content"].as_str().unwrap().contains("Design body"),
        "PUT carries the full document"
    );
}

#[test]
fn artifact_version_conflict_suggests_rereading() {
    let mock = mock_server(vec![(
        "PUT",
        "/changes/demo/artifacts/design",
        409,
        r#"{"reason":"version_conflict","message":"stale","currentVersion":4}"#.into(),
    )]);
    let p = TempProject::remote("artifact-conflict", &mock.base, "backend");
    let out = p.run_stdin(
        &["new", "artifact", "design", "--change", "demo", "--stdin"],
        "content",
    );
    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("re-read"), "stderr: {stderr}");
    assert!(!stderr.contains("409"), "no bare status: {stderr}");
}

#[test]
fn artifact_write_by_non_owner_reports_ownership_lost() {
    let mock = mock_server(vec![(
        "PUT",
        "/changes/demo/artifacts/design",
        409,
        r#"{"reason":"ownership_lost","message":"claimed","claimedBy":"chiang"}"#.into(),
    )]);
    let p = TempProject::remote("artifact-owner", &mock.base, "backend");
    let out = p.run_stdin(
        &["new", "artifact", "design", "--change", "demo", "--stdin"],
        "content",
    );
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("chiang"), "stderr: {}", stderr_of(&out));
}

// --- task done ---

#[test]
fn task_done_posts_and_prints_fs_parity_json() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/tasks/3/done",
        200,
        r#"{"change":"demo","taskId":"3","taskDesc":"1.3 Third","status":"done","alreadyDone":false,"tasksVersion":7}"#.into(),
    )]);
    let p = TempProject::remote("task-done", &mock.base, "backend");
    let out = p.run(&["task", "done", "3", "--change", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    // fs parity: compact single-line JSON with exactly these keys.
    let payload: serde_json::Value = serde_json::from_str(stdout_of(&out).trim()).unwrap();
    let keys: Vec<&str> = payload.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["change", "status", "task_desc", "task_id"]);
    assert_eq!(payload["task_desc"], "1.3 Third");

    let cap = mock.find("POST", "/changes/demo/tasks/3/done");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"));
}

#[test]
fn task_done_during_ingest_says_wait() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/tasks/3/done",
        409,
        r#"{"reason":"change_busy","message":"busy","lifecycle":"busy"}"#.into(),
    )]);
    let p = TempProject::remote("task-busy", &mock.base, "backend");
    let out = p.run(&["task", "done", "3", "--change", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("wait"), "stderr: {}", stderr_of(&out));
}

// --- task undone ---

#[test]
fn task_undone_posts_empty_body_and_prints_fs_parity_json() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/tasks/3/undone",
        200,
        r#"{"change":"demo","taskId":"3","taskDesc":"1.3 Third","status":"undone","alreadyUndone":false,"tasksVersion":8}"#.into(),
    )]);
    let p = TempProject::remote("task-undone", &mock.base, "backend");
    let out = p.run(&["task", "undone", "3", "--change", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    // fs parity: byte-identical compact single-line payload (keys and order included).
    assert_eq!(
        stdout_of(&out),
        "{\"change\":\"demo\",\"status\":\"undone\",\"task_desc\":\"1.3 Third\",\"task_id\":\"3\"}\n"
    );

    let cap = mock.find("POST", "/changes/demo/tasks/3/undone");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"));
    // Unchecking records no touched files — the body is an empty JSON object.
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert_eq!(body, serde_json::json!({}), "request body must be an empty object: {}", cap.body);
}

#[test]
fn task_undone_human_output_matches_fs_mode() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/tasks/3/undone",
        200,
        r#"{"change":"demo","taskId":"3","taskDesc":"1.3 Third","status":"undone","alreadyUndone":false,"tasksVersion":8}"#.into(),
    )]);
    let p = TempProject::remote("task-undone-human", &mock.base, "backend");
    let out = p.run(&["task", "undone", "3", "--change", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ Task 3 marked as not done: 1.3 Third\n");
}

#[test]
fn task_undone_already_translates_to_already_not_done_error() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/tasks/3/undone",
        200,
        r#"{"change":"demo","taskId":"3","taskDesc":"1.3 Third","status":"undone","alreadyUndone":true,"tasksVersion":8}"#.into(),
    )]);
    let p = TempProject::remote("task-undone-already", &mock.base, "backend");
    let out = p.run(&["task", "undone", "3", "--change", "demo"]);
    assert!(!out.status.success(), "alreadyUndone must exit non-zero");
    assert_eq!(stderr_of(&out), "Error: Task 3 is already not done\n");
    assert_eq!(stdout_of(&out), "");
}

// --- claim: atomicity outcomes and the repo chain ---

#[test]
fn claim_success_reports_the_claim() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/claim",
        200,
        r#"{"claimed":true,"claimedBy":"me","statusVersion":5}"#.into(),
    )]);
    let p = TempProject::remote("claim-ok", &mock.base, "backend");
    let out = p.run(&["claim", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("demo"), "stdout: {}", stdout_of(&out));
    let cap = mock.find("POST", "/changes/demo/claim");
    assert_eq!(cap.header("x-speclink-repo"), Some("backend"));
}

#[test]
fn claim_preempted_names_the_holder() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/claim",
        409,
        r#"{"reason":"ownership_lost","message":"claimed","claimedBy":"chiang"}"#.into(),
    )]);
    let p = TempProject::remote("claim-lost", &mock.base, "backend");
    let out = p.run(&["claim", "demo"]);
    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("chiang"), "holder named: {stderr}");
    assert!(!stderr.contains("409"), "no bare status: {stderr}");
}

#[test]
fn claim_in_wrong_repo_names_both_repos() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/add-rate-limit/claim",
        403,
        r#"{"reason":"repo_mismatch","message":"wrong repo","changeRepo":"backend","requestRepo":"frontend"}"#.into(),
    )]);
    let p = TempProject::remote("claim-mismatch", &mock.base, "frontend");
    let out = p.run(&["claim", "add-rate-limit"]);
    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("backend"), "owning repo named: {stderr}");
    assert!(stderr.contains("frontend"), "current repo named: {stderr}");
}

#[test]
fn claim_while_gate_pending_points_at_the_approver() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/claim",
        409,
        r#"{"reason":"gate_pending","message":"pending","gate":"proposal"}"#.into(),
    )]);
    let p = TempProject::remote("claim-gate", &mock.base, "backend");
    let out = p.run(&["claim", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("approval"), "stderr: {}", stderr_of(&out));
}

// --- archive: check-all-then-apply outcomes ---

#[test]
fn archive_success_reports_archived() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/archive",
        200,
        r#"{"archived":true,"change":"demo","specs":[{"capability":"cap-a","version":6}]}"#.into(),
    )]);
    let p = TempProject::remote("archive-ok", &mock.base, "backend");
    let out = p.run(&["archive", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("demo"), "stdout: {}", stdout_of(&out));
}

#[test]
fn archive_with_open_tasks_counts_the_remainder() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/archive",
        409,
        r#"{"reason":"tasks_incomplete","message":"open","remaining":3}"#.into(),
    )]);
    let p = TempProject::remote("archive-tasks", &mock.base, "backend");
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains('3'), "stderr: {}", stderr_of(&out));
}

#[test]
fn archive_spec_conflict_names_the_capabilities() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/archive",
        409,
        r#"{"reason":"version_conflict","message":"moved","conflicts":[{"capability":"cap-a","baseVersion":3,"currentVersion":5}]}"#.into(),
    )]);
    let p = TempProject::remote("archive-conflict", &mock.base, "backend");
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("cap-a"), "stderr: {}", stderr_of(&out));
}

// --- discuss write verbs ---

#[test]
fn discuss_new_posts_the_topic() {
    let mock = mock_server(vec![(
        "POST",
        "/discussions",
        201,
        r#"{"slug":"demo-topic","topic":"Demo topic","path":"discussions/demo-topic.md"}"#.into(),
    )]);
    let p = TempProject::remote("disc-new", &mock.base, "backend");
    let out = p.run(&["discuss", "new", "Demo topic"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("demo-topic"), "stdout: {}", stdout_of(&out));
    let cap = mock.find("POST", "/discussions");
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert_eq!(body["topic"], "Demo topic");
}

#[test]
fn discuss_context_puts_the_content() {
    let mock = mock_server(vec![(
        "PUT",
        "/discussions/demo-topic/context",
        200,
        r#"{"slug":"demo-topic","context":"set"}"#.into(),
    )]);
    let p = TempProject::remote("disc-context", &mock.base, "backend");
    let out = p.run_stdin(&["discuss", "context", "demo-topic", "--stdin"], "Some context\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let cap = mock.find("PUT", "/discussions/demo-topic/context");
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert!(body["content"].as_str().unwrap().contains("Some context"));
}

#[test]
fn discuss_add_round_posts_mode_and_content() {
    let mock = mock_server(vec![(
        "POST",
        "/discussions/demo-topic/rounds",
        200,
        r#"{"slug":"demo-topic","round":2,"mode":"assumptions"}"#.into(),
    )]);
    let p = TempProject::remote("disc-round", &mock.base, "backend");
    let out = p.run_stdin(
        &["discuss", "add-round", "demo-topic", "--mode", "assumptions", "--stdin"],
        "**Focus**: something\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("round 2"), "stdout: {}", stdout_of(&out));
    let cap = mock.find("POST", "/discussions/demo-topic/rounds");
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert_eq!(body["mode"], "assumptions");
    assert!(body["content"].as_str().unwrap().contains("Focus"));
}

#[test]
fn discuss_conclude_posts_the_conclusion() {
    let mock = mock_server(vec![(
        "POST",
        "/discussions/demo-topic/conclude",
        200,
        r#"{"slug":"demo-topic","status":"concluded"}"#.into(),
    )]);
    let p = TempProject::remote("disc-conclude", &mock.base, "backend");
    let out = p.run_stdin(&["discuss", "conclude", "demo-topic", "--stdin"], "**Decision**: do it\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let cap = mock.find("POST", "/discussions/demo-topic/conclude");
    let body: serde_json::Value = serde_json::from_str(&cap.body).unwrap();
    assert!(body["content"].as_str().unwrap().contains("Decision"));
}

#[test]
fn discuss_round_on_archived_discussion_reports_it() {
    let mock = mock_server(vec![(
        "POST",
        "/discussions/old-topic/rounds",
        409,
        r#"{"reason":"discussion_archived","message":"archived","slug":"old-topic"}"#.into(),
    )]);
    let p = TempProject::remote("disc-archived", &mock.base, "backend");
    let out = p.run_stdin(
        &["discuss", "add-round", "old-topic", "--mode", "assumptions", "--stdin"],
        "content",
    );
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("old-topic"), "stderr: {}", stderr_of(&out));
}

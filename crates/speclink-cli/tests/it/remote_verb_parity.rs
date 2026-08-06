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

    /// 對照用 fs 沙盒：本機 openspec 樹、無 remote 連線。
    fn fs(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-remote-parity-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec")).unwrap();
        TempProject { dir }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
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

// --- show：CLI 端讀取組合、與 fs 模式逐位元同形（design D4）---

const SHOW_META: &str = "schema: spec-driven\ncreated: 2026-07-29\nfrom_discussion: auth-scope\n";
const SHOW_PROPOSAL: &str = "## Why\n\n看板搜尋列需要入口。\n";
const SHOW_DESIGN: &str = "## Context\n\n設計內容。\n";
const SHOW_TASKS: &str = "- [ ] 1.1 第一項\n";
const SHOW_SPEC_DOC: &str = "# user-auth\n\n正典內容。\n";

/// 同一份 change 內容的 remote server 路由（wire 形狀 camelCase）。
fn show_change_routes() -> Vec<(&'static str, &'static str, u16, String)> {
    vec![
        ("GET", "/specs", 200, r#"{"specs":[]}"#.into()),
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":0,"totalTasks":1}]}"#.into(),
        ),
        (
            "GET",
            "/changes/demo",
            200,
            r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":false,"applyRequires":["tasks"],"artifacts":[],"created":"2026-07-29","fromDiscussions":["auth-scope"],"deltaCapabilities":["auth"]}"#.into(),
        ),
        (
            "GET",
            "/changes/demo/artifacts/proposal",
            200,
            serde_json::json!({ "artifact": "proposal", "content": SHOW_PROPOSAL, "version": 1 })
                .to_string(),
        ),
        (
            "GET",
            "/changes/demo/artifacts/design",
            200,
            serde_json::json!({ "artifact": "design", "content": SHOW_DESIGN, "version": 1 })
                .to_string(),
        ),
        (
            "GET",
            "/changes/demo/artifacts/tasks",
            200,
            serde_json::json!({ "artifact": "tasks", "content": SHOW_TASKS, "version": 1 })
                .to_string(),
        ),
    ]
}

/// 同一份內容的 fs 沙盒。
fn show_fs_project(tag: &str) -> TempProject {
    let p = TempProject::fs(tag);
    p.write("openspec/changes/demo/.openspec.yaml", SHOW_META);
    p.write("openspec/changes/demo/proposal.md", SHOW_PROPOSAL);
    p.write("openspec/changes/demo/design.md", SHOW_DESIGN);
    p.write("openspec/changes/demo/tasks.md", SHOW_TASKS);
    p.write("openspec/changes/demo/specs/auth/spec.md", "## ADDED Requirements\n");
    p
}

#[test]
fn remote_show_change_matches_fs_output_byte_for_byte() {
    let fs = show_fs_project("show-fs");
    let fs_human = fs.run(&["show", "demo"]);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));
    let fs_json = fs.run(&["show", "demo", "--json"]);
    assert!(fs_json.status.success(), "fs stderr: {}", stderr_of(&fs_json));

    let mock = mock_server(show_change_routes());
    let p = TempProject::remote("show-remote", &mock.base, "backend");
    assert!(!p.dir.join("openspec").exists(), "remote sandbox has no local store");

    let remote_human = p.run(&["show", "demo"]);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));
    assert_eq!(
        stdout_of(&remote_human),
        stdout_of(&fs_human),
        "human output is byte-identical to fs mode"
    );

    let remote_json = p.run(&["show", "demo", "--json"]);
    assert!(remote_json.status.success(), "remote stderr: {}", stderr_of(&remote_json));
    assert_eq!(
        stdout_of(&remote_json),
        stdout_of(&fs_json),
        "--json output is byte-identical to fs mode (camelCase field parity)"
    );

    // link 鑄鏈可經 show 觀察（discussion-docs spec）：payload 帶 from_discussion 鏈。
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&remote_json).trim()).expect("stdout is JSON");
    assert_eq!(payload["fromDiscussions"][0], "auth-scope");
    assert_eq!(payload["deltaSpecs"][0], "auth/spec.md");
    assert!(
        !p.dir.join("openspec").exists(),
        "the remote run never created or read a local store"
    );
}

#[test]
fn remote_show_spec_matches_fs_output_byte_for_byte() {
    let fs = TempProject::fs("show-spec-fs");
    fs.write("openspec/specs/user-auth/spec.md", SHOW_SPEC_DOC);
    let fs_human = fs.run(&["show", "user-auth"]);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));
    let fs_json = fs.run(&["show", "user-auth", "--json"]);
    assert!(fs_json.status.success(), "fs stderr: {}", stderr_of(&fs_json));

    let mock = mock_server(vec![
        (
            "GET",
            "/specs",
            200,
            r#"{"specs":[{"id":"user-auth","path":"specs/user-auth/spec.md"}]}"#.into(),
        ),
        (
            "GET",
            "/specs/user-auth/document",
            200,
            serde_json::json!({ "content": SHOW_SPEC_DOC }).to_string(),
        ),
    ]);
    let p = TempProject::remote("show-spec-remote", &mock.base, "backend");

    let remote_human = p.run(&["show", "user-auth"]);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));
    assert_eq!(stdout_of(&remote_human), stdout_of(&fs_human), "human output parity");

    let remote_json = p.run(&["show", "user-auth", "--json"]);
    assert!(remote_json.status.success(), "remote stderr: {}", stderr_of(&remote_json));
    assert_eq!(stdout_of(&remote_json), stdout_of(&fs_json), "--json output parity");
}

#[test]
fn remote_show_missing_item_is_a_semantic_error_with_engine_wording() {
    let mock = mock_server(vec![
        ("GET", "/specs", 200, r#"{"specs":[]}"#.into()),
        (
            "GET",
            "/changes/ghost",
            404,
            r#"{"status":404,"reason":"not_found","message":"Change 'ghost' not found."}"#.into(),
        ),
    ]);
    let p = TempProject::remote("show-missing", &mock.base, "backend");

    let out = p.run(&["show", "ghost"]);
    assert!(!out.status.success(), "a missing item must exit non-zero");
    assert!(
        stderr_of(&out).contains("Item 'ghost' not found as a change or spec."),
        "the engine's frozen wording: {}",
        stderr_of(&out)
    );

    let out = p.run(&["show", "ghost", "--item-type", "change"]);
    assert!(!out.status.success());
    assert!(
        stderr_of(&out).contains("Change 'ghost' not found."),
        "typed lookup names the type: {}",
        stderr_of(&out)
    );

    let out = p.run(&["show", "ghost", "--item-type", "spec"]);
    assert!(!out.status.success());
    assert!(
        stderr_of(&out).contains("Spec 'ghost' not found."),
        "typed lookup names the type: {}",
        stderr_of(&out)
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

// --- archive 的 wire 形狀（spec verify-evidence「archive trace 注入與零證據
// 提示」的介面面：封存不因 evidence 被擋，wire 上也就沒有放行參數）---

#[test]
fn remote_archive_query_carries_no_evidence_waiver() {
    let mock = mock_server(vec![(
        "POST",
        "/changes/demo/archive",
        200,
        r#"{"specs":[{"capability":"user-auth"}]}"#.to_string(),
    )]);
    let p = TempProject::remote("archive-no-waiver", &mock.base, "backend");
    assert!(p.run(&["archive", "demo"]).status.success());
    let cap = mock.find("POST", "/changes/demo/archive");
    assert!(
        !cap.query.contains("waive"),
        "the waiver parameter is gone from the wire: {}",
        cap.query
    );
    assert!(cap.query.contains("carryReview"), "carryReview still rides: {}", cap.query);
}

// --- review prepare／scope：remote workspace 使用同一 host resolver
//（change-diff-scope spec「remote workspace 使用同一 host resolver」）---

/// 在沙盒目錄跑 git（可帶固定日期 env，讓 commit SHA 決定性）。
fn run_git(dir: &std::path::Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_DATE", "2026-01-02T03:04:05Z")
        .env("GIT_COMMITTER_DATE", "2026-01-02T03:04:05Z")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?} failed");
}

/// git init＋固定身分＋只 commit `src/`（兩個沙盒得到相同 SHA 的前提）。
fn seed_git_src(p: &TempProject) {
    p.write("src/lib.rs", "fn demo() {}\n");
    run_git(&p.dir, &["init", "-q"]);
    run_git(&p.dir, &["config", "user.name", "Sandbox Tester"]);
    run_git(&p.dir, &["config", "user.email", "sandbox@example.com"]);
    run_git(&p.dir, &["add", "src"]);
    run_git(&p.dir, &["commit", "-q", "-m", "init"]);
}

/// host-local touched record（v1 通道）。
fn write_touched(p: &TempProject, change: &str, files: &[&str]) {
    let list: Vec<String> = files.iter().map(|f| format!("\"{f}\"")).collect();
    p.write(
        &format!(".speclink/touched/{change}.json"),
        &format!(
            "{{\"version\":2,\"change\":\"{change}\",\"touched\":[{{\"task_id\":\"1\",\"task_desc\":\"t\",\"files\":[{}]}}]}}",
            list.join(",")
        ),
    );
}

/// listing 的 `status` 只由任務完成度推導（未全完成即 `in-progress`），沒有
/// `proposed` 這個值；`startedAt` 才是「已開工」的事實來源。這裡刻意用未開工
/// 但任務未完成的真實形狀。
fn review_scope_routes() -> Vec<(&'static str, &'static str, u16, String)> {
    vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":0,"totalTasks":2}]}"#.into(),
        ),
        (
            "GET",
            "/changes/demo/review",
            404,
            r#"{"status":404,"reason":"not_found","message":"no review ticket for change 'demo'"}"#.into(),
        ),
    ]
}

#[test]
fn remote_review_prepare_writes_the_local_sidecar_and_posts_nothing() {
    // spec Scenario「remote scope 仍使用 local checkout」的 prepare 面：baseline
    // 建在本地 checkout 的 .speclink，server 只被讀、不收 sidecar。
    let mock = mock_server(review_scope_routes());
    let p = TempProject::remote("review-prepare", &mock.base, "backend");
    seed_git_src(&p);
    p.write("notes/local.txt", "scratch\n");
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty(), "initial capture is silent");
    let baseline = p.dir.join(".speclink").join("review-scopes").join("demo").join("baseline.json");
    let raw = std::fs::read_to_string(&baseline).expect("baseline written locally");
    assert!(raw.contains("\"confidence\": \"initial\""), "{raw}");
    let caps = mock.captured.lock().unwrap();
    assert!(
        caps.iter().all(|c| c.method == "GET"),
        "prepare must not upload anything: {caps:?}"
    );
}

#[test]
fn remote_review_prepare_reads_started_from_started_at_not_task_progress() {
    // listing 的 status 在任務未全完成時一律是 in-progress，拿它當「已開工」會
    // 讓 Apply 前的 prepare 永遠記成 late；已開工的事實只在 startedAt。
    let started = vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":0,"totalTasks":2,"startedAt":"2026-01-02"}]}"#.to_string(),
        ),
        (
            "GET",
            "/changes/demo/review",
            404,
            r#"{"status":404,"reason":"not_found","message":"no review ticket for change 'demo'"}"#.to_string(),
        ),
    ];
    let mock = mock_server(started);
    let p = TempProject::remote("review-prepare-started", &mock.base, "backend");
    seed_git_src(&p);
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let baseline = p.dir.join(".speclink").join("review-scopes").join("demo").join("baseline.json");
    let raw = std::fs::read_to_string(&baseline).expect("baseline written locally");
    assert!(raw.contains("\"confidence\": \"late\""), "startedAt present ⇒ late: {raw}");
    assert!(
        stderr_of(&out).contains("captured late"),
        "late capture warns: {}",
        stderr_of(&out)
    );
}

#[test]
fn remote_review_scope_uses_local_git_and_uploads_nothing() {
    // spec Scenario「remote scope 仍使用 local checkout」：resolved payload 用
    // local Git 產生、server 不收到 patch 或 snapshot。
    let mock = mock_server(review_scope_routes());
    let p = TempProject::remote("review-scope", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    write_touched(&p, "demo", &["src/lib.rs"]);
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved");
    // baseCommit 是本地 checkout 的 HEAD。
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&p.dir)
        .output()
        .expect("git rev-parse");
    let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    assert_eq!(v["baseCommit"].as_str(), Some(head.as_str()), "local Git is the source");
    // 本地 snapshot 凍結；server 全程只被 GET。
    let snapdir = p.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots");
    assert_eq!(std::fs::read_dir(&snapdir).map(|it| it.count()).unwrap_or(0), 1);
    let caps = mock.captured.lock().unwrap();
    assert!(
        caps.iter().all(|c| c.method == "GET"),
        "scope must not upload the patch or snapshot: {caps:?}"
    );
}

#[test]
fn remote_review_scope_json_matches_fs_mode_field_for_field() {
    // spec：local／remote resolved payload 欄位同構——同內容、決定性 commit 下
    // 逐位元相同。
    let fs = TempProject::fs("review-scope-fs");
    fs.write(
        "openspec/changes/demo/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\n",
    );
    fs.write("openspec/changes/demo/tasks.md", "- [x] 1.1 first\n");
    seed_git_src(&fs);
    let prepared = fs.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "fs prepare: {}", stderr_of(&prepared));
    fs.write("src/lib.rs", "fn demo() { changed(); }\n");
    write_touched(&fs, "demo", &["src/lib.rs"]);
    let fs_out = fs.run(&["review", "scope", "demo", "--json"]);
    assert!(fs_out.status.success(), "fs scope: {}", stderr_of(&fs_out));

    let mock = mock_server(review_scope_routes());
    let p = TempProject::remote("review-scope-parity", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "remote prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    write_touched(&p, "demo", &["src/lib.rs"]);
    let remote_out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(remote_out.status.success(), "remote scope: {}", stderr_of(&remote_out));

    assert_eq!(
        stdout_of(&remote_out),
        stdout_of(&fs_out),
        "resolved payload must be byte-identical across local and remote entry points"
    );
}

fn verify_scope_routes() -> Vec<(&'static str, &'static str, u16, String)> {
    vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":0,"totalTasks":2}]}"#.into(),
        ),
        (
            "GET",
            "/changes/demo/verify",
            404,
            r#"{"status":404,"reason":"not_found","message":"no verify ticket for change 'demo'"}"#.into(),
        ),
    ]
}

#[test]
fn remote_verify_scope_json_matches_fs_mode_field_for_field() {
    // spec verify-station Scenario「remote scope 仍使用 local checkout」＋
    // 「local 與 remote payload 同構」：同內容、決定性 commit 下逐位元相同，
    // 且 server 收不到 patch 或 snapshot。
    let fs = TempProject::fs("verify-scope-fs");
    fs.write(
        "openspec/changes/demo/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\n",
    );
    fs.write("openspec/changes/demo/tasks.md", "- [x] 1.1 first\n");
    seed_git_src(&fs);
    let prepared = fs.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "fs prepare: {}", stderr_of(&prepared));
    fs.write("src/lib.rs", "fn demo() { changed(); }\n");
    write_touched(&fs, "demo", &["src/lib.rs"]);
    let fs_out = fs.run(&["verify", "scope", "demo", "--json"]);
    assert!(fs_out.status.success(), "fs scope: {}", stderr_of(&fs_out));

    let mock = mock_server(verify_scope_routes());
    let p = TempProject::remote("verify-scope-parity", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "remote prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    write_touched(&p, "demo", &["src/lib.rs"]);
    let remote_out = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(remote_out.status.success(), "remote scope: {}", stderr_of(&remote_out));

    assert_eq!(
        stdout_of(&remote_out),
        stdout_of(&fs_out),
        "resolved payload must be byte-identical across local and remote entry points"
    );
    let caps = mock.captured.lock().unwrap();
    assert!(
        caps.iter().all(|c| c.method == "GET"),
        "verify scope must not upload the patch or snapshot: {caps:?}"
    );
}

#[test]
fn remote_verify_add_round_offline_changes_nothing() {
    // spec Scenario「離線時追加驗證輪」：連線錯誤 → 非零，遠端與本地投影皆不變。
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("probe port");
        listener.local_addr().expect("addr").port()
    }; // listener dropped — the port now refuses connections
    let url = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let p = TempProject::remote("verify-offline", &url, "backend");
    seed_git_src(&p);
    let out = p.run(&["verify", "show", "demo", "--json"]);
    assert!(!out.status.success(), "offline read must be non-zero");
    assert!(
        !p.dir.join("openspec").join("changes").join("demo").join("verify.md").exists(),
        "no local projection effects on the offline path"
    );
    let out = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "offline scope must be non-zero");
    assert!(
        !p.dir.join(".speclink").join("review-scopes").exists(),
        "no sidecar effects on the offline path"
    );
}

#[test]
fn remote_review_scope_offline_leaves_zero_sidecar_effects() {
    // spec Scenario「remote 離線時零 sidecar effects」：連線失敗 → 非零、
    // baseline 與 snapshots 內容不變。
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("probe port");
        listener.local_addr().expect("addr").port()
    }; // listener dropped — the port now refuses connections
    let url = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let p = TempProject::remote("review-offline", &url, "backend");
    seed_git_src(&p);
    write_touched(&p, "demo", &["src/lib.rs"]);
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "offline must be non-zero");
    assert!(
        !p.dir.join(".speclink").join("review-scopes").exists(),
        "no sidecar effects on the offline path"
    );
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(!out.status.success(), "offline prepare must be non-zero");
    assert!(
        !p.dir.join(".speclink").join("review-scopes").exists(),
        "prepare writes nothing when the remote read fails"
    );
}

#[test]
fn remote_review_prepare_auth_failure_leaves_zero_sidecar_effects() {
    // spec：認證失效 → 非零且不寫 baseline／snapshot。
    let mock = mock_server(vec![(
        "GET",
        "/changes",
        401,
        r#"{"status":401,"reason":"unauthenticated","message":"token expired"}"#.into(),
    )]);
    let p = TempProject::remote("review-auth", &mock.base, "backend");
    seed_git_src(&p);
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(!out.status.success(), "auth failure must be non-zero");
    assert!(
        !p.dir.join(".speclink").join("review-scopes").exists(),
        "zero sidecar effects on auth failure"
    );
}

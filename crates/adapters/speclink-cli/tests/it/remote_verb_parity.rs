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

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_STORE_URL")
            .env("SPECLINK_TOKEN", "tok");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        use std::io::Write;
        let mut child = self
            .cmd(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        // 被測指令提早失敗時 stdin 是 broken pipe——吞掉它，讓斷言看到真正
        // 的 stderr 而不是 helper 的 panic。其他寫入錯誤照炸。
        if let Err(e) = child.stdin.take().unwrap().write_all(input.as_bytes()) {
            assert!(
                e.kind() == std::io::ErrorKind::BrokenPipe,
                "write stdin failed: {e}"
            );
        }
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

// --- validate --specs：規格內容由既有讀取端點取得、本地跑同一驗證器（design D4）---

/// 殘留佔位（warning）與缺 Purpose 區段（error）各一份，覆蓋兩種嚴重度。
const PARITY_SPEC_TBD: &str = "# alpha Specification\n\n## Purpose\n\nTBD - created by archiving change 'old'. Update Purpose after archive.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n";
const PARITY_SPEC_BARE: &str =
    "# beta Specification\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n";

#[test]
fn remote_validate_specs_matches_fs_output_byte_for_byte() {
    let fs = TempProject::fs("validate-specs-fs");
    fs.write("openspec/specs/alpha/spec.md", PARITY_SPEC_TBD);
    fs.write("openspec/specs/beta/spec.md", PARITY_SPEC_BARE);
    let fs_human = fs.run(&["validate", "--specs"]);
    let fs_json = fs.run(&["validate", "--specs", "--json"]);
    assert!(!fs_human.status.success(), "缺 Purpose 的規格使 fs 模式非零收尾");

    let mock = mock_server(vec![
        (
            "GET",
            "/specs",
            200,
            r#"{"specs":[{"id":"beta","path":"specs/beta/spec.md"},{"id":"alpha","path":"specs/alpha/spec.md"}]}"#.into(),
        ),
        (
            "GET",
            "/specs/alpha/document",
            200,
            serde_json::json!({ "content": PARITY_SPEC_TBD }).to_string(),
        ),
        (
            "GET",
            "/specs/beta/document",
            200,
            serde_json::json!({ "content": PARITY_SPEC_BARE }).to_string(),
        ),
    ]);
    let p = TempProject::remote("validate-specs-remote", &mock.base, "backend");

    let remote_human = p.run(&["validate", "--specs"]);
    assert_eq!(stdout_of(&remote_human), stdout_of(&fs_human), "human output parity");
    assert_eq!(
        remote_human.status.success(),
        fs_human.status.success(),
        "exit code parity: {}",
        stderr_of(&remote_human)
    );
    let remote_json = p.run(&["validate", "--specs", "--json"]);
    assert_eq!(stdout_of(&remote_json), stdout_of(&fs_json), "--json output parity");

    // 讀取走既有的規格端點，沒有新開 server 端驗證端點。
    mock.find("GET", "/specs");
    mock.find("GET", "/specs/alpha/document");
    assert!(
        !p.dir.join("openspec").exists(),
        "the remote run never created or read a local store"
    );
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

// --- list：人眼渲染兩模式同形（invalid 標記一度只在 fs 側渲染）---

const LIST_GOOD_META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const LIST_BAD_META: &str = ": : :\n\t bad yaml [unclosed\n";
const LIST_PROPOSAL: &str = "## Why\n\nDemo.\n";
const LIST_TASKS: &str = "- [ ] 1.1 Do the thing\n";

/// 一份含壞 metadata 的 fs 沙盒，與 list_routes 的 wire payload 同內容。
fn list_fs_project(tag: &str) -> TempProject {
    let p = TempProject::fs(tag);
    for (name, meta) in [("good-change", LIST_GOOD_META), ("broken-change", LIST_BAD_META)] {
        p.write(&format!("openspec/changes/{name}/.openspec.yaml"), meta);
        p.write(&format!("openspec/changes/{name}/proposal.md"), LIST_PROPOSAL);
        p.write(&format!("openspec/changes/{name}/tasks.md"), LIST_TASKS);
    }
    p
}

/// server 依 fs 的 name 排序回傳，兩模式因此逐行對得上。
fn list_routes() -> Vec<(&'static str, &'static str, u16, String)> {
    vec![(
        "GET",
        "/changes",
        200,
        serde_json::json!({
            "changes": [
                {
                    "name": "broken-change",
                    "status": "in-progress",
                    "summary": "Demo.",
                    "completedTasks": 0,
                    "totalTasks": 1,
                    "metaError": "did not find expected key, while parsing a block mapping",
                },
                {
                    "name": "good-change",
                    "status": "in-progress",
                    "summary": "Demo.",
                    "completedTasks": 0,
                    "totalTasks": 1,
                },
            ]
        })
        .to_string(),
    )]
}

#[test]
fn remote_list_matches_fs_output_byte_for_byte_including_the_invalid_marker() {
    // wire 一直帶著 metaError，只有 fs 側渲染 `(invalid .openspec.yaml)`——
    // 同一份事實兩種輸出，正是渲染寫兩份養出來的漂移。
    let fs = list_fs_project("list-fs");
    let args = ["list", "--sort", "name", "--no-color"];
    let fs_human = fs.run(&args);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));

    let mock = mock_server(list_routes());
    let p = TempProject::remote("list-remote", &mock.base, "backend");
    let remote_human = p.run(&args);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));

    assert_eq!(
        stdout_of(&remote_human),
        stdout_of(&fs_human),
        "human output is byte-identical to fs mode, invalid marker included"
    );
    assert!(
        !p.dir.join("openspec").exists(),
        "the remote run never created or read a local store"
    );
}

// --- discuss：全家的成功訊息兩模式同形（conclude 的重收清單曾只在 fs 側可見）---

const CONCLUSION: &str = "**Decision**: ship it\n";

/// 一份已轉出變更的討論——re-conclude 會把該變更打回重收。
fn conclude_fs_project(tag: &str) -> TempProject {
    let p = TempProject::fs(tag);
    p.write(
        "openspec/discussions/auth-scope.md",
        "---\ntopic: Auth scope\nslug: auth-scope\nstatus: promoted\npromoted_to: add-auth\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\nold\n",
    );
    p.write(
        "openspec/changes/add-auth/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: auth-scope\n",
    );
    p
}

#[test]
fn remote_discuss_conclude_matches_fs_output_byte_for_byte() {
    // 重收清單（restaleFlagged）一度只有 fs 端算得出來，remote 端整個回應被丟棄，
    // 於是「這次結論打回了哪些變更」在遠端完全看不到。
    let fs = conclude_fs_project("conclude-fs");
    let args = ["discuss", "conclude", "auth-scope", "--stdin"];
    let fs_human = fs.run_stdin(&args, CONCLUSION);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));
    assert!(
        stdout_of(&fs_human).contains("add-auth"),
        "the fs baseline names the flagged change: {}",
        stdout_of(&fs_human)
    );

    let mock = mock_server(vec![(
        "POST",
        "/discussions/auth-scope/conclude",
        200,
        r#"{"restaleFlagged":["add-auth"]}"#.to_string(),
    )]);
    let p = TempProject::remote("conclude-remote", &mock.base, "backend");
    let remote_human = p.run_stdin(&args, CONCLUSION);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));

    assert_eq!(
        stdout_of(&remote_human),
        stdout_of(&fs_human),
        "human output is byte-identical to fs mode, flagged changes included"
    );

    let fs_json = fs.run_stdin(&["discuss", "conclude", "auth-scope", "--stdin", "--json"], CONCLUSION);
    let mock2 = mock_server(vec![(
        "POST",
        "/discussions/auth-scope/conclude",
        200,
        r#"{"restaleFlagged":["add-auth"]}"#.to_string(),
    )]);
    let p2 = TempProject::remote("conclude-remote-json", &mock2.base, "backend");
    let remote_json =
        p2.run_stdin(&["discuss", "conclude", "auth-scope", "--stdin", "--json"], CONCLUSION);
    assert_eq!(
        stdout_of(&remote_json),
        stdout_of(&fs_json),
        "--json output is byte-identical to fs mode"
    );
}

#[test]
fn remote_discuss_conclude_hold_matches_fs_output_byte_for_byte() {
    // --hold 新增的保留行同時落在 fs 與 remote：remote 端由回應的 held 驅動同一
    // render 函式，人眼與 --json 都要與 fs 逐位元一致。
    let fs = conclude_fs_project("conclude-hold-fs");
    let args = ["discuss", "conclude", "auth-scope", "--stdin", "--hold"];
    let fs_human = fs.run_stdin(&args, CONCLUSION);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));
    assert!(
        stdout_of(&fs_human).contains("Held live"),
        "the fs baseline announces the hold: {}",
        stdout_of(&fs_human)
    );

    let mock = mock_server(vec![(
        "POST",
        "/discussions/auth-scope/conclude",
        200,
        r#"{"restaleFlagged":["add-auth"],"held":true}"#.to_string(),
    )]);
    let p = TempProject::remote("conclude-hold-remote", &mock.base, "backend");
    let remote_human = p.run_stdin(&args, CONCLUSION);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));
    assert_eq!(
        stdout_of(&remote_human),
        stdout_of(&fs_human),
        "human output is byte-identical to fs mode, hold line included"
    );

    let json_args = ["discuss", "conclude", "auth-scope", "--stdin", "--hold", "--json"];
    let fs_json = fs.run_stdin(&json_args, CONCLUSION);
    let mock2 = mock_server(vec![(
        "POST",
        "/discussions/auth-scope/conclude",
        200,
        r#"{"restaleFlagged":["add-auth"],"held":true}"#.to_string(),
    )]);
    let p2 = TempProject::remote("conclude-hold-remote-json", &mock2.base, "backend");
    let remote_json = p2.run_stdin(&json_args, CONCLUSION);
    assert_eq!(
        stdout_of(&remote_json),
        stdout_of(&fs_json),
        "--json output is byte-identical to fs mode, held key included"
    );
}

#[test]
fn remote_discuss_new_json_prints_the_wire_response_verbatim() {
    // remote 的 --json 契約是 wire 回應原樣（slug／topic／path 三欄）——組 core
    // 型別會捏造 server 沒說的欄位（status、rounds、空 created），形狀凍結不允許。
    let mock = mock_server(vec![(
        "POST",
        "/discussions",
        201,
        r#"{"slug":"auth-scope","topic":"Auth scope","path":"discussions/auth-scope.md"}"#
            .to_string(),
    )]);
    let p = TempProject::remote("discuss-new-json", &mock.base, "backend");
    let out = p.run(&["discuss", "new", "Auth scope", "--slug", "auth-scope", "--json"]);
    assert!(out.status.success(), "remote stderr: {}", stderr_of(&out));
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&out).trim()).expect("stdout is JSON");
    let keys: Vec<&str> = payload.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys, ["path", "slug", "topic"], "the wire three-field shape, nothing invented");
}

#[test]
fn remote_discuss_new_and_add_round_and_archive_match_fs_human_output() {
    // spec scenario「discuss 動詞成功訊息兩模式同形」點名的動詞逐位元對照。
    let fs = TempProject::fs("discuss-human-fs");
    let fs_new = fs.run(&["discuss", "new", "Auth scope", "--slug", "auth-scope"]);
    assert!(fs_new.status.success(), "fs stderr: {}", stderr_of(&fs_new));
    let fs_round = fs.run_stdin(
        &["discuss", "add-round", "auth-scope", "--mode", "assumptions", "--stdin"],
        "**Focus**: x\n**Position**: y\n",
    );
    assert!(fs_round.status.success(), "fs stderr: {}", stderr_of(&fs_round));
    let fs_conclude = fs.run_stdin(
        &["discuss", "conclude", "auth-scope", "--stdin"],
        "**Decision**: go\n",
    );
    assert!(fs_conclude.status.success(), "fs stderr: {}", stderr_of(&fs_conclude));
    let fs_archive = fs.run(&["discuss", "archive", "auth-scope"]);
    assert!(fs_archive.status.success(), "fs stderr: {}", stderr_of(&fs_archive));

    // discuss new 的 Path 行兩模式同為 store 相對路徑（引擎的 path 欄位）；
    // mock 回 fs 輸出裡的同一字串，兩模式即同文本。
    let fs_new_stdout = stdout_of(&fs_new);
    let fs_path_line = fs_new_stdout
        .lines()
        .find(|l| l.trim_start().starts_with("Path: "))
        .expect("fs discuss new prints a Path line");
    let wire_path = fs_path_line.trim_start().trim_start_matches("Path: ").to_string();
    // fs 的封存檔名帶日期前綴，wire 值取自 fs 實際輸出才對得起來。
    let wire_archived_to = stdout_of(&fs_archive)
        .lines()
        .next()
        .and_then(|l| l.rsplit(" → ").next().map(str::to_string))
        .expect("fs discuss archive names the destination");

    let mock = mock_server(vec![
        (
            "POST",
            "/discussions",
            201,
            serde_json::json!({
                "slug": "auth-scope",
                "topic": "Auth scope",
                "path": wire_path,
            })
            .to_string(),
        ),
        (
            "POST",
            "/discussions/auth-scope/rounds",
            200,
            r#"{"round":1}"#.to_string(),
        ),
        (
            "POST",
            "/discussions/auth-scope/archive",
            200,
            serde_json::json!({ "archivedTo": wire_archived_to }).to_string(),
        ),
    ]);
    let p = TempProject::remote("discuss-human-remote", &mock.base, "backend");

    let remote_new = p.run(&["discuss", "new", "Auth scope", "--slug", "auth-scope"]);
    assert!(remote_new.status.success(), "remote stderr: {}", stderr_of(&remote_new));
    assert_eq!(stdout_of(&remote_new), fs_new_stdout, "discuss new human parity");

    let remote_round = p.run_stdin(
        &["discuss", "add-round", "auth-scope", "--mode", "assumptions", "--stdin"],
        "**Focus**: x\n**Position**: y\n",
    );
    assert!(remote_round.status.success(), "remote stderr: {}", stderr_of(&remote_round));
    assert_eq!(stdout_of(&remote_round), stdout_of(&fs_round), "add-round human parity");

    let remote_archive = p.run(&["discuss", "archive", "auth-scope"]);
    assert!(remote_archive.status.success(), "remote stderr: {}", stderr_of(&remote_archive));
    assert_eq!(
        stdout_of(&remote_archive),
        stdout_of(&fs_archive),
        "discuss archive human parity"
    );
}

#[test]
fn remote_discuss_promote_prints_one_line_and_drops_the_path_pair() {
    // 明文分歧（design D5 第 5 項）：fs 印 Path 行＋propose 提示行，remote 兩行
    // 一起不印——首行則兩模式同文本。
    let fs = TempProject::fs("discuss-promote-fs");
    fs.write(
        "openspec/discussions/auth-scope.md",
        "---\ntopic: Auth scope\nslug: auth-scope\nstatus: concluded\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n**Decision**: go\n",
    );
    let fs_out = fs.run(&["discuss", "promote", "auth-scope"]);
    assert!(fs_out.status.success(), "fs stderr: {}", stderr_of(&fs_out));
    let fs_stdout = stdout_of(&fs_out);
    let fs_first = fs_stdout.lines().next().expect("fs prints the promoted line");
    assert!(fs_stdout.contains("  Path: "), "fs prints the Path line: {fs_stdout}");
    assert!(
        fs_stdout.contains("Proposal prefilled"),
        "fs prints the follow-up hint: {fs_stdout}"
    );

    let mock = mock_server(vec![(
        "POST",
        "/discussions/auth-scope/promote",
        200,
        r#"{"change":"auth-scope"}"#.to_string(),
    )]);
    let p = TempProject::remote("discuss-promote-remote", &mock.base, "backend");
    let remote_out = p.run(&["discuss", "promote", "auth-scope"]);
    assert!(remote_out.status.success(), "remote stderr: {}", stderr_of(&remote_out));
    assert_eq!(
        stdout_of(&remote_out),
        format!("{fs_first}\n"),
        "remote prints the same first line and nothing else — the Path pair stays fs-only"
    );
}

#[test]
fn remote_in_progress_remove_reports_the_idempotent_noop_like_fs_mode() {
    // 未開工的 change 被要求退回時，fs 說「本來就沒開工」，remote 一度只會
    // 說「已移除」——同一個引擎結果，兩種說法。
    let fs = TempProject::fs("inprogress-fs");
    fs.write(
        "openspec/changes/add-auth/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\n",
    );
    let args = ["in-progress", "remove", "add-auth"];
    let fs_human = fs.run(&args);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));
    assert!(
        stdout_of(&fs_human).contains("already proposed"),
        "the fs baseline reports the no-op: {}",
        stdout_of(&fs_human)
    );

    let mock = mock_server(vec![(
        "DELETE",
        "/changes/add-auth/in-progress",
        200,
        r#"{"removed":false}"#.to_string(),
    )]);
    let p = TempProject::remote("inprogress-remote", &mock.base, "backend");
    let remote_human = p.run(&args);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));
    assert_eq!(
        stdout_of(&remote_human),
        stdout_of(&fs_human),
        "human output is byte-identical to fs mode"
    );
}

// --- archive：新 server 走 fs 同一支渲染，舊 server 整體退化 ---

/// 一份可封存的 fs 沙盒：任務全勾、一份 delta spec、一個來源討論。
fn archive_fs_project(tag: &str) -> TempProject {
    let p = TempProject::fs(tag);
    p.write(
        "openspec/changes/add-auth/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: auth-scope\n",
    );
    p.write("openspec/changes/add-auth/tasks.md", "- [x] 1.1 done\n");
    p.write(
        "openspec/changes/add-auth/specs/user-auth/spec.md",
        // 新開 capability 的 delta 自帶合格 Purpose，否則封存被 Purpose 守門擋下。
        "## Purpose\n\n本 capability 負責使用者登入與登出的可觀察行為，涵蓋工作階段的建立、續期與撤銷三段流程。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
    );
    p.write(
        "openspec/discussions/auth-scope.md",
        "---\ntopic: Auth scope\nslug: auth-scope\nstatus: promoted\npromoted_to: add-auth\n---\n\n## Conclusion\n\nGo.\n",
    );
    p
}

#[test]
fn remote_archive_matches_fs_output_byte_for_byte() {
    // 封存目的地、規格計數、封存的來源討論、零證據提示——fs 全印，remote 一度
    // 只有一行「Archived change」。
    let fs = archive_fs_project("archive-fs");
    let fs_out = fs.run(&["archive", "add-auth", "--no-color"]);
    assert!(fs_out.status.success(), "fs stderr: {}", stderr_of(&fs_out));
    let fs_stdout = stdout_of(&fs_out);
    assert!(fs_stdout.contains("Specs applied"), "fs baseline lists counts: {fs_stdout}");
    assert!(
        fs_stdout.contains("Discussion archived"),
        "fs baseline names the co-travelled discussion: {fs_stdout}"
    );

    // wire 回報同一份引擎結果。dated 名稱取自 fs 那一輪，兩邊才對得起來。
    let dated = fs_stdout
        .lines()
        .next()
        .and_then(|l| l.rsplit(" → ").next())
        .expect("the fs line names the destination")
        .to_string();
    let archived_file = format!("{}-auth-scope.md", &dated[..10]);
    let mock = mock_server(vec![(
        "POST",
        "/changes/add-auth/archive",
        200,
        serde_json::json!({
            "specs": [{ "capability": "user-auth", "added": 1, "modified": 0, "removed": 0, "renamed": 0 }],
            "datedName": dated,
            "snapshotCreated": true,
            "archivedDiscussions": [{ "slug": "auth-scope", "file": archived_file }],
            "evidenceRecorded": false,
        })
        .to_string(),
    )]);
    let p = TempProject::remote("archive-remote", &mock.base, "backend");
    let remote_out = p.run(&["archive", "add-auth", "--no-color"]);
    assert!(remote_out.status.success(), "remote stderr: {}", stderr_of(&remote_out));

    assert_eq!(
        stdout_of(&remote_out),
        fs_stdout,
        "human output is byte-identical to fs mode"
    );
    assert_eq!(
        stderr_of(&remote_out),
        stderr_of(&fs_out),
        "the zero-evidence note travels too"
    );
}

#[test]
fn remote_archive_falls_back_whole_for_an_old_server() {
    // 舊 server 沒有哨兵：退回既有兩行輸出，不做半新半舊的混合渲染。
    let mock = mock_server(vec![(
        "POST",
        "/changes/add-auth/archive",
        200,
        r#"{"specs":[{"capability":"user-auth"}]}"#.to_string(),
    )]);
    let p = TempProject::remote("archive-legacy", &mock.base, "backend");
    let out = p.run(&["archive", "add-auth", "--no-color"]);
    assert!(out.status.success(), "remote stderr: {}", stderr_of(&out));
    assert_eq!(
        stdout_of(&out),
        "✓ Archived change: add-auth\n  Specs updated: user-auth\n",
        "the old-server output is unchanged"
    );
}

// --- 品質站工單閱讀：原文走人眼、--json 形狀不因原文上 wire 而變 ---

const TICKET_DOC: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/lib.rs\n\n- [WARNING] src/lib.rs — possible Feature Envy\n";

fn ticket_routes(content: Option<&str>) -> Vec<(&'static str, &'static str, u16, String)> {
    let mut body = serde_json::json!({
        "change": "demo",
        "rounds": [{
            "index": 1, "phase": null, "patchHash": null,
            "scope": ["src/lib.rs"],
            "findings": [{ "severity": "WARNING", "path": "src/lib.rs", "text": "possible Feature Envy" }],
        }],
        "lastRound": {
            "index": 1, "phase": null, "patchHash": null,
            "scope": ["src/lib.rs"],
            "findings": [{ "severity": "WARNING", "path": "src/lib.rs", "text": "possible Feature Envy" }],
        },
    });
    if let Some(content) = content {
        body["content"] = serde_json::json!(content);
    }
    vec![("GET", "/changes/demo/review", 200, body.to_string())]
}

#[test]
fn remote_review_show_prints_the_document_and_keeps_json_free_of_it() {
    // 原文上 wire 是為了人眼路徑；`--json` 的欄位集合是對外契約，原文不屬於它。
    let mock = mock_server(ticket_routes(Some(TICKET_DOC)));
    let p = TempProject::remote("ticket-remote", &mock.base, "backend");

    let human = p.run(&["review", "show", "demo"]);
    assert!(human.status.success(), "remote stderr: {}", stderr_of(&human));
    assert_eq!(stdout_of(&human), TICKET_DOC, "the human path prints the ticket verbatim");

    let mock2 = mock_server(ticket_routes(Some(TICKET_DOC)));
    let p2 = TempProject::remote("ticket-remote-json", &mock2.base, "backend");
    let json = p2.run(&["review", "show", "demo", "--json"]);
    assert!(json.status.success(), "remote stderr: {}", stderr_of(&json));
    let payload: serde_json::Value =
        serde_json::from_str(stdout_of(&json).trim()).expect("stdout is JSON");
    assert!(
        payload.get("content").is_none(),
        "the document body must not leak into the --json contract: {payload}"
    );
    assert_eq!(payload["change"], "demo");
    assert_eq!(payload["lastRound"]["findings"][0]["severity"], "WARNING");
}

#[test]
fn remote_review_show_falls_back_to_the_summary_without_a_document() {
    // 舊 server 不帶原文：印結構化摘要，不拿結構化欄位反推一份假原文。
    let mock = mock_server(ticket_routes(None));
    let p = TempProject::remote("ticket-legacy", &mock.base, "backend");
    let human = p.run(&["review", "show", "demo"]);
    assert!(human.status.success(), "remote stderr: {}", stderr_of(&human));
    let out = stdout_of(&human);
    assert!(out.starts_with("Review — demo"), "summary header: {out}");
    assert!(out.contains("[WARNING] src/lib.rs"), "summary lists findings: {out}");
}

#[test]
fn remote_review_show_refuses_an_unknown_round_phase() {
    // server 比 CLI 新：未知 phase 靜默當 legacy 會讓輸出宣稱錯誤的事實。
    let mut routes = ticket_routes(None);
    routes[0].3 = routes[0].3.replace("\"phase\":null", "\"phase\":\"triage\"");
    let mock = mock_server(routes);
    let p = TempProject::remote("ticket-phase", &mock.base, "backend");
    let out = p.run(&["review", "show", "demo"]);
    assert!(!out.status.success(), "an unknown phase token must fail loud");
    assert!(
        stderr_of(&out).contains("triage"),
        "the refusal names the token: {}",
        stderr_of(&out)
    );
}

#[test]
fn remote_review_show_refuses_an_unknown_finding_severity() {
    // phase 的對稱路徑：severity 也是 server 端詞彙，未知值不得靜默吞掉。
    let mut routes = ticket_routes(None);
    routes[0].3 = routes[0].3.replace("\"severity\":\"WARNING\"", "\"severity\":\"BLOCKER\"");
    let mock = mock_server(routes);
    let p = TempProject::remote("ticket-severity", &mock.base, "backend");
    let out = p.run(&["review", "show", "demo"]);
    assert!(!out.status.success(), "an unknown severity must fail loud");
    assert!(
        stderr_of(&out).contains("BLOCKER"),
        "the refusal names the token: {}",
        stderr_of(&out)
    );
}

#[test]
fn remote_review_show_refuses_an_empty_rounds_ticket() {
    // wire 允許 rounds:[] 搭配獨立 lastRound；核心工單不變量是至少一輪——空
    // 陣列必須是明確錯誤，不是 panic。
    let body = r#"{"change":"demo","rounds":[],"lastRound":{"index":1,"phase":null,"patchHash":null,"scope":[],"findings":[]}}"#;
    let mock = mock_server(vec![("GET", "/changes/demo/review", 200, body.to_string())]);
    let p = TempProject::remote("ticket-empty", &mock.base, "backend");
    let out = p.run(&["review", "show", "demo"]);
    assert!(!out.status.success(), "an empty rounds list must be a clean error");
    let err = stderr_of(&out);
    assert!(err.starts_with("Error:"), "a clean error, not a panic: {err}");
    assert!(err.contains("no rounds"), "the refusal explains the shape: {err}");
}

#[test]
fn remote_review_show_json_matches_fs_key_set() {
    // spec scenario「工單 --json 兩模式同形且無原文欄位」：payload 欄位集合對照
    // fs 模式逐 key 相等。
    let fs = TempProject::fs("ticket-json-fs");
    fs.write(
        "openspec/changes/demo/.openspec.yaml",
        "schema: spec-driven\ncreated: 2026-07-01\n",
    );
    fs.write("openspec/changes/demo/tasks.md", "- [x] 1.1 done\n");
    fs.write(
        "openspec/changes/demo/review.md",
        "# Review — demo\n\n## Round 1\n\n**Scope**: src/lib.rs\n\n- [WARNING] src/lib.rs — possible Feature Envy\n",
    );
    let fs_json = fs.run(&["review", "show", "demo", "--json"]);
    assert!(fs_json.status.success(), "fs stderr: {}", stderr_of(&fs_json));

    let mock = mock_server(ticket_routes(Some(TICKET_DOC)));
    let p = TempProject::remote("ticket-json-remote", &mock.base, "backend");
    let remote_json = p.run(&["review", "show", "demo", "--json"]);
    assert!(remote_json.status.success(), "remote stderr: {}", stderr_of(&remote_json));

    let fs_payload: serde_json::Value =
        serde_json::from_str(stdout_of(&fs_json).trim()).expect("fs stdout is JSON");
    let remote_payload: serde_json::Value =
        serde_json::from_str(stdout_of(&remote_json).trim()).expect("remote stdout is JSON");
    let keys = |v: &serde_json::Value| -> Vec<String> {
        v.as_object().unwrap().keys().cloned().collect()
    };
    assert_eq!(keys(&fs_payload), keys(&remote_payload), "top-level key sets match");
    assert_eq!(
        keys(&fs_payload["lastRound"]),
        keys(&remote_payload["lastRound"]),
        "round key sets match"
    );
    assert_eq!(
        keys(&fs_payload["lastRound"]["findings"][0]),
        keys(&remote_payload["lastRound"]["findings"][0]),
        "finding key sets match"
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

/// server change evidence 端點的單筆 entry（camelCase wire 形狀）。
fn evidence_entry(task_id: &str, actor: &str, files: &[&str]) -> String {
    let files: Vec<String> = files.iter().map(|f| format!("\"{f}\"")).collect();
    format!(
        "{{\"taskId\":\"{task_id}\",\"taskDesc\":\"t\",\"actor\":\"{actor}\",\"touchedFiles\":[{}],\"recordedAt\":\"2026-01-02T03:04:05Z\"}}",
        files.join(",")
    )
}

/// evidence 端點回應體：entries 陣列（空集合＝從未記錄，仍是 200）。
fn evidence_response(entries: &[String]) -> String {
    format!("{{\"entries\":[{}]}}", entries.join(","))
}

/// 帶 headCommit 的 entry（D3：head commit 僅存證，scope 解析不消費）。
fn evidence_entry_with_head(task_id: &str, actor: &str, files: &[&str], head: &str) -> String {
    let entry = evidence_entry(task_id, actor, files);
    entry.replacen('{', &format!("{{\"headCommit\":\"{head}\","), 1)
}

/// review scope 基本路由＋demo 的 evidence 端點（entries 由呼叫端給）。
fn review_scope_routes_with_evidence(
    entries: &[String],
) -> Vec<(&'static str, &'static str, u16, String)> {
    let mut routes = review_scope_routes();
    routes.push(("GET", "/changes/demo/evidence", 200, evidence_response(entries)));
    routes
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
    // local Git 產生、server 不收到 patch 或 snapshot；touched 認領來自 server
    // 的 change evidence，不再手塞本地檔。
    let mock = mock_server(review_scope_routes_with_evidence(&[evidence_entry(
        "1",
        "alice",
        &["src/lib.rs"],
    )]));
    let p = TempProject::remote("review-scope", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
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

    let mock = mock_server(review_scope_routes_with_evidence(&[evidence_entry(
        "1",
        "alice",
        &["src/lib.rs"],
    )]));
    let p = TempProject::remote("review-scope-parity", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "remote prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
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

    let mut routes = verify_scope_routes();
    routes.push((
        "GET",
        "/changes/demo/evidence",
        200,
        evidence_response(&[evidence_entry("1", "alice", &["src/lib.rs"])]),
    ));
    let mock = mock_server(routes);
    let p = TempProject::remote("verify-scope-parity", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "remote prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
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
fn remote_review_scope_auto_resolves_from_server_evidence() {
    // spec Scenario「remote task done 後 scope 自動解析」：touched 認領來自
    // server 的 change evidence，scope 不帶任何手動旗標即回 resolved payload。
    let mock = mock_server(review_scope_routes_with_evidence(&[evidence_entry(
        "1",
        "alice",
        &["src/lib.rs"],
    )]));
    let p = TempProject::remote("review-scope-evidence", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved", "server evidence supplies touched — no needsInput");
    assert_eq!(
        v["paths"],
        serde_json::json!(["src/lib.rs"]),
        "touched claim equals the evidence file set"
    );
    mock.find("GET", "/changes/demo/evidence");
}

#[test]
fn remote_review_scope_absent_evidence_keeps_the_empty_touched_fail_closed() {
    // spec Scenario「remote evidence 缺席維持 fail-closed」：server 回空 entries
    //（從未記錄＝正常狀態、200），scope 維持 EmptyTouched 的 needsInput 手動路徑。
    let mock = mock_server(review_scope_routes_with_evidence(&[]));
    let p = TempProject::remote("review-scope-no-evidence", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "empty touched must stay fail-closed");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "needsInput");
    assert!(
        stderr_of(&out).contains("no touched files recorded"),
        "EmptyTouched reason is reported: {}",
        stderr_of(&out)
    );
    assert!(
        stderr_of(&out).contains("--base") && stderr_of(&out).contains("--include-hunk"),
        "the manual escape hatches stay documented: {}",
        stderr_of(&out)
    );
    // 手動跳脫閥實跑：以 needsInput 提供的 candidateHash 與 hunk id 續行，
    // 證明手動路徑不只被提及、而是真的可用。
    let ch = v["candidateHash"].as_str().expect("needsInput carries the candidate anchor");
    let hunk = v["files"][0]["hunks"][0]["id"].as_str().expect("selectable hunk id");
    let out = p.run(&[
        "review",
        "scope",
        "demo",
        "--json",
        "--candidate-hash",
        ch,
        "--include-hunk",
        hunk,
    ]);
    assert!(out.status.success(), "escape hatch stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved", "the hash-pinned selection resolves the scope");
}

#[test]
fn remote_review_scope_unions_touched_across_multi_actor_evidence_entries() {
    // spec Scenario「多 actor evidence 取聯集」：兩位 actor 的 entries 各認領
    // 不同檔案集合，scope 的 touched 認領為聯集（與 fs 模式 all_files 同語意）；
    // 其中一筆帶 headCommit，釘住 D3——head commit 僅存證，不參與 scope 解析。
    let mock = mock_server(review_scope_routes_with_evidence(&[
        evidence_entry_with_head(
            "1",
            "alice",
            &["src/lib.rs"],
            "1111111111111111111111111111111111111111",
        ),
        evidence_entry("2", "bob", &["src/other.rs"]),
    ]));
    let p = TempProject::remote("review-scope-union", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    p.write("src/other.rs", "fn other() {}\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved");
    let paths: Vec<&str> =
        v["paths"].as_array().unwrap().iter().map(|p| p.as_str().unwrap()).collect();
    assert_eq!(paths.len(), 2, "the union of both actors' claims: {paths:?}");
    assert!(paths.contains(&"src/lib.rs") && paths.contains(&"src/other.rs"), "{paths:?}");
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&p.dir)
        .output()
        .expect("git rev-parse");
    let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    assert_eq!(
        v["baseCommit"].as_str(),
        Some(head.as_str()),
        "the evidence headCommit must not leak into scope resolution"
    );
}

#[test]
fn remote_review_scope_overlapping_server_evidence_triggers_the_other_claims_guard() {
    // spec Scenario「併行認領守門於 remote 生效」：另一 active change 的 server
    // evidence 認領重疊檔案 → 守門結果與 fs 模式同形，不被靜默忽略。
    let mut routes = vec![
        (
            "GET",
            "/changes",
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":0,"totalTasks":2},{"name":"other","status":"in-progress","completedTasks":0,"totalTasks":1}]}"#.to_string(),
        ),
        (
            "GET",
            "/changes/demo/review",
            404,
            r#"{"status":404,"reason":"not_found","message":"no review ticket for change 'demo'"}"#.to_string(),
        ),
    ];
    routes.push((
        "GET",
        "/changes/demo/evidence",
        200,
        evidence_response(&[evidence_entry("1", "alice", &["src/lib.rs"])]),
    ));
    routes.push((
        "GET",
        "/changes/other/evidence",
        200,
        evidence_response(&[evidence_entry("1", "bob", &["src/lib.rs"])]),
    ));
    let mock = mock_server(routes);
    let p = TempProject::remote("review-scope-overlap", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "an overlapping claim must not resolve silently");
    assert!(
        stderr_of(&out).contains("active change 'other' also claims: src/lib.rs"),
        "the other-claims guard fires with the fs-mode wording: {}",
        stderr_of(&out)
    );
}

#[test]
fn remote_validation_scope_reads_no_evidence() {
    // validation 輪（ticket 已存在）由凍結快照鏈解析、不消費 touched 認領：
    // remote 臂不得為它讀 evidence——白發請求，且把不需要的失敗面帶進
    // 必然成功的路徑。
    // 第一階段：discovery（mock A 供應 evidence）→ 落 baseline 與 snapshot。
    let mock_a = mock_server(review_scope_routes_with_evidence(&[evidence_entry(
        "1",
        "alice",
        &["src/lib.rs"],
    )]));
    let p = TempProject::remote("validation-skip", &mock_a.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "discovery stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    let patch_hash = v["patchHash"].as_str().expect("frozen patch hash").to_string();
    // 第二階段：mock B 帶 round 1 的工單、完全沒有 evidence 路由——scope 走
    // validation 時不得碰它。
    let round = serde_json::json!({
        "index": 1, "phase": "discovery", "patchHash": patch_hash,
        "scope": ["src/lib.rs"], "findings": [],
    });
    let ticket = serde_json::json!({ "change": "demo", "rounds": [round], "lastRound": round })
        .to_string();
    // listing 沿用既有 builder；此測試的差異只在 review 端點回 round 1 的工單。
    let mut routes_b = review_scope_routes();
    routes_b.retain(|(_, suffix, _, _)| *suffix != "/changes/demo/review");
    routes_b.push(("GET", "/changes/demo/review", 200, ticket));
    let mock_b = mock_server(routes_b);
    p.write(".speclink.yaml", &format!("remote:\n  url: {}\n  repo: backend\n", mock_b.base));
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "validation stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved");
    assert_eq!(v["phase"], "validation");
    let caps = mock_b.captured.lock().unwrap();
    assert!(
        caps.iter().all(|c| !c.path.ends_with("/evidence")),
        "a validation scope must not read evidence: {caps:?}"
    );
}

#[test]
fn remote_review_scope_evidence_read_failure_is_loud_and_leaves_zero_sidecar() {
    // spec：remote read 錯誤（含 evidence 讀取失敗）→ 非零、不寫 baseline／
    // snapshot，不得靜默降級成空認領；錯誤要指名是哪個 change 的 evidence。
    let mut routes = review_scope_routes();
    routes.push((
        "GET",
        "/changes/demo/evidence",
        500,
        r#"{"status":500,"reason":"internal","message":"evidence record is unreadable"}"#
            .to_string(),
    ));
    let mock = mock_server(routes);
    let p = TempProject::remote("review-scope-evidence-500", &mock.base, "backend");
    seed_git_src(&p);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
    let baseline_path =
        p.dir.join(".speclink").join("review-scopes").join("demo").join("baseline.json");
    let baseline_before = std::fs::read_to_string(&baseline_path).expect("baseline exists");
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "an evidence read failure must be non-zero");
    assert!(
        stderr_of(&out).contains("evidence") && stderr_of(&out).contains("'demo'"),
        "the error names the change whose evidence read failed: {}",
        stderr_of(&out)
    );
    assert!(
        !p.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots").exists(),
        "no snapshot lands on the failure path"
    );
    let baseline_after = std::fs::read_to_string(&baseline_path).expect("baseline still there");
    assert_eq!(baseline_before, baseline_after, "the baseline stays untouched");
}

#[test]
fn remote_review_scope_rejects_hostile_evidence_paths() {
    // server 是外部邊界：evidence 認領的路徑要在進 git pathspec 前把關。
    // `..` 越界由 git 喊停（loud），但 `:(exclude)` 這類 magic 前綴會靜默
    // 縮小審查面——兩類都必須被指名拒絕，不得原樣送進 git。
    for (tag, hostile) in [("dotdot", "../outside.rs"), ("magic", ":(exclude)src/lib.rs")] {
        let mock = mock_server(review_scope_routes_with_evidence(&[evidence_entry(
            "1",
            "alice",
            &[hostile],
        )]));
        let p = TempProject::remote(&format!("review-scope-hostile-{tag}"), &mock.base, "backend");
        seed_git_src(&p);
        let prepared = p.run(&["review", "prepare", "demo"]);
        assert!(prepared.status.success(), "stderr: {}", stderr_of(&prepared));
        p.write("src/lib.rs", "fn demo() { changed(); }\n");
        let out = p.run(&["review", "scope", "demo", "--json"]);
        assert!(!out.status.success(), "'{hostile}' must be refused");
        assert!(
            stderr_of(&out).contains(hostile) && stderr_of(&out).contains("'demo'"),
            "the error names the offending path and change: {}",
            stderr_of(&out)
        );
        assert!(
            !p.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots").exists(),
            "no snapshot lands on the refusal path"
        );
    }
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

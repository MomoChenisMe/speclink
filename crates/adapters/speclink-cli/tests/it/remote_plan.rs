//! `plan` 與 `change depends` 的 remote 臂（change-plan「plan 與 change depends 的
//! remote 臂」、verb-contract「模式分岔的單點宣告」）：remote 模式打 server 端點、
//! 回應轉回引擎型別後走 fs 模式同一渲染——同一份內容兩模式 stdout、stderr 與
//! exit code 逐位元一致，且 remote 模式不讀本機 openspec/。沿 remote_verb_parity
//! 的 capturing mock server 模式。

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

// --- capturing mock verb-contract server ---

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

fn mock_server(mut routes: Vec<(&'static str, String, u16, String)>) -> MockServer {
    routes.push(("GET", "/binding".to_string(), 200, BINDING_BODY.to_string()));
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
                None => (404, r#"{"reason":"not_found","message":"no route"}"#.to_string()),
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
    /// Every captured request except the handshake, as `METHOD suffix`.
    fn verb_calls(&self) -> Vec<String> {
        let prefix = "/api/speclink/v1/projects/demo";
        self.captured
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.path != format!("{prefix}/binding"))
            .map(|c| format!("{} {}", c.method, c.path.trim_start_matches(prefix)))
            .collect()
    }

    fn body_of(&self, method: &str, suffix: &str) -> String {
        let path = format!("/api/speclink/v1/projects/demo{suffix}");
        let caps = self.captured.lock().unwrap();
        caps.iter()
            .find(|c| c.method == method && c.path == path)
            .unwrap_or_else(|| panic!("no captured {method} {suffix}; got {caps:?}"))
            .body
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
    fn bare(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-remote-plan-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    /// fs 專案：只有 openspec/changes/，change 由測試逐一放入。
    fn at(tag: &str) -> TempProject {
        let p = TempProject::bare(tag);
        std::fs::create_dir_all(p.dir.join("openspec").join("changes")).unwrap();
        p
    }

    /// remote 專案：只有指向 mock 的連線設定。
    fn remote(tag: &str, url: &str) -> TempProject {
        let p = TempProject::bare(tag);
        std::fs::write(
            p.dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: backend\n"),
        )
        .unwrap();
        p
    }

    fn put_change(&self, name: &str, meta: &str) {
        let dir = self.dir.join("openspec").join("changes").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".openspec.yaml"), meta).unwrap();
    }

    /// 給 change 一份 capability delta，MODIFIED 一個 requirement。
    fn put_modified(&self, change: &str, cap: &str, requirement: &str) {
        let dir = self.dir.join("openspec").join("changes").join(change).join("specs").join(cap);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("spec.md"),
            format!("## MODIFIED Requirements\n\n### Requirement: {requirement}\n\nbody\n"),
        )
        .unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_STORE_URL",
            "SPECLINK_WORKTREE",
            "NO_COLOR",
            "CLICOLOR",
            "CLICOLOR_FORCE",
        ] {
            cmd.env_remove(key);
        }
        cmd.env("SPECLINK_TOKEN", "tok");
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

/// fs 對照組：add-a 為前置、add-b 依賴它。
fn fs_pair(tag: &str) -> TempProject {
    let p = TempProject::at(tag);
    p.put_change("add-a", "schema: spec-driven\ncreated: 2026-09-01\n");
    p.put_change("add-b", "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: add-a\n");
    p
}

// --- plan ---

#[test]
fn remote_plan_prints_the_fs_output_for_the_same_content() {
    // Scenario「remote plan 同形」：fs 的 --json 就是 server 對同內容的回應形狀；
    // remote 臂把它轉回引擎型別再渲染，--json 與人眼行都要與 fs 逐位元一致——
    // 含 requirement 級重疊與封存順序（兩者 MODIFIED 同一個 requirement）。
    let fs = fs_pair("plan-fs");
    fs.put_modified("add-a", "auth", "Login");
    fs.put_modified("add-b", "auth", "Login");
    let fs_json = fs.run(&["plan", "--json"]);
    assert!(fs_json.status.success(), "fs stderr: {}", stderr_of(&fs_json));
    let fs_human = fs.run(&["plan", "--no-color"]);
    assert!(fs_human.status.success(), "fs stderr: {}", stderr_of(&fs_human));

    let mock = mock_server(vec![("GET", "/plan".into(), 200, stdout_of(&fs_json))]);
    let remote = TempProject::remote("plan-remote", &mock.base);
    // 只存在本機的 change 當誘餌：remote 臂若讀了本機 openspec/，輸出就會多出它
    //（本機樹與 remote 設定並存時 stderr 另有一行並存警告，這裡只比 stdout）。
    remote.put_change("local-only", "schema: spec-driven\ncreated: 2026-01-01\n");
    let out = remote.run(&["plan", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), stdout_of(&fs_json), "remote --json matches fs byte for byte");
    let payload: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON");
    let mut keys: Vec<&str> = payload.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["changes", "next", "skipped", "waves"]);
    let add_b = &payload["changes"][1];
    assert_eq!(add_b["name"], "add-b");
    assert_eq!(add_b.as_object().unwrap().len(), 9, "nine keys per change: {add_b}");
    assert_eq!(add_b["archiveAfter"], serde_json::json!(["add-a"]));
    assert_eq!(add_b["requirementOverlap"][0]["ownOperation"], "MODIFIED");

    let human = remote.run(&["plan", "--no-color"]);
    assert!(human.status.success(), "stderr: {}", stderr_of(&human));
    assert_eq!(stdout_of(&human), stdout_of(&fs_human), "remote human lines match fs");
    assert!(stdout_of(&human).contains("archive after: add-a"), "{}", stdout_of(&human));
    assert!(!stdout_of(&human).contains("local-only"), "the local openspec/ tree is never read");
    assert_eq!(mock.verb_calls(), ["GET /plan", "GET /plan"]);
}

#[test]
fn remote_plan_cycle_exits_non_zero_with_the_engine_line() {
    // Scenario「remote 成環」：server 回 409 refused，stderr 與 fs 模式同一句。
    let fs = TempProject::at("cycle-fs");
    fs.put_change("add-a", "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: add-b\n");
    fs.put_change("add-b", "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: add-a\n");
    let fs_out = fs.run(&["plan", "--json"]);
    assert!(!fs_out.status.success());

    let mock = mock_server(vec![(
        "GET",
        "/plan".into(),
        409,
        r#"{"status":409,"reason":"refused","message":"dependency cycle: add-a -> add-b -> add-a"}"#
            .into(),
    )]);
    let remote = TempProject::remote("cycle-remote", &mock.base);
    let out = remote.run(&["plan", "--json"]);
    assert!(!out.status.success(), "a cycle exits non-zero");
    assert!(
        stderr_of(&out).contains("dependency cycle: add-a -> add-b -> add-a"),
        "stderr: {}",
        stderr_of(&out)
    );
    assert_eq!(stderr_of(&out), stderr_of(&fs_out), "the same line as fs mode");
    assert!(stdout_of(&out).is_empty(), "stdout stays empty");
}

#[test]
fn remote_plan_refuses_strict_overlap_without_a_request() {
    // verb-contract 明文分歧第 6 項：server 只以 requirement 級重疊規劃、沒有目錄級
    // 開關，remote 模式以固定訊息拒絕——只解析模式，握手與 GET /plan 都不發。
    let mock = mock_server(vec![("GET", "/plan".into(), 200, "{}".into())]);
    let remote = TempProject::remote("strict-remote", &mock.base);
    let out = remote.run(&["plan", "--strict-overlap", "--json"]);
    assert!(!out.status.success(), "the flag is refused in remote mode");
    assert!(
        stderr_of(&out).contains("plan --strict-overlap is not available in remote mode"),
        "stderr: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).is_empty(), "stdout stays empty");
    assert!(
        mock.captured.lock().unwrap().is_empty(),
        "no request at all, the handshake included"
    );
}

// --- change depends ---

#[test]
fn remote_change_depends_posts_the_body_and_prints_the_fs_line() {
    // Scenario「remote 寫入前置」：body 為 {"on":[…],"remove":false}，成功行與
    // --json 皆與 fs 模式同形。
    let fs = TempProject::at("depends-fs");
    fs.put_change("add-a", "schema: spec-driven\ncreated: 2026-09-01\n");
    fs.put_change("add-b", "schema: spec-driven\ncreated: 2026-09-02\n");
    let fs_line = fs.run(&["change", "depends", "add-b", "--on", "add-a", "--no-color"]);
    assert!(fs_line.status.success(), "fs stderr: {}", stderr_of(&fs_line));
    let fs_json = fs.run(&["change", "depends", "add-b", "--on", "add-a", "--json"]);
    let fs_removed =
        fs.run(&["change", "depends", "add-b", "--on", "add-a", "--remove", "--no-color"]);
    assert!(fs_removed.status.success(), "fs stderr: {}", stderr_of(&fs_removed));

    let added = mock_server(vec![(
        "POST",
        "/changes/add-b/depends".into(),
        200,
        r#"{"change":"add-b","dependsOn":["add-a"]}"#.into(),
    )]);
    let remote = TempProject::remote("depends-remote", &added.base);
    let out = remote.run(&["change", "depends", "add-b", "--on", "add-a", "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ add-b depends on: add-a\n");
    assert_eq!(stdout_of(&out), stdout_of(&fs_line));
    assert_eq!(added.body_of("POST", "/changes/add-b/depends"), r#"{"on":["add-a"],"remove":false}"#);
    let json = remote.run(&["change", "depends", "add-b", "--on", "add-a", "--json"]);
    assert_eq!(stdout_of(&json), stdout_of(&fs_json), "--json matches fs");

    let removed = mock_server(vec![(
        "POST",
        "/changes/add-b/depends".into(),
        200,
        r#"{"change":"add-b","dependsOn":[]}"#.into(),
    )]);
    let remote = TempProject::remote("depends-remote-rm", &removed.base);
    let out =
        remote.run(&["change", "depends", "add-b", "--on", "add-a", "--remove", "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), stdout_of(&fs_removed));
    assert_eq!(removed.body_of("POST", "/changes/add-b/depends"), r#"{"on":["add-a"],"remove":true}"#);
}

#[test]
fn remote_change_depends_refusals_print_the_engine_line() {
    // 404 與守門 409 的 message 為引擎原文：stderr 與 fs 模式同一句、exit 非零。
    let fs = TempProject::at("refuse-fs");
    fs.put_change("add-b", "schema: spec-driven\ncreated: 2026-09-02\n");
    let fs_self = fs.run(&["change", "depends", "add-b", "--on", "add-b"]);
    let fs_ghost = fs.run(&["change", "depends", "ghost", "--on", "add-b"]);
    assert!(!fs_self.status.success() && !fs_ghost.status.success());

    let mock = mock_server(vec![
        (
            "POST",
            "/changes/add-b/depends".into(),
            409,
            r#"{"status":409,"reason":"refused","message":"'add-b' cannot depend on itself"}"#.into(),
        ),
        (
            "POST",
            "/changes/ghost/depends".into(),
            404,
            r#"{"status":404,"reason":"not_found","message":"Change 'ghost' not found."}"#.into(),
        ),
    ]);
    let remote = TempProject::remote("refuse-remote", &mock.base);
    let out = remote.run(&["change", "depends", "add-b", "--on", "add-b"]);
    assert!(!out.status.success());
    assert_eq!(stderr_of(&out), stderr_of(&fs_self));
    assert!(stdout_of(&out).is_empty());
    let out = remote.run(&["change", "depends", "ghost", "--on", "add-b"]);
    assert!(!out.status.success());
    assert_eq!(stderr_of(&out), stderr_of(&fs_ghost));
}

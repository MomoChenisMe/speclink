//! Remote read-path routing: `speclink list/status/instructions/discuss
//! list/show` against a mock verb-contract server, asserting the stdout JSON
//! field names match fs mode exactly (fs is the shape authority), plus the
//! mode-resolution behaviors only visible at the CLI level (coexistence
//! warning, not-logged-in, connection failure, SPECLINK_STORE_URL override).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Arc;

// --- mock verb-contract server ---

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
}

/// The compatible handshake every verb-focused test needs — the binding
/// precedes any verb, so the mock always serves it unless a test overrides
/// the route to probe handshake failures.
const BINDING_BODY: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;

/// Routes: (method, path — matched against the part after the project base,
/// query string ignored, response body). Unmatched requests get 404. The
/// binding handshake route is injected automatically unless the test
/// declares its own.
fn mock_server(mut routes: Vec<(&'static str, &'static str, String)>) -> MockServer {
    if !routes.iter().any(|(_, suffix, _)| *suffix == "/binding") {
        routes.push(("GET", "/binding", BINDING_BODY.to_string()));
    }
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            let hit = routes.iter().find(|(m, suffix, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, body)) => (200, body.clone()),
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
    MockServer { server, base }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

// --- context-aware mock (POST /context with conditional 304 / 503) ---

/// How the context mock answers `POST /context`.
enum ContextBehavior {
    /// Serve `body` on a fresh request; a matching `If-None-Match` is 304.
    Serve { snapshot_id: String, body: String },
    /// Always fail `/context` with 503 unavailable.
    Unavailable,
}

/// A mock that serves the handshake, the apply-instructions body, and
/// `POST /context` per `behavior`. Everything else is 404.
fn context_mock(apply_body: &str, behavior: ContextBehavior) -> MockServer {
    let apply_body = apply_body.to_string();
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        let full = |s: &str| format!("/api/speclink/v1/projects/demo{s}");
        for req in looper.incoming_requests() {
            let method = req.method().to_string();
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            let inm = req
                .headers()
                .iter()
                .find(|h| h.field.to_string().eq_ignore_ascii_case("if-none-match"))
                .map(|h| h.value.to_string());
            let (status, body): (u16, String) = if method == "GET" && path == full("/binding") {
                (200, BINDING_BODY.to_string())
            } else if method == "GET" && path == full("/changes/demo/instructions/apply") {
                (200, apply_body.clone())
            } else if method == "POST" && path == full("/context") {
                match &behavior {
                    ContextBehavior::Unavailable => (
                        503,
                        r#"{"status":503,"reason":"unavailable","message":"down"}"#.to_string(),
                    ),
                    ContextBehavior::Serve { snapshot_id, body } => {
                        if inm.as_deref() == Some(snapshot_id.as_str()) {
                            (304, String::new())
                        } else {
                            (200, body.clone())
                        }
                    }
                }
            } else {
                (404, r#"{"reason":"not_found","message":"no route"}"#.to_string())
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

/// A full apply-flow context snapshot for `demo` with the given id: the change's
/// artifacts, its delta spec, the matching canonical spec, config and LANGUAGE —
/// digests computed with the contract digest so the materializer accepts it.
fn demo_snapshot(snapshot_id: &str) -> String {
    use speclink_host::projection::content_digest;
    use speclink_protocol::context::{ContextDocument, ContextSnapshot};
    let doc = |path: &str, content: &str| ContextDocument {
        path: path.to_string(),
        content: content.to_string(),
        revision: Some(1),
        digest: content_digest(content),
    };
    let documents = vec![
        doc("openspec/changes/demo/proposal.md", "## Why\n\nDemo change summary\n"),
        doc("openspec/changes/demo/design.md", "## Context\n\nDemo design\n"),
        doc("openspec/changes/demo/tasks.md", "- [x] 1.1 First\n- [x] 1.2 Second\n"),
        doc(
            "openspec/changes/demo/specs/cap-a/spec.md",
            "## MODIFIED Requirements\n\n### Requirement: Demo\n",
        ),
        doc("openspec/specs/cap-a/spec.md", "### Requirement: Demo\nDemo SHALL work.\n"),
        doc("openspec/config.yaml", "schema: spec-driven\n"),
        doc("openspec/LANGUAGE.md", "# Vocabulary\n"),
    ];
    let combined: Vec<&str> = documents.iter().map(|d| d.digest.as_str()).collect();
    let snapshot = ContextSnapshot {
        snapshot_id: snapshot_id.to_string(),
        policy_revision: Some(1),
        digest: content_digest(&combined.join("\n")),
        documents,
    };
    serde_json::to_string(&snapshot).unwrap()
}

// --- throwaway projects ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-remote-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    /// A remote-mode project bound to `url` (remote section in .speclink.yaml).
    fn remote(tag: &str, url: &str, repo: Option<&str>) -> TempProject {
        let p = TempProject::new(tag);
        let mut yaml = format!("remote:\n  url: {url}\n");
        if let Some(r) = repo {
            yaml.push_str(&format!("  repo: {r}\n"));
        }
        std::fs::write(p.dir.join(".speclink.yaml"), yaml).unwrap();
        p
    }

    /// A complete fs-mode change (all artifacts present, all tasks done) —
    /// the shape authority the remote output is compared against.
    fn fs_twin(tag: &str) -> TempProject {
        let p = TempProject::new(tag);
        let change = p.dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("cap-a")).unwrap();
        std::fs::write(
            change.join("proposal.md"),
            "## Why\n\nDemo change summary\n\n## What Changes\n\n- something\n",
        )
        .unwrap();
        std::fs::write(change.join("design.md"), "## Context\n\nDemo design\n").unwrap();
        std::fs::write(
            change.join("specs").join("cap-a").join("spec.md"),
            "## ADDED Requirements\n\n### Requirement: Demo\nDemo SHALL work.\n\n#### Scenario: works\n- **WHEN** run\n- **THEN** ok\n",
        )
        .unwrap();
        std::fs::write(
            change.join("tasks.md"),
            "## 1. Work\n\n- [x] 1.1 First\n- [x] 1.2 Second\n",
        )
        .unwrap();
        // Discussion twin for discuss list/show parity.
        let discussions = p.dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        std::fs::write(
            discussions.join("demo-topic.md"),
            "---\ntopic: Demo topic\nslug: demo-topic\nstatus: open\ncreated: 2026-07-01\n---\n\n# Discussion: Demo topic\n\n## Context\n\nSome context\n",
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

    /// Like [`run`], but redirects the store URL to `url` (a second server) via
    /// `SPECLINK_STORE_URL` — for tests that drive the same project against two
    /// servers in sequence.
    fn run_url(&self, args: &[&str], token: &str, url: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env("SPECLINK_TOKEN", token)
            .env("SPECLINK_STORE_URL", url)
            .output()
            .expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout_json(out: &Output) -> serde_json::Value {
    let text = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {text}\nstderr: {}", String::from_utf8_lossy(&out.stderr)))
}

/// Every JSON object key as a slash path; arrays are flattened so one element
/// stands for all. This compares field NAMES, not values.
fn key_paths(v: &serde_json::Value, prefix: &str, out: &mut BTreeSet<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                let p = format!("{prefix}/{k}");
                out.insert(p.clone());
                key_paths(val, &p, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                key_paths(item, prefix, out);
            }
        }
        _ => {}
    }
}

fn keys_of(v: &serde_json::Value) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    key_paths(v, "", &mut set);
    set
}

fn assert_same_keys(remote: &serde_json::Value, fs: &serde_json::Value, verb: &str) {
    let (r, f) = (keys_of(remote), keys_of(fs));
    assert_eq!(
        r, f,
        "{verb}: remote JSON field names diverge from fs mode\nremote-only: {:?}\nfs-only: {:?}",
        r.difference(&f).collect::<Vec<_>>(),
        f.difference(&r).collect::<Vec<_>>()
    );
}

// --- server payloads mirroring the fs twin ---

const LIST_BODY: &str = r#"{"changes":[{"name":"demo","summary":"Demo change summary","status":"done","completedTasks":2,"totalTasks":2,"repo":"backend","lifecycle":"applying","claimedBy":"me"}]}"#;

const STATUS_BODY: &str = r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":true,"applyRequires":["tasks"],"artifacts":[{"id":"proposal","outputPath":"proposal.md","status":"done","version":3},{"id":"design","outputPath":"design.md","status":"done","version":1},{"id":"specs","outputPath":"specs/**/*.md","status":"done","version":2},{"id":"tasks","outputPath":"tasks.md","status":"done","version":5}],"repo":"backend","lifecycle":"applying","statusVersion":4,"claimedBy":"me"}"#;

const APPLY_BODY: &str = r#"{"changeName":"demo","changeDir":"changes/demo","schemaName":"spec-driven","contextFiles":{"design":"design.md","proposal":"proposal.md","specs":"specs/**/*.md","tasks":"tasks.md"},"progress":{"total":2,"complete":2,"remaining":0,"codeTotal":2,"codeComplete":2,"codeRemaining":0},"tasks":[{"id":"1","description":"1.1 First","done":true,"parallel":false,"manual":false},{"id":"2","description":"1.2 Second","done":true,"parallel":false,"manual":false}],"state":"all_done","locale":"English","instruction":"Work through the tasks.\n"}"#;

const PROPOSAL_INSTR_BODY: &str = r###"{"changeName":"demo","artifactId":"proposal","schemaName":"spec-driven","changeDir":"changes/demo","outputPath":"proposal.md","description":"Initial proposal document outlining the change","instruction":"Create the proposal.\n","locale":"English","template":"## Why\n","dependencies":[],"unlocks":["design"]}"###;

const DISCUSS_LIST_BODY: &str = r#"{"discussions":[{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false}]}"#;

const DISCUSS_SHOW_BODY: &str = r#"{"info":{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false},"content":"---\ntopic: Demo topic\n---\n\n# Discussion: Demo topic\n"}"#;

// --- JSON field-name parity, verb by verb ---

#[test]
fn list_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![("GET", "/changes", LIST_BODY.to_string())]);
    let remote = TempProject::remote("list", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("list-fs");

    let r = remote.run(&["list", "--json"], Some("tok"));
    assert!(r.status.success(), "remote list failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["list", "--json"], None);
    assert!(f.status.success());
    assert_same_keys(&stdout_json(&r), &stdout_json(&f), "list --json");
    // Spec scenario remote list 恆無 worktree 欄位：那是本機主 checkout 的觀察面，
    // server 端不存在此概念，缺席即維持與 fs 無 worktree 情境的逐欄位一致。
    let payload = stdout_json(&r);
    for item in payload["changes"].as_array().expect("changes array") {
        assert!(item.get("worktree").is_none(), "remote item carries worktree: {item}");
    }
}

#[test]
fn status_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![("GET", "/changes/demo", STATUS_BODY.to_string())]);
    let remote = TempProject::remote("status", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("status-fs");

    let r = remote.run(&["status", "--change", "demo", "--json"], Some("tok"));
    assert!(r.status.success(), "remote status failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["status", "--change", "demo", "--json"], None);
    assert!(f.status.success());
    assert_same_keys(&stdout_json(&r), &stdout_json(&f), "status --json");
}

#[test]
fn instructions_apply_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/instructions/apply",
        APPLY_BODY.to_string(),
    )]);
    let remote = TempProject::remote("apply-instr", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("apply-instr-fs");

    let r = remote.run(&["instructions", "apply", "--change", "demo", "--json"], Some("tok"));
    assert!(r.status.success(), "remote instructions failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["instructions", "apply", "--change", "demo", "--json"], None);
    assert!(f.status.success());
    // `preflight` is deliberately fs-only (local file checks) — the contract
    // omits it in remote mode; every other field name must match.
    let mut fs_keys = keys_of(&stdout_json(&f));
    fs_keys.retain(|k| !k.starts_with("/preflight"));
    assert_eq!(keys_of(&stdout_json(&r)), fs_keys, "instructions apply --json field names");
}

// --- remote instructions 指向投影（context-projection）：以 Context API 為來源 ---

/// The projection root of a remote temp project.
fn projection_dir_of(remote: &TempProject) -> PathBuf {
    // macOS 的 temp_dir 在 /var → /private/var symlink 下，CLI 由 getcwd 回報實體
    // 路徑，故非 Windows 平台需解析才能與 CLI 輸出同底比對；Windows 的 canonicalize
    // 會加 \\?\ 前綴並把 8.3 短名展開（RUNNER~1 → runneradmin），兩者 CLI 輸出都沒有，
    // 反而讓同一個目錄變成兩種拼法。與 discuss_promote_snapshot.rs 同一處理。
    let dir = if cfg!(windows) {
        remote.dir.clone()
    } else {
        remote.dir.canonicalize().expect("canonicalize temp dir")
    };
    dir.join(".speclink").join("context")
}

#[test]
fn instructions_apply_materializes_the_projection_from_the_context_api() {
    let mock = context_mock(
        APPLY_BODY,
        ContextBehavior::Serve { snapshot_id: "snap-1".to_string(), body: demo_snapshot("snap-1") },
    );
    let remote = TempProject::remote("proj-apply", &mock.base, Some("backend"));

    let out = remote.run(&["instructions", "apply", "--change", "demo", "--json"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let payload = stdout_json(&out);
    let files = payload["contextFiles"].as_object().expect("contextFiles object");

    // key 與集合邏輯不變（server 宣告的 key 原樣保留）。
    let keys: Vec<&str> = files.keys().map(String::as_str).collect();
    assert_eq!(keys, ["design", "proposal", "specs", "tasks"]);

    // 每個值都指向投影下（.speclink/context 的 openspec 鏡像）。
    let projection = projection_dir_of(&remote);
    for (key, value) in files {
        let value = value.as_str().unwrap();
        assert!(
            PathBuf::from(value).starts_with(&projection),
            "{key} points into the projection: {value}"
        );
    }

    // 投影以 Context API 為來源：含正典 specs 與 delta specs（不再只有三個 artifact）。
    for rel in [
        "openspec/changes/demo/proposal.md",
        "openspec/changes/demo/design.md",
        "openspec/changes/demo/tasks.md",
        "openspec/changes/demo/specs/cap-a/spec.md",
        "openspec/specs/cap-a/spec.md",
    ] {
        assert!(projection.join(rel).is_file(), "{rel} is in the apply-flow projection");
    }

    // manifest snapshot id 為 server 回應的識別。
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(projection.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["snapshotId"], "snap-1", "manifest carries the server snapshot id");
    assert!(
        std::fs::read_to_string(projection.join("openspec/changes/demo/proposal.md"))
            .unwrap()
            .contains("Demo change summary"),
        "projected content is the server's"
    );
}

#[test]
fn an_unchanged_scope_skips_the_projection_rewrite() {
    let mock = context_mock(
        APPLY_BODY,
        ContextBehavior::Serve { snapshot_id: "snap-1".to_string(), body: demo_snapshot("snap-1") },
    );
    let remote = TempProject::remote("proj-unchanged", &mock.base, Some("backend"));

    // First run materializes the projection.
    let out1 = remote.run(&["instructions", "apply", "--change", "demo", "--json"], Some("tok"));
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let projection = projection_dir_of(&remote);
    assert!(projection.join("manifest.json").is_file(), "first run materialized");

    // A sentinel in the projection dir: a rewrite (staging + whole-directory
    // switch) would replace the directory and remove it.
    let sentinel = projection.join("SENTINEL");
    std::fs::write(&sentinel, "probe").unwrap();

    // Second run: the CLI sends the manifest id as If-None-Match; the scope is
    // unchanged, so the mock answers 304 and the projection is not rewritten.
    let out2 = remote.run(&["instructions", "apply", "--change", "demo", "--json"], Some("tok"));
    assert!(out2.status.success(), "stderr: {}", String::from_utf8_lossy(&out2.stderr));
    assert!(sentinel.is_file(), "the sentinel survives → the projection was not rewritten (免重寫)");

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(projection.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["snapshotId"], "snap-1", "the projection still holds the same snapshot");
}

#[test]
fn a_context_api_failure_marks_the_existing_projection_stale_without_blocking_the_verb() {
    // A healthy server materializes the projection first.
    let ok = context_mock(
        APPLY_BODY,
        ContextBehavior::Serve { snapshot_id: "snap-1".to_string(), body: demo_snapshot("snap-1") },
    );
    let remote = TempProject::remote("proj-stale", &ok.base, Some("backend"));
    let out1 = remote.run(&["instructions", "apply", "--change", "demo", "--json"], Some("tok"));
    assert!(out1.status.success(), "stderr: {}", String::from_utf8_lossy(&out1.stderr));
    let projection = projection_dir_of(&remote);
    assert!(projection.join("manifest.json").is_file(), "materialized");
    assert!(!projection.join("STALE").exists(), "not stale after a fresh materialize");

    // A failing Context API: the verb still completes, warns, and marks the
    // existing projection stale (韌性語意不變).
    let bad = context_mock(APPLY_BODY, ContextBehavior::Unavailable);
    let out2 = remote.run_url(&["instructions", "apply", "--change", "demo", "--json"], "tok", &bad.base);
    assert!(
        out2.status.success(),
        "the verb completes despite the projection failure: {}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("not refreshed"), "a loud warning is emitted: {stderr}");
    assert!(projection.join("STALE").is_file(), "the existing projection is marked stale");
    // The instructions payload itself is intact.
    assert_eq!(stdout_json(&out2)["changeName"], "demo", "the verb payload is unaffected");
}

#[test]
fn fs_instructions_apply_carries_no_projection_path() {
    let fs = TempProject::fs_twin("no-proj-fs");
    let f = fs.run(&["instructions", "apply", "--change", "demo", "--json"], None);
    assert!(f.status.success());
    let text = String::from_utf8_lossy(&f.stdout);
    assert!(
        !text.contains(".speclink"),
        "local mode never points at a projection: {text}"
    );
    assert!(!fs.dir.join(".speclink").exists(), "local mode writes no projection");
}

#[test]
fn instructions_artifact_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/instructions/proposal",
        PROPOSAL_INSTR_BODY.to_string(),
    )]);
    let remote = TempProject::remote("prop-instr", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("prop-instr-fs");

    let r = remote.run(&["instructions", "proposal", "--change", "demo", "--json"], Some("tok"));
    assert!(r.status.success(), "remote instructions failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["instructions", "proposal", "--change", "demo", "--json"], None);
    assert!(f.status.success());
    assert_same_keys(&stdout_json(&r), &stdout_json(&f), "instructions proposal --json");
}

#[test]
fn discuss_list_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![("GET", "/discussions", DISCUSS_LIST_BODY.to_string())]);
    let remote = TempProject::remote("disc-list", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("disc-list-fs");

    let r = remote.run(&["discuss", "list", "--json"], Some("tok"));
    assert!(r.status.success(), "remote discuss list failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["discuss", "list", "--json"], None);
    assert!(f.status.success());
    assert_same_keys(&stdout_json(&r), &stdout_json(&f), "discuss list --json");
}

#[test]
fn discuss_show_json_field_names_match_fs_mode() {
    let mock = mock_server(vec![(
        "GET",
        "/discussions/demo-topic",
        DISCUSS_SHOW_BODY.to_string(),
    )]);
    let remote = TempProject::remote("disc-show", &mock.base, Some("backend"));
    let fs = TempProject::fs_twin("disc-show-fs");

    let r = remote.run(&["discuss", "show", "demo-topic", "--json"], Some("tok"));
    assert!(r.status.success(), "remote discuss show failed: {}", String::from_utf8_lossy(&r.stderr));
    let f = fs.run(&["discuss", "show", "demo-topic", "--json"], None);
    assert!(f.status.success());
    assert_same_keys(&stdout_json(&r), &stdout_json(&f), "discuss show --json");
}

// --- CLI-level mode behaviors ---

#[test]
fn coexisting_spec_dir_warns_once_and_remote_wins() {
    let mock = mock_server(vec![("GET", "/changes", LIST_BODY.to_string())]);
    let remote = TempProject::remote("coexist", &mock.base, Some("backend"));
    std::fs::create_dir_all(remote.dir.join("openspec").join("changes")).unwrap();

    let out = remote.run(&["list", "--json"], Some("tok"));
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    // Remote won: stdout carries the server's change, not the (empty) local tree.
    let payload = stdout_json(&out);
    assert_eq!(payload["changes"][0]["name"], "demo");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let warnings: Vec<&str> = stderr
        .lines()
        .filter(|l| l.contains(".speclink.yaml") && l.contains("remote mode"))
        .collect();
    assert_eq!(warnings.len(), 1, "exactly one coexistence warning: {stderr}");
}

#[test]
fn remote_verb_without_login_points_at_auth_login() {
    let mock = mock_server(vec![("GET", "/changes", LIST_BODY.to_string())]);
    let remote = TempProject::remote("no-login", &mock.base, Some("backend"));

    let out = remote.run(&["list"], None);
    assert!(!out.status.success(), "must fail without credentials");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("speclink auth login"),
        "stderr must point at auth login: {stderr}"
    );
    assert!(out.stdout.is_empty(), "no data on stdout when not logged in");
}

#[test]
fn connection_failure_fails_loud_with_empty_stdout() {
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let remote = TempProject::remote("conn-fail", &base, Some("backend"));

    let out = remote.run(&["list"], Some("tok"));
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("server unreachable"),
        "stderr explains the failure: {stderr}"
    );
    assert!(out.stdout.is_empty(), "no cache fallback data on stdout");
}

#[test]
fn speclink_store_url_overrides_the_connection_url() {
    let mock = mock_server(vec![("GET", "/changes", LIST_BODY.to_string())]);
    // The connection file points at a dead port; the env var redirects to the mock.
    let dead_port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let dead = format!("http://127.0.0.1:{dead_port}/api/speclink/v1/projects/demo");
    let remote = TempProject::remote("env-url", &dead, Some("backend"));

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
    let out = cmd
        .args(["list", "--json"])
        .current_dir(&remote.dir)
        .env_remove("SPECLINK_LOCALE")
        .env_remove("SPECLINK_SPEC_LOCALE")
        .env_remove("SPECLINK_TDD")
        .env_remove("SPECLINK_AUDIT")
        .env("SPECLINK_TOKEN", "tok")
        .env("SPECLINK_STORE_URL", &mock.base)
        .output()
        .expect("run speclink binary");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(stdout_json(&out)["changes"][0]["name"], "demo");
}

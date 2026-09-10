//! Remote `drift` (server-drift-api spec「remote drift 合併於 client 且輸出凍結」).
//! The verb takes the spec side from the server, computes the workspace side
//! locally, and merges through the Engine's one merger — so its output shape is
//! fs mode's, which stays the authority. Plus the honest no-checkout case
//! (unavailable is not clean) and the server-failure case (no partial report).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Arc;

// --- mock server ---

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
}

const BINDING_BODY: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;

/// Routes: (method, suffix after the project base, status, body). Unmatched
/// requests get 404. The handshake route is injected unless declared.
fn mock_server(mut routes: Vec<(&'static str, String, u16, String)>) -> MockServer {
    if !routes.iter().any(|(_, suffix, _, _)| suffix == "/binding") {
        routes.push(("GET", "/binding".to_string(), 200, BINDING_BODY.to_string()));
    }
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

// --- the change content both modes see ---

const CREATED: &str = "2026-07-13";
const META: &str = "schema: spec-driven\ncreated: 2026-07-13\n";
const DESIGN: &str = "## Context\n\nUses `Widget_kind`, `Missing_sym` and `src/app.rs`.\n";
const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 wire `src/app.rs`\n";
/// The delta MODIFIES a requirement the canonical spec no longer has — one
/// stale assumption, so the Specs dimension is a non-zero score in both modes.
const DELTA: &str = "## MODIFIED Requirements\n\n### Requirement: Rotate tokens\n\nIt SHALL rotate.\n";
const CANONICAL: &str = "## Purpose\n\nA\n\n## Requirements\n\n### Requirement: Sign in\n\nIt SHALL sign in.\n";

/// The drift response the server serves for this change — the spec side the
/// engine would compute for the content above, plus its basis and inputs.
fn drift_body() -> String {
    serde_json::json!({
        "specDrift": {
            "dimension": {
                "kind": "Specs",
                "status": "1 stale assumptions",
                "score": 4,
                "contributesToTotal": true
            },
            "specAssumptions": [{
                "capability": "auth",
                "operation": "MODIFIED",
                "requirement": "Rotate tokens",
                "reason": "target requirement no longer exists in the canonical spec"
            }]
        },
        "basis": { "spec": "sha256:aaa", "tasks": "sha256:bbb", "policy": "sha256:ccc" },
        "change": { "created": CREATED, "design": DESIGN, "tasks": TASKS }
    })
    .to_string()
}

// --- temp projects ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-remote-drift-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    /// A remote-mode project bound to `url`, with no local `openspec/`.
    fn remote(tag: &str, url: &str) -> TempProject {
        let p = TempProject::new(tag);
        std::fs::write(
            p.dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: backend\n"),
        )
        .unwrap();
        p
    }

    /// The fs-mode twin of the same change content — the shape authority.
    fn fs_twin(tag: &str) -> TempProject {
        let p = TempProject::new(tag);
        let change = p.dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("auth")).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("design.md"), DESIGN).unwrap();
        std::fs::write(change.join("tasks.md"), TASKS).unwrap();
        std::fs::write(change.join("specs").join("auth").join("spec.md"), DELTA).unwrap();
        let specs = p.dir.join("openspec").join("specs").join("auth");
        std::fs::create_dir_all(&specs).unwrap();
        std::fs::write(specs.join("spec.md"), CANONICAL).unwrap();
        p
    }

    /// Seed the code checkout both modes collect workspace facts from: a git
    /// repo where `Widget_kind` and `src/app.rs` exist but `Missing_sym` does not.
    fn init_git_checkout(&self) {
        self.write("src/lib.rs", "pub struct Widget_kind;\n");
        self.write("src/app.rs", "// app\n");
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "Sandbox Tester"],
            vec!["config", "user.email", "sandbox@example.com"],
            vec!["add", "-A"],
            vec!["commit", "-q", "-m", "seed"],
        ] {
            let ok = Command::new("git")
                .args(&args)
                .current_dir(&self.dir)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        }
    }

    fn write(&self, rel: &str, content: &str) {
        let p = self.dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
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

fn stdout_json(out: &Output) -> serde_json::Value {
    let text = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("stdout is not JSON ({e}): {text}\nstderr: {}", String::from_utf8_lossy(&out.stderr))
    })
}

/// Every key path in a JSON value — the structural shape, independent of values.
fn shape(v: &serde_json::Value, prefix: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    match v {
        serde_json::Value::Object(map) => {
            for (k, val) in map {
                let path = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                out.insert(path.clone());
                out.extend(shape(val, &path));
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                out.extend(shape(item, &format!("{prefix}[]")));
            }
        }
        _ => {}
    }
    out
}

fn routes_serving_drift() -> Vec<(&'static str, String, u16, String)> {
    vec![("GET", "/changes/demo/drift".to_string(), 200, drift_body())]
}

// --- Scenario: 有 checkout 輸出同形 ---

#[test]
fn remote_and_fs_drift_json_have_the_same_shape_and_spec_side() {
    let mock = mock_server(routes_serving_drift());

    let fs = TempProject::fs_twin("fstwin");
    fs.init_git_checkout();
    let fs_out = fs.run(&["drift", "demo", "--json"], None);
    assert!(fs_out.status.success(), "fs drift: {}", String::from_utf8_lossy(&fs_out.stderr));
    let fs_json = stdout_json(&fs_out);

    let remote = TempProject::remote("remotetwin", &mock.base);
    remote.init_git_checkout();
    let remote_out = remote.run(&["drift", "demo", "--json"], Some("tok"));
    assert!(
        remote_out.status.success(),
        "remote drift: {}",
        String::from_utf8_lossy(&remote_out.stderr)
    );
    let remote_json = stdout_json(&remote_out);

    assert_eq!(
        shape(&remote_json, ""),
        shape(&fs_json, ""),
        "remote drift --json 的結構與 fs 模式同形\nremote: {remote_json:#}\nfs: {fs_json:#}"
    );

    // 規格面內容相同：維度、規格假設、總分與建議都由同一組事實得出。
    let spec_dim = |j: &serde_json::Value| {
        j["dimensions"]
            .as_array()
            .expect("dimensions")
            .iter()
            .find(|d| d["kind"] == "Specs")
            .cloned()
            .expect("a Specs dimension")
    };
    assert_eq!(spec_dim(&remote_json), spec_dim(&fs_json), "規格面維度相同");
    assert_eq!(
        remote_json["spec_assumptions"], fs_json["spec_assumptions"],
        "規格假設相同"
    );
    assert_eq!(remote_json["total_score"], fs_json["total_score"], "總分相同");
    assert_eq!(
        remote_json["primary_recommendation"], fs_json["primary_recommendation"],
        "建議相同"
    );

    // 工作區面反映本機事實：committed 的錨點命中，缺席的沒有。
    let broken: Vec<String> = remote_json["broken_anchors"]
        .as_array()
        .expect("broken_anchors")
        .iter()
        .map(|b| b["anchor"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(broken.contains(&"Missing_sym".to_string()), "未命中的符號被列出: {broken:?}");
    assert!(!broken.contains(&"Widget_kind".to_string()), "已提交的符號未被列為 broken");
    assert_eq!(remote_json["broken_anchors"], fs_json["broken_anchors"], "工作區面亦逐項相同");
}

// --- Scenario: 無 checkout 誠實標示 ---

#[test]
fn without_a_checkout_the_workspace_side_is_unavailable_never_clean() {
    let mock = mock_server(routes_serving_drift());
    // 無 openspec/、無 git —— 只有 .speclink.yaml。
    let remote = TempProject::remote("nocheckout", &mock.base);

    let out = remote.run(&["drift", "demo", "--json"], Some("tok"));
    assert!(out.status.success(), "動詞成功: {}", String::from_utf8_lossy(&out.stderr));
    let json = stdout_json(&out);

    // 規格面照常回報。
    let dims = json["dimensions"].as_array().expect("dimensions");
    let specs = dims.iter().find(|d| d["kind"] == "Specs").expect("a Specs dimension");
    assert_eq!(specs["score"], 4, "規格面照常回報: {json:#}");

    // 四個工作區維度皆標示不可得，且不計入總分——不可得不等於乾淨。
    for kind in ["Time", "Structure", "Tasks", "Environment"] {
        let d = dims.iter().find(|d| d["kind"] == kind).unwrap_or_else(|| panic!("{kind} 維度"));
        assert_eq!(d["status"], "unavailable", "{kind} 標示不可得: {d}");
        assert_eq!(d["contributes_to_total"], false, "{kind} 不計入總分");
    }
    assert_eq!(json["coverage"], "spec-only", "涵蓋面標示為僅規格面");

    // 不出現任何「乾淨」的斷言。
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(!text.contains("clean"), "不宣稱工作區乾淨: {text}");
}

// --- Scenario: server 失敗不出偽報告 ---

#[test]
fn a_server_failure_fails_the_verb_without_a_partial_report() {
    let mock = mock_server(vec![(
        "GET",
        "/changes/demo/drift".to_string(),
        503,
        r#"{"reason":"unavailable","message":"the store backend is temporarily unavailable"}"#
            .to_string(),
    )]);
    let remote = TempProject::remote("serverdown", &mock.base);
    remote.init_git_checkout();

    let out = remote.run(&["drift", "demo", "--json"], Some("tok"));
    assert!(!out.status.success(), "動詞以非零 exit code 失敗");
    assert!(
        out.stdout.is_empty(),
        "不輸出缺規格面的部分報告: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    // 動詞不自造訊息：走 speclink-remote 既有的 registry 翻譯（凍結文字）。
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(
            "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)"
        ),
        "以既有 remote 錯誤訊息失敗: {stderr}"
    );
}

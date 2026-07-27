//! `speclink workflow-config` — the CLI verb that manages `openspec/config.yaml`
//! (workflow policy, context, rules), in both fs and remote mode.
//!
//! fs mode reads and writes the file through the workspace; remote mode reads
//! the server document with its revision, applies the SAME core rewrite, and
//! writes back guarded by that revision. Both share one rendering path, so the
//! `--json` payload shape is asserted identically for each.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};

// --- fs fixture ---

/// A workflow config with a value in every surface the verb touches: two policy
/// keys set (locale, tdd), two left unset (spec_locale, audit), a context block,
/// and two rule sections.
const WF_YAML: &str = "\
schema: spec-driven
locale: tw
tdd: true
context: |
  Project context line one.
  Line two.
rules:
  design:
  - Name the crate boundary
  - List one alternative
  tasks:
  - Use checkbox format
";

const BAD_YAML: &str = ": not yaml : [\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, wf_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-workflow-config-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::write(dir.join("openspec").join("config.yaml"), wf_yaml).unwrap();
        TempProject { dir }
    }

    fn config_path(&self) -> PathBuf {
        self.dir.join("openspec").join("config.yaml")
    }

    fn config_bytes(&self) -> Vec<u8> {
        std::fs::read(self.config_path()).expect("read config.yaml")
    }

    fn config_text(&self) -> String {
        std::fs::read_to_string(self.config_path()).expect("read config.yaml")
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir).env("NO_COLOR", "1");
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = self.cmd(args);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], stdin: &str) -> Output {
        run_with_stdin(self.cmd(args), stdin)
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn run_with_stdin(mut cmd: Command, stdin: &str) -> Output {
    let mut child = cmd
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

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("stdout is JSON ({e}): {}", stdout_of(out));
    })
}

// --- show (fs) ---

#[test]
fn show_prints_canonical_policy_context_and_rules() {
    // Spec scenario fs 模式顯示正典值.
    let p = TempProject::new("show-human", WF_YAML);
    let out = p.run(&["workflow-config", "show"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("locale"), "policy fields listed: {text}");
    assert!(text.contains("tw"), "locale value shown: {text}");
    let tdd_line = text.lines().find(|l| l.contains("tdd")).unwrap_or_default();
    assert!(tdd_line.contains("on"), "tdd shown as on: {text}");
    let audit_line = text.lines().find(|l| l.contains("audit")).unwrap_or_default();
    assert!(
        audit_line.contains("unset") && audit_line.contains("off"),
        "audit shown as unset with its default: {text}"
    );
    let context_line = text.lines().find(|l| l.contains("context")).unwrap_or_default();
    assert!(context_line.contains('2'), "context line count shown: {text}");
    let rules_line = text.lines().find(|l| l.contains("rules")).unwrap_or_default();
    assert!(
        rules_line.contains("design") && rules_line.contains('2'),
        "rules section counts shown: {text}"
    );
    assert!(!text.contains('\u{1b}'), "no ANSI escapes: {text:?}");
}

#[test]
fn show_json_payload_is_camel_case_with_null_for_unset() {
    // Spec scenario --json payload 形狀.
    let p = TempProject::new("show-json", WF_YAML);
    let out = p.run(&["workflow-config", "show", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload = json_of(&out);
    assert_eq!(payload["locale"], "tw");
    assert!(payload["specLocale"].is_null(), "unset specLocale is null: {payload}");
    assert_eq!(payload["tdd"], true);
    assert_eq!(payload["audit"], false, "unset tdd/audit read as false");
    assert!(
        payload["context"].as_str().unwrap().contains("Project context line one."),
        "context text carried: {payload}"
    );
    assert_eq!(
        payload["rules"]["design"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["Name the crate boundary", "List one alternative"]
    );
    let keys: Vec<&str> = payload.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert!(!keys.contains(&"spec_locale"), "no snake_case keys: {keys:?}");
}

#[test]
fn show_json_reports_the_canonical_value_not_the_env_override() {
    // Spec scenario show 不應用環境變數覆寫.
    let p = TempProject::new("show-env", WF_YAML);
    let out = p.run_env(
        &["workflow-config", "show", "--json"],
        &[("SPECLINK_TDD", "false")],
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(json_of(&out)["tdd"], true, "canonical value wins over the env layer");
}

#[test]
fn show_fails_closed_on_an_unparseable_config() {
    // Spec scenario 壞 config fail-closed.
    let p = TempProject::new("show-bad", BAD_YAML);
    let out = p.run(&["workflow-config", "show"]);
    assert!(!out.status.success(), "must exit non-zero on a broken config");
    assert!(
        stderr_of(&out).contains("openspec/config.yaml"),
        "stderr names the file: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).trim().is_empty(), "no payload on stdout");
}

// --- set (fs) ---

#[test]
fn set_locale_writes_the_key_and_preserves_the_others() {
    // Spec scenario 設定 locale 保留其他鍵.
    let p = TempProject::new("set-locale", WF_YAML);
    let out = p.run(&["workflow-config", "set", "locale", "ja"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(after.contains("locale: ja"), "locale rewritten: {after}");
    assert!(after.contains("schema: spec-driven"), "schema preserved: {after}");
    assert!(after.contains("tdd: true"), "tdd preserved: {after}");
    assert!(after.contains("Project context line one."), "context preserved: {after}");
    assert!(after.contains("Name the crate boundary"), "rules preserved: {after}");
}

#[test]
fn set_rejects_an_unknown_key_without_touching_the_file() {
    // Spec scenario 未知 key 拒絕.
    let p = TempProject::new("set-unknown", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "theme", "dark"]);
    assert!(!out.status.success(), "unknown key must exit non-zero");
    let err = stderr_of(&out);
    for key in ["locale", "spec_locale", "tdd", "audit"] {
        assert!(err.contains(key), "stderr lists the accepted keys ({key}): {err}");
    }
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_rejects_a_non_boolean_toggle_value() {
    // Spec scenario 非法布林值拒絕.
    let p = TempProject::new("set-bad-bool", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "tdd", "yes"]);
    assert!(!out.status.success(), "non-boolean value must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("tdd"), "stderr names the key: {err}");
    assert!(
        err.contains("true") && err.contains("false"),
        "stderr states the accepted values: {err}"
    );
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn set_false_removes_the_key() {
    // Spec scenario 設 false 移除鍵.
    let p = TempProject::new("set-false", "schema: spec-driven\naudit: true\n");
    let out = p.run(&["workflow-config", "set", "audit", "false"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(!after.contains("audit"), "audit key removed (unset = default off): {after}");
    assert!(after.contains("schema: spec-driven"), "schema preserved: {after}");
}

#[test]
fn set_dry_run_prints_a_unified_diff_and_writes_nothing() {
    // Spec scenario dry-run 印 diff 不落檔.
    let p = TempProject::new("set-dry-run", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "ja", "--dry-run"]);
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("--- a/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("+++ b/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("-locale: tw"), "removed line: {diff}");
    assert!(diff.contains("+locale: ja"), "added line: {diff}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_refuses_to_rewrite_an_unparseable_config() {
    // Spec scenario 壞 config 拒絕寫入.
    let p = TempProject::new("set-bad", BAD_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "tw"]);
    assert!(!out.status.success(), "must exit non-zero on a broken config");
    let err = stderr_of(&out);
    assert!(err.contains("openspec/config.yaml"), "stderr names the file: {err}");
    assert!(err.contains("refused"), "stderr says the write was refused: {err}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

// --- context (fs) ---

#[test]
fn context_stdin_sets_the_multiline_value() {
    // Spec scenario context 設定多行內容.
    let p = TempProject::new("context-set", WF_YAML);
    let out = p.run_stdin(
        &["workflow-config", "context", "--stdin"],
        "## Project\n\nA rewritten description.\nSecond line.\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(after.contains("A rewritten description."), "context written: {after}");
    assert!(after.contains("Second line."), "every line written: {after}");
    assert!(!after.contains("Project context line one."), "old context replaced: {after}");
    assert!(after.contains("locale: tw"), "policy fields untouched: {after}");
    assert!(after.contains("tdd: true"), "policy fields untouched: {after}");
    assert!(after.contains("Name the crate boundary"), "rules untouched: {after}");
}

#[test]
fn context_blank_stdin_removes_the_key() {
    // Spec scenario 空白 stdin 移除 context.
    let p = TempProject::new("context-clear", WF_YAML);
    let out = p.run_stdin(&["workflow-config", "context", "--stdin"], "   \n\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(!after.contains("context"), "context key removed: {after}");
    assert!(after.contains("locale: tw"), "policy fields untouched: {after}");
}

#[test]
fn context_without_the_stdin_flag_fails_with_usage() {
    // Spec: 兩個子指令未帶 --stdin 時以非零 exit code 說明用法.
    let p = TempProject::new("context-no-stdin", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(&["workflow-config", "context"], "ignored\n");
    assert!(!out.status.success(), "missing --stdin must exit non-zero");
    assert!(
        stderr_of(&out).contains("--stdin"),
        "stderr states the usage: {}",
        stderr_of(&out)
    );
    assert_eq!(p.config_bytes(), before, "no file effect");
}

// --- rules (fs) ---

#[test]
fn rules_replaces_the_whole_section() {
    // Spec scenario rules 整節代換.
    let p = TempProject::new("rules-replace", WF_YAML);
    let out = p.run_stdin(
        &["workflow-config", "rules", "design", "--stdin"],
        "First rule\nSecond rule\nThird rule\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.run(&["workflow-config", "show", "--json"]);
    let payload = json_of(&after);
    assert_eq!(
        payload["rules"]["design"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["First rule", "Second rule", "Third rule"],
        "the section is replaced wholesale, not merged"
    );
    assert_eq!(
        payload["rules"]["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["Use checkbox format"],
        "other artifact sections are untouched"
    );
}

#[test]
fn rules_empty_stdin_removes_the_section() {
    // Spec: stdin 為空時移除該 artifact 節.
    let p = TempProject::new("rules-clear", WF_YAML);
    let out = p.run_stdin(&["workflow-config", "rules", "design", "--stdin"], "\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload = json_of(&p.run(&["workflow-config", "show", "--json"]));
    assert!(payload["rules"]["design"].is_null(), "design section removed: {payload}");
    assert!(!payload["rules"]["tasks"].is_null(), "tasks section kept: {payload}");
}

#[test]
fn rules_rejects_an_artifact_outside_the_schema() {
    // Spec scenario 未知 artifact 拒絕.
    let p = TempProject::new("rules-unknown", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(
        &["workflow-config", "rules", "blueprint", "--stdin"],
        "Anything\n",
    );
    assert!(!out.status.success(), "unknown artifact must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("blueprint"), "stderr names the artifact: {err}");
    assert!(err.contains("proposal"), "stderr lists the schema's artifacts: {err}");
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn rules_dry_run_prints_a_unified_diff_and_writes_nothing() {
    // Spec scenario rules dry-run 印 diff.
    let p = TempProject::new("rules-dry-run", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(
        &["workflow-config", "rules", "tasks", "--stdin", "--dry-run"],
        "Only one rule now\n",
    );
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("--- a/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("+  - Only one rule now"), "added rule line: {diff}");
    assert!(diff.contains("-  - Use checkbox format"), "removed rule line: {diff}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

// --- remote mode ---

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

    fn none(&self, method: &str, path_suffix: &str) -> bool {
        let caps = self.captured.lock().unwrap();
        !caps.iter().any(|c| {
            c.method == method && c.path == format!("/api/speclink/v1/projects/demo{path_suffix}")
        })
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

struct RemoteProject {
    dir: PathBuf,
}

impl RemoteProject {
    fn new(tag: &str, url: &str) -> RemoteProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-workflow-config-remote-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: backend\n"),
        )
        .unwrap();
        RemoteProject { dir }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env("NO_COLOR", "1")
            .env("SPECLINK_TOKEN", "tok");
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], stdin: &str) -> Output {
        run_with_stdin(self.cmd(args), stdin)
    }
}

impl Drop for RemoteProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// `GET /config` body carrying the fs fixture document at revision 7.
fn config_response() -> String {
    serde_json::json!({
        "schema": "spec-driven",
        "content": WF_YAML,
        "revision": 7,
    })
    .to_string()
}

#[test]
fn remote_show_json_has_the_same_shape_as_fs() {
    // Spec scenario remote 模式輸出形狀一致.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let p = RemoteProject::new("show", &mock.base);
    let out = p.run(&["workflow-config", "show", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let remote = json_of(&out);

    let fs = TempProject::new("remote-shape", WF_YAML);
    let fs_out = fs.run(&["workflow-config", "show", "--json"]);
    assert!(fs_out.status.success(), "stderr: {}", stderr_of(&fs_out));
    assert_eq!(remote, json_of(&fs_out), "remote and fs payloads are identical");
}

#[test]
fn remote_set_reads_then_writes_back_with_the_read_revision() {
    // Spec: remote 先讀 server 端現行內容與版本、套用同一改寫、寫回時附帶讀得的版本.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        ("PUT", "/config", 200, r#"{"revision":8}"#.into()),
    ]);
    let p = RemoteProject::new("set", &mock.base);
    let out = p.run(&["workflow-config", "set", "audit", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let put = mock.find("PUT", "/config");
    let body: serde_json::Value = serde_json::from_str(&put.body).expect("PUT body is JSON");
    assert_eq!(body["expectedRevision"], 7, "the read revision guards the write");
    let content = body["content"].as_str().expect("content string");
    assert!(content.contains("audit: true"), "the edit landed: {content}");
    assert!(content.contains("locale: tw"), "other keys preserved: {content}");
    assert!(
        !stdout_of(&out).contains('7') && !stdout_of(&out).contains("revision"),
        "the revision never surfaces in the command output: {}",
        stdout_of(&out)
    );
}

#[test]
fn remote_context_write_round_trips_through_show() {
    // Spec scenario remote context 寫入經版本檢查.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        ("PUT", "/config", 200, r#"{"revision":8}"#.into()),
    ]);
    let p = RemoteProject::new("context", &mock.base);
    let out = p.run_stdin(
        &["workflow-config", "context", "--stdin"],
        "Remote project description.\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let put = mock.find("PUT", "/config");
    let body: serde_json::Value = serde_json::from_str(&put.body).expect("PUT body is JSON");
    let content = body["content"].as_str().expect("content string");
    assert!(
        content.contains("Remote project description."),
        "context written to the server document: {content}"
    );
}

#[test]
fn remote_revision_conflict_asks_for_a_rerun_without_overwriting() {
    // Spec scenario remote 版本衝突提示重跑.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        (
            "PUT",
            "/config",
            409,
            r#"{"status":409,"reason":"revision_conflict","message":"stale write"}"#.into(),
        ),
    ]);
    let p = RemoteProject::new("conflict", &mock.base);
    let out = p.run(&["workflow-config", "set", "tdd", "true"]);
    assert!(!out.status.success(), "a CAS refusal must exit non-zero");
    let err = stderr_of(&out);
    assert!(
        err.contains("updated by someone else") && err.contains("re-run"),
        "stderr explains the conflict and the fix: {err}"
    );
}

#[test]
fn remote_dry_run_prints_the_diff_without_writing() {
    // Spec: --dry-run 行為與 fs 一致（remote 亦不送出寫入）.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let p = RemoteProject::new("dry-run", &mock.base);
    let out = p.run(&["workflow-config", "set", "locale", "ja", "--dry-run"]);
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("+locale: ja"), "added line: {diff}");
    assert!(mock.none("PUT", "/config"), "dry-run sends no write request");
}

#[test]
fn remote_offline_fails_semantically_without_queueing() {
    // Spec: 連線離線時以非零 exit code 失敗並輸出語義化訊息，不暫存或排隊寫入.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let base = mock.base.clone();
    let p = RemoteProject::new("offline", &base);
    drop(mock); // server gone — the connection is refused
    let out = p.run(&["workflow-config", "set", "tdd", "true"]);
    assert!(!out.status.success(), "offline must exit non-zero");
    let err = stderr_of(&out);
    assert!(
        err.contains("unreachable") || err.contains("connection"),
        "stderr is a semantic connection failure: {err}"
    );
    assert!(
        !p.dir.join(".speclink").exists(),
        "nothing is queued or spooled locally"
    );
}

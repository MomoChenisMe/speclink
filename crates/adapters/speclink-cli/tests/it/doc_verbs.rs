//! Store-document verbs: `speclink artifact cat` and `speclink language
//! show` behave identically in fs mode (reads the local file) and remote mode
//! (contract endpoint), and a missing document is a loud, semantic failure.
//! `speclink language replace` writes the whole vocabulary document and
//! `speclink discuss restore` brings an archived discussion back; their fs
//! mode is here, their remote parity in `remote_verb_parity`.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Arc;

// --- minimal mock server ---

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
}

/// The compatible handshake every verb-focused test needs — the binding
/// precedes any verb, so the mock always serves it unless a test overrides
/// the route to probe handshake failures.
const BINDING_BODY: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0"}"#;

fn mock_server(mut routes: Vec<(&'static str, &'static str, u16, String)>) -> MockServer {
    if !routes.iter().any(|(_, suffix, _, _)| *suffix == "/binding") {
        routes.push(("GET", "/binding", 200, BINDING_BODY.to_string()));
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
        let change = p.dir.join("speclink").join("changes").join("demo");
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

    /// The binary on `args` in this project, with the environment that could
    /// change its behavior taken away.
    fn command(&self, args: &[&str], token: Option<&str>) -> Command {
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
        cmd
    }

    fn run(&self, args: &[&str], token: Option<&str>) -> Output {
        self.command(args, token).output().expect("run speclink binary")
    }

    /// Run with `input` on stdin (fs mode).
    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        use std::io::Write;
        let mut child = self
            .command(args, None)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        // A verb that fails before it reads leaves a broken pipe: the
        // assertion then sees the real stderr.
        if let Err(e) = child.stdin.take().unwrap().write_all(input.as_bytes()) {
            assert!(e.kind() == std::io::ErrorKind::BrokenPipe, "write stdin failed: {e}");
        }
        child.wait_with_output().expect("wait speclink binary")
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
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
        r#"{"status":404,"reason":"not_found","message":"artifact 'design' not found — run `speclink list` (or the matching list verb) to check the name"}"#.into(),
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
        p.dir.join("speclink").join("LANGUAGE.md"),
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
        r#"{"status":404,"reason":"not_found","message":"language document 'LANGUAGE' not found — the project has no shared vocabulary yet"}"#.into(),
    )]);
    let p = TempProject::remote("lang-remote-missing", &mock.base);
    let out = p.run(&["language", "show"], Some("tok"));
    assert!(!out.status.success());
    let stderr = stderr_of(&out).to_ascii_lowercase();
    assert!(stderr.contains("language"), "names the document: {stderr}");
}

// --- language replace (spec verb-contract「詞彙文件的寫入動詞」) ---

fn language_file(p: &TempProject) -> PathBuf {
    p.dir.join("speclink").join("LANGUAGE.md")
}

#[test]
fn fs_language_replace_creates_the_document_and_show_reads_it_back() {
    // Scenario「文件不存在時建立」。
    let p = TempProject::fs_project("lang-replace-create");
    assert!(!p.run(&["language", "show"], None).status.success(), "no document yet");

    let out = p.run_stdin(&["language", "replace", "--stdin", "--json"], "# Language\n");

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("stdout is JSON");
    assert_eq!(value, serde_json::json!({ "created": true }));
    assert_eq!(std::fs::read_to_string(language_file(&p)).unwrap(), "# Language\n");
    let show = p.run(&["language", "show"], None);
    assert!(show.status.success(), "stderr: {}", stderr_of(&show));
    assert_eq!(stdout_of(&show), "# Language\n");
}

#[test]
fn fs_language_replace_replaces_the_whole_document() {
    // Scenario「取代既有的詞彙文件」：整份取代，不合併；內容原樣落地。
    let p = TempProject::fs_project("lang-replace-whole");
    p.write("speclink/LANGUAGE.md", "# Old\n\n- term: 很長的一段舊內容，取代後不能留下尾巴\n");
    let content = "# Language\n\n## 詞彙\n";

    let out = p.run_stdin(&["language", "replace", "--stdin"], content);

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ Replaced the LANGUAGE document\n");
    assert_eq!(stderr_of(&out), "");
    assert_eq!(std::fs::read_to_string(language_file(&p)).unwrap(), content);
    assert_eq!(stdout_of(&p.run(&["language", "show"], None)), content);

    // A new document says so; --no-color changes nothing but the color codes.
    let fresh = TempProject::fs_project("lang-replace-fresh");
    let created = fresh.run_stdin(&["language", "replace", "--stdin", "--no-color"], content);
    assert_eq!(stdout_of(&created), "✓ Created the LANGUAGE document\n");
    let as_json = p.run_stdin(&["language", "replace", "--stdin", "--json"], content);
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&as_json)).expect("stdout is JSON");
    assert_eq!(value, serde_json::json!({ "created": false }));
}

#[test]
fn fs_language_replace_refuses_empty_content_and_keeps_the_document() {
    // Scenario「空內容被拒絕」，Example 的後兩列：空字串與只有空白。
    let p = TempProject::fs_project("lang-replace-empty");
    let file = p.write("speclink/LANGUAGE.md", "# Old\n");
    for stdin in ["", "   \n\n"] {
        let out = p.run_stdin(&["language", "replace", "--stdin"], stdin);
        assert!(!out.status.success(), "{stdin:?} must fail");
        assert_eq!(stdout_of(&out), "", "{stdin:?}: nothing on stdout");
        assert!(stderr_of(&out).contains("empty"), "{stdin:?}: {}", stderr_of(&out));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "# Old\n", "{stdin:?}: unchanged");
    }
}

#[test]
fn language_replace_without_the_stdin_flag_is_a_usage_error() {
    let p = TempProject::fs_project("lang-replace-noflag");
    let file = p.write("speclink/LANGUAGE.md", "# Old\n");
    let out = p.run_stdin(&["language", "replace"], "# New\n");
    assert_eq!(out.status.code(), Some(2), "stderr: {}", stderr_of(&out));
    assert!(stderr_of(&out).contains("--stdin"), "{}", stderr_of(&out));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "# Old\n");
}

// --- discuss restore (spec discussion-docs「討論以 restore 動詞搬回進行中」) ---

fn archived_record(created: &str, note: &str) -> String {
    format!(
        "---\ntopic: Demo topic\nslug: demo-topic\nstatus: concluded\ncreated: {created}\n---\n\n## Context\n\n{note}  \n\n## Rounds\n\n### Round 1 — interview ({created})\n\n**Focus**: scope\n\n## Conclusion\n\n**Decision**: ship it\n"
    )
}

#[test]
fn fs_discuss_restore_brings_the_archived_record_back_unchanged() {
    // Scenario「搬回被隨行封存的討論」。
    let p = TempProject::fs_project("restore-one");
    let record = archived_record("2026-11-10", "被隨行封存的記錄");
    let archived = p.write("speclink/discussions/archive/2026-11-10-demo-topic.md", &record);

    let out = p.run(&["discuss", "restore", "demo-topic"], None);

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ Restored discussion: demo-topic\n  From: 2026-11-10-demo-topic.md\n");
    assert!(!archived.exists(), "the record left the archive");
    let live = p.dir.join("speclink").join("discussions").join("demo-topic.md");
    assert_eq!(std::fs::read_to_string(&live).unwrap(), record, "byte-for-byte");
    assert_eq!(stdout_of(&p.run(&["discuss", "show", "demo-topic"], None)), record);
    assert!(stdout_of(&p.run(&["discuss", "list"], None)).contains("demo-topic"));
    let archived_list = stdout_of(&p.run(&["discuss", "list", "--archived"], None));
    assert!(!archived_list.contains("demo-topic"), "{archived_list}");
    // 搬回之後它是進行中的記錄：續寫成功。
    let round = p.run_stdin(
        &["discuss", "add-round", "demo-topic", "--mode", "explore", "--stdin"],
        "**Focus**: again\n**Position**: more\n",
    );
    assert!(round.status.success(), "stderr: {}", stderr_of(&round));
}

#[test]
fn fs_discuss_restore_takes_the_newest_of_several_archived_records() {
    // Scenario「多份封存時搬回最新的一份」。
    let p = TempProject::fs_project("restore-newest");
    let older = archived_record("2026-09-01", "舊的一份");
    let newer = archived_record("2026-11-10", "新的一份");
    let old_file = p.write("speclink/discussions/archive/2026-09-01-demo-topic.md", &older);
    p.write("speclink/discussions/archive/2026-11-10-demo-topic.md", &newer);

    let out = p.run(&["discuss", "restore", "demo-topic", "--json"], None);

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let value: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("stdout is JSON");
    assert_eq!(value, serde_json::json!({ "slug": "demo-topic", "restoredFrom": "2026-11-10-demo-topic.md" }));
    assert_eq!(std::fs::read_to_string(&old_file).unwrap(), older, "the older record stays archived");
    let live = p.dir.join("speclink").join("discussions").join("demo-topic.md");
    assert_eq!(std::fs::read_to_string(live).unwrap(), newer);
}

#[test]
fn fs_discuss_restore_refuses_a_live_record_and_an_unknown_slug() {
    // Scenarios「已有進行中的記錄時拒絕」與「沒有封存的記錄」。
    let p = TempProject::fs_project("restore-refused");
    let record = archived_record("2026-11-10", "封存的那一份");
    let archived = p.write("speclink/discussions/archive/2026-11-10-demo-topic.md", &record);
    let live = p.write("speclink/discussions/demo-topic.md", "---\ntopic: Demo topic\nslug: demo-topic\nstatus: open\ncreated: 2026-11-15\n---\n\n## Context\n\n進行中的那一份\n");
    let live_text = std::fs::read_to_string(&live).unwrap();

    let out = p.run(&["discuss", "restore", "demo-topic"], None);
    assert!(!out.status.success());
    assert_eq!(stdout_of(&out), "");
    assert!(stderr_of(&out).contains("already exists"), "{}", stderr_of(&out));
    assert_eq!(std::fs::read_to_string(&live).unwrap(), live_text, "the live record is unchanged");
    assert_eq!(std::fs::read_to_string(&archived).unwrap(), record, "the archived record is unchanged");

    let ghost = p.run(&["discuss", "restore", "ghost"], None);
    assert!(!ghost.status.success());
    assert_eq!(stdout_of(&ghost), "");
    let stderr = stderr_of(&ghost);
    assert!(stderr.contains("ghost") && stderr.contains("archived record"), "{stderr}");
    assert!(!p.dir.join("speclink").join("discussions").join("ghost.md").exists(), "zero writes");
}

#[test]
fn fs_an_archived_discussion_refusal_points_at_discuss_restore() {
    // Scenario「已封存的拒絕訊息指向 restore」。
    let p = TempProject::fs_project("restore-pointer");
    p.write("speclink/discussions/archive/2026-11-10-demo-topic.md", &archived_record("2026-11-10", "x"));

    let out = p.run(&["discuss", "promote", "demo-topic"], None);

    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("speclink discuss restore demo-topic"), "{stderr}");
    assert!(!stderr.contains("move it out of"), "{stderr}");
}

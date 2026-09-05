//! `speclink discuss search` (change discuss-search-recall): keywords hit the
//! topic, the slug and the four decision lines of live and archived records —
//! never Evidence or prose — in the spec's order, with the human and `--json`
//! shapes frozen here for both modes.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

// --- capturing mock server（沿 remote_verb_parity.rs 的模式）---

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
    routes.push(("GET", "/binding", 200, BINDING_BODY.to_string()));
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
    /// A throwaway project with empty live and archived discussion directories.
    fn empty(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-discuss-search-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("discussions").join("archive")).unwrap();
        TempProject { dir }
    }

    /// The three-record fixture every search case reads: one live topic hit,
    /// one archived conclusion hit, one record that only mentions its keyword
    /// in an Evidence line.
    fn seeded(tag: &str) -> TempProject {
        let p = TempProject::empty(tag);
        p.write(
            "openspec/discussions/golden-policy.md",
            "---\ntopic: Golden snapshot policy\nslug: golden-policy\nstatus: open\ncreated: 2026-07-01\n---\n\n\
             # Discussion: Golden snapshot policy\n\n## Context\n\nseed\n\n## Rounds\n\n\
             ### Round 1 — interview (2026-07-01)\n\n**Focus**: golden\n**Evidence**: golden.rs\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n",
        );
        p.write(
            "openspec/discussions/archive/2026-08-01-transport-choice.md",
            "---\ntopic: Transport choice\nslug: transport-choice\nstatus: concluded\ncreated: 2026-08-01\n---\n\n\
             # Discussion: Transport choice\n\n## Context\n\nseed\n\n## Rounds\n\n\
             ### Round 1 — assumptions (2026-08-01)\n\n**Focus**: transport\n**Ruled out**: polling\n\n\
             ## Conclusion\n\n**Decision**: keep long-poll\n**Deferred**: SSE reconnect\n",
        );
        // 只有 frontmatter 與 Context 的在途記錄（尚無輪與 Conclusion）。
        p.write(
            "openspec/discussions/release-memo.md",
            "---\ntopic: Release notes\nslug: release-memo\nstatus: open\ncreated: 2026-05-01\n---\n\n\
             # Discussion: Release notes\n\n## Context\n\nseed\n",
        );
        // 封存記錄，Ruled out 標記獨占一行、內容寫在下一行條列（封存記錄的慣用寫法）。
        p.write(
            "openspec/discussions/archive/2026-04-01-tray-fallback.md",
            "---\ntopic: Tray fallback page\nslug: tray-fallback\nstatus: concluded\ncreated: 2026-04-01\n---\n\n\
             # Discussion: Tray fallback page\n\n## Context\n\nseed\n\n## Rounds\n\n\
             ### Round 1 — interview (2026-04-01)\n\n**Focus**: fallback\n\
             **Ruled out**:\n- 只在 tray.ts 修落頁\n- 把 drawer 拿掉\n\n**Open**: drawer naming\n\n\
             ## Conclusion\n\n**Decision**: keep the page\n",
        );
        p.write(
            "openspec/discussions/desktop-packaging.md",
            "---\ntopic: Desktop packaging\nslug: desktop-packaging\nstatus: open\ncreated: 2026-06-01\n---\n\n\
             # Discussion: Desktop packaging\n\n## Context\n\nseed\n\n## Rounds\n\n\
             ### Round 1 — interview (2026-06-01)\n\n**Position**: sidecar first\n**Evidence**: sidecar.rs\n\n\
             ## Conclusion\n\nProse about the sidecar.\n",
        );
        p
    }

    /// A remote-bound project: `.speclink.yaml` points at the mock server, no
    /// local openspec tree at all.
    fn remote(tag: &str, url: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-discuss-search-{tag}-{}",
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

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
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
            // The remote arm authenticates with this; the fs arm ignores it.
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

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("json stdout")
}

/// The frozen human shape: heading, one hit line per record in the list idiom,
/// then one six-space-indented line per match (topic and slug both hit here).
const HUMAN_TWO_HITS: &str = concat!(
    "Discussions matching \"golden sse\":\n",
    "  • golden-policy [open, live] (2026-07-01) — Golden snapshot policy\n",
    "      frontmatter topic: Golden snapshot policy\n",
    "      frontmatter slug: golden-policy\n",
    "  • transport-choice [concluded, archived] (2026-08-01) — Transport choice\n",
    "      conclusion deferred: **Deferred**: SSE reconnect\n",
);

#[test]
fn human_output_lists_hits_with_indented_match_lines_and_no_color() {
    // spec scenario「人眼輸出格式與 --no-color」+「多關鍵字任一命中並依 topic 命中優先排序」。
    let p = TempProject::seeded("human");
    let out = p.run(&["discuss", "search", "golden", "sse", "--no-color"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), HUMAN_TWO_HITS);
    assert!(!stdout_of(&out).contains('\x1b'), "no ANSI under --no-color");
}

#[test]
fn json_output_wraps_hits_with_list_fields_and_matches() {
    let p = TempProject::seeded("json");
    let out = p.run(&["discuss", "search", "golden", "sse", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v = json_of(&out);
    let hits = v["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 2);
    let first = &hits[0];
    for key in ["slug", "topic", "status", "rounds", "created", "path", "archived", "matches"] {
        assert!(first.get(key).is_some(), "hit carries {key}: {first}");
    }
    assert_eq!(first["slug"], "golden-policy");
    assert_eq!(first["archived"], false);
    assert_eq!(first["rounds"], 1);
    assert_eq!(first["matches"][0]["kind"], "topic");
    assert_eq!(first["matches"][0]["where"], "frontmatter");
    assert_eq!(first["matches"][0]["text"], "Golden snapshot policy");
    assert_eq!(first["matches"][1]["kind"], "slug", "the slug hit follows the topic hit");
    let second = &hits[1];
    assert_eq!(second["slug"], "transport-choice");
    assert_eq!(second["archived"], true);
    assert_eq!(second["matches"][0]["kind"], "deferred");
    assert_eq!(second["matches"][0]["where"], "conclusion");
    assert_eq!(second["matches"][0]["text"], "**Deferred**: SSE reconnect");
}

#[test]
fn archived_ruled_out_line_reports_its_round() {
    // spec scenario「決定行命中回傳輪號與原文」。
    let p = TempProject::seeded("round");
    let out = p.run(&["discuss", "search", "polling", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v = json_of(&out);
    assert_eq!(v["hits"].as_array().unwrap().len(), 1);
    let m = &v["hits"][0]["matches"][0];
    assert_eq!(m["kind"], "ruled-out");
    assert_eq!(m["where"], "round-1");
    assert_eq!(m["text"], "**Ruled out**: polling");
}

#[test]
fn evidence_only_mentions_do_not_match_and_zero_hits_exit_zero() {
    // spec scenario「非決定行不命中且零命中回空」。
    let p = TempProject::seeded("zero");
    let out = p.run(&["discuss", "search", "sidecar"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "No discussions match \"sidecar\".\n");

    let out = p.run(&["discuss", "search", "sidecar", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "{\n  \"hits\": []\n}\n");
}

#[test]
fn missing_keyword_is_rejected_by_clap_with_usage_on_stderr() {
    // spec scenario「未帶關鍵字」。
    let p = TempProject::seeded("nokw");
    let out = p.run(&["discuss", "search"]);
    assert!(!out.status.success(), "no keyword must fail");
    assert!(stdout_of(&out).is_empty(), "stdout stays empty: {}", stdout_of(&out));
    assert!(
        stderr_of(&out).contains("Usage:"),
        "stderr explains usage: {}",
        stderr_of(&out)
    );
}

#[test]
fn remote_mode_output_matches_local_mode_for_the_same_hits() {
    // spec scenario「remote 模式輸出同形」：server 回本機引擎會回的同一組 hits，
    // remote 的 --json 與人眼輸出和本機逐位元相同；關鍵字以空白接起放進 q。
    let local = TempProject::seeded("parity-local");
    let local_json = local.run(&["discuss", "search", "golden", "sse", "--json"]);
    assert!(local_json.status.success(), "local stderr: {}", stderr_of(&local_json));
    let local_human = local.run(&["discuss", "search", "golden", "sse", "--no-color"]);
    assert!(local_human.status.success(), "local stderr: {}", stderr_of(&local_human));

    // The wire body is the local payload itself: the wire hit shape is the
    // list item plus matches, exactly what the engine serialized locally.
    let wire_body = stdout_of(&local_json);
    let mock = mock_server(vec![("GET", "/discussions/search", 200, wire_body)]);
    let remote = TempProject::remote("parity-remote", &mock.base);
    let remote_json = remote.run(&["discuss", "search", "golden", "sse", "--json"]);
    assert!(remote_json.status.success(), "remote stderr: {}", stderr_of(&remote_json));
    assert_eq!(json_of(&remote_json), json_of(&local_json), "hits order, slugs and matches agree");
    assert_eq!(stdout_of(&remote_json), stdout_of(&local_json), "--json is byte-identical");

    let remote_human = remote.run(&["discuss", "search", "golden", "sse", "--no-color"]);
    assert!(remote_human.status.success(), "remote stderr: {}", stderr_of(&remote_human));
    assert_eq!(stdout_of(&remote_human), stdout_of(&local_human), "human output is byte-identical");

    let cap = mock.find("GET", "/discussions/search");
    let q = cap.query.strip_prefix("q=").expect("single q parameter").replace('+', " ").replace("%20", " ");
    assert_eq!(q, "golden sse", "keywords travel space-joined in one q");
}

#[test]
fn record_without_rounds_or_conclusion_still_hits_by_topic() {
    // spec scenario「記錄缺區段時不使查詢失敗」——經 CLI 執行的對應案例。
    let p = TempProject::seeded("bare");
    let out = p.run(&["discuss", "search", "notes", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v = json_of(&out);
    let hits = v["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 1, "{v}");
    assert_eq!(hits[0]["slug"], "release-memo");
    assert_eq!(hits[0]["rounds"], 0);
    let matches = hits[0]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["kind"], "topic");
    assert_eq!(matches[0]["where"], "frontmatter");
}

#[test]
fn archived_record_with_a_bare_marker_matches_its_bullet_lines() {
    // spec scenario「標記獨占一行時其下條列行命中」——封存記錄經 CLI --json。
    let p = TempProject::seeded("bullets");
    let out = p.run(&["discuss", "search", "drawer", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v = json_of(&out);
    let hits = v["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 1, "{v}");
    assert_eq!(hits[0]["slug"], "tray-fallback");
    assert_eq!(hits[0]["archived"], true);
    let matches = hits[0]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "the Open line never matches: {matches:?}");
    assert_eq!(matches[0]["kind"], "ruled-out");
    assert_eq!(matches[0]["where"], "round-1");
    assert_eq!(matches[0]["text"], "- 把 drawer 拿掉");
}

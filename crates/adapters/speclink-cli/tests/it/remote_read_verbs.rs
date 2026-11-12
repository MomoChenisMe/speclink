//! The reads that need no checkout (change remote-read-verbs): `trace`,
//! `plan --strict-overlap`, `evidence show`, `artifact cat` of an archived
//! change, and the two time fields of `list --json`.
//!
//! Each test writes one set of documents twice: as files of a local project,
//! and into the store of a real server — the host kit's HTTP door on the
//! in-memory store port, behind a small HTTP listener. The same verb then
//! runs in both projects, and its stdout, stderr and exit code must be the
//! same. The remote project holds nothing but its connection file, so a read
//! of the local store has nothing to find.

use speclink_host::http::{HttpRequest, Method};
use speclink_host::kit::{Hooks, HostKit, HostKitOptions, Identity, ProjectEntry, RepoEntry};
use speclink_store_port::{Actor, Commit, DocKey, Expected, MemoryStorePort, Op, ScopeId, StorePort, Timestamp};
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

// --- the server: the host kit's door on a memory store ---

struct Door {
    server: Arc<tiny_http::Server>,
    base: String,
    /// `METHOD path?query` of every request, the project prefix removed.
    asked: Arc<Mutex<Vec<String>>>,
}

const PROJECT: &str = "/api/speclink/v1/projects/demo";

/// 2026-11-12T03:00:00Z, in milliseconds.
const NOV_12_0300_MS: u64 = 1_794_452_400_000;

/// A server whose store holds `docs` (store keys), committed at `at_millis`.
fn door(docs: &[(&str, &str)], at_millis: u64) -> Door {
    let scope = ScopeId::new("s1").unwrap();
    let store = Arc::new(MemoryStorePort::new());
    let ops = docs
        .iter()
        .map(|(key, content)| Op::Put {
            key: DocKey::new(*key).unwrap(),
            content: content.to_string(),
            expected: Expected::Absent,
        })
        .collect();
    let seed =
        Commit::new(Actor::new("seed", "seed").unwrap(), "seed", Timestamp::new(at_millis).unwrap(), ops, Vec::new())
            .unwrap();
    store.commit(&scope, &seed).expect("seed commit");

    let options = HostKitOptions {
        scopes: Some(Box::new(move |_identity: &Identity| {
            Ok(vec![ProjectEntry {
                key: "demo".to_string(),
                name: "Demo".to_string(),
                repos: vec![RepoEntry {
                    key: "backend".to_string(),
                    name: "Backend".to_string(),
                    scope: scope.clone(),
                    can_write: true,
                    can_write_policy: true,
                }],
            }])
        })),
        ..Default::default()
    };
    let kit = HostKit::with_options(store, Hooks::default(), options);
    let identity = Identity::new("u_1", "Tester").unwrap();

    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let base = format!("http://{}{PROJECT}", server.server_addr());
    let asked: Arc<Mutex<Vec<String>>> = Arc::default();
    let (looper, log) = (server.clone(), asked.clone());
    std::thread::spawn(move || {
        for mut req in looper.incoming_requests() {
            let mut body = Vec::new();
            let _ = req.as_reader().read_to_end(&mut body);
            let url = req.url().to_string();
            let (path, query) = match url.split_once('?') {
                Some((path, query)) => (path.to_string(), Some(query.to_string())),
                None => (url.clone(), None),
            };
            log.lock().unwrap().push(format!("{} {}", req.method(), url.trim_start_matches(PROJECT)));
            let request = HttpRequest {
                method: Method::from_name(&req.method().to_string()).expect("a method the door takes"),
                path,
                query,
                headers: req.headers().iter().map(|h| (h.field.to_string(), h.value.to_string())).collect(),
                body,
            };
            let response = match kit.handle(&request, Some(&identity)) {
                Some(response) => {
                    let mut out = tiny_http::Response::from_data(response.body).with_status_code(response.status);
                    for (name, value) in &response.headers {
                        out.add_header(tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap());
                    }
                    out
                }
                None => tiny_http::Response::from_data(Vec::new()).with_status_code(404),
            };
            let _ = req.respond(response);
        }
    });
    Door { server, base, asked }
}

impl Door {
    /// Every request but the handshake.
    fn verb_calls(&self) -> Vec<String> {
        self.asked.lock().unwrap().iter().filter(|call| *call != "GET /binding").cloned().collect()
    }
}

impl Drop for Door {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

// --- the two projects ---

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn bare(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!("speclink-cli-remote-read-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempProject { dir }
    }

    /// A local project that holds `docs` as files: a store key is its path
    /// under the spec directory, except the archive, which the files keep
    /// under each kind's own folder.
    fn local(tag: &str, docs: &[(&str, &str)]) -> TempProject {
        let p = TempProject::bare(&format!("{tag}-fs"));
        std::fs::write(p.dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::create_dir_all(p.dir.join("speclink").join("changes")).unwrap();
        for (key, content) in docs {
            let rel = match key.strip_prefix("archive/changes/") {
                Some(rest) => format!("changes/archive/{rest}"),
                None => match key.strip_prefix("archive/discussions/") {
                    Some(rest) => format!("discussions/archive/{rest}"),
                    None => key.to_string(),
                },
            };
            let path = p.dir.join("speclink").join(rel.split('/').collect::<PathBuf>());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        p
    }

    /// A project that holds only its connection to `url`.
    fn remote(tag: &str, url: &str) -> TempProject {
        let p = TempProject::bare(&format!("{tag}-remote"));
        std::fs::write(p.dir.join(".speclink.yaml"), format!("remote:\n  url: {url}\n  repo: backend\n")).unwrap();
        p
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

    /// The files of this project, as paths from its root.
    fn files(&self) -> Vec<String> {
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else {
                    out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.dir, &self.dir, &mut out);
        out.sort();
        out
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
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("stdout is JSON ({e}): {}", stdout_of(out)))
}

/// One set of documents as a local project, a server and a remote project.
struct Pair {
    fs: TempProject,
    remote: TempProject,
    door: Door,
}

fn pair(tag: &str, docs: &[(&str, &str)]) -> Pair {
    pair_at(tag, docs, NOV_12_0300_MS)
}

fn pair_at(tag: &str, docs: &[(&str, &str)], at_millis: u64) -> Pair {
    let door = door(docs, at_millis);
    Pair { fs: TempProject::local(tag, docs), remote: TempProject::remote(tag, &door.base), door }
}

impl Pair {
    /// Run `args` in both projects and hold the two runs to each other: the
    /// same stdout, the same stderr, the same exit code. Gives the local run.
    fn same(&self, args: &[&str]) -> Output {
        let (fs, remote) = (self.fs.run(args), self.remote.run(args));
        assert_eq!(stdout_of(&remote), stdout_of(&fs), "{args:?}: stdout");
        assert_eq!(stderr_of(&remote), stderr_of(&fs), "{args:?}: stderr");
        assert_eq!(remote.status.code(), fs.status.code(), "{args:?}: exit code");
        fs
    }

    /// [`Pair::same`], for a run that succeeds with output.
    fn same_ok(&self, args: &[&str]) -> Output {
        let out = self.same(args);
        assert!(out.status.success(), "{args:?}: {}", stderr_of(&out));
        assert!(!out.stdout.is_empty(), "{args:?} prints something");
        out
    }

    /// The remote project still holds its connection file and nothing else.
    fn assert_remote_read_no_local_store(&self) {
        assert_eq!(self.remote.files(), [".speclink.yaml"], "the remote project has no local store");
    }
}

// --- fixtures ---

const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";

const CANON_CHECKOUT: &str = "# checkout Specification\n\n## Purpose\n\nCheckout flow.\n\n## Requirements\n\n\
### Requirement: R1\n\nIt SHALL work.\n\n<!-- @trace\nsource: alpha\nupdated: 2026-07-10\n-->\n\n\
### Requirement: R2\n\nIt SHALL persist.\n\n<!-- @trace\nsource: beta\nupdated: 2026-08-02\n-->\n";

const CANON_BILLING: &str =
    "# billing Specification\n\n## Purpose\n\nBilling flow.\n\n## Requirements\n\n### Requirement: B1\n\nIt SHALL bill.\n";

const EVIDENCE_ALPHA: &str = r#"{"version":2,"change":"alpha","entries":[
  {"taskId":"tsk_01","taskDesc":"d","touchedFiles":["src/a.rs","src/b.rs"],"recordedAt":"2026-07-10T00:00:00Z"},
  {"taskId":"tsk_02","taskDesc":"d2","touchedFiles":["src/c.rs"],"recordedAt":"2026-07-10T00:00:00Z"}]}"#;

const DISCUSSION: &str = "---\ntopic: how checkout came to be\nslug: origin-talk\nstatus: promoted\npromoted_to: alpha, cousin\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: build it\n";

/// Capability checkout: two archived changes touched it — alpha (from a
/// discussion, with evidence) then beta; cousin came from the same
/// discussion and touched billing.
fn traced_checkout() -> Vec<(&'static str, &'static str)> {
    vec![
        ("specs/checkout/spec.md", CANON_CHECKOUT),
        ("specs/billing/spec.md", CANON_BILLING),
        (
            "archive/changes/2026-07-10-alpha/.meta.yaml",
            "schema: spec-driven\ncreated: 2026-07-09\nfrom_discussion: origin-talk\narchived_at: 2026-07-10\n",
        ),
        ("archive/changes/2026-07-10-alpha/specs/checkout/spec.md", "delta"),
        ("archive/changes/2026-07-10-alpha/.evidence.json", EVIDENCE_ALPHA),
        ("archive/changes/2026-08-02-beta/.meta.yaml", META),
        ("archive/changes/2026-08-02-beta/specs/checkout/spec.md", "delta"),
        ("archive/changes/2026-07-12-cousin/.meta.yaml", META),
        ("archive/changes/2026-07-12-cousin/specs/billing/spec.md", "delta"),
        ("discussions/origin-talk.md", DISCUSSION),
    ]
}

// --- trace 於 remote 走 server ---

#[test]
fn trace_prints_the_local_output_in_remote_mode() {
    let p = pair("trace", &traced_checkout());
    for args in [&["trace", "checkout"][..], &["trace", "checkout", "--no-color"], &["trace", "checkout", "--json"]] {
        p.same_ok(args);
    }
    // The chain is the one the fixture holds, oldest first.
    let json = json_of(&p.fs.run(&["trace", "checkout", "--json"]));
    assert_eq!(json["changes"][0]["archivedDir"], "2026-07-10-alpha");
    assert_eq!(json["changes"][1]["archivedDir"], "2026-08-02-beta");
    assert_eq!(json["changes"][0]["fromDiscussion"], "origin-talk");
    assert_eq!(json["discussions"][0]["promotedTo"][1]["capabilities"], serde_json::json!(["billing"]));
    let human = stdout_of(&p.fs.run(&["trace", "checkout", "--no-color"]));
    assert!(human.starts_with("Capability: checkout\n"), "{human}");
    assert!(!human.contains('\u{1b}'), "{human:?}");

    assert_eq!(p.door.verb_calls(), ["GET /trace/checkout"; 3]);
    p.assert_remote_read_no_local_store();
}

#[test]
fn trace_of_an_unknown_capability_fails_alike_in_both_modes() {
    let p = pair("trace-missing", &traced_checkout());
    // One with near names, one with none.
    for capability in ["checkout-flow", "nothing-like-it"] {
        let out = p.same(&["trace", capability]);
        assert!(!out.status.success(), "{capability}");
        assert!(out.stdout.is_empty(), "{capability}");
        assert!(
            stderr_of(&out).contains(&format!("Capability '{capability}' is not in the canonical specs.")),
            "{}",
            stderr_of(&out)
        );
    }
    assert!(stderr_of(&p.fs.run(&["trace", "checkout-flow"])).contains("Re-run with one of these exact names."));
    assert!(stderr_of(&p.fs.run(&["trace", "nothing-like-it"])).contains("No similar capability names found."));
}

// --- plan 的 --strict-overlap 於 remote 同形 ---

const MODIFIED_REWIND: &str = "## MODIFIED Requirements\n\n### Requirement: Run rewind point\n\nbody\n";
const MODIFIED_PIN: &str = "## MODIFIED Requirements\n\n### Requirement: Conversation pin\n\nbody\n";

/// Changes a and b: each has a delta for desktop-app, on different requirements.
fn shared_capability() -> Vec<(&'static str, &'static str)> {
    vec![
        ("changes/a/.meta.yaml", "schema: spec-driven\ncreated: 2026-09-01\n"),
        ("changes/a/specs/desktop-app/spec.md", MODIFIED_REWIND),
        ("changes/b/.meta.yaml", "schema: spec-driven\ncreated: 2026-09-02\n"),
        ("changes/b/specs/desktop-app/spec.md", MODIFIED_PIN),
    ]
}

#[test]
fn plan_strict_overlap_prints_the_local_output_in_remote_mode() {
    let p = pair("strict", &shared_capability());
    for args in [
        &["plan", "--strict-overlap"][..],
        &["plan", "--strict-overlap", "--no-color"],
        &["plan", "--strict-overlap", "--json"],
    ] {
        p.same_ok(args);
    }
    let strict = json_of(&p.remote.run(&["plan", "--strict-overlap", "--json"]));
    assert_eq!(
        strict["waves"],
        serde_json::json!([{ "index": 1, "changes": ["a"] }, { "index": 2, "changes": ["b"] }])
    );
    assert_eq!(strict["changes"][1]["blockedBy"], serde_json::json!(["a"]));
    // Without the flag the two changes share a wave, in both modes.
    let plain = json_of(&p.same_ok(&["plan", "--json"]));
    assert_eq!(plain["waves"], serde_json::json!([{ "index": 1, "changes": ["a", "b"] }]));

    let calls = p.door.verb_calls();
    assert_eq!(calls.iter().filter(|c| *c == "GET /plan?strictOverlap=true").count(), 4, "{calls:?}");
    assert_eq!(calls.iter().filter(|c| *c == "GET /plan").count(), 1, "{calls:?}");
    assert_eq!(calls.len(), 5, "{calls:?}");
    p.assert_remote_read_no_local_store();
}

#[test]
fn a_dependency_cycle_fails_the_strict_plan_alike_in_both_modes() {
    let docs = [
        ("changes/a/.meta.yaml", "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: b\n"),
        ("changes/b/.meta.yaml", "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n"),
    ];
    let p = pair("strict-cycle", &docs);
    let out = p.same(&["plan", "--strict-overlap"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("dependency cycle: "), "{}", stderr_of(&out));
}

// --- evidence show：兩模式同形 ---

/// The spec's two records: task 1.1 on a.rs and b.rs, task 1.2 on b.rs and c.rs.
const EVIDENCE_DEMO: &str = r#"{"version":2,"change":"demo","entries":[
  {"taskId":"tsk_11","taskDesc":"1.1 First","touchedFiles":["src/b.rs","src/a.rs"],"recordedAt":"2026-11-12T03:00:00Z"},
  {"taskId":"tsk_12","taskDesc":"1.2 Second","actor":"Alice <a@example.com>","repo":"backend","headCommit":"0123abc","touchedFiles":["src/b.rs","src/c.rs"],"recordedAt":"2026-11-12T04:00:00Z"}]}"#;

const EVIDENCE_OLD: &str = r#"{"version":2,"change":"old","entries":[
  {"taskId":"tsk_o1","taskDesc":"1.1 Old","touchedFiles":["src/old.rs"],"recordedAt":"2026-11-10T00:00:00Z"}]}"#;

const OLD_PROPOSAL: &str = "## Why\n\nThe archived reason.\n";
const OLD_AUTH_DELTA: &str = "## ADDED Requirements\n\n### Requirement: Session\n\nIt SHALL work.\n";

/// demo (two records), fresh (none), and old, archived as 2026-11-12-old
/// with one record, a proposal and a delta spec for auth.
fn evidence_docs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("changes/demo/.meta.yaml", META),
        ("changes/demo/proposal.md", "## Why\n\nThe active reason.\n"),
        ("changes/demo/.evidence.json", EVIDENCE_DEMO),
        ("changes/fresh/.meta.yaml", META),
        ("archive/changes/2026-11-12-old/.meta.yaml", META),
        ("archive/changes/2026-11-12-old/proposal.md", OLD_PROPOSAL),
        ("archive/changes/2026-11-12-old/specs/auth/spec.md", OLD_AUTH_DELTA),
        ("archive/changes/2026-11-12-old/.evidence.json", EVIDENCE_OLD),
    ]
}

#[test]
fn evidence_show_prints_the_local_output_in_remote_mode() {
    let p = pair("evidence", &evidence_docs());
    for args in [
        &["evidence", "show", "demo"][..],
        &["evidence", "show", "demo", "--no-color"],
        &["evidence", "show", "demo", "--json"],
        &["evidence", "show", "fresh"],
        &["evidence", "show", "fresh", "--json"],
        &["evidence", "show", "old"],
        &["evidence", "show", "old", "--json"],
        &["evidence", "show", "2026-11-12-old", "--no-color"],
        &["evidence", "show", "2026-11-12-old", "--json"],
    ] {
        p.same_ok(args);
    }
    let calls = p.door.verb_calls();
    assert_eq!(calls.len(), 9, "{calls:?}");
    assert_eq!(calls[0], "GET /changes/demo/evidence");
    assert_eq!(calls[7], "GET /changes/2026-11-12-old/evidence");
    p.assert_remote_read_no_local_store();
}

#[test]
fn evidence_show_has_the_fields_the_spec_names() {
    let p = pair("evidence-shape", &evidence_docs());
    // 「讀進行中變更的記錄」
    let demo = json_of(&p.remote.run(&["evidence", "show", "demo", "--json"]));
    let keys = |v: &serde_json::Value| -> Vec<String> { v.as_object().unwrap().keys().cloned().collect() };
    assert_eq!(keys(&demo), ["change", "entries", "files"], "no archivedAs for an active change");
    assert_eq!(demo["change"], "demo");
    assert_eq!(demo["files"], serde_json::json!(["src/a.rs", "src/b.rs", "src/c.rs"]));
    assert_eq!(keys(&demo["entries"][0]), ["recordedAt", "taskDesc", "taskId", "touchedFiles"]);
    assert_eq!(
        keys(&demo["entries"][1]),
        ["actor", "headCommit", "recordedAt", "repo", "taskDesc", "taskId", "touchedFiles"]
    );
    assert_eq!(demo["entries"][0]["touchedFiles"], serde_json::json!(["src/b.rs", "src/a.rs"]));
    // 「人眼輸出」
    let human = stdout_of(&p.remote.run(&["evidence", "show", "demo", "--no-color"]));
    assert_eq!(
        human,
        "Evidence: demo  (2 task(s))\n\n  tsk_11  1.1 First\n    recorded 2026-11-12T03:00:00Z\n    src/b.rs\n    src/a.rs\n\n  tsk_12  1.2 Second\n    recorded 2026-11-12T04:00:00Z\n    src/b.rs\n    src/c.rs\n"
    );
    // 「沒有記錄」
    assert_eq!(stdout_of(&p.remote.run(&["evidence", "show", "fresh"])), "No evidence recorded for 'fresh'.\n");
    let fresh = json_of(&p.remote.run(&["evidence", "show", "fresh", "--json"]));
    assert_eq!(fresh, serde_json::json!({ "change": "fresh", "entries": [], "files": [] }));
    // 「以原本的名稱讀已封存變更的記錄」「以封存名稱讀」
    let by_name = json_of(&p.remote.run(&["evidence", "show", "old", "--json"]));
    assert_eq!(by_name["archivedAs"], "2026-11-12-old");
    assert_eq!(by_name["change"], "old");
    assert_eq!(by_name["entries"][0]["taskId"], "tsk_o1");
    assert_eq!(json_of(&p.remote.run(&["evidence", "show", "2026-11-12-old", "--json"])), by_name);
    let archived = stdout_of(&p.remote.run(&["evidence", "show", "old", "--no-color"]));
    assert!(archived.starts_with("Evidence: old  (1 task(s))\n  [archived as 2026-11-12-old]\n\n"), "{archived}");
}

#[test]
fn evidence_show_of_an_unknown_name_fails_alike_in_both_modes() {
    let p = pair("evidence-ghost", &evidence_docs());
    for args in [&["evidence", "show", "ghost"][..], &["evidence", "show", "ghost", "--json"]] {
        let out = p.same(args);
        assert!(!out.status.success(), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(stderr_of(&out).contains("Change 'ghost' not found."), "{}", stderr_of(&out));
    }
}

#[test]
fn an_active_change_wins_over_an_archived_one_of_its_name() {
    // 「同名的進行中變更優先」：封存過一次，之後又建立同名的變更（沒有記錄）。
    let mut docs = evidence_docs();
    docs.push(("changes/old/.meta.yaml", META));
    docs.push(("changes/old/proposal.md", "## Why\n\nThe new reason.\n"));
    let p = pair("evidence-active-wins", &docs);
    let shown = json_of(&p.same_ok(&["evidence", "show", "old", "--json"]));
    assert_eq!(shown, serde_json::json!({ "change": "old", "entries": [], "files": [] }));
    assert_eq!(stdout_of(&p.same_ok(&["artifact", "cat", "proposal", "--change", "old"])), "## Why\n\nThe new reason.\n");
    // The archived name still reaches the archived one.
    let archived = json_of(&p.same_ok(&["evidence", "show", "2026-11-12-old", "--json"]));
    assert_eq!(archived["archivedAs"], "2026-11-12-old");
}

#[test]
fn a_record_that_does_not_parse_is_an_error_not_no_record() {
    // 「壞的記錄不當成沒有記錄」
    let docs = [("changes/demo/.meta.yaml", META), ("changes/demo/.evidence.json", "SECRET-CONTENT not json")];
    let p = pair("evidence-broken", &docs);
    let fs = p.fs.run(&["evidence", "show", "demo"]);
    assert!(!fs.status.success());
    assert!(fs.stdout.is_empty(), "no 'no evidence' line: {}", stdout_of(&fs));
    let err = stderr_of(&fs);
    assert!(err.contains("evidence record of 'demo' cannot be parsed"), "{err}");
    assert!(!err.contains("SECRET-CONTENT"), "{err}");
    // The server answers a 500; the CLI fails, and prints no record either.
    let remote = p.remote.run(&["evidence", "show", "demo"]);
    assert!(!remote.status.success());
    assert!(remote.stdout.is_empty(), "{}", stdout_of(&remote));
    assert!(!stderr_of(&remote).contains("SECRET-CONTENT"), "{}", stderr_of(&remote));
}

#[test]
fn a_crafted_name_reads_no_other_record_in_either_mode() {
    // Sharp-edges audit: a name with a path separator, or a dot name, is no
    // change name. The local store finds a change by probing a directory, so
    // without the engine's guard `..` would read the record beside the
    // changes folder and `archive/<name>` an archived change as an active one.
    let p = pair("crafted", &evidence_docs());
    let stray = r#"{"version":2,"change":"x","entries":[{"taskId":"tsk_stray","taskDesc":"SECRET","touchedFiles":["SECRET"],"recordedAt":"2026-01-01T00:00:00Z"}]}"#;
    std::fs::write(p.fs.dir.join("speclink").join(".evidence.json"), stray).unwrap();
    std::fs::write(p.fs.dir.join("speclink").join("changes").join("archive").join(".evidence.json"), stray).unwrap();

    for name in ["archive", "archive/2026-11-12-old", "demo/../demo", "../changes/demo", "a/b"] {
        let out = p.same(&["evidence", "show", name]);
        assert!(!out.status.success(), "{name}");
        assert!(out.stdout.is_empty(), "{name}: {}", stdout_of(&out));
        assert!(stderr_of(&out).contains(&format!("Change '{name}' not found.")), "{name}: {}", stderr_of(&out));
        let cat = p.fs.run(&["artifact", "cat", "proposal", "--change", name]);
        assert!(!cat.status.success() && cat.stdout.is_empty(), "{name}: {}", stdout_of(&cat));
        assert!(stderr_of(&cat).contains(&format!("Change '{name}' not found.")), "{name}: {}", stderr_of(&cat));
    }
    // The two dot names cannot travel as a path segment, so only the local
    // message is held; the remote run must still fail and print no record.
    for name in ["..", "."] {
        let fs = p.fs.run(&["evidence", "show", name]);
        assert!(!fs.status.success() && fs.stdout.is_empty(), "{name}: {}", stdout_of(&fs));
        assert!(stderr_of(&fs).contains(&format!("Change '{name}' not found.")), "{name}: {}", stderr_of(&fs));
        let remote = p.remote.run(&["evidence", "show", name]);
        assert!(!remote.status.success() && remote.stdout.is_empty(), "{name}: {}", stdout_of(&remote));
    }
    // A capability name with a separator fails alike in both modes.
    let out = p.same(&["trace", "checkout/../../changes/demo"]);
    assert!(!out.status.success() && out.stdout.is_empty());
    assert!(stderr_of(&out).contains("is not in the canonical specs."), "{}", stderr_of(&out));
}

// --- artifact cat 讀取已封存變更的文件 ---

#[test]
fn artifact_cat_reads_an_archived_change_in_both_modes() {
    let p = pair("cat-archived", &evidence_docs());
    // 「以原本的名稱讀已封存變更的提案」「以封存名稱讀 delta spec」
    for name in ["old", "2026-11-12-old"] {
        let proposal = p.same_ok(&["artifact", "cat", "proposal", "--change", name]);
        assert_eq!(stdout_of(&proposal), OLD_PROPOSAL, "{name}");
        let delta = p.same_ok(&["artifact", "cat", "specs/auth", "--change", name]);
        assert_eq!(stdout_of(&delta), OLD_AUTH_DELTA, "{name}");
    }
    // An active change reads as it always did.
    let active = p.same_ok(&["artifact", "cat", "proposal", "--change", "demo"]);
    assert_eq!(stdout_of(&active), "## Why\n\nThe active reason.\n");
    // 「兩邊都沒有」, and an archived change without the document.
    for (args, message) in [
        (&["artifact", "cat", "proposal", "--change", "ghost"][..], "Change 'ghost' not found."),
        (&["artifact", "cat", "design", "--change", "old"], "artifact 'design' not found for change 'old'"),
    ] {
        let out = p.same(args);
        assert!(!out.status.success(), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(stderr_of(&out).contains(message), "{args:?}: {}", stderr_of(&out));
    }
    p.assert_remote_read_no_local_store();
}

// --- list --json 的時間欄位 ---

const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 First task\n";

fn listed() -> Vec<(&'static str, &'static str)> {
    vec![
        ("changes/demo/.meta.yaml", META),
        ("changes/demo/proposal.md", "## Why\n\nDemo.\n"),
        ("changes/demo/tasks.md", TASKS),
    ]
}

/// `YYYY-MM-DDTHH:MM:SSZ`.
fn is_rfc3339_utc(text: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(text).is_ok() && text.len() == 20 && text.ends_with('Z')
}

#[test]
fn list_json_carries_created_and_modified_at_in_both_modes() {
    let p = pair("list-times", &listed());
    let fs = json_of(&p.fs.run(&["list", "--json"]));
    let remote = json_of(&p.remote.run(&["list", "--json"]));
    for (mode, list) in [("fs", &fs), ("remote", &remote)] {
        let demo = &list["changes"][0];
        assert_eq!(demo["created"], "2026-11-01", "{mode}");
        let at = demo["modifiedAt"].as_str().unwrap_or_else(|| panic!("{mode}: a modifiedAt string: {demo}"));
        assert!(is_rfc3339_utc(at), "{mode}: {at}");
    }
    // The server's store gave the time of its commit.
    assert_eq!(remote["changes"][0]["modifiedAt"], "2026-11-12T03:00:00Z");
    // The same field names, in the same order, in the two modes.
    let keys = |v: &serde_json::Value| -> Vec<String> { v.as_object().unwrap().keys().cloned().collect() };
    assert_eq!(keys(&fs["changes"][0]), keys(&remote["changes"][0]));
    let names = |text: String| -> Vec<String> {
        text.lines().filter_map(|l| l.trim().strip_prefix('"')?.split('"').next().map(str::to_string)).collect()
    };
    assert_eq!(
        names(stdout_of(&p.fs.run(&["list", "--json"]))),
        names(stdout_of(&p.remote.run(&["list", "--json"])))
    );
    // The human form has neither field, in either mode.
    let human = p.same_ok(&["list", "--no-color"]);
    assert_eq!(stdout_of(&human), "Changes:\n  • demo [0/1] — Demo.\n");
    p.same_ok(&["list"]);
}

#[test]
fn a_server_that_reports_no_time_gives_no_modified_at() {
    // 「儲存答不出最後修改時間」：每份文件回報的最後修改時間是 0。
    let p = pair_at("list-undated", &listed(), 0);
    let out = p.remote.run(&["list", "--json"]);
    assert!(out.status.success(), "{}", stderr_of(&out));
    let demo = &json_of(&out)["changes"][0];
    assert_eq!(demo["created"], "2026-11-01");
    assert!(demo.get("modifiedAt").is_none(), "{demo}");
}

#[test]
fn modified_at_moves_forward_when_a_task_is_done() {
    // 「勾選任務後 modifiedAt 不倒退」。每個檔案的修改時間先撥回 2020 年，所以勾選
    // 之後的時間一定比較晚：時間沒有動的話這個測試會失敗。
    let p = TempProject::local("list-task-done", &listed());
    let change = p.dir.join("speclink").join("changes").join("demo");
    let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_577_836_800); // 2020-01-01T00:00:00Z
    for name in [".meta.yaml", "proposal.md", "tasks.md"] {
        let file = std::fs::File::options().write(true).open(change.join(name)).unwrap();
        file.set_modified(old).unwrap();
    }
    let at = |p: &TempProject| {
        let list = json_of(&p.run(&["list", "--json"]));
        let text = list["changes"][0]["modifiedAt"].as_str().expect("a modifiedAt").to_string();
        chrono::DateTime::parse_from_rfc3339(&text).expect("an RFC 3339 time")
    };
    let before = at(&p);
    assert_eq!(before.to_rfc3339_opts(chrono::SecondsFormat::Secs, true), "2020-01-01T00:00:00Z");
    let done = p.run(&["task", "done", "1", "--change", "demo"]);
    assert!(done.status.success(), "{}", stderr_of(&done));
    assert!(at(&p) > before, "the write of tasks.md is newer than 2020");
}

#[test]
fn list_json_keeps_every_field_it_had() {
    // 「list 新增欄位不影響既有欄位」：拿掉兩個新欄位，剩下的就是之前的輸出。
    let p = TempProject::local("list-frozen", &listed());
    let mut list = json_of(&p.run(&["list", "--json"]));
    let demo = list["changes"][0].as_object_mut().unwrap();
    assert!(demo.remove("created").is_some() && demo.remove("modifiedAt").is_some());
    assert_eq!(
        list,
        serde_json::json!({ "changes": [{
            "completedTasks": 0, "name": "demo", "status": "in-progress", "summary": "Demo.", "totalTasks": 1,
        }] })
    );
}

// --- 連到沒有新路由的舊版 Server ---

#[test]
fn a_server_from_before_the_engine_lookup_is_refused_by_evidence_show() {
    // 舊版 server 的 evidence 路由只讀進行中的變更，回應沒有 `change`；它對已封存或
    // 不存在的名稱都回空集合。CLI 不把那個答案印成「沒有記錄」。
    let fake = crate::fake_server::Fake::new();
    fake.route("/api/speclink/v1/projects/demo/binding", 200, BINDING)
        .route("/api/speclink/v1/projects/demo/changes/old/evidence", 200, r#"{"entries":[]}"#);
    let p = TempProject::remote("old-server-evidence", &fake.project_url());
    for args in [&["evidence", "show", "old"][..], &["evidence", "show", "old", "--json"]] {
        let out = p.run(args);
        assert!(!out.status.success(), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}: {}", stdout_of(&out));
        assert!(
            stderr_of(&out).contains("this server does not support evidence show — update the server"),
            "{args:?}: {}",
            stderr_of(&out)
        );
    }
}

const BINDING: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;

#[test]
fn a_server_without_the_trace_route_is_a_not_found_failure() {
    let fake = crate::fake_server::Fake::new();
    fake.route("/api/speclink/v1/projects/demo/binding", 200, BINDING);
    let p = TempProject::remote("old-server", &fake.project_url());
    let out = p.run(&["trace", "checkout"]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "{}", stdout_of(&out));
    assert!(!stderr_of(&out).trim().is_empty());
    assert!(fake.paths().contains(&"/api/speclink/v1/projects/demo/trace/checkout".to_string()), "{:?}", fake.paths());
}

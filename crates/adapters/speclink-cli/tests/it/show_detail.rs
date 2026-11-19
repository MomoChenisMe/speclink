//! `show --json --detail` (verb-contract「單筆變更的詳細輸出」): a stamped
//! station also carries the scope its stamp recorded. Each set of documents
//! runs in a local project and against a real server (`pair`), and the two
//! runs must print the same bytes; the fake server answers only the case of a
//! server that reports no station.

use crate::fake_server::Fake;
use crate::remote_read_verbs::{json_of, pair, stderr_of, stdout_of, Pair, TempProject, BINDING};

const HASH_A: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const HASH_B: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90";

/// demo: the review is stamped over `src/b.rs` then `src/a.rs` (the record is
/// not in path order, so an output in path order would show); the verify has
/// an open ticket. checked: only the verify is stamped. auth is a spec.
fn stamped_docs() -> Vec<(&'static str, String)> {
    let demo = format!(
        "schema: spec-driven\ncreated: 2026-11-01\nreviewed_at: 2026-11-02T10:00:00+08:00\n\
reviewed_by: Rev <r@example.com>\nreviewed_tasks_total: 1\nreviewed_rounds: 2\n\
reviewed_scope:\n  - path: src/b.rs\n    hash: {HASH_B}\n  - path: src/a.rs\n    hash: {HASH_A}\n"
    );
    let checked = format!(
        "schema: spec-driven\ncreated: 2026-11-01\nverified_at: 2026-11-03T10:00:00+08:00\n\
verified_tasks_total: 1\nverified_rounds: 1\nverified_scope:\n  - path: src/a.rs\n    hash: {HASH_A}\n"
    );
    vec![
        ("changes/demo/.meta.yaml", demo),
        ("changes/demo/proposal.md", "## Why\n\nA reason.\n".to_string()),
        ("changes/demo/tasks.md", "- [x] 1.1 done\n".to_string()),
        ("changes/demo/verify.md", "# Verify — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n".to_string()),
        ("changes/checked/.meta.yaml", checked),
        ("changes/checked/proposal.md", "## Why\n\nAnother reason.\n".to_string()),
        ("changes/checked/tasks.md", "- [x] 1.1 done\n".to_string()),
        ("specs/auth/spec.md", "# auth\n".to_string()),
    ]
}

fn stamped_pair(tag: &str) -> Pair {
    let docs = stamped_docs();
    let refs: Vec<(&str, &str)> = docs.iter().map(|(k, v)| (*k, v.as_str())).collect();
    pair(tag, &refs)
}

#[test]
fn show_detail_prints_the_scope_of_each_stamp_alike_in_both_modes() {
    let p = stamped_pair("show-detail");
    let before = p.fs.contents();

    let demo = json_of(&p.same_ok(&["show", "demo", "--json", "--detail"]));
    assert_eq!(demo["review"]["state"], "stamped");
    assert_eq!(
        demo["review"]["scope"],
        serde_json::json!([{ "path": "src/b.rs", "hash": HASH_B }, { "path": "src/a.rs", "hash": HASH_A }]),
        "the order of the record, not path order"
    );
    for entry in demo["review"]["scope"].as_array().expect("scope array") {
        let keys: Vec<&str> = entry.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(keys, ["hash", "path"], "{entry}");
        assert!(entry["path"].is_string() && entry["hash"].is_string(), "{entry}");
    }
    assert_eq!(demo["verify"], serde_json::json!({ "state": "inProgress" }));

    // verify-station「帶 --detail 時帶出蓋章時的指紋」.
    let checked = json_of(&p.same_ok(&["show", "checked", "--json", "--detail"]));
    assert_eq!(checked["verify"]["state"], "stamped");
    assert_eq!(checked["verify"]["scope"], serde_json::json!([{ "path": "src/a.rs", "hash": HASH_A }]));
    assert!(checked.get("review").is_none(), "{checked}");

    assert_eq!(p.fs.contents(), before, "the flag writes no file");
}

#[test]
fn remote_show_detail_sends_the_requests_show_sends() {
    let p = stamped_pair("show-detail-requests");
    assert!(p.remote.run(&["show", "demo", "--json"]).status.success());
    let plain = p.door.verb_calls();
    assert!(p.remote.run(&["show", "demo", "--json", "--detail"]).status.success());
    assert_eq!(p.door.verb_calls()[plain.len()..], plain[..], "the flag adds no request");
}

#[test]
fn show_and_list_without_the_flag_print_no_scope() {
    let p = stamped_pair("show-detail-plain");
    let shown = json_of(&p.same_ok(&["show", "demo", "--json"]));
    assert_eq!(shown["review"]["state"], "stamped");
    assert!(shown["review"].get("scope").is_none(), "{shown}");
    assert!(shown["verify"].get("scope").is_none(), "{shown}");

    let listed = p.fs.run(&["list", "--json", "--detail"]);
    assert!(listed.status.success(), "stderr: {}", stderr_of(&listed));
    for item in json_of(&listed)["changes"].as_array().expect("changes") {
        for station in ["review", "verify"] {
            assert!(item.get(station).is_none_or(|s| s.get("scope").is_none()), "{item}");
        }
    }
}

#[test]
fn show_detail_without_json_is_a_usage_error_in_both_modes() {
    let p = stamped_pair("show-detail-usage");
    let out = p.same(&["show", "demo", "--detail"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("--json"), "{}", stderr_of(&out));
    assert!(out.stdout.is_empty(), "{}", stdout_of(&out));
}

#[test]
fn show_detail_of_a_spec_prints_what_show_prints() {
    let p = stamped_pair("show-detail-spec");
    let detailed = p.same_ok(&["show", "auth", "--item-type", "spec", "--json", "--detail"]);
    let plain = p.same_ok(&["show", "auth", "--item-type", "spec", "--json"]);
    assert_eq!(stdout_of(&detailed), stdout_of(&plain));
}

// --- a server that reports no station, and no server ---

#[test]
fn remote_show_detail_against_a_server_without_stations_succeeds_with_no_station() {
    let fake = Fake::new();
    let at = |suffix: &str| format!("/api/speclink/v1/projects/demo{suffix}");
    fake.route(&at("/binding"), 200, BINDING)
        .route(&at("/specs"), 200, r#"{"specs":[]}"#)
        .route(
            &at("/changes"),
            200,
            r#"{"changes":[{"name":"demo","status":"in-progress","completedTasks":1,"totalTasks":1}]}"#,
        )
        .route(
            &at("/changes/demo"),
            200,
            r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":false,"applyRequires":["tasks"],"artifacts":[],"created":"2026-11-01"}"#,
        );
    let p = TempProject::remote("show-detail-old-server", &fake.project_url());
    let out = p.run(&["show", "demo", "--json", "--detail"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let shown = json_of(&out);
    assert!(shown.get("review").is_none() && shown.get("verify").is_none(), "{shown}");
}

#[test]
fn remote_show_detail_offline_fails_as_show_does() {
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("probe port");
        listener.local_addr().expect("addr").port()
    }; // the listener is dropped: the port now refuses connections
    let url = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let p = TempProject::remote("show-detail-offline", &url);
    let detailed = p.run(&["show", "demo", "--json", "--detail"]);
    let plain = p.run(&["show", "demo", "--json"]);
    assert!(!detailed.status.success());
    assert_eq!(detailed.status.code(), plain.status.code());
    assert_eq!(stderr_of(&detailed), stderr_of(&plain));
    assert!(detailed.stdout.is_empty(), "{}", stdout_of(&detailed));
}

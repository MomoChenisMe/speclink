//! Remote workflow policy reads and CAS writes. The server owns both
//! authorization and validation: a reader cannot bypass the desktop, invalid
//! YAML never lands, and a stale revision leaves the winner untouched.

mod common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

const INITIAL: &str = "schema: spec-driven\nlocale: en\n";
const WINNER: &str = "schema: spec-driven\nlocale: tw\ntdd: true\n";

struct Fixture {
    base: String,
    store: Arc<MemoryStore>,
    editor_pat: String,
    reader_pat: String,
}

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn fixture() -> Fixture {
    let store = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext {
                command: "seed-policy".into(),
                actor: "seed".into(),
            },
        )
        .expect("begin seed");
    uow.create(DocumentId::WorkflowConfig, INITIAL);
    store.commit(uow, Vec::new()).expect("seed policy");

    let state = common::state_with(store.clone());
    let (editor_pat, _editor_id) = common::seed_named_pat(
        &state.identity,
        "editor@example.com",
        "Editor",
        &["demo"],
    );
    let (reader_pat, reader_id) = common::seed_named_pat(
        &state.identity,
        "reader@example.com",
        "Reader",
        &["demo"],
    );
    state
        .identity
        .admin_set_membership(
            &AuditActor::system_cli(),
            &reader_id,
            "demo",
            MembershipRole::Reader,
            true,
        )
        .expect("set reader role");

    Fixture {
        base: common::start(AppState { ..state }),
        store,
        editor_pat,
        reader_pat,
    }
}

fn project_url(f: &Fixture, tail: &str) -> String {
    format!(
        "{}/api/speclink/v1/projects/demo/{tail}",
        f.base
    )
}

fn request(method: &str, f: &Fixture, pat: &str, tail: &str) -> ureq::Request {
    ureq::request(method, &project_url(f, tail))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
}

fn get_json(f: &Fixture, pat: &str, tail: &str) -> (Value, Option<String>) {
    let response = request("GET", f, pat, tail).call().expect("GET succeeds");
    let etag = response.header("ETag").map(str::to_string);
    let body = response.into_json::<Value>().expect("JSON body");
    (body, etag)
}

fn put_config(
    f: &Fixture,
    pat: &str,
    content: &str,
    expected_revision: u64,
) -> Result<ureq::Response, ureq::Error> {
    request("PUT", f, pat, "config").send_json(json!({
        "content": content,
        "expectedRevision": expected_revision,
    }))
}

fn protocol_error(result: Result<ureq::Response, ureq::Error>) -> (u16, ErrorResponse) {
    match result {
        Ok(response) => panic!("expected protocol error, got {}", response.status()),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            let error = serde_json::from_str(&body)
                .unwrap_or_else(|_| panic!("expected ErrorResponse, got {body:?}"));
            (status, error)
        }
        Err(error) => panic!("transport error: {error}"),
    }
}

fn stored_policy(f: &Fixture) -> (String, u64) {
    let snapshot = f.store.snapshot(&scope()).expect("snapshot");
    let document = snapshot
        .read(&DocumentId::WorkflowConfig)
        .expect("read policy")
        .expect("policy exists");
    (document.content, document.revision.0)
}

#[test]
fn config_read_returns_content_revision_and_same_etag_to_both_roles() {
    let f = fixture();
    for pat in [&f.editor_pat, &f.reader_pat] {
        let (config, etag) = get_json(&f, pat, "config");
        assert_eq!(config["schema"], "spec-driven");
        assert_eq!(config["content"], INITIAL);
        let revision = config["revision"].as_u64().expect("numeric revision");
        let expected_etag = format!("\"{revision}\"");
        assert_eq!(
            etag.as_deref(),
            Some(expected_etag.as_str()),
            "revision equals ETag",
        );
    }
}

#[test]
fn put_advances_revision_and_stale_expected_revision_has_no_side_effect() {
    let f = fixture();
    let (before, _) = get_json(&f, &f.editor_pat, "config");
    let revision = before["revision"].as_u64().expect("revision");

    let winner = put_config(&f, &f.editor_pat, WINNER, revision).expect("winner succeeds");
    let winner_body = winner.into_json::<Value>().expect("winner JSON");
    let next = winner_body["revision"].as_u64().expect("new revision");
    assert!(next > revision, "a successful write advances revision");
    let (after, etag) = get_json(&f, &f.editor_pat, "config");
    assert_eq!(after["content"], WINNER);
    assert_eq!(after["revision"], next);
    let expected_etag = format!("\"{next}\"");
    assert_eq!(etag.as_deref(), Some(expected_etag.as_str()));

    let (status, error) = protocol_error(put_config(
        &f,
        &f.editor_pat,
        "schema: spec-driven\nlocale: ja\n",
        revision,
    ));
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::RevisionConflict);
    assert_eq!(stored_policy(&f), (WINNER.to_string(), next));
}

#[test]
fn invalid_yaml_and_missing_expected_revision_are_rejected_without_writes() {
    let f = fixture();
    let (before, _) = get_json(&f, &f.editor_pat, "config");
    let revision = before["revision"].as_u64().expect("revision");

    let (status, error) = protocol_error(put_config(
        &f,
        &f.editor_pat,
        "schema: [unterminated",
        revision,
    ));
    assert_eq!(status, 422);
    assert_eq!(error.reason, ErrorReason::InvalidConfig);
    assert!(!error.message.is_empty(), "the parse error is explained");
    assert_eq!(stored_policy(&f), (INITIAL.to_string(), revision));

    let missing = request("PUT", &f, &f.editor_pat, "config")
        .send_json(json!({ "content": WINNER }))
        .expect_err("expectedRevision is mandatory");
    let status = match missing {
        ureq::Error::Status(status, _) => status,
        other => panic!("unexpected transport error: {other}"),
    };
    assert!((400..500).contains(&status), "missing CAS input is rejected");
    assert_eq!(stored_policy(&f), (INITIAL.to_string(), revision));
}

#[test]
fn reader_write_is_forbidden_and_binding_capability_follows_role() {
    let f = fixture();
    let (config, _) = get_json(&f, &f.reader_pat, "config");
    let revision = config["revision"].as_u64().expect("revision");

    let (status, error) = protocol_error(put_config(&f, &f.reader_pat, WINNER, revision));
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
    assert_eq!(stored_policy(&f), (INITIAL.to_string(), revision));

    let (editor_binding, _) = get_json(&f, &f.editor_pat, "binding");
    let (reader_binding, _) = get_json(&f, &f.reader_pat, "binding");
    assert_eq!(editor_binding["capabilities"]["policyWrite"], true);
    assert_eq!(reader_binding["capabilities"]["policyWrite"], false);
}

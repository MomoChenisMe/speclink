//! Remote board-order reads and CAS writes (spec 「board resource 為 scope 單
//! 文件且 server 不解析」): absence is a normal state, If-Match carries the
//! scope revision, only editors write, content is opaque text under a size
//! cap, and a successful write reaches subscribers as an invalidation.

use crate::common;

use crate::common::subscriber::Recorder;
use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;
use std::time::Duration;

const ORDER: &str = "{\"changes\":{\"add-auth\":\"n\"},\"discussions\":{\"auth-scope\":\"c\"}}";

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

fn request(method: &str, f: &Fixture, pat: &str) -> ureq::Request {
    ureq::request(
        method,
        &format!("{}/api/speclink/v1/projects/demo/board-order", f.base),
    )
    .set("Authorization", &format!("Bearer {pat}"))
    .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
    .set("X-Speclink-Repo", "backend")
}

fn get_board_order(f: &Fixture, pat: &str) -> (Value, String) {
    let response = request("GET", f, pat).call().expect("GET succeeds");
    let etag = response.header("ETag").expect("ETag present").to_string();
    let body = response.into_json::<Value>().expect("JSON body");
    (body, etag)
}

fn put_board_order(
    f: &Fixture,
    pat: &str,
    content: &str,
    if_match: &str,
) -> Result<ureq::Response, ureq::Error> {
    request("PUT", f, pat)
        .set("If-Match", if_match)
        .send_json(json!({ "content": content }))
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

fn stored_order(f: &Fixture) -> Option<String> {
    let snapshot = f.store.snapshot(&scope()).expect("snapshot");
    snapshot
        .read(&DocumentId::BoardOrder)
        .expect("read board order")
        .map(|document| document.content)
}

#[test]
fn absent_board_order_reads_null_with_scope_etag_for_both_roles() {
    let f = fixture();
    for pat in [&f.editor_pat, &f.reader_pat] {
        let (body, etag) = get_board_order(&f, pat);
        assert_eq!(body["content"], Value::Null, "absence is a normal state");
        let revision = body["revision"].as_u64().expect("numeric revision");
        assert_eq!(etag, format!("\"{revision}\""), "revision equals ETag");
    }
}

#[test]
fn put_creates_and_a_stale_if_match_conflicts_without_side_effect() {
    let f = fixture();
    let (_, stale_etag) = get_board_order(&f, &f.editor_pat);

    let winner = put_board_order(&f, &f.editor_pat, ORDER, &stale_etag).expect("first PUT");
    let winner_body = winner.into_json::<Value>().expect("PUT JSON");
    let next = winner_body["revision"].as_u64().expect("new revision");
    let (after, etag) = get_board_order(&f, &f.editor_pat);
    assert_eq!(after["content"], ORDER, "content round-trips byte-for-byte");
    assert_eq!(after["revision"], next);
    assert_eq!(etag, format!("\"{next}\""));

    let (status, error) = protocol_error(put_board_order(
        &f,
        &f.editor_pat,
        "{\"changes\":{},\"discussions\":{}}",
        &stale_etag,
    ));
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::RevisionConflict);
    assert_eq!(stored_order(&f), Some(ORDER.to_string()), "loser left no trace");
}

#[test]
fn reader_write_is_forbidden_without_side_effect() {
    let f = fixture();
    let (_, etag) = get_board_order(&f, &f.reader_pat);
    let (status, error) = protocol_error(put_board_order(&f, &f.reader_pat, ORDER, &etag));
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
    assert_eq!(stored_order(&f), None);
}

#[test]
fn oversized_payload_is_rejected_without_side_effect() {
    let f = fixture();
    let (_, etag) = get_board_order(&f, &f.editor_pat);
    let huge = "x".repeat(2 * 1024 * 1024);
    let status = match put_board_order(&f, &f.editor_pat, &huge, &etag) {
        Ok(response) => panic!("oversized PUT accepted: {}", response.status()),
        Err(ureq::Error::Status(status, _)) => status,
        Err(error) => panic!("transport error: {error}"),
    };
    assert_eq!(status, 413, "the size cap answers before any write");
    assert_eq!(stored_order(&f), None);
}

#[test]
fn successful_put_reaches_subscribers_as_an_invalidation() {
    let f = fixture();
    let events_url = format!("{}/api/speclink/v1/projects/demo/events", f.base);
    let mut subscriber = Recorder::connect(&events_url, &f.editor_pat, "backend");

    let (_, etag) = get_board_order(&f, &f.editor_pat);
    let response = put_board_order(&f, &f.editor_pat, ORDER, &etag).expect("PUT succeeds");
    let revision = response.into_json::<Value>().expect("PUT JSON")["revision"]
        .as_u64()
        .expect("new revision");

    let event = subscriber.await_resource("", Duration::from_secs(5));
    assert_eq!(event.revision, revision, "the invalidation names the commit");
}

#[test]
fn arbitrary_text_is_stored_verbatim_without_validation() {
    let f = fixture();
    let (_, etag) = get_board_order(&f, &f.editor_pat);
    let garbage = "not json at all {{{ 亂碼 \u{1f5c2}";
    put_board_order(&f, &f.editor_pat, garbage, &etag).expect("opaque content lands");
    let (body, _) = get_board_order(&f, &f.editor_pat);
    assert_eq!(body["content"], garbage, "the server neither parses nor repairs");
}

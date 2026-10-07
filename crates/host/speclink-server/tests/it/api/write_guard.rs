//! Project-scope write guard (server-verb-api「寫入端點一律要求 editor 角色」):
//! every non-GET/HEAD project route refuses a reader with 403 before the handler
//! runs, so the scope revision never moves; `POST /context` is the one read-only
//! POST left open. Content-format failures surface as 400 invalid_argument.

use crate::common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

const READER_DENIED: &str =
    "your role in project 'demo' is reader; this action needs the editor role";

/// Every write route on the project router (method, path tail). A new write
/// route belongs here too — the guard covers it by default, this list proves it,
/// and the registration count below fails until the route is listed.
const WRITE_ROUTES: &[(&str, &str)] = &[
    ("POST", "changes"),
    ("DELETE", "changes/demo"),
    ("POST", "changes/demo/tasks/move"),
    ("PUT", "changes/demo/artifacts/proposal"),
    ("POST", "changes/demo/tasks/1/done"),
    ("POST", "changes/demo/tasks/1/undone"),
    ("POST", "changes/demo/review/rounds"),
    ("POST", "changes/demo/review/stamp"),
    ("DELETE", "changes/demo/review"),
    ("POST", "changes/demo/verify/rounds"),
    ("POST", "changes/demo/verify/stamp"),
    ("DELETE", "changes/demo/verify"),
    ("POST", "changes/demo/claim"),
    ("POST", "changes/demo/depends"),
    ("POST", "changes/demo/in-progress"),
    ("DELETE", "changes/demo/in-progress"),
    ("POST", "changes/demo/archive"),
    ("POST", "discussions"),
    ("DELETE", "discussions/talk"),
    ("POST", "discussions/talk/link"),
    ("POST", "discussions/talk/seal"),
    ("PUT", "discussions/talk/context"),
    ("POST", "discussions/talk/rounds"),
    ("POST", "discussions/talk/conclude"),
    ("POST", "discussions/talk/archive"),
    ("POST", "discussions/talk/promote"),
    ("PUT", "config"),
    ("PUT", "board-order"),
    ("POST", "import"),
];

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
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() },
        "## Why\n\noriginal\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n",
    );
    uow.create(
        DocumentId::Discussion { slug: "talk".into(), archived: false },
        "---\ntopic: Talk\nslug: talk\ncreated: 2026-07-01\n---\n\n## Context\n\nx\n\n## Rounds\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");

    let state = common::state_with(store.clone());
    let (editor_pat, _) =
        common::seed_named_pat(&state.identity, "editor@example.com", "Editor", &["demo"]);
    let (reader_pat, reader_id) =
        common::seed_named_pat(&state.identity, "reader@example.com", "Reader", &["demo"]);
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
    Fixture { base: common::start(state), store, editor_pat, reader_pat }
}

fn request(f: &Fixture, method: &str, pat: Option<&str>, tail: &str) -> ureq::Request {
    let req = ureq::request(method, &format!("{}/api/speclink/v1/projects/demo/{tail}", f.base))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend");
    match pat {
        Some(pat) => req.set("Authorization", &format!("Bearer {pat}")),
        None => req,
    }
}

fn revision(f: &Fixture) -> u64 {
    f.store.snapshot(&scope()).expect("snapshot").revision().0
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

#[test]
fn write_routes_lists_every_write_registration_on_the_project_router() {
    // Count the write-method registrations above the guard's `route_layer`;
    // `POST /context` sits below it and stays out of the count.
    let app = include_str!("../../../src/app.rs");
    let start = app.find("let project = Router::new()").expect("project router");
    let end = app.find(".route_layer(").expect("write guard layer");
    let registered: usize = ["post(", "put(", "delete(", "patch("]
        .iter()
        .map(|method| app[start..end].matches(method).count())
        .sum();
    assert_eq!(registered, WRITE_ROUTES.len(), "WRITE_ROUTES is out of sync with app.rs");
}

#[test]
fn every_write_route_refuses_a_reader_before_the_handler_runs() {
    let f = fixture();
    let before = revision(&f);
    for (method, tail) in WRITE_ROUTES {
        let result = request(&f, method, Some(&f.reader_pat), tail).send_json(json!({}));
        let (status, error) = protocol_error(result);
        assert_eq!(status, 403, "{method} {tail}");
        assert_eq!(error.reason, ErrorReason::PermissionDenied, "{method} {tail}");
        assert_eq!(error.message, READER_DENIED, "{method} {tail}");
        assert_eq!(revision(&f), before, "{method} {tail} must not commit");
    }
}

#[test]
fn a_reader_still_reads_and_takes_context_snapshots() {
    let f = fixture();
    let reader = request(&f, "POST", Some(&f.reader_pat), "context")
        .send_json(json!({}))
        .expect("a reader takes a context snapshot");
    let editor = request(&f, "POST", Some(&f.editor_pat), "context")
        .send_json(json!({}))
        .expect("an editor takes a context snapshot");
    assert_eq!(
        reader.into_json::<Value>().expect("reader JSON"),
        editor.into_json::<Value>().expect("editor JSON"),
        "the snapshot does not depend on the role"
    );
    request(&f, "GET", Some(&f.reader_pat), "changes")
        .call()
        .expect("a reader lists changes");
}

#[test]
fn an_unauthenticated_write_keeps_the_401() {
    let f = fixture();
    let (status, error) = protocol_error(
        request(&f, "POST", None, "changes").send_json(json!({ "name": "add-auth" })),
    );
    assert_eq!(status, 401);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
}

#[test]
fn malformed_content_from_an_editor_is_invalid_argument() {
    let f = fixture();
    let read: Value = request(&f, "GET", Some(&f.editor_pat), "changes/demo/artifacts/tasks")
        .call()
        .expect("read tasks")
        .into_json()
        .expect("JSON body");
    let version = read["version"].as_u64().expect("version");
    let (status, error) = protocol_error(
        request(&f, "PUT", Some(&f.editor_pat), "changes/demo/artifacts/tasks")
            .set("If-Match", &version.to_string())
            .send_json(json!({ "content": "no checkboxes here\n" })),
    );
    assert_eq!(status, 400);
    assert_eq!(error.reason, ErrorReason::InvalidArgument);
    assert_eq!(error.message, "Tasks must contain at least one checkbox (- [ ])");

    let (status, error) = protocol_error(
        request(&f, "POST", Some(&f.editor_pat), "changes/demo/review/rounds")
            .send_json(json!({ "content": "- [WARNING] src/lib.rs — no scope line\n" })),
    );
    assert_eq!(status, 400);
    assert_eq!(error.reason, ErrorReason::InvalidArgument);
    assert_eq!(
        error.message,
        "round content must contain a `**Scope**:` line listing the files reviewed"
    );
}

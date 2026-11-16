//! Local-to-remote migration import contract. The HTTP surface is deliberately
//! CreateNew-only: callers can populate one empty bound scope atomically, but
//! cannot select the store's maintenance-only overwrite mode.

use crate::common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::query::{
    ImportBundle, ImportBundleDocument, ImportDocumentId, ImportDocumentOutcome, ImportScope,
};
use speclink_remote::client::Client;
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;

struct Fixture {
    base: String,
    store: common::TestStore,
    editor_pat: String,
    reader_pat: String,
}

/// The digest a bundle carries for `content`: SHA-256, lowercase hex, with
/// the algorithm as a prefix.
fn content_digest(content: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(content.as_bytes()))
}

fn fixture() -> Fixture {
    let store = common::TestStore::memory();
    let state = common::state_with(store.shared());
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
    Fixture {
        base: common::start(state),
        store,
        editor_pat,
        reader_pat,
    }
}

fn request(method: &str, fixture: &Fixture, pat: &str, tail: &str) -> ureq::Request {
    ureq::request(
        method,
        &format!("{}/api/speclink/v1/projects/demo/{tail}", fixture.base),
    )
    .set("Authorization", &format!("Bearer {pat}"))
    .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
    .set("X-Speclink-Repo", "backend")
}

fn get_json(fixture: &Fixture, pat: &str, tail: &str) -> Value {
    request("GET", fixture, pat, tail)
        .call()
        .expect("GET succeeds")
        .into_json()
        .expect("JSON body")
}

fn post_import(fixture: &Fixture, pat: &str, body: Value) -> Result<ureq::Response, ureq::Error> {
    request("POST", fixture, pat, "import").send_json(body)
}

fn error_of(result: Result<ureq::Response, ureq::Error>) -> (u16, String, Option<ErrorReason>) {
    match result {
        Ok(response) => panic!("expected protocol error, got {}", response.status()),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            let parsed = serde_json::from_str::<ErrorResponse>(&body).ok();
            let reason = parsed.as_ref().map(|error| error.reason.clone());
            let message = parsed.map(|error| error.message).unwrap_or(body);
            (status, message, reason)
        }
        Err(error) => panic!("transport error: {error}"),
    }
}

fn document(document: Value, content: &str) -> Value {
    json!({
        "document": document,
        "content": content,
        "digest": content_digest(content),
    })
}

fn migration_documents() -> Vec<Value> {
    vec![
        document(
            json!({ "type": "changeMeta", "change": "demo" }),
            "schema: spec-driven\ncreated: 2026-07-21\n",
        ),
        document(
            json!({
                "type": "changeArtifact",
                "change": "demo",
                "artifact": "proposal.md"
            }),
            "## Why\n\nImported proposal truth.\n",
        ),
        document(
            json!({
                "type": "changeArtifact",
                "change": "demo",
                "artifact": "tasks.md"
            }),
            "- [ ] 1.1 Imported task\n",
        ),
        document(
            json!({ "type": "canonicalSpec", "capability": "payments" }),
            "# payments Specification\n\nImported canonical truth.\n",
        ),
        document(
            json!({ "type": "discussion", "slug": "migration-plan", "archived": false }),
            "---\ntopic: Migration plan\nslug: migration-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nImported live discussion.\n",
        ),
        document(
            json!({
                "type": "discussion",
                "slug": "2026-07-20-old-plan",
                "archived": true
            }),
            "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nImported archived discussion.\n",
        ),
        document(
            json!({
                "type": "archivedChange",
                "change": "2026-07-20-old-change",
                "doc": "proposal.md"
            }),
            "## Why\n\nImported archived proposal.\n",
        ),
        document(
            json!({ "type": "workflowConfig" }),
            "schema: spec-driven\nlocale: tw\n",
        ),
        document(
            json!({ "type": "language" }),
            "Store port: 儲存接口\n",
        ),
    ]
}

fn bundle(documents: Vec<Value>) -> Value {
    json!({
        "formatVersion": 1,
        "scope": { "project": "demo", "repo": "backend" },
        "projectRevision": 27,
        "documents": documents,
    })
}

#[test]
fn empty_scope_import_reports_created_and_every_read_surface_returns_bundle_truth() {
    let fixture = fixture();
    let documents = migration_documents();
    let response = post_import(&fixture, &fixture.editor_pat, bundle(documents.clone()))
        .expect("empty scope import succeeds")
        .into_json::<Value>()
        .expect("ImportReport JSON");

    assert_eq!(response["projectRevision"], 1);
    let outcomes = response["documents"].as_array().expect("document outcomes");
    assert_eq!(outcomes.len(), documents.len());
    assert!(
        outcomes
            .iter()
            .all(|document| document["outcome"] == "created"),
        "CreateNew has no overwrite outcome: {outcomes:?}"
    );

    let changes = get_json(&fixture, &fixture.editor_pat, "changes");
    assert_eq!(changes["changes"][0]["name"], "demo");
    let proposal = get_json(
        &fixture,
        &fixture.editor_pat,
        "changes/demo/artifacts/proposal",
    );
    assert_eq!(proposal["content"], "## Why\n\nImported proposal truth.\n");
    let spec = get_json(&fixture, &fixture.editor_pat, "specs/payments/document");
    assert_eq!(
        spec["content"],
        "# payments Specification\n\nImported canonical truth.\n"
    );
    let discussion = get_json(&fixture, &fixture.editor_pat, "discussions/migration-plan");
    assert!(discussion["content"]
        .as_str()
        .unwrap()
        .contains("Imported live discussion."));
    let archived_discussions = get_json(&fixture, &fixture.editor_pat, "discussions?archived=true");
    assert_eq!(archived_discussions["discussions"][0]["slug"], "old-plan");
    let archived = get_json(&fixture, &fixture.editor_pat, "archived");
    assert_eq!(
        archived["archived"][0]["datedName"],
        "2026-07-20-old-change"
    );
    let archived_proposal = get_json(
        &fixture,
        &fixture.editor_pat,
        "archived/2026-07-20-old-change/artifacts/proposal.md",
    );
    assert_eq!(
        archived_proposal["content"],
        "## Why\n\nImported archived proposal.\n"
    );
    let config = get_json(&fixture, &fixture.editor_pat, "config");
    assert_eq!(config["content"], "schema: spec-driven\nlocale: tw\n");
    let language = get_json(&fixture, &fixture.editor_pat, "language");
    assert_eq!(language["content"], "Store port: 儲存接口\n");
}

#[test]
fn typed_client_exposes_the_create_new_only_import_contract() {
    let fixture = fixture();
    let client = Client::new(
        &format!("{}/api/speclink/v1/projects/demo", fixture.base),
        &fixture.editor_pat,
        Some("backend"),
    );
    let content = "schema: spec-driven\n";
    let report = client
        .import(&ImportBundle {
            format_version: 1,
            scope: ImportScope {
                project: "demo".into(),
                repo: "backend".into(),
            },
            project_revision: 0,
            documents: vec![ImportBundleDocument {
                document: ImportDocumentId::WorkflowConfig,
                content: content.into(),
                digest: content_digest(content),
            }],
        })
        .expect("typed import succeeds");

    assert_eq!(report.project_revision, 1);
    assert_eq!(report.documents.len(), 1);
    assert_eq!(report.documents[0].outcome, ImportDocumentOutcome::Created);
}

#[test]
fn import_accepts_workspace_bundles_above_axums_default_body_limit() {
    let fixture = fixture();
    let content = "x".repeat(3 * 1024 * 1024);
    let response = post_import(
        &fixture,
        &fixture.editor_pat,
        bundle(vec![document(
            json!({ "type": "canonicalSpec", "capability": "large-workspace" }),
            &content,
        )]),
    )
    .expect("large workspace import succeeds")
    .into_json::<Value>()
    .expect("ImportReport JSON");

    assert_eq!(response["projectRevision"], 1);
    assert_eq!(response["documents"][0]["outcome"], "created");
}

#[test]
fn non_empty_scope_is_refused_without_changing_content_or_revision() {
    let fixture = fixture();
    let before_revision =
        fixture.store.put("demo", "backend", &[("config.yaml", "schema: spec-driven\n")]);

    let (status, message, reason) = error_of(post_import(
        &fixture,
        &fixture.editor_pat,
        bundle(migration_documents()),
    ));
    assert_eq!(status, 409);
    assert_eq!(reason, Some(ErrorReason::Refused));
    assert!(
        message.contains("create-new"),
        "the CreateNew gate stays visible: {message}"
    );

    assert_eq!(fixture.store.version("demo", "backend"), before_revision);
    assert_eq!(
        fixture
            .store
            .content("demo", "backend", "config.yaml")
            .expect("existing config"),
        "schema: spec-driven\n"
    );
    assert!(
        fixture
            .store
            .doc("demo", "backend", "specs/payments/spec.md")
            .is_none(),
        "no prefix of the rejected bundle was written"
    );
}

#[test]
fn reader_cannot_import_even_with_a_valid_bundle() {
    let fixture = fixture();
    let (status, _, reason) = error_of(post_import(
        &fixture,
        &fixture.reader_pat,
        bundle(migration_documents()),
    ));
    assert_eq!(status, 403);
    assert_eq!(reason, Some(ErrorReason::PermissionDenied));
    assert_eq!(fixture.store.version("demo", "backend"), 0);
}

#[test]
fn unknown_version_missing_fields_and_an_overwrite_selector_are_not_accepted() {
    let fixture = fixture();
    let mut unknown = bundle(migration_documents());
    unknown["formatVersion"] = json!(999);
    let (status, message, _) = error_of(post_import(&fixture, &fixture.editor_pat, unknown));
    assert_eq!(status, 409);
    assert!(message.contains("format version"));

    let missing = json!({
        "scope": { "project": "demo", "repo": "backend" },
        "projectRevision": 0,
        "documents": [],
    });
    assert_eq!(
        error_of(post_import(&fixture, &fixture.editor_pat, missing)).0,
        422
    );

    let mut overwrite = bundle(migration_documents());
    overwrite["mode"] = json!("overwrite");
    assert_eq!(
        error_of(post_import(&fixture, &fixture.editor_pat, overwrite)).0,
        422,
        "the migration wire has no Overwrite selector"
    );
    assert_eq!(fixture.store.version("demo", "backend"), 0);
}

#[test]
fn a_large_bundle_with_a_late_validation_failure_leaves_no_partial_state() {
    let fixture = fixture();
    let mut documents = migration_documents();
    for index in 0..64 {
        let content = format!("# generated-{index}\n");
        documents.push(document(
            json!({ "type": "canonicalSpec", "capability": format!("generated-{index}") }),
            &content,
        ));
    }
    documents.last_mut().unwrap()["digest"] = json!("sha256:interrupted");

    let (status, _, _) = error_of(post_import(
        &fixture,
        &fixture.editor_pat,
        bundle(documents),
    ));
    assert!(status >= 400);
    assert_eq!(fixture.store.version("demo", "backend"), 0);
    assert!(
        fixture
            .store
            .doc("demo", "backend", "changes/demo/.meta.yaml")
            .is_none(),
        "validation rejects the whole request before any prefix is visible"
    );
}

// ======================================================================
// The import is the host kit's, on the store port (import-export-on-port,
// design D7): no import of the store's own, an event for the stream, and the
// export.
// ======================================================================

use crate::common::subscriber::Recorder;
use speclink_protocol::query::{BundleDocument, ScopeBundle};
use std::time::Duration;

fn typed_client(fixture: &Fixture, pat: &str) -> Client {
    Client::new(&format!("{}/api/speclink/v1/projects/demo", fixture.base), pat, Some("backend"))
}

fn keyed_document(key: &str, content: &str) -> BundleDocument {
    BundleDocument { key: key.into(), content: content.into(), digest: content_digest(content) }
}

#[test]
fn a_successful_import_writes_every_document_in_one_commit() {
    let inner = common::TestStore::memory();
    let state = common::state_with(inner.shared());
    let (pat, _) = common::seed_named_pat(&state.identity, "editor@example.com", "Editor", &["demo"]);
    let base = common::start(state);

    let response = ureq::post(&format!("{base}/api/speclink/v1/projects/demo/import"))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .send_json(bundle(migration_documents()))
        .expect("the import succeeds")
        .into_json::<Value>()
        .expect("ImportReport JSON");

    assert_eq!(response["documents"].as_array().map(Vec::len), Some(migration_documents().len()));
    assert_eq!(inner.version("demo", "backend"), 1, "one commit wrote every document");
    assert!(inner.doc("demo", "backend", "specs/payments/spec.md").is_some());
}

#[test]
fn a_subscriber_of_the_event_stream_gets_one_invalidation_after_an_import() {
    let fixture = fixture();
    let events_url = format!("{}/api/speclink/v1/projects/demo/events", fixture.base);
    let mut subscriber = Recorder::connect(&events_url, &fixture.editor_pat, "backend");

    let revision = post_import(&fixture, &fixture.editor_pat, bundle(migration_documents()))
        .expect("the import succeeds")
        .into_json::<Value>()
        .expect("ImportReport JSON")["projectRevision"]
        .as_u64()
        .expect("the revision");

    // An import changes every resource of the scope: it names none.
    let event = subscriber.await_resource("", Duration::from_secs(5));
    assert_eq!(event.revision, revision, "the invalidation names the import's commit");
    assert_eq!(event.scope, "unknown");
    assert_eq!(subscriber.drain(Duration::from_millis(300)), 0, "one import, one invalidation");
}

#[test]
fn the_export_gives_an_editor_the_keyed_bundle_and_refuses_a_reader() {
    let fixture = fixture();
    post_import(&fixture, &fixture.editor_pat, bundle(migration_documents())).expect("the import succeeds");

    let exported = get_json(&fixture, &fixture.editor_pat, "export");
    assert_eq!(exported["formatVersion"], 2);
    assert_eq!(exported["scopeVersion"], 1);
    let keys: Vec<&str> = exported["documents"]
        .as_array()
        .expect("documents")
        .iter()
        .map(|document| document["key"].as_str().expect("a key"))
        .collect();
    assert_eq!(
        keys,
        [
            "LANGUAGE.md",
            "archive/changes/2026-07-20-old-change/proposal.md",
            "archive/discussions/2026-07-20-old-plan.md",
            "changes/demo/.meta.yaml",
            "changes/demo/proposal.md",
            "changes/demo/tasks.md",
            "config.yaml",
            "discussions/migration-plan.md",
            "specs/payments/spec.md",
        ],
        "every document, in key order"
    );

    let (status, _, reason) = error_of(request("GET", &fixture, &fixture.reader_pat, "export").call());
    assert_eq!(status, 403);
    assert_eq!(reason, Some(ErrorReason::PermissionDenied));
}

#[test]
fn the_typed_client_imports_a_keyed_bundle_and_exports_it_back() {
    let fixture = fixture();
    let client = typed_client(&fixture, &fixture.editor_pat);
    let documents = vec![
        keyed_document("changes/demo/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-21\n"),
        keyed_document("config.yaml", "schema: spec-driven\n"),
    ];

    let report = client
        .import_scope(&ScopeBundle { format_version: 2, scope_version: None, documents: documents.clone() })
        .expect("the keyed import succeeds");
    assert_eq!((report.scope_version, report.documents), (1, 2));

    let exported = client.export().expect("the export succeeds");
    assert_eq!(exported.format_version, 2);
    assert_eq!(exported.scope_version, Some(1));
    assert_eq!(exported.documents, documents);

    // A second import finds the scope not empty.
    let error = client.import_scope(&exported).expect_err("the scope is not empty");
    assert!(error.to_string().contains("create-new"), "{error}");
}

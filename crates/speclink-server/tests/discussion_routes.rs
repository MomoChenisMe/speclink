//! Discussion routes over the same bridge and commit path (reference-server
//! spec). Promoting a discussion returns the new change name and lands both a
//! discussion-promoted and a change-created event in the scope outbox.

mod common;

use serde_json::{json, Value};
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_remote::client::Client;
use speclink_server::audit::AuditActor;
use speclink_server::identity::MembershipRole;
use speclink_store::memory::MemoryStore;
use speclink_store::{
    CommandContext, DocumentId, OutboxCursor, ProjectId, RepoId, Scope, TeamStore,
};
use std::sync::Arc;

fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

fn client(base: &str, token: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        token,
        Some("backend"),
    )
}

#[test]
fn create_and_show_round_trip_a_discussion() {
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let state = common::state_with(store);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let created = client.new_discussion("Rate limiting", None).expect("create discussion");
    assert!(!created.slug.is_empty(), "a slug is derived from the topic");

    let shown = client.show_discussion(&created.slug).expect("show discussion");
    assert_eq!(shown.info.slug, created.slug);
    assert_eq!(shown.info.topic, "Rate limiting");

    let listed = client.list_discussions(false).expect("list discussions");
    assert!(listed.discussions.iter().any(|d| d.slug == created.slug), "the discussion is listed");
}

#[test]
fn promote_returns_the_change_and_lands_both_events() {
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let created = client.new_discussion("Auth scope", None).expect("create discussion");
    let promoted = client
        .discussion_promote(&created.slug, None)
        .expect("promote discussion");
    assert!(!promoted.change.is_empty(), "promote returns the new change name");

    let entries = store.read_outbox(&scope(), OutboxCursor(0)).expect("read outbox");
    let kinds: Vec<&str> = entries.iter().map(|e| e.record.name.as_str()).collect();
    assert!(
        kinds.contains(&"discussion-promoted"),
        "a discussion-promoted event landed: {kinds:?}"
    );
    assert!(
        kinds.contains(&"change-created"),
        "a change-created event landed in the same promote: {kinds:?}"
    );
}

// --- 規格「討論寫入動詞端點補齊」---

struct Fixture {
    base: String,
    store: Arc<MemoryStore>,
    editor_pat: String,
    reader_pat: String,
}

/// Server over an empty store with an editor PAT and a reader PAT both live.
fn fixture() -> Fixture {
    let store = Arc::new(MemoryStore::new());
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

fn request(method: &str, f: &Fixture, pat: &str, tail: &str) -> ureq::Request {
    ureq::request(method, &format!("{}/api/speclink/v1/projects/demo/{tail}", f.base))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
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

fn revision(f: &Fixture) -> u64 {
    f.store.snapshot(&scope()).expect("snapshot").revision().0
}

fn outbox_names(f: &Fixture) -> Vec<String> {
    f.store
        .read_outbox(&scope(), OutboxCursor(0))
        .expect("read outbox")
        .iter()
        .map(|e| e.record.name.clone())
        .collect()
}

fn live_discussion(f: &Fixture, slug: &str) -> Option<String> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::Discussion { slug: slug.into(), archived: false })
        .expect("read discussion")
        .map(|d| d.content)
}

fn change_meta(f: &Fixture, change: &str) -> Option<String> {
    f.store
        .snapshot(&scope())
        .expect("snapshot")
        .read(&DocumentId::ChangeMeta { change: change.into() })
        .expect("read change meta")
        .map(|d| d.content)
}

/// Seed change `demo` directly in the store (the link/seal subject).
fn seed_change(f: &Fixture) {
    let mut uow = f
        .store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    f.store.commit(uow, Vec::new()).expect("seed commit");
}

#[test]
fn create_with_a_legal_slug_override_lands_on_that_slug() {
    let f = fixture();
    let response = request("POST", &f, &f.editor_pat, "discussions")
        .send_json(json!({ "topic": "看板搜尋列", "slug": "board-search-bar" }))
        .expect("create with slug override succeeds");
    assert_eq!(response.status(), 200);
    let body = response.into_json::<Value>().expect("JSON body");
    assert_eq!(body["slug"], "board-search-bar", "the response slug is the override");
    assert_eq!(body["topic"], "看板搜尋列", "the topic stays verbatim");
    let doc = live_discussion(&f, "board-search-bar").expect("the record lands on the slug");
    assert!(doc.contains("topic: 看板搜尋列"), "the document keeps the CJK topic: {doc}");
}

#[test]
fn create_with_an_invalid_slug_refuses_without_writing() {
    let f = fixture();
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("POST", &f, &f.editor_pat, "discussions")
            .send_json(json!({ "topic": "看板搜尋列", "slug": "中文slug" })),
    );
    assert_eq!(status, 400);
    assert_eq!(error.reason, ErrorReason::InvalidArgument, "semantic slug refusal");
    assert!(
        error.message.contains("must be ASCII kebab-case"),
        "the engine's frozen message is relayed verbatim: {}",
        error.message
    );
    assert_eq!(revision(&f), before, "a refused create writes nothing");
    assert!(outbox_names(&f).is_empty(), "a refused create publishes no event");
}

#[test]
fn delete_zero_round_discussion_removes_it_and_advances_revision() {
    let f = fixture();
    let slug = client(&f.base, &f.editor_pat)
        .new_discussion("Scrap idea", None)
        .expect("create discussion")
        .slug;
    let before = revision(&f);
    let response = request("DELETE", &f, &f.editor_pat, &format!("discussions/{slug}"))
        .call()
        .expect("delete a zero-round discussion succeeds");
    assert_eq!(response.status(), 200);
    assert_eq!(live_discussion(&f, &slug), None, "the record is gone");
    assert!(revision(&f) > before, "the delete commit advances the scope revision");
    assert!(
        outbox_names(&f).contains(&"discussion-discarded".to_string()),
        "the commit publishes discussion-discarded: {:?}",
        outbox_names(&f)
    );
}

#[test]
fn delete_with_rounds_requires_force_and_preserves_the_record() {
    let f = fixture();
    let c = client(&f.base, &f.editor_pat);
    let slug = c.new_discussion("Real tradeoffs", None).expect("create discussion").slug;
    c.discussion_add_round(&slug, "assumptions", "第一輪紀錄")
        .expect("add a round");
    let doc_before = live_discussion(&f, &slug).expect("record exists");
    let before = revision(&f);

    let (status, error) = protocol_error(
        request("DELETE", &f, &f.editor_pat, &format!("discussions/{slug}")).call(),
    );
    assert_eq!(status, 409);
    assert_eq!(error.reason, ErrorReason::Refused, "machine-readable needs-force refusal");
    assert!(error.message.contains("--force"), "the refusal names --force: {}", error.message);
    assert_eq!(
        live_discussion(&f, &slug).as_deref(),
        Some(doc_before.as_str()),
        "the record is byte-identical after the refusal"
    );
    assert_eq!(revision(&f), before, "a refused delete writes nothing");

    let response =
        request("DELETE", &f, &f.editor_pat, &format!("discussions/{slug}?force=true"))
            .call()
            .expect("force=true deletes the discussion");
    assert_eq!(response.status(), 200);
    assert_eq!(live_discussion(&f, &slug), None, "force removes the record");
}

#[test]
fn reader_discussion_delete_is_forbidden_with_intact_record() {
    let f = fixture();
    let slug = client(&f.base, &f.editor_pat)
        .new_discussion("Reader target", None)
        .expect("create discussion")
        .slug;
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("DELETE", &f, &f.reader_pat, &format!("discussions/{slug}")).call(),
    );
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied, "machine-readable role refusal");
    assert!(live_discussion(&f, &slug).is_some(), "the discussion is fully preserved");
    assert_eq!(revision(&f), before, "a reader refusal writes nothing");
}

#[test]
fn link_forges_the_meta_chain_and_seal_marks_promoted() {
    let f = fixture();
    seed_change(&f);
    let c = client(&f.base, &f.editor_pat);
    let slug = c.new_discussion("Auth scope", None).expect("create discussion").slug;

    let response = request("POST", &f, &f.editor_pat, &format!("discussions/{slug}/link"))
        .send_json(json!({ "change": "demo" }))
        .expect("link succeeds");
    assert_eq!(response.status(), 200);
    let body = response.into_json::<Value>().expect("JSON body");
    assert_eq!(body["slug"], slug.as_str());
    assert_eq!(body["change"], "demo");
    let meta = change_meta(&f, "demo").expect("change meta exists");
    assert!(
        meta.contains(&format!("from_discussion: {slug}")),
        "link forges the from_discussion chain: {meta}"
    );

    let response = request("POST", &f, &f.editor_pat, &format!("discussions/{slug}/seal"))
        .send_json(json!({ "change": "demo" }))
        .expect("seal succeeds");
    assert_eq!(response.status(), 200);
    let shown = c.show_discussion(&slug).expect("show discussion");
    assert_eq!(shown.info.status, "promoted", "seal marks the discussion promoted");
    let kinds = outbox_names(&f);
    assert!(
        kinds.contains(&"discussion-linked".to_string())
            && kinds.contains(&"discussion-sealed".to_string()),
        "link and seal each publish their event: {kinds:?}"
    );
}

#[test]
fn discussion_writes_on_missing_subjects_are_404_with_engine_messages() {
    let f = fixture();
    seed_change(&f);
    let slug = client(&f.base, &f.editor_pat)
        .new_discussion("Exists", None)
        .expect("create discussion")
        .slug;

    // 缺席的討論：DELETE 與 link/seal 皆 404，訊息為引擎凍結文本。
    let (status, error) =
        protocol_error(request("DELETE", &f, &f.editor_pat, "discussions/no-such").call());
    assert_eq!(status, 404);
    assert_eq!(error.reason, ErrorReason::NotFound);
    assert!(error.message.contains("'no-such' not found"), "{}", error.message);

    for verb in ["link", "seal"] {
        let (status, error) = protocol_error(
            request("POST", &f, &f.editor_pat, &format!("discussions/no-such/{verb}"))
                .send_json(json!({ "change": "demo" })),
        );
        assert_eq!(status, 404, "{verb} on a missing discussion");
        assert_eq!(error.reason, ErrorReason::NotFound);
        assert!(error.message.contains("'no-such' not found"), "{}", error.message);

        // 缺席的 change：同樣 404。
        let (status, error) = protocol_error(
            request("POST", &f, &f.editor_pat, &format!("discussions/{slug}/{verb}"))
                .send_json(json!({ "change": "no-such-change" })),
        );
        assert_eq!(status, 404, "{verb} on a missing change");
        assert_eq!(error.reason, ErrorReason::NotFound);
        assert!(
            error.message.contains("Change 'no-such-change' not found."),
            "{}",
            error.message
        );
    }
}

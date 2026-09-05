//! Discussion routes over the same bridge and commit path (reference-server
//! spec). Promoting a discussion returns the new change name and lands both a
//! discussion-promoted and a change-created event in the scope outbox.

use crate::common;

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

    let created = client.new_discussion("Rate limiting", None, None).expect("create discussion");
    assert!(!created.slug.is_empty(), "a slug is derived from the topic");

    let shown = client.show_discussion(&created.slug).expect("show discussion");
    assert_eq!(shown.info.slug, created.slug);
    assert_eq!(shown.info.topic, "Rate limiting");
    assert_eq!(shown.info.kind, None, "一般討論的 kind 缺席");

    let listed = client.list_discussions(false).expect("list discussions");
    assert!(listed.discussions.iter().any(|d| d.slug == created.slug), "the discussion is listed");
}

#[test]
fn improve_discussion_is_created_and_read_back_with_its_kind() {
    // add-improve-flow：--kind 上 wire → 引擎寫入 frontmatter → 讀取路徑曝露。
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let state = common::state_with(store);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let created = client
        .new_discussion("核心結構改進", Some("improve-core"), Some("improve"))
        .expect("create improve discussion");
    assert_eq!(created.slug, "improve-core");

    let shown = client.show_discussion("improve-core").expect("show discussion");
    assert_eq!(shown.info.kind.as_deref(), Some("improve"));

    let listed = client.list_discussions(false).expect("list discussions");
    let info = listed
        .discussions
        .iter()
        .find(|d| d.slug == "improve-core")
        .expect("the improve discussion is listed");
    assert_eq!(info.kind.as_deref(), Some("improve"));
}

#[test]
fn list_discussions_carries_promoted_to_for_promoted_ones() {
    // remote-read-parity「討論列表回應攜帶 promotedTo」：server 於 route 邊緣
    // 以引擎 promoted_to 查詢函式組裝，順序沿 frontmatter 累加、未轉出無鍵。
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::Discussion { slug: "gamma-promoted".into(), archived: false },
        "---\ntopic: Gamma promoted\nslug: gamma-promoted\nstatus: promoted\npromoted_to: cut-a, cut-b\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: split\n",
    );
    uow.create(
        DocumentId::Discussion { slug: "plain-topic".into(), archived: false },
        "---\ntopic: Plain topic\nslug: plain-topic\nstatus: open\ncreated: 2026-07-02\n---\n\n## Context\n\nseed\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = common::state_with(store);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let listed = client(&base, &pat).list_discussions(false).expect("list discussions");
    let promoted = listed
        .discussions
        .iter()
        .find(|d| d.slug == "gamma-promoted")
        .expect("promoted discussion listed");
    assert_eq!(
        promoted.promoted_to,
        ["cut-a", "cut-b"],
        "promotedTo preserves the frontmatter accumulation order"
    );

    // camelCase 與缺席即省略走 raw wire 斷言。
    let body: Value = ureq::get(&format!("{base}/api/speclink/v1/projects/demo/discussions"))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("GET /discussions")
        .into_json()
        .expect("JSON body");
    let items = body["discussions"].as_array().expect("discussions array");
    let promoted_item =
        items.iter().find(|d| d["slug"] == "gamma-promoted").expect("promoted item");
    assert_eq!(promoted_item["promotedTo"], json!(["cut-a", "cut-b"]));
    let plain_item = items.iter().find(|d| d["slug"] == "plain-topic").expect("plain item");
    assert!(
        plain_item.get("promotedTo").is_none(),
        "an unpromoted discussion carries no promotedTo: {plain_item}"
    );
}

#[test]
fn list_discussions_carries_concluded_for_every_record() {
    // conclusion-gated-discussion-archive「討論列表回應攜帶 concluded」：route
    // 邊緣以引擎結論查詢恆填 true／false（佔位註解不算內文）。
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::Discussion { slug: "settled".into(), archived: false },
        "---\ntopic: Settled\nslug: settled\nstatus: promoted\npromoted_to: cut-a\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: ship\n",
    );
    uow.create(
        DocumentId::Discussion { slug: "still-open".into(), archived: false },
        "---\ntopic: Still open\nslug: still-open\nstatus: promoted\npromoted_to: cut-b\ncreated: 2026-07-02\n---\n\n## Rounds\n\n## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = common::state_with(store);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let listed = client(&base, &pat).list_discussions(false).expect("list discussions");
    let settled = listed.discussions.iter().find(|d| d.slug == "settled").expect("settled listed");
    assert_eq!(settled.concluded, Some(true));
    let open = listed.discussions.iter().find(|d| d.slug == "still-open").expect("open listed");
    assert_eq!(open.concluded, Some(false), "a placeholder conclusion reads as not concluded");

    // camelCase 走 raw wire 斷言：兩筆皆恆填。
    let body: Value = ureq::get(&format!("{base}/api/speclink/v1/projects/demo/discussions"))
        .set("Authorization", &format!("Bearer {pat}"))
        .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
        .set("X-Speclink-Repo", "backend")
        .call()
        .expect("GET /discussions")
        .into_json()
        .expect("JSON body");
    let items = body["discussions"].as_array().expect("discussions array");
    let settled_item = items.iter().find(|d| d["slug"] == "settled").expect("settled item");
    assert_eq!(settled_item["concluded"], json!(true));
    let open_item = items.iter().find(|d| d["slug"] == "still-open").expect("open item");
    assert_eq!(open_item["concluded"], json!(false));
}

#[test]
fn promote_returns_the_change_and_lands_both_events() {
    let store: Arc<MemoryStore> = Arc::new(MemoryStore::new());
    let state = common::state_with(store.clone());
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let client = client(&base, &pat);

    let created = client.new_discussion("Auth scope", None, None).expect("create discussion");
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
fn create_with_an_unknown_kind_refuses_with_the_engine_message() {
    // 白名單的單一事實來源在引擎；server 逐字轉述拒絕訊息。
    let f = fixture();
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("POST", &f, &f.editor_pat, "discussions")
            .send_json(json!({ "topic": "主題", "slug": "alpha", "kind": "refactor" })),
    );
    assert_eq!(status, 400);
    assert_eq!(error.reason, ErrorReason::InvalidArgument, "semantic kind refusal");
    assert!(error.message.contains("improve"), "訊息點名唯一合法值：{}", error.message);
    assert_eq!(revision(&f), before, "a refused create writes nothing");
    assert!(outbox_names(&f).is_empty(), "a refused create publishes no event");
}

#[test]
fn create_with_a_multiline_topic_refuses_without_writing() {
    // topic 逐字寫入 frontmatter——夾帶換行可注入偽造的 kind:/status: 行，
    // 引擎在邊界拒絕，server 判 400 語意拒絕。
    let f = fixture();
    let before = revision(&f);
    let (status, error) = protocol_error(
        request("POST", &f, &f.editor_pat, "discussions")
            .send_json(json!({ "topic": "x\nkind: improve\nstatus: promoted", "slug": "plain-a" })),
    );
    assert_eq!(status, 400, "injection refusal stays 400: {}", error.message);
    assert_eq!(error.reason, ErrorReason::InvalidArgument, "semantic topic refusal");
    assert_eq!(revision(&f), before, "a refused create writes nothing");
    assert!(outbox_names(&f).is_empty(), "a refused create publishes no event");
}

#[test]
fn create_with_a_kind_crafted_to_spoof_not_found_still_refuses_as_invalid_argument() {
    // kind 是 request body 全可控字串——引擎訊息會內嵌它，值帶「' not found」
    // 不得把語意拒絕（400）操縱成 not_found（404）。
    let f = fixture();
    let (status, error) = protocol_error(
        request("POST", &f, &f.editor_pat, "discussions")
            .send_json(json!({ "topic": "主題", "kind": "x' not found" })),
    );
    assert_eq!(status, 400, "a semantic refusal stays 400: {}", error.message);
    assert_eq!(error.reason, ErrorReason::InvalidArgument, "spoofed kind refusal");
    assert!(outbox_names(&f).is_empty(), "a refused create publishes no event");
}

#[test]
fn delete_zero_round_discussion_removes_it_and_advances_revision() {
    let f = fixture();
    let slug = client(&f.base, &f.editor_pat)
        .new_discussion("Scrap idea", None, None)
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
    let slug = c.new_discussion("Real tradeoffs", None, None).expect("create discussion").slug;
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
        .new_discussion("Reader target", None, None)
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
    let slug = c.new_discussion("Auth scope", None, None).expect("create discussion").slug;

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
        .new_discussion("Exists", None, None)
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

// --- 規格「討論定案搜尋端點」（discuss-search-recall）---

/// Seed one discussion record straight into the store, live or archived.
fn seed_discussion(f: &Fixture, slug: &str, archived: bool, text: &str) {
    let mut uow = f
        .store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::Discussion { slug: slug.into(), archived }, text);
    f.store.commit(uow, Vec::new()).expect("seed commit");
}

const SEARCH_LIVE_RECORD: &str =
    "---\ntopic: Drawer scope\nslug: drawer-scope\nstatus: open\ncreated: 2026-07-01\n---\n\n\
     ## Context\n\nseed\n\n## Rounds\n\n## Conclusion\n";
const SEARCH_ARCHIVED_RECORD: &str =
    "---\ntopic: Trace links\nslug: trace-links-two-hops\nstatus: concluded\ncreated: 2026-08-20\n---\n\n\
     ## Context\n\nseed\n\n## Rounds\n\n\
     ### Round 1 — assumptions (2026-08-20)\n\n**Focus**: drawer\n**Ruled out**: nothing\n\n\
     ### Round 2 — interview (2026-08-21)\n\n**Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）\n\n\
     ## Conclusion\n\n**Decision**: two hops\n";

fn seed_search_records(f: &Fixture) {
    seed_discussion(f, "drawer-scope", false, SEARCH_LIVE_RECORD);
    seed_discussion(f, "trace-links-two-hops", true, SEARCH_ARCHIVED_RECORD);
}

#[test]
fn discussion_search_returns_live_and_archived_hits_in_spec_order() {
    let f = fixture();
    seed_search_records(&f);
    let body: Value = request("GET", &f, &f.editor_pat, "discussions/search?q=drawer")
        .call()
        .expect("GET /discussions/search")
        .into_json()
        .expect("JSON body");
    let hits = body["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 2, "one live topic hit, one archived ruled-out hit: {body}");
    assert_eq!(hits[0]["slug"], "drawer-scope");
    assert_eq!(hits[0]["archived"], false);
    assert_eq!(hits[0]["matches"][0]["kind"], "topic");
    assert_eq!(hits[0]["matches"][0]["where"], "frontmatter");
    assert_eq!(hits[1]["slug"], "trace-links-two-hops");
    assert_eq!(hits[1]["archived"], true);
    assert_eq!(hits[1]["matches"][0]["kind"], "ruled-out");
    assert_eq!(hits[1]["matches"][0]["where"], "round-2");
    assert_eq!(
        hits[1]["matches"][0]["text"],
        "**Ruled out**: RichDetailDrawer 加 readOnly 旗標（分支地獄）"
    );
}

#[test]
fn discussion_search_without_keywords_is_invalid_argument() {
    let f = fixture();
    seed_search_records(&f);
    let before = revision(&f);
    let (status, error) =
        protocol_error(request("GET", &f, &f.editor_pat, "discussions/search").call());
    assert_eq!(status, 400);
    assert_eq!(error.reason, ErrorReason::InvalidArgument);
    let (status, error) =
        protocol_error(request("GET", &f, &f.editor_pat, "discussions/search?q=%20").call());
    assert_eq!(status, 400, "an all-blank q is the same refusal");
    assert_eq!(error.reason, ErrorReason::InvalidArgument);
    assert_eq!(revision(&f), before, "a refused search writes nothing");
}

#[test]
fn discussion_search_is_open_to_readers_with_the_editor_shape() {
    let f = fixture();
    seed_search_records(&f);
    let reader: Value = request("GET", &f, &f.reader_pat, "discussions/search?q=drawer")
        .call()
        .expect("reader GET /discussions/search")
        .into_json()
        .expect("JSON body");
    let editor: Value = request("GET", &f, &f.editor_pat, "discussions/search?q=drawer")
        .call()
        .expect("editor GET /discussions/search")
        .into_json()
        .expect("JSON body");
    assert_eq!(reader["hits"].as_array().unwrap().len(), 2);
    assert_eq!(reader, editor, "the read shape does not depend on the role");
}

#[test]
fn discussion_search_reads_extras_from_each_hit_own_record_when_a_slug_is_reused() {
    // review 第一輪 must-fix：slug 封存後可重用；在途與封存同名同時命中時，
    // 封存那筆的 promotedTo／concluded 必須來自封存記錄本身，不得抄在途值。
    let f = fixture();
    seed_discussion(
        &f,
        "reuse",
        false,
        "---\ntopic: Drawer reuse (live)\nslug: reuse\nstatus: open\ncreated: 2026-09-01\n---\n\n\
         ## Context\n\nseed\n\n## Rounds\n\n## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n",
    );
    seed_discussion(
        &f,
        "reuse",
        true,
        "---\ntopic: Drawer reuse (archived)\nslug: reuse\nstatus: promoted\npromoted_to: drawer-cut\ncreated: 2026-07-01\n---\n\n\
         ## Context\n\nseed\n\n## Rounds\n\n## Conclusion\n\n**Decision**: settled\n",
    );
    let body: Value = request("GET", &f, &f.editor_pat, "discussions/search?q=drawer")
        .call()
        .expect("GET /discussions/search")
        .into_json()
        .expect("JSON body");
    let hits = body["hits"].as_array().expect("hits array");
    assert_eq!(hits.len(), 2, "{body}");
    let live = hits.iter().find(|h| h["archived"] == false).expect("live hit");
    let archived = hits.iter().find(|h| h["archived"] == true).expect("archived hit");
    assert_eq!(live["concluded"], false);
    assert!(live.get("promotedTo").is_none(), "live record was never promoted: {live}");
    assert_eq!(archived["concluded"], true, "the archived record's own conclusion: {archived}");
    assert_eq!(archived["promotedTo"], json!(["drawer-cut"]));
}

#[test]
fn discussion_search_route_matches_the_engine_search_over_the_same_records() {
    // command-runtime scenario「discuss search 本機與 server 同語意」：同一組記錄放進
    // 本機 fs store 與 server，引擎函式與端點回同一序列的 hits（slug、欄位、matches）。
    let f = fixture();
    seed_search_records(&f);
    let body: Value = request("GET", &f, &f.editor_pat, "discussions/search?q=drawer")
        .call()
        .expect("GET /discussions/search")
        .into_json()
        .expect("JSON body");

    let root = tempfile::tempdir().expect("tempdir");
    let discussions = root.path().join("openspec").join("discussions");
    std::fs::create_dir_all(discussions.join("archive")).unwrap();
    std::fs::write(discussions.join("drawer-scope.md"), SEARCH_LIVE_RECORD).unwrap();
    std::fs::write(
        discussions.join("archive").join("2026-08-20-trace-links-two-hops.md"),
        SEARCH_ARCHIVED_RECORD,
    )
    .unwrap();
    let engine = speclink_fs::FsStore::new(root.path(), "openspec");
    let hits = speclink_core::discuss::search(&engine, &["drawer".to_string()]).expect("engine search");

    // promotedTo／concluded 是 route 邊緣增欄、path 是各 store 的位置——都不是語意的一部分。
    let strip = |h: &Value| {
        let mut h = h.clone();
        let o = h.as_object_mut().unwrap();
        o.remove("promotedTo");
        o.remove("concluded");
        o.remove("path");
        h
    };
    let route: Vec<Value> = body["hits"].as_array().unwrap().iter().map(strip).collect();
    let engine: Vec<Value> =
        serde_json::to_value(&hits).unwrap().as_array().unwrap().iter().map(strip).collect();
    assert_eq!(route, engine, "the endpoint and the engine agree on order, fields and matches");
}

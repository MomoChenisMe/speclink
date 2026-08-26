//! Changes query routes served as protocol DTOs (reference-server spec「動詞經
//! 正路執行且回 DTO」). The typed client reads a seeded server; the response DTOs
//! carry the fields the stub-tested shapes do, and a missing change is the
//! 404 not_found triple.

use crate::common;

use speclink_remote::client::Client;
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

/// A server seeded with one change (`demo`) and one canonical spec.
fn seeded_base() -> (String, String, String) {
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() },
        "## Why\n\nBecause it is needed.\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        "- [ ] 1.1 First\n- [ ] 1.2 Second\n",
    );
    uow.create(DocumentId::CanonicalSpec { capability: "user-auth".into() }, "# user-auth\n");
    store.commit(uow, Vec::new()).expect("seed commit");

    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    common::seed_demo_registry(&*state.identity);
    let (pat, user) = common::seed_pat(&state.identity, &["demo"]);
    (common::start(state), pat, user)
}

fn client(base: &str, token: &str) -> Client {
    Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        token,
        Some("backend"),
    )
}

#[test]
fn list_status_and_specs_return_typed_dtos() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);

    let list = client.list_changes().expect("list changes");
    assert_eq!(list.changes.len(), 1);
    assert_eq!(list.changes[0].name, "demo");
    assert_eq!(list.changes[0].total_tasks, 2, "the two tasks are counted");

    let status = client.get_change("demo").expect("status");
    assert_eq!(status.change_name, "demo");
    assert_eq!(status.schema_name, "spec-driven");
    assert!(
        status.artifacts.iter().any(|a| a.id == "proposal"),
        "the status lists the proposal artifact"
    );

    let specs = client.list_specs().expect("specs");
    assert!(
        specs.specs.iter().any(|s| s.id == "user-auth"),
        "the canonical spec is listed"
    );
}

#[test]
fn instructions_and_artifact_content_return_typed_dtos() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);

    let apply = client.apply_instructions("demo").expect("apply instructions");
    assert_eq!(apply.change_name, "demo");
    assert_eq!(apply.progress.total, 2, "apply view counts the tasks");
    assert!(apply.context_files.contains_key("tasks"), "apply view names the context files");

    let proposal = client
        .artifact_instructions("demo", "proposal")
        .expect("proposal instructions");
    assert_eq!(proposal.artifact_id, "proposal");
    assert_eq!(proposal.output_path, "proposal.md");

    let content = client.get_artifact("demo", "proposal").expect("artifact content");
    assert_eq!(content.artifact, "proposal");
    assert_eq!(content.content, "## Why\n\nBecause it is needed.\n");
    assert!(content.version > 0, "the artifact version is stamped for If-Match writes");
}

#[test]
fn config_and_whoami_return_typed_dtos() {
    let (base, pat, user_id) = seeded_base();
    let client = client(&base, &pat);

    let config = client.config().expect("config");
    assert_eq!(config.schema, "spec-driven", "the default workflow schema");

    let whoami = client.whoami().expect("whoami");
    assert_eq!(whoami.user.handle, user_id, "whoami reports the PAT owner's id");
    assert_eq!(whoami.user.name, common::SEED_DISPLAY);
    assert!(whoami.repos.iter().any(|r| r.name == "backend"), "whoami lists the repo");
}

#[test]
fn list_carries_started_at_only_for_started_changes() {
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta { change: "started".into() },
        "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-30\nstarted_by: Momo <m@example.com>\n",
    );
    uow.create(DocumentId::ChangeMeta { change: "unstarted".into() }, "schema: spec-driven\n");
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    common::seed_demo_registry(&*state.identity);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let list = client(&base, &pat).list_changes().expect("list changes");
    let started = list.changes.iter().find(|c| c.name == "started").expect("started listed");
    assert_eq!(
        started.started_at.as_deref(),
        Some("2026-07-30"),
        "startedAt comes from the change meta"
    );
    let unstarted = list.changes.iter().find(|c| c.name == "unstarted").expect("unstarted listed");
    assert_eq!(unstarted.started_at, None, "an unstarted change carries no startedAt");

    // camelCase 與缺席即省略走 raw wire 斷言。
    let body: serde_json::Value =
        ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes"))
            .set("Authorization", &format!("Bearer {pat}"))
            .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
            .set("X-Speclink-Repo", "backend")
            .call()
            .expect("GET /changes")
            .into_json()
            .expect("JSON body");
    let items = body["changes"].as_array().expect("changes array");
    let started_item = items.iter().find(|c| c["name"] == "started").expect("started item");
    assert_eq!(started_item["startedAt"], "2026-07-30", "the wire field is camelCase");
    let unstarted_item = items.iter().find(|c| c["name"] == "unstarted").expect("unstarted item");
    assert!(
        unstarted_item.get("startedAt").is_none(),
        "absent startedAt is omitted: {unstarted_item}"
    );
}

#[test]
fn change_status_carries_the_show_composition_meta_fields() {
    // design D4 實作期修正：GET /changes/{name} 增列 show 組合需要的選填
    // meta 欄位（created 成對規則、fromDiscussions、deltaCapabilities）。
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta { change: "demo".into() },
        "schema: spec-driven\ncreated: 2026-07-29\nfrom_discussion: auth-scope\n",
    );
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "proposal.md".into() },
        "## Why\n\nseed\n",
    );
    uow.create(
        DocumentId::ChangeArtifact {
            change: "demo".into(),
            artifact: "specs/auth/spec.md".into(),
        },
        "## ADDED Requirements\n",
    );
    // created 缺席的 change：成對規則使 created 不出現。
    uow.create(DocumentId::ChangeMeta { change: "bare".into() }, "schema: spec-driven\n");
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    common::seed_demo_registry(&*state.identity);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let get = |name: &str| -> serde_json::Value {
        ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes/{name}"))
            .set("Authorization", &format!("Bearer {pat}"))
            .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
            .set("X-Speclink-Repo", "backend")
            .call()
            .expect("GET change")
            .into_json()
            .expect("JSON body")
    };

    let full = get("demo");
    assert_eq!(full["created"], "2026-07-29", "created comes from the meta pair");
    assert_eq!(full["fromDiscussions"][0], "auth-scope");
    assert_eq!(full["deltaCapabilities"][0], "auth");

    let bare = get("bare");
    assert!(
        bare.get("created").is_none(),
        "no created in meta → the pair field is omitted: {bare}"
    );
    assert!(bare.get("fromDiscussions").is_none(), "empty chain is omitted: {bare}");
    assert!(bare.get("deltaCapabilities").is_none(), "no delta specs is omitted: {bare}");
}

#[test]
fn change_status_carries_the_attribution_meta_fields() {
    // remote-read-parity「單 change 讀取回應攜帶 show 組合欄位」擴充：meta 的
    // created_by/created_with/started_at/started_by 上 wire，四欄皆缺即無鍵。
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    // Example「欄位組裝」第 2 列：show 組合三欄「再加」歸屬四欄——同一
    // change 七鍵同時上 wire，故 meta 帶鏈、附一份 delta spec。
    uow.create(
        DocumentId::ChangeMeta { change: "attributed".into() },
        "schema: spec-driven\ncreated: 2026-07-29\ncreated_by: Demo <d@e.com>\ncreated_with: claude-code\nstarted_at: 2026-08-25T00:00:00Z\nstarted_by: Demo <d@e.com>\nfrom_discussion: auth-scope\n",
    );
    uow.create(
        DocumentId::ChangeArtifact {
            change: "attributed".into(),
            artifact: "specs/auth/spec.md".into(),
        },
        "## ADDED Requirements\n",
    );
    uow.create(DocumentId::ChangeMeta { change: "bare".into() }, "schema: spec-driven\n");
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    common::seed_demo_registry(&*state.identity);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let get = |name: &str| -> serde_json::Value {
        ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes/{name}"))
            .set("Authorization", &format!("Bearer {pat}"))
            .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
            .set("X-Speclink-Repo", "backend")
            .call()
            .expect("GET change")
            .into_json()
            .expect("JSON body")
    };

    let full = get("attributed");
    assert_eq!(full["created"], "2026-07-29", "show composition trio rides along");
    assert_eq!(full["fromDiscussions"], serde_json::json!(["auth-scope"]));
    assert_eq!(full["deltaCapabilities"], serde_json::json!(["auth"]));
    assert_eq!(full["createdBy"], "Demo <d@e.com>");
    assert_eq!(full["createdWith"], "claude-code");
    assert_eq!(full["startedAt"], "2026-08-25T00:00:00Z");
    assert_eq!(full["startedBy"], "Demo <d@e.com>");

    let bare = get("bare");
    for key in ["createdBy", "createdWith", "startedAt", "startedBy"] {
        assert!(
            bare.get(key).is_none(),
            "absent {key} is omitted: {bare}"
        );
    }
}

#[test]
fn list_carries_the_creator_and_discussion_fields() {
    // remote-read-parity「變更清單回應攜帶建立者與來源討論欄位」：沿 startedAt
    // 的逐筆 meta 組裝路徑補 createdBy/created/fromDiscussions，缺席即無鍵。
    let store = MemoryStore::new();
    let scope = Scope::new(ProjectId::new("demo"), RepoId::new("backend"));
    let mut uow = store
        .begin_unit_of_work(
            &scope,
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta { change: "attributed".into() },
        "schema: spec-driven\ncreated: 2026-07-29\ncreated_by: Demo <d@e.com>\nfrom_discussion: auth-scope\n",
    );
    uow.create(DocumentId::ChangeMeta { change: "bare".into() }, "schema: spec-driven\n");
    // meta 解析失敗的 change：metaError 容錯路徑三欄缺席、清單不失敗。
    uow.create(
        DocumentId::ChangeMeta { change: "corrupt".into() },
        "schema: [unterminated\n",
    );
    store.commit(uow, Vec::new()).expect("seed commit");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(store),
        identity: common::empty_identity(),
        config: Arc::new(common::demo_config()),
    };
    common::seed_demo_registry(&*state.identity);
    let (pat, _user) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);

    let body: serde_json::Value =
        ureq::get(&format!("{base}/api/speclink/v1/projects/demo/changes"))
            .set("Authorization", &format!("Bearer {pat}"))
            .set("X-Speclink-Api-Version", speclink_protocol::API_VERSION)
            .set("X-Speclink-Repo", "backend")
            .call()
            .expect("GET /changes")
            .into_json()
            .expect("JSON body");
    let items = body["changes"].as_array().expect("changes array");
    let attributed = items.iter().find(|c| c["name"] == "attributed").expect("attributed item");
    assert_eq!(attributed["createdBy"], "Demo <d@e.com>");
    assert_eq!(attributed["created"], "2026-07-29");
    assert_eq!(attributed["fromDiscussions"], serde_json::json!(["auth-scope"]));
    let bare = items.iter().find(|c| c["name"] == "bare").expect("bare item");
    for key in ["createdBy", "created", "fromDiscussions"] {
        assert!(
            bare.get(key).is_none(),
            "absent {key} is omitted: {bare}"
        );
    }
    let corrupt = items.iter().find(|c| c["name"] == "corrupt").expect("corrupt item");
    assert!(corrupt.get("metaError").is_some(), "engine diagnoses the bad meta: {corrupt}");
    for key in ["createdBy", "created", "fromDiscussions"] {
        assert!(
            corrupt.get(key).is_none(),
            "bad meta keeps {key} absent without failing the list: {corrupt}"
        );
    }
}

#[test]
fn a_missing_change_is_the_404_not_found_triple() {
    let (base, pat, _user) = seeded_base();
    let client = client(&base, &pat);
    let err = client.get_change("ghost").expect_err("a missing change is an error");
    assert_eq!(
        err.reason.as_deref(),
        Some("not_found"),
        "the error is the not_found triple: {err:?}"
    );
}

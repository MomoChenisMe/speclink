//! 合併後的系統 view model（server-admin「管理 browser API 提供最小且完整的頁面
//! view model」）。`/admin/data` 與 `/admin/system` 兩份 view model 併為一份：單次
//! 回應同時給執行環境版本、儲存狀態與各範圍保留中的事件筆數、可匯出範圍清單，SPA 不再
//! 為其中任何一組打第二支 API。既有欄位名稱一律不重新命名，新欄位以 camelCase 輸出。
//! 契約版本、等級、能力與遷移可用性是舊儲存層的概念，回應裡沒有它們。

use crate::common;

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use std::sync::Arc;

const PASSWORD: &str = "pw-correct-horse";

fn seed_user(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> String {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("User <{email}>"),
            memberships: vec!["demo".to_string()],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    identity.accept_invitation(&token, PASSWORD).expect("accept")
}

/// The four fields of the old store layer the view no longer has.
const GONE: [&str; 4] = ["storeContractVersion", "storeLevel", "storeCapabilities", "migrateAvailable"];

fn start_over(store: SharedStore) -> String {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    seed_user(&identity, "admin@example.com", true);
    let state = AppState::new(
        store,
        identity,
        Arc::new(common::demo_config()),
        common::detached_events(),
    );
    common::start(state)
}

fn start_healthy() -> String {
    start_over(common::TestStore::memory().shared())
}

/// A store whose `health()` reports an error while identity stays readable.
fn start_crashed() -> String {
    let store = common::TestStore::memory();
    store.break_down();
    start_over(store.shared())
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/api/speclink/v1/web/login"))
        .set("Origin", "http://127.0.0.1")
        .send_json(json!({ "email": "admin@example.com", "password": PASSWORD }))
        .expect("login");
    resp.header("set-cookie")
        .and_then(|c| c.split(';').next())
        .and_then(|c| c.trim().strip_prefix("speclink_session="))
        .expect("session cookie")
        .to_string()
}

fn get(base: &str, path: &str, cookie: &str) -> (u16, Value) {
    let result = agent()
        .get(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={cookie}"))
        .call();
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_json().unwrap_or(Value::Null))
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_json().unwrap_or(Value::Null)),
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn the_system_view_carries_all_three_groups_in_one_response() {
    let base = start_healthy();
    let cookie = login(&base);
    let (status, body) = get(&base, "/api/speclink/v1/web/admin/system", &cookie);
    assert_eq!(status, 200, "the system view loads: {body}");
    let data = &body["data"];

    // 1. 執行環境：引擎與 API 版本、識別資料結構版本（既有欄位名稱不變）。
    assert!(data["engineVersion"].is_string(), "engineVersion: {data}");
    assert!(data["apiVersion"].is_string(), "apiVersion: {data}");
    assert!(data["identitySchemaVersion"].is_number(), "identitySchemaVersion: {data}");

    // 2. 儲存狀態：種類、是否可用，與各範圍保留中的事件筆數（既有欄位名稱不變）。
    assert_eq!(data["storeDriver"], json!("memory"), "storeDriver: {data}");
    assert_eq!(data["storeHealthy"], json!(true), "storeHealthy: {data}");
    let backlogs = data["outboxBacklogs"].as_array().expect("outboxBacklogs array");
    assert_eq!(backlogs[0]["project"], json!("demo"));
    assert_eq!(backlogs[0]["repo"], json!("backend"));
    assert_eq!(backlogs[0]["backlog"], json!(0), "a number: no event yet");

    // 契約版本、等級、能力與遷移可用性已移除；每個鍵都是 camelCase。
    for gone in GONE {
        assert!(data.get(gone).is_none(), "{gone} is gone: {data}");
    }
    let fields = data.as_object().expect("an object");
    assert!(fields.keys().all(|key| !key.contains('_')), "camelCase keys: {data}");

    // 3. 原「資料操作」併入的一組：可匯出範圍清單。
    let scopes = data["scopes"].as_array().expect("scopes array");
    assert_eq!(scopes[0]["project"], json!("demo"));
    assert_eq!(scopes[0]["repo"], json!("backend"));
    assert_eq!(
        scopes[0]["exportPath"],
        json!("/admin/data/export/demo/backend"),
        "the export download path is unchanged"
    );
}

#[test]
fn the_backlog_of_a_scope_is_the_number_of_events_the_store_keeps() {
    let store = common::TestStore::memory();
    store.put(
        "demo",
        "backend",
        &[
            ("changes/demo/.meta.yaml", "schema: spec-driven\n"),
            ("changes/demo/tasks.md", "- [ ] 1.1 First\n- [ ] 1.2 Second\n- [ ] 1.3 Third\n"),
        ],
    );
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    seed_user(&identity, "admin@example.com", true);
    let state = AppState::new(
        store.shared(),
        identity,
        Arc::new(common::demo_config()),
        common::detached_events(),
    );
    let (pat, _) = common::seed_pat(&state.identity, &["demo"]);
    let base = common::start(state);
    let writer = speclink_remote::client::Client::new(
        &format!("{base}/api/speclink/v1/projects/demo"),
        &pat,
        Some("backend"),
    );
    for task in ["1", "2", "3"] {
        writer.task_done("demo", task, &[], None).expect("task done");
    }
    let kept = store.events("demo", "backend").len();
    assert_eq!(kept, 3, "three events are in the scope's log");

    let cookie = login(&base);
    let (_status, body) = get(&base, "/api/speclink/v1/web/admin/system", &cookie);
    assert_eq!(body["data"]["outboxBacklogs"][0]["backlog"], json!(kept), "{body}");
}

#[test]
fn the_standalone_data_view_model_is_gone() {
    let base = start_healthy();
    let cookie = login(&base);
    let (status, _body) = get(&base, "/api/speclink/v1/web/admin/data", &cookie);
    assert_eq!(status, 404, "the data view model no longer exists");
}

#[test]
fn an_unhealthy_store_degrades_the_system_view_without_failing_it() {
    let base = start_crashed();
    let cookie = login(&base);
    let (status, body) = get(&base, "/api/speclink/v1/web/admin/system", &cookie);
    assert_eq!(status, 200, "a store failure degrades rather than 500s: {body}");
    let data = &body["data"];
    assert_eq!(data["storeHealthy"], json!(false));
    assert!(data["storeHealthError"].is_string(), "with a public error");
    for gone in GONE {
        assert!(data.get(gone).is_none(), "{gone} is gone: {data}");
    }
    // 讀不到事件筆數的範圍回報空值，其餘欄位照常。
    let backlogs = data["outboxBacklogs"].as_array().expect("outboxBacklogs array");
    assert_eq!(backlogs[0]["project"], json!("demo"));
    assert!(backlogs[0]["backlog"].is_null(), "an unreadable backlog is null, not a failure: {data}");
    // 識別側資料仍可讀，範圍清單來自 registry。
    assert!(data["identitySchemaVersion"].is_number());
    assert!(!data["scopes"].as_array().expect("scopes").is_empty());
}

#[test]
fn the_system_view_carries_no_secret() {
    let base = start_healthy();
    let cookie = login(&base);
    let (_s, body) = get(&base, "/api/speclink/v1/web/admin/system", &cookie);
    let raw = body.to_string();
    for forbidden in ["hash", "password", "secret", "token", "refresh"] {
        assert!(!raw.contains(forbidden), "the system view must not carry `{forbidden}`: {raw}");
    }
}

#[test]
fn the_export_download_still_works_after_the_merge() {
    // 合併的是 view model，不是匯出下載端點——舊路徑仍要能下載。
    let base = start_healthy();
    let cookie = login(&base);
    let resp = agent()
        .get(&format!("{base}/admin/data/export/demo/backend"))
        .set("Cookie", &format!("speclink_session={cookie}"))
        .call()
        .expect("export download");
    assert_eq!(resp.status(), 200, "the scope export bundle still downloads");
}

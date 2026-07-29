//! CredentialStore 與 connection registry 的契約測試（design 決策 2「credential
//! 唯一落點 OS Keychain，Rust 側進出」、決策 4「connection registry 檔案形狀與
//! 位置」；規格「connection registry 不含 secret 且跨重啟保留」）。
//!
//! CI 無 headless Keychain 可用——trait 注入是唯一可測形狀：此處測 in-memory
//! 實作的逐 origin＋種類語意；keyring 生產實作走同一 trait，於手動驗收
//! （security find-generic-password）確認。registry 檔的序列化欄位全集在此
//! 釘死：無任何 token 欄位是規格鐵律。

use speclink_desktop_lib::connections::{
    read_registry, upsert_connection, write_registry, ConnectionEntry,
};
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};

// --- CredentialStore：逐 origin＋種類（refresh/pat）語意 ---

#[test]
fn credentials_are_keyed_by_origin_and_kind() {
    let store = MemoryCredentialStore::new();
    store
        .set("http://a.example:8080", CredentialKind::Refresh, "spk_rt_a")
        .expect("set");

    // 同 origin 不同種類、不同 origin 同種類，互不可見。
    assert_eq!(
        store
            .get("http://a.example:8080", CredentialKind::Refresh)
            .expect("get")
            .as_deref(),
        Some("spk_rt_a")
    );
    assert_eq!(
        store
            .get("http://a.example:8080", CredentialKind::Pat)
            .expect("get"),
        None
    );
    assert_eq!(
        store
            .get("http://b.example:8080", CredentialKind::Refresh)
            .expect("get"),
        None
    );
}

#[test]
fn set_overwrites_the_previous_secret_for_the_same_slot() {
    let store = MemoryCredentialStore::new();
    store
        .set("http://a.example", CredentialKind::Refresh, "spk_rt_old")
        .expect("set");
    store
        .set("http://a.example", CredentialKind::Refresh, "spk_rt_new")
        .expect("overwrite");
    assert_eq!(
        store
            .get("http://a.example", CredentialKind::Refresh)
            .expect("get")
            .as_deref(),
        Some("spk_rt_new"),
        "rotation 回寫後最新 refresh credential 生效"
    );
}

#[test]
fn delete_removes_the_slot_and_is_idempotent() {
    let store = MemoryCredentialStore::new();
    store
        .set("http://a.example", CredentialKind::Pat, "spk_pat_x")
        .expect("set");
    store
        .delete("http://a.example", CredentialKind::Pat)
        .expect("delete");
    assert_eq!(
        store
            .get("http://a.example", CredentialKind::Pat)
            .expect("get"),
        None
    );
    // 登出時 Keychain 已空不該是錯誤——刪除不存在的 entry 冪等成功。
    store
        .delete("http://a.example", CredentialKind::Pat)
        .expect("double delete is idempotent");
}

// --- registry：序列化欄位全集，無任何 token 欄位 ---

#[test]
fn registry_round_trips_and_serializes_exactly_the_secret_free_field_set() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("connections.json");

    let entries = vec![
        ConnectionEntry {
            id: "conn_a".to_string(),
            origin: "http://a.example:8080".to_string(),
            name: "工作站 A".to_string(),
            last_actor_display: Some("Dev <dev@example.com>".to_string()),
        },
        ConnectionEntry {
            id: "conn_b".to_string(),
            origin: "http://b.example:8080".to_string(),
            name: "工作站 B".to_string(),
            last_actor_display: None,
        },
    ];
    write_registry(&path, &entries).expect("write");
    let back = read_registry(&path);
    assert_eq!(back, entries, "registry 跨重啟保留（往返不變）");

    // 欄位全集釘死：id、origin、name、lastActorDisplay——沒有其他欄位，
    // 特別是沒有任何 token/credential 欄位。
    let json: serde_json::Value = serde_json::to_value(&entries[0]).expect("serialize an entry");
    let keys: Vec<&str> = json
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        ["id", "lastActorDisplay", "name", "origin"],
        "欄位全集（依序排序後）：{json}"
    );

    // 身分未知時該欄位整個省略，而不是落 null。
    let bare: serde_json::Value = serde_json::to_value(&entries[1]).expect("serialize");
    let bare_keys: Vec<&str> = bare
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(bare_keys, ["id", "name", "origin"]);

    // 檔案內容整體掃一遍：不含任何 secret 樣式字面。
    let raw = std::fs::read_to_string(&path).expect("read raw");
    for needle in ["token", "secret", "spk_rt_", "spk_at_", "spk_pat_"] {
        assert!(!raw.contains(needle), "registry 檔不含 {needle}：{raw}");
    }
}

#[test]
fn upserting_the_same_origin_updates_the_display_name_in_place() {
    let mut entries = Vec::new();
    let first = upsert_connection(&mut entries, "http://localhost:8080/", "本地").expect("add");
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].origin, "http://localhost:8080",
        "baseUrl 正規化為 origin（去尾斜線/路徑）"
    );
    assert!(!entries[0].id.is_empty(), "條目有識別 id");

    // 同 origin（即使寫法不同）重複新增＝更新顯示名，不長第二條，id 穩定。
    let second = upsert_connection(&mut entries, "HTTP://localhost:8080/some/path?q=1", "改名")
        .expect("upsert");
    assert_eq!(entries.len(), 1, "一 origin 一條目");
    assert_eq!(entries[0].name, "改名");
    assert_eq!(first, second, "更新顯示名不換 id");

    // 不同 origin 才新增第二條。
    upsert_connection(&mut entries, "http://other.example:9000", "另一台").expect("add other");
    assert_eq!(entries.len(), 2);
}

#[test]
fn an_invalid_base_url_is_refused() {
    let mut entries = Vec::new();
    upsert_connection(&mut entries, "not-a-url", "壞").expect_err("無 scheme 的輸入被拒");
    assert!(entries.is_empty());
}

#[test]
fn a_corrupt_registry_file_reads_as_an_empty_list() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("connections.json");
    std::fs::write(&path, "{{{ not json").expect("write corrupt bytes");
    assert!(read_registry(&path).is_empty(), "壞 JSON 歸零清單、不崩潰");
    // 檔案不存在同樣是空清單。
    assert!(read_registry(&dir.path().join("missing.json")).is_empty());
}

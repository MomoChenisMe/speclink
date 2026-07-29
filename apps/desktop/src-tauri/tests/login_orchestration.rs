//! 登入／登出編排契約（design 決策 5「device login 編排與瀏覽器開啟」、決策 3
//! 「登入前探測＝直接 POST /auth/device」、決策 6「登出與移除語意」；規格
//! 「device login 預設與 PAT fallback」「登出撤銷與移除連帶清理」）。
//!
//! 假瀏覽器開啟器＋in-process speclink-server（memory identity）＋in-memory
//! CredentialStore：核准/拒絕由假開啟器從 verification URL 的 user_code 預填
//! 參數取碼、直接對 identity store 動作——模擬 /activate 頁的人。

use chrono::{Duration, Utc};
use speclink_desktop_lib::connections::{
    device_login_observe, device_login_start, logout, pat_login, read_registry, refresh_connection,
    upsert_connection, write_registry, DeviceLoginObservation, DeviceLoginStart,
};
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const DISPLAY: &str = "Dev <dev@example.com>";

// --- in-process server harness（memory identity）＋registry 檔 ---

struct Harness {
    origin: String,
    identity: Arc<IdentitySqlite>,
    user_id: String,
    registry: PathBuf,
    _dir: tempfile::TempDir,
}

fn harness() -> Harness {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("memory identity"));
    let invite = identity
        .create_invitation(NewInvitation {
            email: "dev@example.com".to_string(),
            display: DISPLAY.to_string(),
            memberships: vec![],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity
        .accept_invitation(&invite, "pw-correct-horse")
        .expect("accept");

    let store: speclink_server::state::SharedStore =
        Arc::new(speclink_store::memory::MemoryStore::new());
    let state = AppState {
        events: EventHub::new(store.clone(), EventSettings::default()),
        store,
        config: Arc::new(ServerConfig {
            store: StoreConfig::Memory,
            identity: IdentityConfig::Memory,
            public_url: "http://127.0.0.1".to_string(),
            events: EventSettings::default(),
        }),
        identity: identity.clone(),
    };

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    listener.set_nonblocking(true).expect("nonblocking");
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
            axum::serve(listener, speclink_server::app::router(state))
                .await
                .expect("serve");
        });
    });

    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let origin = format!("http://{addr}");
    let mut entries = Vec::new();
    upsert_connection(&mut entries, &origin, "本地").expect("seed entry");
    write_registry(&registry, &entries).expect("write registry");

    Harness {
        origin,
        identity,
        user_id,
        registry,
        _dir: dir,
    }
}

/// 從 verification URL 的 user_code 預填參數取碼。
fn code_of(url: &str) -> String {
    url.split("user_code=")
        .nth(1)
        .expect("URL 帶 user_code 預填參數")
        .to_string()
}

/// 兩段編排走到終態（design 決策二）：啟動段可能靜默完成；否則依 server 指示的
/// 間隔做單次觀測，直到終態或有效期限耗盡——即前端 store 排程在測試裡的等價物。
#[derive(Debug)]
enum Settled {
    LoggedIn {
        display: String,
        access_token: String,
    },
    Unsupported,
    Denied,
    Expired,
}

fn login_until_settled(
    origin: &str,
    credentials: &dyn CredentialStore,
    registry: &std::path::Path,
    open_browser: &dyn Fn(&str) -> Result<(), String>,
) -> Result<Settled, String> {
    let auth = match device_login_start(origin, credentials, registry, open_browser)? {
        DeviceLoginStart::LoggedIn {
            display,
            access_token,
        } => {
            return Ok(Settled::LoggedIn {
                display,
                access_token,
            })
        }
        DeviceLoginStart::Unsupported => return Ok(Settled::Unsupported),
        DeviceLoginStart::AwaitingApproval(auth) => auth,
    };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(auth.expires_in);
    let mut interval = std::time::Duration::from_secs(auth.interval.max(1));
    loop {
        match device_login_observe(origin, &auth.device_code, credentials, registry)? {
            DeviceLoginObservation::LoggedIn {
                display,
                access_token,
            } => {
                return Ok(Settled::LoggedIn {
                    display,
                    access_token,
                })
            }
            DeviceLoginObservation::Denied => return Ok(Settled::Denied),
            DeviceLoginObservation::Expired => return Ok(Settled::Expired),
            DeviceLoginObservation::Pending { slow_down } => {
                if std::time::Instant::now() >= deadline {
                    return Ok(Settled::Expired);
                }
                if slow_down {
                    interval *= 2;
                }
                std::thread::sleep(interval);
            }
        }
    }
}

// --- 分段編排：啟動段（design 決策二） ---

#[test]
fn start_with_a_live_refresh_credential_logs_in_without_opening_the_browser() {
    // 靜默 refresh 快路徑留在啟動段（規格「rotation 後舊 credential 失效仍可用」）：
    // 「重啟」＝記憶體 access token 消失、只剩 Keychain 的 refresh credential，此時
    // 按登入應直接換新回已登入，不再開瀏覽器、不進等待授權面。
    let h = harness();
    let store = MemoryCredentialStore::new();
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        assert!(identity
            .approve_device(&code_of(url), &user_id)
            .expect("approve"));
        Ok(())
    };
    login_until_settled(&h.origin, &store, &h.registry, &opener).expect("首次登入");

    let no_browser = |_url: &str| -> Result<(), String> { panic!("靜默重登入不得開瀏覽器") };
    let start = device_login_start(&h.origin, &store, &h.registry, &no_browser).expect("start");
    let display = match start {
        DeviceLoginStart::LoggedIn { display, .. } => display,
        other => panic!("expected LoggedIn, got {other:?}"),
    };
    assert_eq!(display, DISPLAY, "靜默換新後身分即刻可讀");
}

#[test]
fn start_without_a_credential_awaits_approval_with_the_codes_and_poll_metadata() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = opened.clone();
    // 啟動段只開瀏覽器、不代替使用者核准——授權停在 pending。
    let opener = move |url: &str| {
        sink.lock().unwrap().push(url.to_string());
        Ok(())
    };

    let start = device_login_start(&h.origin, &store, &h.registry, &opener).expect("start");
    let auth = match start {
        DeviceLoginStart::AwaitingApproval(auth) => auth,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };
    assert!(!auth.device_code.is_empty(), "後續單次觀測的把手");
    assert!(!auth.user_code.is_empty(), "等待授權面要顯示的裝置碼");
    assert!(
        auth.verification_uri.contains("/activate"),
        "等待授權面要顯示的驗證網址：{}",
        auth.verification_uri
    );
    assert!(auth.expires_in > 0, "倒數用的有效期限（秒）");
    assert!(auth.interval > 0, "排程用的輪詢間隔（秒）");

    // 瀏覽器開的是同一個驗證網址，且帶 user_code 預填。
    let urls = opened.lock().unwrap();
    assert_eq!(urls.len(), 1);
    assert_eq!(
        code_of(&urls[0]),
        auth.user_code,
        "預填的正是等待授權面顯示的碼：{}",
        urls[0]
    );
    assert!(urls[0].starts_with(&auth.verification_uri), "{}", urls[0]);

    // 尚未核准前不留任何 credential、不寫身分。
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None
    );
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- 分段編排：單次觀測段（design 決策二） ---

#[test]
fn observing_an_undecided_request_reports_pending_and_then_slow_down() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opener = |_url: &str| Ok(());
    let auth = match device_login_start(&h.origin, &store, &h.registry, &opener).expect("start") {
        DeviceLoginStart::AwaitingApproval(auth) => auth,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };

    let first = device_login_observe(&h.origin, &auth.device_code, &store, &h.registry)
        .expect("observe");
    assert!(
        matches!(first, DeviceLoginObservation::Pending { slow_down: false }),
        "尚未決定＝pending，不是錯誤：{first:?}"
    );
    // 立刻再觀測一次比 server 指示的間隔更快——server 要求放慢。
    let second = device_login_observe(&h.origin, &auth.device_code, &store, &h.registry)
        .expect("observe");
    assert!(
        matches!(second, DeviceLoginObservation::Pending { slow_down: true }),
        "過快的觀測回 slow_down 讓排程加大間隔：{second:?}"
    );
}

#[test]
fn observing_an_approved_request_stores_the_credential_and_records_the_identity() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = opened.clone();
    let opener = move |url: &str| {
        sink.lock().unwrap().push(url.to_string());
        Ok(())
    };
    let auth = match device_login_start(&h.origin, &store, &h.registry, &opener).expect("start") {
        DeviceLoginStart::AwaitingApproval(auth) => auth,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };
    // 使用者於 /activate 核准（等同瀏覽器那一頭）。
    assert!(h
        .identity
        .approve_device(&auth.user_code, &h.user_id)
        .expect("approve"));

    let observation = device_login_observe(&h.origin, &auth.device_code, &store, &h.registry)
        .expect("observe");
    let display = match observation {
        DeviceLoginObservation::LoggedIn { display, .. } => display,
        other => panic!("expected LoggedIn, got {other:?}"),
    };
    assert_eq!(display, DISPLAY, "/auth/whoami 的身分顯示名回來了");
    let rt = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("credential");
    assert!(rt.starts_with("spk_rt_"), "存的是 refresh credential：{rt}");
    assert_eq!(
        read_registry(&h.registry)[0].last_actor_display.as_deref(),
        Some(DISPLAY),
        "身分寫回 registry"
    );
}

#[test]
fn observing_a_denied_request_is_terminal_and_leaves_no_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opener = |_url: &str| Ok(());
    let auth = match device_login_start(&h.origin, &store, &h.registry, &opener).expect("start") {
        DeviceLoginStart::AwaitingApproval(auth) => auth,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };
    assert!(h
        .identity
        .deny_device(&auth.user_code, &h.user_id)
        .expect("deny"));

    let observation = device_login_observe(&h.origin, &auth.device_code, &store, &h.registry)
        .expect("observe");
    assert!(
        matches!(observation, DeviceLoginObservation::Denied),
        "拒絕是可讀終態，不是 Err：{observation:?}"
    );
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None,
        "不留任何 credential"
    );
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

#[test]
fn observing_an_expired_request_is_terminal_and_leaves_no_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    // 已逾期的授權請求（有效期限落在過去）——等同倒數歸零後才觀測。
    let auth = h
        .identity
        .create_device_authorization(Duration::seconds(0), Duration::seconds(-1))
        .expect("expired authorization");

    let observation = device_login_observe(&h.origin, &auth.device_code, &store, &h.registry)
        .expect("observe");
    assert!(
        matches!(observation, DeviceLoginObservation::Expired),
        "逾期是可讀終態：{observation:?}"
    );
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None,
        "不留任何 credential"
    );
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- device login 全鏈（兩段組合） ---

#[test]
fn device_login_opens_the_browser_approves_and_lands_credential_and_identity() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let sink = opened.clone();
    let opener = move |url: &str| {
        sink.lock().unwrap().push(url.to_string());
        // 模擬使用者在 /activate 核准。
        assert!(identity
            .approve_device(&code_of(url), &user_id)
            .expect("approve"));
        Ok(())
    };

    let outcome = login_until_settled(&h.origin, &store, &h.registry, &opener).expect("device login");
    let display = match outcome {
        Settled::LoggedIn { display, .. } => display,
        other => panic!("expected LoggedIn, got {other:?}"),
    };
    assert_eq!(display, DISPLAY, "/auth/whoami 的身分顯示名回來了");

    // 開啟器收到 verification URL（指向 /activate、帶 user_code 預填）。
    let urls = opened.lock().unwrap();
    assert_eq!(urls.len(), 1);
    assert!(urls[0].contains("/activate"), "指向核准頁：{}", urls[0]);

    // refresh credential 入 store（且是 refresh、不是 pat）。
    let rt = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("credential");
    assert!(rt.starts_with("spk_rt_"), "存的是 refresh credential：{rt}");
    assert_eq!(
        store.get(&h.origin, CredentialKind::Pat).expect("get"),
        None
    );

    // 身分顯示名寫回 registry。
    let entries = read_registry(&h.registry);
    assert_eq!(entries[0].last_actor_display.as_deref(), Some(DISPLAY));
}

#[test]
fn a_browser_denial_reports_denied_and_leaves_no_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        assert!(identity.deny_device(&code_of(url), &user_id).expect("deny"));
        Ok(())
    };

    let outcome = login_until_settled(&h.origin, &store, &h.registry, &opener).expect("orchestration");
    assert!(
        matches!(outcome, Settled::Denied),
        "拒絕是可讀狀態，不是 Err"
    );
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None,
        "不留任何 credential"
    );
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- 探測語意（決策 3）：404 → PAT fallback 訊號；5xx → 連線錯誤 ---

/// 以固定狀態回應所有請求的假 server——探測案例需要「沒有 device 端點」的對象。
fn fixed_server(status: u16) -> (Arc<tiny_http::Server>, String) {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let _ = req.respond(tiny_http::Response::from_string("").with_status_code(status));
        }
    });
    (server, format!("http://127.0.0.1:{port}"))
}

#[test]
fn a_404_probe_reports_unsupported_without_opening_the_browser() {
    let (server, origin) = fixed_server(404);
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let opener = |_url: &str| -> Result<(), String> { panic!("不支援時不得開瀏覽器") };

    let outcome =
        login_until_settled(&origin, &store, &registry, &opener).expect("probe miss 是結果不是錯誤");
    assert!(
        matches!(outcome, Settled::Unsupported),
        "404 是明確的 PAT fallback 訊號"
    );
    server.unblock();
}

#[test]
fn a_5xx_probe_is_a_connection_error_not_a_fallback() {
    let (server, origin) = fixed_server(503);
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let opener = |_url: &str| -> Result<(), String> { panic!("連線錯誤時不得開瀏覽器") };

    login_until_settled(&origin, &store, &registry, &opener)
        .expect_err("5xx 是錯誤、絕不進 PAT fallback");
    assert_eq!(
        store.get(&origin, CredentialKind::Refresh).expect("get"),
        None
    );
    server.unblock();
}

// --- pat_login：/auth/whoami 驗證後才入 store ---

#[test]
fn pat_login_validates_via_whoami_before_storing() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let (_, pat) = h
        .identity
        .create_pat(&h.user_id, "desktop", None)
        .expect("pat");

    let display = pat_login(&h.origin, &pat, &store, &h.registry).expect("pat login");
    assert_eq!(display, DISPLAY);
    assert_eq!(
        store
            .get(&h.origin, CredentialKind::Pat)
            .expect("get")
            .as_deref(),
        Some(pat.as_str()),
        "驗證通過後 PAT 才入 store"
    );
    assert_eq!(
        read_registry(&h.registry)[0].last_actor_display.as_deref(),
        Some(DISPLAY)
    );
}

#[test]
fn an_invalid_pat_is_refused_and_never_stored() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    pat_login(&h.origin, "spk_pat_nope", &store, &h.registry).expect_err("無效 PAT 拒絕");
    assert_eq!(
        store.get(&h.origin, CredentialKind::Pat).expect("get"),
        None,
        "無效 PAT 不落任何盤"
    );
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- rotation：新 refresh credential 覆寫 ---

#[test]
fn rotation_overwrites_the_stored_refresh_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        assert!(identity
            .approve_device(&code_of(url), &user_id)
            .expect("approve"));
        Ok(())
    };
    login_until_settled(&h.origin, &store, &h.registry, &opener).expect("login");
    let old_rt = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("rt");

    let access = refresh_connection(&h.origin, &store).expect("rotation");
    assert!(
        access.starts_with("spk_at_"),
        "rotation 換得新 access token"
    );
    let new_rt = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("rt");
    assert_ne!(new_rt, old_rt, "新 refresh credential 覆寫 Keychain slot");

    // 再 rotation 一次成功——證明 store 裡是活的最新 credential（舊 rt 重用
    // 會被 server 以 family revocation 拒絕）。
    refresh_connection(&h.origin, &store).expect("再次 rotation 走最新 credential");
}

#[test]
fn a_transient_server_failure_never_discards_a_live_refresh_credential() {
    // 決策 3 的同一原則落在 rotation 上：5xx／不可達是連線錯誤，不是
    //「credential 已失效」的語意訊號——抹掉有效 credential 會讓 server 一次
    // 短暫抖動就逼使用者重開瀏覽器核准（並牴觸規格「rotation 後舊 credential
    // 失效仍可用」的免重登入承諾）。
    let (server, origin) = fixed_server(503);
    let store = MemoryCredentialStore::new();
    store
        .set(&origin, CredentialKind::Refresh, "spk_rt_live")
        .expect("set");
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let opener = |_url: &str| -> Result<(), String> { panic!("連線錯誤時不得開瀏覽器") };

    login_until_settled(&origin, &store, &registry, &opener).expect_err("5xx 是連線錯誤");
    assert_eq!(
        store
            .get(&origin, CredentialKind::Refresh)
            .expect("get")
            .as_deref(),
        Some("spk_rt_live"),
        "server 暫時不可達不得抹除有效 credential"
    );
    server.unblock();
}

#[test]
fn a_rejected_refresh_credential_is_cleared_and_the_full_device_flow_takes_over() {
    // 對照組：server 明確拒絕（permission_denied）才是「credential 已死」的
    // 語意訊號——清掉殘骸、走完整 device flow 重新核准。
    let h = harness();
    let store = MemoryCredentialStore::new();
    store
        .set(&h.origin, CredentialKind::Refresh, "spk_rt_dead")
        .expect("set");
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        assert!(identity
            .approve_device(&code_of(url), &user_id)
            .expect("approve"));
        Ok(())
    };

    let outcome = login_until_settled(&h.origin, &store, &h.registry, &opener).expect("device login");
    assert!(
        matches!(outcome, Settled::LoggedIn { .. }),
        "殘骸不擋重新核准"
    );
    let rt = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("credential");
    assert_ne!(
        rt, "spk_rt_dead",
        "殘骸已被新核准的 refresh credential 取代"
    );
}

// --- logout（決策 6）：盡力撤銷＋必刪本機 ---

#[test]
fn logout_revokes_the_family_and_clears_local_state() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        assert!(identity
            .approve_device(&code_of(url), &user_id)
            .expect("approve"));
        Ok(())
    };
    let outcome = login_until_settled(&h.origin, &store, &h.registry, &opener).expect("login");
    let access = match outcome {
        Settled::LoggedIn { access_token, .. } => access_token,
        other => panic!("expected LoggedIn, got {other:?}"),
    };

    let result = logout(&h.origin, &store, &h.registry).expect("logout");
    assert!(
        result.revoked_on_server,
        "refresh 走 /auth/revoke 撤了 device family"
    );
    assert!(!result.pat_notice);
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None,
        "Keychain entry 已刪"
    );
    assert!(
        read_registry(&h.registry)[0].last_actor_display.is_none(),
        "registry 身分已清"
    );
    assert!(
        h.identity
            .authenticate_access_token(&access)
            .expect("authenticate")
            .is_none(),
        "server 端 family 已撤——access token 一併失效"
    );
}

#[test]
fn logout_against_an_unreachable_server_still_cleans_up_locally() {
    let origin = "http://127.0.0.1:1".to_string(); // 無人監聽——server 不可達
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let mut entries = Vec::new();
    upsert_connection(&mut entries, &origin, "離線").expect("seed");
    entries[0].last_actor_display = Some(DISPLAY.to_string());
    write_registry(&registry, &entries).expect("write");
    let store = MemoryCredentialStore::new();
    store
        .set(&origin, CredentialKind::Refresh, "spk_rt_orphan")
        .expect("set");

    let result = logout(&origin, &store, &registry).expect("撤銷失敗不阻擋本機刪除");
    assert!(!result.revoked_on_server, "盡力撤銷失敗如實回報");
    assert_eq!(
        store.get(&origin, CredentialKind::Refresh).expect("get"),
        None,
        "本機 entry 仍被清除"
    );
    assert!(read_registry(&registry)[0].last_actor_display.is_none());
}

#[test]
fn pat_logout_deletes_locally_and_hints_at_the_account_page() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let (_, pat) = h
        .identity
        .create_pat(&h.user_id, "desktop", None)
        .expect("pat");
    pat_login(&h.origin, &pat, &store, &h.registry).expect("pat login");

    let result = logout(&h.origin, &store, &h.registry).expect("logout");
    assert!(
        result.pat_notice,
        "PAT 無自助撤銷端點——提示至 /account 頁撤銷"
    );
    assert!(!result.revoked_on_server);
    assert_eq!(
        store.get(&h.origin, CredentialKind::Pat).expect("get"),
        None
    );
}

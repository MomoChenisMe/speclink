//! 登入／登出編排契約（client-browser-login D7、D9；規格「device login 預設與
//! PAT fallback」「登出撤銷與移除連帶清理」「credential 唯一落點為 OS Keychain」）。
//!
//! 假瀏覽器開啟器＋in-process speclink-server（memory identity）＋in-memory
//! CredentialStore：核准/拒絕由假開啟器從 verification URL 的 user_code 預填
//! 參數取碼、直接對 identity store 動作——模擬 /activate 頁的人。登入的等待迴圈
//! 在 Rust 這一份（speclink-remote）；輪詢間隔與逾時的細節由那邊以可注入的時鐘
//! 釘死，這裡只看 Desktop 的委派、階段事件與取消。

use chrono::{Duration, Utc};
use speclink_desktop_lib::connections::{
    connection_login, logout, pat_login, read_registry, refresh_connection, upsert_connection,
    write_registry, ConnectionLogin, DesktopLoginIo, LoginCancels, LoginPhaseEvent,
};
use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
    let events = EventHub::new(store.clone(), EventSettings::default());
    let state = AppState::new(
        store,
        identity.clone(),
        Arc::new(ServerConfig {
            store: StoreConfig::Memory,
            identity: IdentityConfig::Memory,
            public_url: "http://127.0.0.1".to_string(),
            events: EventSettings::default(),
        }),
        events,
    );

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

/// 一次登入跑到終態，回傳結果與途中送往前端的階段事件。`cancel_on_phase`＝
/// 收到第一個階段事件就按取消（等同使用者在等待面按下取消）。
fn run_login(
    origin: &str,
    store: &dyn CredentialStore,
    registry: &std::path::Path,
    opener: &(dyn Fn(&str) -> Result<(), String> + Sync),
    cancel_on_phase: bool,
) -> (Result<ConnectionLogin, String>, Vec<LoginPhaseEvent>) {
    let events = Mutex::new(Vec::new());
    let cancel = AtomicBool::new(false);
    let emit = |event: LoginPhaseEvent| {
        events.lock().unwrap().push(event);
        if cancel_on_phase {
            cancel.store(true, Ordering::SeqCst);
        }
    };
    let io = DesktopLoginIo {
        origin,
        emit: &emit,
        open_browser: opener,
        cancel: &cancel,
    };
    let outcome = connection_login(origin, None, store, registry, &io);
    (outcome, events.into_inner().unwrap())
}

fn approving_opener(h: &Harness) -> impl Fn(&str) -> Result<(), String> + Sync {
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    move |url: &str| {
        assert!(identity.approve_device(&code_of(url), &user_id).expect("approve"));
        Ok(())
    }
}

fn no_browser(_url: &str) -> Result<(), String> {
    panic!("這條路徑不得開瀏覽器")
}

fn idle_browser(_url: &str) -> Result<(), String> {
    Ok(())
}

// --- 先試一次安靜換新 ---

#[test]
fn a_live_refresh_credential_logs_in_without_opening_the_browser() {
    // 「重啟」＝記憶體 access token 消失、只剩 Keychain 的 refresh credential，此時
    // 按登入應直接換新回已登入，不再開瀏覽器、不送任何階段事件。
    let h = harness();
    let store = MemoryCredentialStore::new();
    let (first, _) = run_login(&h.origin, &store, &h.registry, &approving_opener(&h), false);
    assert!(matches!(first, Ok(ConnectionLogin::LoggedIn { .. })), "{first:?}");

    let (again, events) = run_login(&h.origin, &store, &h.registry, &no_browser, false);
    match again.expect("silent refresh") {
        ConnectionLogin::LoggedIn { display, access_token } => {
            assert_eq!(display, DISPLAY, "靜默換新後身分即刻可讀");
            assert!(access_token.is_some(), "換得的 access token 交給 TokenManager");
        }
        other => panic!("expected LoggedIn, got {other:?}"),
    }
    assert!(events.is_empty(), "沒有等待面：{events:?}");
}

// --- device login：Rust 跑完整個等待 ---

#[test]
fn a_device_login_emits_the_approval_phase_and_lands_credential_and_identity() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let approve = approving_opener(&h);
    let sink = opened.clone();
    let opener = move |url: &str| {
        sink.lock().unwrap().push(url.to_string());
        approve(url)
    };

    let (outcome, events) = run_login(&h.origin, &store, &h.registry, &opener, false);

    match outcome.expect("device login") {
        ConnectionLogin::LoggedIn { display, access_token } => {
            assert_eq!(display, DISPLAY);
            assert!(access_token.is_some());
        }
        other => panic!("expected LoggedIn, got {other:?}"),
    }
    // 等待授權面所需的一切都在階段事件裡：裝置碼、驗證網址、截止時刻。
    assert_eq!(events.len(), 1, "{events:?}");
    let event = serde_json::to_value(&events[0]).unwrap();
    assert_eq!(event["origin"], h.origin);
    assert_eq!(event["kind"], "awaitingApproval");
    let user_code = event["userCode"].as_str().expect("userCode").to_string();
    assert!(event["verificationUri"].as_str().unwrap().contains("/activate"), "{event}");
    assert!(event["expiresAt"].as_u64().unwrap() > 0, "{event}");
    // 瀏覽器開的是同一個驗證網址，且帶 user_code 預填。
    let urls = opened.lock().unwrap();
    assert_eq!(urls.len(), 1);
    assert_eq!(code_of(&urls[0]), user_code);

    let rt = store.get(&h.origin, CredentialKind::Refresh).expect("get").expect("credential");
    assert!(rt.starts_with("spk_rt_"), "存的是 refresh credential：{rt}");
    assert_eq!(store.get(&h.origin, CredentialKind::Pat).expect("get"), None);
    assert_eq!(read_registry(&h.registry)[0].last_actor_display.as_deref(), Some(DISPLAY));
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

    let (outcome, _) = run_login(&h.origin, &store, &h.registry, &opener, false);

    assert!(matches!(outcome, Ok(ConnectionLogin::Denied)), "拒絕是可讀狀態：{outcome:?}");
    assert_eq!(store.get(&h.origin, CredentialKind::Refresh).expect("get"), None);
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

#[test]
fn cancelling_a_device_wait_ends_it_and_stores_nothing() {
    let h = harness();
    let store = MemoryCredentialStore::new();

    let (outcome, events) = run_login(&h.origin, &store, &h.registry, &idle_browser, true);

    assert!(matches!(outcome, Ok(ConnectionLogin::Cancelled)), "{outcome:?}");
    assert_eq!(events.len(), 1);
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
        assert_eq!(store.get(&h.origin, kind).expect("get"), None, "{kind:?}");
    }
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- 瀏覽器登入：階段事件只帶授權網址與期限 ---

/// 宣告瀏覽器登入的假 server：探索回 browser，登入服務的標準文件也由它供應。
fn browser_server() -> (Arc<tiny_http::Server>, String) {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let origin = format!("http://{}", server.server_addr());
    let issuer = format!("{origin}/idp");
    let looper = Arc::clone(&server);
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            let (status, body) = match path.as_str() {
                "/api/speclink/v1/auth-methods" => (
                    200,
                    format!(r#"{{"methods":["browser","device"],"browser":{{"issuer":"{issuer}","clientId":"speclink-desktop"}}}}"#),
                ),
                "/.well-known/oauth-authorization-server/idp" => (
                    200,
                    format!(r#"{{"issuer":"{issuer}","authorization_endpoint":"{issuer}/authorize","token_endpoint":"{issuer}/token"}}"#),
                ),
                _ => (404, "{}".to_string()),
            };
            let _ = req.respond(tiny_http::Response::from_string(body).with_status_code(status));
        }
    });
    (server, origin)
}

#[test]
fn a_browser_login_phase_carries_the_url_and_deadline_and_no_secret() {
    let (server, origin) = browser_server();
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = opened.clone();
    let opener = move |url: &str| {
        sink.lock().unwrap().push(url.to_string());
        Ok(())
    };

    let (outcome, events) = run_login(&origin, &store, &registry, &opener, true);

    assert!(matches!(outcome, Ok(ConnectionLogin::Cancelled)), "{outcome:?}");
    assert_eq!(events.len(), 1, "{events:?}");
    let event = serde_json::to_value(&events[0]).unwrap();
    assert_eq!(event["kind"], "awaitingBrowser");
    assert_eq!(event["origin"], origin);
    let url = event["url"].as_str().expect("url");
    assert!(url.starts_with(&format!("{origin}/idp/authorize?")), "{url}");
    assert!(url.contains("code_challenge_method=S256"), "{url}");
    assert!(event["expiresAt"].as_u64().unwrap() > 0, "{event}");
    assert!(event.get("userCode").is_none(), "沒有裝置碼：{event}");
    let text = event.to_string();
    for secret in ["code_verifier", "verifier", "access_token", "refresh_token", "\"code\""] {
        assert!(!text.contains(secret), "事件不得帶 {secret}：{text}");
    }
    assert_eq!(opened.lock().unwrap().as_slice(), &[url.to_string()], "開的就是事件裡的網址");
    // 取消後：Keychain 無任何新增條目，而且回呼的埠已關閉——之後瀏覽器端完成登入也沒用。
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
        assert_eq!(store.get(&origin, kind).expect("get"), None, "{kind:?}");
    }
    let redirect = url
        .split("redirect_uri=")
        .nth(1)
        .and_then(|rest| rest.split('&').next())
        .expect("redirect_uri")
        .replace("%3A", ":")
        .replace("%2F", "/");
    let port = redirect.trim_start_matches("http://127.0.0.1:").trim_end_matches("/callback");
    assert!(std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_err(), "回呼的埠已關閉");
    server.unblock();
}

// --- 探索與探測：未宣告時先試 device，404 才改收 PAT；5xx 是連線錯誤 ---

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
fn an_undeclared_server_without_a_device_flow_asks_for_a_pat() {
    let (server, origin) = fixed_server(404);
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");

    let (outcome, events) = run_login(&origin, &store, &registry, &no_browser, false);

    assert!(matches!(outcome, Ok(ConnectionLogin::PatRequired)), "{outcome:?}");
    assert!(events.is_empty());
    server.unblock();
}

#[test]
fn a_token_only_server_asks_for_a_pat_without_trying_the_device_flow() {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind"));
    let origin = format!("http://{}", server.server_addr());
    let paths: Arc<Mutex<Vec<String>>> = Arc::default();
    let (looper, log) = (Arc::clone(&server), paths.clone());
    std::thread::spawn(move || {
        for req in looper.incoming_requests() {
            log.lock().unwrap().push(req.url().to_string());
            let (status, body) = if req.url() == "/api/speclink/v1/auth-methods" {
                (200, r#"{"methods":["token"]}"#)
            } else {
                (404, "{}")
            };
            let _ = req.respond(tiny_http::Response::from_string(body).with_status_code(status));
        }
    });
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");

    let (outcome, _) = run_login(&origin, &store, &dir.path().join("connections.json"), &no_browser, false);

    assert!(matches!(outcome, Ok(ConnectionLogin::PatRequired)), "{outcome:?}");
    assert_eq!(paths.lock().unwrap().as_slice(), &["/api/speclink/v1/auth-methods".to_string()]);
    server.unblock();
}

#[test]
fn a_5xx_is_a_connection_error_not_a_fallback() {
    let (server, origin) = fixed_server(503);
    let store = MemoryCredentialStore::new();
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");

    let (outcome, _) = run_login(&origin, &store, &registry, &no_browser, false);

    assert!(outcome.is_err(), "5xx 是錯誤、絕不進 PAT fallback：{outcome:?}");
    assert_eq!(store.get(&origin, CredentialKind::Refresh).expect("get"), None);
    server.unblock();
}

// --- pat_login：身分查詢驗證後才入 store ---

#[test]
fn pat_login_validates_via_whoami_before_storing() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let (_, pat) = h.identity.create_pat(&h.user_id, "desktop", None).expect("pat");

    let display = pat_login(&h.origin, &pat, &store, &h.registry).expect("pat login");
    assert_eq!(display, DISPLAY);
    assert_eq!(
        store.get(&h.origin, CredentialKind::Pat).expect("get").as_deref(),
        Some(pat.as_str()),
        "驗證通過後 PAT 才入 store"
    );
    assert_eq!(read_registry(&h.registry)[0].last_actor_display.as_deref(), Some(DISPLAY));
}

#[test]
fn an_invalid_pat_is_refused_and_never_stored() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    pat_login(&h.origin, "spk_pat_nope", &store, &h.registry).expect_err("無效 PAT 拒絕");
    assert_eq!(store.get(&h.origin, CredentialKind::Pat).expect("get"), None, "無效 PAT 不落任何盤");
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none());
}

// --- rotation：新 refresh credential 覆寫 ---

#[test]
fn rotation_overwrites_the_stored_refresh_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    run_login(&h.origin, &store, &h.registry, &approving_opener(&h), false).0.expect("login");
    let old_rt = store.get(&h.origin, CredentialKind::Refresh).expect("get").expect("rt");

    let access = refresh_connection(&h.origin, &store).expect("rotation");
    assert!(access.starts_with("spk_at_"), "rotation 換得新 access token");
    let new_rt = store.get(&h.origin, CredentialKind::Refresh).expect("get").expect("rt");
    assert_ne!(new_rt, old_rt, "新 refresh credential 覆寫 Keychain slot");

    // 再 rotation 一次成功——證明 store 裡是活的最新 credential（舊 rt 重用
    // 會被 server 以 family revocation 拒絕）。
    refresh_connection(&h.origin, &store).expect("再次 rotation 走最新 credential");
}

#[test]
fn a_transient_server_failure_never_discards_a_live_refresh_credential() {
    // 5xx／不可達是連線錯誤，不是「credential 已失效」的語意訊號——抹掉有效
    // credential 會讓 server 一次短暫抖動就逼使用者重新登入。
    let (server, origin) = fixed_server(503);
    let store = MemoryCredentialStore::new();
    store.set(&origin, CredentialKind::Refresh, "spk_rt_live").expect("set");
    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");

    let (outcome, _) = run_login(&origin, &store, &registry, &no_browser, false);

    assert!(outcome.is_err(), "5xx 是連線錯誤");
    assert_eq!(
        store.get(&origin, CredentialKind::Refresh).expect("get").as_deref(),
        Some("spk_rt_live"),
        "server 暫時不可達不得抹除有效 credential"
    );
    server.unblock();
}

#[test]
fn a_rejected_refresh_credential_is_cleared_and_the_full_device_flow_takes_over() {
    // 對照組：server 明確拒絕才是「credential 已死」——清掉殘骸、走完整登入。
    let h = harness();
    let store = MemoryCredentialStore::new();
    store.set(&h.origin, CredentialKind::Refresh, "spk_rt_dead").expect("set");

    let (outcome, _) = run_login(&h.origin, &store, &h.registry, &approving_opener(&h), false);

    assert!(matches!(outcome, Ok(ConnectionLogin::LoggedIn { .. })), "殘骸不擋重新核准：{outcome:?}");
    let rt = store.get(&h.origin, CredentialKind::Refresh).expect("get").expect("credential");
    assert_ne!(rt, "spk_rt_dead", "殘骸已被新核准的 refresh credential 取代");
}

// --- logout：盡力撤銷＋清掉和 CLI 相同的三個條目 ---

#[test]
fn logout_revokes_the_family_and_clears_every_keyring_entry() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let access = match run_login(&h.origin, &store, &h.registry, &approving_opener(&h), false).0 {
        Ok(ConnectionLogin::LoggedIn { access_token: Some(access), .. }) => access,
        other => panic!("expected LoggedIn, got {other:?}"),
    };
    assert!(store.get(&h.origin, CredentialKind::Bearer).expect("get").is_some(), "登入時快取了 access token");

    let result = logout(&h.origin, &store, &h.registry).expect("logout");
    assert!(result.revoked_on_server, "refresh 走 /auth/revoke 撤了 device family");
    assert!(!result.pat_notice);
    for kind in [CredentialKind::Refresh, CredentialKind::Bearer, CredentialKind::Pat] {
        assert_eq!(store.get(&h.origin, kind).expect("get"), None, "{kind:?} 已刪");
    }
    assert!(read_registry(&h.registry)[0].last_actor_display.is_none(), "registry 身分已清");
    assert!(
        h.identity.authenticate_access_token(&access).expect("authenticate").is_none(),
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
    store.set(&origin, CredentialKind::Refresh, "spk_rt_orphan").expect("set");
    store.set(&origin, CredentialKind::Bearer, "{}").expect("set");

    let result = logout(&origin, &store, &registry).expect("撤銷失敗不阻擋本機刪除");
    assert!(!result.revoked_on_server, "盡力撤銷失敗如實回報");
    assert_eq!(store.get(&origin, CredentialKind::Refresh).expect("get"), None, "本機 entry 仍被清除");
    assert_eq!(store.get(&origin, CredentialKind::Bearer).expect("get"), None);
    assert!(read_registry(&registry)[0].last_actor_display.is_none());
}

#[test]
fn pat_logout_deletes_locally_and_hints_at_the_account_page() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let (_, pat) = h.identity.create_pat(&h.user_id, "desktop", None).expect("pat");
    pat_login(&h.origin, &pat, &store, &h.registry).expect("pat login");

    let result = logout(&h.origin, &store, &h.registry).expect("logout");
    assert!(result.pat_notice, "PAT 無自助撤銷端點——提示至 /account 頁撤銷");
    assert!(!result.revoked_on_server);
    assert_eq!(store.get(&h.origin, CredentialKind::Pat).expect("get"), None);
}

// --- 取消旗標 ---

#[test]
fn a_new_login_cancels_the_one_still_running_for_the_same_origin() {
    let cancels = LoginCancels::default();
    let first = cancels.begin("https://a.example");
    let second = cancels.begin("https://a.example");
    assert!(first.load(Ordering::SeqCst), "重按登入＝取消前一輪");
    assert!(!second.load(Ordering::SeqCst));

    // 前一輪收尾時不得清掉新一輪的旗標。
    cancels.end("https://a.example", &first);
    cancels.cancel("https://a.example");
    assert!(second.load(Ordering::SeqCst), "取消命中仍在跑的那一輪");

    let other = cancels.begin("https://b.example");
    cancels.cancel("https://a.example");
    assert!(!other.load(Ordering::SeqCst), "取消只作用於該 origin");
}

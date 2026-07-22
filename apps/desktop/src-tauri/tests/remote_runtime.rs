//! remote runtime 的 token 生命週期契約（design 決策 4「token 生命週期與 401
//! 語意」；規格「token 換發全程 Rust 側且 401 語意固定」）。
//!
//! in-process speclink-server（memory identity＋seeded change）＋in-memory
//! CredentialStore：TokenManager 請求前自動換發、401 → refresh 一次 → 重試
//! 一次、rotation 新 refresh credential 回寫、refresh 亦失效 → needs-reauth
//! 狀態且後續操作回拒絕錯誤。token 只存在 Rust——測試面沒有任何 secret 出境
//! 斷言對象。

mod common;

use common::Harness;
use speclink_desktop_lib::connections::pat_login;
use speclink_desktop_lib::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_desktop_lib::remote::{
    ConnectionState, ConnectionStateEvent, RemoteWorkspace, TokenManager,
    REMOTE_CONNECTION_STATE_EVENT,
};
use speclink_remote::client::Client;
use speclink_remote::RemoteError;
use speclink_server::identity::IdentityStore;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex};
use std::time::Duration;

/// 讓前兩個 refresh 讀取都先取得同一份舊值再放行。修正後只有 singleflight
/// leader 會讀取 refresh，因此第一個讀取會在短暫 timeout 後自行前進，不會死鎖。
struct RacingCredentialStore {
    inner: MemoryCredentialStore,
    armed: AtomicBool,
    refresh_readers: Mutex<HashSet<std::thread::ThreadId>>,
    arrived: Mutex<usize>,
    release: Condvar,
}

impl RacingCredentialStore {
    fn new() -> Self {
        Self {
            inner: MemoryCredentialStore::new(),
            armed: AtomicBool::new(false),
            refresh_readers: Mutex::new(HashSet::new()),
            arrived: Mutex::new(0),
            release: Condvar::new(),
        }
    }

    fn arm(&self) {
        self.refresh_readers.lock().expect("reader lock").clear();
        *self.arrived.lock().expect("arrival lock") = 0;
        self.armed.store(true, Ordering::SeqCst);
    }

    fn disarm(&self) {
        self.armed.store(false, Ordering::SeqCst);
        self.release.notify_all();
    }

    fn refresh_readers(&self) -> usize {
        self.refresh_readers.lock().expect("reader lock").len()
    }
}

impl CredentialStore for RacingCredentialStore {
    fn get(&self, origin: &str, kind: CredentialKind) -> Result<Option<String>, String> {
        let value = self.inner.get(origin, kind)?;
        if kind == CredentialKind::Refresh && self.armed.load(Ordering::SeqCst) {
            self.refresh_readers
                .lock()
                .expect("reader lock")
                .insert(std::thread::current().id());
            let mut arrived = self.arrived.lock().expect("arrival lock");
            *arrived += 1;
            if *arrived >= 2 {
                self.release.notify_all();
            } else {
                let (next, _) = self
                    .release
                    .wait_timeout_while(arrived, Duration::from_millis(250), |count| *count < 2)
                    .expect("refresh read gate");
                arrived = next;
            }
            drop(arrived);
        }
        Ok(value)
    }

    fn set(&self, origin: &str, kind: CredentialKind, secret: &str) -> Result<(), String> {
        self.inner.set(origin, kind, secret)
    }

    fn delete(&self, origin: &str, kind: CredentialKind) -> Result<(), String> {
        self.inner.delete(origin, kind)
    }
}

struct UnavailableCredentialStore {
    reads: AtomicUsize,
}

impl UnavailableCredentialStore {
    fn new() -> Self {
        Self {
            reads: AtomicUsize::new(0),
        }
    }
}

impl CredentialStore for UnavailableCredentialStore {
    fn get(&self, _origin: &str, _kind: CredentialKind) -> Result<Option<String>, String> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Err("Keychain 暫時不可用".to_string())
    }

    fn set(&self, _origin: &str, _kind: CredentialKind, _secret: &str) -> Result<(), String> {
        Err("Keychain 暫時不可用".to_string())
    }

    fn delete(&self, _origin: &str, _kind: CredentialKind) -> Result<(), String> {
        Err("Keychain 暫時不可用".to_string())
    }
}

fn harness() -> Harness {
    let h = common::harness();
    common::seed_change(h.store.as_ref(), "- [ ] 1.1 First\n- [ ] 1.2 Second\n");
    h
}

/// 對 in-process server 的 demo project 逐請求建構 client（決策 4 的消費形狀）。
fn fetch_tasks(h: &Harness, token: &str) -> Result<String, RemoteError> {
    let client = Client::new(
        &format!("{}/api/speclink/v1/projects/demo", h.origin),
        token,
        Some("backend"),
    );
    client
        .get_artifact("demo", "tasks")
        .map(|artifact| artifact.content)
}

fn state_sink() -> (
    impl Fn(ConnectionStateEvent) + Send + Sync + 'static,
    std::sync::mpsc::Receiver<ConnectionStateEvent>,
) {
    let (tx, rx) = std::sync::mpsc::channel();
    (
        move |event| {
            let _ = tx.send(event);
        },
        rx,
    )
}

// --- per-connection 離線狀態機與事件 payload ---

#[test]
fn consecutive_transport_failures_go_offline_and_one_success_returns_online() {
    let (notify, rx) = state_sink();
    let manager = TokenManager::with_connection_state("http://server.test", "conn_x", 2, notify);
    manager.adopt_access_token("pat");
    let store = MemoryCredentialStore::new();

    let fail = || -> Result<(), RemoteError> { Err(speclink_remote::translate_transport()) };
    manager
        .execute(&store, |_| fail())
        .expect_err("第一次傳輸失敗");
    assert!(
        rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "未達閾值不得提早廣播 offline"
    );
    manager
        .execute(&store, |_| fail())
        .expect_err("第二次傳輸失敗");
    let offline = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("達閾值廣播 offline");
    assert_eq!(REMOTE_CONNECTION_STATE_EVENT, "remote-connection-state");
    assert_eq!(offline.connection_id, "conn_x");
    assert_eq!(offline.state, ConnectionState::Offline);
    assert!(offline
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("離線"));
    assert_eq!(manager.connection_state(), ConnectionState::Offline);

    manager.execute(&store, |_| Ok(())).expect("一次成功即恢復");
    let online = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("恢復廣播 online");
    assert_eq!(online.state, ConnectionState::Online);
    assert_eq!(manager.connection_state(), ConnectionState::Online);

    // 成功已把 failure count 歸零；再失敗一次仍不得立即 offline。
    manager
        .execute(&store, |_| fail())
        .expect_err("成功後第一次失敗");
    assert!(
        rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "成功後 failure count 從零重算"
    );
}

#[test]
fn needs_reauth_is_broadcast_and_wins_over_an_existing_offline_state() {
    let (notify, rx) = state_sink();
    let manager = TokenManager::with_connection_state("http://server.test", "conn_x", 1, notify);
    let store = MemoryCredentialStore::new();
    store
        .set("http://server.test", CredentialKind::Pat, "spk_pat_dead")
        .expect("set PAT");
    manager.adopt_access_token("spk_pat_dead");

    manager
        .execute(&store, |_| -> Result<(), RemoteError> {
            Err(speclink_remote::translate_transport())
        })
        .expect_err("傳輸失敗使狀態 offline");
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(1))
            .expect("offline event")
            .state,
        ConnectionState::Offline
    );

    manager
        .execute(&store, |_| -> Result<(), RemoteError> {
            Err(RemoteError {
                message: "unauthorized".into(),
                reason: Some("permission_denied".into()),
                status: Some(401),
            })
        })
        .expect_err("兩次 401 進 needs-reauth");
    let reauth = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("needs-reauth event");
    assert_eq!(reauth.state, ConnectionState::NeedsReauth);
    assert!(reauth
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("重新登入"));
    assert_eq!(manager.connection_state(), ConnectionState::NeedsReauth);

    manager
        .execute(&store, |_| -> Result<(), RemoteError> {
            panic!("needs-reauth 時不得打 server")
        })
        .expect_err("needs-reauth 後立即拒絕");
    assert_eq!(manager.connection_state(), ConnectionState::NeedsReauth);
}

#[test]
fn offline_writes_are_rejected_without_queueing_while_reads_can_recover_in_place() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let pat = common::pat_of(&h);
    store
        .set(&h.origin, CredentialKind::Pat, &pat)
        .expect("set PAT");
    let manager = Arc::new(TokenManager::with_connection_state(
        &h.origin,
        "conn_x",
        1,
        |_| {},
    ));
    manager.adopt_access_token(&pat);
    let workspace = RemoteWorkspace::at(&h.origin, "demo", "backend", &manager);
    assert_eq!(
        workspace
            .list_changes(&store)
            .expect("baseline read")
            .changes
            .len(),
        1
    );

    h.server.stop();
    workspace
        .list_changes(&store)
        .expect_err("server 停止後讀取失敗並使 runtime offline");
    assert_eq!(manager.connection_state(), ConnectionState::Offline);

    let rejected = workspace
        .set_task_done(&store, "demo", "1", true)
        .expect_err("offline 寫入立即拒絕");
    assert_eq!(rejected.reason.as_deref(), Some("offline"));
    assert!(rejected.message.contains("離線"));

    h.server.start();
    let mut recovered = None;
    for _ in 0..20 {
        match workspace.list_changes(&store) {
            Ok(changes) => {
                recovered = Some(changes);
                break;
            }
            Err(_) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    assert!(
        recovered.is_some(),
        "offline 期間的讀取路徑仍放行，server 回來即可原地恢復"
    );
    assert_eq!(manager.connection_state(), ConnectionState::Online);
    let tasks = workspace
        .document(&store, "demo", "tasks.md")
        .expect("server 恢復後讀取 tasks")
        .content;
    assert!(
        tasks.contains("- [ ] 1.1 First"),
        "離線寫入未排隊、未於恢復後重放：{tasks}"
    );
}

// --- 請求前自動換發＋rotation 回寫 ---

#[test]
fn a_request_with_no_access_token_refreshes_first_and_rotates_the_credential() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    common::device_login_approved(&h, &store);
    let rt_before = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("credential");

    // 「重啟後」：記憶體沒有 access token——execute 請求前先以 refresh credential 換發。
    let manager = TokenManager::new(&h.origin);
    let content = manager
        .execute(&store, |token| fetch_tasks(&h, token))
        .expect("查詢經自動換發後成功");
    assert!(content.contains("- [ ] 1.1"), "查詢回真值：{content}");
    assert!(
        manager.needs_reauth().is_none(),
        "成功路徑不進 needs-reauth"
    );

    // rotation 後的新 refresh credential 立即回寫（規格「access token 過期自動換發」）。
    let rt_after = store
        .get(&h.origin, CredentialKind::Refresh)
        .expect("get")
        .expect("credential");
    assert_ne!(
        rt_after, rt_before,
        "rotation 新 refresh credential 回寫 store"
    );
}

#[test]
fn concurrent_requests_without_a_bearer_share_one_refresh_rotation() {
    let h = harness();
    let store = Arc::new(RacingCredentialStore::new());
    common::device_login_approved(&h, store.as_ref());
    store.arm();

    let (notify, rx) = state_sink();
    let manager = Arc::new(TokenManager::with_connection_state(
        &h.origin, "conn_x", 1, notify,
    ));
    let start = Arc::new(Barrier::new(3));
    let mut threads = Vec::new();
    for _ in 0..2 {
        let manager = manager.clone();
        let store = store.clone();
        let start = start.clone();
        threads.push(std::thread::spawn(move || {
            start.wait();
            manager.execute(store.as_ref(), |_| Ok::<_, RemoteError>(()))
        }));
    }
    start.wait();

    let results = threads
        .into_iter()
        .map(|thread| thread.join().expect("request thread"))
        .collect::<Vec<_>>();
    store.disarm();

    assert!(
        results.iter().all(Result::is_ok),
        "兩個 caller 都應共用成功 rotation：{results:?}"
    );
    assert_eq!(
        store.refresh_readers(),
        1,
        "同一枚 refresh credential 只能由 singleflight leader thread 讀取"
    );
    assert!(manager.needs_reauth().is_none(), "不得誤入 needs-reauth");
    assert!(
        rx.try_iter()
            .all(|event| event.state != ConnectionState::NeedsReauth),
        "不得廣播 needs-reauth"
    );

    TokenManager::new(&h.origin)
        .execute(store.as_ref(), |_| Ok::<_, RemoteError>(()))
        .expect("latest refresh credential 仍可再次輪替，family 未被撤銷");
}

// --- 401 → refresh 一次 → 重試一次成功 ---

#[test]
fn a_401_refreshes_once_and_retries_once_successfully() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    common::device_login_approved(&h, &store);

    // 過期／失效的 access token 在快取裡（adopt＝登入流程交接 token 的同一入口）。
    let manager = TokenManager::new(&h.origin);
    manager.adopt_access_token("spk_at_expired_bogus");

    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let harness_ref = &h;
    let content = manager
        .execute(&store, move |token| {
            sink.lock().unwrap().push(token.to_string());
            fetch_tasks(harness_ref, token)
        })
        .expect("401 經 refresh 一次、重試一次後成功——使用者無感");
    assert!(content.contains("- [ ] 1.1"));

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2, "恰好一次重試：{seen:?}");
    assert_eq!(seen[0], "spk_at_expired_bogus", "第一擊帶著過期 token");
    assert_ne!(seen[1], "spk_at_expired_bogus", "重試帶著換發後的新 token");
}

#[test]
fn concurrent_401_responses_share_the_new_bearer_without_replaying_refresh() {
    const EXPIRED: &str = "spk_at_expired_bogus";

    let h = harness();
    let store = Arc::new(RacingCredentialStore::new());
    common::device_login_approved(&h, store.as_ref());
    store.arm();

    let (notify, rx) = state_sink();
    let manager = Arc::new(TokenManager::with_connection_state(
        &h.origin, "conn_x", 1, notify,
    ));
    manager.adopt_access_token(EXPIRED);
    let start = Arc::new(Barrier::new(3));
    let rejected = Arc::new(Barrier::new(2));
    let mut threads = Vec::new();
    for _ in 0..2 {
        let manager = manager.clone();
        let store = store.clone();
        let start = start.clone();
        let rejected = rejected.clone();
        threads.push(std::thread::spawn(move || {
            start.wait();
            manager.execute(store.as_ref(), |token| {
                if token == EXPIRED {
                    rejected.wait();
                    Err(RemoteError {
                        message: "unauthorized".into(),
                        reason: Some("permission_denied".into()),
                        status: Some(401),
                    })
                } else {
                    Ok(token.to_string())
                }
            })
        }));
    }
    start.wait();

    let results = threads
        .into_iter()
        .map(|thread| thread.join().expect("request thread"))
        .collect::<Vec<_>>();
    store.disarm();

    assert!(
        results.iter().all(Result::is_ok),
        "兩個 401 caller 都應以同一枚新 bearer 重試成功：{results:?}"
    );
    let first = results[0].as_ref().expect("first caller bearer");
    let second = results[1].as_ref().expect("second caller bearer");
    assert_eq!(first, second, "等待者共用 leader 發布的新 bearer");
    assert_eq!(
        store.refresh_readers(),
        1,
        "401 競態只允許 singleflight leader thread 進入 rotation"
    );
    assert!(manager.needs_reauth().is_none(), "不得誤入 needs-reauth");
    assert!(
        rx.try_iter()
            .all(|event| event.state != ConnectionState::NeedsReauth),
        "不得廣播 needs-reauth"
    );
}

// --- refresh 亦失效 → needs-reauth＋後續操作回拒絕 ---

#[test]
fn a_dead_refresh_credential_flags_needs_reauth_and_rejects_further_operations() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    // 只有一枚已死（未經核發）的 refresh credential——server 會明確拒絕 rotation。
    store
        .set(&h.origin, CredentialKind::Refresh, "spk_rt_dead")
        .expect("set");

    let manager = TokenManager::new(&h.origin);
    let err = manager
        .execute(&store, |_token| -> Result<String, RemoteError> {
            panic!("refresh 被拒後不得再打資料面請求")
        })
        .expect_err("refresh 亦失效即失敗");
    assert!(
        err.message.contains("重新登入"),
        "繁中訊息指向重新登入：{}",
        err.message
    );

    let state = manager.needs_reauth().expect("連線進入 needs-reauth 狀態");
    assert!(
        state.contains("重新登入"),
        "TS 可見的是繁中狀態訊息：{state}"
    );

    // 後續操作直接回拒絕錯誤，不再打 server、不再碰 credential。
    let err = manager
        .execute(&store, |_token| -> Result<String, RemoteError> {
            panic!("needs-reauth 後的操作不得觸發任何請求")
        })
        .expect_err("後續操作回拒絕錯誤");
    assert!(
        err.message.contains("重新登入"),
        "拒絕錯誤同樣是繁中：{}",
        err.message
    );
}

#[test]
fn a_revoked_device_family_still_flags_needs_reauth_once() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    common::device_login_approved(&h, &store);
    let family = h
        .identity
        .list_device_families(&h.user_id)
        .expect("list device families")
        .into_iter()
        .find(|family| family.revoked_at.is_none())
        .expect("live device family");
    h.identity
        .revoke_family(&h.user_id, &family.id)
        .expect("revoke device family");

    let (notify, rx) = state_sink();
    let manager = TokenManager::with_connection_state(&h.origin, "conn_x", 1, notify);
    let error = manager
        .execute(&store, |_token| -> Result<(), RemoteError> {
            panic!("revoked refresh 不得進資料面")
        })
        .expect_err("revoked family 必須要求重新登入");
    assert_eq!(error.reason.as_deref(), Some("needs_reauth"));
    assert_eq!(manager.connection_state(), ConnectionState::NeedsReauth);
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(1))
            .expect("needs-reauth event")
            .state,
        ConnectionState::NeedsReauth
    );

    manager
        .execute(&store, |_token| -> Result<(), RemoteError> {
            panic!("needs-reauth 後不得重試 revoked refresh")
        })
        .expect_err("後續操作立即拒絕");
    assert!(
        rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "相同狀態不得重複廣播"
    );
}

#[test]
fn a_temporary_credential_store_failure_remains_retryable_without_reauth() {
    let store = UnavailableCredentialStore::new();
    let manager = TokenManager::new("http://server.test");

    for expected_reads in 1..=2 {
        let error = manager
            .execute(&store, |_token| -> Result<(), RemoteError> {
                panic!("credential 讀取失敗不得進資料面")
            })
            .expect_err("暫時性 Keychain 失敗原樣回傳");
        assert_eq!(error.message, "Keychain 暫時不可用");
        assert_eq!(error.reason, None);
        assert_eq!(error.status, None);
        assert_eq!(
            store.reads.load(Ordering::SeqCst),
            expected_reads,
            "下一個操作仍會重試 credential store"
        );
        assert!(manager.needs_reauth().is_none());
        assert_eq!(manager.connection_state(), ConnectionState::Online);
    }
}

// --- PAT 連線：PAT 即 bearer、無 rotation ---

#[test]
fn a_pat_connection_uses_the_pat_as_bearer_without_rotation() {
    let h = harness();
    let store = MemoryCredentialStore::new();
    let pat = common::pat_of(&h);
    pat_login(&h.origin, &pat, &store, &h.registry).expect("pat login");

    let manager = TokenManager::new(&h.origin);
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let harness_ref = &h;
    let content = manager
        .execute(&store, move |token| {
            sink.lock().unwrap().push(token.to_string());
            fetch_tasks(harness_ref, token)
        })
        .expect("PAT 連線的查詢成功");
    assert!(content.contains("- [ ] 1.1"));
    assert_eq!(
        seen.lock().unwrap().as_slice(),
        &[pat.clone()],
        "PAT 本身就是 bearer"
    );
    assert_eq!(
        store.get(&h.origin, CredentialKind::Refresh).expect("get"),
        None,
        "PAT 連線沒有 refresh credential、無 rotation"
    );
}

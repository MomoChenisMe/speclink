//! event manager 契約（design「決策 3：SSE 消費落 speclink-remote、event
//! manager 落 src-tauri」「決策 5：斷線收斂程序」；規格「Query 加 ETag 為重讀
//! 正典且 push 只做 invalidate」「斷線以 Polling 加 ETag 收斂後續訂」）。
//!
//! in-process speclink-server＋可注入的訂閱閉包與退避序列：invalidate 到達即
//! 發 remote-workspace-changed（payload＝locator key）；同 connection 兩
//! session 共用單一訂閱（計數斷言）；強制斷流（TCP proxy sever）後以
//! /sync-state ETag 收斂、退避重連並 Last-Event-ID 續傳；reset 信號發全量
//! 重載通知後自新位點續訂。

use crate::common;

use speclink_remote::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_desktop_lib::event_manager::EventManager;
use speclink_desktop_lib::remote::{self, ConnectionState, ConnectionStateEvent, DeclaredStreams, TokenManager};
use speclink_remote::client::Client;
use speclink_remote::events;
use speclink_server::events::EventSettings;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const FOUR_TASKS: &str = "- [ ] 1.1 First\n- [ ] 1.2 Second\n- [ ] 1.3 Third\n- [ ] 1.4 Fourth\n";
const KEY: &str = "remote:conn_x/demo/backend";

// --- 退避輪詢的 sync-state 失敗納入 connection failure count ---

#[test]
fn repeated_sync_state_failures_during_backoff_drive_the_connection_offline() {
    let (state_tx, state_rx) = mpsc::channel::<ConnectionStateEvent>();
    let manager = Arc::new(TokenManager::with_connection_state(
        "http://server.test",
        "conn_x",
        2,
        move |event| {
            let _ = state_tx.send(event);
        },
    ));
    manager.adopt_access_token("pat");
    let credentials = Arc::new(MemoryCredentialStore::new());
    credentials
        .set("http://server.test", CredentialKind::Pat, "pat")
        .expect("set PAT");

    let (notify, _rx) = sink();
    let events = EventManager::new(notify);
    let sync_manager = manager.clone();
    let sync_credentials = credentials.clone();
    events.register(
        KEY,
        |_| Err(speclink_remote::translate_transport()),
        move || {
            sync_manager.execute(sync_credentials.as_ref(), |_| -> Result<String, _> {
                Err(speclink_remote::translate_transport())
            })
        },
        vec![Duration::from_millis(5)],
    );

    let offline = state_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("每輪退避的 sync-state 失敗達閾值後廣播 offline");
    assert_eq!(offline.connection_id, "conn_x");
    assert_eq!(offline.state, ConnectionState::Offline);
    events.unregister(KEY);
}

// --- async 化後 IPC 順序不再保證：先到的 unregister 抵銷在途的 register ---

#[test]
fn an_unregister_arriving_before_its_register_cancels_that_subscription() {
    let (notify, _rx) = sink();
    let events = EventManager::new(notify);
    let attempts = Arc::new(AtomicUsize::new(0));

    // 快速 mount/unmount 下 cleanup 的 unregister 可先於 register 抵達。
    events.unregister(KEY);

    let sub_attempts = attempts.clone();
    events.register(
        KEY,
        move |_| {
            sub_attempts.fetch_add(1, Ordering::SeqCst);
            Err(speclink_remote::translate_transport())
        },
        || Err(speclink_remote::translate_transport()),
        vec![Duration::from_millis(5)],
    );
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        0,
        "先到的 unregister 必須抵銷這次 register——不得開出沒人退訂的 worker"
    );

    // 抵銷是一次性的：下一個 register 照常開訂閱。
    let sub_attempts = attempts.clone();
    events.register(
        KEY,
        move |_| {
            sub_attempts.fetch_add(1, Ordering::SeqCst);
            Err(speclink_remote::translate_transport())
        },
        || Err(speclink_remote::translate_transport()),
        vec![Duration::from_millis(5)],
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while attempts.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        attempts.load(Ordering::SeqCst) > 0,
        "抵銷額度用掉後，新 register 必須照常開 worker"
    );
    events.unregister(KEY);
}

#[test]
fn a_failed_watch_neutralizes_its_paired_unwatch() {
    let (notify, _rx) = sink();
    let events = EventManager::new(notify);
    let attempts = Arc::new(AtomicUsize::new(0));

    // remote_watch 於註冊前失敗（如 registry 壞讀）＝沒有 register 入帳，
    // 但前端 unmount 照樣發 unregister——這筆退訂不得毒害之後的訂閱。
    events.register_failed(KEY);
    events.unregister(KEY);

    let sub_attempts = attempts.clone();
    events.register(
        KEY,
        move |_| {
            sub_attempts.fetch_add(1, Ordering::SeqCst);
            Err(speclink_remote::translate_transport())
        },
        || Err(speclink_remote::translate_transport()),
        vec![Duration::from_millis(5)],
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while attempts.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        attempts.load(Ordering::SeqCst) > 0,
        "失敗註冊的配對退訂被中和後，新 register 必須照常開 worker"
    );
    events.unregister(KEY);

    // 反向交錯：unregister 先到（記額度）、失敗註冊後到（消額度）——
    // 同樣不得留下毒藥。
    events.unregister(KEY);
    events.register_failed(KEY);
    let sub_attempts = attempts.clone();
    attempts.store(0, Ordering::SeqCst);
    events.register(
        KEY,
        move |_| {
            sub_attempts.fetch_add(1, Ordering::SeqCst);
            Err(speclink_remote::translate_transport())
        },
        || Err(speclink_remote::translate_transport()),
        vec![Duration::from_millis(5)],
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while attempts.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        attempts.load(Ordering::SeqCst) > 0,
        "先到的退訂被後到的失敗註冊消帳後，新 register 必須照常開 worker"
    );
    events.unregister(KEY);
}

#[test]
fn a_restarted_server_recovers_online_and_invalidates_without_user_action() {
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let credentials = Arc::new(MemoryCredentialStore::new());
    credentials
        .set(&h.origin, CredentialKind::Pat, &pat)
        .expect("set PAT");
    let (state_tx, state_rx) = mpsc::channel::<ConnectionStateEvent>();
    let runtime = Arc::new(TokenManager::with_connection_state(
        &h.origin,
        "conn_x",
        1,
        move |event| {
            let _ = state_tx.send(event);
        },
    ));
    runtime.adopt_access_token(&pat);

    h.server.stop();
    let (notify, notify_rx) = sink();
    let events = EventManager::new(notify);
    let sub_url = common::demo_stream(&project_base(&h.origin));
    let sub_runtime = runtime.clone();
    let sub_credentials = credentials.clone();
    let sync_base = project_base(&h.origin);
    let sync_runtime = runtime.clone();
    let sync_credentials = credentials.clone();
    events.register(
        KEY,
        move |last| {
            sub_runtime.execute(sub_credentials.as_ref(), |token| {
                events::subscribe(&sub_url, token, Some("backend"), last)
            })
        },
        move || {
            sync_runtime.execute(sync_credentials.as_ref(), |token| {
                events::sync_state(&sync_base, token, Some("backend"))
            })
        },
        vec![Duration::from_millis(30)],
    );
    // Windows 對 refused connect 會在 winsock 內部重試（單次 ~1s 才回錯），
    // CI runner 排程再放大——上限只防吊死，放寬不影響通過速度。
    let offline = state_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("server 停止後 worker 自動轉 offline");
    assert_eq!(offline.state, ConnectionState::Offline);

    // 模擬同一持久 store 在 desktop 斷線期間由另一個 client 寫入。
    common::seed_named_change(h.store.as_ref(), "recovered", "- [ ] 1.1 Recovered\n");
    h.server.start();

    let online = state_rx
        .recv_timeout(Duration::from_secs(15))
        .expect("server 重啟後 worker 自動轉 online");
    assert_eq!(online.state, ConnectionState::Online);
    expect_notified(&notify_rx, "恢復收斂後送出全量失效通知");
    let names = writer(&h.origin, &pat)
        .list_changes()
        .expect("恢復後可讀 server 真值")
        .changes
        .into_iter()
        .map(|change| change.name)
        .collect::<Vec<_>>();
    assert!(names.iter().any(|name| name == "recovered"));
    events.unregister(KEY);
}

// --- 訂閱的網址取自握手的宣告：remote_open 記下、remote_watch 每次訂閱時讀取 ---

/// remote_watch 註冊的那一對函式：訂閱走 `remote::subscribe_declared`，輪詢照常。
fn register_declared(
    events: &EventManager,
    streams: &Arc<DeclaredStreams>,
    runtime: &Arc<TokenManager>,
    credentials: &Arc<MemoryCredentialStore>,
    origin: &str,
    backoff: Duration,
) {
    let (sub_streams, sub_runtime, sub_credentials) = (streams.clone(), runtime.clone(), credentials.clone());
    let (sync_base, sync_runtime, sync_credentials) = (project_base(origin), runtime.clone(), credentials.clone());
    events.register(
        KEY,
        move |last| {
            remote::subscribe_declared(
                &sub_streams,
                "conn_x",
                "demo",
                "backend",
                &sub_runtime,
                sub_credentials.as_ref(),
                last,
            )
        },
        move || {
            sync_runtime.execute(sync_credentials.as_ref(), |token| {
                events::sync_state(&sync_base, token, Some("backend"))
            })
        },
        vec![backoff],
    );
}

/// 一個已登入的 runtime，連線狀態送進回傳的 channel。
fn signed_in(h: &common::Harness, pat: &str) -> (Arc<TokenManager>, Arc<MemoryCredentialStore>, mpsc::Receiver<ConnectionStateEvent>) {
    let credentials = Arc::new(MemoryCredentialStore::new());
    credentials.set(&h.origin, CredentialKind::Pat, pat).expect("set PAT");
    let (state_tx, state_rx) = mpsc::channel::<ConnectionStateEvent>();
    let runtime = Arc::new(TokenManager::with_connection_state(&h.origin, "conn_x", 1, move |event| {
        let _ = state_tx.send(event);
    }));
    runtime.adopt_access_token(pat);
    (runtime, credentials, state_rx)
}

#[test]
fn a_tab_subscribes_to_the_stream_its_handshake_declared() {
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (runtime, credentials, _states) = signed_in(&h, &pat);

    // remote_open 的那一半：握手，記下宣告的串流（示範用 server 宣告 `/events`）。
    let (workspace, _info) =
        remote::open_workspace(&h.origin, "demo/backend", &runtime, credentials.as_ref()).expect("open");
    let streams = Arc::new(DeclaredStreams::default());
    streams.remember("conn_x", "demo", "backend", workspace.declared_stream().cloned());
    assert_eq!(streams.get("conn_x", "demo", "backend"), Some(common::demo_stream(&project_base(&h.origin))));

    // remote_watch 的那一半。訂閱失敗時 worker 會先輪詢一次並發一筆通知，所以寫入前
    // 先瀝乾；退避設得比等待長，之後的通知只可能來自串流。
    let (notify, rx) = sink();
    let events = EventManager::new(notify);
    register_declared(&events, &streams, &runtime, &credentials, &h.origin, Duration::from_secs(60));
    std::thread::sleep(Duration::from_millis(150));
    drain(&rx);

    writer(&h.origin, &pat).task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&rx, "宣告的串流送來失效提示");
    events.unregister(KEY);
}

#[test]
fn a_tab_with_no_declared_stream_only_polls_and_shows_no_error() {
    // 規格「不同 origin 時不送出憑證並改用輪詢」「握手沒有宣告 sse transport」：沒有
    // 可用的網址時不建立訂閱、靠 sync-state 收斂，而且不算連線失敗。
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (runtime, credentials, states) = signed_in(&h, &pat);
    let streams = Arc::new(DeclaredStreams::default());

    let refused = remote::subscribe_declared(&streams, "conn_x", "demo", "backend", &runtime, credentials.as_ref(), None)
        .err()
        .expect("no stream, no subscription");
    assert!(refused.message.contains("no usable event stream"), "{}", refused.message);
    assert_eq!(refused.status, None, "nothing was requested");

    let (notify, rx) = sink();
    let events = EventManager::new(notify);
    register_declared(&events, &streams, &runtime, &credentials, &h.origin, Duration::from_millis(30));
    std::thread::sleep(Duration::from_millis(150));
    drain(&rx);

    writer(&h.origin, &pat).task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&rx, "沒有串流時靠輪詢收斂");
    assert!(
        states.try_iter().all(|event| event.state != ConnectionState::Offline),
        "沒有串流不是連線失敗"
    );
    events.unregister(KEY);
}

/// 收 remote-workspace-changed 的測試水槽。
fn sink() -> (
    impl Fn(String) + Send + Sync + 'static,
    mpsc::Receiver<String>,
) {
    let (tx, rx) = mpsc::channel();
    (
        move |key: String| {
            let _ = tx.send(key);
        },
        rx,
    )
}

/// 對 demo project 的資料面 client（直連 server、觸發 outbox 事件用）。
fn writer(origin: &str, pat: &str) -> Client {
    Client::new(
        &format!("{origin}/api/speclink/v1/projects/demo"),
        pat,
        Some("backend"),
    )
}

fn project_base(origin_or_proxy: &str) -> String {
    format!("{origin_or_proxy}/api/speclink/v1/projects/demo")
}

/// 把 channel 內既有的通知全部瀝乾。
fn drain(rx: &mpsc::Receiver<String>) {
    while rx.try_recv().is_ok() {}
}

/// 在 timeout 內至少收到一則指定 key 的通知。
fn expect_notified(rx: &mpsc::Receiver<String>, why: &str) {
    let got = rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap_or_else(|_| panic!("{why}: 未收到通知"));
    assert_eq!(got, KEY, "{why}: 通知 payload 是 locator key");
}

// --- 可 sever 的 TCP proxy：強制斷流而不動 server ---

struct Proxy {
    /// proxy 的 http origin（指給 manager 的訂閱面）。
    origin: String,
    live: Arc<Mutex<Vec<TcpStream>>>,
}

fn proxy_to(upstream_origin: &str) -> Proxy {
    let upstream = upstream_origin.trim_start_matches("http://").to_string();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind proxy");
    let addr = listener.local_addr().expect("proxy addr");
    let live: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = live.clone();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(client) = conn else { break };
            let Ok(server) = TcpStream::connect(&upstream) else {
                continue;
            };
            sink.lock()
                .unwrap()
                .push(client.try_clone().expect("clone"));
            sink.lock()
                .unwrap()
                .push(server.try_clone().expect("clone"));
            let (mut c_read, mut s_write) = (
                client.try_clone().expect("clone"),
                server.try_clone().expect("clone"),
            );
            let (mut s_read, mut c_write) = (server, client);
            std::thread::spawn(move || pump(&mut c_read, &mut s_write));
            std::thread::spawn(move || pump(&mut s_read, &mut c_write));
        }
    });
    Proxy {
        origin: format!("http://{addr}"),
        live,
    }
}

fn pump(from: &mut TcpStream, to: &mut TcpStream) {
    let mut buf = [0u8; 4096];
    loop {
        match from.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if to.write_all(&buf[..n]).is_err() {
                    break;
                }
            }
        }
    }
    let _ = to.shutdown(Shutdown::Both);
}

impl Proxy {
    /// 切斷所有既有連線（含 manager 的 SSE 流）；之後的新連線照常通行。
    fn sever(&self) {
        for stream in self.live.lock().unwrap().drain(..) {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}

/// 對 manager 注入的訂閱與 sync-state 閉包（帶連線計數）。
fn register_via(manager: &EventManager, base: &str, pat: &str, connects: &Arc<AtomicUsize>) {
    let sub_url = common::demo_stream(&project_base(base));
    let sub_pat = pat.to_string();
    let counter = connects.clone();
    let etag_base = project_base(base);
    let etag_pat = pat.to_string();
    manager.register(
        KEY,
        move |last| {
            counter.fetch_add(1, Ordering::SeqCst);
            events::subscribe(&sub_url, &sub_pat, Some("backend"), last)
        },
        move || events::sync_state(&etag_base, &etag_pat, Some("backend")),
        vec![Duration::from_millis(30)],
    );
}

// --- invalidate 分發 ---

#[test]
fn an_invalidate_dispatches_the_locator_key_to_the_frontend() {
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (notify, rx) = sink();
    let manager = EventManager::new(notify);
    let connects = Arc::new(AtomicUsize::new(0));
    register_via(&manager, &h.origin, &pat, &connects);
    std::thread::sleep(Duration::from_millis(150));

    writer(&h.origin, &pat)
        .task_done("demo", "1", &[], None)
        .expect("task done");
    expect_notified(&rx, "invalidate 到達即發前端事件");
}

#[test]
fn a_commit_with_two_events_only_reloads() {
    // cross-resource-events-and-event-names：從討論建立變更的一次提交推兩筆
    // invalidate（變更與討論）。Desktop 照舊每筆只發重讀通知，串流不斷、不重連。
    let h = common::harness_with_events(common::fast_events());
    common::put(
        h.store.as_ref(),
        &common::scope(),
        &[(
            "discussions/two.md",
            "---\ntopic: two\nslug: two\nstatus: concluded\ncreated: 2026-11-01\n---\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n**Decision**: x\n",
        )],
    );
    let pat = common::pat_of(&h);
    let (notify, rx) = sink();
    let manager = EventManager::new(notify);
    let connects = Arc::new(AtomicUsize::new(0));
    register_via(&manager, &h.origin, &pat, &connects);
    std::thread::sleep(Duration::from_millis(150));

    writer(&h.origin, &pat)
        .create_change(speclink_protocol::command::CreateChangeRequest {
            name: "add-two".to_string(),
            schema: None,
            description: None,
            agent: None,
            from_discussion: Some("two".to_string()),
            last: false,
            fields: Default::default(),
        })
        .expect("create change from discussion");
    expect_notified(&rx, "變更的 invalidate");
    expect_notified(&rx, "討論的 invalidate");
    assert_eq!(connects.load(Ordering::SeqCst), 1, "兩筆都在同一條串流上，沒有重連");
}

// --- 同 connection 兩 session 共用單一訂閱 ---

#[test]
fn two_sessions_on_one_connection_share_a_single_subscription() {
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (notify, rx) = sink();
    let manager = EventManager::new(notify);
    let connects = Arc::new(AtomicUsize::new(0));

    register_via(&manager, &h.origin, &pat, &connects);
    register_via(&manager, &h.origin, &pat, &connects);
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        connects.load(Ordering::SeqCst),
        1,
        "兩 session 共用單一訂閱"
    );

    let w = writer(&h.origin, &pat);
    w.task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&rx, "共用訂閱仍分發");
    assert!(
        rx.recv_timeout(Duration::from_millis(350)).is_err(),
        "同 key 的事件只通知一次（session 以 key 對應、不重複）"
    );

    // 退掉一個 session：流仍在。
    manager.unregister(KEY);
    drain(&rx);
    w.task_done("demo", "2", &[], None).expect("task done");
    expect_notified(&rx, "仍有 session 註冊時流不得中止");

    // 退掉最後一個 session：流收束、不再通知也不再重連。
    manager.unregister(KEY);
    std::thread::sleep(Duration::from_millis(300));
    drain(&rx);
    w.task_done("demo", "3", &[], None).expect("task done");
    assert!(
        rx.recv_timeout(Duration::from_millis(400)).is_err(),
        "最後一個 session 退出後訂閱收束"
    );
    assert_eq!(connects.load(Ordering::SeqCst), 1, "全程只開過一條流");
}

// --- 斷流 → ETag 收斂 → 退避重連＋Last-Event-ID 續傳 ---

#[test]
fn a_severed_stream_converges_via_etag_and_resumes_live() {
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let proxy = proxy_to(&h.origin);
    let (notify, rx) = sink();
    let manager = EventManager::new(notify);
    let connects = Arc::new(AtomicUsize::new(0));
    register_via(&manager, &proxy.origin, &pat, &connects);
    std::thread::sleep(Duration::from_millis(150));

    let w = writer(&h.origin, &pat);
    w.task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&rx, "斷流前的基線事件");
    drain(&rx);

    // 強制斷流；斷線期間 server 側發生變更。
    proxy.sever();
    w.task_done("demo", "2", &[], None)
        .expect("task done while severed");
    // 收斂：sync-state ETag 相異即發重載通知（Query 為重讀正典），並以注入
    // 退避序列重連、Last-Event-ID 續傳——錯過的變更不得遺漏。
    expect_notified(&rx, "斷線期間的變更經收斂反映");

    // 重連後的新事件走正常推播。
    drain(&rx);
    w.task_done("demo", "3", &[], None)
        .expect("task done after recovery");
    expect_notified(&rx, "續訂後新事件照常分發");
    assert!(
        connects.load(Ordering::SeqCst) >= 2,
        "斷流後經重連（計數前進）"
    );
}

// --- reset 信號 → 全量重載通知 → 自新位點續訂 ---

#[test]
fn a_reset_triggers_a_full_reload_notification_and_a_stable_fresh_subscription() {
    // retention 2：斷線期間的寫入把 acked floor 推過 manager 的續傳位點，
    // 重連的 Last-Event-ID 續傳被 server 以 reset 回應。
    let h = common::harness_with_events(EventSettings {
        retention: 2,
        buffer: 64,
        heartbeat: Duration::from_millis(100),
    });
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);

    // 一條測試自有的直連訂閱持續瀝乾——讓 server 的保留期 floor 前進。
    let mut driver = events::subscribe(&common::demo_stream(&project_base(&h.origin)), &pat, Some("backend"), None)
        .expect("driver subscribe");
    let driver_abort = driver.abort_handle();
    std::thread::spawn(move || while let Ok(Some(_)) = driver.next() {});

    let proxy = proxy_to(&h.origin);
    let (notify, rx) = sink();
    let manager = EventManager::new(notify);
    let connects = Arc::new(AtomicUsize::new(0));
    register_via(&manager, &proxy.origin, &pat, &connects);
    std::thread::sleep(Duration::from_millis(150));

    let w = writer(&h.origin, &pat);
    w.task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&rx, "基線事件（manager 持有續傳位點）");
    drain(&rx);

    // 斷流後連寫四筆：driver 瀝乾使 floor 越過 manager 的位點。
    proxy.sever();
    for task in ["2", "3", "4", "1"] {
        // 最後一筆是 undone——只為多產生一個 outbox 事件。
        if task == "1" {
            w.task_undone("demo", task).expect("undone");
        } else {
            w.task_done("demo", task, &[], None)
                .expect("task done while severed");
        }
    }
    std::thread::sleep(Duration::from_millis(300));

    // 重連的續傳位點已被清理——server 回 reset，manager 發全量重載通知。
    expect_notified(&rx, "reset 信號觸發全量重載通知");

    // 自新位點續訂：新事件照常分發，且流保持穩定（不重連震盪）。
    drain(&rx);
    w.task_done("demo", "1", &[], None).expect("a fresh write");
    expect_notified(&rx, "reset 後自新位點續訂");
    let settled = connects.load(Ordering::SeqCst);
    drain(&rx);
    w.task_undone("demo", "1").expect("another fresh write");
    expect_notified(&rx, "續訂後的流持續分發");
    assert_eq!(
        connects.load(Ordering::SeqCst),
        settled,
        "reset 後的流是穩定的單一訂閱、不重連震盪"
    );

    driver_abort.abort();
}

// --- 存取被拒由輪詢的 403 判定、以分頁為鍵（remote-resilience「存取被拒的狀態跟著分頁」） ---

use speclink_desktop_lib::event_manager::{AccessNotice, AccessState};

const DISABLED: &str = r#"{"status":403,"reason":"permission_denied","message":"帳號已停用"}"#;
const NOT_A_MEMBER: &str =
    r#"{"status":403,"reason":"permission_denied","message":"actor is not a member of project 'demo'"}"#;

/// 一個 worker 的輪詢下一次會得到什麼。
#[derive(Clone, Copy)]
enum Poll {
    Ok,
    /// 403，本文是這段文字。
    Refused(&'static str),
    Unreachable,
}

/// 腳本化的訂閱：串流一律開不起來（`subscribe_error`），輪詢照 `poll` 回應。
struct Scripted {
    poll: Arc<Mutex<Poll>>,
    subscribes: Arc<AtomicUsize>,
}

impl Scripted {
    fn register(events: &EventManager, key: &str, subscribe_error: fn() -> speclink_remote::RemoteError) -> Scripted {
        let scripted = Scripted { poll: Arc::new(Mutex::new(Poll::Ok)), subscribes: Arc::default() };
        let (poll, subscribes) = (scripted.poll.clone(), scripted.subscribes.clone());
        events.register(
            key,
            move |_| {
                subscribes.fetch_add(1, Ordering::SeqCst);
                Err(subscribe_error())
            },
            move || match *poll.lock().unwrap() {
                Poll::Ok => Ok("\"7\"".to_string()),
                Poll::Refused(body) => Err(speclink_remote::translate_protocol_error(403, body)),
                Poll::Unreachable => Err(speclink_remote::translate_transport()),
            },
            vec![Duration::from_millis(5)],
        );
        scripted
    }

    fn next(&self, poll: Poll) {
        *self.poll.lock().unwrap() = poll;
    }
}

fn access_sink() -> (impl Fn(AccessNotice) + Send + Sync + 'static, mpsc::Receiver<AccessNotice>) {
    let (tx, rx) = mpsc::channel();
    (
        move |notice: AccessNotice| {
            let _ = tx.send(notice);
        },
        rx,
    )
}

fn expect_access(rx: &mpsc::Receiver<AccessNotice>, why: &str) -> AccessNotice {
    rx.recv_timeout(Duration::from_secs(5)).unwrap_or_else(|_| panic!("{why}: 未收到存取狀態通知"))
}

/// 許多輪退避（每輪 5 毫秒）之內沒有新的存取狀態通知。
fn expect_no_access(rx: &mpsc::Receiver<AccessNotice>, why: &str) {
    if let Ok(notice) = rx.recv_timeout(Duration::from_millis(150)) {
        panic!("{why}: 多了一則通知 {notice:?}");
    }
}

fn denied(key: &str, message: Option<&str>) -> AccessNotice {
    AccessNotice { key: key.to_string(), denied: true, message: message.map(str::to_string) }
}

fn regained(key: &str) -> AccessNotice {
    AccessNotice { key: key.to_string(), denied: false, message: None }
}

#[test]
fn a_403_poll_denies_the_tab_once_and_a_good_poll_clears_it_with_one_reload() {
    let (notify, reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);

    // 第一次輪詢成功：只是一般的收斂，沒有存取狀態通知。
    expect_notified(&reloads, "第一次收斂");
    expect_no_access(&access, "輪詢成功時");
    drain(&reloads);

    tab.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "輪詢回 403"), denied(KEY, Some("帳號已停用")));
    expect_no_access(&access, "原因相同時不重複通知");
    assert!(reloads.try_recv().is_err(), "被拒期間不發重新載入");

    tab.next(Poll::Ok);
    assert_eq!(expect_access(&access, "輪詢成功即解除"), regained(KEY));
    expect_notified(&reloads, "解除時重新載入");
    std::thread::sleep(Duration::from_millis(100));
    assert!(reloads.try_recv().is_err(), "解除只重新載入一次（ETag 沒變）");
    expect_no_access(&access, "解除之後");
    events.unregister(KEY);
}

#[test]
fn a_403_with_no_readable_reason_denies_without_a_message_and_a_new_reason_is_told() {
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);

    tab.next(Poll::Refused("<html>Forbidden</html>"));
    assert_eq!(expect_access(&access, "403 而沒有信封"), denied(KEY, None));

    tab.next(Poll::Refused(NOT_A_MEMBER));
    assert_eq!(
        expect_access(&access, "原因改變"),
        denied(KEY, Some("actor is not a member of project 'demo'"))
    );
    expect_no_access(&access, "原因相同時不重複通知");
    events.unregister(KEY);
}

#[test]
fn a_denied_tab_stays_denied_while_the_server_cannot_be_reached() {
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);

    tab.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "輪詢回 403"), denied(KEY, Some("帳號已停用")));

    tab.next(Poll::Unreachable);
    expect_no_access(&access, "連不上時維持被拒");

    tab.next(Poll::Ok);
    assert_eq!(expect_access(&access, "輪詢成功即解除"), regained(KEY));
    events.unregister(KEY);
}

#[test]
fn a_refused_subscription_decides_nothing_and_the_poll_after_it_does() {
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    // 訂閱被 403 拒絕，輪詢卻成功：不算被拒（只認輪詢的結果）。
    let tab = Scripted::register(&events, KEY, || speclink_remote::translate_protocol_error(403, DISABLED));
    expect_no_access(&access, "訂閱的 403 不直接判定");

    tab.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "接著的輪詢回 403"), denied(KEY, Some("帳號已停用")));
    events.unregister(KEY);
}

#[test]
fn while_a_tab_is_denied_the_worker_polls_and_does_not_subscribe() {
    // 被拒期間只輪詢：重新訂閱若先成功，worker 就停在串流上、不再輪詢，「被拒」
    // 便沒有解除的機會。解除之後才再訂閱。
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);

    tab.next(Poll::Refused(DISABLED));
    expect_access(&access, "輪詢回 403");
    std::thread::sleep(Duration::from_millis(30));
    let while_denied = tab.subscribes.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(tab.subscribes.load(Ordering::SeqCst), while_denied, "被拒期間不訂閱");

    tab.next(Poll::Ok);
    expect_access(&access, "輪詢成功即解除");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while tab.subscribes.load(Ordering::SeqCst) == while_denied && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(tab.subscribes.load(Ordering::SeqCst) > while_denied, "解除之後再訂閱");
    events.unregister(KEY);
}

#[test]
fn another_tab_of_the_same_connection_is_not_denied() {
    const OTHER: &str = "remote:conn_x/beta/backend";
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let alpha = Scripted::register(&events, KEY, speclink_remote::translate_transport);
    let _beta = Scripted::register(&events, OTHER, speclink_remote::translate_transport);

    alpha.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "alpha 的輪詢回 403"), denied(KEY, Some("帳號已停用")));
    expect_no_access(&access, "beta 的輪詢照常成功");
    events.unregister(KEY);
    events.unregister(OTHER);
}

#[test]
fn the_manager_answers_the_access_state_of_a_key_while_someone_watches_it() {
    // 寫入守門問的就是這個答案：沒有另一份要保持同步的登記。
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    assert_eq!(events.access_of(KEY), AccessState::Allowed, "沒有訂閱的 key 沒有人在判定");
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);
    // 同一個鍵的第二個 session：共用同一條訂閱。
    events.register(KEY, |_| Err(speclink_remote::translate_transport()), || Ok(String::new()), Vec::new());

    tab.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "輪詢回 403"), denied(KEY, Some("帳號已停用")));
    assert_eq!(events.access_of(KEY), AccessState::Denied { reason: Some("帳號已停用".to_string()) });
    assert_eq!(events.access_of("remote:conn_x/beta/backend"), AccessState::Allowed, "別的 key 不受影響");

    events.unregister(KEY);
    assert!(matches!(events.access_of(KEY), AccessState::Denied { .. }), "還有一個 session 在");

    // 最後一個 session 退訂：訂閱收束，這個 key 不再有人判定。收束不是「解除」，不送通知；
    // 收束之後這條訂閱也不再改變狀態。
    events.unregister(KEY);
    assert_eq!(events.access_of(KEY), AccessState::Allowed);
    tab.next(Poll::Refused(NOT_A_MEMBER));
    expect_no_access(&access, "收束之後");

    // 同一個鍵再訂閱是一條新的訂閱：仍被拒的話重新判定、重新通知。
    let again = Scripted::register(&events, KEY, speclink_remote::translate_transport);
    again.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "新的訂閱重新判定"), denied(KEY, Some("帳號已停用")));
    events.unregister(KEY);
}

#[test]
fn a_session_that_joins_a_denied_subscription_is_told_the_state_again() {
    // 同一個 key 重新訂閱時，新的 register 可能比舊的 unregister 先到：舊的 worker 被沿用，
    // 而它在原因相同時不再通知。加入的一方要拿到目前的狀態，畫面才不會以為沒有被拒。
    let (notify, _reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    let tab = Scripted::register(&events, KEY, speclink_remote::translate_transport);
    tab.next(Poll::Refused(DISABLED));
    assert_eq!(expect_access(&access, "輪詢回 403"), denied(KEY, Some("帳號已停用")));
    expect_no_access(&access, "原因相同時不重複通知");

    events.register(KEY, |_| Err(speclink_remote::translate_transport()), || Ok(String::new()), Vec::new());
    assert_eq!(expect_access(&access, "加入既有的訂閱"), denied(KEY, Some("帳號已停用")));

    // 沒有被拒的訂閱：加入時沒有東西要補。
    const OTHER: &str = "remote:conn_x/beta/backend";
    let _other = Scripted::register(&events, OTHER, speclink_remote::translate_transport);
    events.register(OTHER, |_| Err(speclink_remote::translate_transport()), || Ok(String::new()), Vec::new());
    expect_no_access(&access, "沒有被拒的訂閱");
    for key in [KEY, KEY, OTHER, OTHER] {
        events.unregister(key);
    }
}

#[test]
fn a_member_removed_from_the_project_is_denied_with_the_servers_reason_and_recovers() {
    // 只用輪詢的分頁（握手沒有宣告串流），接真的示範用 Server。
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (runtime, credentials, states) = signed_in(&h, &pat);
    let streams = Arc::new(DeclaredStreams::default());
    let (notify, reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    register_declared(&events, &streams, &runtime, &credentials, &h.origin, Duration::from_millis(30));
    expect_notified(&reloads, "第一次收斂");

    let membership = |member: bool| {
        speclink_server::identity::IdentityStore::admin_set_membership(
            h.identity.as_ref(),
            &speclink_server::audit::AuditActor::system_cli(),
            &h.user_id,
            "demo",
            speclink_server::identity::MembershipRole::Editor,
            member,
        )
        .expect("set membership");
    };

    membership(false);
    assert_eq!(
        expect_access(&access, "被移出專案後的輪詢"),
        denied(KEY, Some("actor is not a member of project 'demo'"))
    );
    drain(&reloads);

    membership(true);
    assert_eq!(expect_access(&access, "加回專案後的輪詢"), regained(KEY));
    expect_notified(&reloads, "恢復存取後重新載入");
    assert!(
        states.try_iter().all(|event| event.state != ConnectionState::Offline),
        "403 不計入離線（這條連線的門檻是 1）"
    );
    events.unregister(KEY);
}

#[test]
fn a_stream_the_server_closes_for_a_removed_member_ends_in_the_same_denied_state() {
    // spec「推送串流被 Server 關閉後偵測到被拒」：示範用 Server 在心跳間隔發現使用者不再是
    // 成員而結束串流；重新訂閱與接著的輪詢都回 403，分頁進入存取被拒，與只用輪詢時相同。
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (runtime, credentials, _states) = signed_in(&h, &pat);
    let (workspace, _info) =
        remote::open_workspace(&h.origin, "demo/backend", &runtime, credentials.as_ref()).expect("open");
    let streams = Arc::new(DeclaredStreams::default());
    streams.remember("conn_x", "demo", "backend", workspace.declared_stream().cloned());

    let (notify, reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    register_declared(&events, &streams, &runtime, &credentials, &h.origin, Duration::from_millis(30));
    // 串流確實開著：一筆寫入經串流送來失效提示。
    std::thread::sleep(Duration::from_millis(150));
    drain(&reloads);
    writer(&h.origin, &pat).task_done("demo", "1", &[], None).expect("task done");
    expect_notified(&reloads, "串流送來失效提示");

    speclink_server::identity::IdentityStore::admin_set_membership(
        h.identity.as_ref(),
        &speclink_server::audit::AuditActor::system_cli(),
        &h.user_id,
        "demo",
        speclink_server::identity::MembershipRole::Editor,
        false,
    )
    .expect("remove the member");

    assert_eq!(
        expect_access(&access, "串流被關閉後的輪詢"),
        denied(KEY, Some("actor is not a member of project 'demo'"))
    );
    events.unregister(KEY);
}

#[test]
fn a_denied_tab_whose_server_went_down_shows_denied_again_when_the_server_returns() {
    // spec「離線期間 Server 以 403 回應即回 online」：被拒期間 worker 只輪詢，而輪詢只會得到 403。
    // Server 斷線又恢復之後沒有任何成功的請求；連線仍要回 online，分頁才不會停在離線。
    let h = common::harness_with_events(common::fast_events());
    common::seed_change(h.store.as_ref(), FOUR_TASKS);
    let pat = common::pat_of(&h);
    let (runtime, credentials, states) = signed_in(&h, &pat);
    let streams = Arc::new(DeclaredStreams::default());
    let (notify, reloads) = sink();
    let (on_access, access) = access_sink();
    let events = EventManager::with_access_observer(notify, on_access);
    register_declared(&events, &streams, &runtime, &credentials, &h.origin, Duration::from_millis(30));
    expect_notified(&reloads, "第一次收斂");

    speclink_server::identity::IdentityStore::admin_set_membership(
        h.identity.as_ref(),
        &speclink_server::audit::AuditActor::system_cli(),
        &h.user_id,
        "demo",
        speclink_server::identity::MembershipRole::Editor,
        false,
    )
    .expect("remove the member");
    assert_eq!(
        expect_access(&access, "被移出專案後的輪詢"),
        denied(KEY, Some("actor is not a member of project 'demo'"))
    );

    // Windows 對 refused connect 會在 winsock 內部重試，上限只防吊死。
    h.server.stop();
    let offline = states.recv_timeout(Duration::from_secs(15)).expect("Server 停止後連線轉 offline");
    assert_eq!(offline.state, ConnectionState::Offline);

    h.server.start();
    let online = states.recv_timeout(Duration::from_secs(15)).expect("Server 回來後輪詢回 403，連線仍回 online");
    assert_eq!(online.state, ConnectionState::Online);
    assert!(matches!(events.access_of(KEY), AccessState::Denied { .. }), "分頁仍是存取被拒");
    assert!(access.try_recv().is_err(), "沒有解除的通知");
    events.unregister(KEY);
}

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
use speclink_desktop_lib::remote::{ConnectionState, ConnectionStateEvent, TokenManager};
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
    let sub_base = project_base(&h.origin);
    let sub_runtime = runtime.clone();
    let sub_credentials = credentials.clone();
    let sync_base = project_base(&h.origin);
    let sync_runtime = runtime.clone();
    let sync_credentials = credentials.clone();
    events.register(
        KEY,
        move |last| {
            sub_runtime.execute(sub_credentials.as_ref(), |token| {
                events::subscribe(&sub_base, token, Some("backend"), last)
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
    let sub_base = project_base(base);
    let sub_pat = pat.to_string();
    let counter = connects.clone();
    let etag_base = project_base(base);
    let etag_pat = pat.to_string();
    manager.register(
        KEY,
        move |last| {
            counter.fetch_add(1, Ordering::SeqCst);
            events::subscribe(&sub_base, &sub_pat, Some("backend"), last)
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
    let mut driver = events::subscribe(&project_base(&h.origin), &pat, Some("backend"), None)
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

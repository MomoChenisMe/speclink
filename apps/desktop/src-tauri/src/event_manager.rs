//! per-connection 事件管理者（design 決策 3、5；規格「Query 加 ETag 為重讀
//! 正典且 push 只做 invalidate」「斷線以 Polling 加 ETag 收斂後續訂」）。
//!
//! 以 locator key（connection＋project/repo）為訂閱鍵：server 的 /events 是
//! project-scoped＋repo header，同 scope 的 sessions 以參照計數共用單一條流、
//! 事件以 key 分發（每事件每 key 恰一次通知）。收 invalidate 只轉發失效提示
//! ——payload＝locator key，前端一律經 Query 重讀，這裡不攜帶、不快取任何
//! 資料實體。
//!
//! 斷線收斂程序（決策 5）：(1) 停流；(2) 以 /sync-state ETag 與已知值比對
//! ——不同即發重載通知（已知值由 invalidate 的 revision 推導：ETag＝加引號的
//! project revision，零額外請求）；(3) 依注入的退避序列重連並帶 Last-Event-ID
//! 續傳。SSE 持續不可用期間，每輪退避都跑一次 ETag 比對——輪詢即心跳。收到
//! reset 信號：發全量重載通知、位點歸零，自同一條流的新位點續訂。
//!
//! 已知邊界：server 若以全新 outbox 重啟（例如 memory store），過大的
//! Last-Event-ID 會使續傳靜默失聰——重連當下的 ETag 比對會補發一次重載通知，
//! 其後的收斂依賴下一次斷線或重訂（sqlite/pg 後端的 outbox 跨重啟連續，
//! 不受此影響）。

use speclink_remote::events::{EventStream, RemoteEvent};
use speclink_remote::RemoteError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 一條共用訂閱的控制面：參照計數＋收束把手。
struct SubEntry {
    count: usize,
    shutdown: Arc<AtomicBool>,
    /// 目前開著的流的中止把手（worker 每次開流後掛上）。
    stream_abort: Arc<Mutex<Option<speclink_remote::events::AbortHandle>>>,
}

/// remote 事件的分發中樞：tauri 接線注入 notify（emit remote-workspace-changed），
/// 測試注入 channel 水槽。
pub struct EventManager {
    notify: Arc<dyn Fn(String) + Send + Sync>,
    subs: Mutex<HashMap<String, SubEntry>>,
}

impl Drop for EventManager {
    fn drop(&mut self) {
        let mut subs = self.subs.lock().expect("subs lock");
        for (_, entry) in subs.drain() {
            entry.shutdown.store(true, Ordering::Relaxed);
            if let Some(abort) = entry.stream_abort.lock().expect("abort lock").take() {
                abort.abort();
            }
        }
    }
}

impl EventManager {
    pub fn new(notify: impl Fn(String) + Send + Sync + 'static) -> EventManager {
        EventManager {
            notify: Arc::new(notify),
            subs: Mutex::new(HashMap::new()),
        }
    }

    /// 註冊一個 session：同 key 已有訂閱即共用（參照計數＋1，閉包棄用），
    /// 否則以給定的訂閱／sync-state 閉包與退避序列開 worker 執行緒。
    pub fn register<S, P>(&self, key: &str, subscribe: S, sync_state: P, backoff: Vec<Duration>)
    where
        S: Fn(Option<u64>) -> Result<EventStream, RemoteError> + Send + 'static,
        P: Fn() -> Result<String, RemoteError> + Send + 'static,
    {
        let mut subs = self.subs.lock().expect("subs lock");
        if let Some(entry) = subs.get_mut(key) {
            entry.count += 1;
            return;
        }
        let shutdown = Arc::new(AtomicBool::new(false));
        let stream_abort: Arc<Mutex<Option<speclink_remote::events::AbortHandle>>> =
            Arc::new(Mutex::new(None));
        subs.insert(
            key.to_string(),
            SubEntry {
                count: 1,
                shutdown: shutdown.clone(),
                stream_abort: stream_abort.clone(),
            },
        );
        let notify = Arc::clone(&self.notify);
        let key = key.to_string();
        std::thread::spawn(move || {
            worker(
                key,
                notify,
                subscribe,
                sync_state,
                backoff,
                shutdown,
                stream_abort,
            )
        });
    }

    /// 退出一個 session：參照計數歸零時收束訂閱（中止流、worker 退場）。
    pub fn unregister(&self, key: &str) {
        let mut subs = self.subs.lock().expect("subs lock");
        let Some(entry) = subs.get_mut(key) else {
            return;
        };
        entry.count = entry.count.saturating_sub(1);
        if entry.count == 0 {
            let entry = subs.remove(key).expect("entry exists");
            entry.shutdown.store(true, Ordering::Relaxed);
            let abort = entry.stream_abort.lock().expect("abort lock").take();
            if let Some(abort) = abort {
                abort.abort();
            }
        }
    }
}

/// 一條訂閱的 worker：開流→分發→斷線收斂→重連，直到 shutdown。
fn worker(
    key: String,
    notify: Arc<dyn Fn(String) + Send + Sync>,
    subscribe: impl Fn(Option<u64>) -> Result<EventStream, RemoteError>,
    sync_state: impl Fn() -> Result<String, RemoteError>,
    backoff: Vec<Duration>,
    shutdown: Arc<AtomicBool>,
    stream_abort: Arc<Mutex<Option<speclink_remote::events::AbortHandle>>>,
) {
    let mut last_event_id: Option<u64> = None;
    let mut last_etag: Option<String> = None;
    let mut attempt = 0usize;
    let mut recovery_episode = false;
    let mut recovery_invalidation_sent = false;
    loop {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        if let Ok(mut stream) = subscribe(last_event_id) {
            // 曾斷線後第一次成功重訂即全量失效；若前一輪 sync-state 已送過，
            // 此處只結束 recovery episode，不重複通知。
            if recovery_episode && !recovery_invalidation_sent {
                notify(key.clone());
            }
            let abort = stream.abort_handle();
            *stream_abort.lock().expect("abort lock") = Some(abort.clone());
            // shutdown 恰於掛上把手前發生：自行收束。
            if shutdown.load(Ordering::Relaxed) {
                abort.abort();
                return;
            }
            attempt = 0;
            loop {
                match stream.next() {
                    Ok(Some(RemoteEvent::Invalidate(hint))) => {
                        if let Ok(seq) = hint.event_id.parse::<u64>() {
                            last_event_id = Some(seq);
                        }
                        // ETag＝加引號的 project revision（server 的 scope token
                        // 形制）——由事件推導，收斂比對零額外請求。
                        last_etag = Some(format!("\"{}\"", hint.revision));
                        notify(key.clone());
                    }
                    Ok(Some(RemoteEvent::Reset)) => {
                        // 續傳位點已過保留期：全量重載，位點歸零，自同一條流
                        // 的新位點續訂。
                        last_event_id = None;
                        last_etag = None;
                        notify(key.clone());
                    }
                    // Ok(None)＝中止把手收束（unregister）。
                    Ok(None) => return,
                    // 斷線：進收斂程序。
                    Err(_) => {
                        recovery_episode = true;
                        recovery_invalidation_sent = false;
                        break;
                    }
                }
            }
            *stream_abort.lock().expect("abort lock") = None;
        } else if !recovery_episode {
            recovery_episode = true;
            recovery_invalidation_sent = false;
        }
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        // 收斂 (2)：ETag 相異即發重載通知——完全漏掉 push 仍經 Query 收斂。
        // SSE 不可用期間每輪退避都經過這裡：輪詢即心跳。
        if let Ok(etag) = sync_state() {
            let changed = last_etag.as_deref() != Some(etag.as_str());
            if changed {
                last_etag = Some(etag);
                notify(key.clone());
            }
            // 同一斷線 episode 第一次成功收斂必送一次全量失效；ETag 已變時
            // 上方通知即同一筆，不再重複。SSE 持續失敗時後續輪詢也不洗版。
            if recovery_episode && !recovery_invalidation_sent {
                if !changed {
                    notify(key.clone());
                }
                recovery_invalidation_sent = true;
            }
        }
        // 收斂 (3)：退避後重連（Last-Event-ID 續傳）。序列走完即停在最末值。
        let delay = backoff
            .get(attempt.min(backoff.len().saturating_sub(1)))
            .copied()
            .unwrap_or(Duration::from_secs(5));
        attempt = attempt.saturating_add(1);
        sleep_interruptible(delay, &shutdown);
    }
}

/// 可被 shutdown 打斷的睡眠（25ms 切片）。
fn sleep_interruptible(total: Duration, shutdown: &AtomicBool) {
    let slice = Duration::from_millis(25);
    let mut slept = Duration::ZERO;
    while slept < total {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        let step = slice.min(total - slept);
        std::thread::sleep(step);
        slept += step;
    }
}

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
//! 存取被拒（remote-resilience「存取被拒的狀態跟著分頁」）：收斂程序的輪詢回
//! 403 時，這個 key 進入「被拒」並以（key、Server 的原因）通知；之後的輪詢成功
//! 即解除，並發一次重載通知。只有輪詢判定——訂閱被拒不算，它之後一定接著輪詢。
//! 被拒期間只輪詢、不訂閱：重新訂閱若先成功，worker 會停在串流上不再輪詢，
//! 「被拒」便沒有解除的機會。
//!
//! 已知邊界：server 若以全新 outbox 重啟（例如 memory store），過大的
//! Last-Event-ID 會使續傳靜默失聰——重連當下的 ETag 比對會補發一次重載通知，
//! 其後的收斂依賴下一次斷線或重訂（sqlite/pg 後端的 outbox 跨重啟連續，
//! 不受此影響）。

use serde::Serialize;
use speclink_remote::events::{EventStream, RemoteEvent};
use speclink_remote::RemoteError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 一個 key 的存取狀態通知，也是 `remote-access-state` 事件的 payload：
/// `denied` 為 true 時 `message` 是 Server 回報的原因（沒有可讀的原因時是 None）；
/// 為 false 時是解除，`message` 恆為 None。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessNotice {
    pub key: String,
    pub denied: bool,
    pub message: Option<String>,
}

/// 一個 key 目前的存取狀態。沒有訂閱的 key 沒有人在判定，視同 `Allowed`。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AccessState {
    #[default]
    Allowed,
    /// 輪詢回 403。`reason` 是 Server 回報的原因；沒有可讀的原因時是 None。
    Denied { reason: Option<String> },
}

impl AccessState {
    /// 這個狀態對 `key` 的通知。
    fn notice(&self, key: &str) -> AccessNotice {
        let (denied, message) = match self {
            AccessState::Allowed => (false, None),
            AccessState::Denied { reason } => (true, reason.clone()),
        };
        AccessNotice { key: key.to_string(), denied, message }
    }
}

/// 一條訂閱的存取狀態。它就是寫入守門問的那一份（[`EventManager::access_of`]），
/// 沒有另一份要同步的登記。退訂（`closed`）之後 worker 不再改它。
#[derive(Default)]
struct Access {
    state: AccessState,
    closed: bool,
}

/// worker 對外的出口：這個 key 的重載通知與存取狀態。
struct Outlet {
    key: String,
    notify: Arc<dyn Fn(String) + Send + Sync>,
    on_access: Arc<dyn Fn(AccessNotice) + Send + Sync>,
    access: Arc<Mutex<Access>>,
}

impl Outlet {
    /// 發一筆重載通知。
    fn reload(&self) {
        (self.notify)(self.key.clone());
    }

    /// 把存取狀態設成 `state`，有變才通知；回傳這次是否由「被拒」解除。訂閱已退訂時
    /// 什麼都不做。通知在放開鎖之後才送：回呼會做什麼由接線的一方決定，不能讓它
    /// 擋住等同一把鎖的退訂。
    fn set_access(&self, state: AccessState) -> bool {
        {
            let mut access = self.access.lock().expect("access lock");
            if access.closed || access.state == state {
                return false;
            }
            access.state = state.clone();
        }
        (self.on_access)(state.notice(&self.key));
        state == AccessState::Allowed
    }
}

/// 一條共用訂閱的控制面：參照計數＋收束把手。
struct SubEntry {
    count: usize,
    shutdown: Arc<AtomicBool>,
    /// 目前開著的流的中止把手（worker 每次開流後掛上）。
    stream_abort: Arc<Mutex<Option<speclink_remote::events::AbortHandle>>>,
    access: Arc<Mutex<Access>>,
}

/// 訂閱表：共用訂閱之外，另記兩種帳目，讓 register／unregister 在任何抵達
/// 順序下總帳都平（async IPC 後順序不再保證，cleanup 可先於 subscribe 抵達）：
///
/// - `debts`——先到的 unregister：對不存在的 key 退訂＝記一筆額度，該 key 的
///   下一個 register（或失敗註冊的入帳）直接被抵銷、不開 worker。
/// - `credits`——註冊失敗的入帳（[`EventManager::register_failed`]）：註冊沒
///   發生但前端 unmount 照樣退訂，先到的失敗額度讓那筆退訂變 no-op，不毒害
///   之後的訂閱。
///
/// 每筆 register（成功、失敗皆入帳）＋1、每筆 unregister −1，兩種額度互不
/// 共存（入帳先消 debt、退訂先消 credit）——收斂後帳目歸零：不會留下沒人
/// 退訂的孤兒 worker，也不會留下吃掉未來訂閱的毒藥額度（代價是罕見交錯下
/// 訂閱短暫斷開重建，可接受）。
#[derive(Default)]
struct Subs {
    entries: HashMap<String, SubEntry>,
    debts: HashMap<String, usize>,
    credits: HashMap<String, usize>,
}

/// remote 事件的分發中樞：tauri 接線注入 notify（emit remote-workspace-changed）
/// 與 on_access（emit remote-access-state），測試注入 channel 水槽。
pub struct EventManager {
    notify: Arc<dyn Fn(String) + Send + Sync>,
    on_access: Arc<dyn Fn(AccessNotice) + Send + Sync>,
    subs: Mutex<Subs>,
}

impl Drop for EventManager {
    fn drop(&mut self) {
        let mut subs = self.subs.lock().expect("subs lock");
        for (_, entry) in subs.entries.drain() {
            entry.access.lock().expect("access lock").closed = true;
            entry.shutdown.store(true, Ordering::Relaxed);
            if let Some(abort) = entry.stream_abort.lock().expect("abort lock").take() {
                abort.abort();
            }
        }
    }
}

impl EventManager {
    /// 不關心存取狀態的中樞（通知被丟掉）。
    pub fn new(notify: impl Fn(String) + Send + Sync + 'static) -> EventManager {
        Self::with_access_observer(notify, |_| {})
    }

    /// `on_access` 收每個 key 進入與解除「被拒」的通知。
    pub fn with_access_observer(
        notify: impl Fn(String) + Send + Sync + 'static,
        on_access: impl Fn(AccessNotice) + Send + Sync + 'static,
    ) -> EventManager {
        EventManager {
            notify: Arc::new(notify),
            on_access: Arc::new(on_access),
            subs: Mutex::new(Subs::default()),
        }
    }

    /// 這個 key 目前的存取狀態：寫入守門在送出寫入之前問它。
    pub fn access_of(&self, key: &str) -> AccessState {
        let subs = self.subs.lock().expect("subs lock");
        match subs.entries.get(key) {
            Some(entry) => entry.access.lock().expect("access lock").state.clone(),
            None => AccessState::Allowed,
        }
    }

    /// 註冊一個 session：先消掉先到的 unregister 額度（見 [`Subs`]），
    /// 其次同 key 已有訂閱即共用（參照計數＋1，閉包棄用）——這條訂閱正在
    /// 「被拒」時，把目前的狀態再通知一次：加入的一方（例如同一個分頁重新訂閱，
    /// 而新的 register 比舊的 unregister 先到）不會由 worker 得到，worker 在原因
    /// 相同時不重複通知。
    /// 否則以給定的訂閱／sync-state 閉包與退避序列開 worker 執行緒。
    pub fn register<S, P>(&self, key: &str, subscribe: S, sync_state: P, backoff: Vec<Duration>)
    where
        S: Fn(Option<u64>) -> Result<EventStream, RemoteError> + Send + 'static,
        P: Fn() -> Result<String, RemoteError> + Send + 'static,
    {
        let mut subs = self.subs.lock().expect("subs lock");
        if let Some(debt) = subs.debts.get_mut(key) {
            *debt -= 1;
            if *debt == 0 {
                subs.debts.remove(key);
            }
            return;
        }
        if let Some(entry) = subs.entries.get_mut(key) {
            entry.count += 1;
            let state = entry.access.lock().expect("access lock").state.clone();
            drop(subs);
            if state != AccessState::Allowed {
                (self.on_access)(state.notice(key));
            }
            return;
        }
        let shutdown = Arc::new(AtomicBool::new(false));
        let stream_abort: Arc<Mutex<Option<speclink_remote::events::AbortHandle>>> =
            Arc::new(Mutex::new(None));
        let access = Arc::new(Mutex::new(Access::default()));
        subs.entries.insert(
            key.to_string(),
            SubEntry {
                count: 1,
                shutdown: shutdown.clone(),
                stream_abort: stream_abort.clone(),
                access: access.clone(),
            },
        );
        let outlet = Outlet {
            key: key.to_string(),
            notify: Arc::clone(&self.notify),
            on_access: Arc::clone(&self.on_access),
            access,
        };
        std::thread::spawn(move || worker(outlet, subscribe, sync_state, backoff, shutdown, stream_abort));
    }

    /// 註冊失敗的入帳：command 在 register 之前失敗（如 registry 壞讀）時
    /// 呼叫——沒有訂閱入帳，但前端 unmount 照樣退訂，這筆額度讓配對的
    /// unregister 變 no-op。先消先到的退訂額度（反向交錯）。
    pub fn register_failed(&self, key: &str) {
        let mut subs = self.subs.lock().expect("subs lock");
        if let Some(debt) = subs.debts.get_mut(key) {
            *debt -= 1;
            if *debt == 0 {
                subs.debts.remove(key);
            }
            return;
        }
        *subs.credits.entry(key.to_string()).or_insert(0) += 1;
    }

    /// 退出一個 session：先消失敗註冊的額度（該筆退訂沒有對應的訂閱）；
    /// 其次參照計數歸零時收束訂閱（中止流、worker 退場）；
    /// key 不存在＝這筆退訂先於它的 register 抵達，記一筆抵銷額度。
    ///
    /// 收束不送存取狀態通知：沒有訂閱的 key 沒有人在判定，它的狀態是「不知道」
    /// 而不是「解除」。它的存取狀態隨訂閱一起消失（[`EventManager::access_of`]）。
    pub fn unregister(&self, key: &str) {
        let mut subs = self.subs.lock().expect("subs lock");
        if let Some(credit) = subs.credits.get_mut(key) {
            *credit -= 1;
            if *credit == 0 {
                subs.credits.remove(key);
            }
            return;
        }
        let Some(entry) = subs.entries.get_mut(key) else {
            *subs.debts.entry(key.to_string()).or_insert(0) += 1;
            return;
        };
        entry.count = entry.count.saturating_sub(1);
        if entry.count == 0 {
            let entry = subs.entries.remove(key).expect("entry exists");
            entry.shutdown.store(true, Ordering::Relaxed);
            let abort = entry.stream_abort.lock().expect("abort lock").take();
            if let Some(abort) = abort {
                abort.abort();
            }
            entry.access.lock().expect("access lock").closed = true;
        }
    }
}

/// 一條訂閱的 worker：開流→分發→斷線收斂→重連，直到 shutdown。
fn worker(
    outlet: Outlet,
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
    // 這個 key 是否在「被拒」。只有下面的輪詢會改它。
    let mut denied = false;
    loop {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        if denied {
            // 被拒期間只輪詢（見模組說明）。
        } else if let Ok(mut stream) = subscribe(last_event_id) {
            // 曾斷線後第一次成功重訂即全量失效；若前一輪 sync-state 已送過，
            // 此處只結束 recovery episode，不重複通知。
            if recovery_episode && !recovery_invalidation_sent {
                outlet.reload();
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
                        outlet.reload();
                    }
                    Ok(Some(RemoteEvent::Reset)) => {
                        // 續傳位點已過保留期：全量重載，位點歸零，自同一條流
                        // 的新位點續訂。
                        last_event_id = None;
                        last_etag = None;
                        outlet.reload();
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
        match sync_state() {
            Ok(etag) => {
                let regained = outlet.set_access(AccessState::Allowed);
                denied = false;
                let changed = last_etag.as_deref() != Some(etag.as_str());
                if changed {
                    last_etag = Some(etag);
                }
                // 一輪最多一筆重載通知，三種情況各自成立就送：ETag 變了；同一斷線
                // episode 第一次成功收斂（SSE 持續失敗時後續輪詢不洗版）；由「被拒」
                // 解除。
                let first_convergence = recovery_episode && !recovery_invalidation_sent;
                if changed || first_convergence || regained {
                    outlet.reload();
                }
                if recovery_episode {
                    recovery_invalidation_sent = true;
                }
            }
            // 輪詢對任何讀得到這個範圍的人都成功：它的 403 才代表「這個範圍讀不到」。
            Err(error) if error.status == Some(403) => {
                let reason = speclink_remote::server_reason(&error).map(str::to_string);
                outlet.set_access(AccessState::Denied { reason });
                denied = true;
            }
            // 連不上或其他失敗：維持現狀，連線層另外累計離線。
            Err(_) => {}
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

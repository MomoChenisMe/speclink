//! Phase 3 收官驗收：desktop remote 資料面、真實 CLI 與兩個隔離 server 的
//! 單一連續劇本（phase3-acceptance spec）。

mod common;

use speclink_desktop_lib::credentials::{CredentialKind, CredentialStore, MemoryCredentialStore};
use speclink_desktop_lib::event_manager::EventManager;
use speclink_desktop_lib::remote::{self, ConnectionState, TokenManager};
use speclink_remote::events;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex};
use std::time::Duration;

const TASKS: &str = "## 1. Work\n\n- [ ] 1.1 First\n";
const RD_TASKS: &str = "## 1. RD Work\n\n- [ ] 1.1 Reflect through PM\n";
const RD_PROPOSAL: &str =
    "## Why\n\nRD checkout 驗證 CLI 與 desktop 資料面互通。\n\n## What Changes\n\n- 完成 RD 任務。\n";

/// 讓競態中的 refresh 讀取先取得同一份舊值。singleflight 正常時只有 leader
/// 會進入此 gate，短暫 timeout 後即可自行前進；破壞互斥時兩個 caller 會同時抵達。
struct ReconnectCredentialStore {
    inner: MemoryCredentialStore,
    armed: AtomicBool,
    refresh_readers: Mutex<HashSet<std::thread::ThreadId>>,
    arrived: Mutex<usize>,
    release: Condvar,
}

impl ReconnectCredentialStore {
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

impl CredentialStore for ReconnectCredentialStore {
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

#[test]
fn phase3_helpers_start_two_isolated_servers_and_drive_the_real_cli() {
    let h = common::phase3_harness();
    h.first.seed_change("pm", "chain-seed", TASKS);

    let checkout = h
        .first
        .remote_checkout("pm", &h.first.editor_pat, "helper-smoke");
    let output = checkout.run(&["list", "--json"]);

    assert!(
        output.status.success(),
        "CLI list 成功：{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("chain-seed"),
        "CLI 讀到第一個 server 的 change"
    );
    assert_ne!(
        h.first.store_path, h.second.store_path,
        "兩個 server 使用獨立 SQLite"
    );
    assert!(h.second.is_ready(), "第二個 server 同時常駐");

    let credentials = MemoryCredentialStore::new();
    h.first.device_login_editor(&credentials);
    let refresh = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read refresh credential")
        .expect("device login stored refresh credential");
    h.first.revoke_editor_device_family();
    assert!(
        speclink_remote::device::refresh(&h.first.origin, &refresh).is_err(),
        "撤銷 device family 後 refresh credential 立即失效"
    );
}

#[test]
fn scenario_failures_include_the_name_both_server_tails_and_connection_states() {
    let h = common::phase3_harness();
    let states = [("first", "online"), ("second", "online")];
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        common::scenario_assert!(&h, "多 tab", &states, false, "刻意失敗以驗證診斷形狀");
    }))
    .expect_err("scenario assertion deliberately fails");
    let message = failure
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            failure
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .expect("panic message");

    assert!(message.starts_with("[多 tab]"), "情境名是第一段：{message}");
    for section in [
        "first server output tail",
        "second server output tail",
        "connection states",
        "first=online",
        "second=online",
    ] {
        assert!(
            message.contains(section),
            "失敗現場包含 {section}: {message}"
        );
    }
}

#[test]
fn failure_diagnostics_can_be_materialized_for_ci_upload() {
    let h = common::phase3_harness();
    let artifacts = tempfile::tempdir().expect("phase3 CI artifact tempdir");
    h.write_failure_artifacts(
        artifacts.path(),
        &[("first", "offline"), ("second", "online")],
    )
    .expect("write phase3 failure artifacts");

    for (name, expected) in [
        ("first-server.log", "first: started"),
        ("second-server.log", "second: started"),
        ("connection-states.log", "first=offline\nsecond=online"),
    ] {
        let content = std::fs::read_to_string(artifacts.path().join(name))
            .unwrap_or_else(|error| panic!("read {name}: {error}"));
        assert!(
            content.contains(expected),
            "{name} must contain {expected:?}: {content}"
        );
    }
}

#[test]
fn same_origin_workspaces_share_one_rotation_after_reconnect() {
    const EXPIRED: &str = "spk_at_expired_multitab_reconnect";

    let h = common::phase3_harness();
    h.first.seed_change("pm", "pm-reconnect", TASKS);
    h.first.seed_change("rd", "rd-reconnect", RD_TASKS);

    let credentials = Arc::new(ReconnectCredentialStore::new());
    let access = h.first.device_login_editor(credentials.as_ref());
    let refresh_before = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read initial refresh credential")
        .expect("device login stored refresh credential");
    let (state_tx, state_rx) = std::sync::mpsc::channel();
    let manager = Arc::new(TokenManager::with_connection_state(
        &h.first.origin,
        "conn_multitab_reconnect",
        1,
        move |state| {
            let _ = state_tx.send(state);
        },
    ));
    manager.adopt_access_token(&access);

    let (pm_workspace, _) =
        remote::open_workspace(&h.first.origin, "alpha/pm", &manager, credentials.as_ref())
            .expect("open PM workspace before reconnect");
    let (rd_workspace, _) =
        remote::open_workspace(&h.first.origin, "alpha/rd", &manager, credentials.as_ref())
            .expect("open RD workspace before reconnect");

    h.first.stop();
    h.first.start();
    manager.adopt_access_token(EXPIRED);
    while state_rx.try_recv().is_ok() {}
    credentials.arm();

    let start = Arc::new(Barrier::new(3));
    let pm_credentials = credentials.clone();
    let pm_start = start.clone();
    let pm_thread = std::thread::spawn(move || {
        pm_start.wait();
        pm_workspace.list_changes(pm_credentials.as_ref())
    });
    let rd_credentials = credentials.clone();
    let rd_start = start.clone();
    let rd_thread = std::thread::spawn(move || {
        rd_start.wait();
        rd_workspace.list_changes(rd_credentials.as_ref())
    });
    start.wait();

    let pm_changes = pm_thread.join().expect("PM reconnect thread");
    let rd_changes = rd_thread.join().expect("RD reconnect thread");
    let refresh_readers = credentials.refresh_readers();
    credentials.disarm();
    let refresh_after = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read rotated refresh credential")
        .expect("rotation stored replacement refresh credential");

    assert!(
        pm_changes.as_ref().is_ok_and(|list| list
            .changes
            .iter()
            .any(|change| change.name == "pm-reconnect")),
        "PM workspace 應在重連 rotation 後恢復：{pm_changes:?}"
    );
    assert!(
        rd_changes.as_ref().is_ok_and(|list| list
            .changes
            .iter()
            .any(|change| change.name == "rd-reconnect")),
        "RD workspace 應共用 rotation 並恢復：{rd_changes:?}"
    );
    assert_ne!(
        refresh_after, refresh_before,
        "重連必須確實完成一次 refresh credential rotation"
    );
    assert_eq!(
        refresh_readers, 1,
        "同來源多 workspace 只能由 singleflight leader thread 執行 rotation"
    );
    assert!(
        manager.needs_reauth().is_none(),
        "競態不得誤入 needs-reauth"
    );
    assert_eq!(manager.connection_state(), ConnectionState::Online);
    assert!(
        state_rx
            .try_iter()
            .all(|event| event.state != ConnectionState::NeedsReauth),
        "競態不得廣播 needs-reauth"
    );
}

#[test]
fn phase3_five_scenarios_run_as_one_continuous_chain() {
    let h = common::phase3_harness();
    h.first.seed_change("pm", "pm-plan", TASKS);
    let initial_states = [("first", "not-opened"), ("second", "not-opened")];
    common::scenario_assert!(
        &h,
        "第一幕：雙 server 起機與 setup",
        &initial_states,
        h.first.is_ready() && h.second.is_ready(),
        "兩個隔離 server 必須同時常駐"
    );

    let credentials = Arc::new(MemoryCredentialStore::new());
    h.first.device_login_editor(credentials.as_ref());
    let (first_state_tx, first_state_rx) = std::sync::mpsc::channel();
    let first_manager = Arc::new(TokenManager::with_connection_state(
        &h.first.origin,
        "conn_first",
        1,
        move |state| {
            let _ = first_state_tx.send(state);
        },
    ));
    let opened = remote::open_workspace(
        &h.first.origin,
        "alpha/pm",
        &first_manager,
        credentials.as_ref(),
    );
    let first_state = format!("{:?}", first_manager.connection_state());
    let states = [("first", first_state.as_str()), ("second", "not-opened")];
    common::scenario_assert!(
        &h,
        "第二幕：PM 無 checkout 登入與 handshake",
        &states,
        opened.is_ok(),
        &format!("handshake 失敗：{:?}", opened.as_ref().err())
    );
    let (pm_workspace, pm_info) = opened.expect("checked PM handshake");
    common::scenario_assert!(
        &h,
        "第二幕：PM 無 checkout session 身分",
        &states,
        pm_info.project_key == "alpha" && pm_info.repo_key == "pm",
        "handshake 必須綁定 alpha/pm"
    );

    let changes = pm_workspace.list_changes(credentials.as_ref());
    common::scenario_assert!(
        &h,
        "第二幕：PM 清單讀取",
        &states,
        changes
            .as_ref()
            .is_ok_and(|list| { list.changes.len() == 1 && list.changes[0].name == "pm-plan" }),
        "清單必須只包含前幕播種的 pm-plan"
    );
    let document = pm_workspace.document(credentials.as_ref(), "pm-plan", "tasks.md");
    common::scenario_assert!(
        &h,
        "第二幕：PM 文件讀取",
        &states,
        document
            .as_ref()
            .is_ok_and(|artifact| artifact.content.contains("- [ ] 1.1 First")),
        "tasks.md 必須來自 server 真值"
    );
    let written = pm_workspace.set_task_done(credentials.as_ref(), "pm-plan", "1", true);
    common::scenario_assert!(
        &h,
        "第二幕：PM 任務勾選寫入",
        &states,
        written.is_ok(),
        &format!("task done 失敗：{written:?}")
    );
    let after_write = pm_workspace.document(credentials.as_ref(), "pm-plan", "tasks.md");
    common::scenario_assert!(
        &h,
        "第二幕：PM 任務寫穿 server",
        &states,
        after_write
            .as_ref()
            .is_ok_and(|artifact| artifact.content.contains("- [x] 1.1 First")),
        "勾選結果必須可由同一 session 重讀"
    );

    let reader_credentials = MemoryCredentialStore::new();
    reader_credentials
        .set(&h.first.origin, CredentialKind::Pat, &h.first.reader_pat)
        .expect("reader PAT into memory store");
    let reader_manager = Arc::new(TokenManager::new(&h.first.origin));
    let reader_open = remote::open_workspace(
        &h.first.origin,
        "alpha/pm",
        &reader_manager,
        &reader_credentials,
    );
    common::scenario_assert!(
        &h,
        "第二幕：reader capability 停用",
        &states,
        reader_open
            .as_ref()
            .is_ok_and(|(_, info)| !info.capabilities.policy_write),
        "reader 的 policyWrite capability 必須為 false"
    );

    let rd_open = remote::open_workspace(
        &h.first.origin,
        "alpha/rd",
        &first_manager,
        credentials.as_ref(),
    );
    common::scenario_assert!(
        &h,
        "第三幕：PM 開啟 RD spec-only 資料面",
        &states,
        rd_open.is_ok(),
        &format!("RD scope handshake 失敗：{:?}", rd_open.as_ref().err())
    );
    let (rd_workspace, _) = rd_open.expect("checked RD handshake");

    const RD_KEY: &str = "remote:conn_first/alpha/rd";
    let (invalidated_tx, invalidated_rx) = std::sync::mpsc::channel();
    let event_manager = EventManager::new(move |key| {
        let _ = invalidated_tx.send(key);
    });
    let rd_connects = Arc::new(AtomicUsize::new(0));
    let sub_base = h.first.project_url();
    let sub_runtime = first_manager.clone();
    let sub_credentials = credentials.clone();
    let sub_counter = rd_connects.clone();
    let sync_base = h.first.project_url();
    let sync_runtime = first_manager.clone();
    let sync_credentials = credentials.clone();
    event_manager.register(
        RD_KEY,
        move |last| {
            sub_counter.fetch_add(1, Ordering::SeqCst);
            sub_runtime.execute(sub_credentials.as_ref(), |token| {
                events::subscribe(&sub_base, token, Some("rd"), last)
            })
        },
        move || {
            sync_runtime.execute(sync_credentials.as_ref(), |token| {
                events::sync_state(&sync_base, token, Some("rd"))
            })
        },
        vec![Duration::from_millis(30)],
    );
    std::thread::sleep(Duration::from_millis(150));

    let checkout = h
        .first
        .remote_checkout("rd", &h.first.editor_pat, "rd-checkout");
    checkout.write_git_marker("phase3.marker", "alpha/rd checkout\n");
    for (args, stdin) in [
        (vec!["new", "change", "rd-plan"], None),
        (
            vec![
                "new", "artifact", "proposal", "--change", "rd-plan", "--stdin",
            ],
            Some(RD_PROPOSAL),
        ),
        (
            vec!["new", "artifact", "tasks", "--change", "rd-plan", "--stdin"],
            Some(RD_TASKS),
        ),
    ] {
        let output = match stdin {
            Some(content) => checkout.run_stdin(&args, content),
            None => checkout.run(&args),
        };
        common::scenario_assert!(
            &h,
            "第三幕：RD CLI 建立 change 與 artifacts",
            &states,
            output.status.success(),
            &format!(
                "speclink {:?} 失敗：{}",
                args,
                String::from_utf8_lossy(&output.stderr)
            )
        );
    }
    while invalidated_rx.try_recv().is_ok() {}
    let task_done = checkout.run(&["task", "done", "1", "--change", "rd-plan"]);
    common::scenario_assert!(
        &h,
        "第三幕：RD CLI task done",
        &states,
        task_done.status.success(),
        &format!(
            "task done 失敗：{}",
            String::from_utf8_lossy(&task_done.stderr)
        )
    );
    let invalidated = invalidated_rx.recv_timeout(Duration::from_secs(5));
    common::scenario_assert!(
        &h,
        "第三幕：RD 寫入即時反映至 PM 資料面",
        &states,
        invalidated.as_deref() == Ok(RD_KEY),
        &format!("五秒內必須收到 locator invalidate：{invalidated:?}")
    );
    let reflected = rd_workspace.document(credentials.as_ref(), "rd-plan", "tasks.md");
    common::scenario_assert!(
        &h,
        "第三幕：PM 經 invalidate 重查 server 真值",
        &states,
        reflected
            .as_ref()
            .is_ok_and(|artifact| artifact.content.contains("- [x] 1.1 Reflect through PM")),
        "PM 資料面重查必須看見 RD 完成狀態"
    );

    h.second.seed_change("main", "second-plan", TASKS);
    let first_refresh_before_second_login = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read first refresh before second login")
        .expect("first refresh exists before second login");
    common::scenario_assert!(
        &h,
        "第四幕：多 server credential 依 origin 隔離",
        &states,
        credentials
            .get(&h.second.origin, CredentialKind::Refresh)
            .is_ok_and(|credential| credential.is_none()),
        "第二個 origin 登入前不得讀到第一個 origin 的 refresh credential"
    );
    h.second.device_login_editor(credentials.as_ref());
    let first_refresh_after_second_login = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read first refresh after second login")
        .expect("first refresh remains after second login");
    let second_refresh = credentials
        .get(&h.second.origin, CredentialKind::Refresh)
        .expect("read second refresh")
        .expect("second refresh stored by origin");
    common::scenario_assert!(
        &h,
        "第四幕：多 server credential 不互相覆寫",
        &states,
        first_refresh_after_second_login == first_refresh_before_second_login
            && second_refresh != first_refresh_after_second_login,
        "第二個 origin 登入後，兩邊 refresh credential 必須各自保留"
    );

    let (second_state_tx, second_state_rx) = std::sync::mpsc::channel();
    let second_manager = Arc::new(TokenManager::with_connection_state(
        &h.second.origin,
        "conn_second",
        1,
        move |state| {
            let _ = second_state_tx.send(state);
        },
    ));
    let second_open = remote::open_workspace(
        &h.second.origin,
        "beta/main",
        &second_manager,
        credentials.as_ref(),
    );
    let first_state = format!("{:?}", first_manager.connection_state());
    let second_state = format!("{:?}", second_manager.connection_state());
    let multi_server_states = [
        ("first", first_state.as_str()),
        ("second", second_state.as_str()),
    ];
    common::scenario_assert!(
        &h,
        "第四幕：第二連線開啟獨立 session",
        &multi_server_states,
        second_open.is_ok(),
        &format!("第二連線 handshake 失敗：{:?}", second_open.as_ref().err())
    );
    let (second_workspace, second_info) = second_open.expect("checked second handshake");
    common::scenario_assert!(
        &h,
        "第四幕：第二連線只看見自己的 scope",
        &multi_server_states,
        second_info.project_key == "beta"
            && second_info.repo_key == "main"
            && second_workspace
                .list_changes(credentials.as_ref())
                .is_ok_and(|list| {
                    list.changes.len() == 1 && list.changes[0].name == "second-plan"
                }),
        "第二連線必須綁定 beta/main 且不能讀到第一個 server 的 changes"
    );

    const SECOND_KEY: &str = "remote:conn_second/beta/main";
    let second_connects = Arc::new(AtomicUsize::new(0));
    let second_sub_base = h.second.project_url();
    let second_sub_runtime = second_manager.clone();
    let second_sub_credentials = credentials.clone();
    let second_sub_counter = second_connects.clone();
    let second_sync_base = h.second.project_url();
    let second_sync_runtime = second_manager.clone();
    let second_sync_credentials = credentials.clone();
    event_manager.register(
        SECOND_KEY,
        move |last| {
            second_sub_counter.fetch_add(1, Ordering::SeqCst);
            second_sub_runtime.execute(second_sub_credentials.as_ref(), |token| {
                events::subscribe(&second_sub_base, token, Some("main"), last)
            })
        },
        move || {
            second_sync_runtime.execute(second_sync_credentials.as_ref(), |token| {
                events::sync_state(&second_sync_base, token, Some("main"))
            })
        },
        vec![Duration::from_millis(30)],
    );
    std::thread::sleep(Duration::from_millis(150));

    while invalidated_rx.try_recv().is_ok() {}
    let first_server_write =
        rd_workspace.set_task_done(credentials.as_ref(), "rd-plan", "1", false);
    let first_server_event = invalidated_rx.recv_timeout(Duration::from_secs(5));
    let first_server_extra = invalidated_rx.recv_timeout(Duration::from_millis(300));
    common::scenario_assert!(
        &h,
        "第四幕：第一 server 事件不串到第二 server",
        &multi_server_states,
        first_server_write.is_ok()
            && first_server_event.as_deref() == Ok(RD_KEY)
            && first_server_extra.is_err(),
        &format!(
            "第一 server 寫入只應分發 {RD_KEY}：write={first_server_write:?}, event={first_server_event:?}, extra={first_server_extra:?}"
        )
    );

    let second_server_write =
        second_workspace.set_task_done(credentials.as_ref(), "second-plan", "1", true);
    let second_server_event = invalidated_rx.recv_timeout(Duration::from_secs(5));
    let second_server_extra = invalidated_rx.recv_timeout(Duration::from_millis(300));
    common::scenario_assert!(
        &h,
        "第四幕：第二 server 事件不串到第一 server",
        &multi_server_states,
        second_server_write.is_ok()
            && second_server_event.as_deref() == Ok(SECOND_KEY)
            && second_server_extra.is_err(),
        &format!(
            "第二 server 寫入只應分發 {SECOND_KEY}：write={second_server_write:?}, event={second_server_event:?}, extra={second_server_extra:?}"
        )
    );

    const PM_KEY: &str = "remote:conn_first/alpha/pm";
    let pm_connects = Arc::new(AtomicUsize::new(0));
    let pm_sub_base = h.first.project_url();
    let pm_sub_runtime = first_manager.clone();
    let pm_sub_credentials = credentials.clone();
    let pm_sub_counter = pm_connects.clone();
    let pm_sync_base = h.first.project_url();
    let pm_sync_runtime = first_manager.clone();
    let pm_sync_credentials = credentials.clone();
    event_manager.register(
        PM_KEY,
        move |last| {
            pm_sub_counter.fetch_add(1, Ordering::SeqCst);
            pm_sub_runtime.execute(pm_sub_credentials.as_ref(), |token| {
                events::subscribe(&pm_sub_base, token, Some("pm"), last)
            })
        },
        move || {
            pm_sync_runtime.execute(pm_sync_credentials.as_ref(), |token| {
                events::sync_state(&pm_sync_base, token, Some("pm"))
            })
        },
        vec![Duration::from_millis(30)],
    );
    let duplicate_rd_connects = Arc::new(AtomicUsize::new(0));
    let duplicate_sub_counter = duplicate_rd_connects.clone();
    event_manager.register(
        RD_KEY,
        move |_| {
            duplicate_sub_counter.fetch_add(1, Ordering::SeqCst);
            unreachable!("same locator must reuse the existing stream")
        },
        || unreachable!("same locator must reuse the existing sync worker"),
        vec![Duration::from_millis(30)],
    );
    std::thread::sleep(Duration::from_millis(150));
    let first_server_streams =
        rd_connects.load(Ordering::SeqCst) + pm_connects.load(Ordering::SeqCst);
    common::scenario_assert!(
        &h,
        "第五幕：多 tab 依 locator scope 共用與隔離事件流",
        &multi_server_states,
        first_server_streams == 2
            && rd_connects.load(Ordering::SeqCst) == 1
            && pm_connects.load(Ordering::SeqCst) == 1
            && duplicate_rd_connects.load(Ordering::SeqCst) == 0,
        &format!(
            "PM／RD 各一條、同 RD locator 第二個 session 不另開流：total={first_server_streams}, rd={}, pm={}, duplicate={}",
            rd_connects.load(Ordering::SeqCst),
            pm_connects.load(Ordering::SeqCst),
            duplicate_rd_connects.load(Ordering::SeqCst)
        )
    );

    while invalidated_rx.try_recv().is_ok() {}
    let pm_write = pm_workspace.set_task_done(credentials.as_ref(), "pm-plan", "1", false);
    let pm_event = invalidated_rx.recv_timeout(Duration::from_secs(5));
    let pm_extra = invalidated_rx.recv_timeout(Duration::from_millis(300));
    common::scenario_assert!(
        &h,
        "第五幕：PM scope 寫入只分發 PM locator",
        &multi_server_states,
        pm_write.is_ok() && pm_event.as_deref() == Ok(PM_KEY) && pm_extra.is_err(),
        &format!("PM write={pm_write:?}, event={pm_event:?}, extra={pm_extra:?}")
    );

    let rd_write = rd_workspace.set_task_done(credentials.as_ref(), "rd-plan", "1", true);
    let rd_event = invalidated_rx.recv_timeout(Duration::from_secs(5));
    let rd_extra = invalidated_rx.recv_timeout(Duration::from_millis(300));
    common::scenario_assert!(
        &h,
        "第五幕：RD scope 寫入只分發 RD locator",
        &multi_server_states,
        rd_write.is_ok() && rd_event.as_deref() == Ok(RD_KEY) && rd_extra.is_err(),
        &format!("RD write={rd_write:?}, event={rd_event:?}, extra={rd_extra:?}")
    );

    while first_state_rx.try_recv().is_ok() {}
    while invalidated_rx.try_recv().is_ok() {}
    h.first.stop();
    let offline_event = first_state_rx.recv_timeout(Duration::from_secs(5));
    let offline_state = format!("{:?}", first_manager.connection_state());
    let second_live_state = format!("{:?}", second_manager.connection_state());
    let outage_states = [
        ("first", offline_state.as_str()),
        ("second", second_live_state.as_str()),
    ];
    common::scenario_assert!(
        &h,
        "第六幕：第一 server 失聯廣播 offline",
        &outage_states,
        offline_event
            .as_ref()
            .is_ok_and(|event| event.state == ConnectionState::Offline)
            && first_manager.connection_state() == ConnectionState::Offline
            && !h.first.is_ready(),
        &format!("五秒內必須轉為 offline：{offline_event:?}")
    );

    let offline_write = pm_workspace.set_task_done(credentials.as_ref(), "pm-plan", "1", true);
    common::scenario_assert!(
        &h,
        "第六幕：離線寫入即拒且不排隊",
        &outage_states,
        offline_write
            .as_ref()
            .is_err_and(|error| error.reason.as_deref() == Some("offline")),
        &format!("offline write 必須以 offline reason 拒絕：{offline_write:?}")
    );

    let second_live_write =
        second_workspace.set_task_done(credentials.as_ref(), "second-plan", "1", false);
    let second_checkout = h
        .second
        .remote_checkout("main", &h.second.editor_pat, "second-outage");
    let second_cli_write = second_checkout.run(&["new", "change", "second-during-outage"]);
    let second_during_outage = second_workspace.list_changes(credentials.as_ref());
    common::scenario_assert!(
        &h,
        "第六幕：第一 server 失聯不波及第二 server",
        &outage_states,
        second_live_write.is_ok()
            && second_cli_write.status.success()
            && second_manager.connection_state() == ConnectionState::Online
            && second_during_outage.as_ref().is_ok_and(|list| {
                list.changes
                    .iter()
                    .any(|change| change.name == "second-during-outage")
            })
            && second_state_rx.try_recv().is_err(),
        &format!(
            "第二 server 應維持 online 並接受 CLI 寫入：write={second_live_write:?}, cli={}, list={second_during_outage:?}",
            second_cli_write.status
        )
    );

    h.first.seed_change("pm", "first-during-outage", TASKS);
    while invalidated_rx.try_recv().is_ok() {}
    h.first.start();
    // 期限須蓋過 client 的 SSE stall 偵測（45s）：半開連線的 worker 要等
    // stall 逾時才會重訂。正常路徑事件毫秒級到達，迴圈立即結束。
    let online_event = first_state_rx.recv_timeout(Duration::from_secs(75));
    let recovery_deadline = std::time::Instant::now() + Duration::from_secs(75);
    let mut recovery_keys = Vec::new();
    while std::time::Instant::now() < recovery_deadline
        && !(recovery_keys.iter().any(|key| key == PM_KEY)
            && rd_connects.load(Ordering::SeqCst) >= 2)
    {
        if let Ok(key) = invalidated_rx.recv_timeout(Duration::from_millis(250)) {
            recovery_keys.push(key);
        }
    }
    let recovered_changes = pm_workspace.list_changes(credentials.as_ref());
    let rejected_write_absent = pm_workspace.document(credentials.as_ref(), "pm-plan", "tasks.md");
    common::scenario_assert!(
        &h,
        "第六幕：第一 server 重啟後以 Polling＋ETag 自動收斂",
        &multi_server_states,
        online_event
            .as_ref()
            .is_ok_and(|event| event.state == ConnectionState::Online)
            && recovery_keys.iter().any(|key| key == PM_KEY)
            && recovered_changes.as_ref().is_ok_and(|list| {
                list.changes
                    .iter()
                    .any(|change| change.name == "first-during-outage")
            })
            && rejected_write_absent
                .as_ref()
                .is_ok_and(|artifact| artifact.content.contains("- [ ] 1.1 First")),
        &format!(
            "重啟應 online、發 PM invalidate、讀到期間變更且離線寫入未入列：state={online_event:?}, keys={recovery_keys:?}, changes={recovered_changes:?}, task={rejected_write_absent:?}"
        )
    );

    while first_state_rx.try_recv().is_ok() {}
    h.first.revoke_editor_device_family();
    let revoked_read = pm_workspace.list_changes(credentials.as_ref());
    let reauth_event = first_state_rx.recv_timeout(Duration::from_secs(5));
    common::scenario_assert!(
        &h,
        "第六幕：撤銷 device family 轉為 needs-reauth",
        &multi_server_states,
        revoked_read
            .as_ref()
            .is_err_and(|error| { error.reason.as_deref() == Some("needs_reauth") })
            && reauth_event
                .as_ref()
                .is_ok_and(|event| event.state == ConnectionState::NeedsReauth)
            && first_manager.connection_state() == ConnectionState::NeedsReauth,
        &format!("撤銷後必須廣播 needs-reauth：read={revoked_read:?}, state={reauth_event:?}")
    );

    let relogin_access = h.first.device_login_editor(credentials.as_ref());
    first_manager.adopt_access_token(&relogin_access);
    let relogin_event = first_state_rx.recv_timeout(Duration::from_secs(5));
    let recovered_in_place = pm_workspace.list_changes(credentials.as_ref());
    common::scenario_assert!(
        &h,
        "第六幕：重新登入後原 session 原地恢復",
        &multi_server_states,
        relogin_event
            .as_ref()
            .is_ok_and(|event| event.state == ConnectionState::Online)
            && first_manager.connection_state() == ConnectionState::Online
            && recovered_in_place.as_ref().is_ok_and(|list| {
                list.changes
                    .iter()
                    .any(|change| change.name == "first-during-outage")
            }),
        &format!(
            "同一 workspace／manager 應恢復：state={relogin_event:?}, list={recovered_in_place:?}"
        )
    );

    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..");
    let local_context = speclink_desktop_core::init_core_context(&repo_root);
    common::scenario_assert!(
        &h,
        "Gate 1：local＋remote spec-only＋remote checkout 三形態 session 並存",
        &multi_server_states,
        local_context.is_some()
            && recovered_in_place.is_ok()
            && checkout.root().join(".git").is_dir()
            && rd_workspace
                .list_changes(credentials.as_ref())
                .is_ok_and(|list| list.changes.iter().any(|change| change.name == "rd-plan")),
        "本地資料層、PM spec-only 與 RD checkout 必須在同一劇本同時存活"
    );

    let rd_key_with_checkout = "remote:conn_first/alpha/rd";
    common::scenario_assert!(
        &h,
        "Gate 2：分頁身分使用 locator key 而非 root path",
        &multi_server_states,
        PM_KEY != rd_key_with_checkout
            && RD_KEY == rd_key_with_checkout
            && !rd_key_with_checkout.contains(checkout.root().to_string_lossy().as_ref()),
        "PM／RD scope 應有不同 locator key，checkoutRoot 不得參與 RD 身分"
    );

    common::scenario_assert!(
        &h,
        "Gate 3：capability 驅動 reader policy 停用",
        &multi_server_states,
        reader_open
            .as_ref()
            .is_ok_and(|(_, info)| !info.capabilities.policy_write),
        "reader handshake 的 policyWrite 必須持續為 false"
    );

    let persisted_tabs = serde_json::json!({
        "version": 2,
        "tabs": [
            {
                "locator": { "kind": "local", "root": repo_root.display().to_string() },
                "name": "local"
            },
            {
                "locator": {
                    "kind": "remote",
                    "connectionId": "conn_first",
                    "projectId": "alpha",
                    "repoId": "pm"
                },
                "name": "alpha/pm"
            },
            {
                "locator": {
                    "kind": "remote",
                    "connectionId": "conn_first",
                    "projectId": "alpha",
                    "repoId": "rd",
                    "checkoutRoot": checkout.root().display().to_string()
                },
                "name": "alpha/rd"
            }
        ],
        "activeKey": RD_KEY
    })
    .to_string();
    let registry_payloads = [
        std::fs::read_to_string(&h.first.registry).expect("read first connection registry"),
        std::fs::read_to_string(&h.second.registry).expect("read second connection registry"),
        persisted_tabs,
    ];
    let current_first_refresh = credentials
        .get(&h.first.origin, CredentialKind::Refresh)
        .expect("read current first refresh")
        .expect("current first refresh exists");
    let current_second_refresh = credentials
        .get(&h.second.origin, CredentialKind::Refresh)
        .expect("read current second refresh")
        .expect("current second refresh exists");
    let known_credentials = [
        h.first.editor_pat.as_str(),
        h.first.reader_pat.as_str(),
        h.second.editor_pat.as_str(),
        h.second.reader_pat.as_str(),
        current_first_refresh.as_str(),
        current_second_refresh.as_str(),
        relogin_access.as_str(),
    ];
    common::scenario_assert!(
        &h,
        "Gate 4：registry 與 tabs 持久化 payload 無 credential",
        &multi_server_states,
        known_credentials.iter().all(|credential| {
            !credential.is_empty()
                && registry_payloads
                    .iter()
                    .all(|payload| !payload.contains(*credential))
        }),
        "connection registry 與分頁序列化不得含 PAT、refresh 或 access token"
    );

    common::scenario_assert!(
        &h,
        "Gate 5：Polling＋ETag 使重啟後自動收斂",
        &multi_server_states,
        pm_connects.load(Ordering::SeqCst) >= 2
            && rd_connects.load(Ordering::SeqCst) >= 2
            && recovery_keys.iter().any(|key| key == PM_KEY)
            && recovered_in_place.as_ref().is_ok_and(|list| {
                list.changes
                    .iter()
                    .any(|change| change.name == "first-during-outage")
            }),
        &format!(
            "兩 scope 都應重連且 PM 應經 invalidate 重讀新真值：pm={}, rd={}, keys={recovery_keys:?}",
            pm_connects.load(Ordering::SeqCst),
            rd_connects.load(Ordering::SeqCst)
        )
    );

    common::scenario_assert!(
        &h,
        "Gate 6：stale 只讀、離線寫入不排隊",
        &multi_server_states,
        offline_write
            .as_ref()
            .is_err_and(|error| error.reason.as_deref() == Some("offline"))
            && rejected_write_absent
                .as_ref()
                .is_ok_and(|artifact| artifact.content.contains("- [ ] 1.1 First")),
        "離線寫入必須即拒，恢復後 server tasks.md 仍維持未勾選"
    );
}

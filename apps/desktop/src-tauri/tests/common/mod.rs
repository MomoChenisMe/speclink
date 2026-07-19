//! remote 整合測試共用 harness：in-process speclink-server（memory identity＋
//! memory store）＋registry 檔。使用者為 demo／multi 兩專案成員（multi 供
//! 多 repo 多義案例）；credential 注入一律 in-memory store。

#![allow(dead_code)]

use chrono::{Duration, Utc};
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::path::PathBuf;
use std::sync::Arc;

pub const DISPLAY: &str = "Dev <dev@example.com>";

pub struct Harness {
    pub origin: String,
    pub identity: Arc<IdentitySqlite>,
    pub user_id: String,
    pub store: SharedStore,
    pub registry: PathBuf,
    _dir: tempfile::TempDir,
}

/// demo project 的 scope（repo backend）。
pub fn scope() -> Scope {
    Scope::new(ProjectId::new("demo"), RepoId::new("backend"))
}

/// 事件測試用的快節奏設定：短心跳、充足 live buffer。
pub fn fast_events() -> EventSettings {
    EventSettings {
        retention: 1024,
        buffer: 64,
        heartbeat: std::time::Duration::from_millis(100),
    }
}

/// 起 in-process server：demo（repo backend）＋multi（repos web、api）入
/// registry，使用者為兩者成員。store 空白——各測試自行 seed。
pub fn harness() -> Harness {
    harness_with_events(EventSettings::default())
}

/// 同 [`harness`]，但以指定的 EventSettings 起 server（SSE／retention 測試用）。
pub fn harness_with_events(settings: EventSettings) -> Harness {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("memory identity"));
    identity.create_project("demo", "Demo").expect("seed demo project");
    identity.create_repo("demo", "backend", "backend").expect("seed demo repo");
    identity.create_project("multi", "Multi").expect("seed multi project");
    identity.create_repo("multi", "web", "web").expect("seed multi web repo");
    identity.create_repo("multi", "api", "api").expect("seed multi api repo");
    let invite = identity
        .create_invitation(NewInvitation {
            email: "dev@example.com".to_string(),
            display: DISPLAY.to_string(),
            memberships: vec!["demo".to_string(), "multi".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&invite, "pw-correct-horse").expect("accept");

    let store: SharedStore = Arc::new(speclink_store::memory::MemoryStore::new());
    let state = AppState {
        events: EventHub::new(store.clone(), settings),
        store: store.clone(),
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
            axum::serve(listener, speclink_server::app::router(state)).await.expect("serve");
        });
    });

    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let origin = format!("http://{addr}");
    let mut entries = Vec::new();
    speclink_desktop_lib::connections::upsert_connection(&mut entries, &origin, "本地")
        .expect("seed entry");
    speclink_desktop_lib::connections::write_registry(&registry, &entries)
        .expect("write registry");

    Harness { origin, identity, user_id, store, registry, _dir: dir }
}

/// 種 change `demo`（schema＋給定的 tasks.md 內容）進 demo/backend scope。
pub fn seed_change(store: &dyn TeamStore, tasks: &str) {
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext { command: "seed".into(), actor: "seed".into() },
        )
        .expect("begin uow");
    uow.create(DocumentId::ChangeMeta { change: "demo".into() }, "schema: spec-driven\n");
    uow.create(
        DocumentId::ChangeArtifact { change: "demo".into(), artifact: "tasks.md".into() },
        tasks,
    );
    store.commit(uow, Vec::new()).expect("seed commit");
}

/// 以 device flow（假瀏覽器直接核准）登入，讓 credential store 落下 refresh
/// credential。
pub fn device_login_approved(
    h: &Harness,
    store: &dyn speclink_desktop_lib::credentials::CredentialStore,
) {
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        let code = url.split("user_code=").nth(1).expect("user_code 預填參數").to_string();
        assert!(identity.approve_device(&code, &user_id).expect("approve"));
        Ok(())
    };
    speclink_desktop_lib::connections::device_login(&h.origin, store, &h.registry, &opener)
        .expect("device login");
}

/// 給使用者簽一枚 PAT（資料面測試最短的 credential 路徑）。
pub fn pat_of(h: &Harness) -> String {
    let (_, pat) = h.identity.create_pat(&h.user_id, "test", None).expect("pat");
    pat
}

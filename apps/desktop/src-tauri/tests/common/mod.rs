//! remote 整合測試共用 harness：in-process speclink-server（memory identity＋
//! memory store）＋registry 檔。使用者為 demo／multi 兩專案成員（multi 供
//! 多 repo 多義案例）；credential 注入一律 in-memory store。

#![allow(dead_code)]

use chrono::{Duration, Utc};
use speclink_server::audit::AuditActor;
use speclink_server::config::{IdentityConfig, ServerConfig, StoreConfig};
use speclink_server::events::{EventHub, EventSettings};
use speclink_server::identity::{IdentitySqlite, IdentityStore, MembershipRole, NewInvitation};
use speclink_server::state::{AppState, SharedStore};
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex, MutexGuard, Once};

pub const DISPLAY: &str = "Dev <dev@example.com>";

/// 同一 test binary 內的 harness 互斥。多個 in-process loopback server 併行
/// 且 CPU 吃緊時，macOS 核心偶發把回應中的連線整條重置——客端讀到一半收
/// EINVAL，動詞已在 server 提交卻回報「server unreachable」（本機以 8 條
/// 忙迴圈壓測可穩定重現；單 harness 序列跑同等壓力下不出現）。以 harness
/// 生命週期互斥將 loopback server 序列化即根除觸發條件；測試內部的併發
/// （多 session、proxy、worker）不受影響。
static HARNESS_GATE: Mutex<()> = Mutex::new(());

fn acquire_harness_gate() -> MutexGuard<'static, ()> {
    // 前一個測試 panic 只代表該測試失敗，poison 不該連坐後續測試。
    HARNESS_GATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub struct Harness {
    pub origin: String,
    pub identity: Arc<IdentitySqlite>,
    pub user_id: String,
    pub store: SharedStore,
    pub registry: PathBuf,
    pub server: RestartableServer,
    _dir: tempfile::TempDir,
    /// 最後宣告：server 完整收束後才釋放互斥。
    _gate: MutexGuard<'static, ()>,
}

pub struct RestartableServer {
    addr: std::net::SocketAddr,
    state: AppState,
    running: std::sync::Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            std::thread::JoinHandle<()>,
        )>,
    >,
}

impl RestartableServer {
    fn from_listener(listener: std::net::TcpListener, state: AppState) -> RestartableServer {
        let addr = listener.local_addr().expect("local addr");
        let server = RestartableServer {
            addr,
            state,
            running: std::sync::Mutex::new(None),
        };
        server.spawn(listener);
        server
    }

    fn spawn(&self, listener: std::net::TcpListener) {
        listener.set_nonblocking(true).expect("nonblocking");
        let state = self.state.clone();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let thread = std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().expect("runtime");
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener).expect("adopt listener");
                let server = axum::serve(listener, speclink_server::app::router(state));
                tokio::select! {
                    result = server => result.expect("serve"),
                    _ = shutdown_rx => {}
                }
            });
        });
        *self.running.lock().expect("server lock") = Some((shutdown_tx, thread));
    }

    pub fn stop(&self) {
        let Some((shutdown, thread)) = self.running.lock().expect("server lock").take() else {
            return;
        };
        let _ = shutdown.send(());
        thread.join().expect("server stops");
    }

    pub fn start(&self) {
        if self.running.lock().expect("server lock").is_some() {
            return;
        }
        let listener = std::net::TcpListener::bind(self.addr).expect("rebind loopback");
        self.spawn(listener);
    }
}

impl Drop for RestartableServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Phase 3 全鏈劇本的兩個常駐 server。各自持有獨立 SQLite store、identity
/// database 與隨機 loopback port；第一個 server 有 `pm`／`rd` 兩個 scope，
/// 第二個 server 有 `main` scope。
pub struct Phase3Harness {
    pub first: Phase3Server,
    pub second: Phase3Server,
    /// 最後宣告：兩台 server 都收束後才釋放互斥。
    _gate: MutexGuard<'static, ()>,
}

impl Phase3Harness {
    pub fn write_failure_artifacts(
        &self,
        dir: &Path,
        connection_states: &[(&str, &str)],
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        std::fs::write(
            dir.join("first-server.log"),
            self.first.output_tail(usize::MAX).join("\n"),
        )?;
        std::fs::write(
            dir.join("second-server.log"),
            self.second.output_tail(usize::MAX).join("\n"),
        )?;
        let states = connection_states
            .iter()
            .map(|(connection, state)| format!("{connection}={state}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(dir.join("connection-states.log"), states)
    }

    pub fn failure_report(
        &self,
        scenario: &str,
        connection_states: &[(&str, &str)],
        detail: &str,
    ) -> String {
        fn render_tail(lines: Vec<String>) -> String {
            if lines.is_empty() {
                "(no captured output)".to_string()
            } else {
                lines.join("\n")
            }
        }

        let states = connection_states
            .iter()
            .map(|(connection, state)| format!("{connection}={state}"))
            .collect::<Vec<_>>()
            .join("\n");
        let artifact_error = std::env::var_os("PHASE3_E2E_ARTIFACT_DIR").and_then(|dir| {
            self.write_failure_artifacts(Path::new(&dir), connection_states)
                .err()
                .map(|error| format!("\n--- artifact write error ---\n{error}"))
        });
        format!(
            "[{scenario}] {detail}\n\
             --- first server output tail ---\n{}\n\
             --- second server output tail ---\n{}\n\
             --- connection states ---\n{}{}",
            render_tail(self.first.output_tail(20)),
            render_tail(self.second.output_tail(20)),
            if states.is_empty() {
                "(no connection states)"
            } else {
                &states
            },
            artifact_error.unwrap_or_default()
        )
    }
}

#[allow(unused_macros)]
macro_rules! scenario_assert {
    ($harness:expr, $scenario:expr, $states:expr, $condition:expr, $detail:expr $(,)?) => {{
        if !$condition {
            panic!("{}", $harness.failure_report($scenario, $states, $detail));
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use scenario_assert;

pub struct Phase3Server {
    pub label: String,
    pub project: String,
    pub origin: String,
    pub identity: Arc<IdentitySqlite>,
    pub editor_id: String,
    pub reader_id: String,
    pub editor_pat: String,
    pub reader_pat: String,
    pub store: SharedStore,
    pub store_path: PathBuf,
    pub registry: PathBuf,
    diagnostics: Arc<Mutex<Vec<String>>>,
    server: RestartableServer,
    _dir: tempfile::TempDir,
}

impl Phase3Server {
    fn new(label: &str, project: &str, repos: &[&str]) -> Phase3Server {
        let dir = tempfile::tempdir().expect("phase3 server tempdir");
        let store_path = dir.path().join("store.db");
        let identity_path = dir.path().join("identity.db");
        let identity = Arc::new(IdentitySqlite::open(&identity_path).expect("phase3 identity"));
        identity
            .create_project(project, &format!("{label} Project"))
            .expect("seed phase3 project");
        for repo in repos {
            identity
                .create_repo(project, repo, &repo.to_ascii_uppercase())
                .expect("seed phase3 repo");
        }

        let editor_id = seed_phase3_user(&identity, label, project, "editor");
        let reader_id = seed_phase3_user(&identity, label, project, "reader");
        identity
            .admin_set_membership(
                &AuditActor::system_cli(),
                &reader_id,
                project,
                MembershipRole::Reader,
                true,
            )
            .expect("make phase3 reader membership read-only");
        let (_, editor_pat) = identity
            .create_pat(&editor_id, "phase3-editor", None)
            .expect("phase3 editor PAT");
        let (_, reader_pat) = identity
            .create_pat(&reader_id, "phase3-reader", None)
            .expect("phase3 reader PAT");

        let store = speclink_server::build_store(&StoreConfig::Sqlite {
            path: store_path.clone(),
        })
        .expect("phase3 sqlite store");
        let settings = fast_events();
        let state = AppState {
            events: EventHub::new(store.clone(), settings.clone()),
            store: store.clone(),
            config: Arc::new(ServerConfig {
                store: StoreConfig::Sqlite {
                    path: store_path.clone(),
                },
                identity: IdentityConfig::Sqlite {
                    path: identity_path,
                },
                public_url: "http://127.0.0.1".to_string(),
                events: settings,
            }),
            identity: identity.clone(),
        };
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("phase3 loopback");
        let addr = listener.local_addr().expect("phase3 local addr");
        let server = RestartableServer::from_listener(listener, state);
        let origin = format!("http://{addr}");
        let registry = dir.path().join("connections.json");
        let mut entries = Vec::new();
        speclink_desktop_lib::connections::upsert_connection(&mut entries, &origin, label)
            .expect("seed phase3 connection");
        speclink_desktop_lib::connections::write_registry(&registry, &entries)
            .expect("write phase3 registry");
        let diagnostics = Arc::new(Mutex::new(vec![format!(
            "{label}: started at http://{addr} (sqlite {})",
            store_path.display()
        )]));
        let result = Phase3Server {
            label: label.to_string(),
            project: project.to_string(),
            origin,
            identity,
            editor_id,
            reader_id,
            editor_pat,
            reader_pat,
            store,
            store_path,
            registry,
            diagnostics,
            server,
            _dir: dir,
        };
        result.wait_ready();
        result
    }

    pub fn project_url(&self) -> String {
        format!("{}/api/speclink/v1/projects/{}", self.origin, self.project)
    }

    pub fn scope(&self, repo: &str) -> Scope {
        Scope::new(ProjectId::new(&self.project), RepoId::new(repo))
    }

    pub fn seed_change(&self, repo: &str, change: &str, tasks: &str) {
        let mut uow = self
            .store
            .begin_unit_of_work(
                &self.scope(repo),
                CommandContext {
                    command: "phase3-seed".into(),
                    actor: "phase3-seed".into(),
                },
            )
            .expect("begin phase3 seed");
        uow.create(
            DocumentId::ChangeMeta {
                change: change.into(),
            },
            "schema: spec-driven\n",
        );
        uow.create(
            DocumentId::ChangeArtifact {
                change: change.into(),
                artifact: "tasks.md".into(),
            },
            tasks,
        );
        self.store
            .commit(uow, Vec::new())
            .expect("commit phase3 seed");
        self.record(format!("seeded {}/{repo}/{change}", self.project));
    }

    pub fn remote_checkout(&self, repo: &str, token: &str, tag: &str) -> RemoteCheckout {
        RemoteCheckout::new(
            &self.project_url(),
            repo,
            token,
            tag,
            Arc::clone(&self.diagnostics),
        )
    }

    pub fn device_login_editor(
        &self,
        store: &dyn speclink_remote::credentials::CredentialStore,
    ) -> String {
        let identity = self.identity.clone();
        let user_id = self.editor_id.clone();
        let opener = move |url: &str| {
            let code = url
                .split("user_code=")
                .nth(1)
                .expect("phase3 user_code parameter")
                .to_string();
            assert!(identity.approve_device(&code, &user_id).expect("approve"));
            Ok(())
        };
        let access_token = device_login_two_segments(&self.origin, store, &self.registry, &opener)
            .expect("phase3 device login");
        self.record("editor device login approved".to_string());
        access_token
    }

    pub fn revoke_editor_device_family(&self) {
        let family = self
            .identity
            .list_device_families(&self.editor_id)
            .expect("list phase3 device families")
            .into_iter()
            .find(|family| family.revoked_at.is_none())
            .expect("live phase3 device family");
        self.identity
            .revoke_family(&self.editor_id, &family.id)
            .expect("revoke phase3 device family");
        self.record(format!("revoked device family {}", family.id));
    }

    pub fn stop(&self) {
        self.server.stop();
        self.record("stopped".to_string());
    }

    pub fn start(&self) {
        self.server.start();
        self.wait_ready();
        self.record("restarted".to_string());
    }

    pub fn is_ready(&self) -> bool {
        self.origin
            .trim_start_matches("http://")
            .parse::<std::net::SocketAddr>()
            .ok()
            .and_then(|addr| {
                std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(100))
                    .ok()
            })
            .is_some()
    }

    pub fn output_tail(&self, limit: usize) -> Vec<String> {
        let output = self.diagnostics.lock().expect("phase3 diagnostic lock");
        output[output.len().saturating_sub(limit)..].to_vec()
    }

    fn wait_ready(&self) {
        for _ in 0..100 {
            if self.is_ready() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!(
            "{} server did not become ready at {}",
            self.label, self.origin
        );
    }

    fn record(&self, line: String) {
        self.diagnostics
            .lock()
            .expect("phase3 diagnostic lock")
            .push(format!("{}: {line}", self.label));
    }
}

pub struct RemoteCheckout {
    dir: tempfile::TempDir,
    root: PathBuf,
    token: String,
    diagnostics: Arc<Mutex<Vec<String>>>,
}

impl RemoteCheckout {
    fn new(
        project_url: &str,
        repo: &str,
        token: &str,
        tag: &str,
        diagnostics: Arc<Mutex<Vec<String>>>,
    ) -> RemoteCheckout {
        let dir = tempfile::Builder::new()
            .prefix(&format!("speclink-phase3-{tag}-"))
            .tempdir()
            .expect("phase3 checkout tempdir");
        let root = dir.path().join("checkout");
        std::fs::create_dir_all(&root).expect("create phase3 checkout");
        std::fs::write(
            root.join(".speclink.yaml"),
            format!("remote:\n  url: {project_url}\n  repo: {repo}\n"),
        )
        .expect("write phase3 remote config");
        RemoteCheckout {
            dir,
            root,
            token: token.to_string(),
            diagnostics,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write_git_marker(&self, name: &str, content: &str) {
        std::fs::write(self.root.join(name), content).expect("write phase3 checkout marker");
        for args in [
            vec!["init", "-q"],
            vec!["add", "."],
            vec![
                "-c",
                "user.name=Phase3",
                "-c",
                "user.email=phase3@example.com",
                "commit",
                "-q",
                "-m",
                "phase3 marker",
            ],
        ] {
            let output = Command::new("git")
                .args(&args)
                .current_dir(&self.root)
                .output()
                .expect("run git for phase3 checkout");
            assert!(
                output.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        self.diagnostics
            .lock()
            .expect("phase3 diagnostic lock")
            .push(format!("checkout marker committed: {name}"));
    }

    pub fn run(&self, args: &[&str]) -> Output {
        let output = Command::new(phase3_cli_bin())
            .args(args)
            .current_dir(&self.root)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env("SPECLINK_TOKEN", &self.token)
            .env("HOME", self.dir.path())
            .env("USERPROFILE", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path())
            .output()
            .expect("run phase3 CLI");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        self.diagnostics
            .lock()
            .expect("phase3 diagnostic lock")
            .push(format!(
                "cli {:?}: status={} stdout={stdout:?} stderr={stderr:?}",
                args, output.status
            ));
        output
    }

    pub fn run_stdin(&self, args: &[&str], content: &str) -> Output {
        let mut child = Command::new(phase3_cli_bin())
            .args(args)
            .current_dir(&self.root)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env("SPECLINK_TOKEN", &self.token)
            .env("HOME", self.dir.path())
            .env("USERPROFILE", self.dir.path())
            .env("XDG_CONFIG_HOME", self.dir.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn phase3 CLI with stdin");
        child
            .stdin
            .as_mut()
            .expect("phase3 CLI stdin")
            .write_all(content.as_bytes())
            .expect("write phase3 CLI stdin");
        drop(child.stdin.take());
        let output = child.wait_with_output().expect("wait phase3 CLI");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        self.diagnostics
            .lock()
            .expect("phase3 diagnostic lock")
            .push(format!(
                "cli {:?} with stdin: status={} stdout={stdout:?} stderr={stderr:?}",
                args, output.status
            ));
        output
    }
}

pub fn phase3_harness() -> Phase3Harness {
    let gate = acquire_harness_gate();
    Phase3Harness {
        first: Phase3Server::new("first", "alpha", &["pm", "rd"]),
        second: Phase3Server::new("second", "beta", &["main"]),
        _gate: gate,
    }
}

fn seed_phase3_user(
    identity: &Arc<IdentitySqlite>,
    label: &str,
    project: &str,
    kind: &str,
) -> String {
    let invite = identity
        .create_invitation(NewInvitation {
            email: format!("{kind}-{label}@example.com"),
            display: format!("Phase3 {kind} {label}"),
            memberships: vec![project.to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("phase3 invite");
    identity
        .accept_invitation(&invite, "phase3-correct-horse")
        .expect("accept phase3 invitation")
}

static BUILD_PHASE3_CLI: Once = Once::new();

fn phase3_cli_bin() -> PathBuf {
    BUILD_PHASE3_CLI.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args(["build", "-p", "speclink-cli", "--bin", "speclink"])
            .status()
            .expect("spawn cargo build for phase3 CLI");
        assert!(status.success(), "building the phase3 CLI failed");
    });
    let test_bin = std::env::current_exe().expect("phase3 test executable");
    let profile_dir = test_bin
        .parent()
        .and_then(Path::parent)
        .expect("target profile directory");
    profile_dir.join(if cfg!(windows) {
        "speclink.exe"
    } else {
        "speclink"
    })
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
    let gate = acquire_harness_gate();
    let identity = Arc::new(IdentitySqlite::open_memory().expect("memory identity"));
    identity
        .create_project("demo", "Demo")
        .expect("seed demo project");
    identity
        .create_repo("demo", "backend", "backend")
        .expect("seed demo repo");
    identity
        .create_project("multi", "Multi")
        .expect("seed multi project");
    identity
        .create_repo("multi", "web", "web")
        .expect("seed multi web repo");
    identity
        .create_repo("multi", "api", "api")
        .expect("seed multi api repo");
    let invite = identity
        .create_invitation(NewInvitation {
            email: "dev@example.com".to_string(),
            display: DISPLAY.to_string(),
            memberships: vec!["demo".to_string(), "multi".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity
        .accept_invitation(&invite, "pw-correct-horse")
        .expect("accept");

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
    let server = RestartableServer::from_listener(listener, state);

    let dir = tempfile::tempdir().expect("tempdir");
    let registry = dir.path().join("connections.json");
    let origin = format!("http://{addr}");
    let mut entries = Vec::new();
    speclink_desktop_lib::connections::upsert_connection(&mut entries, &origin, "本地")
        .expect("seed entry");
    speclink_desktop_lib::connections::write_registry(&registry, &entries).expect("write registry");

    Harness {
        origin,
        identity,
        user_id,
        store,
        registry,
        server,
        _dir: dir,
        _gate: gate,
    }
}

/// 種 change `demo`（schema＋給定的 tasks.md 內容）進 demo/backend scope。
pub fn seed_change(store: &dyn TeamStore, tasks: &str) {
    seed_named_change(store, "demo", tasks);
}

/// 在 demo/backend scope 種指定名稱的 change，供重啟期間外部寫入情境使用。
pub fn seed_named_change(store: &dyn TeamStore, change: &str, tasks: &str) {
    let mut uow = store
        .begin_unit_of_work(
            &scope(),
            CommandContext {
                command: "seed".into(),
                actor: "seed".into(),
            },
        )
        .expect("begin uow");
    uow.create(
        DocumentId::ChangeMeta {
            change: change.into(),
        },
        "schema: spec-driven\n",
    );
    uow.create(
        DocumentId::ChangeArtifact {
            change: change.into(),
            artifact: "tasks.md".into(),
        },
        tasks,
    );
    store.commit(uow, Vec::new()).expect("seed commit");
}

/// 以 device flow（假瀏覽器直接核准）登入，讓 credential store 落下 refresh
/// credential。
pub fn device_login_approved(
    h: &Harness,
    store: &dyn speclink_remote::credentials::CredentialStore,
) {
    let identity = h.identity.clone();
    let user_id = h.user_id.clone();
    let opener = move |url: &str| {
        let code = url
            .split("user_code=")
            .nth(1)
            .expect("user_code 預填參數")
            .to_string();
        assert!(identity.approve_device(&code, &user_id).expect("approve"));
        Ok(())
    };
    device_login_two_segments(&h.origin, store, &h.registry, &opener).expect("device login");
}

/// 兩段編排（design 決策二）走完一次 device login：啟動段開「瀏覽器」（假開啟器
/// 就地核准）後，做一次觀測即得終態——測試不需要真的排程輪詢。回傳 access token。
fn device_login_two_segments(
    origin: &str,
    store: &dyn speclink_remote::credentials::CredentialStore,
    registry: &std::path::Path,
    opener: &dyn Fn(&str) -> Result<(), String>,
) -> Result<String, String> {
    use speclink_desktop_lib::connections::{
        device_login_observe, device_login_start, DeviceLoginObservation, DeviceLoginStart,
    };
    let auth = match device_login_start(origin, store, registry, opener)? {
        DeviceLoginStart::LoggedIn { access_token, .. } => return Ok(access_token),
        DeviceLoginStart::AwaitingApproval(auth) => auth,
        other => return Err(format!("device login did not complete: {other:?}")),
    };
    match device_login_observe(origin, &auth.device_code, store, registry)? {
        DeviceLoginObservation::LoggedIn { access_token, .. } => Ok(access_token),
        other => Err(format!("device login did not complete: {other:?}")),
    }
}

/// 給使用者簽一枚 PAT（資料面測試最短的 credential 路徑）。
pub fn pat_of(h: &Harness) -> String {
    let (_, pat) = h
        .identity
        .create_pat(&h.user_id, "test", None)
        .expect("pat");
    pat
}

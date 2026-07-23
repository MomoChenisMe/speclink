//! Speclink 桌面 app 的 Tauri 殼。
//!
//! 每個 #[tauri::command] 是對 speclink-desktop-core 的單行委派（薄包裝）——
//! 真正的邏輯與測試在 speclink-desktop-core，此層只做 IPC 接線。
//! Rust 側無 current-root 可變全域（workspace-session 決策 4）：所有讀寫
//! command 逐呼叫收 root，直通 desktop-core 的帶路徑函式；分頁切換不再改寫
//! 任何全域，前一分頁 in-flight 呼叫以其原 root 結算。

pub mod connections;
pub mod credentials;
pub mod event_manager;
pub mod remote;
pub mod tray;
mod watch;

#[cfg(target_os = "macos")]
mod panel;

use std::path::PathBuf;

use serde_json::Value;
use tauri::{Emitter, Manager};

#[tauri::command]
fn list_changes(root: PathBuf) -> Value {
    speclink_desktop_core::query::list_changes_at(&root)
}

#[tauri::command]
fn list_specs(root: PathBuf) -> Value {
    speclink_desktop_core::query::list_specs_at(&root)
}

#[tauri::command]
fn status(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::query::status_at(&root, &change)
}

#[tauri::command]
fn document(root: PathBuf, change: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::document_at(&root, &change, &artifact)
}

#[tauri::command]
fn spec_document(root: PathBuf, capability: String) -> Option<String> {
    speclink_desktop_core::query::spec_document_at(&root, &capability)
}

#[tauri::command]
fn search_workspace(root: PathBuf, query: String) -> Value {
    speclink_desktop_core::search::search_workspace_at(&root, &query)
}

#[tauri::command]
fn change_capabilities(root: PathBuf, change: String) -> Vec<String> {
    speclink_desktop_core::query::change_capabilities_at(&root, &change)
}

#[tauri::command]
fn change_meta(root: PathBuf, change: String) -> Option<Value> {
    speclink_desktop_core::manage::change_meta_at(&root, &change)
}

#[tauri::command]
fn delete_change(root: PathBuf, change: String) -> Result<(), String> {
    speclink_desktop_core::manage::delete_change_at(&root, &change)
}

#[tauri::command]
// 寫入型 command 一律 async＋spawn_blocking（design D2）：完成路徑可能秒級
// （git spawn 在部分環境極慢），非 async command 會佔用主執行緒凍結整窗。
// 委派移至執行緒池；並發寫回由 desktop-core 的全域寫鎖序列化。
async fn set_task_done(
    root: PathBuf,
    change: String,
    task: String,
    done: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_task_done_at(&root, &change, &task, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn set_all_tasks(root: PathBuf, change: String, done: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::set_all_tasks_at(&root, &change, done)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn move_task(
    root: PathBuf,
    change: String,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::move_task_at(&root, &change, from, to, before)
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
async fn reorder_card(
    root: PathBuf,
    kind: String,
    id: String,
    prev_id: Option<String>,
    next_id: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        speclink_desktop_core::manage::reorder_card_at(
            &root,
            &kind,
            &id,
            prev_id.as_deref(),
            next_id.as_deref(),
        )
    })
    .await
    .map_err(|e| format!("task write worker failed: {e}"))?
}

#[tauri::command]
fn validate(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::validate_at(&root, &change)
}

#[tauri::command]
fn analyze(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::analyze_at(&root, &change)
}

#[tauri::command]
fn archive(root: PathBuf, change: String) -> Result<Value, String> {
    speclink_desktop_core::verbs::archive_at(&root, &change)
}

#[tauri::command]
fn archived_changes(root: PathBuf) -> Value {
    speclink_desktop_core::cache::archived_changes_at(&root)
}

#[tauri::command]
fn archived_document(root: PathBuf, dated_name: String, artifact: String) -> Option<String> {
    speclink_desktop_core::query::archived_document_at(&root, &dated_name, &artifact)
}

#[tauri::command]
fn archived_capabilities(root: PathBuf, dated_name: String) -> Vec<String> {
    speclink_desktop_core::query::archived_capabilities_at(&root, &dated_name)
}

#[tauri::command]
fn list_discussions(root: PathBuf) -> Value {
    speclink_desktop_core::discussions::list_discussions_at(&root)
}

#[tauri::command]
fn discussion_document(root: PathBuf, slug: String) -> Option<String> {
    speclink_desktop_core::discussions::discussion_document_at(&root, &slug)
}

#[tauri::command]
fn promote_discussion(root: PathBuf, slug: String, name: Option<String>) -> Result<Value, String> {
    speclink_desktop_core::discussions::promote_discussion_at(&root, &slug, name.as_deref())
}

#[tauri::command]
fn archive_discussion(root: PathBuf, slug: String) -> Result<Value, String> {
    speclink_desktop_core::discussions::archive_discussion_at(&root, &slug)
}

/// 監看器槽位：重掛時整顆替換（drop 舊監看即停止）。
type WatcherState = std::sync::Mutex<Option<watch::WorkspaceWatcher>>;

/// git 身分預熱（design D1）：首抓可能秒級（GUI 進程 spawn git 極慢的環境），
/// 掛監看（＝專案成為活躍）時背景執行緒先填快取——首次勾選不再付這筆成本。
/// 失敗靜默，完成路徑的 cached_git_identity 會自行補抓。
fn prewarm_identity(root: PathBuf) {
    std::thread::spawn(move || {
        let _ = speclink_desktop_core::manage::cached_git_identity(&root);
    });
}

/// 開啟專案＝純探測：只回報本機 project／remoteBinding／uninitialized payload，
/// 錯誤走 Result；不改寫任何全域、不重掛
/// 監看——同一路徑重複呼叫冪等無副作用。監看跟隨由前端顯式 watch_workspace。
#[tauri::command]
fn open_project(path: String) -> Result<Value, String> {
    let probe = speclink_desktop_core::project::open_project_at(std::path::Path::new(&path))?;
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

#[tauri::command]
fn init_project(path: String, tools: Vec<String>) -> Result<Value, String> {
    let probe =
        speclink_desktop_core::project::init_project_at(std::path::Path::new(&path), &tools)?;
    serde_json::to_value(&probe).map_err(|e| e.to_string())
}

/// 啟動語境的預設目錄（決策 4 首啟路徑）：回傳行程啟動時的工作目錄——前端
/// 首啟無持久化分頁時據此顯式 openProjectAt（自專案目錄啟動的自動開啟語意
/// 凍結）。純讀、無任何可變全域。
#[tauri::command]
fn startup_dir() -> String {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .display()
        .to_string()
}

/// 專案統計（背景分頁徽章快照）：唯讀、收路徑參數。
#[tauri::command]
fn project_stats(path: String) -> Result<Value, String> {
    speclink_desktop_core::project::project_stats_at(std::path::Path::new(&path))
}

/// 監看重掛（決策 5）：顯式跟隨活躍 session——整顆替換單一 watcher 並預熱
/// git 身分；workspace-changed 事件 payload 為被監看的 root 字串（session 的
/// 事件來源據此過濾）。root 收字串並原樣回送，避免 PathBuf 往返改寫字面。
/// 監看不可用僅記錄、不回錯——app 照常、僅失去自動刷新（既有降級語意）。
#[tauri::command]
fn watch_workspace(app: tauri::AppHandle, root: String) {
    let root_path = PathBuf::from(&root);
    prewarm_identity(root_path.clone());
    let emitter = app.clone();
    let watcher = watch::resolve_watch_target(&root_path).and_then(|target| {
        watch::watch_openspec(&target, std::time::Duration::from_millis(400), move || {
            let _ = emitter.emit("workspace-changed", root.clone());
        })
    });
    if let Some(slot) = app.try_state::<WatcherState>() {
        *slot.lock().expect("watcher lock poisoned") = match watcher {
            Ok(w) => Some(w),
            Err(e) => {
                eprintln!("speclink-desktop: file watching unavailable: {e}");
                None
            }
        };
    }
}

#[tauri::command]
fn read_settings(root: PathBuf) -> Result<Value, String> {
    let snapshot = speclink_desktop_core::settings::read_settings_at(&root)?;
    serde_json::to_value(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_app_tools(root: PathBuf, tools: Vec<String>) -> Result<(), String> {
    speclink_desktop_core::settings::write_tools_at(&root, &tools)
}

#[tauri::command]
fn write_workflow_config(
    root: PathBuf,
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
) -> Result<(), String> {
    let fields = speclink_desktop_core::settings::WorkflowPolicyFields {
        locale,
        spec_locale,
        tdd,
        audit,
    };
    speclink_desktop_core::settings::write_workflow_fields_at(&root, &fields)
}

/// 寫入 config.yaml 的「專案說明」與「產出規則」。`context: None`＝不動、
/// `Some(文字)`＝設值（空白即移除鍵，core 落實）；`rules: None`＝不動、
/// `Some(節序清單)`＝整份代換。政策欄位不受本 command 波及。
#[tauri::command]
fn write_workflow_content(
    root: PathBuf,
    context: Option<String>,
    rules: Option<Vec<(String, Vec<String>)>>,
) -> Result<(), String> {
    let edit = match context {
        Some(text) => speclink_desktop_core::settings::ContextEdit::Set(text),
        None => speclink_desktop_core::settings::ContextEdit::Keep,
    };
    speclink_desktop_core::settings::write_workflow_content_at(&root, &edit, rules.as_deref())
}

/// 連線 registry 檔位置：appConfigDir 下 connections.json（design 決策 4）。
fn connections_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| connections::registry_path(&dir))
        .map_err(|e| format!("無法取得 app 設定目錄：{e}"))
}

/// 連線層狀態：credential 出入口（生產＝OS Keychain）與 per-origin 的
/// TokenManager（access token 記憶體持有——短效、絕不落盤、絕不過境 TS，
/// 決策 2；換發與 401 語意見 remote 模組）。
struct ConnectionsState {
    credentials: Box<dyn credentials::CredentialStore>,
    managers:
        std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<remote::TokenManager>>>,
    state_observer: std::sync::Arc<dyn Fn(remote::ConnectionStateEvent) + Send + Sync>,
}

impl ConnectionsState {
    /// 該 origin 的 TokenManager——惰性建立、跨 command 共用（needs-reauth
    /// 狀態與 token 快取都掛在同一顆上）。
    fn manager_for(&self, origin: &str) -> std::sync::Arc<remote::TokenManager> {
        let observer = self.state_observer.clone();
        self.managers
            .lock()
            .expect("manager lock")
            .entry(origin.to_string())
            .or_insert_with(|| {
                std::sync::Arc::new(remote::TokenManager::with_state_observer(
                    origin,
                    remote::DEFAULT_FAILURE_THRESHOLD,
                    move |event| observer(event),
                ))
            })
            .clone()
    }

    fn manager_for_connection(
        &self,
        connection_id: &str,
        origin: &str,
    ) -> std::sync::Arc<remote::TokenManager> {
        let manager = self.manager_for(origin);
        manager.bind_connection_id(connection_id);
        manager
    }
}

/// 條目的 TS 檢視：registry 欄位＋由 Keychain 推導的登入狀態。secret 不出現。
fn entry_view(entry: &connections::ConnectionEntry, state: &ConnectionsState) -> Value {
    let logged_in = state
        .credentials
        .get(&entry.origin, credentials::CredentialKind::Refresh)
        .ok()
        .flatten()
        .is_some()
        || state
            .credentials
            .get(&entry.origin, credentials::CredentialKind::Pat)
            .ok()
            .flatten()
            .is_some();
    let mut view = serde_json::to_value(entry).expect("entry serializes");
    view["loggedIn"] = Value::Bool(logged_in);
    view
}

#[tauri::command]
fn connection_list(app: tauri::AppHandle) -> Result<Vec<Value>, String> {
    let state = app.state::<std::sync::Arc<ConnectionsState>>();
    Ok(connections::read_registry(&connections_path(&app)?)
        .iter()
        .map(|entry| entry_view(entry, &state))
        .collect())
}

#[tauri::command]
fn connection_add(app: tauri::AppHandle, base_url: String, name: String) -> Result<Value, String> {
    let path = connections_path(&app)?;
    let mut entries = connections::read_registry(&path);
    let id = connections::upsert_connection(&mut entries, &base_url, &name)?;
    connections::write_registry(&path, &entries)?;
    let state = app.state::<std::sync::Arc<ConnectionsState>>();
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .expect("剛 upsert 的條目存在");
    Ok(entry_view(entry, &state))
}

/// chooser 的 checkout 綁定邊界：驗證 marker 一致性，或在無 marker 的 git
/// checkout 寫入與 CLI init remote 同構的 `.speclink.yaml` remote section。
#[tauri::command]
fn bind_checkout(
    path: String,
    origin: String,
    project: String,
    repo: String,
) -> Result<String, String> {
    connections::bind_checkout(std::path::Path::new(&path), &origin, &project, &repo)
}

/// 移除連線＝先走登出語意（撤銷＋刪 Keychain entry）再刪 registry 條目
/// （決策 6）。登出的本機刪除失敗會上拋、不刪條目——避免留下孤兒 credential。
#[tauri::command]
async fn connection_remove(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let path = connections_path(&app)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut entries = connections::read_registry(&path);
        if let Some(entry) = entries.iter().find(|e| e.id == id) {
            let origin = entry.origin.clone();
            connections::logout(&origin, &*state.credentials, &path)?;
            state.managers.lock().expect("manager lock").remove(&origin);
            entries = connections::read_registry(&path); // logout 剛清了身分欄位
            entries.retain(|e| e.id != id);
            connections::write_registry(&path, &entries)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("connection worker failed: {e}"))?
}

/// device login（決策 5）：網路與輪詢在 blocking pool 執行；瀏覽器開啟走
/// tauri-plugin-opener。回 TS 的只有狀態與顯示名——access token 存入記憶體
/// 持有、refresh credential 已由編排層寫入 Keychain。
#[tauri::command]
async fn device_login(app: tauri::AppHandle, origin: String) -> Result<Value, String> {
    let path = connections_path(&app)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    let opener_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let opener = move |url: &str| -> Result<(), String> {
            use tauri_plugin_opener::OpenerExt;
            opener_app
                .opener()
                .open_url(url.to_string(), None::<String>)
                .map_err(|e| format!("無法開啟系統瀏覽器：{e}"))
        };
        match connections::device_login(&origin, &*state.credentials, &path, &opener)? {
            connections::DeviceLoginOutcome::LoggedIn {
                display,
                access_token,
            } => {
                state.manager_for(&origin).adopt_access_token(&access_token);
                Ok(serde_json::json!({ "status": "loggedIn", "display": display }))
            }
            connections::DeviceLoginOutcome::Unsupported => {
                Ok(serde_json::json!({ "status": "unsupported" }))
            }
            connections::DeviceLoginOutcome::Denied => {
                Ok(serde_json::json!({ "status": "denied" }))
            }
            connections::DeviceLoginOutcome::Expired => {
                Ok(serde_json::json!({ "status": "expired" }))
            }
        }
    })
    .await
    .map_err(|e| format!("login worker failed: {e}"))?
}

/// PAT 登入：PAT 僅單次過境此參數（不回讀、不入 log、不進 TS 狀態）。
#[tauri::command]
async fn pat_login(app: tauri::AppHandle, origin: String, pat: String) -> Result<Value, String> {
    let path = connections_path(&app)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let display = connections::pat_login(&origin, &pat, &*state.credentials, &path)?;
        state.manager_for(&origin).adopt_access_token(&pat);
        Ok(serde_json::json!({ "status": "loggedIn", "display": display }))
    })
    .await
    .map_err(|e| format!("login worker failed: {e}"))?
}

/// 連線的 runtime 狀態（remote 決策 4）：needs-reauth 布林＋繁中訊息。TS 據此
/// 呈現需重新認證，token 本身永不出現。
#[tauri::command]
fn connection_state(app: tauri::AppHandle, origin: String) -> Value {
    let state = app.state::<std::sync::Arc<ConnectionsState>>();
    let message = state.manager_for(&origin).needs_reauth();
    serde_json::json!({ "needsReauth": message.is_some(), "message": message })
}

/// 登出（決策 6）：盡力撤銷＋必刪本機，並丟棄記憶體中的 access token。
#[tauri::command]
async fn connection_logout(app: tauri::AppHandle, origin: String) -> Result<Value, String> {
    let path = connections_path(&app)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let outcome = connections::logout(&origin, &*state.credentials, &path)?;
        state.managers.lock().expect("manager lock").remove(&origin);
        Ok(serde_json::json!({
            "revokedOnServer": outcome.revoked_on_server,
            "patNotice": outcome.pat_notice,
        }))
    })
    .await
    .map_err(|e| format!("logout worker failed: {e}"))?
}

// --- remote workspace 資料面（remote-data-source 決策 6、7） ---

/// connectionId → origin（registry 查找）。條目消失＝連線已被移除。
fn connection_origin(app: &tauri::AppHandle, connection_id: &str) -> Result<String, String> {
    let path = connections_path(app)?;
    connections::read_registry(&path)
        .iter()
        .find(|entry| entry.id == connection_id)
        .map(|entry| entry.origin.clone())
        .ok_or_else(|| "連線不存在——請重新設定 server 連線".to_string())
}

/// 資料面命令的共用骨架：解析 origin、取 TokenManager，於 blocking pool 以
/// 無狀態重建的 RemoteWorkspace 執行一擊。
async fn with_remote<T, F>(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    call: F,
) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&remote::RemoteWorkspace, &dyn credentials::CredentialStore) -> Result<T, String>
        + Send
        + 'static,
{
    let origin = connection_origin(&app, &connection_id)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = state.manager_for_connection(&connection_id, &origin);
        let workspace = remote::RemoteWorkspace::at(&origin, &project, &repo, &manager);
        call(&workspace, &*state.credentials)
    })
    .await
    .map_err(|e| format!("remote worker failed: {e}"))?
}

/// remote settings 專用骨架：錯誤保留 reason/status 的結構化形狀，供前端
/// 辨識 revision_conflict；其他資料面仍沿用既有單行 String 契約。
async fn with_remote_settings<T, F>(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    call: F,
) -> Result<T, remote::RemoteSettingsError>
where
    T: Send + 'static,
    F: FnOnce(
            &remote::RemoteWorkspace,
            &dyn credentials::CredentialStore,
        ) -> Result<T, remote::RemoteSettingsError>
        + Send
        + 'static,
{
    let origin =
        connection_origin(&app, &connection_id).map_err(remote::RemoteSettingsError::command)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = state.manager_for_connection(&connection_id, &origin);
        let workspace = remote::RemoteWorkspace::at(&origin, &project, &repo, &manager);
        call(&workspace, &*state.credentials)
    })
    .await
    .map_err(|error| {
        remote::RemoteSettingsError::command(format!("remote worker failed: {error}"))
    })?
}

/// 開啟 remote workspace（決策 6，fail-closed）：以 project[/repo] 識別
/// handshake，成功回 project/repo 顯示名與 capability 描述；失敗原樣回錯、
/// 不建立任何 runtime 狀態。
#[tauri::command]
async fn remote_open(
    app: tauri::AppHandle,
    connection_id: String,
    target: String,
) -> Result<Value, remote::RemoteOpenFailure> {
    let origin = connection_origin(&app, &connection_id)
        .map_err(remote::RemoteOpenFailure::unknown)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = state.manager_for_connection(&connection_id, &origin);
        let (_, info) = remote::open_workspace(&origin, &target, &manager, &*state.credentials)
            .map_err(remote::RemoteOpenFailure::from)?;
        serde_json::to_value(&info)
            .map_err(|error| remote::RemoteOpenFailure::unknown(error.to_string()))
    })
    .await
    .map_err(|error| {
        remote::RemoteOpenFailure::unknown(format!("remote worker failed: {error}"))
    })?
}

/// remote Workflow 設定快照：`/config` 原文經 desktop-core 文字 seam，並帶
/// 與該原文同一 response 的 revision。
#[tauri::command]
async fn remote_read_settings(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<remote::RemoteSettingsSnapshot, remote::RemoteSettingsError> {
    with_remote_settings(app, connection_id, project, repo, |ws, credentials| {
        ws.read_settings(credentials)
    })
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn remote_write_workflow_config(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
    expected_revision: u64,
) -> Result<u64, remote::RemoteSettingsError> {
    let fields = speclink_desktop_core::settings::WorkflowPolicyFields {
        locale,
        spec_locale,
        tdd,
        audit,
    };
    with_remote_settings(app, connection_id, project, repo, move |ws, credentials| {
        ws.write_workflow_fields(credentials, &fields, expected_revision)
    })
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn remote_write_workflow_content(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    context: Option<String>,
    rules: Option<Vec<(String, Vec<String>)>>,
    expected_revision: u64,
) -> Result<u64, remote::RemoteSettingsError> {
    with_remote_settings(app, connection_id, project, repo, move |ws, credentials| {
        ws.write_workflow_content(
            credentials,
            context.as_deref(),
            rules.as_deref(),
            expected_revision,
        )
    })
    .await
}

#[tauri::command]
async fn remote_list_changes(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, |ws, credentials| {
        let changes = ws.list_changes(credentials).map_err(|e| e.message)?;
        serde_json::to_value(&changes).map_err(|e| e.to_string())
    })
    .await
}

/// chooser 的 identity-scoped `/scopes` 清單；選定 repo 前不建立 remote session。
#[tauri::command]
async fn remote_scopes(app: tauri::AppHandle, connection_id: String) -> Result<Value, String> {
    let origin = connection_origin(&app, &connection_id)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = state.manager_for_connection(&connection_id, &origin);
        let scopes = remote::list_scopes(&origin, &manager, &*state.credentials)
            .map_err(|error| error.message)?;
        serde_json::to_value(scopes).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Local-to-remote migration: import first, then retain a dated local backup
/// and write the checkout marker before returning the conversion result.
#[tauri::command]
async fn migrate_workspace(
    app: tauri::AppHandle,
    connection_id: String,
    root: PathBuf,
    project: String,
    repo: String,
) -> Result<Value, String> {
    let origin = connection_origin(&app, &connection_id)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        let manager = state.manager_for_connection(&connection_id, &origin);
        let result = remote::migrate_workspace(
            &root,
            &origin,
            &project,
            &repo,
            &manager,
            &*state.credentials,
        )?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("migration worker failed: {error}"))?
}

/// Resolve local/remote coexistence in favor of server truth. The UI performs
/// a read-only handshake first; this command only retains local openspec/ as a
/// dated backup and never sends an import request.
#[tauri::command]
async fn adopt_remote_workspace(root: PathBuf) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let result = remote::adopt_remote_workspace(&root)?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("remote adoption worker failed: {error}"))?
}

#[tauri::command]
async fn remote_list_specs(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, |ws, credentials| {
        let specs = ws.list_specs(credentials).map_err(|e| e.message)?;
        serde_json::to_value(&specs).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn remote_list_archived(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, |ws, credentials| {
        let archived = ws.list_archived(credentials).map_err(|e| e.message)?;
        serde_json::to_value(&archived).map_err(|e| e.to_string())
    })
    .await
}

/// 正典 spec 內文：404 與本地語意一致回 null。
#[tauri::command]
async fn remote_spec_document(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    capability: String,
) -> Result<Option<String>, String> {
    with_remote(
        app,
        connection_id,
        project,
        repo,
        move |ws, credentials| match ws.spec_document(credentials, &capability) {
            Ok(document) => Ok(Some(document.content)),
            Err(error) if error.status == Some(404) => Ok(None),
            Err(error) => Err(error.message),
        },
    )
    .await
}

#[tauri::command]
async fn remote_search_workspace(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    query: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let search = ws
            .search_workspace(credentials, &query)
            .map_err(|e| e.message)?;
        serde_json::to_value(&search).map_err(|e| e.to_string())
    })
    .await
}

/// 封存 artifact 內文：404 與本地語意一致回 null。
#[tauri::command]
async fn remote_archived_document(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    dated_name: String,
    artifact: String,
) -> Result<Option<String>, String> {
    with_remote(
        app,
        connection_id,
        project,
        repo,
        move |ws, credentials| match ws.archived_document(credentials, &dated_name, &artifact) {
            Ok(document) => Ok(Some(document.content)),
            Err(error) if error.status == Some(404) => Ok(None),
            Err(error) => Err(error.message),
        },
    )
    .await
}

#[tauri::command]
async fn remote_archived_capabilities(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    dated_name: String,
) -> Result<Vec<String>, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.archived_capabilities(credentials, &dated_name)
            .map_err(|error| error.message)
    })
    .await
}

#[tauri::command]
async fn remote_status(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let status = ws
            .change_status(credentials, &change)
            .map_err(|e| e.message)?;
        serde_json::to_value(&status).map_err(|e| e.to_string())
    })
    .await
}

/// artifact 內文：404（change 或 artifact 不存在）與本地語意一致回 null。
#[tauri::command]
async fn remote_document(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
    artifact: String,
) -> Result<Option<String>, String> {
    with_remote(
        app,
        connection_id,
        project,
        repo,
        move |ws, credentials| match ws.document(credentials, &change, &artifact) {
            Ok(doc) => Ok(Some(doc.content)),
            Err(e) if e.status == Some(404) => Ok(None),
            Err(e) => Err(e.message),
        },
    )
    .await
}

#[tauri::command]
async fn remote_set_task_done(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
    task: String,
    done: bool,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.set_task_done(credentials, &change, &task, done)
            .map_err(|e| e.message)
    })
    .await
}

/// 組合類批次寫回（決策 1 (b)）：中途失敗中止並回報已完成筆數。
#[tauri::command]
async fn remote_set_all_tasks(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
    done: bool,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.set_all_tasks(credentials, &change, done)
            .map(|_| ())
            .map_err(|failure| {
                format!(
                    "批次寫回中止（已完成 {} 筆）：{}",
                    failure.completed, failure.error.message
                )
            })
    })
    .await
}

#[tauri::command]
async fn remote_archive(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let archived = ws.archive(credentials, &change).map_err(|e| e.message)?;
        serde_json::to_value(&archived).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn remote_validate(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let result = ws.validate(credentials, &change).map_err(|e| e.message)?;
        serde_json::to_value(&result).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn remote_analyze(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let report = ws.analyze(credentials, &change).map_err(|e| e.message)?;
        serde_json::to_value(&report).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn remote_delete_change(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
    force: bool,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.delete_change(credentials, &change, force)
            .map_err(|e| e.message)
    })
    .await
}

#[tauri::command]
async fn remote_move_task(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    change: String,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.move_task(credentials, &change, from, to, before)
            .map_err(|e| e.message)
    })
    .await
}

/// 看板拖排直達（remote-board-order 決策 5）：鄰居定址與本地 reorder_card
/// 同形；順序寫入 scope 的 board resource，不觸碰卡片 meta。
#[tauri::command]
async fn remote_reorder_card(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    kind: String,
    id: String,
    prev_id: Option<String>,
    next_id: Option<String>,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.reorder_card(
            credentials,
            &kind,
            &id,
            prev_id.as_deref(),
            next_id.as_deref(),
        )
        .map_err(|e| e.message)
    })
    .await
}

#[tauri::command]
async fn remote_list_discussions(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, |ws, credentials| {
        let lists = ws.list_discussions(credentials).map_err(|e| e.message)?;
        serde_json::to_value(&lists).map_err(|e| e.to_string())
    })
    .await
}

/// 討論內文：404 與本地語意一致回 null。
#[tauri::command]
async fn remote_discussion_document(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    slug: String,
) -> Result<Option<String>, String> {
    with_remote(
        app,
        connection_id,
        project,
        repo,
        move |ws, credentials| match ws.discussion_document(credentials, &slug) {
            Ok(shown) => Ok(Some(shown.content)),
            Err(e) if e.status == Some(404) => Ok(None),
            Err(e) => Err(e.message),
        },
    )
    .await
}

#[tauri::command]
async fn remote_promote_discussion(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    slug: String,
    name: Option<String>,
) -> Result<Value, String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        let promoted = ws
            .promote_discussion(credentials, &slug, name.as_deref())
            .map_err(|e| e.message)?;
        serde_json::to_value(&promoted).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
async fn remote_archive_discussion(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
    slug: String,
) -> Result<(), String> {
    with_remote(app, connection_id, project, repo, move |ws, credentials| {
        ws.archive_discussion(credentials, &slug)
            .map(|_| ())
            .map_err(|e| e.message)
    })
    .await
}

/// remote 事件訂閱的生產退避序列（決策 5：指數退避、封頂 30s）。
const REMOTE_BACKOFF: [std::time::Duration; 6] = [
    std::time::Duration::from_millis(500),
    std::time::Duration::from_secs(1),
    std::time::Duration::from_secs(2),
    std::time::Duration::from_secs(5),
    std::time::Duration::from_secs(10),
    std::time::Duration::from_secs(30),
];

/// remote session 的 locator key（與 TS session.ts 的 locatorKey 同構）。
fn remote_locator_key(connection_id: &str, project: &str, repo: &str) -> String {
    format!("remote:{connection_id}/{project}/{repo}")
}

/// 註冊 remote 分頁的事件訂閱（決策 3）：同 connection 同 scope 的 sessions
/// 共用單一 SSE 流；invalidate 即 emit remote-workspace-changed（payload＝
/// locator key），前端據此經 Query 重讀。
#[tauri::command]
fn remote_watch(
    app: tauri::AppHandle,
    connection_id: String,
    project: String,
    repo: String,
) -> Result<(), String> {
    let origin = connection_origin(&app, &connection_id)?;
    let state = app
        .state::<std::sync::Arc<ConnectionsState>>()
        .inner()
        .clone();
    let events = app
        .state::<std::sync::Arc<event_manager::EventManager>>()
        .inner()
        .clone();
    let key = remote_locator_key(&connection_id, &project, &repo);
    let base = format!("{origin}/api/speclink/v1/projects/{project}");
    let manager = state.manager_for_connection(&connection_id, &origin);
    let sub_state = state.clone();
    let sub_manager = manager.clone();
    let sub_base = base.clone();
    let sub_repo = repo.clone();
    let etag_repo = repo;
    events.register(
        &key,
        move |last| {
            sub_manager.execute(&*sub_state.credentials, |token| {
                speclink_remote::events::subscribe(&sub_base, token, Some(&sub_repo), last)
            })
        },
        move || {
            manager.execute(&*state.credentials, |token| {
                speclink_remote::events::sync_state(&base, token, Some(&etag_repo))
            })
        },
        REMOTE_BACKOFF.to_vec(),
    );
    Ok(())
}

/// 退出 remote 分頁的事件訂閱：最後一個 session 退出即收束該流。
#[tauri::command]
fn remote_unwatch(app: tauri::AppHandle, connection_id: String, project: String, repo: String) {
    let events = app.state::<std::sync::Arc<event_manager::EventManager>>();
    events.unregister(&remote_locator_key(&connection_id, &project, &repo));
}

/// 系統匣面板 toggle（tray-status-menu「面板樣式（macOS）」）：macOS 委派
/// panel 模組；其他平台恆回 Err（面板樣式偏好在非 macOS 不可達，此為守門）。
#[cfg(target_os = "macos")]
#[tauri::command]
fn toggle_tray_panel(app: tauri::AppHandle) -> Result<(), String> {
    panel::toggle(&app)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
fn toggle_tray_panel() -> Result<(), String> {
    Err("tray panel is macOS-only".to_string())
}

/// 結束 app（tray-status-menu「開啟視窗與結束動作」）：webview 無法自行結束
/// 行程的能力橋接——面板動作區「結束」經此命令結束整個 app。
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init());
    // 面板樣式相依僅 macOS 註冊（design D6）：positioner 供 tray 相對定位、
    // nspanel 供不搶焦點的 NSPanel 容器。
    #[cfg(target_os = "macos")]
    let builder = builder
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_nspanel::init());
    builder
        .setup(|app| {
            // 監看槽位（決策 5）：啟動僅註冊空槽——前端還原分頁後以
            // watch_workspace 顯式掛上活躍專案（監看與資料載入同由前端
            // session 驅動；建立失敗僅記錄，app 照常、只失去自動刷新）。
            let slot: WatcherState = std::sync::Mutex::new(None);
            app.manage(slot);
            // 連線層：credential 生產出入口＝OS Keychain；access token 記憶體持有。
            let state_emitter = app.handle().clone();
            app.manage(std::sync::Arc::new(ConnectionsState {
                credentials: Box::new(credentials::KeyringCredentialStore),
                managers: std::sync::Mutex::new(std::collections::HashMap::new()),
                state_observer: std::sync::Arc::new(move |event| {
                    let _ = state_emitter.emit(remote::REMOTE_CONNECTION_STATE_EVENT, event);
                }),
            }));
            // remote 事件中樞：invalidate → remote-workspace-changed（payload＝
            // locator key），前端 session 據此過濾重讀。
            let emitter = app.handle().clone();
            app.manage(std::sync::Arc::new(event_manager::EventManager::new(
                move |key: String| {
                    let _ = emitter.emit("remote-workspace-changed", key);
                },
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_changes,
            list_specs,
            status,
            document,
            spec_document,
            search_workspace,
            change_capabilities,
            change_meta,
            delete_change,
            set_task_done,
            set_all_tasks,
            move_task,
            reorder_card,
            validate,
            analyze,
            archive,
            archived_changes,
            archived_document,
            archived_capabilities,
            list_discussions,
            discussion_document,
            promote_discussion,
            archive_discussion,
            open_project,
            init_project,
            startup_dir,
            project_stats,
            watch_workspace,
            read_settings,
            write_app_tools,
            write_workflow_config,
            write_workflow_content,
            connection_list,
            connection_add,
            bind_checkout,
            connection_remove,
            device_login,
            pat_login,
            connection_state,
            connection_logout,
            remote_open,
            remote_read_settings,
            remote_write_workflow_config,
            remote_write_workflow_content,
            remote_scopes,
            migrate_workspace,
            adopt_remote_workspace,
            remote_list_changes,
            remote_list_specs,
            remote_list_archived,
            remote_spec_document,
            remote_search_workspace,
            remote_archived_document,
            remote_archived_capabilities,
            remote_status,
            remote_document,
            remote_set_task_done,
            remote_set_all_tasks,
            remote_archive,
            remote_validate,
            remote_analyze,
            remote_delete_change,
            remote_move_task,
            remote_reorder_card,
            remote_list_discussions,
            remote_discussion_document,
            remote_promote_discussion,
            remote_archive_discussion,
            remote_watch,
            remote_unwatch,
            toggle_tray_panel,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Speclink desktop app");
}

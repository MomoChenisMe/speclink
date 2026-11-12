//! connection registry（design 決策 4）與登入／登出編排（決策 3、5、6）。
//!
//! registry：saved servers 的無 secret profile 清單，存 app 設定目錄
//! （appConfigDir）下的 connections.json。條目＝{id、origin（baseUrl 正規化）、
//! name、lastActorDisplay?}——絕不含任何 token 欄位（規格「connection registry
//! 不含 secret 且跨重啟保留」）。一 origin 一條目，重複新增即更新顯示名。
//! 壞 JSON 歸零清單（與分頁持久化同一寬容哲學）。
//!
//! 編排：connection_login／pat_login／logout／refresh_connection 收注入的
//! CredentialStore 與 [`DesktopLoginIo`]、無 tauri 型別——tauri 接線在 lib.rs。
//! 探索、選擇、三種登入與登出都委派給 speclink-remote 的同一份登入程式
//! （client-browser-login D7），和 CLI 共用；等待瀏覽器或裝置核准的迴圈也在那邊，
//! 這裡只把階段轉成送往前端的事件、把取消接到旗標。secret 只在此層與
//! CredentialStore 之間流動，錯誤訊息與事件不夾帶 secret。

use serde::{Deserialize, Serialize};
use speclink_remote::credentials::{CredentialKind, CredentialStore};
use speclink_remote::device;
use speclink_remote::login::{
    self as remote_login, LoginOutcome, LoginPhase, LoginTarget, PatLoginOutcome, Plan,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub use speclink_remote::login::normalize_origin;

/// registry 的一個 saved server 條目。欄位全集即此——序列化測試釘死。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEntry {
    pub id: String,
    /// 正規化後的 server origin（scheme://host[:port]，小寫、無路徑）。
    pub origin: String,
    pub name: String,
    /// 最後登入身分的顯示名；未登入過即省略。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_actor_display: Option<String>,
}

/// registry 檔在 app 設定目錄下的固定位置。
pub fn registry_path(config_dir: &Path) -> PathBuf {
    config_dir.join("connections.json")
}

/// 讀 registry：檔案不存在或壞 JSON 一律歸零清單、不崩潰。
pub fn read_registry(path: &Path) -> Vec<ConnectionEntry> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// registry 檔的行程內互斥：command async 化後 add／remove 的
/// read→modify→write 不再由主執行緒天然序列化，跨執行緒併行的整檔覆寫會
/// 互相蓋掉——所有 mutation 先取這把鎖（跨行程仍無鎖，屬既有邊界）。
static REGISTRY_MUTATION: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 取得 registry mutation 鎖；poisoned（前一持有者 panic）照常放行——
/// registry 操作間無跨呼叫的記憶體不變量，鎖只管檔案序列化。
pub fn registry_guard() -> std::sync::MutexGuard<'static, ()> {
    REGISTRY_MUTATION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 新增連線的完整交易（鎖內 read→upsert→write）：併行呼叫不互相蓋寫。
/// 回傳剛落地的條目。
pub fn add_connection(
    path: &Path,
    base_url: &str,
    name: &str,
) -> Result<ConnectionEntry, String> {
    let _guard = registry_guard();
    let mut entries = read_registry(path);
    let id = upsert_connection(&mut entries, base_url, name)?;
    write_registry(path, &entries)?;
    Ok(entries
        .into_iter()
        .find(|e| e.id == id)
        .expect("剛 upsert 的條目存在"))
}

/// 移除連線的完整交易（鎖內 read→logout→write）：先走登出語意（撤銷＋刪
/// Keychain entry）再刪條目；登出的本機刪除失敗上拋、不刪條目——避免留下
/// 孤兒 credential。回傳被移除條目的 origin（無此 id＝`Ok(None)`，冪等）。
pub fn remove_connection(
    path: &Path,
    id: &str,
    credentials: &dyn CredentialStore,
) -> Result<Option<String>, String> {
    let _guard = registry_guard();
    let mut entries = read_registry(path);
    let Some(entry) = entries.iter().find(|e| e.id == id) else {
        return Ok(None);
    };
    let origin = entry.origin.clone();
    logout(&origin, credentials, path)?;
    entries = read_registry(path); // logout 剛清了身分欄位
    entries.retain(|e| e.id != id);
    write_registry(path, &entries)?;
    Ok(Some(origin))
}

/// 寫 registry：建父目錄後整檔覆寫。
pub fn write_registry(path: &Path, entries: &[ConnectionEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("無法建立設定目錄：{e}"))?;
    }
    let text =
        serde_json::to_string_pretty(entries).map_err(|e| format!("無法序列化連線清單：{e}"))?;
    std::fs::write(path, text).map_err(|e| format!("無法寫入連線清單：{e}"))
}

/// 新增或更新連線：同 origin 已存在即更新顯示名（id 穩定），否則新增條目。
/// 回傳該條目的 id。
pub fn upsert_connection(
    entries: &mut Vec<ConnectionEntry>,
    base_url: &str,
    name: &str,
) -> Result<String, String> {
    let origin = normalize_origin(base_url)?;
    if let Some(existing) = entries.iter_mut().find(|e| e.origin == origin) {
        existing.name = name.to_string();
        return Ok(existing.id.clone());
    }
    let id = format!(
        "conn_{}",
        ulid::Ulid::new().to_string().to_ascii_lowercase()
    );
    entries.push(ConnectionEntry {
        id: id.clone(),
        origin,
        name: name.to_string(),
        last_actor_display: None,
    });
    Ok(id)
}

/// `inspect_checkout` 的零寫入結果：確認過的 checkout 根路徑，以及要在 picker
/// 中預選的既有 built-in 工具選集（claude／codex／copilot／pi／cursor，順序穩定）。IPC 序列化為
/// camelCase `{ root, tools }`——不攜帶任何 credential 或 Server 資料。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutInspection {
    pub root: String,
    pub tools: Vec<String>,
}

/// 一次驗證資料夾與 marker 一致性，回傳確認過的 checkout 根路徑（display 字串）。
/// 零寫入：`inspect_checkout` 與 `bind_checkout` 共用此邊界，因此提交時重做的是
/// 同一組檢查。相符 marker 只接受相同 origin/repo；無 marker 時要求 `.git` 存在。
fn validate_checkout(
    root: &Path,
    selected_origin: &str,
    selected_repo: &str,
) -> Result<String, String> {
    if !root.is_dir() {
        return Err(format!(
            "無法連接 checkout：{} 不是現有資料夾",
            root.display()
        ));
    }
    let selected_origin = normalize_origin(selected_origin)?;
    let app =
        speclink_core::config::AppConfig::load(&root.join(".speclink.yaml")).map_err(|e| e.to_string())?;

    if let Some(remote) = app.remote {
        let marker_url = remote
            .url
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| "remote marker 缺少 remote.url，無法確認 checkout 綁定".to_string())?;
        let marker_origin = normalize_origin(&marker_url)
            .map_err(|e| format!("remote marker 的 url「{marker_url}」無效：{e}"))?;
        let marker_repo = remote
            .repo
            .filter(|repo| !repo.trim().is_empty())
            .unwrap_or_else(|| "（未指定 repo）".to_string());
        if marker_origin != selected_origin || marker_repo != selected_repo {
            return Err(format!(
                "此資料夾的 remote marker 指向 {marker_origin} / {marker_repo}，與所選 {selected_origin} / {selected_repo} 不一致"
            ));
        }
    } else if !root.join(".git").exists() {
        return Err("選擇的資料夾不是 Git repository，無法連接 checkout".to_string());
    }
    Ok(root.display().to_string())
}

/// 決定 picker 的預選：`.speclink.yaml` 記錄了 built-in 選集就用它（claude／codex／copilot／
/// pi／cursor，去重、順序穩定）；缺清單、或清單裡沒有任何可解析的內建名（只有描述子／未知名）
/// 時，只依實際 Claude／Codex footprint 預選（`.github/`、`.pi/`、`.cursor/` 不算 Copilot、Pi、
/// Cursor 的 footprint），絕不補 Claude fallback。
fn preselected_tools(root: &Path) -> Vec<String> {
    let app = match speclink_core::config::AppConfig::load(&root.join(".speclink.yaml")) {
        Ok(app) => app,
        Err(_) => return Vec::new(),
    };
    // 工具選集的解析只有 core 一份（ToolSelection）：這裡不再自寫 tools 迴圈。
    let selection = speclink_core::init::ToolSelection::resolve(root, &app);
    let picked = if selection.legacy_fallback || selection.builtins.is_empty() {
        speclink_core::init::detect_footprint_tools(root)
    } else {
        selection.builtins
    };
    picked.iter().map(|t| t.name().to_string()).collect()
}

/// 先檢查階段（零寫入）：驗證資料夾與 marker，回傳確認過的根路徑與要預選的工具選集。
pub fn inspect_checkout(
    root: &Path,
    selected_origin: &str,
    selected_project: &str,
    selected_repo: &str,
) -> Result<CheckoutInspection, String> {
    let _ = selected_project;
    let root_str = validate_checkout(root, selected_origin, selected_repo)?;
    Ok(CheckoutInspection {
        root: root_str,
        tools: preselected_tools(root),
    })
}

/// 提交階段：重做 marker 邊界驗證，無 marker 時寫入與 CLI init remote 同構的
/// remote section，再對非空 built-in 選集執行 Core reconciliation（生成所選、
/// 清理未選、保留自訂描述子與使用者內容）。全部成功才回傳 checkout 根路徑供
/// remote locator 的 checkoutRoot 使用；任一步失敗回傳單行 Err、不回傳 root。
pub fn bind_checkout(
    root: &Path,
    selected_origin: &str,
    selected_project: &str,
    selected_repo: &str,
    tools: &[String],
) -> Result<String, String> {
    // 空選集／未知工具在任何寫入之前被拒（fail loud）。
    let selected =
        speclink_core::init::parse_tool_names(tools).map_err(|e| single_line(&e.to_string()))?;
    if selected.is_empty() {
        return Err("請至少選擇一個內建工具：claude、codex、copilot、pi、cursor".to_string());
    }
    let root_str = validate_checkout(root, selected_origin, selected_repo)?;

    // 無 marker 時先寫入同構 remote section，讓後續 reconciliation 認得 remote 模式。
    if speclink_core::config::AppConfig::load(&root.join(".speclink.yaml"))
        .map_err(|e| e.to_string())?
        .remote
        .is_none()
    {
        let normalized = normalize_origin(selected_origin)?;
        let project_url = format!("{normalized}/api/speclink/v1/projects/{selected_project}");
        speclink_core::config::write_remote_section(root, &project_url, Some(selected_repo))
            .map_err(|e| format!("無法寫入 remote marker：{e}"))?;
    }

    speclink_core::init::reconcile_builtin_tools(root, &selected)
        .map_err(|e| single_line(&e.to_string()))?;
    Ok(root_str)
}

/// 把多行錯誤壓成單行（IPC 錯誤在 UI 只呈現一行）。
fn single_line(message: &str) -> String {
    message.split('\n').next().unwrap_or(message).trim().to_string()
}

// --- 登入／登出編排（client-browser-login D7、D9） ---

/// 送往前端的登入階段事件名稱。
pub const LOGIN_PHASE_EVENT: &str = "connection-login-phase";

/// 送往前端的登入階段（事件 [`LOGIN_PHASE_EVENT`] 的內容）。`kind` 與欄位名和
/// 前端的 `ConnectionPhase` 一致，前端收到即可直接顯示。只帶使用者要看的東西：
/// 授權網址，或裝置碼與驗證網址，以及截止時刻（epoch 毫秒）——授權網址含
/// state 與 PKCE challenge，不含授權碼、PKCE verifier 或任何 token。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoginPhaseEvent {
    pub origin: String,
    #[serde(flatten)]
    pub phase: PhaseView,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PhaseView {
    #[serde(rename_all = "camelCase")]
    AwaitingBrowser { url: String, expires_at: u64 },
    #[serde(rename_all = "camelCase")]
    AwaitingApproval {
        user_code: String,
        verification_uri: String,
        expires_at: u64,
    },
}

/// Desktop 的登入 IO：階段經 `emit` 送往前端、瀏覽器經 opener 外掛開啟、
/// 取消讀 `cancel` 旗標（由 connection_login_cancel 設定）。
pub struct DesktopLoginIo<'a> {
    pub origin: &'a str,
    pub emit: &'a (dyn Fn(LoginPhaseEvent) + Sync),
    pub open_browser: &'a (dyn Fn(&str) -> Result<(), String> + Sync),
    pub cancel: &'a AtomicBool,
}

/// 取消的反應時間：等待被切成這麼長的片段，每段之間看一次旗標。
const CANCEL_POLL: std::time::Duration = std::time::Duration::from_millis(100);

fn expires_at_ms(expires_in: u64) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    now + expires_in * 1000
}

impl remote_login::LoginIo for DesktopLoginIo<'_> {
    fn phase(&self, phase: LoginPhase) {
        let phase = match phase {
            LoginPhase::AwaitingBrowser { url, expires_in } => PhaseView::AwaitingBrowser {
                url,
                expires_at: expires_at_ms(expires_in),
            },
            LoginPhase::AwaitingDevice { verification_uri, user_code, expires_in } => {
                PhaseView::AwaitingApproval {
                    user_code,
                    verification_uri,
                    expires_at: expires_at_ms(expires_in),
                }
            }
        };
        (self.emit)(LoginPhaseEvent { origin: self.origin.to_string(), phase });
    }

    /// 開不了也繼續等：等待面有可複製的網址。
    fn open_browser(&self, url: &str) -> bool {
        (self.open_browser)(url).is_ok()
    }

    fn sleep_secs(&self, secs: u64) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        while !self.cancelled() && std::time::Instant::now() < deadline {
            std::thread::sleep(CANCEL_POLL);
        }
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

/// 被拒或逾時的那一輪用的是哪一種登入：兩種的說法不同，前端照它選文案。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LoginMethod {
    Browser,
    Device,
}

/// 一次登入的可讀結果。連線錯誤走 Err、絕不進 PAT fallback。
pub enum ConnectionLogin {
    /// 已登入。`access_token` 是登入當下快取的短效 token，交給 TokenManager
    /// 在記憶體持有；不落盤、不過境 TS。
    LoggedIn {
        display: String,
        access_token: Option<String>,
    },
    /// server 沒有 device flow（沒有宣告時）或只收存取金鑰：就地改收 PAT。
    PatRequired,
    Denied(LoginMethod),
    Expired(LoginMethod),
    Cancelled,
}

impl ConnectionLogin {
    /// 回給前端的內容：狀態，加上顯示名（已登入）或登入方式（被拒、逾時）。
    /// access token 不在裡面。
    pub fn view(&self) -> serde_json::Value {
        match self {
            ConnectionLogin::LoggedIn { display, .. } => {
                serde_json::json!({ "status": "loggedIn", "display": display })
            }
            ConnectionLogin::PatRequired => serde_json::json!({ "status": "patRequired" }),
            ConnectionLogin::Denied(method) => serde_json::json!({ "status": "denied", "method": method }),
            ConnectionLogin::Expired(method) => serde_json::json!({ "status": "expired", "method": method }),
            ConnectionLogin::Cancelled => serde_json::json!({ "status": "cancelled" }),
        }
    }
}

/// 印出時不帶 access token，`{:?}` 進了 log 也不會洩漏。
impl std::fmt::Debug for ConnectionLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionLogin::LoggedIn { display, access_token } => f
                .debug_struct("LoggedIn")
                .field("display", display)
                .field("access_token", &access_token.as_ref().map(|_| "<redacted>"))
                .finish(),
            ConnectionLogin::PatRequired => f.write_str("PatRequired"),
            ConnectionLogin::Denied(method) => write!(f, "Denied({method:?})"),
            ConnectionLogin::Expired(method) => write!(f, "Expired({method:?})"),
            ConnectionLogin::Cancelled => f.write_str("Cancelled"),
        }
    }
}

/// 登入（D7）：先以 Keychain 既有 refresh credential 靜默換新（規格「rotation 後
/// 舊 credential 失效仍可用」——重啟後無需重新登入）；沒有或已失效才探索
/// server 宣告的方式、依固定順序選擇並跑到終態。Desktop 不指定方式。
pub fn connection_login(
    origin: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
    io: &DesktopLoginIo,
) -> Result<ConnectionLogin, String> {
    if credentials.get(origin, CredentialKind::Refresh)?.is_some() {
        match refresh_connection(origin, credentials) {
            Ok(access) => {
                let display = record_identity_from_server(origin, &access, registry_path)?;
                return Ok(ConnectionLogin::LoggedIn { display, access_token: Some(access) });
            }
            // 明確被拒＝credential 已死：清掉殘骸、走完整登入。
            Err(RefreshFailure::Rejected(_)) => {
                speclink_remote::auth::clear_keychain_login(origin, credentials);
            }
            // 生死未卜（5xx／不可達／Keychain 故障）：credential 原封不動，
            // 回報連線錯誤——暫時性失敗不是語意訊號。
            Err(RefreshFailure::Unavailable(message)) => return Err(message),
        }
    }

    let discovery = remote_login::discover(origin).map_err(|e| e.message)?;
    let plan = remote_login::choose(&discovery, None).map_err(|e| e.to_string())?;
    let method = if matches!(plan, Plan::Browser(_)) { LoginMethod::Browser } else { LoginMethod::Device };
    let lock_dir = speclink_remote::auth::speclink_config_dir();
    let target = LoginTarget { origin, credentials, lock_dir: &lock_dir };
    Ok(match remote_login::login(&target, &plan, io) {
        LoginOutcome::LoggedIn { display, .. } => {
            record_identity(registry_path, origin, Some(&display))?;
            ConnectionLogin::LoggedIn {
                display,
                access_token: speclink_remote::refresh::cached_bearer(origin, credentials),
            }
        }
        // credential 已存好、只是查不到身分：照樣是已登入，身分欄留空。
        LoginOutcome::LoggedInWithoutIdentity { .. } => ConnectionLogin::LoggedIn {
            display: String::new(),
            access_token: speclink_remote::refresh::cached_bearer(origin, credentials),
        },
        LoginOutcome::Denied => ConnectionLogin::Denied(method),
        LoginOutcome::Expired => ConnectionLogin::Expired(method),
        LoginOutcome::Cancelled => ConnectionLogin::Cancelled,
        LoginOutcome::Unsupported | LoginOutcome::TokenRequired => ConnectionLogin::PatRequired,
        LoginOutcome::KeychainUnavailable => {
            return Err("無法安全儲存登入憑證：系統 Keychain 無法使用".to_string())
        }
        LoginOutcome::Failed(message) => return Err(message),
    })
}

/// 這個 origin 在 Keychain 有沒有登入：瀏覽器或裝置登入的 refresh 條目，或
/// 存取金鑰的條目。CLI 寫的是同一組條目，所以在 CLI 以存取金鑰登入後，這裡
/// 也算已登入，反過來也一樣。
pub fn logged_in(origin: &str, credentials: &dyn CredentialStore) -> bool {
    [CredentialKind::Refresh, CredentialKind::Pat]
        .into_iter()
        .any(|kind| matches!(credentials.get(origin, kind), Ok(Some(_))))
}

/// 進行中的登入的取消旗標（keyed by origin）。同一 origin 再按登入＝取消前一輪。
#[derive(Default)]
pub struct LoginCancels(Mutex<HashMap<String, Arc<AtomicBool>>>);

impl LoginCancels {
    /// 開始一輪登入，回傳它的取消旗標；同 origin 前一輪仍在跑就先取消它。
    pub fn begin(&self, origin: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        let previous = self
            .0
            .lock()
            .expect("login cancel lock")
            .insert(origin.to_string(), flag.clone());
        if let Some(previous) = previous {
            previous.store(true, Ordering::SeqCst);
        }
        flag
    }

    /// 取消該 origin 正在跑的那一輪（沒有就什麼都不做）。
    pub fn cancel(&self, origin: &str) {
        if let Some(flag) = self.0.lock().expect("login cancel lock").get(origin) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    /// 一輪結束：只移除自己的旗標，不動之後開始的那一輪。
    pub fn end(&self, origin: &str, flag: &Arc<AtomicBool>) {
        let mut map = self.0.lock().expect("login cancel lock");
        if map.get(origin).is_some_and(|current| Arc::ptr_eq(current, flag)) {
            map.remove(origin);
        }
    }
}

/// PAT 登入（規格「PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain」）：
/// 委派共用的 pat_login，Desktop 不退到憑證檔（credential 只在 Keychain）。
/// PAT 僅單次過境此參數，不回讀、不入 log。
pub fn pat_login(
    origin: &str,
    pat: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
) -> Result<String, String> {
    let lock_dir = speclink_remote::auth::speclink_config_dir();
    let target = LoginTarget { origin, credentials, lock_dir: &lock_dir };
    match remote_login::pat_login(&target, pat, None) {
        PatLoginOutcome::LoggedIn { display, .. } => {
            record_identity(registry_path, origin, Some(&display))?;
            Ok(display)
        }
        PatLoginOutcome::Denied => Err("PAT 無效或已被撤銷".to_string()),
        PatLoginOutcome::KeychainUnavailable => Err("無法存入系統 Keychain".to_string()),
        PatLoginOutcome::Failed(message) => Err(message),
    }
}

/// 登出結果：撤銷是盡力語意（server 不可達不阻擋本機刪除）；PAT 無自助撤銷
/// 端點，回報提示請使用者至 server 帳號頁撤銷。
#[derive(Debug)]
pub struct LogoutOutcome {
    pub revoked_on_server: bool,
    pub pat_notice: bool,
}

/// 登出：委派共用的 logout（清掉的 Keychain 範圍與 CLI 相同：refresh、access
/// token 快取、PAT），再清 registry 身分。
pub fn logout(
    origin: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
) -> Result<LogoutOutcome, String> {
    let outcome = remote_login::logout(origin, credentials, None)?;
    record_identity(registry_path, origin, None)?;
    Ok(LogoutOutcome {
        revoked_on_server: outcome.revoked,
        pat_notice: outcome.removed_pat,
    })
}

/// rotation 失敗的兩種語意（實作下沉至 speclink-remote，與 CLI 共用）——
/// 呼叫端據此決定「清掉 credential 要求重登入」還是「原封保留、稍後再試」。
/// 決策 3 的原則不只適用於登入前探測：任何暫時性失敗都不得被讀成
/// 「credential 已失效」。
pub use speclink_remote::refresh::RefreshFailure;

/// 換發全程持跨行程檔案鎖（cli-desktop-credential-sharing）：CLI 與 desktop
/// 共用同一個 credential family，兩邊同時換發會撞上 server 的 reuse 偵測而
/// 整族撤銷。新 refresh credential 與 access token 快取由下沉層一併回寫。
pub fn refresh_connection(
    origin: &str,
    credentials: &dyn CredentialStore,
) -> Result<String, RefreshFailure> {
    speclink_remote::refresh::rotate(
        origin,
        credentials,
        &speclink_remote::auth::speclink_config_dir(),
    )
}

/// 查身分（API 根的 whoami，沒有時退回 /auth/whoami）並把顯示名寫回 registry。
fn record_identity_from_server(
    origin: &str,
    bearer: &str,
    registry_path: &Path,
) -> Result<String, String> {
    let who = device::identity(origin, bearer).map_err(|e| e.to_string())?;
    record_identity(registry_path, origin, Some(&who.user.name))?;
    Ok(who.user.name)
}

/// 更新 registry 中該 origin 條目的身分顯示名（None＝清除）。條目不存在時
/// 靜默略過——登入流程由 connection_add 先建條目。
fn record_identity(
    registry_path: &Path,
    origin: &str,
    display: Option<&str>,
) -> Result<(), String> {
    let mut entries = read_registry(registry_path);
    if let Some(entry) = entries.iter_mut().find(|e| e.origin == origin) {
        entry.last_actor_display = display.map(str::to_string);
        write_registry(registry_path, &entries)?;
    }
    Ok(())
}

#[cfg(test)]
mod checkout_tests {
    //! spec「checkout 綁定驗證與 marker 寫入」、design「Desktop checkout 採先檢查、
    //! 後同步的兩階段 IPC」與 Implementation Contract 的「Desktop IPC and UI contract」。
    use super::*;

    const ORIGIN: &str = "https://spec.example.test";
    const PROJECT: &str = "acme";
    const REPO: &str = "desktop";

    fn git_checkout() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp checkout");
        std::fs::create_dir(dir.path().join(".git")).expect("git marker");
        dir
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel.split('/').collect::<PathBuf>());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent dir");
        }
        std::fs::write(path, content).expect("write file");
    }

    fn write_marker(root: &Path, url: &str, repo: &str) {
        write(root, ".speclink.yaml", &format!("remote:\n  url: {url}\n  repo: {repo}\n"));
    }

    /// 帶 built-in 選集、custom descriptor 與未知頂層鍵的相符 marker。
    fn write_full_marker(root: &Path, builtins: &[&str]) {
        let listed: String = builtins.iter().map(|t| format!("  - {t}\n")).collect();
        write(
            root,
            ".speclink.yaml",
            &format!(
                "tools:\n{listed}  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\nremote:\n  url: {ORIGIN}/api/speclink/v1/projects/{PROJECT}\n  repo: {REPO}\nfuture_top_level: keep me\n"
            ),
        );
    }

    fn exists(root: &Path, rel: &str) -> bool {
        root.join(rel.split('/').collect::<PathBuf>()).exists()
    }

    fn read(root: &Path, rel: &str) -> String {
        std::fs::read_to_string(root.join(rel.split('/').collect::<PathBuf>())).expect("read file")
    }

    /// 目錄快照，供「零寫入」逐位元組比對。
    fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
                if entry.path().is_dir() {
                    out.push((format!("{rel}/"), Vec::new()));
                    walk(&entry.path(), &rel, out);
                } else {
                    out.push((rel, std::fs::read(entry.path()).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(root, "", &mut out);
        out
    }

    fn inspect(root: &Path) -> Result<CheckoutInspection, String> {
        inspect_checkout(root, ORIGIN, PROJECT, REPO)
    }

    fn bind(root: &Path, tools: &[&str]) -> Result<String, String> {
        let owned: Vec<String> = tools.iter().map(|t| t.to_string()).collect();
        bind_checkout(root, ORIGIN, PROJECT, REPO, &owned)
    }

    // --- inspect_checkout：零寫入的先檢查階段 ---

    #[test]
    fn inspect_matching_marker_reports_root_and_recorded_tools_without_writing() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["codex"]);
        let before = snapshot(dir.path());

        let seen = inspect(dir.path()).expect("matching origin and repo");

        assert_eq!(seen.root, dir.path().display().to_string());
        assert_eq!(seen.tools, vec!["codex".to_string()], "只回報既有 built-in 選集");
        assert_eq!(snapshot(dir.path()), before, "檢查階段必須零寫入");
    }

    #[test]
    fn inspect_mismatched_marker_origin_reports_where_the_marker_points() {
        let dir = git_checkout();
        write_marker(dir.path(), "https://other.example.test/team", REPO);
        let before = snapshot(dir.path());

        let err = inspect(dir.path()).expect_err("origin mismatch");

        assert!(err.contains("https://other.example.test"), "{err}");
        assert!(err.contains(REPO), "{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn inspect_mismatched_marker_repo_reports_where_the_marker_points() {
        let dir = git_checkout();
        write_marker(dir.path(), ORIGIN, "api");
        let before = snapshot(dir.path());

        let err = inspect(dir.path()).expect_err("repo mismatch");

        assert!(err.contains(ORIGIN), "{err}");
        assert!(err.contains("api"), "{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn inspect_non_git_directory_without_a_marker_is_rejected() {
        let dir = tempfile::tempdir().expect("plain directory");
        let before = snapshot(dir.path());

        let err = inspect(dir.path()).expect_err("not a git checkout");

        assert!(err.contains("Git"), "{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn inspect_unparseable_config_is_rejected_fail_closed() {
        let dir = git_checkout();
        write(dir.path(), ".speclink.yaml", "tools: [unclosed\n");
        let before = snapshot(dir.path());

        let err = inspect(dir.path()).expect_err("bad yaml");

        assert!(err.contains(".speclink.yaml"), "錯誤須指名檔案：{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn inspect_without_a_tools_list_preselects_only_actual_footprints() {
        // marker 缺 tools 清單時只依實際 footprint 預選——不再補 Claude fallback。
        let bare = git_checkout();
        write_marker(bare.path(), ORIGIN, REPO);
        assert!(
            inspect(bare.path()).expect("bare checkout").tools.is_empty(),
            "沒有任何 footprint 時不得預選 Claude"
        );

        let codex = git_checkout();
        write_marker(codex.path(), ORIGIN, REPO);
        write(codex.path(), "AGENTS.md", "使用者文字\n");
        assert_eq!(
            inspect(codex.path()).expect("codex footprint").tools,
            vec!["codex".to_string()]
        );
    }

    #[test]
    fn inspect_without_a_tools_list_never_preselects_copilot() {
        // spec Scenario「勾選 Copilot 綁定 checkout」：.github/ 的存在不預選 Copilot。
        let dir = git_checkout();
        write_marker(dir.path(), ORIGIN, REPO);
        write(dir.path(), ".github/workflows/ci.yml", "on: push\n");
        write(
            dir.path(),
            ".github/skills/my-skill/SKILL.md",
            "---\nname: my-skill\n---\n",
        );

        assert!(inspect(dir.path())
            .expect("github checkout")
            .tools
            .is_empty());
    }

    #[test]
    fn inspect_reports_a_copilot_only_marker() {
        // spec Scenario「只選 Copilot 的 checkout 直達 remote tab」的判定面：
        // [copilot] 是有效的 built-in 選集。
        let dir = git_checkout();
        write_full_marker(dir.path(), &["copilot"]);

        assert_eq!(
            inspect(dir.path()).expect("copilot marker").tools,
            vec!["copilot"]
        );
    }

    #[test]
    fn bind_claude_and_copilot_generates_both_skill_sets() {
        let dir = git_checkout();
        write(dir.path(), ".github/workflows/ci.yml", "on: push\n");

        bind(dir.path(), &["claude", "copilot"]).expect("new binding");

        assert!(exists(
            dir.path(),
            ".claude/skills/speclink-propose/SKILL.md"
        ));
        assert!(exists(
            dir.path(),
            ".github/skills/speclink-propose/SKILL.md"
        ));
        for absent in [".github/copilot-instructions.md", "AGENTS.md", "CLAUDE.md"] {
            assert!(!exists(dir.path(), absent), "{absent} 不得生成");
        }
        assert_eq!(read(dir.path(), ".github/workflows/ci.yml"), "on: push\n");
        assert_eq!(
            inspect(dir.path()).expect("re-inspect").tools,
            vec!["claude", "copilot"]
        );
    }

    #[test]
    fn inspect_reports_a_pi_only_marker() {
        // spec Scenario「只選 Pi 的 checkout 直達 remote tab」的判定面：
        // [pi] 是有效的 built-in 選集。
        let dir = git_checkout();
        write_full_marker(dir.path(), &["pi"]);

        assert_eq!(inspect(dir.path()).expect("pi marker").tools, vec!["pi"]);
    }

    #[test]
    fn inspect_without_a_tools_list_never_preselects_pi_or_cursor() {
        // spec「checkout 綁定驗證與 marker 寫入」：`.pi/`、`.cursor/` 的存在不預選 Pi、Cursor。
        let dir = git_checkout();
        write_marker(dir.path(), ORIGIN, REPO);
        write(dir.path(), ".cursor/rules/style.mdc", "---\nalwaysApply: true\n---\n");
        write(dir.path(), ".pi/settings.json", "{}\n");
        // 兩個工具的 skills 目錄本身存在（放著使用者自己的技能）也不預選。
        for skills in [".cursor/skills", ".pi/skills"] {
            write(dir.path(), &format!("{skills}/my-skill/SKILL.md"), "---\nname: my-skill\n---\n");
        }

        assert!(inspect(dir.path()).expect("cursor checkout").tools.is_empty());
    }

    #[test]
    fn bind_claude_and_cursor_generates_both_skill_sets() {
        // spec Scenario「勾選 Cursor 綁定 checkout」。
        let dir = git_checkout();
        write(dir.path(), ".cursor/rules/style.mdc", "---\nalwaysApply: true\n---\n");

        bind(dir.path(), &["claude", "cursor"]).expect("new binding");

        for skill in [
            ".claude/skills/speclink-propose/SKILL.md",
            ".cursor/skills/speclink-propose/SKILL.md",
        ] {
            assert!(exists(dir.path(), skill), "{skill} 須生成");
        }
        for absent in [".cursor/agents", "AGENTS.md", "CLAUDE.md"] {
            assert!(!exists(dir.path(), absent), "{absent} 不得生成");
        }
        assert_eq!(
            read(dir.path(), ".cursor/rules/style.mdc"),
            "---\nalwaysApply: true\n---\n"
        );
        assert_eq!(
            inspect(dir.path()).expect("re-inspect").tools,
            vec!["claude", "cursor"]
        );
    }

    #[test]
    fn inspect_with_a_descriptor_only_tools_list_falls_back_to_footprints() {
        // 清單存在但沒有任何可解析的內建名：與缺清單同樣只依實際 footprint 預選。
        let dir = git_checkout();
        write_full_marker(dir.path(), &[]);
        write(dir.path(), "AGENTS.md", "使用者文字\n");

        assert_eq!(
            inspect(dir.path()).expect("descriptor-only list").tools,
            vec!["codex".to_string()]
        );
    }

    #[test]
    fn inspect_reports_only_builtins_from_a_mixed_tools_list() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["codex"]);

        let seen = inspect(dir.path()).expect("mixed tools list");

        assert_eq!(seen.tools, vec!["codex".to_string()], "自訂描述子不進入 picker");
    }

    // --- bind_checkout：提交階段 ---

    #[test]
    fn bind_rejects_an_empty_tool_selection_without_writing() {
        let dir = git_checkout();
        let before = snapshot(dir.path());

        let err = bind(dir.path(), &[]).expect_err("空選集必須被拒");

        // `pi` 是 `copilot` 的子字串：比對完整清單，逐名比對測不到漏列。
        assert!(err.contains("claude、codex、copilot、pi、cursor"), "must list the five tools: {err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn bind_rejects_an_unknown_tool_name_without_writing() {
        let dir = git_checkout();
        let before = snapshot(dir.path());

        let err = bind(dir.path(), &["claude", "vscode"]).expect_err("未知工具必須被拒");

        assert!(err.contains("vscode"), "須指出違規名稱：{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    #[test]
    fn bind_markerless_git_checkout_writes_cli_compatible_marker_and_syncs_tools() {
        let dir = git_checkout();

        let bound = bind(dir.path(), &["claude", "codex"]).expect("new binding");
        assert_eq!(bound, dir.path().display().to_string());

        let workspace = speclink_core::workspace::Workspace::discover(dir.path())
            .expect("CLI discovery succeeds")
            .expect("marker creates workspace");
        let resolution = workspace
            .resolve_mode_with(None)
            .expect("CLI mode resolution succeeds");
        match resolution.mode {
            speclink_core::workspace::StoreMode::Remote(remote) => {
                assert_eq!(
                    remote.url,
                    format!("{ORIGIN}/api/speclink/v1/projects/{PROJECT}")
                );
                assert_eq!(remote.repo.as_deref(), Some(REPO));
            }
            speclink_core::workspace::StoreMode::Fs => panic!("written marker must select remote"),
        }
        for (md, skill) in [
            ("CLAUDE.md", ".claude/skills/speclink-propose/SKILL.md"),
            ("AGENTS.md", ".agents/skills/speclink-propose/SKILL.md"),
        ] {
            // 指令檔已退出受管（change: remove-marker-injection）：只生成技能檔。
            assert!(!exists(dir.path(), md), "{md} 不得生成");
            assert!(exists(dir.path(), skill), "{skill} 應生成");
        }
        assert!(!exists(dir.path(), "openspec"), "Remote checkout 不建本機規格樹");
        assert_eq!(inspect(dir.path()).expect("re-inspect").tools, vec!["claude", "codex"]);
    }

    #[test]
    fn bind_existing_matching_marker_backfills_missing_artifacts() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["codex"]);
        write(dir.path(), "AGENTS.md", "只剩使用者文字\n");

        bind(dir.path(), &["codex"]).expect("既有相符 marker 仍須同步");

        let agents = read(dir.path(), "AGENTS.md");
        assert_eq!(agents, "只剩使用者文字\n", "指令檔須位元級不變:\n{agents}");
        assert!(exists(dir.path(), ".agents/skills/speclink-propose/SKILL.md"));
        let config = read(dir.path(), ".speclink.yaml");
        assert!(config.contains(&format!("projects/{PROJECT}")), "remote 值不變:\n{config}");
        assert!(config.contains("keep me"), "未知頂層鍵須保留:\n{config}");
        assert!(!exists(dir.path(), "openspec"));
    }

    #[test]
    fn bind_switches_claude_to_codex_preserving_user_text_and_descriptors() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["claude"]);
        write(dir.path(), "CLAUDE.md", "使用者寫的段落\n");
        bind(dir.path(), &["claude"]).expect("先收斂到 claude");
        assert!(exists(dir.path(), ".claude/skills/speclink-propose/SKILL.md"), "precondition");

        bind(dir.path(), &["codex"]).expect("切換為 codex");

        assert_eq!(inspect(dir.path()).expect("re-inspect").tools, vec!["codex"]);
        let claude_md = read(dir.path(), "CLAUDE.md");
        assert!(!claude_md.contains("<!-- SPECLINK:START"), "Claude 區塊須移除:\n{claude_md}");
        assert!(claude_md.contains("使用者寫的段落"), "使用者文字須保留:\n{claude_md}");
        assert!(!exists(dir.path(), ".claude/skills/speclink-propose/SKILL.md"));
        assert!(exists(dir.path(), ".agents/skills/speclink-propose/SKILL.md"));
        let config = read(dir.path(), ".speclink.yaml");
        assert!(config.contains("wad-harness"), "custom descriptor 須保留:\n{config}");
        assert!(!exists(dir.path(), "openspec"));
    }

    #[test]
    fn bind_failure_returns_no_root_and_leaves_the_folder_untouched() {
        let dir = tempfile::tempdir().expect("plain directory");
        let before = snapshot(dir.path());

        let err = bind(dir.path(), &["claude"]).expect_err("not a git checkout");

        assert!(err.contains("Git"), "{err}");
        assert_eq!(snapshot(dir.path()), before, "失敗不得留下任何寫入");
    }

    // --- 3.3 AUDIT：sharp-edges 負向鎖定（無 silent success、無危險預設） ---

    /// Lazy Developer：純空白的工具值不是「有效選集」——收斂為空並被拒，零寫入。
    #[test]
    fn bind_whitespace_only_tools_collapse_to_empty_and_are_rejected() {
        let dir = git_checkout();
        let before = snapshot(dir.path());

        let err = bind(dir.path(), &[" ", ""]).expect_err("純空白視為空選集");

        assert!(err.contains("claude") && err.contains("codex"), "{err}");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟不變");
    }

    /// Confused／Scoundrel：重複工具名去重，不寫出重複的 tools 條目。
    #[test]
    fn bind_duplicate_tool_names_do_not_duplicate_markers_or_entries() {
        let dir = git_checkout();

        bind(dir.path(), &["codex", "codex", "codex"]).expect("重複名須被吸收");

        assert_eq!(inspect(dir.path()).expect("re-inspect").tools, vec!["codex"]);
        assert!(!exists(dir.path(), "AGENTS.md"), "指令檔不得生成");
        let config = read(dir.path(), ".speclink.yaml");
        assert_eq!(config.matches("- codex").count(), 1, "tools 條目不得重複:\n{config}");
    }

    /// Scoundrel：marker 不一致時 bind 必須在任何 reconciliation／marker 寫入之前 fail loud，
    /// 既有的 Speclink 受管產物與自訂描述子逐位元組不動。
    #[test]
    fn bind_mismatched_marker_fails_loud_without_reconciling() {
        let dir = git_checkout();
        write_marker(dir.path(), "https://other.example.test", REPO);
        write(dir.path(), "CLAUDE.md", "使用者段落\n");
        let before = snapshot(dir.path());

        let err = bind(dir.path(), &["claude", "codex"]).expect_err("origin 不符必須被拒");

        assert!(err.contains("https://other.example.test"), "{err}");
        assert!(!exists(dir.path(), ".claude/skills/speclink-propose/SKILL.md"), "不得生成");
        assert!(!exists(dir.path(), ".agents"), "不得生成");
        assert_eq!(snapshot(dir.path()), before, "拒絕時磁碟逐位元組不變");
    }

    /// 可交換字串語意：marker 相符時 `project` 參數不參與驗證（只在寫新 marker 時使用），
    /// 因此傳入任意 project 仍成功——鎖住「別把 project 誤當一致性檢查的一部分」。
    #[test]
    fn bind_ignores_project_when_an_existing_marker_matches() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["codex"]);

        let owned = vec!["codex".to_string()];
        let bound = bind_checkout(dir.path(), ORIGIN, "a-totally-different-project", REPO, &owned)
            .expect("marker 相符時 project 不影響驗證");

        assert_eq!(bound, dir.path().display().to_string());
        // remote 值仍是 marker 原本的 project，未被傳入的 project 覆寫。
        let config = read(dir.path(), ".speclink.yaml");
        assert!(config.contains(&format!("projects/{PROJECT}")), "remote 值不變:\n{config}");
        assert!(!config.contains("a-totally-different-project"), "不得改寫 remote:\n{config}");
    }

    /// 最簡呼叫也不得跳過 custom descriptor 保留——單一工具 bind 後描述子仍在。
    #[test]
    fn bind_preserves_custom_descriptor_on_the_simplest_call() {
        let dir = git_checkout();
        write_full_marker(dir.path(), &["codex"]);

        bind(dir.path(), &["codex"]).expect("最簡 bind");

        let config = read(dir.path(), ".speclink.yaml");
        assert!(config.contains("wad-harness"), "custom descriptor 須保留:\n{config}");
        assert!(config.contains(".wad/skills"), "descriptor 欄位須保留:\n{config}");
    }

    // --- 6.1 跨入口一致性（spec「Remote Workspace bootstrap 跨入口一致性」） ---

    /// Desktop bind 與 CLI Remote init（`speclink_core::init::init_remote`——CLI
    /// `cmd_init_remote` 呼叫的正是它）對等價的新 Git checkout 選取相同 Codex，
    /// SHALL 產生同構的 built-in tools、Remote marker 與 Skills，兩者皆不建 `openspec/`
    /// 也不建任何指令檔。此測試把「一致性」釘在共用 Core 上。
    #[test]
    fn desktop_bind_and_cli_remote_init_produce_isomorphic_artifacts() {
        use speclink_core::skills::Tool;

        // 兩入口收斂到相同的最終 project URL，remote section 才可能同構。
        let project_url = format!("{ORIGIN}/api/speclink/v1/projects/{PROJECT}");

        let desktop = git_checkout();
        bind(desktop.path(), &["codex"]).expect("Desktop bind");

        let cli = git_checkout();
        speclink_core::init::init_remote(cli.path(), &[Tool::Codex], false, &project_url, Some(REPO))
            .expect("CLI Remote init");

        // 兩入口都不生成指令檔。
        assert!(!exists(desktop.path(), "AGENTS.md"), "Desktop bind 不得生成指令檔");
        assert!(!exists(cli.path(), "AGENTS.md"), "CLI Remote init 不得生成指令檔");
        // Skills 正典逐位元組同構。
        for skill in ["speclink-apply", "speclink-archive", "speclink-propose"] {
            let rel = format!(".agents/skills/{skill}/SKILL.md");
            assert_eq!(read(desktop.path(), &rel), read(cli.path(), &rel), "{rel} 須同構");
        }
        // built-in tools 選集同構。
        assert_eq!(inspect(desktop.path()).unwrap().tools, vec!["codex"]);
        assert_eq!(
            preselected_tools(cli.path()),
            vec!["codex".to_string()],
            "CLI init 的 tools 同構"
        );
        // 兩者都不建本機規格樹。
        assert!(!exists(desktop.path(), "openspec"));
        assert!(!exists(cli.path(), "openspec"));
    }

    /// 重試冪等：相同選集再次 bind SHALL 收斂到逐位元組相同的受管產物，不重複 marker。
    #[test]
    fn bind_retry_with_the_same_selection_is_byte_identical() {
        let dir = git_checkout();
        bind(dir.path(), &["codex"]).expect("first bind");
        let after_first = snapshot(dir.path());

        bind(dir.path(), &["codex"]).expect("retry bind");

        assert_eq!(snapshot(dir.path()), after_first, "相同選集重試須冪等");
    }
}

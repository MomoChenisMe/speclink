//! connection registry（design 決策 4）與登入／登出編排（決策 3、5、6）。
//!
//! registry：saved servers 的無 secret profile 清單，存 app 設定目錄
//! （appConfigDir）下的 connections.json。條目＝{id、origin（baseUrl 正規化）、
//! name、lastActorDisplay?}——絕不含任何 token 欄位（規格「connection registry
//! 不含 secret 且跨重啟保留」）。一 origin 一條目，重複新增即更新顯示名。
//! 壞 JSON 歸零清單（與分頁持久化同一寬容哲學）。
//!
//! 編排：device_login／pat_login／logout／refresh_connection 收注入的
//! CredentialStore 與瀏覽器開啟器、無 tauri 型別——tauri 接線在 lib.rs。
//! secret 只在此層與 CredentialStore 之間流動，錯誤訊息不夾帶 secret。

use crate::credentials::{CredentialKind, CredentialStore};
use serde::{Deserialize, Serialize};
use speclink_protocol::device::DeviceTokenStatus;
use speclink_remote::device::{self, InitiateOutcome};
use std::path::{Path, PathBuf};

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

/// 寫 registry：建父目錄後整檔覆寫。
pub fn write_registry(path: &Path, entries: &[ConnectionEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("無法建立設定目錄：{e}"))?;
    }
    let text = serde_json::to_string_pretty(entries).map_err(|e| format!("無法序列化連線清單：{e}"))?;
    std::fs::write(path, text).map_err(|e| format!("無法寫入連線清單：{e}"))
}

/// 把 baseUrl 正規化為 origin：scheme://authority，scheme 與 host 小寫、
/// 去除路徑/查詢/片段。僅接受 http/https。
pub fn normalize_origin(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim();
    let (scheme, rest) = trimmed
        .split_once("://")
        .ok_or_else(|| "伺服器位址需以 http:// 或 https:// 開頭".to_string())?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(format!("不支援的協定「{scheme}」：僅接受 http 或 https"));
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() {
        return Err("伺服器位址缺少主機名".to_string());
    }
    Ok(format!("{scheme}://{}", authority.to_ascii_lowercase()))
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
    let id = format!("conn_{}", ulid::Ulid::new().to_string().to_ascii_lowercase());
    entries.push(ConnectionEntry {
        id: id.clone(),
        origin,
        name: name.to_string(),
        last_actor_display: None,
    });
    Ok(id)
}

// --- 登入／登出編排（決策 3、5、6） ---

/// device_login 的可讀結果：Unsupported 是 PAT fallback 訊號（決策 3），
/// denied／expired 逐一回報；連線錯誤（5xx／不可達）走 Err、絕不進 fallback。
/// access token 短效、只在 Rust 記憶體——由 lib.rs 的命令層持有，不落盤。
#[derive(Debug)]
pub enum DeviceLoginOutcome {
    LoggedIn { display: String, access_token: String },
    Unsupported,
    Denied,
    Expired,
}

/// device login 全鏈（決策 5）：先以 Keychain 既有 refresh credential 靜默
/// 換新（規格「rotation 後舊 credential 失效仍可用」——重啟後無需重新核准）；
/// 沒有或已失效才走 device flow：initiate（兼探測，決策 3）→ 開瀏覽器至
/// verification 頁（URL 帶 user_code 預填參數）→ 依 server interval 輪詢至
/// 終態 → granted 即存 refresh credential、打 /auth/whoami 取身分寫回 registry。
pub fn device_login(
    origin: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
    open_browser: &dyn Fn(&str) -> Result<(), String>,
) -> Result<DeviceLoginOutcome, String> {
    // 靜默快路徑：有 refresh credential 就先試 rotation，成功即免瀏覽器。
    if credentials.get(origin, CredentialKind::Refresh)?.is_some() {
        match refresh_connection(origin, credentials) {
            Ok(access) => {
                let display = record_identity_from_server(origin, &access, registry_path)?;
                return Ok(DeviceLoginOutcome::LoggedIn { display, access_token: access });
            }
            // 明確被拒＝credential 已死：清掉殘骸、走完整 device flow 重新核准。
            Err(RefreshFailure::Rejected(_)) => {
                credentials.delete(origin, CredentialKind::Refresh)?;
            }
            // 生死未卜（5xx／不可達／Keychain 故障）：credential 原封不動，
            // 回報連線錯誤（決策 3 同一原則——暫時性失敗不是語意訊號）。
            Err(RefreshFailure::Unavailable(message)) => return Err(message),
        }
    }

    let auth = match device::initiate(origin).map_err(|e| e.to_string())? {
        InitiateOutcome::Supported(auth) => auth,
        InitiateOutcome::Unsupported => return Ok(DeviceLoginOutcome::Unsupported),
    };

    let sep = if auth.verification_uri.contains('?') { '&' } else { '?' };
    open_browser(&format!("{}{sep}user_code={}", auth.verification_uri, auth.user_code))?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(auth.expires_in);
    let interval = std::time::Duration::from_secs(auth.interval.max(1));
    loop {
        let poll = device::poll(origin, &auth.device_code).map_err(|e| e.to_string())?;
        match poll.status {
            DeviceTokenStatus::Approved => {
                let access =
                    poll.access_token.ok_or("server 未隨核准回傳 access token")?;
                let refresh =
                    poll.refresh_token.ok_or("server 未隨核准回傳 refresh credential")?;
                credentials.set(origin, CredentialKind::Refresh, &refresh)?;
                let display = record_identity_from_server(origin, &access, registry_path)?;
                return Ok(DeviceLoginOutcome::LoggedIn { display, access_token: access });
            }
            DeviceTokenStatus::Denied => return Ok(DeviceLoginOutcome::Denied),
            DeviceTokenStatus::Expired => return Ok(DeviceLoginOutcome::Expired),
            DeviceTokenStatus::Pending | DeviceTokenStatus::SlowDown => {
                if std::time::Instant::now() >= deadline {
                    return Ok(DeviceLoginOutcome::Expired);
                }
                std::thread::sleep(interval);
            }
        }
    }
}

/// PAT 登入（規格「PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain」）：
/// 先打 /auth/whoami 驗證，通過才入 store 並寫回身分顯示名。PAT 僅單次過境
/// 此參數，不回讀、不入 log。
pub fn pat_login(
    origin: &str,
    pat: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
) -> Result<String, String> {
    let display = match device::whoami(origin, pat) {
        Ok(who) => who.user.name,
        Err(e) if e.reason.as_deref() == Some("permission_denied") => {
            return Err("PAT 無效或已被撤銷".to_string());
        }
        Err(e) => return Err(e.to_string()),
    };
    credentials.set(origin, CredentialKind::Pat, pat)?;
    record_identity(registry_path, origin, Some(&display))?;
    Ok(display)
}

/// 登出結果：撤銷是盡力語意（server 不可達不阻擋本機刪除）；PAT 無自助撤銷
/// 端點，回報提示請使用者至 server 帳號頁撤銷。
#[derive(Debug)]
pub struct LogoutOutcome {
    pub revoked_on_server: bool,
    pub pat_notice: bool,
}

/// 登出（決策 6）：refresh credential 走 /auth/revoke 盡力撤銷 device family、
/// PAT 僅刪本機 entry 並提示；兩者皆必刪 Keychain entry、清 registry 身分。
pub fn logout(
    origin: &str,
    credentials: &dyn CredentialStore,
    registry_path: &Path,
) -> Result<LogoutOutcome, String> {
    let mut revoked_on_server = false;
    let mut pat_notice = false;
    if let Some(refresh) = credentials.get(origin, CredentialKind::Refresh)? {
        revoked_on_server = device::revoke(origin, &refresh).is_ok();
        credentials.delete(origin, CredentialKind::Refresh)?;
    }
    if credentials.get(origin, CredentialKind::Pat)?.is_some() {
        credentials.delete(origin, CredentialKind::Pat)?;
        pat_notice = true;
    }
    record_identity(registry_path, origin, None)?;
    Ok(LogoutOutcome { revoked_on_server, pat_notice })
}

/// rotation 失敗的兩種語意——呼叫端據此決定「清掉 credential 要求重登入」
/// 還是「原封保留、稍後再試」。決策 3 的原則不只適用於登入前探測：任何
/// 暫時性失敗都不得被讀成「credential 已失效」。
#[derive(Debug)]
pub enum RefreshFailure {
    /// server 明確拒絕（permission_denied），或本機根本沒有 credential——
    /// 該 credential 已無用，重新登入是唯一解。
    Rejected(String),
    /// credential 生死未卜：網路不可達、5xx、本機 Keychain 故障——一律保留，
    /// 回報連線錯誤讓使用者稍後再試。
    Unavailable(String),
}

impl RefreshFailure {
    /// 給使用者看的單行訊息（兩種語意在 UI 上都只是一行錯誤）。
    pub fn message(self) -> String {
        match self {
            RefreshFailure::Rejected(m) | RefreshFailure::Unavailable(m) => m,
        }
    }
}

/// 以 Keychain 的 refresh credential 換新一輪 token pair（rotation）：成功即以
/// 新 refresh credential 覆寫 Keychain slot（決策 2——回寫失敗屬 corrupt 邊界，
/// 錯誤上拋令使用者重登入），回傳新 access token。
pub fn refresh_connection(
    origin: &str,
    credentials: &dyn CredentialStore,
) -> Result<String, RefreshFailure> {
    let Some(refresh) = credentials
        .get(origin, CredentialKind::Refresh)
        .map_err(RefreshFailure::Unavailable)?
    else {
        return Err(RefreshFailure::Rejected(
            "此連線沒有 refresh credential——請重新登入".to_string(),
        ));
    };
    let rotated = device::refresh(origin, &refresh).map_err(|e| match e.reason.as_deref() {
        Some("permission_denied") => RefreshFailure::Rejected(e.message),
        _ => RefreshFailure::Unavailable(e.message),
    })?;
    credentials
        .set(origin, CredentialKind::Refresh, &rotated.refresh_token)
        .map_err(RefreshFailure::Unavailable)?;
    Ok(rotated.access_token)
}

/// 打 /auth/whoami 取身分顯示名並寫回 registry。
fn record_identity_from_server(
    origin: &str,
    bearer: &str,
    registry_path: &Path,
) -> Result<String, String> {
    let who = device::whoami(origin, bearer).map_err(|e| e.to_string())?;
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

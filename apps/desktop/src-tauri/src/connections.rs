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
    let text =
        serde_json::to_string_pretty(entries).map_err(|e| format!("無法序列化連線清單：{e}"))?;
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
/// 中預選的既有 built-in 工具選集（僅 claude／codex，順序穩定）。IPC 序列化為
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

/// 決定 picker 的預選：`.speclink.yaml` 記錄了 built-in 選集就用它（僅 claude／codex，
/// 去重、順序穩定）；缺清單時只依實際 Claude／Codex footprint 預選，絕不補 Claude fallback。
fn preselected_tools(root: &Path) -> Vec<String> {
    use speclink_core::config::ToolEntry;
    use speclink_core::skills::Tool;
    let app = match speclink_core::config::AppConfig::load(&root.join(".speclink.yaml")) {
        Ok(app) => app,
        Err(_) => return Vec::new(),
    };
    let mut picked: Vec<Tool> = Vec::new();
    for entry in &app.tools {
        if let ToolEntry::Builtin(name) = entry {
            if let Some(t) = Tool::parse(name) {
                if !picked.contains(&t) {
                    picked.push(t);
                }
            }
        }
    }
    if picked.is_empty() {
        for tool in speclink_core::init::detect_footprint_tools(root) {
            if !picked.contains(&tool) {
                picked.push(tool);
            }
        }
    }
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
        return Err("請至少選擇一個內建工具：claude、codex 或兩者".to_string());
    }
    let root_str = validate_checkout(root, selected_origin, selected_repo)?;

    // 無 marker 時先寫入同構 remote section，讓後續 reconciliation 沿用 remote 措辭。
    if speclink_core::config::AppConfig::load(&root.join(".speclink.yaml"))
        .map_err(|e| e.to_string())?
        .remote
        .is_none()
    {
        let normalized = normalize_origin(selected_origin)?;
        let project_url = format!("{normalized}/api/speclink/v1/projects/{selected_project}");
        speclink_core::init::write_remote_section(root, &project_url, Some(selected_repo))
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

// --- 登入／登出編排（決策 3、5、6） ---

/// device_login 的可讀結果：Unsupported 是 PAT fallback 訊號（決策 3），
/// denied／expired 逐一回報；連線錯誤（5xx／不可達）走 Err、絕不進 fallback。
/// access token 短效、只在 Rust 記憶體——由 lib.rs 的命令層持有，不落盤。
#[derive(Debug)]
pub enum DeviceLoginOutcome {
    LoggedIn {
        display: String,
        access_token: String,
    },
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
                return Ok(DeviceLoginOutcome::LoggedIn {
                    display,
                    access_token: access,
                });
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

    let sep = if auth.verification_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    open_browser(&format!(
        "{}{sep}user_code={}",
        auth.verification_uri, auth.user_code
    ))?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(auth.expires_in);
    let interval = std::time::Duration::from_secs(auth.interval.max(1));
    loop {
        let poll = device::poll(origin, &auth.device_code).map_err(|e| e.to_string())?;
        match poll.status {
            DeviceTokenStatus::Approved => {
                let access = poll
                    .access_token
                    .ok_or("server 未隨核准回傳 access token")?;
                let refresh = poll
                    .refresh_token
                    .ok_or("server 未隨核准回傳 refresh credential")?;
                credentials.set(origin, CredentialKind::Refresh, &refresh)?;
                let display = record_identity_from_server(origin, &access, registry_path)?;
                return Ok(DeviceLoginOutcome::LoggedIn {
                    display,
                    access_token: access,
                });
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
    Ok(LogoutOutcome {
        revoked_on_server,
        pat_notice,
    })
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

        assert!(err.contains("claude") && err.contains("codex"), "{err}");
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
            assert!(read(dir.path(), md).contains("<!-- SPECLINK:START"), "{md} marker");
            assert!(exists(dir.path(), skill), "{skill} 應生成");
        }
        assert!(
            read(dir.path(), "AGENTS.md").contains("team system's spec store"),
            "Remote checkout 須用 remote 措辭"
        );
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
        assert!(agents.contains("<!-- SPECLINK:START"), "缺席的區塊須補齊:\n{agents}");
        assert!(agents.contains("只剩使用者文字"), "使用者文字須保留:\n{agents}");
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

    /// Confused／Scoundrel：重複工具名去重，不寫出重複的 tools 條目或重複 marker。
    #[test]
    fn bind_duplicate_tool_names_do_not_duplicate_markers_or_entries() {
        let dir = git_checkout();

        bind(dir.path(), &["codex", "codex", "codex"]).expect("重複名須被吸收");

        assert_eq!(inspect(dir.path()).expect("re-inspect").tools, vec!["codex"]);
        let agents = read(dir.path(), "AGENTS.md");
        assert_eq!(
            agents.matches("<!-- SPECLINK:START").count(),
            1,
            "marker 不得重複:\n{agents}"
        );
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
    /// SHALL 產生同構的 built-in tools、Remote marker、Skills 與 `AGENTS.md` Remote
    /// Speclink 區塊，且兩者皆不建 `openspec/`。此測試把「一致性」釘在共用 Core 上。
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

        // AGENTS.md（含 Remote Speclink 區塊）逐位元組同構。
        assert_eq!(
            read(desktop.path(), "AGENTS.md"),
            read(cli.path(), "AGENTS.md"),
            "AGENTS.md Remote 區塊須同構"
        );
        assert!(
            read(desktop.path(), "AGENTS.md").contains("team system's spec store"),
            "須為 Remote 措辭"
        );
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

//! remote runtime 的 token 生命週期（design 決策 4「token 生命週期與 401 語意」；
//! 規格「token 換發全程 Rust 側且 401 語意固定」）。
//!
//! per-connection TokenManager：bearer（access token 或 PAT）只在 Rust 記憶體；
//! 請求前無 bearer 即以 Keychain refresh credential 換發（rotation 新 refresh
//! credential 由 refresh_connection 立即回寫）、無 refresh credential 則以 PAT
//! 為 bearer；任何請求 401 → 換發一次 → 重試一次 → 仍 401 即進入 needs-reauth
//! 狀態——TS 只見布林與繁中訊息，後續操作直接回拒絕錯誤、不再打 server。
//! access token 的過期採反應式 401 路徑處理（規格「過期自動換發、使用者無感」
//! 的可觀察契約相同），不另外追蹤 expires_in。

use crate::connections::{refresh_connection, RefreshFailure};
use crate::credentials::{CredentialKind, CredentialStore};
use serde::Serialize;
use speclink_protocol::binding::BindingResponse;
use speclink_protocol::command::{
    ArchiveDiscussionResponse, ArchiveResponse, ClaimResponse, PromoteDiscussionResponse,
};
use speclink_protocol::events::TransportKind;
use speclink_protocol::query::{
    ArtifactContent, ChangeStatus, DiscussionInfo, ListChangesResponse, ListSpecsResponse,
    ShowDiscussionResponse,
};
use speclink_remote::client::Client;
use speclink_remote::RemoteError;
use std::sync::{Arc, Mutex};

/// needs-reauth 的繁中狀態訊息（完整重新認證 UX 屬後續刀，本刀只回報狀態）。
const REAUTH_MESSAGE: &str = "此連線的登入已失效——請重新登入";

/// 一條 connection 的 token 生命週期管理者。
pub struct TokenManager {
    origin: String,
    /// 記憶體持有的 bearer；絕不落盤、絕不過境 TS。
    bearer: Mutex<Option<String>>,
    /// needs-reauth 狀態訊息；Some 之後所有操作直接拒絕。
    needs_reauth: Mutex<Option<String>>,
}

impl TokenManager {
    pub fn new(origin: &str) -> TokenManager {
        TokenManager {
            origin: origin.to_string(),
            bearer: Mutex::new(None),
            needs_reauth: Mutex::new(None),
        }
    }

    /// 登入流程換得的 access token 交接進來；重新登入即復原 needs-reauth。
    pub fn adopt_access_token(&self, token: &str) {
        *self.bearer.lock().expect("bearer lock") = Some(token.to_string());
        *self.needs_reauth.lock().expect("reauth lock") = None;
    }

    /// TS 可查的連線狀態：Some(繁中訊息)＝需重新認證。
    pub fn needs_reauth(&self) -> Option<String> {
        self.needs_reauth.lock().expect("reauth lock").clone()
    }

    /// 以有效 bearer 執行一次請求（呼叫端逐請求建構 speclink-remote Client）：
    /// 請求前自動換發、401 → 換發一次 → 重試一次 → 仍 401 即 needs-reauth。
    pub fn execute<T>(
        &self,
        credentials: &dyn CredentialStore,
        call: impl Fn(&str) -> Result<T, RemoteError>,
    ) -> Result<T, RemoteError> {
        if self.needs_reauth().is_some() {
            return Err(rejected());
        }
        let bearer = self.acquire(credentials)?;
        match call(&bearer) {
            Err(e) if e.status == Some(401) => {
                // 快取的 bearer 已死：換發後恰好重試一次。PAT 連線無可換發，
                // mint 會交回同一枚 PAT，重試再 401 即進 needs-reauth。
                *self.bearer.lock().expect("bearer lock") = None;
                let fresh = self.mint(credentials)?;
                match call(&fresh) {
                    Err(e) if e.status == Some(401) => Err(self.flag_reauth()),
                    other => other,
                }
            }
            other => other,
        }
    }

    /// 請求 bearer：快取優先，否則換發。
    fn acquire(&self, credentials: &dyn CredentialStore) -> Result<String, RemoteError> {
        if let Some(cached) = self.bearer.lock().expect("bearer lock").clone() {
            return Ok(cached);
        }
        self.mint(credentials)
    }

    /// 忽略快取取得新 bearer：refresh credential 換發（rotation 回寫在
    /// refresh_connection）→ PAT → 兩者皆無即 needs-reauth。Unavailable
    /// （5xx／不可達／Keychain 故障）是暫時性錯誤、不進 needs-reauth——
    /// 與登入編排同一原則：暫時性失敗不是 credential 失效的語意訊號。
    fn mint(&self, credentials: &dyn CredentialStore) -> Result<String, RemoteError> {
        let has_refresh = credentials
            .get(&self.origin, CredentialKind::Refresh)
            .map_err(unavailable)?
            .is_some();
        let bearer = if has_refresh {
            match refresh_connection(&self.origin, credentials) {
                Ok(access) => access,
                Err(RefreshFailure::Rejected(_)) => return Err(self.flag_reauth()),
                Err(RefreshFailure::Unavailable(message)) => return Err(unavailable(message)),
            }
        } else {
            match credentials.get(&self.origin, CredentialKind::Pat).map_err(unavailable)? {
                Some(pat) => pat,
                None => return Err(self.flag_reauth()),
            }
        };
        *self.bearer.lock().expect("bearer lock") = Some(bearer.clone());
        Ok(bearer)
    }

    /// 進入 needs-reauth：丟棄記憶體 bearer、記下狀態，回拒絕錯誤。
    fn flag_reauth(&self) -> RemoteError {
        *self.bearer.lock().expect("bearer lock") = None;
        *self.needs_reauth.lock().expect("reauth lock") = Some(REAUTH_MESSAGE.to_string());
        rejected()
    }
}

/// needs-reauth 的拒絕錯誤：繁中訊息＋機讀 reason。
fn rejected() -> RemoteError {
    RemoteError {
        message: REAUTH_MESSAGE.to_string(),
        reason: Some("needs_reauth".to_string()),
        status: None,
    }
}

/// 暫時性失敗原樣中繼（訊息來自 refresh 編排或 Keychain）。
fn unavailable(message: String) -> RemoteError {
    RemoteError { message, reason: None, status: None }
}

/// SpeclinkDataSource 的 artifact 定址是檔名（proposal.md、tasks.md、
/// specs/{cap}/spec.md），server 端點吃 artifact id（proposal、tasks、
/// specs/{cap}）——此處單點正規化，缺席才會是 404 而非 400。
fn artifact_id(artifact: &str) -> &str {
    if let Some(cap_dir) = artifact.strip_suffix("/spec.md") {
        return cap_dir;
    }
    artifact.strip_suffix(".md").unwrap_or(artifact)
}

// --- handshake 與資料面三類矩陣（決策 1、2、6） ---

/// (c) 類操作的拒絕錯誤（決策 1 凍結原則：server 缺什麼就停用什麼，不在
/// client 偽造）。TS RemoteDataSource 與任何誤達 Rust 層的呼叫共用此語意。
pub fn unsupported(operation: &str) -> RemoteError {
    RemoteError {
        message: format!("此 server 尚未提供「{operation}」——功能已停用"),
        reason: Some("unsupported".to_string()),
        status: None,
    }
}

/// 逐操作的 capability 描述（決策 2）：來源＝決策 1 矩陣常量＋handshake 的
/// events 宣告。serde 化即隨 remote_open 回 TS 的 payload；UI 據此停用
/// affordance。本地 session 的對應描述全真——同一 UI 路徑、零分岐維護。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCapabilities {
    // (a) 直達
    pub list_changes: bool,
    pub list_specs: bool,
    pub status: bool,
    pub get_document: bool,
    pub set_task_done: bool,
    pub archive: bool,
    pub list_discussions: bool,
    pub get_discussion_document: bool,
    pub promote_discussion: bool,
    pub archive_discussion: bool,
    // (b) 組合
    pub set_all_tasks: bool,
    // (c) 不支援（server 無端點；changeMeta/changeCapabilities 依實際 payload
    // 定奪為無來源——ChangeStatus/ChangeSummary 皆不帶 metadata 與 capability
    // 名清單）
    pub list_archived: bool,
    pub get_archived_document: bool,
    pub archived_capabilities: bool,
    pub search_workspace: bool,
    pub get_spec_document: bool,
    pub validate: bool,
    pub analyze: bool,
    pub delete_change: bool,
    pub move_task: bool,
    pub reorder_card: bool,
    pub change_meta: bool,
    pub change_capabilities: bool,
    /// handshake 宣告了事件能力（SSE transport 或 polling）——缺席時 UI 退化
    /// 為手動重整。
    pub live_updates: bool,
}

impl RemoteCapabilities {
    fn from_binding(binding: &BindingResponse) -> RemoteCapabilities {
        let events = &binding.capabilities.events;
        let live_updates = events.polling.is_some()
            || events.transports.iter().any(|t| t.kind == TransportKind::Sse);
        RemoteCapabilities {
            list_changes: true,
            list_specs: true,
            status: true,
            get_document: true,
            set_task_done: true,
            archive: true,
            list_discussions: true,
            get_discussion_document: true,
            promote_discussion: true,
            archive_discussion: true,
            set_all_tasks: true,
            list_archived: false,
            get_archived_document: false,
            archived_capabilities: false,
            search_workspace: false,
            get_spec_document: false,
            validate: false,
            analyze: false,
            delete_change: false,
            move_task: false,
            reorder_card: false,
            change_meta: false,
            change_capabilities: false,
            live_updates,
        }
    }
}

/// handshake 成功的開啟結果：locator 識別、顯示名與 capability 描述——
/// serde 化即 remote_open 回 TS 的 payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteOpenInfo {
    pub project_key: String,
    pub project_name: String,
    pub repo_key: String,
    pub repo_name: String,
    pub capabilities: RemoteCapabilities,
}

/// 討論清單的 TS 形狀：active＋archived（決策 1——listDiscussions 直達，
/// 背後是兩次 `/discussions` 查詢）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionLists {
    pub active: Vec<DiscussionInfo>,
    pub archived: Vec<DiscussionInfo>,
}

/// 組合類寫入中止時的回報：已完成筆數＋當下錯誤（規格「批次任務操作……
/// 中途失敗 SHALL 中止並回報已完成筆數」）。
#[derive(Debug)]
pub struct SetTasksFailure {
    pub completed: usize,
    pub error: RemoteError,
}

/// 以 project[/repo] 識別對 origin handshake（決策 6，fail-closed）：成功回
/// workspace 把手與開啟結果；任何失敗（403/404/多義）原樣回錯、不建立任何
/// runtime 狀態。
pub fn open_workspace(
    origin: &str,
    target: &str,
    manager: &Arc<TokenManager>,
    credentials: &dyn CredentialStore,
) -> Result<(RemoteWorkspace, RemoteOpenInfo), RemoteError> {
    let (project, repo) = match target.split_once('/') {
        Some((project, repo)) => (project.trim(), Some(repo.trim().to_string())),
        None => (target.trim(), None),
    };
    let project_base =
        format!("{}/api/speclink/v1/projects/{project}", origin.trim_end_matches('/'));
    let repo_header = repo.clone();
    let binding = manager.execute(credentials, |token| {
        Client::new(&project_base, token, repo_header.as_deref()).handshake()
    })?;
    let info = RemoteOpenInfo {
        project_key: binding.project.key.clone(),
        project_name: binding.project.name.clone(),
        repo_key: binding.repo.key.clone(),
        repo_name: binding.repo.name.clone(),
        capabilities: RemoteCapabilities::from_binding(&binding),
    };
    let workspace = RemoteWorkspace {
        project_base,
        // 以 handshake 裁定後的 repo 為準（省略 repo 時 server 綁定唯一 repo）。
        repo: binding.repo.key,
        manager: Arc::clone(manager),
    };
    Ok((workspace, info))
}

/// 一個 handshake 成功後的 remote workspace 把手：資料面操作的 (a) 直達與
/// (b) 組合實作，全部經 TokenManager 逐請求建構 Client（決策 4、7）。
pub struct RemoteWorkspace {
    project_base: String,
    repo: String,
    manager: Arc<TokenManager>,
}

// 手寫 Debug：TokenManager 持有 bearer，絕不進任何 Debug 輸出。
impl std::fmt::Debug for RemoteWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteWorkspace")
            .field("project_base", &self.project_base)
            .field("repo", &self.repo)
            .finish_non_exhaustive()
    }
}

impl RemoteWorkspace {
    /// 以已知的 locator 識別直接建構把手——open 之後的資料面呼叫走這裡
    /// （handshake 是 session 前置、不是逐請求前置），命令層據此無狀態重建。
    pub fn at(
        origin: &str,
        project: &str,
        repo: &str,
        manager: &Arc<TokenManager>,
    ) -> RemoteWorkspace {
        RemoteWorkspace {
            project_base: format!(
                "{}/api/speclink/v1/projects/{project}",
                origin.trim_end_matches('/')
            ),
            repo: repo.to_string(),
            manager: Arc::clone(manager),
        }
    }

    /// 逐請求建構 Client 執行一擊（token 生命週期由 TokenManager 承擔）。
    fn run<T>(
        &self,
        credentials: &dyn CredentialStore,
        call: impl Fn(&Client) -> Result<T, RemoteError>,
    ) -> Result<T, RemoteError> {
        self.manager.execute(credentials, |token| {
            call(&Client::new(&self.project_base, token, Some(&self.repo)))
        })
    }

    pub fn list_changes(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<ListChangesResponse, RemoteError> {
        self.run(credentials, |client| client.list_changes())
    }

    pub fn list_specs(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<ListSpecsResponse, RemoteError> {
        self.run(credentials, |client| client.list_specs())
    }

    pub fn change_status(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<ChangeStatus, RemoteError> {
        self.run(credentials, |client| client.get_change(change))
    }

    pub fn document(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        artifact: &str,
    ) -> Result<ArtifactContent, RemoteError> {
        let artifact = artifact_id(artifact);
        self.run(credentials, |client| client.get_artifact(change, artifact))
    }

    pub fn set_task_done(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        task: &str,
        done: bool,
    ) -> Result<(), RemoteError> {
        self.run(credentials, |client| {
            if done {
                client.task_done(change, task, &[]).map(|_| ())
            } else {
                client.task_undone(change, task).map(|_| ())
            }
        })
    }

    pub fn claim(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<ClaimResponse, RemoteError> {
        self.run(credentials, |client| client.claim(change))
    }

    pub fn archive(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<ArchiveResponse, RemoteError> {
        self.run(credentials, |client| client.archive(change))
    }

    pub fn list_discussions(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<DiscussionLists, RemoteError> {
        let active = self.run(credentials, |client| client.list_discussions(false))?;
        let archived = self.run(credentials, |client| client.list_discussions(true))?;
        Ok(DiscussionLists { active: active.discussions, archived: archived.discussions })
    }

    pub fn discussion_document(
        &self,
        credentials: &dyn CredentialStore,
        slug: &str,
    ) -> Result<ShowDiscussionResponse, RemoteError> {
        self.run(credentials, |client| client.show_discussion(slug))
    }

    pub fn promote_discussion(
        &self,
        credentials: &dyn CredentialStore,
        slug: &str,
        name: Option<&str>,
    ) -> Result<PromoteDiscussionResponse, RemoteError> {
        self.run(credentials, |client| client.discussion_promote(slug, name))
    }

    pub fn archive_discussion(
        &self,
        credentials: &dyn CredentialStore,
        slug: &str,
    ) -> Result<ArchiveDiscussionResponse, RemoteError> {
        self.run(credentials, |client| client.discussion_archive(slug))
    }

    /// (b) 組合：逐任務寫回，非原子——中途失敗即中止並回報已完成筆數。
    pub fn set_tasks(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        task_ids: &[String],
        done: bool,
    ) -> Result<usize, SetTasksFailure> {
        let mut completed = 0;
        for task in task_ids {
            self.set_task_done(credentials, change, task, done)
                .map_err(|error| SetTasksFailure { completed, error })?;
            completed += 1;
        }
        Ok(completed)
    }

    /// (b) 組合的 setAllTasks：以 server 的任務清單（apply instructions）取
    /// 未達目標態的任務逐筆寫回。
    pub fn set_all_tasks(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        done: bool,
    ) -> Result<usize, SetTasksFailure> {
        let instructions = self
            .run(credentials, |client| client.apply_instructions(change))
            .map_err(|error| SetTasksFailure { completed: 0, error })?;
        let pending: Vec<String> = instructions
            .tasks
            .into_iter()
            .filter(|task| task.done != done)
            .map(|task| task.id)
            .collect();
        self.set_tasks(credentials, change, &pending, done)
    }
}

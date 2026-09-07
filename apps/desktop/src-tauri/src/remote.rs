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

use crate::connections::RefreshFailure;
use speclink_remote::credentials::{CredentialKind, CredentialStore};
use serde::{Deserialize, Serialize};
use speclink_core::model::{require_valid_meta, ChangeMeta};
use speclink_core::store::Store;
use speclink_desktop_core::settings::{
    read_workflow_settings_from_text, rewrite_workflow_content_text, rewrite_workflow_fields_text,
    AppSettings, ContextEdit, WorkflowPolicyFields, WorkflowSettings,
};
use speclink_protocol::binding::BindingResponse;
use speclink_protocol::command::{
    ArchiveDiscussionResponse, ArchiveResponse, ClaimResponse, PromoteDiscussionResponse,
};
use speclink_protocol::events::TransportKind;
use speclink_protocol::query::{
    ArchivedListResponse, ArtifactContent, ChangeStatus, ChangeSummary, DiscussionInfo,
    ImportBundle, ImportBundleDocument, ImportDocumentId, ImportReportResponse, ImportScope,
    ListChangesResponse, ListSpecsResponse, ScopesResponse, SearchResponse,
    ShowDiscussionResponse, SpecDocumentResponse,
};
use speclink_remote::client::Client;
use speclink_remote::RemoteError;
use speclink_store::content_digest;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// needs-reauth 的繁中狀態訊息（完整重新認證 UX 屬後續刀，本刀只回報狀態）。
const REAUTH_MESSAGE: &str = "此連線的登入已失效——請重新登入";
const OFFLINE_MESSAGE: &str = "此連線目前離線——顯示最後成功載入的內容";
const OFFLINE_WRITE_MESSAGE: &str = "此連線目前離線——寫入已拒絕，未排隊或暫存";
pub const REMOTE_CONNECTION_STATE_EVENT: &str = "remote-connection-state";
pub const DEFAULT_FAILURE_THRESHOLD: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionState {
    Online,
    Offline,
    NeedsReauth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStateEvent {
    pub connection_id: String,
    pub state: ConnectionState,
    pub message: Option<String>,
}

/// `remote_open` 專用的 IPC failure shape。只公開復原分類需要的欄位，
/// credential、header 與 Keychain 內容都不跨越 Tauri 邊界。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteOpenFailure {
    pub message: String,
    pub reason: Option<String>,
    pub status: Option<u16>,
}

impl RemoteOpenFailure {
    pub fn unknown(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            reason: None,
            status: None,
        }
    }
}

impl From<RemoteError> for RemoteOpenFailure {
    fn from(error: RemoteError) -> Self {
        Self {
            message: error.message,
            reason: error.reason,
            status: error.status,
        }
    }
}

struct ConnectionHealth {
    connection_id: Option<String>,
    state: ConnectionState,
    consecutive_failures: usize,
}

/// 一條 connection 的 token 生命週期管理者。
pub struct TokenManager {
    origin: String,
    /// 記憶體持有的 bearer；絕不落盤、絕不過境 TS。
    bearer: Mutex<Option<String>>,
    /// 同一 connection 的 credential 取得／輪替 singleflight。
    rotation: Mutex<()>,
    /// needs-reauth 狀態訊息；Some 之後所有操作直接拒絕。
    needs_reauth: Mutex<Option<String>>,
    health: Mutex<ConnectionHealth>,
    failure_threshold: usize,
    state_observer: Arc<dyn Fn(ConnectionStateEvent) + Send + Sync>,
}

impl TokenManager {
    pub fn new(origin: &str) -> TokenManager {
        Self::with_state_observer(origin, DEFAULT_FAILURE_THRESHOLD, |_| {})
    }

    pub fn with_connection_state(
        origin: &str,
        connection_id: &str,
        failure_threshold: usize,
        observer: impl Fn(ConnectionStateEvent) + Send + Sync + 'static,
    ) -> TokenManager {
        let manager = Self::with_state_observer(origin, failure_threshold, observer);
        manager.bind_connection_id(connection_id);
        manager
    }

    pub(crate) fn with_state_observer(
        origin: &str,
        failure_threshold: usize,
        observer: impl Fn(ConnectionStateEvent) + Send + Sync + 'static,
    ) -> TokenManager {
        TokenManager {
            origin: origin.to_string(),
            bearer: Mutex::new(None),
            rotation: Mutex::new(()),
            needs_reauth: Mutex::new(None),
            health: Mutex::new(ConnectionHealth {
                connection_id: None,
                state: ConnectionState::Online,
                consecutive_failures: 0,
            }),
            failure_threshold: failure_threshold.max(1),
            state_observer: Arc::new(observer),
        }
    }

    pub fn bind_connection_id(&self, connection_id: &str) {
        self.health.lock().expect("health lock").connection_id = Some(connection_id.to_string());
    }

    /// 登入流程換得的 access token 交接進來；重新登入即復原 needs-reauth。
    pub fn adopt_access_token(&self, token: &str) {
        *self.bearer.lock().expect("bearer lock") = Some(token.to_string());
        *self.needs_reauth.lock().expect("reauth lock") = None;
        self.transition_online();
    }

    /// TS 可查的連線狀態：Some(繁中訊息)＝需重新認證。
    pub fn needs_reauth(&self) -> Option<String> {
        self.needs_reauth.lock().expect("reauth lock").clone()
    }

    pub fn connection_state(&self) -> ConnectionState {
        self.health.lock().expect("health lock").state
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
        let bearer = match self.acquire(credentials) {
            Ok(bearer) => bearer,
            Err(error) => return self.observe_result(Err(error)),
        };
        match call(&bearer) {
            Err(e) if e.status == Some(401) => {
                // 快取的 bearer 已死：換發後恰好重試一次。PAT 連線無可換發，
                // mint 會交回同一枚 PAT，重試再 401 即進 needs-reauth。
                let fresh = match self.recover_after_unauthorized(credentials, &bearer) {
                    Ok(fresh) => fresh,
                    Err(error) => return self.observe_result(Err(error)),
                };
                match call(&fresh) {
                    Err(e) if e.status == Some(401) => Err(self.flag_reauth()),
                    other => self.observe_result(other),
                }
            }
            other => self.observe_result(other),
        }
    }

    pub fn execute_write<T>(
        &self,
        credentials: &dyn CredentialStore,
        call: impl Fn(&str) -> Result<T, RemoteError>,
    ) -> Result<T, RemoteError> {
        self.ensure_write_allowed()?;
        self.execute(credentials, call)
    }

    fn ensure_write_allowed(&self) -> Result<(), RemoteError> {
        match self.connection_state() {
            ConnectionState::Online => Ok(()),
            ConnectionState::Offline => Err(offline_rejected()),
            ConnectionState::NeedsReauth => Err(rejected()),
        }
    }

    fn observe_result<T>(&self, result: Result<T, RemoteError>) -> Result<T, RemoteError> {
        match result {
            Ok(value) => {
                self.record_success();
                Ok(value)
            }
            Err(error) => {
                if is_transport_failure(&error) {
                    self.record_transport_failure();
                }
                Err(error)
            }
        }
    }

    fn record_success(&self) {
        let event = {
            let mut health = self.health.lock().expect("health lock");
            health.consecutive_failures = 0;
            if health.state == ConnectionState::Offline {
                health.state = ConnectionState::Online;
                health
                    .connection_id
                    .clone()
                    .map(|connection_id| ConnectionStateEvent {
                        connection_id,
                        state: ConnectionState::Online,
                        message: None,
                    })
            } else {
                None
            }
        };
        self.emit_state(event);
    }

    fn record_transport_failure(&self) {
        let event = {
            let mut health = self.health.lock().expect("health lock");
            if health.state == ConnectionState::NeedsReauth {
                return;
            }
            health.consecutive_failures = health.consecutive_failures.saturating_add(1);
            if health.consecutive_failures >= self.failure_threshold
                && health.state != ConnectionState::Offline
            {
                health.state = ConnectionState::Offline;
                health
                    .connection_id
                    .clone()
                    .map(|connection_id| ConnectionStateEvent {
                        connection_id,
                        state: ConnectionState::Offline,
                        message: Some(OFFLINE_MESSAGE.to_string()),
                    })
            } else {
                None
            }
        };
        self.emit_state(event);
    }

    fn transition_online(&self) {
        let event = {
            let mut health = self.health.lock().expect("health lock");
            health.consecutive_failures = 0;
            if health.state != ConnectionState::Online {
                health.state = ConnectionState::Online;
                health
                    .connection_id
                    .clone()
                    .map(|connection_id| ConnectionStateEvent {
                        connection_id,
                        state: ConnectionState::Online,
                        message: None,
                    })
            } else {
                None
            }
        };
        self.emit_state(event);
    }

    fn transition_needs_reauth(&self) {
        let event = {
            let mut health = self.health.lock().expect("health lock");
            health.consecutive_failures = 0;
            if health.state != ConnectionState::NeedsReauth {
                health.state = ConnectionState::NeedsReauth;
                health
                    .connection_id
                    .clone()
                    .map(|connection_id| ConnectionStateEvent {
                        connection_id,
                        state: ConnectionState::NeedsReauth,
                        message: Some(REAUTH_MESSAGE.to_string()),
                    })
            } else {
                None
            }
        };
        self.emit_state(event);
    }

    fn emit_state(&self, event: Option<ConnectionStateEvent>) {
        if let Some(event) = event {
            (self.state_observer)(event);
        }
    }

    /// 請求 bearer：快取優先，否則換發。
    fn acquire(&self, credentials: &dyn CredentialStore) -> Result<String, RemoteError> {
        if let Some(cached) = self.bearer.lock().expect("bearer lock").clone() {
            return Ok(cached);
        }
        let _rotation = self.rotation.lock().expect("rotation lock");
        if self.needs_reauth().is_some() {
            return Err(rejected());
        }
        if let Some(cached) = self.bearer.lock().expect("bearer lock").clone() {
            return Ok(cached);
        }
        self.mint(credentials)
    }

    /// 401 後只允許一個 caller 輪替；等待者若看到已發布且不同於自己被拒的
    /// bearer，直接複用先行者結果，不得清掉新 token 再消耗 refresh。
    fn recover_after_unauthorized(
        &self,
        credentials: &dyn CredentialStore,
        rejected_bearer: &str,
    ) -> Result<String, RemoteError> {
        let _rotation = self.rotation.lock().expect("rotation lock");
        if self.needs_reauth().is_some() {
            return Err(rejected());
        }
        if let Some(current) = self.bearer.lock().expect("bearer lock").clone() {
            if current != rejected_bearer {
                return Ok(current);
            }
        }
        *self.bearer.lock().expect("bearer lock") = None;
        // 被拒的 bearer 也躺在共用快取裡——不清掉的話下一步會原樣撿回來。
        speclink_remote::refresh::clear_cached_bearer(&self.origin, credentials);
        self.mint(credentials)
    }

    /// 記憶體無 bearer 時取得一個：共用的 access token 快取（CLI 可能剛換發過）
    /// → refresh credential 換發（持跨行程鎖，回寫在下沉層）→ PAT → 皆無即
    /// needs-reauth。Unavailable（5xx／不可達／Keychain 故障）是暫時性錯誤、
    /// 不進 needs-reauth——與登入編排同一原則：暫時性失敗不是 credential 失效
    /// 的語意訊號。
    fn mint(&self, credentials: &dyn CredentialStore) -> Result<String, RemoteError> {
        let has_refresh = credentials
            .get(&self.origin, CredentialKind::Refresh)
            .map_err(unavailable)?
            .is_some();
        let bearer = if has_refresh {
            match speclink_remote::refresh::bearer_for(
                &self.origin,
                credentials,
                &speclink_remote::auth::speclink_config_dir(),
            ) {
                Ok(access) => access,
                Err(RefreshFailure::Rejected(_)) => return Err(self.flag_reauth()),
                Err(RefreshFailure::Unavailable(message)) => return Err(unavailable(message)),
            }
        } else {
            match credentials
                .get(&self.origin, CredentialKind::Pat)
                .map_err(unavailable)?
            {
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
        self.transition_needs_reauth();
        rejected()
    }
}

fn is_transport_failure(error: &RemoteError) -> bool {
    error.status.is_none()
        && error.reason.is_none()
        && error.message.starts_with("server unreachable")
}

/// needs-reauth 的拒絕錯誤：繁中訊息＋機讀 reason。
fn rejected() -> RemoteError {
    RemoteError {
        message: REAUTH_MESSAGE.to_string(),
        reason: Some("needs_reauth".to_string()),
        status: None,
        evidence: None,
    }
}

fn offline_rejected() -> RemoteError {
    RemoteError {
        message: OFFLINE_WRITE_MESSAGE.to_string(),
        reason: Some("offline".to_string()),
        status: None,
        evidence: None,
    }
}

/// 暫時性失敗原樣中繼（訊息來自 refresh 編排或 Keychain）。
fn unavailable(message: String) -> RemoteError {
    RemoteError {
        message,
        reason: None,
        status: None,
        evidence: None,
    }
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

/// Read one local filesystem workspace through the Engine Store seam and
/// produce the migration wire bundle. Every source is read before upload; any
/// corrupt metadata or unsupported artifact pattern aborts the whole build.
pub fn build_import_bundle(root: &Path, project: &str, repo: &str) -> Result<ImportBundle, String> {
    if project.trim().is_empty() || repo.trim().is_empty() {
        return Err("migration target project and repo must both be selected".to_string());
    }
    let context = speclink_desktop_core::init_core_context(root)
        .ok_or_else(|| format!("not a local speclink workspace: {}", root.display()))?;
    let store: &dyn Store = &context.store;
    let user_dir = speclink_host::context::global_config_dir();
    let mut documents = BTreeMap::<ImportDocumentId, String>::new();

    for change in store.list_changes() {
        require_valid_meta(&change).map_err(|error| error.to_string())?;
        let meta = store.read_change_meta(&change.name).unwrap_or_default();
        insert_import_document(
            &mut documents,
            ImportDocumentId::ChangeMeta {
                change: change.name.clone(),
            },
            meta,
        )?;
        let schema =
            resolve_migration_schema(&context.workspace, &user_dir, &change.meta.schema_name())?;
        for artifact in migration_artifact_paths(&schema, store.delta_capabilities(&change.name))? {
            if let Some(content) = store.read_artifact(&change.name, &artifact) {
                insert_import_document(
                    &mut documents,
                    ImportDocumentId::ChangeArtifact {
                        change: change.name.clone(),
                        artifact,
                    },
                    content,
                )?;
            }
        }
    }

    let mut capabilities = store.list_canonical_capabilities();
    capabilities.sort();
    for capability in capabilities {
        if let Some(content) = store.read_canonical_spec(&capability) {
            insert_import_document(
                &mut documents,
                ImportDocumentId::CanonicalSpec { capability },
                content,
            )?;
        }
    }

    for discussion in store.list_live_discussions() {
        insert_import_document(
            &mut documents,
            ImportDocumentId::Discussion {
                slug: discussion.slug,
                archived: false,
            },
            discussion.text,
        )?;
    }
    for discussion in store.list_archived_discussions() {
        insert_import_document(
            &mut documents,
            ImportDocumentId::Discussion {
                slug: discussion.slug,
                archived: true,
            },
            discussion.text,
        )?;
    }

    for dated_name in store.list_archived_changes() {
        let meta = store.read_archived_meta(&dated_name).unwrap_or_default();
        let parsed = ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
            format!("invalid openspec/changes/archive/{dated_name}/.openspec.yaml: {reason}")
        })?;
        insert_import_document(
            &mut documents,
            ImportDocumentId::ArchivedChange {
                change: dated_name.clone(),
                doc: ".openspec.yaml".to_string(),
            },
            meta,
        )?;
        let schema =
            resolve_migration_schema(&context.workspace, &user_dir, &parsed.schema_name())?;
        for artifact in
            migration_artifact_paths(&schema, store.archived_delta_capabilities(&dated_name))?
        {
            if let Some(content) = store.read_archived_artifact(&dated_name, &artifact) {
                insert_import_document(
                    &mut documents,
                    ImportDocumentId::ArchivedChange {
                        change: dated_name.clone(),
                        doc: artifact,
                    },
                    content,
                )?;
            }
        }
    }

    if let Some(content) = store.read_workflow_config() {
        insert_import_document(&mut documents, ImportDocumentId::WorkflowConfig, content)?;
    }
    if let Some(content) = store.read_language() {
        insert_import_document(&mut documents, ImportDocumentId::Language, content)?;
    }

    Ok(ImportBundle {
        format_version: speclink_store::BUNDLE_FORMAT_VERSION,
        scope: ImportScope {
            project: project.to_string(),
            repo: repo.to_string(),
        },
        project_revision: 0,
        documents: documents
            .into_iter()
            .map(|(document, content)| ImportBundleDocument {
                digest: content_digest(&content),
                document,
                content,
            })
            .collect(),
    })
}

fn resolve_migration_schema(
    workspace: &speclink_core::workspace::Workspace,
    user_dir: &Path,
    name: &str,
) -> Result<speclink_core::schema::Schema, String> {
    speclink_core::schema::resolve_with(Some(workspace), Some(user_dir), name)
        .ok_or_else(|| speclink_core::schema::not_found_msg(name))?
}

fn migration_artifact_paths(
    schema: &speclink_core::schema::Schema,
    delta_capabilities: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut paths = BTreeSet::new();
    for artifact in &schema.artifacts {
        if artifact.output_path == "specs/**/*.md" {
            paths.extend(
                delta_capabilities
                    .iter()
                    .map(|capability| format!("specs/{capability}/spec.md")),
            );
        } else if artifact.output_path.contains('*') {
            return Err(format!(
                "cannot enumerate migration artifact pattern '{}' in schema '{}'",
                artifact.output_path, schema.name
            ));
        } else {
            paths.insert(artifact.output_path.clone());
        }
    }
    Ok(paths.into_iter().collect())
}

fn insert_import_document(
    documents: &mut BTreeMap<ImportDocumentId, String>,
    document: ImportDocumentId,
    content: String,
) -> Result<(), String> {
    if documents.insert(document.clone(), content).is_some() {
        return Err(format!("duplicate local migration document: {document:?}"));
    }
    Ok(())
}

/// Successful local-to-remote conversion. The UI uses `checkoutRoot` to replace
/// the local tab in place and displays the retained backup path.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationResult {
    pub report: ImportReportResponse,
    pub backup_path: String,
    pub checkout_root: String,
}

/// Result of resolving a coexistence conflict in favor of server truth. This
/// path never builds or uploads a Bundle; it only retains the local tree.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteAdoptionResult {
    pub backup_path: String,
    pub checkout_root: String,
}

/// Bundle and import a local workspace, then—and only then—rename the local
/// `openspec/` tree to a dated backup and write the remote marker.
pub fn migrate_workspace(
    root: &Path,
    origin: &str,
    project: &str,
    repo: &str,
    manager: &Arc<TokenManager>,
    credentials: &dyn CredentialStore,
) -> Result<MigrationResult, String> {
    let context = speclink_desktop_core::init_core_context(root)
        .ok_or_else(|| format!("not a local speclink workspace: {}", root.display()))?;
    if context.workspace.spec_dir_name != "openspec" {
        return Err(format!(
            "local-to-remote migration requires the default openspec/ directory; this workspace uses {}/",
            context.workspace.spec_dir_name
        ));
    }
    let checkout_root = context.workspace.root;
    let bundle = build_import_bundle(root, project, repo)?;
    let project_base = format!(
        "{}/api/speclink/v1/projects/{project}",
        origin.trim_end_matches('/')
    );
    let report = manager
        .execute_write(credentials, |token| {
            Client::new(&project_base, token, Some(repo)).import(&bundle)
        })
        .map_err(|error| error.message)?;

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let backup_path =
        finalize_local_migration_with(&checkout_root, origin, project, repo, &date, |from, to| {
            std::fs::rename(from, to)
        })?;
    Ok(MigrationResult {
        report,
        backup_path: backup_path.display().to_string(),
        checkout_root: checkout_root.display().to_string(),
    })
}

/// Retain the local `openspec/` tree and leave the existing remote marker
/// byte-for-byte intact. Used only after the UI has successfully handshaken
/// with the marker's server scope.
pub fn adopt_remote_workspace(root: &Path) -> Result<RemoteAdoptionResult, String> {
    let context = speclink_desktop_core::init_core_context(root)
        .ok_or_else(|| format!("not a local speclink workspace: {}", root.display()))?;
    if context.workspace.spec_dir_name != "openspec" {
        return Err(format!(
            "adopting server truth requires the default openspec/ directory; this workspace uses {}/",
            context.workspace.spec_dir_name
        ));
    }
    let checkout_root = context.workspace.root;
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let backup_path =
        adopt_remote_workspace_with(&checkout_root, &date, |from, to| std::fs::rename(from, to))?;
    Ok(RemoteAdoptionResult {
        backup_path: backup_path.display().to_string(),
        checkout_root: checkout_root.display().to_string(),
    })
}

fn adopt_remote_workspace_with(
    root: &Path,
    date: &str,
    rename: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<std::path::PathBuf, String> {
    let marker = root.join(".speclink.yaml");
    if !marker.is_file() {
        return Err(format!(
            "cannot adopt server truth without an existing remote marker: {}",
            marker.display()
        ));
    }
    let source = root.join("openspec");
    let backup = next_migration_backup(root, date);
    rename(&source, &backup).map_err(|error| {
        format!(
            "server content was verified but the local backup rename failed ({error}). Local openspec/ remains intact; no upload or server write was attempted. Rename '{}' to '{}' manually before reopening the checkout.",
            source.display(),
            backup.display()
        )
    })?;
    Ok(backup)
}

fn finalize_local_migration_with(
    root: &Path,
    origin: &str,
    project: &str,
    repo: &str,
    date: &str,
    rename: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<std::path::PathBuf, String> {
    let source = root.join("openspec");
    let backup = next_migration_backup(root, date);
    rename(&source, &backup).map_err(|error| {
        format!(
            "server import succeeded, but the local backup rename failed ({error}). Local openspec/ remains intact and no remote marker was written. Rename '{}' to '{}' manually, then reconnect this folder as a checkout; do not retry import into the same scope.",
            source.display(),
            backup.display()
        )
    })?;

    let project_url = format!(
        "{}/api/speclink/v1/projects/{project}",
        origin.trim_end_matches('/')
    );
    speclink_core::config::write_remote_section(root, &project_url, Some(repo)).map_err(|error| {
        format!(
            "server import and local backup succeeded, but the remote marker could not be written ({error}). The backup is retained at '{}'; repair .speclink.yaml before opening this folder as a checkout.",
            backup.display()
        )
    })?;
    Ok(backup)
}

fn next_migration_backup(root: &Path, date: &str) -> std::path::PathBuf {
    let base = format!("openspec.migrated-{date}");
    let first = root.join(&base);
    if !first.exists() {
        return first;
    }
    let mut sequence = 2_u64;
    loop {
        let candidate = root.join(format!("{base}-{sequence}"));
        if !candidate.exists() {
            return candidate;
        }
        sequence += 1;
    }
}

// --- handshake 與資料面三類矩陣（決策 1、2、6） ---

/// (c) 類操作的拒絕錯誤（決策 1 凍結原則：server 缺什麼就停用什麼，不在
/// client 偽造）。TS RemoteDataSource 與任何誤達 Rust 層的呼叫共用此語意。
pub fn unsupported(operation: &str) -> RemoteError {
    RemoteError {
        message: format!("此 server 尚未提供「{operation}」——功能已停用"),
        reason: Some("unsupported".to_string()),
        status: None,
        evidence: None,
    }
}

// --- 看板順序 overlay 與拖排寫回（remote-board-order） ---

/// 桌面側 board resource 的預期 JSON 形狀（決策 2）：兩段 rank 圖。server 視
/// 內容為不透明文本，解析（與損壞容錯）整個歸桌面（決策 6）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct BoardOrderDoc {
    pub(crate) changes: BTreeMap<String, String>,
    pub(crate) discussions: BTreeMap<String, String>,
}

/// 解析 board resource 內容：缺席或無法解析為預期形狀（壞 JSON、非物件）
/// 一律視為全員缺 rank——回退序照常渲染、看板不 fail（決策 6）。
pub(crate) fn parse_board_order(content: Option<&str>) -> BoardOrderDoc {
    content
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or_default()
}

/// 穩定排序疊上 rank 複合鍵（與本地 board_sorted_changes 同構——決策 4）：
/// 缺值置頂維持 server 回傳序、具值依 rank 位元組字典序升冪、同值以名稱決斷。
fn sort_with_ranks<T>(
    items: &mut [T],
    ranks: &BTreeMap<String, String>,
    name_of: impl Fn(&T) -> &str,
) {
    items.sort_by(|x, y| match (ranks.get(name_of(x)), ranks.get(name_of(y))) {
        (None, None) => std::cmp::Ordering::Equal, // 穩定排序保留 server 回傳序
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(b).then_with(|| name_of(x).cmp(name_of(y))),
    });
}

pub(crate) fn overlay_changes_order(items: &mut [ChangeSummary], board: &BoardOrderDoc) {
    sort_with_ranks(items, &board.changes, |c| &c.name);
}

pub(crate) fn overlay_discussions_order(items: &mut [DiscussionInfo], board: &BoardOrderDoc) {
    sort_with_ranks(items, &board.discussions, |d| &d.slug);
}

/// 變更卡所屬欄——與前端 changeStage 的推導同構：全完成＝已就緒優先；
/// meta 開工章（wire 的 startedAt）或任務完成數 > 0＝進行中（fallback 涵蓋
/// 手改 tasks.md 等繞過工具的寫入路徑）；其餘＝提案中。
pub fn change_stage(c: &ChangeSummary) -> u8 {
    if c.total_tasks > 0 && c.completed_tasks >= c.total_tasks {
        2
    } else if c.started_at.is_some() || c.completed_tasks > 0 {
        1
    } else {
        0
    }
}

/// reorder 一次嘗試所需的 server 現況：兩份清單（修剪與欄推導用）、board
/// resource 內容與其 CAS revision。
#[derive(Debug, Clone)]
pub(crate) struct BoardSnapshot {
    pub(crate) changes: Vec<ChangeSummary>,
    pub(crate) discussions: Vec<DiscussionInfo>,
    pub(crate) content: Option<String>,
    pub(crate) revision: u64,
}

/// 欄內有缺 rank 卡時依當前顯示序整欄補章（決策 5——等距鍵只落在 board
/// resource 圖內，與本地 reorder 的補章語意同構）；回傳欄成員的 id→rank 表。
fn ensure_column_ranks<'a>(
    members: &[&'a str],
    map: &mut BTreeMap<String, String>,
) -> HashMap<&'a str, String> {
    if members.iter().any(|member| !map.contains_key(*member)) {
        let keys = speclink_desktop_core::rank::spread(members.len());
        for (member, key) in members.iter().zip(&keys) {
            map.insert((*member).to_string(), key.clone());
        }
        members.iter().copied().zip(keys).collect()
    } else {
        members
            .iter()
            .map(|member| (*member, map[*member].clone()))
            .collect()
    }
}

fn card_not_found(id: &str) -> RemoteError {
    RemoteError {
        message: format!("查無此卡：{id}"),
        reason: Some("not_found".to_string()),
        status: None,
        evidence: None,
    }
}

/// 計算一次拖排寫回的全文（決策 5）：修剪不在現行清單的條目 → 推導被拖卡
/// 所在欄成員（顯示序）→ 缺 rank 整欄補章 → 落點鄰居中點鍵 → 序列化。
pub(crate) fn reorder_full_text(
    snapshot: &BoardSnapshot,
    kind: &str,
    id: &str,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> Result<String, RemoteError> {
    let mut doc = parse_board_order(snapshot.content.as_deref());
    doc.changes
        .retain(|name, _| snapshot.changes.iter().any(|c| &c.name == name));
    doc.discussions
        .retain(|slug, _| snapshot.discussions.iter().any(|d| &d.slug == slug));

    match kind {
        "change" => {
            let mut ordered = snapshot.changes.clone();
            overlay_changes_order(&mut ordered, &doc);
            let dragged = ordered
                .iter()
                .find(|c| c.name == id)
                .ok_or_else(|| card_not_found(id))?;
            let stage = change_stage(dragged);
            let column: Vec<&str> = ordered
                .iter()
                .filter(|c| change_stage(c) == stage)
                .map(|c| c.name.as_str())
                .collect();
            let ranks = ensure_column_ranks(&column, &mut doc.changes);
            let key = speclink_desktop_core::rank::neighbor_midpoint(&ranks, prev_id, next_id);
            doc.changes.insert(id.to_string(), key);
        }
        "discussion" => {
            let mut ordered = snapshot.discussions.clone();
            overlay_discussions_order(&mut ordered, &doc);
            if !ordered.iter().any(|d| d.slug == id) {
                return Err(card_not_found(id));
            }
            let column: Vec<&str> = ordered.iter().map(|d| d.slug.as_str()).collect();
            let ranks = ensure_column_ranks(&column, &mut doc.discussions);
            let key = speclink_desktop_core::rank::neighbor_midpoint(&ranks, prev_id, next_id);
            doc.discussions.insert(id.to_string(), key);
        }
        other => {
            return Err(RemoteError {
                message: format!("無效的卡片類別：{other}"),
                reason: Some("invalid_argument".to_string()),
                status: None,
                evidence: None,
            })
        }
    }
    serde_json::to_string(&doc).map_err(|error| RemoteError {
        message: format!("board resource 序列化失敗：{error}"),
        reason: None,
        status: None,
        evidence: None,
    })
}

/// 拖排寫回的 CAS 收斂迴圈（決策 5）：讀現況 → 重算全文 → PUT 帶 If-Match；
/// 409 重讀重算重試恰一次，再敗原樣回錯——呼叫端刷新 server 現況，絕不保留
/// 未落檔的假象順序。
pub(crate) fn reorder_via(
    read: impl Fn() -> Result<BoardSnapshot, RemoteError>,
    put: impl Fn(&str, u64) -> Result<(), RemoteError>,
    kind: &str,
    id: &str,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> Result<(), RemoteError> {
    let mut retried = false;
    loop {
        let snapshot = read()?;
        let text = reorder_full_text(&snapshot, kind, id, prev_id, next_id)?;
        match put(&text, snapshot.revision) {
            Ok(()) => return Ok(()),
            Err(error)
                if !retried && error.reason.as_deref() == Some("revision_conflict") =>
            {
                retried = true;
            }
            Err(error) => return Err(error),
        }
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
    // 新增的 server 純讀取面（封存、搜尋、正典 spec 內文）
    pub list_archived: bool,
    pub get_archived_document: bool,
    pub archived_capabilities: bool,
    pub search_workspace: bool,
    pub get_spec_document: bool,
    // 動詞端點直達（remote-verb-parity）：唯讀衍生查詢全 role 真、寫入動詞
    // 依 handshake 的 role 呈現（server 仍是最終權限防線）。
    pub validate: bool,
    pub analyze: bool,
    pub delete_change: bool,
    pub move_task: bool,
    // 看板拖排直達 board resource（remote-board-order 決策 7）：依 role 翻真
    //（editor 真、reader 假——server 的 PUT /board-order 以同一 role bit 強制）。
    pub reorder_card: bool,
    // change 詮釋資料與 capability 清單（remote-read-parity）：ChangeStatus
    // 已攜歸屬四欄與 deltaCapabilities，TS 端以既有 remote_status payload
    // 映射實作——資料在 wire 上，capability 為真；舊 server 不送的欄位以
    // 缺席呈現（誠實降級，缺的是欄位而非能力）。
    pub change_meta: bool,
    pub change_capabilities: bool,
    /// 認領（remote-claim-ownership D4）：RemoteOnly 動詞，依 handshake role
    /// 呈現。wire 的 capability 宣告沒有專屬的 claim 位，而 editor 限定的寫入
    /// 動詞共用同一道 role 閘門——沿 reorderCard 借 policyWrite 的既有作法，
    /// 由 deleteChange 這個「本 membership 可寫 change」的宣告代讀角色。
    pub claim: bool,
    /// 此 membership 是否可寫 workflow policy；server 仍是最終權限防線。
    pub policy_write: bool,
    /// handshake 宣告了事件能力（SSE transport 或 polling）——缺席時 UI 退化
    /// 為手動重整。
    pub live_updates: bool,
}

impl RemoteCapabilities {
    fn from_binding(binding: &BindingResponse) -> RemoteCapabilities {
        let events = &binding.capabilities.events;
        let live_updates = events.polling.is_some()
            || events
                .transports
                .iter()
                .any(|t| t.kind == TransportKind::Sse);
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
            list_archived: true,
            get_archived_document: true,
            archived_capabilities: true,
            search_workspace: true,
            get_spec_document: true,
            validate: binding.capabilities.validate,
            analyze: binding.capabilities.analyze,
            delete_change: binding.capabilities.delete_change,
            move_task: binding.capabilities.move_task,
            reorder_card: binding.capabilities.policy_write,
            change_meta: true,
            change_capabilities: true,
            claim: binding.capabilities.delete_change,
            policy_write: binding.capabilities.policy_write,
            live_updates,
        }
    }
}

/// `/config` 經 desktop-core 文字 seam 後交給前端的設定快照。remote 無
/// `.speclink.yaml` 面，仍保留空 app 欄位以沿用 SettingsSnapshot 的穩定形狀。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSettingsSnapshot {
    pub app: AppSettings,
    pub workflow: RemoteWorkflowSettings,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteWorkflowSettings {
    #[serde(flatten)]
    pub settings: WorkflowSettings,
    pub revision: u64,
}

/// Tauri 保留 remote protocol 的 machine-readable reason/status，讓前端可將
/// revision_conflict 與一般錯誤分流；message 仍是使用者可讀單行文字。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSettingsError {
    pub message: String,
    pub reason: Option<String>,
    pub status: Option<u16>,
}

impl RemoteSettingsError {
    pub fn local(message: impl Into<String>) -> RemoteSettingsError {
        RemoteSettingsError {
            message: message.into(),
            reason: Some("invalid_config".to_string()),
            status: None,
        }
    }

    pub fn command(message: impl Into<String>) -> RemoteSettingsError {
        RemoteSettingsError {
            message: message.into(),
            reason: None,
            status: None,
        }
    }
}

impl From<RemoteError> for RemoteSettingsError {
    fn from(error: RemoteError) -> Self {
        RemoteSettingsError {
            message: error.message,
            reason: error.reason,
            status: error.status,
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
    let project_base = format!(
        "{}/api/speclink/v1/projects/{project}",
        origin.trim_end_matches('/')
    );
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

/// chooser 在選定 project/repo 前讀取登入者可見 scopes。此請求只有 bearer，
/// 不攜帶 fabricated repo header；membership 過濾由 server 負責。
pub fn list_scopes(
    origin: &str,
    manager: &Arc<TokenManager>,
    credentials: &dyn CredentialStore,
) -> Result<ScopesResponse, RemoteError> {
    let api_root = format!("{}/api/speclink/v1", origin.trim_end_matches('/'));
    manager.execute(credentials, |token| {
        Client::new(&api_root, token, None).list_scopes()
    })
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

    fn run_write<T>(
        &self,
        credentials: &dyn CredentialStore,
        call: impl Fn(&Client) -> Result<T, RemoteError>,
    ) -> Result<T, RemoteError> {
        self.manager.execute_write(credentials, |token| {
            call(&Client::new(&self.project_base, token, Some(&self.repo)))
        })
    }

    /// 清單同時取 board resource 於 Rust 側合併排序（決策 4）：UI 與 TS 層
    /// 不做排序、不知道 board resource 存在。board resource 讀取失敗（端點
    /// 缺席的舊 server）視為缺席——回退序照常渲染。
    pub fn list_changes(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<ListChangesResponse, RemoteError> {
        let mut response = self.run(credentials, |client| client.list_changes())?;
        let board = self.board_order_doc(credentials);
        overlay_changes_order(&mut response.changes, &board);
        Ok(response)
    }

    /// 讀 board resource 並解析為桌面側形狀；任何讀取失敗與缺席同義
    /// （決策 6：爆炸半徑＝退回預設序）。
    fn board_order_doc(&self, credentials: &dyn CredentialStore) -> BoardOrderDoc {
        self.run(credentials, |client| client.board_order())
            .ok()
            .map(|response| parse_board_order(response.content.as_deref()))
            .unwrap_or_default()
    }

    pub fn list_specs(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<ListSpecsResponse, RemoteError> {
        self.run(credentials, |client| client.list_specs())
    }

    /// GET `/config` 的文件原文一律經 desktop-core from-text seam；revision
    /// 與解析結果同一份 response snapshot，避免欄位與 CAS token 分裂。
    pub fn read_settings(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<RemoteSettingsSnapshot, RemoteSettingsError> {
        let config = self
            .run(credentials, |client| client.config())
            .map_err(RemoteSettingsError::from)?;
        Ok(RemoteSettingsSnapshot {
            app: AppSettings {
                tools: Vec::new(),
                custom_tools: Vec::new(),
                parse_error: None,
            },
            workflow: RemoteWorkflowSettings {
                settings: read_workflow_settings_from_text(config.content.as_deref()),
                revision: config.revision,
            },
        })
    }

    /// 政策四欄位 targeted rewrite：GET 全文 → 共用文字 seam → PUT 全文＋
    /// 畫面讀得的 expected revision。PUT 仍由 server 做 role／驗證／CAS。
    pub fn write_workflow_fields(
        &self,
        credentials: &dyn CredentialStore,
        fields: &WorkflowPolicyFields,
        expected_revision: u64,
    ) -> Result<u64, RemoteSettingsError> {
        self.manager
            .ensure_write_allowed()
            .map_err(RemoteSettingsError::from)?;
        let config = self
            .run(credentials, |client| client.config())
            .map_err(RemoteSettingsError::from)?;
        let rewritten =
            rewrite_workflow_fields_text(config.content.as_deref().unwrap_or_default(), fields)
                .map_err(RemoteSettingsError::local)?;
        self.run_write(credentials, |client| {
            client.put_config(&rewritten, expected_revision)
        })
        .map(|response| response.revision)
        .map_err(RemoteSettingsError::from)
    }

    /// schema 鍵 targeted rewrite（desktop-schema-panel design D2）：GET 全文 →
    /// 引擎 byte-preserving setter → PUT 全文＋expected revision。與政策欄位
    /// 寫入同一條 revision 守門通道，壞檔在 setter 即拒。
    pub fn write_workflow_schema(
        &self,
        credentials: &dyn CredentialStore,
        name: &str,
        expected_revision: u64,
    ) -> Result<u64, RemoteSettingsError> {
        self.manager
            .ensure_write_allowed()
            .map_err(RemoteSettingsError::from)?;
        let config = self
            .run(credentials, |client| client.config())
            .map_err(RemoteSettingsError::from)?;
        let rewritten =
            speclink_core::config::set_workflow_schema_text(config.content.as_deref(), name)
                .map_err(|e| RemoteSettingsError::local(e.to_string()))?;
        self.run_write(credentials, |client| {
            client.put_config(&rewritten, expected_revision)
        })
        .map(|response| response.revision)
        .map_err(RemoteSettingsError::from)
    }

    /// context/rules targeted rewrite；None 表示不觸及該鍵，Some 空值沿用 seam
    /// 的移除語意。沒有任何省略 expected revision 的寫入路徑。
    pub fn write_workflow_content(
        &self,
        credentials: &dyn CredentialStore,
        context: Option<&str>,
        rules: Option<&[(String, Vec<String>)]>,
        expected_revision: u64,
    ) -> Result<u64, RemoteSettingsError> {
        self.manager
            .ensure_write_allowed()
            .map_err(RemoteSettingsError::from)?;
        let config = self
            .run(credentials, |client| client.config())
            .map_err(RemoteSettingsError::from)?;
        let context_edit = context
            .map(|value| ContextEdit::Set(value.to_string()))
            .unwrap_or(ContextEdit::Keep);
        let rewritten = rewrite_workflow_content_text(
            config.content.as_deref().unwrap_or_default(),
            &context_edit,
            rules,
        )
        .map_err(RemoteSettingsError::local)?;
        self.run_write(credentials, |client| {
            client.put_config(&rewritten, expected_revision)
        })
        .map(|response| response.revision)
        .map_err(RemoteSettingsError::from)
    }

    pub fn list_archived(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<ArchivedListResponse, RemoteError> {
        self.run(credentials, |client| client.archived_list())
    }

    pub fn spec_document(
        &self,
        credentials: &dyn CredentialStore,
        capability: &str,
    ) -> Result<SpecDocumentResponse, RemoteError> {
        self.run(credentials, |client| client.spec_document(capability))
    }

    pub fn search_workspace(
        &self,
        credentials: &dyn CredentialStore,
        query: &str,
    ) -> Result<SearchResponse, RemoteError> {
        self.run(credentials, |client| client.search(query))
    }

    pub fn archived_document(
        &self,
        credentials: &dyn CredentialStore,
        dated_name: &str,
        artifact: &str,
    ) -> Result<SpecDocumentResponse, RemoteError> {
        self.run(credentials, |client| {
            client.archived_artifact(dated_name, artifact)
        })
    }

    pub fn archived_capabilities(
        &self,
        credentials: &dyn CredentialStore,
        dated_name: &str,
    ) -> Result<Vec<String>, RemoteError> {
        self.run(credentials, |client| {
            client.archived_capabilities(dated_name)
        })
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
        self.run_write(credentials, |client| {
            if done {
                client.task_done(change, task, &[], None).map(|_| ())
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
        self.run_write(credentials, |client| client.claim(change))
    }

    /// 退回提案中:打 DELETE /changes/{name}/in-progress。200 Ack 涵蓋實際
    /// 移除與未開工冪等;守門 409 的證據落在 RemoteError::evidence,由
    /// command 層轉為與本地 bridge 同形狀的結構化錯誤 JSON。
    pub fn revert_change_to_proposed(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<(), RemoteError> {
        // 回應的 removed 旗標 desktop 用不到——看板靠重讀清單刷新，不看這句回報。
        self.run_write(credentials, |client| client.in_progress_remove(change)).map(|_| ())
    }

    /// validate 動詞（唯讀衍生查詢）：wire DTO 轉回引擎型別，序列化後與本地
    /// verbs::validate_at 的 payload 同形（remote-verb-parity）。
    pub fn validate(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<speclink_core::validate::ValidationResult, RemoteError> {
        self.run(credentials, |client| client.validate_change(change))
            .map(speclink_remote::convert::validation_result)
    }

    /// analyze 動詞（唯讀衍生查詢）：payload 與本地 verbs::analyze_at 同形。
    pub fn analyze(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<speclink_core::analyzer::AnalyzeReport, RemoteError> {
        self.run(credentials, |client| client.analyze_change(change))
            .map(speclink_remote::convert::analyze_report)
    }

    /// 刪除變更＝server 端 discard 全語意（guard/unlink/原子刪除）；force 由
    /// 呼叫端決定（桌面固定 true——決策 3：與本地無 guard 直刪同模式）。
    pub fn delete_change(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        force: bool,
    ) -> Result<(), RemoteError> {
        self.run_write(credentials, |client| client.discard(change, force))
            .map(|_| ())
    }

    /// 任務搬移：index 定址直達 server 端點，重編號效果與本地拖排一致。
    pub fn move_task(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        from: usize,
        to: usize,
        before: Option<bool>,
    ) -> Result<(), RemoteError> {
        self.run_write(credentials, |client| client.move_task(change, from, to, before))
            .map(|_| ())
    }

    pub fn archive(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
    ) -> Result<ArchiveResponse, RemoteError> {
        // 兩個 carry 旗標恆 false：remote 清單項不帶 reviewStatus／verifyStatus，
        // UI 不會判到 inReview／inVerify、封存三選項對話框在 remote 不出現
        //（design「Out of scope」）。
        self.run_write(credentials, |client| client.archive(change, false, false))
    }

    /// active 討論同樣疊 board resource 排序 overlay（決策 4）；archived 清單
    /// 不屬看板、維持 server 回傳序。
    pub fn list_discussions(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<DiscussionLists, RemoteError> {
        let active = self.run(credentials, |client| client.list_discussions(false))?;
        let archived = self.run(credentials, |client| client.list_discussions(true))?;
        let mut active = active.discussions;
        let board = self.board_order_doc(credentials);
        overlay_discussions_order(&mut active, &board);
        Ok(DiscussionLists {
            active,
            archived: archived.discussions,
        })
    }

    /// 看板拖排直達（決策 5）：讀清單＋board resource → 補章／中點 → PUT 全文
    /// 帶 If-Match；409 重讀重算重試恰一次，再敗原樣回錯（前端刷新 server
    /// 現況，不留假象順序）。全程不觸碰任何卡片 meta／frontmatter。
    pub fn reorder_card(
        &self,
        credentials: &dyn CredentialStore,
        kind: &str,
        id: &str,
        prev_id: Option<&str>,
        next_id: Option<&str>,
    ) -> Result<(), RemoteError> {
        reorder_via(
            || {
                let changes = self
                    .run(credentials, |client| client.list_changes())?
                    .changes;
                let discussions = self
                    .run(credentials, |client| client.list_discussions(false))?
                    .discussions;
                let board = self.run(credentials, |client| client.board_order())?;
                Ok(BoardSnapshot {
                    changes,
                    discussions,
                    content: board.content,
                    revision: board.revision,
                })
            },
            |content, revision| {
                self.run_write(credentials, |client| client.put_board_order(content, revision))
                    .map(|_| ())
            },
            kind,
            id,
            prev_id,
            next_id,
        )
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
        self.run_write(credentials, |client| client.discussion_promote(slug, name))
    }

    pub fn archive_discussion(
        &self,
        credentials: &dyn CredentialStore,
        slug: &str,
    ) -> Result<ArchiveDiscussionResponse, RemoteError> {
        self.run_write(credentials, |client| client.discussion_archive(slug))
    }

    /// (b) 組合：逐任務寫回，非原子——中途失敗即中止並回報已完成筆數。
    pub fn set_tasks(
        &self,
        credentials: &dyn CredentialStore,
        change: &str,
        task_ids: &[String],
        done: bool,
    ) -> Result<usize, SetTasksFailure> {
        self.manager
            .ensure_write_allowed()
            .map_err(|error| SetTasksFailure {
                completed: 0,
                error,
            })?;
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
        self.manager
            .ensure_write_allowed()
            .map_err(|error| SetTasksFailure {
                completed: 0,
                error,
            })?;
        let instructions = self
            .run(credentials, |client| client.apply_instructions(change))
            .map_err(|error| SetTasksFailure {
                completed: 0,
                error,
            })?;
        let pending: Vec<String> = instructions
            .tasks
            .into_iter()
            .filter(|task| task.done != done)
            .map(|task| task.id)
            .collect();
        self.set_tasks(credentials, change, &pending, done)
    }
}

#[cfg(test)]
mod capability_tests {
    use super::RemoteCapabilities;
    use speclink_protocol::binding::{Actor, BindingResponse, Capabilities, ScopeRef};

    fn binding_with(capabilities: Capabilities) -> BindingResponse {
        let scope = |key: &str| ScopeRef {
            id: format!("id_{key}"),
            key: key.to_string(),
            name: key.to_string(),
        };
        BindingResponse {
            actor: Actor { id: "u_1".to_string(), name: "Tester".to_string() },
            project: scope("demo"),
            repo: scope("backend"),
            api_version: "1".to_string(),
            engine_version: "0.1.0".to_string(),
            capabilities,
        }
    }

    #[test]
    fn editor_handshake_unlocks_all_write_capabilities_including_board_reorder() {
        // 規格「capability 驅動停用且不偽造缺口」修訂後語意：editor 全操作面
        // 直達，停用清單清空（remote-board-order 決策 7）。
        let caps = RemoteCapabilities::from_binding(&binding_with(Capabilities {
            validate: true,
            analyze: true,
            delete_change: true,
            move_task: true,
            policy_write: true,
            ..Default::default()
        }));
        assert!(caps.validate, "validate follows the handshake");
        assert!(caps.analyze, "analyze follows the handshake");
        assert!(caps.delete_change, "deleteChange follows the handshake");
        assert!(caps.move_task, "moveTask follows the handshake");
        assert!(caps.reorder_card, "board reorder follows the editor role");
    }

    #[test]
    fn reader_handshake_keeps_write_verbs_disabled_but_derived_queries_on() {
        let caps = RemoteCapabilities::from_binding(&binding_with(Capabilities {
            validate: true,
            analyze: true,
            delete_change: false,
            move_task: false,
            ..Default::default()
        }));
        assert!(caps.validate, "reader may run the read-only derived queries");
        assert!(caps.analyze);
        assert!(!caps.delete_change, "reader write verbs stay disabled");
        assert!(!caps.move_task);
        assert!(!caps.reorder_card, "reader board reorder stays disabled");
    }
}

#[cfg(test)]
mod board_order_tests {
    use super::{overlay_changes_order, overlay_discussions_order, parse_board_order};
    use speclink_protocol::query::{ChangeSummary, DiscussionInfo};

    pub(super) fn change(name: &str, completed: usize, total: usize) -> ChangeSummary {
        ChangeSummary {
            name: name.into(),
            summary: String::new(),
            status: "in-progress".into(),
            completed_tasks: completed,
            total_tasks: total,
            restale_from: Vec::new(),
            meta_error: None,
            repo: None,
            lifecycle: None,
            claimed_by: None,
            started_at: None,
            created_by: None,
            created: None,
            from_discussions: Vec::new(),
        }
    }

    pub(super) fn discussion(slug: &str) -> DiscussionInfo {
        DiscussionInfo {
            slug: slug.into(),
            topic: slug.to_uppercase(),
            status: "open".into(),
            rounds: 1,
            created: "2026-01-02".into(),
            created_by: None,
            kind: None,
            promoted_to: Vec::new(),
            concluded: None,
            path: format!("openspec/discussions/{slug}.md"),
            archived: false,
        }
    }

    fn names(items: &[ChangeSummary]) -> Vec<&str> {
        items.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn ranked_changes_sort_ascending_with_unranked_on_top_in_server_order() {
        // 規格「remote 排序 overlay 與本地語意同構」＋ board-card-order
        // Example「四卡混排」：W(b)、X(f)、Y(n)、Z(無 rank) → Z、W、X、Y。
        let board = parse_board_order(Some(
            "{\"changes\":{\"w\":\"b\",\"x\":\"f\",\"y\":\"n\"},\"discussions\":{}}",
        ));
        let mut items = vec![change("y", 0, 2), change("x", 0, 2), change("z", 0, 2), change("w", 0, 2)];
        overlay_changes_order(&mut items, &board);
        assert_eq!(names(&items), ["z", "w", "x", "y"]);
    }

    #[test]
    fn equal_ranks_break_ties_by_name() {
        // board-card-order Example「同值以名稱決斷」：beta 與 alpha 同 rank n
        // → alpha 在前，跨機器確定。
        let board = parse_board_order(Some(
            "{\"changes\":{\"beta\":\"n\",\"alpha\":\"n\"},\"discussions\":{}}",
        ));
        let mut items = vec![change("beta", 0, 2), change("alpha", 0, 2)];
        overlay_changes_order(&mut items, &board);
        assert_eq!(names(&items), ["alpha", "beta"]);
    }

    #[test]
    fn multiple_unranked_keep_the_server_order_among_themselves() {
        let board = parse_board_order(Some("{\"changes\":{\"m\":\"n\"},\"discussions\":{}}"));
        let mut items = vec![change("delta", 0, 2), change("bravo", 0, 2), change("m", 0, 2)];
        overlay_changes_order(&mut items, &board);
        assert_eq!(names(&items), ["delta", "bravo", "m"], "缺 rank 卡維持 server 回傳序置頂");
    }

    #[test]
    fn absent_board_resource_leaves_the_server_order_untouched() {
        // 規格 Scenario「無 board resource 時行為不變」：逐項一致。
        let mut items = vec![change("c", 1, 2), change("a", 0, 2), change("b", 2, 2)];
        let before = names(&items).into_iter().map(String::from).collect::<Vec<_>>();
        let board = parse_board_order(None);
        overlay_changes_order(&mut items, &board);
        assert_eq!(names(&items), before.iter().map(String::as_str).collect::<Vec<_>>());
    }

    #[test]
    fn corrupt_board_resource_is_treated_as_all_unranked() {
        // 規格 Scenario「壞文件退回回退序」＋ design 決策 6：非法 JSON／非預期
        // 形狀＝全員缺 rank，照常渲染不失效。
        for corrupt in ["not json at all {{{", "[1,2,3]", "{\"changes\":\"not-a-map\"}"] {
            let mut items = vec![change("c", 0, 2), change("a", 0, 2)];
            let board = parse_board_order(Some(corrupt));
            overlay_changes_order(&mut items, &board);
            assert_eq!(names(&items), ["c", "a"], "corrupt content {corrupt:?} falls back");
        }
    }

    #[test]
    fn discussions_overlay_follows_the_same_semantics() {
        let board = parse_board_order(Some(
            "{\"changes\":{},\"discussions\":{\"delta\":\"b\",\"charlie\":\"n\",\"echo\":\"n\"}}",
        ));
        let mut items = vec![
            discussion("charlie"),
            discussion("zulu"),
            discussion("echo"),
            discussion("delta"),
        ];
        overlay_discussions_order(&mut items, &board);
        let slugs: Vec<&str> = items.iter().map(|i| i.slug.as_str()).collect();
        // zulu 缺 rank 置頂；delta(b) < charlie(n)==echo(n) 同值以 slug 決斷。
        assert_eq!(slugs, ["zulu", "delta", "charlie", "echo"]);
    }
}

#[cfg(test)]
mod board_reorder_tests {
    use super::board_order_tests::{change, discussion};
    use super::{parse_board_order, reorder_via, BoardOrderDoc, BoardSnapshot};
    use speclink_remote::RemoteError;
    use std::cell::{Cell, RefCell};

    fn conflict() -> RemoteError {
        RemoteError {
            message: "revision 衝突".into(),
            reason: Some("revision_conflict".into()),
            status: Some(409),
            evidence: None,
        }
    }

    /// 依 snapshot 產生 read/put 假 IO；記錄每次 put 的 (content, revision)。
    struct FakeIo {
        snapshots: RefCell<Vec<BoardSnapshot>>,
        reads: Cell<usize>,
        puts: RefCell<Vec<(String, u64)>>,
        /// 依呼叫序回應 put：true＝成功、false＝409。
        put_plan: Vec<bool>,
    }

    impl FakeIo {
        fn new(snapshots: Vec<BoardSnapshot>, put_plan: Vec<bool>) -> FakeIo {
            FakeIo {
                snapshots: RefCell::new(snapshots),
                reads: Cell::new(0),
                puts: RefCell::new(Vec::new()),
                put_plan,
            }
        }

        fn run(
            &self,
            kind: &str,
            id: &str,
            prev: Option<&str>,
            next: Option<&str>,
        ) -> Result<(), RemoteError> {
            reorder_via(
                || {
                    let index = self.reads.get();
                    self.reads.set(index + 1);
                    let snapshots = self.snapshots.borrow();
                    Ok(snapshots[index.min(snapshots.len() - 1)].clone())
                },
                |content, revision| {
                    let attempt = self.puts.borrow().len();
                    self.puts.borrow_mut().push((content.to_string(), revision));
                    if self.put_plan[attempt] {
                        Ok(())
                    } else {
                        Err(conflict())
                    }
                },
                kind,
                id,
                prev,
                next,
            )
        }
    }

    fn snapshot(
        changes: Vec<speclink_protocol::query::ChangeSummary>,
        discussions: Vec<speclink_protocol::query::DiscussionInfo>,
        content: Option<&str>,
        revision: u64,
    ) -> BoardSnapshot {
        BoardSnapshot {
            changes,
            discussions,
            content: content.map(String::from),
            revision,
        }
    }

    #[test]
    fn unranked_column_is_backfilled_into_the_board_resource_only() {
        // design 決策 5：欄內缺 rank → 依當前顯示序整欄補章，等距鍵只落在
        // board resource 圖內；落點取鄰居中點。
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("a", 0, 4), change("b", 0, 4), change("c", 0, 4)],
                vec![],
                None,
                5,
            )],
            vec![true],
        );
        io.run("change", "c", Some("a"), Some("b")).expect("reorder lands");

        let puts = io.puts.borrow();
        assert_eq!(puts.len(), 1, "恰一次 PUT 全文");
        let (content, revision) = &puts[0];
        assert_eq!(*revision, 5, "If-Match 帶讀到的 scope revision");
        let doc = parse_board_order(Some(content));
        let (a, b, c) = (&doc.changes["a"], &doc.changes["b"], &doc.changes["c"]);
        assert!(a < c && c < b, "c 落在 a 與 b 之間：{a} < {c} < {b}");
        assert!(doc.discussions.is_empty(), "補章只涵蓋被拖卡所在欄");
    }

    #[test]
    fn backfill_covers_only_the_dragged_cards_column() {
        // 欄推導與前端 changeStage 同構：proposed(0) 與 ready(全完成) 分欄，
        // 拖 proposed 卡不為 ready 卡補章。
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("done", 4, 4), change("a", 0, 4), change("b", 0, 4)],
                vec![],
                None,
                3,
            )],
            vec![true],
        );
        io.run("change", "b", None, Some("a")).expect("reorder lands");
        let puts = io.puts.borrow();
        let doc = parse_board_order(Some(&puts[0].0));
        assert!(!doc.changes.contains_key("done"), "他欄的卡不被補章觸碰");
        assert!(doc.changes["b"] < doc.changes["a"], "b 落在 a 之前（欄頂）");
    }

    #[test]
    fn vanished_neighbors_are_open_ends_and_inverted_ranks_drop_the_upper_bound() {
        // 規格「拖排寫回以全文 CAS 與一次重試收斂」：消失的鄰居視開放端、
        // 鄰居現值逆序時棄上界保底——沿本地 neighbor_midpoint 語意。
        let board = "{\"changes\":{\"a\":\"f\",\"b\":\"n\",\"drag\":\"t\"},\"discussions\":{}}";
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("a", 0, 4), change("b", 0, 4), change("drag", 0, 4)],
                vec![],
                Some(board),
                7,
            )],
            vec![true],
        );
        // prev 指向已消失的卡（開放端＝欄頂）、next 為 a(f) → key < f。
        io.run("change", "drag", Some("ghost"), Some("a")).expect("reorder lands");
        let doc = parse_board_order(Some(&io.puts.borrow()[0].0));
        assert!(doc.changes["drag"].as_str() < "f", "消失的 prev 視為欄頂開放端");

        // 逆序鄰居（prev=b(n)、next=a(f)，n > f）→ 棄上界：key > n。
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("a", 0, 4), change("b", 0, 4), change("drag", 0, 4)],
                vec![],
                Some(board),
                7,
            )],
            vec![true],
        );
        io.run("change", "drag", Some("b"), Some("a")).expect("reorder lands");
        let doc = parse_board_order(Some(&io.puts.borrow()[0].0));
        assert!(doc.changes["drag"].as_str() > "n", "逆序鄰居棄上界保底");
    }

    #[test]
    fn entries_absent_from_the_current_lists_are_pruned_on_rewrite() {
        // 規格 Scenario「已封存卡的條目被修剪」：PUT 全文不含不在現行清單
        // 的條目——changes 與 discussions 兩段圖皆修剪。
        let board = "{\"changes\":{\"gone\":\"d\",\"a\":\"f\",\"b\":\"n\"},\
                     \"discussions\":{\"stale\":\"g\",\"topic\":\"m\"}}";
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("a", 0, 4), change("b", 0, 4)],
                vec![discussion("topic")],
                Some(board),
                9,
            )],
            vec![true],
        );
        io.run("change", "a", Some("b"), None).expect("reorder lands");
        let doc = parse_board_order(Some(&io.puts.borrow()[0].0));
        assert!(!doc.changes.contains_key("gone"), "封存變更的條目被修剪");
        assert!(!doc.discussions.contains_key("stale"), "孤兒討論條目一併修剪");
        assert!(doc.discussions.contains_key("topic"), "現行清單內的條目保留");
    }

    #[test]
    fn a_conflict_rereads_recomputes_and_retries_exactly_once() {
        // 規格 Scenario「409 重讀後落位」：重讀 → 重算 → 重試恰一次成功，
        // 第二次 PUT 帶新 revision 且落點基於他人已寫入的新 board。
        let first = snapshot(
            vec![change("a", 0, 4), change("b", 0, 4), change("drag", 0, 4)],
            vec![],
            Some("{\"changes\":{\"a\":\"f\",\"b\":\"n\",\"drag\":\"t\"},\"discussions\":{}}"),
            5,
        );
        let second = snapshot(
            vec![change("a", 0, 4), change("b", 0, 4), change("drag", 0, 4)],
            vec![],
            Some("{\"changes\":{\"a\":\"c\",\"b\":\"x\",\"drag\":\"t\"},\"discussions\":{}}"),
            8,
        );
        let io = FakeIo::new(vec![first, second], vec![false, true]);
        io.run("change", "drag", Some("a"), Some("b")).expect("retry lands");

        assert_eq!(io.reads.get(), 2, "409 後重讀一次");
        let puts = io.puts.borrow();
        assert_eq!(puts.len(), 2, "重試恰一次");
        assert_eq!(puts[1].1, 8, "重試帶重讀後的新 revision");
        let doc = parse_board_order(Some(&puts[1].0));
        let key = doc.changes["drag"].as_str();
        assert!("c" < key && key < "x", "落點基於重讀後的鄰居現值：c < {key} < x");
    }

    #[test]
    fn a_second_conflict_surfaces_the_error_without_a_third_attempt() {
        // 規格 Scenario「重試仍敗不留假象」：再敗回錯誤（機器可判），呼叫端
        // 據此刷新 server 現況；絕不第三次嘗試。
        let base = snapshot(
            vec![change("a", 0, 4), change("drag", 0, 4)],
            vec![],
            None,
            5,
        );
        let io = FakeIo::new(vec![base.clone(), base], vec![false, false]);
        let error = io.run("change", "drag", Some("a"), None).expect_err("both attempts conflict");
        assert_eq!(error.reason.as_deref(), Some("revision_conflict"));
        assert_eq!(io.puts.borrow().len(), 2, "重試上限恰一次，無第三次 PUT");
    }

    #[test]
    fn a_dragged_card_missing_from_the_list_is_an_error_without_any_put() {
        let io = FakeIo::new(
            vec![snapshot(vec![change("a", 0, 4)], vec![], None, 5)],
            vec![true],
        );
        io.run("change", "ghost", None, None).expect_err("unknown card is refused");
        assert!(io.puts.borrow().is_empty(), "查無此卡不產生任何寫入");
    }

    #[test]
    fn discussions_reorder_writes_the_discussions_map() {
        let io = FakeIo::new(
            vec![snapshot(
                vec![],
                vec![discussion("alpha"), discussion("beta"), discussion("gamma")],
                None,
                4,
            )],
            vec![true],
        );
        io.run("discussion", "gamma", Some("alpha"), Some("beta")).expect("reorder lands");
        let doc = parse_board_order(Some(&io.puts.borrow()[0].0));
        let (a, b, g) = (
            &doc.discussions["alpha"],
            &doc.discussions["beta"],
            &doc.discussions["gamma"],
        );
        assert!(a < g && g < b, "gamma 落在 alpha 與 beta 之間");
        assert!(doc.changes.is_empty(), "變更圖不被討論拖排觸碰");
    }

    #[test]
    fn corrupt_board_content_is_rebuilt_by_the_next_reorder() {
        // 規格 Scenario「壞文件退回回退序」後半：下一次成功拖排重建合法文件。
        let io = FakeIo::new(
            vec![snapshot(
                vec![change("a", 0, 4), change("b", 0, 4)],
                vec![],
                Some("not json at all {{{"),
                6,
            )],
            vec![true],
        );
        io.run("change", "b", None, Some("a")).expect("reorder lands");
        let text = &io.puts.borrow()[0].0;
        let doc: BoardOrderDoc = serde_json::from_str(text).expect("重建後為合法 JSON 形狀");
        assert!(doc.changes["b"] < doc.changes["a"]);
    }
}

#[cfg(test)]
mod migration_bundle_tests {
    use super::{adopt_remote_workspace_with, build_import_bundle, finalize_local_migration_with};
    use speclink_protocol::query::ImportDocumentId;
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::TempDir;

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture parent")).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn complete_workspace() -> TempDir {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "openspec/config.yaml",
            "schema: spec-driven\nlocale: tw\n",
        );
        write(root.path(), "openspec/LANGUAGE.md", "TeamStore: 團隊儲存\n");
        write(
            root.path(),
            "openspec/changes/active-change/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-21\n",
        );
        write(
            root.path(),
            "openspec/changes/active-change/proposal.md",
            "## Why\n\nActive proposal.\n",
        );
        write(
            root.path(),
            "openspec/changes/active-change/design.md",
            "## Context\n\nActive design.\n",
        );
        write(
            root.path(),
            "openspec/changes/active-change/tasks.md",
            "- [ ] 1.1 Active task\n",
        );
        write(
            root.path(),
            "openspec/changes/active-change/specs/payments/spec.md",
            "## ADDED Requirements\n\n### Requirement: Active payment\n",
        );
        write(
            root.path(),
            "openspec/specs/accounts/spec.md",
            "# accounts Specification\n\nCanonical accounts.\n",
        );
        write(
            root.path(),
            "openspec/discussions/live-plan.md",
            "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n",
        );
        write(
            root.path(),
            "openspec/discussions/archive/2026-07-20-old-plan.md",
            "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nArchived discussion.\n",
        );
        write(
            root.path(),
            "openspec/changes/archive/2026-07-20-old-change/.openspec.yaml",
            "schema: spec-driven\ncreated: 2026-07-20\n",
        );
        write(
            root.path(),
            "openspec/changes/archive/2026-07-20-old-change/proposal.md",
            "## Why\n\nArchived proposal.\n",
        );
        write(
            root.path(),
            "openspec/changes/archive/2026-07-20-old-change/tasks.md",
            "- [x] 1.1 Archived task\n",
        );
        write(
            root.path(),
            "openspec/changes/archive/2026-07-20-old-change/specs/payments/spec.md",
            "## ADDED Requirements\n\n### Requirement: Archived payment\n",
        );
        root
    }

    #[test]
    fn bundle_contains_every_local_workspace_document_with_exact_content() {
        let root = complete_workspace();
        let bundle = build_import_bundle(root.path(), "demo", "backend")
            .expect("complete local workspace builds");
        assert_eq!(bundle.format_version, 1);
        assert_eq!(bundle.scope.project, "demo");
        assert_eq!(bundle.scope.repo, "backend");
        assert_eq!(bundle.project_revision, 0);

        let documents: BTreeMap<ImportDocumentId, String> = bundle
            .documents
            .into_iter()
            .map(|document| (document.document, document.content))
            .collect();
        let expected = BTreeMap::from([
            (
                ImportDocumentId::ChangeMeta {
                    change: "active-change".into(),
                },
                "schema: spec-driven\ncreated: 2026-07-21\n".into(),
            ),
            (
                ImportDocumentId::ChangeArtifact {
                    change: "active-change".into(),
                    artifact: "proposal.md".into(),
                },
                "## Why\n\nActive proposal.\n".into(),
            ),
            (
                ImportDocumentId::ChangeArtifact {
                    change: "active-change".into(),
                    artifact: "design.md".into(),
                },
                "## Context\n\nActive design.\n".into(),
            ),
            (
                ImportDocumentId::ChangeArtifact {
                    change: "active-change".into(),
                    artifact: "tasks.md".into(),
                },
                "- [ ] 1.1 Active task\n".into(),
            ),
            (
                ImportDocumentId::ChangeArtifact {
                    change: "active-change".into(),
                    artifact: "specs/payments/spec.md".into(),
                },
                "## ADDED Requirements\n\n### Requirement: Active payment\n".into(),
            ),
            (
                ImportDocumentId::CanonicalSpec {
                    capability: "accounts".into(),
                },
                "# accounts Specification\n\nCanonical accounts.\n".into(),
            ),
            (
                ImportDocumentId::Discussion {
                    slug: "live-plan".into(),
                    archived: false,
                },
                "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n".into(),
            ),
            (
                ImportDocumentId::Discussion {
                    slug: "old-plan".into(),
                    archived: true,
                },
                "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nArchived discussion.\n".into(),
            ),
            (
                ImportDocumentId::ArchivedChange {
                    change: "2026-07-20-old-change".into(),
                    doc: ".openspec.yaml".into(),
                },
                "schema: spec-driven\ncreated: 2026-07-20\n".into(),
            ),
            (
                ImportDocumentId::ArchivedChange {
                    change: "2026-07-20-old-change".into(),
                    doc: "proposal.md".into(),
                },
                "## Why\n\nArchived proposal.\n".into(),
            ),
            (
                ImportDocumentId::ArchivedChange {
                    change: "2026-07-20-old-change".into(),
                    doc: "tasks.md".into(),
                },
                "- [x] 1.1 Archived task\n".into(),
            ),
            (
                ImportDocumentId::ArchivedChange {
                    change: "2026-07-20-old-change".into(),
                    doc: "specs/payments/spec.md".into(),
                },
                "## ADDED Requirements\n\n### Requirement: Archived payment\n".into(),
            ),
            (
                ImportDocumentId::WorkflowConfig,
                "schema: spec-driven\nlocale: tw\n".into(),
            ),
            (
                ImportDocumentId::Language,
                "TeamStore: 團隊儲存\n".into(),
            ),
        ]);
        assert_eq!(documents, expected);
    }

    #[test]
    fn corrupt_change_metadata_aborts_the_bundle_and_names_the_file() {
        let root = complete_workspace();
        write(
            root.path(),
            "openspec/changes/active-change/.openspec.yaml",
            "schema: [unterminated\n",
        );

        let error = build_import_bundle(root.path(), "demo", "backend")
            .expect_err("corrupt metadata must fail closed");
        assert!(
            error.contains("openspec/changes/active-change/.openspec.yaml"),
            "error names the corrupt metadata file: {error}"
        );
    }

    #[test]
    fn successful_conversion_renames_the_local_tree_before_writing_the_marker() {
        let root = complete_workspace();
        let marker = root.path().join(".speclink.yaml");
        let backup = finalize_local_migration_with(
            root.path(),
            "https://spec.example.test",
            "demo",
            "backend",
            "2026-07-21",
            |from, to| {
                assert!(from.is_dir(), "local truth exists until the rename");
                assert!(!marker.exists(), "marker is not written before the backup");
                std::fs::rename(from, to)
            },
        )
        .expect("conversion succeeds");

        assert_eq!(backup, root.path().join("openspec.migrated-2026-07-21"));
        assert!(!root.path().join("openspec").exists());
        assert!(backup.join("changes/active-change/proposal.md").is_file());
        let marker = std::fs::read_to_string(marker).expect("remote marker");
        assert!(marker.contains("https://spec.example.test/api/speclink/v1/projects/demo"));
        assert!(marker.contains("repo: backend"));
    }

    #[test]
    fn an_existing_dated_backup_uses_the_next_available_sequence() {
        let root = complete_workspace();
        std::fs::create_dir(root.path().join("openspec.migrated-2026-07-21")).unwrap();
        std::fs::create_dir(root.path().join("openspec.migrated-2026-07-21-2")).unwrap();

        let backup = finalize_local_migration_with(
            root.path(),
            "https://spec.example.test",
            "demo",
            "backend",
            "2026-07-21",
            |from, to| std::fs::rename(from, to),
        )
        .expect("conversion succeeds");
        assert_eq!(backup, root.path().join("openspec.migrated-2026-07-21-3"));
        assert!(backup.is_dir());
    }

    #[test]
    fn adopting_server_truth_only_renames_local_truth_and_preserves_the_marker() {
        let root = complete_workspace();
        let marker = "remote:\n  url: https://spec.example.test/api/speclink/v1/projects/demo\n  repo: backend\n";
        write(root.path(), ".speclink.yaml", marker);

        let backup = adopt_remote_workspace_with(root.path(), "2026-07-21", |from, to| {
            std::fs::rename(from, to)
        })
        .expect("server truth adoption succeeds");

        assert_eq!(backup, root.path().join("openspec.migrated-2026-07-21"));
        assert!(!root.path().join("openspec").exists());
        assert!(backup.join("changes/active-change/proposal.md").is_file());
        assert_eq!(
            std::fs::read_to_string(root.path().join(".speclink.yaml")).unwrap(),
            marker,
            "remote marker is not rewritten"
        );
    }

    #[test]
    fn a_rename_failure_keeps_local_truth_and_writes_no_marker() {
        let root = complete_workspace();
        let error = finalize_local_migration_with(
            root.path(),
            "https://spec.example.test",
            "demo",
            "backend",
            "2026-07-21",
            |_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "fixture denied",
                ))
            },
        )
        .expect_err("rename failure is reported");

        assert!(error.contains("server import succeeded"));
        assert!(error.contains("Rename") && error.contains("do not retry import"));
        assert!(root.path().join("openspec").is_dir());
        assert!(!root.path().join("openspec.migrated-2026-07-21").exists());
        assert!(!root.path().join(".speclink.yaml").exists());
    }
}

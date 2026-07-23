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
    ArchivedListResponse, ArtifactContent, ChangeStatus, DiscussionInfo, ImportBundle,
    ImportBundleDocument, ImportDocumentId, ImportReportResponse, ImportScope, ListChangesResponse,
    ListSpecsResponse, ScopesResponse, SearchResponse, ShowDiscussionResponse,
    SpecDocumentResponse,
};
use speclink_remote::client::Client;
use speclink_remote::RemoteError;
use speclink_store::content_digest;
use std::collections::{BTreeMap, BTreeSet};
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
    }
}

fn offline_rejected() -> RemoteError {
    RemoteError {
        message: OFFLINE_WRITE_MESSAGE.to_string(),
        reason: Some("offline".to_string()),
        status: None,
    }
}

/// 暫時性失敗原樣中繼（訊息來自 refresh 編排或 Keychain）。
fn unavailable(message: String) -> RemoteError {
    RemoteError {
        message,
        reason: None,
        status: None,
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
    speclink_core::init::write_remote_section(root, &project_url, Some(repo)).map_err(|error| {
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
    // (c) 不支援（server 無端點；changeMeta/changeCapabilities 依實際 payload
    // 定奪為無來源——ChangeStatus/ChangeSummary 皆不帶 metadata 與 capability
    // 名清單；看板拖排待 remote-board-order 刀）
    pub reorder_card: bool,
    pub change_meta: bool,
    pub change_capabilities: bool,
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
            reorder_card: false,
            change_meta: false,
            change_capabilities: false,
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
        self.run_write(credentials, |client| client.claim(change))
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
        self.run_write(credentials, |client| client.archive(change))
    }

    pub fn list_discussions(
        &self,
        credentials: &dyn CredentialStore,
    ) -> Result<DiscussionLists, RemoteError> {
        let active = self.run(credentials, |client| client.list_discussions(false))?;
        let archived = self.run(credentials, |client| client.list_discussions(true))?;
        Ok(DiscussionLists {
            active: active.discussions,
            archived: archived.discussions,
        })
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
    fn editor_handshake_unlocks_all_four_verb_capabilities() {
        // 規格「capability 驅動停用且不偽造缺口」修訂後語意：editor 四欄全真、
        // 看板拖排維持停用（待 remote-board-order 刀）。
        let caps = RemoteCapabilities::from_binding(&binding_with(Capabilities {
            validate: true,
            analyze: true,
            delete_change: true,
            move_task: true,
            ..Default::default()
        }));
        assert!(caps.validate, "validate follows the handshake");
        assert!(caps.analyze, "analyze follows the handshake");
        assert!(caps.delete_change, "deleteChange follows the handshake");
        assert!(caps.move_task, "moveTask follows the handshake");
        assert!(!caps.reorder_card, "board reorder stays disabled (no endpoint yet)");
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

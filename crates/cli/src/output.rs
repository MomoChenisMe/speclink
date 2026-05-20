//! `--json` envelope、`propose create` / `artifact write` / `status` data schema。

use provider::model::{
    ArchivedChange, ArtifactKind, ArtifactState, ArtifactStatus, ChangeStatus, State,
};
use serde::Serialize;
use uuid::Uuid;

/// 測試用環境變數：固定 `requestId` 以利 snapshot 比對。
pub const ENV_TEST_REQUEST_ID: &str = "SPECLINK_TEST_REQUEST_ID";

/// `--json` envelope。`T` 為 command 特定 data schema。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T: Serialize> {
    /// `true` 成功；`false` 失敗。
    pub ok: bool,
    /// 成功時為 data payload；失敗時為 `null`。
    pub data: Option<T>,
    /// 非致命警告陣列；陣列必須存在（即使為空）。
    pub warnings: Vec<Warning>,
    /// 失敗時為 [`ErrorBody`]；成功時為 `null`。
    pub error: Option<ErrorBody>,
    /// `req_<32-hex>` 唯一 invocation 識別碼。
    pub request_id: String,
}

impl<T: Serialize> Envelope<T> {
    /// 建構成功 envelope。
    pub fn success(data: T, warnings: Vec<Warning>, request_id: String) -> Self {
        Self {
            ok: true,
            data: Some(data),
            warnings,
            error: None,
            request_id,
        }
    }

    /// 建構失敗 envelope。
    pub fn failure(error: ErrorBody, request_id: String) -> Self {
        Self {
            ok: false,
            data: None,
            warnings: Vec::new(),
            error: Some(error),
            request_id,
        }
    }
}

/// Envelope 中的單一 warning。
#[derive(Debug, Clone, Serialize)]
pub struct Warning {
    /// 點分隔 code（例如 `provider.not_authenticated`）。
    pub code: String,
    /// 給人讀的訊息。
    pub message: String,
}

/// Envelope 中的失敗 detail。
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    /// 點分隔 error code。
    pub code: String,
    /// 給人讀的訊息。
    pub message: String,
    /// 結構化細節，可為空 object（`{}`）。
    pub details: serde_json::Value,
}

/// `propose create` 成功時的 `data` payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeCreateData {
    /// 已建立的 change id。
    pub change_id: String,
    /// 當前 lifecycle 狀態（成功時為 `"proposed"`）。
    pub state: String,
    /// proposal.md 相對於專案根目錄的 POSIX 路徑。
    pub artifact_path: String,
    /// 解析後的 provider mode（`"local"`）。
    pub mode: String,
}

/// `artifact write` 成功時的 `data` payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactWriteData {
    /// 已寫入的 change id。
    pub change_id: String,
    /// Artifact 識別碼：`"proposal"` / `"design"` / `"tasks"` 或 `"spec:<capability>"`。
    pub artifact_id: String,
    /// Artifact 種類字串（`"proposal"` / `"design"` / `"tasks"` / `"spec"`）。
    pub kind: String,
    /// 寫入檔案相對於專案根目錄的 POSIX 路徑。
    pub path: String,
    /// 解析後的 provider mode（`"local"`）。
    pub mode: String,
}

/// `status` 成功時的 `data` payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusData {
    /// 查詢的 change id。
    pub change_id: String,
    /// Lifecycle 狀態字串（讀自 `metadata.json`）。
    pub state: String,
    /// Artifact 列表，順序固定：proposal、design、tasks、`spec:CAP`（capability 名稱字典序）。
    pub artifacts: Vec<ArtifactStatusJson>,
}

/// `status` JSON output 中單一 artifact 的描述。
///
/// 對應 spec ``` `status` JSON output schema ``` 的 ArtifactStatus object。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactStatusJson {
    /// Artifact 識別碼。
    pub id: String,
    /// Artifact 種類字串。
    pub kind: String,
    /// 相對於 base 的 POSIX 路徑。
    pub path: String,
    /// `"missing"` 或 `"done"`。
    pub status: String,
    /// 是否為必要 artifact。
    pub required: bool,
    /// 依賴的其他 artifact id 列表。
    pub dependencies: Vec<String>,
}

/// 把 [`ArtifactStatus`] 轉為 JSON 友善版本，並套用本 change 固定的 Required/Dependency Rules。
///
/// **Required Rules**：proposal/spec=true、design/tasks=false。
///
/// **Dependency Rules**（單引號避免被解析為 intradoc link）：`proposal=[]`、
/// `design=['proposal']`、`tasks=['proposal','spec']`、`spec=['proposal']`。
pub fn artifact_status_to_json(status: &ArtifactStatus) -> ArtifactStatusJson {
    let (required, dependencies) = required_and_deps(&status.id, status.kind);
    ArtifactStatusJson {
        id: status.id.clone(),
        kind: artifact_kind_str(status.kind).to_string(),
        path: status.path.clone(),
        status: artifact_state_str(status.status).to_string(),
        required,
        dependencies,
    }
}

/// `archive` 成功時的 `data` payload。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveData {
    /// 已 archive 的 change id。
    pub change_id: String,
    /// archive 目錄 POSIX 相對路徑（dry-run 時為「將會用的」路徑）。
    pub archive_path: String,
    /// archive 後的 lifecycle 狀態字串（成功時為 `"archived"`）。
    pub state: String,
    /// archive 時間，ISO 8601 UTC 秒精度字串。
    pub archived_at: String,
    /// 是否為 dry-run。
    pub dry_run: bool,
    /// 各 capability 的 spec delta 套用摘要。
    pub spec_sync: SpecSyncSummaryJson,
}

/// `archive` data 中的 `specSync` 結構。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecSyncSummaryJson {
    /// 各 capability 的套用結果，依 capability 字典序。
    pub capabilities_synced: Vec<CapabilitySyncResultJson>,
}

/// `archive` data 中單一 capability 的套用結果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySyncResultJson {
    /// Capability 名稱。
    pub capability: String,
    /// 主 spec 檔案 POSIX 相對路徑。
    pub main_spec_path: String,
    /// `## ADDED Requirements` 下的區塊數量。
    pub added_count: usize,
    /// `## MODIFIED Requirements` 下的區塊數量。
    pub modified_count: usize,
    /// `## REMOVED Requirements` 下的區塊數量。
    pub removed_count: usize,
    /// `## RENAMED Requirements` 下的區塊數量。
    pub renamed_count: usize,
    /// 主 spec 是否為本次 archive 新建。
    pub created_main_spec: bool,
}

/// 把 [`ArchivedChange`] 轉為 [`ArchiveData`]。
pub fn archived_change_to_archive_data(ac: ArchivedChange) -> ArchiveData {
    ArchiveData {
        change_id: ac.change_id.as_str().to_string(),
        archive_path: ac.archive_path,
        state: state_str(ac.state).to_string(),
        archived_at: ac.archived_at,
        dry_run: ac.dry_run,
        spec_sync: SpecSyncSummaryJson {
            capabilities_synced: ac
                .spec_sync
                .capabilities_synced
                .into_iter()
                .map(|c| CapabilitySyncResultJson {
                    capability: c.capability,
                    main_spec_path: c.main_spec_path,
                    added_count: c.added_count,
                    modified_count: c.modified_count,
                    removed_count: c.removed_count,
                    renamed_count: c.renamed_count,
                    created_main_spec: c.created_main_spec,
                })
                .collect(),
        },
    }
}

/// 把 [`ChangeStatus`] 轉為 [`StatusData`]，套用本 change 固定規則。
pub fn change_status_to_status_data(status: ChangeStatus) -> StatusData {
    let artifacts = status
        .artifacts
        .iter()
        .map(artifact_status_to_json)
        .collect();
    StatusData {
        change_id: status.change_id.as_str().to_string(),
        state: state_str(status.state).to_string(),
        artifacts,
    }
}

fn artifact_kind_str(k: ArtifactKind) -> &'static str {
    match k {
        ArtifactKind::Proposal => "proposal",
        ArtifactKind::Design => "design",
        ArtifactKind::Tasks => "tasks",
        ArtifactKind::Spec => "spec",
    }
}

fn artifact_state_str(s: ArtifactState) -> &'static str {
    match s {
        ArtifactState::Missing => "missing",
        ArtifactState::Done => "done",
    }
}

fn state_str(s: State) -> &'static str {
    match s {
        State::Draft => "draft",
        State::Proposed => "proposed",
        State::Archived => "archived",
    }
}

fn required_and_deps(id: &str, kind: ArtifactKind) -> (bool, Vec<String>) {
    match kind {
        ArtifactKind::Proposal => (true, Vec::new()),
        ArtifactKind::Design => (false, vec!["proposal".to_string()]),
        ArtifactKind::Tasks => (false, vec!["proposal".to_string(), "spec".to_string()]),
        ArtifactKind::Spec => (true, vec!["proposal".to_string()]),
    }
    .pipe(|(req, deps)| {
        // 防呆：spec id 必須以 "spec:" 起頭
        debug_assert!(
            kind != ArtifactKind::Spec || id.starts_with("spec:"),
            "spec artifact must have id starting with 'spec:': {id}"
        );
        (req, deps)
    })
}

trait Pipe: Sized {
    fn pipe<U>(self, f: impl FnOnce(Self) -> U) -> U {
        f(self)
    }
}
impl<T> Pipe for T {}

/// 取得本次 invocation 的 `requestId`。
///
/// 若 `SPECLINK_TEST_REQUEST_ID` 環境變數已設定且非空，則直接使用該值（測試用途）；
/// 否則生成 UUID v4 並格式化為 `req_<32-hex>`（無連字號）。
pub fn request_id() -> String {
    match std::env::var(ENV_TEST_REQUEST_ID) {
        Ok(v) if !v.is_empty() => v,
        _ => format!("req_{}", Uuid::new_v4().simple()),
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use crate::output::{
        ArtifactStatusJson, ArtifactWriteData, ENV_TEST_REQUEST_ID, Envelope, ErrorBody,
        ProposeCreateData, StatusData, Warning, artifact_status_to_json,
        change_status_to_status_data, request_id,
    };
    use provider::model::{
        ArtifactKind, ArtifactState, ArtifactStatus, ChangeId, ChangeStatus, State,
    };
    use serde_json::Value;

    fn parse(v: &impl serde::Serialize) -> Value {
        serde_json::from_str(&serde_json::to_string(v).unwrap()).unwrap()
    }

    #[test]
    fn success_envelope_fields() {
        let data = ProposeCreateData {
            change_id: "demo".to_string(),
            state: "proposed".to_string(),
            artifact_path: ".speclink/changes/demo/proposal.md".to_string(),
            mode: "local".to_string(),
        };
        let env: Envelope<ProposeCreateData> = Envelope::success(data, vec![], "req_x".to_string());
        let v = parse(&env);
        assert_eq!(v["ok"], Value::Bool(true));
        assert!(v["error"].is_null());
        assert!(v["data"].is_object());
        assert!(v["warnings"].is_array());
        assert_eq!(v["requestId"], "req_x");
    }

    #[test]
    fn failure_envelope_fields() {
        let env: Envelope<ProposeCreateData> = Envelope::failure(
            ErrorBody {
                code: "change.already_exists".to_string(),
                message: "change 'demo' already exists".to_string(),
                details: serde_json::json!({}),
            },
            "req_x".to_string(),
        );
        let v = parse(&env);
        assert_eq!(v["ok"], Value::Bool(false));
        assert!(v["data"].is_null());
        assert!(v["error"].is_object());
        assert_eq!(v["error"]["code"], "change.already_exists");
        assert!(v["error"]["message"].is_string());
        assert!(v["error"]["details"].is_object());
    }

    #[test]
    fn propose_create_data_uses_camel_case() {
        let data = ProposeCreateData {
            change_id: "demo".to_string(),
            state: "proposed".to_string(),
            artifact_path: ".speclink/changes/demo/proposal.md".to_string(),
            mode: "local".to_string(),
        };
        let v = parse(&data);
        // 必要欄位
        assert_eq!(v["changeId"], "demo");
        assert_eq!(v["state"], "proposed");
        assert_eq!(v["artifactPath"], ".speclink/changes/demo/proposal.md");
        assert_eq!(v["mode"], "local");
        // 不應該有 snake_case 欄位
        assert!(v.get("change_id").is_none());
        assert!(v.get("artifact_path").is_none());
    }

    #[test]
    fn warning_struct_serializes_correctly() {
        let w = Warning {
            code: "provider.not_authenticated".to_string(),
            message: "Provider 'acme' is configured but not authenticated.".to_string(),
        };
        let v = parse(&w);
        assert_eq!(v["code"], "provider.not_authenticated");
        assert!(v["message"].is_string());
    }

    #[test]
    fn request_id_env_override_is_honored() {
        // Use a guard pattern to avoid leaking env var
        let key = ENV_TEST_REQUEST_ID;
        let prev = std::env::var(key).ok();
        // Safety: only this test mutates this env var.
        unsafe {
            std::env::set_var(key, "req_00000000000000000000000000000000");
        }
        let id = request_id();
        assert_eq!(id, "req_00000000000000000000000000000000");
        unsafe {
            match prev {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn artifact_write_data_uses_camel_case() {
        let data = ArtifactWriteData {
            change_id: "demo".to_string(),
            artifact_id: "spec:user-auth".to_string(),
            kind: "spec".to_string(),
            path: ".speclink/changes/demo/specs/user-auth/spec.md".to_string(),
            mode: "local".to_string(),
        };
        let v = parse(&data);
        assert_eq!(v["changeId"], "demo");
        assert_eq!(v["artifactId"], "spec:user-auth");
        assert_eq!(v["kind"], "spec");
        assert_eq!(v["path"], ".speclink/changes/demo/specs/user-auth/spec.md");
        assert_eq!(v["mode"], "local");
        assert!(v.get("change_id").is_none());
    }

    #[test]
    fn status_data_serializes_camelcase() {
        let data = StatusData {
            change_id: "demo".to_string(),
            state: "proposed".to_string(),
            artifacts: vec![ArtifactStatusJson {
                id: "proposal".to_string(),
                kind: "proposal".to_string(),
                path: ".speclink/changes/demo/proposal.md".to_string(),
                status: "done".to_string(),
                required: true,
                dependencies: vec![],
            }],
        };
        let v = parse(&data);
        assert_eq!(v["changeId"], "demo");
        assert_eq!(v["state"], "proposed");
        assert!(v["artifacts"].is_array());
        assert_eq!(v["artifacts"][0]["id"], "proposal");
        assert_eq!(v["artifacts"][0]["status"], "done");
        assert_eq!(v["artifacts"][0]["required"], true);
        assert!(v["artifacts"][0]["dependencies"].is_array());
    }

    #[test]
    fn artifact_status_to_json_applies_proposal_required() {
        let s = ArtifactStatus {
            id: "proposal".to_string(),
            kind: ArtifactKind::Proposal,
            path: ".speclink/changes/demo/proposal.md".to_string(),
            status: ArtifactState::Done,
            required: false,
            dependencies: vec![],
        };
        let j = artifact_status_to_json(&s);
        assert!(j.required, "proposal must be required");
        assert!(j.dependencies.is_empty());
        assert_eq!(j.status, "done");
    }

    #[test]
    fn artifact_status_to_json_applies_design_deps() {
        let s = ArtifactStatus {
            id: "design".to_string(),
            kind: ArtifactKind::Design,
            path: ".speclink/changes/demo/design.md".to_string(),
            status: ArtifactState::Missing,
            required: false,
            dependencies: vec![],
        };
        let j = artifact_status_to_json(&s);
        assert!(!j.required);
        assert_eq!(j.dependencies, vec!["proposal".to_string()]);
        assert_eq!(j.status, "missing");
    }

    #[test]
    fn artifact_status_to_json_applies_tasks_deps() {
        let s = ArtifactStatus {
            id: "tasks".to_string(),
            kind: ArtifactKind::Tasks,
            path: ".speclink/changes/demo/tasks.md".to_string(),
            status: ArtifactState::Missing,
            required: false,
            dependencies: vec![],
        };
        let j = artifact_status_to_json(&s);
        assert!(!j.required);
        assert_eq!(
            j.dependencies,
            vec!["proposal".to_string(), "spec".to_string()]
        );
    }

    #[test]
    fn artifact_status_to_json_applies_spec_required() {
        let s = ArtifactStatus {
            id: "spec:user-auth".to_string(),
            kind: ArtifactKind::Spec,
            path: ".speclink/changes/demo/specs/user-auth/spec.md".to_string(),
            status: ArtifactState::Done,
            required: false,
            dependencies: vec![],
        };
        let j = artifact_status_to_json(&s);
        assert!(j.required, "spec must be required");
        assert_eq!(j.dependencies, vec!["proposal".to_string()]);
        assert_eq!(j.id, "spec:user-auth");
        assert_eq!(j.kind, "spec");
    }

    #[test]
    fn change_status_to_status_data_preserves_order() {
        let status = ChangeStatus {
            change_id: ChangeId::from("demo"),
            state: State::Proposed,
            artifacts: vec![
                ArtifactStatus {
                    id: "proposal".to_string(),
                    kind: ArtifactKind::Proposal,
                    path: ".speclink/changes/demo/proposal.md".to_string(),
                    status: ArtifactState::Done,
                    required: false,
                    dependencies: vec![],
                },
                ArtifactStatus {
                    id: "spec:auth".to_string(),
                    kind: ArtifactKind::Spec,
                    path: ".speclink/changes/demo/specs/auth/spec.md".to_string(),
                    status: ArtifactState::Done,
                    required: false,
                    dependencies: vec![],
                },
            ],
        };
        let data = change_status_to_status_data(status);
        assert_eq!(data.change_id, "demo");
        assert_eq!(data.state, "proposed");
        assert_eq!(data.artifacts.len(), 2);
        assert_eq!(data.artifacts[0].id, "proposal");
        assert_eq!(data.artifacts[1].id, "spec:auth");
    }

    #[test]
    fn archive_data_serializes_camelcase() {
        use crate::output::{ArchiveData, CapabilitySyncResultJson, SpecSyncSummaryJson};
        let data = ArchiveData {
            change_id: "demo".to_string(),
            archive_path: ".speclink/changes/archive/2026-05-19-demo".to_string(),
            state: "archived".to_string(),
            archived_at: "2026-05-19T12:34:56Z".to_string(),
            dry_run: false,
            spec_sync: SpecSyncSummaryJson {
                capabilities_synced: vec![CapabilitySyncResultJson {
                    capability: "auth".to_string(),
                    main_spec_path: ".speclink/specs/auth/spec.md".to_string(),
                    added_count: 2,
                    modified_count: 0,
                    removed_count: 0,
                    renamed_count: 0,
                    created_main_spec: true,
                }],
            },
        };
        let v = parse(&data);
        assert_eq!(v["changeId"], "demo");
        assert_eq!(
            v["archivePath"],
            ".speclink/changes/archive/2026-05-19-demo"
        );
        assert_eq!(v["state"], "archived");
        assert_eq!(v["archivedAt"], "2026-05-19T12:34:56Z");
        assert_eq!(v["dryRun"], false);
        let cs = &v["specSync"]["capabilitiesSynced"][0];
        assert_eq!(cs["capability"], "auth");
        assert_eq!(cs["mainSpecPath"], ".speclink/specs/auth/spec.md");
        assert_eq!(cs["addedCount"], 2);
        assert_eq!(cs["createdMainSpec"], true);
        assert!(v.get("change_id").is_none());
        assert!(v.get("spec_sync").is_none());
        assert!(cs.get("main_spec_path").is_none());
    }

    #[test]
    fn archived_change_to_archive_data_state_is_archived() {
        use crate::output::archived_change_to_archive_data;
        use provider::model::{
            ArchivedChange, CapabilitySyncResult, ChangeId, SpecDeltaSummary, State,
        };
        let ac = ArchivedChange {
            change_id: ChangeId::from("demo"),
            archive_path: ".speclink/changes/archive/2026-05-19-demo".to_string(),
            state: State::Archived,
            archived_at: "2026-05-19T12:34:56Z".to_string(),
            spec_sync: SpecDeltaSummary {
                capabilities_synced: vec![CapabilitySyncResult {
                    capability: "auth".to_string(),
                    main_spec_path: ".speclink/specs/auth/spec.md".to_string(),
                    added_count: 1,
                    modified_count: 0,
                    removed_count: 0,
                    renamed_count: 0,
                    created_main_spec: true,
                }],
            },
            dry_run: true,
        };
        let data = archived_change_to_archive_data(ac);
        assert_eq!(data.state, "archived");
        assert!(data.dry_run);
        assert_eq!(data.spec_sync.capabilities_synced.len(), 1);
    }

    #[test]
    fn request_id_random_matches_regex() {
        let key = ENV_TEST_REQUEST_ID;
        let prev = std::env::var(key).ok();
        unsafe { std::env::remove_var(key) };
        let id = request_id();
        assert!(
            id.starts_with("req_"),
            "request_id should start with req_: {id}"
        );
        let hex = &id[4..];
        assert_eq!(hex.len(), 32, "hex part must be 32 chars: {id}");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hex must be lowercase hex: {id}"
        );
        unsafe {
            if let Some(v) = prev {
                std::env::set_var(key, v);
            }
        }
    }
}

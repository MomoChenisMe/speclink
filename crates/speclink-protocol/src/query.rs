//! Query API DTOs: change, spec, discussion, artifact, and derived-status
//! read shapes.
//!
//! These mirror the fs-mode `--json` field names (camelCase) — the verb
//! contract's "remote output shape matches fs" requirement makes the fs
//! serialization the wire canon for reads. Server-side extras (repo,
//! lifecycle, versions) are optional with defaults; the parity view drops
//! them at the CLI boundary.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::binding::ScopeRef;

/// `POST /import` request. This migration surface is intentionally closed:
/// there is no mode field, so the store's maintenance-only Overwrite mode can
/// never be selected over the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportBundle {
    pub format_version: u32,
    pub scope: ImportScope,
    pub project_revision: u64,
    pub documents: Vec<ImportBundleDocument>,
}

/// The source scope declared by an import bundle. The server requires this to
/// match the authenticated project/repo binding instead of trusting it as an
/// alternate destination selector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportScope {
    pub project: String,
    pub repo: String,
}

/// One content-addressed document in an [`ImportBundle`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportBundleDocument {
    pub document: ImportDocumentId,
    pub content: String,
    pub digest: String,
}

/// The closed logical document identity used by migration bundles. The tagged
/// shape stays path-free and maps one-to-one to the TeamStore contract.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ImportDocumentId {
    ChangeMeta { change: String },
    ChangeArtifact { change: String, artifact: String },
    CanonicalSpec { capability: String },
    Discussion { slug: String, archived: bool },
    WorkflowConfig,
    ArchivedChange { change: String, doc: String },
    Language,
    BoardOrder,
}

/// Successful `POST /import` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportReportResponse {
    pub project_revision: u64,
    pub documents: Vec<ImportedDocument>,
}

/// One imported document and its migration-only outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportedDocument {
    pub document: ImportDocumentId,
    pub outcome: ImportDocumentOutcome,
}

/// CreateNew is the only wire operation, so Created is the only representable
/// success outcome. Overwritten deliberately does not exist in this registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ImportDocumentOutcome {
    Created,
}

/// `GET /board-order` response: the scope's opaque board-order document and
/// the scope revision (the CAS token a later PUT sends back as `If-Match`).
/// An absent document is a normal state — `content` is null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardOrderResponse {
    pub content: Option<String>,
    pub revision: u64,
}

/// `PUT /board-order` request: the full replacement text. The server stores
/// it verbatim — the CAS precondition travels in the `If-Match` header, not
/// the body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PutBoardOrderRequest {
    pub content: String,
}

/// Successful `PUT /board-order` response: the scope revision the write
/// landed at (also the new ETag value).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutBoardOrderResponse {
    pub revision: u64,
}

/// `GET /scopes` response: every project the caller is a member of, with its
/// registered repos. This route is identity-scoped and precedes repo binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScopesResponse {
    pub projects: Vec<ProjectScope>,
}

/// One membership-filtered project inside [`ScopesResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScope {
    pub id: String,
    pub key: String,
    pub name: String,
    pub repos: Vec<ScopeRef>,
}

/// `GET /specs/{capability}/document` and archived-artifact response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecDocumentResponse {
    pub content: String,
}

/// `GET /archived` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedListResponse {
    pub archived: Vec<ArchivedItem>,
}

/// One archived change, shaped like the desktop archive card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedItem {
    pub dated_name: String,
    pub date: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks_total: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks_done: Option<usize>,
    #[serde(default)]
    pub spec_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default)]
    pub from_discussions: Vec<String>,
}

/// `GET /search` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
}

/// The first matching artifact for one active change or live discussion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub id: String,
    pub artifact: String,
    pub snippet: String,
}

/// `GET /changes` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListChangesResponse {
    pub changes: Vec<ChangeSummary>,
}

/// One change in the listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSummary {
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub completed_tasks: usize,
    #[serde(default)]
    pub total_tasks: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restale_from: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
}

/// `GET /changes/{name}` response — the fs `StatusReport` shape plus the
/// server's own fields. The trailing meta trio (`created`, `fromDiscussions`,
/// `deltaCapabilities`) feeds the CLI's remote `show` composition (design D4
/// 實作期修正): `created` appears only when the meta carries the
/// schema+created pair (the engine ShowChange unit rule), the lists are
/// omitted when empty, and an older server simply never sends them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeStatus {
    pub change_name: String,
    pub schema_name: String,
    pub is_complete: bool,
    pub apply_requires: Vec<String>,
    pub artifacts: Vec<ArtifactStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub from_discussions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delta_capabilities: Vec<String>,
}

/// One artifact's status inside [`ChangeStatus`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactStatus {
    pub id: String,
    pub output_path: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_deps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,
}

/// `GET /changes/{name}/instructions/apply` response. `preflight` is
/// deliberately fs-only (local file checks) — the wire contract omits it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApplyInstructions {
    pub change_name: String,
    pub change_dir: String,
    pub schema_name: String,
    pub context_files: std::collections::BTreeMap<String, String>,
    pub progress: Progress,
    pub tasks: Vec<TaskEntry>,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_artifacts: Option<Vec<String>>,
    pub locale: String,
    /// Effective TDD/audit policy toggles. Deliberately NOT `serde(default)`:
    /// a defaulted `false` from an old server would silently switch the apply
    /// discipline off — fail closed on version skew instead (same rationale as
    /// [`Progress`]'s code counters).
    pub tdd: bool,
    pub audit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
}

/// Task-completion counters inside [`ApplyInstructions`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
    /// Code tasks alone (`[M]` manual tasks excluded) — what the
    /// station gates judge against. Deliberately NOT `serde(default)`: a
    /// defaulted 0/0/0 from an old server would read as "code work finished"
    /// and let the gates pass — fail closed on version skew instead.
    pub code_total: usize,
    pub code_complete: usize,
    pub code_remaining: usize,
}

/// One task inside [`ApplyInstructions`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskEntry {
    pub id: String,
    pub description: String,
    pub done: bool,
    /// No `serde(default)` for the same reason as [`Progress`]'s code counts:
    /// a silent `false` from an old server would hide manual tasks from the
    /// gates — fail closed on version skew instead.
    pub manual: bool,
}

/// `GET /changes/{name}/instructions/{artifact}` response for a schema
/// artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactInstructions {
    pub change_name: String,
    pub artifact_id: String,
    pub schema_name: String,
    pub change_dir: String,
    pub output_path: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<String>>,
    pub locale: String,
    pub template: String,
    pub dependencies: Vec<DependencyEntry>,
    pub unlocks: Vec<String>,
}

/// One upstream artifact inside [`ArtifactInstructions::dependencies`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEntry {
    pub id: String,
    pub done: bool,
    pub path: String,
    pub description: String,
}

/// `GET /changes/{name}/artifacts/{artifact}` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactContent {
    #[serde(default)]
    pub artifact: String,
    pub content: String,
    #[serde(default)]
    pub version: u64,
}

/// `GET /specs` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListSpecsResponse {
    pub specs: Vec<SpecSummary>,
}

/// One canonical capability in the spec listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecSummary {
    pub id: String,
    pub path: String,
}

/// `GET /language` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LanguageResponse {
    pub content: String,
}

/// `GET /changes/{name}/validate` response — one change's structural
/// validation, the engine's `ValidationResult` on the wire (server-verb-api:
/// 端點固定單 change，CLI 的聚合語意由 client 組合).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValidateChangeResponse {
    pub change: String,
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// `GET /changes/{name}/analyze` response — the engine's full `AnalyzeReport`
/// on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeReportResponse {
    pub change_id: String,
    #[serde(default)]
    pub dimensions: Vec<AnalyzeDimension>,
    #[serde(default)]
    pub findings: Vec<AnalyzeFinding>,
    #[serde(default)]
    pub artifacts_analyzed: Vec<String>,
    #[serde(default)]
    pub artifacts_missing: Vec<String>,
}

/// One analysis dimension's rollup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeDimension {
    pub dimension: String,
    pub status: String,
    pub finding_count: usize,
}

/// One analysis finding, with its typed i18n messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeFinding {
    pub id: String,
    pub dimension: String,
    pub severity: String,
    pub location: String,
    pub summary: String,
    pub recommendation: String,
    pub summary_msg: AnalyzeMsg,
    pub recommendation_msg: AnalyzeMsg,
}

/// A typed, locale-independent message: key plus named parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeMsg {
    pub key: String,
    #[serde(default)]
    pub params: std::collections::BTreeMap<String, String>,
}

/// `GET /config` response — the workflow policy view a client may read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub revision: u64,
}

/// `PUT /config` request — a full workflow policy document guarded by the
/// scope revision returned from [`ConfigResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutConfigRequest {
    pub content: String,
    pub expected_revision: u64,
}

/// `PUT /config` response — the scope revision after the successful commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutConfigResponse {
    pub revision: u64,
}

/// `GET /whoami` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    pub user: WhoamiUser,
    #[serde(default)]
    pub repos: Vec<WhoamiRepo>,
}

/// `GET /auth/whoami` response (root level, no project scope): the identity
/// behind a bearer — the display a client shows right after logging in,
/// before any project is chosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthWhoamiResponse {
    pub user: WhoamiUser,
}

/// The authenticated identity inside [`WhoamiResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiUser {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub handle: String,
}

/// One registered repo inside [`WhoamiResponse`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiRepo {
    pub name: String,
    #[serde(default)]
    pub git_url: String,
}

/// `GET /discussions` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListDiscussionsResponse {
    pub discussions: Vec<DiscussionInfo>,
}

/// One discussion's metadata — mirrors the fs serialization exactly
/// (`createdBy` omitted when absent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionInfo {
    pub slug: String,
    pub topic: String,
    pub status: String,
    pub rounds: usize,
    pub created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// 討論型別（目前唯一合法值 `improve`）；一般討論缺席時省略。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub path: String,
    pub archived: bool,
}

/// `GET /discussions/{slug}` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShowDiscussionResponse {
    pub info: DiscussionInfo,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use crate::query::*;

    #[test]
    fn server_scope_read_dtos_round_trip_in_desktop_aligned_shapes() {
        let scopes: ScopesResponse = serde_json::from_str(
            r#"{"projects":[{"id":"prj_demo","key":"demo","name":"Demo","repos":[{"id":"repo_backend","key":"backend","name":"Backend"}]}]}"#,
        )
        .unwrap();
        assert_eq!(scopes.projects[0].repos[0].key, "backend");

        let archived: ArchivedListResponse = serde_json::from_str(
            r#"{"archived":[{"datedName":"2026-07-20-old","date":"2026-07-20","name":"old","tasksTotal":2,"tasksDone":1,"specCount":1,"createdBy":"momo","fromDiscussions":["source"]}]}"#,
        )
        .unwrap();
        assert_eq!(archived.archived[0].tasks_total, Some(2));
        assert_eq!(archived.archived[0].from_discussions, ["source"]);

        let document = SpecDocumentResponse {
            content: "# spec\n".to_string(),
        };
        let search = SearchResponse {
            hits: vec![SearchHit {
                kind: "change".to_string(),
                id: "demo".to_string(),
                artifact: "proposal.md".to_string(),
                snippet: "…needle…".to_string(),
            }],
        };
        assert_eq!(
            serde_json::to_value(document).unwrap()["content"],
            "# spec\n"
        );
        assert_eq!(
            serde_json::to_value(search).unwrap()["hits"][0]["artifact"],
            "proposal.md"
        );
    }

    #[test]
    fn server_scope_read_dtos_export_json_schema() {
        for schema in [
            serde_json::to_string(&schemars::schema_for!(ScopesResponse)).unwrap(),
            serde_json::to_string(&schemars::schema_for!(SpecDocumentResponse)).unwrap(),
            serde_json::to_string(&schemars::schema_for!(ArchivedListResponse)).unwrap(),
            serde_json::to_string(&schemars::schema_for!(SearchResponse)).unwrap(),
        ] {
            assert!(
                schema.contains("properties"),
                "DTO schema is structural: {schema}"
            );
        }
    }

    #[test]
    fn change_summary_round_trips_and_defaults_extras() {
        let full: ChangeSummary = serde_json::from_str(
            r#"{"name":"demo","summary":"Demo change summary","status":"done","completedTasks":2,"totalTasks":2,"repo":"backend","lifecycle":"applying","claimedBy":"me"}"#,
        )
        .unwrap();
        assert_eq!(full.name, "demo");
        assert_eq!(full.completed_tasks, 2);
        assert_eq!(full.repo.as_deref(), Some("backend"));
        assert!(
            full.restale_from.is_empty(),
            "absent restaleFrom defaults to empty"
        );
        assert_eq!(full.meta_error, None);

        let list: ListChangesResponse =
            serde_json::from_str(r#"{"changes":[{"name":"demo"}]}"#).unwrap();
        assert_eq!(list.changes.len(), 1);

        let json = serde_json::to_value(&full).unwrap();
        assert_eq!(
            json["completedTasks"], 2,
            "fields serialize camelCase: {json}"
        );
        let back: ChangeSummary = serde_json::from_value(json).unwrap();
        assert_eq!(back, full);
    }

    #[test]
    fn change_summary_started_at_is_optional_and_camel_case() {
        let started: ChangeSummary =
            serde_json::from_str(r#"{"name":"demo","startedAt":"2026-07-30"}"#).unwrap();
        assert_eq!(started.started_at.as_deref(), Some("2026-07-30"));
        let json = serde_json::to_value(&started).unwrap();
        assert_eq!(json["startedAt"], "2026-07-30", "field serializes camelCase: {json}");
        let back: ChangeSummary = serde_json::from_value(json).unwrap();
        assert_eq!(back, started);

        let legacy: ChangeSummary = serde_json::from_str(r#"{"name":"demo"}"#).unwrap();
        assert_eq!(
            legacy.started_at, None,
            "old payloads without startedAt still deserialize"
        );
        let legacy_json = serde_json::to_value(&legacy).unwrap();
        assert!(
            legacy_json.get("startedAt").is_none(),
            "absent startedAt is omitted: {legacy_json}"
        );
    }

    #[test]
    fn change_status_mirrors_the_fs_status_report_shape() {
        let status: ChangeStatus = serde_json::from_str(
            r#"{"changeName":"demo","schemaName":"spec-driven","isComplete":true,"applyRequires":["tasks"],"artifacts":[{"id":"proposal","outputPath":"proposal.md","status":"done","version":3}],"repo":"backend","lifecycle":"applying","statusVersion":4,"claimedBy":"me"}"#,
        )
        .unwrap();
        assert_eq!(status.change_name, "demo");
        assert!(status.is_complete);
        assert_eq!(status.apply_requires, ["tasks"]);
        assert_eq!(status.artifacts[0].output_path, "proposal.md");
        assert_eq!(status.artifacts[0].version, Some(3));
        assert_eq!(status.status_version, Some(4));

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["changeName"], "demo");
        assert_eq!(json["artifacts"][0]["outputPath"], "proposal.md");
        let back: ChangeStatus = serde_json::from_value(json).unwrap();
        assert_eq!(back, status);
    }

    #[test]
    fn apply_instructions_round_trip_without_a_preflight_field() {
        let apply: ApplyInstructions = serde_json::from_str(
            r#"{"changeName":"demo","changeDir":"changes/demo","schemaName":"spec-driven","contextFiles":{"design":"design.md","proposal":"proposal.md","specs":"specs/**/*.md","tasks":"tasks.md"},"progress":{"total":3,"complete":3,"remaining":0,"codeTotal":2,"codeComplete":2,"codeRemaining":0},"tasks":[{"id":"1","description":"1.1 First","done":true,"manual":false},{"id":"3","description":"1.3 Hand check","done":true,"manual":true}],"state":"all_done","locale":"English","tdd":true,"audit":false,"instruction":"Work through the tasks.\n"}"#,
        )
        .unwrap();
        assert_eq!(apply.change_name, "demo");
        assert_eq!(apply.progress.total, 3);
        assert_eq!(apply.progress.code_total, 2, "manual tasks stay out of the code counts");
        assert_eq!(apply.tasks[0].description, "1.1 First");
        assert!(!apply.tasks[0].manual);
        assert!(apply.tasks[1].manual, "[M] task rides the wire as manual");
        assert_eq!(apply.state, "all_done");
        assert!(apply.tdd, "tdd toggle rides the wire");
        assert!(!apply.audit);

        let json = serde_json::to_value(&apply).unwrap();
        assert_eq!(json["contextFiles"]["design"], "design.md");
        assert_eq!(json["progress"]["codeRemaining"], 0, "camelCase on the new counters");
        assert_eq!(json["tasks"][1]["manual"], true);
        assert_eq!(json["tdd"], true, "camelCase policy toggles on the wire");
        assert_eq!(json["audit"], false);

        // Version-skew fail closed: an old server's payload without the policy
        // toggles must be rejected, never defaulted to "discipline off".
        let mut skewed = json.clone();
        skewed.as_object_mut().unwrap().remove("tdd");
        assert!(
            serde_json::from_value::<ApplyInstructions>(skewed).is_err(),
            "a payload missing tdd must fail to deserialize"
        );
        assert!(
            json.get("preflight").is_none(),
            "preflight is deliberately fs-only — the wire contract omits it"
        );
        assert!(
            json.get("missingArtifacts").is_none(),
            "absent missingArtifacts is omitted, matching fs serialization"
        );
        let back: ApplyInstructions = serde_json::from_value(json).unwrap();
        assert_eq!(back, apply);
    }

    #[test]
    fn artifact_instructions_round_trip_in_camel_case() {
        let instr: ArtifactInstructions = serde_json::from_str(
            r###"{"changeName":"demo","artifactId":"proposal","schemaName":"spec-driven","changeDir":"changes/demo","outputPath":"proposal.md","description":"Initial proposal","instruction":"Create the proposal.\n","locale":"English","template":"## Why\n","dependencies":[{"id":"design","done":false,"path":"design.md","description":"Design"}],"unlocks":["design"]}"###,
        )
        .unwrap();
        assert_eq!(instr.artifact_id, "proposal");
        assert_eq!(instr.dependencies[0].id, "design");
        assert_eq!(instr.unlocks, ["design"]);
        let json = serde_json::to_value(&instr).unwrap();
        assert_eq!(json["outputPath"], "proposal.md");
        assert!(
            json.get("context").is_none() && json.get("rules").is_none(),
            "absent optional policy fields are omitted: {json}"
        );
        let back: ArtifactInstructions = serde_json::from_value(json).unwrap();
        assert_eq!(back, instr);
    }

    #[test]
    fn artifact_content_carries_content_and_version() {
        let got: ArtifactContent = serde_json::from_str(
            r###"{"artifact":"design","content":"## Context\n","version":8}"###,
        )
        .unwrap();
        assert_eq!(got.content, "## Context\n");
        assert_eq!(got.version, 8);
        let bare: ArtifactContent = serde_json::from_str(r#"{"content":""}"#).unwrap();
        assert_eq!(bare.version, 0, "absent version defaults to zero");
    }

    #[test]
    fn spec_language_config_and_whoami_shapes_deserialize() {
        let specs: ListSpecsResponse = serde_json::from_str(
            r#"{"specs":[{"id":"user-auth","path":"specs/user-auth/spec.md"}]}"#,
        )
        .unwrap();
        assert_eq!(specs.specs[0].id, "user-auth");

        let language: LanguageResponse =
            serde_json::from_str(r##"{"content":"# Language\n"}"##).unwrap();
        assert_eq!(language.content, "# Language\n");

        let config: ConfigResponse = serde_json::from_str(r#"{"schema":"spec-driven"}"#).unwrap();
        assert_eq!(config.schema, "spec-driven");
        assert_eq!(config.content, None, "older servers omit policy content");
        assert_eq!(config.revision, 0, "older servers omit policy revision");

        let put: PutConfigRequest = serde_json::from_str(
            r#"{"content":"schema: spec-driven\n","expectedRevision":7}"#,
        )
        .unwrap();
        assert_eq!(put.expected_revision, 7);
        let put_json = serde_json::to_value(&put).unwrap();
        assert_eq!(put_json["expectedRevision"], 7, "request is camelCase");

        let whoami: WhoamiResponse = serde_json::from_str(
            r#"{"user":{"name":"王小明","handle":"ming"},"repos":[{"name":"backend","gitUrl":"https://git.example.com/erp.git"}]}"#,
        )
        .unwrap();
        assert_eq!(whoami.user.handle, "ming");
        assert_eq!(whoami.repos[0].git_url, "https://git.example.com/erp.git");
        let json = serde_json::to_value(&whoami).unwrap();
        assert_eq!(
            json["repos"][0]["gitUrl"],
            "https://git.example.com/erp.git"
        );
    }

    #[test]
    fn discussion_shapes_mirror_fs_serialization() {
        let list: ListDiscussionsResponse = serde_json::from_str(
            r#"{"discussions":[{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false}]}"#,
        )
        .unwrap();
        let info = &list.discussions[0];
        assert_eq!(info.slug, "demo-topic");
        assert_eq!(info.created_by, None);
        let json = serde_json::to_value(info).unwrap();
        assert!(
            json.get("createdBy").is_none(),
            "absent createdBy is omitted, matching fs serialization: {json}"
        );

        let show: ShowDiscussionResponse = serde_json::from_str(
            r##"{"info":{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","createdBy":"Ming <m@example.com>","path":"discussions/demo-topic.md","archived":false},"content":"# Discussion\n"}"##,
        )
        .unwrap();
        assert_eq!(
            show.info.created_by.as_deref(),
            Some("Ming <m@example.com>")
        );
        assert_eq!(show.content, "# Discussion\n");
    }

    #[test]
    fn discussion_kind_is_optional_and_camel_case() {
        // add-improve-flow：kind 有值時以單字 camelCase 曝露、無值時整個鍵省略，
        // 既有 payload 形狀逐位元不變。
        let marked: ListDiscussionsResponse = serde_json::from_str(
            r#"{"discussions":[{"slug":"improve-core","topic":"核心結構改進","status":"open","rounds":1,"created":"2026-08-07","kind":"improve","path":"discussions/improve-core.md","archived":false}]}"#,
        )
        .unwrap();
        let info = &marked.discussions[0];
        assert_eq!(info.kind.as_deref(), Some("improve"));
        assert_eq!(serde_json::to_value(info).unwrap()["kind"], "improve");

        let plain: ListDiscussionsResponse = serde_json::from_str(
            r#"{"discussions":[{"slug":"demo-topic","topic":"Demo topic","status":"open","rounds":0,"created":"2026-07-01","path":"discussions/demo-topic.md","archived":false}]}"#,
        )
        .unwrap();
        let info = &plain.discussions[0];
        assert_eq!(info.kind, None, "舊 payload 無 kind 仍可解析");
        let json = serde_json::to_value(info).unwrap();
        assert!(json.get("kind").is_none(), "absent kind is omitted: {json}");
        assert_eq!(
            json,
            serde_json::json!({
                "slug": "demo-topic",
                "topic": "Demo topic",
                "status": "open",
                "rounds": 0,
                "path": "discussions/demo-topic.md",
                "created": "2026-07-01",
                "archived": false,
            }),
            "既有形狀不因新欄位改變"
        );
    }

    #[test]
    fn query_dtos_export_json_schema() {
        for (name, schema) in [
            (
                "ListChangesResponse",
                schemars::schema_for!(ListChangesResponse),
            ),
            ("ChangeStatus", schemars::schema_for!(ChangeStatus)),
            (
                "ApplyInstructions",
                schemars::schema_for!(ApplyInstructions),
            ),
            (
                "ArtifactInstructions",
                schemars::schema_for!(ArtifactInstructions),
            ),
            (
                "ShowDiscussionResponse",
                schemars::schema_for!(ShowDiscussionResponse),
            ),
        ] {
            let text = serde_json::to_string(&schema)
                .unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains("properties"), "{name} schema has properties");
        }
        let status = serde_json::to_string(&schemars::schema_for!(ChangeStatus)).unwrap();
        assert!(
            status.contains("changeName") && status.contains("applyRequires"),
            "schema fields are camelCase: {status}"
        );
    }
}

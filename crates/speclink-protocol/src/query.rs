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
}

/// `GET /changes/{name}` response — the fs `StatusReport` shape plus the
/// server's own fields.
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
}

/// One task inside [`ApplyInstructions`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskEntry {
    pub id: String,
    pub description: String,
    pub done: bool,
    pub parallel: bool,
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

/// `GET /config` response — the workflow policy view a client may read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    #[serde(default)]
    pub schema: String,
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
    fn change_summary_round_trips_and_defaults_extras() {
        let full: ChangeSummary = serde_json::from_str(
            r#"{"name":"demo","summary":"Demo change summary","status":"done","completedTasks":2,"totalTasks":2,"repo":"backend","lifecycle":"applying","claimedBy":"me"}"#,
        )
        .unwrap();
        assert_eq!(full.name, "demo");
        assert_eq!(full.completed_tasks, 2);
        assert_eq!(full.repo.as_deref(), Some("backend"));
        assert!(full.restale_from.is_empty(), "absent restaleFrom defaults to empty");
        assert_eq!(full.meta_error, None);

        let list: ListChangesResponse =
            serde_json::from_str(r#"{"changes":[{"name":"demo"}]}"#).unwrap();
        assert_eq!(list.changes.len(), 1);

        let json = serde_json::to_value(&full).unwrap();
        assert_eq!(json["completedTasks"], 2, "fields serialize camelCase: {json}");
        let back: ChangeSummary = serde_json::from_value(json).unwrap();
        assert_eq!(back, full);
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
            r#"{"changeName":"demo","changeDir":"changes/demo","schemaName":"spec-driven","contextFiles":{"design":"design.md","proposal":"proposal.md","specs":"specs/**/*.md","tasks":"tasks.md"},"progress":{"total":2,"complete":2,"remaining":0},"tasks":[{"id":"1","description":"1.1 First","done":true,"parallel":false}],"state":"all_done","locale":"English","instruction":"Work through the tasks.\n"}"#,
        )
        .unwrap();
        assert_eq!(apply.change_name, "demo");
        assert_eq!(apply.progress.total, 2);
        assert_eq!(apply.tasks[0].description, "1.1 First");
        assert_eq!(apply.state, "all_done");

        let json = serde_json::to_value(&apply).unwrap();
        assert_eq!(json["contextFiles"]["design"], "design.md");
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
        let specs: ListSpecsResponse =
            serde_json::from_str(r#"{"specs":[{"id":"user-auth","path":"specs/user-auth/spec.md"}]}"#)
                .unwrap();
        assert_eq!(specs.specs[0].id, "user-auth");

        let language: LanguageResponse =
            serde_json::from_str(r##"{"content":"# Language\n"}"##).unwrap();
        assert_eq!(language.content, "# Language\n");

        let config: ConfigResponse = serde_json::from_str(r#"{"schema":"spec-driven"}"#).unwrap();
        assert_eq!(config.schema, "spec-driven");

        let whoami: WhoamiResponse = serde_json::from_str(
            r#"{"user":{"name":"王小明","handle":"ming"},"repos":[{"name":"backend","gitUrl":"https://git.example.com/erp.git"}]}"#,
        )
        .unwrap();
        assert_eq!(whoami.user.handle, "ming");
        assert_eq!(whoami.repos[0].git_url, "https://git.example.com/erp.git");
        let json = serde_json::to_value(&whoami).unwrap();
        assert_eq!(json["repos"][0]["gitUrl"], "https://git.example.com/erp.git");
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
        assert_eq!(show.info.created_by.as_deref(), Some("Ming <m@example.com>"));
        assert_eq!(show.content, "# Discussion\n");
    }

    #[test]
    fn query_dtos_export_json_schema() {
        for (name, schema) in [
            ("ListChangesResponse", schemars::schema_for!(ListChangesResponse)),
            ("ChangeStatus", schemars::schema_for!(ChangeStatus)),
            ("ApplyInstructions", schemars::schema_for!(ApplyInstructions)),
            ("ArtifactInstructions", schemars::schema_for!(ArtifactInstructions)),
            ("ShowDiscussionResponse", schemars::schema_for!(ShowDiscussionResponse)),
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

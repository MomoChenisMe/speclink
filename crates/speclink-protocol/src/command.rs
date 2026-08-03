//! Command API DTOs: the write-verb request and response shapes.
//!
//! Every field serializes camelCase; absent optional fields are omitted so
//! request bodies stay byte-identical to the pre-typed client's. Response
//! extras the server may add (repo, lifecycle) are optional with defaults —
//! a minimal server stays conformant.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `POST /changes` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeRequest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_discussion: Option<String>,
}

/// `POST /changes` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeResponse {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
}

/// `PUT /changes/{name}/artifacts/{artifact}` request body. The write
/// precondition travels as the `If-Match` header, not in the body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutArtifactRequest {
    pub content: String,
}

/// `PUT /changes/{name}/artifacts/{artifact}` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutArtifactResponse {
    pub artifact: String,
    pub version: u64,
}

/// `POST /changes/{name}/tasks/{taskId}/done` request body. An empty
/// attribution set serializes as the bare object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskDoneRequest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub touched_files: Vec<String>,
}

/// `POST /changes/{name}/tasks/{taskId}/done` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskDoneResponse {
    #[serde(default)]
    pub task_desc: String,
    #[serde(default)]
    pub already_done: bool,
}

/// `POST /changes/{name}/tasks/{taskId}/undone` response (the request body
/// is always the bare object — unchecking records no touched files).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskUndoneResponse {
    #[serde(default)]
    pub task_desc: String,
    #[serde(default)]
    pub already_undone: bool,
}

/// `DELETE /changes/{name}/in-progress` 409 evidence — the work traces that
/// blocked the marker removal, flattened into the error envelope. Additive
/// wire contract: fields only ever get added, never changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevertBlockedEvidence {
    /// Checked-task count of the change's tasks.md.
    pub checked_tasks: usize,
    /// Union of the touched record's v1 and v2 file lists, deduplicated.
    pub touched_files: Vec<String>,
}

/// `POST /changes/{name}/claim` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClaimResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_by: Option<String>,
}

/// `POST /changes/{name}/archive` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResponse {
    #[serde(default)]
    pub specs: Vec<ArchivedSpec>,
}

/// One canonical capability the archive updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedSpec {
    pub capability: String,
}

/// `DELETE /changes/{name}` response — the discarded change and the source
/// discussions the discard unlinked (server-verb-api: discard 全語意).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscardResponse {
    #[serde(default)]
    pub change: String,
    #[serde(default)]
    pub unlinked_discussions: Vec<UnlinkedDiscussion>,
}

/// One source discussion the discard unlinked, with its status after unlinking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkedDiscussion {
    pub slug: String,
    pub status: String,
}

/// `POST /changes/{name}/tasks/move` request — 1-based checkbox ordinals plus
/// the optional explicit side (absent = direction inference), mirroring the UI
/// moveTask signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskRequest {
    pub from: usize,
    pub to: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<bool>,
}

/// `POST /changes/{name}/tasks/move` response — the moved task's post-move
/// description (prefixes already renumbered).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveTaskResponse {
    #[serde(default)]
    pub change: String,
    #[serde(default)]
    pub description: String,
}

/// `POST /discussions` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionRequest {
    pub topic: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

/// `POST /discussions` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionResponse {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub path: String,
}

/// `DELETE /discussions/{slug}` response (`force` travels as a query
/// parameter, mirroring the change-side DELETE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscardDiscussionResponse {
    #[serde(default)]
    pub slug: String,
}

/// `POST /discussions/{slug}/link` and `/seal` request body: the change side
/// of the bind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BindDiscussionRequest {
    pub change: String,
}

/// `POST /discussions/{slug}/link` and `/seal` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BindDiscussionResponse {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub change: String,
}

/// `PUT /discussions/{slug}/context` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetDiscussionContextRequest {
    pub content: String,
}

/// `POST /discussions/{slug}/rounds` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddDiscussionRoundRequest {
    pub mode: String,
    pub content: String,
}

/// `POST /discussions/{slug}/rounds` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddDiscussionRoundResponse {
    #[serde(default)]
    pub round: u64,
}

/// `POST /discussions/{slug}/conclude` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConcludeDiscussionRequest {
    pub content: String,
}

/// `POST /discussions/{slug}/archive` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDiscussionResponse {
    #[serde(default)]
    pub archived_to: String,
}

/// `POST /discussions/{slug}/promote` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromoteDiscussionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `POST /discussions/{slug}/promote` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PromoteDiscussionResponse {
    #[serde(default)]
    pub change: String,
}

/// 工單一輪的 wire 形狀——鏡射 CLI `review show --json`（審查站對外契約）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRoundDto {
    pub index: u64,
    /// 結構化輪次的 phase token（discovery|validation）；legacy round 為 null。
    /// 刻意的 additive shape change——欄位恆出現、缺席以明確 null 表示。
    #[serde(default)]
    pub phase: Option<String>,
    /// 結構化輪次綁定的 frozen patch identity（`sha256:<hex>`）；legacy 為 null。
    #[serde(default)]
    pub patch_hash: Option<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub findings: Vec<ReviewFindingDto>,
}

/// 工單 findings 行的 wire 形狀（severity ∈ CRITICAL／WARNING／SUGGESTION）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFindingDto {
    pub severity: String,
    pub path: String,
    pub text: String,
}

/// `GET /changes/{name}/review` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewTicketResponse {
    pub change: String,
    pub rounds: Vec<ReviewRoundDto>,
    pub last_round: ReviewRoundDto,
}

/// `POST /changes/{name}/review/rounds` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddReviewRoundRequest {
    pub content: String,
}

/// `POST /changes/{name}/review/rounds` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddReviewRoundResponse {
    #[serde(default)]
    pub round: u64,
}

/// `POST /changes/{name}/review/stamp` request body（design D4a）：指紋由
/// 工作樹持有者預算上 wire，server 不重算；`missing` 明示宣告聯集中已不存在
/// 的檔（工作樹持有者才知道），server 驗「scope ∪ missing ＝工單聯集且不相交」。
/// 欄位缺席讀作空清單——舊 client 不帶時即原本的嚴格集合相等。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StampReviewRequest {
    #[serde(default)]
    pub accept: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default)]
    pub scope: Vec<ReviewScopeEntryDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

/// `reviewed_scope` 清單項的 wire 形狀。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewScopeEntryDto {
    pub path: String,
    pub hash: String,
}

/// `POST /changes/{name}/review/stamp` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StampReviewResponse {
    pub change: String,
}

/// `DELETE /changes/{name}/review` response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiscardReviewResponse {
    pub change: String,
}

#[cfg(test)]
mod tests {
    use crate::command::*;

    #[test]
    fn create_change_request_round_trips_in_camel_case() {
        let req = CreateChangeRequest {
            name: "add-rate-limit".into(),
            schema: Some("spec-driven".into()),
            description: None,
            agent: Some("claude".into()),
            from_discussion: Some("auth-scope".into()),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["name"], "add-rate-limit");
        assert_eq!(json["fromDiscussion"], "auth-scope");
        assert_eq!(json["agent"], "claude");
        assert!(
            json.get("description").is_none(),
            "absent optional fields are omitted: {json}"
        );
        let back: CreateChangeRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn create_change_response_tolerates_server_extras_being_absent() {
        let resp: CreateChangeResponse =
            serde_json::from_str(r#"{"name":"demo"}"#).unwrap();
        assert_eq!(resp.name, "demo");
        assert_eq!(resp.schema, None);
        let full: CreateChangeResponse = serde_json::from_str(
            r#"{"name":"demo","schema":"spec-driven","repo":"backend","lifecycle":"drafting"}"#,
        )
        .unwrap();
        assert_eq!(full.schema.as_deref(), Some("spec-driven"));
        assert_eq!(full.lifecycle.as_deref(), Some("drafting"));
    }

    #[test]
    fn review_round_dto_carries_nullable_phase_and_patch_hash() {
        // review-station spec「審查工單的讀取」：rounds[] 增列 phase／patchHash
        // string|null；legacy payload（無兩欄）解析為 None、序列化為明確 null。
        let structured: ReviewRoundDto = serde_json::from_str(
            r#"{"index":1,"scope":["src/lib.rs"],"findings":[],"phase":"discovery","patchHash":"sha256:aa"}"#,
        )
        .unwrap();
        let json = serde_json::to_value(&structured).unwrap();
        assert_eq!(json["phase"], "discovery");
        assert_eq!(json["patchHash"], "sha256:aa");

        let legacy: ReviewRoundDto =
            serde_json::from_str(r#"{"index":1,"scope":[],"findings":[]}"#).unwrap();
        let json = serde_json::to_value(&legacy).unwrap();
        assert!(
            json.as_object().unwrap().get("phase").is_some_and(serde_json::Value::is_null),
            "phase must serialize as explicit null: {json}"
        );
        assert!(
            json.as_object().unwrap().get("patchHash").is_some_and(serde_json::Value::is_null),
            "patchHash must serialize as explicit null: {json}"
        );
    }

    #[test]
    fn put_artifact_shapes_round_trip() {
        let req = PutArtifactRequest { content: "## Why\n".into() };
        assert_eq!(
            serde_json::to_string(&req).unwrap(),
            r###"{"content":"## Why\n"}"###,
            "request body is exactly the content envelope"
        );
        let resp: PutArtifactResponse =
            serde_json::from_str(r#"{"artifact":"design","version":8}"#).unwrap();
        assert_eq!(resp.artifact, "design");
        assert_eq!(resp.version, 8);
    }

    #[test]
    fn task_done_request_omits_empty_touched_files() {
        let empty = TaskDoneRequest { touched_files: vec![] };
        assert_eq!(
            serde_json::to_string(&empty).unwrap(),
            "{}",
            "empty attribution serializes as the bare object, matching the current body"
        );
        let some = TaskDoneRequest { touched_files: vec!["src/lib.rs".into()] };
        let json = serde_json::to_value(&some).unwrap();
        assert_eq!(json["touchedFiles"][0], "src/lib.rs");
        let back: TaskDoneRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, some);
    }

    #[test]
    fn task_responses_default_their_flags() {
        let done: TaskDoneResponse =
            serde_json::from_str(r#"{"taskDesc":"1.1 First"}"#).unwrap();
        assert_eq!(done.task_desc, "1.1 First");
        assert!(!done.already_done);
        let dup: TaskDoneResponse =
            serde_json::from_str(r#"{"taskDesc":"1.1 First","alreadyDone":true}"#).unwrap();
        assert!(dup.already_done);
        let undone: TaskUndoneResponse =
            serde_json::from_str(r#"{"taskDesc":"1.1 First","alreadyUndone":true}"#).unwrap();
        assert!(undone.already_undone);
    }

    #[test]
    fn archive_response_lists_updated_capabilities() {
        let resp: ArchiveResponse = serde_json::from_str(
            r#"{"specs":[{"capability":"user-auth"},{"capability":"rate-limit"}]}"#,
        )
        .unwrap();
        let caps: Vec<&str> = resp.specs.iter().map(|s| s.capability.as_str()).collect();
        assert_eq!(caps, ["user-auth", "rate-limit"]);
        let empty: ArchiveResponse = serde_json::from_str("{}").unwrap();
        assert!(empty.specs.is_empty(), "specs defaults to empty");
    }

    #[test]
    fn discussion_write_shapes_round_trip_in_camel_case() {
        let round = AddDiscussionRoundRequest {
            mode: "assumptions".into(),
            content: "…".into(),
        };
        let json = serde_json::to_value(&round).unwrap();
        assert_eq!(json["mode"], "assumptions");
        let round_resp: AddDiscussionRoundResponse =
            serde_json::from_str(r#"{"round":3}"#).unwrap();
        assert_eq!(round_resp.round, 3);

        let created: CreateDiscussionResponse = serde_json::from_str(
            r#"{"slug":"auth-scope","topic":"Auth scope","path":"discussions/auth-scope.md"}"#,
        )
        .unwrap();
        assert_eq!(created.slug, "auth-scope");

        let archived: ArchiveDiscussionResponse =
            serde_json::from_str(r#"{"archivedTo":"discussions/archive/auth-scope.md"}"#).unwrap();
        assert_eq!(archived.archived_to, "discussions/archive/auth-scope.md");

        let promote_none = PromoteDiscussionRequest { name: None };
        assert_eq!(
            serde_json::to_string(&promote_none).unwrap(),
            "{}",
            "promote without an explicit name posts the bare object"
        );
        let promoted: PromoteDiscussionResponse =
            serde_json::from_str(r#"{"change":"add-auth"}"#).unwrap();
        assert_eq!(promoted.change, "add-auth");
    }

    #[test]
    fn create_discussion_request_slug_is_optional_and_camel_case() {
        let bare = CreateDiscussionRequest { topic: "看板搜尋列".into(), slug: None };
        assert_eq!(
            serde_json::to_string(&bare).unwrap(),
            r#"{"topic":"看板搜尋列"}"#,
            "absent slug is omitted so request bodies stay byte-identical to old clients'"
        );

        let with_slug = CreateDiscussionRequest {
            topic: "看板搜尋列".into(),
            slug: Some("board-search-bar".into()),
        };
        let json = serde_json::to_value(&with_slug).unwrap();
        assert_eq!(json["slug"], "board-search-bar");
        let back: CreateDiscussionRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, with_slug);

        let legacy: CreateDiscussionRequest =
            serde_json::from_str(r#"{"topic":"Auth scope"}"#).unwrap();
        assert_eq!(legacy.slug, None, "old payloads without slug still deserialize");
    }

    #[test]
    fn review_shapes_round_trip_in_camel_case() {
        // D4a wire 契約：lastRound 為 camelCase；stamp 請求載明 accept／agent／scope。
        let round = ReviewRoundDto {
            index: 2,
            phase: Some("validation".into()),
            patch_hash: Some(format!("sha256:{}", "a".repeat(64))),
            scope: vec!["src/lib.rs".into()],
            findings: vec![ReviewFindingDto {
                severity: "WARNING".into(),
                path: "src/lib.rs".into(),
                text: "possible Feature Envy".into(),
            }],
        };
        let resp =
            ReviewTicketResponse { change: "demo".into(), rounds: vec![round.clone()], last_round: round };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("lastRound").is_some(), "camelCase lastRound: {json}");
        assert_eq!(json["rounds"][0]["findings"][0]["severity"], "WARNING");
        let back: ReviewTicketResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, resp);

        let req = StampReviewRequest {
            accept: true,
            agent: Some("claude".into()),
            scope: vec![ReviewScopeEntryDto { path: "src/lib.rs".into(), hash: "abc".into() }],
            missing: vec!["src/gone.rs".into()],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["scope"][0]["path"], "src/lib.rs");
        assert_eq!(json["missing"][0], "src/gone.rs");
        let back: StampReviewRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, req);
        // 省略欄位的預設：accept=false、scope=[]、missing=[]（GET 後空輪蓋章的
        // 最小請求；舊 client 不帶 missing 即原嚴格集合相等）。
        let minimal: StampReviewRequest = serde_json::from_str("{}").unwrap();
        assert!(!minimal.accept);
        assert!(minimal.scope.is_empty());
        assert!(minimal.missing.is_empty());
    }

    #[test]
    fn command_dtos_export_json_schema() {
        for (name, schema) in [
            ("CreateChangeRequest", schemars::schema_for!(CreateChangeRequest)),
            ("PutArtifactResponse", schemars::schema_for!(PutArtifactResponse)),
            ("TaskDoneRequest", schemars::schema_for!(TaskDoneRequest)),
            ("ArchiveResponse", schemars::schema_for!(ArchiveResponse)),
            ("ReviewTicketResponse", schemars::schema_for!(ReviewTicketResponse)),
            ("StampReviewRequest", schemars::schema_for!(StampReviewRequest)),
        ] {
            let text = serde_json::to_string(&schema)
                .unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains("properties"), "{name} schema has properties: {text}");
        }
        let create = serde_json::to_string(&schemars::schema_for!(CreateChangeRequest)).unwrap();
        assert!(
            create.contains("fromDiscussion"),
            "schema fields are camelCase: {create}"
        );
    }
}

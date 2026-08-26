//! Query and command route handlers. Every handler runs its verb through the
//! bridge (the canonical Host → Engine → TeamStore path), converts the typed
//! Engine outcome into a speclink-protocol DTO, and attaches the scope ETag.

use crate::auth::Binding;
use crate::error::ApiError;
use crate::events::Subscription;
use crate::state::AppState;
use crate::verb;
use axum::extract::{Path, Query, State};
use axum::http::header::ETAG;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use speclink_core::command::{Command, CommandOutcome, InstructionsOutcome};
use speclink_core::config::WorkflowConfig;
use speclink_core::discuss::DiscussionInfo as EngineDiscussionInfo;
use speclink_core::instructions as engine;
use speclink_core::listing::ListChangeJson;
use speclink_core::status::StatusReport;
use speclink_host::drift as host_drift;
use speclink_protocol::command::{
    AddDiscussionRoundRequest, AddDiscussionRoundResponse, AddReviewRoundRequest,
    AddReviewRoundResponse, ArchiveDiscussionResponse, ArchiveResponse, ArchivedDiscussion,
    ArchivedSpec, BindDiscussionRequest, BindDiscussionResponse, ClaimResponse,
    ConcludeDiscussionRequest, ConcludeDiscussionResponse, CreateChangeRequest,
    CreateChangeResponse, CreateDiscussionRequest, CreateDiscussionResponse,
    DiscardDiscussionResponse, DiscardResponse, DiscardReviewResponse, InProgressRemoveResponse,
    MoveTaskRequest, MoveTaskResponse, PromoteDiscussionRequest, PromoteDiscussionResponse,
    PutArtifactRequest, PutArtifactResponse, ReviewFindingDto, ReviewRoundDto,
    ReviewTicketResponse, SetDiscussionContextRequest, StampReviewRequest, StampReviewResponse,
    TaskDoneRequest, TaskDoneResponse, TaskUndoneResponse, UnlinkedDiscussion,
};
use speclink_protocol::drift::SpecDriftResponse;
use speclink_protocol::events::InvalidationEvent;
use speclink_protocol::query::{
    AnalyzeDimension, AnalyzeFinding, AnalyzeMsg, AnalyzeReportResponse, ApplyInstructions,
    ArtifactContent, ArtifactInstructions, ArtifactStatus, BoardOrderResponse,
    ChangeEvidenceResponse, ChangeStatus,
    ChangeSummary, ConfigResponse, DependencyEntry, DiscussionInfo, ImportBundle, ImportDocumentId,
    ImportDocumentOutcome, ImportReportResponse, ImportedDocument, LanguageResponse,
    ListChangesResponse, ListDiscussionsResponse, ListSpecsResponse, Progress,
    PutBoardOrderRequest, PutBoardOrderResponse, PutConfigRequest, PutConfigResponse,
    ShowDiscussionResponse, SpecSummary, TaskEntry, ValidateChangeResponse, WhoamiRepo,
    WhoamiResponse, WhoamiUser,
};
use speclink_store::{
    content_digest, Bundle, BundleDoc, CommandContext, DocumentId, EventRecord, ImportMode,
    ImportOutcome, Revision, StoreError, BUNDLE_FORMAT_VERSION,
};
use std::convert::Infallible;
use tokio::sync::broadcast::error::RecvError;

/// An acknowledgment body for verbs whose response the client ignores.
#[derive(Serialize)]
struct Ack {}

/// A JSON response carrying the scope ETag.
fn ok<T: Serialize>(dto: T, etag: &str) -> Response {
    ([(ETAG, etag.to_string())], Json(dto)).into_response()
}

/// The unexpected-outcome guard: the command runtime returned an outcome kind
/// this route never asks for. A server bug, not a client error.
fn wrong_outcome(route: &str) -> ApiError {
    ApiError::internal(format!("{route}: unexpected command outcome"))
}

// --- queries ---

/// `GET /changes`
pub async fn list_changes(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::List {
            sort: "name".to_string(),
            specs: false,
            changes: false,
            worktrees: Default::default(),
        },
    )
    .await?;
    let changes = match result.execution.outcome {
        CommandOutcome::List(list) => list.changes.unwrap_or_default(),
        _ => return Err(wrong_outcome("list")),
    };
    // started 站（change-lifecycle spec）與建立者／來源討論欄位
    // （remote-read-parity design D4）：引擎的 list item 凍結不帶這些 meta
    // 欄位（fs `list --json` parity pin），wire 的 startedAt／createdBy／
    // created／fromDiscussions 由這裡讀各 change meta 沿同一條路徑組裝。
    // 壞 meta 已由 metaError 診斷，這裡對解析失敗維持缺席、清單不失敗。
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let names: Vec<String> = changes.iter().map(|c| c.name.clone()).collect();
    let metas: std::collections::HashMap<String, speclink_core::model::ChangeMeta> =
        tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
            let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
            let mut map = std::collections::HashMap::new();
            for name in names {
                let doc = snapshot
                    .read(&DocumentId::ChangeMeta {
                        change: name.clone(),
                    })
                    .map_err(ApiError::from)?;
                let Some(doc) = doc else { continue };
                if let Ok(meta) =
                    speclink_core::model::ChangeMeta::from_text(Some(&doc.content))
                {
                    map.insert(name, meta);
                }
            }
            Ok(map)
        })
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    let dto = ListChangesResponse {
        changes: changes
            .into_iter()
            .map(|c| {
                let meta = metas.get(&c.name);
                change_summary(c, meta)
            })
            .collect(),
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /changes/{name}/in-progress` — Command::InProgressAdd 直通（design
/// D5）：首蓋以呼叫者認證身分寫 started_at/started_by；重複與未知名稱維持
/// 引擎的靜默成功語意（HTTP 200、零寫入——引擎 outcome 未蓋章時 bridge 不
/// commit、不發事件）。
pub async fn in_progress(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::InProgressAdd { name }).await?;
    match result.execution.outcome {
        CommandOutcome::InProgressAdd(_) => Ok(ok(Ack {}, &result.etag)),
        _ => Err(wrong_outcome("in-progress")),
    }
}

/// `DELETE /changes/{name}/in-progress` — Command::InProgressRemove 直通:與
/// POST 同資源、反向語意(D4)。零痕跡移除 200 Ack(bridge commit 發退回事
/// 件);未開工冪等 200(引擎零寫入,bridge 不 commit、不發事件);有工作痕
/// 跡 409,證據欄位(checkedTasks/touchedFiles)隨錯誤封套 flatten 輸出;
/// 未知 change 404。
pub async fn in_progress_remove(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::InProgressRemove { name }).await?;
    match result.execution.outcome {
        // `removed` 區分實際移除與未開工冪等——兩者印不同的行，遠端也該分得出。
        CommandOutcome::InProgressRemove(o) => {
            Ok(ok(InProgressRemoveResponse { removed: o.removed }, &result.etag))
        }
        _ => Err(wrong_outcome("in-progress-remove")),
    }
}

/// `GET /changes/{name}`
pub async fn get_change(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::Status {
            change: Some(name.clone()),
            schema: None,
        },
    )
    .await?;
    let report = match result.execution.outcome {
        CommandOutcome::Status(report) => report,
        _ => return Err(wrong_outcome("status")),
    };
    // show 組合的 meta 欄位（design D4 實作期修正）：created 沿 ShowChange 的
    // schema+created 成對規則、fromDiscussions 自 meta、deltaCapabilities 自
    // scope 文件列舉（export 與每個 verb 的 bridge 物化同成本級）；歸屬四欄
    // createdBy/createdWith/startedAt/startedBy 亦自同一份 parsed meta 補上
    // （remote-read-parity「單 change 讀取回應攜帶 show 組合欄位」擴充）。
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let (meta, delta_capabilities) =
        tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
            let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
            let meta = snapshot
                .read(&DocumentId::ChangeMeta {
                    change: name.clone(),
                })
                .map_err(ApiError::from)?
                .and_then(|doc| {
                    speclink_core::model::ChangeMeta::from_text(Some(&doc.content)).ok()
                });
            let prefix = "specs/";
            let mut caps: Vec<String> = store
                .export(&scope)
                .map_err(ApiError::from)?
                .documents
                .iter()
                .filter_map(|entry| match &entry.doc {
                    DocumentId::ChangeArtifact { change, artifact } if *change == name => artifact
                        .strip_prefix(prefix)
                        .and_then(|rest| rest.strip_suffix("/spec.md"))
                        .filter(|cap| !cap.is_empty() && !cap.contains('/'))
                        .map(str::to_string),
                    _ => None,
                })
                .collect();
            caps.sort();
            Ok((meta, caps))
        })
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    let mut dto = change_status(report);
    if let Some(meta) = meta {
        // 成對規則：schema 與 created 同時存在才回報 created。
        dto.created = meta.schema.is_some().then_some(meta.created.clone()).flatten();
        dto.from_discussions = meta.from_discussions();
        dto.created_by = meta.created_by;
        dto.created_with = meta.created_with;
        dto.started_at = meta.started_at;
        dto.started_by = meta.started_by;
    }
    dto.delta_capabilities = delta_capabilities;
    Ok(ok(dto, &result.etag))
}

/// `GET /changes/{name}/drift` — the change's spec-side drift over one store
/// snapshot, with that snapshot's basis digests. The Host's `spec_drift` entry
/// owns the composition (server-drift-api design 決策 2), so both halves come
/// from the same snapshot. Workspace facts stay the client's own business: the
/// Server runs no git, and the wire gives it no field to claim one in. The
/// computation is a read — no unit of work, no event.
pub async fn drift(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    // The report and the ETag are read in the same blocking hop, as a verb's are.
    let (payload, etag) =
        tokio::task::spawn_blocking(move || -> Result<(SpecDriftResponse, String), ApiError> {
            let view = host_drift::spec_drift(store.as_ref(), &scope, &name)?;
            let etag = verb::scope_token(store.as_ref(), &scope)?;
            Ok((host_drift::spec_drift_view_to_wire(&view), etag))
        })
        .await
        .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    Ok(ok(payload, &etag))
}

/// `GET /changes/{name}/validate` — a read-only derived query through the
/// Command gateway (server-verb-api 決策 1)，與 drift 端點同形。端點固定單
/// change（決策 2：CLI 的聚合語意由 client 組合）、非 strict；查詢無 commit，
/// scope revision 不前進、不發事件。
pub async fn validate_change(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::Validate {
            item: Some(name),
            all: false,
            changes: false,
            specs: false,
            strict: false,
        },
    )
    .await?;
    let mut results = match result.execution.outcome {
        CommandOutcome::Validate(v) => v.results,
        _ => return Err(wrong_outcome("validate")),
    };
    // 帶 item 的 Validate 恰回一筆。
    let r = results
        .pop()
        .ok_or_else(|| ApiError::internal("validate: empty result set"))?;
    let dto = ValidateChangeResponse {
        change: r.change,
        valid: r.valid,
        errors: r.errors,
        warnings: r.warnings,
    };
    Ok(ok(dto, &result.etag))
}

/// `GET /changes/{name}/analyze` — the engine's full AnalyzeReport as a
/// read-only derived query (server-verb-api 決策 1)。
pub async fn analyze_change(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::Analyze { change: Some(name) }).await?;
    let report = match result.execution.outcome {
        CommandOutcome::Analyze(report) => report,
        _ => return Err(wrong_outcome("analyze")),
    };
    Ok(ok(analyze_report(report), &result.etag))
}

fn analyze_msg(m: speclink_core::analyzer::Msg) -> AnalyzeMsg {
    AnalyzeMsg { key: m.key, params: m.params }
}

fn analyze_report(report: speclink_core::analyzer::AnalyzeReport) -> AnalyzeReportResponse {
    AnalyzeReportResponse {
        change_id: report.change_id,
        dimensions: report
            .dimensions
            .into_iter()
            .map(|d| AnalyzeDimension {
                dimension: d.dimension,
                status: d.status,
                finding_count: d.finding_count,
            })
            .collect(),
        findings: report
            .findings
            .into_iter()
            .map(|f| AnalyzeFinding {
                id: f.id,
                dimension: f.dimension,
                severity: f.severity,
                location: f.location,
                summary: f.summary,
                recommendation: f.recommendation,
                summary_msg: analyze_msg(f.summary_msg),
                recommendation_msg: analyze_msg(f.recommendation_msg),
            })
            .collect(),
        artifacts_analyzed: report.artifacts_analyzed,
        artifacts_missing: report.artifacts_missing,
    }
}

/// `DELETE /changes/{name}?force=` query input; force defaults to false.
#[derive(Deserialize)]
pub struct DeleteChangeQuery {
    #[serde(default)]
    force: bool,
}

/// `DELETE /changes/{name}` — Command::Discard 全語意（server-verb-api 決策
/// 3）：fail-closed meta 檢查、started-work guard（force=false 拒絕、reason
/// 機器可判為 refused）、來源討論 unlink、UoW 原子刪除；commit 的 outbox 事件
/// 讓 SSE invalidate 自動發生。editor 限定（決策 5）。
pub async fn delete_change(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Query(query): Query<DeleteChangeQuery>,
) -> Result<Response, ApiError> {
    // UI capability 只是提示；request-time role 檢查才是最終執行點。
    if !binding.editor {
        return Err(ApiError::forbidden("reader memberships cannot delete changes"));
    }
    let result = verb::run(
        &state,
        &binding,
        Command::Discard { change: name, force: query.force },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::Discard(o) => o,
        _ => return Err(wrong_outcome("discard")),
    };
    let dto = DiscardResponse {
        change: outcome.change_name,
        unlinked_discussions: outcome
            .unlinked_discussions
            .into_iter()
            .map(|(slug, status)| UnlinkedDiscussion { slug, status })
            .collect(),
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /changes/{name}/tasks/move` — Command::TaskMove（server-verb-api 決策
/// 4）：index 定址的搬移＋重編號，越界拒絕零副作用。editor 限定（決策 5）。
pub async fn move_task(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Json(req): Json<MoveTaskRequest>,
) -> Result<Response, ApiError> {
    if !binding.editor {
        return Err(ApiError::forbidden("reader memberships cannot move tasks"));
    }
    let result = verb::run(
        &state,
        &binding,
        Command::TaskMove { change: name, from: req.from, to: req.to, before: req.before },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::TaskMove(o) => o,
        _ => return Err(wrong_outcome("task move")),
    };
    let dto = MoveTaskResponse { change: outcome.change, description: outcome.description };
    Ok(ok(dto, &result.etag))
}

/// `GET /changes/{name}/instructions/{*artifact}` — `apply` yields the apply
/// view, any other artifact its instructions.
pub async fn instructions(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name, artifact)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::Instructions {
            artifact: Some(artifact),
            change: Some(name),
            schema: None,
        },
    )
    .await?;
    match result.execution.outcome {
        CommandOutcome::Instructions(InstructionsOutcome::Apply(apply)) => {
            Ok(ok(apply_instructions(apply), &result.etag))
        }
        CommandOutcome::Instructions(InstructionsOutcome::Artifact(artifact)) => {
            Ok(ok(artifact_instructions(artifact), &result.etag))
        }
        _ => Err(wrong_outcome("instructions")),
    }
}

/// `GET /changes/{name}/artifacts/{*artifact}`
pub async fn get_artifact(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name, artifact)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::ArtifactCat {
            artifact: artifact.clone(),
            change: Some(name.clone()),
        },
    )
    .await?;
    let content = match result.execution.outcome {
        CommandOutcome::ArtifactCat(content) => content,
        _ => return Err(wrong_outcome("artifact")),
    };
    // Stamp the version so a later If-Match write can CAS against it.
    let version = match artifact_rel_path(&artifact) {
        Some(rel) => verb::read_doc(
            &state,
            &binding,
            DocumentId::ChangeArtifact {
                change: name,
                artifact: rel,
            },
        )
        .await?
        .map(|doc| doc.revision.0)
        .unwrap_or(0),
        None => 0,
    };
    let dto = ArtifactContent {
        artifact,
        content,
        version,
    };
    Ok(ok(dto, &result.etag))
}

/// `GET /specs`
pub async fn list_specs(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::List {
            sort: "name".to_string(),
            specs: true,
            changes: false,
            worktrees: Default::default(),
        },
    )
    .await?;
    let specs_value = match result.execution.outcome {
        CommandOutcome::List(list) => list.specs.unwrap_or_else(|| serde_json::json!([])),
        _ => return Err(wrong_outcome("specs")),
    };
    let specs: Vec<SpecSummary> = serde_json::from_value(specs_value)
        .map_err(|e| ApiError::internal(format!("specs shape: {e}")))?;
    Ok(ok(ListSpecsResponse { specs }, &result.etag))
}

/// `GET /language`
pub async fn language(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::LanguageShow).await?;
    let content = match result.execution.outcome {
        CommandOutcome::Language(content) => content,
        _ => return Err(wrong_outcome("language")),
    };
    Ok(ok(LanguageResponse { content }, &result.etag))
}

/// `GET /config` — the workflow policy source and its scope revision, read from
/// one store snapshot so `revision` is exactly the ETag value.
pub async fn config(State(state): State<AppState>, binding: Binding) -> Result<Response, ApiError> {
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let (dto, etag) = tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        let revision = snapshot.revision().0;
        let content = snapshot
            .read(&DocumentId::WorkflowConfig)
            .map_err(ApiError::from)?
            .map(|doc| doc.content);
        let schema = WorkflowConfig::from_text(content.as_deref())
            .map_err(|e| ApiError::invalid_config(e.to_string()))?
            .schema_name();
        Ok((
            ConfigResponse {
                schema,
                content,
                revision,
            },
            format!("\"{revision}\""),
        ))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    Ok(ok(dto, &etag))
}

/// `PUT /config` — server-authoritative workflow policy write. Authorization
/// and engine parsing happen before any unit of work exists; only then is the
/// client's scope revision compared and the document committed with store CAS.
pub async fn put_config(
    State(state): State<AppState>,
    binding: Binding,
    Json(req): Json<PutConfigRequest>,
) -> Result<Response, ApiError> {
    // First defense: the UI capability is only a hint; this request-time role
    // decision is the final enforcement point.
    if !binding.policy_write {
        return Err(ApiError::forbidden(
            "reader memberships cannot write workflow policy",
        ));
    }

    // Second defense: parse the complete document through the engine config
    // model before a write is staged. A malformed document never reaches CAS.
    let parsed = WorkflowConfig::from_text(Some(&req.content))
        .map_err(|e| ApiError::invalid_config(e.to_string()))?;
    // Value-domain gate shares the engine rule with every client seam; the
    // server stays the final defense (client validation is UX only).
    speclink_core::config::validate_policy_locales(&speclink_core::config::WorkflowPolicyFields {
        locale: parsed.locale.clone(),
        spec_locale: parsed.spec_locale.clone(),
        ..Default::default()
    })
    .map_err(|e| ApiError::invalid_config(e.to_string()))?;

    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let actor = binding.actor.id.clone();
    let expected_revision = req.expected_revision;
    let content = req.content;
    let result = tokio::task::spawn_blocking(move || -> Result<(u64, String), ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        let actual_revision = snapshot.revision().0;
        if actual_revision != expected_revision {
            return Err(ApiError::revision_conflict(format!(
                "expected scope revision {expected_revision}, actual {actual_revision}"
            )));
        }
        let current = snapshot
            .read(&DocumentId::WorkflowConfig)
            .map_err(ApiError::from)?;
        drop(snapshot);

        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "put-config".into(),
                    actor,
                },
            )
            .map_err(ApiError::from)?;
        match current {
            Some(doc) => uow.update(DocumentId::WorkflowConfig, content, doc.revision),
            None => uow.create(DocumentId::WorkflowConfig, content),
        }
        let revision = store.commit(uow, Vec::new()).map_err(ApiError::from)?.0;
        Ok((revision, format!("\"{revision}\"")))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?;

    if result.is_ok() {
        state.events.notify(&verb::scope_of(&binding));
    }
    let (revision, etag) = result?;
    Ok(ok(PutConfigResponse { revision }, &etag))
}

/// `GET /board-order` — the scope's opaque board-order document, read from one
/// store snapshot so `revision` is exactly the ETag value. Absence is a normal
/// state: `content` is null and the ETag still carries the scope revision.
pub async fn board_order(
    State(state): State<AppState>,
    binding: Binding,
) -> Result<Response, ApiError> {
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let (dto, etag) = tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        let revision = snapshot.revision().0;
        let content = snapshot
            .read(&DocumentId::BoardOrder)
            .map_err(ApiError::from)?
            .map(|doc| doc.content);
        Ok((
            BoardOrderResponse { content, revision },
            format!("\"{revision}\""),
        ))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    Ok(ok(dto, &etag))
}

/// `GET /changes/{name}/evidence` — the change's recorded completion evidence,
/// read from one store snapshot. Absence is a normal state: a change that never
/// recorded any answers with an empty set, so a reader never has to tell
/// "no evidence" apart from "no change" by error code.
pub async fn change_evidence(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let (dto, etag) = tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        let revision = snapshot.revision().0;
        let text = snapshot
            .read(&DocumentId::ChangeEvidence { change: name })
            .map_err(ApiError::from)?
            .map(|doc| doc.content);
        // The store keeps the record opaque; the shape is the engine's, and a
        // record this server cannot parse is corruption, not an empty set —
        // deliberately louder than the engine's own lenient read (which treats
        // a corrupt record as empty so a completion is never blocked): a query
        // face answering "no evidence" over a corrupt record would be a lie.
        let entries = match text {
            Some(text) => serde_json::from_str::<StoredEvidence>(&text)
                .map_err(|e| ApiError::internal(format!("evidence record is unreadable: {e}")))?
                .entries,
            None => Vec::new(),
        };
        Ok((ChangeEvidenceResponse { entries }, format!("\"{revision}\"")))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))??;
    Ok(ok(dto, &etag))
}

/// The stored evidence record, as far as this endpoint reads it: the v2 entry
/// list. The v1 file-list channel and the version marker are deliberately not
/// part of this response — v1 entries carry no actor/recordedAt and this
/// endpoint never fabricates them; a v1-only record (a migration import can
/// carry one) reads as an empty set here while the engine's own consumers
/// (`all_files`, the in-progress-remove gate) still see its files.
/// The field shape mirrors `speclink_core::tasks::EvidenceEntry`; the sync is
/// pinned by `evidence_wire_shape_matches_the_engine_record` below.
#[derive(serde::Deserialize)]
struct StoredEvidence {
    #[serde(default)]
    entries: Vec<speclink_protocol::query::EvidenceEntry>,
}

/// `PUT /board-order` — full-replacement CAS write of the opaque board
/// resource, along the put_config shape: role check, `If-Match` against the
/// scope revision, then store CAS. The content is a presentation resource the
/// server never parses; the commit carries an event so subscribers re-read.
pub async fn put_board_order(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
    Json(req): Json<PutBoardOrderRequest>,
) -> Result<Response, ApiError> {
    // The UI capability is only a hint; this request-time role decision is
    // the final enforcement point.
    if !binding.policy_write {
        return Err(ApiError::forbidden(
            "reader memberships cannot write the board order",
        ));
    }

    // The board-order document is a small rank map; anything near this cap
    // is a malfunctioning client, refused before any write is staged.
    const BOARD_ORDER_CONTENT_CAP_BYTES: usize = 1024 * 1024;
    if req.content.len() > BOARD_ORDER_CONTENT_CAP_BYTES {
        return Err(ApiError::payload_too_large(format!(
            "board order content exceeds the {BOARD_ORDER_CONTENT_CAP_BYTES}-byte cap"
        )));
    }

    let expected_revision = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.trim().trim_matches('"').parse::<u64>().ok())
        .ok_or_else(|| {
            ApiError::invalid_argument("If-Match must carry the scope revision ETag")
        })?;

    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let actor = binding.actor.id.clone();
    let content = req.content;
    let result = tokio::task::spawn_blocking(move || -> Result<(u64, String), ApiError> {
        let snapshot = store.snapshot(&scope).map_err(ApiError::from)?;
        let actual_revision = snapshot.revision().0;
        if actual_revision != expected_revision {
            return Err(ApiError::revision_conflict(format!(
                "expected scope revision {expected_revision}, actual {actual_revision}"
            )));
        }
        let current = snapshot
            .read(&DocumentId::BoardOrder)
            .map_err(ApiError::from)?;
        drop(snapshot);

        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "put-board-order".into(),
                    actor: actor.clone(),
                },
            )
            .map_err(ApiError::from)?;
        match current {
            Some(doc) => uow.update(DocumentId::BoardOrder, content, doc.revision),
            None => uow.create(DocumentId::BoardOrder, content),
        }
        // The event lands in the outbox so subscribed clients receive an
        // invalidation and re-read; put_config has no such need.
        let event = EventRecord {
            name: "board-order-updated".into(),
            payload: serde_json::json!({}),
            actor,
            at: chrono::Utc::now(),
        };
        let revision = store.commit(uow, vec![event]).map_err(ApiError::from)?.0;
        Ok((revision, format!("\"{revision}\"")))
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?;

    if result.is_ok() {
        state.events.notify(&verb::scope_of(&binding));
    }
    let (revision, etag) = result?;
    Ok(ok(PutBoardOrderResponse { revision }, &etag))
}

/// `POST /import` — local-to-remote migration into one empty bound scope.
/// The wire has no operation selector; the handler always invokes CreateNew.
pub async fn import_bundle(
    State(state): State<AppState>,
    binding: Binding,
    Json(request): Json<ImportBundle>,
) -> Result<Response, ApiError> {
    if !binding.policy_write {
        return Err(ApiError::forbidden(
            "reader memberships cannot import a workspace",
        ));
    }

    if request.format_version != BUNDLE_FORMAT_VERSION {
        return Err(ApiError::refused(format!(
            "unsupported bundle format version {} (supported: {})",
            request.format_version, BUNDLE_FORMAT_VERSION
        )));
    }

    let scope = verb::scope_of(&binding);
    if request.scope.project != scope.project.as_str() || request.scope.repo != scope.repo.as_str()
    {
        return Err(ApiError::refused(format!(
            "bundle scope {}/{} does not match authenticated binding {}/{}",
            request.scope.project,
            request.scope.repo,
            scope.project.as_str(),
            scope.repo.as_str()
        )));
    }

    let mut documents = Vec::with_capacity(request.documents.len());
    for document in request.documents {
        let actual_digest = content_digest(&document.content);
        if actual_digest != document.digest {
            return Err(ApiError::invalid_argument(format!(
                "bundle digest mismatch for {:?}",
                document.document
            )));
        }
        documents.push(BundleDoc {
            doc: import_document_id(document.document),
            content: document.content,
            digest: document.digest,
        });
    }

    let bundle = Bundle {
        format_version: request.format_version,
        scope: scope.clone(),
        project_revision: Revision(request.project_revision),
        documents,
    };
    let store = state.store.clone();
    let report = tokio::task::spawn_blocking(move || {
        store
            .import(bundle, ImportMode::CreateNew)
            .map_err(import_store_error)
    })
    .await
    .map_err(|error| ApiError::internal(format!("blocking task failed: {error}")))??;

    let project_revision = report.project_revision.0;
    let documents = report
        .documents
        .into_iter()
        .map(|document| {
            if document.outcome != ImportOutcome::Created {
                return Err(ApiError::internal(
                    "CreateNew import returned an impossible overwrite outcome",
                ));
            }
            Ok(ImportedDocument {
                document: store_document_id(document.doc),
                outcome: ImportDocumentOutcome::Created,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    state.events.notify(&scope);
    Ok(ok(
        ImportReportResponse {
            project_revision,
            documents,
        },
        &format!("\"{project_revision}\""),
    ))
}

fn import_store_error(error: StoreError) -> ApiError {
    match error {
        StoreError::Backend { source } if source.contains("import (create-new)") => {
            ApiError::refused(source)
        }
        other => ApiError::from(other),
    }
}

fn import_document_id(document: ImportDocumentId) -> DocumentId {
    match document {
        ImportDocumentId::ChangeMeta { change } => DocumentId::ChangeMeta { change },
        ImportDocumentId::ChangeArtifact { change, artifact } => {
            DocumentId::ChangeArtifact { change, artifact }
        }
        ImportDocumentId::ChangeEvidence { change } => DocumentId::ChangeEvidence { change },
        ImportDocumentId::CanonicalSpec { capability } => DocumentId::CanonicalSpec { capability },
        ImportDocumentId::Discussion { slug, archived } => {
            DocumentId::Discussion { slug, archived }
        }
        ImportDocumentId::WorkflowConfig => DocumentId::WorkflowConfig,
        ImportDocumentId::ArchivedChange { change, doc } => {
            DocumentId::ArchivedChange { change, doc }
        }
        ImportDocumentId::Language => DocumentId::Language,
        ImportDocumentId::BoardOrder => DocumentId::BoardOrder,
    }
}

fn store_document_id(document: DocumentId) -> ImportDocumentId {
    match document {
        DocumentId::ChangeMeta { change } => ImportDocumentId::ChangeMeta { change },
        DocumentId::ChangeArtifact { change, artifact } => {
            ImportDocumentId::ChangeArtifact { change, artifact }
        }
        DocumentId::ChangeEvidence { change } => ImportDocumentId::ChangeEvidence { change },
        DocumentId::CanonicalSpec { capability } => ImportDocumentId::CanonicalSpec { capability },
        DocumentId::Discussion { slug, archived } => {
            ImportDocumentId::Discussion { slug, archived }
        }
        DocumentId::WorkflowConfig => ImportDocumentId::WorkflowConfig,
        DocumentId::ArchivedChange { change, doc } => {
            ImportDocumentId::ArchivedChange { change, doc }
        }
        DocumentId::Language => ImportDocumentId::Language,
        DocumentId::BoardOrder => ImportDocumentId::BoardOrder,
    }
}

/// `GET /whoami` — the authenticated identity and the project's repos, from the
/// binding.
pub async fn whoami(State(state): State<AppState>, binding: Binding) -> Result<Response, ApiError> {
    let etag = verb::scope_etag(&state, &binding).await?;
    let repos = state
        .identity
        .list_repos(&binding.project.key)
        .map_err(|_| ApiError::internal("identity store unavailable"))?;
    let dto = WhoamiResponse {
        user: WhoamiUser {
            name: binding.actor.display.clone(),
            handle: binding.actor.id.clone(),
        },
        repos: repos
            .iter()
            .map(|r| WhoamiRepo {
                name: r.key.clone(),
                git_url: String::new(),
            })
            .collect(),
    };
    Ok(ok(dto, &etag))
}

/// `GET /sync-state` — the scope state token for change polling (design 決策
/// 五). With `If-None-Match` matching the current ETag the scope is unchanged
/// (304); otherwise the new ETag is returned (200). A dropped-events client
/// converges by polling this and re-reading through Query.
pub async fn sync_state(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let etag = verb::scope_etag(&state, &binding).await?;
    let unchanged = headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(|inm| inm == etag)
        .unwrap_or(false);
    if unchanged {
        return Ok((StatusCode::NOT_MODIFIED, [(ETAG, etag)]).into_response());
    }
    Ok((StatusCode::OK, [(ETAG, etag)], Json(Ack {})).into_response())
}

/// `GET /events` — the project-scoped SSE invalidation stream (server-event-stream
/// spec). The `Binding` extractor has already run the same bearer/binding
/// precondition as every route, so an unauthenticated or non-member request
/// never reaches here. A `Last-Event-ID` below the cleaned floor gets a reset
/// frame first; a resumable one backfills the gap; an idle stream is kept alive
/// by comment heartbeats. Each hint carries only the outbox sequence, scope,
/// resource id, and revision — never document content.
pub async fn events(
    State(state): State<AppState>,
    binding: Binding,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let heartbeat = state.events.settings().heartbeat;
    let scope = verb::scope_of(&binding);
    let Subscription { mut rx, plan } = state.events.subscribe(&scope, last_event_id)?;

    let stream = async_stream::stream! {
        if plan.reset {
            yield Ok::<Event, Infallible>(reset_event());
        }
        // Everything at or below `last` is already delivered (backfill or
        // history), so the live tail skips it — no gap, no repeat.
        let mut last = plan.cursor;
        for hint in plan.backfill {
            if let Ok(seq) = hint.event_id.parse::<u64>() {
                last = seq;
            }
            yield Ok(invalidation_event(&hint));
        }
        loop {
            match rx.recv().await {
                Ok(item) => {
                    if item.seq > last {
                        last = item.seq;
                        yield Ok(invalidation_event(&item.event));
                    }
                }
                // A slow consumer that overflowed its buffer is dropped; it
                // reconnects with Last-Event-ID and resumes.
                Err(RecvError::Lagged(_)) | Err(RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(heartbeat).text("heartbeat"))
        .into_response())
}

/// One invalidation hint as an SSE event: `id` is the outbox sequence (so a
/// client's `Last-Event-ID` resumes from it), the event type is `invalidate`,
/// and the data is the DTO JSON.
fn invalidation_event(hint: &InvalidationEvent) -> Event {
    Event::default()
        .id(hint.event_id.clone())
        .event("invalidate")
        .json_data(hint)
        .unwrap_or_else(|_| Event::default().comment("invalidation serialize failed"))
}

/// The reset signal — a distinct SSE event type telling the client its cursor
/// is cleaned; it converges by re-reading through Query + ETag.
fn reset_event() -> Event {
    Event::default()
        .event("reset")
        .data("cursor expired; re-read via query")
}

// --- change commands ---

/// `POST /changes`
pub async fn create_change(
    State(state): State<AppState>,
    binding: Binding,
    Json(req): Json<CreateChangeRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::NewChange {
            name: req.name,
            description: req.description,
            schema: req.schema,
            agent: req.agent,
            from_discussion: req.from_discussion,
        },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::NewChange(o) => o,
        _ => return Err(wrong_outcome("create-change")),
    };
    let dto = CreateChangeResponse {
        name: outcome.name,
        schema: Some(outcome.schema),
        repo: Some(binding.repo.clone()),
        lifecycle: None,
    };
    Ok(ok(dto, &result.etag))
}

/// `PUT /changes/{name}/artifacts/{*artifact}` with the `If-Match` write
/// precondition. The write commits atomically through the bridge; a stale
/// precondition is a 409 revision_conflict with no partial write.
pub async fn put_artifact(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name, artifact)): Path<(String, String, String)>,
    headers: HeaderMap,
    Json(req): Json<PutArtifactRequest>,
) -> Result<Response, ApiError> {
    let if_match = parse_if_match(&headers)?;
    let rel = artifact_rel_path(&artifact)
        .ok_or_else(|| ApiError::invalid_argument(format!("unknown artifact '{artifact}'")))?;
    let cmd = artifact_write_command(&artifact, name.clone(), req.content)?;
    let doc = DocumentId::ChangeArtifact {
        change: name,
        artifact: rel,
    };
    let (version, etag) =
        verb::run_write_with_if_match(&state, &binding, doc, if_match, cmd).await?;
    Ok(ok(PutArtifactResponse { artifact, version }, &etag))
}

// --- 品質站（design D4a：動詞端點承載 remote，引擎守門與原子性隨 Command 而來）---

fn review_round_dto(r: &speclink_core::station::Round) -> ReviewRoundDto {
    ReviewRoundDto {
        index: r.index as u64,
        phase: r.phase.map(|p| p.as_str().to_string()),
        patch_hash: r.patch_hash.clone(),
        scope: r.scope.clone(),
        findings: r
            .findings
            .iter()
            .map(|f| ReviewFindingDto {
                severity: f.severity.as_str().to_string(),
                path: f.path.clone(),
                text: f.text.clone(),
            })
            .collect(),
    }
}

/// `GET /changes/{name}/review`
pub async fn review_show(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::ReviewShow { change: name }).await?;
    let o = match result.execution.outcome {
        CommandOutcome::ReviewShow(o) => o,
        _ => return Err(wrong_outcome("review-show")),
    };
    let last_round = review_round_dto(o.ticket.last_round());
    let rounds: Vec<ReviewRoundDto> = o.ticket.rounds.iter().map(review_round_dto).collect();
    Ok(ok(
        ReviewTicketResponse { change: o.change, rounds, last_round, content: o.content },
        &result.etag,
    ))
}

/// `POST /changes/{name}/review/rounds`
pub async fn review_add_round(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Json(req): Json<AddReviewRoundRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::ReviewAddRound { change: name, content: req.content },
    )
    .await?;
    let round = match result.execution.outcome {
        CommandOutcome::ReviewAddRound(o) => o.round,
        _ => return Err(wrong_outcome("review-add-round")),
    };
    Ok(ok(AddReviewRoundResponse { round: round as u64 }, &result.etag))
}

/// `POST /changes/{name}/review/stamp` — the submitted fingerprints are the
/// caller's work-tree truth; the engine validates the path set, never re-hashes.
pub async fn review_stamp(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Json(req): Json<StampReviewRequest>,
) -> Result<Response, ApiError> {
    // 蓋章同樣以刪掉工單收場——editor 限定比照 discard，只擋 DELETE 守不住。
    if !binding.editor {
        return Err(ApiError::forbidden("reader memberships cannot stamp reviews"));
    }
    let scope = req
        .scope
        .into_iter()
        .map(|e| speclink_core::model::ReviewedScopeEntry { path: e.path, hash: e.hash })
        .collect();
    let result = verb::run(
        &state,
        &binding,
        Command::ReviewStamp { change: name, accept: req.accept, tool: req.agent, scope, missing: req.missing },
    )
    .await?;
    let change = match result.execution.outcome {
        CommandOutcome::ReviewStamp(o) => o.change,
        _ => return Err(wrong_outcome("review-stamp")),
    };
    Ok(ok(StampReviewResponse { change }, &result.etag))
}

/// `DELETE /changes/{name}/review` — editor 限定比照 change 刪除：工單是審查
/// 過程的唯一紀錄，刪掉不可回復。
pub async fn review_discard(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    if !binding.editor {
        return Err(ApiError::forbidden(
            "reader memberships cannot discard review tickets",
        ));
    }
    let result = verb::run(&state, &binding, Command::ReviewDiscard { change: name }).await?;
    let change = match result.execution.outcome {
        CommandOutcome::ReviewDiscard(o) => o.change,
        _ => return Err(wrong_outcome("review-discard")),
    };
    Ok(ok(DiscardReviewResponse { change }, &result.etag))
}

/// `GET /changes/{name}/verify`
pub async fn verify_show(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::VerifyShow { change: name }).await?;
    let o = match result.execution.outcome {
        CommandOutcome::VerifyShow(o) => o,
        _ => return Err(wrong_outcome("verify-show")),
    };
    let last_round = review_round_dto(o.ticket.last_round());
    let rounds: Vec<ReviewRoundDto> = o.ticket.rounds.iter().map(review_round_dto).collect();
    Ok(ok(
        ReviewTicketResponse { change: o.change, rounds, last_round, content: o.content },
        &result.etag,
    ))
}

/// `POST /changes/{name}/verify/rounds` — 任務未全數完成時引擎拒絕（design D3）。
pub async fn verify_add_round(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Json(req): Json<AddReviewRoundRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::VerifyAddRound { change: name, content: req.content },
    )
    .await?;
    let round = match result.execution.outcome {
        CommandOutcome::VerifyAddRound(o) => o.round,
        _ => return Err(wrong_outcome("verify-add-round")),
    };
    Ok(ok(AddReviewRoundResponse { round: round as u64 }, &result.etag))
}

/// `POST /changes/{name}/verify/stamp` — 指紋歸屬與審查站同一條（design D4a）。
pub async fn verify_stamp(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Json(req): Json<StampReviewRequest>,
) -> Result<Response, ApiError> {
    if !binding.editor {
        return Err(ApiError::forbidden("reader memberships cannot stamp verifications"));
    }
    let scope = req
        .scope
        .into_iter()
        .map(|e| speclink_core::model::ReviewedScopeEntry { path: e.path, hash: e.hash })
        .collect();
    let result = verb::run(
        &state,
        &binding,
        Command::VerifyStamp { change: name, accept: req.accept, tool: req.agent, scope, missing: req.missing },
    )
    .await?;
    let change = match result.execution.outcome {
        CommandOutcome::VerifyStamp(o) => o.change,
        _ => return Err(wrong_outcome("verify-stamp")),
    };
    Ok(ok(StampReviewResponse { change }, &result.etag))
}

/// `DELETE /changes/{name}/verify` — editor 限定比照審查站：工單是驗證過程的
/// 唯一紀錄，刪掉不可回復。
pub async fn verify_discard(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    if !binding.editor {
        return Err(ApiError::forbidden(
            "reader memberships cannot discard verify tickets",
        ));
    }
    let result = verb::run(&state, &binding, Command::VerifyDiscard { change: name }).await?;
    let change = match result.execution.outcome {
        CommandOutcome::VerifyDiscard(o) => o.change,
        _ => return Err(wrong_outcome("verify-discard")),
    };
    Ok(ok(DiscardReviewResponse { change }, &result.etag))
}

/// `POST /changes/{name}/tasks/{taskId}/done`
pub async fn task_done(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name, task_id)): Path<(String, String, String)>,
    Json(req): Json<TaskDoneRequest>,
) -> Result<Response, ApiError> {
    // The Host resolved the candidates at its own boundary — here, the wire
    // request. An absent list is "nothing to attribute", the same normal state
    // a clean local checkout reports; it is never an error and never a reason
    // to go probing this machine's own working tree.
    let result = verb::run(
        &state,
        &binding,
        Command::TaskDone {
            task_id,
            change: Some(name),
            touched_files: Some(req.touched_files),
            head_commit: req.head_commit,
        },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::TaskDone(o) => o,
        _ => return Err(wrong_outcome("task-done")),
    };
    let dto = TaskDoneResponse {
        task_desc: outcome.description,
        already_done: outcome.already,
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /changes/{name}/tasks/{taskId}/undone`
pub async fn task_undone(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name, task_id)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::TaskUndone {
            task_id,
            change: Some(name),
        },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::TaskUndone(o) => o,
        _ => return Err(wrong_outcome("task-undone")),
    };
    let dto = TaskUndoneResponse {
        task_desc: outcome.description,
        already_undone: outcome.already,
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /changes/{name}/claim` — a minimal team-mode acknowledgment; durable
/// ownership arrives with the auth/admin knife. Refuses on a missing change.
pub async fn claim(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let meta = verb::read_doc(
        &state,
        &binding,
        DocumentId::ChangeMeta {
            change: name.clone(),
        },
    )
    .await?;
    if meta.is_none() {
        return Err(ApiError::not_found(format!("Change '{name}' not found.")));
    }
    let etag = verb::scope_etag(&state, &binding).await?;
    let dto = ClaimResponse {
        lifecycle: None,
        claimed_by: Some(binding.actor.display.clone()),
    };
    Ok(ok(dto, &etag))
}

/// `POST /changes/{name}/archive[?carryReview=true]` 的查詢參數——旗標比照
/// `DELETE /discussions/{slug}?force=` 走 query（缺席＝false，既有無 body 的
/// 呼叫端不受影響）。
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveQuery {
    #[serde(default)]
    carry_review: bool,
    #[serde(default)]
    carry_verify: bool,
}

/// `POST /changes/{name}/archive[?carryReview=true][&carryVerify=true]` — 帶旗標
/// 時該站的未結工單隨 change 搬入封存區（design D4／D5 的第三處置；不接通的話
/// remote 只剩兩條出路）。兩個旗標各自獨立，可同時帶。
pub async fn archive(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, name)): Path<(String, String)>,
    Query(query): Query<ArchiveQuery>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::Archive {
            change: Some(name),
            skip_specs: false,
            no_validate: false,
            mark_tasks_complete: false,
            carry_review: query.carry_review,
            carry_verify: query.carry_verify,
        },
    )
    .await?;
    let outcome = match result.execution.outcome {
        CommandOutcome::Archive(o) => o,
        _ => return Err(wrong_outcome("archive")),
    };
    let dto = ArchiveResponse {
        specs: outcome
            .caps
            .into_iter()
            .map(|c| ArchivedSpec {
                capability: c.capability,
                added: c.added,
                modified: c.modified,
                removed: c.removed,
                renamed: c.renamed,
            })
            .collect(),
        // datedName is the sentinel the remote caller keys its full rendering
        // on, so it travels whenever the engine produced an outcome at all.
        dated_name: Some(outcome.dated_name),
        snapshot_created: Some(outcome.snapshot_created),
        archived_discussions: outcome
            .archived_discussions
            .into_iter()
            .map(|(slug, file)| ArchivedDiscussion { slug, file })
            .collect(),
        evidence_recorded: Some(outcome.evidence_recorded),
    };
    Ok(ok(dto, &result.etag))
}

/// Parse the `If-Match` header as the write precondition version (`0` =
/// create-only). Absent or unparseable is an invalid argument — the client
/// always sends it.
fn parse_if_match(headers: &HeaderMap) -> Result<u64, ApiError> {
    let value = headers
        .get("if-match")
        .ok_or_else(|| ApiError::invalid_argument("missing If-Match header"))?;
    value
        .to_str()
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .ok_or_else(|| ApiError::invalid_argument("If-Match must be a version number"))
}

/// Map an artifact id to the Engine's new-artifact command. `force` is always
/// set — the If-Match precondition already decided create vs overwrite.
fn artifact_write_command(
    artifact: &str,
    change: String,
    content: String,
) -> Result<Command, ApiError> {
    let (kind, capability) = match artifact {
        "proposal" | "design" | "tasks" => (artifact.to_string(), None),
        _ => match artifact.strip_prefix("specs/") {
            Some(cap) if !cap.is_empty() && !cap.contains('/') => {
                ("spec".to_string(), Some(cap.to_string()))
            }
            _ => {
                return Err(ApiError::invalid_argument(format!(
                    "unknown artifact '{artifact}'"
                )))
            }
        },
    };
    Ok(Command::NewArtifact {
        kind,
        capability,
        change: Some(change),
        content: Some(content),
        force: true,
        // The raw artifact PUT is a direct write, not the CLI's creation
        // verb — the naming gate's second net (the validate warning) covers
        // this entrance, so the write itself stays ungated.
        new_capability: true,
    })
}

// --- discussions ---

/// Query string of `GET /discussions`.
#[derive(Deserialize)]
pub struct ListDiscussionsQuery {
    #[serde(default)]
    archived: bool,
}

/// `GET /discussions[?archived=true]`
pub async fn list_discussions(
    State(state): State<AppState>,
    binding: Binding,
    Query(query): Query<ListDiscussionsQuery>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussList {
            archived: query.archived,
        },
    )
    .await?;
    let discussions = match result.execution.outcome {
        CommandOutcome::DiscussList(list) => list,
        _ => return Err(wrong_outcome("discuss-list")),
    };
    // promotedTo 於 route 邊緣以引擎查詢函式組裝（remote-read-parity design
    // D1）：引擎 DiscussionInfo 不帶此欄以保 CLI JSON 逐位元不變。查詢失敗
    // 以欄位缺席容錯（全空清單）、列表不失敗。
    let store = state.store.clone();
    let scope = verb::scope_of(&binding);
    let slugs: Vec<String> = discussions.iter().map(|d| d.slug.clone()).collect();
    let count = slugs.len();
    let promoted = tokio::task::spawn_blocking(move || {
        speclink_host::bridge::discussions_promoted_to(store.as_ref(), &scope, &slugs)
            .unwrap_or_else(|_| vec![Vec::new(); count])
    })
    .await
    .map_err(|e| ApiError::internal(format!("blocking task failed: {e}")))?;
    let dto = ListDiscussionsResponse {
        discussions: discussions
            .into_iter()
            .zip(promoted)
            .map(|(info, promoted_to)| {
                let mut dto = discussion_info(info);
                dto.promoted_to = promoted_to;
                dto
            })
            .collect(),
    };
    Ok(ok(dto, &result.etag))
}

/// 討論寫入的引擎錯誤映射（design D1：驗證與 guard 的單一事實來源在引擎，
/// 凍結文本逐字上 wire 以保 fs/remote 訊息 parity）。引擎的討論 bail 一律
/// 歸類 `Error`（→500）；這裡按文本的語意形式改判語義化狀態碼、訊息不動：
/// 主體不存在→404、slug 無法成立→400、既有守衛（已存在／已封存／未鑄鏈）→409。
fn refine_discussion_write(e: ApiError) -> ApiError {
    use speclink_protocol::error::ErrorReason;
    if e.reason != ErrorReason::Internal {
        return e;
    }
    let m = e.message;
    // 語意拒絕（前綴判定）先於 not_found（子字串判定）：slug／kind 是請求可控
    // 字串，會被引擎訊息內嵌，值帶「' not found」不得把 400 操縱成 404。
    if m.starts_with("invalid slug '")
        || m.starts_with("invalid kind '")
        || m.starts_with("invalid topic '")
        || m.starts_with("could not derive a slug")
    {
        ApiError::invalid_argument(m)
    } else if m.contains("' not found") {
        ApiError::not_found(m)
    } else if m.contains("' already exists")
        || m.contains("' is archived")
        || m.contains("is not linked to discussion")
    {
        ApiError::refused(m)
    } else {
        ApiError::internal(m)
    }
}

/// `POST /discussions`
pub async fn create_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Json(req): Json<CreateDiscussionRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussNew {
            topic: req.topic,
            slug: req.slug,
            kind: req.kind,
        },
    )
    .await
    .map_err(refine_discussion_write)?;
    let info = match result.execution.outcome {
        CommandOutcome::DiscussNew(info) => info,
        _ => return Err(wrong_outcome("discuss-new")),
    };
    let dto = CreateDiscussionResponse {
        slug: info.slug,
        topic: info.topic,
        path: info.path,
    };
    Ok(ok(dto, &result.etag))
}

/// `DELETE /discussions/{slug}?force=` query input; force defaults to false.
#[derive(Deserialize)]
pub struct DeleteDiscussionQuery {
    #[serde(default)]
    force: bool,
}

/// `DELETE /discussions/{slug}` — Command::DiscussDiscard 直通（design D3，
/// 複製 change 側 DELETE 模式）：0 輪即刪、有輪無 force 拒絕（reason 機器可判
/// 為 refused）。editor 限定比照 change 刪除。
pub async fn delete_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Query(query): Query<DeleteDiscussionQuery>,
) -> Result<Response, ApiError> {
    if !binding.editor {
        return Err(ApiError::forbidden(
            "reader memberships cannot delete discussions",
        ));
    }
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussDiscard {
            slug,
            force: query.force,
        },
    )
    .await
    .map_err(refine_discussion_write)?;
    let outcome = match result.execution.outcome {
        CommandOutcome::DiscussDiscard(o) => o,
        _ => return Err(wrong_outcome("discuss-discard")),
    };
    Ok(ok(DiscardDiscussionResponse { slug: outcome.slug }, &result.etag))
}

/// `POST /discussions/{slug}/link` — Command::DiscussLink 直通（design D3，
/// 比照 promote 的 POST 模式）：鑄變更側 from_discussion 鏈。
pub async fn link_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<BindDiscussionRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussLink {
            slug,
            change: req.change,
        },
    )
    .await
    .map_err(refine_discussion_write)?;
    let outcome = match result.execution.outcome {
        CommandOutcome::DiscussLink(o) => o,
        _ => return Err(wrong_outcome("discuss-link")),
    };
    let dto = BindDiscussionResponse {
        slug: outcome.slug,
        change: outcome.change,
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /discussions/{slug}/seal` — Command::DiscussSeal 直通（design D3）：
/// 內容落地後把討論標記已轉出（promoted），前置守衛（鏈須先鑄妥）在引擎。
pub async fn seal_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<BindDiscussionRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussSeal {
            slug,
            change: req.change,
        },
    )
    .await
    .map_err(refine_discussion_write)?;
    let outcome = match result.execution.outcome {
        CommandOutcome::DiscussSeal(o) => o,
        _ => return Err(wrong_outcome("discuss-seal")),
    };
    let dto = BindDiscussionResponse {
        slug: outcome.slug,
        change: outcome.change,
    };
    Ok(ok(dto, &result.etag))
}

/// `GET /discussions/{slug}`
pub async fn show_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::DiscussShow { slug }).await?;
    let show = match result.execution.outcome {
        CommandOutcome::DiscussShow(show) => show,
        _ => return Err(wrong_outcome("discuss-show")),
    };
    let info = show
        .info
        .ok_or_else(|| ApiError::internal("discuss-show: missing discussion info"))?;
    let dto = ShowDiscussionResponse {
        info: discussion_info(info),
        content: show.content,
    };
    Ok(ok(dto, &result.etag))
}

/// `PUT /discussions/{slug}/context`
pub async fn set_discussion_context(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<SetDiscussionContextRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussContext {
            slug,
            content: req.content,
        },
    )
    .await?;
    Ok(ok(Ack {}, &result.etag))
}

/// `POST /discussions/{slug}/rounds`
pub async fn add_discussion_round(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<AddDiscussionRoundRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussAddRound {
            slug,
            mode: req.mode,
            content: req.content,
        },
    )
    .await?;
    let round = match result.execution.outcome {
        CommandOutcome::DiscussAddRound(o) => o.round,
        _ => return Err(wrong_outcome("discuss-add-round")),
    };
    Ok(ok(
        AddDiscussionRoundResponse {
            round: round as u64,
        },
        &result.etag,
    ))
}

/// `POST /discussions/{slug}/conclude`
pub async fn conclude_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<ConcludeDiscussionRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussConclude {
            slug,
            content: req.content,
        },
    )
    .await?;
    let restale_flagged = match result.execution.outcome {
        CommandOutcome::DiscussConclude(o) => o.restale_flagged,
        _ => return Err(wrong_outcome("discuss-conclude")),
    };
    Ok(ok(ConcludeDiscussionResponse { restale_flagged }, &result.etag))
}

/// `POST /discussions/{slug}/archive`
pub async fn archive_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    let result = verb::run(&state, &binding, Command::DiscussArchive { slug }).await?;
    let archived_file = match result.execution.outcome {
        CommandOutcome::DiscussArchive(o) => o.archived_file,
        _ => return Err(wrong_outcome("discuss-archive")),
    };
    let dto = ArchiveDiscussionResponse {
        archived_to: format!("discussions/archive/{archived_file}"),
    };
    Ok(ok(dto, &result.etag))
}

/// `POST /discussions/{slug}/promote`
pub async fn promote_discussion(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, slug)): Path<(String, String)>,
    Json(req): Json<PromoteDiscussionRequest>,
) -> Result<Response, ApiError> {
    let result = verb::run(
        &state,
        &binding,
        Command::DiscussPromote {
            slug,
            name: req.name,
        },
    )
    .await?;
    let change = match result.execution.outcome {
        CommandOutcome::DiscussPromote(o) => o.change,
        _ => return Err(wrong_outcome("discuss-promote")),
    };
    // 新變更的目錄刻意不上 wire：那是 store 端的檔案系統位置，對本機使用者
    // 無意義——與 `new change` 的 Path 行同一條裁定（design D5）。
    Ok(ok(PromoteDiscussionResponse { change }, &result.etag))
}

fn discussion_info(info: EngineDiscussionInfo) -> DiscussionInfo {
    DiscussionInfo {
        slug: info.slug,
        topic: info.topic,
        status: info.status,
        rounds: info.rounds,
        created: info.created,
        created_by: info.created_by,
        kind: info.kind,
        promoted_to: Vec::new(),
        path: info.path,
        archived: info.archived,
    }
}

// --- Engine outcome → protocol DTO (typed field mapping, no raw JSON) ---

fn change_summary(
    change: ListChangeJson,
    meta: Option<&speclink_core::model::ChangeMeta>,
) -> ChangeSummary {
    ChangeSummary {
        name: change.name,
        summary: change.summary,
        status: change.status,
        completed_tasks: change.completed_tasks,
        total_tasks: change.total_tasks,
        restale_from: change.restale_from,
        meta_error: change.meta_error,
        repo: None,
        lifecycle: None,
        claimed_by: None,
        started_at: meta.and_then(|m| m.started_at.clone()),
        created_by: meta.and_then(|m| m.created_by.clone()),
        created: meta.and_then(|m| m.created.clone()),
        from_discussions: meta.map(|m| m.from_discussions()).unwrap_or_default(),
    }
}

fn change_status(report: StatusReport) -> ChangeStatus {
    ChangeStatus {
        change_name: report.change_name,
        schema_name: report.schema_name,
        is_complete: report.is_complete,
        apply_requires: report.apply_requires,
        artifacts: report
            .artifacts
            .into_iter()
            .map(|a| ArtifactStatus {
                id: a.id,
                output_path: a.output_path,
                status: a.status,
                missing_deps: a.blocked_by,
                version: None,
            })
            .collect(),
        status_version: None,
        repo: None,
        lifecycle: None,
        claimed_by: None,
        created: None,
        from_discussions: Vec::new(),
        delta_capabilities: Vec::new(),
        created_by: None,
        created_with: None,
        started_at: None,
        started_by: None,
    }
}

fn apply_instructions(apply: engine::ApplyInstructions) -> ApplyInstructions {
    ApplyInstructions {
        change_name: apply.change_name,
        change_dir: apply.change_dir,
        schema_name: apply.schema_name,
        context_files: apply.context_files,
        progress: Progress {
            total: apply.progress.total,
            complete: apply.progress.complete,
            remaining: apply.progress.remaining,
            code_total: apply.progress.code_total,
            code_complete: apply.progress.code_complete,
            code_remaining: apply.progress.code_remaining,
        },
        tasks: apply.tasks.into_iter().map(task_entry).collect(),
        state: apply.state,
        missing_artifacts: apply.missing_artifacts,
        locale: apply.locale,
        tdd: apply.tdd,
        audit: apply.audit,
        instruction: apply.instruction,
    }
}

fn artifact_instructions(instr: engine::ArtifactInstructions) -> ArtifactInstructions {
    ArtifactInstructions {
        change_name: instr.change_name,
        artifact_id: instr.artifact_id,
        schema_name: instr.schema_name,
        change_dir: instr.change_dir,
        output_path: instr.output_path,
        description: instr.description,
        instruction: instr.instruction,
        context: instr.context,
        rules: instr.rules,
        locale: instr.locale,
        template: instr.template,
        dependencies: instr
            .dependencies
            .into_iter()
            .map(|d| DependencyEntry {
                id: d.id,
                done: d.done,
                path: d.path,
                description: d.description,
            })
            .collect(),
        unlocks: instr.unlocks,
    }
}

fn task_entry(task: engine::TaskJson) -> TaskEntry {
    TaskEntry {
        id: task.id,
        description: task.description,
        done: task.done,
        manual: task.manual,
    }
}

/// The artifact id → change-relative document path (the `artifact cat`
/// vocabulary), mirroring the Engine's mapping.
fn artifact_rel_path(artifact: &str) -> Option<String> {
    match artifact {
        "proposal" => Some("proposal.md".to_string()),
        "design" => Some("design.md".to_string()),
        "tasks" => Some("tasks.md".to_string()),
        _ => artifact
            .strip_prefix("specs/")
            .filter(|cap| !cap.is_empty() && !cap.contains('/'))
            .map(|cap| format!("specs/{cap}/spec.md")),
    }
}

#[cfg(test)]
mod tests {
    use super::StoredEvidence;

    /// 記錄形狀的三份宣告（core 的 EvidenceEntry、protocol 的 wire DTO、此處的
    /// StoredEvidence 讀取面）以這條測試釘住同步：core 寫出的 entry 必須被 wire
    /// 面逐欄讀回。core 加欄位而 wire 沒跟上時，這裡最先紅。
    #[test]
    fn evidence_wire_shape_matches_the_engine_record() {
        let record = speclink_core::tasks::TouchedRecord {
            version: Some(2),
            change: "demo".to_string(),
            touched: Vec::new(),
            entries: vec![speclink_core::tasks::EvidenceEntry {
                task_id: "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                task_desc: "1.1 First".to_string(),
                actor: Some("Tester <t@example.com>".to_string()),
                repo: Some("backend".to_string()),
                head_commit: Some("0123456789012345678901234567890123456789".to_string()),
                touched_files: vec!["src/app.rs".to_string()],
                recorded_at: "2026-08-23T00:00:00Z".to_string(),
            }],
        };
        let text = serde_json::to_string(&record).expect("engine record serializes");

        let parsed: StoredEvidence = serde_json::from_str(&text).expect("wire face reads it");
        assert_eq!(parsed.entries.len(), 1);
        let e = &parsed.entries[0];
        assert_eq!(e.task_id, "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(e.task_desc, "1.1 First");
        assert_eq!(e.actor.as_deref(), Some("Tester <t@example.com>"));
        assert_eq!(e.repo.as_deref(), Some("backend"));
        assert_eq!(e.head_commit.as_deref(), Some("0123456789012345678901234567890123456789"));
        assert_eq!(e.touched_files, vec!["src/app.rs".to_string()]);
        assert_eq!(e.recorded_at, "2026-08-23T00:00:00Z");
    }

    /// v1-only 記錄（無 entries 欄）讀成空集合，不偽造 actor 或 recordedAt。
    #[test]
    fn a_v1_only_record_reads_as_an_empty_entry_set() {
        let parsed: StoredEvidence = serde_json::from_str(
            r#"{"change":"demo","touched":[{"task_id":"1","task_desc":"1.1 a","files":["src/a.rs"]}]}"#,
        )
        .expect("v1 shape still parses");
        assert!(parsed.entries.is_empty());
    }
}

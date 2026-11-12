//! The door's route handlers (design D4): each takes the request already
//! bound to its caller, project and repo, with its path params decoded, and
//! gives the wire response with the scope ETag.

use super::error::HttpError;
use super::wire;
use super::{HttpRequest, HttpResponse};
use crate::documents::{self, ContextOutcome};
use crate::kit::{Caller, HostKit, ProjectEntry, RepoEntry, RunOptions};
use crate::port_exec::{self, AtVersion, ExecError, PortExecution};
use serde::de::DeserializeOwned;
use serde::Serialize;
use speclink_core::command::{Command, CommandOutcome, ErrorCode, InstructionsOutcome};
use speclink_core::discuss::{WriteRefusal, WriteRefusalKind};
use speclink_core::config::WorkflowConfig;
use speclink_protocol::binding::{Actor as BindingActor, BindingResponse, Capabilities, ScopeRef};
use speclink_protocol::command::{
    AddDiscussionRoundRequest, AddDiscussionRoundResponse, AddReviewRoundRequest, AddReviewRoundResponse,
    ArchiveDiscussionResponse, ArchiveResponse, ArchivedDiscussion, ArchivedSpec, BindDiscussionRequest,
    BindDiscussionResponse, ClaimResponse, ConcludeDiscussionRequest, ConcludeDiscussionResponse,
    CreateChangeRequest, CreateChangeResponse, CreateDiscussionRequest, CreateDiscussionResponse,
    DiscardDiscussionResponse, DiscardResponse, DiscardReviewResponse, InProgressRemoveResponse, MoveTaskRequest,
    MoveTaskResponse, PromoteDiscussionRequest, PromoteDiscussionResponse, PutArtifactRequest, PutArtifactResponse,
    RankChangeRequest, RankChangeResponse, ReviewTicketResponse, SetDependsRequest, SetDependsResponse,
    SetDiscussionContextRequest, SetSchemaRequest, SetSchemaResponse, StampReviewRequest, StampReviewResponse,
    TaskDoneRequest, TaskDoneResponse, TaskUndoneResponse, UnlinkedDiscussion,
};
use speclink_protocol::context::ContextSnapshotRequest;
use speclink_protocol::events::{EventsDeclaration, PollingDeclaration};
use speclink_protocol::query::{
    ArtifactContent, BoardOrderDoc, DiscussionCut, DiscussionHit, DiscussionMatch, LanguageResponse,
    ListDiscussionsResponse, SearchDiscussionsResponse, ShowDiscussionResponse, ListChangesResponse, ListSpecsResponse, PutBoardOrderRequest,
    PutBoardOrderResponse, PutConfigRequest, PutConfigResponse, SpecSummary, ValidateChangeResponse, WhoamiRepo,
    WhoamiResponse, WhoamiUser,
};
use speclink_protocol::API_VERSION;
use speclink_store_port::{Expected, ScopeId, StorePort, Version};

/// The engine version the handshake declares.
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A request the door accepted: who, where, and the decoded path params
/// after the project key.
pub(crate) struct Request<'a> {
    pub kit: &'a HostKit,
    pub http: &'a HttpRequest,
    pub caller: Caller,
    pub project: &'a ProjectEntry,
    pub repo: &'a RepoEntry,
    pub params: Vec<String>,
}

pub(crate) type Handler = fn(&Request<'_>) -> Result<HttpResponse, HttpError>;

impl Request<'_> {
    fn port(&self) -> &dyn StorePort {
        self.kit.store.as_ref()
    }

    fn scope(&self) -> &ScopeId {
        &self.caller.scope
    }

    /// The `i`th path param after the project key.
    fn param(&self, i: usize) -> String {
        self.params[i].clone()
    }

    /// The body as JSON of `T`. Not JSON, or not that shape: 400.
    fn body<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_json::from_slice(&self.http.body)
            .map_err(|e| HttpError::invalid_argument(format!("the request body is not valid: {e}")))
    }

    /// The query string as `T`. Not that shape: 400.
    fn query<T: DeserializeOwned>(&self) -> Result<T, HttpError> {
        serde_urlencoded::from_str(self.http.query.as_deref().unwrap_or(""))
            .map_err(|e| HttpError::invalid_argument(format!("the query string is not valid: {e}")))
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.http.header(name)
    }

    /// The scope's ETag now.
    fn etag_now(&self) -> Result<String, HttpError> {
        Ok(documents::etag(documents::scope_version(self.port(), self.scope())?))
    }

    /// Run `cmd` through the kit, and its ETag: the version its commit
    /// landed at, or — when it committed nothing — the scope version taken
    /// before it read anything, so the ETag is never newer than the content.
    fn run(&self, cmd: Command) -> Result<(PortExecution, String), HttpError> {
        self.run_with(cmd, self.options())
    }

    /// [`Request::run`] with these options. They state no expected scope
    /// version, so the run never ends as a moved scope.
    fn run_with(&self, cmd: Command, options: RunOptions) -> Result<(PortExecution, String), HttpError> {
        Ok(self.try_run(cmd, options)?)
    }

    /// [`Request::run_with`], with the failure as the kit gave it.
    fn try_run(&self, cmd: Command, options: RunOptions) -> Result<(PortExecution, String), ExecError> {
        let ran = self.kit.run_with(&self.caller, cmd, &RunOptions { version_first: true, ..options })?;
        let AtVersion::Done(execution) = ran.at else {
            unreachable!("a run with no expected version never sees the scope move")
        };
        let version = execution.scope_version.or(ran.before).expect("the version was taken before the run");
        Ok((execution, documents::etag(version)))
    }

    /// How the kit runs a command for this request: the repo key goes into
    /// completion evidence.
    fn options(&self) -> RunOptions {
        RunOptions { repo: Some(self.repo.key.clone()), ..RunOptions::default() }
    }

    /// The scope version a client says it last saw. A number past the
    /// largest version names none any scope can be at, so it is stale.
    fn expected_version(&self, sent: u64) -> Result<Version, HttpError> {
        Version::new(sent).map_err(|_| match documents::scope_version(self.port(), self.scope()) {
            Ok(actual) => HttpError::revision_conflict(documents::scope_moved(sent, actual)),
            Err(e) => HttpError::from(e),
        })
    }
}

/// A 200 JSON response with the scope ETag.
pub(crate) fn ok<T: Serialize>(dto: &T, etag: String) -> HttpResponse {
    let mut response = HttpResponse::json(200, dto);
    response.headers.push(("etag".to_string(), etag));
    response
}

/// A 200 JSON response with the ETag of `version`.
fn ok_at<T: Serialize>(dto: &T, version: Version) -> HttpResponse {
    ok(dto, documents::etag(version))
}

/// A 304 with the ETag and no body.
fn not_modified(etag: String) -> HttpResponse {
    HttpResponse { status: 304, headers: vec![("etag".to_string(), etag)], body: Vec::new() }
}

// --- handshake, polling, identity ---

/// `GET /binding`: who, where, and what the caller may do. The fields that
/// depend on the caller follow the repo's two switches, so the handshake
/// says what the door then enforces.
pub(crate) fn binding(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (identity, project, repo) = (&req.caller.identity, req.project, req.repo);
    let dto = BindingResponse {
        actor: BindingActor { id: identity.id().to_string(), name: identity.shown_name().to_string() },
        project: ScopeRef { id: format!("prj_{}", project.key), key: project.key.clone(), name: project.name.clone() },
        repo: ScopeRef { id: format!("repo_{}", repo.key), key: repo.key.clone(), name: repo.name.clone() },
        api_version: API_VERSION.to_string(),
        engine_version: ENGINE_VERSION.to_string(),
        capabilities: Capabilities {
            context_snapshots: true,
            policy_write: repo.can_write && repo.can_write_policy,
            validate: true,
            analyze: true,
            delete_change: repo.can_write,
            move_task: repo.can_write,
            authentication: Vec::new(),
            events: EventsDeclaration {
                transports: req.kit.options.push_transports.clone(),
                polling: Some(PollingDeclaration { url: "/sync-state".to_string(), etag: true }),
            },
        },
    };
    Ok(HttpResponse::json(200, &dto))
}

/// `GET /sync-state`: 304 while `If-None-Match` is the scope's ETag, else
/// 200 with the new one. One list, no document read.
pub(crate) fn sync_state(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let etag = req.etag_now()?;
    if req.header("if-none-match") == Some(etag.as_str()) {
        return Ok(not_modified(etag));
    }
    Ok(ok(&serde_json::json!({}), etag))
}

/// `GET /whoami`: the identity, and the project's repos by key.
pub(crate) fn whoami(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let identity = &req.caller.identity;
    let dto = WhoamiResponse {
        user: WhoamiUser { name: identity.shown_name().to_string(), handle: identity.id().to_string() },
        repos: req.project.repos.iter().map(|r| WhoamiRepo { name: r.key.clone(), git_url: String::new() }).collect(),
    };
    Ok(ok(&dto, req.etag_now()?))
}

// --- the workflow config ---

/// `GET /config`: the policy source and the scope revision, which is the
/// ETag.
pub(crate) fn config(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::config_document(req.port(), req.scope())?;
    Ok(ok_at(&dto, version))
}

/// `PUT /config`: the whole document, written by the Engine's command while
/// the scope stays at the client's revision. A malformed document is told
/// before a stale revision; a caller who may not change policy is refused
/// once the Engine named the changed keys (design D5).
pub(crate) fn put_config(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: PutConfigRequest = req.body()?;
    let parsed =
        WorkflowConfig::from_text(Some(&body.content)).map_err(|e| HttpError::invalid_config(e.to_string()))?;
    speclink_core::config::validate_policy_locales(&speclink_core::config::WorkflowPolicyFields {
        locale: parsed.locale.clone(),
        spec_locale: parsed.spec_locale.clone(),
        ..Default::default()
    })
    .map_err(|e| HttpError::invalid_config(e.to_string()))?;

    // The wire's expected revision is the scope's (what `GET /config`
    // returns), not the config document's own version.
    let expected = req.expected_version(body.expected_revision)?;
    let options = RunOptions { expected: Some(expected), policy_locked: !req.repo.can_write_policy, ..req.options() };
    let cmd = Command::WorkflowConfigReplace { content: body.content };
    let version = match req.kit.run_with(&req.caller, cmd, &options)?.at {
        // Content that did not change commits nothing: the revision stays.
        AtVersion::Done(execution) => execution.scope_version.unwrap_or(expected),
        AtVersion::Moved { actual } => return Err(documents::DocumentsError::ScopeMoved { expected, actual }.into()),
    };
    Ok(ok_at(&PutConfigResponse { revision: version.get() }, version))
}

// --- the board order ---

/// `GET /board-order`: the opaque document, or null, and the scope revision.
pub(crate) fn board_order(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::board_order(req.port(), req.scope())?;
    Ok(ok_at(&dto, version))
}

/// `PUT /board-order`: the whole document while the scope stays at the
/// `If-Match` revision. It is not an Engine command, but it goes through
/// the same hooks, as `put-board-order` with no params.
pub(crate) fn put_board_order(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    const COMMAND: &str = "put-board-order";
    let body: PutBoardOrderRequest = req.body()?;
    // An oversized body is refused before the precondition is looked at.
    documents::check_board_order_size(&body.content)?;
    let sent = req
        .header("if-match")
        .and_then(|raw| raw.trim().trim_matches('"').parse::<u64>().ok())
        .ok_or_else(|| HttpError::invalid_argument("If-Match must carry the scope revision ETag"))?;

    req.kit.ask(&req.caller, COMMAND, &serde_json::json!({}), None)?;
    let expected = req.expected_version(sent)?;
    let at = port_exec::now()?;
    let actor = req.caller.identity.actor();
    let version = documents::put_board_order(req.port(), req.scope(), actor, body.content, expected, at)?;
    req.kit.watch(&req.caller, COMMAND, version, vec![documents::board_order_updated()?]);
    Ok(ok_at(&PutBoardOrderResponse { revision: version.get() }, version))
}

// --- documents that are not Engine commands ---

/// `GET /changes/{name}/evidence`: the change's recorded evidence; none is
/// an empty set.
pub(crate) fn change_evidence(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::change_evidence(req.port(), req.scope(), &req.params[0])?;
    Ok(ok_at(&dto, version))
}

/// `GET /specs/{capability}/document`.
pub(crate) fn spec_document(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::spec_document(req.port(), req.scope(), &req.params[0])?;
    Ok(ok_at(&dto, version))
}

/// `GET /archived`.
pub(crate) fn archived_list(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::archived_list(req.port(), req.scope())?;
    Ok(ok_at(&dto, version))
}

/// `GET /archived/{datedName}/artifacts/{*artifact}`.
pub(crate) fn archived_artifact(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::archived_artifact(req.port(), req.scope(), &req.params[0], &req.params[1])?;
    Ok(ok_at(&dto, version))
}

/// `GET /archived/{datedName}/capabilities`.
pub(crate) fn archived_capabilities(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (dto, version) = documents::archived_capabilities(req.port(), req.scope(), &req.params[0])?;
    Ok(ok_at(&dto, version))
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
}

/// `GET /search?q=`: the desktop's search over active changes and live
/// discussions.
pub(crate) fn search(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: SearchQuery = req.query()?;
    let (dto, version) = documents::search(req.port(), req.scope(), &query.q)?;
    Ok(ok_at(&dto, version))
}

/// `POST /context`: one consistent snapshot of the scope; 304 while
/// `If-None-Match` is the scope's state. It only reads, so a caller who
/// cannot write still takes it.
pub(crate) fn context(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: ContextSnapshotRequest = req.body()?;
    match documents::context_snapshot(req.port(), req.scope(), &body, req.header("if-none-match"))? {
        ContextOutcome::NotModified(version) => Ok(not_modified(documents::etag(version))),
        ContextOutcome::Snapshot(snapshot) => {
            let etag = snapshot.snapshot_id.clone();
            Ok(ok(&snapshot, etag))
        }
    }
}

/// The unexpected-outcome guard: the command returned an outcome kind this
/// route never asks for. A bug, not a client error.
fn wrong_outcome(route: &str) -> HttpError {
    HttpError::internal(format!("{route}: unexpected command outcome"))
}

// --- queries and checks ---

/// `GET /changes`: the Engine's list, with the fields its items do not
/// carry (started, creator, created, source discussions) read from each
/// change's data file. A broken data file stays absent: the list reports it
/// as `metaError` and does not fail.
pub(crate) fn list_changes(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let cmd = Command::List { sort: "name".to_string(), specs: false, changes: false, worktrees: Default::default() };
    let (execution, etag) = req.run(cmd)?;
    let changes = match execution.outcome {
        CommandOutcome::List(list) => list.changes.unwrap_or_default(),
        _ => return Err(wrong_outcome("list")),
    };
    let names: Vec<String> = changes.iter().map(|c| c.name.clone()).collect();
    let metas = documents::change_extras(req.port(), req.scope(), &names)?;
    let changes = changes
        .into_iter()
        .map(|c| {
            let meta = metas.get(&c.name);
            wire::change_summary(c, meta)
        })
        .collect();
    Ok(ok(&ListChangesResponse { changes }, etag))
}

/// `GET /changes/{name}`: the change's status, with the show fields read
/// from its data file and its delta specs.
pub(crate) fn get_change(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let name = req.param(0);
    let (execution, etag) = req.run(Command::Status { change: Some(name.clone()), schema: None })?;
    let report = match execution.outcome {
        CommandOutcome::Status(report) => report,
        _ => return Err(wrong_outcome("status")),
    };
    let meta = documents::change_extras(req.port(), req.scope(), std::slice::from_ref(&name))?.remove(&name);
    let delta_capabilities = documents::change_delta_capabilities(req.port(), req.scope(), &name)?;
    let mut dto = wire::change_status(report);
    if let Some(meta) = meta {
        // `created` only with `schema`, as `show` pairs them.
        dto.created = meta.schema.is_some().then_some(meta.created.clone()).flatten();
        dto.from_discussions = meta.from_discussions();
        dto.created_by = meta.created_by;
        dto.created_with = meta.created_with;
        dto.started_at = meta.started_at;
        dto.started_by = meta.started_by;
        dto.claimed_by = meta.claimed_by;
    }
    dto.delta_capabilities = delta_capabilities;
    Ok(ok(&dto, etag))
}

/// `GET /changes/{name}/drift`: the spec-side drift over one store view,
/// with that view's basis digests. A read: no commit, no event. The
/// version is taken first, so it is never newer than the report.
pub(crate) fn drift(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let etag = req.etag_now()?;
    let view = crate::drift::spec_drift(req.port(), req.scope(), &req.params[0])?;
    Ok(ok(&crate::drift::spec_drift_view_to_wire(&view), etag))
}

/// `GET /changes/{name}/validate`: one change, not strict.
pub(crate) fn validate_change(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let cmd = Command::Validate { item: Some(req.param(0)), all: false, changes: false, specs: false, strict: false };
    let (execution, etag) = req.run(cmd)?;
    let mut results = match execution.outcome {
        CommandOutcome::Validate(v) => v.results,
        _ => return Err(wrong_outcome("validate")),
    };
    // A Validate with an item answers exactly one result.
    let r = results.pop().ok_or_else(|| HttpError::internal("validate: empty result set"))?;
    let dto = ValidateChangeResponse { change: r.change, valid: r.valid, errors: r.errors, warnings: r.warnings };
    Ok(ok(&dto, etag))
}

/// `GET /changes/{name}/analyze`: the Engine's whole report.
pub(crate) fn analyze_change(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::Analyze { change: Some(req.param(0)) })?;
    match execution.outcome {
        CommandOutcome::Analyze(report) => Ok(ok(&wire::analyze_report(report), etag)),
        _ => Err(wrong_outcome("analyze")),
    }
}

/// `GET /changes/{name}/instructions/{*artifact}`: `apply` gives the apply
/// view, any other artifact its instructions.
pub(crate) fn instructions(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let cmd = Command::Instructions { artifact: Some(req.param(1)), change: Some(req.param(0)), schema: None };
    let (execution, etag) = req.run(cmd)?;
    match execution.outcome {
        CommandOutcome::Instructions(InstructionsOutcome::Apply(apply)) => {
            Ok(ok(&wire::apply_instructions(apply), etag))
        }
        CommandOutcome::Instructions(InstructionsOutcome::Artifact(artifact)) => {
            Ok(ok(&wire::artifact_instructions(artifact), etag))
        }
        _ => Err(wrong_outcome("instructions")),
    }
}

/// `GET /changes/{name}/artifacts/{*artifact}`: the content, and the
/// document's version for a later `If-Match` write.
pub(crate) fn get_artifact(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (name, artifact) = (req.param(0), req.param(1));
    let cmd = Command::ArtifactCat { artifact: artifact.clone(), change: Some(name.clone()) };
    let (execution, etag) = req.run(cmd)?;
    let content = match execution.outcome {
        CommandOutcome::ArtifactCat(content) => content,
        _ => return Err(wrong_outcome("artifact")),
    };
    let version = match wire::artifact_rel_path(&artifact) {
        Some(rel) => documents::document_version(req.port(), req.scope(), &name, &rel)?.map_or(0, |v| v.get()),
        None => 0,
    };
    Ok(ok(&ArtifactContent { artifact, content, version }, etag))
}

/// `GET /specs`.
pub(crate) fn list_specs(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let cmd = Command::List { sort: "name".to_string(), specs: true, changes: false, worktrees: Default::default() };
    let (execution, etag) = req.run(cmd)?;
    let specs = match execution.outcome {
        CommandOutcome::List(list) => list.specs.unwrap_or_else(|| serde_json::json!([])),
        _ => return Err(wrong_outcome("specs")),
    };
    let specs: Vec<SpecSummary> =
        serde_json::from_value(specs).map_err(|e| HttpError::internal(format!("specs shape: {e}")))?;
    Ok(ok(&ListSpecsResponse { specs }, etag))
}

/// `GET /language`.
pub(crate) fn language(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::LanguageShow)?;
    match execution.outcome {
        CommandOutcome::Language(content) => Ok(ok(&LanguageResponse { content }, etag)),
        _ => Err(wrong_outcome("language")),
    }
}

// --- changes, tasks and the quality stations ---

/// `POST /changes`.
pub(crate) fn create_change(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: CreateChangeRequest = req.body()?;
    let cmd = Command::NewChange {
        name: body.name,
        description: body.description,
        schema: body.schema,
        agent: body.agent,
        from_discussion: body.from_discussion,
        last: body.last,
    };
    let (execution, etag) = req.run(cmd)?;
    let outcome = match execution.outcome {
        CommandOutcome::NewChange(o) => o,
        _ => return Err(wrong_outcome("create-change")),
    };
    let dto = CreateChangeResponse {
        name: outcome.name,
        schema: Some(outcome.schema),
        repo: Some(req.repo.key.clone()),
        lifecycle: None,
    };
    Ok(ok(&dto, etag))
}

#[derive(serde::Deserialize)]
struct ForceQuery {
    #[serde(default)]
    force: bool,
}

/// `DELETE /changes/{name}[?force=true]`: the Engine's discard, its
/// started-work guard included.
pub(crate) fn delete_change(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: ForceQuery = req.query()?;
    let (execution, etag) = req.run(Command::Discard { change: req.param(0), force: query.force })?;
    let outcome = match execution.outcome {
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
    Ok(ok(&dto, etag))
}

/// `POST /changes/{name}/tasks/move`.
pub(crate) fn move_task(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: MoveTaskRequest = req.body()?;
    let cmd = Command::TaskMove { change: req.param(0), from: body.from, to: body.to, before: body.before };
    let (execution, etag) = req.run(cmd)?;
    match execution.outcome {
        CommandOutcome::TaskMove(o) => Ok(ok(&MoveTaskResponse { change: o.change, description: o.description }, etag)),
        _ => Err(wrong_outcome("task move")),
    }
}

/// `PUT /changes/{name}/artifacts/{*artifact}` with `If-Match`: `0` creates,
/// a positive number overwrites that exact version. The check here answers
/// a stale client read at once. The command then reads the document itself,
/// and its commit lands only over the version `If-Match` named: a writer
/// that lands in between makes it a conflict, on any store. The new
/// document version is read after the commit.
pub(crate) fn put_artifact(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (name, artifact) = (req.param(0), req.param(1));
    let body: PutArtifactRequest = req.body()?;
    let if_match = req
        .header("if-match")
        .ok_or_else(|| HttpError::invalid_argument("missing If-Match header"))?
        .trim()
        .parse::<u64>()
        .map_err(|_| HttpError::invalid_argument("If-Match must be a version number"))?;
    let rel = wire::artifact_rel_path(&artifact)
        .ok_or_else(|| HttpError::invalid_argument(format!("unknown artifact '{artifact}'")))?;
    let cmd = artifact_write_command(&artifact, name.clone(), body.content)?;

    let version = || documents::document_version(req.port(), req.scope(), &name, &rel);
    let seen = version()?;
    check_if_match(if_match, seen.map(|v| v.get()))?;
    let write_at = documents::artifact_key(&name, &rel).map(|key| (key, seen.map_or(Expected::Absent, Expected::At)));
    let (_, etag) = req.run_with(cmd, RunOptions { write_at, ..req.options() })?;
    let written = version()?.map_or(0, |v| v.get());
    Ok(ok(&PutArtifactResponse { artifact, version: written }, etag))
}

/// `0` is create-only (the document must be absent); a positive value needs
/// the document at exactly that version.
fn check_if_match(if_match: u64, current: Option<u64>) -> Result<(), HttpError> {
    let satisfied = match (if_match, current) {
        (0, None) => true,
        (0, Some(_)) => false,
        (n, Some(c)) => n == c,
        (_, None) => false,
    };
    if satisfied {
        return Ok(());
    }
    let actual = current.map(|c| c.to_string()).unwrap_or_else(|| "absent".to_string());
    Err(HttpError::revision_conflict(format!("expected version {if_match}, actual {actual}")))
}

/// An artifact id as the Engine's new-artifact command. `force` is always
/// set: the `If-Match` precondition already chose create or overwrite.
fn artifact_write_command(artifact: &str, change: String, content: String) -> Result<Command, HttpError> {
    let (kind, capability) = match artifact {
        "proposal" | "design" | "tasks" => (artifact.to_string(), None),
        _ => match artifact.strip_prefix("specs/") {
            Some(cap) if !cap.is_empty() && !cap.contains('/') => ("spec".to_string(), Some(cap.to_string())),
            _ => return Err(HttpError::invalid_argument(format!("unknown artifact '{artifact}'"))),
        },
    };
    Ok(Command::NewArtifact {
        kind,
        capability,
        change: Some(change),
        content: Some(content),
        force: true,
        // A direct write, not the CLI's creation verb: the naming gate's
        // second net (the validate warning) covers this entrance.
        new_capability: true,
    })
}

/// `POST /changes/{name}/tasks/{taskId}/done`. The touched files are the
/// sender's; none is "nothing to attribute", never a reason to probe this
/// machine.
pub(crate) fn task_done(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: TaskDoneRequest = req.body()?;
    let cmd = Command::TaskDone {
        task_id: req.param(1),
        change: Some(req.param(0)),
        touched_files: Some(body.touched_files),
        head_commit: body.head_commit,
    };
    let (execution, etag) = req.run(cmd)?;
    match execution.outcome {
        CommandOutcome::TaskDone(o) => {
            Ok(ok(&TaskDoneResponse { task_desc: o.description, already_done: o.already }, etag))
        }
        _ => Err(wrong_outcome("task-done")),
    }
}

/// `POST /changes/{name}/tasks/{taskId}/undone`.
pub(crate) fn task_undone(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::TaskUndone { task_id: req.param(1), change: Some(req.param(0)) })?;
    match execution.outcome {
        CommandOutcome::TaskUndone(o) => {
            Ok(ok(&TaskUndoneResponse { task_desc: o.description, already_undone: o.already }, etag))
        }
        _ => Err(wrong_outcome("task-undone")),
    }
}

/// One of the two quality stations. Their routes differ only in the
/// command and the outcome variant; the wire types are shared.
#[derive(Clone, Copy)]
enum Station {
    Review,
    Verify,
}

impl Station {
    fn name(self) -> &'static str {
        match self {
            Station::Review => "review",
            Station::Verify => "verify",
        }
    }

    /// `GET /changes/{name}/<station>`: the ticket.
    fn show(self, req: &Request<'_>) -> Result<HttpResponse, HttpError> {
        let change = req.param(0);
        let (execution, etag) = req.run(match self {
            Station::Review => Command::ReviewShow { change },
            Station::Verify => Command::VerifyShow { change },
        })?;
        let (CommandOutcome::ReviewShow(o) | CommandOutcome::VerifyShow(o)) = execution.outcome else {
            return Err(wrong_outcome(&format!("{}-show", self.name())));
        };
        let dto = ReviewTicketResponse {
            change: o.change,
            rounds: o.ticket.rounds.iter().map(wire::review_round).collect(),
            last_round: wire::review_round(o.ticket.last_round()),
            content: o.content,
        };
        Ok(ok(&dto, etag))
    }

    /// `POST /changes/{name}/<station>/rounds`. The verify station's Engine
    /// refuses while tasks are open.
    fn add_round(self, req: &Request<'_>) -> Result<HttpResponse, HttpError> {
        let body: AddReviewRoundRequest = req.body()?;
        let (change, content) = (req.param(0), body.content);
        let (execution, etag) = req.run(match self {
            Station::Review => Command::ReviewAddRound { change, content },
            Station::Verify => Command::VerifyAddRound { change, content },
        })?;
        let (CommandOutcome::ReviewAddRound(o) | CommandOutcome::VerifyAddRound(o)) = execution.outcome else {
            return Err(wrong_outcome(&format!("{}-add-round", self.name())));
        };
        Ok(ok(&AddReviewRoundResponse { round: o.round as u64 }, etag))
    }

    /// `POST /changes/{name}/<station>/stamp`. The submitted fingerprints are
    /// the caller's work-tree truth; the Engine checks the path set and never
    /// hashes again.
    fn stamp(self, req: &Request<'_>) -> Result<HttpResponse, HttpError> {
        let body: StampReviewRequest = req.body()?;
        let scope = body
            .scope
            .into_iter()
            .map(|e| speclink_core::model::ReviewedScopeEntry { path: e.path, hash: e.hash })
            .collect();
        let (change, accept, tool, missing) = (req.param(0), body.accept, body.agent, body.missing);
        let (execution, etag) = req.run(match self {
            Station::Review => Command::ReviewStamp { change, accept, tool, scope, missing },
            Station::Verify => Command::VerifyStamp { change, accept, tool, scope, missing },
        })?;
        let (CommandOutcome::ReviewStamp(o) | CommandOutcome::VerifyStamp(o)) = execution.outcome else {
            return Err(wrong_outcome(&format!("{}-stamp", self.name())));
        };
        Ok(ok(&StampReviewResponse { change: o.change }, etag))
    }

    /// `DELETE /changes/{name}/<station>`: the ticket is the only record of
    /// the station's work.
    fn discard(self, req: &Request<'_>) -> Result<HttpResponse, HttpError> {
        let change = req.param(0);
        let (execution, etag) = req.run(match self {
            Station::Review => Command::ReviewDiscard { change },
            Station::Verify => Command::VerifyDiscard { change },
        })?;
        let (CommandOutcome::ReviewDiscard(o) | CommandOutcome::VerifyDiscard(o)) = execution.outcome else {
            return Err(wrong_outcome(&format!("{}-discard", self.name())));
        };
        Ok(ok(&DiscardReviewResponse { change: o.change }, etag))
    }
}

pub(crate) fn review_show(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Review.show(req)
}

pub(crate) fn review_add_round(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Review.add_round(req)
}

pub(crate) fn review_stamp(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Review.stamp(req)
}

pub(crate) fn review_discard(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Review.discard(req)
}

pub(crate) fn verify_show(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Verify.show(req)
}

pub(crate) fn verify_add_round(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Verify.add_round(req)
}

pub(crate) fn verify_stamp(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Verify.stamp(req)
}

pub(crate) fn verify_discard(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    Station::Verify.discard(req)
}

/// `POST /changes/{name}/claim`: the first claim stamps the caller; the same
/// caller again writes nothing; someone else's claim is refused with its
/// holder.
pub(crate) fn claim(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::Claim { name: req.param(0) })?;
    match execution.outcome {
        CommandOutcome::Claim(o) => Ok(ok(&ClaimResponse { lifecycle: None, claimed_by: o.claimed_by }, etag)),
        _ => Err(wrong_outcome("claim")),
    }
}

/// `POST /changes/{name}/depends`.
pub(crate) fn change_depends(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: SetDependsRequest = req.body()?;
    let (execution, etag) = req.run(Command::ChangeDepends { name: req.param(0), on: body.on, remove: body.remove })?;
    match execution.outcome {
        CommandOutcome::ChangeDepends(o) => {
            Ok(ok(&SetDependsResponse { change: o.change, depends_on: o.depends_on }, etag))
        }
        _ => Err(wrong_outcome("change depends")),
    }
}

/// `POST /changes/{name}/schema`.
pub(crate) fn change_schema(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: SetSchemaRequest = req.body()?;
    let (execution, etag) = req.run(Command::ChangeSchema { name: req.param(0), schema: body.schema })?;
    match execution.outcome {
        CommandOutcome::ChangeSchema(o) => Ok(ok(
            &SetSchemaResponse {
                change: o.change,
                schema: o.schema,
                previous: o.previous,
                changed: o.changed,
                verify_stamp_cleared: o.verify_stamp_cleared,
            },
            etag,
        )),
        _ => Err(wrong_outcome("change schema")),
    }
}

/// `POST /changes/{name}/rank`: `position` is `before` or `after`, else 400.
pub(crate) fn change_rank(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: RankChangeRequest = req.body()?;
    let position = speclink_core::plan::Side::parse(&body.position).ok_or_else(|| {
        HttpError::invalid_argument(format!("position must be 'before' or 'after', got '{}'", body.position))
    })?;
    let cmd = Command::ChangeRank { name: req.param(0), anchor: body.anchor, position, force: body.force };
    let (execution, etag) = req.run(cmd)?;
    match execution.outcome {
        CommandOutcome::ChangeRank(o) => Ok(ok(
            &RankChangeResponse { change: o.change, anchor: o.anchor, position: o.position.as_str().to_string() },
            etag,
        )),
        _ => Err(wrong_outcome("change rank")),
    }
}

/// `POST /changes/{name}/in-progress`: the first mark stamps the caller; a
/// repeat or an unknown name is the Engine's silent success.
pub(crate) fn in_progress(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::InProgressAdd { name: req.param(0) })?;
    match execution.outcome {
        CommandOutcome::InProgressAdd(_) => Ok(ok(&serde_json::json!({}), etag)),
        _ => Err(wrong_outcome("in-progress")),
    }
}

/// `DELETE /changes/{name}/in-progress`: refused with the evidence when
/// work was recorded.
pub(crate) fn in_progress_remove(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::InProgressRemove { name: req.param(0) })?;
    match execution.outcome {
        CommandOutcome::InProgressRemove(o) => Ok(ok(&InProgressRemoveResponse { removed: o.removed }, etag)),
        _ => Err(wrong_outcome("in-progress-remove")),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveQuery {
    #[serde(default)]
    carry_review: bool,
    #[serde(default)]
    carry_verify: bool,
}

/// `POST /changes/{name}/archive[?carryReview=true][&carryVerify=true]`.
pub(crate) fn archive(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: ArchiveQuery = req.query()?;
    let cmd = Command::Archive {
        change: Some(req.param(0)),
        skip_specs: false,
        no_validate: false,
        mark_tasks_complete: false,
        carry_review: query.carry_review,
        carry_verify: query.carry_verify,
    };
    let (execution, etag) = req.run(cmd)?;
    let outcome = match execution.outcome {
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
        // The remote caller keys its full rendering on datedName, so it
        // travels whenever the Engine produced an outcome.
        dated_name: Some(outcome.dated_name),
        snapshot_created: Some(outcome.snapshot_created),
        archived_discussions: outcome
            .archived_discussions
            .into_iter()
            .map(|(slug, file)| ArchivedDiscussion { slug, file })
            .collect(),
        evidence_recorded: Some(outcome.evidence_recorded),
    };
    Ok(ok(&dto, etag))
}

// --- discussions and the plan ---

/// `GET /plan`: the change-level execution order. The ranks come from the
/// board order, read leniently (absent or unusable is no ranks) and never
/// written back. A dependency cycle is a refused 409 with the Engine's text.
/// The ETag is the version the board order was read at: the first read.
pub(crate) fn plan(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (board, version) = documents::board_order(req.port(), req.scope())?;
    let ranks = BoardOrderDoc::from_content(board.content.as_deref()).changes;
    let (execution, _) = req.run(Command::Plan { ranks: Some(ranks) })?;
    match execution.outcome {
        CommandOutcome::Plan(plan) => Ok(ok_at(&wire::plan(plan), version)),
        _ => Err(wrong_outcome("plan")),
    }
}

#[derive(serde::Deserialize)]
struct ListDiscussionsQuery {
    #[serde(default)]
    archived: bool,
}

/// `GET /discussions[?archived=true]`.
pub(crate) fn list_discussions(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: ListDiscussionsQuery = req.query()?;
    let (execution, etag) = req.run(Command::DiscussList { archived: query.archived })?;
    match execution.outcome {
        CommandOutcome::DiscussList(list) => {
            Ok(ok(&ListDiscussionsResponse { discussions: wire::discussions(list) }, etag))
        }
        _ => Err(wrong_outcome("discuss-list")),
    }
}

/// `GET /discussions/search?q=<space-separated keywords>`: the search
/// `speclink discuss search` runs. A missing or blank `q` is the Engine's
/// refusal of an empty keyword list, a 400.
pub(crate) fn search_discussions(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: SearchQuery = req.query()?;
    let terms: Vec<String> = query.q.split_whitespace().map(str::to_string).collect();
    let (execution, etag) = req.run(Command::DiscussSearch { terms })?;
    let hits = match execution.outcome {
        CommandOutcome::DiscussSearch(hits) => hits,
        _ => return Err(wrong_outcome("discuss-search")),
    };
    let (infos, matches): (Vec<_>, Vec<_>) = hits.into_iter().map(|h| (h.info, h.matches)).unzip();
    let hits = wire::discussions(infos)
        .into_iter()
        .zip(matches)
        .map(|(info, matches)| DiscussionHit {
            info,
            matches: matches
                .into_iter()
                .map(|m| DiscussionMatch { kind: m.kind, where_: m.where_, text: m.text })
                .collect(),
        })
        .collect();
    Ok(ok(&SearchDiscussionsResponse { hits }, etag))
}

/// [`Request::run`] for the four discussion writes the Engine turns away
/// with a [`WriteRefusal`]. The command layer files it as `Error` (500);
/// its kind says what it is, and the text travels as it is: the subject
/// missing is 404, a slug, kind or topic that cannot be one 400, a guard
/// (exists, archived, not linked) 409.
fn run_discussion_write(req: &Request<'_>, cmd: Command) -> Result<(PortExecution, String), HttpError> {
    req.try_run(cmd, req.options()).map_err(|e| {
        let kind = match &e {
            ExecError::Command(c) if c.code == ErrorCode::Error => {
                c.source.as_ref().and_then(|s| s.downcast_ref::<WriteRefusal>()).map(|refusal| refusal.kind)
            }
            _ => None,
        };
        let error = HttpError::from(e);
        match kind {
            Some(WriteRefusalKind::NotFound) => HttpError::not_found(error.message),
            Some(WriteRefusalKind::Invalid) => HttpError::invalid_argument(error.message),
            Some(WriteRefusalKind::Guard) => HttpError::refused(error.message),
            None => error,
        }
    })
}

/// `POST /discussions`.
pub(crate) fn create_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: CreateDiscussionRequest = req.body()?;
    let cmd = Command::DiscussNew { topic: body.topic, slug: body.slug, kind: body.kind };
    let (execution, etag) = run_discussion_write(req, cmd)?;
    match execution.outcome {
        CommandOutcome::DiscussNew(info) => {
            Ok(ok(&CreateDiscussionResponse { slug: info.slug, topic: info.topic, path: info.path }, etag))
        }
        _ => Err(wrong_outcome("discuss-new")),
    }
}

/// `DELETE /discussions/{slug}[?force=true]`: no round is deleted at once;
/// rounds without force are refused.
pub(crate) fn delete_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let query: ForceQuery = req.query()?;
    let cmd = Command::DiscussDiscard { slug: req.param(0), force: query.force };
    let (execution, etag) = run_discussion_write(req, cmd)?;
    match execution.outcome {
        CommandOutcome::DiscussDiscard(o) => Ok(ok(&DiscardDiscussionResponse { slug: o.slug }, etag)),
        _ => Err(wrong_outcome("discuss-discard")),
    }
}

/// `POST /discussions/{slug}/link`: the change side's `from_discussion` link.
pub(crate) fn link_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: BindDiscussionRequest = req.body()?;
    let (execution, etag) = run_discussion_write(req, Command::DiscussLink { slug: req.param(0), change: body.change })?;
    match execution.outcome {
        CommandOutcome::DiscussLink(o) => Ok(ok(&BindDiscussionResponse { slug: o.slug, change: o.change }, etag)),
        _ => Err(wrong_outcome("discuss-link")),
    }
}

/// `POST /discussions/{slug}/seal`: marks the discussion promoted once the
/// link exists.
pub(crate) fn seal_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: BindDiscussionRequest = req.body()?;
    let cmd = Command::DiscussSeal { slug: req.param(0), change: body.change, last: body.last };
    let (execution, etag) = run_discussion_write(req, cmd)?;
    match execution.outcome {
        CommandOutcome::DiscussSeal(o) => Ok(ok(&BindDiscussionResponse { slug: o.slug, change: o.change }, etag)),
        _ => Err(wrong_outcome("discuss-seal")),
    }
}

/// `GET /discussions/{slug}`: with the cuts of the same record.
pub(crate) fn show_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::DiscussShow { slug: req.param(0) })?;
    let show = match execution.outcome {
        CommandOutcome::DiscussShow(show) => show,
        _ => return Err(wrong_outcome("discuss-show")),
    };
    let info = show.info.ok_or_else(|| HttpError::internal("discuss-show: missing discussion info"))?;
    let cuts = speclink_core::discuss::record_cuts(&show.content, &info.head.promoted_to)
        .into_iter()
        .map(|c| DiscussionCut {
            cut: c.cut,
            title: c.title,
            note: c.note,
            line: c.line,
            changes: c.changes,
            changes_in_bold: c.changes_in_bold,
            unpromoted: c.unpromoted,
        })
        .collect();
    Ok(ok(&ShowDiscussionResponse { info: wire::discussion(info), content: show.content, cuts }, etag))
}

/// `PUT /discussions/{slug}/context`.
pub(crate) fn set_discussion_context(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: SetDiscussionContextRequest = req.body()?;
    let (_, etag) = req.run(Command::DiscussContext { slug: req.param(0), content: body.content })?;
    Ok(ok(&serde_json::json!({}), etag))
}

/// `POST /discussions/{slug}/rounds`.
pub(crate) fn add_discussion_round(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: AddDiscussionRoundRequest = req.body()?;
    let cmd = Command::DiscussAddRound { slug: req.param(0), mode: body.mode, content: body.content };
    let (execution, etag) = req.run(cmd)?;
    match execution.outcome {
        CommandOutcome::DiscussAddRound(o) => Ok(ok(&AddDiscussionRoundResponse { round: o.round as u64 }, etag)),
        _ => Err(wrong_outcome("discuss-add-round")),
    }
}

/// `POST /discussions/{slug}/conclude`. When the closing archive step fails,
/// the conclusion already landed; the error says so, as the CLI does.
pub(crate) fn conclude_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: ConcludeDiscussionRequest = req.body()?;
    let cmd = Command::DiscussConclude { slug: req.param(0), content: body.content, hold: body.hold };
    let (execution, etag) = req.run(cmd)?;
    let o = match execution.outcome {
        CommandOutcome::DiscussConclude(o) => o,
        _ => return Err(wrong_outcome("discuss-conclude")),
    };
    if let Some(reason) = o.closing_error {
        return Err(HttpError::internal(format!("conclude closing step failed to archive the record: {reason}")));
    }
    let dto = ConcludeDiscussionResponse { restale_flagged: o.restale_flagged, auto_archived: o.auto_archived, held: o.held };
    Ok(ok(&dto, etag))
}

/// `POST /discussions/{slug}/archive`.
pub(crate) fn archive_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let (execution, etag) = req.run(Command::DiscussArchive { slug: req.param(0) })?;
    match execution.outcome {
        CommandOutcome::DiscussArchive(o) => Ok(ok(
            &ArchiveDiscussionResponse { archived_to: format!("discussions/archive/{}", o.archived_file) },
            etag,
        )),
        _ => Err(wrong_outcome("discuss-archive")),
    }
}

/// `POST /discussions/{slug}/promote`. The new change's directory is not on
/// the wire: it is a place in the store, meaningless to a local user.
pub(crate) fn promote_discussion(req: &Request<'_>) -> Result<HttpResponse, HttpError> {
    let body: PromoteDiscussionRequest = req.body()?;
    let (execution, etag) = req.run(Command::DiscussPromote { slug: req.param(0), name: body.name, last: body.last })?;
    match execution.outcome {
        CommandOutcome::DiscussPromote(o) => Ok(ok(&PromoteDiscussionResponse { change: o.change }, etag)),
        _ => Err(wrong_outcome("discuss-promote")),
    }
}

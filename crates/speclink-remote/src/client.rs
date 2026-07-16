//! The single request layer plus the per-verb path mapping.
//!
//! Every request carries the contract's three headers (`Authorization`,
//! `X-Speclink-Api-Version`, `X-Speclink-Repo`); every non-2xx response goes
//! through the protocol registry mapping. Requests and responses are
//! speclink-protocol DTOs end to end — no verb assembles or picks apart raw
//! JSON, and no verb re-implements transport or error handling.

use crate::{translate_protocol_error, translate_transport, RemoteError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use speclink_protocol::binding::BindingResponse;
use speclink_protocol::context::{ContextSnapshot, ContextSnapshotRequest};
use speclink_protocol::command::{
    AddDiscussionRoundRequest, AddDiscussionRoundResponse, ArchiveDiscussionResponse,
    ArchiveResponse, ClaimResponse, ConcludeDiscussionRequest, CreateChangeRequest,
    CreateChangeResponse, CreateDiscussionRequest, CreateDiscussionResponse,
    PromoteDiscussionRequest, PromoteDiscussionResponse, PutArtifactRequest,
    PutArtifactResponse, SetDiscussionContextRequest, TaskDoneRequest, TaskDoneResponse,
    TaskUndoneResponse,
};
use speclink_protocol::drift::SpecDriftResponse;
use speclink_protocol::query::{
    ApplyInstructions, ArtifactContent, ArtifactInstructions, ChangeStatus, ConfigResponse,
    LanguageResponse, ListChangesResponse, ListDiscussionsResponse, ListSpecsResponse,
    ShowDiscussionResponse, WhoamiResponse,
};

/// The contract major version this client speaks (`X-Speclink-Api-Version`)
/// — the protocol crate's constant is the single source.
pub const API_VERSION: &str = speclink_protocol::API_VERSION;

/// The bare-object body for verbs whose request carries no fields.
#[derive(Serialize)]
struct Empty {}

/// The outcome of a context snapshot request: the scope was unchanged since the
/// caller's known snapshot id (a 304 with no body), or a fresh snapshot. The
/// two-valued shape lets a projection refresh skip rewriting when nothing moved.
#[derive(Debug)]
pub enum ContextSnapshotOutcome {
    /// The scope state token still matches the caller's known snapshot id.
    Unchanged,
    /// A fresh consistent snapshot of the scope's context documents.
    Fresh(ContextSnapshot),
}

/// A client bound to one project-scoped base URL, one token, and (optionally)
/// one repo identity.
pub struct Client {
    base: String,
    token: String,
    repo: Option<String>,
    agent: ureq::Agent,
}

impl Client {
    /// `base_url` is the project-scoped connection URL (a trailing slash is
    /// tolerated); `repo` is the registered repo name from the connection
    /// file, when declared.
    pub fn new(base_url: &str, token: &str, repo: Option<&str>) -> Client {
        Client {
            base: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            repo: repo.map(str::to_string),
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .build(),
        }
    }

    /// One request skeleton with the contract's three headers — every call,
    /// handshake included, goes through here so the headers exist exactly once.
    fn request(&self, method: &str, path: &str) -> ureq::Request {
        let mut req = self
            .agent
            .request(method, &format!("{}{}", self.base, path))
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("X-Speclink-Api-Version", API_VERSION);
        if let Some(repo) = &self.repo {
            req = req.set("X-Speclink-Repo", repo);
        }
        req
    }

    /// One request through the contract's header and error rules. Every verb
    /// funnels through here — the body is the request DTO's serialization,
    /// the response parses into the response DTO, and translation exists
    /// exactly once.
    fn send<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&B>,
        extra_headers: &[(&str, &str)],
    ) -> Result<T, RemoteError> {
        let mut req = self.request(method, path);
        for (k, v) in extra_headers {
            req = req.set(k, v);
        }
        let result = match body {
            Some(payload) => req.send_json(payload),
            None => req.call(),
        };
        match result {
            Ok(resp) => resp.into_json().map_err(|_| RemoteError {
                message: "unexpected server response — the server did not return valid JSON"
                    .into(),
                reason: None,
            }),
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(translate_protocol_error(status, &body))
            }
            Err(ureq::Error::Transport(_)) => Err(translate_transport()),
        }
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, RemoteError> {
        self.send::<T, Empty>("GET", path, None, &[])
    }

    fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, RemoteError> {
        self.send("POST", path, Some(body), &[])
    }

    // --- binding handshake (the connection precondition, fail closed) ---

    /// `GET /binding` — the handshake that precedes any verb flow. The only
    /// client-side judgment is API version compatibility; a missing,
    /// unauthorized, or ambiguous binding arrives as the server's registry
    /// refusal and is relayed, never resolved by probing or picking a
    /// candidate. Capability declarations (events transports, polling) are
    /// parsed and kept — no event connection is opened here.
    pub fn handshake(&self) -> Result<BindingResponse, RemoteError> {
        let binding: BindingResponse = self.get("/binding")?;
        if binding.api_version != API_VERSION {
            return Err(RemoteError {
                message: "server does not support this CLI's API version — upgrade the CLI or the server"
                    .into(),
                reason: Some("api_version_unsupported".into()),
            });
        }
        Ok(binding)
    }

    // --- read path ---

    /// `GET /changes`
    pub fn list_changes(&self) -> Result<ListChangesResponse, RemoteError> {
        self.get("/changes")
    }

    /// `GET /changes/{name}`
    pub fn get_change(&self, name: &str) -> Result<ChangeStatus, RemoteError> {
        self.get(&format!("/changes/{name}"))
    }

    /// `GET /changes/{name}/instructions/apply`
    pub fn apply_instructions(&self, name: &str) -> Result<ApplyInstructions, RemoteError> {
        self.get(&format!("/changes/{name}/instructions/apply"))
    }

    /// `GET /changes/{name}/instructions/{artifact}`
    pub fn artifact_instructions(
        &self,
        name: &str,
        artifact: &str,
    ) -> Result<ArtifactInstructions, RemoteError> {
        self.get(&format!("/changes/{name}/instructions/{artifact}"))
    }

    /// `GET /changes/{name}/artifacts/{artifact}`
    pub fn get_artifact(
        &self,
        name: &str,
        artifact: &str,
    ) -> Result<ArtifactContent, RemoteError> {
        self.get(&format!("/changes/{name}/artifacts/{artifact}"))
    }

    /// `GET /changes/{name}/drift` — the change's spec-side drift and the basis
    /// digests of the snapshot it was computed at. The workspace side is the
    /// caller's own to collect: it is not on the wire and never was.
    pub fn spec_drift(&self, name: &str) -> Result<SpecDriftResponse, RemoteError> {
        self.get(&format!("/changes/{name}/drift"))
    }

    /// `GET /specs`
    pub fn list_specs(&self) -> Result<ListSpecsResponse, RemoteError> {
        self.get("/specs")
    }

    /// `GET /language`
    pub fn language(&self) -> Result<LanguageResponse, RemoteError> {
        self.get("/language")
    }

    /// `GET /config`
    pub fn config(&self) -> Result<ConfigResponse, RemoteError> {
        self.get("/config")
    }

    /// `GET /whoami`
    pub fn whoami(&self) -> Result<WhoamiResponse, RemoteError> {
        self.get("/whoami")
    }

    // --- context snapshot ---

    /// `POST /context` with the request body and, when the caller already holds
    /// a snapshot, its id as `If-None-Match`. A 304 (the scope state token still
    /// matches) is [`ContextSnapshotOutcome::Unchanged`]; a 200 is a fresh
    /// snapshot. Every other status goes through the registry translation like
    /// any verb — this is the only conditional path that reads a 304, so it does
    /// not funnel through `send` (which has no not-modified outcome). A 304 is a
    /// non-redirect 3xx: ureq surfaces it as `Ok` with status 304, not an error.
    pub fn context_snapshot(
        &self,
        request: &ContextSnapshotRequest,
        known_snapshot_id: Option<&str>,
    ) -> Result<ContextSnapshotOutcome, RemoteError> {
        let mut req = self.request("POST", "/context");
        if let Some(id) = known_snapshot_id {
            req = req.set("If-None-Match", id);
        }
        match req.send_json(request) {
            Ok(resp) if resp.status() == 304 => Ok(ContextSnapshotOutcome::Unchanged),
            Ok(resp) => {
                let snapshot: ContextSnapshot = resp.into_json().map_err(|_| RemoteError {
                    message: "unexpected server response — the server did not return valid JSON"
                        .into(),
                    reason: None,
                })?;
                Ok(ContextSnapshotOutcome::Fresh(snapshot))
            }
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(translate_protocol_error(status, &body))
            }
            Err(ureq::Error::Transport(_)) => Err(translate_transport()),
        }
    }

    // --- write path ---

    /// `POST /changes`
    pub fn create_change(
        &self,
        request: CreateChangeRequest,
    ) -> Result<CreateChangeResponse, RemoteError> {
        self.post("/changes", &request)
    }

    /// `PUT /changes/{name}/artifacts/{artifact}` with `If-Match: <version>`
    /// (`0` = create-only). The version must always be the one obtained when
    /// the content was read — there is no force-write.
    pub fn put_artifact(
        &self,
        name: &str,
        artifact: &str,
        content: &str,
        if_match: u64,
    ) -> Result<PutArtifactResponse, RemoteError> {
        self.send(
            "PUT",
            &format!("/changes/{name}/artifacts/{artifact}"),
            Some(&PutArtifactRequest { content: content.to_string() }),
            &[("If-Match", &if_match.to_string())],
        )
    }

    /// `POST /changes/{name}/tasks/{taskId}/done`
    pub fn task_done(
        &self,
        name: &str,
        task_id: &str,
        touched_files: &[String],
    ) -> Result<TaskDoneResponse, RemoteError> {
        self.post(
            &format!("/changes/{name}/tasks/{task_id}/done"),
            &TaskDoneRequest { touched_files: touched_files.to_vec() },
        )
    }

    /// `POST /changes/{name}/tasks/{taskId}/undone` — unchecking records no
    /// touched files, so the body is always the bare object.
    pub fn task_undone(
        &self,
        name: &str,
        task_id: &str,
    ) -> Result<TaskUndoneResponse, RemoteError> {
        self.post(&format!("/changes/{name}/tasks/{task_id}/undone"), &Empty {})
    }

    /// `POST /changes/{name}/claim`
    pub fn claim(&self, name: &str) -> Result<ClaimResponse, RemoteError> {
        self.post(&format!("/changes/{name}/claim"), &Empty {})
    }

    /// `POST /changes/{name}/archive`
    pub fn archive(&self, name: &str) -> Result<ArchiveResponse, RemoteError> {
        self.post(&format!("/changes/{name}/archive"), &Empty {})
    }

    // --- discussions ---

    /// `GET /discussions?archived=`
    pub fn list_discussions(
        &self,
        archived: bool,
    ) -> Result<ListDiscussionsResponse, RemoteError> {
        if archived {
            self.get("/discussions?archived=true")
        } else {
            self.get("/discussions")
        }
    }

    /// `POST /discussions`
    pub fn new_discussion(&self, topic: &str) -> Result<CreateDiscussionResponse, RemoteError> {
        self.post("/discussions", &CreateDiscussionRequest { topic: topic.to_string() })
    }

    /// `GET /discussions/{slug}`
    pub fn show_discussion(&self, slug: &str) -> Result<ShowDiscussionResponse, RemoteError> {
        self.get(&format!("/discussions/{slug}"))
    }

    /// `PUT /discussions/{slug}/context` — the response body carries nothing
    /// the client consumes.
    pub fn discussion_context(&self, slug: &str, content: &str) -> Result<(), RemoteError> {
        self.send::<serde::de::IgnoredAny, _>(
            "PUT",
            &format!("/discussions/{slug}/context"),
            Some(&SetDiscussionContextRequest { content: content.to_string() }),
            &[],
        )
        .map(|_| ())
    }

    /// `POST /discussions/{slug}/rounds`
    pub fn discussion_add_round(
        &self,
        slug: &str,
        mode: &str,
        content: &str,
    ) -> Result<AddDiscussionRoundResponse, RemoteError> {
        self.post(
            &format!("/discussions/{slug}/rounds"),
            &AddDiscussionRoundRequest {
                mode: mode.to_string(),
                content: content.to_string(),
            },
        )
    }

    /// `POST /discussions/{slug}/conclude` — the response body carries
    /// nothing the client consumes.
    pub fn discussion_conclude(&self, slug: &str, content: &str) -> Result<(), RemoteError> {
        self.post::<serde::de::IgnoredAny, _>(
            &format!("/discussions/{slug}/conclude"),
            &ConcludeDiscussionRequest { content: content.to_string() },
        )
        .map(|_| ())
    }

    /// `POST /discussions/{slug}/archive`
    pub fn discussion_archive(
        &self,
        slug: &str,
    ) -> Result<ArchiveDiscussionResponse, RemoteError> {
        self.post(&format!("/discussions/{slug}/archive"), &Empty {})
    }

    /// `POST /discussions/{slug}/promote`
    pub fn discussion_promote(
        &self,
        slug: &str,
        name: Option<&str>,
    ) -> Result<PromoteDiscussionResponse, RemoteError> {
        self.post(
            &format!("/discussions/{slug}/promote"),
            &PromoteDiscussionRequest { name: name.map(str::to_string) },
        )
    }
}

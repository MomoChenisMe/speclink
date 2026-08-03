//! Server-scope read routes authenticate the caller without requiring a
//! project or repo header, then expose only the scopes granted by membership.

use crate::common;

use chrono::{Duration, Utc};
use speclink_protocol::binding::ScopeRef;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::query::{
    ArchivedListResponse, ProjectScope, ScopesResponse, SearchResponse,
    SpecDocumentResponse as DocumentResponse,
};
use speclink_protocol::API_VERSION;
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use speclink_store::{CommandContext, DocumentId, ProjectId, RepoId, Scope, TeamStore};
use std::sync::Arc;

struct Fixture {
    base: String,
    identity: Arc<IdentitySqlite>,
    store: Arc<MemoryStore>,
}

impl Fixture {
    fn new() -> Self {
        let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
        let store = Arc::new(MemoryStore::new());
        common::seed_demo_registry(&*identity);
        common::seed_multi_project(&*identity);
        let state = AppState {
            events: common::detached_events(),
            store: store.clone(),
            config: Arc::new(common::demo_config()),
            identity: identity.clone(),
        };
        Self {
            base: common::start(state),
            identity,
            store,
        }
    }

    fn seed_user(&self, email: &str, memberships: &[&str], admin: bool) -> (String, String) {
        let invitation = self
            .identity
            .create_invitation(NewInvitation {
                email: email.to_string(),
                display: email.to_string(),
                memberships: memberships
                    .iter()
                    .map(|project| project.to_string())
                    .collect(),
                admin,
                expires_at: Utc::now() + Duration::days(1),
            })
            .expect("invitation");
        let user_id = self
            .identity
            .accept_invitation(&invitation, "seed-password")
            .expect("accept invitation");
        let (_, pat) = self
            .identity
            .create_pat(&user_id, "test", None)
            .expect("PAT");
        (pat, user_id)
    }

    fn request(&self, token: Option<&str>) -> Result<ureq::Response, ureq::Error> {
        let url = format!("{}/api/speclink/v1/scopes", self.base);
        let mut request = ureq::get(&url);
        if let Some(token) = token {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        request.call()
    }

    fn project_request(
        &self,
        project: &str,
        repo: &str,
        token: &str,
        path: &str,
    ) -> Result<ureq::Response, ureq::Error> {
        ureq::get(&format!(
            "{}/api/speclink/v1/projects/{project}{path}",
            self.base
        ))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", repo)
        .call()
    }

    fn archive_change(&self, project: &str, repo: &str, token: &str, change: &str) {
        let response = ureq::post(&format!(
            "{}/api/speclink/v1/projects/{project}/changes/{change}/archive",
            self.base
        ))
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", API_VERSION)
        .set("X-Speclink-Repo", repo)
        .call()
        .expect("archive verb succeeds");
        assert_eq!(response.status(), 200);
    }

    fn seed_docs(&self, project: &str, repo: &str, docs: Vec<(DocumentId, String)>) {
        let scope = Scope::new(ProjectId::new(project), RepoId::new(repo));
        let mut uow = self
            .store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "seed-read-api".to_string(),
                    actor: "test".to_string(),
                },
            )
            .expect("begin seed unit of work");
        for (document, content) in docs {
            uow.create(document, content);
        }
        self.store.commit(uow, Vec::new()).expect("seed documents");
    }

    fn scopes(&self, token: &str) -> ScopesResponse {
        let response = self.request(Some(token)).expect("scopes request succeeds");
        assert_eq!(response.status(), 200);
        serde_json::from_str(&response.into_string().expect("response body"))
            .expect("ScopesResponse body")
    }
}

struct ReadFixture {
    fixture: Fixture,
    pat: String,
    dated_name: String,
}

impl ReadFixture {
    fn new() -> Self {
        let fixture = Fixture::new();
        let (pat, _) = fixture.seed_user("reader@example.com", &["demo", "multi"], false);
        let long_change_text = format!(
            "## Why\n\n{}MagicToken{}\n",
            "前".repeat(80),
            "後".repeat(80)
        );
        fixture.seed_docs(
            "demo",
            "backend",
            vec![
                (
                    DocumentId::CanonicalSpec {
                        capability: "payments".to_string(),
                    },
                    "# payments Specification\n\nCanonical truth.\n".to_string(),
                ),
                (
                    DocumentId::ChangeMeta {
                        change: "searchable".to_string(),
                    },
                    "schema: spec-driven\ncreated: 2026-07-19\n".to_string(),
                ),
                (
                    DocumentId::ChangeArtifact {
                        change: "searchable".to_string(),
                        artifact: "proposal.md".to_string(),
                    },
                    long_change_text,
                ),
                (
                    DocumentId::ChangeArtifact {
                        change: "searchable".to_string(),
                        artifact: "design.md".to_string(),
                    },
                    "## Context\n\nA second magictoken must not win.\n".to_string(),
                ),
                (
                    DocumentId::Discussion {
                        slug: "search-talk".to_string(),
                        archived: false,
                    },
                    "---\ntopic: Search\nslug: search-talk\nstatus: open\ncreated: 2026-07-19\n---\n\nA live discussion contains MAGICTOKEN too.\n"
                        .to_string(),
                ),
                (
                    DocumentId::ChangeMeta {
                        change: "old-feature".to_string(),
                    },
                    "schema: spec-driven\ncreated: 2026-07-18\ncreated_by: Original Creator <creator@example.com>\nfrom_discussion: source-one, source-two\n"
                        .to_string(),
                ),
                (
                    DocumentId::ChangeArtifact {
                        change: "old-feature".to_string(),
                        artifact: "proposal.md".to_string(),
                    },
                    "## Why\n\nArchived proposal truth.\n".to_string(),
                ),
            ],
        );
        fixture.archive_change("demo", "backend", &pat, "old-feature");

        // The live change carries no tasks.md: the archive verb refuses incomplete
        // changes (task-readiness gate), so the 1/2 tasks document that backs the
        // incomplete-badge assertions is seeded straight into the archived
        // namespace — same pattern as the delta spec below.
        let dated_name = format!("{}-old-feature", chrono::Local::now().format("%Y-%m-%d"));
        fixture.seed_docs(
            "demo",
            "backend",
            vec![
                (
                    DocumentId::ArchivedChange {
                        change: dated_name.clone(),
                        doc: "tasks.md".to_string(),
                    },
                    "- [x] 1.1 Completed\n- [ ] 1.2 Pending\n".to_string(),
                ),
                (
                    DocumentId::ArchivedChange {
                        change: dated_name.clone(),
                        doc: "specs/payments/spec.md".to_string(),
                    },
                    "## ADDED Requirements\n\n### Requirement: Archived delta\n".to_string(),
                ),
            ],
        );

        Self {
            fixture,
            pat,
            dated_name,
        }
    }

    fn get(&self, project: &str, repo: &str, path: &str) -> ureq::Response {
        self.fixture
            .project_request(project, repo, &self.pat, path)
            .expect("read request succeeds")
    }
}

fn error_of(result: Result<ureq::Response, ureq::Error>) -> (u16, ErrorResponse) {
    match result {
        Ok(response) => panic!("expected a protocol error, got HTTP {}", response.status()),
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            let error = serde_json::from_str::<ErrorResponse>(&body)
                .unwrap_or_else(|_| panic!("body is an ErrorResponse envelope, got: {body}"));
            (code, error)
        }
        Err(error) => panic!("transport error: {error}"),
    }
}

#[test]
fn scopes_are_filtered_to_each_callers_memberships() {
    let fixture = Fixture::new();
    let (demo_pat, _) = fixture.seed_user("demo@example.com", &["demo"], false);
    let (multi_pat, _) = fixture.seed_user("multi@example.com", &["multi"], false);

    assert_eq!(
        fixture.scopes(&demo_pat).projects,
        vec![ProjectScope {
            id: "prj_demo".to_string(),
            key: "demo".to_string(),
            name: "Demo".to_string(),
            repos: vec![ScopeRef {
                id: "repo_backend".to_string(),
                key: "backend".to_string(),
                name: "backend".to_string(),
            }],
        }]
    );
    assert_eq!(
        fixture.scopes(&multi_pat).projects,
        vec![ProjectScope {
            id: "prj_multi".to_string(),
            key: "multi".to_string(),
            name: "Multi".to_string(),
            repos: vec![
                ScopeRef {
                    id: "repo_api".to_string(),
                    key: "api".to_string(),
                    name: "api".to_string(),
                },
                ScopeRef {
                    id: "repo_web".to_string(),
                    key: "web".to_string(),
                    name: "web".to_string(),
                },
            ],
        }]
    );
}

#[test]
fn a_user_without_memberships_gets_an_empty_successful_response() {
    let fixture = Fixture::new();
    let (pat, _) = fixture.seed_user("empty@example.com", &[], false);

    assert!(fixture.scopes(&pat).projects.is_empty());
}

#[test]
fn an_admin_without_memberships_has_no_scope_bypass() {
    let fixture = Fixture::new();
    let (pat, _) = fixture.seed_user("admin@example.com", &[], true);

    assert!(fixture.scopes(&pat).projects.is_empty());
}

#[test]
fn a_suspended_account_is_forbidden() {
    let fixture = Fixture::new();
    let (pat, user_id) = fixture.seed_user("suspended@example.com", &["demo"], false);
    fixture
        .identity
        .set_user_active(&user_id, false)
        .expect("suspend account");

    let (status, error) = error_of(fixture.request(Some(&pat)));
    assert_eq!(status, 403);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_missing_bearer_is_unauthorized() {
    let fixture = Fixture::new();

    let (status, error) = error_of(fixture.request(None));
    assert_eq!(status, 401);
    assert_eq!(error.reason, ErrorReason::PermissionDenied);
}

#[test]
fn a_canonical_spec_document_returns_store_truth_and_is_scope_isolated() {
    let read = ReadFixture::new();

    let response: DocumentResponse = serde_json::from_str(
        &read
            .get("demo", "backend", "/specs/payments/document")
            .into_string()
            .expect("spec body"),
    )
    .expect("SpecDocumentResponse");
    assert_eq!(
        response.content,
        "# payments Specification\n\nCanonical truth.\n"
    );

    for (project, repo, capability) in
        [("demo", "backend", "missing"), ("multi", "web", "payments")]
    {
        let (status, error) = error_of(read.fixture.project_request(
            project,
            repo,
            &read.pat,
            &format!("/specs/{capability}/document"),
        ));
        assert_eq!(status, 404);
        assert_eq!(error.reason, ErrorReason::NotFound);
    }
}

#[test]
fn an_archived_list_derives_truth_after_the_archive_verb() {
    let read = ReadFixture::new();

    let response: ArchivedListResponse = serde_json::from_str(
        &read
            .get("demo", "backend", "/archived")
            .into_string()
            .expect("archived body"),
    )
    .expect("ArchivedListResponse");
    assert_eq!(response.archived.len(), 1);
    let item = &response.archived[0];
    assert_eq!(item.dated_name, read.dated_name);
    assert_eq!(
        item.date,
        chrono::Local::now().format("%Y-%m-%d").to_string()
    );
    assert_eq!(item.name, "old-feature");
    assert_eq!(item.tasks_total, Some(2));
    assert_eq!(item.tasks_done, Some(1));
    assert_eq!(item.spec_count, 1);
    assert_eq!(
        item.created_by.as_deref(),
        Some("Original Creator <creator@example.com>")
    );
    assert_eq!(item.from_discussions, ["source-one", "source-two"]);

    let other_scope: ArchivedListResponse = serde_json::from_str(
        &read
            .get("multi", "web", "/archived")
            .into_string()
            .expect("other-scope archived body"),
    )
    .expect("ArchivedListResponse");
    assert!(
        other_scope.archived.is_empty(),
        "another scope cannot see the archive"
    );
}

#[test]
fn archived_artifact_and_capabilities_return_store_truth_without_cross_scope_leaks() {
    let read = ReadFixture::new();
    let artifact_path = format!("/archived/{}/artifacts/proposal.md", read.dated_name);
    let capabilities_path = format!("/archived/{}/capabilities", read.dated_name);

    let artifact: DocumentResponse = serde_json::from_str(
        &read
            .get("demo", "backend", &artifact_path)
            .into_string()
            .expect("artifact body"),
    )
    .expect("document response");
    assert_eq!(artifact.content, "## Why\n\nArchived proposal truth.\n");

    let capabilities: Vec<String> = serde_json::from_str(
        &read
            .get("demo", "backend", &capabilities_path)
            .into_string()
            .expect("capabilities body"),
    )
    .expect("capabilities array");
    assert_eq!(capabilities, ["payments"]);

    for path in [
        format!("/archived/{}/artifacts/missing.md", read.dated_name),
        artifact_path,
    ] {
        let (project, repo) = if path.ends_with("missing.md") {
            ("demo", "backend")
        } else {
            ("multi", "web")
        };
        let (status, error) = error_of(
            read.fixture
                .project_request(project, repo, &read.pat, &path),
        );
        assert_eq!(status, 404);
        assert_eq!(error.reason, ErrorReason::NotFound);
    }
}

#[test]
fn search_is_case_insensitive_first_match_per_card_and_returns_both_kinds() {
    let read = ReadFixture::new();

    let response: SearchResponse = serde_json::from_str(
        &read
            .get("demo", "backend", "/search?q=magictoken")
            .into_string()
            .expect("search body"),
    )
    .expect("SearchResponse");
    assert_eq!(response.hits.len(), 2);
    let change = response
        .hits
        .iter()
        .find(|hit| hit.kind == "change")
        .expect("change hit");
    assert_eq!(change.id, "searchable");
    assert_eq!(
        change.artifact, "proposal.md",
        "the first matching artifact wins"
    );
    assert!(
        change.snippet.contains("MagicToken"),
        "original case is preserved"
    );
    assert!(
        change.snippet.starts_with('…') && change.snippet.ends_with('…'),
        "both truncated ends carry an ellipsis: {}",
        change.snippet
    );
    assert!(change.snippet.chars().count() <= 30 + 10 + 30 + 2);

    let discussion = response
        .hits
        .iter()
        .find(|hit| hit.kind == "discussion")
        .expect("discussion hit");
    assert_eq!(discussion.id, "search-talk");
    assert_eq!(discussion.artifact, "search-talk.md");
    assert!(discussion.snippet.contains("MAGICTOKEN"));
}

#[test]
fn search_returns_empty_for_blank_queries_and_other_scopes() {
    let read = ReadFixture::new();

    let blank: SearchResponse = serde_json::from_str(
        &read
            .get("demo", "backend", "/search?q=%20%20%20")
            .into_string()
            .expect("blank search body"),
    )
    .expect("SearchResponse");
    assert!(blank.hits.is_empty());

    let other_scope: SearchResponse = serde_json::from_str(
        &read
            .get("multi", "web", "/search?q=magictoken")
            .into_string()
            .expect("other-scope search body"),
    )
    .expect("SearchResponse");
    assert!(other_scope.hits.is_empty());
}

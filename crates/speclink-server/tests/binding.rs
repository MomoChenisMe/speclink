//! Binding handshake and authentication precondition, fail closed
//! (reference-server spec「binding 與認證前置 fail closed」). Every project-scoped
//! route runs this precondition; `/binding` is where its success shape is
//! observed.

mod common;

use speclink_protocol::binding::BindingResponse;
use speclink_protocol::error::{ErrorReason, ErrorResponse};
use speclink_protocol::API_VERSION;

const TOKEN: &str = "secret";

/// `GET {project}/binding` with the given credentials and headers.
fn get_binding(
    base: &str,
    project: &str,
    token: &str,
    api_version: &str,
    repo: Option<&str>,
) -> Result<ureq::Response, ureq::Error> {
    let url = format!("{base}/api/speclink/v1/projects/{project}/binding");
    let mut req = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("X-Speclink-Api-Version", api_version);
    if let Some(repo) = repo {
        req = req.set("X-Speclink-Repo", repo);
    }
    req.call()
}

/// The `(status, envelope)` of a request expected to fail with a protocol error.
fn error_of(result: Result<ureq::Response, ureq::Error>) -> (u16, ErrorResponse) {
    match result {
        Ok(resp) => panic!("expected a protocol error, got HTTP {}", resp.status()),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            let err = serde_json::from_str::<ErrorResponse>(&body)
                .unwrap_or_else(|_| panic!("body is an ErrorResponse envelope, got: {body}"));
            (code, err)
        }
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn unknown_token_is_rejected_401_permission_denied() {
    let base = common::start(common::state_with_config(common::demo_config()));
    let (status, err) = error_of(get_binding(&base, "demo", "wrong-token", API_VERSION, Some("backend")));
    assert_eq!(status, 401);
    assert_eq!(err.reason, ErrorReason::PermissionDenied);
}

#[test]
fn unregistered_project_is_404_not_found() {
    let base = common::start(common::state_with_config(common::demo_config()));
    let (status, err) = error_of(get_binding(&base, "ghost", TOKEN, API_VERSION, Some("backend")));
    assert_eq!(status, 404);
    assert_eq!(err.reason, ErrorReason::NotFound);
}

#[test]
fn unregistered_repo_header_is_not_found() {
    let base = common::start(common::state_with_config(common::demo_config()));
    let (status, err) = error_of(get_binding(&base, "demo", TOKEN, API_VERSION, Some("ghost-repo")));
    assert_eq!(status, 404);
    assert_eq!(err.reason, ErrorReason::NotFound);
}

#[test]
fn ambiguous_repo_without_a_header_is_refused_and_names_candidates() {
    let base = common::start(common::state_with_config(common::config_with_dual_repo_project()));
    let (_status, err) = error_of(get_binding(&base, "multi", TOKEN, API_VERSION, None));
    assert_eq!(err.reason, ErrorReason::Refused);
    assert!(
        err.message.contains("web") && err.message.contains("api"),
        "the refusal names the repo candidates: {}",
        err.message
    );
}

#[test]
fn incompatible_api_version_is_refused_with_a_version_reason() {
    let base = common::start(common::state_with_config(common::demo_config()));
    let (_status, err) = error_of(get_binding(&base, "demo", TOKEN, "999", Some("backend")));
    assert_eq!(err.reason, ErrorReason::Refused);
    assert!(
        err.message.to_lowercase().contains("version"),
        "the refusal explains the version incompatibility: {}",
        err.message
    );
}

#[test]
fn a_compatible_request_returns_the_binding_with_capabilities() {
    let base = common::start(common::state_with_config(common::demo_config()));
    let resp = get_binding(&base, "demo", TOKEN, API_VERSION, Some("backend"))
        .expect("a compatible request binds");
    assert_eq!(resp.status(), 200);
    let binding: BindingResponse =
        serde_json::from_str(&resp.into_string().unwrap()).expect("a BindingResponse body");

    assert_eq!(binding.actor.id, "u_1");
    assert_eq!(binding.actor.name, "Tester <tester@example.com>");
    assert_eq!(binding.project.key, "demo");
    assert_eq!(binding.repo.key, "backend");
    assert_eq!(binding.api_version, API_VERSION);
    assert!(!binding.engine_version.is_empty(), "engine version is declared");

    let events = &binding.capabilities.events;
    assert!(events.transports.is_empty(), "no push transport is declared");
    let polling = events.polling.as_ref().expect("polling is declared");
    assert_eq!(polling.url, "/sync-state");
    assert!(polling.etag, "etag polling is declared");
}

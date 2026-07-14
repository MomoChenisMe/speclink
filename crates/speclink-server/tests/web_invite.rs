//! The invite acceptance page (server-identity spec「邀請一次性且到期失效」,
//! 決策 4). A valid token shows a set-password form; submitting it atomically
//! creates an active user with the invited memberships and consumes the
//! invitation. A used, expired or unknown token yields the same "邀請無效" page
//! (404), without distinguishing the reason.

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// An identity store seeded with one invitation, plus a running server over it.
/// Returns the server base URL, the invitation token, and the identity store
/// (so the test can inspect the outcome).
fn seeded(days: i64) -> (String, String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: "invitee@example.com".to_string(),
            display: "Invitee".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(days),
        })
        .expect("seed invitation");
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    let base = common::start(state);
    (base, token, identity)
}

/// A ureq agent that does not follow redirects, so we observe the immediate
/// response.
fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

/// The `(status, body)` of a request, whether it succeeded or carried an error
/// status.
fn status_body(result: Result<ureq::Response, ureq::Error>) -> (u16, String) {
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn walking_through_a_valid_invitation_creates_the_account() {
    let (base, token, identity) = seeded(7);

    // GET shows a set-password form.
    let (status, body) = status_body(agent().get(&format!("{base}/invite/{token}")).call());
    assert_eq!(status, 200, "a valid invitation shows the form");
    assert!(body.contains("password"), "the form has a password field: {body}");
    assert!(body.contains(&format!("/invite/{token}")), "the form posts back to the invite URL");

    // POST creates the account.
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/invite/{token}"))
            .send_form(&[("password", "hunter2password")]),
    );
    assert!((200..400).contains(&status), "accepting the invitation succeeds, got {status}");

    let user = identity
        .find_user_by_email("invitee@example.com")
        .expect("lookup")
        .expect("the account was created");
    assert!(user.active, "the created user is active");
    assert!(identity.is_member(&user.id, "demo").expect("membership"), "the invited membership is granted");

    // The invitation is consumed: re-opening the URL is the invalid page.
    let (status, body) = status_body(agent().get(&format!("{base}/invite/{token}")).call());
    assert_eq!(status, 404, "a consumed invitation is the invalid page");
    assert!(body.contains("邀請無效"), "the consumed page reads 邀請無效: {body}");
}

#[test]
fn an_expired_invitation_shows_the_invalid_page_and_creates_nothing() {
    let (base, token, identity) = seeded(-1);

    let (status, body) = status_body(agent().get(&format!("{base}/invite/{token}")).call());
    assert_eq!(status, 404, "an expired invitation is the invalid page");
    assert!(body.contains("邀請無效"), "the page reads 邀請無效: {body}");

    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/invite/{token}"))
            .send_form(&[("password", "hunter2password")]),
    );
    assert_eq!(status, 404, "posting to an expired invitation is refused with the invalid page");
    assert!(
        identity.find_user_by_email("invitee@example.com").expect("lookup").is_none(),
        "no user was created from the expired invitation"
    );
}

#[test]
fn an_unknown_token_is_the_same_invalid_page() {
    let (base, _token, _identity) = seeded(7);
    let (status, body) = status_body(agent().get(&format!("{base}/invite/does-not-exist")).call());
    assert_eq!(status, 404, "an unknown token is the invalid page");
    assert!(body.contains("邀請無效"), "unknown and used tokens share one page: {body}");
}

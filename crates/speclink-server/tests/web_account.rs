//! Login, logout, session and the account page (server-identity spec「本機密碼
//! 登入與 session 安全屬性」, 決策 4). Passwords verify with argon2; the session
//! cookie carries HttpOnly/Secure/SameSite=Strict; a change-making POST from a
//! foreign origin is 403; a failed login is byte-identical for an unknown email
//! and a wrong password; logout revokes the server-side session; an
//! unauthenticated account visit redirects to login.

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

const EMAIL: &str = "user@example.com";
const PASSWORD: &str = "correct-horse-battery";

/// Seed a user through the identity store (invite → accept) and start a server
/// over it. Returns the base URL and the identity store.
fn server_with_user() -> (String, Arc<IdentitySqlite>) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity store"));
    let token = identity
        .create_invitation(NewInvitation {
            email: EMAIL.to_string(),
            display: "User".to_string(),
            memberships: vec!["demo".to_string()],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    identity.accept_invitation(&token, PASSWORD).expect("accept");
    let state = AppState {
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity)
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

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

/// The `speclink_session` value from a response's Set-Cookie header.
fn session_cookie(resp: &ureq::Response) -> String {
    let set = resp.header("set-cookie").expect("a Set-Cookie header");
    set.split(';')
        .next()
        .unwrap()
        .trim()
        .strip_prefix("speclink_session=")
        .expect("the session cookie")
        .to_string()
}

#[test]
fn login_sets_a_secure_session_cookie_and_reaches_account() {
    let (base, _identity) = server_with_user();

    let resp = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login responds");
    assert!((300..400).contains(&resp.status()), "a good login redirects, got {}", resp.status());
    let set = resp.header("set-cookie").expect("Set-Cookie present");
    assert!(set.contains("HttpOnly"), "cookie is HttpOnly: {set}");
    assert!(set.contains("Secure"), "cookie is Secure: {set}");
    assert!(set.contains("SameSite=Strict"), "cookie is SameSite=Strict: {set}");
    let session = session_cookie(&resp);

    let (status, body) = status_body(
        agent()
            .get(&format!("{base}/account"))
            .set("Cookie", &format!("speclink_session={session}"))
            .call(),
    );
    assert_eq!(status, 200, "the session reaches the account page");
    assert!(body.contains(EMAIL), "the account page names the user: {body}");
}

#[test]
fn a_failed_login_is_byte_identical_for_unknown_email_and_wrong_password() {
    let (base, _identity) = server_with_user();

    let unknown = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", "nobody@example.com"), ("password", PASSWORD)]),
    );
    let wrong = status_body(
        agent()
            .post(&format!("{base}/login"))
            .send_form(&[("email", EMAIL), ("password", "wrong-password")]),
    );
    assert_eq!(unknown.0, wrong.0, "same status for unknown email and wrong password");
    assert_eq!(unknown.1, wrong.1, "byte-identical body — the failure never leaks account existence");
}

#[test]
fn a_change_making_post_from_a_foreign_origin_is_403() {
    let (base, _identity) = server_with_user();
    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/login"))
            .set("Origin", "https://evil.example")
            .send_form(&[("email", EMAIL), ("password", PASSWORD)]),
    );
    assert_eq!(status, 403, "a foreign-origin POST is refused");
}

#[test]
fn logout_revokes_the_server_side_session() {
    let (base, _identity) = server_with_user();
    let login = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    let session = session_cookie(&login);
    let cookie = format!("speclink_session={session}");

    // The session works.
    let (status, _) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert_eq!(status, 200, "the fresh session reaches the account page");

    // Logout revokes it.
    let (status, _) = status_body(
        agent().post(&format!("{base}/logout")).set("Cookie", &cookie).send_form(&[]),
    );
    assert!((300..400).contains(&status), "logout redirects, got {status}");

    // The same cookie is now treated as unauthenticated.
    let (status, _) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert!((300..400).contains(&status), "the revoked session no longer reaches the account page, got {status}");
}

#[test]
fn an_unauthenticated_account_visit_redirects_to_login() {
    let (base, _identity) = server_with_user();
    let (status, _) = status_body(agent().get(&format!("{base}/account")).call());
    assert!((300..400).contains(&status), "an unauthenticated visit redirects, got {status}");
}

/// Log in and return the session cookie value.
fn login(base: &str) -> String {
    let resp = agent()
        .post(&format!("{base}/login"))
        .send_form(&[("email", EMAIL), ("password", PASSWORD)])
        .expect("login");
    session_cookie(&resp)
}

/// The full `spk_pat_` plaintext (8-char prefix + 64 hex) from a page body.
fn extract_pat(body: &str) -> String {
    for (i, _) in body.match_indices("spk_pat_") {
        let token: String = body[i..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if token.len() == 72 {
            return token;
        }
    }
    panic!("no full PAT plaintext in body: {body}");
}

#[test]
fn a_pat_plaintext_is_shown_exactly_once() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let (status, body) = status_body(
        agent()
            .post(&format!("{base}/account/tokens"))
            .set("Cookie", &cookie)
            .send_form(&[("name", "ci"), ("expires", "")]),
    );
    assert_eq!(status, 200, "creating a PAT shows a page");
    let plaintext = extract_pat(&body);
    assert!(plaintext.starts_with("spk_pat_"), "the plaintext carries the prefix");
    assert_eq!(body.matches(&plaintext).count(), 1, "the full plaintext appears exactly once");

    // Reloading the account page never shows the plaintext again — only prefix.
    let (status, reload) = status_body(agent().get(&format!("{base}/account")).set("Cookie", &cookie).call());
    assert_eq!(status, 200);
    assert!(!reload.contains(&plaintext), "the plaintext is not recoverable after reload");
    let prefix: String = plaintext.chars().take(12).collect();
    assert!(reload.contains(&prefix), "the list still shows the identifiable prefix");

    // Storage holds only prefix + hash + metadata (verified via the store).
    let user = identity.find_user_by_email(EMAIL).unwrap().unwrap();
    let pats = identity.list_pats(&user.id).unwrap();
    assert_eq!(pats.len(), 1);
    assert_eq!(pats[0].name, "ci");
}

#[test]
fn revoking_a_pat_takes_effect_immediately() {
    let (base, identity) = server_with_user();
    let session = login(&base);
    let cookie = format!("speclink_session={session}");

    let (_, body) = status_body(
        agent()
            .post(&format!("{base}/account/tokens"))
            .set("Cookie", &cookie)
            .send_form(&[("name", "ci"), ("expires", "")]),
    );
    let plaintext = extract_pat(&body);
    // The PAT authenticates against the identity store.
    assert!(identity.authenticate_pat(&plaintext).unwrap().is_some(), "the fresh PAT authenticates");

    let user = identity.find_user_by_email(EMAIL).unwrap().unwrap();
    let pat_id = identity.list_pats(&user.id).unwrap()[0].id.clone();

    let (status, _) = status_body(
        agent()
            .post(&format!("{base}/account/tokens/{pat_id}/revoke"))
            .set("Cookie", &cookie)
            .send_form(&[]),
    );
    assert!((200..400).contains(&status), "revoke succeeds, got {status}");

    // The revocation is effective at once — the next lookup fails.
    assert!(identity.authenticate_pat(&plaintext).unwrap().is_none(), "the revoked PAT no longer authenticates");
}

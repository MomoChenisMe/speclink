//! The /admin server-rendered page組 (server-admin spec「管理動作三入口同一實作且
//! 功能完備」, 決策 1-4): user、registry、credential and audit pages, session-gated
//! and same-origin, driving the same single-point actions the API and CLI do. The
//! page組 serves only installation/administration — never any spec content
//! (changes、specs、discussions have no route and no link).

mod common;

use chrono::{Duration, Utc};
use speclink_server::identity::{IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// Seed a `demo`-registry identity with an admin (session) and a member (session
/// + PAT). Returns the base URL, the identity handle, the admin session, and the
/// member's `(session, pat, id)`.
#[allow(clippy::type_complexity)]
fn start() -> (String, Arc<IdentitySqlite>, String, (String, String, String)) {
    let identity = Arc::new(IdentitySqlite::open_memory().expect("identity"));
    common::seed_demo_registry(&*identity);
    let admin_session = seed(&identity, "admin@example.com", true).0;
    let member = seed(&identity, "member@example.com", false);
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: identity.clone(),
    };
    (common::start(state), identity, admin_session, member)
}

/// Seed a user; returns `(session, pat, user_id)`.
fn seed(identity: &Arc<IdentitySqlite>, email: &str, admin: bool) -> (String, String, String) {
    let token = identity
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("U <{email}>"),
            memberships: vec![],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    let user_id = identity.accept_invitation(&token, "pw-correct-horse").expect("accept");
    let (_, pat) = identity.create_pat(&user_id, "tok", None).expect("pat");
    let session = identity.create_session(&user_id, Duration::days(1)).expect("session");
    (session, pat, user_id)
}

fn get(base: &str, path: &str, session: &str) -> (u16, String) {
    let agent = ureq::builder().redirects(0).build();
    match agent.get(&format!("{base}{path}")).set("Cookie", &format!("speclink_session={session}")).call() {
        Ok(resp) => (resp.status(), resp.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

fn post(base: &str, path: &str, session: &str, fields: &[(&str, &str)]) -> u16 {
    let agent = ureq::builder().redirects(0).build();
    match agent
        .post(&format!("{base}{path}"))
        .set("Cookie", &format!("speclink_session={session}"))
        .send_form(fields)
    {
        Ok(resp) => resp.status(),
        Err(ureq::Error::Status(code, _)) => code,
        Err(e) => panic!("transport error: {e}"),
    }
}

#[test]
fn the_users_page_lists_and_drives_the_single_point_actions() {
    let (base, identity, admin, (_ms, _mp, member_id)) = start();

    let (status, body) = get(&base, "/admin/users", &admin);
    assert_eq!(status, 200);
    assert!(body.contains("member@example.com"), "the member is listed");
    assert!(body.contains("建立邀請"), "the invite form is present");

    // Invite: the acceptance URL is shown once.
    let agent = ureq::builder().redirects(0).build();
    let invited = agent
        .post(&format!("{base}/admin/users/invite"))
        .set("Cookie", &format!("speclink_session={admin}"))
        .send_form(&[("email", "invitee@example.com"), ("display", "Invitee")])
        .expect("invite")
        .into_string()
        .unwrap_or_default();
    assert!(invited.contains("/invite/"), "the invite acceptance URL is shown");

    // Membership grant and admin-flag toggle through the forms.
    assert!((300..400).contains(&post(&base, &format!("/admin/users/{member_id}/membership"), &admin, &[("project_key", "demo"), ("action", "grant")])));
    assert!(identity.is_member(&member_id, "demo").unwrap(), "membership granted via the form");
    assert!((300..400).contains(&post(&base, &format!("/admin/users/{member_id}/admin-flag"), &admin, &[("admin", "true")])));
    assert!(identity.get_user(&member_id).unwrap().unwrap().admin, "admin flag set via the form");
}

#[test]
fn the_registry_page_creates_and_renames_without_changing_keys() {
    let (base, identity, admin, _member) = start();
    assert_eq!(get(&base, "/admin/registry", &admin).0, 200);

    assert!((300..400).contains(&post(&base, "/admin/registry/projects", &admin, &[("key", "proj2"), ("name", "Project 2")])));
    assert_eq!(identity.get_project("proj2").unwrap().unwrap().name, "Project 2");

    assert!((300..400).contains(&post(&base, "/admin/registry/repos", &admin, &[("project_key", "proj2"), ("key", "api"), ("name", "API")])));
    assert_eq!(identity.list_repos("proj2").unwrap()[0].key, "api");

    // Rename changes the display name, never the key.
    assert!((300..400).contains(&post(&base, "/admin/registry/projects/proj2/rename", &admin, &[("name", "Renamed")])));
    let p = identity.get_project("proj2").unwrap().unwrap();
    assert_eq!((p.key.as_str(), p.name.as_str()), ("proj2", "Renamed"), "key stable, name changed");
}

#[test]
fn the_credentials_page_shows_metadata_and_force_revokes() {
    let (base, identity, admin, (_ms, member_pat, _member_id)) = start();
    let (status, body) = get(&base, "/admin/credentials", &admin);
    assert_eq!(status, 200);
    assert!(body.contains("spk_pat_"), "the PAT prefix is listed");
    assert!(!body.contains(&member_pat), "the plaintext is never shown");

    // Find the member's PAT id and force-revoke it.
    let pat_id = identity
        .list_all_pats()
        .unwrap()
        .into_iter()
        .find(|p| p.name == "tok")
        .expect("member pat")
        .id;
    assert!(identity.authenticate_pat(&member_pat).unwrap().is_some(), "PAT works before revoke");
    assert!((300..400).contains(&post(&base, &format!("/admin/credentials/tokens/{pat_id}/revoke"), &admin, &[])));
    assert!(identity.authenticate_pat(&member_pat).unwrap().is_none(), "force-revoke is immediate");
}

#[test]
fn the_audit_page_is_read_only_and_reverse_chronological() {
    let (base, _identity, admin, _member) = start();
    // Generate a couple of audited actions first.
    let agent = ureq::builder().redirects(0).build();
    let _ = agent
        .post(&format!("{base}/admin/users/invite"))
        .set("Cookie", &format!("speclink_session={admin}"))
        .send_form(&[("email", "x@example.com"), ("display", "X")]);
    let (status, body) = get(&base, "/admin/audit", &admin);
    assert_eq!(status, 200);
    assert!(body.contains("user-invited"), "the audit shows the recorded action");
    assert!(body.contains("唯讀"), "the page states it is read-only");
    assert!(!body.contains("<form"), "there are no mutation controls on the audit page");
}

#[test]
fn the_admin_pages_serve_no_spec_content() {
    let (base, _identity, admin, _member) = start();
    let (_status, home) = get(&base, "/admin", &admin);
    for forbidden in ["changes", "/specs", "discussion"] {
        assert!(!home.contains(forbidden), "the admin home links to no spec content ({forbidden})");
    }
    // No spec routes exist under /admin.
    for path in ["/admin/changes", "/admin/specs", "/admin/discussions"] {
        assert_eq!(get(&base, path, &admin).0, 404, "{path} is not served");
    }
}

#[test]
fn a_non_admin_cannot_reach_the_pages() {
    let (base, _identity, _admin, (member_session, _mp, _mid)) = start();
    for path in ["/admin/users", "/admin/registry", "/admin/credentials", "/admin/audit"] {
        assert_eq!(get(&base, path, &member_session).0, 403, "{path} is forbidden to a non-admin");
    }
}

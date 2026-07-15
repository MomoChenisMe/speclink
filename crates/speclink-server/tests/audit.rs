//! Management actions are single functions that write the identity mutation and
//! its audit record in one transaction (server-admin spec「管理動作三入口同一實作
//! 且功能完備」「audit log 只增不改且動作全覆蓋」, 決策 2/3). Every variant-type
//! action writes exactly one audit; a refused or failed action writes none — the
//! audit and the action share a fate. Suspending the last active admin is
//! refused with a reason.

use chrono::{Duration, Utc};
use speclink_server::audit::{AuditActor, AuditSource};
use speclink_server::identity::{DevicePoll, IdentityError, IdentitySqlite, IdentityStore, NewInvitation};
use speclink_server::identity::{Project, Repo};

/// A fresh in-memory identity store.
fn store() -> IdentitySqlite {
    IdentitySqlite::open_memory().expect("open in-memory identity store")
}

/// Seed a user with `email` and the given admin flag through the invitation flow
/// (which writes no audit), returning the new user id. Memberships start empty.
fn seed_user(s: &IdentitySqlite, email: &str, admin: bool) -> String {
    let token = s
        .create_invitation(NewInvitation {
            email: email.to_string(),
            display: format!("U <{email}>"),
            memberships: vec![],
            admin,
            expires_at: Utc::now() + Duration::days(1),
        })
        .expect("invite");
    s.accept_invitation(&token, "pw-correct-horse").expect("accept")
}

/// A web-sourced audit actor for `operator`.
fn web(operator: &str) -> AuditActor {
    AuditActor::user(operator, AuditSource::Web)
}

#[test]
fn the_last_active_admin_cannot_be_suspended() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);

    // The sole active admin cannot self-suspend: refused, still active, no audit.
    let err = s
        .admin_set_user_suspended(&web(&admin_id), &admin_id, true)
        .expect_err("suspending the last active admin must be refused");
    assert!(matches!(err, IdentityError::Refused(_)), "the refusal is a Refused error: {err:?}");
    assert!(!err.to_string().is_empty(), "the reason is stated");
    assert!(s.get_user(&admin_id).expect("get").expect("exists").active, "the admin stays active");
    assert!(s.list_audit(10, 0).expect("audit").is_empty(), "a refused action writes no audit");

    // With a second active admin in place, suspending the first is allowed and
    // records exactly one user-suspended audit under the acting admin.
    let admin2 = seed_user(&s, "admin2@example.com", true);
    s.admin_set_user_suspended(&web(&admin2), &admin_id, true).expect("now allowed");
    assert!(!s.get_user(&admin_id).expect("get").expect("exists").active, "the first admin is suspended");
    let audit = s.list_audit(10, 0).expect("audit");
    assert_eq!(audit.len(), 1, "exactly one audit for the suspension");
    assert_eq!(audit[0].action, "user-suspended");
    assert_eq!(audit[0].actor_id, admin2, "the acting admin is the operator");
    assert_eq!(audit[0].subject, admin_id, "the subject is the suspended user");
    assert_eq!(audit[0].source, "web");
}

#[test]
fn a_failed_action_writes_no_audit() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);

    // A duplicate-email invite fails inside the transaction before the audit
    // would be written — the invitation and the audit roll back together.
    seed_user(&s, "taken@example.com", false);
    let before = s.list_audit(100, 0).expect("audit").len();
    let err = s
        .admin_create_invitation(
            &web(&admin_id),
            NewInvitation {
                email: "taken@example.com".to_string(),
                display: "Dup".to_string(),
                memberships: vec![],
                admin: false,
                expires_at: Utc::now() + Duration::days(1),
            },
        )
        .expect_err("a duplicate email must be refused");
    assert!(matches!(err, IdentityError::Duplicate(_)));
    assert_eq!(s.list_audit(100, 0).expect("audit").len(), before, "the failed invite wrote no audit");

    // A management action on an unknown user id is NotFound and, again, no audit.
    let err = s
        .admin_set_user_suspended(&web(&admin_id), "usr_ghost", true)
        .expect_err("an unknown subject must be NotFound");
    assert!(matches!(err, IdentityError::NotFound(_)));
    assert_eq!(s.list_audit(100, 0).expect("audit").len(), before, "the failed suspend wrote no audit");
}

#[test]
fn each_variant_type_user_action_writes_exactly_one_audit() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);
    let target = seed_user(&s, "target@example.com", false);
    let op = web(&admin_id);

    // invite → user-invited
    s.admin_create_invitation(
        &op,
        NewInvitation {
            email: "invitee@example.com".to_string(),
            display: "Invitee".to_string(),
            memberships: vec![],
            admin: false,
            expires_at: Utc::now() + Duration::days(1),
        },
    )
    .expect("invite");

    // suspend → user-suspended, then reactivate → user-reactivated
    s.admin_set_user_suspended(&op, &target, true).expect("suspend");
    assert!(!s.get_user(&target).unwrap().unwrap().active, "suspend took effect");
    s.admin_set_user_suspended(&op, &target, false).expect("reactivate");
    assert!(s.get_user(&target).unwrap().unwrap().active, "reactivate took effect");

    // membership grant → membership-changed, then revoke → membership-changed
    s.admin_set_membership(&op, &target, "demo", true).expect("grant");
    assert!(s.is_member(&target, "demo").unwrap(), "membership granted");
    s.admin_set_membership(&op, &target, "demo", false).expect("revoke");
    assert!(!s.is_member(&target, "demo").unwrap(), "membership revoked");

    // admin flag → admin-flag-changed
    s.admin_set_admin_flag(&op, &target, true).expect("promote");
    assert!(s.get_user(&target).unwrap().unwrap().admin, "admin flag set");

    // Every action landed exactly one audit, newest first, all under source web.
    let audit = s.list_audit(100, 0).expect("audit");
    let actions: Vec<&str> = audit.iter().map(|e| e.action.as_str()).collect();
    assert_eq!(
        actions,
        vec![
            "admin-flag-changed",
            "membership-changed",
            "membership-changed",
            "user-reactivated",
            "user-suspended",
            "user-invited",
        ],
        "six actions, one audit each, reverse-chronological"
    );
    assert!(audit.iter().all(|e| e.source == "web"), "all recorded with the web source");
    assert!(audit.iter().all(|e| e.actor_id == admin_id), "all under the acting admin");
}

#[test]
fn list_users_and_memberships_read_back() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);
    let member_id = seed_user(&s, "member@example.com", false);
    s.admin_set_membership(&web(&admin_id), &member_id, "demo", true).expect("grant");
    s.admin_set_membership(&web(&admin_id), &member_id, "extra", true).expect("grant");

    let users = s.list_users().expect("list users");
    assert_eq!(users.len(), 2, "both users are listed");
    assert!(users.iter().any(|u| u.email == "admin@example.com" && u.admin));
    assert!(users.iter().any(|u| u.email == "member@example.com" && !u.admin));

    let memberships = s.list_memberships(&member_id).expect("list memberships");
    assert_eq!(memberships, vec!["demo".to_string(), "extra".to_string()], "memberships read back, ordered");
}

#[test]
fn the_registry_key_is_immutable_only_the_display_name_changes() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);
    let op = web(&admin_id);

    // Create a project and a repo; each writes exactly one create audit.
    s.admin_create_project(&op, "demo", "Demo").expect("create project");
    s.admin_create_repo(&op, "demo", "backend", "Backend").expect("create repo");
    // A duplicate key is rejected and leaves the original untouched, no audit.
    let before = s.list_audit(100, 0).expect("audit").len();
    assert!(matches!(
        s.admin_create_project(&op, "demo", "Other").unwrap_err(),
        IdentityError::Duplicate(_)
    ));
    assert_eq!(s.list_audit(100, 0).expect("audit").len(), before, "a rejected create writes no audit");

    // Renaming changes only the display name; the key stays the stable identifier.
    s.admin_rename_project(&op, "demo", "Demo 團隊").expect("rename project");
    s.admin_rename_repo(&op, "demo", "backend", "後端").expect("rename repo");
    let project = s.get_project("demo").expect("get").expect("project exists");
    assert_eq!(project, Project { key: "demo".to_string(), name: "Demo 團隊".to_string() }, "key stable, name changed");
    let repos = s.list_repos("demo").expect("repos");
    assert_eq!(repos, vec![Repo { key: "backend".to_string(), name: "後端".to_string() }], "repo key stable, name changed");

    // There is no interface to change a key: binding resolves the same original
    // key after the rename (the URL-stable identifier is untouched).
    assert!(s.get_project("demo").expect("get").is_some(), "the original project key still resolves");
    assert_eq!(s.list_repos("demo").expect("repos")[0].key, "backend", "the original repo key still resolves");

    // Renaming an unknown project/repo is NotFound and writes no audit.
    assert!(matches!(s.admin_rename_project(&op, "ghost", "x").unwrap_err(), IdentityError::NotFound(_)));
    assert!(matches!(s.admin_rename_repo(&op, "demo", "ghost", "x").unwrap_err(), IdentityError::NotFound(_)));

    // The registry audit trail: create ×2 then rename ×2, newest first, source web.
    let audit = s.list_audit(100, 0).expect("audit");
    let actions: Vec<&str> = audit.iter().map(|e| e.action.as_str()).collect();
    assert_eq!(
        actions,
        vec!["repo-renamed", "project-renamed", "repo-created", "project-created"],
        "each registry action logged exactly once"
    );
    assert_eq!(audit[3].subject, "demo", "project-created records the key");
    assert_eq!(audit[2].subject, "demo/backend", "repo-created records the scoped key");
    assert!(audit.iter().all(|e| e.source == "web" && e.actor_id == admin_id));
}

#[test]
fn pat_force_revocation_is_immediate_and_audited_without_secrets() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);
    let member_id = seed_user(&s, "member@example.com", false);
    let (pat_meta, pat_plaintext) = s.create_pat(&member_id, "member-token", None).expect("pat");

    // Before revocation the token authenticates.
    assert!(s.authenticate_pat(&pat_plaintext).expect("auth").is_some(), "the PAT works before revocation");

    // The admin force-revokes it (a PAT it does not own).
    s.admin_revoke_pat(&web(&admin_id), &pat_meta.id).expect("force revoke");

    // Immediate: the very next use no longer authenticates.
    assert!(s.authenticate_pat(&pat_plaintext).expect("auth").is_none(), "the PAT stops authenticating at once");

    // A trail: one token-revoked audit, recording the token id and prefix and
    // never a hash or the plaintext.
    let audit = s.list_audit(10, 0).expect("audit");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].action, "token-revoked");
    assert_eq!(audit[0].actor_id, admin_id, "the acting admin is the operator");
    assert!(audit[0].subject.contains(&pat_meta.id), "the audit names the token id");
    assert!(audit[0].subject.contains(&pat_meta.prefix), "the audit names the prefix");
    // The prefix (12 chars) is the identifiable, non-secret head; the full
    // plaintext and its hash never appear.
    assert!(!audit[0].subject.contains(&pat_plaintext), "the audit never carries the plaintext");
    assert!(!audit[0].subject.contains(&pat_plaintext[12..]), "not even the plaintext body past the prefix");

    // The metadata list shows the token across all users, revoked, with no secret
    // field on the type at all (Pat carries prefix/name/timestamps, never a hash).
    let all = s.list_all_pats().expect("list all pats");
    let found = all.iter().find(|p| p.id == pat_meta.id).expect("the token is listed site-wide");
    assert_eq!(found.user_id, member_id, "the owning user is recorded");
    assert_eq!(found.prefix, pat_meta.prefix);
    assert!(found.revoked_at.is_some(), "the metadata shows it revoked");
}

#[test]
fn the_credential_view_spans_all_users_and_families_force_revoke() {
    let s = store();
    let admin_id = seed_user(&s, "admin@example.com", true);
    let alice = seed_user(&s, "alice@example.com", false);
    let bob = seed_user(&s, "bob@example.com", false);
    s.create_pat(&alice, "alice-token", None).expect("pat a");
    s.create_pat(&bob, "bob-token", None).expect("pat b");

    // The credential list spans every user's tokens (全站).
    let all = s.list_all_pats().expect("list all pats");
    assert_eq!(all.len(), 2, "both users' tokens are listed");
    assert!(all.iter().any(|p| p.user_id == alice && p.name == "alice-token"));
    assert!(all.iter().any(|p| p.user_id == bob && p.name == "bob-token"));

    // Seed a device credential family for alice through the device flow.
    let auth = s
        .create_device_authorization(Duration::seconds(0), Duration::minutes(15))
        .expect("device authorization");
    assert!(s.approve_device(&auth.user_code, &alice).expect("approve"));
    let pair = match s.poll_device(&auth.device_code).expect("poll") {
        DevicePoll::Approved(pair) => pair,
        other => panic!("expected an approved token pair, got {other:?}"),
    };
    assert!(s.authenticate_access_token(&pair.access_token).expect("auth").is_some(), "the access token works");

    // The family is listed site-wide with its owning user.
    let families = s.list_all_device_families().expect("list families");
    assert_eq!(families.len(), 1);
    let (owner, family) = &families[0];
    assert_eq!(owner, &alice, "the owning user is recorded");

    // Force-revoking it is immediate and audited.
    s.admin_revoke_family(&web(&admin_id), &family.id).expect("force revoke family");
    assert!(
        s.authenticate_access_token(&pair.access_token).expect("auth").is_none(),
        "the family's access token stops authenticating at once"
    );
    let audit = s.list_audit(10, 0).expect("audit");
    assert_eq!(audit[0].action, "token-revoked");
    assert_eq!(audit[0].subject, family.id, "the audit names the family, no secret");
    assert_eq!(audit[0].actor_id, admin_id);
}

//! The server's own identity store: users, project memberships, invitations,
//! PATs and sessions (server-identity spec). The store is abstracted behind a
//! trait so the routes and the auth precondition depend on the behaviour, not
//! the backend; [`IdentitySqlite`] is the only implementation, in a file-backed
//! and an in-memory (test-only) variant.
//!
//! Every credential lands as a hash and never as plaintext (決策 2): passwords
//! use argon2id; PATs, invitation tokens and session ids are high-entropy random
//! values stored as their SHA-256. A PAT's plaintext is shown once at creation
//! and carries the identifiable `spk_pat_` prefix.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

use crate::audit::{AuditAction, AuditActor, AuditEntry};
pub use crate::identity_sqlite::IdentitySqlite;

/// A user account. The password hash never leaves the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub display: String,
    pub email: String,
    pub active: bool,
    pub admin: bool,
    /// When the account was created — the admin users list shows it as the
    /// account's age; nothing authenticates or authorizes on it.
    pub created_at: DateTime<Utc>,
}

/// The deliberately small project-membership role model. Authorization code
/// treats an unknown stored value as corruption instead of silently granting
/// editor access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipRole {
    Editor,
    Reader,
}

impl MembershipRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Reader => "reader",
        }
    }

    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        match value {
            "editor" => Ok(Self::Editor),
            "reader" => Ok(Self::Reader),
            other => Err(IdentityError::Refused(format!(
                "unknown membership role '{other}'"
            ))),
        }
    }
}

impl Default for MembershipRole {
    fn default() -> Self {
        Self::Editor
    }
}

/// A registered project in the registry: its URL key and display name. The
/// repos it holds are listed separately (决策 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub key: String,
    pub name: String,
}

/// A repo registered within a project: its key (unique in the project) and
/// display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pub key: String,
    pub name: String,
}

/// A filter over the management audit log (the browser audit view's query
/// string). Every field is optional; `None` means "do not narrow on this".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFilter {
    /// Case-insensitive substring, matched against action, subject and actor id.
    pub keyword: Option<String>,
    /// An exact action name. An unknown name simply matches nothing.
    pub action: Option<String>,
    /// An exact source name (`web`, `api`, `cli`).
    pub source: Option<String>,
    /// Inclusive lower bound on the record time.
    pub from: Option<DateTime<Utc>>,
    /// Inclusive upper bound on the record time.
    pub to: Option<DateTime<Utc>>,
    /// 1-based page number. The caller validates it before building the filter.
    pub page: u32,
    /// Records per page.
    pub per_page: u32,
}

/// One page of a filtered audit query: the page's records (newest first) and how
/// many records the filter matches in total, so the caller can derive page count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditPage {
    pub entries: Vec<AuditEntry>,
    pub total: u64,
}

/// A pending invitation, resolved from its one-time token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    pub id: String,
    pub email: String,
    pub display: String,
    pub admin: bool,
    pub memberships: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// The parameters the invite subcommand supplies to mint an invitation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewInvitation {
    pub email: String,
    pub display: String,
    pub memberships: Vec<String>,
    pub admin: bool,
    pub expires_at: DateTime<Utc>,
}

/// A backup or verify result to record in the identity store's backup log, for
/// the admin backup-info view (决策 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewBackupRecord {
    /// `backup` or `verify`.
    pub kind: String,
    /// The backup's own creation time (from its manifest/summary).
    pub created_at: DateTime<Utc>,
    pub format_version: u32,
    pub scope_count: usize,
    /// Whether the run succeeded (a verify's integrity result; a backup is true).
    pub ok: bool,
    /// A short human summary shown on the admin page.
    pub detail: String,
}

/// A recorded backup/verify result, read back for the admin backup-info view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRecord {
    pub id: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
    pub format_version: u32,
    pub scope_count: usize,
    pub ok: bool,
    pub detail: String,
    /// When this record was written.
    pub recorded_at: DateTime<Utc>,
}

/// A PAT's stored metadata — never the plaintext, which exists only at creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat {
    pub id: String,
    pub user_id: String,
    pub prefix: String,
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A session's metadata for the account page. The session id is never stored in
/// plaintext, so it cannot be shown back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// The plaintext codes and metadata from initiating a device authorization. The
/// two codes are returned only here; the store keeps only their hashes (決策 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceAuthorization {
    /// High-entropy code the initiating client polls and exchanges for tokens.
    pub device_code: String,
    /// Short, human-enterable code shown on the approval page.
    pub user_code: String,
    pub expires_at: DateTime<Utc>,
    /// Minimum interval a client must wait between polls.
    pub interval: Duration,
}

/// A device credential family's metadata for the account page: when it was
/// approved, when it was last refreshed, its approval source, and whether it is
/// revoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceFamily {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_refresh_at: DateTime<Utc>,
    pub source: String,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// A freshly issued access + refresh pair (plaintext, shown once), from an
/// approval or a refresh rotation. The store keeps only the hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    /// When the access token expires (short-lived; the refresh rotates).
    pub access_expires_at: DateTime<Utc>,
}

/// The outcome of polling a device authorization by its device code. The
/// intermediate and terminal states are typed values, not wire errors (決策 1);
/// only an unknown device code is an error to the caller.
#[derive(Debug)]
pub enum DevicePoll {
    /// No authorization holds this device code.
    NotFound,
    /// Not yet approved or denied.
    Pending,
    /// Polled sooner than the declared interval; the request is undisturbed.
    SlowDown,
    /// The authorization expired before it was approved.
    Expired,
    /// A user denied the request on the approval page.
    Denied,
    /// Approved: the token pair, minted and returned exactly once.
    Approved(TokenPair),
}

/// The outcome of a refresh rotation.
#[derive(Debug)]
pub enum RefreshOutcome {
    /// No refresh credential holds this value.
    NotFound,
    /// The value was already rotated away, or itself/its family revoked — a
    /// reuse signal. The whole family has now been torn down; the request fails.
    Reused,
    /// Rotated: the old value is dead and a fresh pair is issued.
    Rotated(TokenPair),
}

/// Why an identity operation failed.
#[derive(Debug)]
pub enum IdentityError {
    /// The identity database could not be opened, is not a speclink identity
    /// store, or records a version this server cannot read (fail closed).
    Open(String),
    /// A uniqueness guard rejected the write (e.g. the email already has an
    /// active user or an outstanding invitation).
    Duplicate(String),
    /// The invitation is used, expired or unknown at accept time.
    InvalidInvitation,
    /// A management action was refused by a guard (e.g. suspending the last
    /// active admin). The reason is carried for display.
    Refused(String),
    /// The subject of a management action does not exist (e.g. an unknown user
    /// id). The reason is carried for display.
    NotFound(String),
    /// A backend failure.
    Backend(String),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityError::Open(r) => write!(f, "cannot open identity store: {r}"),
            IdentityError::Duplicate(r) => write!(f, "{r}"),
            IdentityError::InvalidInvitation => write!(f, "the invitation is invalid"),
            IdentityError::Refused(r) => write!(f, "{r}"),
            IdentityError::NotFound(r) => write!(f, "{r}"),
            IdentityError::Backend(r) => write!(f, "identity store backend error: {r}"),
        }
    }
}

impl std::error::Error for IdentityError {}

/// The identity store contract. Object-safe: the routes and the auth layer hold
/// `Arc<dyn IdentityStore>`.
pub trait IdentityStore: Send + Sync {
    /// Mint an invitation and return its one-time plaintext token. Rejects an
    /// email that already has an active user or an outstanding invitation.
    fn create_invitation(&self, req: NewInvitation) -> Result<String, IdentityError>;

    /// Every still-outstanding invitation — unconsumed and unexpired, newest
    /// first. The users view lists these alongside real users: a person who has
    /// been invited but has not accepted yet has no user row, and would
    /// otherwise be invisible to the admin who just invited them.
    fn list_pending_invitations(&self) -> Result<Vec<Invitation>, IdentityError>;

    /// How many invitations are still outstanding — unconsumed and unexpired. The
    /// overview reports it as the "waiting to activate" count; an expired
    /// invitation is not actionable, so it does not count.
    fn count_pending_invitations(&self) -> Result<u64, IdentityError>;

    /// Resolve a token to a still-valid invitation (unconsumed and unexpired);
    /// `None` covers used, expired and unknown tokens alike.
    fn find_valid_invitation(&self, token: &str) -> Result<Option<Invitation>, IdentityError>;

    /// Accept an invitation atomically: create an active user with the invited
    /// memberships and admin flag, and consume the invitation. Returns the new
    /// user id, or [`IdentityError::InvalidInvitation`] if the token is no
    /// longer valid.
    fn accept_invitation(&self, token: &str, password: &str) -> Result<String, IdentityError>;

    /// Fetch a user by id.
    fn get_user(&self, user_id: &str) -> Result<Option<User>, IdentityError>;

    /// Fetch a user by email.
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, IdentityError>;

    /// Verify an email/password pair. Returns the user on success, and `None`
    /// uniformly for an unknown email, a wrong password, or a suspended user —
    /// the caller cannot tell which, so login failure never leaks account
    /// existence.
    fn authenticate_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<User>, IdentityError>;

    /// Set a user's active state (admin/test support).
    fn set_user_active(&self, user_id: &str, active: bool) -> Result<(), IdentityError>;

    /// Whether `user_id` is a member of `project_key`.
    fn is_member(&self, user_id: &str, project_key: &str) -> Result<bool, IdentityError>;

    /// The user's role in `project_key`, or `None` when there is no membership.
    fn membership_role(
        &self,
        user_id: &str,
        project_key: &str,
    ) -> Result<Option<MembershipRole>, IdentityError>;

    // --- Project/Repo registry (决策 1) ---

    /// Every registered project, ordered by key.
    fn list_projects(&self) -> Result<Vec<Project>, IdentityError>;

    /// Resolve a project by its key; `None` if unregistered.
    fn get_project(&self, key: &str) -> Result<Option<Project>, IdentityError>;

    /// A project's repos, ordered by key. An unregistered project yields an
    /// empty list.
    fn list_repos(&self, project_key: &str) -> Result<Vec<Repo>, IdentityError>;

    /// Register a project. Rejects a key that already exists
    /// ([`IdentityError::Duplicate`]).
    fn create_project(&self, key: &str, name: &str) -> Result<(), IdentityError>;

    /// Register a repo within a project. Rejects a repo key that already exists
    /// in that project ([`IdentityError::Duplicate`]).
    fn create_repo(&self, project_key: &str, key: &str, name: &str) -> Result<(), IdentityError>;

    // --- first-run bootstrap setup token (決策 3) ---

    /// Whether any admin user exists. Setup is open only while this is false.
    fn has_admin(&self) -> Result<bool, IdentityError>;

    /// Whether an unconsumed, unexpired setup token is outstanding — so a
    /// restart does not mint a redundant one while the operator still holds a
    /// live token.
    fn has_valid_setup_token(&self) -> Result<bool, IdentityError>;

    /// Mint a bootstrap setup token: invalidate any prior token (作廢), store the
    /// new hash with `ttl`, and return the one-time plaintext (shown once on
    /// stdout, never persisted).
    fn create_setup_token(&self, ttl: Duration) -> Result<String, IdentityError>;

    /// Whether `token` is a valid (known, unconsumed, unexpired) setup token.
    /// Unknown, expired and consumed tokens are all `false` — the gate never
    /// distinguishes the reason.
    fn is_valid_setup_token(&self, token: &str) -> Result<bool, IdentityError>;

    /// Consume the setup token by its plaintext (setup completion). Idempotent;
    /// an unknown or already-consumed token is a no-op.
    fn consume_setup_token(&self, token: &str) -> Result<(), IdentityError>;

    /// Create the first admin directly (决策 4): an active user with the admin
    /// flag, no invitation and no memberships. Rejects an email that already has
    /// a user ([`IdentityError::Duplicate`]). Returns the new user id.
    fn create_admin_user(
        &self,
        email: &str,
        display: &str,
        password: &str,
    ) -> Result<String, IdentityError>;

    /// The identity database's current schema version, for the setup store-status
    /// panel.
    fn schema_version(&self) -> Result<u32, IdentityError>;

    /// Open a session for `user_id`; returns the plaintext session id (the
    /// cookie value).
    fn create_session(&self, user_id: &str, ttl: Duration) -> Result<String, IdentityError>;

    /// Resolve a session id to its user if the session is live (unrevoked,
    /// unexpired, active user).
    fn authenticate_session(&self, session_id: &str) -> Result<Option<User>, IdentityError>;

    /// Revoke a session by its plaintext id (logout). Idempotent.
    fn revoke_session(&self, session_id: &str) -> Result<(), IdentityError>;

    /// List a user's sessions for the account page.
    fn list_sessions(&self, user_id: &str) -> Result<Vec<SessionInfo>, IdentityError>;

    /// Create a PAT for `user_id`; returns its metadata and the one-time
    /// plaintext (shown once at creation).
    fn create_pat(
        &self,
        user_id: &str,
        name: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(Pat, String), IdentityError>;

    /// List a user's PATs (metadata only).
    fn list_pats(&self, user_id: &str) -> Result<Vec<Pat>, IdentityError>;

    /// Revoke one of `user_id`'s own PATs. Idempotent.
    fn revoke_pat(&self, user_id: &str, pat_id: &str) -> Result<(), IdentityError>;

    /// Authenticate a bearer PAT for API access: hash-match, unrevoked,
    /// unexpired, owning user active. Membership is the caller's check;
    /// last-used is advanced separately by [`IdentityStore::touch_pat`].
    fn authenticate_pat(&self, token: &str) -> Result<Option<(Pat, User)>, IdentityError>;

    /// Validate a bearer PAT's credential state while retaining the owning
    /// user's active flag. This is only for identity-scoped gates that must
    /// distinguish a valid suspended account (403) from an invalid credential
    /// (401); project binding continues to use [`IdentityStore::authenticate_pat`].
    fn authenticate_pat_allow_suspended(
        &self,
        token: &str,
    ) -> Result<Option<(Pat, User)>, IdentityError>;

    /// Advance a PAT's last-used timestamp after a successful request.
    fn touch_pat(&self, pat_id: &str) -> Result<(), IdentityError>;

    // --- device authorization flow ---

    /// Initiate a device authorization: mint a high-entropy device code and a
    /// short human-enterable user code — both hashed at rest — sharing `ttl`
    /// and the minimum poll `interval`. Returns the plaintext codes.
    fn create_device_authorization(
        &self,
        interval: Duration,
        ttl: Duration,
    ) -> Result<DeviceAuthorization, IdentityError>;

    /// Poll a device authorization by its device code. A poll sooner than the
    /// declared interval since the last accepted poll is [`DevicePoll::SlowDown`]
    /// and leaves the request untouched; otherwise it reports the current state.
    fn poll_device(&self, device_code: &str) -> Result<DevicePoll, IdentityError>;

    /// Deny the still-pending authorization named by `user_code`, recording the
    /// acting user. Returns whether it applied — unknown, used and expired codes
    /// are all `false` (the approval page shows one invalid response for all).
    fn deny_device(&self, user_code: &str, approver_id: &str) -> Result<bool, IdentityError>;

    /// Whether a user code names a still-pending, unexpired authorization — the
    /// approval page's guard before it shows the confirm step. Unknown, used and
    /// expired codes are all `false`.
    fn device_is_pending(&self, user_code: &str) -> Result<bool, IdentityError>;

    /// Approve the still-pending authorization named by `user_code`, binding the
    /// approver. Returns whether it applied (same `false` as [`IdentityStore::deny_device`]
    /// for unknown/used/expired).
    fn approve_device(&self, user_code: &str, approver_id: &str) -> Result<bool, IdentityError>;

    /// Authenticate a bearer device access token for API access: hash-match,
    /// unrevoked, unexpired, its credential family unrevoked, owning user active.
    /// Per-request with no cache, so suspension and revocation take effect at
    /// once. The membership check is the caller's, as for PATs.
    fn authenticate_access_token(&self, token: &str) -> Result<Option<User>, IdentityError>;

    /// Validate a device access token and retain the owning user's active flag,
    /// for the same identity-scoped 401/403 distinction as
    /// [`IdentityStore::authenticate_pat_allow_suspended`].
    fn authenticate_access_token_allow_suspended(
        &self,
        token: &str,
    ) -> Result<Option<User>, IdentityError>;

    /// Rotate a refresh credential: a valid, unused one is spent and a fresh
    /// access token + refresh credential minted in the same family. Reusing an
    /// already-spent or revoked value (a leak signal) tears down the whole
    /// family — its access tokens and refresh credentials — and fails.
    fn refresh(&self, refresh_token: &str) -> Result<RefreshOutcome, IdentityError>;

    /// Revoke the credential family a refresh credential belongs to (logout).
    /// Returns whether the refresh credential was recognized.
    fn revoke_family_by_refresh(&self, refresh_token: &str) -> Result<bool, IdentityError>;

    /// List a user's device credential families for the account page (决策 5).
    fn list_device_families(&self, user_id: &str) -> Result<Vec<DeviceFamily>, IdentityError>;

    /// Revoke one of `user_id`'s own device credential families — the access
    /// tokens and refresh credentials under it die at once. Idempotent; a family
    /// that is not the user's own is a no-op.
    fn revoke_family(&self, user_id: &str, family_id: &str) -> Result<(), IdentityError>;

    // --- audit log (決策 3) ---

    /// Append one audit record: the operator and source from `actor`, the
    /// closed-set `action`, and the `subject` it acted on. Append-only — there
    /// is no update or delete. State-changing management actions instead write
    /// their audit in the same transaction as the mutation (see the `admin_*`
    /// methods); this standalone write records actions with no identity mutation
    /// of their own (setup completion) and backs the audit lifecycle tests.
    fn record_audit(
        &self,
        actor: &AuditActor,
        action: AuditAction,
        subject: &str,
    ) -> Result<(), IdentityError>;

    /// A page of audit records, newest first (reverse chronological). `limit`
    /// caps the page and `offset` skips that many newest records — the /admin
    /// audit view's read-only pagination.
    fn list_audit(&self, limit: u32, offset: u32) -> Result<Vec<AuditEntry>, IdentityError>;

    /// One filtered, newest-first page of audit records plus the total number of
    /// records the filter matches. The filtering and paging happen in the store,
    /// not in the caller — the log grows monotonically with operation, so a
    /// caller that read it whole and narrowed in memory would degrade linearly.
    fn query_audit(&self, filter: &AuditFilter) -> Result<AuditPage, IdentityError>;

    // --- backup records (决策 5): mutation + audit in one transaction ---

    /// Record a backup/verify result summary and a `backup-recorded` audit in the
    /// same transaction, under `actor` — the source of the admin backup-info view.
    fn record_backup(
        &self,
        actor: &AuditActor,
        record: NewBackupRecord,
    ) -> Result<(), IdentityError>;

    /// The most recently recorded backup/verify result, or `None` if none has
    /// been recorded.
    fn latest_backup(&self) -> Result<Option<BackupRecord>, IdentityError>;

    // --- admin user management (決策 2): mutation + audit in one transaction ---

    /// Every user, oldest first — the /admin user list.
    fn list_users(&self) -> Result<Vec<User>, IdentityError>;

    /// A user's project memberships (the project keys), for the /admin user view.
    fn list_memberships(&self, user_id: &str) -> Result<Vec<String>, IdentityError>;

    /// Cancel a still-pending invitation: the invitation is removed and its
    /// token stops working immediately. Returns [`IdentityError::NotFound`] when
    /// the id is unknown or the invitation was already accepted — an accepted
    /// invitation has a real account behind it, which is suspended, not cancelled.
    fn admin_revoke_invitation(
        &self,
        actor: &AuditActor,
        invitation_id: &str,
    ) -> Result<(), IdentityError>;

    /// Mint an invitation (as [`IdentityStore::create_invitation`]) and record a
    /// `user-invited` audit in the same transaction, under `actor`. The invite
    /// subcommand (source cli) and the /admin form (source web/api) share this
    /// single path. Returns the one-time plaintext token.
    fn admin_create_invitation(
        &self,
        actor: &AuditActor,
        req: NewInvitation,
    ) -> Result<String, IdentityError>;

    /// Suspend (`suspended = true`) or reactivate a user, recording a
    /// `user-suspended` / `user-reactivated` audit atomically. Suspending the
    /// last active admin is [`IdentityError::Refused`] with the reason — the
    /// admin stays active and no audit is written. An unknown user id is
    /// [`IdentityError::NotFound`].
    fn admin_set_user_suspended(
        &self,
        actor: &AuditActor,
        user_id: &str,
        suspended: bool,
    ) -> Result<(), IdentityError>;

    /// Grant/update (`member = true`) or revoke a user's membership of a
    /// project, recording the requested `role` in a `membership-changed` audit
    /// atomically. Idempotent on the membership itself; an unknown user id is
    /// [`IdentityError::NotFound`].
    fn admin_set_membership(
        &self,
        actor: &AuditActor,
        user_id: &str,
        project_key: &str,
        role: MembershipRole,
        member: bool,
    ) -> Result<(), IdentityError>;

    /// Set or clear a user's admin flag, recording an `admin-flag-changed` audit
    /// atomically. An unknown user id is [`IdentityError::NotFound`].
    fn admin_set_admin_flag(
        &self,
        actor: &AuditActor,
        user_id: &str,
        admin: bool,
    ) -> Result<(), IdentityError>;

    // --- admin registry management (決策 2): mutation + audit in one transaction ---

    /// Register a project (as [`IdentityStore::create_project`]) and record a
    /// `project-created` audit atomically. Rejects a duplicate key
    /// ([`IdentityError::Duplicate`]).
    fn admin_create_project(
        &self,
        actor: &AuditActor,
        key: &str,
        name: &str,
    ) -> Result<(), IdentityError>;

    /// Register a repo in a project (as [`IdentityStore::create_repo`]) and
    /// record a `repo-created` audit atomically. Rejects a duplicate key in that
    /// project ([`IdentityError::Duplicate`]).
    fn admin_create_repo(
        &self,
        actor: &AuditActor,
        project_key: &str,
        key: &str,
        name: &str,
    ) -> Result<(), IdentityError>;

    /// Change a project's display name, recording a `project-renamed` audit
    /// atomically. The key is the stable identifier and has no change interface;
    /// an unknown key is [`IdentityError::NotFound`].
    fn admin_rename_project(
        &self,
        actor: &AuditActor,
        key: &str,
        name: &str,
    ) -> Result<(), IdentityError>;

    /// Change a repo's display name, recording a `repo-renamed` audit atomically.
    /// The key is stable and has no change interface; an unknown repo is
    /// [`IdentityError::NotFound`].
    fn admin_rename_repo(
        &self,
        actor: &AuditActor,
        project_key: &str,
        key: &str,
        name: &str,
    ) -> Result<(), IdentityError>;

    // --- admin credential oversight (決策 4): metadata only, force revoke ---

    /// Every PAT across all users, metadata only ([`Pat`] carries no hash or
    /// plaintext), newest first — the /admin credential view. There is no
    /// interface that reads back a secret value.
    fn list_all_pats(&self) -> Result<Vec<Pat>, IdentityError>;

    /// Every device credential family across all users, paired with its owning
    /// user id, newest first. Metadata only — no secret is ever returned.
    fn list_all_device_families(&self) -> Result<Vec<(String, DeviceFamily)>, IdentityError>;

    /// Force-revoke any user's PAT by id — the same immediate revocation as
    /// self-service ([`IdentityStore::revoke_pat`]) but not scoped to an owner —
    /// recording a `token-revoked` audit (the token id and prefix, never a hash
    /// or plaintext) in the same transaction. An unknown id is
    /// [`IdentityError::NotFound`]; an already-revoked token is idempotent.
    fn admin_revoke_pat(&self, actor: &AuditActor, pat_id: &str) -> Result<(), IdentityError>;

    /// Force-revoke any device credential family by id — tearing down its access
    /// tokens and refresh credentials at once, as self-service revocation does —
    /// recording a `token-revoked` audit (the family id, no secret) in the same
    /// transaction. An unknown id is [`IdentityError::NotFound`].
    fn admin_revoke_family(&self, actor: &AuditActor, family_id: &str)
        -> Result<(), IdentityError>;
}

// --- credential hashing (決策 2) ---

/// SHA-256 of a high-entropy token, hex-encoded. High-entropy secrets need no
/// slow hash; a table lookup on the digest is enough.
pub(crate) fn hash_token(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    to_hex(&digest)
}

/// A fresh high-entropy random token: 32 bytes of OS randomness, hex-encoded.
pub(crate) fn random_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    to_hex(&bytes)
}

/// A short, human-enterable user code from an alphabet that avoids visually
/// confusable characters (no O/0, I/1, L). Eight characters grouped `XXXX-XXXX`
/// for reading (决策 2). Entropy (31^8 ≈ 8.5e11) plus the short TTL is the abuse
/// guard this knife makes; global rate limiting is a deployment-layer concern.
pub(crate) fn user_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    let c: Vec<char> = bytes
        .iter()
        .map(|b| ALPHABET[*b as usize % ALPHABET.len()] as char)
        .collect();
    format!(
        "{}{}{}{}-{}{}{}{}",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
    )
}

/// Hash a password with argon2id (a PHC string carrying its own salt).
pub(crate) fn hash_password(password: &str) -> Result<String, IdentityError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| IdentityError::Backend(format!("password hashing failed: {e}")))
}

/// Verify a password against a stored argon2 PHC hash.
pub(crate) fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Lower-hex encode bytes.
fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

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

pub use crate::identity_sqlite::IdentitySqlite;

/// A user account. The password hash never leaves the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub display: String,
    pub email: String,
    pub active: bool,
    pub admin: bool,
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

/// A pending invitation, resolved from its one-time token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    pub id: String,
    pub email: String,
    pub display: String,
    pub admin: bool,
    pub memberships: Vec<String>,
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
    /// A backend failure.
    Backend(String),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityError::Open(r) => write!(f, "cannot open identity store: {r}"),
            IdentityError::Duplicate(r) => write!(f, "{r}"),
            IdentityError::InvalidInvitation => write!(f, "the invitation is invalid"),
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
    fn authenticate_password(&self, email: &str, password: &str) -> Result<Option<User>, IdentityError>;

    /// Set a user's active state (admin/test support).
    fn set_user_active(&self, user_id: &str, active: bool) -> Result<(), IdentityError>;

    /// Whether `user_id` is a member of `project_key`.
    fn is_member(&self, user_id: &str, project_key: &str) -> Result<bool, IdentityError>;

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

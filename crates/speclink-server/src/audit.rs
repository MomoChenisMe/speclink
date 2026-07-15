//! The management audit log (決策 3): an append-only record of every
//! state-changing admin action, written in the same identity-database
//! transaction as the action it describes so the two share a fate — there is no
//! "action without audit" or "audit without action". Each entry is a five-tuple:
//! the operator, a closed-set action kind, the subject, a UTC timestamp and the
//! entry source (web、api、cli). There is no update or delete interface; the
//! /admin audit view is read-only and reverse-chronological.

use chrono::{DateTime, Utc};

/// Which entry point drove a management action. Recorded so an operator can tell
/// a browser action (web) from a bearer-token API call (api) from a headless CLI
/// subcommand (cli) apart in the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditSource {
    /// A /admin server-rendered form (session cookie).
    Web,
    /// The admin JSON API (bearer token).
    Api,
    /// A server CLI subcommand (headless, host trust).
    Cli,
}

impl AuditSource {
    /// The stable string persisted in the audit row.
    pub fn as_str(self) -> &'static str {
        match self {
            AuditSource::Web => "web",
            AuditSource::Api => "api",
            AuditSource::Cli => "cli",
        }
    }
}

/// The closed set of auditable action kinds (決策 3). Every state-changing
/// management action maps to exactly one; adding a new management action means
/// adding a variant here, and the "one audit per variant-type action" unit
/// tests keep a missed record red.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    UserInvited,
    UserSuspended,
    UserReactivated,
    MembershipChanged,
    AdminFlagChanged,
    ProjectCreated,
    ProjectRenamed,
    RepoCreated,
    RepoRenamed,
    TokenRevoked,
    SetupCompleted,
    /// An admin downloaded a scope's export bundle (决策 5).
    ScopeExported,
    /// An admin triggered a store migration that succeeded (决策 5).
    StoreMigrated,
    /// A backup or verify run recorded its result summary (决策 5).
    BackupRecorded,
}

impl AuditAction {
    /// The stable string persisted in the audit row.
    pub fn as_str(self) -> &'static str {
        match self {
            AuditAction::UserInvited => "user-invited",
            AuditAction::UserSuspended => "user-suspended",
            AuditAction::UserReactivated => "user-reactivated",
            AuditAction::MembershipChanged => "membership-changed",
            AuditAction::AdminFlagChanged => "admin-flag-changed",
            AuditAction::ProjectCreated => "project-created",
            AuditAction::ProjectRenamed => "project-renamed",
            AuditAction::RepoCreated => "repo-created",
            AuditAction::RepoRenamed => "repo-renamed",
            AuditAction::TokenRevoked => "token-revoked",
            AuditAction::SetupCompleted => "setup-completed",
            AuditAction::ScopeExported => "scope-exported",
            AuditAction::StoreMigrated => "store-migrated",
            AuditAction::BackupRecorded => "backup-recorded",
        }
    }
}

/// Who performed a management action and through which entry point. The operator
/// is a user id for a web/api action, and the sentinel [`AuditActor::SYSTEM`] for
/// a CLI subcommand, whose trust model is host file access rather than a login.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditActor {
    pub id: String,
    pub source: AuditSource,
}

impl AuditActor {
    /// The operator id recorded for a headless CLI action — the host itself, not
    /// an authenticated user.
    pub const SYSTEM: &'static str = "system";

    /// An operator identified by a user id, acting through `source`.
    pub fn user(id: impl Into<String>, source: AuditSource) -> Self {
        AuditActor { id: id.into(), source }
    }

    /// The host acting through a CLI subcommand (operator recorded as `system`).
    pub fn system_cli() -> Self {
        AuditActor { id: AuditActor::SYSTEM.to_string(), source: AuditSource::Cli }
    }
}

/// One audit record as read back for the /admin audit view. The action and
/// source are the raw closed-set strings; a subject identifies what the action
/// acted on (a user id, a project/repo key, or a token id and prefix — never a
/// secret value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub subject: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

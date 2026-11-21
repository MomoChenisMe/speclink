//! speclink-remote: the typed protocol client for the Speclink Client
//! Protocol (see `openspec/specs/client-protocol/spec.md`), used by the
//! CLI's remote mode.
//!
//! These concerns live here and nowhere else:
//! - `client`: a single request layer plus the per-verb path mapping, all
//!   speclink-protocol DTOs — no raw JSON travels through this crate.
//! - `credentials`: the OS keyring store, keyed by origin and kind.
//! - `auth`: credentials-file read/write and the resolution ladder.
//! - `refresh`: bearer acquisition and rotation, serialized across processes.
//! - `discovery`: asking a server which login methods it supports, and
//!   choosing one.
//! - `login`: the one login, logout and identity flow both front ends call.
//! - the registry mapping below: every non-2xx response becomes a
//!   single-line semantic message — a bare HTTP status code is never the
//!   primary error output.
//!
//! The last four are shared with the desktop app, not CLI-only: both front
//! ends read and write the same keyring entries, so one login on a machine
//! covers both (cli-desktop-credential-sharing).
//!
//! `speclink-core` must never depend on this crate (the core keeps its
//! "no network calls" red line).

pub mod auth;
pub mod client;
pub mod convert;
pub mod credentials;
pub mod device;
pub mod discovery;
pub mod events;
mod http;
pub mod login;
mod loopback;
mod oauth;
pub mod refresh;
pub mod skills;

/// A translated remote failure: one semantic line for the user/agent, plus
/// the machine-readable `reason` when the server provided one.
#[derive(Debug)]
pub struct RemoteError {
    pub message: String,
    pub reason: Option<String>,
    /// The HTTP status of the protocol response this failure translates, when
    /// there was one (transport and local failures carry `None`). `reason`
    /// alone cannot split 401 from 403 — both are `permission_denied` — and a
    /// client's refresh-and-retry decision hinges on exactly that split.
    pub status: Option<u16>,
    /// Structured refusal evidence when the error envelope carried it (the
    /// in-progress removal's work-trace gate) — consumed by callers that
    /// render evidence structurally (the desktop gate dialog) instead of
    /// re-parsing the message text.
    pub evidence: Option<speclink_protocol::command::RevertBlockedEvidence>,
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RemoteError {}

/// The client's frame around a server's reason for a 403: the sentence is
/// [`DENIED_BEFORE`], the reason, [`DENIED_AFTER`]. [`server_reason`] takes it
/// apart with the same two, so the wording has one place.
const DENIED_BEFORE: &str = "access denied — ";
const DENIED_AFTER: &str = "; ask a project admin";

/// The start of every sentence for a 500 or above. A server's reason follows
/// it; with no reason, [`CHECK_URL`] does.
const UNAVAILABLE_BEFORE: &str = "server unavailable — ";
const CHECK_URL: &str = "check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)";

/// The sentence for a server that answered and cannot serve, when it gave no
/// reason to show.
fn unavailable_without_a_reason() -> String {
    format!("{UNAVAILABLE_BEFORE}{CHECK_URL}")
}

/// The reason the server gave, when `error` is one of the two sentences that
/// carry one: a 403's `access denied — <reason>; ask a project admin`, and a
/// 500 or above's `server unavailable — <reason>`. Every other error — the
/// fixed sentences, a transport failure, an empty reason — gives none.
pub fn server_reason(error: &RemoteError) -> Option<&str> {
    let reason = match error.status? {
        403 => error.message.strip_prefix(DENIED_BEFORE)?.strip_suffix(DENIED_AFTER)?,
        500.. => error.message.strip_prefix(UNAVAILABLE_BEFORE).filter(|rest| *rest != CHECK_URL)?,
        _ => return None,
    };
    (!reason.is_empty()).then_some(reason)
}

/// Translate a transport-level failure (refused/timeout/DNS) — loud, no
/// cache fallback, no retry loop. This crate's request paths go through
/// `http::translate_transport_error`, which also names a rejected certificate.
pub fn translate_transport() -> RemoteError {
    RemoteError {
        message: "server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".into(),
        reason: None,
        status: None,
        evidence: None,
    }
}

/// Translate a non-2xx protocol response — the registry mapping table
/// (design decision three). The client owns connection-layer wording only;
/// engine-class refusals (`not_found`, `invalid_argument`, `invalid_config`,
/// `refused`) relay the server's message verbatim, mirroring how fs mode
/// prints engine messages. An unknown reason or an unparseable envelope is a
/// generic error — never a panic, never a bare status as the primary line.
pub fn translate_protocol_error(status: u16, body: &str) -> RemoteError {
    use speclink_protocol::error::{ErrorReason, ErrorResponse};

    // A server-side failure is translated on its status first. Its envelope
    // gives one thing only: the reason of a server that says it is
    // unavailable. The `reason` field stays empty, as it always was here.
    if status >= 500 {
        let said = serde_json::from_str::<ErrorResponse>(body)
            .ok()
            .filter(|err| err.reason == ErrorReason::Unavailable && !err.message.is_empty());
        let message = match said {
            Some(err) => format!("{UNAVAILABLE_BEFORE}{}", err.message),
            None => unavailable_without_a_reason(),
        };
        return RemoteError { message, reason: None, status: Some(status), evidence: None };
    }

    let Ok(err) = serde_json::from_str::<ErrorResponse>(body) else {
        // No parseable envelope: keep the existing reason-less fallbacks.
        let message = match status {
            401 => "authentication failed — run `speclink auth login`".to_string(),
            404 => "resource not found — run `speclink list` to check the name".to_string(),
            _ => format!(
                "unexpected server response — update speclink or report a bug (HTTP {status})"
            ),
        };
        return RemoteError { message, reason: None, status: Some(status), evidence: None };
    };

    let reason = Some(err.reason.as_str().to_string());
    let message = match err.reason {
        ErrorReason::RevisionConflict => {
            "content changed since you read it — re-read it and re-apply your edit".to_string()
        }
        ErrorReason::Unavailable => unavailable_without_a_reason(),
        ErrorReason::Internal => {
            "internal speclink error — update speclink or report a bug".to_string()
        }
        ErrorReason::PermissionDenied if status == 401 => {
            "authentication failed — run `speclink auth login`".to_string()
        }
        // 403 relays the server's reason inside the client-owned frame: a
        // non-member and a reader write need different fixes.
        ErrorReason::PermissionDenied => {
            format!("{DENIED_BEFORE}{}{DENIED_AFTER}", err.message)
        }
        ErrorReason::NotFound
        | ErrorReason::InvalidArgument
        | ErrorReason::InvalidConfig
        | ErrorReason::Refused => err.message,
        ErrorReason::Unknown(_) => format!(
            "unexpected server response — update speclink or report a bug (HTTP {status})"
        ),
    };

    RemoteError { message, reason, status: Some(status), evidence: err.evidence }
}

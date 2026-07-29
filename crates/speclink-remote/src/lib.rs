//! speclink-remote: the typed protocol client for the Speclink Client
//! Protocol (see `docs/platform-architecture.zh-TW.md` §4.5), used by the
//! CLI's remote mode.
//!
//! These concerns live here and nowhere else:
//! - `client`: a single request layer plus the per-verb path mapping, all
//!   speclink-protocol DTOs — no raw JSON travels through this crate.
//! - `credentials`: the OS keyring store, keyed by origin and kind.
//! - `auth`: credentials-file read/write and the resolution ladder.
//! - `refresh`: bearer acquisition and rotation, serialized across processes.
//! - `login`: driving a device authorization to a verdict.
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
pub mod events;
pub mod login;
pub mod refresh;

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
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RemoteError {}

/// Translate a transport-level failure (refused/timeout/DNS) — loud, no
/// cache fallback, no retry loop.
pub fn translate_transport() -> RemoteError {
    RemoteError {
        message: "server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".into(),
        reason: None,
        status: None,
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

    // Server-side failures never carry a usable envelope — translate on
    // status first, like the 5xx rule the client has always had.
    if status >= 500 {
        return RemoteError {
            message: "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".into(),
            reason: None,
            status: Some(status),
        };
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
        return RemoteError { message, reason: None, status: Some(status) };
    };

    let reason = Some(err.reason.as_str().to_string());
    let message = match err.reason {
        ErrorReason::RevisionConflict => {
            "content changed since you read it — re-read it and re-apply your edit".to_string()
        }
        ErrorReason::Unavailable => {
            "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".to_string()
        }
        ErrorReason::Internal => {
            "internal speclink error — update speclink or report a bug".to_string()
        }
        ErrorReason::PermissionDenied if status == 401 => {
            "authentication failed — run `speclink auth login`".to_string()
        }
        ErrorReason::PermissionDenied => {
            "access denied — your account has no access to this project; ask a project admin"
                .to_string()
        }
        ErrorReason::NotFound
        | ErrorReason::InvalidArgument
        | ErrorReason::InvalidConfig
        | ErrorReason::Refused => err.message,
        ErrorReason::Unknown(_) => format!(
            "unexpected server response — update speclink or report a bug (HTTP {status})"
        ),
    };

    RemoteError { message, reason, status: Some(status) }
}

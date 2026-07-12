//! speclink-remote: the thin HTTP client for the Speclink verb contract
//! (see `docs/platform-architecture.zh-TW.md`), used by the CLI's remote mode.
//!
//! Three concerns live here and nowhere else:
//! - `client`: a single request layer plus the per-verb path mapping.
//! - `auth`: credentials-file read/write and token resolution order.
//! - the error-translation table below: every non-2xx response becomes a
//!   single-line semantic message with a suggested action — a bare HTTP
//!   status code is never the primary error output.
//!
//! `speclink-core` must never depend on this crate (the core keeps its
//! "no network calls" red line); only the CLI does.

pub mod auth;
pub mod client;

/// A translated remote failure: one semantic line for the user/agent, plus
/// the machine-readable `reason` when the server provided one.
#[derive(Debug)]
pub struct RemoteError {
    pub message: String,
    pub reason: Option<String>,
}

impl std::fmt::Display for RemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RemoteError {}

fn body_str<'a>(body: Option<&'a serde_json::Value>, key: &str) -> Option<&'a str> {
    body.and_then(|b| b.get(key)).and_then(|v| v.as_str())
}

fn body_u64(body: Option<&serde_json::Value>, key: &str) -> Option<u64> {
    body.and_then(|b| b.get(key)).and_then(|v| v.as_u64())
}

fn body_list(body: Option<&serde_json::Value>, key: &str) -> Vec<String> {
    body.and_then(|b| b.get(key))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Translate a non-2xx response into a [`RemoteError`] — the single,
/// centralized error-translation table (contract §4). Adding an error path
/// anywhere in the client means adding a row here, nowhere else.
pub fn translate_status(status: u16, body: Option<&serde_json::Value>) -> RemoteError {
    let reason = body_str(body, "reason").map(str::to_string);

    // Server-side failures never carry a usable reason — translate on status
    // before consulting the reason table.
    if status >= 500 {
        return RemoteError {
            message: "server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".into(),
            reason,
        };
    }

    let message = match reason.as_deref() {
        Some("token_missing") | Some("token_invalid") => {
            "authentication failed — run `speclink auth login`".to_string()
        }
        Some("token_expired") => "credentials expired — run `speclink auth login`".to_string(),
        Some("token_revoked") => "credentials revoked — run `speclink auth login`".to_string(),
        Some("access_denied") => {
            "access denied — your account has no access to this project; ask a project admin"
                .to_string()
        }
        Some("repo_unknown") => {
            let available = body_list(body, "availableRepos").join(", ");
            format!(
                "repo is not registered in this project (available: {available}) — fix `remote.repo` in .speclink.yaml or re-run `speclink link`"
            )
        }
        Some("repo_mismatch") => {
            let change_repo = body_str(body, "changeRepo").unwrap_or("another repo");
            let request_repo = body_str(body, "requestRepo").unwrap_or("this repo");
            format!(
                "change belongs to repo '{change_repo}' but you are '{request_repo}' — run this verb from the owning repo"
            )
        }
        Some("repo_required") => {
            "this project has multiple repos — set `remote.repo` in .speclink.yaml (see `speclink link`)"
                .to_string()
        }
        Some("not_found") => {
            let resource = body_str(body, "resource").unwrap_or("resource");
            let name = body_str(body, "name").unwrap_or("(unnamed)");
            format!(
                "{resource} '{name}' not found — run `speclink list` (or the matching list verb) to check the name"
            )
        }
        Some("already_exists") => "name already in use — pick another".to_string(),
        Some("version_conflict") => {
            // Archive reports per-capability conflicts as objects; artifact
            // writes report a plain currentVersion — pick the message by shape.
            let caps: Vec<String> = body
                .and_then(|b| b.get("conflicts"))
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|c| c.get("capability").and_then(|v| v.as_str()))
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            if caps.is_empty() {
                "content changed since you read it — re-read it and re-apply your edit".to_string()
            } else {
                format!(
                    "canonical spec(s) {} moved since propose — resolve in the team system, then retry",
                    caps.join(", ")
                )
            }
        }
        Some("ownership_lost") => {
            let holder = body_str(body, "claimedBy").unwrap_or("someone else");
            format!("change is held by {holder} — coordinate, or re-claim if it was released")
        }
        Some("change_busy") => {
            let lifecycle = body_str(body, "lifecycle").unwrap_or("busy");
            format!("change is {lifecycle} — wait for the in-flight operation to finish, then retry")
        }
        Some("gate_pending") => {
            let gate = body_str(body, "gate").unwrap_or("gate");
            format!("waiting for {gate} approval in the team system — ask the approver")
        }
        Some("tasks_incomplete") => {
            let remaining = body_u64(body, "remaining").unwrap_or(0);
            format!("{remaining} task(s) still open — finish them before archiving")
        }
        Some("discussion_archived") => {
            let slug = body_str(body, "slug").unwrap_or("(unknown)");
            format!("discussion '{slug}' is archived — restore it in the team system first")
        }
        Some("project_not_empty") => {
            "target project already contains changes — push requires an empty project".to_string()
        }
        Some("api_version_unsupported") => {
            "server does not support this CLI's API version — upgrade the CLI or the server"
                .to_string()
        }
        Some("validation_failed") => {
            let errors = body_list(body, "errors");
            if errors.is_empty() {
                "validation failed".to_string()
            } else {
                format!("validation failed: {}", errors.join("; "))
            }
        }
        Some("bad_request") | Some("if_match_required") => {
            "internal speclink error — update speclink or report a bug".to_string()
        }
        // Reason-less status fallbacks for servers that omit the envelope.
        None if status == 401 => "authentication failed — run `speclink auth login`".to_string(),
        None if status == 404 => {
            "resource not found — run `speclink list` to check the name".to_string()
        }
        // Unknown reason / unknown status: generic fallback. The status code
        // appears parenthetically for bug reports only — never as the primary
        // message.
        _ => format!(
            "unexpected server response — update speclink or report a bug (HTTP {status})"
        ),
    };

    RemoteError { message, reason }
}

/// Translate a transport-level failure (refused/timeout/DNS) — loud, no
/// cache fallback, no retry loop.
pub fn translate_transport() -> RemoteError {
    RemoteError {
        message: "server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)".into(),
        reason: None,
    }
}

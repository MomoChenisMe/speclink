//! The device authorization flow client (design 決策 1): initiate, poll,
//! refresh and revoke against a server's root-level `/auth/*` endpoints,
//! speclink-protocol device DTOs end to end.
//!
//! Unlike [`crate::client::Client`], these calls are pre-authentication and
//! server-rooted: `base_url` is the server origin (`http://host:port`), not a
//! project-scoped connection URL, and no bearer travels.
//!
//! The pre-login probe (design 決策 3) is baked into [`initiate`]: a 404/405
//! answer is the explicit [`InitiateOutcome::Unsupported`] signal (the PAT
//! fallback trigger), while a 5xx or transport failure stays an error — a
//! broken server must never read as "use a PAT instead".
//!
//! `poll` is a single observation returning the protocol's typed status; the
//! interval-respecting loop (and its cancellation) belongs to the caller's
//! orchestration (design 決策 5).

use crate::{translate_protocol_error, translate_transport, RemoteError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use speclink_protocol::device::{
    DeviceAuthorizationResponse, DeviceTokenRequest, DeviceTokenResponse, RefreshRequest,
    RefreshResponse, RevokeRequest, RevokeResponse,
};
use speclink_protocol::query::AuthWhoamiResponse;

/// The outcome of probing the initiation endpoint (design 決策 3).
#[derive(Debug)]
pub enum InitiateOutcome {
    /// The server offers the device flow: the two codes and poll metadata.
    Supported(DeviceAuthorizationResponse),
    /// The server answered 404/405 — no device flow; fall back to a PAT.
    Unsupported,
}

/// One root-level `/auth/*` POST: the body is the request DTO's serialization,
/// the response parses into the response DTO, non-2xx goes through the shared
/// registry mapping.
fn post<T: DeserializeOwned, B: Serialize>(
    base_url: &str,
    path: &str,
    body: Option<&B>,
) -> Result<T, (Option<u16>, RemoteError)> {
    let url = format!("{}{path}", base_url.trim_end_matches('/'));
    let req = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .post(&url);
    let result = match body {
        Some(payload) => req.send_json(payload),
        None => req.call(),
    };
    match result {
        Ok(resp) => resp.into_json().map_err(|_| {
            (
                None,
                RemoteError {
                    message:
                        "unexpected server response — the server did not return valid JSON".into(),
                    reason: None,
                    status: None,
                },
            )
        }),
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err((Some(status), translate_protocol_error(status, &body)))
        }
        Err(ureq::Error::Transport(_)) => Err((None, translate_transport())),
    }
}

/// `POST /auth/device` — initiate a device authorization, doubling as the
/// pre-login probe: 2xx is [`InitiateOutcome::Supported`], 404/405 is the
/// explicit [`InitiateOutcome::Unsupported`] fallback signal, anything else
/// (5xx, transport) is an error.
pub fn initiate(base_url: &str) -> Result<InitiateOutcome, RemoteError> {
    match post::<DeviceAuthorizationResponse, ()>(base_url, "/auth/device", None) {
        Ok(auth) => Ok(InitiateOutcome::Supported(auth)),
        Err((Some(404 | 405), _)) => Ok(InitiateOutcome::Unsupported),
        Err((_, err)) => Err(err),
    }
}

/// `POST /auth/device/token` — one poll of a device authorization. The state
/// machine (pending/slow_down/approved/expired/denied) comes back as the
/// response's typed `status`; only protocol-level faults are errors.
pub fn poll(base_url: &str, device_code: &str) -> Result<DeviceTokenResponse, RemoteError> {
    post(
        base_url,
        "/auth/device/token",
        Some(&DeviceTokenRequest { device_code: device_code.to_string() }),
    )
    .map_err(|(_, err)| err)
}

/// `POST /auth/refresh` — rotate a refresh credential for a fresh pair. A
/// spent or unknown credential is a refusal (`permission_denied`).
pub fn refresh(base_url: &str, refresh_token: &str) -> Result<RefreshResponse, RemoteError> {
    post(
        base_url,
        "/auth/refresh",
        Some(&RefreshRequest { refresh_token: refresh_token.to_string() }),
    )
    .map_err(|(_, err)| err)
}

/// `GET /auth/whoami` — the identity behind a bearer (access token or PAT) at
/// the server root, no project scope (design 決策 8): the display a client
/// writes back to its registry right after logging in, and the validation
/// call of a pasted PAT.
pub fn whoami(base_url: &str, bearer: &str) -> Result<AuthWhoamiResponse, RemoteError> {
    let url = format!("{}/auth/whoami", base_url.trim_end_matches('/'));
    let result = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .get(&url)
        .set("Authorization", &format!("Bearer {bearer}"))
        .call();
    match result {
        Ok(resp) => resp.into_json().map_err(|_| RemoteError {
            message: "unexpected server response — the server did not return valid JSON".into(),
            reason: None,
            status: None,
        }),
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(translate_protocol_error(status, &body))
        }
        Err(ureq::Error::Transport(_)) => Err(translate_transport()),
    }
}

/// `POST /auth/revoke` — revoke the family a refresh credential belongs to
/// (logout semantics). An unknown credential is a refusal.
pub fn revoke(base_url: &str, refresh_token: &str) -> Result<(), RemoteError> {
    post::<RevokeResponse, RevokeRequest>(
        base_url,
        "/auth/revoke",
        Some(&RevokeRequest { refresh_token: refresh_token.to_string() }),
    )
    .map(|_| ())
    .map_err(|(_, err)| err)
}

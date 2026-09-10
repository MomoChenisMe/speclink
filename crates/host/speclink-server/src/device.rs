//! Device authorization endpoints (blueprint §13.3): initiation and polling.
//!
//! The polling state machine's states travel as the response's typed `status`
//! field (决策 1) on HTTP 200 — they are not wire errors. Only a truly
//! protocol-level fault (an unknown or blank device code) leaves as an
//! [`ApiError`] from the eight-value registry. Token minting, refresh rotation
//! and revoke are layered on in the following tasks.

use crate::error::ApiError;
use crate::identity::{DevicePoll, RefreshOutcome};
use crate::state::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration, Utc};
use speclink_protocol::device::{
    DeviceAuthorizationResponse, DeviceTokenRequest, DeviceTokenResponse, DeviceTokenStatus,
    RefreshRequest, RefreshResponse, RevokeRequest, RevokeResponse,
};

/// How long a device authorization request stays valid before it expires.
fn device_ttl() -> Duration {
    Duration::minutes(15)
}

/// The minimum interval a client must wait between polls.
fn device_interval() -> Duration {
    Duration::seconds(5)
}

/// `POST /auth/device` — initiate a device authorization. No authentication: a
/// yet-unauthenticated client asks for the two codes and the approval URL.
pub async fn initiate(State(state): State<AppState>) -> Result<Response, ApiError> {
    let ttl = device_ttl();
    let interval = device_interval();
    let auth = state
        .identity
        .create_device_authorization(interval, ttl)
        .map_err(|_| ApiError::internal("identity store unavailable"))?;
    let resp = DeviceAuthorizationResponse {
        device_code: auth.device_code,
        user_code: auth.user_code,
        verification_uri: format!("{}/activate", state.config.public_url.trim_end_matches('/')),
        expires_in: ttl.num_seconds().max(0) as u64,
        interval: interval.num_seconds().max(0) as u64,
    };
    Ok(Json(resp).into_response())
}

/// `POST /auth/device/token` — poll a device authorization by its device code.
/// The intermediate and terminal states answer 200 with a typed `status`; a
/// blank code is `invalid_argument` and an unknown one is `not_found`.
pub async fn poll_token(
    State(state): State<AppState>,
    Json(req): Json<DeviceTokenRequest>,
) -> Result<Response, ApiError> {
    if req.device_code.trim().is_empty() {
        return Err(ApiError::invalid_argument("device code is required"));
    }
    let resp = match state
        .identity
        .poll_device(&req.device_code)
        .map_err(|_| ApiError::internal("identity store unavailable"))?
    {
        DevicePoll::NotFound => return Err(ApiError::not_found("unknown device code")),
        DevicePoll::Pending => bare(DeviceTokenStatus::Pending),
        DevicePoll::SlowDown => bare(DeviceTokenStatus::SlowDown),
        DevicePoll::Expired => bare(DeviceTokenStatus::Expired),
        DevicePoll::Denied => bare(DeviceTokenStatus::Denied),
        DevicePoll::Approved(pair) => DeviceTokenResponse {
            status: DeviceTokenStatus::Approved,
            expires_in: Some((pair.access_expires_at - Utc::now()).num_seconds().max(0) as u64),
            access_token: Some(pair.access_token),
            refresh_token: Some(pair.refresh_token),
        },
    };
    Ok(Json(resp).into_response())
}

/// A poll response carrying only a non-approved status (no token fields).
fn bare(status: DeviceTokenStatus) -> DeviceTokenResponse {
    DeviceTokenResponse {
        status,
        access_token: None,
        refresh_token: None,
        expires_in: None,
    }
}

/// `POST /auth/refresh` — rotate a refresh credential for a fresh pair. A
/// rotated-away or revoked value (reuse) has just torn down its whole family and
/// returns 401, as does an unknown one.
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Response, ApiError> {
    match state
        .identity
        .refresh(&req.refresh_token)
        .map_err(|_| ApiError::internal("identity store unavailable"))?
    {
        RefreshOutcome::Rotated(pair) => Ok(Json(RefreshResponse {
            expires_in: (pair.access_expires_at - Utc::now()).num_seconds().max(0) as u64,
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
        })
        .into_response()),
        RefreshOutcome::Reused | RefreshOutcome::NotFound => {
            Err(ApiError::permission_denied("invalid refresh credential"))
        }
    }
}

/// `POST /auth/revoke` — revoke the family a refresh credential belongs to
/// (logout). An unknown credential is 401.
pub async fn revoke(
    State(state): State<AppState>,
    Json(req): Json<RevokeRequest>,
) -> Result<Response, ApiError> {
    if state
        .identity
        .revoke_family_by_refresh(&req.refresh_token)
        .map_err(|_| ApiError::internal("identity store unavailable"))?
    {
        Ok(Json(RevokeResponse {}).into_response())
    } else {
        Err(ApiError::permission_denied("invalid refresh credential"))
    }
}

//! Device authorization flow DTOs (blueprint §13.3): the initiation and polling
//! shapes, refresh rotation, and revoke.
//!
//! The polling state machine's states (pending, slow_down, approved, expired,
//! denied) travel as a typed `status` field (design 決策一) — they are not wire
//! errors, so the eight-value error reason registry is never widened. Every
//! field serializes camelCase and every DTO exports a JSON Schema, matching the
//! rest of the protocol crate.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `POST /auth/device` response: the two codes plus the approval URL, the
/// expiry, and the minimum poll interval. The device code is high-entropy (used
/// to poll and to exchange for tokens); the user code is short and human-entered
/// on the approval page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    /// Seconds until the authorization request expires.
    pub expires_in: u64,
    /// Minimum seconds a client must wait between polls.
    pub interval: u64,
}

/// `POST /auth/device/token` request: the device code obtained at initiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRequest {
    pub device_code: String,
}

/// The polling state machine's states (design 決策一). Not wire errors — a poll
/// always answers HTTP 200 with one of these in `status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeviceTokenStatus {
    Pending,
    SlowDown,
    Approved,
    Expired,
    Denied,
}

/// `POST /auth/device/token` response. On `approved` the token pair and the
/// access token's lifetime are present; every other status carries only
/// `status`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenResponse {
    pub status: DeviceTokenStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Seconds until the issued access token expires (present with `approved`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
}

/// `POST /auth/refresh` request: the current refresh credential.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// `POST /auth/refresh` response: the rotated token pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    /// Seconds until the new access token expires.
    pub expires_in: u64,
}

/// `POST /auth/revoke` request: the refresh credential whose family to revoke
/// (logout semantics).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevokeRequest {
    pub refresh_token: String,
}

#[cfg(test)]
mod tests {
    use crate::device::*;

    #[test]
    fn initiation_response_round_trips_camel_case() {
        let resp: DeviceAuthorizationResponse = serde_json::from_str(
            r#"{"deviceCode":"dc_abc","userCode":"WDJB-MJHT","verificationUri":"https://speclink.example/activate","expiresIn":900,"interval":5}"#,
        )
        .unwrap();
        assert_eq!(resp.device_code, "dc_abc");
        assert_eq!(resp.user_code, "WDJB-MJHT");
        assert_eq!(resp.verification_uri, "https://speclink.example/activate");
        assert_eq!(resp.expires_in, 900);
        assert_eq!(resp.interval, 5);

        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["deviceCode"], "dc_abc", "fields serialize camelCase: {json}");
        assert_eq!(json["verificationUri"], "https://speclink.example/activate");
        assert_eq!(json["expiresIn"], 900);
        let back: DeviceAuthorizationResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, resp);
    }

    #[test]
    fn poll_states_are_snake_case_and_pending_carries_no_tokens() {
        let cases = [
            (DeviceTokenStatus::Pending, "pending"),
            (DeviceTokenStatus::SlowDown, "slow_down"),
            (DeviceTokenStatus::Approved, "approved"),
            (DeviceTokenStatus::Expired, "expired"),
            (DeviceTokenStatus::Denied, "denied"),
        ];
        for (status, wire) in cases {
            assert_eq!(serde_json::to_value(status).unwrap(), wire, "stable wire string for {status:?}");
            let back: DeviceTokenStatus =
                serde_json::from_value(serde_json::Value::String(wire.into())).unwrap();
            assert_eq!(back, status, "round-trips from {wire}");
        }

        // A pending poll omits the token fields entirely.
        let pending = DeviceTokenResponse {
            status: DeviceTokenStatus::Pending,
            access_token: None,
            refresh_token: None,
            expires_in: None,
        };
        let json = serde_json::to_value(&pending).unwrap();
        assert_eq!(json.as_object().unwrap().len(), 1, "only status present: {json}");
        assert_eq!(json["status"], "pending");
    }

    #[test]
    fn an_approved_poll_carries_the_token_pair() {
        let approved: DeviceTokenResponse = serde_json::from_str(
            r#"{"status":"approved","accessToken":"spk_at_aaa","refreshToken":"spk_rt_bbb","expiresIn":3600}"#,
        )
        .unwrap();
        assert_eq!(approved.status, DeviceTokenStatus::Approved);
        assert_eq!(approved.access_token.as_deref(), Some("spk_at_aaa"));
        assert_eq!(approved.refresh_token.as_deref(), Some("spk_rt_bbb"));
        assert_eq!(approved.expires_in, Some(3600));

        let json = serde_json::to_value(&approved).unwrap();
        assert_eq!(json["accessToken"], "spk_at_aaa", "camelCase: {json}");
        assert_eq!(json["refreshToken"], "spk_rt_bbb");
        let back: DeviceTokenResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, approved);
    }

    #[test]
    fn refresh_and_revoke_round_trip_camel_case() {
        let req: RefreshRequest =
            serde_json::from_str(r#"{"refreshToken":"spk_rt_old"}"#).unwrap();
        assert_eq!(req.refresh_token, "spk_rt_old");
        assert_eq!(serde_json::to_value(&req).unwrap()["refreshToken"], "spk_rt_old");

        let resp: RefreshResponse = serde_json::from_str(
            r#"{"accessToken":"spk_at_new","refreshToken":"spk_rt_new","expiresIn":3600}"#,
        )
        .unwrap();
        assert_eq!(resp.access_token, "spk_at_new");
        assert_eq!(resp.refresh_token, "spk_rt_new");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["accessToken"], "spk_at_new", "camelCase: {json}");
        let back: RefreshResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, resp);

        let revoke: RevokeRequest =
            serde_json::from_str(r#"{"refreshToken":"spk_rt_bye"}"#).unwrap();
        assert_eq!(revoke.refresh_token, "spk_rt_bye");
        assert_eq!(serde_json::to_value(&revoke).unwrap()["refreshToken"], "spk_rt_bye");
    }

    #[test]
    fn device_dtos_export_json_schema() {
        for (name, schema) in [
            ("DeviceAuthorizationResponse", schemars::schema_for!(DeviceAuthorizationResponse)),
            ("DeviceTokenRequest", schemars::schema_for!(DeviceTokenRequest)),
            ("DeviceTokenResponse", schemars::schema_for!(DeviceTokenResponse)),
            ("RefreshResponse", schemars::schema_for!(RefreshResponse)),
        ] {
            let text = serde_json::to_string(&schema)
                .unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains("properties"), "{name} schema has properties: {text}");
        }
        let init = serde_json::to_string(&schemars::schema_for!(DeviceAuthorizationResponse)).unwrap();
        assert!(
            init.contains("deviceCode") && init.contains("verificationUri"),
            "schema fields are camelCase: {init}"
        );
    }
}

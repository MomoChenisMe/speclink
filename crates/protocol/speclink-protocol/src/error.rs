//! The standard error reason registry and the wire error envelope.
//!
//! Every protocol failure is the `{ status, reason, message }` triple:
//! HTTP status, a machine-readable reason from the closed registry, and the
//! human message. The registry is the union of the store's six classes and
//! the command layer's five codes, deduplicated (design decision three) —
//! no fine-grained code field and no detail discriminator fields exist on
//! the wire; sub-case wording travels in `message`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The wire error envelope: the `{ status, reason, message }` triple, plus —
/// only on the refusals that carry it — flattened structured evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// The HTTP status the response travelled with, repeated in the body so
    /// the envelope is self-describing for non-HTTP consumers of the schema.
    pub status: u16,
    pub reason: ErrorReason,
    pub message: String,
    /// Work-trace evidence of the in-progress removal gate (D4): flattened
    /// additive camelCase fields, absent on every other error so existing
    /// payloads stay byte-identical.
    #[serde(flatten)]
    pub evidence: Option<crate::command::RevertBlockedEvidence>,
}

/// The closed error reason registry. A reason string outside the registry
/// deserializes as [`ErrorReason::Unknown`] — a client must treat it as a
/// generic error (keeping `message` for display), never fail on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorReason {
    NotFound,
    PermissionDenied,
    RevisionConflict,
    InvalidArgument,
    InvalidConfig,
    Refused,
    Unavailable,
    Internal,
    /// Forward-compatibility escape hatch — not a registry member. Carries
    /// the raw reason string a newer or non-conformant server sent.
    #[serde(untagged)]
    Unknown(String),
}

impl ErrorReason {
    /// The wire string for this reason.
    pub fn as_str(&self) -> &str {
        match self {
            ErrorReason::NotFound => "not_found",
            ErrorReason::PermissionDenied => "permission_denied",
            ErrorReason::RevisionConflict => "revision_conflict",
            ErrorReason::InvalidArgument => "invalid_argument",
            ErrorReason::InvalidConfig => "invalid_config",
            ErrorReason::Refused => "refused",
            ErrorReason::Unavailable => "unavailable",
            ErrorReason::Internal => "internal",
            ErrorReason::Unknown(raw) => raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::*;

    #[test]
    fn error_response_is_the_status_reason_message_triple() {
        let err: ErrorResponse = serde_json::from_str(
            r#"{"status":409,"reason":"revision_conflict","message":"stale write"}"#,
        )
        .unwrap();
        assert_eq!(err.status, 409);
        assert_eq!(err.reason, ErrorReason::RevisionConflict);
        assert_eq!(err.message, "stale write");

        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["status"], 409);
        assert_eq!(json["reason"], "revision_conflict");
        assert_eq!(json["message"], "stale write");
        assert_eq!(
            json.as_object().unwrap().len(),
            3,
            "the envelope is exactly the triple: {json}"
        );
    }

    #[test]
    fn registry_is_the_closed_eight_value_set() {
        let wire_of = |r: &ErrorReason| serde_json::to_value(r).unwrap();
        let cases = [
            (ErrorReason::NotFound, "not_found"),
            (ErrorReason::PermissionDenied, "permission_denied"),
            (ErrorReason::RevisionConflict, "revision_conflict"),
            (ErrorReason::InvalidArgument, "invalid_argument"),
            (ErrorReason::InvalidConfig, "invalid_config"),
            (ErrorReason::Refused, "refused"),
            (ErrorReason::Unavailable, "unavailable"),
            (ErrorReason::Internal, "internal"),
        ];
        for (reason, wire) in &cases {
            assert_eq!(wire_of(reason), *wire, "stable wire string for {reason:?}");
            let back: ErrorReason =
                serde_json::from_value(serde_json::Value::String((*wire).into())).unwrap();
            assert_eq!(&back, reason, "round-trips from {wire}");
        }
        // Exhaustive match without a wildcard arm for the registry: adding a
        // ninth registry value breaks this test at compile time — that is the
        // point of a closed set.
        for (reason, _) in cases {
            match reason {
                ErrorReason::NotFound
                | ErrorReason::PermissionDenied
                | ErrorReason::RevisionConflict
                | ErrorReason::InvalidArgument
                | ErrorReason::InvalidConfig
                | ErrorReason::Refused
                | ErrorReason::Unavailable
                | ErrorReason::Internal => {}
                ErrorReason::Unknown(_) => unreachable!("registry values only"),
            }
        }
    }

    #[test]
    fn unknown_reason_deserializes_without_failing_and_keeps_the_string() {
        let err: ErrorResponse = serde_json::from_str(
            r#"{"status":418,"reason":"im_a_teapot","message":"short and stout"}"#,
        )
        .unwrap();
        assert_eq!(err.reason, ErrorReason::Unknown("im_a_teapot".into()));
        assert_eq!(
            err.message, "short and stout",
            "the message survives for generic display"
        );
        // A generic handler can still read the raw string back out.
        assert_eq!(err.reason.as_str(), "im_a_teapot");
        assert_eq!(
            serde_json::to_value(&err.reason).unwrap(),
            "im_a_teapot",
            "unknown reasons re-serialize as their raw string"
        );
    }

    #[test]
    fn registry_reasons_expose_their_wire_string() {
        assert_eq!(ErrorReason::NotFound.as_str(), "not_found");
        assert_eq!(ErrorReason::Refused.as_str(), "refused");
        assert_eq!(ErrorReason::Internal.as_str(), "internal");
    }

    #[test]
    fn error_dtos_export_json_schema() {
        let schema = schemars::schema_for!(ErrorResponse);
        let text = serde_json::to_string(&schema).expect("ErrorResponse schema serializes");
        assert!(text.contains("properties"), "schema has properties: {text}");
        assert!(
            text.contains("revision_conflict"),
            "schema documents the registry values: {text}"
        );
    }
}

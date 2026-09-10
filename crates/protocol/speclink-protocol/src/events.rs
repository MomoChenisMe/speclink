//! Event discovery declaration types (blueprint §9.2): transports, polling,
//! and resume capability. Declaration only — no transport is implemented or
//! connected in this knife; Query + ETag remains the recovery bedrock.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The `capabilities.events` declaration a server hands out at handshake.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventsDeclaration {
    #[serde(default)]
    pub transports: Vec<EventTransport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polling: Option<PollingDeclaration>,
}

/// One declared push transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventTransport {
    #[serde(rename = "type")]
    pub kind: TransportKind,
    pub url: String,
    #[serde(default)]
    pub resume: bool,
}

/// The declared transport kind. Unknown kinds deserialize as
/// [`TransportKind::Unknown`] so a newer server never breaks an older client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TransportKind {
    Sse,
    Websocket,
    #[serde(untagged)]
    Unknown(String),
}

/// The polling fallback declaration — the mandatory recovery bedrock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollingDeclaration {
    pub url: String,
    #[serde(default)]
    pub etag: bool,
}

/// A commit invalidation hint (blueprint §9.1): the event is a pointer, not a
/// payload. The client re-reads the canon through Query + ETag, so a missed
/// event still converges by polling. Carries only the event id (the scope's
/// outbox sequence, as a string), the resource category, the resource id, and
/// the project revision — never spec content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvalidationEvent {
    /// This event's outbox sequence in its scope, as a string — the same value
    /// as the SSE `id:` line, so a client's `Last-Event-ID` resumes from it.
    pub event_id: String,
    /// The invalidated resource category.
    pub scope: InvalidationScope,
    /// The invalidated resource's identity — a change name or a discussion slug
    /// (empty for the `unknown` category).
    pub resource_id: String,
    /// The project revision at the commit that produced this event.
    pub revision: u64,
}

/// The resource category an invalidation points at. Unknown categories
/// deserialize as [`InvalidationScope::Unknown`] so a newer server never breaks
/// an older client (mirrors [`TransportKind`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvalidationScope {
    Change,
    Discussion,
    Spec,
    #[serde(untagged)]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invalidation_event_round_trips_camelcase() {
        let ev = InvalidationEvent {
            event_id: "42".to_string(),
            scope: InvalidationScope::Change,
            resource_id: "add-payment".to_string(),
            revision: 42,
        };
        let json = serde_json::to_value(&ev).unwrap();
        assert_eq!(json["eventId"], "42", "fields serialize camelCase: {json}");
        assert_eq!(json["scope"], "change");
        assert_eq!(json["resourceId"], "add-payment");
        assert_eq!(json["revision"], 42);
        let back: InvalidationEvent = serde_json::from_value(json).unwrap();
        assert_eq!(back, ev, "the DTO round-trips");
    }

    #[test]
    fn scope_categories_serialize_lowercase() {
        assert_eq!(serde_json::to_value(InvalidationScope::Change).unwrap(), "change");
        assert_eq!(serde_json::to_value(InvalidationScope::Discussion).unwrap(), "discussion");
        assert_eq!(serde_json::to_value(InvalidationScope::Spec).unwrap(), "spec");
    }

    #[test]
    fn an_unknown_scope_category_survives_as_a_declaration() {
        // A newer server's scope kind never breaks an older client's parse.
        let scope: InvalidationScope = serde_json::from_value(serde_json::json!("config")).unwrap();
        assert_eq!(scope, InvalidationScope::Unknown("config".to_string()));
        // The server's own unmapped category serializes to the literal "unknown".
        assert_eq!(
            serde_json::to_value(InvalidationScope::Unknown("unknown".to_string())).unwrap(),
            "unknown"
        );
    }

    #[test]
    fn invalidation_event_exports_json_schema() {
        let schema = schemars::schema_for!(InvalidationEvent);
        let text = serde_json::to_string(&schema).expect("InvalidationEvent schema serializes");
        assert!(
            text.contains("eventId") && text.contains("resourceId"),
            "schema fields are camelCase: {text}"
        );
    }
}

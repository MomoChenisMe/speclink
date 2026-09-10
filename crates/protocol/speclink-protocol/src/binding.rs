//! Binding handshake shapes (blueprint §4.7 `GET /binding`): the unambiguous
//! actor/project/repo identity, version pair, and capability declarations a
//! client must obtain before any verb flows.

use crate::events::EventsDeclaration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `GET /binding` response. A server that cannot produce an unambiguous
/// binding (missing, unauthorized, or several candidates) rejects with a
/// registry error instead — there is no multi-candidate success shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BindingResponse {
    pub actor: Actor,
    pub project: ScopeRef,
    pub repo: ScopeRef,
    pub api_version: String,
    pub engine_version: String,
    #[serde(default)]
    pub capabilities: Capabilities,
}

/// The authenticated actor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: String,
    pub name: String,
}

/// A bound project or repo: immutable server id plus the stable readable key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScopeRef {
    pub id: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub name: String,
}

/// Capability declarations carried by the handshake.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub context_snapshots: bool,
    #[serde(default)]
    pub policy_write: bool,
    /// validate/analyze 唯讀衍生查詢端點（全 role 可用）。
    #[serde(default)]
    pub validate: bool,
    #[serde(default)]
    pub analyze: bool,
    /// 寫入動詞（editor 限定；reader 收 false 呈現停用）。
    #[serde(default)]
    pub delete_change: bool,
    #[serde(default)]
    pub move_task: bool,
    #[serde(default)]
    pub authentication: Vec<String>,
    #[serde(default)]
    pub events: EventsDeclaration,
}

#[cfg(test)]
mod tests {
    use crate::binding::*;
    use crate::events::TransportKind;

    #[test]
    fn handshake_response_carries_identity_versions_and_event_declarations() {
        let handshake: BindingResponse = serde_json::from_str(
            r#"{"actor":{"id":"u_42","name":"王小明"},"project":{"id":"prj_01H","key":"erp","name":"ERP"},"repo":{"id":"repo_01H","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"1.4.0","capabilities":{"contextSnapshots":true,"authentication":["pat"],"events":{"transports":[{"type":"sse","url":"/events","resume":true}],"polling":{"url":"/sync-state","etag":true}}}}"#,
        )
        .unwrap();
        assert_eq!(handshake.actor.id, "u_42");
        assert_eq!(handshake.project.key, "erp");
        assert_eq!(handshake.repo.key, "backend");
        assert_eq!(handshake.api_version, crate::API_VERSION);
        assert_eq!(handshake.engine_version, "1.4.0");
        assert!(handshake.capabilities.context_snapshots);
        assert!(!handshake.capabilities.policy_write, "older handshakes default read-only");

        let events = &handshake.capabilities.events;
        assert_eq!(events.transports.len(), 1);
        assert_eq!(events.transports[0].kind, TransportKind::Sse);
        assert_eq!(events.transports[0].url, "/events");
        assert!(events.transports[0].resume);
        let polling = events.polling.as_ref().expect("polling declaration kept");
        assert_eq!(polling.url, "/sync-state");
        assert!(polling.etag);

        let json = serde_json::to_value(&handshake).unwrap();
        assert_eq!(json["apiVersion"], "1", "fields serialize camelCase: {json}");
        assert_eq!(json["capabilities"]["events"]["transports"][0]["type"], "sse");
        let back: BindingResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, handshake);
    }

    #[test]
    fn capabilities_default_when_the_server_omits_them() {
        let handshake: BindingResponse = serde_json::from_str(
            r#"{"actor":{"id":"u_1","name":"m"},"project":{"id":"p_1","key":"erp","name":"ERP"},"repo":{"id":"r_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0"}"#,
        )
        .unwrap();
        assert!(handshake.capabilities.events.transports.is_empty());
        assert!(handshake.capabilities.events.polling.is_none());
        assert!(!handshake.capabilities.policy_write);
    }

    #[test]
    fn unknown_transport_kinds_survive_as_declarations() {
        let handshake: BindingResponse = serde_json::from_str(
            r#"{"actor":{"id":"u_1","name":"m"},"project":{"id":"p_1","key":"erp","name":"ERP"},"repo":{"id":"r_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[{"type":"webtransport","url":"/wt"}]}}}"#,
        )
        .unwrap();
        assert_eq!(
            handshake.capabilities.events.transports[0].kind,
            TransportKind::Unknown("webtransport".into()),
            "a newer server's transport kind never breaks parsing"
        );
    }

    #[test]
    fn binding_dtos_export_json_schema() {
        let schema = schemars::schema_for!(BindingResponse);
        let text = serde_json::to_string(&schema).expect("BindingResponse schema serializes");
        assert!(
            text.contains("apiVersion") && text.contains("engineVersion"),
            "schema fields are camelCase: {text}"
        );
    }
}

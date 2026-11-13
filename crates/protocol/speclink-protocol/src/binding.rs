//! Binding handshake shapes (blueprint §4.7 `GET /binding`): the unambiguous
//! actor/project/repo identity, version pair, and capability declarations a
//! client must obtain before any verb flows.

use crate::events::EventsDeclaration;
use crate::skills::SkillsDeclaration;
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
    /// The skills this Server gives; absent from a Server that gives none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<SkillsDeclaration>,
    /// The Server keeps free fields. Absent from an older Server, which reads
    /// as false: a client that sent `fields` with a create request then tells
    /// the user they were not kept.
    #[serde(default)]
    pub fields: bool,
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

    const WITH_SKILLS: &str = r#"{"actor":{"id":"u_1","name":"m"},"project":{"id":"p_1","key":"erp","name":"ERP"},"repo":{"id":"r_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.12.0","capabilities":{"contextSnapshots":true,"skills":{"version":"v1.58.0"}}}"#;

    #[test]
    fn the_handshake_declares_the_skill_template_version() {
        let handshake: BindingResponse = serde_json::from_str(WITH_SKILLS).unwrap();
        let skills = handshake.capabilities.skills.as_ref().expect("the declaration is kept");
        assert_eq!(skills.version, "v1.58.0");
        assert_eq!(handshake.engine_version, "0.12.0", "engineVersion keeps its meaning");
        let json = serde_json::to_value(&handshake).unwrap();
        assert_eq!(json["capabilities"]["skills"]["version"], "v1.58.0");
    }

    #[test]
    fn a_server_that_declares_no_skills_reads_as_none() {
        let handshake: BindingResponse = serde_json::from_str(
            r#"{"actor":{"id":"u_1","name":"m"},"project":{"id":"p_1","key":"erp","name":"ERP"},"repo":{"id":"r_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"contextSnapshots":true}}"#,
        )
        .unwrap();
        assert_eq!(handshake.capabilities.skills, None);
        let json = serde_json::to_value(&handshake).unwrap();
        assert!(json["capabilities"].get("skills").is_none(), "an absent declaration is not serialized: {json}");
    }

    /// The handshake as a client built before the declaration reads it.
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OldCapabilities {
        #[serde(default)]
        context_snapshots: bool,
        #[serde(default)]
        policy_write: bool,
    }

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OldBindingResponse {
        actor: Actor,
        api_version: String,
        engine_version: String,
        #[serde(default)]
        capabilities: Option<OldCapabilities>,
    }

    #[test]
    fn the_handshake_says_whether_the_server_keeps_free_fields() {
        // An older Server has no `fields` key: it reads as false.
        let old: BindingResponse = serde_json::from_str(WITH_SKILLS).unwrap();
        assert!(!old.capabilities.fields);
        let mut value: serde_json::Value = serde_json::from_str(WITH_SKILLS).unwrap();
        value["capabilities"]["fields"] = serde_json::json!(true);
        let new: BindingResponse = serde_json::from_value(value).unwrap();
        assert!(new.capabilities.fields);
        assert_eq!(serde_json::to_value(&new).unwrap()["capabilities"]["fields"], true);
    }

    #[test]
    fn a_client_that_does_not_know_the_declaration_still_reads_the_handshake() {
        let old: OldBindingResponse = serde_json::from_str(WITH_SKILLS).unwrap();
        assert_eq!(old.actor.id, "u_1");
        assert_eq!(old.api_version, "1");
        assert_eq!(old.engine_version, "0.12.0");
        let caps = old.capabilities.expect("capabilities are kept");
        assert!(caps.context_snapshots);
        assert!(!caps.policy_write);
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

//! Context API DTOs: the consistent-snapshot shapes for Agent context
//! projection (blueprint §7). This knife declares the wire shapes only;
//! materialization into `.speclink/context/` belongs to the next knife.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Snapshot request scope: narrow by change and/or flow (blueprint §7.3);
/// both absent means the full projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshotRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
}

/// One consistent snapshot: every document read at the same store state,
/// digests included so a materialized projection can be verified fail-closed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub snapshot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_revision: Option<u64>,
    pub digest: String,
    pub documents: Vec<ContextDocument>,
}

/// One document inside [`ContextSnapshot`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextDocument {
    pub path: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    pub digest: String,
}

#[cfg(test)]
mod tests {
    use crate::context::*;

    #[test]
    fn snapshot_request_omits_absent_scope() {
        let bare = ContextSnapshotRequest { change: None, flow: None };
        assert_eq!(serde_json::to_string(&bare).unwrap(), "{}");
        let scoped = ContextSnapshotRequest {
            change: Some("add-payment".into()),
            flow: Some("apply".into()),
        };
        let json = serde_json::to_value(&scoped).unwrap();
        assert_eq!(json["change"], "add-payment");
        assert_eq!(json["flow"], "apply");
        let back: ContextSnapshotRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, scoped);
    }

    #[test]
    fn snapshot_round_trips_with_documents_and_digests() {
        let snapshot: ContextSnapshot = serde_json::from_str(
            r###"{"snapshotId":"snap_01H","policyRevision":7,"digest":"sha256:abc","documents":[{"path":"openspec/changes/add-payment/proposal.md","content":"## Why\n","revision":42,"digest":"sha256:def"}]}"###,
        )
        .unwrap();
        assert_eq!(snapshot.snapshot_id, "snap_01H");
        assert_eq!(snapshot.policy_revision, Some(7));
        assert_eq!(snapshot.documents[0].revision, Some(42));
        assert_eq!(snapshot.documents[0].digest, "sha256:def");

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["snapshotId"], "snap_01H", "fields serialize camelCase: {json}");
        assert_eq!(json["policyRevision"], 7);
        let back: ContextSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(back, snapshot);
    }

    #[test]
    fn api_version_constant_exists_and_reaches_the_handshake_response() {
        assert_eq!(crate::API_VERSION, "1");
        let handshake: crate::binding::BindingResponse = serde_json::from_str(
            r#"{"actor":{"id":"u_42","name":"王小明"},"project":{"id":"prj_01H","key":"erp","name":"ERP"},"repo":{"id":"repo_01H","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"1.4.0"}"#,
        )
        .unwrap();
        assert_eq!(
            handshake.api_version,
            crate::API_VERSION,
            "the handshake response carries the protocol's version constant"
        );
    }

    #[test]
    fn context_dtos_export_json_schema() {
        for (name, schema) in [
            ("ContextSnapshotRequest", schemars::schema_for!(ContextSnapshotRequest)),
            ("ContextSnapshot", schemars::schema_for!(ContextSnapshot)),
        ] {
            let text = serde_json::to_string(&schema)
                .unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains("properties"), "{name} schema has properties");
        }
        let snapshot = serde_json::to_string(&schemars::schema_for!(ContextSnapshot)).unwrap();
        assert!(snapshot.contains("snapshotId"), "schema fields are camelCase: {snapshot}");
    }
}

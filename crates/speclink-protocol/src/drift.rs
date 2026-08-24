//! Drift API DTOs: the spec-side drift report and the basis it was computed
//! against (server-drift-api spec「規格面 drift 端點且工作區面不進 wire」).
//!
//! Only what a Server can know from a Store snapshot lives here. Workspace
//! facts — broken anchors, the git-derived dimensions, the commit window —
//! are the client's local business and are given no field to travel in: the
//! Server never runs git, so it has nothing to say about them and no way to
//! say it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The spec-side drift report plus the basis digests of the snapshot it was
/// computed at — both halves come from that one snapshot, so the report always
/// names the state it describes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecDriftResponse {
    pub spec_drift: SpecDrift,
    pub basis: DriftBasisDigests,
    pub change: DriftChangeInputs,
}

/// The change's store-side inputs that the client's workspace-side drift
/// computation reads. These are store facts, not workspace facts: the Server
/// knows them from the same snapshot, and sending them down is what lets a
/// client with no local `openspec/` compute its own half. Absence is not
/// emptiness — a missing design.md and an empty one drive different Structure
/// dimensions, so each field stays optional rather than defaulting to "".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriftChangeInputs {
    /// The change's `created` metadata — the Time dimension's input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// design.md's content — the anchor source. `None` = no design.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub design: Option<String>,
    /// tasks.md's content — the task path references. `None` = no tasks.md.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<String>,
    /// The change's completion-evidence record text (the same opaque serialized
    /// form the store holds), or absent when the change has none — what lets a
    /// checkout-side drift computation see the store-recorded touched files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// The spec side of a drift report: the Specs dimension and the stale
/// delta-spec assumptions behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecDrift {
    pub dimension: DriftDimension,
    pub spec_assumptions: Vec<SpecAssumption>,
}

/// One scored drift dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriftDimension {
    pub kind: String,
    pub status: String,
    pub score: i64,
    pub contributes_to_total: bool,
}

/// A delta-spec operation whose canonical target has drifted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpecAssumption {
    pub capability: String,
    pub operation: String,
    pub requirement: String,
    pub reason: String,
}

/// The spec/tasks/policy basis digests fixed by the snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriftBasisDigests {
    pub spec: String,
    pub tasks: String,
    pub policy: String,
}

#[cfg(test)]
mod tests {
    use crate::drift::*;

    fn sample() -> SpecDriftResponse {
        SpecDriftResponse {
            spec_drift: SpecDrift {
                dimension: DriftDimension {
                    kind: "Specs".to_string(),
                    status: "2 stale assumptions".to_string(),
                    score: 8,
                    contributes_to_total: true,
                },
                spec_assumptions: vec![
                    SpecAssumption {
                        capability: "auth".to_string(),
                        operation: "MODIFIED".to_string(),
                        requirement: "Token rotation".to_string(),
                        reason: "target requirement no longer exists in the canonical spec"
                            .to_string(),
                    },
                    SpecAssumption {
                        capability: "billing".to_string(),
                        operation: "ADDED".to_string(),
                        requirement: "Invoice export".to_string(),
                        reason: "already exists in the canonical spec — archive would refuse it"
                            .to_string(),
                    },
                ],
            },
            basis: DriftBasisDigests {
                spec: "sha256:aaa".to_string(),
                tasks: "sha256:bbb".to_string(),
                policy: "sha256:ccc".to_string(),
            },
            change: DriftChangeInputs {
                created: Some("2026-07-13".to_string()),
                design: Some("## Context\n\nUses `Widget_kind`.\n".to_string()),
                tasks: Some("- [ ] 1.1 wire `src/app.rs`\n".to_string()),
                evidence: None,
            },
        }
    }

    #[test]
    fn response_round_trips_camel_case() {
        let response = sample();
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["specDrift"]["dimension"]["kind"], "Specs");
        assert_eq!(
            json["specDrift"]["dimension"]["contributesToTotal"],
            true,
            "fields serialize camelCase: {json}"
        );
        assert_eq!(json["specDrift"]["specAssumptions"][0]["capability"], "auth");
        assert_eq!(json["basis"]["spec"], "sha256:aaa");
        assert_eq!(json["change"]["created"], "2026-07-13");
        assert_eq!(json["change"]["tasks"], "- [ ] 1.1 wire `src/app.rs`\n");

        let back: SpecDriftResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, response);
    }

    /// A missing artifact stays missing across the wire: it must not arrive as
    /// an empty string, which the Structure dimension would read as "a design
    /// exists and has no anchors" rather than "there is no design".
    #[test]
    fn absent_change_inputs_stay_absent_and_do_not_become_empty_strings() {
        let mut response = sample();
        response.change =
            DriftChangeInputs { created: None, design: None, tasks: None, evidence: None };
        let json = serde_json::to_value(&response).unwrap();

        let inputs = json["change"].as_object().expect("change is an object");
        assert!(inputs.is_empty(), "absent inputs serialize to no keys at all: {json}");

        let back: SpecDriftResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, response, "absent stays absent on the way back");
        assert_eq!(back.change.design, None, "a missing design is None, never Some(\"\")");

        // An empty design is a distinct, representable state.
        let mut empty = sample();
        empty.change.design = Some(String::new());
        let back: SpecDriftResponse =
            serde_json::from_value(serde_json::to_value(&empty).unwrap()).unwrap();
        assert_eq!(back.change.design, Some(String::new()), "an empty design survives as empty");
    }

    #[test]
    fn dtos_export_json_schema() {
        for (name, schema) in [
            ("SpecDriftResponse", schemars::schema_for!(SpecDriftResponse)),
            ("SpecDrift", schemars::schema_for!(SpecDrift)),
        ] {
            let text = serde_json::to_string(&schema)
                .unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains("properties"), "{name} schema has properties");
        }
        let text = serde_json::to_string(&schemars::schema_for!(SpecDriftResponse)).unwrap();
        assert!(text.contains("specDrift"), "schema fields are camelCase: {text}");
        assert!(text.contains("contributesToTotal"), "schema fields are camelCase: {text}");
    }

    /// The wire carries no workspace/git field — the type layer is what keeps
    /// the Server from claiming knowledge it never ran git to obtain.
    #[test]
    fn wire_has_no_workspace_or_git_field() {
        let text = serde_json::to_string(&schemars::schema_for!(SpecDriftResponse)).unwrap();
        for absent in [
            "brokenAnchors",
            "broken_anchors",
            "commitWindow",
            "trackedDocs",
            "symbolHeadHits",
            "pathStatus",
            "touchedFiles",
            "tasksMaybeResolved",
            "tasksBlockedExternal",
            "commitsSinceCreated",
            "lastCommit",
        ] {
            assert!(
                !text.contains(absent),
                "'{absent}' is a workspace-side fact and must not exist on the wire: {text}"
            );
        }
    }
}

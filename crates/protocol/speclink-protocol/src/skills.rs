//! Skill delivery shapes: the handshake's skills declaration, and the request
//! and response of `POST /projects/{key}/skills` — the skills a Server gives a
//! checkout for the tools the client names.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The handshake's `capabilities.skills`: this Server gives skills, rendered
/// from the skill templates at `version` — the value a skill file carries in
/// its frontmatter, not the package version. A handshake without it comes
/// from a Server that gives no skills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillsDeclaration {
    pub version: String,
}

/// `POST /skills` request: the tools the client installs skills for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillsRequest {
    /// At least one.
    pub targets: Vec<SkillTarget>,
    /// True asks for each skill's full text; absent or false, digests only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub contents: bool,
}

/// One tool of a [`SkillsRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillTarget {
    /// A built-in tool's name, or — with `custom` — the name of a custom tool.
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomTarget>,
}

/// What the Server must know of a custom tool to render its skills. Where the
/// tool keeps its skills is the client's setting and is not sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomTarget {
    /// `cli` or `tool-call`.
    pub invocation: String,
}

/// `POST /skills` response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillsResponse {
    /// The skill template version the contents were rendered at.
    pub version: String,
    /// One per requested target, in the request's order.
    pub targets: Vec<TargetSkills>,
}

/// The skills of one requested tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TargetSkills {
    pub tool: String,
    /// Sorted by name.
    pub skills: Vec<SkillEntry>,
}

/// One skill: its name (`apply`, without the `speclink-` prefix of its
/// folder), the digest of its text, and the text when it was asked for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SkillEntry {
    pub name: String,
    /// `sha256:` and the hex SHA-256 of the text with its line ends as LF.
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::skills::*;

    #[test]
    fn a_request_serializes_camel_case_and_leaves_out_absent_fields() {
        let request = SkillsRequest {
            targets: vec![
                SkillTarget { tool: "claude".into(), custom: None },
                SkillTarget { tool: "my-agent".into(), custom: Some(CustomTarget { invocation: "tool-call".into() }) },
            ],
            contents: true,
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "targets": [{ "tool": "claude" }, { "tool": "my-agent", "custom": { "invocation": "tool-call" } }],
                "contents": true
            })
        );
        let back: SkillsRequest = serde_json::from_value(json).unwrap();
        assert_eq!(back, request);
    }

    #[test]
    fn contents_reads_as_false_when_it_is_absent() {
        let request: SkillsRequest = serde_json::from_str(r#"{"targets":[{"tool":"claude"}]}"#).unwrap();
        assert!(!request.contents);
        assert_eq!(request.targets[0].custom, None);
        assert_eq!(
            serde_json::to_string(&request).unwrap(),
            r#"{"targets":[{"tool":"claude"}]}"#,
            "a digest-only request does not serialize `contents`"
        );
    }

    #[test]
    fn a_response_serializes_camel_case_and_content_only_when_it_is_there() {
        let response = SkillsResponse {
            version: "v1.58.0".into(),
            targets: vec![TargetSkills {
                tool: "claude".into(),
                skills: vec![
                    SkillEntry { name: "apply".into(), digest: "sha256:ab".into(), content: None },
                    SkillEntry { name: "discuss".into(), digest: "sha256:cd".into(), content: Some("body\n".into()) },
                ],
            }],
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "version": "v1.58.0",
                "targets": [{
                    "tool": "claude",
                    "skills": [
                        { "name": "apply", "digest": "sha256:ab" },
                        { "name": "discuss", "digest": "sha256:cd", "content": "body\n" }
                    ]
                }]
            })
        );
        let back: SkillsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(back, response);
    }

    #[test]
    fn skills_dtos_export_json_schema() {
        for (name, schema, field) in [
            ("SkillsRequest", schemars::schema_for!(SkillsRequest), "contents"),
            ("SkillsResponse", schemars::schema_for!(SkillsResponse), "digest"),
            ("SkillsDeclaration", schemars::schema_for!(SkillsDeclaration), "version"),
        ] {
            let text = serde_json::to_string(&schema).unwrap_or_else(|e| panic!("{name} schema must serialize: {e}"));
            assert!(text.contains(field), "{name} schema names `{field}`: {text}");
        }
    }
}

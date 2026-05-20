//! Runtime 編排：`get_instructions` — `speclink instructions` 工作流的純編排層，
//! 與 local provider 共用的 `compose_local_instructions` helper。
//!
//! `get_instructions` 純轉發 `provider.get_artifact_instructions`；`compose_local_instructions`
//! 則負責讀取 runtime 內嵌的 hardcoded markdown、解析為 [`ArtifactInstructions`] 結構，
//! 並套用固定的 dependencies / unlocks / output_path 規則。

use provider::Provider;
use provider::model::{
    ArtifactInstructions, ArtifactKind, ChangeId, InstructionRule, ProjectId, RuleLevel,
};
use std::sync::Arc;
use thiserror::Error;

use crate::propose::RuntimeError;

/// 內嵌四個 hardcoded instruction markdown。
static PROPOSAL_INSTRUCTION: &str = include_str!("../instructions/proposal.md");
static DESIGN_INSTRUCTION: &str = include_str!("../instructions/design.md");
static TASKS_INSTRUCTION: &str = include_str!("../instructions/tasks.md");
static SPEC_INSTRUCTION: &str = include_str!("../instructions/spec.md");

/// `get_instructions` 的輸入。
#[derive(Debug, Clone)]
pub struct InstructionsInput {
    /// 專案識別碼。
    pub project_id: ProjectId,
    /// Change 識別碼。
    pub change_id: ChangeId,
    /// Artifact 種類。
    pub kind: ArtifactKind,
    /// Capability 名稱（僅 `kind == Spec` 時填）。
    pub capability: Option<String>,
}

/// 純轉發 `provider.get_artifact_instructions`。
pub async fn get_instructions(
    provider: Arc<dyn Provider>,
    input: InstructionsInput,
) -> Result<ArtifactInstructions, RuntimeError> {
    provider
        .get_artifact_instructions(
            &input.project_id,
            &input.change_id,
            input.kind,
            input.capability.as_deref(),
        )
        .await
        .map_err(RuntimeError::Provider)
}

/// `compose_local_instructions` 的錯誤型別。
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum InstructionsError {
    /// 內嵌 markdown 缺 `## Instruction` / `## Template` / `## Rules` 任一 section。
    #[error("hardcoded instruction missing section '{section}'")]
    MissingSection {
        /// 缺少的 H2 heading 名稱。
        section: &'static str,
    },
    /// `## Rules` 區段內出現無法解析為 `- [level] id: description` 的行。
    #[error("hardcoded rule line malformed: '{line}'")]
    MalformedRule {
        /// 觸發錯誤的原始行。
        line: String,
    },
    /// kind 為 Spec 但未提供 capability。
    #[error("spec instructions require capability")]
    MissingCapability,
}

/// 依 kind 與 capability 組裝 [`ArtifactInstructions`]。
///
/// 從 runtime 內嵌 markdown 取對應 `Instruction` / `Template` / `Rules` 三段內容，
/// 套用固定 dependencies / unlocks / output_path 規則：
///
/// | kind     | dependencies            | unlocks                              |
/// |----------|-------------------------|--------------------------------------|
/// | proposal | `[]`                    | `["design", "tasks", "spec"]`        |
/// | design   | `["proposal"]`          | `["tasks"]`                          |
/// | tasks    | `["proposal", "spec"]`  | `[]`                                 |
/// | spec     | `["proposal"]`          | `["tasks"]`                          |
///
/// `change_id` 用於組裝 `output_path`（POSIX 風格相對於 base）。
pub fn compose_local_instructions(
    kind: ArtifactKind,
    change_id: &str,
    capability: Option<&str>,
) -> Result<ArtifactInstructions, InstructionsError> {
    let raw = match kind {
        ArtifactKind::Proposal => PROPOSAL_INSTRUCTION,
        ArtifactKind::Design => DESIGN_INSTRUCTION,
        ArtifactKind::Tasks => TASKS_INSTRUCTION,
        ArtifactKind::Spec => SPEC_INSTRUCTION,
    };
    let parsed = parse_sections(raw)?;
    let rules = parse_rules(&parsed.rules_block)?;

    let (artifact_id, output_path, dependencies, unlocks) = match kind {
        ArtifactKind::Proposal => (
            "proposal".to_string(),
            format!(".speclink/changes/{change_id}/proposal.md"),
            Vec::<String>::new(),
            vec![
                "design".to_string(),
                "tasks".to_string(),
                "spec".to_string(),
            ],
        ),
        ArtifactKind::Design => (
            "design".to_string(),
            format!(".speclink/changes/{change_id}/design.md"),
            vec!["proposal".to_string()],
            vec!["tasks".to_string()],
        ),
        ArtifactKind::Tasks => (
            "tasks".to_string(),
            format!(".speclink/changes/{change_id}/tasks.md"),
            vec!["proposal".to_string(), "spec".to_string()],
            Vec::new(),
        ),
        ArtifactKind::Spec => {
            let cap = capability.ok_or(InstructionsError::MissingCapability)?;
            (
                format!("spec:{cap}"),
                format!(".speclink/changes/{change_id}/specs/{cap}/spec.md"),
                vec!["proposal".to_string()],
                vec!["tasks".to_string()],
            )
        }
    };

    Ok(ArtifactInstructions {
        artifact_id,
        kind,
        output_path,
        dependencies,
        unlocks,
        instruction: parsed.instruction,
        template: parsed.template,
        rules,
        locale: "Traditional Chinese (繁體中文)".to_string(),
    })
}

struct ParsedSections {
    instruction: String,
    template: String,
    rules_block: String,
}

/// 切出 `## Instruction` / `## Template` / `## Rules` 三段。
fn parse_sections(content: &str) -> Result<ParsedSections, InstructionsError> {
    let mut instruction: Option<String> = None;
    let mut template: Option<String> = None;
    let mut rules: Option<String> = None;
    let mut current_heading: Option<&'static str> = None;
    let mut current_buf: String = String::new();

    fn flush(
        current_heading: &mut Option<&'static str>,
        current_buf: &mut String,
        instruction: &mut Option<String>,
        template: &mut Option<String>,
        rules: &mut Option<String>,
    ) {
        if let Some(h) = current_heading.take() {
            let value = std::mem::take(current_buf).trim().to_string();
            match h {
                "Instruction" => *instruction = Some(value),
                "Template" => *template = Some(value),
                "Rules" => *rules = Some(value),
                _ => unreachable!(),
            }
        }
    }

    for line in content.lines() {
        if let Some(stripped) = line.strip_prefix("## ") {
            let heading_key: Option<&'static str> = match stripped.trim() {
                "Instruction" => Some("Instruction"),
                "Template" => Some("Template"),
                "Rules" => Some("Rules"),
                _ => None,
            };
            if heading_key.is_some() {
                flush(
                    &mut current_heading,
                    &mut current_buf,
                    &mut instruction,
                    &mut template,
                    &mut rules,
                );
                current_heading = heading_key;
                continue;
            }
        }
        if current_heading.is_some() {
            current_buf.push_str(line);
            current_buf.push('\n');
        }
    }
    flush(
        &mut current_heading,
        &mut current_buf,
        &mut instruction,
        &mut template,
        &mut rules,
    );

    let instruction = instruction.ok_or(InstructionsError::MissingSection {
        section: "Instruction",
    })?;
    let template = template.ok_or(InstructionsError::MissingSection {
        section: "Template",
    })?;
    let rules_block = rules.ok_or(InstructionsError::MissingSection { section: "Rules" })?;
    Ok(ParsedSections {
        instruction,
        template,
        rules_block,
    })
}

/// 解析 rules 段：每行格式 `- [level] id: description`。
fn parse_rules(block: &str) -> Result<Vec<InstructionRule>, InstructionsError> {
    let mut out = Vec::new();
    for raw in block.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let after_dash =
            line.strip_prefix("- ")
                .ok_or_else(|| InstructionsError::MalformedRule {
                    line: raw.to_string(),
                })?;
        let rest =
            after_dash
                .strip_prefix('[')
                .ok_or_else(|| InstructionsError::MalformedRule {
                    line: raw.to_string(),
                })?;
        let (level_str, rest) =
            rest.split_once(']')
                .ok_or_else(|| InstructionsError::MalformedRule {
                    line: raw.to_string(),
                })?;
        let level = match level_str.trim() {
            "error" => RuleLevel::Error,
            "warning" => RuleLevel::Warning,
            "info" => RuleLevel::Info,
            _ => {
                return Err(InstructionsError::MalformedRule {
                    line: raw.to_string(),
                });
            }
        };
        let body = rest
            .trim_start()
            .strip_prefix(|c: char| c.is_whitespace() || c == ':')
            .map(|s| s.trim())
            .unwrap_or(rest.trim());
        let (id, description) =
            body.split_once(':')
                .ok_or_else(|| InstructionsError::MalformedRule {
                    line: raw.to_string(),
                })?;
        out.push(InstructionRule {
            id: id.trim().to_string(),
            level,
            description: description.trim().to_string(),
        });
    }
    if out.is_empty() {
        return Err(InstructionsError::MissingSection { section: "Rules" });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propose::RuntimeError;
    use async_trait::async_trait;
    use provider::Provider;
    use provider::error::ProviderError;
    use provider::model::{
        ArchiveOptions, ArchivedChange, Artifact, Change, ChangeStatus, NewArtifact, NewChange,
        TaskUpdate,
    };
    use std::sync::Arc;

    // -- compose_local_instructions (task 8.6) --

    #[test]
    fn compose_proposal_non_empty() {
        let ai = compose_local_instructions(ArtifactKind::Proposal, "demo", None).expect("ok");
        assert_eq!(ai.artifact_id, "proposal");
        assert_eq!(ai.kind, ArtifactKind::Proposal);
        assert_eq!(ai.output_path, ".speclink/changes/demo/proposal.md");
        assert!(ai.dependencies.is_empty());
        assert_eq!(ai.unlocks, vec!["design", "tasks", "spec"]);
        assert!(!ai.instruction.is_empty());
        assert!(!ai.template.is_empty());
        assert!(!ai.rules.is_empty());
        assert_eq!(ai.locale, "Traditional Chinese (繁體中文)");
    }

    #[test]
    fn compose_design_dependencies() {
        let ai = compose_local_instructions(ArtifactKind::Design, "demo", None).expect("ok");
        assert_eq!(ai.artifact_id, "design");
        assert_eq!(ai.dependencies, vec!["proposal"]);
        assert_eq!(ai.unlocks, vec!["tasks"]);
        assert_eq!(ai.output_path, ".speclink/changes/demo/design.md");
        assert!(!ai.rules.is_empty());
    }

    #[test]
    fn compose_tasks_dependencies() {
        let ai = compose_local_instructions(ArtifactKind::Tasks, "demo", None).expect("ok");
        assert_eq!(ai.artifact_id, "tasks");
        assert_eq!(ai.dependencies, vec!["proposal", "spec"]);
        assert!(ai.unlocks.is_empty());
        assert_eq!(ai.output_path, ".speclink/changes/demo/tasks.md");
    }

    #[test]
    fn compose_spec_with_capability() {
        let ai =
            compose_local_instructions(ArtifactKind::Spec, "demo", Some("user-auth")).expect("ok");
        assert_eq!(ai.artifact_id, "spec:user-auth");
        assert_eq!(
            ai.output_path,
            ".speclink/changes/demo/specs/user-auth/spec.md"
        );
        assert_eq!(ai.dependencies, vec!["proposal"]);
        assert_eq!(ai.unlocks, vec!["tasks"]);
    }

    #[test]
    fn compose_spec_missing_capability_is_error() {
        let err = compose_local_instructions(ArtifactKind::Spec, "demo", None).expect_err("err");
        assert!(matches!(err, InstructionsError::MissingCapability));
    }

    // -- get_instructions forwarding (task 7.5) --

    #[derive(Default)]
    struct MockProvider {
        not_found: bool,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn create_change(
            &self,
            _project_id: &ProjectId,
            _input: NewChange,
        ) -> Result<Change, ProviderError> {
            unimplemented!()
        }

        async fn write_artifact(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _input: NewArtifact,
        ) -> Result<Artifact, ProviderError> {
            unimplemented!()
        }

        async fn get_change(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
        ) -> Result<Change, ProviderError> {
            unimplemented!()
        }

        async fn get_status(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
        ) -> Result<ChangeStatus, ProviderError> {
            unimplemented!()
        }

        async fn archive_change(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _options: ArchiveOptions,
        ) -> Result<ArchivedChange, ProviderError> {
            unimplemented!()
        }

        async fn get_artifact_instructions(
            &self,
            _project_id: &ProjectId,
            change_id: &ChangeId,
            kind: ArtifactKind,
            _capability: Option<&str>,
        ) -> Result<ArtifactInstructions, ProviderError> {
            if self.not_found {
                return Err(ProviderError::ChangeNotFound {
                    change_id: change_id.clone(),
                });
            }
            Ok(ArtifactInstructions {
                artifact_id: "design".to_string(),
                kind,
                output_path: format!(".speclink/changes/{}/design.md", change_id.as_str()),
                dependencies: vec!["proposal".to_string()],
                unlocks: vec!["tasks".to_string()],
                instruction: "stub".to_string(),
                template: "## stub\n".to_string(),
                rules: vec![InstructionRule {
                    id: "stub.id".to_string(),
                    level: RuleLevel::Error,
                    description: "stub".to_string(),
                }],
                locale: "Traditional Chinese (繁體中文)".to_string(),
            })
        }

        async fn mark_task_done(
            &self,
            _project_id: &ProjectId,
            _change_id: &ChangeId,
            _task_id: &str,
        ) -> Result<TaskUpdate, ProviderError> {
            unimplemented!()
        }
    }

    fn input(kind: ArtifactKind) -> InstructionsInput {
        InstructionsInput {
            project_id: ProjectId::from("p"),
            change_id: ChangeId::from("demo"),
            kind,
            capability: None,
        }
    }

    #[tokio::test]
    async fn get_instructions_happy_path() {
        let provider: Arc<dyn Provider> = Arc::new(MockProvider::default());
        let out = get_instructions(provider, input(ArtifactKind::Design))
            .await
            .expect("ok");
        assert_eq!(out.artifact_id, "design");
        assert_eq!(out.kind, ArtifactKind::Design);
    }

    #[tokio::test]
    async fn get_instructions_change_not_found_propagates() {
        let provider: Arc<dyn Provider> = Arc::new(MockProvider { not_found: true });
        let err = get_instructions(provider, input(ArtifactKind::Design))
            .await
            .expect_err("err");
        assert!(matches!(
            err,
            RuntimeError::Provider(ProviderError::ChangeNotFound { .. })
        ));
    }
}

//! Embedded schema bundle compiled into the SpecLink runtime binary.
//!
//! 把 `spec-driven` schema 的 markdown 資產（schema.yaml、4 個 artifact
//! template、8 個 instruction body）透過 `include_str!` 編進 binary。
//! `instructions_ops::run` 從此模組取 template / instruction body；MVP 不走
//! filesystem lazy load、不支援 user override（Phase 2 `add-schema-ops` 才開）。
//!
//! 設計參考：
//! - `doc/speclink-design.md` §10 — Schema 抽象
//! - `doc/speclink-design.md` §18.4 — Phase 1 P1-3 `add-instructions-get`
//! - 同模組目錄下 `schemas/spec-driven/schema.yaml` — 8 個 kind 的 DAG 描述
//!
//! Asset bundle layout：
//! ```text
//! schemas/spec-driven/
//!   schema.yaml                       (1 file — DAG descriptor)
//!   templates/{proposal,spec,design,tasks}.md  (4 files — artifact skeletons)
//!   instructions/{proposal,spec,design,tasks,
//!                 apply,ingest,archive,commit}.md  (8 files — AI prompts)
//! ```

#![allow(clippy::doc_markdown)]

/// `spec-driven` schema 的 DAG descriptor。
///
/// runtime 不解析此檔案做 dispatch（dispatch 走 `instructions_ops::Kind`
/// 內的硬表）；本 const 只供 sync test 比對硬表與 schema.yaml 一致。
pub const EMBEDDED_SCHEMA_YAML: &str = include_str!("schemas/spec-driven/schema.yaml");

/// 4 個 artifact kind 的 markdown template skeleton。
///
/// `instructions_ops::run` 對 artifact kinds (proposal/spec/design/tasks)
/// 回傳 `template = Some(EMBEDDED_TEMPLATES["<kind>"])`；對 workflow phase
/// kinds (apply/ingest/archive/commit) 回 `template = None`。
pub const EMBEDDED_TEMPLATES: &[(&str, &str)] = &[
    (
        "proposal",
        include_str!("schemas/spec-driven/templates/proposal.md"),
    ),
    (
        "spec",
        include_str!("schemas/spec-driven/templates/spec.md"),
    ),
    (
        "design",
        include_str!("schemas/spec-driven/templates/design.md"),
    ),
    (
        "tasks",
        include_str!("schemas/spec-driven/templates/tasks.md"),
    ),
];

/// 8 個 kind 的 instruction body（給 AI 看的 prompt）。
///
/// 涵蓋 4 個 artifact kinds + 4 個 workflow phase kinds。`instructions_ops::run`
/// 對任何支援 kind 都會回對應的 instruction body。
pub const EMBEDDED_INSTRUCTIONS: &[(&str, &str)] = &[
    (
        "proposal",
        include_str!("schemas/spec-driven/instructions/proposal.md"),
    ),
    (
        "spec",
        include_str!("schemas/spec-driven/instructions/spec.md"),
    ),
    (
        "design",
        include_str!("schemas/spec-driven/instructions/design.md"),
    ),
    (
        "tasks",
        include_str!("schemas/spec-driven/instructions/tasks.md"),
    ),
    (
        "apply",
        include_str!("schemas/spec-driven/instructions/apply.md"),
    ),
    (
        "ingest",
        include_str!("schemas/spec-driven/instructions/ingest.md"),
    ),
    (
        "archive",
        include_str!("schemas/spec-driven/instructions/archive.md"),
    ),
    (
        "commit",
        include_str!("schemas/spec-driven/instructions/commit.md"),
    ),
];

/// 查 template by kind（kebab-case key）。Artifact kinds 之外回 `None`。
#[must_use]
pub fn template_for(kind: &str) -> Option<&'static str> {
    EMBEDDED_TEMPLATES
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
}

/// 查 instruction body by kind（kebab-case key）。8 個支援 kind 之外回 `None`。
#[must_use]
pub fn instruction_for(kind: &str) -> Option<&'static str> {
    EMBEDDED_INSTRUCTIONS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_assets_nonempty() {
        // 13 個 const 都 `!is_empty()`：1 schema.yaml + 4 templates + 8 instructions。
        assert!(!EMBEDDED_SCHEMA_YAML.is_empty(), "schema.yaml is empty");
        for (kind, body) in EMBEDDED_TEMPLATES {
            assert!(!body.is_empty(), "template {} is empty", kind);
        }
        for (kind, body) in EMBEDDED_INSTRUCTIONS {
            assert!(!body.is_empty(), "instruction {} is empty", kind);
        }
    }

    #[test]
    fn templates_cover_four_artifact_kinds() {
        let kinds: Vec<&str> = EMBEDDED_TEMPLATES.iter().map(|(k, _)| *k).collect();
        assert_eq!(kinds, vec!["proposal", "spec", "design", "tasks"]);
    }

    #[test]
    fn instructions_cover_eight_kinds() {
        let kinds: Vec<&str> = EMBEDDED_INSTRUCTIONS.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                "proposal", "spec", "design", "tasks", "apply", "ingest", "archive", "commit",
            ]
        );
    }

    #[test]
    fn artifact_templates_contain_expected_sections() {
        // 對齊 design.md Risk Mitigation: smoke check 防 markdown stale。
        assert!(template_for("proposal").unwrap().contains("## Why"));
        assert!(
            template_for("spec")
                .unwrap()
                .contains("## ADDED Requirements")
        );
        assert!(template_for("design").unwrap().contains("## Context"));
        assert!(template_for("tasks").unwrap().contains("## 1."));
    }

    #[test]
    fn instruction_bodies_have_kind_header_and_minimum_length() {
        for (kind, body) in EMBEDDED_INSTRUCTIONS {
            let first_line = body.lines().next().unwrap_or("");
            assert_eq!(
                first_line,
                format!("# Instructions: {kind}"),
                "instruction {kind} first line mismatch"
            );
            assert!(
                body.len() > 100,
                "instruction {kind} body too short ({} bytes)",
                body.len()
            );
        }
    }

    #[test]
    fn workflow_phase_instructions_declare_no_artifact() {
        // apply/ingest/archive/commit 必須明確標示非 artifact-producing。
        let phase_kinds = ["apply", "ingest", "archive", "commit"];
        for k in phase_kinds {
            let body = instruction_for(k).unwrap_or_else(|| panic!("no instruction for {k}"));
            assert!(
                body.contains("workflow phase, not an artifact-producing step"),
                "instruction {k} missing phase-not-artifact declaration"
            );
        }
    }

    #[test]
    fn template_for_unknown_kind_returns_none() {
        assert!(template_for("apply").is_none());
        assert!(template_for("nonexistent").is_none());
        assert!(template_for("").is_none());
    }

    #[test]
    fn instruction_for_unknown_kind_returns_none() {
        assert!(instruction_for("discuss").is_none());
        assert!(instruction_for("nonexistent").is_none());
        assert!(instruction_for("").is_none());
    }
}

//! Structural validation of changes and specs.

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::util;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub change: String,
    pub errors: Vec<String>,
    pub valid: bool,
    pub warnings: Vec<String>,
}

/// Validate a change's artifacts structurally.
pub fn validate_change(change: &Change, _schema: &Schema, strict: bool) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Proposal
    let proposal_path = change.dir.join("proposal.md");
    match util::read_opt(&proposal_path) {
        Some(text) if !text.trim().is_empty() => {
            let has_header = ["## Why", "## Problem", "## Summary"]
                .iter()
                .any(|h| contains_header(&text, h));
            if !has_header {
                errors.push(
                    "proposal.md must contain one of: ## Why, ## Problem, ## Summary".to_string(),
                );
            }
        }
        _ => errors.push("proposal.md is missing or empty".to_string()),
    }

    // Design (optional)
    let design_path = change.dir.join("design.md");
    if let Some(text) = util::read_opt(&design_path) {
        if !text.trim().is_empty() && !contains_header(&text, "## Context") {
            errors.push("design.md must contain ## Context".to_string());
        }
    }

    // Tasks (optional but validated if present)
    let tasks_path = change.dir.join("tasks.md");
    if let Some(text) = util::read_opt(&tasks_path) {
        if !text.trim().is_empty() {
            let has_checkbox = text.lines().any(|l| {
                let t = l.trim_start();
                t.starts_with("- [ ] ") || t.starts_with("- [x] ") || t.starts_with("- [X] ")
            });
            if !has_checkbox {
                errors.push("tasks.md must contain at least one - [ ] checkbox".to_string());
            }
        }
    }

    // Delta specs
    for spec_path in model::spec_files(&change.dir) {
        let rel = spec_path
            .strip_prefix(&change.dir)
            .map(|p| util::to_slash(p))
            .unwrap_or_else(|_| util::to_slash(&spec_path));
        let Some(text) = util::read_opt(&spec_path) else {
            continue;
        };
        validate_delta_spec(&rel, &text, &mut errors, &mut warnings, strict);
    }

    let valid = errors.is_empty();
    ValidationResult {
        change: change.name.clone(),
        errors,
        valid,
        warnings,
    }
}

const FORBIDDEN_WORDS: &[&str] = &[
    "TBD", "TODO", "???", "TKTK",
];

fn validate_delta_spec(
    rel: &str,
    text: &str,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    strict: bool,
) {
    let has_operation = ["## ADDED Requirements", "## MODIFIED Requirements", "## REMOVED Requirements", "## RENAMED Requirements"]
        .iter()
        .any(|op| text.contains(op));
    if !has_operation {
        errors.push(format!(
            "{rel}: delta spec must contain at least one operation (ADDED/MODIFIED/REMOVED/RENAMED Requirements)"
        ));
        return;
    }
    // Each requirement must have at least one scenario.
    let mut current_req: Option<String> = None;
    let mut scenario_count = 0usize;
    let mut req_had_scenario = true;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("### Requirement:") {
            if let Some(req) = &current_req {
                if !req_had_scenario {
                    errors.push(format!("{rel}: requirement '{}' has no scenario", req.trim()));
                }
            }
            current_req = Some(name.trim().to_string());
            req_had_scenario = false;
        } else if line.trim_start().starts_with("#### Scenario:") {
            scenario_count += 1;
            req_had_scenario = true;
        }
    }
    if let Some(req) = &current_req {
        if !req_had_scenario {
            errors.push(format!("{rel}: requirement '{}' has no scenario", req.trim()));
        }
    }
    let _ = scenario_count;

    if strict {
        for word in FORBIDDEN_WORDS {
            if text.contains(word) {
                warnings.push(format!("{rel}: contains forbidden word '{word}'"));
            }
        }
    }
}

fn contains_header(text: &str, header: &str) -> bool {
    text.lines().any(|l| l.trim_end() == header || l.trim_start().starts_with(&format!("{header} ")))
}

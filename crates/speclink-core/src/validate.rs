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

    // Spectra's validate is lenient: a missing proposal is NOT an error, and a scenario-less
    // requirement is NOT an error. The one hard error is a delta spec file that contains
    // requirements but no operation section (a malformed delta). A change with no delta operations
    // at all produces the informational "No delta specs found" warning.
    let specs = model::spec_files(&change.dir);
    let mut any_operation = false;
    let mut had_malformed = false;
    for spec_path in &specs {
        let Some(text) = util::read_opt(spec_path) else {
            continue;
        };
        if model::has_delta_operation(&text) {
            any_operation = true;
        } else if text.contains("### Requirement:") {
            errors.push(format!(
                "{}: Parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)",
                spec_path.to_string_lossy()
            ));
            had_malformed = true;
        }
    }
    if !any_operation && !had_malformed {
        warnings.push("No delta specs found".to_string());
    }
    let _ = strict;

    let valid = errors.is_empty();
    ValidationResult {
        change: change.name.clone(),
        errors,
        valid,
        warnings,
    }
}


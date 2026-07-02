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
    // requirement is NOT an error. The one hard error is an EXISTING delta spec file that parses
    // to zero applied operations (empty, RENAMED-only, or an operation-less requirement). The
    // informational "No delta specs found" warning fires only when there is not even a capability
    // directory under specs/.
    let specs = model::spec_files(&change.dir);
    for spec_path in &specs {
        let text = util::read_opt(spec_path).unwrap_or_default();
        if !model::has_delta_operation(&text) {
            errors.push(format!(
                "{}: Parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)",
                spec_path.to_string_lossy()
            ));
        }
    }
    let has_cap_dirs = std::fs::read_dir(change.dir.join("specs"))
        .map(|it| it.flatten().any(|e| e.path().is_dir()))
        .unwrap_or(false);
    if specs.is_empty() && !has_cap_dirs {
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


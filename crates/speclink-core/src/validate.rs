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
        // Duplicate requirement names are hard errors (probed against Spectra): the same
        // name twice inside one ADDED/MODIFIED/REMOVED section, or the same name across
        // two different sections. Reported with the change-relative path.
        let rel = spec_path
            .strip_prefix(&change.dir)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| spec_path.to_string_lossy().to_string());
        let mut section = "";
        let mut seen_in: Vec<(String, Vec<&str>)> = Vec::new(); // name -> sections (ordered)
        let mut reported_dup: Vec<(String, &str)> = Vec::new();
        let mut reported_cross: Vec<String> = Vec::new();
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("## ") {
                let head = rest.split_whitespace().next().unwrap_or("");
                if rest.trim_end().ends_with("Requirements")
                    && matches!(head, "ADDED" | "MODIFIED" | "REMOVED")
                {
                    section = match head {
                        "ADDED" => "ADDED",
                        "MODIFIED" => "MODIFIED",
                        _ => "REMOVED",
                    };
                } else {
                    section = "";
                }
            } else if let Some(name) = t.strip_prefix("### Requirement:") {
                if section.is_empty() {
                    continue;
                }
                let name = name.trim().to_string();
                let entry = match seen_in.iter_mut().find(|(n, _)| *n == name) {
                    Some(e) => e,
                    None => {
                        seen_in.push((name.clone(), Vec::new()));
                        seen_in.last_mut().unwrap()
                    }
                };
                if entry.1.contains(&section) {
                    if !reported_dup.iter().any(|(n, s)| *n == name && *s == section) {
                        errors.push(format!(
                            "{rel}: Duplicate requirement '{name}' in {section} section"
                        ));
                        reported_dup.push((name.clone(), section));
                    }
                } else {
                    if let Some(first) = entry.1.first() {
                        if !reported_cross.contains(&name) {
                            errors.push(format!(
                                "{rel}: Requirement '{name}' appears in both {first} and {section} sections"
                            ));
                            reported_cross.push(name.clone());
                        }
                    }
                    entry.1.push(section);
                }
            }
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


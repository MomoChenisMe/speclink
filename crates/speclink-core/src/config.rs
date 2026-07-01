//! Configuration: `.speclink.yaml` (app) and `openspec/config.yaml` (workflow).

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// `.speclink.yaml` — application configuration.
///
/// Only the fields speclink supports are modeled; unknown keys are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    pub spec_dir: Option<String>,
    pub locale: Option<String>,
    #[serde(default)]
    pub tdd: bool,
    #[serde(default)]
    pub audit: bool,
    #[serde(default)]
    pub tools: Vec<String>,
}

impl AppConfig {
    /// Load from a `.speclink.yaml` path. Missing file or parse error → defaults.
    pub fn load(path: &Path) -> AppConfig {
        match std::fs::read_to_string(path) {
            Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
            Err(_) => AppConfig::default(),
        }
    }

    /// Human-readable locale name for instruction injection.
    pub fn locale_display(&self) -> String {
        locale_display(self.locale.as_deref())
    }
}

/// Map a locale code to its human-readable name (matches Spectra).
///
/// Matching is case-SENSITIVE. Only `ja`/`tw`/`en` (and no locale) are mapped; any other code is
/// echoed back verbatim, exactly like Spectra.
pub fn locale_display(code: Option<&str>) -> String {
    match code.map(|c| c.trim()) {
        None => "English".to_string(),
        Some("") => "English".to_string(),
        Some("en") => "English".to_string(),
        Some("ja") => "Japanese (日本語)".to_string(),
        Some("tw") => "Traditional Chinese (繁體中文)".to_string(),
        Some(other) => other.to_string(),
    }
}

/// `openspec/config.yaml` — workflow configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkflowConfig {
    pub schema: Option<String>,
    pub context: Option<String>,
    pub locale: Option<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,
}

/// Resolve the effective locale display name: the app-level `.speclink.yaml` locale wins, with the
/// `openspec/config.yaml` locale as a fallback (matches Spectra).
pub fn resolve_locale(app: &AppConfig, wf: &WorkflowConfig) -> String {
    let code = app
        .locale
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| wf.locale.as_deref().filter(|s| !s.trim().is_empty()));
    locale_display(code)
}

impl WorkflowConfig {
    pub fn load(path: &Path) -> WorkflowConfig {
        match std::fs::read_to_string(path) {
            Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
            Err(_) => WorkflowConfig::default(),
        }
    }

    pub fn schema_name(&self) -> String {
        self.schema
            .clone()
            .unwrap_or_else(|| "spec-driven".to_string())
    }

    /// Context text with trailing whitespace trimmed, or None if empty.
    pub fn context_text(&self) -> Option<String> {
        self.context
            .as_ref()
            .map(|c| c.trim_end().to_string())
            .filter(|c| !c.is_empty())
    }

    /// Rules for a specific artifact id, or None if absent/empty.
    pub fn rules_for(&self, artifact: &str) -> Option<Vec<String>> {
        self.rules
            .get(artifact)
            .filter(|v| !v.is_empty())
            .cloned()
    }
}

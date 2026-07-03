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
    /// Language for spec files (specs/*/spec.md). Unset → English; "auto" → follow `locale`;
    /// any locale code → that language. Consumed by the skills (like `tdd`/`audit`).
    #[serde(default)]
    pub spec_locale: Option<String>,
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
    // No trimming and no case folding: Spectra preserves any unmapped value verbatim (including
    // empty/whitespace and case-variants like "JA").
    match code {
        None => "English".to_string(),
        Some("en") => "English".to_string(),
        Some("ja") => "Japanese (日本語)".to_string(),
        Some("tw") => "Traditional Chinese (繁體中文)".to_string(),
        Some(other) => other.to_string(),
    }
}

/// Machine-level speclink directory (global config, user schemas), by OS convention:
/// Windows `%USERPROFILE%\AppData\Roaming` (derived from the profile — Spectra ignores a
/// redirected APPDATA env var, probed), macOS `~/Library/Application Support`,
/// Linux `$XDG_CONFIG_HOME`|`~/.config`.
pub fn global_config_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    let base = if cfg!(windows) {
        std::env::var("USERPROFILE")
            .map(|h| PathBuf::from(h).join("AppData").join("Roaming"))
            .ok()
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .ok()
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")).ok())
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("speclink")
}

/// `openspec/config.yaml` — workflow configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WorkflowConfig {
    pub schema: Option<String>,
    pub context: Option<String>,
    pub locale: Option<String>,
    #[serde(default)]
    pub spec_locale: Option<String>,
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,
}

/// Resolve the effective locale display name: the app-level `.speclink.yaml` locale wins, with the
/// `openspec/config.yaml` locale as a fallback (matches Spectra).
pub fn resolve_locale(app: &AppConfig, wf: &WorkflowConfig) -> String {
    // App-level locale wins when the key is present at all (even if empty); otherwise fall back to
    // the workflow-level locale. Values are passed through verbatim (see `locale_display`).
    let code = app.locale.as_deref().or_else(|| wf.locale.as_deref());
    locale_display(code)
}

/// Resolve the effective spec-file language: `.speclink.yaml` wins over `openspec/config.yaml`.
/// Unset / empty / "en" / "english" → `None` (specs default to English); `"auto"` follows the
/// project locale (again `None` when that resolves to English).
pub fn resolve_spec_locale(app: &AppConfig, wf: &WorkflowConfig) -> Option<String> {
    let code = app
        .spec_locale
        .as_deref()
        .or_else(|| wf.spec_locale.as_deref())?
        .trim()
        .to_string();
    let code = if code.eq_ignore_ascii_case("auto") {
        app.locale
            .as_deref()
            .or_else(|| wf.locale.as_deref())?
            .trim()
            .to_string()
    } else {
        code
    };
    if code.is_empty() || code.eq_ignore_ascii_case("en") || code.eq_ignore_ascii_case("english") {
        return None;
    }
    Some(code)
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

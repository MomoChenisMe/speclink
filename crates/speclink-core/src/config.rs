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
    /// Deprecated policy keys (canonical home: `openspec/config.yaml`). `Option` so the
    /// compat layer can tell "key present" (old key wins) from "key absent" (fall through).
    #[serde(default)]
    pub tdd: Option<bool>,
    #[serde(default)]
    pub audit: Option<bool>,
    #[serde(default)]
    pub tools: Vec<ToolEntry>,
    /// Remote connection settings. Presence of the section (even empty) is the
    /// remote-mode signal — a bare `remote:` key must parse as present, not vanish
    /// into fs mode, so missing url fails loudly downstream.
    #[serde(default, deserialize_with = "de_remote_section")]
    pub remote: Option<RemoteConfig>,
}

/// `remote:` section of `.speclink.yaml` — connection settings for team mode.
/// Both fields are optional at the parse layer: url may come from the
/// SPECLINK_STORE_URL environment variable instead (committed files can omit it).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RemoteConfig {
    pub url: Option<String>,
    pub repo: Option<String>,
}

/// Map a present-but-null `remote:` key to an empty section (Some) instead of None,
/// so "section present" stays distinguishable from "key absent" (serde default).
fn de_remote_section<'de, D>(d: D) -> Result<Option<RemoteConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<RemoteConfig>::deserialize(d)?;
    Ok(Some(v.unwrap_or_default()))
}

/// One entry of the `tools:` list — a built-in tool name string (claude, codex) or a
/// custom harness descriptor object.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ToolEntry {
    Builtin(String),
    Descriptor(ToolDescriptor),
}

/// Custom tool descriptor as parsed from YAML. All fields are optional at the serde
/// layer ON PURPOSE: a half-written descriptor must reach `validate()` and fail there
/// with a semantic single-line error naming the field — not fall into serde's
/// "did not match any variant", which `AppConfig::load` would silently swallow into
/// a default config (losing the whole tools list).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ToolDescriptor {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub skills_dir: Option<String>,
    #[serde(default)]
    pub instructions_file: Option<String>,
    /// Raw invocation value; `validate` restricts it to cli | tool-call (default cli).
    #[serde(default)]
    pub invocation: Option<String>,
}

/// How a custom harness invokes speclink verbs — decides the wording of generated text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Invocation {
    /// "run `speclink <verb>`" wording.
    #[default]
    Cli,
    /// "call the speclink tool (argv array)" wording.
    ToolCall,
}

/// A descriptor that passed validation — generation and pruning only ever see this form,
/// so an unvalidated path can never reach the filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomTool {
    pub name: String,
    pub skills_dir: String,
    pub instructions_file: String,
    pub invocation: Invocation,
}

impl ToolDescriptor {
    /// Validate into a `CustomTool`. Errors are single-line, name the offending field and
    /// the reason — the CLI surfaces them verbatim as its error line (exit code non-zero).
    pub fn validate(&self) -> Result<CustomTool, String> {
        let name = require_field(self.name.as_deref(), "name")?;
        if !is_kebab_case(name) {
            return Err(format!(
                "tool descriptor: name '{name}' must be kebab-case (2-50 chars of [a-z0-9-])"
            ));
        }
        // "agents" is Tool::parse's alias for codex, so it is reserved alongside the
        // canonical built-in names.
        if ["claude", "codex", "agents"].contains(&name) {
            return Err(format!(
                "tool descriptor: name '{name}' conflicts with a built-in tool name (claude, codex)"
            ));
        }
        let skills_dir = require_field(self.skills_dir.as_deref(), "skills_dir")?;
        check_project_relative(name, "skills_dir", skills_dir)?;
        let instructions_file = require_field(self.instructions_file.as_deref(), "instructions_file")?;
        check_project_relative(name, "instructions_file", instructions_file)?;
        let invocation = match self.invocation.as_deref() {
            None => Invocation::Cli,
            Some("cli") => Invocation::Cli,
            Some("tool-call") => Invocation::ToolCall,
            Some(other) => {
                return Err(format!(
                    "tool descriptor '{name}': invocation '{other}' must be 'cli' or 'tool-call'"
                ))
            }
        };
        Ok(CustomTool {
            name: name.to_string(),
            skills_dir: skills_dir.to_string(),
            instructions_file: instructions_file.to_string(),
            invocation,
        })
    }
}

fn require_field<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, String> {
    match value.map(str::trim) {
        Some(v) if !v.is_empty() => Ok(value.unwrap()),
        _ => Err(format!("tool descriptor: missing required field '{field}'")),
    }
}

fn is_kebab_case(s: &str) -> bool {
    (2..=50).contains(&s.chars().count())
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Security red line: descriptor paths are project-root-relative and must not escape the
/// root after LEXICAL normalization (no filesystem access here) — otherwise a descriptor
/// could write or delete files anywhere on the host.
fn check_project_relative(name: &str, field: &str, raw: &str) -> Result<(), String> {
    if is_project_relative(raw) {
        Ok(())
    } else {
        Err(format!(
            "tool descriptor '{name}': {field} '{raw}' escapes the project root (must be a relative path inside the project)"
        ))
    }
}

/// Lexical containment check shared by validation and prune (prune re-checks recorded
/// paths so a tampered state file cannot delete outside the project root).
pub fn is_project_relative(raw: &str) -> bool {
    use std::path::Component;
    let mut depth: i64 = 0;
    for comp in Path::new(raw).components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => return false,
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
        }
    }
    true
}

impl AppConfig {
    /// Load from a `.speclink.yaml` path. Missing file or parse error → defaults.
    /// This stays a direct host-side read: `.speclink.yaml` is the bootstrap
    /// that locates the project before any storage adapter exists.
    pub fn load(path: &Path) -> AppConfig {
        match crate::util::read_opt(path) {
            Some(s) => serde_yaml::from_str(&s).unwrap_or_default(),
            None => AppConfig::default(),
        }
    }

    /// Human-readable locale name for instruction injection.
    pub fn locale_display(&self) -> String {
        locale_display(self.locale.as_deref())
    }

    /// Names of deprecated policy keys present in this `.speclink.yaml`, in canonical
    /// order. Non-empty → the CLI surfaces a single deprecation warning pointing at
    /// the keys' canonical home, `openspec/config.yaml`.
    pub fn deprecated_policy_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if self.locale.is_some() {
            keys.push("locale");
        }
        if self.spec_locale.is_some() {
            keys.push("spec_locale");
        }
        if self.tdd.is_some() {
            keys.push("tdd");
        }
        if self.audit.is_some() {
            keys.push("audit");
        }
        keys
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
    /// Canonical home of the workflow-policy toggles (nullable: absent falls to defaults).
    #[serde(default)]
    pub tdd: Option<bool>,
    #[serde(default)]
    pub audit: Option<bool>,
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,
}

/// `SPECLINK_*` environment overrides — the top layer of the four-layer policy resolution
/// (personal/CI overrides beat both config files).
#[derive(Debug, Clone, Default)]
pub struct EnvOverrides {
    pub locale: Option<String>,
    pub spec_locale: Option<String>,
    pub tdd: Option<bool>,
    pub audit: Option<bool>,
}

impl EnvOverrides {
    /// Read overrides from the process environment.
    pub fn from_env() -> EnvOverrides {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Read overrides through an injectable lookup, so resolution stays testable
    /// without mutating process-global environment state.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> EnvOverrides {
        EnvOverrides {
            locale: get("SPECLINK_LOCALE").and_then(non_empty),
            spec_locale: get("SPECLINK_SPEC_LOCALE").and_then(non_empty),
            tdd: get("SPECLINK_TDD").as_deref().and_then(parse_env_bool),
            audit: get("SPECLINK_AUDIT").as_deref().and_then(parse_env_bool),
        }
    }
}

fn non_empty(v: String) -> Option<String> {
    let t = v.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Boolean env values accept only true/false (case-insensitive, trimmed). Anything else —
/// including "1"/"0"/"yes" — is treated as UNSET and falls to the next layer, never as an
/// error and never as a truthy value.
fn parse_env_bool(v: &str) -> Option<bool> {
    let t = v.trim();
    if t.eq_ignore_ascii_case("true") {
        Some(true)
    } else if t.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

/// Effective workflow policy after the four-layer resolution:
/// env var > deprecated `.speclink.yaml` key > `openspec/config.yaml` > built-in default.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPolicy {
    /// Human-readable locale display name (e.g. "English"), see `locale_display`.
    pub locale: String,
    /// Normalized spec-file language code; `None` = English.
    pub spec_locale: Option<String>,
    pub tdd: bool,
    pub audit: bool,
}

/// Four-layer policy resolution — the single entry point for effective policy values.
pub fn resolve_policy(env: &EnvOverrides, app: &AppConfig, wf: &WorkflowConfig) -> ResolvedPolicy {
    ResolvedPolicy {
        locale: locale_display(locale_code(env, app, wf)),
        spec_locale: spec_locale_code(env, app, wf),
        tdd: env.tdd.or(app.tdd).or(wf.tdd).unwrap_or(false),
        audit: env.audit.or(app.audit).or(wf.audit).unwrap_or(false),
    }
}

/// Layered locale code: first layer where the key is present wins (an app-level key present
/// but empty still wins, matching Spectra; values pass through verbatim, see `locale_display`).
fn locale_code<'a>(env: &'a EnvOverrides, app: &'a AppConfig, wf: &'a WorkflowConfig) -> Option<&'a str> {
    env.locale
        .as_deref()
        .or(app.locale.as_deref())
        .or(wf.locale.as_deref())
}

/// Layered spec-file language: unset / empty / "en" / "english" → `None` (specs default to
/// English); `"auto"` follows the locale resolved through the same layers.
fn spec_locale_code(env: &EnvOverrides, app: &AppConfig, wf: &WorkflowConfig) -> Option<String> {
    let code = env
        .spec_locale
        .as_deref()
        .or(app.spec_locale.as_deref())
        .or(wf.spec_locale.as_deref())?
        .trim()
        .to_string();
    let code = if code.eq_ignore_ascii_case("auto") {
        locale_code(env, app, wf)?.trim().to_string()
    } else {
        code
    };
    if code.is_empty() || code.eq_ignore_ascii_case("en") || code.eq_ignore_ascii_case("english") {
        return None;
    }
    Some(code)
}

/// Resolve the effective locale display name: the app-level `.speclink.yaml` locale wins, with the
/// `openspec/config.yaml` locale as a fallback (matches Spectra). Env-blind two-layer view —
/// callers that honor `SPECLINK_*` use `resolve_policy` instead.
pub fn resolve_locale(app: &AppConfig, wf: &WorkflowConfig) -> String {
    locale_display(locale_code(&EnvOverrides::default(), app, wf))
}

/// Resolve the effective spec-file language: `.speclink.yaml` wins over `openspec/config.yaml`.
/// Unset / empty / "en" / "english" → `None` (specs default to English); `"auto"` follows the
/// project locale (again `None` when that resolves to English). Env-blind two-layer view —
/// callers that honor `SPECLINK_*` use `resolve_policy` instead.
pub fn resolve_spec_locale(app: &AppConfig, wf: &WorkflowConfig) -> Option<String> {
    spec_locale_code(&EnvOverrides::default(), app, wf)
}

impl WorkflowConfig {
    /// Parse the raw workflow-config document (as handed over by the Store).
    /// A missing document or a parse error yields the defaults — serialization
    /// format and tolerance are unchanged from the path-based loader.
    pub fn from_text(text: Option<&str>) -> WorkflowConfig {
        match text {
            Some(s) => serde_yaml::from_str(s).unwrap_or_default(),
            None => WorkflowConfig::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn app(yaml: &str) -> AppConfig {
        serde_yaml::from_str(yaml).expect("app yaml")
    }

    fn wf(yaml: &str) -> WorkflowConfig {
        serde_yaml::from_str(yaml).expect("wf yaml")
    }

    fn env_of(pairs: &[(&str, &str)]) -> EnvOverrides {
        EnvOverrides::from_lookup(|key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
        })
    }

    const NO_ENV: &[(&str, &str)] = &[];

    // --- locale: env > old app key > config.yaml > default ---

    #[test]
    fn locale_env_var_wins_over_all_layers() {
        let p = resolve_policy(
            &env_of(&[("SPECLINK_LOCALE", "ja")]),
            &app("locale: tw"),
            &wf("locale: en"),
        );
        assert_eq!(p.locale, "Japanese (日本語)");
    }

    #[test]
    fn locale_old_app_key_wins_over_canonical() {
        // Spec scenario 舊鍵相容層勝過正典值: app tw + wf ja → Traditional Chinese.
        let p = resolve_policy(&env_of(NO_ENV), &app("locale: tw"), &wf("locale: ja"));
        assert_eq!(p.locale, "Traditional Chinese (繁體中文)");
    }

    #[test]
    fn locale_canonical_value_applies_without_upper_layers() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("locale: ja"));
        assert_eq!(p.locale, "Japanese (日本語)");
    }

    #[test]
    fn locale_defaults_to_english() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("{}"));
        assert_eq!(p.locale, "English");
    }

    // --- spec_locale: env > old app key > config.yaml > default ---

    #[test]
    fn spec_locale_env_var_wins_over_all_layers() {
        let p = resolve_policy(
            &env_of(&[("SPECLINK_SPEC_LOCALE", "ja")]),
            &app("spec_locale: tw"),
            &wf("spec_locale: en"),
        );
        assert_eq!(p.spec_locale.as_deref(), Some("ja"));
    }

    #[test]
    fn spec_locale_old_app_key_wins_over_canonical() {
        let p = resolve_policy(
            &env_of(NO_ENV),
            &app("spec_locale: tw"),
            &wf("spec_locale: ja"),
        );
        assert_eq!(p.spec_locale.as_deref(), Some("tw"));
    }

    #[test]
    fn spec_locale_canonical_value_applies_without_upper_layers() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("spec_locale: ja"));
        assert_eq!(p.spec_locale.as_deref(), Some("ja"));
    }

    #[test]
    fn spec_locale_defaults_to_none() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("{}"));
        assert_eq!(p.spec_locale, None);
    }

    #[test]
    fn spec_locale_auto_follows_resolved_locale() {
        // Existing "auto" semantics survive the extra env layer: auto follows the
        // locale resolved through the same four layers.
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("locale: ja\nspec_locale: auto"));
        assert_eq!(p.spec_locale.as_deref(), Some("ja"));
        let p = resolve_policy(
            &env_of(&[("SPECLINK_LOCALE", "tw")]),
            &app("{}"),
            &wf("locale: ja\nspec_locale: auto"),
        );
        assert_eq!(p.spec_locale.as_deref(), Some("tw"));
    }

    // --- tdd: env > old app key > config.yaml > default ---

    #[test]
    fn tdd_env_var_wins_over_all_layers() {
        // Spec scenario 環境變數覆寫一切: SPECLINK_TDD=false beats both files' true.
        let p = resolve_policy(
            &env_of(&[("SPECLINK_TDD", "false")]),
            &app("tdd: true"),
            &wf("tdd: true"),
        );
        assert!(!p.tdd);
    }

    #[test]
    fn tdd_old_app_key_wins_over_canonical() {
        // Presence wins, not truthiness: an explicit `tdd: false` in .speclink.yaml
        // must beat config.yaml's `tdd: true` (existing "app wins" semantics kept).
        let p = resolve_policy(&env_of(NO_ENV), &app("tdd: false"), &wf("tdd: true"));
        assert!(!p.tdd);
        let p = resolve_policy(&env_of(NO_ENV), &app("tdd: true"), &wf("tdd: false"));
        assert!(p.tdd);
    }

    #[test]
    fn tdd_canonical_value_applies_without_upper_layers() {
        // Spec scenario 正典值生效: only config.yaml sets tdd: true.
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("tdd: true"));
        assert!(p.tdd);
    }

    #[test]
    fn tdd_defaults_to_false() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("{}"));
        assert!(!p.tdd);
    }

    // --- audit: env > old app key > config.yaml > default ---

    #[test]
    fn audit_env_var_wins_over_all_layers() {
        let p = resolve_policy(
            &env_of(&[("SPECLINK_AUDIT", "false")]),
            &app("audit: true"),
            &wf("audit: true"),
        );
        assert!(!p.audit);
    }

    #[test]
    fn audit_old_app_key_wins_over_canonical() {
        let p = resolve_policy(&env_of(NO_ENV), &app("audit: false"), &wf("audit: true"));
        assert!(!p.audit);
    }

    #[test]
    fn audit_canonical_value_applies_without_upper_layers() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("audit: true"));
        assert!(p.audit);
    }

    #[test]
    fn audit_defaults_to_false() {
        let p = resolve_policy(&env_of(NO_ENV), &app("{}"), &wf("{}"));
        assert!(!p.audit);
    }

    // --- boolean env vars: only true/false are accepted; anything else is unset ---

    #[test]
    fn invalid_bool_env_var_falls_to_next_layer() {
        // Spec scenario 非法布林環境變數落到下一層: SPECLINK_AUDIT=yes is ignored.
        let p = resolve_policy(
            &env_of(&[("SPECLINK_AUDIT", "yes")]),
            &app("{}"),
            &wf("audit: true"),
        );
        assert!(p.audit);
        // Numeric truthiness is NOT a boolean here — "1"/"0" are unset too.
        let p = resolve_policy(
            &env_of(&[("SPECLINK_TDD", "1")]),
            &app("{}"),
            &wf("tdd: false"),
        );
        assert!(!p.tdd);
        // An invalid env value falls PAST the env layer only — the old app key still wins.
        let p = resolve_policy(
            &env_of(&[("SPECLINK_TDD", "yes")]),
            &app("tdd: false"),
            &wf("tdd: true"),
        );
        assert!(!p.tdd);
    }

    #[test]
    fn bool_env_var_is_case_insensitive_and_trimmed() {
        // A CI system exporting SPECLINK_TDD=TRUE must not silently disable the
        // override (confused-developer trap): case and surrounding whitespace are
        // normalized before matching.
        let p = resolve_policy(
            &env_of(&[("SPECLINK_TDD", " TRUE ")]),
            &app("tdd: false"),
            &wf("{}"),
        );
        assert!(p.tdd);
    }

    #[test]
    fn empty_string_env_vars_are_unset() {
        // `SPECLINK_LOCALE=` (empty) must not shadow lower layers with emptiness.
        let p = resolve_policy(
            &env_of(&[
                ("SPECLINK_LOCALE", ""),
                ("SPECLINK_SPEC_LOCALE", "  "),
                ("SPECLINK_TDD", ""),
            ]),
            &app("locale: tw\nspec_locale: tw\ntdd: true"),
            &wf("{}"),
        );
        assert_eq!(p.locale, "Traditional Chinese (繁體中文)");
        assert_eq!(p.spec_locale.as_deref(), Some("tw"));
        assert!(p.tdd);
    }

    // --- serde compatibility ---

    #[test]
    fn workflow_config_parses_new_policy_fields() {
        let w = wf("tdd: true\naudit: false");
        assert_eq!(w.tdd, Some(true));
        assert_eq!(w.audit, Some(false));
    }

    #[test]
    fn workflow_config_without_policy_fields_still_parses() {
        let w = WorkflowConfig::from_text(Some("schema: spec-driven\nlocale: tw"));
        assert_eq!(w.tdd, None);
        assert_eq!(w.audit, None);
        assert_eq!(w.locale.as_deref(), Some("tw"));
    }

    #[test]
    fn app_config_distinguishes_absent_from_false_policy_keys() {
        let a = app("tools:\n  - claude");
        assert_eq!(a.tdd, None);
        assert_eq!(a.audit, None);
        let a = app("tdd: false\naudit: true");
        assert_eq!(a.tdd, Some(false));
        assert_eq!(a.audit, Some(true));
    }

    // --- tools: dual-form entries (builtin name string | descriptor object) ---

    #[test]
    fn tools_list_parses_builtin_strings_and_descriptors() {
        let a = app(
            "tools:\n  - claude\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: tool-call\n",
        );
        assert_eq!(a.tools.len(), 2);
        match &a.tools[0] {
            ToolEntry::Builtin(s) => assert_eq!(s, "claude"),
            other => panic!("expected builtin string entry, got {other:?}"),
        }
        match &a.tools[1] {
            ToolEntry::Descriptor(d) => {
                assert_eq!(d.name.as_deref(), Some("wad-harness"));
                assert_eq!(d.skills_dir.as_deref(), Some(".wad/skills"));
                assert_eq!(d.instructions_file.as_deref(), Some("WAD.md"));
                assert_eq!(d.invocation.as_deref(), Some("tool-call"));
            }
            other => panic!("expected descriptor entry, got {other:?}"),
        }
    }

    #[test]
    fn tools_list_of_plain_strings_still_parses() {
        let a = app("tools:\n  - claude\n  - codex\n");
        assert_eq!(a.tools.len(), 2);
        assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
        assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
    }

    // --- descriptor validation: single-line semantic errors naming the field ---

    fn descriptor(name: &str, skills_dir: &str, instructions_file: &str) -> ToolDescriptor {
        ToolDescriptor {
            name: Some(name.to_string()),
            skills_dir: Some(skills_dir.to_string()),
            instructions_file: Some(instructions_file.to_string()),
            invocation: None,
        }
    }

    #[test]
    fn descriptor_validation_accepts_valid_input_and_defaults_invocation_to_cli() {
        let v = descriptor("wad-harness", ".wad/skills", "WAD.md")
            .validate()
            .expect("valid descriptor");
        assert_eq!(v.name, "wad-harness");
        assert_eq!(v.invocation, Invocation::Cli);
        let mut d = descriptor("wad-harness", ".wad/skills", "WAD.md");
        d.invocation = Some("tool-call".to_string());
        assert_eq!(d.validate().unwrap().invocation, Invocation::ToolCall);
    }

    #[test]
    fn descriptor_validation_rejects_builtin_name_conflict() {
        let err = descriptor("claude", ".wad/skills", "WAD.md").validate().unwrap_err();
        assert!(err.contains("name"), "must name the field: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    #[test]
    fn descriptor_validation_rejects_non_kebab_case_names() {
        for bad in ["Wad-Harness", "wad_harness", "-wad", "wad-", "w", &"x".repeat(51)] {
            let err = descriptor(bad, ".wad/skills", "WAD.md").validate().unwrap_err();
            assert!(err.contains("name"), "must name the field for {bad:?}: {err}");
            assert!(!err.contains('\n'), "single line: {err:?}");
        }
    }

    #[test]
    fn descriptor_validation_rejects_paths_escaping_project_root() {
        let err = descriptor("wad-harness", "../outside/skills", "WAD.md").validate().unwrap_err();
        assert!(err.contains("skills_dir"), "must name the field: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        let err = descriptor("wad-harness", ".wad/skills", "../WAD.md").validate().unwrap_err();
        assert!(err.contains("instructions_file"), "must name the field: {err}");
        // Lexical dot-dot tricks and absolute paths are escapes too.
        assert!(descriptor("wad-harness", ".wad/../../x", "WAD.md").validate().is_err());
        assert!(descriptor("wad-harness", "/abs/skills", "WAD.md").validate().is_err());
        assert!(descriptor("wad-harness", "C:\\abs\\skills", "WAD.md").validate().is_err());
    }

    #[test]
    fn descriptor_validation_rejects_unknown_invocation() {
        let mut d = descriptor("wad-harness", ".wad/skills", "WAD.md");
        d.invocation = Some("http".to_string());
        let err = d.validate().unwrap_err();
        assert!(err.contains("invocation"), "must name the field: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    #[test]
    fn descriptor_validation_rejects_missing_required_fields() {
        let d = ToolDescriptor {
            name: Some("wad-harness".to_string()),
            skills_dir: None,
            instructions_file: Some("WAD.md".to_string()),
            invocation: None,
        };
        let err = d.validate().unwrap_err();
        assert!(err.contains("skills_dir"), "must name the missing field: {err}");
    }

    // --- remote section: optional connection settings (url / repo) ---

    #[test]
    fn remote_section_with_url_and_repo_parses_both_fields() {
        let a = app("remote:\n  url: https://team.example.com/speclink/projects/foo\n  repo: backend\n");
        let r = a.remote.as_ref().expect("remote section present");
        assert_eq!(
            r.url.as_deref(),
            Some("https://team.example.com/speclink/projects/foo")
        );
        assert_eq!(r.repo.as_deref(), Some("backend"));
    }

    #[test]
    fn remote_section_with_only_repo_leaves_url_absent() {
        // Committed files may omit url (supplied at runtime via SPECLINK_STORE_URL).
        let a = app("remote:\n  repo: backend\n");
        let r = a.remote.as_ref().expect("remote section present");
        assert_eq!(r.url, None);
        assert_eq!(r.repo.as_deref(), Some("backend"));
    }

    #[test]
    fn empty_remote_section_is_present_with_absent_fields() {
        // Both `remote: {}` and a bare `remote:` key mean "section present, fields
        // empty" — a bare key must not silently read as fs mode (the mode signal is
        // key presence, and missing url must fail loudly downstream, not vanish here).
        for yaml in ["remote: {}\n", "remote:\n"] {
            let a = app(yaml);
            let r = a
                .remote
                .as_ref()
                .unwrap_or_else(|| panic!("remote section present for {yaml:?}"));
            assert_eq!(r.url, None);
            assert_eq!(r.repo, None);
        }
    }

    #[test]
    fn config_without_remote_key_parses_with_section_absent() {
        // Backward compatibility: existing .speclink.yaml files without a remote key
        // keep parsing, other fields intact, and the section reads as absent.
        let a = app("tools:\n  - claude\n  - codex\n");
        assert!(a.remote.is_none());
        assert_eq!(a.tools.len(), 2);
    }

    // --- existing two-layer resolvers keep their observable behavior ---

    #[test]
    fn resolve_locale_app_wins_then_workflow_fallback() {
        assert_eq!(
            resolve_locale(&app("locale: tw"), &wf("locale: ja")),
            "Traditional Chinese (繁體中文)"
        );
        assert_eq!(resolve_locale(&app("{}"), &wf("locale: ja")), "Japanese (日本語)");
        assert_eq!(resolve_locale(&app("{}"), &wf("{}")), "English");
    }

    #[test]
    fn resolve_spec_locale_keeps_auto_and_english_normalization() {
        assert_eq!(
            resolve_spec_locale(&app("spec_locale: auto\nlocale: tw"), &wf("{}")).as_deref(),
            Some("tw")
        );
        assert_eq!(resolve_spec_locale(&app("spec_locale: en"), &wf("{}")), None);
        assert_eq!(resolve_spec_locale(&app("{}"), &wf("{}")), None);
    }
}

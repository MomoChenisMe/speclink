//! Configuration: `.speclink.yaml` (app) and `openspec/config.yaml` (workflow).

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// A config file that EXISTS but cannot be parsed (YAML syntax error or type
/// mismatch). Fail-closed: loading never falls back to defaults on this error —
/// only a missing (or empty/null) document yields defaults. Mapped to
/// `invalid_config` at the command layer. Carries the workspace-relative file
/// path and the parser's reason so every entry point names the exact file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// Workspace-relative display path (".speclink.yaml" / "openspec/config.yaml").
    pub file: String,
    /// Parse-failure reason as reported by the YAML parser.
    pub reason: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid {}: {}", self.file, self.reason)
    }
}

impl std::error::Error for ConfigError {}

/// Fail-closed deserialization shared by the config loaders and change
/// metadata: an empty or null document (fresh template, comments only) is
/// valid and yields defaults; any parse failure on a non-empty document is an
/// error carrying the parser's reason.
pub(crate) fn parse_lenient_or_reason<T: Default + serde::de::DeserializeOwned>(
    text: &str,
) -> Result<T, String> {
    if text.trim().is_empty() {
        return Ok(T::default());
    }
    // A comments-only document parses as Null — that is an absent document, not
    // a broken one (matches `parse_yaml_mapping`'s Null tolerance).
    if matches!(serde_yaml::from_str::<serde_yaml::Value>(text), Ok(serde_yaml::Value::Null)) {
        return Ok(T::default());
    }
    serde_yaml::from_str(text).map_err(|e| e.to_string())
}

/// [`parse_lenient_or_reason`] wrapped into a `ConfigError` naming `file`.
fn parse_config<T: Default + serde::de::DeserializeOwned>(
    text: &str,
    file: &str,
) -> Result<T, ConfigError> {
    parse_lenient_or_reason(text).map_err(|reason| ConfigError {
        file: file.to_string(),
        reason,
    })
}

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
    /// Load from a `.speclink.yaml` path. Missing file → defaults; a file that
    /// exists but cannot parse → `ConfigError` (fail-closed, never defaults).
    /// This stays a direct host-side read: `.speclink.yaml` is the bootstrap
    /// that locates the project before any storage adapter exists.
    pub fn load(path: &Path) -> Result<AppConfig, ConfigError> {
        match crate::util::read_opt(path) {
            Some(s) => parse_config(&s, ".speclink.yaml"),
            None => Ok(AppConfig::default()),
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
    /// Read overrides through an injectable lookup — the only constructor:
    /// the process-env read lives at the Host boundary (speclink-host), so
    /// the Engine's policy resolution runs on injected values only.
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
    /// A missing (or empty/null) document yields the defaults; a document that
    /// exists but cannot parse → `ConfigError` (fail-closed, never defaults).
    /// Tolerance for successfully parsing documents is unchanged.
    pub fn from_text(text: Option<&str>) -> Result<WorkflowConfig, ConfigError> {
        match text {
            Some(s) => parse_config(s, "openspec/config.yaml"),
            None => Ok(WorkflowConfig::default()),
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

/// Target state of the four workflow-policy fields for a settings-page write.
/// `None` / `false` means "back to default": the key is REMOVED from the document
/// (preserving unset-means-default semantics) instead of writing an explicit value.
///
/// This is the COMPLETE target state, not a patch — a caller that writes without
/// first loading the current values wipes the fields it left at `Default`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkflowPolicyFields {
    pub locale: Option<String>,
    pub spec_locale: Option<String>,
    pub tdd: bool,
    pub audit: bool,
}

/// Context edit for a settings-page write: leave the key untouched, set it to a
/// value, or remove it (unset-means-default). A distinct three-state enum instead
/// of `Option<Option<String>>` — the two `None`s would be indistinguishable at a
/// call site. `Set` with a blank (whitespace-only) value degrades to `Remove`:
/// "clearing the text area removes the key" is enforced here, in the single
/// write-path truth, not left to each caller.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextEdit {
    Keep,
    Set(String),
    Remove,
}

/// Rewrite the workflow-config document (`openspec/config.yaml`) with the given
/// change set — a text→text pure function; the caller owns file reads and writes.
///
/// - `fields` is the COMPLETE target state of the four policy keys (see
///   [`WorkflowPolicyFields`]).
/// - `context` is a three-state edit; see [`ContextEdit`].
/// - `rules` is `None` to leave the key untouched, or `Some` for a wholesale
///   replacement: sections in slice order, entries trimmed with blank entries
///   dropped, an emptied section drops its artifact key, and an all-empty map
///   drops the `rules` key itself. Entries starting with YAML-reserved characters
///   (backtick, `@`, `*`, …) are quoted by serialization and round-trip verbatim.
///
/// Every other key (schema, remote, spec_dir, unknown keys) carries over through
/// a raw-mapping read–modify–write. Template comments are lost (same trade-off as
/// `init::write_remote_section`). Unlike `WorkflowConfig::from_text` (silent
/// defaults), malformed input is a loud error — rewriting an unparseable document
/// would destroy the user's content.
pub fn update_workflow_config_text(
    original: &str,
    fields: &WorkflowPolicyFields,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
) -> anyhow::Result<String> {
    let mut doc = parse_yaml_mapping(original, "openspec/config.yaml")?;
    set_or_remove(&mut doc, "locale", fields.locale.as_deref().map(Into::into));
    set_or_remove(&mut doc, "spec_locale", fields.spec_locale.as_deref().map(Into::into));
    set_or_remove(&mut doc, "tdd", fields.tdd.then(|| true.into()));
    set_or_remove(&mut doc, "audit", fields.audit.then(|| true.into()));
    match context {
        ContextEdit::Keep => {}
        ContextEdit::Set(text) if !text.trim().is_empty() => {
            set_or_remove(&mut doc, "context", Some(text.as_str().into()));
        }
        // Set(blank) 與 Remove 同義：清空即移除鍵。
        ContextEdit::Set(_) | ContextEdit::Remove => set_or_remove(&mut doc, "context", None),
    }
    if let Some(sections) = rules {
        let mut map = serde_yaml::Mapping::new();
        for (artifact, entries) in sections {
            let cleaned: Vec<serde_yaml::Value> = entries
                .iter()
                .map(|e| e.trim())
                .filter(|e| !e.is_empty())
                .map(Into::into)
                .collect();
            if !cleaned.is_empty() {
                map.insert(artifact.as_str().into(), serde_yaml::Value::Sequence(cleaned));
            }
        }
        let value = (!map.is_empty()).then(|| serde_yaml::Value::Mapping(map));
        set_or_remove(&mut doc, "rules", value);
    }
    Ok(serde_yaml::to_string(&doc)?)
}

/// Rewrite the `.speclink.yaml` tools list with the given built-in tool selection —
/// a text→text pure function with the same read–modify–write contract as
/// `update_workflow_config_text`. Walking the original list: custom descriptor
/// objects and unrecognized entries carry over verbatim in place (their unknown
/// fields included); built-in entries still selected stay under their original
/// spelling (deduplicated); deselected built-ins are removed; newly selected
/// built-ins append at the end under their canonical name. Every other top-level
/// key (spec_dir, remote, unknown keys) is untouched.
pub fn update_app_config_tools_text(
    original: &str,
    builtins: &[crate::skills::Tool],
) -> anyhow::Result<String> {
    use crate::skills::Tool;
    let mut doc = parse_yaml_mapping(original, ".speclink.yaml")?;
    let old = match doc.get("tools") {
        Some(serde_yaml::Value::Sequence(seq)) => seq.clone(),
        _ => Vec::new(),
    };
    let mut seen: Vec<Tool> = Vec::new();
    let mut new_list = Vec::new();
    for entry in old {
        match entry.as_str().and_then(Tool::parse) {
            Some(t) => {
                if builtins.contains(&t) && !seen.contains(&t) {
                    seen.push(t);
                    new_list.push(entry);
                }
            }
            None => new_list.push(entry),
        }
    }
    for t in builtins {
        if !seen.contains(t) {
            seen.push(*t);
            new_list.push(t.name().into());
        }
    }
    doc.insert("tools".into(), serde_yaml::Value::Sequence(new_list));
    Ok(serde_yaml::to_string(&doc)?)
}

/// Parse a config document as a raw top-level mapping for read–modify–write.
/// Empty or null input (absent file) yields an empty mapping; parse failures and
/// non-mapping documents are single-line errors naming the file.
fn parse_yaml_mapping(text: &str, file: &str) -> anyhow::Result<serde_yaml::Mapping> {
    if text.trim().is_empty() {
        return Ok(serde_yaml::Mapping::new());
    }
    let value: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|e| anyhow::anyhow!("invalid {file}: {e}"))?;
    match value {
        serde_yaml::Value::Mapping(m) => Ok(m),
        serde_yaml::Value::Null => Ok(serde_yaml::Mapping::new()),
        _ => anyhow::bail!("invalid {file}: expected a mapping at the top level"),
    }
}

/// Insert (replacing in place, order preserved) or remove a top-level key.
fn set_or_remove(doc: &mut serde_yaml::Mapping, key: &str, value: Option<serde_yaml::Value>) {
    match value {
        Some(v) => {
            doc.insert(key.into(), v);
        }
        None => {
            doc.remove(key);
        }
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

    // --- 注入形 lookup 是唯一 env 來源：process env 不可見 ---

    #[test]
    fn from_lookup_never_reads_the_process_environment() {
        std::env::set_var("SPECLINK_AUDIT", "true");
        let env = EnvOverrides::from_lookup(|_| None);
        std::env::remove_var("SPECLINK_AUDIT");
        assert_eq!(
            env.audit, None,
            "an empty injected lookup yields no overrides regardless of process env"
        );
    }

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
        let w = WorkflowConfig::from_text(Some("schema: spec-driven\nlocale: tw")).expect("parses");
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

    // --- fail-closed loading: a present file must parse; only a MISSING file yields defaults ---

    /// Throwaway dir for load() tests, removed on drop.
    struct TempCfgDir {
        dir: std::path::PathBuf,
    }

    impl TempCfgDir {
        fn new(tag: &str) -> TempCfgDir {
            let dir = std::env::temp_dir().join(format!(
                "speclink-core-cfg-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempCfgDir { dir }
        }

        fn app_yaml(&self, content: &str) -> std::path::PathBuf {
            let path = self.dir.join(".speclink.yaml");
            std::fs::write(&path, content).unwrap();
            path
        }
    }

    impl Drop for TempCfgDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn app_config_load_missing_file_gives_defaults() {
        let t = TempCfgDir::new("missing");
        let cfg = AppConfig::load(&t.dir.join(".speclink.yaml")).expect("missing file → defaults");
        assert!(cfg.spec_dir.is_none());
        assert!(cfg.remote.is_none());
        assert!(cfg.tools.is_empty());
    }

    #[test]
    fn app_config_load_empty_or_comment_only_file_gives_defaults() {
        // A template-fresh or commented-out file is a NULL document — that is valid
        // YAML, not a parse failure. Fail-closed must not break it.
        for content in ["", "\n\n", "# Speclink application config\n# tools:\n"] {
            let t = TempCfgDir::new("empty");
            let cfg = AppConfig::load(&t.app_yaml(content))
                .unwrap_or_else(|e| panic!("{content:?} must give defaults, got: {e}"));
            assert!(cfg.remote.is_none(), "for {content:?}");
        }
    }

    #[test]
    fn app_config_load_bad_yaml_is_a_config_error() {
        // P0 fail-closed: a file that EXISTS but cannot parse is an error carrying
        // the workspace-relative path and the parser's reason — never defaults.
        for bad in ["remote: [unclosed", ": not yaml : [", "remote: 42", "tools: notalist"] {
            let t = TempCfgDir::new("bad");
            let err = AppConfig::load(&t.app_yaml(bad))
                .expect_err(&format!("{bad:?} must be a config error"));
            assert_eq!(err.file, ".speclink.yaml", "for {bad:?}");
            assert!(!err.reason.is_empty(), "reason must not be empty for {bad:?}");
            let msg = err.to_string();
            assert!(msg.contains(".speclink.yaml"), "display names the file: {msg}");
            assert!(msg.contains(&err.reason), "display carries the reason: {msg}");
        }
    }

    #[test]
    fn app_config_load_valid_file_behavior_unchanged() {
        // Successfully parsing files keep their exact behavior (unknown keys tolerated).
        let t = TempCfgDir::new("valid");
        let path = t.app_yaml("spec_dir: docs/specs\nfuture_key: ignored\ntools:\n  - claude\n");
        let cfg = AppConfig::load(&path).expect("valid file parses");
        assert_eq!(cfg.spec_dir.as_deref(), Some("docs/specs"));
        assert_eq!(cfg.tools.len(), 1);
    }

    #[test]
    fn workflow_config_from_text_missing_gives_defaults() {
        let w = WorkflowConfig::from_text(None).expect("missing document → defaults");
        assert_eq!(w.schema, None);
        assert_eq!(w.tdd, None);
    }

    #[test]
    fn workflow_config_from_text_empty_or_comment_only_gives_defaults() {
        for text in ["", "\n", "# workflow config\n# tdd: true\n"] {
            let w = WorkflowConfig::from_text(Some(text))
                .unwrap_or_else(|e| panic!("{text:?} must give defaults, got: {e}"));
            assert_eq!(w.tdd, None, "for {text:?}");
        }
    }

    #[test]
    fn workflow_config_from_text_bad_yaml_is_a_config_error() {
        for bad in ["rules: [unclosed", "tdd: [true]", ": not yaml : ["] {
            let err = WorkflowConfig::from_text(Some(bad))
                .expect_err(&format!("{bad:?} must be a config error"));
            assert_eq!(err.file, "openspec/config.yaml", "for {bad:?}");
            assert!(!err.reason.is_empty(), "reason must not be empty for {bad:?}");
            let msg = err.to_string();
            assert!(msg.contains("openspec/config.yaml"), "display names the file: {msg}");
        }
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
        // A drive-letter path is absolute only on Windows; on unix the same string is a
        // legal (if odd) relative dir name — the containment check is lexical host-platform
        // semantics, so the rejection can only be asserted where it actually applies.
        if cfg!(windows) {
            assert!(descriptor("wad-harness", "C:\\abs\\skills", "WAD.md").validate().is_err());
        }
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

    // --- update_workflow_config_text: settings-page rewrite (text→text) ---

    const WF_DOC: &str = "schema: spec-driven\n\nlocale: tw\ncontext: |\n  line one\n  line two\n\nrules:\n  proposal:\n    - first rule\n    - second rule\n";

    #[test]
    fn workflow_update_sets_all_policy_fields_and_output_parses() {
        let fields = WorkflowPolicyFields {
            locale: Some("ja".into()),
            spec_locale: Some("auto".into()),
            tdd: true,
            audit: true,
        };
        let out = update_workflow_config_text(WF_DOC, &fields, &ContextEdit::Keep, None).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(w.locale.as_deref(), Some("ja"));
        assert_eq!(w.spec_locale.as_deref(), Some("auto"));
        assert_eq!(w.tdd, Some(true));
        assert_eq!(w.audit, Some(true));
    }

    #[test]
    fn workflow_update_preserves_untouched_key_values_verbatim() {
        // Re-serialization may change YAML styling; the parsed VALUES of every
        // untouched key (schema, multi-line context, rules) must stay identical
        // character for character.
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        let out = update_workflow_config_text(WF_DOC, &fields, &ContextEdit::Keep, None).expect("rewrite ok");
        let (orig, new) = (wf(WF_DOC), wf(&out));
        assert_eq!(new.schema, orig.schema);
        assert_eq!(new.context, orig.context);
        assert_eq!(new.rules, orig.rules);
    }

    #[test]
    fn workflow_update_keeps_unknown_keys() {
        let doc = "schema: spec-driven\nfuture_key: keep me\n";
        let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert_eq!(m.get("future_key").and_then(|v| v.as_str()), Some("keep me"));
    }

    #[test]
    fn workflow_update_default_values_remove_keys() {
        let doc = "locale: tw\nspec_locale: auto\ntdd: true\naudit: true\nschema: spec-driven\n";
        let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        for key in ["locale", "spec_locale", "tdd", "audit"] {
            assert!(!m.contains_key(key), "key '{key}' must be removed, got: {out}");
        }
        assert!(m.contains_key("schema"));
    }

    #[test]
    fn workflow_update_bad_yaml_is_a_loud_error() {
        // Unlike WorkflowConfig::from_text (silent defaults), rewriting a malformed
        // document must fail loudly — otherwise the GUI would destroy user content.
        for bad in ["rules: [unclosed", "just a top-level scalar"] {
            assert!(
                update_workflow_config_text(bad, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).is_err(),
                "must reject {bad:?}"
            );
        }
    }

    #[test]
    fn workflow_update_empty_input_creates_fresh_document() {
        // Absent config.yaml: the caller hands an empty text and gets a fresh
        // parseable document containing exactly the requested fields.
        let fields = WorkflowPolicyFields { locale: Some("tw".into()), ..Default::default() };
        let out = update_workflow_config_text("", &fields, &ContextEdit::Keep, None).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(w.locale.as_deref(), Some("tw"));
    }

    #[test]
    fn workflow_update_matches_spec_example_table() {
        // spec「政策欄位寫入效果」Example 表——逐行對應。
        // 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        let out = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).unwrap();
        assert_eq!(WorkflowConfig::from_text(Some(&out)).expect("output parses").tdd, Some(true));
        // tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false）
        let out = update_workflow_config_text("tdd: true\n", &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).unwrap();
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert!(!m.contains_key("tdd"), "got: {out}");
        // locale: tw、含 rules | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留
        let doc = "locale: tw\nrules:\n  proposal:\n    - keep\n";
        let fields = WorkflowPolicyFields {
            locale: Some("tw".into()),
            spec_locale: Some("auto".into()),
            ..Default::default()
        };
        let out = update_workflow_config_text(doc, &fields, &ContextEdit::Keep, None).unwrap();
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(w.spec_locale.as_deref(), Some("auto"));
        assert_eq!(w.locale.as_deref(), Some("tw"));
        assert_eq!(w.rules, wf(doc).rules);
    }

    #[test]
    fn workflow_update_injected_special_values_round_trip_safely() {
        // Sharp-edges audit（Scoundrel）：值含換行或 YAML 語法時不得破壞文件
        // 結構——序列化必須 escape，輸出仍可解析且值逐字元 round-trip。
        for evil in ["tw\nrules: {}", "a: b", "#comment", "'quoted'", "- item"] {
            let fields = WorkflowPolicyFields { locale: Some(evil.into()), ..Default::default() };
            let out = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).expect("rewrite ok");
            let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
            assert_eq!(w.locale.as_deref(), Some(evil), "round-trip for {evil:?}");
            assert_eq!(w.schema.as_deref(), Some("spec-driven"), "structure intact for {evil:?}");
        }
    }

    // --- update_workflow_config_text: context 三態與 rules 整份代換（desktop-config-rules-context） ---

    /// 政策欄位與 context/rules 皆不動的呼叫縮寫（多數測試只關心其中一個變更集）。
    fn rewrite(
        doc: &str,
        context: &ContextEdit,
        rules: Option<&[(String, Vec<String>)]>,
    ) -> anyhow::Result<String> {
        update_workflow_config_text(doc, &wf_fields_of(doc), context, rules)
    }

    /// 從原文讀出政策欄位現值（「完整目標狀態」契約：不想動政策就得先讀再回填）。
    fn wf_fields_of(doc: &str) -> WorkflowPolicyFields {
        let w = wf(doc);
        WorkflowPolicyFields {
            locale: w.locale.clone(),
            spec_locale: w.spec_locale.clone(),
            tdd: w.tdd.unwrap_or(false),
            audit: w.audit.unwrap_or(false),
        }
    }

    fn section(pairs: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn workflow_update_context_set_round_trips_value_verbatim() {
        // spec Scenario 編輯專案說明並儲存：值逐字元一致、其餘鍵原樣保留。
        let text = "第一行說明\n\n第二行：含冒號: 與 # 井號\n";
        let out = rewrite(WF_DOC, &ContextEdit::Set(text.into()), None).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(w.context.as_deref(), Some(text));
        assert_eq!(w.schema, wf(WF_DOC).schema);
        assert_eq!(w.rules, wf(WF_DOC).rules);
        assert_eq!(w.locale, wf(WF_DOC).locale);
    }

    #[test]
    fn workflow_update_context_three_states() {
        // 三態：Keep 不動、Set 設值、Remove 移除鍵。
        let out = rewrite(WF_DOC, &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(wf(&out).context, wf(WF_DOC).context, "Keep must not touch context");
        let out = rewrite(WF_DOC, &ContextEdit::Remove, None).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert!(!m.contains_key("context"), "Remove must delete the key, got: {out}");
        let out = rewrite("schema: spec-driven\n", &ContextEdit::Set("新說明".into()), None).unwrap();
        assert_eq!(wf(&out).context.as_deref(), Some("新說明"));
    }

    #[test]
    fn workflow_update_context_blank_set_removes_key() {
        // 清空即移除鍵的語意在 core 落實：Set 空白字串視同 Remove（zero-value 安全）。
        for blank in ["", "   ", "\n\n"] {
            let out = rewrite(WF_DOC, &ContextEdit::Set(blank.into()), None).expect("rewrite ok");
            let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
            assert!(!m.contains_key("context"), "blank {blank:?} must remove the key");
        }
    }

    #[test]
    fn workflow_update_rules_replace_preserves_entry_order() {
        // spec Example 條目對調：tasks 節「先寫失敗測試」「更新文件」→ 上移後順序對調。
        let doc = "schema: spec-driven\nrules:\n  tasks:\n    - 先寫失敗測試\n    - 更新文件\n";
        let rules = section(&[("tasks", &["更新文件", "先寫失敗測試"])]);
        let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(
            w.rules.get("tasks").map(Vec::as_slice),
            Some(["更新文件".to_string(), "先寫失敗測試".to_string()].as_slice())
        );
        assert_eq!(w.schema.as_deref(), Some("spec-driven"));
    }

    #[test]
    fn workflow_update_rules_key_removal_matches_spec_example_table() {
        // spec Example 鍵移除語意——逐行對應。
        // context: 舊說明、rules 含 tasks 兩條 | 清空專案說明 | context 移除，rules.tasks 原樣保留
        let doc = "context: 舊說明\nrules:\n  tasks:\n    - a\n    - b\n";
        let out = rewrite(doc, &ContextEdit::Remove, None).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert!(!m.contains_key("context"));
        assert_eq!(wf(&out).rules, wf(doc).rules);
        // rules 含 proposal 與 tasks 兩節 | 刪除 tasks 節全部條目 | rules 僅餘 proposal 節
        let doc = "rules:\n  proposal:\n    - p1\n  tasks:\n    - t1\n";
        let rules = section(&[("proposal", &["p1"]), ("tasks", &[])]);
        let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(w.rules.get("proposal").map(Vec::as_slice), Some(["p1".to_string()].as_slice()));
        assert!(!w.rules.contains_key("tasks"), "empty section must drop its key");
        // rules 僅含 tasks 一節 | 刪除該節全部條目 | rules 鍵整個被移除
        let doc = "schema: spec-driven\nrules:\n  tasks:\n    - t1\n";
        let rules = section(&[("tasks", &[])]);
        let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert!(!m.contains_key("rules"), "all-empty rules must remove the key, got: {out}");
        assert!(m.contains_key("schema"));
    }

    #[test]
    fn workflow_update_rules_entries_trimmed_and_blank_dropped() {
        // 條目存入前 trim、空字串條目滌除；滌除後空節一併移除。
        let rules = section(&[("tasks", &["  先寫失敗測試  ", "   ", ""]), ("design", &["  ", ""])]);
        let out = rewrite("{}", &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(
            w.rules.get("tasks").map(Vec::as_slice),
            Some(["先寫失敗測試".to_string()].as_slice())
        );
        assert!(!w.rules.contains_key("design"), "all-blank section must drop its key");
    }

    #[test]
    fn workflow_update_rules_reserved_char_entries_round_trip() {
        // spec Example 保留字元條目自動加引號：GIVEN proposal 節一條，WHEN tasks 節新增
        // 「@完成後執行全部測試」，THEN 可解析、值逐字元還原、proposal 與 schema 保留。
        let doc = "schema: spec-driven\nrules:\n  proposal:\n    - 提案必須列出影響的 crates\n";
        let rules = section(&[
            ("proposal", &["提案必須列出影響的 crates"]),
            ("tasks", &["@完成後執行全部測試"]),
        ]);
        let out = rewrite(doc, &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
        let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
        assert_eq!(
            w.rules.get("tasks").map(Vec::as_slice),
            Some(["@完成後執行全部測試".to_string()].as_slice())
        );
        assert_eq!(
            w.rules.get("proposal").map(Vec::as_slice),
            Some(["提案必須列出影響的 crates".to_string()].as_slice())
        );
        assert_eq!(w.schema.as_deref(), Some("spec-driven"));
        // 反引號開頭（既知炸檔地雷）與其他 YAML 保留起始字元亦須 round-trip。
        for evil in ["`cargo test` 全綠", "@標註開頭", "*星號開頭", "&錨點開頭"] {
            let rules = section(&[("tasks", &[evil])]);
            let out = rewrite("{}", &ContextEdit::Keep, Some(&rules)).expect("rewrite ok");
            let w = WorkflowConfig::from_text(Some(&out)).expect("output parses");
            assert_eq!(
                w.rules.get("tasks").map(Vec::as_slice),
                Some([evil.to_string()].as_slice()),
                "round-trip for {evil:?}"
            );
            assert!(!w.rules.is_empty(), "document must stay parseable for {evil:?}");
        }
    }

    #[test]
    fn workflow_update_rules_none_leaves_rules_untouched() {
        // rules: None＝不動——政策欄位寫入路徑不得波及 rules。
        let out = rewrite(WF_DOC, &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(wf(&out).rules, wf(WF_DOC).rules);
    }

    #[test]
    fn workflow_update_content_edit_preserves_reserved_keys_verbatim() {
        // MODIFIED 需求保留名單：remote、spec_dir、未知鍵於 context/rules 寫入時逐值保留。
        let doc = "spec_dir: docs/specs\nremote:\n  url: https://example.com\n  repo: main\nfuture_key: keep me\ncontext: old\n";
        let rules = section(&[("tasks", &["新規則"])]);
        let out = rewrite(doc, &ContextEdit::Set("新說明".into()), Some(&rules)).expect("rewrite ok");
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        let orig: serde_yaml::Mapping = serde_yaml::from_str(doc).expect("mapping");
        for key in ["spec_dir", "remote", "future_key"] {
            assert_eq!(m.get(key), orig.get(key), "key '{key}' must carry over");
        }
        assert_eq!(wf(&out).context.as_deref(), Some("新說明"));
    }

    #[test]
    fn workflow_update_content_edit_bad_yaml_is_a_loud_error() {
        // 不經 rewrite helper——壞 YAML 連現值都讀不出，直接以 default 政策呼叫。
        for bad in ["rules: [unclosed", "just a top-level scalar"] {
            assert!(
                update_workflow_config_text(
                    bad,
                    &WorkflowPolicyFields::default(),
                    &ContextEdit::Set("x".into()),
                    None,
                )
                .is_err(),
                "must reject {bad:?}"
            );
        }
    }

    // --- update_app_config_tools_text: builtin tool selection rewrite ---

    use crate::skills::Tool;

    #[test]
    fn tools_update_replaces_builtin_selection() {
        let out = update_app_config_tools_text("tools:\n  - claude\n", &[Tool::Claude, Tool::Codex])
            .expect("rewrite ok");
        let a = app(&out);
        assert_eq!(a.tools.len(), 2);
        assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
        assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
    }

    #[test]
    fn tools_update_removes_deselected_builtins() {
        let out = update_app_config_tools_text("tools:\n  - claude\n  - codex\n", &[Tool::Claude])
            .expect("rewrite ok");
        let a = app(&out);
        assert_eq!(a.tools.len(), 1);
        assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
    }

    #[test]
    fn tools_update_preserves_descriptors_and_other_keys() {
        let doc = concat!(
            "spec_dir: docs/specs\n",
            "tools:\n",
            "  - claude\n",
            "  - name: wad-harness\n",
            "    skills_dir: .wad/skills\n",
            "    instructions_file: WAD.md\n",
            "    future_field: keep me\n",
            "remote:\n",
            "  url: https://team.example.com/x\n",
            "  repo: backend\n",
        );
        let out = update_app_config_tools_text(doc, &[Tool::Codex]).expect("rewrite ok");
        let a = app(&out);
        assert_eq!(a.spec_dir.as_deref(), Some("docs/specs"));
        let r = a.remote.as_ref().expect("remote section kept");
        assert_eq!(r.url.as_deref(), Some("https://team.example.com/x"));
        assert_eq!(r.repo.as_deref(), Some("backend"));
        // claude 落選移除；descriptor 原樣保留（保序）；codex 新入選 append 尾端。
        assert_eq!(a.tools.len(), 2);
        match &a.tools[0] {
            ToolEntry::Descriptor(d) => {
                assert_eq!(d.name.as_deref(), Some("wad-harness"));
                assert_eq!(d.skills_dir.as_deref(), Some(".wad/skills"));
                assert_eq!(d.instructions_file.as_deref(), Some("WAD.md"));
            }
            other => panic!("expected descriptor first, got {other:?}"),
        }
        assert!(matches!(&a.tools[1], ToolEntry::Builtin(s) if s == "codex"));
        // 描述子的未知欄位也逐字保留（raw value carry-over）。
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        let tools = m.get("tools").and_then(|v| v.as_sequence()).expect("tools seq");
        assert_eq!(
            tools[0].get("future_field").and_then(|v| v.as_str()),
            Some("keep me")
        );
    }

    #[test]
    fn tools_update_bad_yaml_is_a_loud_error() {
        for bad in ["tools: [unclosed", "just a top-level scalar"] {
            assert!(
                update_app_config_tools_text(bad, &[Tool::Claude]).is_err(),
                "must reject {bad:?}"
            );
        }
    }

    #[test]
    fn tools_update_creates_tools_key_when_absent() {
        for doc in ["", "spec_dir: docs/specs\n"] {
            let out = update_app_config_tools_text(doc, &[Tool::Claude]).expect("rewrite ok");
            let a = app(&out);
            assert_eq!(a.tools.len(), 1, "for input {doc:?}");
            assert!(matches!(&a.tools[0], ToolEntry::Builtin(s) if s == "claude"));
        }
    }

    /// Spec example「built-in 選集轉換」第三列（codex → claude,codex）：兩者都在且不重複，
    /// 同時鎖住 Implementation Contract 的「其他鍵保持可解析且值不變」——包含未知頂層鍵。
    #[test]
    fn tools_update_adds_missing_builtin_and_keeps_unknown_top_level_keys() {
        let doc = concat!(
            "tools:\n",
            "  - codex\n",
            "  - name: wad-harness\n",
            "    skills_dir: .wad/skills\n",
            "    instructions_file: WAD.md\n",
            "future_top_level: keep me\n",
        );
        let out = update_app_config_tools_text(doc, &[Tool::Claude, Tool::Codex]).expect("rewrite ok");
        let a = app(&out);
        let builtins: Vec<&str> = a
            .tools
            .iter()
            .filter_map(|e| match e {
                ToolEntry::Builtin(s) => Some(s.as_str()),
                ToolEntry::Descriptor(_) => None,
            })
            .collect();
        assert_eq!(builtins, ["codex", "claude"], "既有項保序、新項 append，且不重複");
        assert!(a.tools.iter().any(|e| matches!(e, ToolEntry::Descriptor(_))));
        let m: serde_yaml::Mapping = serde_yaml::from_str(&out).expect("mapping");
        assert_eq!(
            m.get("future_top_level").and_then(|v| v.as_str()),
            Some("keep me")
        );
    }

    #[test]
    fn tools_update_empty_selection_keeps_descriptors_only() {
        let doc = "tools:\n  - claude\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n";
        let out = update_app_config_tools_text(doc, &[]).expect("rewrite ok");
        let a = app(&out);
        assert_eq!(a.tools.len(), 1);
        assert!(matches!(&a.tools[0], ToolEntry::Descriptor(_)));
    }
}

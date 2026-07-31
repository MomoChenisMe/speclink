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

/// Map a locale code to its human-readable name (frozen mapping).
///
/// Matching is case-SENSITIVE. Only `ja`/`tw`/`en` (and no locale) are mapped; any other code is
/// echoed back verbatim.
pub fn locale_display(code: Option<&str>) -> String {
    // No trimming and no case folding: any unmapped value is preserved verbatim (including
    // empty/whitespace and case-variants like "JA").
    match code {
        None => "English".to_string(),
        Some("en") => "English".to_string(),
        Some("ja") => "Japanese (日本語)".to_string(),
        Some("tw") => "Traditional Chinese (繁體中文)".to_string(),
        Some(other) => other.to_string(),
    }
}

/// Locale codes the official write paths accept for `locale` — exactly the keys of
/// `locale_display`'s frozen mapping. Matching is case-sensitive, like the mapping.
pub const LOCALE_CODES: [&str; 3] = ["tw", "ja", "en"];

/// Codes accepted for `spec_locale`: the `locale` set plus `auto` (follow `locale`).
pub const SPEC_LOCALE_CODES: [&str; 4] = ["tw", "ja", "en", "auto"];

/// Validate the locale fields of a policy write. Write-side only: read paths stay
/// lenient (`locale_display` echoes unknown codes verbatim), so pre-existing
/// out-of-set values remain readable — they just can no longer be (re)written
/// through official verbs. `None` (unset) is always valid.
pub fn validate_policy_locales(fields: &WorkflowPolicyFields) -> anyhow::Result<()> {
    check_locale_code("locale", fields.locale.as_deref(), &LOCALE_CODES)?;
    check_locale_code("spec_locale", fields.spec_locale.as_deref(), &SPEC_LOCALE_CODES)
}

fn check_locale_code(key: &str, value: Option<&str>, codes: &[&str]) -> anyhow::Result<()> {
    match value {
        None => Ok(()),
        Some(v) if codes.contains(&v) => Ok(()),
        Some(v) => anyhow::bail!("Value for '{key}' must be one of {} (got '{v}')", codes.join(", ")),
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
/// but empty still wins; values pass through verbatim, see `locale_display`).
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
/// `openspec/config.yaml` locale as a fallback. Env-blind two-layer view —
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
/// The rewrite is line-level text surgery, not re-serialization: only the target
/// key's line or block changes; every other line — comments, blank lines, unknown
/// top-level keys, user content below `schema` — is preserved byte for byte.
/// Policy keys update in place (never move); missing policy keys insert as one
/// canonical-order block right below the `schema` key line (top of file when
/// `schema` is absent), separated by exactly one blank line on each side.
/// Before returning, the result is re-parsed and compared key by key against the
/// intended state — a mismatch is a single-line error (fail-closed: a surgery bug
/// can at worst refuse to write, never corrupt the file). Unlike
/// `WorkflowConfig::from_text` (silent defaults), malformed input is a loud
/// error — rewriting an unparseable document would destroy the user's content.
pub fn update_workflow_config_text(
    original: &str,
    fields: &WorkflowPolicyFields,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
) -> anyhow::Result<String> {
    validate_policy_locales(fields)?;
    // Pure-syntax empty documents carry no user content to preserve; normalize so
    // surgery starts from a blank page instead of appending after a `{}` line.
    let base = match original.trim() {
        "{}" | "null" | "~" => "",
        _ => original,
    };
    let parsed = parse_yaml_mapping(base, "openspec/config.yaml")?;

    // rules 的滌洗結果由手術（序列化區塊）與目標狀態（驗證基準）共用。
    let rules_value = rules.map(|sections| {
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
        map
    });

    // Target state: the mapping-level edit the old rewrite used to serialize,
    // now demoted to the verification oracle for the text surgery.
    let mut target = parsed;
    set_or_remove(&mut target, "locale", fields.locale.as_deref().map(Into::into));
    set_or_remove(&mut target, "spec_locale", fields.spec_locale.as_deref().map(Into::into));
    set_or_remove(&mut target, "tdd", fields.tdd.then(|| true.into()));
    set_or_remove(&mut target, "audit", fields.audit.then(|| true.into()));
    match context {
        ContextEdit::Keep => {}
        ContextEdit::Set(text) if !text.trim().is_empty() => {
            set_or_remove(&mut target, "context", Some(text.as_str().into()));
        }
        // Set(blank) 與 Remove 同義：清空即移除鍵。
        ContextEdit::Set(_) | ContextEdit::Remove => set_or_remove(&mut target, "context", None),
    }
    if let Some(map) = &rules_value {
        let value = (!map.is_empty()).then(|| serde_yaml::Value::Mapping(map.clone()));
        set_or_remove(&mut target, "rules", value);
    }

    let output = surgical_rewrite(base, fields, context, rules_value.as_ref())?;
    verify_rewritten_config(&output, &target)?;
    Ok(output)
}

/// Fail-closed guard on the surgical output: re-parse through the same path and
/// compare against the intended state key by key. Any mismatch — including output
/// that no longer parses — refuses the write with a single-line error, so a
/// text-surgery bug can never reach the user's file.
fn verify_rewritten_config(output: &str, target: &serde_yaml::Mapping) -> anyhow::Result<()> {
    let reparsed = parse_yaml_mapping(output, "openspec/config.yaml").map_err(|e| {
        anyhow::anyhow!("internal rewrite verification failed: rewritten config does not parse ({e})")
    })?;
    if &reparsed != target {
        anyhow::bail!(
            "internal rewrite verification failed: rewritten openspec/config.yaml does not match the intended state"
        );
    }
    Ok(())
}

/// One top-level key block: `[start, end)` line range covering the key line plus
/// its indented continuation lines. A blank line belongs to the block only when
/// further indented content follows (a blank inside a block scalar); trailing
/// blanks and column-zero comments between blocks stay outside every block, so
/// surgery never deletes them alongside a key.
struct KeyBlock {
    key: String,
    start: usize,
    end: usize,
}

/// Line-level edit plan for the surgical rewrite.
#[derive(Clone)]
enum LineOp {
    Keep,
    Delete,
    Replace(String),
}

fn replace_block(ops: &mut [LineOp], block: &KeyBlock, text: String) {
    ops[block.start] = LineOp::Replace(text);
    for op in &mut ops[block.start + 1..block.end] {
        *op = LineOp::Delete;
    }
}

fn delete_block(ops: &mut [LineOp], block: &KeyBlock) {
    for op in &mut ops[block.start..block.end] {
        *op = LineOp::Delete;
    }
}

/// The line-level surgery itself (see `update_workflow_config_text` for the
/// semantics). Pure text transform; correctness is enforced by the caller's
/// re-parse verification, never assumed here.
fn surgical_rewrite(
    original: &str,
    fields: &WorkflowPolicyFields,
    context: &ContextEdit,
    rules: Option<&serde_yaml::Mapping>,
) -> anyhow::Result<String> {
    let eol = if original.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = original.split_inclusive('\n').collect();
    let blocks = scan_top_level_blocks(&lines);
    let block_of = |key: &str| blocks.iter().find(|b| b.key == key);

    let mut ops: Vec<LineOp> = vec![LineOp::Keep; lines.len()];
    // 缺鍵按此陣列順序收集＝範本正典序（locale、spec_locale、tdd、audit）。
    let desired = [
        ("locale", fields.locale.clone()),
        ("spec_locale", fields.spec_locale.clone()),
        ("tdd", fields.tdd.then(|| "true".to_string())),
        ("audit", fields.audit.then(|| "true".to_string())),
    ];
    let mut missing: Vec<(&str, String)> = Vec::new();
    for (key, value) in desired {
        match (block_of(key), value) {
            (Some(b), Some(v)) => {
                let line = format!("{key}: {v}{}", line_terminator(lines[b.start]));
                replace_block(&mut ops, b, line);
            }
            (Some(b), None) => delete_block(&mut ops, b),
            (None, Some(v)) => missing.push((key, v)),
            (None, None) => {}
        }
    }

    let mut appends: Vec<String> = Vec::new();
    match context {
        ContextEdit::Keep => {}
        ContextEdit::Set(text) if !text.trim().is_empty() => {
            let block = serialize_top_level_block("context", text.as_str().into())?;
            match block_of("context") {
                Some(b) => replace_block(&mut ops, b, block),
                None => appends.push(block),
            }
        }
        ContextEdit::Set(_) | ContextEdit::Remove => {
            if let Some(b) = block_of("context") {
                delete_block(&mut ops, b);
            }
        }
    }
    if let Some(map) = rules {
        if map.is_empty() {
            if let Some(b) = block_of("rules") {
                delete_block(&mut ops, b);
            }
        } else {
            let block = serialize_top_level_block("rules", serde_yaml::Value::Mapping(map.clone()))?;
            match block_of("rules") {
                Some(b) => replace_block(&mut ops, b, block),
                None => appends.push(block),
            }
        }
    }

    let insert_at = block_of("schema").map(|b| b.end).unwrap_or(0);
    let mut out = String::with_capacity(original.len() + 64);
    for i in 0..=lines.len() {
        if i == insert_at && !missing.is_empty() {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(eol);
            }
            if insert_at > 0 {
                out.push_str(eol); // schema 之下：區塊前恰一空行
            }
            for (key, value) in &missing {
                out.push_str(key);
                out.push_str(": ");
                out.push_str(value);
                out.push_str(eol);
            }
            // 區塊後恰一空行：後續第一個保留行已是空行時不重複補。
            let next = (i..lines.len()).find(|&j| !matches!(ops[j], LineOp::Delete));
            if next.is_some_and(|j| !is_blank_line(lines[j])) {
                out.push_str(eol);
            }
        }
        if i < lines.len() {
            match &ops[i] {
                LineOp::Keep => out.push_str(lines[i]),
                LineOp::Delete => {}
                LineOp::Replace(text) => out.push_str(text),
            }
        }
    }
    for block in appends {
        if !out.is_empty() {
            if !out.ends_with('\n') {
                out.push_str(eol);
            }
            // 附加區塊與前文之間恰一空行。
            if !(out.ends_with("\n\n") || out.ends_with("\n\r\n")) {
                out.push_str(eol);
            }
        }
        out.push_str(&block);
    }
    Ok(out)
}

/// Serialize `key: value` as a standalone top-level block via serde_yaml — the
/// exact bytes a full-document dump would produce for that one key, so the
/// replacement block round-trips to the identical value on re-parse.
fn serialize_top_level_block(key: &str, value: serde_yaml::Value) -> anyhow::Result<String> {
    let mut one = serde_yaml::Mapping::new();
    one.insert(key.into(), value);
    Ok(serde_yaml::to_string(&one)?)
}

/// Split the document's lines into top-level key blocks. YAML's indentation
/// rules guarantee continuation lines of a top-level value are indented, so a
/// column-zero `key:` line always starts a new block; a misjudgment on an exotic
/// document is caught by the re-parse verification (refuse, never corrupt).
fn scan_top_level_blocks(lines: &[&str]) -> Vec<KeyBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(key) = top_level_key_of(lines[i]) else {
            i += 1;
            continue;
        };
        let mut end = i + 1;
        let mut j = i + 1;
        while j < lines.len() {
            if is_blank_line(lines[j]) {
                j += 1; // 空行可能在 block scalar 中段——其後還有縮排內容才算在塊內
            } else if lines[j].starts_with(' ') || lines[j].starts_with('\t') {
                j += 1;
                end = j;
            } else {
                break;
            }
        }
        blocks.push(KeyBlock { key: key.to_string(), start: i, end });
        i = end;
    }
    blocks
}

/// The top-level key a line introduces, or `None` for comments, blanks,
/// indented continuations, and non-key lines.
fn top_level_key_of(line: &str) -> Option<&str> {
    let content = line.strip_suffix('\n').unwrap_or(line);
    let content = content.strip_suffix('\r').unwrap_or(content);
    let first = content.chars().next()?;
    if first.is_whitespace() || first == '#' {
        return None;
    }
    let colon = content.find(':')?;
    let key = content[..colon].trim_end();
    let rest = &content[colon + 1..];
    if key.is_empty() || !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }
    Some(key)
}

/// Terminator bytes of a raw line as produced by `split_inclusive('\n')`.
fn line_terminator(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
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
        // tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false）——唯一鍵移除後輸出為空文件
        let out = update_workflow_config_text("tdd: true\n", &WorkflowPolicyFields::default(), &ContextEdit::Keep, None).unwrap();
        assert_eq!(WorkflowConfig::from_text(Some(&out)).expect("output parses").tdd, None, "got: {out}");
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

    // --- policy locale 值域驗證（workflow-config-locale-validation，design D1／D3）---

    #[test]
    fn validate_policy_locales_accepts_codes_and_unset() {
        for code in ["tw", "ja", "en"] {
            let f = WorkflowPolicyFields { locale: Some(code.into()), ..Default::default() };
            assert!(validate_policy_locales(&f).is_ok(), "locale {code} must pass");
            let f = WorkflowPolicyFields { spec_locale: Some(code.into()), ..Default::default() };
            assert!(validate_policy_locales(&f).is_ok(), "spec_locale {code} must pass");
        }
        // spec_locale 另接受 auto；locale 不接受 auto
        let f = WorkflowPolicyFields { spec_locale: Some("auto".into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_ok());
        let f = WorkflowPolicyFields { locale: Some("auto".into()), ..Default::default() };
        assert!(validate_policy_locales(&f).is_err(), "locale auto must be rejected");
        // None（未設定）恆合法
        assert!(validate_policy_locales(&WorkflowPolicyFields::default()).is_ok());
    }

    #[test]
    fn validate_policy_locales_rejects_display_names_and_case_variants() {
        // spec「locale 值域判定」Example 表的拒絕列
        for bad in ["繁體中文", "TW", "Auto", ""] {
            let f = WorkflowPolicyFields { locale: Some(bad.into()), ..Default::default() };
            assert!(validate_policy_locales(&f).is_err(), "locale {bad:?} must be rejected");
        }
        for bad in ["繁體中文", "zh-Hant", "AUTO"] {
            let f = WorkflowPolicyFields { spec_locale: Some(bad.into()), ..Default::default() };
            assert!(validate_policy_locales(&f).is_err(), "spec_locale {bad:?} must be rejected");
        }
    }

    #[test]
    fn validate_policy_locales_error_names_field_value_and_codes() {
        let f = WorkflowPolicyFields { locale: Some("繁體中文".into()), ..Default::default() };
        let err = validate_policy_locales(&f).unwrap_err().to_string();
        for needle in ["locale", "繁體中文", "tw", "ja", "en"] {
            assert!(err.contains(needle), "error must contain {needle:?}, got: {err}");
        }
        let f = WorkflowPolicyFields { spec_locale: Some("zh-Hant".into()), ..Default::default() };
        let err = validate_policy_locales(&f).unwrap_err().to_string();
        for needle in ["spec_locale", "zh-Hant", "auto"] {
            assert!(err.contains(needle), "error must contain {needle:?}, got: {err}");
        }
    }

    #[test]
    fn workflow_update_rejects_invalid_locale_fields_without_output() {
        let fields = WorkflowPolicyFields { locale: Some("繁體中文".into()), ..Default::default() };
        let err = update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("locale") && err.contains("繁體中文"), "got: {err}");
        let fields = WorkflowPolicyFields { spec_locale: Some("zh-Hant".into()), ..Default::default() };
        assert!(
            update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).is_err()
        );
    }

    #[test]
    fn workflow_update_injected_special_values_are_rejected() {
        // Sharp-edges audit（Scoundrel）：locale 含換行或 YAML 語法的注入向量，
        // 自值域驗證上線後在序列化前即被拒絕——文件結構不可能被破壞。
        // （原測試斷言 escape 後 round-trip；值域驗證使拒絕成為更強的防護。）
        for evil in ["tw\nrules: {}", "a: b", "#comment", "'quoted'", "- item"] {
            let fields = WorkflowPolicyFields { locale: Some(evil.into()), ..Default::default() };
            assert!(
                update_workflow_config_text("schema: spec-driven\n", &fields, &ContextEdit::Keep, None).is_err(),
                "injection vector must be rejected: {evil:?}"
            );
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

    // --- update_workflow_config_text: 文字層手術（workflow-config-surgical-write） ---

    /// 只設定 locale、其餘政策欄位維持預設的縮寫。
    fn loc(code: &str) -> WorkflowPolicyFields {
        WorkflowPolicyFields { locale: Some(code.into()), ..Default::default() }
    }

    #[test]
    fn surgical_missing_key_inserts_below_schema_with_single_blank_lines() {
        // spec Scenario 缺鍵插於 schema 之下且空行區隔——相鄰處已有空行時不重複補。
        let doc = "# 模板註解\nschema: spec-driven\n\ncontext: |\n  第一行\n\nrules:\n  proposal:\n    - 條目一\n";
        let out = update_workflow_config_text(doc, &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(
            out,
            "# 模板註解\nschema: spec-driven\n\nlocale: tw\n\ncontext: |\n  第一行\n\nrules:\n  proposal:\n    - 條目一\n"
        );
        // 相鄰處原本沒有空行時補上：插入區塊與前後內容各恰一空行。
        let out = update_workflow_config_text(
            "schema: spec-driven\ncontext: 說明\n",
            &loc("tw"),
            &ContextEdit::Keep,
            None,
        )
        .expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\n\nlocale: tw\n\ncontext: 說明\n");
    }

    #[test]
    fn surgical_multiple_missing_keys_insert_one_canonical_block() {
        // 多缺鍵一次寫入：正典序（locale、spec_locale、tdd、audit）成連續區塊，不分散。
        let fields = WorkflowPolicyFields {
            locale: Some("tw".into()),
            spec_locale: Some("auto".into()),
            tdd: true,
            audit: true,
        };
        let out = update_workflow_config_text(
            "schema: spec-driven\n\ncontext: 說明\n",
            &fields,
            &ContextEdit::Keep,
            None,
        )
        .expect("rewrite ok");
        assert_eq!(
            out,
            "schema: spec-driven\n\nlocale: tw\nspec_locale: auto\ntdd: true\naudit: true\n\ncontext: 說明\n"
        );
    }

    #[test]
    fn surgical_preserves_comments_and_blank_lines_byte_for_byte() {
        // spec Scenario 註解與空行逐位元保留：原位改值只動目標行。
        let doc = "# 頭註解\nschema: spec-driven\n\n# locale 註解\nlocale: tw\n\n# 尾註解\ncontext: 說明\n";
        let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(
            out,
            "# 頭註解\nschema: spec-driven\n\n# locale 註解\nlocale: ja\n\n# 尾註解\ncontext: 說明\n"
        );
    }

    #[test]
    fn surgical_tail_key_updated_in_place_never_moves() {
        // spec Scenario 檔尾既有鍵原位改值不搬家（曾被舊版附加在 rules 之後）。
        let doc = "schema: spec-driven\n\nrules:\n  tasks:\n    - a\n\nlocale: tw\n";
        let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\n\nrules:\n  tasks:\n    - a\n\nlocale: ja\n");
    }

    #[test]
    fn surgical_missing_schema_inserts_at_file_top() {
        // spec Scenario schema 缺席時插於檔案最頂端，與後續內容之間恰一空行。
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        let out = update_workflow_config_text("context: 說明\n", &fields, &ContextEdit::Keep, None)
            .expect("rewrite ok");
        assert_eq!(out, "tdd: true\n\ncontext: 說明\n");
        // 空檔案：只有插入區塊本身，無多餘空行。
        let out = update_workflow_config_text("", &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(out, "locale: tw\n");
    }

    #[test]
    fn surgical_user_content_below_schema_shifts_down_verbatim() {
        // schema 底下使用者自加內容：插入點仍在 schema 鍵行之後，使用者內容原樣後移。
        let doc = "schema: spec-driven\n# 使用者自加說明\nmy_key: 自訂\n\ncontext: 說明\n";
        let out = update_workflow_config_text(doc, &loc("tw"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(
            out,
            "schema: spec-driven\n\nlocale: tw\n\n# 使用者自加說明\nmy_key: 自訂\n\ncontext: 說明\n"
        );
    }

    #[test]
    fn surgical_set_false_removes_key_line_keeps_comment_above() {
        // spec Scenario 設 false 移除鍵：只刪鍵行，上方註解行仍在。
        let doc = "schema: spec-driven\n\n# audit 開關\naudit: true\n\ncontext: 說明\n";
        let out = update_workflow_config_text(doc, &WorkflowPolicyFields::default(), &ContextEdit::Keep, None)
            .expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\n\n# audit 開關\n\ncontext: 說明\n");
    }

    #[test]
    fn surgical_context_replaces_only_its_block() {
        // context 整塊替換：僅 context 區塊變動，前後註解與其他區段逐位元不變；
        // 舊值內含空行（block scalar 中段空行）仍屬同一區塊、不誤切。
        let doc = "schema: spec-driven\n\n# 說明區\ncontext: |\n  舊一\n\n  舊二\n\n# 規則區\nrules:\n  tasks:\n    - a\n";
        let out = update_workflow_config_text(
            doc,
            &WorkflowPolicyFields::default(),
            &ContextEdit::Set("新一\n新二\n".into()),
            None,
        )
        .expect("rewrite ok");
        assert_eq!(
            out,
            "schema: spec-driven\n\n# 說明區\ncontext: |\n  新一\n  新二\n\n# 規則區\nrules:\n  tasks:\n    - a\n"
        );
    }

    #[test]
    fn surgical_multiline_unknown_key_is_not_misjudged() {
        // 未知頂層鍵含多行 block scalar：縮排內容行（即使含冒號）不被誤判為頂層鍵。
        let doc = "notes: |\n  first: 看似鍵\n  second: 也是\nlocale: tw\n";
        let out = update_workflow_config_text(doc, &loc("ja"), &ContextEdit::Keep, None).expect("rewrite ok");
        assert_eq!(out, "notes: |\n  first: 看似鍵\n  second: 也是\nlocale: ja\n");
    }

    #[test]
    fn surgical_no_trailing_newline_and_crlf_edges() {
        // 結尾無換行：插入前先補行終止，插入區塊照常。
        let out = update_workflow_config_text("schema: spec-driven", &loc("tw"), &ContextEdit::Keep, None)
            .expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\n\nlocale: tw\n");
        // 結尾無換行的末行原位改值：不憑空補結尾換行。
        let out = update_workflow_config_text("schema: spec-driven\nlocale: tw", &loc("ja"), &ContextEdit::Keep, None)
            .expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\nlocale: ja");
        // CRLF 檔：未動行逐位元保留，改寫行與插入行沿用檔案的 CRLF。
        let fields = WorkflowPolicyFields { locale: Some("ja".into()), tdd: true, ..Default::default() };
        let out = update_workflow_config_text(
            "schema: spec-driven\r\nlocale: tw\r\n",
            &fields,
            &ContextEdit::Keep,
            None,
        )
        .expect("rewrite ok");
        assert_eq!(out, "schema: spec-driven\r\n\r\ntdd: true\r\n\r\nlocale: ja\r\n");
    }

    #[test]
    fn surgical_rewrite_verification_failure_is_fail_closed() {
        // spec Scenario 內部改寫驗證失敗拒絕寫入：引號鍵 'locale' 對文字層手術不可見，
        // 插入裸鍵後重新解析成重複鍵——驗證必須攔下並以單行錯誤拒寫（函式為純 text→text，
        // 呼叫端收到 Err 即不落檔，原檔逐位元不變）。
        let err = update_workflow_config_text("'locale': tw\n", &loc("ja"), &ContextEdit::Keep, None)
            .expect_err("must fail closed")
            .to_string();
        assert!(err.contains("internal rewrite verification failed"), "got: {err}");
        assert!(!err.contains('\n'), "error must be single-line, got: {err}");
        // 另一分支：輸出可解析但與目標狀態不等值——同樣單行拒絕。
        let target: serde_yaml::Mapping = serde_yaml::from_str("locale: tw\n").expect("mapping");
        let err = verify_rewritten_config("locale: ja\n", &target)
            .expect_err("mismatch must fail closed")
            .to_string();
        assert!(err.contains("internal rewrite verification failed"), "got: {err}");
        assert!(!err.contains('\n'), "error must be single-line, got: {err}");
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

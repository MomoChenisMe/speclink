//! Configuration: `.speclink.yaml` (app) and `openspec/config.yaml` (workflow).

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    /// Deprecated (change: remove-marker-injection). Nothing is generated into this file
    /// any more; it survives as the strip target for a legacy SPECLINK block and so old
    /// `.speclink.yaml` files keep parsing.
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
    /// Deprecated and optional: no longer drives generation, only legacy stripping.
    pub instructions_file: Option<String>,
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
        // skills_dir 在這個邊界一次正規化：削去結尾分隔符，讓生成、足跡記錄與過期
        // 探測的路徑回報只看得到同一種形式，下游不必再各自處理字串。
        let raw = require_field(self.skills_dir.as_deref(), "skills_dir")?;
        let skills_dir = raw.trim_end_matches('/');
        check_project_relative(name, "skills_dir", skills_dir)?;
        // 以下兩條守門比對正規化後的路徑，不是字串相等：`.claude/skills/.` 與
        // `./.claude/skills` 指的是同一個目錄，只擋字面拼法等於沒擋。
        let normalized = lexical_normalize(skills_dir);
        // 正規化後為空＝專案根本身：生成物會散進專案根、清理會掃到根目錄，比原字串
        // 更危險，故在這裡擋掉而非讓下游承受。
        if normalized.as_os_str().is_empty() {
            return Err(format!(
                "tool descriptor '{name}': skills_dir '{raw}' resolves to the project root itself (give it a directory)"
            ));
        }
        // 內建工具的 skills 目錄同樣被保留：兩個 target 指向同一個目錄時，探測會對同
        // 一份 SKILL.md 比兩種期望內容而永遠回報過期，生成時後手的 for_codex 子集也
        // 會把前手剛寫的 claude 專屬技能當成孤兒刪掉。名稱衝突已擋，目錄衝突同理。
        for builtin in [crate::skills::Tool::Claude, crate::skills::Tool::Codex] {
            if normalized == Path::new(builtin.skills_dir()) {
                return Err(format!(
                    "tool descriptor '{name}': skills_dir '{raw}' is the built-in {} skills directory (choose another directory)",
                    builtin.name()
                ));
            }
        }
        // instructions_file 已棄用（change: remove-marker-injection）：缺席合法，
        // 存在時仍驗證專案根相對與不逸出——剝除要照它去定位檔案。
        let instructions_file = match self.instructions_file.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(file) => {
                check_project_relative(name, "instructions_file", file)?;
                Some(file.to_string())
            }
        };
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
            instructions_file,
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

/// Lexically resolve a project-relative path: `.` segments dropped, `..` popping the
/// previous segment. No filesystem access — the same discipline as
/// [`is_project_relative`], which has already rejected anything escaping the root, so a
/// leading `..` cannot survive here. An empty result means the project root itself.
fn lexical_normalize(raw: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in Path::new(raw).components() {
        match comp {
            std::path::Component::Normal(part) => out.push(part),
            std::path::Component::ParentDir => {
                out.pop();
            }
            _ => {}
        }
    }
    out
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
    /// Parallel-apply worktree flow (nullable: absent falls to defaults).
    #[serde(default)]
    pub worktree: Option<bool>,
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,
}

/// `SPECLINK_*` environment overrides — the top layer of the three-layer policy resolution
/// (personal/CI overrides beat the canonical config).
#[derive(Debug, Clone, Default)]
pub struct EnvOverrides {
    pub locale: Option<String>,
    pub spec_locale: Option<String>,
    pub tdd: Option<bool>,
    pub audit: Option<bool>,
    pub worktree: Option<bool>,
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
            worktree: get("SPECLINK_WORKTREE").as_deref().and_then(parse_env_bool),
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

/// Effective workflow policy after the three-layer resolution:
/// env var > `openspec/config.yaml` > built-in default.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPolicy {
    /// Human-readable locale display name (e.g. "English"), see `locale_display`.
    pub locale: String,
    /// Normalized spec-file language code; `None` = English.
    pub spec_locale: Option<String>,
    pub tdd: bool,
    pub audit: bool,
    pub worktree: bool,
}

/// Three-layer policy resolution — the single entry point for effective policy values.
/// `.speclink.yaml` never participates: its policy keys parse (unknown keys are
/// ignored) but stay inert and warning-free, so the resolution takes no `AppConfig`.
pub fn resolve_policy(env: &EnvOverrides, wf: &WorkflowConfig) -> ResolvedPolicy {
    ResolvedPolicy {
        locale: locale_display(locale_code(env, wf)),
        spec_locale: spec_locale_code(env, wf),
        tdd: env.tdd.or(wf.tdd).unwrap_or(false),
        audit: env.audit.or(wf.audit).unwrap_or(false),
        worktree: env.worktree.or(wf.worktree).unwrap_or(false),
    }
}

/// Layered locale code: first layer where the key is present wins (values pass
/// through verbatim, see `locale_display`).
fn locale_code<'a>(env: &'a EnvOverrides, wf: &'a WorkflowConfig) -> Option<&'a str> {
    env.locale.as_deref().or(wf.locale.as_deref())
}

/// Layered spec-file language: unset / empty / "en" / "english" → `None` (specs default to
/// English); `"auto"` follows the locale resolved through the same layers.
fn spec_locale_code(env: &EnvOverrides, wf: &WorkflowConfig) -> Option<String> {
    let code = env
        .spec_locale
        .as_deref()
        .or(wf.spec_locale.as_deref())?
        .trim()
        .to_string();
    let code = if code.eq_ignore_ascii_case("auto") {
        locale_code(env, wf)?.trim().to_string()
    } else {
        code
    };
    if code.is_empty() || code.eq_ignore_ascii_case("en") || code.eq_ignore_ascii_case("english") {
        return None;
    }
    Some(code)
}

/// Resolve the locale display name from the canonical `openspec/config.yaml` alone.
/// Env-blind view — callers that honor `SPECLINK_*` use `resolve_policy` instead.
pub fn resolve_locale(wf: &WorkflowConfig) -> String {
    locale_display(locale_code(&EnvOverrides::default(), wf))
}

/// Resolve the spec-file language from the canonical `openspec/config.yaml` alone.
/// Unset / empty / "en" / "english" → `None` (specs default to English); `"auto"` follows the
/// project locale (again `None` when that resolves to English). Env-blind view —
/// callers that honor `SPECLINK_*` use `resolve_policy` instead.
pub fn resolve_spec_locale(wf: &WorkflowConfig) -> Option<String> {
    spec_locale_code(&EnvOverrides::default(), wf)
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

/// Target state of the workflow-policy fields for a settings-page write.
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
    pub worktree: bool,
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
    set_or_remove(&mut target, "worktree", fields.worktree.then(|| true.into()));
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

/// Set the `schema` key in a workflow config document, preserving every other byte.
/// `None` (the file does not exist yet) yields a document carrying only that key.
///
/// Like [`update_workflow_config_text`], this is line-level surgery guarded by a re-parse:
/// an unparseable input is a loud error rather than a silent overwrite of user content.
pub fn set_workflow_schema_text(original: Option<&str>, name: &str) -> anyhow::Result<String> {
    // An absent file and a pure-syntax empty document are the same case: no user
    // content to preserve, the result is a document carrying only the schema key.
    let base = match original.map(str::trim) {
        None | Some("" | "{}" | "null" | "~") => return Ok(format!("schema: {name}\n")),
        Some(_) => original.unwrap(),
    };
    let mut target = parse_yaml_mapping(base, "openspec/config.yaml")?;
    target.insert("schema".into(), name.into());

    let lines: Vec<&str> = base.split_inclusive('\n').collect();
    let blocks = scan_top_level_blocks(&lines);
    let eol = if base.contains("\r\n") { "\r\n" } else { "\n" };

    let mut out = String::with_capacity(base.len() + 32);
    match blocks.iter().find(|b| b.key == "schema") {
        Some(block) => {
            for (i, line) in lines.iter().enumerate() {
                if i == block.start {
                    out.push_str(&format!("schema: {name}{}", line_terminator(line)));
                } else if !(block.start..block.end).contains(&i) {
                    out.push_str(line);
                }
            }
        }
        None => {
            // Canonical layout puts `schema` first, then one blank line before the rest.
            out.push_str(&format!("schema: {name}{eol}"));
            if lines.first().is_some_and(|l| !is_blank_line(l)) {
                out.push_str(eol);
            }
            for line in &lines {
                out.push_str(line);
            }
        }
    }

    verify_rewritten_config(&out, &target)?;
    Ok(out)
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
    // 缺鍵按此陣列順序收集＝範本正典序（locale、spec_locale、tdd、audit、worktree）。
    let desired = [
        ("locale", fields.locale.clone()),
        ("spec_locale", fields.spec_locale.clone()),
        ("tdd", fields.tdd.then(|| "true".to_string())),
        ("audit", fields.audit.then(|| "true".to_string())),
        ("worktree", fields.worktree.then(|| "true".to_string())),
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

/// Write or replace the `remote:` section of `.speclink.yaml` via
/// read–modify–write: other fields keep their values (comments do not survive
/// re-serialization — a documented limitation). A missing file is created.
pub fn write_remote_section(
    root: &Path,
    url: &str,
    repo: Option<&str>,
) -> anyhow::Result<()> {
    let path = root.join(".speclink.yaml");
    let mut doc = read_app_yaml_doc(&path)?;
    let mut section = serde_yaml::Mapping::new();
    section.insert("url".into(), url.into());
    if let Some(r) = repo {
        section.insert("repo".into(), r.into());
    }
    doc.insert("remote".into(), serde_yaml::Value::Mapping(section));
    crate::util::write_file(&path, &serde_yaml::to_string(&doc)?)?;
    Ok(())
}

/// Remove the `remote:` section of `.speclink.yaml`, keeping every other
/// field. `Ok(true)` when a section was removed; `Ok(false)` when there was
/// nothing to remove (missing file included).
pub fn remove_remote_section(root: &Path) -> anyhow::Result<bool> {
    let path = root.join(".speclink.yaml");
    if !path.is_file() {
        return Ok(false);
    }
    let mut doc = read_app_yaml_doc(&path)?;
    if doc.remove("remote").is_none() {
        return Ok(false);
    }
    crate::util::write_file(&path, &serde_yaml::to_string(&doc)?)?;
    Ok(true)
}

/// `.speclink.yaml` as a raw mapping for read–modify–write: a missing file is an
/// empty mapping, a malformed one a loud error (rewriting it would destroy user content).
fn read_app_yaml_doc(path: &Path) -> anyhow::Result<serde_yaml::Mapping> {
    parse_yaml_mapping(&crate::util::read_opt(path).unwrap_or_default(), ".speclink.yaml")
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
mod tests;

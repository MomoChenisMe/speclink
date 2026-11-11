//! Configuration: `.speclink.yaml` (app) and `<spec dir>/config.yaml` (workflow).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A config file that EXISTS but cannot be parsed (YAML syntax error or type
/// mismatch). Fail-closed: loading never falls back to defaults on this error —
/// only a missing (or empty/null) document yields defaults. Mapped to
/// `invalid_config` at the command layer. Carries the workspace-relative file
/// path and the parser's reason so every entry point names the exact file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// Workspace-relative display path (".speclink.yaml" / "speclink/config.yaml").
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

/// One entry of the `tools:` list — a built-in tool name string (claude, codex, copilot, pi,
/// cursor) or a custom harness descriptor object.
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
        if crate::skills::Tool::parse(name).is_some() {
            return Err(format!(
                "tool descriptor: name '{name}' conflicts with a built-in tool name ({})",
                crate::skills::Tool::supported_names()
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
        // 一份 SKILL.md 比兩種期望內容而永遠回報過期，生成時後手的非 Claude 子集也
        // 會把前手剛寫的 claude 專屬技能當成孤兒刪掉。名稱衝突已擋，目錄衝突同理。
        // 比對不分大小寫：macOS／Windows 上 `.GitHub/skills` 就是 `.github/skills`。
        let folded = normalized.to_string_lossy().to_lowercase();
        for builtin in crate::skills::Tool::ALL {
            if Path::new(&folded) == Path::new(builtin.skills_dir()) {
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
    /// Project review rules for the review station's Standards axis. Not a policy
    /// field: no env override, never read from `.speclink.yaml`. `None` (key absent
    /// or null) reads as the defaults via [`WorkflowConfig::review`].
    #[serde(default)]
    pub review: Option<ReviewConfig>,
    /// Sub-agent model per quality station and AI tool. Not a policy field, like
    /// `review`. `None` (key absent or null) reads as the defaults via
    /// [`WorkflowConfig::subagent_model`]: every cell follows the main conversation.
    #[serde(default)]
    pub subagent_model: Option<SubagentModelConfig>,
}

/// `subagent_model:` block of `speclink/config.yaml`. Strict like [`ReviewConfig`]:
/// an unknown sub-key, a tool name that is not kebab-case or appears twice in one
/// map, or a value that is not a string fails the whole document. Model names are
/// never checked — the tool that receives one is the judge. Blank values read as
/// "not set" and empty lists as "no custom list", so the maps only hold cells that
/// carry a value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubagentModelConfig {
    /// AI tool name → model for the review station's two sub-agents.
    #[serde(default, deserialize_with = "de_model_map")]
    pub review: BTreeMap<ToolName, String>,
    /// AI tool name → model for the verify station's sub-agent.
    #[serde(default, deserialize_with = "de_model_map")]
    pub verify: BTreeMap<ToolName, String>,
    /// AI tool name → the project's custom model list (the settings page's choices).
    #[serde(default, deserialize_with = "de_choice_map")]
    pub choices: BTreeMap<ToolName, Vec<String>>,
}

/// The engine's built-in sub-agent model lists. Only Claude Code has one: its
/// Agent tool takes these four short names, which carry no version and never go
/// stale. Every other tool's names carry versions, so they live in `choices`.
pub const BUILTIN_SUBAGENT_MODELS: &[(&str, &[&str])] = &[("claude", &["sonnet", "opus", "haiku", "fable"])];

/// An AI tool name in `subagent_model`: kebab-case, the tool descriptor's rule.
/// [`ToolName::parse`] is the only way to build one, so every key the block holds —
/// read from the file or handed to a write — has passed that one check.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToolName(String);

impl ToolName {
    pub fn parse(name: &str) -> Result<ToolName, String> {
        if is_kebab_case(name) {
            Ok(ToolName(name.to_string()))
        } else {
            Err(format!(
                "tool name '{name}' is not kebab-case (2-50 lowercase letters, digits or hyphens; no leading, trailing or doubled hyphen)"
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Lets a map keyed by tool name be looked up with a plain `&str`.
impl std::borrow::Borrow<str> for ToolName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Serialize for ToolName {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ToolName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<ToolName, D::Error> {
        ToolName::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// A map keyed by tool name, each name at most once. serde's own `BTreeMap` keeps
/// the last of two equal keys, while the write side's untyped parse refuses them;
/// refusing here too keeps reads and writes in agreement. A null map reads as empty.
fn de_tool_map<'de, D, V>(d: D) -> Result<BTreeMap<ToolName, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct ToolMap<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for ToolMap<V> {
        type Value = BTreeMap<ToolName, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a map of AI tool names")
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(BTreeMap::new())
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(tool) = map.next_key::<ToolName>()? {
                if out.contains_key(&tool) {
                    return Err(serde::de::Error::custom(format!("duplicate tool name '{tool}'")));
                }
                let value = map.next_value::<V>()?;
                out.insert(tool, value);
            }
            Ok(out)
        }
    }
    d.deserialize_any(ToolMap(std::marker::PhantomData))
}

/// A station's cells: values trimmed, blank or null values dropped (not set).
fn de_model_map<'de, D>(d: D) -> Result<BTreeMap<ToolName, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = de_tool_map::<D, Option<StrictString>>(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(tool, model)| Some((tool, trimmed(&model?.0)?.to_string())))
        .collect())
}

/// The custom lists: entries cleaned like the review rule lists, empty or null lists dropped.
fn de_choice_map<'de, D>(d: D) -> Result<BTreeMap<ToolName, Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = de_tool_map::<D, Option<Vec<StrictString>>>(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(tool, entries)| {
            let entries = cleaned(entries?.iter().map(|e| e.0.as_str()));
            (!entries.is_empty()).then_some((tool, entries))
        })
        .collect())
}

/// The text without surrounding whitespace, or `None` when nothing is left — the
/// cleanup every read and write of a rule, model name or list entry applies.
fn trimmed(text: &str) -> Option<&str> {
    let text = text.trim();
    (!text.is_empty()).then_some(text)
}

/// List entries trimmed, blank ones dropped.
fn cleaned<'a>(entries: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    entries.into_iter().filter_map(trimmed).map(str::to_string).collect()
}

/// `review:` block of `openspec/config.yaml`. Strict on purpose: an unknown
/// sub-key (a typo like `critcal`), a baseline outside extend/replace, or a list
/// that is not a string array fails the whole document through the fail-closed
/// `ConfigError` path — a typo must never silently drop a rule list. The top
/// level of `WorkflowConfig` stays lenient, so older CLIs skip this block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewConfig {
    #[serde(default)]
    pub baseline: ReviewBaseline,
    #[serde(default, deserialize_with = "de_rule_list")]
    pub critical: Vec<String>,
    #[serde(default, deserialize_with = "de_rule_list")]
    pub warning: Vec<String>,
    #[serde(default, deserialize_with = "de_rule_list")]
    pub suggestion: Vec<String>,
}

/// A rule list whose entries must be YAML strings (see [`StrictString`]). A null
/// list (`critical:` with nothing after it) reads as an empty one. Entries are
/// trimmed and blank ones dropped on read — the same cleanup every write applies —
/// so a hand-written blank never reaches the review, and rewriting a list the
/// caller did not edit never changes its values.
fn de_rule_list<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let entries = Option::<Vec<StrictString>>::deserialize(d)?.unwrap_or_default();
    Ok(cleaned(entries.iter().map(|e| e.0.as_str())))
}

/// A YAML string only — a review rule, a model name or a list entry. `String` would
/// let serde_yaml coerce `1`, `true` or an empty item (null) into the text
/// "1"/"true"/"null" and hand it on as a rule or a model; `deserialize_any` sees the
/// real scalar type, and the error rises at the entry, so it names the entry's path
/// (`review.<list>[i]`, `subagent_model.review.<tool>`).
struct StrictString(String);

impl<'de> Deserialize<'de> for StrictString {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<StrictString, D::Error> {
        struct StringOnly;
        impl serde::de::Visitor<'_> for StringOnly {
            type Value = StrictString;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<StrictString, E> {
                Ok(StrictString(v.to_string()))
            }
        }
        d.deserialize_any(StringOnly)
    }
}

/// Whether the review station's built-in smell baseline applies on top of the
/// project rules (`extend`, the default) or not at all (`replace`). Values are
/// case-sensitive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewBaseline {
    #[default]
    Extend,
    Replace,
}

impl ReviewBaseline {
    /// The config-file spelling (`extend` / `replace`).
    pub fn as_str(self) -> &'static str {
        match self {
            ReviewBaseline::Extend => "extend",
            ReviewBaseline::Replace => "replace",
        }
    }

    /// The inverse of [`as_str`](Self::as_str), case-sensitive — the token parser for
    /// the review ticket's `**Rules**:` line and the remote round DTO. The config block
    /// does not call it: serde's `rename_all = "lowercase"` reads the same spellings.
    pub fn parse(s: &str) -> Option<ReviewBaseline> {
        match s {
            "extend" => Some(ReviewBaseline::Extend),
            "replace" => Some(ReviewBaseline::Replace),
            _ => None,
        }
    }
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

/// The workflow config's display path where no spec directory is known — the
/// remote shape, and the label the pure text functions use.
fn default_config_label() -> String {
    format!("{}/config.yaml", crate::workspace::DEFAULT_SPEC_DIR)
}

impl WorkflowConfig {
    /// Parse the raw workflow-config document (as handed over by the Store).
    /// A missing (or empty/null) document yields the defaults; a document that
    /// exists but cannot parse → `ConfigError` (fail-closed, never defaults).
    /// Tolerance for successfully parsing documents is unchanged.
    /// Errors name the document under the default spec directory — the remote
    /// shape; local callers go through [`WorkflowConfig::from_store`].
    pub fn from_text(text: Option<&str>) -> Result<WorkflowConfig, ConfigError> {
        match text {
            Some(s) => parse_config(s, &default_config_label()),
            None => Ok(WorkflowConfig::default()),
        }
    }

    /// Parse the store's workflow-config document; a parse error names it under
    /// the store's own spec directory.
    pub fn from_store(store: &dyn crate::store::Store) -> Result<WorkflowConfig, ConfigError> {
        WorkflowConfig::from_text(store.read_workflow_config().as_deref()).map_err(|e| ConfigError {
            file: format!("{}/config.yaml", store.spec_dir_display()),
            ..e
        })
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

    /// The review block, with an absent key read as its defaults (extend, three
    /// empty lists).
    pub fn review(&self) -> ReviewConfig {
        self.review.clone().unwrap_or_default()
    }

    /// The sub-agent model block, with an absent key read as three empty maps.
    pub fn subagent_model(&self) -> SubagentModelConfig {
        self.subagent_model.clone().unwrap_or_default()
    }
}

/// A policy key: one of the five fields `workflow-config set` accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyKey {
    Locale,
    SpecLocale,
    Tdd,
    Audit,
    Worktree,
}

impl PolicyKey {
    /// Every policy key, in canonical order.
    pub const ALL: [PolicyKey; 5] = [
        PolicyKey::Locale,
        PolicyKey::SpecLocale,
        PolicyKey::Tdd,
        PolicyKey::Audit,
        PolicyKey::Worktree,
    ];

    /// The config-file key.
    pub fn as_str(self) -> &'static str {
        match self {
            PolicyKey::Locale => "locale",
            PolicyKey::SpecLocale => "spec_locale",
            PolicyKey::Tdd => "tdd",
            PolicyKey::Audit => "audit",
            PolicyKey::Worktree => "worktree",
        }
    }

    /// The key a `set <key>` word names, case-sensitive; `None` for any other word.
    pub fn parse(key: &str) -> Option<PolicyKey> {
        PolicyKey::ALL.into_iter().find(|k| k.as_str() == key)
    }

    /// The keys as `set`'s help and its unknown-key message list them.
    pub fn list() -> String {
        PolicyKey::ALL.map(PolicyKey::as_str).join(", ")
    }
}

/// One key of the workflow config whose value a write changed. `Display` gives
/// the name the spec lists: `schema`, a policy key, `context`,
/// `rules.<artifact>`, `review.baseline`, `review.<list>`,
/// `subagent_model.<station>`, `subagent_model.choices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangedKey {
    Schema,
    Policy(PolicyKey),
    Context,
    /// One artifact's rule section.
    Rules(String),
    ReviewBaseline,
    ReviewList(ReviewList),
    /// One station's sub-agent models (any tool).
    SubagentModel(SubagentStation),
    SubagentChoices,
}

impl std::fmt::Display for ChangedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangedKey::Schema => f.write_str("schema"),
            ChangedKey::Policy(key) => f.write_str(key.as_str()),
            ChangedKey::Context => f.write_str("context"),
            ChangedKey::Rules(artifact) => write!(f, "rules.{artifact}"),
            ChangedKey::ReviewBaseline => f.write_str("review.baseline"),
            ChangedKey::ReviewList(list) => write!(f, "review.{}", list.key()),
            ChangedKey::SubagentModel(station) => write!(f, "subagent_model.{}", station.key()),
            ChangedKey::SubagentChoices => f.write_str("subagent_model.choices"),
        }
    }
}

/// The three severity-grouped rule lists of the `review` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewList {
    Critical,
    Warning,
    Suggestion,
}

impl ReviewList {
    /// The config-file key (`critical` / `warning` / `suggestion`).
    pub fn key(self) -> &'static str {
        match self {
            ReviewList::Critical => "critical",
            ReviewList::Warning => "warning",
            ReviewList::Suggestion => "suggestion",
        }
    }

    pub fn entries(self, review: &mut ReviewConfig) -> &mut Vec<String> {
        match self {
            ReviewList::Critical => &mut review.critical,
            ReviewList::Warning => &mut review.warning,
            ReviewList::Suggestion => &mut review.suggestion,
        }
    }
}

/// The two stations of the `subagent_model` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubagentStation {
    Review,
    Verify,
}

impl SubagentStation {
    /// The config-file key (`review` / `verify`).
    pub fn key(self) -> &'static str {
        match self {
            SubagentStation::Review => "review",
            SubagentStation::Verify => "verify",
        }
    }

    pub fn cells(self, block: &mut SubagentModelConfig) -> &mut BTreeMap<ToolName, String> {
        match self {
            SubagentStation::Review => &mut block.review,
            SubagentStation::Verify => &mut block.verify,
        }
    }
}

/// One edit of the workflow config, as a `workflow-config` write verb states
/// it once argv and stdin are resolved. `body` fields are the stdin text: one
/// list entry per line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WorkflowConfigEdit {
    /// Set one policy field. `key` and `value` are the raw argv words: the
    /// command reads them with [`PolicyKey::parse`] and refuses an unknown key
    /// or a bad value only after it has refused a document that does not
    /// parse — the order the CLI has always reported them in.
    Policy { key: String, value: String },
    /// Replace the context; blank text removes it.
    Context(String),
    /// Replace one artifact's rule section; no entries removes it.
    Rules { artifact: String, body: String },
    /// Replace one severity list of the review rules; no entries removes it.
    ReviewList { list: ReviewList, body: String },
    ReviewBaseline(ReviewBaseline),
    /// Set one station's sub-agent model for one AI tool; a blank model removes it.
    SubagentCell { station: SubagentStation, tool: ToolName, model: String },
    /// Replace one AI tool's custom model list; no entries removes it.
    SubagentChoices { tool: ToolName, body: String },
}

/// The keys whose parsed value differs between two workflow configs, in a
/// fixed order: `schema`, `locale`, `spec_locale`, `tdd`, `audit`, `worktree`,
/// `context`, `rules.<artifact>` (by artifact name), `review.baseline`,
/// `review.critical`, `review.warning`, `review.suggestion`,
/// `subagent_model.review`, `subagent_model.verify`, `subagent_model.choices`.
///
/// The comparison is on what the config means: a toggle that is absent equals
/// `false`, an empty rule section equals an absent one, and a change of layout
/// or comments gives no key. For a file that did not exist, pass
/// `WorkflowConfig::default()` as `before`.
///
/// Only the keys this engine reads are compared. A document can also carry
/// keys it does not know (they are kept, never read): a change of those gives
/// no key here, so "written, with no changed key" does not mean "nothing but
/// layout moved".
pub fn changed_keys(before: &WorkflowConfig, after: &WorkflowConfig) -> Vec<ChangedKey> {
    // Every field, with no `..`: a field added to `WorkflowConfig` does not
    // compile until it has its key below — a policy key cannot go unreported.
    let WorkflowConfig {
        schema: _,
        context: _,
        locale: _,
        spec_locale: _,
        tdd: _,
        audit: _,
        worktree: _,
        rules: _,
        review: _,
        subagent_model: _,
    } = before;
    let on = |toggle: Option<bool>| toggle.unwrap_or(false);
    let (review_a, review_b) = (before.review(), after.review());
    let (models_a, models_b) = (before.subagent_model(), after.subagent_model());
    // Both maps are sorted; a set gives each artifact once, in name order.
    let artifacts: std::collections::BTreeSet<&String> =
        before.rules.keys().chain(after.rules.keys()).collect();
    let rules = artifacts
        .into_iter()
        .filter(|artifact| before.rules_for(artifact) != after.rules_for(artifact))
        .map(|artifact| ChangedKey::Rules(artifact.clone()));

    let head = [
        (ChangedKey::Schema, before.schema != after.schema),
        (ChangedKey::Policy(PolicyKey::Locale), before.locale != after.locale),
        (ChangedKey::Policy(PolicyKey::SpecLocale), before.spec_locale != after.spec_locale),
        (ChangedKey::Policy(PolicyKey::Tdd), on(before.tdd) != on(after.tdd)),
        (ChangedKey::Policy(PolicyKey::Audit), on(before.audit) != on(after.audit)),
        (ChangedKey::Policy(PolicyKey::Worktree), on(before.worktree) != on(after.worktree)),
        (ChangedKey::Context, before.context_text() != after.context_text()),
    ];
    let tail = [
        (ChangedKey::ReviewBaseline, review_a.baseline != review_b.baseline),
        (ChangedKey::ReviewList(ReviewList::Critical), review_a.critical != review_b.critical),
        (ChangedKey::ReviewList(ReviewList::Warning), review_a.warning != review_b.warning),
        (ChangedKey::ReviewList(ReviewList::Suggestion), review_a.suggestion != review_b.suggestion),
        (ChangedKey::SubagentModel(SubagentStation::Review), models_a.review != models_b.review),
        (ChangedKey::SubagentModel(SubagentStation::Verify), models_a.verify != models_b.verify),
        (ChangedKey::SubagentChoices, models_a.choices != models_b.choices),
    ];
    let moved = |(key, changed): (ChangedKey, bool)| changed.then_some(key);
    head.into_iter().filter_map(moved).chain(rules).chain(tail.into_iter().filter_map(moved)).collect()
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
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ContextEdit {
    #[default]
    Keep,
    Set(String),
    Remove,
}

/// The content blocks one write may change. Each is untouched unless set, so a
/// caller names only the blocks it changes:
/// `ContentEdit { review: Some(&r), ..Default::default() }`.
///
/// - `context` is a three-state edit; see [`ContextEdit`].
/// - `rules` is `None` to leave the key untouched, or `Some` for a wholesale
///   replacement: sections in slice order, entries trimmed with blank entries
///   dropped, an emptied section drops its artifact key, and an all-empty map
///   drops the `rules` key itself. Entries starting with YAML-reserved characters
///   (backtick, `@`, `*`, …) are quoted by serialization and round-trip verbatim.
/// - `review` is `None` to leave the key untouched, or `Some` for a wholesale
///   replacement of the block: entries are cleaned like `rules`, `extend` writes
///   no `baseline` key, an empty list drops its key, and nothing left (extend
///   with three empty lists) drops the `review` key itself. Comments inside the
///   old block do not survive.
/// - `subagent_model` is `None` to leave the key untouched, or `Some` for a
///   wholesale replacement of the block: model names are trimmed and blank cells
///   dropped, list entries are cleaned like `rules`, a station or list left empty
///   drops its key, and nothing left drops the `subagent_model` key itself. Tool
///   names ([`ToolName`], so kebab-case by construction) go out in alphabetical
///   order. Comments inside the old block do not survive.
#[derive(Debug, Clone, Default)]
pub struct ContentEdit<'a> {
    pub context: ContextEdit,
    pub rules: Option<&'a [(String, Vec<String>)]>,
    pub review: Option<&'a ReviewConfig>,
    pub subagent_model: Option<&'a SubagentModelConfig>,
}

/// Rewrite the workflow-config document (`openspec/config.yaml`) with the given
/// change set — a text→text pure function; the caller owns file reads and writes.
///
/// - `fields` is the COMPLETE target state of the four policy keys (see
///   [`WorkflowPolicyFields`]).
/// - `edit` names the content blocks to change; see [`ContentEdit`]. Every
///   rewritten block follows the document's line endings.
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
    edit: &ContentEdit,
) -> anyhow::Result<String> {
    let ContentEdit { context, rules, review, subagent_model } = edit;
    validate_policy_locales(fields)?;
    // Pure-syntax empty documents carry no user content to preserve; normalize so
    // surgery starts from a blank page instead of appending after a `{}` line.
    let base = match original.trim() {
        "{}" | "null" | "~" => "",
        _ => original,
    };
    let parsed = parse_yaml_mapping(base, &default_config_label())?;

    // rules 與 review 的滌洗結果由手術（序列化區塊）與目標狀態（驗證基準）共用。
    let rules_value = rules.map(|sections| {
        let mut map = serde_yaml::Mapping::new();
        for (artifact, entries) in sections {
            insert_cleaned_list(&mut map, artifact, entries);
        }
        map
    });
    let review_value = review.map(|r| {
        let mut map = serde_yaml::Mapping::new();
        if r.baseline != ReviewBaseline::default() {
            map.insert("baseline".into(), r.baseline.as_str().into());
        }
        insert_cleaned_list(&mut map, "critical", &r.critical);
        insert_cleaned_list(&mut map, "warning", &r.warning);
        insert_cleaned_list(&mut map, "suggestion", &r.suggestion);
        map
    });
    let subagent_value = subagent_model.map(subagent_model_mapping);

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
    let blocks = [("rules", rules_value), ("review", review_value), ("subagent_model", subagent_value)];
    for (key, edit) in &blocks {
        if let Some(map) = edit {
            let value = (!map.is_empty()).then(|| serde_yaml::Value::Mapping(map.clone()));
            set_or_remove(&mut target, key, value);
        }
    }

    let output = surgical_rewrite(base, fields, context, &blocks)?;
    verify_rewritten_config(&output, &target)?;
    Ok(output)
}

/// The cleaned `subagent_model` block: blank cells, empty lists and empty
/// stations dropped, tool names in alphabetical order (the maps are sorted).
fn subagent_model_mapping(s: &SubagentModelConfig) -> serde_yaml::Mapping {
    let mut map = serde_yaml::Mapping::new();
    for (key, cells) in [("review", &s.review), ("verify", &s.verify)] {
        let cleaned: serde_yaml::Mapping = cells
            .iter()
            .filter_map(|(tool, model)| Some((tool.as_str().into(), trimmed(model)?.into())))
            .collect();
        if !cleaned.is_empty() {
            map.insert(key.into(), serde_yaml::Value::Mapping(cleaned));
        }
    }
    let mut choices = serde_yaml::Mapping::new();
    for (tool, entries) in &s.choices {
        insert_cleaned_list(&mut choices, tool.as_str(), entries);
    }
    if !choices.is_empty() {
        map.insert("choices".into(), serde_yaml::Value::Mapping(choices));
    }
    map
}

/// Insert `key: [entries]` with entries trimmed and blank ones dropped; a list
/// left empty is not inserted (an emptied list drops its key).
fn insert_cleaned_list(map: &mut serde_yaml::Mapping, key: &str, entries: &[String]) {
    let entries: Vec<serde_yaml::Value> =
        cleaned(entries.iter().map(String::as_str)).into_iter().map(Into::into).collect();
    if !entries.is_empty() {
        map.insert(key.into(), serde_yaml::Value::Sequence(entries));
    }
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
    let mut target = parse_yaml_mapping(base, &default_config_label())?;
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
    let reparsed = parse_yaml_mapping(output, &default_config_label()).map_err(|e| {
        anyhow::anyhow!("internal rewrite verification failed: rewritten config does not parse ({e})")
    })?;
    if &reparsed != target {
        anyhow::bail!(
            "internal rewrite verification failed: rewritten config.yaml does not match the intended state"
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

/// The serialized block for a wholesale mapping edit, or `None` when the mapping
/// is empty (the key is to be removed).
fn mapping_block(key: &str, map: &serde_yaml::Mapping) -> anyhow::Result<Option<String>> {
    if map.is_empty() {
        return Ok(None);
    }
    serialize_top_level_block(key, serde_yaml::Value::Mapping(map.clone())).map(Some)
}

/// Plan a wholesale block swap: `None` deletes the existing block; `Some`
/// replaces it in place, or appends it at the end when the key is absent.
fn swap_block(
    ops: &mut [LineOp],
    appends: &mut Vec<String>,
    existing: Option<&KeyBlock>,
    block: Option<String>,
) {
    match (existing, block) {
        (Some(b), Some(text)) => replace_block(ops, b, text),
        (None, Some(text)) => appends.push(text),
        (Some(b), None) => delete_block(ops, b),
        (None, None) => {}
    }
}

/// The line-level surgery itself (see `update_workflow_config_text` for the
/// semantics). Pure text transform; correctness is enforced by the caller's
/// re-parse verification, never assumed here.
fn surgical_rewrite(
    original: &str,
    fields: &WorkflowPolicyFields,
    context: &ContextEdit,
    content_blocks: &[(&str, Option<serde_yaml::Mapping>)],
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

    // serde_yaml 一律以 \n 斷行；CRLF 檔的代換區塊沿用檔案的行尾，整份行尾一致。
    let follow_eol = |block: String| if eol == "\n" { block } else { block.replace('\n', eol) };
    let mut appends: Vec<String> = Vec::new();
    match context {
        ContextEdit::Keep => {}
        ContextEdit::Set(text) if !text.trim().is_empty() => {
            let block = follow_eol(serialize_top_level_block("context", text.as_str().into())?);
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
    for (key, edit) in content_blocks {
        if let Some(map) = edit {
            let block = mapping_block(key, map)?.map(follow_eol);
            swap_block(&mut ops, &mut appends, block_of(key), block);
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

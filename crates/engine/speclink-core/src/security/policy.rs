//! One policy model for canonical YAML and the camelCase public read view.
//! Unknown existing fields are retained in the original document, never rebuilt
//! from this typed (known-fields-only) view.

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::{Mapping, Value};

pub const MAX_POLICY_PATCH_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityScope {
    #[default]
    Off,
    WebApi,
    Api,
    Web,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsvsLevel {
    #[default]
    L1,
    L2,
    L3,
}

impl AsvsLevel {
    pub fn number(self) -> u8 {
        match self {
            Self::L1 => 1,
            Self::L2 => 2,
            Self::L3 => 3,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardSelection {
    pub enabled: bool,
    #[serde(deserialize_with = "strict_optional_string")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AsvsSelection {
    pub enabled: bool,
    #[serde(deserialize_with = "strict_optional_string")]
    pub version: Option<String>,
    pub level: AsvsLevel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SecurityStandards {
    #[serde(alias = "owasp_top10")]
    pub owasp_top10: StandardSelection,
    #[serde(alias = "owasp_api_top10")]
    pub owasp_api_top10: StandardSelection,
    #[serde(alias = "owasp_asvs")]
    pub owasp_asvs: AsvsSelection,
    pub cwe: StandardSelection,
}

impl SecurityStandards {
    pub fn selected(&self) -> Vec<(&'static str, &str, Option<AsvsLevel>)> {
        let mut selected = Vec::new();
        for (id, enabled, version, level) in [
            (
                "owasp_top10",
                self.owasp_top10.enabled,
                &self.owasp_top10.version,
                None,
            ),
            (
                "owasp_api_top10",
                self.owasp_api_top10.enabled,
                &self.owasp_api_top10.version,
                None,
            ),
            (
                "owasp_asvs",
                self.owasp_asvs.enabled,
                &self.owasp_asvs.version,
                Some(self.owasp_asvs.level),
            ),
            ("cwe", self.cwe.enabled, &self.cwe.version, None),
        ] {
            if enabled {
                if let Some(version) = version.as_deref() {
                    selected.push((id, version, level));
                }
            }
        }
        selected
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SecurityReviews {
    pub spec: bool,
    pub code: bool,
    #[serde(alias = "api_boundary")]
    pub api_boundary: bool,
}

impl Default for SecurityReviews {
    fn default() -> Self {
        Self {
            spec: true,
            code: true,
            api_boundary: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "PolicyInput")]
pub struct SecurityPolicy {
    pub enabled: bool,
    pub required_for_acceptance: bool,
    pub scope: SecurityScope,
    pub standards: SecurityStandards,
    pub reviews: SecurityReviews,
    pub disabled_rules: Vec<String>,
    pub exclude_dirs: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PolicyInput {
    enabled: bool,
    #[serde(alias = "required_for_acceptance")]
    required_for_acceptance: bool,
    scope: SecurityScope,
    standards: SecurityStandards,
    reviews: SecurityReviews,
    #[serde(alias = "disabled_rules", deserialize_with = "strict_strings")]
    disabled_rules: Vec<String>,
    #[serde(alias = "exclude_dirs", deserialize_with = "strict_strings")]
    exclude_dirs: Vec<String>,
}

impl TryFrom<PolicyInput> for SecurityPolicy {
    type Error = String;
    fn try_from(input: PolicyInput) -> std::result::Result<Self, String> {
        let policy = Self {
            enabled: input.enabled,
            required_for_acceptance: input.required_for_acceptance,
            scope: input.scope,
            standards: input.standards,
            reviews: input.reviews,
            disabled_rules: input.disabled_rules,
            exclude_dirs: input.exclude_dirs,
        };
        policy.validate().map_err(|e| e.to_string())?;
        Ok(policy)
    }
}

impl SecurityPolicy {
    /// Only review behavior affects evidence freshness; acceptance is evaluated separately.
    pub fn review_digest(&self) -> Result<String> {
        let mut policy = self.clone();
        policy.required_for_acceptance = false;
        Ok(super::data::sha256(&serde_json::to_vec(&policy)?))
    }
    pub fn validate(&self) -> Result<()> {
        let standards = &self.standards;
        for (id, enabled, version, allowed) in [
            (
                "owasp_top10",
                standards.owasp_top10.enabled,
                &standards.owasp_top10.version,
                &["2021", "2025"][..],
            ),
            (
                "owasp_api_top10",
                standards.owasp_api_top10.enabled,
                &standards.owasp_api_top10.version,
                &["2019", "2023"][..],
            ),
            (
                "owasp_asvs",
                standards.owasp_asvs.enabled,
                &standards.owasp_asvs.version,
                &["4.0.3", "5.0.0"][..],
            ),
            (
                "cwe",
                standards.cwe.enabled,
                &standards.cwe.version,
                &[][..],
            ),
        ] {
            ensure!(
                !enabled || version.is_some(),
                "security_verification.standards.{id}.version is required when enabled"
            );
            if let Some(version) = version {
                ensure!(
                    if id == "cwe" {
                        valid_version(version)
                    } else {
                        allowed.contains(&version.as_str())
                    },
                    "invalid security_verification.standards.{id}.version"
                );
            }
        }
        for path in &self.exclude_dirs {
            ensure!(valid_relative_directory(path), "invalid security_verification.exclude_dirs: use a nonempty project-relative directory");
        }
        for id in &self.disabled_rules {
            let valid = ["SL-SEC-", "SL-SPEC-", "SL-API-"].iter().any(|prefix| {
                id.strip_prefix(prefix)
                    .is_some_and(|n| n.len() == 3 && n.bytes().all(|c| c.is_ascii_digit()))
            });
            ensure!(valid, "invalid security_verification.disabled_rules ID");
        }
        Ok(())
    }
}

pub fn valid_version(value: &str) -> bool {
    let parts: Vec<_> = value.split('.').collect();
    (1..=3).contains(&parts.len())
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 3 && part.bytes().all(|b| b.is_ascii_digit())
        })
}

pub fn valid_relative_directory(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}

fn strict_optional_string<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    match Value::deserialize(d)? {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s)),
        _ => Err(serde::de::Error::custom("version must be a string or null")),
    }
}

fn strict_strings<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    Vec::<Value>::deserialize(d)?
        .into_iter()
        .map(|v| match v {
            Value::String(s) => Ok(s),
            _ => Err(serde::de::Error::custom("expected a string array")),
        })
        .collect()
}

/// Capability negotiation inspects the same bounded YAML mapping accepted by patches.
pub fn acceptance_patch_requested(patch:&str)->Result<bool>{
    ensure!(patch.len()<=MAX_POLICY_PATCH_BYTES,"size_limit: security policy patch");
    let map:Mapping=serde_yaml::from_str(patch).map_err(|_|anyhow::anyhow!("invalid_config: security policy patch"))?;
    Ok(map.contains_key(Value::String("required_for_acceptance".into())))
}

/// Apply only known leaves. The original raw mapping is the preservation and
/// verification oracle; serializing SecurityPolicy would drop future fields.
pub fn patch_workflow_security(original: &str, patch: &str) -> Result<String> {
    const LEAVES: &[&str] = &[
        "enabled", "required_for_acceptance", "scope", "reviews.spec", "reviews.code",
        "reviews.api_boundary", "disabled_rules", "exclude_dirs",
        "standards.owasp_top10.enabled", "standards.owasp_top10.version",
        "standards.owasp_api_top10.enabled", "standards.owasp_api_top10.version",
        "standards.owasp_asvs.enabled", "standards.owasp_asvs.version", "standards.owasp_asvs.level",
        "standards.cwe.enabled", "standards.cwe.version",
    ];
    patch_workflow_section(original, patch, "security_verification", LEAVES)
}

pub fn patch_workflow_check_records(original: &str, patch: &str) -> Result<String> {
    patch_workflow_section(original, patch, "check_records", &["archive.review", "archive.verify", "archive.security"])
}

fn patch_workflow_section(original: &str, patch: &str, section: &str, leaves: &[&str]) -> Result<String> {
    ensure!(
        patch.len() <= MAX_POLICY_PATCH_BYTES,
        "security policy patch exceeds 64 KiB"
    );
    ensure!(
        !patch.trim().is_empty(),
        "security policy patch must be a nonempty mapping"
    );
    crate::config::WorkflowConfig::from_text(Some(original))?;
    let patch: Mapping = serde_yaml::from_str(patch)?;
    ensure!(
        !patch.is_empty(),
        "security policy patch must be a nonempty mapping"
    );
    validate_patch(&patch, "", leaves)?;
    let base = if matches!(original.trim(), "{}" | "null" | "~") {
        ""
    } else {
        original
    };
    let mut expected: Value = serde_yaml::from_str(base)?;
    if expected.is_null() {
        expected = Value::Mapping(Mapping::new());
    }
    let root = expected
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("workflow config must be a mapping"))?;
    let current = root
        .entry(Value::String(section.into()))
        .or_insert(Value::Mapping(Mapping::new()));
    merge_patch(current, &Value::Mapping(patch.clone()))?;
    let target_text = serde_yaml::to_string(&expected)?;
    crate::config::WorkflowConfig::from_text(Some(&target_text))?;
    let mut top = Mapping::new();
    top.insert(section.into(), Value::Mapping(patch));
    let eol = if base.contains("\r\n") { "\r\n" } else { "\n" };
    let output = rewrite_mapping(base, &top, 0, eol)?;
    let actual: Value = serde_yaml::from_str(&output)?;
    ensure!(
        actual == expected,
        "security policy rewrite verification failed; original is unchanged"
    );
    crate::config::WorkflowConfig::from_text(Some(&output))?;
    Ok(output)
}

fn validate_patch(patch: &Mapping, prefix: &str, leaves: &[&str]) -> Result<()> {
    ensure!(!patch.is_empty(), "policy patch must contain a leaf value");
    for (key, value) in patch {
        let key = key
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("security policy keys must be strings"))?;
        let path = if prefix.is_empty() {
            key.into()
        } else {
            format!("{prefix}.{key}")
        };
        if leaves.contains(&path.as_str()) {
            ensure!(!value.is_mapping(), "invalid security policy leaf: {path}");
        } else {
            ensure!(
                leaves
                    .iter()
                    .any(|leaf| leaf.starts_with(&(path.clone() + "."))),
                "unknown security policy patch key: {path}"
            );
            validate_patch(
                value
                    .as_mapping()
                    .ok_or_else(|| anyhow::anyhow!("{path} must be a mapping"))?,
                &path,
                leaves,
            )?;
        }
    }
    Ok(())
}

fn merge_patch(target: &mut Value, patch: &Value) -> Result<()> {
    if let Value::Mapping(entries) = patch {
        if target.is_null() {
            *target = Value::Mapping(Mapping::new());
        }
        let target = target
            .as_mapping_mut()
            .ok_or_else(|| anyhow::anyhow!("cannot merge security policy into a non-mapping"))?;
        for (key, value) in entries {
            let old = target.entry(key.clone()).or_insert(Value::Null);
            merge_patch(old, value)?;
        }
    } else {
        *target = patch.clone();
    }
    Ok(())
}

/// Work only on ordinary block mappings. Exotic forms fail verification or
/// are refused; a failed edit never replaces the caller's original document.
fn rewrite_mapping(text: &str, patch: &Mapping, indent: usize, eol: &str) -> Result<String> {
    let mut out = text.to_string();
    for (key, value) in patch {
        let key = key.as_str().expect("validated patch key");
        let lines: Vec<_> = out.split_inclusive('\n').collect();
        let prefix = format!("{}{key}:", " ".repeat(indent));
        let hit = lines.iter().position(|line| {
            line.starts_with(&prefix)
                && line[prefix.len()..]
                    .chars()
                    .next()
                    .is_none_or(char::is_whitespace)
        });
        if let Some(start) = hit {
            let end = (start + 1..lines.len())
                .find(|&i| {
                    let line = lines[i];
                    !line.trim().is_empty()
                        && !line.trim_start().starts_with('#')
                        && line.len() - line.trim_start().len() <= indent
                })
                .unwrap_or(lines.len());
            let raw_value = lines[start][prefix.len()..].trim();
            ensure!(
                !raw_value.starts_with(['&', '*'])
                    && !lines[start..end]
                        .iter()
                        .any(|line| line.trim_start().starts_with("<<:")),
                "security policy aliases and merge keys cannot be edited safely"
            );
            let (scalar, comment) = split_comment(raw_value);
            let block = if let Value::Mapping(children) = value {
                ensure!(
                    matches!(scalar.trim(), "" | "null" | "~"),
                    "security policy flow mappings cannot be edited safely"
                );
                let body = lines[start + 1..end].concat();
                let child_indent = body
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                    .map(|l| l.len() - l.trim_start().len())
                    .min()
                    .unwrap_or(indent + 2);
                format!(
                    "{prefix}{}{eol}{}",
                    comment,
                    rewrite_mapping(&body, children, child_indent, eol)?
                )
            } else {
                let mut block = render_entry(key, value, indent, eol)?;
                if !comment.is_empty() {
                    block = format!("{}{comment}{eol}", block.trim_end_matches(['\r', '\n']));
                }
                for line in &lines[start + 1..end] {
                    if line.trim().is_empty() || line.trim_start().starts_with('#') {
                        block.push_str(line);
                    }
                }
                block
            };
            out = format!(
                "{}{}{}",
                lines[..start].concat(),
                block,
                lines[end..].concat()
            );
        } else {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(eol);
            }
            out.push_str(&render_entry(key, value, indent, eol)?);
        }
    }
    Ok(out)
}

fn split_comment(value: &str) -> (&str, &str) {
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quote == Some('"') {
            escaped = true;
            continue;
        }
        if Some(c) == quote {
            quote = None;
        } else if quote.is_none() && matches!(c, '\'' | '"') {
            quote = Some(c);
        } else if quote.is_none()
            && c == '#'
            && (i == 0 || value[..i].ends_with(char::is_whitespace))
        {
            let start = value[..i].trim_end().len();
            return (&value[..start], &value[start..]);
        }
    }
    (value, "")
}

fn scalar(value: &Value) -> Result<String> {
    match value {
        Value::String(s) => {
            let escaped = crate::util::yaml_scalar(s);
            if serde_yaml::from_str::<Value>(&escaped).is_ok_and(|v| v == *value) {
                Ok(escaped)
            } else {
                Ok(serde_json::to_string(s)?)
            }
        }
        Value::Bool(v) => Ok(v.to_string()),
        Value::Null => Ok("null".into()),
        _ => bail!("security policy leaf must have its declared scalar type"),
    }
}

fn render_entry(key: &str, value: &Value, indent: usize, eol: &str) -> Result<String> {
    let pad = " ".repeat(indent);
    match value {
        Value::Mapping(map) => {
            let mut out = format!("{pad}{key}:{eol}");
            for (key, v) in map {
                out.push_str(&render_entry(
                    key.as_str().expect("validated key"),
                    v,
                    indent + 2,
                    eol,
                )?);
            }
            Ok(out)
        }
        Value::Sequence(seq) if seq.is_empty() => Ok(format!("{pad}{key}: []{eol}")),
        Value::Sequence(seq) => {
            let mut out = format!("{pad}{key}:{eol}");
            for value in seq {
                out.push_str(&format!("{pad}  - {}{eol}", scalar(value)?));
            }
            Ok(out)
        }
        _ => Ok(format!("{pad}{key}: {}{eol}", scalar(value)?)),
    }
}

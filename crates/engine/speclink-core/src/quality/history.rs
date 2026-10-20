//! Completed checks share one index; paths are generated from kind and ULID only.
mod archive;
use crate::{security::data::sha256, store::Store};
use anyhow::{ensure, Result};
pub use archive::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const INDEX: &str = "checks/index.json";
pub const JOURNAL: &str = ".checks-transaction.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    Review,
    Verify,
    Security,
}
impl CheckKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Verify => "verify",
            Self::Security => "security",
        }
    }
    pub fn path(self, id: &str) -> Result<String> {
        validate_id(id)?;
        Ok(match self {
            Self::Security => format!("checks/security/{id}/report.md"),
            _ => format!("checks/{}/{id}.md", self.as_str()),
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckEntry {
    pub record_id: String,
    pub kind: CheckKind,
    pub created_at: String,
    pub outcome: String,
    pub relative_path: String,
    pub content_digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckIndex {
    pub format_version: u8,
    pub entries: Vec<CheckEntry>,
}
impl Default for CheckIndex {
    fn default() -> Self {
        Self {
            format_version: 1,
            entries: vec![],
        }
    }
}
impl CheckIndex {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format_version == 1 && self.entries.len() <= 100_000,
            "invalid_check_index"
        );
        let mut ids = BTreeSet::new();
        for entry in &self.entries {
            ensure!(
                ids.insert(&entry.record_id)
                    && entry.relative_path == entry.kind.path(&entry.record_id)?,
                "invalid_check_index: duplicate ID or path"
            );
            ensure!(
                valid_digest(&entry.content_digest)
                    && !entry.created_at.is_empty()
                    && !entry.outcome.is_empty(),
                "invalid_check_index: metadata"
            );
        }
        Ok(())
    }
    pub fn append(&mut self, entry: CheckEntry) -> Result<()> {
        self.entries.push(entry);
        self.validate()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckMetadata {
    pub created_at: String,
    pub actor: Option<String>,
    pub tool: Option<String>,
    pub outcome: String,
    pub scope_digest: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckRecord {
    pub record_id: String,
    pub content: String,
    pub metadata: CheckMetadata,
}

/// Narrow check publication plan. Implementations must commit every effect or recover it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChecksCommit {
    pub writes: BTreeMap<String, String>,
    pub deletes: Vec<String>,
    pub expected: BTreeMap<String, Option<String>>,
}
impl ChecksCommit {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.writes.len() <= 16 && self.deletes.len() <= 2 && self.expected.len() <= 18,
            "invalid_checks_commit"
        );
        for path in self
            .writes
            .keys()
            .chain(self.deletes.iter())
            .chain(self.expected.keys())
        {
            ensure!(
                valid_check_path(path)
                    || matches!(path.as_str(), ".openspec.yaml" | "review.md" | "verify.md"),
                "invalid_checks_commit: path"
            );
        }
        ensure!(
            self.writes
                .keys()
                .chain(self.deletes.iter())
                .all(|p| self.expected.contains_key(p)),
            "invalid_checks_commit: missing comparison"
        );
        Ok(())
    }
}
pub fn valid_check_path(path: &str) -> bool {
    if matches!(path, INDEX | "checks/acceptance.json") {
        return true;
    }
    for kind in [CheckKind::Review, CheckKind::Verify] {
        if let Some(id) = path
            .strip_prefix(&format!("checks/{}/", kind.as_str()))
            .and_then(|p| p.strip_suffix(".md"))
        {
            return validate_id(id).is_ok();
        }
    }
    if let Some((id, file)) = path
        .strip_prefix("checks/security/")
        .and_then(|p| p.split_once('/'))
    {
        return validate_id(id).is_ok()
            && matches!(file, "report.md" | "report.json" | "decisions.jsonl");
    }
    false
}
pub fn validate_id(id: &str) -> Result<()> {
    ensure!(
        ulid::Ulid::from_string(id).is_ok_and(|value| value.to_string() == id),
        "invalid_record_id"
    );
    Ok(())
}
pub fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn read(store: &dyn Store, change: &str, path: &str) -> Option<String> {
    if store.change_exists(change) {
        store.read_artifact(change, path)
    } else {
        store.read_archived_artifact(change, path)
    }
}
pub fn index(store: &dyn Store, change: &str) -> Result<CheckIndex> {
    ensure!(
        crate::security::prepared::valid_relative(change) && !change.contains('/'),
        "invalid_change"
    );
    if !store.change_exists(change) && !store.archived_change_exists(change) {
        anyhow::bail!(crate::station::NotFound(format!(
            "change not found: {change}"
        )));
    }
    store.ensure_checks_readable(change)?;
    let index = match read(store, change, INDEX) {
        Some(text) => serde_json::from_str::<CheckIndex>(&text)?,
        None => CheckIndex::default(),
    };
    index.validate()?;
    for entry in &index.entries {
        let content = read(store, change, &entry.relative_path)
            .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
        ensure!(
            sha256(content.as_bytes()) == entry.content_digest,
            "integrity_error: check record"
        );
    }
    Ok(index)
}
pub fn record(store: &dyn Store, change: &str, id: &str, kind: CheckKind) -> Result<CheckRecord> {
    validate_id(id)?;
    let index = index(store, change)?;
    let entry = index
        .entries
        .iter()
        .find(|e| e.record_id == id && e.kind == kind)
        .ok_or_else(|| crate::station::NotFound("check record not found".into()))?;
    let content = read(store, change, &entry.relative_path)
        .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
    let header = content
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("<!-- speclink-check: "))
        .and_then(|line| line.strip_suffix(" -->"))
        .ok_or_else(|| anyhow::anyhow!("invalid_check_record"))?;
    let mut metadata: CheckMetadata = serde_json::from_str(header)?;
    ensure!(
        metadata.created_at == entry.created_at
            && metadata.outcome == entry.outcome
            && valid_digest(&metadata.scope_digest),
        "invalid_check_record"
    );
    metadata.actor = metadata
        .actor
        .as_deref()
        .map(crate::security::report::redact_text);
    metadata.tool = metadata
        .tool
        .as_deref()
        .map(crate::security::report::redact_text);
    Ok(CheckRecord {
        record_id: id.into(),
        content: crate::security::report::redact_text(&content),
        metadata,
    })
}

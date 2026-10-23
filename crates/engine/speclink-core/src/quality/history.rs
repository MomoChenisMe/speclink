//! Completed checks share one index; review/verify paths include the run start time and ULID.
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
    pub fn path_with_started_at(self, id: &str, started_at: &str) -> Result<String> {
        validate_id(id)?;
        let timestamp = chrono::DateTime::parse_from_rfc3339(started_at)?
            .with_timezone(&chrono::Utc)
            .format("%Y%m%d%H%M%S");
        Ok(match self {
            Self::Security => self.path(id)?,
            _ => format!("checks/{}/{timestamp}{id}.md", self.as_str()),
        })
    }
    fn accepts_path(self, path: &str, id: &str) -> Result<bool> {
        if path == self.path(id)? {
            return Ok(true);
        }
        let Some(stem) = path
            .strip_prefix(&format!("checks/{}/", self.as_str()))
            .and_then(|name| name.strip_suffix(".md"))
        else {
            return Ok(false);
        };
        if matches!(self, Self::Security) || stem.len() != 14 + id.len() {
            return Ok(false);
        }
        let bytes = stem.as_bytes();
        Ok(bytes[..14].iter().all(u8::is_ascii_digit) && &bytes[14..] == id.as_bytes())
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
                    && entry.kind.accepts_path(&entry.relative_path, &entry.record_id)?,
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
            if validate_id(id).is_ok() {
                return true;
            }
            let bytes = id.as_bytes();
            if bytes.len() == 14 + 26 && bytes[..14].iter().all(u8::is_ascii_digit) {
                if let Ok(suffix) = std::str::from_utf8(&bytes[14..]) {
                    if validate_id(suffix).is_ok() {
                        return true;
                    }
                }
            }
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
        if entry.outcome == "omitted" { continue; }
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
    ensure!(entry.outcome != "omitted", "check_record_omitted: record excluded by archive policy");
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

#[cfg(test)]
mod tests {
    use super::*;

    const ID_A: &str = "01M5JZP06MG4MC3S53AGJAZMX9";
    const ID_B: &str = "01M5JZP06NPVGDYDSJCWP53Q7A";

    #[test]
    fn started_paths_convert_offsets_to_utc_and_keep_same_second_ids_distinct() {
        let started_at = "2026-10-22T23:37:33-08:00";
        let first = CheckKind::Review.path_with_started_at(ID_A, started_at).unwrap();
        let second = CheckKind::Review.path_with_started_at(ID_B, started_at).unwrap();
        assert_eq!(first, format!("checks/review/20261023073733{ID_A}.md"));
        assert_eq!(second, format!("checks/review/20261023073733{ID_B}.md"));
        assert_ne!(first, second);
        assert!(valid_check_path(&first));
    }

    #[test]
    fn index_accepts_started_and_legacy_ulid_paths() {
        let index = CheckIndex {
            format_version: 1,
            entries: vec![
                CheckEntry {
                    record_id: ID_A.into(),
                    kind: CheckKind::Review,
                    created_at: "2026-10-23T07:37:33Z".into(),
                    outcome: "clean".into(),
                    relative_path: CheckKind::Review
                        .path_with_started_at(ID_A, "2026-10-23T07:37:33Z")
                        .unwrap(),
                    content_digest: "a".repeat(64),
                },
                CheckEntry {
                    record_id: ID_B.into(),
                    kind: CheckKind::Verify,
                    created_at: "2026-10-23T07:38:33Z".into(),
                    outcome: "clean".into(),
                    relative_path: CheckKind::Verify.path(ID_B).unwrap(),
                    content_digest: "b".repeat(64),
                },
            ],
        };
        index.validate().unwrap();
        assert!(!valid_check_path("checks/review/../escape.md"));
        assert!(!valid_check_path(&format!("checks/review/{}x.md", "9".repeat(14) + ID_A)));
    }
}

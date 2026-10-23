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

#[derive(Debug, Clone)]
pub struct BrowsedCheckEntry {
    pub entry: CheckEntry,
    pub read_error: Option<CheckReadError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckReadError {
    MissingCheckRecord,
    IntegrityError,
    CheckRecordReadError,
}
impl CheckReadError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingCheckRecord => "missing_check_record",
            Self::IntegrityError => "integrity_error",
            Self::CheckRecordReadError => "check_record_read_error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrowseIndex {
    pub entries: Vec<BrowsedCheckEntry>,
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

fn check_location(store: &dyn Store, change: &str) -> Result<bool> {
    ensure!(
        crate::security::prepared::valid_relative(change) && !change.contains('/'),
        "invalid_change"
    );
    let archived = !store.change_exists(change) && store.archived_change_exists(change);
    ensure!(
        store.change_exists(change) || archived,
        "change not found: {change}"
    );
    Ok(archived)
}

fn checked_read(
    store: &dyn Store,
    change: &str,
    path: &str,
    archived: bool,
) -> Result<Option<String>> {
    store.read_check_document(change, path, archived)
}

fn checked_metadata(entry: &CheckEntry, content: &str) -> Result<CheckMetadata> {
    let header = content
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("<!-- speclink-check: "))
        .and_then(|line| line.strip_suffix(" -->"))
        .ok_or_else(|| anyhow::anyhow!("integrity_error: check record header"))?;
    let metadata: CheckMetadata = serde_json::from_str(header)
        .map_err(|_| anyhow::anyhow!("integrity_error: check record metadata"))?;
    ensure!(
        metadata.created_at == entry.created_at
            && metadata.outcome == entry.outcome
            && valid_digest(&metadata.scope_digest),
        "integrity_error: check record metadata"
    );
    Ok(metadata)
}

/// Read the selected station's index entries while reporting per-document
/// problems. Index structure remains an all-or-nothing trust boundary.
pub fn browse_index(store: &dyn Store, change: &str, kind: CheckKind) -> Result<BrowseIndex> {
    ensure!(kind != CheckKind::Security, "unsupported_check_history_kind");
    let archived = check_location(store, change)?;
    if !archived {
        store.ensure_checks_readable(change)?;
    }
    let index = checked_read(store, change, INDEX, archived)?
        .map(|text| serde_json::from_str::<CheckIndex>(&text))
        .transpose()?
        .unwrap_or_default();
    index.validate()?;
    let entries = index
        .entries
        .into_iter()
        .filter(|entry| entry.kind == kind)
        .map(|entry| {
            if entry.outcome == "omitted" {
                return BrowsedCheckEntry { entry, read_error: None };
            }
            let read_error = match checked_read(store, change, &entry.relative_path, archived) {
                Err(_) => Some(CheckReadError::CheckRecordReadError),
                Ok(None) => Some(CheckReadError::MissingCheckRecord),
                Ok(Some(content)) => {
                    if sha256(content.as_bytes()) != entry.content_digest
                        || checked_metadata(&entry, &content).is_err()
                    {
                        Some(CheckReadError::IntegrityError)
                    } else {
                        None
                    }
                }
            };
            BrowsedCheckEntry { entry, read_error }
        })
        .collect();
    Ok(BrowseIndex { entries })
}

/// Read and revalidate only the selected review or verify record.
pub fn browse_record(
    store: &dyn Store,
    change: &str,
    id: &str,
    kind: CheckKind,
) -> Result<CheckRecord> {
    ensure!(kind != CheckKind::Security, "unsupported_check_history_kind");
    validate_id(id)?;
    let archived = check_location(store, change)?;
    if !archived {
        store.ensure_checks_readable(change)?;
    }
    let index = checked_read(store, change, INDEX, archived)?
        .map(|text| serde_json::from_str::<CheckIndex>(&text))
        .transpose()?
        .unwrap_or_default();
    index.validate()?;
    let entry = index
        .entries
        .iter()
        .find(|entry| entry.record_id == id && entry.kind == kind)
        .ok_or_else(|| crate::station::NotFound("check record not found".into()))?;
    ensure!(entry.outcome != "omitted", "check_record_omitted: record excluded by archive policy");
    let content = checked_read(store, change, &entry.relative_path, archived)?
        .ok_or_else(|| anyhow::anyhow!("missing_check_record"))?;
    ensure!(
        sha256(content.as_bytes()) == entry.content_digest,
        "integrity_error: check record"
    );
    let mut metadata = checked_metadata(entry, &content)?;
    metadata.actor = metadata.actor.as_deref().map(crate::security::report::redact_text);
    metadata.tool = metadata.tool.as_deref().map(crate::security::report::redact_text);
    Ok(CheckRecord {
        record_id: id.into(),
        content: crate::security::report::redact_text(&content),
        metadata,
    })
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

    fn six_per_kind_fixture() -> (crate::teststore::TestStore, Vec<String>, Vec<String>) {
        let store = crate::teststore::TestStore::with_meta("demo", "schema: spec-driven\n");
        let mut entries = Vec::new();
        let mut review_ids = Vec::new();
        let mut verify_ids = Vec::new();
        for (kind, ids) in [
            (CheckKind::Review, &mut review_ids),
            (CheckKind::Verify, &mut verify_ids),
        ] {
            for offset in 0..6u128 {
                let id = ulid::Ulid::from(1000 + offset + if kind == CheckKind::Verify { 100 } else { 0 }).to_string();
                let created_at = format!("2026-10-22T15:{:02}:00Z", offset + 10);
                let metadata = CheckMetadata {
                    created_at: created_at.clone(),
                    actor: None,
                    tool: None,
                    outcome: "clean".into(),
                    scope_digest: "a".repeat(64),
                };
                let content = format!("<!-- speclink-check: {} -->\n# {kind:?} {offset}\n", serde_json::to_string(&metadata).unwrap());
                let path = kind.path_with_started_at(&id, &created_at).unwrap();
                entries.push(CheckEntry {
                    record_id: id.clone(),
                    kind,
                    created_at,
                    outcome: "clean".into(),
                    relative_path: path.clone(),
                    content_digest: sha256(content.as_bytes()),
                });
                store.put_artifact("demo", &path, &content);
                ids.push(id);
            }
        }
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        store.put_artifact(
            "demo",
            INDEX,
            &serde_json::to_string(&CheckIndex { format_version: 1, entries }).unwrap(),
        );
        (store, review_ids, verify_ids)
    }

    #[test]
    fn browse_history_isolates_missing_and_modified_entries_while_strict_index_stays_strict() {
        let (store, review_ids, verify_ids) = six_per_kind_fixture();
        let check_index: CheckIndex = serde_json::from_str(&store.read_artifact("demo", INDEX).unwrap()).unwrap();
        let missing = check_index.entries.iter().find(|entry| entry.record_id == review_ids[1]).unwrap();
        store.delete_artifact("demo", &missing.relative_path).unwrap();
        let altered = check_index.entries.iter().find(|entry| entry.record_id == verify_ids[1]).unwrap();
        store.put_artifact("demo", &altered.relative_path, "tampered record");

        let reviews = browse_index(&store, "demo", CheckKind::Review).unwrap();
        assert_eq!(reviews.entries.len(), 6);
        assert_eq!(reviews.entries.iter().find(|row| row.entry.record_id == review_ids[1]).unwrap().read_error.map(CheckReadError::as_str), Some("missing_check_record"));
        assert!(browse_record(&store, "demo", &review_ids[5], CheckKind::Review).is_ok());
        assert!(browse_record(&store, "demo", &review_ids[1], CheckKind::Review).is_err());

        let verifies = browse_index(&store, "demo", CheckKind::Verify).unwrap();
        assert_eq!(verifies.entries.len(), 6);
        assert_eq!(verifies.entries.iter().find(|row| row.entry.record_id == verify_ids[1]).unwrap().read_error.map(CheckReadError::as_str), Some("integrity_error"));
        assert!(browse_record(&store, "demo", &verify_ids[5], CheckKind::Verify).is_ok());
        assert!(browse_record(&store, "demo", &verify_ids[1], CheckKind::Verify).is_err());
        assert!(index(&store, "demo").is_err(), "strict validation must still reject a damaged history");
    }

    #[test]
    fn damaged_history_still_blocks_both_stamps_and_archive_without_writes() {
        let (store, review_ids, _) = six_per_kind_fixture();
        store.put_artifact("demo", "tasks.md", "- [x] Done\n");
        let before_index = store.read_artifact("demo", INDEX).unwrap();
        let parsed: CheckIndex = serde_json::from_str(&before_index).unwrap();
        let missing_path = parsed.entries.iter().find(|entry| entry.record_id == review_ids[1]).unwrap().relative_path.clone();
        store.delete_artifact("demo", &missing_path).unwrap();
        let damaged_index = store.read_artifact("demo", INDEX).unwrap();
        let intact_records: Vec<_> = parsed.entries.iter().filter(|entry| entry.relative_path != missing_path).map(|entry| {
            (entry.relative_path.clone(), store.read_artifact("demo", &entry.relative_path).unwrap())
        }).collect();

        for station in [&crate::station::REVIEW, &crate::station::VERIFY] {
            crate::station::add_round(station, &store, "demo", "**Scope**: crates/a.rs\n").unwrap();
            let ticket_before = store.read_artifact("demo", station.doc).unwrap();
            let read = |_: &str| Some(b"fn a() {}\n".to_vec());
            let err = crate::station::stamp(station, &store, "demo", false, None, None, &read, &|_| true)
                .expect_err("a damaged history must block either quality stamp");
            assert!(err.to_string().contains("missing_check_record"), "strict stamp error: {err}");
            assert_eq!(store.read_artifact("demo", station.doc).as_deref(), Some(ticket_before.as_str()));
            assert_eq!(store.read_artifact("demo", INDEX).as_deref(), Some(damaged_index.as_str()));
            assert!(!store.artifact_exists("demo", &missing_path));
        }

        let meta_before = store.meta("demo");
        let change = crate::model::find_change(&store, "demo").unwrap();
        let ws = crate::workspace::Workspace {
            root: std::env::temp_dir().join("speclink-damaged-history-archive-ghost-root"),
            spec_dir_name: "openspec".into(),
        };
        let opts = crate::lifecycle::archive::ArchiveOptions {
            security_acceptance: None,
            skip_specs: true,
            carry_review: true,
            carry_verify: true,
            no_validate: true,
            mark_tasks_complete: false,
        };
        let err = crate::lifecycle::archive::archive(&ws, &store, &change, &opts, None)
            .expect_err("the same damaged history must block archive");
        assert!(err.to_string().contains("missing_check_record"), "archive error: {err}");
        assert!(store.change_exists("demo"));
        assert!(!store.archived_change_exists(&format!("{}-demo", crate::util::today())));
        assert_eq!(store.meta("demo"), meta_before);
        assert_eq!(store.read_artifact("demo", INDEX).as_deref(), Some(damaged_index.as_str()));
        assert!(!store.artifact_exists("demo", &missing_path));
        for (path, content) in intact_records {
            assert_eq!(store.read_artifact("demo", &path).as_deref(), Some(content.as_str()), "record changed: {path}");
        }
    }

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

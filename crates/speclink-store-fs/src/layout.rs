//! The on-disk layout of the filesystem TeamStore driver: what lives where,
//! how logical identity becomes a file name, and the version marker that
//! gates every open.
//!
//! One data directory holds the whole store. At its root sit the meta file
//! (driver identity and schema version) and the lock file; every scope gets
//! one subdirectory carrying `index.json` — the scope's single source of
//! truth — plus the `documents`, `history` and `outbox` directories of
//! immutable, sequence-named record files. Nothing in a file name or
//! timestamp carries meaning the index does not already fix: names exist to
//! be looked up, never to be sorted by mtime.
//!
//! The directory is a driver-private format. Names are escaped rather than
//! readable so that arbitrary project, repo and document identifiers cannot
//! collide, escape their directory, or alias each other on a
//! case-insensitive filesystem.

use speclink_store::{DocumentId, Scope, StoreError};
use std::path::{Path, PathBuf};

/// The schema version this driver reads and writes. A directory recording a
/// higher version is refused (fail closed); a lower one reports needing
/// migration. Bump this and add a migrate path when the shape changes.
pub const SCHEMA_VERSION: u32 = 1;

/// The store-identity marker written to the meta file at initialization. Its
/// presence distinguishes a speclink FS store from an unrelated directory.
pub const STORE_MARKER: &str = "speclink-team-store-fs";

/// The root meta file: driver identity and schema version.
pub const META_FILE: &str = "meta.json";

/// The root lock file backing the single-writer advisory lock.
pub const LOCK_FILE: &str = "lock";

/// A scope's index file — the one and only atomic publish point of a commit.
pub const INDEX_FILE: &str = "index.json";

/// Staging name of a new index, renamed over [`INDEX_FILE`] to publish.
pub const INDEX_STAGING_FILE: &str = "index.json.new";

pub const DOCUMENTS_DIR: &str = "documents";
pub const HISTORY_DIR: &str = "history";
pub const OUTBOX_DIR: &str = "outbox";

/// Escape one identifier component into a file-name-safe token.
///
/// Only `a-z`, `0-9`, `-` and `_` survive; everything else becomes `%XX` of
/// its UTF-8 bytes. Escaping uppercase is deliberate: on a case-insensitive
/// filesystem `repo-A` and `repo-a` would otherwise be the same directory.
/// Escaping `.` and `/` is what keeps a component from acting as a separator
/// or climbing out of its directory.
fn escape(component: &str) -> String {
    let mut out = String::with_capacity(component.len());
    for byte in component.as_bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => out.push(*byte as char),
            other => out.push_str(&format!("%{other:02x}")),
        }
    }
    out
}

/// Reverse of [`escape`]. A token that is not a valid escaping is persisted
/// corruption, not an absence.
fn unescape(token: &str) -> Result<String, StoreError> {
    let corrupt = || StoreError::Corrupt {
        reason: format!("undecodable name component: {token:?}"),
    };
    let bytes = token.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                let hex = token.get(i + 1..i + 3).ok_or_else(corrupt)?;
                out.push(u8::from_str_radix(hex, 16).map_err(|_| corrupt())?);
                i += 3;
            }
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {
                out.push(bytes[i]);
                i += 1;
            }
            _ => return Err(corrupt()),
        }
    }
    String::from_utf8(out).map_err(|_| corrupt())
}

/// The directory name of a scope: escaped project and repo, `.`-joined.
/// Unambiguous because [`escape`] leaves no literal `.` in a component.
pub fn scope_dir(scope: &Scope) -> String {
    format!(
        "{}.{}",
        escape(scope.project.as_str()),
        escape(scope.repo.as_str())
    )
}

/// The project a scope directory belongs to, without decoding the repo.
/// Used to find the sibling scopes of one project.
pub fn scope_dir_project(name: &str) -> Result<String, StoreError> {
    let (project, _repo) = name.split_once('.').ok_or_else(|| StoreError::Corrupt {
        reason: format!("scope directory is not a project.repo pair: {name:?}"),
    })?;
    unescape(project)
}

/// The stable key of a document inside an index and in file names: a short
/// tag followed by its identifying fields, `.`-joined. Deterministic and
/// reversible by [`decode_doc_key`].
pub fn doc_key(doc: &DocumentId) -> String {
    match doc {
        DocumentId::ChangeMeta { change } => format!("cm.{}", escape(change)),
        DocumentId::ChangeArtifact { change, artifact } => {
            format!("ca.{}.{}", escape(change), escape(artifact))
        }
        DocumentId::CanonicalSpec { capability } => format!("cs.{}", escape(capability)),
        DocumentId::Discussion { slug, archived } => {
            format!("di.{}.{}", u8::from(*archived), escape(slug))
        }
        DocumentId::WorkflowConfig => "wc".to_string(),
        DocumentId::ArchivedChange { change, doc } => {
            format!("ac.{}.{}", escape(change), escape(doc))
        }
        DocumentId::Language => "lg".to_string(),
        DocumentId::BoardOrder => "bo".to_string(),
    }
}

/// Reverse of [`doc_key`]. A key outside the closed set of shapes is
/// persisted corruption, surfaced as [`StoreError::Corrupt`].
pub fn decode_doc_key(key: &str) -> Result<DocumentId, StoreError> {
    let corrupt = || StoreError::Corrupt {
        reason: format!("undecodable document key: {key:?}"),
    };
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["wc"] => Ok(DocumentId::WorkflowConfig),
        ["lg"] => Ok(DocumentId::Language),
        ["bo"] => Ok(DocumentId::BoardOrder),
        ["cm", change] => Ok(DocumentId::ChangeMeta {
            change: unescape(change)?,
        }),
        ["cs", capability] => Ok(DocumentId::CanonicalSpec {
            capability: unescape(capability)?,
        }),
        ["ca", change, artifact] => Ok(DocumentId::ChangeArtifact {
            change: unescape(change)?,
            artifact: unescape(artifact)?,
        }),
        ["ac", change, doc] => Ok(DocumentId::ArchivedChange {
            change: unescape(change)?,
            doc: unescape(doc)?,
        }),
        ["di", flag, slug] => Ok(DocumentId::Discussion {
            slug: unescape(slug)?,
            archived: match *flag {
                "1" => true,
                "0" => false,
                _ => return Err(corrupt()),
            },
        }),
        _ => Err(corrupt()),
    }
}

/// The content file of one document revision: immutable once written, which
/// is what lets a snapshot keep reading its own fixed point while later
/// commits publish new files beside it.
pub fn content_file(doc_key: &str, revision: u64) -> String {
    format!("{doc_key}.{revision}")
}

/// The revision a content file name carries, or `None` when the name is not
/// one this driver wrote.
pub fn content_file_revision(name: &str) -> Option<u64> {
    name.rsplit_once('.')?.1.parse().ok()
}

/// The history record file of one document revision.
pub fn history_file(doc_key: &str, revision: u64) -> String {
    format!("{doc_key}.{revision}.json")
}

/// The revision a history file name carries, or `None` when the name is not
/// one this driver wrote.
pub fn history_file_revision(name: &str, doc_key: &str) -> Option<u64> {
    name.strip_suffix(".json")?
        .strip_prefix(&format!("{doc_key}."))?
        .parse()
        .ok()
}

/// The revision of any history file name, without knowing its document.
pub fn any_history_revision(name: &str) -> Option<u64> {
    let stem = name.strip_suffix(".json")?;
    stem.rsplit_once('.')?.1.parse().ok()
}

/// The record file of one outbox sequence number.
pub fn outbox_file(seq: u64) -> String {
    format!("{seq}.json")
}

/// The sequence number an outbox file name carries.
pub fn outbox_file_seq(name: &str) -> Option<u64> {
    name.strip_suffix(".json")?.parse().ok()
}

/// Absolute paths of one scope's directories, derived from the data root.
pub struct ScopePaths {
    pub dir: PathBuf,
}

impl ScopePaths {
    pub fn new(root: &Path, scope: &Scope) -> Self {
        Self {
            dir: root.join(scope_dir(scope)),
        }
    }

    pub fn index(&self) -> PathBuf {
        self.dir.join(INDEX_FILE)
    }

    pub fn index_staging(&self) -> PathBuf {
        self.dir.join(INDEX_STAGING_FILE)
    }

    pub fn documents(&self) -> PathBuf {
        self.dir.join(DOCUMENTS_DIR)
    }

    pub fn history(&self) -> PathBuf {
        self.dir.join(HISTORY_DIR)
    }

    pub fn outbox(&self) -> PathBuf {
        self.dir.join(OUTBOX_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_store::{ProjectId, RepoId};

    fn scope(project: &str, repo: &str) -> Scope {
        Scope::new(ProjectId::new(project), RepoId::new(repo))
    }

    #[test]
    fn document_keys_round_trip_across_the_closed_set() {
        let docs = [
            DocumentId::ChangeMeta {
                change: "add-auth".into(),
            },
            DocumentId::ChangeArtifact {
                change: "add-auth".into(),
                artifact: "specs/auth/spec.md".into(),
            },
            DocumentId::CanonicalSpec {
                capability: "auth".into(),
            },
            DocumentId::Discussion {
                slug: "auth-scope".into(),
                archived: false,
            },
            DocumentId::Discussion {
                slug: "auth-scope".into(),
                archived: true,
            },
            DocumentId::WorkflowConfig,
            DocumentId::ArchivedChange {
                change: "old".into(),
                doc: "proposal.md".into(),
            },
            DocumentId::Language,
        ];
        for doc in &docs {
            let key = doc_key(doc);
            assert_eq!(&decode_doc_key(&key).unwrap(), doc, "round trip of {key}");
        }
        // Distinct identities never share a key — including the two
        // Discussion states, which differ only by the archived flag.
        let keys: std::collections::BTreeSet<String> = docs.iter().map(doc_key).collect();
        assert_eq!(keys.len(), docs.len());
    }

    #[test]
    fn names_escape_separators_traversal_and_case() {
        // A document id is arbitrary user text. None of it may become a path
        // separator, a parent hop, or a case-folded alias of its neighbour.
        let sneaky = DocumentId::ChangeArtifact {
            change: "../../etc".into(),
            artifact: "specs/auth/spec.md".into(),
        };
        let key = doc_key(&sneaky);
        assert!(!key.contains('/'), "no path separator survives: {key}");
        assert!(!key.contains(".."), "no parent hop survives: {key}");
        assert_eq!(decode_doc_key(&key).unwrap(), sneaky);

        // Case-insensitive filesystems: two ids differing only by case must
        // not collide on disk.
        let upper = doc_key(&DocumentId::CanonicalSpec {
            capability: "Auth".into(),
        });
        let lower = doc_key(&DocumentId::CanonicalSpec {
            capability: "auth".into(),
        });
        assert_ne!(upper.to_lowercase(), lower.to_lowercase());

        // The same holds for scope directories.
        assert_ne!(
            scope_dir(&scope("acme", "Web")).to_lowercase(),
            scope_dir(&scope("acme", "web")).to_lowercase()
        );
        let dir = scope_dir(&scope("../evil", "web"));
        assert!(!dir.contains('/') && !dir.contains(".."), "{dir}");
    }

    #[test]
    fn a_scope_directory_names_its_project() {
        // Finding one project's sibling scopes reads the project back out of
        // the directory name; a repo containing the separator's escape must
        // not confuse it.
        assert_eq!(scope_dir_project(&scope_dir(&scope("acme", "web"))).unwrap(), "acme");
        assert_eq!(
            scope_dir_project(&scope_dir(&scope("a.b", "c.d"))).unwrap(),
            "a.b"
        );
        assert!(scope_dir_project("no-separator").is_err());
    }

    #[test]
    fn record_file_names_carry_their_sequence() {
        let key = doc_key(&DocumentId::CanonicalSpec {
            capability: "auth".into(),
        });
        assert_eq!(content_file(&key, 3), "cs.auth.3");
        assert_eq!(content_file_revision("cs.auth.3"), Some(3));
        assert_eq!(content_file_revision("cs.auth"), None);
        assert_eq!(history_file(&key, 3), "cs.auth.3.json");
        assert_eq!(history_file_revision("cs.auth.3.json", &key), Some(3));
        assert_eq!(any_history_revision("cs.auth.3.json"), Some(3));

        // A neighbouring document whose key merely starts the same must not
        // be mistaken for this one's history.
        assert_eq!(history_file_revision("cs.authx.3.json", &key), None);
        assert_eq!(history_file_revision("cs.auth.json", &key), None);

        assert_eq!(outbox_file(7), "7.json");
        assert_eq!(outbox_file_seq("7.json"), Some(7));
        assert_eq!(outbox_file_seq("seven.json"), None);
    }

    #[test]
    fn corrupt_keys_are_failures_not_guesses() {
        for key in ["", "zz.x", "cs", "cs.a.b.c", "cs.%zz", "cs.%", "cs.AUTH"] {
            assert!(
                matches!(decode_doc_key(key), Err(StoreError::Corrupt { .. })),
                "{key:?} should decode as corrupt"
            );
        }
    }
}

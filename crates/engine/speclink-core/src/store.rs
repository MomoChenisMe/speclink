//! The storage seam: the engine reads and writes spec documents exclusively
//! through this interface. Implementations own the physical layout (paths,
//! directory structure, archive naming, timestamps); the engine speaks in
//! domain terms: changes, artifacts, delta/canonical specs, discussions, and
//! the workflow config.
//!
//! The trait is synchronous and object-safe on purpose — the engine carries a
//! `&dyn Store`, and the CLI (or a future SDK host) picks the implementation
//! at its assembly point. Method inventory strictly mirrors the engine's
//! current filesystem calls; nothing is added speculatively for future
//! backends.
//!
//! `PathBuf` return values are *display locations* (what payloads and human
//! output print), not an invitation to do filesystem work on them — all
//! content access goes through the trait.

use crate::model::Change;
use anyhow::Result;
use std::path::PathBuf;

/// A discussion document as stored: raw text plus its identity and location.
/// Parsing (frontmatter, rounds, sections) is engine logic and stays out of
/// the storage layer.
#[derive(Debug, Clone)]
pub struct DiscussionDoc {
    /// Slug identity. For archived documents this is derived from the stored
    /// name with the archive date prefix removed (a `-N` reuse suffix is kept,
    /// matching the listing behavior the CLI has always had).
    pub slug: String,
    pub text: String,
    /// Display location of the document.
    pub path: PathBuf,
    pub archived: bool,
}

/// Storage interface for spec documents.
///
/// Artifact identifiers (`artifact` parameters) are the schema-defined output
/// paths relative to a change — e.g. `proposal.md`, `specs/<cap>/spec.md`.
/// They are domain vocabulary (every schema names its artifacts this way),
/// not storage layout.
pub trait Store {
    // --- changes ---

    /// Active changes with parsed metadata, sorted by name. Missing storage
    /// yields an empty list.
    fn list_changes(&self) -> Vec<Change>;
    /// A single active change by name.
    fn find_change(&self, name: &str) -> Option<Change>;
    /// Whether an active change exists.
    fn change_exists(&self, name: &str) -> bool;
    /// Create a change with the given raw metadata document. Returns the
    /// change's display location.
    fn create_change(&self, name: &str, meta_text: &str) -> Result<PathBuf>;
    /// Last-modified time of a change in whole seconds since the Unix epoch —
    /// the sort key for "most recently updated" orderings. Missing change → 0.
    fn updated_at_secs(&self, name: &str) -> u64;
    /// Raw metadata document of an active change, or None when the change does
    /// not exist (symmetric with `read_archived_meta`). Stamping flows read the
    /// raw text, append, and write back — never re-serialize.
    fn read_change_meta(&self, name: &str) -> Option<String>;
    /// Overwrite the metadata document of an active change (symmetric with
    /// `write_archived_meta`).
    fn write_change_meta(&self, name: &str, content: &str) -> Result<()>;
    /// Delete an active change outright — the storage side of `discard`.
    /// Removes the change's entire document tree; a removal failure (a locked
    /// directory, a permissions error) surfaces as an error. The engine guards
    /// existence before calling, so this is never a "not found" path.
    fn delete_change(&self, name: &str) -> Result<()>;

    /// Whether this backend adjudicates ownership — the `claim` verb's
    /// precondition. Ownership is a team-system concept: a single-writer
    /// local checkout has nobody to coordinate with, so the plain fs store
    /// keeps the default `false` and `claim` refuses there as it always has.
    fn supports_ownership(&self) -> bool {
        false
    }

    // --- artifacts ---

    /// Artifact content, or None when it does not exist.
    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String>;
    /// Write (create or overwrite) an artifact. Returns its display location.
    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> Result<PathBuf>;
    /// Whether an artifact exists (an empty document counts).
    fn artifact_exists(&self, change: &str, artifact: &str) -> bool;
    /// Delete a document inside an active change — the storage side of the
    /// review ticket's discard/stamp (its only current caller). The engine
    /// guards existence before calling. Backends that cannot delete refuse
    /// loudly via this default instead of silently keeping the document.
    fn delete_artifact(&self, change: &str, artifact: &str) -> Result<()> {
        let _ = (change, artifact);
        anyhow::bail!("this store backend does not support deleting change documents")
    }

    // --- completion evidence ---

    /// The change's completion-evidence record text (the serialized
    /// [`crate::tasks::TouchedRecord`]), or None when the change carries no
    /// record. Absence is a normal state: a change whose tasks claimed no new
    /// file has nothing to record.
    fn read_evidence(&self, change: &str) -> Option<String>;
    /// Write (create or overwrite) the change's evidence record text. Every
    /// backend implements this: a store that silently dropped the write would
    /// lose the only durable answer to "which files did this task touch".
    fn write_evidence(&self, change: &str, content: &str) -> Result<()>;

    // --- delta specs ---

    /// Capability names that have a delta spec document in the change, sorted.
    fn delta_capabilities(&self, change: &str) -> Vec<String>;
    /// Whether the change has any capability container at all, even without a
    /// spec document inside (drives the "No delta specs found" warning).
    fn has_capability_dirs(&self, change: &str) -> bool;

    // --- canonical specs ---

    /// Capability names with a canonical spec, unsorted (callers sort).
    fn list_canonical_capabilities(&self) -> Vec<String>;
    /// Whether a canonical spec exists for the capability.
    fn canonical_spec_exists(&self, cap: &str) -> bool;
    /// Canonical spec content, or None when absent.
    fn read_canonical_spec(&self, cap: &str) -> Option<String>;
    /// Write (create or overwrite) a canonical spec.
    fn write_canonical_spec(&self, cap: &str, content: &str) -> Result<()>;
    /// Display location of a capability's canonical spec.
    fn canonical_spec_path(&self, cap: &str) -> PathBuf;

    // --- archive ---

    /// Archived change dated names, sorted descending. Backends that do not
    /// expose archive browsing may keep the empty default.
    fn list_archived_changes(&self) -> Vec<String> {
        Vec::new()
    }
    /// Whether an archived change with this dated name exists.
    fn archived_change_exists(&self, dated_name: &str) -> bool;
    /// Move an active change into the archive under its dated name.
    fn archive_change(&self, name: &str, dated_name: &str) -> Result<()>;
    /// Raw metadata document of an archived change.
    fn read_archived_meta(&self, dated_name: &str) -> Option<String>;
    /// Overwrite the metadata document of an archived change.
    fn write_archived_meta(&self, dated_name: &str, content: &str) -> Result<()>;
    /// Raw artifact content of an archived change, addressed by dated name +
    /// output path (e.g. `proposal.md`, `specs/<cap>/spec.md`) — the read-only
    /// counterpart of `read_artifact` for the archive. Default: no archived
    /// document browsing (backends opt in by overriding).
    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        let _ = (dated_name, artifact);
        None
    }
    /// Capability names with a delta spec inside an archived change, sorted.
    /// Default: empty (same opt-in rule as `read_archived_artifact`).
    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        let _ = dated_name;
        Vec::new()
    }

    // --- discussions ---

    /// Whether a live discussion exists for the slug.
    fn live_discussion_exists(&self, slug: &str) -> bool;
    /// Whether any archived discussion exists for the slug.
    fn archived_discussion_exists(&self, slug: &str) -> bool;
    /// Display location a live discussion has (or would have).
    fn live_discussion_path(&self, slug: &str) -> PathBuf;
    /// Live discussion content, or None when absent.
    fn read_live_discussion(&self, slug: &str) -> Option<String>;
    /// Write (create or overwrite) a live discussion. Returns its display
    /// location.
    fn write_live_discussion(&self, slug: &str, content: &str) -> Result<PathBuf>;
    /// Delete a live discussion document.
    fn delete_live_discussion(&self, slug: &str) -> Result<()>;
    /// Resolve a slug to its document: live first, then the newest archived
    /// candidate.
    fn read_discussion(&self, slug: &str) -> Option<DiscussionDoc>;
    /// All live discussions. Missing storage yields an empty list.
    fn list_live_discussions(&self) -> Vec<DiscussionDoc>;
    /// All archived discussions, ordered by stored name (archive date order).
    fn list_archived_discussions(&self) -> Vec<DiscussionDoc>;
    /// Move a live discussion into the archive, named by its creation date.
    /// Returns the stored archive name, or None when no live document exists.
    /// A name collision (same day, reused slug) must be resolved by the
    /// implementation, never an error.
    fn archive_discussion(&self, slug: &str, created: &str) -> Result<Option<String>>;

    // --- workflow config ---

    /// Raw workflow configuration document, or None when absent.
    fn read_workflow_config(&self) -> Option<String>;

    // --- shared vocabulary ---

    /// The project's LANGUAGE document (shared vocabulary), or None when the
    /// project has none — a missing vocabulary is a normal state, not an error.
    fn read_language(&self) -> Option<String>;
}

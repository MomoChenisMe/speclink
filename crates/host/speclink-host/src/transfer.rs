//! The export and the import of one scope (design D1–D3).
//!
//! Both use only the three operations of a store port, so any store that
//! implements the port has them with no work of its own. An export is every
//! document of the scope at one scope version. An import takes an empty scope
//! only, and writes the whole bundle in one commit: there is no state in
//! between.
//!
//! The functions here ask no hook and check no permission: the host kit's
//! [`crate::kit::HostKit::export_scope`] and
//! [`crate::kit::HostKit::import_scope`] do, and the HTTP door on top of them.

use crate::documents::{check_board_order_size, invalid_value, scope_version};
use crate::port_exec::ExecError;
use crate::port_view::{parse_key, KeyKind};
use speclink_core::command::{CommandError, ErrorCode};
use speclink_protocol::query::{
    ImportBundle, ImportDocumentId, ImportDocumentOutcome, ImportReportResponse, ImportedDocument,
};
use speclink_store::content_digest;
use speclink_store_port::{
    keys, Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, PortError, ScopeId, StorePort, Timestamp, Version,
};
use std::collections::HashSet;
use std::fmt;

pub use speclink_protocol::query::{BundleDocument, ScopeBundle};

/// The bundle format this module writes and reads.
pub const FORMAT_VERSION: u32 = 2;

/// The bundle format that names each document by its kind
/// ([`from_first_version`]).
const FIRST_VERSION: u32 = 1;

/// The most keys one read of an export asks for. A host's store often turns a
/// read into one query with a list of keys, and such a list has a limit.
const READ_BATCH: usize = 200;

/// How many times an export starts again when the scope changes during it.
const EXPORT_TRIES: usize = 3;

/// The commit's command name, and the name the hooks are asked about.
pub const IMPORT_COMMAND: &str = "scope-import";

/// The name the before hook is asked about for an export.
pub const EXPORT_COMMAND: &str = "scope-export";

/// How every refusal of a format version starts. The wire has no reason of
/// its own for it, so a client that sent the second version looks for these
/// words in a `refused` answer to know it can send the first.
pub const UNSUPPORTED_VERSION: &str = "unsupported bundle format version";

/// What an import wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportReport {
    /// The scope version after the import.
    pub scope_version: Version,
    /// How many documents were written.
    pub documents: usize,
}

/// Why an export or an import failed.
#[derive(Debug)]
pub enum TransferError {
    /// The bundle is not valid. The text names the first document that is
    /// wrong and the rule it breaks; it never holds document content.
    InvalidBundle(String),
    /// The target scope holds a document: before the import, or by the time
    /// it committed.
    NotEmpty,
    /// The scope changed during every try of the export.
    Unstable,
    /// The store failed.
    Store(PortError),
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::InvalidBundle(message) => f.write_str(message),
            TransferError::NotEmpty => {
                f.write_str("import (create-new): the target scope already holds documents")
            }
            TransferError::Unstable => write!(
                f,
                "the scope changed during each of the {EXPORT_TRIES} tries of the export; try again later"
            ),
            TransferError::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TransferError {}

impl From<PortError> for TransferError {
    fn from(e: PortError) -> Self {
        TransferError::Store(e)
    }
}

/// The failure as a call through the host kit gives it: a bundle that is not
/// valid is `invalid_argv`, a target that is not empty is `refused`, and an
/// export that found no still moment is the store's temporary failure, since
/// the caller can try again.
impl From<TransferError> for ExecError {
    fn from(e: TransferError) -> ExecError {
        let command = |code| ExecError::Command(CommandError::new(code, e.to_string()));
        match e {
            TransferError::InvalidBundle(_) => command(ErrorCode::InvalidArgv),
            TransferError::NotEmpty => command(ErrorCode::Refused),
            TransferError::Unstable => ExecError::Store(PortError::Unavailable { message: e.to_string() }),
            TransferError::Store(e) => ExecError::Store(e),
        }
    }
}

fn everything() -> KeyPrefix {
    KeyPrefix::new("").expect("the empty prefix is a prefix")
}

/// The most characters of a key, a name or a parser's text an error shows.
const SHOWN_CHARS: usize = 120;

/// `text`, cut when it is long: an error never grows with what a bundle
/// holds.
pub(crate) fn brief(text: &str) -> String {
    match text.char_indices().nth(SHOWN_CHARS) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_string(),
    }
}

/// A key as an error shows it: cut when it is long, and with its control
/// characters escaped. A key is an address, not content.
fn shown(key: &str) -> String {
    format!("{:?}", brief(key))
}

/// True when `key` is one the Engine's own key builders make: it names a
/// document kind, and that kind's names make the same key again. A key that
/// only looks like a kind (`changes/archive/…`, a change the naming rules do
/// not allow) is none.
fn follows_naming_rules(key: &DocKey) -> bool {
    document_of_key(key).and_then(|doc| key_of_document(&doc)).as_ref() == Some(key)
}

// --- the export ---

/// Every document of `scope`: its key, its content and its digest, in key
/// order, with the scope version they were read at.
///
/// The documents are read in batches of [`READ_BATCH`] keys. A scope that
/// changes during the reads would give old and new content together, so the
/// scope version is compared after them, and the export starts again when it
/// moved: [`EXPORT_TRIES`] times at most, then [`TransferError::Unstable`].
/// An empty scope is a bundle with no documents.
pub fn export(port: &dyn StorePort, scope: &ScopeId) -> Result<ScopeBundle, TransferError> {
    for _ in 0..EXPORT_TRIES {
        let listing = port.list(scope, &everything())?;
        let keys: Vec<DocKey> = listing.entries.into_iter().map(|entry| entry.key).collect();
        let Some(documents) = read_in_batches(port, scope, keys)? else {
            continue;
        };
        if scope_version(port, scope)? == listing.version {
            return Ok(ScopeBundle {
                format_version: FORMAT_VERSION,
                scope_version: Some(listing.version.get()),
                documents,
            });
        }
    }
    Err(TransferError::Unstable)
}

/// The documents of `keys`, in that order, or `None` when a listed key has
/// no document any more: the scope changed after the list. A read that does
/// not answer every key is the store's failure: a bundle with a document
/// missing is no export.
fn read_in_batches(
    port: &dyn StorePort,
    scope: &ScopeId,
    keys: Vec<DocKey>,
) -> Result<Option<Vec<BundleDocument>>, PortError> {
    let mut documents = Vec::with_capacity(keys.len());
    for batch in keys.chunks(READ_BATCH) {
        let read = port.read(scope, batch)?;
        if read.len() != batch.len() {
            return Err(PortError::Backend {
                message: format!("the store's read gave {} documents for {} keys", read.len(), batch.len()),
            });
        }
        for (key, doc) in batch.iter().zip(read) {
            let Some(doc) = doc else {
                return Ok(None);
            };
            documents.push(BundleDocument {
                key: key.as_str().to_string(),
                digest: content_digest(&doc.content),
                content: doc.content,
            });
        }
    }
    Ok(Some(documents))
}

// --- the import ---

/// Write `bundle` into `scope`, which must be empty, in one commit.
///
/// Checked before the store is called: the format version, then for each
/// document that its key follows the contract's format and one of the
/// Engine's naming rules, that its digest is its content's, and that no key
/// appears twice; a board order must also be within the size its write route
/// takes. Then the scope must list no document. The commit writes
/// every document with the expectation "no document", with one
/// `scope-imported` event; a conflict there means another writer landed after
/// the list, and the answer is [`TransferError::NotEmpty`] too. Every failure
/// leaves the scope as it was.
///
/// The naming-rule check keeps arbitrary keys out of a host's store. It does
/// not look at content: a broken document is reported when it is read.
///
/// A bundle with no documents commits nothing.
pub fn import(
    port: &dyn StorePort,
    scope: &ScopeId,
    actor: &Actor,
    bundle: ScopeBundle,
    at: Timestamp,
) -> Result<ImportReport, TransferError> {
    import_with_events(port, scope, actor, bundle, at).map(|(report, _)| report)
}

/// [`import`], with the events of its commit for the after hook: none when
/// nothing was committed.
pub(crate) fn import_with_events(
    port: &dyn StorePort,
    scope: &ScopeId,
    actor: &Actor,
    bundle: ScopeBundle,
    at: Timestamp,
) -> Result<(ImportReport, Vec<Event>), TransferError> {
    if bundle.format_version != FORMAT_VERSION {
        return Err(unsupported(bundle.format_version, FORMAT_VERSION));
    }
    let mut seen = HashSet::with_capacity(bundle.documents.len());
    let mut ops = Vec::with_capacity(bundle.documents.len());
    for doc in bundle.documents {
        let refused = |rule: &dyn fmt::Display| {
            TransferError::InvalidBundle(format!("bundle document {}: {rule}", shown(&doc.key)))
        };
        let key = DocKey::new(doc.key.as_str()).map_err(|e| refused(&e))?;
        if !follows_naming_rules(&key) {
            return Err(refused(&"the key follows none of the Engine's naming rules"));
        }
        if key == keys::board_order() {
            check_board_order_size(&doc.content).map_err(|e| refused(&e))?;
        }
        if content_digest(&doc.content) != doc.digest {
            return Err(refused(&"the digest is not the digest of the content"));
        }
        if !seen.insert(key.clone()) {
            return Err(refused(&"the key appears twice"));
        }
        ops.push(Op::Put { key, content: doc.content, expected: Expected::Absent });
    }

    let listing = port.list(scope, &everything())?;
    if !listing.entries.is_empty() {
        return Err(TransferError::NotEmpty);
    }
    let documents = ops.len();
    if documents == 0 {
        return Ok((ImportReport { scope_version: listing.version, documents }, Vec::new()));
    }
    // The one event of an import: how many documents, and nothing of them.
    let events = vec![Event::new("scope-imported", serde_json::json!({ "documents": documents })).map_err(invalid_value)?];
    let commit = Commit::new(actor.clone(), IMPORT_COMMAND, at, ops, events.clone()).map_err(invalid_value)?;
    match port.commit(scope, &commit) {
        Ok(scope_version) => Ok((ImportReport { scope_version, documents }, events)),
        Err(PortError::Conflict { .. }) => Err(TransferError::NotEmpty),
        Err(e) => Err(TransferError::Store(e)),
    }
}

fn unsupported(sent: u32, supported: u32) -> TransferError {
    TransferError::InvalidBundle(format!("{UNSUPPORTED_VERSION} {sent} (supported: {supported})"))
}

// --- the first version of the bundle ---

/// The key of a document the first version of the bundle names by its kind,
/// or `None` when its names cannot make a key that reads back as the same
/// document (a name with a `/`, for one).
pub fn key_of_document(doc: &ImportDocumentId) -> Option<DocKey> {
    let key = match doc {
        ImportDocumentId::ChangeMeta { change } => keys::change_meta(change),
        ImportDocumentId::ChangeEvidence { change } => keys::change_evidence(change),
        ImportDocumentId::ChangeArtifact { change, artifact } => keys::change_artifact(change, artifact),
        ImportDocumentId::CanonicalSpec { capability } => keys::spec(capability),
        ImportDocumentId::Discussion { slug, archived: false } => keys::discussion(slug),
        // The contract's builders for the two archived kinds take dated names
        // only. A store can hold older records without a date, so these keep
        // the stored name as it is.
        ImportDocumentId::ArchivedChange { change, doc } => {
            DocKey::new(format!("{}{change}/{doc}", keys::ARCHIVED_CHANGES))
        }
        ImportDocumentId::Discussion { slug, archived: true } => {
            DocKey::new(format!("{}{slug}.md", keys::ARCHIVED_DISCUSSIONS))
        }
        ImportDocumentId::WorkflowConfig => Ok(keys::config()),
        ImportDocumentId::Language => Ok(keys::language()),
        ImportDocumentId::BoardOrder => Ok(keys::board_order()),
    }
    .ok()?;
    (document_of_key(&key).as_ref() == Some(doc)).then_some(key)
}

/// The document kind a key names, or `None` when the key follows none of the
/// naming rules: the reverse of [`key_of_document`].
pub(crate) fn document_of_key(key: &DocKey) -> Option<ImportDocumentId> {
    Some(match parse_key(key.as_str())? {
        KeyKind::ChangeMeta { change } => ImportDocumentId::ChangeMeta { change: change.to_string() },
        KeyKind::ChangeEvidence { change } => ImportDocumentId::ChangeEvidence { change: change.to_string() },
        KeyKind::ChangeArtifact { change, artifact } => {
            ImportDocumentId::ChangeArtifact { change: change.to_string(), artifact: artifact.to_string() }
        }
        KeyKind::ArchivedChangeDoc { dated_name, doc } => {
            ImportDocumentId::ArchivedChange { change: dated_name.to_string(), doc: doc.to_string() }
        }
        KeyKind::Spec { capability } => ImportDocumentId::CanonicalSpec { capability: capability.to_string() },
        KeyKind::Discussion { slug } => ImportDocumentId::Discussion { slug: slug.to_string(), archived: false },
        // The stored name is the slug, as it is: a record the Engine archives
        // through the port carries its creation date, an older one does not.
        KeyKind::ArchivedDiscussion { stem } => ImportDocumentId::Discussion { slug: stem.to_string(), archived: true },
        KeyKind::Config => ImportDocumentId::WorkflowConfig,
        KeyKind::Language => ImportDocumentId::Language,
        KeyKind::BoardOrder => ImportDocumentId::BoardOrder,
    })
}

/// A first-version bundle as a bundle by key, with the same contents in the
/// same order. The first version's scope and project revision are not looked
/// at here: a door that has a binding compares the scope itself. A document
/// with no key ([`key_of_document`]) makes the bundle not valid.
pub fn from_first_version(bundle: ImportBundle) -> Result<ScopeBundle, TransferError> {
    if bundle.format_version != FIRST_VERSION {
        return Err(unsupported(bundle.format_version, FIRST_VERSION));
    }
    let documents = bundle
        .documents
        .into_iter()
        .map(|doc| match key_of_document(&doc.document) {
            Some(key) => Ok(BundleDocument { key: key.as_str().to_string(), content: doc.content, digest: doc.digest }),
            None => Err(TransferError::InvalidBundle(format!(
                "bundle document {}: its names make no key under the Engine's naming rules",
                brief(&format!("{:?}", doc.document))
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ScopeBundle { format_version: FORMAT_VERSION, scope_version: None, documents })
}

/// The answer a first-version import always had: every document the bundle
/// named, as created.
pub fn first_version_report(
    scope_version: u64,
    documents: impl IntoIterator<Item = ImportDocumentId>,
) -> ImportReportResponse {
    let documents = documents
        .into_iter()
        .map(|document| ImportedDocument { document, outcome: ImportDocumentOutcome::Created })
        .collect();
    ImportReportResponse { project_revision: scope_version, documents }
}

// --- a bundle as a door received it ---

/// A bundle a door received, by its format version.
#[derive(Debug)]
pub enum SentBundle {
    /// The first version: each document named by its kind.
    First(ImportBundle),
    /// The second version: each document named by its key.
    Keyed(ScopeBundle),
}

impl SentBundle {
    /// The bundle by key, whichever version it came in.
    pub fn into_keyed(self) -> Result<ScopeBundle, TransferError> {
        match self {
            SentBundle::First(bundle) => from_first_version(bundle),
            SentBundle::Keyed(bundle) => Ok(bundle),
        }
    }
}

/// Why a door could not read a bundle. A parse failure gives the parser's
/// text, cut to [`SHOWN_CHARS`] characters, then the line and the column. The
/// parser's text can repeat the start of a value of the wrong type, so the
/// cut is also the most of a document's content a text can hold.
#[derive(Debug)]
pub enum UnreadBundle {
    /// The text is not JSON.
    NotJson(String),
    /// JSON, but not a bundle: a field is missing, or is one no bundle has.
    WrongShape(String),
    /// A format version that is neither of the two.
    UnknownVersion(u32),
}

impl fmt::Display for UnreadBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnreadBundle::NotJson(e) | UnreadBundle::WrongShape(e) => write!(f, "the bundle is not valid: {e}"),
            UnreadBundle::UnknownVersion(sent) => {
                write!(f, "{UNSUPPORTED_VERSION} {sent} (supported: {FIRST_VERSION}, {FORMAT_VERSION})")
            }
        }
    }
}

/// Read the JSON text of a bundle: the format version first, then the shape
/// that version has. The text is parsed twice and never copied whole, and
/// both shapes refuse a field they do not have (a `mode`, for one).
pub fn read_bundle(json: &[u8]) -> Result<SentBundle, UnreadBundle> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Versioned {
        format_version: u32,
    }
    fn parse<'j, T: serde::Deserialize<'j>>(json: &'j [u8]) -> Result<T, UnreadBundle> {
        serde_json::from_slice(json).map_err(|e| {
            // The parser's text ends with the position: the cut keeps it.
            let (text, position) = (e.to_string(), format!(" at line {} column {}", e.line(), e.column()));
            let told = format!("{}{position}", brief(text.strip_suffix(position.as_str()).unwrap_or(&text)));
            match e.classify() {
                serde_json::error::Category::Data => UnreadBundle::WrongShape(told),
                _ => UnreadBundle::NotJson(told),
            }
        })
    }
    // A bundle is a JSON object. The two shapes would also read an array of
    // their fields' values, so another JSON value stops here.
    if json.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
        parse::<serde::de::IgnoredAny>(json)?;
        return Err(UnreadBundle::WrongShape("a bundle is a JSON object".to_string()));
    }
    match parse::<Versioned>(json)?.format_version {
        FIRST_VERSION => parse(json).map(SentBundle::First),
        FORMAT_VERSION => parse(json).map(SentBundle::Keyed),
        other => Err(UnreadBundle::UnknownVersion(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bundle_is_read_by_its_format_version() {
        let keyed = br#"{"formatVersion":2,"scopeVersion":7,"documents":[]}"#;
        assert!(matches!(read_bundle(keyed), Ok(SentBundle::Keyed(b)) if b.scope_version == Some(7)));
        let first = br#"{"formatVersion":1,"scope":{"project":"p","repo":"r"},"projectRevision":0,"documents":[]}"#;
        assert!(matches!(read_bundle(first), Ok(SentBundle::First(b)) if b.scope.repo == "r"));

        let unknown = br#"{"formatVersion":3,"documents":[]}"#;
        assert!(matches!(read_bundle(unknown), Err(UnreadBundle::UnknownVersion(3))));
        for wrong_shape in [
            &br#"{"documents":[]}"#[..],
            br#"{"formatVersion":2}"#,
            br#"{"formatVersion":2,"documents":[],"mode":"overwrite"}"#,
            br#"{"formatVersion":1,"documents":[]}"#,
            br#"{"formatVersion":"2","documents":[]}"#,
            br#""a bundle""#,
            b"null",
            // A bundle is an object: the fields' values in an array are none.
            b"[2,null,[]]",
            b"[2]",
            b"[]",
            b" 7",
        ] {
            let result = read_bundle(wrong_shape);
            assert!(matches!(result, Err(UnreadBundle::WrongShape(_))), "{}: {result:?}", String::from_utf8_lossy(wrong_shape));
        }
        for not_json in [&b"{formatVersion: 2"[..], b"", b"\xff"] {
            assert!(matches!(read_bundle(not_json), Err(UnreadBundle::NotJson(_))));
        }
    }

    #[test]
    fn a_parse_failure_cuts_the_parsers_text_and_keeps_the_position() {
        let text = format!("{{\"formatVersion\":\"{}\"}}", "v".repeat(1000));
        let told = read_bundle(text.as_bytes()).expect_err("not a bundle").to_string();
        assert!(told.ends_with("at line 1 column 1019"), "{told}");
        assert!(told.chars().count() < 2 * SHOWN_CHARS, "{told}");
    }

    #[test]
    fn a_body_of_deep_nesting_is_refused_and_not_followed() {
        // The parser stops at its depth limit: a small body cannot make it
        // recurse without end.
        let deep = format!("{{\"formatVersion\":2,\"documents\":{}", "[".repeat(100_000));
        assert!(read_bundle(deep.as_bytes()).is_err());
    }

    #[test]
    fn a_key_in_an_error_is_escaped_and_cut() {
        assert_eq!(shown("config.yaml"), "\"config.yaml\"");
        assert_eq!(shown("a\nb"), "\"a\\nb\"");
        let long = "é".repeat(600);
        let cut = shown(&long);
        assert_eq!(cut.chars().count(), SHOWN_CHARS + 3, "{cut}");
        assert!(cut.ends_with("…\""));
    }

    #[test]
    fn a_key_follows_the_naming_rules_only_when_the_builders_make_it() {
        let follows = |text: &str| follows_naming_rules(&DocKey::new(text).unwrap());
        for key in [
            "changes/add-x/.meta.yaml",
            "changes/add-x/specs/auth/spec.md",
            "archive/changes/2026-11-10-add-x/.evidence.json",
            // An older archived record with no date in its name.
            "archive/changes/add-x/proposal.md",
            "archive/discussions/reuse.md",
            "specs/auth/spec.md",
            "discussions/talk.md",
            "config.yaml",
            "LANGUAGE.md",
            "board/order",
        ] {
            assert!(follows(key), "{key}");
        }
        for key in ["changes/archive/add-x/proposal.md", "secrets/token", "specs/auth/notes.md", "board/other", "config.yml"] {
            assert!(!follows(key), "{key}");
        }
    }
}

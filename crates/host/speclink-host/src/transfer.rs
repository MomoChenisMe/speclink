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
use crate::port_view::{parse_key, KeyKind, EVIDENCE_FILE};
use speclink_core::command::{CommandError, ErrorCode};
use speclink_core::model::{require_valid_meta, ChangeMeta, LEGACY_META_FILE, META_FILE};
use speclink_core::store::Store;
use speclink_core::workspace::Workspace;
use speclink_protocol::query::{
    BoardOrderDoc, ImportBundle, ImportBundleDocument, ImportDocumentId, ImportDocumentOutcome, ImportReportResponse,
    ImportedDocument,
};
use crate::projection::{content_digest, is_plain_relative};
use speclink_store_port::{
    keys, Actor, Commit, DocKey, Event, Expected, KeyPrefix, Op, PortError, ScopeId, StorePort, Timestamp, Version,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

pub use speclink_protocol::query::{BundleDocument, ScopeBundle};

/// The bundle format this module writes and reads.
pub const FORMAT_VERSION: u32 = 2;

/// The bundle format that names each document by its kind
/// ([`from_first_version`]).
pub const FIRST_VERSION: u32 = 1;

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
pub fn document_of_key(key: &DocKey) -> Option<ImportDocumentId> {
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

// --- the documents of a local project ---

/// Every document of the local project `workspace`, read through `store`,
/// each named by its kind and in kind order: the documents of a first-version
/// bundle. Speclink Desktop's move to a server and the Node engine's export
/// both take this one list, so the two give the same documents.
///
/// It lists each active change's data file, the artifacts its schema names
/// and its evidence; the canonical specs; the live and the archived
/// discussions; every document of each archived change, its evidence
/// included; the workflow config; the language document; and the board
/// order, made of the rank keys of the changes and the discussions (none when
/// no rank key is set). A change's schema is resolved in the project, then in
/// `user_dir`, the user-level config directory.
///
/// Every source is read before the list is given: a change (active or
/// archived) whose data file does not parse, or whose schema names an
/// artifact pattern that cannot be listed, stops it, and the text names the
/// file or the pattern.
///
/// Only a spec folder inside the project is read (engine-export-scope D6):
/// before anything is read, the spec folder's name must be one or more plain
/// names. An absolute path, a drive or a network path, a `..` or a `.`
/// segment, or an empty name stops the list, and the text names it as it is
/// written. The name comes from the project's own `.speclink.yaml` (or from
/// the caller): a checkout could name a folder of another project with it.
///
/// No file is read through a symbolic link (engine-export-scope D5). Before
/// any document is read, the folders on the way to the spec folder, the
/// spec folder and everything under it are looked at without following
/// links, and so is the older evidence folder (`.speclink/touched/`), which
/// the store reads when a change has no evidence in its folder. The first
/// link stops the list: its text names the link by its path from the
/// project root, never what it points at. A folder there that cannot be
/// listed stops it too: it could hold a link the store reads by a known
/// name. A file store follows links, so a link in a checkout would put the
/// content of a file outside the project into a bundle. Whole folders are
/// looked at, not only the files that become documents: the store gives no
/// file paths, and a link of no document stops the list too.
///
/// What the sharp-edges audit found (engine-export-scope 3.1):
///
/// - It only reads. It writes no file, and it calls no store write.
/// - A document's name in a bundle is its kind and the names the store
///   lists, never a file path. Its key is made by [`key_of_document`], which
///   uses the key builders of the store contract and reads the key back; a
///   name that makes no key stops [`from_first_version`]. So a file name in
///   a project cannot make a key outside the naming rules, and an import
///   checks each key again.
/// - Any failure above is the whole answer: no part of the list is given,
///   and so no part of a bundle. A file that is there but cannot be read
///   (no permission, or not UTF-8) is no failure: the store reads nothing
///   from it, so the list has no such document.
/// - It reads the files one by one, with no snapshot: a write to the same
///   project during the export can leave the list with old and new content
///   together. The Node SDK's `exportScope` tells the integrator not to
///   write to the project while it exports.
/// - The list holds the full text of the workflow config and of every
///   discussion. Who may run an export is the caller's decision: the Node
///   SDK's `exportScope` says so to the integrator.
/// - A large project makes a large bundle, and an import refuses one over
///   its size limit (`limits.importBytes` of a Node host): the Node SDK's
///   `exportScope` and its guide say so.
/// - A folder that is not a project, or has no spec folder, gives an empty
///   list, as the engine's other reads give nothing there: the caller
///   checks how many documents it got.
pub fn local_documents(
    workspace: &Workspace,
    store: &dyn Store,
    user_dir: &Path,
) -> Result<Vec<ImportBundleDocument>, String> {
    if !inside_the_project(&workspace.spec_dir_name) {
        return Err(format!(
            "spec folder outside the project: \"{}\" — spec_dir must name a folder under the project root",
            workspace.spec_dir_name
        ));
    }
    check_for_links(&workspace.root, Path::new(&workspace.spec_dir_name))?;
    // A change with no evidence in its folder is read from the older place,
    // outside the spec folder: the same rule holds there.
    let older_evidence = workspace.touched_dir();
    check_for_links(&workspace.root, older_evidence.strip_prefix(&workspace.root).expect("the work folder is under the root"))?;
    let mut documents = BTreeMap::<ImportDocumentId, String>::new();

    let changes = store.list_changes();
    for change in &changes {
        require_valid_meta(store, change).map_err(|error| error.to_string())?;
        let meta = store.read_change_meta(&change.name).unwrap_or_default();
        insert_local_document(&mut documents, ImportDocumentId::ChangeMeta { change: change.name.clone() }, meta)?;
        let schema = resolve_local_schema(workspace, user_dir, &change.meta.schema_name())?;
        for artifact in local_artifact_paths(&schema, store.delta_capabilities(&change.name))? {
            if let Some(content) = store.read_artifact(&change.name, &artifact) {
                insert_local_document(
                    &mut documents,
                    ImportDocumentId::ChangeArtifact { change: change.name.clone(), artifact },
                    content,
                )?;
            }
        }
        if let Some(content) = store.read_evidence(&change.name) {
            insert_local_document(&mut documents, ImportDocumentId::ChangeEvidence { change: change.name.clone() }, content)?;
        }
    }

    let mut capabilities = store.list_canonical_capabilities();
    capabilities.sort();
    for capability in capabilities {
        if let Some(content) = store.read_canonical_spec(&capability) {
            insert_local_document(&mut documents, ImportDocumentId::CanonicalSpec { capability }, content)?;
        }
    }

    for discussion in store.list_live_discussions() {
        insert_local_document(
            &mut documents,
            ImportDocumentId::Discussion { slug: discussion.slug, archived: false },
            discussion.text,
        )?;
    }
    for discussion in store.list_archived_discussions() {
        insert_local_document(
            &mut documents,
            ImportDocumentId::Discussion { slug: discussion.slug, archived: true },
            discussion.text,
        )?;
    }

    for dated_name in store.list_archived_changes() {
        let meta = store.read_archived_meta(&dated_name).unwrap_or_default();
        let parsed = ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
            // The new file name is read first: when it is not there, the
            // broken one is the legacy file.
            let dir = format!("{}/changes/archive/{dated_name}", workspace.spec_dir_name);
            let file = if workspace.root.join(&dir).join(META_FILE).is_file() { META_FILE } else { LEGACY_META_FILE };
            format!("invalid {dir}/{file}: {reason}")
        })?;
        // Read new name first, then the legacy one; listed as .meta.yaml
        // always (desktop-spec-dir-migration D5).
        insert_local_document(
            &mut documents,
            ImportDocumentId::ArchivedChange { change: dated_name.clone(), doc: META_FILE.to_string() },
            meta,
        )?;
        let schema = resolve_local_schema(workspace, user_dir, &parsed.schema_name())?;
        let mut paths = local_artifact_paths(&schema, store.archived_delta_capabilities(&dated_name))?;
        paths.push(EVIDENCE_FILE.to_string());
        for artifact in paths {
            if let Some(content) = store.read_archived_artifact(&dated_name, &artifact) {
                insert_local_document(
                    &mut documents,
                    ImportDocumentId::ArchivedChange { change: dated_name.clone(), doc: artifact },
                    content,
                )?;
            }
        }
    }

    if let Some(content) = store.read_workflow_config() {
        insert_local_document(&mut documents, ImportDocumentId::WorkflowConfig, content)?;
    }
    if let Some(content) = store.read_language() {
        insert_local_document(&mut documents, ImportDocumentId::Language, content)?;
    }

    // A local board order lives in the rank keys of each change and each
    // discussion; a store holds it as one document. A change's data file is
    // listed as it is, its rank key kept: a store does not read it there, and
    // it is still there when the project moves back.
    let board = BoardOrderDoc {
        changes: store.board_ranks(&changes),
        discussions: speclink_core::discuss::list_discussions(store)
            .into_iter()
            .filter_map(|info| Some((info.slug, info.head.board_rank?)))
            .collect(),
    };
    if !board.changes.is_empty() || !board.discussions.is_empty() {
        let content = serde_json::to_string(&board).map_err(|error| error.to_string())?;
        insert_local_document(&mut documents, ImportDocumentId::BoardOrder, content)?;
    }

    Ok(documents
        .into_iter()
        .map(|(document, content)| ImportBundleDocument { digest: content_digest(&content), document, content })
        .collect())
}

/// True when the spec folder `name` is one or more plain names: no root, no
/// drive, no `..` and no `.`. A path drops a `.` that is not its first
/// segment, so the segments of the text are looked at too.
fn inside_the_project(name: &str) -> bool {
    is_plain_relative(name) && !name.split(['/', '\\']).any(|segment| segment == ".")
}

/// Stops at the first symbolic link on the way to `relative` under `root`,
/// at `relative` itself, or below it: the folders on the way first, then the
/// entries of each real folder in name order, depth first. A folder there
/// that cannot be listed stops it too: it could hold a link the store reads
/// by a known name. An entry that is not there is no stop. The text names the
/// entry by its path from `root`, with forward slashes, and never what a link
/// points at.
fn check_for_links(root: &Path, relative: &Path) -> Result<(), String> {
    let parts: Vec<_> = relative.components().collect();
    let mut on_the_way = PathBuf::new();
    for part in &parts[..parts.len().saturating_sub(1)] {
        on_the_way.push(part);
        if std::fs::symlink_metadata(root.join(&on_the_way)).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(link_found(&on_the_way));
        }
    }
    check_below(root, relative)
}

fn check_below(root: &Path, relative: &Path) -> Result<(), String> {
    let path = root.join(relative);
    let Ok(meta) = std::fs::symlink_metadata(&path) else {
        return Ok(());
    };
    if meta.file_type().is_symlink() {
        return Err(link_found(relative));
    }
    if !meta.is_dir() {
        return Ok(());
    }
    let unlisted =
        |error: std::io::Error| format!("cannot list {} to check it for symbolic links: {error}", shown_path(relative));
    let mut names = std::fs::read_dir(&path)
        .map_err(unlisted)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(unlisted)?;
    names.sort();
    names.into_iter().try_for_each(|name| check_below(root, &relative.join(name)))
}

fn link_found(relative: &Path) -> String {
    format!("symbolic link in the spec folder: {} — replace it with the file itself or remove it", shown_path(relative))
}

/// `relative` with forward slashes, as an error shows a path.
fn shown_path(relative: &Path) -> String {
    let parts: Vec<_> = relative.components().map(|part| part.as_os_str().to_string_lossy()).collect();
    parts.join("/")
}

fn resolve_local_schema(workspace: &Workspace, user_dir: &Path, name: &str) -> Result<speclink_core::schema::Schema, String> {
    speclink_core::schema::resolve_with(Some(workspace), Some(user_dir), name)
        .ok_or_else(|| speclink_core::schema::not_found_msg(name))?
}

fn local_artifact_paths(schema: &speclink_core::schema::Schema, delta_capabilities: Vec<String>) -> Result<Vec<String>, String> {
    let mut paths = BTreeSet::new();
    for artifact in &schema.artifacts {
        if artifact.output_path == "specs/**/*.md" {
            paths.extend(delta_capabilities.iter().map(|capability| format!("specs/{capability}/spec.md")));
        } else if artifact.output_path.contains('*') {
            return Err(format!(
                "cannot enumerate migration artifact pattern '{}' in schema '{}'",
                artifact.output_path, schema.name
            ));
        } else {
            paths.insert(artifact.output_path.clone());
        }
    }
    Ok(paths.into_iter().collect())
}

fn insert_local_document(
    documents: &mut BTreeMap<ImportDocumentId, String>,
    document: ImportDocumentId,
    content: String,
) -> Result<(), String> {
    if documents.insert(document.clone(), content).is_some() {
        return Err(format!("duplicate local migration document: {document:?}"));
    }
    Ok(())
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

    // --- the documents of a local project (engine-export-scope D1, D3) ---

    use speclink_fs::FsStore;
    use speclink_protocol::query::ImportScope;
    use tempfile::TempDir;

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture parent")).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// A local project in the legacy `openspec/` folder with one document of
    /// every kind but evidence and the board order.
    fn complete_project() -> TempDir {
        let root = tempfile::tempdir().unwrap();
        for (relative, content) in [
            ("openspec/config.yaml", "schema: spec-driven\nlocale: tw\n"),
            ("openspec/LANGUAGE.md", "TeamStore: 團隊儲存\n"),
            ("openspec/changes/active-change/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-21\n"),
            ("openspec/changes/active-change/proposal.md", "## Why\n\nActive proposal.\n"),
            ("openspec/changes/active-change/design.md", "## Context\n\nActive design.\n"),
            ("openspec/changes/active-change/tasks.md", "- [ ] 1.1 Active task\n"),
            ("openspec/changes/active-change/specs/payments/spec.md", "## ADDED Requirements\n\n### Requirement: Active payment\n"),
            ("openspec/specs/accounts/spec.md", "# accounts Specification\n\nCanonical accounts.\n"),
            (
                "openspec/discussions/live-plan.md",
                "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n",
            ),
            (
                "openspec/discussions/archive/2026-07-20-old-plan.md",
                "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nArchived discussion.\n",
            ),
            ("openspec/changes/archive/2026-07-20-old-change/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-20\n"),
            ("openspec/changes/archive/2026-07-20-old-change/proposal.md", "## Why\n\nArchived proposal.\n"),
            ("openspec/changes/archive/2026-07-20-old-change/tasks.md", "- [x] 1.1 Archived task\n"),
            (
                "openspec/changes/archive/2026-07-20-old-change/specs/payments/spec.md",
                "## ADDED Requirements\n\n### Requirement: Archived payment\n",
            ),
        ] {
            write(root.path(), relative, content);
        }
        root
    }

    /// The documents of the project at `root`, with an empty folder as the
    /// user-level config directory: no schema of the machine leaks in.
    fn documents_of(root: &Path) -> Result<Vec<ImportBundleDocument>, String> {
        let workspace = Workspace::discover(root).unwrap().expect("a local project");
        documents_in(&workspace.root, &workspace.spec_dir_name)
    }

    /// The documents of the project at `root` whose spec folder is named
    /// `spec_dir`, as `.speclink.yaml` or `createEngine` would name it.
    fn documents_in(root: &Path, spec_dir: &str) -> Result<Vec<ImportBundleDocument>, String> {
        let workspace = Workspace { root: root.to_path_buf(), spec_dir_name: spec_dir.to_string() };
        let store = FsStore::new(&workspace.root, &workspace.spec_dir_name);
        let user_dir = tempfile::tempdir().unwrap();
        local_documents(&workspace, &store, user_dir.path())
    }

    fn content_of<'d>(documents: &'d [ImportBundleDocument], id: &ImportDocumentId) -> Option<&'d str> {
        documents.iter().find(|doc| &doc.document == id).map(|doc| doc.content.as_str())
    }

    /// `documents` as the first version of a bundle, for [`from_first_version`].
    fn first_version(documents: Vec<ImportBundleDocument>) -> ImportBundle {
        ImportBundle {
            format_version: FIRST_VERSION,
            scope: ImportScope { project: "demo".into(), repo: "backend".into() },
            project_revision: 0,
            documents,
        }
    }

    const EVIDENCE: &str = "{\"version\":2,\"change\":\"active-change\",\"entries\":[{\"taskId\":\"tsk_1\",\"taskDesc\":\"1.1 Active task\",\"touchedFiles\":[\"src/a.rs\"],\"recordedAt\":\"2026-07-21T10:00:00Z\"}]}\n";
    const ARCHIVED_EVIDENCE: &str = "{\"version\":2,\"change\":\"old-change\",\"entries\":[]}\n";

    #[test]
    fn every_document_of_a_local_project_is_listed_with_its_exact_content() {
        let root = complete_project();
        let documents = documents_of(root.path()).expect("the documents are listed");
        for doc in &documents {
            assert_eq!(doc.digest, content_digest(&doc.content), "{:?}", doc.document);
        }

        let listed: BTreeMap<ImportDocumentId, String> =
            documents.into_iter().map(|doc| (doc.document, doc.content)).collect();
        let change = |artifact: &str| ImportDocumentId::ChangeArtifact { change: "active-change".into(), artifact: artifact.into() };
        let archived = |doc: &str| ImportDocumentId::ArchivedChange { change: "2026-07-20-old-change".into(), doc: doc.into() };
        let expected = BTreeMap::from([
            (ImportDocumentId::ChangeMeta { change: "active-change".into() }, "schema: spec-driven\ncreated: 2026-07-21\n"),
            (change("proposal.md"), "## Why\n\nActive proposal.\n"),
            (change("design.md"), "## Context\n\nActive design.\n"),
            (change("tasks.md"), "- [ ] 1.1 Active task\n"),
            (change("specs/payments/spec.md"), "## ADDED Requirements\n\n### Requirement: Active payment\n"),
            (ImportDocumentId::CanonicalSpec { capability: "accounts".into() }, "# accounts Specification\n\nCanonical accounts.\n"),
            (
                ImportDocumentId::Discussion { slug: "live-plan".into(), archived: false },
                "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\n---\n\nLive discussion.\n",
            ),
            (
                ImportDocumentId::Discussion { slug: "old-plan".into(), archived: true },
                "---\ntopic: Old plan\nslug: old-plan\nstatus: concluded\ncreated: 2026-07-20\n---\n\nArchived discussion.\n",
            ),
            // The legacy data file of an archived change is listed under
            // .meta.yaml (desktop-spec-dir-migration D5).
            (archived(".meta.yaml"), "schema: spec-driven\ncreated: 2026-07-20\n"),
            (archived("proposal.md"), "## Why\n\nArchived proposal.\n"),
            (archived("tasks.md"), "- [x] 1.1 Archived task\n"),
            (archived("specs/payments/spec.md"), "## ADDED Requirements\n\n### Requirement: Archived payment\n"),
            (ImportDocumentId::WorkflowConfig, "schema: spec-driven\nlocale: tw\n"),
            (ImportDocumentId::Language, "TeamStore: 團隊儲存\n"),
        ])
        .into_iter()
        .map(|(id, content)| (id, content.to_string()))
        .collect::<BTreeMap<_, _>>();
        assert_eq!(listed, expected);
    }

    #[test]
    fn the_evidence_of_an_active_and_of_an_archived_change_is_listed() {
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.evidence.json", EVIDENCE);
        write(root.path(), "openspec/changes/archive/2026-07-20-old-change/.evidence.json", ARCHIVED_EVIDENCE);

        let documents = documents_of(root.path()).expect("the documents are listed");

        let active = ImportDocumentId::ChangeEvidence { change: "active-change".into() };
        assert_eq!(content_of(&documents, &active), Some(EVIDENCE));
        let archived = ImportDocumentId::ArchivedChange { change: "2026-07-20-old-change".into(), doc: ".evidence.json".into() };
        assert_eq!(content_of(&documents, &archived), Some(ARCHIVED_EVIDENCE));
    }

    #[test]
    fn the_rank_keys_of_changes_and_discussions_become_the_board_order_document() {
        let root = complete_project();
        let ranked_meta = "schema: spec-driven\ncreated: 2026-07-21\nboard_rank: f\n";
        write(root.path(), "openspec/changes/active-change/.openspec.yaml", ranked_meta);
        write(root.path(), "openspec/changes/second-change/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-21\nboard_rank: b\n");
        write(root.path(), "openspec/changes/second-change/proposal.md", "## Why\n\nSecond.\n");
        write(root.path(), "openspec/changes/unranked-change/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-21\n");
        write(
            root.path(),
            "openspec/discussions/live-plan.md",
            "---\ntopic: Live plan\nslug: live-plan\nstatus: open\ncreated: 2026-07-21\nboard_rank: n\n---\n\nLive discussion.\n",
        );

        let documents = documents_of(root.path()).expect("the documents are listed");

        let board = BoardOrderDoc::from_content(content_of(&documents, &ImportDocumentId::BoardOrder));
        let ranks = |pairs: &[(&str, &str)]| -> BTreeMap<String, String> {
            pairs.iter().map(|(name, rank)| (name.to_string(), rank.to_string())).collect()
        };
        assert_eq!(board.changes, ranks(&[("active-change", "f"), ("second-change", "b")]));
        assert_eq!(board.discussions, ranks(&[("live-plan", "n")]));
        // A change's own data file is listed as it is, its rank key kept.
        let meta = ImportDocumentId::ChangeMeta { change: "active-change".into() };
        assert_eq!(content_of(&documents, &meta), Some(ranked_meta));
    }

    #[test]
    fn a_project_with_no_rank_key_has_no_board_order_document() {
        let root = complete_project();
        let documents = documents_of(root.path()).expect("the documents are listed");
        assert_eq!(content_of(&documents, &ImportDocumentId::BoardOrder), None);
    }

    #[test]
    fn the_documents_by_key_have_the_same_content_in_the_same_order() {
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.evidence.json", EVIDENCE);
        let documents = documents_of(root.path()).expect("the documents are listed");

        let keyed = from_first_version(first_version(documents.clone())).expect("every document has a key");

        assert_eq!((keyed.format_version, keyed.scope_version), (2, None));
        let mut keys: Vec<&str> = keyed.documents.iter().map(|doc| doc.key.as_str()).collect();
        keys.sort_unstable();
        let mut expected = vec![
            "LANGUAGE.md",
            "archive/changes/2026-07-20-old-change/.meta.yaml",
            "archive/changes/2026-07-20-old-change/proposal.md",
            "archive/changes/2026-07-20-old-change/specs/payments/spec.md",
            "archive/changes/2026-07-20-old-change/tasks.md",
            "archive/discussions/old-plan.md",
            "changes/active-change/.evidence.json",
            "changes/active-change/.meta.yaml",
            "changes/active-change/design.md",
            "changes/active-change/proposal.md",
            "changes/active-change/specs/payments/spec.md",
            "changes/active-change/tasks.md",
            "config.yaml",
            "discussions/live-plan.md",
            "specs/accounts/spec.md",
        ];
        expected.sort_unstable();
        assert_eq!(keys, expected);
        for (by_key, by_kind) in keyed.documents.iter().zip(&documents) {
            assert_eq!((&by_key.content, &by_key.digest), (&by_kind.content, &by_kind.digest), "{}", by_key.key);
        }
    }

    #[test]
    fn each_document_has_the_key_its_kind_makes_and_every_kind_is_covered() {
        // spec workspace-migration「本機專案的匯出與遷移產生相同的文件集合」: the
        // two doors read one list, and its kinds all have a key.
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.evidence.json", EVIDENCE);
        write(root.path(), "openspec/changes/archive/2026-07-20-old-change/.evidence.json", ARCHIVED_EVIDENCE);
        write(root.path(), "openspec/changes/active-change/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-21\nboard_rank: f\n");
        let documents = documents_of(root.path()).expect("the documents are listed");

        let keyed = from_first_version(first_version(documents.clone())).expect("every document has a key");

        assert_eq!(keyed.documents.len(), documents.len());
        for (by_key, by_kind) in keyed.documents.iter().zip(&documents) {
            let key = key_of_document(&by_kind.document).expect("the kind makes a key");
            assert_eq!(by_key.key, key.as_str(), "{:?}", by_kind.document);
        }
        let kind = |id: &ImportDocumentId| match id {
            ImportDocumentId::ChangeMeta { .. } => "change meta",
            ImportDocumentId::ChangeArtifact { .. } => "change artifact",
            ImportDocumentId::ChangeEvidence { .. } => "change evidence",
            ImportDocumentId::CanonicalSpec { .. } => "canonical spec",
            ImportDocumentId::Discussion { archived: false, .. } => "live discussion",
            ImportDocumentId::Discussion { archived: true, .. } => "archived discussion",
            ImportDocumentId::WorkflowConfig => "workflow config",
            ImportDocumentId::ArchivedChange { .. } => "archived change",
            ImportDocumentId::Language => "language",
            ImportDocumentId::BoardOrder => "board order",
        };
        let kinds: BTreeSet<&str> = documents.iter().map(|doc| kind(&doc.document)).collect();
        assert_eq!(kinds.len(), 10, "{kinds:?}");
    }

    #[test]
    fn a_document_whose_names_make_no_key_stops_the_keyed_bundle_and_is_named() {
        let root = complete_project();
        let mut documents = documents_of(root.path()).expect("the documents are listed");
        documents[0].document = ImportDocumentId::CanonicalSpec { capability: "a/b".into() };
        let error = from_first_version(first_version(documents)).expect_err("no key").to_string();
        assert!(error.contains("a/b"), "{error}");
    }

    #[test]
    fn corrupt_change_metadata_stops_the_listing_and_names_the_file() {
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.openspec.yaml", "schema: [unterminated\n");

        let error = documents_of(root.path()).expect_err("corrupt metadata must fail closed");
        assert!(error.contains("openspec/changes/active-change/.openspec.yaml"), "{error}");
    }

    #[test]
    fn a_corrupt_legacy_archived_data_file_is_named_as_it_is_on_disk() {
        let root = complete_project();
        write(root.path(), "openspec/changes/archive/2026-07-20-old-change/.openspec.yaml", "schema: [unterminated\n");

        let error = documents_of(root.path()).expect_err("corrupt metadata must fail closed");
        assert!(error.contains("openspec/changes/archive/2026-07-20-old-change/.openspec.yaml"), "{error}");
    }

    #[test]
    fn an_archived_change_with_both_data_files_lists_only_the_new_one() {
        let root = complete_project();
        write(root.path(), "openspec/changes/archive/2026-07-20-old-change/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-19\n");

        let documents = documents_of(root.path()).expect("the documents are listed");

        let metas: Vec<(&str, &str)> = documents
            .iter()
            .filter_map(|doc| match &doc.document {
                ImportDocumentId::ArchivedChange { doc: name, .. } if name.ends_with(".yaml") => {
                    Some((name.as_str(), doc.content.as_str()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(metas, [(".meta.yaml", "schema: spec-driven\ncreated: 2026-07-19\n")]);
    }

    // --- a symbolic link in the spec folder (engine-export-scope D5) ---

    /// The content outside the project that a link points at: no error may
    /// show it.
    #[cfg(unix)]
    const OUTSIDE: &str = "TOP SECRET\n";

    /// The error of a local project in `speclink/` whose `link` (a path under
    /// the project root) is a symbolic link to a file or a folder outside the
    /// project. The folder outside holds one change, so a project whose spec
    /// folder is the link is still a project.
    #[cfg(unix)]
    fn error_with_link(link: &str, to_folder: bool) -> String {
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "secret.md", OUTSIDE);
        write(outside.path(), "folder/changes/hidden/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-21\n");
        write(outside.path(), "folder/changes/hidden/proposal.md", OUTSIDE);
        let root = tempfile::tempdir().unwrap();
        if link != "speclink" {
            write(root.path(), "speclink/changes/demo/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-21\n");
            write(root.path(), "speclink/changes/demo/proposal.md", "## Why\n\nDemo.\n");
        }
        let target = if to_folder { outside.path().join("folder") } else { outside.path().join("secret.md") };
        let at = root.path().join(link);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&target, &at).unwrap();

        let error = documents_of(root.path()).expect_err("a link stops the listing");
        assert!(!error.contains(OUTSIDE.trim()), "{error}");
        error
    }

    #[cfg(unix)]
    fn link_message(path: &str) -> String {
        format!("symbolic link in the spec folder: {path} — replace it with the file itself or remove it")
    }

    #[test]
    #[cfg(unix)]
    fn a_file_in_the_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let error = error_with_link("speclink/specs/leak/spec.md", false);
        assert_eq!(error, link_message("speclink/specs/leak/spec.md"));
    }

    #[test]
    #[cfg(unix)]
    fn a_folder_in_the_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let error = error_with_link("speclink/changes/linked", true);
        assert_eq!(error, link_message("speclink/changes/linked"));
    }

    #[test]
    #[cfg(unix)]
    fn a_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let error = error_with_link("speclink", true);
        assert_eq!(error, link_message("speclink"));
    }

    // --- a spec folder outside the project (engine-export-scope D6) ---

    /// The content of a spec folder outside the project: no error may show it.
    const OTHER: &str = "OTHER PROJECT SECRET\n";

    fn outside_message(name: &str) -> String {
        format!("spec folder outside the project: \"{name}\" — spec_dir must name a folder under the project root")
    }

    #[test]
    fn a_spec_folder_above_the_project_stops_the_listing_and_is_named() {
        let base = tempfile::tempdir().unwrap();
        write(base.path(), "other/speclink/specs/private/spec.md", OTHER);
        std::fs::create_dir(base.path().join("project")).unwrap();

        let error = documents_in(&base.path().join("project"), "../other/speclink").expect_err("outside the project");

        assert_eq!(error, outside_message("../other/speclink"));
    }

    #[test]
    fn a_spec_folder_named_by_an_absolute_path_stops_the_listing_and_is_named() {
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "speclink/specs/private/spec.md", OTHER);
        let root = tempfile::tempdir().unwrap();
        let name = outside.path().join("speclink").to_string_lossy().into_owned();

        let error = documents_in(root.path(), &name).expect_err("outside the project");

        assert_eq!(error, outside_message(&name));
    }

    #[test]
    fn a_spec_folder_in_a_folder_of_the_project_is_listed() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "docs/specs/specs/auth/spec.md", "# auth\n");

        let documents = documents_in(root.path(), "docs/specs").expect("inside the project");

        let auth = ImportDocumentId::CanonicalSpec { capability: "auth".into() };
        assert_eq!(content_of(&documents, &auth), Some("# auth\n"));
    }

    #[test]
    fn an_empty_name_and_a_dot_segment_stop_the_listing() {
        // Contract D6: an empty name, `.` and `..` are no plain names either.
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "specs/auth/spec.md", "# auth\n");
        // A path drops a `.` that is not its first segment: `speclink/.` is
        // refused by its text.
        for name in ["", ".", "speclink/..", "speclink/."] {
            assert_eq!(documents_in(root.path(), name).expect_err(name), outside_message(name));
        }
    }

    // --- links on the way to the spec folder, and the older evidence (quality round 1) ---

    #[test]
    #[cfg(unix)]
    fn a_folder_on_the_way_to_the_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "specs/specs/private/spec.md", OUTSIDE);
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("docs")).unwrap();

        let error = documents_in(root.path(), "docs/specs").expect_err("a link on the way stops the listing");

        assert_eq!(error, link_message("docs"));
    }

    #[test]
    #[cfg(unix)]
    fn a_link_in_the_folder_of_the_older_evidence_stops_the_listing_and_is_named() {
        // A change with no evidence in its folder is read from the older
        // place outside the spec folder, `.speclink/touched/<change>.json`.
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "secret.json", OUTSIDE);
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "speclink/changes/demo/.meta.yaml", "schema: spec-driven\ncreated: 2026-07-21\n");
        std::fs::create_dir_all(root.path().join(".speclink/touched")).unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret.json"), root.path().join(".speclink/touched/demo.json"))
            .unwrap();

        let error = documents_of(root.path()).expect_err("a link in the older evidence folder stops the listing");

        assert_eq!(error, link_message(".speclink/touched/demo.json"));
    }

    #[test]
    #[cfg(unix)]
    fn a_folder_in_the_spec_folder_that_cannot_be_listed_stops_the_listing() {
        // A folder that cannot be listed could hold a link the store reads by
        // a known name: the export stops instead of guessing.
        use std::os::unix::fs::PermissionsExt;
        let root = complete_project();
        let locked = root.path().join("openspec/specs/accounts");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o111)).unwrap();
        let listable = std::fs::read_dir(&locked).is_ok();

        let result = documents_of(root.path());

        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        if listable {
            return; // A privileged user lists any folder: nothing to check here.
        }
        let error = result.expect_err("a folder that cannot be listed stops the listing");
        assert!(error.starts_with("cannot list openspec/specs/accounts to check it for symbolic links:"), "{error}");
    }
}

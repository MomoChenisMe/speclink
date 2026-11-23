//! The export and the import of one scope (design D1–D3).
//!
//! Both use only the three operations of a store port, so any store that
//! implements the port has them with no work of its own. An export is every
//! document of the scope at one scope version. An import has two modes
//! ([`ImportMode`]): it takes an empty scope only, or it makes a scope that
//! holds documents equal to the bundle. Both write in one commit: there is no
//! state in between.
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

/// The one event of an import's commit. It is no Engine event: the host kit
/// adds it to its event names ([`crate::kit::event_names`]).
pub const IMPORTED_EVENT: &str = "scope-imported";

/// How every refusal of a format version starts. The wire has no reason of
/// its own for it, so a client that sent the second version looks for these
/// words in a `refused` answer to know it can send the first.
pub const UNSUPPORTED_VERSION: &str = "unsupported bundle format version";

/// What an import does with a scope that holds documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportMode {
    /// Take an empty scope only, and write every document of the bundle.
    #[default]
    Create,
    /// Make the scope equal the bundle: add, update and delete documents.
    Replace,
}

impl ImportMode {
    /// The mode as the before hook's params give it, and as a door names it.
    pub fn as_str(self) -> &'static str {
        match self {
            ImportMode::Create => "create",
            ImportMode::Replace => "replace",
        }
    }

    /// The mode a door named by the text of [`ImportMode::as_str`], or `None`
    /// for any other text.
    pub fn parse(text: &str) -> Option<ImportMode> {
        [ImportMode::Create, ImportMode::Replace].into_iter().find(|mode| mode.as_str() == text)
    }
}

/// How an import runs. The default is the import as it always was: the
/// create mode, and a real run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportOptions {
    pub mode: ImportMode,
    /// Do every check, the list and the compare, and report what a real run
    /// would do; commit nothing.
    pub dry_run: bool,
}

/// What an import wrote, or what its dry run would write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    /// The scope version after the commit; with no commit (a dry run, or
    /// nothing to write or delete), the version the scope was listed at.
    pub scope_version: Version,
    /// How many documents were written: the added and the changed ones.
    pub documents: usize,
    /// The keys the scope did not hold, in byte order. The create mode gives
    /// every key of the bundle here.
    pub added: Vec<String>,
    /// The keys both sides hold with different content, in byte order.
    pub changed: Vec<String>,
    /// The keys only the scope held, in byte order.
    pub removed: Vec<String>,
}

/// Why an export or an import failed, as a fixed code a host can show its
/// own text for (spec node-sdk: 匯出與匯入的失敗帶原因代碼). The code comes
/// from the kind of the failure, never from its text. A failure with no code
/// here has none: a host takes "no reason" as a valid answer, and a code
/// added later goes only to a failure that has none yet.
///
/// What the sharp-edges audit found (sdk-command-names-and-transfer-reasons
/// 3.3), for this enum, [`TransferError::reason`], the split variants and
/// [`LocalExportError`]:
///
/// - Can a caller's input decide the code? No. The enum is closed and
///   [`TransferReason::as_str`] is a fixed match, so no text of a bundle, a
///   key, a file name or a hook reaches the code. [`TransferError::reason`]
///   reads the variant only. [`LocalExportError`] gets a code only from its
///   private constructors, at the failures of [`local_documents`] that have
///   one: a spec folder outside the project, a link, a folder that cannot be
///   listed, a data file that does not parse, and a schema that cannot be
///   used (the last two for an active and for an archived change). Its one
///   public constructor, `From<String>`, gives no code. A variant a crate
///   user builds by hand gets the code of its kind, whatever its text.
/// - Does the source of a command failure carry a store's text further? No.
///   Only the six failures with a code or a whole-bundle refusal
///   ([`TransferError::InvalidBundle`], `TooLarge`, `InvalidDocument`,
///   `NotEmpty`, `EmptyBundle`, `ScopeChanged`) become a command failure with
///   a source, and its text is the command failure's own message. A store's failure and a moving scope
///   stay store failures with no source. The readers of a command failure's
///   source (the HTTP door's error mapping, the Node binding) only ask its
///   type; none of them writes it into an answer.
/// - Does the text of [`LocalExportError`] show a path outside the project?
///   No more than before: its text is the string the function always gave.
///   A spec folder outside the project is named as it is written, a link by
///   its path from the project root and never by its target, and a folder
///   that cannot be listed by its path from the root.
/// - The edge that stays: a new failure point in [`local_documents`] that
///   returns a `String` through `?` gets no code, with no warning. That is
///   the safe side (no code is a valid answer), and the code table of the
///   Node SDK lists only the six kinds of failure of a local export that
///   have one: the five kinds here, and `no_spec_folder`, which the Node
///   binding gives before it calls [`local_documents`].
///
/// What the sharp-edges audit found (export-failure-reason-and-location
/// 2.5), for [`TransferReason::ChangeSchemaUnusable`] and the two values of
/// [`ChangeLocation`]:
///
/// - Can a change's folder name or the path of its data file decide the
///   code? No. The call site gives the code, by the check that failed: a
///   data file that does not parse is `invalid_change_metadata`, and a schema
///   that does not resolve or whose artifacts cannot be listed is
///   `change_schema_unusable`. The location is data only. No code reads it,
///   and its two values have names, so a caller cannot swap them and not see
///   it.
/// - Can the two values show a path outside the project, or the content of
///   a file? No. The change is one name a folder listing gave, so it has no
///   separator and is no `..`. The path is the spec folder's name (plain
///   names only, checked before anything is read, and put together again
///   with forward slashes), fixed segments, the change's name and one of two
///   fixed file names. No file is read to make either value, and neither is
///   an absolute path.
/// - What does a long name, or a name with control characters, look like?
///   As it is on disk, in the text and in the two values: not escaped and
///   not cut. A file system limits one name (255 bytes on the common ones),
///   and bytes that are not UTF-8 become U+FFFD. The text of a data file
///   that does not parse always had the same name in its path. The values
///   cross to JS as JSON strings, so they cannot break the envelope. A host
///   that shows them must show them as plain text: the Node SDK's guide says
///   so.
/// - Is a new failure point with no location on the safe side? Yes. Only
///   the two private constructors that take a [`ChangeLocation`] give one,
///   and they are called only in the two loops of [`local_documents`], for
///   an active and for an archived change. A failure made any other way has
///   no location, and a host takes "no location" as a valid answer.
/// - The edge that stays, and that this change did not make: the text of
///   `change_schema_unusable` is the text the failure always had, after the
///   change's name. It holds the schema name as the data file has it, with
///   no limit on its length. The engine's schema lookup joins that name to
///   the schema folders with no check of its shape, as it does for every
///   verb: a name with `..` segments can name a `schema.yaml` outside them
///   when the project has a `schemas/` folder, and the parser's text for a
///   definition that does not load can quote a value of that file. The
///   artifacts a schema names are read by the same kind of join, but a name
///   that makes no key stops [`from_first_version`] before a bundle exists,
///   and its text shows names only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferReason {
    /// The root of a local export holds no spec folder.
    NoSpecFolder,
    /// The spec folder's name points out of the project.
    SpecFolderOutsideProject,
    /// A symbolic link on the way to the spec folder, at it or below it, or
    /// in the older evidence folder (`.speclink/touched/`).
    SymlinkInSpecFolder,
    /// A folder below the spec folder, or the older evidence folder, cannot
    /// be listed.
    SpecFolderUnreadable,
    /// The data file of an active or an archived change does not parse.
    InvalidChangeMetadata,
    /// The schema an active or an archived change names cannot be used here:
    /// it is not found, its definition does not load, or it names an artifact
    /// pattern that cannot be listed.
    ChangeSchemaUnusable,
    /// The target scope of an import in the create mode holds documents.
    ScopeNotEmpty,
    /// The bundle of an import in the replace mode holds no document.
    EmptyBundle,
    /// During an import in the replace mode, another writer wrote or deleted
    /// a key that import writes or deletes.
    ScopeChanged,
    /// The content of an import is over the host's limit.
    BundleTooLarge,
    /// A document of an import breaks a rule of its own.
    InvalidBundleDocument,
}

impl TransferReason {
    /// The code as the Node SDK gives it in `error.reason`.
    pub fn as_str(self) -> &'static str {
        match self {
            TransferReason::NoSpecFolder => "no_spec_folder",
            TransferReason::SpecFolderOutsideProject => "spec_folder_outside_project",
            TransferReason::SymlinkInSpecFolder => "symlink_in_spec_folder",
            TransferReason::SpecFolderUnreadable => "spec_folder_unreadable",
            TransferReason::InvalidChangeMetadata => "invalid_change_metadata",
            TransferReason::ChangeSchemaUnusable => "change_schema_unusable",
            TransferReason::ScopeNotEmpty => "scope_not_empty",
            TransferReason::EmptyBundle => "empty_bundle",
            TransferReason::ScopeChanged => "scope_changed",
            TransferReason::BundleTooLarge => "bundle_too_large",
            TransferReason::InvalidBundleDocument => "invalid_bundle_document",
        }
    }
}

/// Why an export or an import failed.
#[derive(Debug)]
pub enum TransferError {
    /// The bundle is not valid as a whole: a format version this module does
    /// not take, or a first-version document whose names make no key.
    InvalidBundle(String),
    /// The bundle's content is over the import limit. The text gives the two
    /// sizes.
    TooLarge(String),
    /// A document of the bundle breaks a rule: its key, its digest, a key
    /// that appears twice, or a board order over its size. The text names
    /// the first such document and the rule; it never holds document
    /// content.
    InvalidDocument(String),
    /// The create mode's target scope holds a document: before the import,
    /// or by the time it committed.
    NotEmpty,
    /// The replace mode got a bundle with no documents: it would remove every
    /// document of the scope.
    EmptyBundle,
    /// During a replace, after the list, another writer wrote or deleted a
    /// key the replace writes or deletes. Nothing was written.
    ScopeChanged,
    /// The scope changed during every try of the export.
    Unstable,
    /// The store failed.
    Store(PortError),
}

impl TransferError {
    /// The failure's code, when it has one.
    pub fn reason(&self) -> Option<TransferReason> {
        match self {
            TransferError::TooLarge(_) => Some(TransferReason::BundleTooLarge),
            TransferError::InvalidDocument(_) => Some(TransferReason::InvalidBundleDocument),
            TransferError::NotEmpty => Some(TransferReason::ScopeNotEmpty),
            TransferError::EmptyBundle => Some(TransferReason::EmptyBundle),
            TransferError::ScopeChanged => Some(TransferReason::ScopeChanged),
            TransferError::InvalidBundle(_) | TransferError::Unstable | TransferError::Store(_) => None,
        }
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::InvalidBundle(message)
            | TransferError::TooLarge(message)
            | TransferError::InvalidDocument(message) => f.write_str(message),
            TransferError::NotEmpty => {
                f.write_str("import (create-new): the target scope already holds documents")
            }
            TransferError::EmptyBundle => f.write_str(
                "import (replace): the bundle holds no documents — a replace with it would remove every document of the scope",
            ),
            TransferError::ScopeChanged => {
                f.write_str("import (replace): the target scope changed during the import — run it again")
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
/// valid (or, for a replace, holds no documents) is `invalid_argv`, a target
/// that is not empty or that changed during a replace is `refused`, and an
/// export that found no still moment is the store's temporary failure, since
/// the caller can try again. A command failure keeps the transfer failure as
/// its source, so a door can ask it for its [`TransferError::reason`].
impl From<TransferError> for ExecError {
    fn from(e: TransferError) -> ExecError {
        let command = |code, e: TransferError| {
            ExecError::Command(CommandError { code, message: e.to_string(), source: Some(anyhow::Error::new(e)) })
        };
        match e {
            TransferError::InvalidBundle(_)
            | TransferError::TooLarge(_)
            | TransferError::InvalidDocument(_)
            | TransferError::EmptyBundle => command(ErrorCode::InvalidArgv, e),
            TransferError::NotEmpty | TransferError::ScopeChanged => command(ErrorCode::Refused, e),
            TransferError::Unstable => ExecError::Store(PortError::Unavailable { message: e.to_string() }),
            TransferError::Store(e) => ExecError::Store(e),
        }
    }
}

/// The code of a command failure that came from a transfer (the conversion
/// above keeps it as the source). Any other command failure has none.
pub fn reason_of(failure: &CommandError) -> Option<TransferReason> {
    failure.source.as_ref()?.downcast_ref::<TransferError>()?.reason()
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
        let Some(contents) = read_in_batches(port, scope, &keys)? else {
            continue;
        };
        if scope_version(port, scope)? == listing.version {
            let documents = keys
                .iter()
                .zip(contents)
                .map(|(key, content)| BundleDocument {
                    key: key.as_str().to_string(),
                    digest: content_digest(&content),
                    content,
                })
                .collect();
            return Ok(ScopeBundle {
                format_version: FORMAT_VERSION,
                scope_version: Some(listing.version.get()),
                documents,
            });
        }
    }
    Err(TransferError::Unstable)
}

/// The content of each of `keys`, in that order and in batches of
/// [`READ_BATCH`] keys, or `None` when a listed key has no document any more:
/// the scope changed after the list. A read that does not answer every key is
/// the store's failure: a bundle with a document missing is no export.
fn read_in_batches(port: &dyn StorePort, scope: &ScopeId, keys: &[DocKey]) -> Result<Option<Vec<String>>, PortError> {
    let mut contents = Vec::with_capacity(keys.len());
    for batch in keys.chunks(READ_BATCH) {
        let read = port.read(scope, batch)?;
        if read.len() != batch.len() {
            return Err(PortError::Backend {
                message: format!("the store's read gave {} documents for {} keys", read.len(), batch.len()),
            });
        }
        for doc in read {
            let Some(doc) = doc else {
                return Ok(None);
            };
            contents.push(doc.content);
        }
    }
    Ok(Some(contents))
}

// --- the import ---

/// Write `bundle` into `scope`, which must be empty, in one commit: the
/// create mode of [`import_with`], and a real run.
pub fn import(
    port: &dyn StorePort,
    scope: &ScopeId,
    actor: &Actor,
    bundle: ScopeBundle,
    at: Timestamp,
) -> Result<ImportReport, TransferError> {
    import_with(port, scope, actor, bundle, at, ImportOptions::default()).map(|(report, _)| report)
}

/// Write `bundle` into `scope` in one commit, in the mode `options` names.
/// It gives the report, and the events of its commit for the after hook:
/// none when nothing was committed.
///
/// Checked before the store is called, in both modes: the format version,
/// then for each document that its key follows the contract's format and one
/// of the Engine's naming rules, that its digest is its content's, and that
/// no key appears twice; a board order must also be within the size its write
/// route takes. The naming-rule check keeps arbitrary keys out of a host's
/// store. It does not look at content: a broken document is reported when it
/// is read.
///
/// The create mode ([`ImportMode::Create`]): the scope must list no document.
/// The commit writes every document with the expectation "no document", with
/// one `scope-imported` event (`documents`); a conflict there means another
/// writer landed after the list, and the answer is
/// [`TransferError::NotEmpty`] too. A bundle with no documents commits
/// nothing.
///
/// The replace mode ([`ImportMode::Replace`]) makes the scope equal the
/// bundle. A bundle with no documents is refused before the store is called
/// ([`TransferError::EmptyBundle`]). Then the scope is listed, and the keys
/// both sides hold are read in batches and compared with the bundle:
///
/// - a key only the bundle holds is written, with the expectation "no
///   document";
/// - a key both hold with different content is written, with the expectation
///   "the listed version";
/// - a key both hold with the same content gets no operation, so its version
///   and its modified time stay;
/// - a key only the scope holds is deleted, with the expectation "the listed
///   version", whatever document it is: the workflow config, a change or a
///   discussion made after the first import, a key outside the naming rules.
///
/// All of it is one commit with one `scope-imported` event (`documents` and
/// `removed`). With nothing to write or delete there is no commit and no
/// event. A key that was listed and has no document at the read, or a
/// conflict at the commit, means another writer changed a key this replace
/// touches: the answer is [`TransferError::ScopeChanged`], and the replace
/// does not start again.
///
/// A commit states what it expects of its own keys only. So a replace does
/// not see another writer's change to a key it has no operation for: a key
/// with the same content on both sides, or a key that neither side held at
/// the list. That document stays as the other writer left it.
///
/// A dry run (`options.dry_run`) does all of the above but the commit, in
/// both modes, and its refusals are the same.
///
/// Every failure leaves the scope as it was.
///
/// What the sharp-edges audit found (import-replace-mode 3.3), for the
/// replace mode and the dry run:
///
/// - Are the defaults on the safe side? The mode is: [`ImportOptions`] with
///   no mode is the create mode, which deletes nothing, and so is a JS call
///   with no `mode`. No `dry_run` is a real run, as an import always was.
///   So a replace that is meant as a dry run and does not say so is real.
///   That edge stays: the replace mode must be named, and nothing else is
///   needed to delete.
/// - Can a misspelled field or a value of the wrong type turn a dry run
///   into a real delete? Not here: the mode is an enum and the dry run is a
///   `bool` in a struct with named fields. The JS call is where it could, so
///   the Node binding checks it before anything crosses to this crate: a
///   `mode` that is not one of the two texts and a `dryRun` key that holds
///   no boolean are refused (the text `'false'`, the number `0`, and
///   `undefined`, which is what `dryRun: body.dryRun` gives when the body
///   says `dry_run`), and a call in the replace mode with any other key is
///   refused too (`dryrun`, `dry_run`). A misspelled `mode` key is a create,
///   which a scope that holds documents refuses.
/// - How much can a bundle with no document, or with one, delete? With
///   none: nothing, [`TransferError::EmptyBundle`] before the store is
///   called. With one: every other document of the scope. There is no limit
///   on how much of a scope one replace removes: the rule is "the scope
///   equals the bundle". A local export of a spec folder that holds only a
///   `config.yaml` is such a bundle. What stands against it is the dry run,
///   whose `removed` names every key, and the host's before hook.
/// - Can a caller decide the code of a failure by what a bundle holds? No.
///   `empty_bundle` comes from the count of documents only, and
///   `scope_changed` from the store: a listed key with no document at the
///   read, or a conflict at the commit. No key and no content reaches
///   [`TransferError::reason`], and the two texts are fixed.
/// - Do the three lists show document content? No: they hold keys only.
///   A key is an address, but it names a change, a discussion or a document
///   the host put into the scope itself, and `removed` shows the keys of the
///   scope to whoever calls. So the host kit asks its before hook about a
///   dry run too. The keys are given whole, as the store listed them.
/// - What does a write the replace does not see leave? Never a part of the
///   bundle: every write and every delete is in the one commit. The scope is
///   then the bundle, but for the other writer's document: a key neither
///   side held at the list stays, and a key with the same content on both
///   sides is as the other writer left it — changed, or deleted when the
///   delete landed after the read. A dry run after the replace shows each
///   of them. A host that needs the scope to equal the bundle refuses other
///   writes in its before hook while it replaces.
/// - Does any path of the HTTP door reach the replace mode or the dry run?
///   No. The import route is the door's only caller of
///   [`crate::kit::HostKit::import_scope`], and it passes the default
///   options. [`read_bundle`] reads the body, and both bundle shapes refuse
///   a field they do not have, so a `mode` or a `dryRun` in the body is 422
///   before the hook is asked. The demo server's restore calls [`import`],
///   which is the create mode. A host's own Rust code can call this function
///   and the kit's with any options: that is what the two modes are for.
pub fn import_with(
    port: &dyn StorePort,
    scope: &ScopeId,
    actor: &Actor,
    bundle: ScopeBundle,
    at: Timestamp,
    options: ImportOptions,
) -> Result<(ImportReport, Vec<Event>), TransferError> {
    if bundle.format_version != FORMAT_VERSION {
        return Err(unsupported(bundle.format_version, FORMAT_VERSION));
    }
    let mut seen = HashSet::with_capacity(bundle.documents.len());
    let mut documents = Vec::with_capacity(bundle.documents.len());
    for doc in bundle.documents {
        let refused = |rule: &dyn fmt::Display| {
            TransferError::InvalidDocument(format!("bundle document {}: {rule}", shown(&doc.key)))
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
        documents.push((key, doc.content));
    }

    let ImportPlan { listed_at, ops, added, changed, removed, payload, on_conflict } = match options.mode {
        ImportMode::Create => plan_create(port, scope, documents)?,
        ImportMode::Replace => plan_replace(port, scope, documents)?,
    };
    let report =
        |scope_version| ImportReport { scope_version, documents: added.len() + changed.len(), added, changed, removed };
    if ops.is_empty() || options.dry_run {
        return Ok((report(listed_at), Vec::new()));
    }
    let events = vec![Event::new(IMPORTED_EVENT, payload).map_err(invalid_value)?];
    let commit = Commit::new(actor.clone(), IMPORT_COMMAND, at, ops, events.clone()).map_err(invalid_value)?;
    match port.commit(scope, &commit) {
        Ok(scope_version) => Ok((report(scope_version), events)),
        Err(PortError::Conflict { .. }) => Err(on_conflict),
        Err(e) => Err(TransferError::Store(e)),
    }
}

/// What an import commits, with the scope version it was listed at, the
/// three lists of its report (each in byte order of the keys), and the two
/// things its mode decides about the commit.
struct ImportPlan {
    listed_at: Version,
    ops: Vec<Op>,
    added: Vec<String>,
    changed: Vec<String>,
    removed: Vec<String>,
    /// The payload of the one event of an import: how many documents, and
    /// nothing of them.
    payload: serde_json::Value,
    /// What a conflict at the commit means in this mode.
    on_conflict: TransferError,
}

/// The create mode: every document is written into a scope that lists none.
fn plan_create(
    port: &dyn StorePort,
    scope: &ScopeId,
    documents: Vec<(DocKey, String)>,
) -> Result<ImportPlan, TransferError> {
    let listing = port.list(scope, &everything())?;
    if !listing.entries.is_empty() {
        return Err(TransferError::NotEmpty);
    }
    let mut added: Vec<String> = documents.iter().map(|(key, _)| key.as_str().to_string()).collect();
    added.sort_unstable();
    let ops = documents.into_iter().map(|(key, content)| Op::Put { key, content, expected: Expected::Absent }).collect();
    Ok(ImportPlan {
        listed_at: listing.version,
        ops,
        payload: serde_json::json!({ "documents": added.len() }),
        added,
        changed: Vec::new(),
        removed: Vec::new(),
        on_conflict: TransferError::NotEmpty,
    })
}

/// The replace mode: the operations that make the scope equal `documents`.
fn plan_replace(
    port: &dyn StorePort,
    scope: &ScopeId,
    mut documents: Vec<(DocKey, String)>,
) -> Result<ImportPlan, TransferError> {
    if documents.is_empty() {
        return Err(TransferError::EmptyBundle);
    }
    let listing = port.list(scope, &everything())?;
    let mut listed: BTreeMap<DocKey, Version> = listing.entries.into_iter().map(|entry| (entry.key, entry.version)).collect();
    documents.sort_by(|a, b| a.0.cmp(&b.0));

    let held: Vec<DocKey> = documents.iter().map(|(key, _)| key).filter(|key| listed.contains_key(key)).cloned().collect();
    let Some(stored) = read_in_batches(port, scope, &held)? else {
        return Err(TransferError::ScopeChanged);
    };
    let stored: BTreeMap<DocKey, String> = held.into_iter().zip(stored).collect();

    let (mut ops, mut added, mut changed) = (Vec::new(), Vec::new(), Vec::new());
    for (key, content) in documents {
        let (expected, reported) = match listed.remove(&key) {
            None => (Expected::Absent, &mut added),
            Some(_) if stored[&key] == content => continue,
            Some(version) => (Expected::At(version), &mut changed),
        };
        reported.push(key.as_str().to_string());
        ops.push(Op::Put { key, content, expected });
    }
    // What is left was listed and is not in the bundle.
    let removed: Vec<String> = listed.keys().map(|key| key.as_str().to_string()).collect();
    ops.extend(listed.into_iter().map(|(key, expected)| Op::Delete { key, expected }));
    Ok(ImportPlan {
        listed_at: listing.version,
        ops,
        payload: serde_json::json!({ "documents": added.len() + changed.len(), "removed": removed.len() }),
        added,
        changed,
        removed,
        on_conflict: TransferError::ScopeChanged,
    })
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

/// Which change a failure of [`local_documents`] is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLocation {
    /// The change's folder name on disk: an archived change's is its dated
    /// name.
    pub change: String,
    /// The change's data file by its path from the project root, with
    /// forward slashes only, whatever way the spec folder's name is written.
    /// It names the file that is there: the legacy name when only that one
    /// is, and the new name when neither is.
    pub path: String,
}

/// Why [`local_documents`] stopped: its text, the code of the failure when it
/// has one, and the change it is about when it is about one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExportError {
    message: String,
    reason: Option<TransferReason>,
    location: Option<ChangeLocation>,
}

impl LocalExportError {
    fn new(reason: TransferReason, message: String) -> LocalExportError {
        LocalExportError { message, reason: Some(reason), location: None }
    }

    /// A failure about one change. The code is the caller's: the location
    /// never decides it.
    fn about(location: ChangeLocation, reason: TransferReason, message: String) -> LocalExportError {
        LocalExportError { message, reason: Some(reason), location: Some(location) }
    }

    /// The schema the change at `location` names cannot be used. The text is
    /// the one this failure always had, with the change's name before it.
    fn schema_unusable(location: ChangeLocation, message: String) -> LocalExportError {
        let message = format!("change '{}': {message}", location.change);
        LocalExportError::about(location, TransferReason::ChangeSchemaUnusable, message)
    }

    /// The failure's code, when it has one.
    pub fn reason(&self) -> Option<TransferReason> {
        self.reason
    }

    /// The change the failure is about. Only a failure whose code is
    /// [`TransferReason::InvalidChangeMetadata`] or
    /// [`TransferReason::ChangeSchemaUnusable`] has one.
    pub fn location(&self) -> Option<&ChangeLocation> {
        self.location.as_ref()
    }
}

impl fmt::Display for LocalExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LocalExportError {}

/// A failure with no code and no location: a document listed twice, a board
/// order that cannot be written as JSON.
impl From<String> for LocalExportError {
    fn from(message: String) -> LocalExportError {
        LocalExportError { message, reason: None, location: None }
    }
}

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
/// Every source is read before the list is given. A change (active or
/// archived) whose data file does not parse stops it, and the text names the
/// file. So does a change whose schema cannot be used (it is not found, its
/// definition does not load, or it names an artifact pattern that cannot be
/// listed), and the text starts with the change's name: `change '<name>': `.
/// Both failures give the change and its data file
/// ([`LocalExportError::location`]).
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
) -> Result<Vec<ImportBundleDocument>, LocalExportError> {
    if !inside_the_project(&workspace.spec_dir_name) {
        return Err(LocalExportError::new(
            TransferReason::SpecFolderOutsideProject,
            format!(
                "spec folder outside the project: \"{}\" — spec_dir must name a folder under the project root",
                workspace.spec_dir_name
            ),
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
        let located = |written: &str| change_location(workspace, &change.name, written);
        require_valid_meta(store, change).map_err(|error| {
            // The text and the location name one file: the one the failure found.
            LocalExportError::about(located(&error.file), TransferReason::InvalidChangeMetadata, error.to_string())
        })?;
        let meta = store.read_change_meta(&change.name).unwrap_or_default();
        insert_local_document(&mut documents, ImportDocumentId::ChangeMeta { change: change.name.clone() }, meta)?;
        let deltas = store.delta_capabilities(&change.name);
        let artifacts = local_artifact_paths(workspace, user_dir, &change.meta.schema_name(), deltas).map_err(|message| {
            LocalExportError::schema_unusable(located(&store.change_meta_display(&change.name)), message)
        })?;
        for artifact in artifacts {
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
        let located = |written: &str| change_location(workspace, &dated_name, written);
        let meta = store.read_archived_meta(&dated_name).unwrap_or_default();
        let parsed = ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
            let written = archived_meta_path(workspace, &dated_name);
            let message = format!("invalid {written}: {reason}");
            LocalExportError::about(located(&written), TransferReason::InvalidChangeMetadata, message)
        })?;
        // Read new name first, then the legacy one; listed as .meta.yaml
        // always (desktop-spec-dir-migration D5).
        insert_local_document(
            &mut documents,
            ImportDocumentId::ArchivedChange { change: dated_name.clone(), doc: META_FILE.to_string() },
            meta,
        )?;
        let deltas = store.archived_delta_capabilities(&dated_name);
        let mut paths = local_artifact_paths(workspace, user_dir, &parsed.schema_name(), deltas).map_err(|message| {
            LocalExportError::schema_unusable(located(&archived_meta_path(workspace, &dated_name)), message)
        })?;
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
fn check_for_links(root: &Path, relative: &Path) -> Result<(), LocalExportError> {
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

fn check_below(root: &Path, relative: &Path) -> Result<(), LocalExportError> {
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
    let mut names = std::fs::read_dir(&path)
        .map_err(|error| unlisted(relative, error))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| unlisted(relative, error))?;
    names.sort();
    names.into_iter().try_for_each(|name| check_below(root, &relative.join(name)))
}

fn unlisted(relative: &Path, error: std::io::Error) -> LocalExportError {
    LocalExportError::new(
        TransferReason::SpecFolderUnreadable,
        format!("cannot list {} to check it for symbolic links: {error}", shown_path(relative)),
    )
}

fn link_found(relative: &Path) -> LocalExportError {
    LocalExportError::new(
        TransferReason::SymlinkInSpecFolder,
        format!("symbolic link in the spec folder: {} — replace it with the file itself or remove it", shown_path(relative)),
    )
}

/// `relative` with forward slashes, as an error shows a path.
fn shown_path(relative: &Path) -> String {
    let parts: Vec<_> = relative.components().map(|part| part.as_os_str().to_string_lossy()).collect();
    parts.join("/")
}

/// The data file of the archived change `dated_name` by its path from the
/// project root, as a text shows it (the spec folder's name as it is
/// written): the rule a file store has for an active change. The new file
/// name is read first, so the legacy name is the answer only when it alone
/// is there.
fn archived_meta_path(workspace: &Workspace, dated_name: &str) -> String {
    let dir = format!("{}/changes/archive/{dated_name}", workspace.spec_dir_name);
    let on_disk = |file: &str| workspace.root.join(&dir).join(file).exists();
    let file = if !on_disk(META_FILE) && on_disk(LEGACY_META_FILE) { LEGACY_META_FILE } else { META_FILE };
    format!("{dir}/{file}")
}

/// The location of `change`, whose data file a text shows as `written`: the
/// spec folder's name as it is written, then segments with forward slashes.
/// The name can hold a slash at its end, or backslashes on Windows, so the
/// location has it put together again segment by segment: its path has
/// forward slashes only, and no empty segment.
fn change_location(workspace: &Workspace, change: &str, written: &str) -> ChangeLocation {
    let name = workspace.spec_dir_name.as_str();
    let path = match written.strip_prefix(name) {
        Some(below) => format!("{}/{}", shown_path(Path::new(name)), below.trim_start_matches('/')),
        None => written.to_string(),
    };
    ChangeLocation { change: change.to_string(), path }
}

/// The artifact paths of a change whose data file names the schema `name`,
/// with one path for each of `delta_capabilities`. The failure is the text of
/// a schema that cannot be used: not found, a definition that does not load,
/// or an artifact pattern that cannot be listed.
fn local_artifact_paths(
    workspace: &Workspace,
    user_dir: &Path,
    name: &str,
    delta_capabilities: Vec<String>,
) -> Result<Vec<String>, String> {
    let schema = speclink_core::schema::resolve_with(Some(workspace), Some(user_dir), name)
        .ok_or_else(|| speclink_core::schema::not_found_msg(name))??;
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
    fn documents_of(root: &Path) -> Result<Vec<ImportBundleDocument>, LocalExportError> {
        let workspace = Workspace::discover(root).unwrap().expect("a local project");
        documents_in(&workspace.root, &workspace.spec_dir_name)
    }

    /// The documents of the project at `root` whose spec folder is named
    /// `spec_dir`, as `.speclink.yaml` or `createEngine` would name it.
    fn documents_in(root: &Path, spec_dir: &str) -> Result<Vec<ImportBundleDocument>, LocalExportError> {
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
        let error = from_first_version(first_version(documents)).expect_err("no key");
        assert!(error.to_string().contains("a/b"), "{error}");
        assert_eq!(error.reason(), None);
    }

    #[test]
    fn corrupt_change_metadata_stops_the_listing_and_names_the_file() {
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.openspec.yaml", "schema: [unterminated\n");

        let error = documents_of(root.path()).expect_err("corrupt metadata must fail closed");
        // The text is the one this failure always had: `invalid <path>: <the parser's reason>`.
        let reason = ChangeMeta::from_text(Some("schema: [unterminated\n")).expect_err("the fixture does not parse");
        assert_eq!(error.to_string(), format!("invalid openspec/changes/active-change/.openspec.yaml: {reason}"));
        assert_eq!(error.reason(), Some(TransferReason::InvalidChangeMetadata));
        assert_eq!(location(&error), Some(("active-change", "openspec/changes/active-change/.openspec.yaml")));
    }

    #[test]
    fn a_corrupt_legacy_archived_data_file_is_named_as_it_is_on_disk() {
        let root = complete_project();
        write(root.path(), "openspec/changes/archive/2026-07-20-old-change/.openspec.yaml", "schema: [unterminated\n");

        let error = documents_of(root.path()).expect_err("corrupt metadata must fail closed");
        let reason = ChangeMeta::from_text(Some("schema: [unterminated\n")).expect_err("the fixture does not parse");
        let path = "openspec/changes/archive/2026-07-20-old-change/.openspec.yaml";
        assert_eq!(error.to_string(), format!("invalid {path}: {reason}"));
        assert_eq!(error.reason(), Some(TransferReason::InvalidChangeMetadata));
        assert_eq!(location(&error), Some(("2026-07-20-old-change", path)));
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
    fn error_with_link(link: &str, to_folder: bool) -> LocalExportError {
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
        assert!(!error.to_string().contains(OUTSIDE.trim()), "{error}");
        assert_eq!(error.reason(), Some(TransferReason::SymlinkInSpecFolder));
        assert_eq!(error.location(), None, "a link is about no change");
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
        assert_eq!(error.to_string(), link_message("speclink/specs/leak/spec.md"));
    }

    #[test]
    #[cfg(unix)]
    fn a_folder_in_the_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let error = error_with_link("speclink/changes/linked", true);
        assert_eq!(error.to_string(), link_message("speclink/changes/linked"));
    }

    #[test]
    #[cfg(unix)]
    fn a_spec_folder_that_is_a_link_stops_the_listing_and_is_named() {
        let error = error_with_link("speclink", true);
        assert_eq!(error.to_string(), link_message("speclink"));
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

        assert_eq!(error.to_string(), outside_message("../other/speclink"));
        assert_eq!(error.reason(), Some(TransferReason::SpecFolderOutsideProject));
        assert_eq!(error.location(), None, "a spec folder outside the project is about no change");
    }

    #[test]
    fn a_spec_folder_named_by_an_absolute_path_stops_the_listing_and_is_named() {
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "speclink/specs/private/spec.md", OTHER);
        let root = tempfile::tempdir().unwrap();
        let name = outside.path().join("speclink").to_string_lossy().into_owned();

        let error = documents_in(root.path(), &name).expect_err("outside the project");

        assert_eq!(error.to_string(), outside_message(&name));
        assert_eq!(error.reason(), Some(TransferReason::SpecFolderOutsideProject));
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
    fn a_folder_with_no_spec_folder_lists_no_document() {
        // Speclink Desktop's "Migrate to Server" sends an empty set for such a
        // folder. Only the Node export refuses it (show-detail-and-export-check).
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "notes/README.md", "# notes\n");

        let documents = documents_in(root.path(), "speclink").expect("no spec folder is no failure here");

        assert!(documents.is_empty(), "{documents:?}");
    }

    #[test]
    fn an_empty_name_and_a_dot_segment_stop_the_listing() {
        // Contract D6: an empty name, `.` and `..` are no plain names either.
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "specs/auth/spec.md", "# auth\n");
        // A path drops a `.` that is not its first segment: `speclink/.` is
        // refused by its text.
        for name in ["", ".", "speclink/..", "speclink/."] {
            let error = documents_in(root.path(), name).expect_err(name);
            assert_eq!(error.to_string(), outside_message(name));
            assert_eq!(error.reason(), Some(TransferReason::SpecFolderOutsideProject), "{name}");
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

        assert_eq!(error.to_string(), link_message("docs"));
        assert_eq!(error.reason(), Some(TransferReason::SymlinkInSpecFolder));
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

        assert_eq!(error.to_string(), link_message(".speclink/touched/demo.json"));
        assert_eq!(error.reason(), Some(TransferReason::SymlinkInSpecFolder));
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
        assert!(
            error.to_string().starts_with("cannot list openspec/specs/accounts to check it for symbolic links:"),
            "{error}"
        );
        assert_eq!(error.reason(), Some(TransferReason::SpecFolderUnreadable));
    }

    // --- the reason of a failure (sdk-command-names-and-transfer-reasons) ---

    #[test]
    fn a_folder_that_cannot_be_listed_names_its_reason_whatever_the_user_may_read() {
        // No file permission is set here: a privileged user gets the same check.
        let error = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let expected = format!("cannot list speclink/changes/locked to check it for symbolic links: {error}");

        let failure = unlisted(Path::new("speclink/changes/locked"), error);

        assert_eq!(failure.to_string(), expected);
        assert_eq!(failure.reason(), Some(TransferReason::SpecFolderUnreadable));
        assert_eq!(failure.location(), None);
    }

    #[test]
    fn a_mode_is_parsed_from_the_text_it_gives_and_from_no_other() {
        for mode in [ImportMode::Create, ImportMode::Replace] {
            assert_eq!(ImportMode::parse(mode.as_str()), Some(mode));
        }
        for text in ["", "Replace", "replace ", "overwrite"] {
            assert_eq!(ImportMode::parse(text), None, "{text:?}");
        }
    }

    #[test]
    fn each_reason_has_its_fixed_text() {
        let texts: Vec<&str> = [
            TransferReason::NoSpecFolder,
            TransferReason::SpecFolderOutsideProject,
            TransferReason::SymlinkInSpecFolder,
            TransferReason::SpecFolderUnreadable,
            TransferReason::InvalidChangeMetadata,
            TransferReason::ChangeSchemaUnusable,
            TransferReason::ScopeNotEmpty,
            TransferReason::EmptyBundle,
            TransferReason::ScopeChanged,
            TransferReason::BundleTooLarge,
            TransferReason::InvalidBundleDocument,
        ]
        .into_iter()
        .map(TransferReason::as_str)
        .collect();
        assert_eq!(
            texts,
            [
                "no_spec_folder",
                "spec_folder_outside_project",
                "symlink_in_spec_folder",
                "spec_folder_unreadable",
                "invalid_change_metadata",
                "change_schema_unusable",
                "scope_not_empty",
                "empty_bundle",
                "scope_changed",
                "bundle_too_large",
                "invalid_bundle_document",
            ]
        );
    }

    // --- which change a failure is about (export-failure-reason-and-location D2, D3, D4) ---

    /// The change and the data file a failure names, when it names one.
    fn location(error: &LocalExportError) -> Option<(&str, &str)> {
        error.location().map(|at| (at.change.as_str(), at.path.as_str()))
    }

    /// A custom schema with one artifact, written to `generates`.
    fn schema_yaml(name: &str, generates: &str) -> String {
        format!("name: {name}\nversion: 1\nartifacts:\n- id: plan\n  generates: {generates}\n  description: The plan\n  template: plan.md\n")
    }

    /// A project in `speclink/` with one change, active (`demo`) or archived
    /// (`2026-03-03-old-one`), whose data file `file` holds `meta`. It gives
    /// the root, the change's name and the path of the data file.
    fn project_with_change(archived: bool, file: &str, meta: &str) -> (TempDir, &'static str, String) {
        let (name, dir) = if archived {
            ("2026-03-03-old-one", "speclink/changes/archive/2026-03-03-old-one")
        } else {
            ("demo", "speclink/changes/demo")
        };
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &format!("{dir}/{file}"), meta);
        write(root.path(), &format!("{dir}/proposal.md"), "## Why\n\nDemo.\n");
        (root, name, format!("{dir}/{file}"))
    }

    #[test]
    fn a_change_whose_schema_is_not_found_stops_the_listing_and_names_the_change() {
        let root = complete_project();
        write(root.path(), "openspec/changes/active-change/.openspec.yaml", "schema: no-such-schema\ncreated: 2026-07-21\n");

        let error = documents_of(root.path()).expect_err("an unknown schema stops the listing");

        assert_eq!(
            error.to_string(),
            format!("change 'active-change': {}", speclink_core::schema::not_found_msg("no-such-schema"))
        );
        assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
        // The fixture holds the legacy file name only: the path names the file that is there.
        assert_eq!(location(&error), Some(("active-change", "openspec/changes/active-change/.openspec.yaml")));
    }

    #[test]
    fn an_archived_change_whose_schema_is_not_found_is_named_by_its_dated_name() {
        let (root, name, path) = project_with_change(true, ".meta.yaml", "schema: nope\n");

        let error = documents_in(root.path(), "speclink").expect_err("an unknown schema stops the listing");

        assert_eq!(
            error.to_string(),
            "change '2026-03-03-old-one': Schema not found: Schema 'nope' not found in project, user, or built-in locations"
        );
        assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
        assert_eq!(location(&error), Some((name, path.as_str())));
        assert_eq!(path, "speclink/changes/archive/2026-03-03-old-one/.meta.yaml");
    }

    #[test]
    fn each_failure_about_a_change_gives_its_location() {
        // spec node-sdk, Example「位置屬性在哪些失敗出現」: the rows a local list makes.
        let rows = [
            (false, ".meta.yaml", "schema: nope\n", TransferReason::ChangeSchemaUnusable, "demo", "speclink/changes/demo/.meta.yaml"),
            (
                true,
                ".openspec.yaml",
                "schema: nope\n",
                TransferReason::ChangeSchemaUnusable,
                "2026-03-03-old-one",
                "speclink/changes/archive/2026-03-03-old-one/.openspec.yaml",
            ),
            (
                false,
                ".meta.yaml",
                "reviewed_rounds: ''\n",
                TransferReason::InvalidChangeMetadata,
                "demo",
                "speclink/changes/demo/.meta.yaml",
            ),
        ];
        for (archived, file, meta, reason, change, path) in rows {
            let (root, _, _) = project_with_change(archived, file, meta);
            let error = documents_in(root.path(), "speclink").expect_err(meta);
            assert_eq!(error.reason(), Some(reason), "{meta}");
            assert_eq!(location(&error), Some((change, path)), "{meta}");
        }
    }

    #[test]
    fn an_archived_change_with_no_data_file_is_located_at_the_new_file_name() {
        // No data file means the default schema; the project's own copy of it
        // does not load. The path names where the data file would be.
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "speclink/schemas/spec-driven/schema.yaml", ": [\n");
        write(root.path(), "speclink/changes/archive/2026-03-03-old-one/proposal.md", "## Why\n\nDemo.\n");

        let error = documents_in(root.path(), "speclink").expect_err("the schema does not load");

        assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
        assert_eq!(
            location(&error),
            Some(("2026-03-03-old-one", "speclink/changes/archive/2026-03-03-old-one/.meta.yaml"))
        );
    }

    #[test]
    fn a_schema_that_does_not_load_gives_the_same_reason_as_one_that_is_not_found() {
        for archived in [false, true] {
            let (root, name, path) = project_with_change(archived, ".meta.yaml", "schema: broken\n");
            write(root.path(), "speclink/schemas/broken/schema.yaml", ": [\n");
            let workspace = Workspace { root: root.path().to_path_buf(), spec_dir_name: "speclink".to_string() };
            let before = speclink_core::schema::resolve_with(Some(&workspace), None, "broken")
                .expect("the schema is found")
                .expect_err("the schema does not load");

            let error = documents_in(root.path(), "speclink").expect_err("a schema that does not load stops the listing");

            assert_eq!(error.to_string(), format!("change '{name}': {before}"));
            assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
            assert_eq!(location(&error), Some((name, path.as_str())));
        }
    }

    #[test]
    fn a_schema_with_a_pattern_that_cannot_be_listed_gives_the_same_reason() {
        for archived in [false, true] {
            let (root, name, path) = project_with_change(archived, ".meta.yaml", "schema: wide\n");
            write(root.path(), "speclink/schemas/wide/schema.yaml", &schema_yaml("wide", "docs/*.md"));

            let error = documents_in(root.path(), "speclink").expect_err("a pattern that cannot be listed stops the listing");

            assert_eq!(
                error.to_string(),
                format!("change '{name}': cannot enumerate migration artifact pattern 'docs/*.md' in schema 'wide'")
            );
            assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
            assert_eq!(location(&error), Some((name, path.as_str())));
        }
    }

    #[test]
    fn an_archived_change_with_a_blank_schema_is_listed_by_the_default_schema() {
        // spec change-lifecycle「change metadata 的 schema 空白時視同沒有設定」:
        // an older tool wrote `schema: ''`. The data file is listed as it is.
        const META: &str = "schema: ''\ncreated: ''\n";
        let (root, name, _) = project_with_change(true, ".meta.yaml", META);

        let documents = documents_in(root.path(), "speclink").expect("a blank schema is no failure");

        let doc = |doc: &str| ImportDocumentId::ArchivedChange { change: name.to_string(), doc: doc.to_string() };
        assert_eq!(content_of(&documents, &doc(".meta.yaml")), Some(META));
        assert_eq!(content_of(&documents, &doc("proposal.md")), Some("## Why\n\nDemo.\n"));
    }

    #[test]
    #[cfg(unix)]
    fn a_folder_name_with_odd_characters_is_given_as_it_is_and_never_as_an_absolute_path() {
        // Audit 2.5: the name is not escaped and not cut, and it decides no code.
        let name = "it's <b>\n\u{1b}[31m";
        let root = tempfile::tempdir().unwrap();
        write(root.path(), &format!("speclink/changes/{name}/.meta.yaml"), "schema: nope\n");

        let error = documents_in(root.path(), "speclink").expect_err("an unknown schema stops the listing");

        assert_eq!(error.reason(), Some(TransferReason::ChangeSchemaUnusable));
        let path = format!("speclink/changes/{name}/.meta.yaml");
        assert_eq!(location(&error), Some((name, path.as_str())));
        assert_eq!(error.to_string(), format!("change '{name}': {}", speclink_core::schema::not_found_msg("nope")));
        assert!(!error.to_string().contains(&*root.path().to_string_lossy()), "{error}");
    }

    #[test]
    fn the_path_of_a_location_has_forward_slashes_whatever_the_spec_folder_is_named() {
        // A name with a slash at its end is plain names too: the path has no `//`.
        for archived in [false, true] {
            let (root, name, path) = project_with_change(archived, ".meta.yaml", "schema: nope\n");
            let error = documents_in(root.path(), "speclink/").expect_err("an unknown schema stops the listing");
            assert_eq!(location(&error), Some((name, path.as_str())), "archived: {archived}");
        }
        // A data file that does not parse: the path is the same, and the text
        // names the file as it always did, with the spec folder as it is written.
        for (archived, text) in [
            (false, "invalid speclink//changes/demo/.meta.yaml: "),
            (true, "invalid speclink//changes/archive/2026-03-03-old-one/.meta.yaml: "),
        ] {
            let (root, name, path) = project_with_change(archived, ".meta.yaml", ": [\n");
            let error = documents_in(root.path(), "speclink/").expect_err("corrupt metadata must fail closed");
            assert_eq!(location(&error), Some((name, path.as_str())), "archived: {archived}");
            assert!(error.to_string().starts_with(text), "{error}");
        }
    }

    #[test]
    #[cfg(windows)]
    fn the_path_of_a_location_has_forward_slashes_for_a_spec_folder_named_with_backslashes() {
        for (archived, path) in [
            (false, "docs/specs/changes/demo/.meta.yaml"),
            (true, "docs/specs/changes/archive/2026-03-03-old-one/.meta.yaml"),
        ] {
            let root = tempfile::tempdir().unwrap();
            write(root.path(), path, "schema: nope\n");
            let error = documents_in(root.path(), "docs\\specs").expect_err("an unknown schema stops the listing");
            assert_eq!(error.location().map(|at| at.path.as_str()), Some(path), "archived: {archived}");
        }
    }

    #[test]
    fn a_failure_with_no_code_has_no_location() {
        let failure = LocalExportError::from("duplicate local migration document".to_string());
        assert_eq!(failure.reason(), None);
        assert_eq!(failure.location(), None);
    }

    fn port_doc(key: &str, content: &str) -> BundleDocument {
        BundleDocument { key: key.to_string(), content: content.to_string(), digest: content_digest(content) }
    }

    fn keyed_bundle(documents: Vec<BundleDocument>) -> ScopeBundle {
        ScopeBundle { format_version: FORMAT_VERSION, scope_version: None, documents }
    }

    fn import_into(port: &dyn StorePort, bundle: ScopeBundle) -> Result<ImportReport, TransferError> {
        let scope = ScopeId::new("default/main").unwrap();
        import(port, &scope, &Actor::new("u_42", "Alice").unwrap(), bundle, Timestamp::new(1_800_000_000_000).unwrap())
    }

    #[test]
    fn each_import_failure_names_its_reason_and_keeps_its_message() {
        let port = speclink_store_port::MemoryStorePort::new();
        let config = || port_doc("config.yaml", "schema: spec-driven\n");
        let mut wrong_digest = config();
        wrong_digest.digest = content_digest("other");
        let board_order_message = format!(
            "bundle document \"board/order\": board order content exceeds the {}-byte cap",
            crate::documents::BOARD_ORDER_LIMIT
        );
        let rows: Vec<(ScopeBundle, &str, Option<TransferReason>)> = vec![
            (
                keyed_bundle(vec![wrong_digest]),
                "bundle document \"config.yaml\": the digest is not the digest of the content",
                Some(TransferReason::InvalidBundleDocument),
            ),
            (
                keyed_bundle(vec![port_doc("notes/a.md", "x")]),
                "bundle document \"notes/a.md\": the key follows none of the Engine's naming rules",
                Some(TransferReason::InvalidBundleDocument),
            ),
            (
                keyed_bundle(vec![config(), config()]),
                "bundle document \"config.yaml\": the key appears twice",
                Some(TransferReason::InvalidBundleDocument),
            ),
            (
                keyed_bundle(vec![port_doc("board/order", &"x".repeat(crate::documents::BOARD_ORDER_LIMIT + 1))]),
                &board_order_message,
                Some(TransferReason::InvalidBundleDocument),
            ),
            (
                ScopeBundle { format_version: 99, scope_version: None, documents: Vec::new() },
                "unsupported bundle format version 99 (supported: 2)",
                None,
            ),
        ];
        for (bundle, message, reason) in rows {
            let error = import_into(&port, bundle).expect_err(message);
            assert_eq!(error.to_string(), message);
            assert_eq!(error.reason(), reason, "{message}");
        }

        import_into(&port, keyed_bundle(vec![config()])).expect("the first import");
        let error = import_into(&port, keyed_bundle(vec![config()])).expect_err("the scope holds a document");
        assert_eq!(error.to_string(), "import (create-new): the target scope already holds documents");
        assert_eq!(error.reason(), Some(TransferReason::ScopeNotEmpty));
    }

    #[test]
    fn the_two_refusals_of_a_replace_have_their_reason_and_their_fixed_message() {
        // import-replace-mode D3.
        let port = speclink_store_port::MemoryStorePort::new();
        let scope = ScopeId::new("default/main").unwrap();
        let replace = ImportOptions { mode: ImportMode::Replace, dry_run: false };
        let import = |bundle| {
            let (actor, at) = (Actor::new("u_42", "Alice").unwrap(), Timestamp::new(1_800_000_000_000).unwrap());
            import_with(&port, &scope, &actor, bundle, at, replace).map(|(report, _)| report)
        };

        let error = import(keyed_bundle(Vec::new())).expect_err("a replace with no documents");
        assert_eq!(
            error.to_string(),
            "import (replace): the bundle holds no documents — a replace with it would remove every document of the scope"
        );
        assert_eq!(error.reason(), Some(TransferReason::EmptyBundle));

        assert_eq!(
            TransferError::ScopeChanged.to_string(),
            "import (replace): the target scope changed during the import — run it again"
        );
        assert_eq!(TransferError::ScopeChanged.reason(), Some(TransferReason::ScopeChanged));

        // A bundle that is not valid is refused by its own rule first, as in the create mode.
        let error = import(ScopeBundle { format_version: 99, scope_version: None, documents: Vec::new() })
            .expect_err("not a format version");
        assert_eq!(error.to_string(), "unsupported bundle format version 99 (supported: 2)");
        assert_eq!(error.reason(), None);
    }

    #[test]
    fn a_failure_of_the_store_or_of_a_moving_scope_has_no_reason() {
        assert_eq!(TransferError::Unstable.reason(), None);
        assert_eq!(TransferError::Store(PortError::Unavailable { message: "down".into() }).reason(), None);
        let first = ImportBundle {
            format_version: 3,
            scope: ImportScope { project: "demo".into(), repo: "backend".into() },
            project_revision: 0,
            documents: Vec::new(),
        };
        assert_eq!(from_first_version(first).expect_err("not the first version").reason(), None);
    }
}

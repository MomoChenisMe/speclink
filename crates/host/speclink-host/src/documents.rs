//! The document reads and writes a remote client needs that are not Engine
//! commands (design D1–D4).
//!
//! Every function takes a store port and a scope, uses only the three
//! operations of the port, and returns the wire type with the scope version
//! where an ETag needs it. Nothing here knows HTTP.
//!
//! Each function takes the scope version first and reads the content after
//! it, so the version it reports is never newer than the content (D2). A
//! function never exports a whole scope: it lists only the prefix it needs,
//! and it reads a batch of documents in one call.

use crate::port_view::{archived_doc_key, parse_key, spec_capability, KeyKind};
use speclink_core::config::WorkflowConfig;
use speclink_core::model::{ChangeMeta, META_READ_ORDER};
use speclink_core::station::{self, ArchivedOutcome, Station, StationStatus, REVIEW, VERIFY};
use speclink_core::workspace::DEFAULT_SPEC_DIR;
use speclink_protocol::context::{ContextDocument, ContextSnapshot, ContextSnapshotRequest};
use speclink_protocol::query::{
    ArchivedItem, ArchivedListResponse, BoardOrderResponse, ConfigResponse,
    SearchHit, SearchResponse, SpecDocumentResponse, StationOutcome,
};
use speclink_store::content_digest;
use speclink_store_port::{
    keys, Actor, Commit, DocKey, Document, Event, Expected, InvalidValue, KeyPrefix, Listing, Op,
    PortError, ScopeId, StorePort, Timestamp, Version,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The largest board order a client can write. The document is a small rank
/// map; anything near this is a client that does not work.
pub const BOARD_ORDER_LIMIT: usize = 1024 * 1024;

const SNIPPET_CONTEXT: usize = 30;

/// Why a document read or write failed.
#[derive(Debug)]
pub enum DocumentsError {
    /// The addressed document does not exist. The text names it.
    NotFound(String),
    /// The workflow config cannot be parsed.
    InvalidConfig(String),
    /// The board order is larger than [`BOARD_ORDER_LIMIT`].
    TooLarge { limit: usize },
    /// The scope is not at the version the caller expected.
    ScopeMoved { expected: Version, actual: Version },
    /// The store failed.
    Store(PortError),
}

impl fmt::Display for DocumentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentsError::NotFound(message) | DocumentsError::InvalidConfig(message) => f.write_str(message),
            DocumentsError::TooLarge { limit } => {
                write!(f, "board order content exceeds the {limit}-byte cap")
            }
            DocumentsError::ScopeMoved { expected, actual } => {
                f.write_str(&scope_moved(expected.get(), *actual))
            }
            DocumentsError::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for DocumentsError {}

/// The text of a scope that is not at the version the caller expected. The
/// caller's number can be one no version reaches (past the largest), so it
/// is a plain number here.
pub fn scope_moved(expected: u64, actual: Version) -> String {
    format!("expected scope revision {expected}, actual {actual}")
}

impl From<PortError> for DocumentsError {
    fn from(e: PortError) -> Self {
        DocumentsError::Store(e)
    }
}

/// The ETag of a scope version: the number in double quotes.
pub fn etag(version: Version) -> String {
    format!("\"{}\"", version.get())
}

/// One of the contract's fixed prefixes (`keys::CHANGES` and the like).
fn prefix(fixed: &'static str) -> KeyPrefix {
    KeyPrefix::new(fixed).expect("a fixed prefix of the contract")
}

/// A prefix that lists exactly one document, for the scope version that
/// comes with every list. A document key follows every rule of a prefix.
fn just(key: &DocKey) -> KeyPrefix {
    KeyPrefix::new(key.as_str()).expect("a document key is a prefix")
}

fn read_one(
    port: &dyn StorePort,
    scope: &ScopeId,
    key: DocKey,
) -> Result<Option<Document>, PortError> {
    Ok(port.read(scope, &[key])?.into_iter().next().flatten())
}

/// The scope version, then one document: the version first, so it is never
/// newer than the content (D2).
fn versioned(
    port: &dyn StorePort,
    scope: &ScopeId,
    key: DocKey,
) -> Result<(Version, Option<Document>), PortError> {
    let version = port.list(scope, &just(&key))?.version;
    Ok((version, read_one(port, scope, key)?))
}

/// The documents of `keys`, by key, in one read. No read for no key.
fn read_all(
    port: &dyn StorePort,
    scope: &ScopeId,
    keys: Vec<DocKey>,
) -> Result<BTreeMap<DocKey, Document>, PortError> {
    if keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    let docs = port.read(scope, &keys)?;
    Ok(keys
        .into_iter()
        .zip(docs)
        .filter_map(|(key, doc)| Some((key, doc?)))
        .collect())
}

/// The key of a document of an active change, or `None` when the names
/// cannot make one that reads back as that same document.
pub(crate) fn artifact_key(change: &str, artifact: &str) -> Option<DocKey> {
    let key = keys::change_artifact(change, artifact).ok()?;
    let same = matches!(
        parse_key(key.as_str()),
        Some(KeyKind::ChangeArtifact { change: c, artifact: a }) if c == change && a == artifact
    );
    same.then_some(key)
}

/// `changes/<change>/<under>`, or `None` when the name is not a change name.
/// A name from a request goes through the contract's builder first.
fn change_prefix(change: &str, under: &str) -> Option<KeyPrefix> {
    keys::change_meta(change).ok()?;
    KeyPrefix::new(format!("{}{change}/{under}", keys::CHANGES)).ok()
}

/// `archive/changes/<dated_name>/`, or `None` when the name cannot be an
/// archived change's: it must make a key that reads back as that change's.
fn archived_change_prefix(dated_name: &str) -> Option<KeyPrefix> {
    archived_doc_key(dated_name, "x")?;
    KeyPrefix::new(format!("{}{dated_name}/", keys::ARCHIVED_CHANGES)).ok()
}

// --- the scope version ---

/// The scope's current version (the ETag and the polling token), from the
/// narrowest list there is: the key of the workflow config. Every list
/// returns the scope version, and this one returns one document at most.
pub fn scope_version(port: &dyn StorePort, scope: &ScopeId) -> Result<Version, PortError> {
    Ok(port.list(scope, &just(&keys::config()))?.version)
}

/// A value the store contract rejects never reaches the store; the caller
/// sees a backend failure that names the rule.
pub(crate) fn invalid_value(e: InvalidValue) -> PortError {
    PortError::Backend {
        message: e.to_string(),
    }
}

// --- the workflow config ---

/// The workflow config's text, its schema name and the scope version. A
/// scope with no config answers with no text and the default schema.
pub fn config_document(
    port: &dyn StorePort,
    scope: &ScopeId,
) -> Result<(ConfigResponse, Version), DocumentsError> {
    let (version, doc) = versioned(port, scope, keys::config())?;
    let content = doc.map(|doc| doc.content);
    let schema = WorkflowConfig::from_text(content.as_deref())
        .map_err(|e| DocumentsError::InvalidConfig(e.to_string()))?
        .schema_name();
    let response = ConfigResponse {
        schema,
        content,
        revision: version.get(),
    };
    Ok((response, version))
}

/// The workflow config's text as stored, or none: one read, and no parse.
pub fn config_text(port: &dyn StorePort, scope: &ScopeId) -> Result<Option<String>, PortError> {
    Ok(read_one(port, scope, keys::config())?.map(|doc| doc.content))
}

// --- the board order ---

/// The board order's text, or none, and the scope version.
pub fn board_order(
    port: &dyn StorePort,
    scope: &ScopeId,
) -> Result<(BoardOrderResponse, Version), DocumentsError> {
    let (version, doc) = versioned(port, scope, keys::board_order())?;
    let response = BoardOrderResponse {
        content: doc.map(|doc| doc.content),
        revision: version.get(),
    };
    Ok((response, version))
}

/// Refuse a board order larger than [`BOARD_ORDER_LIMIT`]. A caller that
/// must refuse an oversized body before anything else checks it first.
pub fn check_board_order_size(content: &str) -> Result<(), DocumentsError> {
    if content.len() > BOARD_ORDER_LIMIT {
        return Err(DocumentsError::TooLarge {
            limit: BOARD_ORDER_LIMIT,
        });
    }
    Ok(())
}

/// Replace the board order (design D3) when the scope is at `expected`, and
/// return the new scope version.
///
/// Every refusal comes before the commit, so a refused write changes
/// nothing: content over [`BOARD_ORDER_LIMIT`] reaches no store call, and a
/// scope at another version commits nothing. The write is one commit: the
/// document, expected at its listed version or absent, and one
/// `board-order-updated` event.
pub fn put_board_order(
    port: &dyn StorePort,
    scope: &ScopeId,
    actor: &Actor,
    content: String,
    expected: Version,
    at: Timestamp,
) -> Result<Version, DocumentsError> {
    check_board_order_size(&content)?;
    let key = keys::board_order();
    let listing = port.list(scope, &just(&key))?;
    if listing.version != expected {
        return Err(DocumentsError::ScopeMoved {
            expected,
            actual: listing.version,
        });
    }
    let current = listing
        .entries
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.version);
    let event = board_order_updated()?;
    let commit = Commit::new(
        actor.clone(),
        "put-board-order",
        at,
        vec![Op::Put {
            key,
            content,
            expected: current.map_or(Expected::Absent, Expected::At),
        }],
        vec![event],
    )
    .map_err(invalid_value)?;
    Ok(port.commit(scope, &commit)?)
}

/// The one event of a board-order write.
pub(crate) fn board_order_updated() -> Result<Event, PortError> {
    Event::new("board-order-updated", serde_json::json!({})).map_err(invalid_value)
}

// --- canonical specs and the archive ---

/// One canonical spec and the scope version.
pub fn spec_document(
    port: &dyn StorePort,
    scope: &ScopeId,
    capability: &str,
) -> Result<(SpecDocumentResponse, Version), DocumentsError> {
    let not_found = || DocumentsError::NotFound(format!("Spec '{capability}' not found."));
    let key = keys::spec(capability).map_err(|_| not_found())?;
    let (version, doc) = versioned(port, scope, key)?;
    let content = doc.ok_or_else(not_found)?.content;
    Ok((SpecDocumentResponse { content }, version))
}

/// Every archived change, newest name first, with its task counts, spec
/// count and creator — one list of the archive and one read of every
/// change's task list and data file.
pub fn archived_list(
    port: &dyn StorePort,
    scope: &ScopeId,
) -> Result<(ArchivedListResponse, Version), DocumentsError> {
    let listing = port.list(scope, &prefix(keys::ARCHIVED_CHANGES))?;
    // Each archived change and the paths of its documents.
    let mut changes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut wanted = Vec::new();
    for entry in &listing.entries {
        if let Some(KeyKind::ArchivedChangeDoc { dated_name, doc }) = parse_key(entry.key.as_str())
        {
            changes.entry(dated_name).or_default().push(doc);
            if doc == "tasks.md" || META_READ_ORDER.contains(&doc) {
                wanted.push(entry.key.clone());
            }
        }
    }
    let docs = read_all(port, scope, wanted)?;
    let text = |dated_name: &str, doc: &str| {
        archived_doc_key(dated_name, doc)
            .and_then(|key| docs.get(&key))
            .map(|doc| doc.content.as_str())
    };

    let archived = changes
        .iter()
        .rev()
        .map(|(dated_name, paths)| {
            let (date, name) = split_dated_name(dated_name);
            let tasks = text(dated_name, "tasks.md").map(|text| {
                let parsed = speclink_core::tasks::parse(text);
                let (total, done, _) = speclink_core::tasks::progress(&parsed);
                (total, done)
            });
            let meta = META_READ_ORDER
                .into_iter()
                .find_map(|doc| text(dated_name, doc));
            let meta = ChangeMeta::from_text(meta).unwrap_or_default();
            // How a station ended: its ticket archived with the change, or a
            // stamp, or nothing. The paths are the ones already listed.
            let outcome = |st: &Station| {
                let stamped = station::is_stamped(meta.anchors(st));
                match ArchivedOutcome::of(paths.contains(&st.doc), stamped) {
                    ArchivedOutcome::None => StationOutcome::None,
                    ArchivedOutcome::Stamped => StationOutcome::Stamped,
                    ArchivedOutcome::NotPassed => StationOutcome::NotPassed,
                }
            };
            ArchivedItem {
                review_outcome: Some(outcome(&REVIEW)),
                verify_outcome: Some(outcome(&VERIFY)),
                dated_name: dated_name.to_string(),
                date: date.to_string(),
                name: name.to_string(),
                tasks_total: tasks.map(|counts| counts.0),
                tasks_done: tasks.map(|counts| counts.1),
                spec_count: paths
                    .iter()
                    .filter(|p| spec_capability(p).is_some())
                    .count(),
                from_discussions: meta.from_discussions(),
                created_by: meta.created_by,
                // From the archived data file this list already read.
                fields: meta.fields,
            }
        })
        .collect();
    Ok((ArchivedListResponse { archived }, listing.version))
}

/// One document of an archived change and the scope version.
pub fn archived_artifact(
    port: &dyn StorePort,
    scope: &ScopeId,
    dated_name: &str,
    artifact: &str,
) -> Result<(SpecDocumentResponse, Version), DocumentsError> {
    let not_found = || {
        DocumentsError::NotFound(format!(
            "Archived artifact '{artifact}' not found in '{dated_name}'."
        ))
    };
    let key = archived_doc_key(dated_name, artifact).ok_or_else(not_found)?;
    let (version, doc) = versioned(port, scope, key)?;
    let content = doc.ok_or_else(not_found)?.content;
    Ok((SpecDocumentResponse { content }, version))
}

/// The capabilities an archived change carries a spec for, and the scope
/// version. An archived change that does not exist has none.
pub fn archived_capabilities(
    port: &dyn StorePort,
    scope: &ScopeId,
    dated_name: &str,
) -> Result<(Vec<String>, Version), DocumentsError> {
    // A name that cannot be an archived change's names none.
    let Some(under) = archived_change_prefix(dated_name) else {
        return Ok((Vec::new(), scope_version(port, scope)?));
    };
    let listing = port.list(scope, &under)?;
    let capabilities = capabilities_in(&listing, |kind| match kind {
        KeyKind::ArchivedChangeDoc { dated_name: name, doc } if name == dated_name => Some(doc),
        _ => None,
    });
    Ok((capabilities, listing.version))
}

/// The capability of every listed document that `spec_path` takes for a spec
/// (`specs/<capability>/spec.md` inside its change), in key order.
fn capabilities_in<'k>(
    listing: &'k Listing,
    spec_path: impl Fn(KeyKind<'k>) -> Option<&'k str>,
) -> Vec<String> {
    listing
        .entries
        .iter()
        .filter_map(|entry| spec_path(parse_key(entry.key.as_str())?))
        .filter_map(spec_capability)
        .map(str::to_string)
        .collect()
}

// --- what the change list and one change add ---

/// What the change list and one change add to the Engine's answer: the
/// change's parsed data file and its two quality stations.
pub struct ChangeExtras {
    pub meta: ChangeMeta,
    pub review: StationStatus,
    pub verify: StationStatus,
}

/// The [`ChangeExtras`] of each named change, in one read: each change's
/// data file and its two tickets go into the same read, so the stations cost
/// no store call. A name with no data file, or one that does not parse, has
/// no entry: the Engine already reports a broken data file, and the list
/// must not fail on it.
pub fn change_extras(
    port: &dyn StorePort,
    scope: &ScopeId,
    names: &[String],
) -> Result<BTreeMap<String, ChangeExtras>, DocumentsError> {
    let named: Vec<(&String, DocKey, [Option<DocKey>; 2])> = names
        .iter()
        .filter_map(|name| {
            let tickets = [&REVIEW, &VERIFY].map(|st| artifact_key(name, st.doc));
            Some((name, keys::change_meta(name).ok()?, tickets))
        })
        .collect();
    let wanted = named
        .iter()
        .flat_map(|(_, meta, tickets)| std::iter::once(meta.clone()).chain(tickets.iter().flatten().cloned()))
        .collect();
    let docs = read_all(port, scope, wanted)?;
    Ok(named
        .into_iter()
        .filter_map(|(name, key, [review, verify])| {
            let meta = ChangeMeta::from_text(Some(&docs.get(&key)?.content)).ok()?;
            let status = |st: &Station, ticket: Option<DocKey>| {
                station::status(st, &meta, ticket.is_some_and(|key| docs.contains_key(&key)))
            };
            let (review, verify) = (status(&REVIEW, review), status(&VERIFY, verify));
            Some((name.clone(), ChangeExtras { meta, review, verify }))
        })
        .collect())
}

/// The capabilities a change carries a delta spec for, sorted: one list of
/// `changes/<change>/specs/`, and no read.
pub fn change_delta_capabilities(
    port: &dyn StorePort,
    scope: &ScopeId,
    change: &str,
) -> Result<Vec<String>, DocumentsError> {
    let Some(under) = change_prefix(change, "specs/") else {
        return Ok(Vec::new());
    };
    let listing = port.list(scope, &under)?;
    let mut capabilities = capabilities_in(&listing, |kind| match kind {
        KeyKind::ChangeArtifact { change: c, artifact } if c == change => Some(artifact),
        _ => None,
    });
    capabilities.sort();
    Ok(capabilities)
}

/// The version of one document of an active change, or `None` when it does
/// not exist: what a write's `If-Match` is compared against.
pub fn document_version(
    port: &dyn StorePort,
    scope: &ScopeId,
    change: &str,
    artifact: &str,
) -> Result<Option<Version>, DocumentsError> {
    let Some(key) = artifact_key(change, artifact) else {
        return Ok(None);
    };
    Ok(read_one(port, scope, key)?.map(|doc| doc.version))
}

/// The version of one document of the archived change `dated_name`, or
/// `None` when it does not exist.
pub fn archived_document_version(
    port: &dyn StorePort,
    scope: &ScopeId,
    dated_name: &str,
    artifact: &str,
) -> Result<Option<Version>, DocumentsError> {
    let Some(key) = archived_doc_key(dated_name, artifact) else {
        return Ok(None);
    };
    Ok(read_one(port, scope, key)?.map(|doc| doc.version))
}

// --- search ---

/// The desktop's search over active changes and live discussions: a fixed
/// artifact order, one hit for each change, char-safe snippets. Two lists
/// and one read; an empty query only takes the scope version.
pub fn search(
    port: &dyn StorePort,
    scope: &ScopeId,
    query: &str,
) -> Result<(SearchResponse, Version), DocumentsError> {
    let q = query.trim();
    if q.is_empty() {
        return Ok((
            SearchResponse { hits: Vec::new() },
            scope_version(port, scope)?,
        ));
    }
    let changes = port.list(scope, &prefix(keys::CHANGES))?;
    let discussions = port.list(scope, &prefix(keys::DISCUSSIONS))?;

    // Each change with a data file, and the artifacts it has.
    #[derive(Default)]
    struct Card<'k> {
        has_meta: bool,
        artifacts: BTreeSet<&'k str>,
        deltas: Vec<&'k str>,
    }
    let mut cards: BTreeMap<&str, Card> = BTreeMap::new();
    for entry in &changes.entries {
        match parse_key(entry.key.as_str()) {
            Some(KeyKind::ChangeMeta { change }) => {
                cards.entry(change).or_default().has_meta = true
            }
            Some(KeyKind::ChangeArtifact { change, artifact }) => {
                let card = cards.entry(change).or_default();
                card.artifacts.insert(artifact);
                if let Some(capability) = spec_capability(artifact) {
                    card.deltas.push(capability);
                }
            }
            _ => {}
        }
    }
    let order: Vec<(&str, Vec<String>)> = cards
        .iter()
        .filter(|(_, card)| card.has_meta)
        .map(|(change, card)| {
            let artifacts = ["proposal.md", "design.md", "tasks.md"]
                .into_iter()
                .map(str::to_string)
                .chain(card.deltas.iter().map(|cap| format!("specs/{cap}/spec.md")))
                .filter(|artifact| card.artifacts.contains(artifact.as_str()))
                .collect();
            (*change, artifacts)
        })
        .collect();
    let slugs: BTreeSet<&str> = discussions
        .entries
        .iter()
        .filter_map(|entry| match parse_key(entry.key.as_str()) {
            Some(KeyKind::Discussion { slug }) => Some(slug),
            _ => None,
        })
        .collect();

    let mut wanted: Vec<DocKey> = order
        .iter()
        .flat_map(|(change, artifacts)| {
            artifacts
                .iter()
                .filter_map(|a| artifact_key(change, a))
        })
        .collect();
    wanted.extend(slugs.iter().filter_map(|slug| keys::discussion(slug).ok()));
    let docs = read_all(port, scope, wanted)?;

    let mut hits = Vec::new();
    for (change, artifacts) in &order {
        for artifact in artifacts {
            let Some(doc) = artifact_key(change, artifact).and_then(|key| docs.get(&key))
            else {
                continue;
            };
            if let Some(snippet) = find_snippet(&doc.content, q) {
                hits.push(SearchHit {
                    kind: "change".to_string(),
                    id: change.to_string(),
                    artifact: artifact.clone(),
                    snippet,
                });
                break;
            }
        }
    }
    for slug in slugs {
        let Some(doc) = keys::discussion(slug).ok().and_then(|key| docs.get(&key)) else {
            continue;
        };
        if let Some(snippet) = find_snippet(&doc.content, q) {
            hits.push(SearchHit {
                kind: "discussion".to_string(),
                id: slug.to_string(),
                artifact: format!("{slug}.md"),
                snippet,
            });
        }
    }
    Ok((SearchResponse { hits }, changes.version))
}

// --- the context snapshot ---

/// What [`context_snapshot`] answers.
#[derive(Debug)]
pub enum ContextOutcome {
    /// The caller's `If-None-Match` is the current scope version.
    NotModified(Version),
    Snapshot(ContextSnapshot),
}

/// The projection path of the workflow config: the source of the policy
/// revision.
fn config_path() -> String {
    format!("{DEFAULT_SPEC_DIR}/config.yaml")
}

/// The projection path (under `speclink/`) of a document when the request
/// selects it, or `None` to leave it out. A change narrows to its own
/// artifacts, every canonical spec, the config and the vocabulary; no change
/// is the full mirror, with every change's artifacts and the live
/// discussions too. Change data files, evidence and the archive are never
/// part of the mirror.
fn select_path(kind: KeyKind<'_>, change: Option<&str>) -> Option<String> {
    match kind {
        KeyKind::ChangeArtifact {
            change: c,
            artifact,
        } => change
            .map_or(true, |n| n == c)
            .then(|| format!("{DEFAULT_SPEC_DIR}/changes/{c}/{artifact}")),
        KeyKind::Spec { capability } => {
            Some(format!("{DEFAULT_SPEC_DIR}/specs/{capability}/spec.md"))
        }
        KeyKind::Config => Some(config_path()),
        KeyKind::Language => Some(format!("{DEFAULT_SPEC_DIR}/LANGUAGE.md")),
        KeyKind::Discussion { slug } if change.is_none() => {
            Some(format!("{DEFAULT_SPEC_DIR}/discussions/{slug}.md"))
        }
        _ => None,
    }
}

/// The context snapshot of a scope (design D4). The snapshot id is the scope
/// version's ETag, so an `If-None-Match` that equals it answers "not
/// modified" after one list and no read.
pub fn context_snapshot(
    port: &dyn StorePort,
    scope: &ScopeId,
    request: &ContextSnapshotRequest,
    if_none_match: Option<&str>,
) -> Result<ContextOutcome, DocumentsError> {
    let changes = port.list(scope, &prefix(keys::CHANGES))?;
    let snapshot_id = etag(changes.version);
    if if_none_match == Some(snapshot_id.as_str()) {
        return Ok(ContextOutcome::NotModified(changes.version));
    }
    let change = request.change.as_deref();
    // A change with no data file and no artifact does not exist.
    if let Some(name) = change {
        let exists = changes.entries.iter().any(|entry| {
            matches!(
                parse_key(entry.key.as_str()),
                Some(KeyKind::ChangeMeta { change: c } | KeyKind::ChangeArtifact { change: c, .. })
                    if c == name
            )
        });
        if !exists {
            return Err(DocumentsError::NotFound(format!(
                "change '{name}' not found"
            )));
        }
    }
    let specs = port.list(scope, &prefix(keys::SPECS))?;
    let discussions = match change {
        None => Some(port.list(scope, &prefix(keys::DISCUSSIONS))?),
        Some(_) => None,
    };

    let listed = [Some(&changes), Some(&specs), discussions.as_ref()]
        .into_iter()
        .flatten()
        .flat_map(|listing: &Listing| listing.entries.iter().map(|entry| entry.key.clone()));
    let mut selected: Vec<(DocKey, String)> = listed
        .chain([keys::config(), keys::language()])
        .filter_map(|key| {
            let path = select_path(parse_key(key.as_str())?, change)?;
            Some((key, path))
        })
        .collect();
    let docs = port.read(
        scope,
        &selected
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>(),
    )?;

    // A document that was listed but is gone by the read is left out.
    let mut documents: Vec<ContextDocument> = selected
        .drain(..)
        .zip(docs)
        .filter_map(|((_, path), doc)| {
            let doc = doc?;
            Some(ContextDocument {
                path,
                digest: content_digest(&doc.content),
                content: doc.content,
                revision: Some(doc.version.get()),
            })
        })
        .collect();
    documents.sort_by(|a, b| a.path.cmp(&b.path));

    let policy_revision = documents
        .iter()
        .find(|d| d.path == config_path())
        .and_then(|d| d.revision);
    let combined: Vec<&str> = documents.iter().map(|d| d.digest.as_str()).collect();
    let digest = content_digest(&combined.join("\n"));
    Ok(ContextOutcome::Snapshot(ContextSnapshot {
        snapshot_id,
        policy_revision,
        digest,
        documents,
    }))
}

// --- text helpers ---

/// The date and the name of an archived change, by the rule the archived
/// list has always answered with: a `-` at bytes 4, 7 and 10, digits not
/// checked, and no date at all when the name does not fit. It is lenient on
/// purpose and differs from `port_view::split_archived_stem`, the key
/// contract's strict rule for archived discussions: changing this one would
/// change the `date` and `name` the archived list returns.
fn split_dated_name(dated_name: &str) -> (&str, &str) {
    let bytes = dated_name.as_bytes();
    if bytes.len() > 11
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'-')
        && dated_name.is_char_boundary(10)
        && dated_name.is_char_boundary(11)
    {
        (&dated_name[..10], &dated_name[11..])
    } else {
        ("", dated_name)
    }
}

fn find_snippet(text: &str, query: &str) -> Option<String> {
    let haystack: Vec<char> = text.chars().collect();
    let folded_haystack: Vec<char> = haystack.iter().map(lower_first).collect();
    let needle: Vec<char> = query
        .chars()
        .map(|character| lower_first(&character))
        .collect();
    if needle.is_empty() || folded_haystack.len() < needle.len() {
        return None;
    }
    let start = (0..=folded_haystack.len() - needle.len())
        .find(|&index| folded_haystack[index..index + needle.len()] == needle[..])?;
    let end = start + needle.len();
    let from = start.saturating_sub(SNIPPET_CONTEXT);
    let to = (end + SNIPPET_CONTEXT).min(haystack.len());
    let mut snippet = String::new();
    if from > 0 {
        snippet.push('…');
    }
    snippet.extend(haystack[from..to].iter().map(|character| match character {
        '\n' | '\r' => ' ',
        character => *character,
    }));
    if to < haystack.len() {
        snippet.push('…');
    }
    Some(snippet)
}

fn lower_first(character: &char) -> char {
    character.to_lowercase().next().unwrap_or(*character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dated_names_and_capabilities_parse_fail_closed() {
        assert_eq!(split_dated_name("2026-07-20-demo"), ("2026-07-20", "demo"));
        assert_eq!(split_dated_name("demo"), ("", "demo"));
        assert_eq!(spec_capability("specs/auth/spec.md"), Some("auth"));
        assert!(spec_capability("specs/nested/auth/spec.md").is_none());
    }

    #[test]
    fn a_snippet_follows_the_desktop_char_level_rules() {
        let text = format!("{}MagicToken{}", "前".repeat(80), "後".repeat(80));
        let snippet = find_snippet(&text, "magictoken").expect("case-insensitive hit");
        assert!(snippet.starts_with('…') && snippet.ends_with('…'));
        assert!(snippet.contains("MagicToken"));
        assert!(snippet.chars().count() <= 30 + 10 + 30 + 2);
    }
}

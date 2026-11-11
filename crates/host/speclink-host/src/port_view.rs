//! The Engine store view over a store port (design D1).
//!
//! The Engine's command layer reads and writes through the synchronous
//! `speclink_core::store::Store` interface. [`PortView`] supplies that
//! interface from the three operations of a [`StorePort`]: it lists a prefix
//! at most once, reads a key at most once, reads several documents in one
//! call, and keeps every write in memory until the caller commits them
//! together. A view lives for one command.

use speclink_core::model::{Change, ChangeMeta, META_FILE, META_READ_ORDER};
use speclink_core::store::{DiscussionDoc, Store};
use speclink_core::workspace::DEFAULT_SPEC_DIR;
use speclink_protocol::query::BoardOrderDoc;
use speclink_store_port::{
    keys, DocKey, Expected, InvalidValue, KeyPrefix, Op, PortError, ScopeId, StorePort, Timestamp,
    Version,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

const EVIDENCE_FILE: &str = ".evidence.json";

/// What a document key names, with the names taken from the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyKind<'k> {
    ChangeMeta { change: &'k str },
    ChangeEvidence { change: &'k str },
    ChangeArtifact { change: &'k str, artifact: &'k str },
    ArchivedChangeDoc { dated_name: &'k str, doc: &'k str },
    Spec { capability: &'k str },
    Discussion { slug: &'k str },
    /// `stem` is the stored name without `.md`: `<created>-<slug>` for a
    /// document this view archived.
    ArchivedDiscussion { stem: &'k str },
    Config,
    Language,
    BoardOrder,
}

/// The reverse of the `speclink_store_port::keys` builders. `None` for a key
/// that follows none of the naming rules.
pub(crate) fn parse_key(key: &str) -> Option<KeyKind<'_>> {
    /// `<name>.md` with a one-segment name.
    fn md_name(rest: &str) -> Option<&str> {
        let name = rest.strip_suffix(".md")?;
        (!name.is_empty() && !name.contains('/')).then_some(name)
    }
    /// `<name>/<path>` with both parts present.
    fn name_and_path(rest: &str) -> Option<(&str, &str)> {
        let (name, path) = rest.split_once('/')?;
        (!name.is_empty() && !path.is_empty()).then_some((name, path))
    }

    if let Some(rest) = key.strip_prefix(keys::ARCHIVED_CHANGES) {
        let (dated_name, doc) = name_and_path(rest)?;
        return Some(KeyKind::ArchivedChangeDoc { dated_name, doc });
    }
    if let Some(rest) = key.strip_prefix(keys::ARCHIVED_DISCUSSIONS) {
        return md_name(rest).map(|stem| KeyKind::ArchivedDiscussion { stem });
    }
    if let Some(rest) = key.strip_prefix(keys::CHANGES) {
        let (change, doc) = name_and_path(rest)?;
        return Some(match doc {
            META_FILE => KeyKind::ChangeMeta { change },
            EVIDENCE_FILE => KeyKind::ChangeEvidence { change },
            artifact => KeyKind::ChangeArtifact { change, artifact },
        });
    }
    if key.starts_with(keys::SPECS) {
        return spec_capability(key).map(|capability| KeyKind::Spec { capability });
    }
    if let Some(rest) = key.strip_prefix(keys::DISCUSSIONS) {
        return md_name(rest).map(|slug| KeyKind::Discussion { slug });
    }
    match key {
        "config.yaml" => Some(KeyKind::Config),
        "LANGUAGE.md" => Some(KeyKind::Language),
        "board/order" => Some(KeyKind::BoardOrder),
        _ => None,
    }
}

/// Split the stored name of an archived discussion into its creation date
/// and the rest. `None` when the name does not start with `YYYY-MM-DD-`.
/// This is the key contract's strict rule; the archived list answers with
/// its own lenient rule (`documents::split_dated_name`), kept as it was.
pub(crate) fn split_archived_stem(stem: &str) -> Option<(&str, &str)> {
    let date = stem.get(..10)?;
    let rest = stem.get(10..)?.strip_prefix('-')?;
    // The contract owns the date rule: its builder takes exactly a
    // `YYYY-MM-DD` creation date.
    (!rest.is_empty() && keys::archived_discussion(date, "x").is_ok()).then_some((date, rest))
}

/// The key of a document of an archived change that is already in the store.
/// It is read whatever the change is called: a store can hold older archived
/// changes without a date in their name. A new archived document goes through
/// the contract's builder, which takes dated names only.
pub(crate) fn archived_doc_key(name: &str, doc: &str) -> Option<DocKey> {
    let key = DocKey::new(format!("{}{name}/{doc}", keys::ARCHIVED_CHANGES)).ok()?;
    let same = matches!(
        parse_key(key.as_str()),
        Some(KeyKind::ArchivedChangeDoc { dated_name, doc: path }) if dated_name == name && path == doc
    );
    same.then_some(key)
}

/// `archive/discussions/<created>-<name>.md` for a creation date that does not
/// start with a date. The contract's builder takes a `YYYY-MM-DD` date only;
/// the fs store takes any text there, and this keeps its stored name. Both
/// parts must still be one path segment.
fn archived_discussion_key(created: &str, name: &str) -> anyhow::Result<DocKey> {
    let stem = format!("{created}-{name}");
    let key = DocKey::new(format!("{}{stem}.md", keys::ARCHIVED_DISCUSSIONS))?;
    match parse_key(key.as_str()) {
        Some(KeyKind::ArchivedDiscussion { stem: parsed }) if parsed == stem => Ok(key),
        _ => anyhow::bail!(
            "the creation date and the slug of a discussion must each be one path segment"
        ),
    }
}

/// The capability of a `specs/<cap>/spec.md` path, exactly one level deep.
pub(crate) fn spec_capability(path: &str) -> Option<&str> {
    let cap = path.strip_prefix("specs/")?.strip_suffix("/spec.md")?;
    (!cap.is_empty() && !cap.contains('/')).then_some(cap)
}

/// Build a `Change` from raw metadata the way the fs adapter does: a corrupt
/// document yields the default meta plus the parse reason in `meta_error`.
fn change_from(name: &str, meta_text: &str) -> Change {
    let (meta, meta_error) = match ChangeMeta::from_text(Some(meta_text)) {
        Ok(meta) => (meta, None),
        Err(reason) => (ChangeMeta::default(), Some(reason)),
    };
    Change {
        name: name.to_string(),
        meta,
        meta_error,
        // The fs adapter's workspace-relative shape: the Engine builds
        // messages and path output from `dir`, and both modes must agree.
        dir: change_dir(name),
    }
}

fn change_dir(name: &str) -> PathBuf {
    PathBuf::from(format!("{DEFAULT_SPEC_DIR}/changes/{name}"))
}

/// The prefixes a view lists. Every listing is one of these, whole.
const FIXED_PREFIXES: [&str; 5] = [
    keys::ARCHIVED_CHANGES,
    keys::ARCHIVED_DISCUSSIONS,
    keys::CHANGES,
    keys::SPECS,
    keys::DISCUSSIONS,
];

fn fixed_prefix(key: &DocKey) -> Option<&'static str> {
    FIXED_PREFIXES
        .into_iter()
        .find(|prefix| key.as_str().starts_with(prefix))
}

#[derive(Default)]
struct State {
    /// The fixed prefixes already listed.
    listed: BTreeSet<&'static str>,
    /// Every listed key with its last-modified time.
    entries: BTreeMap<DocKey, Timestamp>,
    /// Every key already read; `None` is "no document".
    read: BTreeMap<DocKey, Option<String>>,
    /// The version of each key when this execution first saw it; `None` is
    /// "no document". Every write expects exactly this.
    base: BTreeMap<DocKey, Option<Version>>,
    /// The writes to commit, one per key; `None` is a delete.
    staged: BTreeMap<DocKey, Option<String>>,
    /// The first failure the store reported.
    error: Option<PortError>,
}

impl State {
    /// A listing already showed that the store has no document at `key`.
    fn listed_as_absent(&self, key: &DocKey) -> bool {
        fixed_prefix(key).is_some_and(|prefix| self.listed.contains(prefix))
            && !self.entries.contains_key(key)
    }
}

/// The Engine `Store` interface on one scope of a store port, for one
/// command.
///
/// Crate-visible and no wider: the writes of a view reach the store only
/// through `port_exec`, so a view handed to an adapter would stage writes
/// that nothing commits.
///
/// The Engine interface has no place for a store failure: a read returns
/// `Option`. The view keeps the first failure, answers "no document" from
/// then on without another store call, and [`PortView::into_ops`] returns the
/// failure. The caller must not use the Engine's result when that happens.
pub(crate) struct PortView<'a> {
    port: &'a dyn StorePort,
    scope: ScopeId,
    state: RefCell<State>,
}

impl<'a> PortView<'a> {
    pub(crate) fn new(port: &'a dyn StorePort, scope: ScopeId) -> PortView<'a> {
        PortView {
            port,
            scope,
            state: RefCell::new(State::default()),
        }
    }

    /// The staged writes in key order, or the failure the store reported
    /// during this execution.
    pub(crate) fn into_ops(self) -> Result<Vec<Op>, PortError> {
        let State {
            staged,
            base,
            error,
            ..
        } = self.state.into_inner();
        if let Some(error) = error {
            return Err(error);
        }
        Ok(staged
            .into_iter()
            .filter_map(|(key, staged)| {
                let first_seen = base.get(&key).copied().flatten();
                match (staged, first_seen) {
                    (Some(content), first_seen) => Some(Op::Put {
                        key,
                        content,
                        expected: first_seen.map_or(Expected::Absent, Expected::At),
                    }),
                    (None, Some(expected)) => Some(Op::Delete { key, expected }),
                    // `del` never stages a delete of a document the store
                    // does not have.
                    (None, None) => None,
                }
            })
            .collect())
    }

    /// List one fixed prefix, once.
    fn list(&self, prefix: &'static str) {
        let mut state = self.state.borrow_mut();
        if state.error.is_some() || !state.listed.insert(prefix) {
            return;
        }
        let key_prefix = KeyPrefix::new(prefix).expect("a fixed prefix follows the prefix format");
        match self.port.list(&self.scope, &key_prefix) {
            Ok(listing) => {
                for entry in listing.entries {
                    state
                        .base
                        .entry(entry.key.clone())
                        .or_insert(Some(entry.version));
                    state.entries.insert(entry.key, entry.modified_at);
                }
            }
            Err(error) => state.error = Some(error),
        }
    }

    /// Make sure every key of `keys` is in the read memory, with one store
    /// read for all the keys that are not there yet.
    fn fetch(&self, keys: &[DocKey]) {
        let mut state = self.state.borrow_mut();
        if state.error.is_some() {
            return;
        }
        let mut wanted: Vec<DocKey> = Vec::new();
        for key in keys {
            if state.read.contains_key(key) || state.staged.contains_key(key) || wanted.contains(key)
            {
                continue;
            }
            if state.listed_as_absent(key) {
                state.read.insert(key.clone(), None);
                state.base.entry(key.clone()).or_insert(None);
                continue;
            }
            wanted.push(key.clone());
        }
        if wanted.is_empty() {
            return;
        }
        match self.port.read(&self.scope, &wanted) {
            // The contract is one result for each key. A store that breaks it
            // must not make a missing result read as "no document".
            Ok(docs) if docs.len() != wanted.len() => {
                state.error = Some(PortError::Backend {
                    message: format!(
                        "the store returned {} results for a read of {} keys",
                        docs.len(),
                        wanted.len()
                    ),
                });
            }
            Ok(docs) => {
                for (key, doc) in wanted.into_iter().zip(docs) {
                    state
                        .base
                        .entry(key.clone())
                        .or_insert(doc.as_ref().map(|d| d.version));
                    state.read.insert(key, doc.map(|d| d.content));
                }
            }
            Err(error) => state.error = Some(error),
        }
    }

    /// The content of one document as this execution sees it.
    fn content(&self, key: &DocKey) -> Option<String> {
        if let Some(staged) = self.state.borrow().staged.get(key) {
            return staged.clone();
        }
        self.fetch(std::slice::from_ref(key));
        self.state.borrow().read.get(key).cloned().flatten()
    }

    fn exists(&self, key: &DocKey) -> bool {
        {
            let state = self.state.borrow();
            if let Some(staged) = state.staged.get(key) {
                return staged.is_some();
            }
            if let Some(read) = state.read.get(key) {
                return read.is_some();
            }
            if fixed_prefix(key).is_some_and(|prefix| state.listed.contains(prefix)) {
                return state.entries.contains_key(key);
            }
        }
        self.content(key).is_some()
    }

    /// The keys that start with `sub` as this execution sees them, in byte
    /// order. `sub` is `fixed` or a longer prefix under it.
    fn keys_under(&self, fixed: &'static str, sub: &str) -> Vec<DocKey> {
        self.list(fixed);
        let state = self.state.borrow();
        let mut found: BTreeSet<&DocKey> = state
            .entries
            .keys()
            .filter(|key| key.as_str().starts_with(sub))
            .collect();
        for (key, staged) in &state.staged {
            if !key.as_str().starts_with(sub) {
                continue;
            }
            if staged.is_some() {
                found.insert(key);
            } else {
                found.remove(key);
            }
        }
        found.into_iter().cloned().collect()
    }

    /// Make sure the first-seen version of every key of `keys` is known. A
    /// key this execution did not see yet costs one store read, shared by
    /// all such keys of the call.
    fn ensure_base(&self, keys: &[DocKey]) {
        let unseen: Vec<DocKey> = {
            let state = self.state.borrow();
            keys.iter()
                .filter(|key| !state.base.contains_key(*key))
                .cloned()
                .collect()
        };
        if !unseen.is_empty() {
            self.fetch(&unseen);
        }
    }

    /// Stage a write. The last write of a key wins; what it expects of the
    /// store stays the version first seen.
    fn put(&self, key: DocKey, content: String) {
        self.ensure_base(std::slice::from_ref(&key));
        self.state.borrow_mut().staged.insert(key, Some(content));
    }

    /// Stage a delete. A document the store does not have never reached it,
    /// so its staged write is dropped and nothing else is staged.
    fn del(&self, key: DocKey) {
        self.ensure_base(std::slice::from_ref(&key));
        let mut state = self.state.borrow_mut();
        if state.base.get(&key).copied().flatten().is_some() {
            state.staged.insert(key, None);
        } else {
            state.staged.remove(&key);
        }
    }

    fn put_named(&self, key: Result<DocKey, InvalidValue>, content: &str) -> anyhow::Result<()> {
        self.put(key?, content.to_string());
        Ok(())
    }

    /// Read the document a key builder named. A name that cannot make a key
    /// names no document.
    fn read_named(&self, key: Result<DocKey, InvalidValue>) -> Option<String> {
        self.content(&key.ok()?)
    }

    fn exists_named(&self, key: Result<DocKey, InvalidValue>) -> bool {
        key.is_ok_and(|key| self.exists(&key))
    }

    /// `changes/<name>/`, when `name` can be a change name.
    fn change_prefix(name: &str) -> Result<String, InvalidValue> {
        keys::change_meta(name)?;
        Ok(format!("{}{name}/", keys::CHANGES))
    }

    /// `archive/changes/<name>/`, when `name` is one key segment.
    fn archived_change_prefix(name: &str) -> Option<String> {
        archived_doc_key(name, META_FILE)?;
        Some(format!("{}{name}/", keys::ARCHIVED_CHANGES))
    }

    /// The delta capabilities of one change: the keys under `fixed` that
    /// start with `sub` and continue with `specs/<cap>/spec.md`.
    fn capabilities_under(&self, fixed: &'static str, sub: &str) -> Vec<String> {
        let mut caps: Vec<String> = self
            .keys_under(fixed, sub)
            .iter()
            .filter_map(|key| spec_capability(&key.as_str()[sub.len()..]))
            .map(str::to_string)
            .collect();
        caps.sort();
        caps
    }

    /// The key of the newest archived discussion for `slug`, ranked by
    /// creation date and then by the `-N` reuse suffix (a plain name counts
    /// as 1). A stored name without a date ranks oldest.
    fn newest_archived_discussion(&self, slug: &str) -> Option<DocKey> {
        let mut newest: Option<(String, u64, DocKey)> = None;
        for key in self.keys_under(keys::ARCHIVED_DISCUSSIONS, keys::ARCHIVED_DISCUSSIONS) {
            let Some(KeyKind::ArchivedDiscussion { stem }) = parse_key(key.as_str()) else {
                continue;
            };
            let ranked = match split_archived_stem(stem) {
                // An older record without a date: its whole name is the slug.
                None => (stem == slug).then_some(("", 1)),
                Some((date, rest)) if rest == slug => Some((date, 1)),
                Some((date, rest)) => rest
                    .strip_prefix(slug)
                    .and_then(|suffix| suffix.strip_prefix('-'))
                    .filter(|n| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()))
                    .and_then(|n| n.parse::<u64>().ok())
                    .map(|n| (date, n)),
            };
            let Some((date, seq)) = ranked else { continue };
            let candidate = (date.to_string(), seq, key.clone());
            if newest.as_ref().is_none_or(|best| candidate > *best) {
                newest = Some(candidate);
            }
        }
        newest.map(|(_, _, key)| key)
    }

    /// The discussions under `fixed`, read in one store call.
    fn discussions_under(&self, fixed: &'static str) -> Vec<DiscussionDoc> {
        let keys = self.keys_under(fixed, fixed);
        self.fetch(&keys);
        keys.iter()
            .filter_map(|key| {
                let (slug, path, archived) = match parse_key(key.as_str())? {
                    KeyKind::Discussion { slug } => {
                        (slug, format!("discussions/{slug}.md"), false)
                    }
                    KeyKind::ArchivedDiscussion { stem } => (
                        split_archived_stem(stem).map_or(stem, |(_, rest)| rest),
                        format!("discussions/archive/{stem}.md"),
                        true,
                    ),
                    _ => return None,
                };
                Some(DiscussionDoc {
                    slug: slug.to_string(),
                    text: self.content(key)?,
                    path: PathBuf::from(path),
                    archived,
                })
            })
            .collect()
    }
}

impl Store for PortView<'_> {
    // --- changes ---

    /// A store port backs a shared project. Ownership is the concept `claim`
    /// adjudicates, so the verb takes its team-mode path here.
    fn supports_ownership(&self) -> bool {
        true
    }

    fn list_changes(&self) -> Vec<Change> {
        let metas: Vec<DocKey> = self
            .keys_under(keys::CHANGES, keys::CHANGES)
            .into_iter()
            .filter(|key| {
                // Only a name `find_change` can find: the key naming rules
                // keep `archive` for the archive folder.
                matches!(
                    parse_key(key.as_str()),
                    Some(KeyKind::ChangeMeta { change }) if keys::change_meta(change).is_ok()
                )
            })
            .collect();
        self.fetch(&metas);
        let mut out: Vec<Change> = metas
            .iter()
            .filter_map(|key| {
                let KeyKind::ChangeMeta { change } = parse_key(key.as_str())? else {
                    return None;
                };
                Some(change_from(change, &self.content(key)?))
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    fn find_change(&self, name: &str) -> Option<Change> {
        self.read_change_meta(name)
            .map(|meta| change_from(name, &meta))
    }

    fn change_exists(&self, name: &str) -> bool {
        self.exists_named(keys::change_meta(name))
    }

    fn create_change(&self, name: &str, meta_text: &str) -> anyhow::Result<PathBuf> {
        self.put_named(keys::change_meta(name), meta_text)?;
        Ok(change_dir(name))
    }

    fn updated_at_secs(&self, name: &str) -> u64 {
        let Ok(prefix) = Self::change_prefix(name) else {
            return 0;
        };
        self.list(keys::CHANGES);
        self.state
            .borrow()
            .entries
            .iter()
            .filter(|(key, _)| key.as_str().starts_with(&prefix))
            .map(|(_, modified_at)| modified_at.get() / 1000)
            .max()
            .unwrap_or(0)
    }

    fn read_change_meta(&self, name: &str) -> Option<String> {
        self.read_named(keys::change_meta(name))
    }

    fn write_change_meta(&self, name: &str, content: &str) -> anyhow::Result<()> {
        self.put_named(keys::change_meta(name), content)
    }

    fn delete_change(&self, name: &str) -> anyhow::Result<()> {
        let prefix = Self::change_prefix(name)?;
        for key in self.keys_under(keys::CHANGES, &prefix) {
            self.del(key);
        }
        Ok(())
    }

    // --- artifacts ---

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        self.read_named(keys::change_artifact(change, artifact))
    }

    fn write_artifact(
        &self,
        change: &str,
        artifact: &str,
        content: &str,
    ) -> anyhow::Result<PathBuf> {
        self.put_named(keys::change_artifact(change, artifact), content)?;
        // One string, so the separators are `/` on every platform.
        Ok(PathBuf::from(format!(
            "{DEFAULT_SPEC_DIR}/changes/{change}/{artifact}"
        )))
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.exists_named(keys::change_artifact(change, artifact))
    }

    fn delete_artifact(&self, change: &str, artifact: &str) -> anyhow::Result<()> {
        // Staged like every write: a review stamp's "delete the ticket, write
        // the stamp" lands in one commit, with no state in between.
        self.del(keys::change_artifact(change, artifact)?);
        Ok(())
    }

    // --- completion evidence ---

    fn read_evidence(&self, change: &str) -> Option<String> {
        self.read_named(keys::change_evidence(change))
    }

    fn write_evidence(&self, change: &str, content: &str) -> anyhow::Result<()> {
        // Staged like every write: the record commits with the task checkbox
        // and the task-completed event, never without them.
        self.put_named(keys::change_evidence(change), content)
    }

    // --- delta specs ---

    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        Self::change_prefix(change)
            .map(|prefix| self.capabilities_under(keys::CHANGES, &prefix))
            .unwrap_or_default()
    }

    fn has_capability_dirs(&self, change: &str) -> bool {
        // A capability container is a folder under `specs/`: a key with one
        // more segment after it, whatever the document is called.
        Self::change_prefix(change).is_ok_and(|prefix| {
            let specs = format!("{prefix}specs/");
            self.keys_under(keys::CHANGES, &specs)
                .iter()
                .any(|key| key.as_str()[specs.len()..].contains('/'))
        })
    }

    // --- canonical specs ---

    fn list_canonical_capabilities(&self) -> Vec<String> {
        self.keys_under(keys::SPECS, keys::SPECS)
            .iter()
            .filter_map(|key| match parse_key(key.as_str())? {
                KeyKind::Spec { capability } => Some(capability.to_string()),
                _ => None,
            })
            .collect()
    }

    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.exists_named(keys::spec(cap))
    }

    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        self.read_named(keys::spec(cap))
    }

    fn write_canonical_spec(&self, cap: &str, content: &str) -> anyhow::Result<()> {
        self.put_named(keys::spec(cap), content)
    }

    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        PathBuf::from(format!("specs/{cap}/spec.md"))
    }

    // --- archive ---

    fn list_archived_changes(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .keys_under(keys::ARCHIVED_CHANGES, keys::ARCHIVED_CHANGES)
            .iter()
            .filter_map(|key| match parse_key(key.as_str())? {
                KeyKind::ArchivedChangeDoc { dated_name, .. } => Some(dated_name.to_string()),
                _ => None,
            })
            .collect();
        names.sort_by(|a, b| b.cmp(a));
        names.dedup();
        names
    }

    fn archived_change_exists(&self, dated_name: &str) -> bool {
        let Some(prefix) = Self::archived_change_prefix(dated_name) else {
            return false;
        };
        let listed = self.state.borrow().listed.contains(keys::ARCHIVED_CHANGES);
        if listed {
            return !self.keys_under(keys::ARCHIVED_CHANGES, &prefix).is_empty();
        }
        // The archive is the largest prefix. Do not list it for one name:
        // an archived change always has its data file, so read that.
        self.read_archived_meta(dated_name).is_some()
            || self
                .state
                .borrow()
                .staged
                .iter()
                .any(|(key, staged)| staged.is_some() && key.as_str().starts_with(&prefix))
    }

    fn archive_change(&self, name: &str, dated_name: &str) -> anyhow::Result<()> {
        // Every document of the change moves under the dated name with the
        // same relative path: the data file, each artifact, the evidence.
        let prefix = Self::change_prefix(name)?;
        let mut moves: Vec<(DocKey, DocKey)> = Vec::new();
        for source in self.keys_under(keys::CHANGES, &prefix) {
            let target = keys::archived_change_doc(dated_name, &source.as_str()[prefix.len()..])?;
            moves.push((source, target));
        }
        // One store read: the sources not read yet, and the targets, whose
        // first-seen version the writes need.
        let all: Vec<DocKey> = moves
            .iter()
            .flat_map(|(source, target)| [source.clone(), target.clone()])
            .collect();
        self.fetch(&all);
        for (source, target) in moves {
            // `content` sees an earlier write of the same flow (the
            // mark-tasks-complete pre-write), so it travels into the archive.
            if let Some(content) = self.content(&source) {
                self.put(target, content);
            }
            self.del(source);
        }
        Ok(())
    }

    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        // New name first, legacy name as the fallback, both in one read.
        let candidates: Vec<DocKey> = META_READ_ORDER
            .into_iter()
            .filter_map(|file| archived_doc_key(dated_name, file))
            .collect();
        self.fetch(&candidates);
        candidates.iter().find_map(|key| self.content(key))
    }

    fn write_archived_meta(&self, dated_name: &str, content: &str) -> anyhow::Result<()> {
        self.put_named(keys::archived_change_doc(dated_name, META_FILE), content)
    }

    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        self.content(&archived_doc_key(dated_name, artifact)?)
    }

    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        Self::archived_change_prefix(dated_name)
            .map(|prefix| self.capabilities_under(keys::ARCHIVED_CHANGES, &prefix))
            .unwrap_or_default()
    }

    // --- discussions ---

    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.exists_named(keys::discussion(slug))
    }

    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.newest_archived_discussion(slug).is_some()
    }

    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        PathBuf::from(format!("discussions/{slug}.md"))
    }

    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        self.read_named(keys::discussion(slug))
    }

    fn write_live_discussion(&self, slug: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.put_named(keys::discussion(slug), content)?;
        Ok(self.live_discussion_path(slug))
    }

    fn delete_live_discussion(&self, slug: &str) -> anyhow::Result<()> {
        self.del(keys::discussion(slug)?);
        Ok(())
    }

    fn read_discussion(&self, slug: &str) -> Option<DiscussionDoc> {
        if let Some(text) = self.read_live_discussion(slug) {
            return Some(DiscussionDoc {
                slug: slug.to_string(),
                text,
                path: self.live_discussion_path(slug),
                archived: false,
            });
        }
        let key = self.newest_archived_discussion(slug)?;
        let file = key.as_str().strip_prefix(keys::ARCHIVED_DISCUSSIONS)?;
        Some(DiscussionDoc {
            slug: slug.to_string(),
            text: self.content(&key)?,
            path: PathBuf::from(format!("discussions/archive/{file}")),
            archived: true,
        })
    }

    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        self.discussions_under(keys::DISCUSSIONS)
    }

    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        // Key order is stored-name order, which is archive-date order.
        self.discussions_under(keys::ARCHIVED_DISCUSSIONS)
    }

    fn archive_discussion(&self, slug: &str, created: &str) -> anyhow::Result<Option<String>> {
        let source = keys::discussion(slug)?;
        let Some(text) = self.content(&source) else {
            return Ok(None);
        };
        // A stored name already in use gets a `-N` suffix, so archiving a
        // reused slug on the same day never fails.
        let taken = self.keys_under(keys::ARCHIVED_DISCUSSIONS, keys::ARCHIVED_DISCUSSIONS);
        // The creation date comes from a document a person can edit. When it
        // starts with a date (a date and a time, for one), the stored name
        // takes that date: only a dated name is found by its slug later.
        let date = created
            .get(..10)
            .filter(|date| keys::archived_discussion(date, "x").is_ok());
        let mut name = slug.to_string();
        let mut n = 2;
        let target = loop {
            let candidate = match date {
                Some(date) => keys::archived_discussion(date, &name)?,
                None => archived_discussion_key(created, &name)?,
            };
            if !taken.contains(&candidate) {
                break candidate;
            }
            name = format!("{slug}-{n}");
            n += 1;
        };
        let file = target.as_str()[keys::ARCHIVED_DISCUSSIONS.len()..].to_string();
        self.put(target, text);
        self.del(source);
        Ok(Some(file))
    }

    // --- workflow config / shared vocabulary ---

    fn read_workflow_config(&self) -> Option<String> {
        self.content(&keys::config())
    }

    fn write_workflow_config(&self, content: &str) -> anyhow::Result<()> {
        self.put(keys::config(), content.to_string());
        Ok(())
    }

    fn read_language(&self) -> Option<String> {
        self.content(&keys::language())
    }

    // --- board order: the scope's board-order document (skills-verbs-first-wave D4) ---

    /// The change section of `board/order`, read by the one lenient rule every
    /// reader of the document uses ([`BoardOrderDoc::from_content`]), limited
    /// to `changes`: an entry left by an archived or discarded change is not
    /// an order key.
    fn board_ranks(&self, changes: &[Change]) -> BTreeMap<String, String> {
        let mut ranks = BoardOrderDoc::from_content(self.content(&keys::board_order()).as_deref()).changes;
        ranks.retain(|name, _| changes.iter().any(|c| &c.name == name));
        ranks
    }

    /// Stage the whole `board/order` document with `ranks` merged in
    /// ([`BoardOrderDoc::merged_content`]): one write, whatever the batch
    /// size, committed with the command's events.
    fn write_board_ranks(&self, ranks: &[(String, String)]) -> anyhow::Result<()> {
        let key = keys::board_order();
        let active = self.list_changes();
        let active: Vec<&str> = active.iter().map(|c| c.name.as_str()).collect();
        let content = BoardOrderDoc::merged_content(self.content(&key).as_deref(), &active, ranks);
        self.put(key, content);
        Ok(())
    }
}

/// Test doubles shared by this crate's unit tests.
#[cfg(test)]
pub(crate) mod testing {
    use crate::binding::local_default_binding;
    use crate::context::{Actor, ActorSource, ExecutionMode, SpeclinkExecutionContext};
    use crate::policy::EffectiveWorkflowPolicy;
    use speclink_core::config::ResolvedPolicy;
    use speclink_store_port::{
        Actor as PortActor, Commit, DocKey, Document, Expected, KeyPrefix, Listing,
        MemoryStorePort, Op, PortError, ScopeId, StorePort, Timestamp, Version,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    pub(crate) fn scope() -> ScopeId {
        ScopeId::new("default/main").unwrap()
    }

    pub(crate) const ACTOR: &str = "Alice <alice@example.com>";

    /// The Host context of an identified actor on the default binding.
    pub(crate) fn host_ctx() -> SpeclinkExecutionContext {
        let binding = local_default_binding();
        SpeclinkExecutionContext {
            actor: Actor::Identified {
                display: ACTOR.to_string(),
                source: ActorSource::Explicit,
            },
            project: binding.project,
            repo: binding.repo,
            mode: ExecutionMode::SharedStore,
            policy: EffectiveWorkflowPolicy::new(
                ResolvedPolicy {
                    locale: "English".to_string(),
                    spec_locale: None,
                    tdd: false,
                    audit: false,
                    worktree: false,
                },
                "",
            ),
        }
    }

    pub(crate) fn key(text: &str) -> DocKey {
        DocKey::new(text).unwrap()
    }

    /// Write `docs` into `port` in one commit at `at_millis`. Each document
    /// must not exist yet.
    pub(crate) fn seed(port: &dyn StorePort, at_millis: u64, docs: &[(&str, &str)]) {
        let ops = docs
            .iter()
            .map(|(k, content)| Op::Put {
                key: key(k),
                content: content.to_string(),
                expected: Expected::Absent,
            })
            .collect();
        let commit = Commit::new(
            PortActor::new("seed", "seed").unwrap(),
            "seed",
            Timestamp::new(at_millis).unwrap(),
            ops,
            Vec::new(),
        )
        .unwrap();
        port.commit(&scope(), &commit).expect("seed commit");
    }

    /// Replace an existing document from outside a view: another writer.
    pub(crate) fn overwrite(port: &dyn StorePort, at_millis: u64, k: &str, content: &str) {
        let current = port.read(&scope(), &[key(k)]).expect("read")[0]
            .clone()
            .expect("the document exists");
        let commit = Commit::new(
            PortActor::new("other", "other").unwrap(),
            "other",
            Timestamp::new(at_millis).unwrap(),
            vec![Op::Put {
                key: key(k),
                content: content.to_string(),
                expected: Expected::At(current.version),
            }],
            Vec::new(),
        )
        .unwrap();
        port.commit(&scope(), &commit).expect("overwrite commit");
    }

    /// A store port that records every call and can fail one of them.
    pub(crate) struct Probe {
        pub(crate) inner: MemoryStorePort,
        lists: Mutex<Vec<String>>,
        reads: Mutex<Vec<Vec<String>>>,
        commits: Mutex<Vec<Commit>>,
        calls: AtomicUsize,
        fail_at: Option<usize>,
        before_commit: Mutex<Option<BeforeCommit>>,
    }

    type BeforeCommit = Box<dyn FnOnce(&MemoryStorePort) + Send>;

    impl Probe {
        pub(crate) fn new(inner: MemoryStorePort) -> Probe {
            Probe {
                inner,
                lists: Mutex::new(Vec::new()),
                reads: Mutex::new(Vec::new()),
                commits: Mutex::new(Vec::new()),
                calls: AtomicUsize::new(0),
                fail_at: None,
                before_commit: Mutex::new(None),
            }
        }

        /// Run `other_writer` on the store just before the first commit
        /// reaches it: a write that lands after the command's reads.
        pub(crate) fn before_commit(
            self,
            other_writer: impl FnOnce(&MemoryStorePort) + Send + 'static,
        ) -> Probe {
            *self.before_commit.lock().unwrap() = Some(Box::new(other_writer));
            self
        }

        /// The `n`th call (1 is the first) reports "temporarily unavailable".
        pub(crate) fn failing_at(mut self, n: usize) -> Probe {
            self.fail_at = Some(n);
            self
        }

        fn call(&self) -> Result<(), PortError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if self.fail_at == Some(n) {
                return Err(PortError::Unavailable {
                    message: "probe failure".into(),
                });
            }
            Ok(())
        }

        /// The prefix of every list call, in call order.
        pub(crate) fn lists(&self) -> Vec<String> {
            self.lists.lock().unwrap().clone()
        }

        /// The keys of every read call, in call order.
        pub(crate) fn reads(&self) -> Vec<Vec<String>> {
            self.reads.lock().unwrap().clone()
        }

        /// Every commit the store received, also the rejected ones.
        pub(crate) fn commits(&self) -> Vec<Commit> {
            self.commits.lock().unwrap().clone()
        }
    }

    impl StorePort for Probe {
        fn list(&self, scope: &ScopeId, prefix: &KeyPrefix) -> Result<Listing, PortError> {
            self.lists.lock().unwrap().push(prefix.as_str().to_string());
            self.call()?;
            self.inner.list(scope, prefix)
        }

        fn read(
            &self,
            scope: &ScopeId,
            keys: &[DocKey],
        ) -> Result<Vec<Option<Document>>, PortError> {
            self.reads
                .lock()
                .unwrap()
                .push(keys.iter().map(|k| k.as_str().to_string()).collect());
            self.call()?;
            self.inner.read(scope, keys)
        }

        fn commit(&self, scope: &ScopeId, commit: &Commit) -> Result<Version, PortError> {
            self.commits.lock().unwrap().push(commit.clone());
            self.call()?;
            if let Some(other_writer) = self.before_commit.lock().unwrap().take() {
                other_writer(&self.inner);
            }
            self.inner.commit(scope, commit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{overwrite, scope, seed, Probe};
    use super::*;
    use speclink_core::store::Store;
    use speclink_store_port::{keys, Expected, MemoryStorePort, PortError};
    use std::path::PathBuf;

    const META_X: &str = "schema: spec-driven\ncreated: 2026-11-01\n";
    const META_Y: &str = "schema: spec-driven\n";

    /// One of everything the Engine store interface can ask for.
    fn fixture() -> MemoryStorePort {
        let port = MemoryStorePort::new();
        seed(
            &port,
            5_000,
            &[
                ("changes/add-x/.meta.yaml", META_X),
                ("changes/add-x/proposal.md", "# Proposal\n"),
                ("changes/add-x/tasks.md", "- [ ] 1.1 a\n"),
                ("changes/add-x/specs/auth/spec.md", "## ADDED Requirements\n"),
                ("changes/add-x/.evidence.json", "{}"),
                ("changes/add-y/.meta.yaml", META_Y),
                ("specs/auth/spec.md", "# auth\n"),
                ("specs/board/spec.md", "# board\n"),
                ("discussions/board-search.md", "live text"),
                ("archive/discussions/2026-11-01-old-talk.md", "archived text"),
                ("archive/discussions/2026-11-01-old-talk-2.md", "newer text"),
                ("archive/changes/2026-11-10-add-w/.meta.yaml", "created: 2026-11-09\n"),
                ("archive/changes/2026-11-10-add-w/proposal.md", "# W\n"),
                ("archive/changes/2026-11-10-add-w/specs/auth/spec.md", "## ADDED\n"),
                ("archive/changes/2026-10-01-add-v/.openspec.yaml", "created: 2026-09-30\n"),
                ("config.yaml", "schema: spec-driven\n"),
                ("LANGUAGE.md", "# Vocabulary\n"),
            ],
        );
        // A later commit, so `add-y` is newer than `add-x`.
        seed(&port, 9_000, &[("changes/add-y/tasks.md", "- [ ] 1.1 b\n")]);
        port
    }

    // --- 鍵解析回名稱：每一類鍵「建立再解析得到原值」 ---

    #[test]
    fn every_kind_of_key_parses_back_to_its_names() {
        assert_eq!(
            parse_key(keys::change_meta("add-x").unwrap().as_str()),
            Some(KeyKind::ChangeMeta { change: "add-x" })
        );
        assert_eq!(
            parse_key(keys::change_evidence("add-x").unwrap().as_str()),
            Some(KeyKind::ChangeEvidence { change: "add-x" })
        );
        assert_eq!(
            parse_key(keys::change_artifact("add-x", "specs/auth/spec.md").unwrap().as_str()),
            Some(KeyKind::ChangeArtifact { change: "add-x", artifact: "specs/auth/spec.md" })
        );
        assert_eq!(
            parse_key(keys::archived_change_doc("2026-11-10-add-x", "proposal.md").unwrap().as_str()),
            Some(KeyKind::ArchivedChangeDoc { dated_name: "2026-11-10-add-x", doc: "proposal.md" })
        );
        assert_eq!(
            parse_key(keys::spec("auth").unwrap().as_str()),
            Some(KeyKind::Spec { capability: "auth" })
        );
        assert_eq!(
            parse_key(keys::discussion("board-search").unwrap().as_str()),
            Some(KeyKind::Discussion { slug: "board-search" })
        );
        let archived = keys::archived_discussion("2026-11-10", "board-search").unwrap();
        assert_eq!(
            parse_key(archived.as_str()),
            Some(KeyKind::ArchivedDiscussion { stem: "2026-11-10-board-search" })
        );
        assert_eq!(
            split_archived_stem("2026-11-10-board-search"),
            Some(("2026-11-10", "board-search"))
        );
        assert_eq!(parse_key(keys::config().as_str()), Some(KeyKind::Config));
        assert_eq!(parse_key(keys::language().as_str()), Some(KeyKind::Language));
        assert_eq!(parse_key(keys::board_order().as_str()), Some(KeyKind::BoardOrder));
    }

    #[test]
    fn keys_outside_the_naming_rules_parse_to_nothing() {
        for text in [
            "notes.txt",
            "specs/auth/notes.md",
            "specs/auth/deep/spec.md",
            "discussions/board-search.txt",
            "discussions/nested/talk.md",
            "archive/discussions/nested/talk.md",
            "archive/other/x.md",
            "board/other",
        ] {
            assert_eq!(parse_key(text), None, "{text}");
        }
        assert_eq!(split_archived_stem("board-search"), None);
    }

    // --- 覆蓋表：引擎儲存介面的每個方法，至少一個經 PortView 的斷言 ---

    fn spec_dir_display() {
        let port = fixture();
        assert_eq!(PortView::new(&port, scope()).spec_dir_display(), "speclink");
    }

    fn change_meta_display() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).change_meta_display("add-x"),
            "speclink/changes/add-x/.meta.yaml"
        );
    }

    fn list_changes() {
        let port = fixture();
        let changes = PortView::new(&port, scope()).list_changes();
        let names: Vec<&str> = changes.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["add-x", "add-y"]);
        assert_eq!(changes[0].meta.created.as_deref(), Some("2026-11-01"));
        assert_eq!(changes[0].dir, PathBuf::from("speclink/changes/add-x"));
        assert_eq!(changes[0].meta_error, None);
    }

    fn find_change() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.find_change("add-x").map(|c| c.name), Some("add-x".to_string()));
        assert!(view.find_change("nope").is_none());
    }

    fn change_exists() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.change_exists("add-x"));
        assert!(!view.change_exists("nope"));
        // A name that cannot be a key segment names no change.
        assert!(!view.change_exists("a/b"));
    }

    fn updated_at_secs() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.updated_at_secs("add-x"), 5);
        assert_eq!(view.updated_at_secs("add-y"), 9, "the newest document of the change");
        assert_eq!(view.updated_at_secs("nope"), 0);
    }

    fn read_change_meta() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_change_meta("add-x").as_deref(), Some(META_X));
        assert_eq!(view.read_change_meta("nope"), None);
    }

    fn supports_ownership() {
        let port = fixture();
        assert!(PortView::new(&port, scope()).supports_ownership());
    }

    fn read_artifact() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_artifact("add-x", "proposal.md").as_deref(), Some("# Proposal\n"));
        assert_eq!(view.read_artifact("add-x", "design.md"), None);
        assert_eq!(view.read_artifact("add-x", "../config.yaml"), None);
    }

    fn artifact_exists() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.artifact_exists("add-x", "tasks.md"));
        assert!(!view.artifact_exists("add-x", "design.md"));
        // After the prefix was listed, the answer comes from memory.
        view.list_changes();
        assert!(view.artifact_exists("add-x", "tasks.md"));
        assert!(!view.artifact_exists("add-x", "design.md"));
    }

    fn read_evidence() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_evidence("add-x").as_deref(), Some("{}"));
        assert_eq!(view.read_evidence("add-y"), None);
    }

    fn delta_capabilities() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.delta_capabilities("add-x"), ["auth"]);
        assert!(view.delta_capabilities("add-y").is_empty());
    }

    fn has_capability_dirs() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.has_capability_dirs("add-x"));
        assert!(!view.has_capability_dirs("add-y"));
    }

    fn list_canonical_capabilities() {
        let port = fixture();
        let mut caps = PortView::new(&port, scope()).list_canonical_capabilities();
        caps.sort();
        assert_eq!(caps, ["auth", "board"]);
    }

    fn canonical_spec_exists() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.canonical_spec_exists("auth"));
        assert!(!view.canonical_spec_exists("nope"));
    }

    fn read_canonical_spec() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_canonical_spec("auth").as_deref(), Some("# auth\n"));
        assert_eq!(view.read_canonical_spec("nope"), None);
    }

    fn canonical_spec_path() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).canonical_spec_path("auth"),
            PathBuf::from("specs/auth/spec.md")
        );
    }

    fn list_archived_changes() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).list_archived_changes(),
            ["2026-11-10-add-w", "2026-10-01-add-v"]
        );
    }

    fn archived_change_exists() {
        let port = fixture();
        // Before the prefix was listed: answered by reading the data file,
        // new name or legacy name.
        let view = PortView::new(&port, scope());
        assert!(view.archived_change_exists("2026-11-10-add-w"));
        assert!(view.archived_change_exists("2026-10-01-add-v"));
        assert!(!view.archived_change_exists("2026-11-10-nope"));
        // After the prefix was listed: answered from memory.
        let view = PortView::new(&port, scope());
        view.list_archived_changes();
        assert!(view.archived_change_exists("2026-11-10-add-w"));
        assert!(!view.archived_change_exists("2026-11-10-nope"));
    }

    fn read_archived_meta() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(
            view.read_archived_meta("2026-11-10-add-w").as_deref(),
            Some("created: 2026-11-09\n")
        );
        assert_eq!(
            view.read_archived_meta("2026-10-01-add-v").as_deref(),
            Some("created: 2026-09-30\n"),
            "the legacy data file is the fallback"
        );
        assert_eq!(view.read_archived_meta("2026-11-10-nope"), None);
        // Both names exist: the new one wins.
        seed(&port, 6_000, &[("archive/changes/2026-10-01-add-v/.meta.yaml", "created: new\n")]);
        assert_eq!(
            PortView::new(&port, scope()).read_archived_meta("2026-10-01-add-v").as_deref(),
            Some("created: new\n")
        );
    }

    fn read_archived_artifact() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(
            view.read_archived_artifact("2026-11-10-add-w", "proposal.md").as_deref(),
            Some("# W\n")
        );
        assert_eq!(view.read_archived_artifact("2026-11-10-add-w", "design.md"), None);
    }

    fn archived_delta_capabilities() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.archived_delta_capabilities("2026-11-10-add-w"), ["auth"]);
        assert!(view.archived_delta_capabilities("2026-10-01-add-v").is_empty());
    }

    fn live_discussion_exists() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.live_discussion_exists("board-search"));
        assert!(!view.live_discussion_exists("old-talk"));
    }

    fn archived_discussion_exists() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.archived_discussion_exists("old-talk"));
        assert!(!view.archived_discussion_exists("board-search"));
        assert!(!view.archived_discussion_exists("old"), "a slug is not a prefix match");
    }

    fn live_discussion_path() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).live_discussion_path("board-search"),
            PathBuf::from("discussions/board-search.md")
        );
    }

    fn read_live_discussion() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_live_discussion("board-search").as_deref(), Some("live text"));
        assert_eq!(view.read_live_discussion("old-talk"), None);
    }

    fn read_discussion() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        let live = view.read_discussion("board-search").expect("live record");
        assert_eq!((live.text.as_str(), live.archived), ("live text", false));
        assert_eq!(live.path, PathBuf::from("discussions/board-search.md"));
        // No live record: the newest archived candidate (`-2` beats the plain name).
        let archived = view.read_discussion("old-talk").expect("archived record");
        assert_eq!(archived.slug, "old-talk");
        assert_eq!((archived.text.as_str(), archived.archived), ("newer text", true));
        assert_eq!(archived.path, PathBuf::from("discussions/archive/2026-11-01-old-talk-2.md"));
        assert!(view.read_discussion("nope").is_none());
    }

    fn list_live_discussions() {
        let port = fixture();
        let docs = PortView::new(&port, scope()).list_live_discussions();
        assert_eq!(docs.len(), 1);
        assert_eq!((docs[0].slug.as_str(), docs[0].text.as_str()), ("board-search", "live text"));
        assert!(!docs[0].archived);
    }

    fn list_archived_discussions() {
        let port = fixture();
        let docs = PortView::new(&port, scope()).list_archived_discussions();
        let rows: Vec<(&str, &str)> =
            docs.iter().map(|d| (d.slug.as_str(), d.text.as_str())).collect();
        // Stored-name order; the slug keeps a `-N` reuse suffix.
        assert_eq!(rows, [("old-talk-2", "newer text"), ("old-talk", "archived text")]);
        assert!(docs.iter().all(|d| d.archived));
        assert_eq!(docs[1].path, PathBuf::from("discussions/archive/2026-11-01-old-talk.md"));
    }

    fn read_workflow_config() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).read_workflow_config().as_deref(),
            Some("schema: spec-driven\n")
        );
        let empty = MemoryStorePort::new();
        assert_eq!(PortView::new(&empty, scope()).read_workflow_config(), None);
    }

    fn read_language() {
        let port = fixture();
        assert_eq!(
            PortView::new(&port, scope()).read_language().as_deref(),
            Some("# Vocabulary\n")
        );
    }

    /// The staged operations of a view, one line each.
    fn staged(view: PortView) -> Vec<String> {
        view.into_ops()
            .expect("no store failure")
            .iter()
            .map(|op| match op {
                Op::Put { key, content, expected: Expected::Absent } => {
                    format!("put {} new {content:?}", key.as_str())
                }
                Op::Put { key, content, expected: Expected::At(version) } => {
                    format!("put {} at {version} {content:?}", key.as_str())
                }
                Op::Delete { key, expected } => format!("delete {} at {expected}", key.as_str()),
            })
            .collect()
    }

    fn create_change() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(
            view.create_change("add-z", META_Y).unwrap(),
            PathBuf::from("speclink/changes/add-z")
        );
        assert!(view.change_exists("add-z"));
        let names: Vec<String> = view.list_changes().into_iter().map(|c| c.name).collect();
        assert_eq!(names, ["add-x", "add-y", "add-z"]);
        assert!(view.create_change("a/b", META_Y).is_err(), "a name that cannot be a key");
        assert_eq!(staged(view), [format!("put changes/add-z/.meta.yaml new {META_Y:?}")]);
    }

    fn write_change_meta() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.write_change_meta("add-x", "schema: other\n").unwrap();
        assert_eq!(view.read_change_meta("add-x").as_deref(), Some("schema: other\n"));
        assert_eq!(staged(view), ["put changes/add-x/.meta.yaml at 1 \"schema: other\\n\""]);
    }

    fn delete_change() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.delete_change("add-x").unwrap();
        assert!(!view.change_exists("add-x"));
        assert!(view.find_change("add-x").is_none());
        assert_eq!(view.read_artifact("add-x", "tasks.md"), None);
        assert_eq!(view.read_evidence("add-x"), None, "no evidence outlives its change");
        assert!(view.delta_capabilities("add-x").is_empty());
        let names: Vec<String> = view.list_changes().into_iter().map(|c| c.name).collect();
        assert_eq!(names, ["add-y"]);
        assert_eq!(
            staged(view),
            [
                "delete changes/add-x/.evidence.json at 1",
                "delete changes/add-x/.meta.yaml at 1",
                "delete changes/add-x/proposal.md at 1",
                "delete changes/add-x/specs/auth/spec.md at 1",
                "delete changes/add-x/tasks.md at 1",
            ]
        );
    }

    fn write_artifact() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(
            view.write_artifact("add-x", "design.md", "# D\n").unwrap(),
            PathBuf::from("speclink/changes/add-x/design.md")
        );
        assert_eq!(view.read_artifact("add-x", "design.md").as_deref(), Some("# D\n"));
        assert!(view.artifact_exists("add-x", "design.md"));
        assert!(view.write_artifact("add-x", "../x.md", "x").is_err());
        // The display location uses `/` on every platform, also for a nested
        // artifact: compare the text, not the `PathBuf`.
        assert_eq!(
            view.write_artifact("add-x", "specs/board/spec.md", "## ADDED\n")
                .unwrap()
                .to_string_lossy(),
            "speclink/changes/add-x/specs/board/spec.md"
        );
        assert_eq!(
            staged(view),
            [
                "put changes/add-x/design.md new \"# D\\n\"",
                "put changes/add-x/specs/board/spec.md new \"## ADDED\\n\"",
            ]
        );
    }

    fn delete_artifact() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.delete_artifact("add-x", "tasks.md").unwrap();
        assert!(!view.artifact_exists("add-x", "tasks.md"));
        assert_eq!(view.read_artifact("add-x", "tasks.md"), None);
        // A missing document counts as deleted.
        view.delete_artifact("add-x", "design.md").unwrap();
        assert_eq!(staged(view), ["delete changes/add-x/tasks.md at 1"]);
    }

    fn write_evidence() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.write_evidence("add-y", "{\"version\":2}").unwrap();
        view.write_evidence("add-x", "{\"version\":2}").unwrap();
        assert_eq!(view.read_evidence("add-y").as_deref(), Some("{\"version\":2}"));
        assert_eq!(
            staged(view),
            [
                "put changes/add-x/.evidence.json at 1 \"{\\\"version\\\":2}\"",
                "put changes/add-y/.evidence.json new \"{\\\"version\\\":2}\"",
            ]
        );
    }

    fn write_canonical_spec() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.write_canonical_spec("auth", "# auth v2\n").unwrap();
        view.write_canonical_spec("search", "# search\n").unwrap();
        assert_eq!(view.read_canonical_spec("auth").as_deref(), Some("# auth v2\n"));
        assert!(view.canonical_spec_exists("search"));
        let mut caps = view.list_canonical_capabilities();
        caps.sort();
        assert_eq!(caps, ["auth", "board", "search"]);
        assert_eq!(
            staged(view),
            [
                "put specs/auth/spec.md at 1 \"# auth v2\\n\"",
                "put specs/search/spec.md new \"# search\\n\"",
            ]
        );
    }

    fn archive_change() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        // An earlier write of the same flow travels into the archive.
        view.write_artifact("add-x", "tasks.md", "- [x] 1.1 a\n").unwrap();
        view.archive_change("add-x", "2026-11-11-add-x").unwrap();

        assert!(!view.change_exists("add-x"));
        assert_eq!(view.read_evidence("add-x"), None, "moved, not copied");
        assert!(view.archived_change_exists("2026-11-11-add-x"));
        assert_eq!(view.read_archived_meta("2026-11-11-add-x").as_deref(), Some(META_X));
        assert_eq!(
            view.read_archived_artifact("2026-11-11-add-x", "tasks.md").as_deref(),
            Some("- [x] 1.1 a\n")
        );
        assert_eq!(
            view.read_archived_artifact("2026-11-11-add-x", ".evidence.json").as_deref(),
            Some("{}"),
            "the evidence travels under its relative name"
        );
        assert_eq!(view.archived_delta_capabilities("2026-11-11-add-x"), ["auth"]);
        assert_eq!(
            view.list_archived_changes(),
            ["2026-11-11-add-x", "2026-11-10-add-w", "2026-10-01-add-v"]
        );
        assert!(view.archive_change("add-y", "add-y").is_err(), "not a dated name");
        assert_eq!(
            staged(view),
            [
                "put archive/changes/2026-11-11-add-x/.evidence.json new \"{}\"".to_string(),
                format!("put archive/changes/2026-11-11-add-x/.meta.yaml new {META_X:?}"),
                "put archive/changes/2026-11-11-add-x/proposal.md new \"# Proposal\\n\"".to_string(),
                "put archive/changes/2026-11-11-add-x/specs/auth/spec.md new \"## ADDED Requirements\\n\"".to_string(),
                "put archive/changes/2026-11-11-add-x/tasks.md new \"- [x] 1.1 a\\n\"".to_string(),
                "delete changes/add-x/.evidence.json at 1".to_string(),
                "delete changes/add-x/.meta.yaml at 1".to_string(),
                "delete changes/add-x/proposal.md at 1".to_string(),
                "delete changes/add-x/specs/auth/spec.md at 1".to_string(),
                "delete changes/add-x/tasks.md at 1".to_string(),
            ]
        );
    }

    fn write_archived_meta() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.write_archived_meta("2026-11-10-add-w", "archived_at: x\n").unwrap();
        // The legacy-named change gets the new name; the legacy key stays.
        view.write_archived_meta("2026-10-01-add-v", "archived_at: y\n").unwrap();
        assert_eq!(view.read_archived_meta("2026-11-10-add-w").as_deref(), Some("archived_at: x\n"));
        assert_eq!(view.read_archived_meta("2026-10-01-add-v").as_deref(), Some("archived_at: y\n"));
        assert_eq!(
            staged(view),
            [
                "put archive/changes/2026-10-01-add-v/.meta.yaml new \"archived_at: y\\n\"",
                "put archive/changes/2026-11-10-add-w/.meta.yaml at 1 \"archived_at: x\\n\"",
            ]
        );
    }

    fn write_live_discussion() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(
            view.write_live_discussion("new-talk", "t").unwrap(),
            PathBuf::from("discussions/new-talk.md")
        );
        assert!(view.live_discussion_exists("new-talk"));
        let slugs: Vec<String> = view.list_live_discussions().into_iter().map(|d| d.slug).collect();
        assert_eq!(slugs, ["board-search", "new-talk"]);
        assert_eq!(staged(view), ["put discussions/new-talk.md new \"t\""]);
    }

    fn delete_live_discussion() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.delete_live_discussion("board-search").unwrap();
        assert!(!view.live_discussion_exists("board-search"));
        assert!(view.list_live_discussions().is_empty());
        assert_eq!(staged(view), ["delete discussions/board-search.md at 1"]);
    }

    fn archive_discussion() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.archive_discussion("nope", "2026-11-10").unwrap(), None);
        assert_eq!(
            view.archive_discussion("board-search", "2026-11-10").unwrap().as_deref(),
            Some("2026-11-10-board-search.md")
        );
        assert!(!view.live_discussion_exists("board-search"));
        assert!(view.archived_discussion_exists("board-search"));
        let moved = view.read_discussion("board-search").expect("the archived record");
        assert_eq!((moved.text.as_str(), moved.archived), ("live text", true));
        // A reused slug on the same day gets the next free `-N`.
        view.write_live_discussion("old-talk", "third").unwrap();
        assert_eq!(
            view.archive_discussion("old-talk", "2026-11-01").unwrap().as_deref(),
            Some("2026-11-01-old-talk-3.md")
        );
        // The creation date comes from a document a person can edit. A date
        // with more text after it (a time, for one) still gives a dated name,
        // so the record is found by its slug afterwards.
        view.write_live_discussion("odd", "odd text").unwrap();
        assert_eq!(
            view.archive_discussion("odd", "2026-11-10T08:00").unwrap().as_deref(),
            Some("2026-11-10-odd.md")
        );
        assert!(!view.live_discussion_exists("odd"));
        assert!(view.archived_discussion_exists("odd"));
        let odd = view.read_discussion("odd").expect("found by its slug");
        assert_eq!((odd.text.as_str(), odd.archived), ("odd text", true));
        // Text with no date keeps the fs store's name: the whole text, then
        // the slug. A listing shows the record; its slug alone does not find it.
        view.write_live_discussion("undated", "u").unwrap();
        assert_eq!(
            view.archive_discussion("undated", "someday").unwrap().as_deref(),
            Some("someday-undated.md")
        );
        assert!(!view.archived_discussion_exists("undated"));
        assert!(view
            .list_archived_discussions()
            .iter()
            .any(|d| d.slug == "someday-undated" && d.text == "u"));
        view.write_live_discussion("worse", "x").unwrap();
        assert!(view.archive_discussion("worse", "2026/11/10").is_err(), "not one segment");
        assert_eq!(
            staged(view),
            [
                "put archive/discussions/2026-11-01-old-talk-3.md new \"third\"",
                "put archive/discussions/2026-11-10-board-search.md new \"live text\"",
                "put archive/discussions/2026-11-10-odd.md new \"odd text\"",
                "put archive/discussions/someday-undated.md new \"u\"",
                "delete discussions/board-search.md at 1",
                "put discussions/worse.md new \"x\"",
            ]
        );
    }

    fn write_workflow_config() {
        // An existing document is written at the version first seen.
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_workflow_config().as_deref(), Some("schema: spec-driven\n"));
        view.write_workflow_config("schema: my-flow\n").unwrap();
        assert_eq!(view.read_workflow_config().as_deref(), Some("schema: my-flow\n"));
        assert_eq!(staged(view), ["put config.yaml at 1 \"schema: my-flow\\n\""]);
        // A missing document is created.
        let empty = MemoryStorePort::new();
        let view = PortView::new(&empty, scope());
        view.write_workflow_config("tdd: true\n").unwrap();
        assert_eq!(staged(view), ["put config.yaml new \"tdd: true\\n\""]);
    }

    fn board_ranks() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        assert!(view.board_ranks(&view.list_changes()).is_empty(), "no document, no ranks");
        seed(&port, 10_000, &[("board/order", r#"{"changes":{"add-x":"n","gone":"c"},"discussions":{"t":"f"}}"#)]);
        let view = PortView::new(&port, scope());
        let ranks = view.board_ranks(&view.list_changes());
        assert_eq!(ranks, BTreeMap::from([("add-x".to_string(), "n".to_string())]), "only the active changes");
    }

    fn write_board_ranks() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        let pair = |c: &str, k: &str| (c.to_string(), k.to_string());
        view.write_board_ranks(&[pair("add-y", "f"), pair("add-x", "n")]).unwrap();
        view.write_board_ranks(&[pair("add-y", "g")]).unwrap();
        assert_eq!(view.board_ranks(&view.list_changes())["add-y"], "g", "a command sees its own writes");
        let doc = r#"{"changes":{"add-x":"n","add-y":"g"},"discussions":{}}"#;
        assert_eq!(staged(view), [format!("put board/order new {doc:?}")], "one write");
        // An existing document: the orphan goes, the discussion section stays.
        let port = fixture();
        seed(&port, 10_000, &[("board/order", r#"{"changes":{"gone":"c"},"discussions":{"t":"f"}}"#)]);
        let view = PortView::new(&port, scope());
        view.write_board_ranks(&[pair("add-x", "n")]).unwrap();
        let doc = r#"{"changes":{"add-x":"n"},"discussions":{"t":"f"}}"#;
        assert_eq!(staged(view), [format!("put board/order at 3 {doc:?}")], "orphan pruned");
    }

    /// One row per method of `speclink_core::store::Store`.
    const COVERAGE: &[(&str, fn())] = &[
        ("write_workflow_config", write_workflow_config),
        ("create_change", create_change),
        ("write_change_meta", write_change_meta),
        ("delete_change", delete_change),
        ("write_artifact", write_artifact),
        ("delete_artifact", delete_artifact),
        ("write_evidence", write_evidence),
        ("write_canonical_spec", write_canonical_spec),
        ("archive_change", archive_change),
        ("write_archived_meta", write_archived_meta),
        ("write_live_discussion", write_live_discussion),
        ("delete_live_discussion", delete_live_discussion),
        ("archive_discussion", archive_discussion),
        ("spec_dir_display", spec_dir_display),
        ("change_meta_display", change_meta_display),
        ("list_changes", list_changes),
        ("find_change", find_change),
        ("change_exists", change_exists),
        ("updated_at_secs", updated_at_secs),
        ("read_change_meta", read_change_meta),
        ("supports_ownership", supports_ownership),
        ("read_artifact", read_artifact),
        ("artifact_exists", artifact_exists),
        ("read_evidence", read_evidence),
        ("delta_capabilities", delta_capabilities),
        ("has_capability_dirs", has_capability_dirs),
        ("list_canonical_capabilities", list_canonical_capabilities),
        ("canonical_spec_exists", canonical_spec_exists),
        ("read_canonical_spec", read_canonical_spec),
        ("canonical_spec_path", canonical_spec_path),
        ("list_archived_changes", list_archived_changes),
        ("archived_change_exists", archived_change_exists),
        ("read_archived_meta", read_archived_meta),
        ("read_archived_artifact", read_archived_artifact),
        ("archived_delta_capabilities", archived_delta_capabilities),
        ("live_discussion_exists", live_discussion_exists),
        ("archived_discussion_exists", archived_discussion_exists),
        ("live_discussion_path", live_discussion_path),
        ("read_live_discussion", read_live_discussion),
        ("read_discussion", read_discussion),
        ("list_live_discussions", list_live_discussions),
        ("list_archived_discussions", list_archived_discussions),
        ("read_workflow_config", read_workflow_config),
        ("read_language", read_language),
        ("board_ranks", board_ranks),
        ("write_board_ranks", write_board_ranks),
    ];

    #[test]
    fn every_covered_store_method_works_on_three_operations() {
        for (method, check) in COVERAGE {
            eprintln!("store method: {method}");
            check();
        }
    }

    #[test]
    fn the_coverage_table_names_every_method_of_the_store_interface() {
        // 「三個操作足以供應全部方法」的證明要涵蓋整個介面：介面多一個方法而
        // 表沒有跟上時，這個測試失敗。
        let source = include_str!("../../../engine/speclink-core/src/store.rs");
        let (_, body) = source.split_once("pub trait Store").expect("the Store trait");
        let mut declared: Vec<&str> = body
            .lines()
            .filter_map(|line| line.trim_start().strip_prefix("fn "))
            .filter_map(|rest| rest.split_once('(').map(|(name, _)| name))
            .collect();
        declared.sort_unstable();
        let mut covered: Vec<&str> = COVERAGE.iter().map(|(method, _)| *method).collect();
        covered.sort_unstable();
        assert!(declared.len() > 40, "the trait was found: {declared:?}");
        assert_eq!(covered, declared);
    }

    // --- 寫入面：暫存、基準版本、搬移 ---

    #[test]
    fn a_command_sees_its_own_writes() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.list_changes();
        view.write_artifact("add-y", "proposal.md", "# Y\n").unwrap();
        view.delete_artifact("add-x", "proposal.md").unwrap();
        assert_eq!(view.read_artifact("add-y", "proposal.md").as_deref(), Some("# Y\n"));
        assert!(view.artifact_exists("add-y", "proposal.md"), "列得到剛建立的文件");
        assert!(!view.artifact_exists("add-x", "proposal.md"), "列不到剛刪除的文件");
        view.write_artifact("add-y", "specs/board/spec.md", "## ADDED\n").unwrap();
        assert_eq!(view.delta_capabilities("add-y"), ["board"]);
        view.delete_artifact("add-x", "specs/auth/spec.md").unwrap();
        assert!(view.delta_capabilities("add-x").is_empty());
        assert!(!view.has_capability_dirs("add-x"));
    }

    #[test]
    fn the_expected_version_is_the_first_one_seen() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        // First seen in a listing at version 1.
        view.list_changes();
        // Another writer moves the document to version 3.
        overwrite(&port, 20_000, "changes/add-x/tasks.md", "- [x] 1.1 a\n");
        view.write_artifact("add-x", "tasks.md", "first").unwrap();
        view.write_artifact("add-x", "tasks.md", "second").unwrap();
        // 同一個鍵寫兩次：一筆寫入，預期版本不變。
        assert_eq!(staged(view), ["put changes/add-x/tasks.md at 1 \"second\""]);
    }

    #[test]
    fn a_key_written_without_a_look_is_read_once_first() {
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        view.write_artifact("add-x", "design.md", "# D\n").unwrap();
        view.write_artifact("add-x", "design.md", "# D2\n").unwrap();
        view.write_canonical_spec("auth", "# auth v2\n").unwrap();
        assert_eq!(
            probe.reads(),
            [vec!["changes/add-x/design.md"], vec!["specs/auth/spec.md"]]
        );
        assert!(probe.lists().is_empty());
        assert_eq!(
            staged(view),
            [
                "put changes/add-x/design.md new \"# D2\\n\"",
                "put specs/auth/spec.md at 1 \"# auth v2\\n\"",
            ]
        );
    }

    #[test]
    fn a_key_under_a_listed_prefix_is_written_without_a_read() {
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        view.delta_capabilities("add-x");
        view.write_artifact("add-x", "design.md", "# D\n").unwrap();
        view.write_artifact("add-x", "tasks.md", "- [x] 1.1 a\n").unwrap();
        assert!(probe.reads().is_empty(), "the listing gave both versions");
        assert_eq!(
            staged(view),
            [
                "put changes/add-x/design.md new \"# D\\n\"",
                "put changes/add-x/tasks.md at 1 \"- [x] 1.1 a\\n\"",
            ]
        );
    }

    #[test]
    fn deleting_a_document_created_in_this_execution_leaves_nothing_staged() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.write_artifact("add-x", "design.md", "# D\n").unwrap();
        view.delete_artifact("add-x", "design.md").unwrap();
        view.write_live_discussion("new-talk", "t").unwrap();
        view.delete_live_discussion("new-talk").unwrap();
        assert_eq!(staged(view), Vec::<String>::new());
    }

    #[test]
    fn deleting_then_writing_a_document_is_one_write_at_its_first_version() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.delete_artifact("add-x", "tasks.md").unwrap();
        view.write_artifact("add-x", "tasks.md", "again").unwrap();
        assert_eq!(staged(view), ["put changes/add-x/tasks.md at 1 \"again\""]);
    }

    #[test]
    fn archiving_a_change_reads_its_documents_in_one_call() {
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        view.archive_change("add-x", "2026-11-11-add-x").unwrap();
        assert_eq!(probe.lists(), ["changes/"]);
        let reads = probe.reads();
        assert_eq!(reads.len(), 1, "sources and targets in one read: {reads:?}");
        assert_eq!(reads[0].len(), 10);
    }

    #[test]
    fn a_store_failure_during_a_write_is_kept_and_nothing_is_handed_out() {
        // The read that gives the expected version fails.
        let probe = Probe::new(fixture()).failing_at(1);
        let view = PortView::new(&probe, scope());
        view.write_artifact("add-x", "design.md", "# D\n").unwrap();
        assert_eq!(
            view.into_ops(),
            Err(PortError::Unavailable { message: "probe failure".into() })
        );
    }

    // --- 既有文件的名稱不照規則時 ---

    #[test]
    fn an_archived_change_without_a_dated_name_is_still_readable() {
        // A store can hold older archived changes whose names carry no date.
        // What a listing shows, a read must find.
        let port = fixture();
        seed(
            &port,
            6_000,
            &[
                ("archive/changes/legacy-name/.meta.yaml", "created: old\n"),
                ("archive/changes/legacy-name/specs/auth/spec.md", "## ADDED\n"),
            ],
        );
        for listed_first in [false, true] {
            let view = PortView::new(&port, scope());
            if listed_first {
                assert!(view.list_archived_changes().contains(&"legacy-name".to_string()));
            }
            assert!(view.archived_change_exists("legacy-name"));
            assert_eq!(view.read_archived_meta("legacy-name").as_deref(), Some("created: old\n"));
            assert_eq!(
                view.read_archived_artifact("legacy-name", "specs/auth/spec.md").as_deref(),
                Some("## ADDED\n")
            );
            assert_eq!(view.archived_delta_capabilities("legacy-name"), ["auth"]);
            // A new archived document must have a dated name.
            assert!(view.write_archived_meta("legacy-name", "x").is_err());
        }
    }

    #[test]
    fn a_change_named_like_the_archive_folder_is_not_a_change() {
        // `archive` cannot be a change name (the key naming rules). A listing
        // must not show what `find_change` cannot find.
        let port = fixture();
        seed(&port, 6_000, &[("changes/archive/.meta.yaml", META_Y)]);
        let view = PortView::new(&port, scope());
        let names: Vec<String> = view.list_changes().into_iter().map(|c| c.name).collect();
        assert_eq!(names, ["add-x", "add-y"]);
        assert!(view.find_change("archive").is_none());
        assert!(!view.change_exists("archive"));
    }

    #[test]
    fn an_undated_archived_record_matches_its_whole_name_only() {
        // An older record `plan-2` is the discussion `plan-2`, not the second
        // archive of `plan`: the `-N` suffix belongs to dated names.
        let port = fixture();
        seed(&port, 6_000, &[("archive/discussions/plan-2.md", "old plan 2")]);
        let view = PortView::new(&port, scope());
        assert!(!view.archived_discussion_exists("plan"));
        assert!(view.read_discussion("plan").is_none());
        assert!(view.archived_discussion_exists("plan-2"));
        let record = view.read_discussion("plan-2").expect("the older record");
        assert_eq!((record.text.as_str(), record.archived), ("old plan 2", true));
    }

    // --- 經儲存接口執行時只讀取用到的文件 ---

    #[test]
    fn a_prefix_is_listed_at_most_once() {
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        view.list_changes();
        view.list_changes();
        view.delta_capabilities("add-x");
        view.updated_at_secs("add-x");
        assert_eq!(probe.lists(), ["changes/"]);
    }

    #[test]
    fn a_key_is_read_at_most_once() {
        // 「同一個鍵不重複讀取」
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        assert_eq!(view.read_artifact("add-x", "tasks.md").as_deref(), Some("- [ ] 1.1 a\n"));
        assert_eq!(view.read_artifact("add-x", "tasks.md").as_deref(), Some("- [ ] 1.1 a\n"));
        assert!(view.artifact_exists("add-x", "tasks.md"));
        assert_eq!(view.read_artifact("add-x", "design.md"), None);
        assert_eq!(view.read_artifact("add-x", "design.md"), None, "「沒有」也記下來");
        assert_eq!(
            probe.reads(),
            [vec!["changes/add-x/tasks.md"], vec!["changes/add-x/design.md"]]
        );
        assert!(probe.lists().is_empty());
    }

    #[test]
    fn three_metadata_documents_come_from_one_read() {
        // 「多份 metadata 一次讀取」：3 個進行中的變更。
        let port = MemoryStorePort::new();
        seed(
            &port,
            1,
            &[
                ("changes/a/.meta.yaml", META_Y),
                ("changes/a/tasks.md", "x"),
                ("changes/b/.meta.yaml", META_Y),
                ("changes/c/.meta.yaml", META_Y),
                ("archive/changes/2026-11-10-old/.meta.yaml", META_Y),
            ],
        );
        let probe = Probe::new(port);
        let view = PortView::new(&probe, scope());
        assert_eq!(view.list_changes().len(), 3);
        assert_eq!(probe.lists(), ["changes/"]);
        assert_eq!(
            probe.reads(),
            [vec!["changes/a/.meta.yaml", "changes/b/.meta.yaml", "changes/c/.meta.yaml"]]
        );
    }

    #[test]
    fn a_document_missing_from_a_listed_prefix_is_not_read() {
        let probe = Probe::new(fixture());
        let view = PortView::new(&probe, scope());
        view.list_changes();
        let reads = probe.reads().len();
        assert_eq!(view.read_artifact("add-x", "design.md"), None);
        assert!(!view.change_exists("nope"));
        assert_eq!(probe.reads().len(), reads, "the listing already says it is not there");
    }

    #[test]
    fn a_store_failure_reads_as_nothing_and_is_kept_for_the_caller() {
        let probe = Probe::new(fixture()).failing_at(2);
        let view = PortView::new(&probe, scope());
        assert_eq!(view.read_language().as_deref(), Some("# Vocabulary\n"));
        assert_eq!(view.read_artifact("add-x", "tasks.md"), None, "the failed read");
        // After a failure the view asks the store nothing more.
        assert!(view.list_changes().is_empty());
        assert_eq!(probe.lists().len() + probe.reads().len(), 2);
        assert!(probe.commits().is_empty());
        assert_eq!(
            view.into_ops(),
            Err(PortError::Unavailable { message: "probe failure".into() })
        );
    }

    #[test]
    fn a_store_that_answers_with_the_wrong_number_of_documents_is_a_failure() {
        // A host's store that breaks "one result per requested key": a missing
        // result must not read as "no document".
        struct Short(MemoryStorePort);
        impl StorePort for Short {
            fn list(
                &self,
                scope: &ScopeId,
                prefix: &KeyPrefix,
            ) -> Result<speclink_store_port::Listing, PortError> {
                self.0.list(scope, prefix)
            }
            fn read(
                &self,
                scope: &ScopeId,
                keys: &[DocKey],
            ) -> Result<Vec<Option<speclink_store_port::Document>>, PortError> {
                let mut docs = self.0.read(scope, keys)?;
                docs.pop();
                Ok(docs)
            }
            fn commit(
                &self,
                scope: &ScopeId,
                commit: &speclink_store_port::Commit,
            ) -> Result<Version, PortError> {
                self.0.commit(scope, commit)
            }
        }
        let port = Short(fixture());
        let view = PortView::new(&port, scope());
        assert_eq!(view.read_archived_meta("2026-10-01-add-v"), None);
        match view.into_ops() {
            Err(PortError::Backend { message }) => {
                assert_eq!(message, "the store returned 1 results for a read of 2 keys")
            }
            other => panic!("expected a backend failure, got {other:?}"),
        }
    }

    #[test]
    fn a_view_without_writes_has_no_operations() {
        let port = fixture();
        let view = PortView::new(&port, scope());
        view.list_changes();
        assert_eq!(view.into_ops(), Ok(Vec::new()));
    }
}

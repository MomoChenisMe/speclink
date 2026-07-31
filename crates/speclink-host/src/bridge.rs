//! The engine-over-TeamStore execution bridge (design 決策一).
//!
//! The Engine's command layer reads and writes through the synchronous
//! `speclink_core::store::Store` seam — historically the local `openspec/`
//! filesystem. This bridge is a second supplier of that seam backed by the
//! TeamStore contract: reads are served from a consistent snapshot of the
//! scope, a mutating verb's writes are captured as staged operations, and on
//! success they commit — together with the verb's domain events — through the
//! Host's UoW/event commit path, atomically. The Engine command layer is not
//! forked; the bridge is only the store view it runs against.

use crate::commit::{begin_unit_of_work, commit_with_events};
use crate::context::SpeclinkExecutionContext;
use speclink_core::command::{
    execute as engine_execute, Command, CommandError, CommandOutcome, DomainEvent, ExecutionContext,
};
use speclink_core::model::{Change, ChangeMeta};
use speclink_core::store::{DiscussionDoc, Store};
use speclink_store::{
    DocumentId, ExpectedRevision, Revision, Scope, StagedOp, StoreError, TeamStore,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A bridge execution failure, keeping the two error vocabularies distinct so
/// the server maps each at exactly one place (design 決策六): the Engine
/// command layer's five typed codes, or the store's six-class failures —
/// including `revision_conflict` with its expected/actual detail preserved.
#[derive(Debug)]
pub enum BridgeError {
    /// The Engine command layer refused before or during the flow.
    Command(CommandError),
    /// The store rejected the atomic commit (CAS conflict, unavailable, …).
    Store(StoreError),
}

/// The result of running one command over a TeamStore scope: the typed
/// outcome, the domain events the execution produced (empty for queries), and
/// — for a mutation that committed — the resulting project revision (`None`
/// when nothing was written, i.e. queries and no-op mutations).
#[derive(Debug)]
pub struct BridgeExecution {
    pub outcome: CommandOutcome,
    pub events: Vec<DomainEvent>,
    pub revision: Option<Revision>,
}

/// The Engine execution context the bridge runs under. Identity comes from the
/// Host-resolved context; there is no local workspace or user schema directory
/// in server mode (built-in schemas resolve without one), and env overrides do
/// not apply — policy was already resolved at the Host boundary.
fn engine_context(ctx: &SpeclinkExecutionContext) -> ExecutionContext {
    ExecutionContext {
        actor: ctx.actor.display().map(str::to_string),
        repo: Some(ctx.repo.as_str().to_string()),
        ..Default::default()
    }
}

/// Run one Engine command over `store`'s scope (resolved from `ctx`) through
/// the bridge. Reads are served from a consistent snapshot; a mutating verb's
/// writes are committed atomically with its domain events. Queries never open
/// a unit of work.
pub fn execute(
    store: &dyn TeamStore,
    ctx: &SpeclinkExecutionContext,
    cmd: Command,
) -> Result<BridgeExecution, BridgeError> {
    let scope = Scope::new(ctx.project.clone(), ctx.repo.clone());
    let view = BridgeStore::materialize(store, &scope).map_err(BridgeError::Store)?;
    let (outcome, events) =
        engine_execute(&view, &engine_context(ctx), cmd).map_err(BridgeError::Command)?;
    let staged = view.into_staged();
    if staged.is_empty() {
        // A query (or a no-op mutation): nothing to commit.
        return Ok(BridgeExecution { outcome, events, revision: None });
    }
    let mut uow =
        begin_unit_of_work(store, ctx, &command_label(&outcome)).map_err(BridgeError::Store)?;
    for op in staged {
        match op {
            StagedOp::Put { doc, content, expected } => uow.put(doc, content, expected),
            StagedOp::Delete { doc, expected } => uow.delete(doc, expected),
        }
    }
    let revision = commit_with_events(store, ctx, uow, &events).map_err(BridgeError::Store)?;
    Ok(BridgeExecution { outcome, events, revision: Some(revision) })
}

/// A short, stable command label for the unit-of-work's audit record. Derived
/// from the outcome so it names the verb that produced the writes.
fn command_label(outcome: &CommandOutcome) -> String {
    match outcome {
        CommandOutcome::NewChange(_) => "new-change",
        CommandOutcome::NewArtifact(_) => "new-artifact",
        CommandOutcome::TaskDone(_) => "task-done",
        CommandOutcome::TaskUndone(_) => "task-undone",
        CommandOutcome::InProgressAdd(_) => "in-progress-add",
        CommandOutcome::InProgressRemove(_) => "in-progress-remove",
        CommandOutcome::Archive(_) => "archive",
        CommandOutcome::Discard(_) => "discard",
        CommandOutcome::DiscussNew(_) => "discuss-new",
        CommandOutcome::DiscussContext(_) => "discuss-context",
        CommandOutcome::DiscussAddRound(_) => "discuss-add-round",
        CommandOutcome::DiscussConclude(_) => "discuss-conclude",
        CommandOutcome::DiscussPromote(_) => "discuss-promote",
        CommandOutcome::DiscussLink(_) => "discuss-link",
        CommandOutcome::DiscussSeal(_) => "discuss-seal",
        CommandOutcome::DiscussArchive(_) => "discuss-archive",
        CommandOutcome::DiscussDiscard(_) => "discuss-discard",
        _ => "command",
    }
    .to_string()
}

/// The bridge's realization of the Engine `Store` seam over one TeamStore
/// scope. `view` is the materialized read-after-write content map; `base`
/// carries each materialized document's revision (the CAS precondition writes
/// derive), fixed at materialize time; `staged` collects the writes to commit,
/// coalesced per document (last write of a document wins).
///
/// Crate-visible so the Host's own read-only query entry points can compose the
/// Engine over a scope, and no wider: an adapter holding this view could run
/// arbitrary Engine functions outside the Host's composition points, and — since
/// the Engine's `Store` seam includes writes — could stage writes that are then
/// silently dropped for want of a commit. Both stay impossible by visibility.
pub(crate) struct BridgeStore {
    view: RefCell<BTreeMap<DocumentId, String>>,
    base: BTreeMap<DocumentId, Revision>,
    staged: RefCell<BTreeMap<DocumentId, StagedOp>>,
}

impl BridgeStore {
    /// Build a consistent read view of `scope`: a snapshot fixes the revision
    /// base, `export` enumerates the scope's documents (the contract's only
    /// enumeration seam — `Snapshot` reads are point lookups), and each is read
    /// back through the same snapshot so content and revision agree at one
    /// project revision.
    pub(crate) fn materialize(
        store: &dyn TeamStore,
        scope: &Scope,
    ) -> Result<BridgeStore, StoreError> {
        let snapshot = store.snapshot(scope)?;
        let bundle = store.export(scope)?;
        let mut view = BTreeMap::new();
        let mut base = BTreeMap::new();
        for entry in bundle.documents {
            // A document `export` lists but the fixed-point snapshot does not
            // hold was written after the snapshot: it is outside this view.
            if let Some(doc) = snapshot.read(&entry.doc)? {
                base.insert(entry.doc.clone(), doc.revision);
                view.insert(entry.doc, doc.content);
            }
        }
        Ok(BridgeStore {
            view: RefCell::new(view),
            base,
            staged: RefCell::new(BTreeMap::new()),
        })
    }

    /// The staged operations to commit, in a deterministic document order.
    fn into_staged(self) -> Vec<StagedOp> {
        self.staged.into_inner().into_values().collect()
    }

    /// Read a materialized document's content.
    fn read_doc(&self, doc: &DocumentId) -> Option<String> {
        self.view.borrow().get(doc).cloned()
    }

    /// Capture a write, updating the read-after-write view. The CAS
    /// precondition is derived from the materialized base — creation when the
    /// document was absent, a revision check when it existed — and stays fixed
    /// however many times the document is written within one execution.
    fn put(&self, doc: DocumentId, content: String) {
        let expected = self
            .base
            .get(&doc)
            .map(|r| ExpectedRevision::At(*r))
            .unwrap_or(ExpectedRevision::Absent);
        self.view.borrow_mut().insert(doc.clone(), content.clone());
        self.staged
            .borrow_mut()
            .insert(doc.clone(), StagedOp::Put { doc, content, expected });
    }

    /// Capture a delete. A document created within this execution (absent from
    /// the base) never reached the store, so its staging is simply dropped.
    fn del(&self, doc: DocumentId) {
        self.view.borrow_mut().remove(&doc);
        match self.base.get(&doc) {
            Some(rev) => {
                self.staged
                    .borrow_mut()
                    .insert(doc.clone(), StagedOp::Delete { doc, expected: *rev });
            }
            None => {
                self.staged.borrow_mut().remove(&doc);
            }
        }
    }

    /// Build a `Change` from raw metadata the way the fs adapter does: a corrupt
    /// document yields the default meta plus the parse reason in `meta_error`.
    fn change_from(name: &str, meta_text: Option<&str>) -> Change {
        let (meta, meta_error) = match ChangeMeta::from_text(meta_text) {
            Ok(meta) => (meta, None),
            Err(reason) => (ChangeMeta::default(), Some(reason)),
        };
        Change {
            name: name.to_string(),
            meta,
            meta_error,
            // fs adapter 的 workspace 相對形狀：引擎以 dir 組錯誤訊息與
            // 路徑輸出，remote 模式必須與 fs 模式逐位元一致（verb-contract）。
            dir: PathBuf::from(format!("openspec/changes/{name}")),
        }
    }

    /// Delta capability names of a change: exactly `specs/<cap>/spec.md`, one
    /// level deep — mirrors the fs adapter's rule.
    fn delta_caps_of(&self, change: &str) -> Vec<String> {
        let mut caps: Vec<String> = self
            .view
            .borrow()
            .keys()
            .filter_map(|id| match id {
                DocumentId::ChangeArtifact { change: c, artifact } if c == change => {
                    spec_capability(artifact)
                }
                _ => None,
            })
            .collect();
        caps.sort();
        caps
    }
}

/// The capability name of a `specs/<cap>/spec.md` artifact path, or `None` when
/// the path is not exactly that shape.
fn spec_capability(artifact: &str) -> Option<String> {
    let rest = artifact.strip_prefix("specs/")?.strip_suffix("/spec.md")?;
    if rest.is_empty() || rest.contains('/') {
        None
    } else {
        Some(rest.to_string())
    }
}

impl Store for BridgeStore {
    // --- changes ---

    fn list_changes(&self) -> Vec<Change> {
        let mut out: Vec<Change> = self
            .view
            .borrow()
            .iter()
            .filter_map(|(id, content)| match id {
                DocumentId::ChangeMeta { change } => {
                    Some(BridgeStore::change_from(change, Some(content)))
                }
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    fn find_change(&self, name: &str) -> Option<Change> {
        self.read_doc(&DocumentId::ChangeMeta { change: name.to_string() })
            .map(|meta| BridgeStore::change_from(name, Some(&meta)))
    }

    fn change_exists(&self, name: &str) -> bool {
        self.view
            .borrow()
            .contains_key(&DocumentId::ChangeMeta { change: name.to_string() })
    }

    fn create_change(&self, name: &str, meta_text: &str) -> anyhow::Result<PathBuf> {
        self.put(DocumentId::ChangeMeta { change: name.to_string() }, meta_text.to_string());
        Ok(PathBuf::from(format!("openspec/changes/{name}")))
    }

    fn updated_at_secs(&self, _name: &str) -> u64 {
        // No wall-clock mtime in the store; ordering by "modified" is not a
        // team-mode sort target (the twin scenarios sort by name).
        0
    }

    fn read_change_meta(&self, name: &str) -> Option<String> {
        self.read_doc(&DocumentId::ChangeMeta { change: name.to_string() })
    }

    fn write_change_meta(&self, name: &str, content: &str) -> anyhow::Result<()> {
        self.put(DocumentId::ChangeMeta { change: name.to_string() }, content.to_string());
        Ok(())
    }

    fn delete_change(&self, name: &str) -> anyhow::Result<()> {
        let targets: Vec<DocumentId> = self
            .view
            .borrow()
            .keys()
            .filter(|id| match id {
                DocumentId::ChangeMeta { change } => change == name,
                DocumentId::ChangeArtifact { change, .. } => change == name,
                _ => false,
            })
            .cloned()
            .collect();
        for doc in targets {
            self.del(doc);
        }
        Ok(())
    }

    // --- artifacts ---

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        self.read_doc(&DocumentId::ChangeArtifact {
            change: change.to_string(),
            artifact: artifact.to_string(),
        })
    }

    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.put(
            DocumentId::ChangeArtifact {
                change: change.to_string(),
                artifact: artifact.to_string(),
            },
            content.to_string(),
        );
        Ok(PathBuf::from(format!("openspec/changes/{change}/{artifact}")))
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.view.borrow().contains_key(&DocumentId::ChangeArtifact {
            change: change.to_string(),
            artifact: artifact.to_string(),
        })
    }

    // --- delta specs ---

    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        self.delta_caps_of(change)
    }

    fn has_capability_dirs(&self, change: &str) -> bool {
        self.view.borrow().keys().any(|id| match id {
            DocumentId::ChangeArtifact { change: c, artifact } => {
                c == change && artifact.starts_with("specs/")
            }
            _ => false,
        })
    }

    // --- canonical specs ---

    fn list_canonical_capabilities(&self) -> Vec<String> {
        self.view
            .borrow()
            .keys()
            .filter_map(|id| match id {
                DocumentId::CanonicalSpec { capability } => Some(capability.clone()),
                _ => None,
            })
            .collect()
    }

    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.view
            .borrow()
            .contains_key(&DocumentId::CanonicalSpec { capability: cap.to_string() })
    }

    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        self.read_doc(&DocumentId::CanonicalSpec { capability: cap.to_string() })
    }

    fn write_canonical_spec(&self, cap: &str, content: &str) -> anyhow::Result<()> {
        self.put(DocumentId::CanonicalSpec { capability: cap.to_string() }, content.to_string());
        Ok(())
    }

    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        PathBuf::from(format!("specs/{cap}/spec.md"))
    }

    // --- archive ---

    fn list_archived_changes(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .view
            .borrow()
            .keys()
            .filter_map(|id| match id {
                DocumentId::ArchivedChange { change, .. } => Some(change.clone()),
                _ => None,
            })
            .collect();
        names.sort_by(|a, b| b.cmp(a));
        names.dedup();
        names
    }

    fn archived_change_exists(&self, dated_name: &str) -> bool {
        self.view.borrow().keys().any(|id| matches!(
            id,
            DocumentId::ArchivedChange { change, .. } if change == dated_name
        ))
    }

    fn archive_change(&self, name: &str, dated_name: &str) -> anyhow::Result<()> {
        // Move every document of the active change to the archive: metadata to
        // `.openspec.yaml`, each artifact to its own relative name. Reads the
        // read-after-write view so an earlier write in the same flow (the
        // mark-tasks-complete pre-write) travels into the archive.
        let moves: Vec<(DocumentId, DocumentId, String)> = self
            .view
            .borrow()
            .iter()
            .filter_map(|(id, content)| match id {
                DocumentId::ChangeMeta { change } if change == name => Some((
                    id.clone(),
                    DocumentId::ArchivedChange {
                        change: dated_name.to_string(),
                        doc: ".openspec.yaml".to_string(),
                    },
                    content.clone(),
                )),
                DocumentId::ChangeArtifact { change, artifact } if change == name => Some((
                    id.clone(),
                    DocumentId::ArchivedChange {
                        change: dated_name.to_string(),
                        doc: artifact.clone(),
                    },
                    content.clone(),
                )),
                _ => None,
            })
            .collect();
        for (src, dst, content) in moves {
            self.put(dst, content);
            self.del(src);
        }
        Ok(())
    }

    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        self.read_doc(&DocumentId::ArchivedChange {
            change: dated_name.to_string(),
            doc: ".openspec.yaml".to_string(),
        })
    }

    fn write_archived_meta(&self, dated_name: &str, content: &str) -> anyhow::Result<()> {
        self.put(
            DocumentId::ArchivedChange {
                change: dated_name.to_string(),
                doc: ".openspec.yaml".to_string(),
            },
            content.to_string(),
        );
        Ok(())
    }

    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        self.read_doc(&DocumentId::ArchivedChange {
            change: dated_name.to_string(),
            doc: artifact.to_string(),
        })
    }

    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        let mut caps: Vec<String> = self
            .view
            .borrow()
            .keys()
            .filter_map(|id| match id {
                DocumentId::ArchivedChange { change, doc } if change == dated_name => {
                    spec_capability(doc)
                }
                _ => None,
            })
            .collect();
        caps.sort();
        caps
    }

    // --- discussions ---

    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.view
            .borrow()
            .contains_key(&DocumentId::Discussion { slug: slug.to_string(), archived: false })
    }

    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.view
            .borrow()
            .contains_key(&DocumentId::Discussion { slug: slug.to_string(), archived: true })
    }

    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        PathBuf::from(format!("discussions/{slug}.md"))
    }

    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        self.read_doc(&DocumentId::Discussion { slug: slug.to_string(), archived: false })
    }

    fn write_live_discussion(&self, slug: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.put(
            DocumentId::Discussion { slug: slug.to_string(), archived: false },
            content.to_string(),
        );
        Ok(PathBuf::from(format!("discussions/{slug}.md")))
    }

    fn delete_live_discussion(&self, slug: &str) -> anyhow::Result<()> {
        self.del(DocumentId::Discussion { slug: slug.to_string(), archived: false });
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
        let text = self.read_doc(&DocumentId::Discussion { slug: slug.to_string(), archived: true })?;
        Some(DiscussionDoc {
            slug: slug.to_string(),
            text,
            path: PathBuf::from(format!("discussions/archive/{slug}.md")),
            archived: true,
        })
    }

    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        self.view
            .borrow()
            .iter()
            .filter_map(|(id, content)| match id {
                DocumentId::Discussion { slug, archived: false } => Some(DiscussionDoc {
                    slug: slug.clone(),
                    text: content.clone(),
                    path: PathBuf::from(format!("discussions/{slug}.md")),
                    archived: false,
                }),
                _ => None,
            })
            .collect()
    }

    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        self.view
            .borrow()
            .iter()
            .filter_map(|(id, content)| match id {
                DocumentId::Discussion { slug, archived: true } => Some(DiscussionDoc {
                    slug: slug.clone(),
                    text: content.clone(),
                    path: PathBuf::from(format!("discussions/archive/{slug}.md")),
                    archived: true,
                }),
                _ => None,
            })
            .collect()
    }

    fn archive_discussion(&self, slug: &str, created: &str) -> anyhow::Result<Option<String>> {
        let Some(text) = self.read_live_discussion(slug) else {
            return Ok(None);
        };
        self.put(DocumentId::Discussion { slug: slug.to_string(), archived: true }, text);
        self.del(DocumentId::Discussion { slug: slug.to_string(), archived: false });
        Ok(Some(format!("{created}-{slug}.md")))
    }

    // --- workflow config / shared vocabulary ---

    fn read_workflow_config(&self) -> Option<String> {
        self.read_doc(&DocumentId::WorkflowConfig)
    }

    fn read_language(&self) -> Option<String> {
        // The scope's shared-vocabulary document; a missing LANGUAGE document is
        // a normal state, not an error.
        self.read_doc(&DocumentId::Language)
    }
}

#[cfg(test)]
mod tests {
    use super::BridgeStore;
    use crate::binding::local_default_binding;
    use speclink_core::store::Store;
    use speclink_store::memory::MemoryStore;
    use speclink_store::{CommandContext, DocumentId, Scope, TeamStore};

    #[test]
    fn archived_changes_match_point_reads_and_are_sorted_descending() {
        let store = MemoryStore::new();
        let binding = local_default_binding();
        let scope = Scope::new(binding.project, binding.repo);
        let mut uow = store
            .begin_unit_of_work(
                &scope,
                CommandContext {
                    command: "seed".to_string(),
                    actor: "test".to_string(),
                },
            )
            .expect("begin seed unit of work");
        for change in ["older", "newer"] {
            uow.create(
                DocumentId::ChangeMeta {
                    change: change.to_string(),
                },
                "schema: spec-driven\n",
            );
        }
        store.commit(uow, Vec::new()).expect("commit seed data");

        let bridge = BridgeStore::materialize(&store, &scope).expect("materialize bridge");
        bridge
            .archive_change("older", "2026-06-01-older")
            .expect("archive older change");
        bridge
            .archive_change("newer", "2026-07-20-newer")
            .expect("archive newer change");

        let archived = bridge.list_archived_changes();
        assert_eq!(archived, vec!["2026-07-20-newer", "2026-06-01-older"]);
        assert!(
            archived
                .iter()
                .all(|dated_name| bridge.archived_change_exists(dated_name)),
            "archive enumeration and point existence checks must agree"
        );
    }
}

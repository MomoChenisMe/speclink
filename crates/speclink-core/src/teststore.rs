//! In-memory [`Store`] test double, shared by the engine's in-module tests.
//!
//! Live surfaces mirror what the flows under test actually touch (change
//! metadata, artifacts, canonical specs, archive move-and-stamp); everything a
//! flow must never reach stays `unreachable!` so a test fails loudly instead
//! of silently widening the storage surface.

use crate::model::{Change, ChangeMeta};
use crate::store::{DiscussionDoc, Store};
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub(crate) struct TestStore {
    /// Active change name → raw `.openspec.yaml` text.
    pub metas: RefCell<HashMap<String, String>>,
    /// (change, artifact rel path) → content.
    pub artifacts: RefCell<HashMap<(String, String), String>>,
    /// Archived dated name → raw `.openspec.yaml` text.
    pub archived_metas: RefCell<HashMap<String, String>>,
    /// Capability → canonical spec content.
    pub canonical: RefCell<HashMap<String, String>>,
    /// Number of `write_change_meta` calls (idempotence assertions).
    pub meta_writes: RefCell<u32>,
    /// Number of `write_artifact` calls (no-write assertions).
    pub artifact_writes: RefCell<u32>,
    /// Live discussion slug → document text.
    pub discussions: RefCell<HashMap<String, String>>,
    /// Archived discussion slug → document text (promote must refuse these).
    pub archived_discussions: RefCell<HashMap<String, String>>,
}

impl TestStore {
    pub fn with_meta(name: &str, meta: &str) -> TestStore {
        let store = TestStore::default();
        store.metas.borrow_mut().insert(name.to_string(), meta.to_string());
        store
    }

    pub fn meta(&self, name: &str) -> String {
        self.metas.borrow().get(name).cloned().unwrap_or_default()
    }

    pub fn put_artifact(&self, change: &str, artifact: &str, content: &str) {
        self.artifacts
            .borrow_mut()
            .insert((change.to_string(), artifact.to_string()), content.to_string());
    }

    pub fn with_live_discussion(slug: &str, text: &str) -> TestStore {
        let store = TestStore::default();
        store.discussions.borrow_mut().insert(slug.to_string(), text.to_string());
        store
    }

    pub fn discussion(&self, slug: &str) -> String {
        self.discussions.borrow().get(slug).cloned().unwrap_or_default()
    }
}

impl Store for TestStore {
    fn list_changes(&self) -> Vec<Change> {
        let mut out: Vec<Change> = self
            .metas
            .borrow()
            .iter()
            .map(|(name, meta)| Change {
                name: name.clone(),
                dir: PathBuf::from(format!("changes/{name}")),
                meta: ChangeMeta::from_text(Some(meta)),
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }
    fn find_change(&self, name: &str) -> Option<Change> {
        let metas = self.metas.borrow();
        let meta = metas.get(name)?;
        Some(Change {
            name: name.to_string(),
            dir: PathBuf::from(format!("changes/{name}")),
            meta: ChangeMeta::from_text(Some(meta)),
        })
    }
    fn change_exists(&self, name: &str) -> bool {
        self.metas.borrow().contains_key(name)
    }
    fn create_change(&self, name: &str, meta_text: &str) -> Result<PathBuf> {
        self.metas.borrow_mut().insert(name.to_string(), meta_text.to_string());
        Ok(PathBuf::from(format!("changes/{name}")))
    }
    fn updated_at_secs(&self, _name: &str) -> u64 {
        0
    }
    fn read_change_meta(&self, name: &str) -> Option<String> {
        self.metas.borrow().get(name).cloned()
    }
    fn write_change_meta(&self, name: &str, content: &str) -> Result<()> {
        self.metas.borrow_mut().insert(name.to_string(), content.to_string());
        *self.meta_writes.borrow_mut() += 1;
        Ok(())
    }
    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        self.artifacts
            .borrow()
            .get(&(change.to_string(), artifact.to_string()))
            .cloned()
    }
    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> Result<PathBuf> {
        self.put_artifact(change, artifact, content);
        *self.artifact_writes.borrow_mut() += 1;
        Ok(PathBuf::from(format!("changes/{change}/{artifact}")))
    }
    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.artifacts
            .borrow()
            .contains_key(&(change.to_string(), artifact.to_string()))
    }
    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        let mut caps: Vec<String> = self
            .artifacts
            .borrow()
            .keys()
            .filter(|(c, _)| c == change)
            .filter_map(|(_, a)| {
                a.strip_prefix("specs/")
                    .and_then(|rest| rest.strip_suffix("/spec.md"))
                    .filter(|cap| !cap.contains('/'))
                    .map(str::to_string)
            })
            .collect();
        caps.sort();
        caps
    }
    fn has_capability_dirs(&self, change: &str) -> bool {
        !self.delta_capabilities(change).is_empty()
    }
    fn list_canonical_capabilities(&self) -> Vec<String> {
        self.canonical.borrow().keys().cloned().collect()
    }
    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.canonical.borrow().contains_key(cap)
    }
    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        self.canonical.borrow().get(cap).cloned()
    }
    fn write_canonical_spec(&self, cap: &str, content: &str) -> Result<()> {
        self.canonical.borrow_mut().insert(cap.to_string(), content.to_string());
        Ok(())
    }
    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        PathBuf::from(format!("specs/{cap}/spec.md"))
    }
    fn archived_change_exists(&self, dated_name: &str) -> bool {
        self.archived_metas.borrow().contains_key(dated_name)
    }
    fn archive_change(&self, name: &str, dated_name: &str) -> Result<()> {
        let meta = self
            .metas
            .borrow_mut()
            .remove(name)
            .ok_or_else(|| anyhow::anyhow!("no such change: {name}"))?;
        self.archived_metas.borrow_mut().insert(dated_name.to_string(), meta);
        Ok(())
    }
    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        self.archived_metas.borrow().get(dated_name).cloned()
    }
    fn write_archived_meta(&self, dated_name: &str, content: &str) -> Result<()> {
        self.archived_metas
            .borrow_mut()
            .insert(dated_name.to_string(), content.to_string());
        Ok(())
    }
    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.discussions.borrow().contains_key(slug)
    }
    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.archived_discussions.borrow().contains_key(slug)
    }
    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        PathBuf::from(format!("discussions/{slug}.md"))
    }
    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        self.discussions.borrow().get(slug).cloned()
    }
    fn write_live_discussion(&self, slug: &str, content: &str) -> Result<PathBuf> {
        self.discussions.borrow_mut().insert(slug.to_string(), content.to_string());
        Ok(self.live_discussion_path(slug))
    }
    fn delete_live_discussion(&self, _slug: &str) -> Result<()> {
        unreachable!()
    }
    fn read_discussion(&self, slug: &str) -> Option<DiscussionDoc> {
        if let Some(text) = self.discussions.borrow().get(slug) {
            return Some(DiscussionDoc {
                slug: slug.to_string(),
                text: text.clone(),
                path: self.live_discussion_path(slug),
                archived: false,
            });
        }
        let archived = self.archived_discussions.borrow();
        let text = archived.get(slug)?;
        Some(DiscussionDoc {
            slug: slug.to_string(),
            text: text.clone(),
            path: PathBuf::from(format!("discussions/archive/2026-01-02-{slug}.md")),
            archived: true,
        })
    }
    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        Vec::new()
    }
    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        Vec::new()
    }
    fn archive_discussion(&self, slug: &str, created: &str) -> Result<Option<String>> {
        // Faithful move: a live record is relocated into the archived map and its
        // dated filename returned; an absent slug yields None (matches the fs store).
        let Some(text) = self.discussions.borrow_mut().remove(slug) else {
            return Ok(None);
        };
        self.archived_discussions.borrow_mut().insert(slug.to_string(), text);
        Ok(Some(format!("{created}-{slug}.md")))
    }
    fn read_workflow_config(&self) -> Option<String> {
        None
    }
    fn read_language(&self) -> Option<String> {
        None
    }
}

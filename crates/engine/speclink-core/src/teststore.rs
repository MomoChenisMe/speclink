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
    /// (archived dated name, artifact rel path) → content — populated by
    /// `archive_change` (faithful directory move: every document travels).
    pub archived_artifacts: RefCell<HashMap<(String, String), String>>,
    /// Change name → evidence record text (the seam's storage side).
    pub evidence: RefCell<HashMap<String, String>>,
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
    /// When set, `delete_change` fails — lets discard tests exercise the
    /// "directory removal failed, unlinks not rolled back" path.
    pub fail_delete_change: RefCell<bool>,
    /// When set, `archive_discussion` fails — lets conclude tests exercise the
    /// "conclusion written, closing archive step failed" path.
    pub fail_archive_discussion: RefCell<bool>,
    /// When set to a capability name, `write_canonical_spec` fails for it —
    /// lets archive tests probe the commit-phase write order (every snapshot
    /// must already be on disk when the first canonical write happens).
    pub fail_canonical_write: RefCell<Option<String>>,
    /// Whether this double declares ownership adjudication (a team-mode
    /// backend). Default `false` keeps every existing test on the fs-parity
    /// path; `claim` tests flip it via [`TestStore::team_with_meta`].
    pub ownership: bool,
}

impl TestStore {
    pub fn with_meta(name: &str, meta: &str) -> TestStore {
        let store = TestStore::default();
        store.metas.borrow_mut().insert(name.to_string(), meta.to_string());
        store
    }

    /// A store that declares ownership adjudication — the `claim` verb's
    /// team-mode path.
    pub fn team_with_meta(name: &str, meta: &str) -> TestStore {
        TestStore {
            ownership: true,
            ..TestStore::with_meta(name, meta)
        }
    }

    pub fn meta(&self, name: &str) -> String {
        self.metas.borrow().get(name).cloned().unwrap_or_default()
    }

    pub fn put_evidence(&self, change: &str, content: &str) {
        self.evidence.borrow_mut().insert(change.to_string(), content.to_string());
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

/// Build a Change the way FsStore does: corrupt metadata yields default `meta`
/// plus the parse reason in `meta_error` (the double must stay faithful).
fn change_from_meta(name: &str, meta_text: &str) -> Change {
    let (meta, meta_error) = match ChangeMeta::from_text(Some(meta_text)) {
        Ok(meta) => (meta, None),
        Err(reason) => (ChangeMeta::default(), Some(reason)),
    };
    Change {
        name: name.to_string(),
        dir: PathBuf::from(format!("changes/{name}")),
        meta,
        meta_error,
    }
}

impl Store for TestStore {
    fn supports_ownership(&self) -> bool {
        self.ownership
    }
    fn list_changes(&self) -> Vec<Change> {
        let mut out: Vec<Change> = self
            .metas
            .borrow()
            .iter()
            .map(|(name, meta)| change_from_meta(name, meta))
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }
    fn find_change(&self, name: &str) -> Option<Change> {
        let metas = self.metas.borrow();
        let meta = metas.get(name)?;
        Some(change_from_meta(name, meta))
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
    fn delete_change(&self, name: &str) -> Result<()> {
        if *self.fail_delete_change.borrow() {
            anyhow::bail!("simulated delete failure");
        }
        self.metas.borrow_mut().remove(name);
        self.artifacts.borrow_mut().retain(|(c, _), _| c != name);
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
    fn delete_artifact(&self, change: &str, artifact: &str) -> Result<()> {
        // Faithful delete（review discard/stamp 的活面）：移除紀錄；缺席為
        // no-op——引擎在呼叫前守存在性，與 fs 端冪等刪除一致。
        self.artifacts
            .borrow_mut()
            .remove(&(change.to_string(), artifact.to_string()));
        Ok(())
    }
    fn read_evidence(&self, change: &str) -> Option<String> {
        self.evidence.borrow().get(change).cloned()
    }
    fn write_evidence(&self, change: &str, content: &str) -> Result<()> {
        self.evidence.borrow_mut().insert(change.to_string(), content.to_string());
        Ok(())
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
        if self.fail_canonical_write.borrow().as_deref() == Some(cap) {
            return Err(anyhow::anyhow!("injected canonical write failure for '{cap}'"));
        }
        self.canonical.borrow_mut().insert(cap.to_string(), content.to_string());
        Ok(())
    }
    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        PathBuf::from(format!("specs/{cap}/spec.md"))
    }
    fn list_archived_changes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.archived_metas.borrow().keys().cloned().collect();
        names.sort_by(|a, b| b.cmp(a));
        names
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
        // Faithful directory move（FsStore 整目錄搬移）：change 的所有文件隨行
        // ——review.md 化石工單即由此進入封存區。
        let mut artifacts = self.artifacts.borrow_mut();
        let moved: Vec<(String, String)> =
            artifacts.keys().filter(|(c, _)| c == name).cloned().collect();
        let mut archived = self.archived_artifacts.borrow_mut();
        for key in moved {
            let content = artifacts.remove(&key).expect("key just enumerated");
            archived.insert((dated_name.to_string(), key.1), content);
        }
        Ok(())
    }
    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        self.archived_artifacts
            .borrow()
            .get(&(dated_name.to_string(), artifact.to_string()))
            .cloned()
    }
    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        self.archived_metas.borrow().get(dated_name).cloned()
    }
    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        // 與 FsStore 同規則：恰為 specs/<cap>/spec.md 一層才算。
        let mut caps: Vec<String> = self
            .archived_artifacts
            .borrow()
            .keys()
            .filter(|(d, _)| d == dated_name)
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
    fn delete_live_discussion(&self, slug: &str) -> Result<()> {
        // Faithful delete (live surface for the command runtime's discuss
        // discard flow): remove the live record; an absent slug is a no-op,
        // matching the fs store's idempotent remove.
        self.discussions.borrow_mut().remove(slug);
        Ok(())
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
            path: archived_discussion_path(slug),
            archived: true,
        })
    }
    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        // Faithful listing: every live record in the map, mirroring
        // `read_discussion`'s live shape (order is the map's — callers sort).
        self.discussions
            .borrow()
            .iter()
            .map(|(slug, text)| DiscussionDoc {
                slug: slug.clone(),
                text: text.clone(),
                path: self.live_discussion_path(slug),
                archived: false,
            })
            .collect()
    }
    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        self.archived_discussions
            .borrow()
            .iter()
            .map(|(slug, text)| DiscussionDoc {
                slug: slug.clone(),
                text: text.clone(),
                path: archived_discussion_path(slug),
                archived: true,
            })
            .collect()
    }
    fn archive_discussion(&self, slug: &str, created: &str) -> Result<Option<String>> {
        if *self.fail_archive_discussion.borrow() {
            anyhow::bail!("simulated discussion archive failure");
        }
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

/// The double's archived location: one fixed date prefix, the same shape the
/// fs store produces (`<created>-<slug>.md`).
fn archived_discussion_path(slug: &str) -> PathBuf {
    PathBuf::from(format!("discussions/archive/2026-01-02-{slug}.md"))
}

#[cfg(test)]
mod tests {
    use super::TestStore;
    use crate::store::Store;

    #[test]
    fn archived_changes_are_visible_after_archive_in_dated_name_descending_order() {
        let store = TestStore::default();
        store
            .create_change("older", "schema: spec-driven\n")
            .expect("create older change");
        store
            .create_change("newer", "schema: spec-driven\n")
            .expect("create newer change");

        store
            .archive_change("older", "2026-06-01-older")
            .expect("archive older change");
        store
            .archive_change("newer", "2026-07-20-newer")
            .expect("archive newer change");

        assert_eq!(
            store.list_archived_changes(),
            vec!["2026-07-20-newer", "2026-06-01-older"]
        );
    }
}

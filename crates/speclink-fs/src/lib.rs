//! speclink-fs: the default filesystem implementation of the engine's storage
//! interface, preserving the classic `openspec/` layout byte-for-byte.
//!
//! All layout knowledge migrated here from the engine (path composition,
//! directory enumeration, mtime-derived ordering, archive move-and-name,
//! discussion file naming); the engine itself only speaks the domain verbs of
//! `speclink_core::store::Store`.

pub mod layout;

use anyhow::Result;
use layout::Layout;
use speclink_core::model::{Change, ChangeMeta};
use speclink_core::store::{DiscussionDoc, Store};
use speclink_core::util;
use std::path::{Path, PathBuf};

/// Filesystem-backed [`Store`] over a project's spec directory.
#[derive(Debug, Clone)]
pub struct FsStore {
    layout: Layout,
}

impl FsStore {
    /// Build a store rooted at the project directory with the given spec
    /// directory name (default "openspec").
    pub fn new(root: &Path, spec_dir_name: &str) -> FsStore {
        FsStore {
            layout: Layout::new(root, spec_dir_name),
        }
    }

    /// The underlying layout — for host-side callers (the CLI) that need to
    /// render storage locations this adapter owns.
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    fn change_at(&self, name: &str, dir: PathBuf) -> Change {
        let meta_text = util::read_opt(&dir.join(".openspec.yaml"));
        Change {
            name: name.to_string(),
            meta: ChangeMeta::from_text(meta_text.as_deref()),
            dir,
        }
    }
}

/// Strip a `YYYY-MM-DD-` prefix from an archived file stem.
fn strip_date_prefix(name: &str) -> Option<&str> {
    let b = name.as_bytes();
    if b.len() > 11
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[4] == b'-'
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[7] == b'-'
        && b[8..10].iter().all(|c| c.is_ascii_digit())
        && b[10] == b'-'
    {
        Some(&name[11..])
    } else {
        None
    }
}

/// Whether `s` is a `-N` same-day disambiguation suffix.
fn is_dup_suffix(s: &str) -> bool {
    s.strip_prefix('-')
        .map(|d| !d.is_empty() && d.bytes().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

impl FsStore {
    /// Locate an archived discussion by slug (`archive/<YYYY-MM-DD>-<slug>[-N].md`)
    /// — return the newest, ranked by (date prefix, `-N` suffix number; a plain
    /// name counts as 1).
    fn find_archived_discussion(&self, slug: &str) -> Option<PathBuf> {
        let mut candidates: Vec<(String, u64, PathBuf)> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.layout.discussions_archive_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let Some(stem) = name.strip_suffix(".md") else { continue };
                let Some(rest) = strip_date_prefix(stem) else { continue };
                let seq = if rest == slug {
                    Some(1)
                } else {
                    rest.strip_prefix(slug)
                        .filter(|s| is_dup_suffix(s))
                        .and_then(|s| s[1..].parse::<u64>().ok())
                };
                if let Some(seq) = seq {
                    candidates.push((stem[..10].to_string(), seq, path));
                }
            }
        }
        candidates.sort();
        candidates.pop().map(|(_, _, p)| p)
    }
}

impl Store for FsStore {
    // --- changes ---

    fn list_changes(&self) -> Vec<Change> {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(self.layout.changes_dir()) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "archive" {
                continue;
            }
            out.push(self.change_at(&name, path));
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    fn find_change(&self, name: &str) -> Option<Change> {
        let dir = self.layout.change_dir(name);
        if dir.is_dir() {
            Some(self.change_at(name, dir))
        } else {
            None
        }
    }

    fn change_exists(&self, name: &str) -> bool {
        self.layout.change_dir(name).exists()
    }

    fn create_change(&self, name: &str, meta_text: &str) -> Result<PathBuf> {
        let dir = self.layout.change_dir(name);
        util::write_file(&dir.join(".openspec.yaml"), meta_text)?;
        Ok(dir)
    }

    fn read_change_meta(&self, name: &str) -> Option<String> {
        let dir = self.layout.change_dir(name);
        if !dir.is_dir() {
            return None;
        }
        util::read_opt(&dir.join(".openspec.yaml"))
    }

    fn write_change_meta(&self, name: &str, content: &str) -> Result<()> {
        util::write_file(&self.layout.change_dir(name).join(".openspec.yaml"), content)?;
        Ok(())
    }

    fn updated_at_secs(&self, name: &str) -> u64 {
        // Newest file mtime inside the change (recursive), truncated to whole
        // seconds — the engine's "most recently modified" sort key.
        util::walk_files(&self.layout.change_dir(name))
            .into_iter()
            .filter_map(|p| std::fs::metadata(&p).and_then(|m| m.modified()).ok())
            .filter_map(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .max()
            .unwrap_or(0)
    }

    // --- artifacts ---

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        util::read_opt(&self.layout.artifact_path(change, artifact))
    }

    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> Result<PathBuf> {
        let path = self.layout.artifact_path(change, artifact);
        util::write_file(&path, content)?;
        Ok(path)
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.layout.artifact_path(change, artifact).is_file()
    }

    // --- delta specs ---

    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        // Exactly `specs/<capability>/spec.md`, one level deep — nested or
        // differently-named .md files under specs/ do not count.
        let specs = self.layout.change_dir(change).join("specs");
        let mut caps = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&specs) {
            for entry in entries.flatten() {
                if entry.path().is_dir() && entry.path().join("spec.md").is_file() {
                    caps.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        caps.sort();
        caps
    }

    fn has_capability_dirs(&self, change: &str) -> bool {
        std::fs::read_dir(self.layout.change_dir(change).join("specs"))
            .map(|it| it.flatten().any(|e| e.path().is_dir()))
            .unwrap_or(false)
    }

    // --- canonical specs ---

    fn list_canonical_capabilities(&self) -> Vec<String> {
        let mut specs = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.layout.specs_dir()) {
            for e in entries.flatten() {
                if e.path().is_dir() && e.path().join("spec.md").is_file() {
                    specs.push(e.file_name().to_string_lossy().to_string());
                }
            }
        }
        specs
    }

    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.layout.canonical_spec(cap).is_file()
    }

    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        util::read_opt(&self.layout.canonical_spec(cap))
    }

    fn write_canonical_spec(&self, cap: &str, content: &str) -> Result<()> {
        util::write_file(&self.layout.canonical_spec(cap), content)?;
        Ok(())
    }

    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        self.layout.canonical_spec(cap)
    }

    // --- archive ---

    fn archived_change_exists(&self, dated_name: &str) -> bool {
        self.layout.archived_change_dir(dated_name).exists()
    }

    fn archive_change(&self, name: &str, dated_name: &str) -> Result<()> {
        std::fs::create_dir_all(self.layout.archive_dir())?;
        std::fs::rename(
            self.layout.change_dir(name),
            self.layout.archived_change_dir(dated_name),
        )?;
        Ok(())
    }

    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        util::read_opt(&self.layout.archived_change_dir(dated_name).join(".openspec.yaml"))
    }

    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        let path = self
            .layout
            .archived_change_dir(dated_name)
            .join(artifact.split('/').collect::<std::path::PathBuf>());
        util::read_opt(&path)
    }

    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        // 與 active 的 delta_capabilities 同規則：恰為 specs/<cap>/spec.md 一層。
        let specs = self.layout.archived_change_dir(dated_name).join("specs");
        let mut caps = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&specs) {
            for entry in entries.flatten() {
                if entry.path().is_dir() && entry.path().join("spec.md").is_file() {
                    caps.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        caps.sort();
        caps
    }

    fn write_archived_meta(&self, dated_name: &str, content: &str) -> Result<()> {
        util::write_file(
            &self.layout.archived_change_dir(dated_name).join(".openspec.yaml"),
            content,
        )?;
        Ok(())
    }

    // --- discussions ---

    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.layout.live_discussion(slug).is_file()
    }

    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.find_archived_discussion(slug).is_some()
    }

    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        self.layout.live_discussion(slug)
    }

    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        util::read_opt(&self.layout.live_discussion(slug))
    }

    fn write_live_discussion(&self, slug: &str, content: &str) -> Result<PathBuf> {
        let path = self.layout.live_discussion(slug);
        util::write_file(&path, content)?;
        Ok(path)
    }

    fn delete_live_discussion(&self, slug: &str) -> Result<()> {
        std::fs::remove_file(self.layout.live_discussion(slug))?;
        Ok(())
    }

    fn read_discussion(&self, slug: &str) -> Option<DiscussionDoc> {
        let live = self.layout.live_discussion(slug);
        if live.is_file() {
            return Some(DiscussionDoc {
                slug: slug.to_string(),
                text: util::read_opt(&live)?,
                path: live,
                archived: false,
            });
        }
        let path = self.find_archived_discussion(slug)?;
        Some(DiscussionDoc {
            slug: slug.to_string(),
            text: util::read_opt(&path)?,
            path,
            archived: true,
        })
    }

    fn list_live_discussions(&self) -> Vec<DiscussionDoc> {
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.layout.discussions_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let Some(slug) = name.strip_suffix(".md") else { continue };
                let Some(text) = util::read_opt(&path) else { continue };
                out.push(DiscussionDoc {
                    slug: slug.to_string(),
                    text,
                    path,
                    archived: false,
                });
            }
        }
        out
    }

    fn list_archived_discussions(&self) -> Vec<DiscussionDoc> {
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(self.layout.discussions_archive_dir()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let Some(stem) = name.strip_suffix(".md") else { continue };
                let slug = strip_date_prefix(stem).unwrap_or(stem).to_string();
                let Some(text) = util::read_opt(&path) else { continue };
                out.push(DiscussionDoc {
                    slug,
                    text,
                    path,
                    archived: true,
                });
            }
        }
        // Stored-name order = archive-date order.
        out.sort_by(|a, b| a.path.cmp(&b.path));
        out
    }

    fn archive_discussion(&self, slug: &str, created: &str) -> Result<Option<String>> {
        let src = self.layout.live_discussion(slug);
        if !src.is_file() {
            return Ok(None);
        }
        let dir = self.layout.discussions_archive_dir();
        std::fs::create_dir_all(&dir)?;
        // Same-day name collisions get a `-N` suffix so co-archival never
        // fails on a reused slug.
        let base = format!("{created}-{slug}");
        let mut name = format!("{base}.md");
        let mut n = 2;
        while dir.join(&name).exists() {
            name = format!("{base}-{n}.md");
            n += 1;
        }
        std::fs::rename(&src, dir.join(&name))?;
        Ok(Some(name))
    }

    // --- workflow config ---

    fn read_workflow_config(&self) -> Option<String> {
        util::read_opt(&self.layout.workflow_config())
    }

    // --- shared vocabulary ---

    fn read_language(&self) -> Option<String> {
        util::read_opt(&self.layout.language_doc())
    }
}

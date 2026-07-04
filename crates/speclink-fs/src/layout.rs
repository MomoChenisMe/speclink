//! The `openspec/` directory layout — the single place that knows where spec
//! documents physically live: `specs/<capability>/spec.md` (canonical),
//! `changes/<name>/` (active changes), `changes/archive/<date>-<name>/`
//! (archived changes), `discussions/<slug>.md` (+ `discussions/archive/`),
//! and `config.yaml` (workflow configuration).

use std::path::{Path, PathBuf};

/// Path composition for a project's spec directory.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Project root (the directory that contains the spec dir).
    pub root: PathBuf,
    /// Spec directory name relative to root (default "openspec").
    pub spec_dir_name: String,
}

impl Layout {
    pub fn new(root: &Path, spec_dir_name: &str) -> Layout {
        Layout {
            root: root.to_path_buf(),
            spec_dir_name: spec_dir_name.to_string(),
        }
    }

    pub fn spec_dir(&self) -> PathBuf {
        self.root.join(&self.spec_dir_name)
    }
    pub fn workflow_config(&self) -> PathBuf {
        self.spec_dir().join("config.yaml")
    }
    pub fn language_doc(&self) -> PathBuf {
        self.spec_dir().join("LANGUAGE.md")
    }
    pub fn specs_dir(&self) -> PathBuf {
        self.spec_dir().join("specs")
    }
    pub fn canonical_spec(&self, cap: &str) -> PathBuf {
        self.specs_dir().join(cap).join("spec.md")
    }
    pub fn changes_dir(&self) -> PathBuf {
        self.spec_dir().join("changes")
    }
    pub fn change_dir(&self, name: &str) -> PathBuf {
        self.changes_dir().join(name)
    }
    pub fn archive_dir(&self) -> PathBuf {
        self.changes_dir().join("archive")
    }
    pub fn archived_change_dir(&self, dated_name: &str) -> PathBuf {
        self.archive_dir().join(dated_name)
    }
    pub fn discussions_dir(&self) -> PathBuf {
        self.spec_dir().join("discussions")
    }
    pub fn live_discussion(&self, slug: &str) -> PathBuf {
        self.discussions_dir().join(format!("{slug}.md"))
    }
    pub fn discussions_archive_dir(&self) -> PathBuf {
        self.discussions_dir().join("archive")
    }
    /// An artifact's location inside a change. `artifact` is the schema output
    /// path (e.g. `specs/<cap>/spec.md`); joined component-by-component so the
    /// native separator is used throughout.
    pub fn artifact_path(&self, change: &str, artifact: &str) -> PathBuf {
        artifact
            .split('/')
            .fold(self.change_dir(change), |p, c| p.join(c))
    }
}

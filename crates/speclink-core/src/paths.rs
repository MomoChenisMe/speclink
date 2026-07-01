//! Project path resolution.

use crate::config::AppConfig;
use std::path::{Path, PathBuf};

/// Resolved project paths.
#[derive(Debug, Clone)]
pub struct Paths {
    /// Project root (contains .speclink.yaml and/or the spec dir).
    pub root: PathBuf,
    /// Spec directory name relative to root (default "openspec").
    pub spec_dir_name: String,
}

impl Paths {
    /// Discover the project root by walking up from `start`.
    ///
    /// A directory qualifies as root if it contains `.speclink.yaml` or the `openspec` directory.
    pub fn discover(start: &Path) -> Option<Paths> {
        let mut cur = Some(start);
        while let Some(dir) = cur {
            let app_cfg = dir.join(".speclink.yaml");
            if app_cfg.is_file() {
                let spec_dir_name = AppConfig::load(&app_cfg)
                    .spec_dir
                    .unwrap_or_else(|| "openspec".to_string());
                return Some(Paths {
                    root: dir.to_path_buf(),
                    spec_dir_name,
                });
            }
            if dir.join("openspec").is_dir() {
                return Some(Paths {
                    root: dir.to_path_buf(),
                    spec_dir_name: "openspec".to_string(),
                });
            }
            cur = dir.parent();
        }
        None
    }

    /// Discover from the current working directory.
    pub fn discover_cwd() -> Option<Paths> {
        let cwd = std::env::current_dir().ok()?;
        Paths::discover(&cwd)
    }

    pub fn app_config(&self) -> PathBuf {
        self.root.join(".speclink.yaml")
    }
    pub fn spec_dir(&self) -> PathBuf {
        self.root.join(&self.spec_dir_name)
    }
    pub fn workflow_config(&self) -> PathBuf {
        self.spec_dir().join("config.yaml")
    }
    pub fn specs_dir(&self) -> PathBuf {
        self.spec_dir().join("specs")
    }
    pub fn changes_dir(&self) -> PathBuf {
        self.spec_dir().join("changes")
    }
    pub fn archive_dir(&self) -> PathBuf {
        self.changes_dir().join("archive")
    }
    pub fn discussions_dir(&self) -> PathBuf {
        self.spec_dir().join("discussions")
    }
    pub fn language_file(&self) -> PathBuf {
        self.spec_dir().join("LANGUAGE.md")
    }
    /// Work data directory (gitignored).
    pub fn work_dir(&self) -> PathBuf {
        self.root.join(".speclink")
    }
    pub fn touched_dir(&self) -> PathBuf {
        self.work_dir().join("touched")
    }
    pub fn snapshots_dir(&self) -> PathBuf {
        self.work_dir().join("snapshots")
    }
    pub fn in_progress_file(&self) -> PathBuf {
        self.work_dir().join("in_progress.json")
    }
    pub fn change_dir(&self, name: &str) -> PathBuf {
        self.changes_dir().join(name)
    }
}

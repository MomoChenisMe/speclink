//! Host-side workspace paths and project discovery.
//!
//! These are the engine host's own locations — the `.speclink/` work data
//! (touched records, archive snapshots), the `.speclink.yaml` app config, and
//! the project-root walk-up — NOT spec documents. Spec-document storage is
//! behind `crate::store::Store`; a remote storage backend would still keep all
//! of these local to the host.

use crate::config::AppConfig;
use std::path::{Path, PathBuf};

/// Resolved host workspace.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Project root (contains .speclink.yaml and/or the spec dir).
    pub root: PathBuf,
    /// Spec directory name relative to root (default "openspec") — recorded so
    /// the host can construct the storage adapter and locate host-managed
    /// files that live beside the spec dir (workflow schema definitions).
    pub spec_dir_name: String,
}

impl Workspace {
    /// Discover the project root by walking up from `start`.
    ///
    /// A directory qualifies as root if it contains `.speclink.yaml` or the `openspec` directory.
    pub fn discover(start: &Path) -> Option<Workspace> {
        let mut cur = Some(start);
        while let Some(dir) = cur {
            let app_cfg = dir.join(".speclink.yaml");
            if app_cfg.is_file() {
                let spec_dir_name = AppConfig::load(&app_cfg)
                    .spec_dir
                    .unwrap_or_else(|| "openspec".to_string());
                return Some(Workspace {
                    root: dir.to_path_buf(),
                    spec_dir_name,
                });
            }
            if dir.join("openspec").is_dir() {
                return Some(Workspace {
                    root: dir.to_path_buf(),
                    spec_dir_name: "openspec".to_string(),
                });
            }
            cur = dir.parent();
        }
        None
    }

    /// Discover from the current working directory.
    pub fn discover_cwd() -> Option<Workspace> {
        let cwd = std::env::current_dir().ok()?;
        Workspace::discover(&cwd)
    }

    pub fn app_config(&self) -> PathBuf {
        self.root.join(".speclink.yaml")
    }
    /// The spec directory location — used only for host-managed content that
    /// sits beside the spec documents (`schemas/` definitions); document
    /// access itself goes through the Store.
    pub fn spec_dir(&self) -> PathBuf {
        self.root.join(&self.spec_dir_name)
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
}

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
            // A remote workspace has no openspec/ tree (and may have no
            // .speclink.yaml) — the connection file alone marks the root.
            if dir.join("openspec").is_dir() || dir.join(REMOTE_FILE).is_file() {
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

    /// The remote connection file (`.speclink.remote.yaml`) location.
    pub fn remote_config(&self) -> PathBuf {
        self.root.join(REMOTE_FILE)
    }

    /// Resolve fs-vs-remote mode: the connection file's presence is the mode
    /// signal. `env_store_url` (SPECLINK_STORE_URL) overrides the connection
    /// url only — it never flips an fs workspace into remote mode.
    pub fn resolve_mode_with(
        &self,
        env_store_url: Option<String>,
    ) -> anyhow::Result<ModeResolution> {
        let remote_file = self.remote_config();
        if !remote_file.is_file() {
            return Ok(ModeResolution {
                mode: StoreMode::Fs,
                coexists: false,
            });
        }
        let text = crate::util::read_opt(&remote_file)
            .ok_or_else(|| anyhow::anyhow!("cannot read {REMOTE_FILE}"))?;
        let mut conn = RemoteConnection::from_text(&text)?;
        // The env var overrides the url only — an empty value counts as unset,
        // and it never turns an fs workspace into a remote one.
        if let Some(url) = env_store_url.filter(|u| !u.trim().is_empty()) {
            conn.url = url;
        }
        Ok(ModeResolution {
            coexists: self.spec_dir().is_dir(),
            mode: StoreMode::Remote(conn),
        })
    }

    /// [`Workspace::resolve_mode_with`] against the process environment.
    pub fn resolve_mode(&self) -> anyhow::Result<ModeResolution> {
        self.resolve_mode_with(std::env::var("SPECLINK_STORE_URL").ok())
    }
}

/// File name of the remote connection file — its presence IS the mode signal.
pub const REMOTE_FILE: &str = ".speclink.remote.yaml";

/// Which storage the CLI talks to for this workspace.
#[derive(Debug, Clone)]
pub enum StoreMode {
    /// Local `openspec/` layout via the fs adapter (the default).
    Fs,
    /// Remote verb-contract server described by the connection file.
    Remote(RemoteConnection),
}

/// Parsed `.speclink.remote.yaml` — `url` is required (project-scoped),
/// `repo` is this repo's registered name in the project (optional on
/// single-repo projects).
#[derive(Debug, Clone)]
pub struct RemoteConnection {
    pub url: String,
    pub repo: Option<String>,
}

impl RemoteConnection {
    /// Parse the connection file text. A missing or empty `url` is a
    /// semantic error naming the file and the field — never a silent default.
    pub fn from_text(text: &str) -> anyhow::Result<RemoteConnection> {
        #[derive(serde::Deserialize)]
        struct Raw {
            url: Option<String>,
            repo: Option<String>,
        }
        let raw: Raw = serde_yaml::from_str(text).map_err(|e| {
            anyhow::anyhow!("invalid {REMOTE_FILE}: {e}")
        })?;
        let url = raw
            .url
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("invalid {REMOTE_FILE}: missing required `url` field")
            })?;
        Ok(RemoteConnection {
            url: url.trim().to_string(),
            repo: raw.repo.filter(|r| !r.trim().is_empty()),
        })
    }
}

/// Outcome of mode resolution: the mode plus whether the connection file and
/// a local spec directory coexist (remote wins; the CLI prints one warning).
#[derive(Debug)]
pub struct ModeResolution {
    pub mode: StoreMode,
    /// True when `.speclink.remote.yaml` and the local spec dir both exist.
    pub coexists: bool,
}

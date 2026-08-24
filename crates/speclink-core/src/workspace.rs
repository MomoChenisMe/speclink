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
    /// A directory qualifies as root if it contains `.speclink.yaml` or the `openspec`
    /// directory. `Ok(None)` = not in a project; `Err` = the root was found but its
    /// `.speclink.yaml` cannot be parsed (fail-closed: spec_dir must not silently
    /// default off a broken file).
    pub fn discover(start: &Path) -> Result<Option<Workspace>, crate::config::ConfigError> {
        let mut cur = Some(start);
        while let Some(dir) = cur {
            let app_cfg = dir.join(".speclink.yaml");
            if app_cfg.is_file() {
                let spec_dir_name = AppConfig::load(&app_cfg)?
                    .spec_dir
                    .unwrap_or_else(|| "openspec".to_string());
                return Ok(Some(Workspace {
                    root: dir.to_path_buf(),
                    spec_dir_name,
                }));
            }
            // A leftover .speclink.remote.yaml still marks the root so an
            // unmigrated project reaches the migration warning instead of
            // failing discovery with "not in a project".
            if dir.join("openspec").is_dir() || dir.join(REMOTE_FILE).is_file() {
                return Ok(Some(Workspace {
                    root: dir.to_path_buf(),
                    spec_dir_name: "openspec".to_string(),
                }));
            }
            cur = dir.parent();
        }
        Ok(None)
    }

    /// Discover from the current working directory.
    pub fn discover_cwd() -> Result<Option<Workspace>, crate::config::ConfigError> {
        match std::env::current_dir() {
            Ok(cwd) => Workspace::discover(&cwd),
            Err(_) => Ok(None),
        }
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
    /// The pre-move evidence location, gitignored. Read-only compatibility on
    /// the storage side (`speclink-fs` reads it as a fallback); the host keeps
    /// this path only to sweep the file when its change archives or discards.
    pub fn legacy_touched_file(&self, change: &str) -> PathBuf {
        self.touched_dir().join(format!("{change}.json"))
    }
    pub fn touched_dir(&self) -> PathBuf {
        self.work_dir().join("touched")
    }
    pub fn snapshots_dir(&self) -> PathBuf {
        self.work_dir().join("snapshots")
    }
    /// Host-local review scope sidecars (Apply baselines + frozen review
    /// snapshots) — never touched records, never store documents.
    pub fn review_scopes_dir(&self) -> PathBuf {
        self.work_dir().join("review-scopes")
    }

    /// The legacy remote connection file (`.speclink.remote.yaml`) location —
    /// only consulted for leftover detection, never parsed.
    pub fn remote_config(&self) -> PathBuf {
        self.root.join(REMOTE_FILE)
    }

    /// True when a leftover legacy connection file sits in the project root
    /// (the structured fact behind the CLI's migration warning).
    pub fn has_leftover_remote_file(&self) -> bool {
        self.remote_config().is_file()
    }

    /// Resolve fs-vs-remote mode: the `remote:` section of `.speclink.yaml` is
    /// the mode signal. `env_store_url` (SPECLINK_STORE_URL) overrides or
    /// supplies the section url only — it never flips an fs workspace into
    /// remote mode. A leftover `.speclink.remote.yaml` is flagged for the
    /// migration warning but never parsed and never mode-relevant.
    pub fn resolve_mode_with(
        &self,
        env_store_url: Option<String>,
    ) -> anyhow::Result<ModeResolution> {
        let leftover_remote_file = self.has_leftover_remote_file();
        let remote = AppConfig::load(&self.app_config())?.remote;
        let Some(section) = remote else {
            return Ok(ModeResolution {
                mode: StoreMode::Fs,
                coexists: false,
                leftover_remote_file,
            });
        };
        // The env var overrides (or supplies) the url only — an empty value
        // counts as unset. Both missing is an explicit failure naming both
        // settings: silently falling back to fs mode would fabricate truth.
        let url = env_store_url
            .filter(|u| !u.trim().is_empty())
            .or_else(|| section.url.clone().filter(|u| !u.trim().is_empty()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "remote store url is not set: add `remote.url` to .speclink.yaml or set the SPECLINK_STORE_URL environment variable"
                )
            })?;
        Ok(ModeResolution {
            coexists: self.spec_dir().is_dir(),
            mode: StoreMode::Remote(RemoteConnection {
                url: url.trim().to_string(),
                repo: section.repo.filter(|r| !r.trim().is_empty()),
            }),
            leftover_remote_file,
        })
    }

}

/// File name of the LEGACY remote connection file. No longer parsed: its
/// presence only triggers the one-line migration warning.
pub const REMOTE_FILE: &str = ".speclink.remote.yaml";

/// Which storage the CLI talks to for this workspace.
#[derive(Debug, Clone)]
pub enum StoreMode {
    /// Local `openspec/` layout via the fs adapter (the default).
    Fs,
    /// Remote verb-contract server described by the connection file.
    Remote(RemoteConnection),
}

/// Resolved remote connection — `url` is the effective store url after the
/// env-var override (project-scoped), `repo` is this repo's registered name
/// in the project (optional on single-repo projects).
#[derive(Debug, Clone)]
pub struct RemoteConnection {
    pub url: String,
    pub repo: Option<String>,
}

/// Outcome of mode resolution: the mode plus the facts the CLI turns into
/// warnings (structured results only — no presentation strings in core).
#[derive(Debug)]
pub struct ModeResolution {
    pub mode: StoreMode,
    /// True when the `remote:` section and the local spec dir both exist
    /// (remote wins; the CLI prints one warning).
    pub coexists: bool,
    /// True when a leftover legacy `.speclink.remote.yaml` sits in the project
    /// root (never parsed; the CLI prints one migration warning).
    pub leftover_remote_file: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Throwaway workspace root with the given `.speclink.yaml` content.
    /// `label` keeps parallel tests in distinct sandboxes.
    fn workspace_with(label: &str, app_yaml: &str) -> Workspace {
        let dir = std::env::temp_dir().join(format!(
            "speclink-ws-mode-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp workspace");
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).expect("write app config");
        Workspace {
            root: dir,
            spec_dir_name: "openspec".to_string(),
        }
    }

    // --- store 模式解析以注入值運作（env 層由 host 邊界注入） ---

    #[test]
    fn injected_store_url_supplies_the_remote_url() {
        let ws = workspace_with("inject-url", "remote:\n  repo: web\n");
        let resolution = ws
            .resolve_mode_with(Some("http://injected.example".to_string()))
            .expect("mode resolves");
        match resolution.mode {
            StoreMode::Remote(conn) => {
                assert_eq!(conn.url, "http://injected.example");
                assert_eq!(conn.repo.as_deref(), Some("web"));
            }
            StoreMode::Fs => panic!("remote section + injected url must resolve remote"),
        }
        std::fs::remove_dir_all(&ws.root).ok();
    }

    #[test]
    fn missing_injected_url_with_bare_remote_section_fails_loudly() {
        let ws = workspace_with("bare-remote", "remote:\n");
        let err = ws.resolve_mode_with(None).expect_err("both url sources missing");
        let msg = format!("{err}");
        assert!(msg.contains("remote.url"), "error names the config key: {msg}");
        assert!(msg.contains("SPECLINK_STORE_URL"), "error names the env var: {msg}");
        std::fs::remove_dir_all(&ws.root).ok();
    }

    #[test]
    fn injected_url_never_flips_an_fs_workspace_into_remote() {
        let ws = workspace_with("fs-only", "locale: tw\n");
        let resolution = ws
            .resolve_mode_with(Some("http://injected.example".to_string()))
            .expect("mode resolves");
        assert!(
            matches!(resolution.mode, StoreMode::Fs),
            "no remote section: the injected url must not create remote mode"
        );
        std::fs::remove_dir_all(&ws.root).ok();
    }
}

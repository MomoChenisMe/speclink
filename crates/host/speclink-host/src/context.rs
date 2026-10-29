//! SpeclinkExecutionContext — the Host-resolved execution identity.
//!
//! Resolved exactly once at the entry point:
//! who acts (actor), where (project/repo binding), how storage is reached
//! (mode), and under which effective workflow policy. The Engine only ever
//! consumes this context; command inputs carry no actor or policy fields,
//! so neither callers nor models can override identity.

use crate::policy::EffectiveWorkflowPolicy;
use speclink_core::workspace::{ModeResolution, Workspace};
use speclink_store::{ProjectId, RepoId};

/// Where an actor identity came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorSource {
    /// Resolved from git config at the Host boundary (local mode).
    GitConfig,
    /// Supplied explicitly by the embedding host (server mode, Phase 2).
    Explicit,
}

/// Who performs this execution. Anonymous keeps the current local behavior:
/// no git or no user.name means stamping flows stamp nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    Identified { display: String, source: ActorSource },
    Anonymous,
}

impl Actor {
    /// The display identity string ("Name <email>"), None when anonymous.
    pub fn display(&self) -> Option<&str> {
        match self {
            Actor::Identified { display, .. } => Some(display),
            Actor::Anonymous => None,
        }
    }
}

/// How the execution reaches storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Local `openspec/` layout via the fs adapter.
    Fs,
    /// A shared TeamStore backend.
    SharedStore,
}

/// The Host-resolved execution context: resolved once at the entry point,
/// consumed by everything downstream. Command inputs carry no actor or
/// policy fields, so this context is the only identity source.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeclinkExecutionContext {
    pub actor: Actor,
    pub project: ProjectId,
    pub repo: RepoId,
    pub mode: ExecutionMode,
    pub policy: EffectiveWorkflowPolicy,
}

/// [`Workspace::resolve_mode_with`] against the process environment — the
/// SPECLINK_STORE_URL read lives at the Host boundary, never in the Engine.
pub fn resolve_store_mode(ws: &Workspace) -> anyhow::Result<ModeResolution> {
    ws.resolve_mode_with(std::env::var("SPECLINK_STORE_URL").ok())
}

/// Machine-level speclink directory (global config, user schemas), by OS
/// convention: Windows `%USERPROFILE%\AppData\Roaming` (derived from the
/// profile — a redirected APPDATA env var is deliberately ignored), macOS
/// `~/Library/Application Support`, Linux `$XDG_CONFIG_HOME`|`~/.config`.
/// A Host-boundary lookup: the Engine receives the resolved directory and
/// never reads these variables itself.
/// Windows portable packages instead use `<exe directory>/data/speclink`.
pub fn global_config_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    if let Some(data) = crate::portable::data_dir() {
        return data.join("speclink");
    }
    let base = if cfg!(windows) {
        std::env::var("USERPROFILE")
            .map(|h| PathBuf::from(h).join("AppData").join("Roaming"))
            .ok()
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .ok()
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .ok()
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")).ok())
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("speclink")
}

/// Shared CLI/Desktop data location. Security operations never fall back to a project.
pub fn security_data_dir() -> anyhow::Result<std::path::PathBuf> {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let xdg = std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from);
    let local = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from);
    security_data_dir_for(
        std::env::consts::OS,
        home.as_deref(),
        xdg.as_deref(),
        local.as_deref(),
        crate::portable::data_dir(),
    )
}

fn security_data_dir_for(
    os: &str,
    home: Option<&std::path::Path>,
    xdg: Option<&std::path::Path>,
    local: Option<&std::path::Path>,
    portable: Option<&std::path::Path>,
) -> anyhow::Result<std::path::PathBuf> {
    let base = if let Some(portable) = portable {
        portable.to_path_buf()
    } else {
        match os {
            "macos" => home.map(|h| h.join("Library/Application Support")),
            "windows" => local.map(std::path::Path::to_path_buf),
            "linux" => xdg
                .map(std::path::Path::to_path_buf)
                .or_else(|| home.map(|h| h.join(".local/share"))),
            _ => None,
        }
        .ok_or_else(|| {
            anyhow::anyhow!("data_directory_unavailable: no application data directory")
        })?
    };
    anyhow::ensure!(
        base.is_absolute(),
        "data_directory_unavailable: expected an absolute application data directory"
    );
    Ok(base.join("speclink"))
}

/// Build the "Name <email>" identity string from git config, if available.
/// Identity resolution is a Host responsibility — moved verbatim from the
/// engine's util: full identity when name and email are both set, the name
/// alone when only user.name is set, None (anonymous) otherwise.
pub fn git_identity(root: &std::path::Path) -> Option<String> {
    let name = speclink_core::util::git(root, &["config", "user.name"]);
    let email = speclink_core::util::git(root, &["config", "user.email"]);
    match (name, email) {
        (Some(n), Some(e)) if !n.is_empty() && !e.is_empty() => Some(format!("{n} <{e}>")),
        (Some(n), _) if !n.is_empty() => Some(n),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_path_preferences_are_device_only_atomic_and_fail_closed() {
        let config=tempfile::tempdir().unwrap();
        let device=tempfile::tempdir().unwrap();
        let shared=tempfile::tempdir().unwrap();
        let snapshot=crate::security_paths::read_at(config.path(),device.path()).unwrap();
        assert_eq!(std::path::Path::new(&snapshot.effective_path).canonicalize().unwrap(),device.path().join("security/standards").canonicalize().unwrap());
        let json=serde_json::to_value(snapshot).unwrap();
        assert!(json["standardsPath"].is_null() && json["effectivePath"].is_string() && json["isDefault"].is_boolean() && json["access"].is_string());
        crate::security_paths::write_at(config.path(),device.path(),Some(shared.path().to_string_lossy().into())).unwrap();
        assert_eq!(std::path::Path::new(&crate::security_paths::read_at(config.path(),device.path()).unwrap().effective_path).canonicalize().unwrap(),shared.path().canonicalize().unwrap());
        assert!(crate::security_paths::write_at(config.path(),device.path(),Some("relative".into())).is_err());
        assert_eq!(std::path::Path::new(&crate::security_paths::read_at(config.path(),device.path()).unwrap().effective_path).canonicalize().unwrap(),shared.path().canonicalize().unwrap());
        std::fs::write(config.path().join("security-data-preferences.json"),r#"{"formatVersion":2,"standardsPath":null}"#).unwrap();
        assert!(crate::security_paths::read_at(config.path(),device.path()).is_err());
        for raw in [r#"{"formatVersion":1,"standardsPath":true}"#,r#"{"formatVersion":1,"standardsPath":"https://example.com"}"#] {
            std::fs::write(config.path().join("security-data-preferences.json"),raw).unwrap();
            assert!(crate::security_paths::read_at(config.path(),device.path()).is_err());
        }
    }
    #[test]
    fn standard_path_change_is_busy_while_a_reader_holds_its_snapshot() {
        let config=tempfile::tempdir().unwrap();let device=tempfile::tempdir().unwrap();let shared=tempfile::tempdir().unwrap();
        let lease=crate::security_paths::lease_at(config.path(),device.path()).unwrap();
        assert!(crate::security_paths::write_at(config.path(),device.path(),Some(shared.path().to_string_lossy().into())).unwrap_err().to_string().contains("busy"));
        drop(lease);
        crate::security_paths::write_at(config.path(),device.path(),None).unwrap();
        assert!(crate::security_paths::read_at(config.path(),device.path()).unwrap().is_default);
    }
    #[test]
    #[cfg(unix)]
    fn standard_path_lease_release_survives_inherited_handle() {
        let config = tempfile::tempdir().unwrap();
        let device = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let lease = crate::security_paths::lease_at(config.path(), device.path()).unwrap();
        let inherited = lease.lock.try_clone().unwrap();
        assert!(crate::security_paths::write_at(
            config.path(),
            device.path(),
            Some(shared.path().to_string_lossy().into())
        )
        .is_err());
        drop(lease);
        crate::security_paths::write_at(config.path(), device.path(), None).unwrap();
        assert!(crate::security_paths::read_at(config.path(), device.path()).unwrap().is_default);
        drop(inherited);
    }
    #[cfg(windows)]
    #[test]
    fn standard_path_accepts_complete_unc_and_rejects_partial_or_device_paths() {
        for raw in [r"\\server\team\standards",r"D:\standards"] { assert!(crate::security_paths::validate_path(raw).is_ok()); }
        for raw in [r"\\server",r"\standards",r"C:relative",r"\\?\C:\data",r"\\.\pipe\test", "~", "", "a\0b"] { assert!(crate::security_paths::validate_path(raw).is_err(),"{raw}"); }
    }
    use crate::binding::local_default_binding;
    use crate::policy::EffectiveWorkflowPolicy;
    use speclink_core::config::ResolvedPolicy;

    #[test]
    #[cfg(not(windows))]
    fn security_data_directory_is_shared_absolute_and_never_project_fallback() {
        use std::path::Path;
        let home = Path::new("/home/example");
        let xdg = Path::new("/data/example");
        let portable = Path::new("/portable/data");
        assert_eq!(
            security_data_dir_for("macos", Some(home), None, None, None).unwrap(),
            home.join("Library/Application Support/speclink")
        );
        assert_eq!(
            security_data_dir_for("linux", Some(home), None, None, None).unwrap(),
            home.join(".local/share/speclink")
        );
        assert_eq!(
            security_data_dir_for("linux", Some(home), Some(xdg), None, None).unwrap(),
            xdg.join("speclink")
        );
        assert_eq!(
            security_data_dir_for("windows", None, None, Some(xdg), None).unwrap(),
            xdg.join("speclink")
        );
        assert_eq!(
            security_data_dir_for("windows", None, None, None, Some(portable)).unwrap(),
            portable.join("speclink")
        );
        for (os, h, x, l, p) in [
            ("macos", None, None, None, None),
            ("windows", None, None, None, None),
            ("linux", None, None, None, None),
            ("linux", Some(home), Some(Path::new("relative")), None, None),
            ("windows", None, None, None, Some(Path::new("relative"))),
        ] {
            assert!(security_data_dir_for(os, h, x, l, p)
                .unwrap_err()
                .to_string()
                .contains("data_directory_unavailable"));
        }
    }

    #[cfg(windows)]
    #[test]
    fn security_data_directory_windows_native_paths() {
        use std::path::Path;
        let local = Path::new(r"C:\Users\Example\AppData\Local");
        assert_eq!(
            security_data_dir_for("windows", None, None, Some(local), None).unwrap(),
            local.join("speclink")
        );
        let portable = Path::new(r"D:\SpecLink\data");
        assert_eq!(
            security_data_dir_for("windows", None, None, None, Some(portable)).unwrap(),
            portable.join("speclink")
        );
        assert!(
            security_data_dir_for("windows", None, None, Some(Path::new("relative")), None)
                .is_err()
        );
        assert!(security_data_dir_for("windows", None, None, None, None).is_err());
    }

    fn sample_policy() -> EffectiveWorkflowPolicy {
        EffectiveWorkflowPolicy::new(
            ResolvedPolicy {
                locale: "English".to_string(),
                spec_locale: None,
                tdd: true,
                audit: false,
                worktree: false,
            },
            "tdd: true\n",
        )
    }

    // --- ExecutionContext 型別組成：actor／project／repo／mode／resolved policy ---

    #[test]
    fn execution_context_carries_actor_binding_mode_and_policy() {
        let binding = local_default_binding();
        let ctx = SpeclinkExecutionContext {
            actor: Actor::Identified {
                display: "Alice <alice@example.com>".to_string(),
                source: ActorSource::GitConfig,
            },
            project: binding.project,
            repo: binding.repo,
            mode: ExecutionMode::Fs,
            policy: sample_policy(),
        };
        assert_eq!(ctx.project.as_str(), "default");
        assert_eq!(ctx.repo.as_str(), "main");
        assert!(matches!(ctx.mode, ExecutionMode::Fs));
        assert_eq!(
            ctx.actor.display(),
            Some("Alice <alice@example.com>"),
            "actor exposes the display identity string"
        );
        assert!(ctx.policy.resolved().tdd, "policy wraps the core ResolvedPolicy");
    }

    #[test]
    fn policy_carries_a_content_digest() {
        // EffectiveWorkflowPolicy 帶政策文件 digest（policyRevision 前身；
        // 不進任何現有輸出）。
        let policy = sample_policy();
        assert!(
            policy.digest().starts_with("sha256:"),
            "digest is the contract-defined sha256 form, got {}",
            policy.digest()
        );
        let other = EffectiveWorkflowPolicy::new(policy.resolved().clone(), "tdd: false\n");
        assert_ne!(policy.digest(), other.digest(), "digest follows document content");
    }

    #[test]
    fn anonymous_actor_has_no_display_identity() {
        // 無 git 或未設 user.name：匿名、蓋章流程不蓋章（行為同現行）。
        let actor = Actor::Anonymous;
        assert_eq!(actor.display(), None);
    }

    #[test]
    fn execution_mode_is_a_closed_set() {
        for mode in [ExecutionMode::Fs, ExecutionMode::SharedStore] {
            match mode {
                ExecutionMode::Fs => {}
                ExecutionMode::SharedStore => {}
            }
        }
    }
}

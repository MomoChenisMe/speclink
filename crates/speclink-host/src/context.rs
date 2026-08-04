//! SpeclinkExecutionContext — the Host-resolved execution identity.
//!
//! Resolved exactly once at the entry point (platform architecture §4.6):
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
pub fn global_config_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
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
    use crate::binding::local_default_binding;
    use crate::policy::EffectiveWorkflowPolicy;
    use speclink_core::config::ResolvedPolicy;

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

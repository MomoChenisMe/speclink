//! EffectiveWorkflowPolicy — the Host-resolved workflow policy.
//!
//! Wraps the Engine's four-layer ResolvedPolicy together with a digest of
//! the policy document content (the local stand-in for policyRevision; it
//! enters no existing output). The env layer of policy resolution happens
//! at the Host boundary — the Engine only ever receives injected lookups.

use sha2::{Digest, Sha256};
use speclink_core::config::{
    resolve_policy, AppConfig, ConfigError, EnvOverrides, ResolvedPolicy, WorkflowConfig,
};

/// The effective workflow policy an execution runs under: the resolved
/// values plus the digest of the policy document they were resolved from.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveWorkflowPolicy {
    resolved: ResolvedPolicy,
    digest: String,
}

impl EffectiveWorkflowPolicy {
    /// Wrap a resolved policy with the digest of `policy_document` (the
    /// config.yaml content the resolution read; empty when absent).
    pub fn new(resolved: ResolvedPolicy, policy_document: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(policy_document.as_bytes());
        Self {
            resolved,
            digest: format!("sha256:{:x}", hasher.finalize()),
        }
    }

    pub fn resolved(&self) -> &ResolvedPolicy {
        &self.resolved
    }

    /// The policy-document content digest — the policyRevision precursor.
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

/// Resolve the effective workflow policy through an injected env lookup —
/// the Engine's four-layer resolution with the env layer supplied by the
/// Host. A workflow document that exists but cannot parse fails closed.
pub fn resolve_effective_policy(
    env_lookup: impl Fn(&str) -> Option<String>,
    app: &AppConfig,
    workflow_document: Option<&str>,
) -> Result<EffectiveWorkflowPolicy, ConfigError> {
    let wf = WorkflowConfig::from_text(workflow_document)?;
    let env = EnvOverrides::from_lookup(env_lookup);
    let resolved = resolve_policy(&env, app, &wf);
    Ok(EffectiveWorkflowPolicy::new(
        resolved,
        workflow_document.unwrap_or(""),
    ))
}

/// The Host boundary's process-env read: the SPECLINK_* override layer as
/// one injected value set. The only place the policy env layer touches the
/// process environment.
pub fn process_env_overrides() -> EnvOverrides {
    EnvOverrides::from_lookup(|key| std::env::var(key).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::config::AppConfig;

    // --- Engine 規格面不讀 process env：政策 env 層由 host 注入 ---

    #[test]
    fn injected_lookup_decides_policy_not_process_env() {
        // process env 設相反值：解析結果只反映注入集合，process env 無效果。
        std::env::set_var("SPECLINK_TDD", "false");
        let injected = |key: &str| (key == "SPECLINK_TDD").then(|| "true".to_string());
        let policy =
            resolve_effective_policy(injected, &AppConfig::default(), Some("audit: true\n"))
                .expect("workflow document parses");
        std::env::remove_var("SPECLINK_TDD");
        assert!(
            policy.resolved().tdd,
            "injected SPECLINK_TDD=true wins over the opposite process-env value"
        );
        assert!(policy.resolved().audit, "config-document layers still apply");
    }

    #[test]
    fn absent_injected_keys_fall_to_document_layers() {
        std::env::set_var("SPECLINK_TDD", "true");
        let policy = resolve_effective_policy(|_| None, &AppConfig::default(), Some("tdd: false\n"))
            .expect("workflow document parses");
        std::env::remove_var("SPECLINK_TDD");
        assert!(
            !policy.resolved().tdd,
            "with no injected override the document decides; process env stays invisible"
        );
    }

    #[test]
    fn effective_policy_digests_the_workflow_document() {
        let a = resolve_effective_policy(|_| None, &AppConfig::default(), Some("tdd: true\n"))
            .expect("parses");
        let b = resolve_effective_policy(|_| None, &AppConfig::default(), Some("tdd: false\n"))
            .expect("parses");
        assert!(a.digest().starts_with("sha256:"));
        assert_ne!(a.digest(), b.digest(), "digest follows the policy document content");
    }

    #[test]
    fn broken_workflow_document_fails_closed() {
        // 政策解析沿用既有 fail-closed：壞文件是錯誤，不是預設值。
        let err = resolve_effective_policy(|_| None, &AppConfig::default(), Some("rules: ["));
        assert!(err.is_err(), "a broken policy document must not resolve to defaults");
    }
}

//! EffectiveWorkflowPolicy — the Host-resolved workflow policy.
//!
//! Wraps the Engine's three-layer ResolvedPolicy together with a digest of
//! the policy document content (the local stand-in for policyRevision; it
//! enters no existing output). The env layer of policy resolution happens
//! at the Host boundary — the Engine only ever receives injected lookups.

use sha2::{Digest, Sha256};
use speclink_core::config::{
    resolve_policy, ConfigError, EnvOverrides, ResolvedPolicy, WorkflowConfig,
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
/// the Engine's three-layer resolution with the env layer supplied by the
/// Host. A workflow document that exists but cannot parse fails closed.
pub fn resolve_effective_policy(
    env_lookup: impl Fn(&str) -> Option<String>,
    workflow_document: Option<&str>,
) -> Result<EffectiveWorkflowPolicy, ConfigError> {
    let wf = WorkflowConfig::from_text(workflow_document)?;
    let env = EnvOverrides {
        system_locale: sys_locale::get_locale(),
        ..EnvOverrides::from_lookup(env_lookup)
    };
    let resolved = resolve_policy(&env, &wf);
    Ok(EffectiveWorkflowPolicy::new(
        resolved,
        workflow_document.unwrap_or(""),
    ))
}

/// Read process overrides and the OS language at the Host boundary, then
/// inject both into the Engine without persisting the detected language.
pub fn process_env_overrides() -> EnvOverrides {
    let mut env = EnvOverrides::from_lookup(|key| std::env::var(key).ok());
    env.system_locale = sys_locale::get_locale();
    env
}

/// The canonical language choice for baseline, with OS defaults but without
/// SPECLINK_* overrides. The server and local CLI expose the same read view.
pub fn workflow_languages(
    workflow_document: Option<&str>,
) -> Result<speclink_protocol::query::WorkflowLanguages, ConfigError> {
    let wf = WorkflowConfig::from_text(workflow_document)?;
    speclink_core::config::validate_policy_locales(&speclink_core::config::WorkflowPolicyFields {
        locale: wf.locale.clone(), spec_locale: wf.spec_locale.clone(), ..Default::default()
    }).map_err(|error| ConfigError { file: "config.yaml".into(), reason: error.to_string() })?;
    let policy = resolve_policy(&EnvOverrides {
        system_locale: sys_locale::get_locale(), ..Default::default()
    }, &wf);
    Ok(speclink_protocol::query::WorkflowLanguages {
        locale: policy.locale,
        spec_locale: policy.spec_locale.unwrap_or_else(|| "en".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_languages_rejects_unknown_canonical_language_values() {
        for yaml in ["spec_locale: zh-Hant\n", "locale: zh-Hant\nspec_locale: auto\n", "locale: JA\nspec_locale: ja\n"] {
            let error = workflow_languages(Some(yaml)).expect_err("unknown language is not a concrete supported code");
            assert!(error.reason.contains("must be one of"));
            assert!(WorkflowConfig::from_text(Some(yaml)).is_ok(), "canonical reads remain lenient");
        }
    }

    // --- Engine 規格面不讀 process env：政策 env 層由 host 注入 ---

    #[test]
    fn injected_lookup_decides_policy_not_process_env() {
        // process env 設相反值：解析結果只反映注入集合，process env 無效果。
        std::env::set_var("SPECLINK_TDD", "false");
        let injected = |key: &str| (key == "SPECLINK_TDD").then(|| "true".to_string());
        let policy =
            resolve_effective_policy(injected, Some("audit: true\n"))
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
        let policy = resolve_effective_policy(|_| None, Some("tdd: false\n"))
            .expect("workflow document parses");
        std::env::remove_var("SPECLINK_TDD");
        assert!(
            !policy.resolved().tdd,
            "with no injected override the document decides; process env stays invisible"
        );
    }

    #[test]
    fn effective_policy_digests_the_workflow_document() {
        let a = resolve_effective_policy(|_| None, Some("tdd: true\n"))
            .expect("parses");
        let b = resolve_effective_policy(|_| None, Some("tdd: false\n"))
            .expect("parses");
        assert!(a.digest().starts_with("sha256:"));
        assert_ne!(a.digest(), b.digest(), "digest follows the policy document content");
    }

    #[test]
    fn broken_workflow_document_fails_closed() {
        // 政策解析沿用既有 fail-closed：壞文件是錯誤，不是預設值。
        let err = resolve_effective_policy(|_| None, Some("rules: ["));
        assert!(err.is_err(), "a broken policy document must not resolve to defaults");
    }
}

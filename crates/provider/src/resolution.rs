//! Provider resolution：五層優先序 + MVP fallback 行為。
//!
//! `resolve()` 是純函式：所有 I/O（讀檔、讀環境變數）由 caller 完成並透過 [`ResolutionInputs`]
//! 注入。MVP 因為沒有遠端 provider 實作，凡是被解析為「非 local」的 provider 都會依
//! `FallbackPolicy` 降級為 [`ResolvedProvider::Local`] 並附帶 `provider.not_authenticated` warning，
//! 或在 `fallback=disabled` 時回傳 [`ResolutionError::AuthRequiredNoFallback`]。

use crate::config::{FallbackPolicy, GlobalConfig, ProjectConfig};
use crate::error::ResolutionError;

/// 解析後的 provider。MVP 僅有 `Local` 變體；HTTP 等遠端實作在後續 change 引入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedProvider {
    /// Local filesystem provider。
    Local {
        /// 選擇 local 的原因。
        reason: LocalReason,
    },
}

/// 選擇 local provider 的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalReason {
    /// 任何來源都沒有設定 provider。
    NoConfig,
    /// 設定明確指定 `provider = "local"`。
    Explicit,
    /// 選擇了非 local 的 provider 名稱，但 MVP 無遠端實作 → 因「未認證」降級。
    FallbackFromUnauthenticated {
        /// 被略過的原 provider 名稱（用於 warning 訊息）。
        bypassed_provider: String,
    },
}

/// 純函式輸入；caller 將檔案 / 環境變數讀好再注入。
#[derive(Debug, Clone, Default)]
pub struct ResolutionInputs<'a> {
    /// `--provider` 旗標（最高優先序）。
    pub flag_provider: Option<&'a str>,
    /// 已載入的專案 config。
    pub project_config: Option<&'a ProjectConfig>,
    /// 已載入的全域 config。
    pub global_config: Option<&'a GlobalConfig>,
    /// `SPECLINK_PROVIDER` 環境變數值（最低優先序）。
    pub env_provider: Option<&'a str>,
}

/// 非致命的 warning，會放入 JSON envelope 的 `warnings` 陣列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// 點分隔 error/warning code。
    pub code: String,
    /// 給人讀的訊息。
    pub message: String,
}

/// `resolve` 的輸出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionOutput {
    /// 最終選用的 provider。
    pub provider: ResolvedProvider,
    /// 過程中累積的 warning。
    pub warnings: Vec<Warning>,
}

/// 解析五層優先序並回傳實際 provider 與 warning 集合。
pub fn resolve(inputs: ResolutionInputs<'_>) -> Result<ResolutionOutput, ResolutionError> {
    // 1) flag → 2) project config provider → 3) global active profile → 4) env var
    let selected: Option<String> = inputs
        .flag_provider
        .map(str::to_string)
        .or_else(|| {
            inputs
                .project_config
                .and_then(|c| c.provider.clone())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            inputs
                .global_config
                .and_then(|c| c.active_provider())
                .map(str::to_string)
        })
        .or_else(|| {
            inputs
                .env_provider
                .map(str::to_string)
                .filter(|s| !s.is_empty())
        });

    // 5) 全空 → local fallback (NoConfig)
    let name = match selected {
        None => {
            return Ok(ResolutionOutput {
                provider: ResolvedProvider::Local {
                    reason: LocalReason::NoConfig,
                },
                warnings: Vec::new(),
            });
        }
        Some(n) => n,
    };

    if name == "local" {
        return Ok(ResolutionOutput {
            provider: ResolvedProvider::Local {
                reason: LocalReason::Explicit,
            },
            warnings: Vec::new(),
        });
    }

    let fallback = inputs
        .project_config
        .map(|c| c.fallback)
        .unwrap_or(FallbackPolicy::Local);

    match fallback {
        FallbackPolicy::Disabled => Err(ResolutionError::AuthRequiredNoFallback {
            provider_name: name,
        }),
        FallbackPolicy::Local => {
            let warning = Warning {
                code: "provider.not_authenticated".to_string(),
                message: format!(
                    "Provider '{name}' is configured but not authenticated. Using local provider fallback."
                ),
            };
            Ok(ResolutionOutput {
                provider: ResolvedProvider::Local {
                    reason: LocalReason::FallbackFromUnauthenticated {
                        bypassed_provider: name,
                    },
                },
                warnings: vec![warning],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{FallbackPolicy, GlobalConfig, ProfileEntry, ProjectConfig};
    use crate::error::ResolutionError;
    use crate::resolution::{LocalReason, ResolutionInputs, ResolvedProvider, resolve};
    use std::collections::BTreeMap;

    fn project_config_with(provider: Option<&str>, fallback: FallbackPolicy) -> ProjectConfig {
        ProjectConfig {
            provider: provider.map(str::to_string),
            fallback,
        }
    }

    fn global_config_with_active(provider: &str) -> GlobalConfig {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "default".to_string(),
            ProfileEntry {
                provider: provider.to_string(),
            },
        );
        GlobalConfig {
            active_profile: Some("default".to_string()),
            profiles,
        }
    }

    #[test]
    fn flag_wins() {
        let project = project_config_with(Some("billing"), FallbackPolicy::Local);
        let global = global_config_with_active("other");
        let inputs = ResolutionInputs {
            flag_provider: Some("acme"),
            project_config: Some(&project),
            global_config: Some(&global),
            env_provider: None,
        };
        let out = resolve(inputs).expect("resolve");
        match out.provider {
            ResolvedProvider::Local {
                reason: LocalReason::FallbackFromUnauthenticated { bypassed_provider },
            } => {
                assert_eq!(bypassed_provider, "acme");
            }
            other => panic!("expected fallback-from-unauthenticated 'acme', got {other:?}"),
        }
        assert_eq!(out.warnings.len(), 1);
        assert_eq!(out.warnings[0].code, "provider.not_authenticated");
    }

    #[test]
    fn project_wins_over_global_and_env() {
        let project = project_config_with(Some("billing"), FallbackPolicy::Local);
        let global = global_config_with_active("acme");
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: Some(&project),
            global_config: Some(&global),
            env_provider: Some("ignored"),
        };
        let out = resolve(inputs).expect("resolve");
        match out.provider {
            ResolvedProvider::Local {
                reason: LocalReason::FallbackFromUnauthenticated { bypassed_provider },
            } => {
                assert_eq!(bypassed_provider, "billing");
            }
            other => panic!("expected billing, got {other:?}"),
        }
    }

    #[test]
    fn global_wins_when_project_unset() {
        let global = global_config_with_active("acme");
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: None,
            global_config: Some(&global),
            env_provider: Some("ignored"),
        };
        let out = resolve(inputs).expect("resolve");
        match out.provider {
            ResolvedProvider::Local {
                reason: LocalReason::FallbackFromUnauthenticated { bypassed_provider },
            } => {
                assert_eq!(bypassed_provider, "acme");
            }
            other => panic!("expected acme, got {other:?}"),
        }
    }

    #[test]
    fn env_wins_when_others_unset() {
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: None,
            global_config: None,
            env_provider: Some("acme"),
        };
        let out = resolve(inputs).expect("resolve");
        match out.provider {
            ResolvedProvider::Local {
                reason: LocalReason::FallbackFromUnauthenticated { bypassed_provider },
            } => {
                assert_eq!(bypassed_provider, "acme");
            }
            other => panic!("expected acme, got {other:?}"),
        }
    }

    #[test]
    fn all_empty_returns_local_no_config() {
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: None,
            global_config: None,
            env_provider: None,
        };
        let out = resolve(inputs).expect("resolve");
        assert!(
            matches!(
                out.provider,
                ResolvedProvider::Local {
                    reason: LocalReason::NoConfig
                }
            ),
            "expected Local{{NoConfig}}, got {:?}",
            out.provider
        );
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn fallback_disabled_with_unauthenticated_remote_errors() {
        let project = project_config_with(Some("acme"), FallbackPolicy::Disabled);
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: Some(&project),
            global_config: None,
            env_provider: None,
        };
        let err = resolve(inputs).expect_err("must error");
        match err {
            ResolutionError::AuthRequiredNoFallback { provider_name } => {
                assert_eq!(provider_name, "acme");
            }
            other => panic!("expected AuthRequiredNoFallback, got {other:?}"),
        }
    }

    #[test]
    fn fallback_local_warning_message_mentions_provider_name() {
        let project = project_config_with(Some("acme"), FallbackPolicy::Local);
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: Some(&project),
            global_config: None,
            env_provider: None,
        };
        let out = resolve(inputs).expect("resolve");
        assert_eq!(out.warnings.len(), 1);
        assert_eq!(out.warnings[0].code, "provider.not_authenticated");
        assert!(
            out.warnings[0].message.contains("acme"),
            "warning message must mention provider name: {}",
            out.warnings[0].message
        );
    }

    #[test]
    fn explicit_local_in_project_config() {
        let project = project_config_with(Some("local"), FallbackPolicy::Local);
        let inputs = ResolutionInputs {
            flag_provider: None,
            project_config: Some(&project),
            global_config: None,
            env_provider: None,
        };
        let out = resolve(inputs).expect("resolve");
        assert!(
            matches!(
                out.provider,
                ResolvedProvider::Local {
                    reason: LocalReason::Explicit
                }
            ),
            "expected Local{{Explicit}}, got {:?}",
            out.provider
        );
        assert!(out.warnings.is_empty());
    }
}

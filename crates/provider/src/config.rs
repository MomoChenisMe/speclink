//! Provider 設定載入：`ProjectConfig`、`GlobalConfig`、`FallbackPolicy`。
//!
//! - `ProjectConfig`：當前專案的 `.speclink/config.toml`
//! - `GlobalConfig`：使用者層級的 `<config_home>/speclink/config.toml`
//! - `FallbackPolicy`：未認證或不可達時是否降級至 local provider

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// 設定載入錯誤型別。對應 CLI 的 `input.invalid` exit code 2。
#[derive(Debug, Error)]
pub enum ConfigError {
    /// 檔案 I/O 失敗。
    #[error("failed to read config file '{path}': {source}")]
    Io {
        /// 嘗試讀取的路徑。
        path: PathBuf,
        /// I/O 來源錯誤。
        #[source]
        source: std::io::Error,
    },
    /// TOML parse 失敗。
    #[error("malformed config in '{path}': {message}")]
    Parse {
        /// 嘗試解析的路徑。
        path: PathBuf,
        /// 來自 toml crate 的描述。
        message: String,
    },
}

/// Fallback 策略；當設定的 remote provider 未認證或不可用時是否降級至 local。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FallbackPolicy {
    /// 降級至 local provider（預設）。
    #[default]
    Local,
    /// 拒絕降級；改為以 exit code 6 失敗。
    Disabled,
}

/// 專案層級設定，對應 `.speclink/config.toml`。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 設定的 provider 名稱。`None` 表示未指定。
    pub provider: Option<String>,
    /// Fallback 策略；序列化時為 `"local"` / `"disabled"`。
    #[serde(default)]
    pub fallback: FallbackPolicy,
}

impl ProjectConfig {
    /// 從 TOML 字串解析。
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// 從檔案讀取並解析。
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml_str(&content).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }
}

/// 全域設定，對應 `<config_home>/speclink/config.toml`。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// 啟用的 profile 名稱。`None` 表示未啟用任何 profile。
    pub active_profile: Option<String>,
    /// 所有可用 profile，鍵為 profile 名稱。
    #[serde(default)]
    pub profiles: std::collections::BTreeMap<String, ProfileEntry>,
}

impl GlobalConfig {
    /// 從 TOML 字串解析。
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// 從檔案讀取並解析。
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml_str(&content).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    /// 回傳當前啟用 profile 的 provider 名稱。
    pub fn active_provider(&self) -> Option<&str> {
        let active = self.active_profile.as_deref()?;
        self.profiles.get(active).map(|e| e.provider.as_str())
    }
}

/// 全域設定中的單一 profile 項。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileEntry {
    /// 此 profile 指向的 provider 名稱。
    pub provider: String,
}

#[cfg(test)]
mod parse_tests {
    use crate::config::{FallbackPolicy, ProjectConfig};

    #[test]
    fn parses_provider_and_fallback_local() {
        let toml = r#"
            provider = "acme"
            fallback = "local"
        "#;
        let cfg: ProjectConfig = ProjectConfig::from_toml_str(toml).expect("parse");
        assert_eq!(cfg.provider.as_deref(), Some("acme"));
        assert_eq!(cfg.fallback, FallbackPolicy::Local);
    }

    #[test]
    fn parses_provider_and_fallback_disabled() {
        let toml = r#"
            provider = "acme"
            fallback = "disabled"
        "#;
        let cfg: ProjectConfig = ProjectConfig::from_toml_str(toml).expect("parse");
        assert_eq!(cfg.provider.as_deref(), Some("acme"));
        assert_eq!(cfg.fallback, FallbackPolicy::Disabled);
    }

    #[test]
    fn missing_fallback_defaults_to_local() {
        let toml = r#"
            provider = "acme"
        "#;
        let cfg: ProjectConfig = ProjectConfig::from_toml_str(toml).expect("parse");
        assert_eq!(cfg.fallback, FallbackPolicy::Local);
    }

    #[test]
    fn invalid_fallback_value_rejected() {
        let toml = r#"
            provider = "acme"
            fallback = "remote"
        "#;
        let err = ProjectConfig::from_toml_str(toml).expect_err("invalid fallback");
        // 任何 parse 失敗：上層 CLI 會映射到 input.invalid
        let msg = err.to_string();
        assert!(
            msg.contains("fallback") || msg.contains("remote"),
            "expected error mentioning fallback or remote; got: {msg}"
        );
    }
}

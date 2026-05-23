//! `ConfigStore` trait + 配套型別。
//!
//! 對應 `config-rw` capability requirement「`ConfigStore` trait SHALL be exposed
//! via `Provider::config_store()`」以及 design contract §「介面 shape」對
//! `WriteConfigRequest` / `Config` / `Rules` 的型別定義。
//!
//! `LocalProvider` 為唯一具體實作（位於 `speclink-provider-local`），HttpProvider
//! 實作預留給未來 slice、不阻擋 A5 完成（spec requirement 註明）。

use serde::{Deserialize, Serialize};

use crate::error::ProviderError;
use crate::jsonpath::JsonPath;
use crate::types::{Actor, Etag, Versioned};

/// `.speclink/config.yaml` 的 `rules.*` 子樹。
///
/// A5 只承載兩個 review flag；其餘 rule key 由後續 slice 增補。Default 對齊
/// walking-skeleton 行為（兩個 flag 皆 `false`、走 `derive(Default)` 取得）。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Rules {
    /// `true` 時，artifact DAG 齊全後狀態進 `reviewing` 而非 `ready`。
    #[serde(default)]
    pub require_artifact_review: bool,
    /// `true` 時，最後一個 task done 後狀態進 `code_reviewing` 而非可 archive。
    #[serde(default)]
    pub require_code_review: bool,
}

/// `.speclink/config.yaml` 解析後的結構。
///
/// `roles` 為 opaque map（A5 不解析）；roles slice 接通後會替換為強型別。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub rules: Rules,
    /// Roles 子樹；A5 不解析、僅原樣保留以便 write path round-trip。
    #[serde(default = "default_roles")]
    pub roles: serde_yaml::Value,
}

fn default_roles() -> serde_yaml::Value {
    serde_yaml::Value::Null
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rules: Rules::default(),
            roles: default_roles(),
        }
    }
}

/// `config.write` 的 value 表達；對應 spec scenario「Value parsing precedence」
/// 的 5 種輸出型別（`true/false/null → int → float → string`）。
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Null,
    Int(i64),
    Float(f64),
    String(String),
}

impl ConfigValue {
    /// 依 spec scenario「Value parsing precedence」順序解析 CLI 字串輸入：
    /// `true` / `false` / `null` literal → integer → float → string raw。
    ///
    /// JSON literal（如 `[1,2,3]` / `{"a":1}`）SHALL NOT 解析、回 `String(raw)`；
    /// 引號夾住的字串（`"1.5"`）保留引號、回 `String(raw)`（shell 已將引號剝除的
    /// 情境由 CLI 上游處理）。
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw {
            "true" => return Self::Bool(true),
            "false" => return Self::Bool(false),
            "null" => return Self::Null,
            _ => {}
        }
        // Integer：`-?[0-9]+`，需排除 1.5 這類 float 形態以避免吞掉小數。
        if !raw.contains('.') {
            if let Ok(n) = raw.parse::<i64>() {
                return Self::Int(n);
            }
        }
        // Float：`-?[0-9]+\.[0-9]+`（簡化版；`f64::from_str` 接受更寬鬆的 grammar、
        // 但本 slice 對齊 spec table 即可）。
        if raw.contains('.')
            && !raw.starts_with('"')
            && raw.bytes().filter(|&b| b == b'.').count() == 1
        {
            if let Ok(f) = raw.parse::<f64>() {
                if f.is_finite() {
                    return Self::Float(f);
                }
            }
        }
        Self::String(raw.to_string())
    }
}

/// `ConfigStore::write_config` 的輸入請求。
///
/// `Set` 走 JSONPath subset 單 key 修補；`Edit` 走整檔覆寫（stdin 或 editor 路徑
/// 提供 content）。`expected_etag=None` 仍由實作端對讀到的當前 etag 做 internal
/// CAS（spec requirement 註明）。
#[derive(Debug, Clone, PartialEq)]
pub enum WriteConfigRequest {
    Set {
        key: JsonPath,
        value: ConfigValue,
        expected_etag: Option<Etag>,
        actor: Option<Actor>,
    },
    Edit {
        content: String,
        expected_etag: Option<Etag>,
        actor: Option<Actor>,
    },
}

/// `read_config()` / `write_config()` 在 read/reconcile/write 過程中累積的 warning。
///
/// 由 caller（runtime layer `config_ops`）透過 `ConfigStore::take_warnings()` drain，
/// 再包到 JSON envelope `warnings` 陣列。對應 spec requirement「Audit-only codes
/// SHALL also be added — `config.external_edit_detected` / `config.malformed_using_defaults`」。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigWarning {
    /// 對應 declared warning code（如 `config.external_edit_detected`）。
    pub code: &'static str,
    /// 人類可讀描述，CLI human renderer 直接印出。
    pub message: String,
}

/// Config read / write trait。
///
/// 沿 design contract sync 表面；A5 LocalConfigStore 在單一 SQLite tx 內完成
/// CAS + audit row insert + atomic file write。HttpProvider 實作預留未來 slice。
///
/// Warnings via [`Self::take_warnings`] 為 additive method — spec scenario
/// 「Provider trait surface stable across crates」核心斷言（`read_config()` 回
/// `Result<Versioned<Config>, ProviderError>`）不受影響。
pub trait ConfigStore: Send + Sync {
    /// 讀取當前 config + 計算 etag；檔案缺失 / YAML 解析失敗時走 fallback
    /// （回 defaults + warning，**不** raise error）。Read path 偵測到 external
    /// edit 也走 reconcile + warning（不 raise error），不會回 `state.etag_mismatch`。
    fn read_config(&self) -> Result<Versioned<Config>, ProviderError>;

    /// 寫入 config；`Set` 對 key 套 patch、`Edit` 整檔覆寫。
    /// CAS 失敗回 `state.etag_mismatch`、unknown key 回 `config.key_not_found`、
    /// malformed content 回 `config.malformed`。
    fn write_config(&self, request: WriteConfigRequest)
    -> Result<Versioned<Config>, ProviderError>;

    /// 純函式，回 walking-skeleton defaults（`require_*_review: false`）。
    fn read_defaults(&self) -> Config;

    /// 取出最近一次 `read_config()` / `write_config()` 累積的 warning 並清空 buffer。
    ///
    /// 對 stateless impl（trait-surface stub、HttpProvider future slice）回空 vec
    /// 即可；LocalConfigStore 透過內部 `Mutex` 在 read/reconcile 路徑累積。
    fn take_warnings(&self) -> Vec<ConfigWarning>;
}

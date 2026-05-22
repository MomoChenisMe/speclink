//! 共用資料型別：`LinkYaml`、`ProjectInfo`、`ProjectStatus`、`InitOptions`。
//!
//! 這些型別是 SpecLink 各 provider 實作之間的 stable contract。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `.speclink/link.yaml` v1 schema。
///
/// 由 `LocalProvider::init` 寫入，由 `LocalProvider::get_link` / `save_link` 讀取。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkYaml {
    /// Schema 版本，目前固定為 `1`。
    pub version: u32,
    /// SpecLink project 唯一識別碼（UUID v4）。
    pub project_id: String,
    /// 本機 binding 的 instance 識別碼（UUID v4），`init --force` 後會輪轉。
    pub instance_id: String,
    /// Provider 種類，LocalProvider 寫 `"local"`。
    pub provider: String,
    /// `init` 寫入或 `init --force` 重生時的 RFC 3339 時間戳。
    pub created_at: String,
    /// Working tree root 絕對路徑的 SHA-256 hex digest（64 字元小寫）。
    pub working_dir_fingerprint: String,
}

/// `speclink status` 回傳的專案狀態。
///
/// JSON key 順序與 design「Implementation Contract → Observable behavior」表
/// 一致：`project_id`、`provider`、`artifact_root`、`state_root`、`git_head`、`requires_git`。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectStatus {
    pub project_id: String,
    pub provider: String,
    pub artifact_root: String,
    pub state_root: String,
    pub git_head: String,
    pub requires_git: bool,
}

/// `init` / `link` 等命令成功後回傳的精簡資訊。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    pub project_id: String,
    pub artifact_root: String,
    pub state_root: String,
}

/// `init` 的輸入旗標。
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// 目標 working dir（必須為 git working tree）。
    pub working_dir: PathBuf,
    /// 若 `.speclink/link.yaml` 已存在，是否強制覆寫並輪轉 `instance_id`。
    pub force: bool,
}

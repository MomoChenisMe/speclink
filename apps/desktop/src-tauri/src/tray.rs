//! Tray recovery action 的封閉 IPC／focus policy。
//!
//! 前端 Zustand store 仍是 snapshot 與動作的單一真相；本模組只釘住跨 surface
//! 的 action 名稱與「哪些顯式動作可以聚焦主視窗」，避免未知 payload 誤降階。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrayRecoveryAction {
    Retry,
    OpenRecovery,
    OpenSettings,
    Reauthenticate,
}

impl TrayRecoveryAction {
    /// 直接 retry 必須留在 Tray；只有使用者明確要求詳情／設定／登入才聚焦主視窗。
    pub const fn focuses_main_window(self) -> bool {
        !matches!(self, Self::Retry)
    }
}

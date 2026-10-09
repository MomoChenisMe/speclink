//! 專案層檔案系統動作（design D5）：候選組裝與依序啟動在 speclink-desktop-core，
//! 這裡只委派。「在檔案管理員顯示」沿用既有的 `reveal_in_folder`。
//!
//! PATH 取使用者 login shell 的值：macOS 從 Dock 啟動的 app 只有系統預設 PATH，
//! 找不到 Homebrew 或 /usr/local/bin 下的 code 等指令。

use speclink_desktop_core::fs_actions::{editor_candidates, launch_first, terminal_candidates, Platform};

use crate::cli_install::user_shell_path;

#[tauri::command]
pub async fn open_in_terminal(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let candidates = terminal_candidates(Platform::current(), &path);
        launch_first(&path, candidates, &user_shell_path(), "fs.noTerminal")
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn open_in_editor(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let candidates = editor_candidates(Platform::current(), &path);
        launch_first(&path, candidates, &user_shell_path(), "fs.noEditor")
    })
    .await
    .map_err(|e| e.to_string())?
}

//! CLI 佈署的殼層（desktop-app spec「安裝 CLI 指令到 PATH」，design D5）：
//! 收集環境事實（平台、PATH、sidecar 路徑、已佈署版本輸出）與執行佈署計畫
//! （symlink／copy）。狀態判定與平台分流的決策歸前端 core（vitest 覆蓋），
//! 此處不做任何判斷。

use std::path::PathBuf;
use std::process::Command;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliInstallProbe {
    pub platform: String,
    pub home: Option<String>,
    pub path_env: String,
    pub path_delimiter: String,
    pub bundled_cli_path: Option<String>,
    pub app_version: String,
    pub deployed_version_output: Option<String>,
}

fn platform_key() -> String {
    if cfg!(target_os = "macos") {
        "macos".into()
    } else if cfg!(target_os = "windows") {
        "windows".into()
    } else if std::env::var_os("APPIMAGE").is_some() {
        "linux-appimage".into()
    } else {
        "linux-deb".into()
    }
}

/// 隨附 CLI 的位置：Tauri externalBin 佈於主執行檔同目錄。
fn bundled_cli_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let name = if cfg!(target_os = "windows") { "speclink.exe" } else { "speclink" };
    let path = exe.parent()?.join(name);
    path.exists().then_some(path)
}

/// 已佈署 CLI 的 `--version` 原始輸出（解析歸前端 core）；不存在／不可執行＝None。
fn deployed_version_output(platform: &str, home: Option<&str>) -> Option<String> {
    let program: PathBuf = match platform {
        "macos" | "linux-appimage" => PathBuf::from(home?).join(".local/bin/speclink"),
        "linux-deb" => PathBuf::from("/usr/bin/speclink"),
        // Windows 由安裝器把 CLI 佈於 app 同目錄並寫 PATH；剛裝完時 PATH
        // broadcast 未及已執行程序，走程序 PATH 會誤報未安裝——直接以
        // sidecar 同目錄（即 $INSTDIR）執行。
        _ => bundled_cli_path()?,
    };
    let output = Command::new(program).arg("--version").output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// PATH 提示的判定基準是「使用者終端」的 PATH——macOS GUI app 從 Dock 啟動
/// 不繼承 shell 設定檔，程序 PATH 只有系統預設，直接用會誤報「不在 PATH」。
/// 以 login shell 解析真實 PATH；失敗（非 unix、shell 異常）退回程序 PATH。
fn user_shell_path() -> String {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        if let Ok(output) = Command::new(&shell).args(["-lc", r#"printf %s "$PATH""#]).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).into_owned();
                if !path.trim().is_empty() {
                    return path;
                }
            }
        }
    }
    std::env::var("PATH").unwrap_or_default()
}

pub fn probe(app_version: String) -> CliInstallProbe {
    let platform = platform_key();
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok());
    let deployed_version_output = deployed_version_output(&platform, home.as_deref());
    CliInstallProbe {
        platform,
        path_env: user_shell_path(),
        path_delimiter: if cfg!(target_os = "windows") { ";".into() } else { ":".into() },
        bundled_cli_path: bundled_cli_path().map(|p| p.to_string_lossy().into_owned()),
        app_version,
        deployed_version_output,
        home,
    }
}

/// 前端 core 產出的佈署計畫（「none」不會被送來——前端已擋）。
#[derive(serde::Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum CliDeployPlan {
    #[serde(rename = "symlink", rename_all = "camelCase")]
    Symlink { link_path: PathBuf, target_path: PathBuf },
    #[serde(rename = "copy", rename_all = "camelCase")]
    Copy { dest_path: PathBuf, source_path: PathBuf },
}

pub fn deploy(plan: CliDeployPlan) -> Result<(), String> {
    match plan {
        CliDeployPlan::Symlink { link_path, target_path } => {
            let parent = link_path.parent().ok_or("symlink 路徑缺少父目錄")?;
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            // 既有 symlink／檔案先移除，重複安裝與換目標皆冪等。
            if std::fs::symlink_metadata(&link_path).is_ok() {
                std::fs::remove_file(&link_path).map_err(|e| e.to_string())?;
            }
            #[cfg(unix)]
            return std::os::unix::fs::symlink(&target_path, &link_path).map_err(|e| e.to_string());
            #[cfg(not(unix))]
            {
                let _ = target_path;
                Err("此平台不支援 symlink 佈署".into())
            }
        }
        CliDeployPlan::Copy { dest_path, source_path } => {
            let parent = dest_path.parent().ok_or("複製目的地缺少父目錄")?;
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            std::fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
    }
}

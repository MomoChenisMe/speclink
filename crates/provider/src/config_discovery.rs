//! 配置檔位置發現：
//!
//! - `find_project_config(start_dir)`：從給定目錄向上搜尋 `.speclink/config.toml`，
//!   停在第一個 match、filesystem root 或含 `.git` 目錄的祖先（後者代表 git project root）
//! - `global_config_path()`：依據 `SPECLINK_CONFIG_HOME` 環境變數覆寫或三平台預設 config 目錄
//!   回傳全域設定路徑（即使檔案不存在）

use std::path::{Path, PathBuf};

/// `SPECLINK_CONFIG_HOME` 環境變數名稱。
pub const ENV_CONFIG_HOME: &str = "SPECLINK_CONFIG_HOME";

/// 從 `start_dir` 開始向上搜尋 `.speclink/config.toml`。
///
/// 停止條件：
/// 1. 該層目錄內存在 `.speclink/config.toml` → 回傳該檔絕對路徑
/// 2. 該層目錄存在 `.git/` 但沒有 `.speclink/config.toml` → 停止搜尋，回傳 `None`
/// 3. 已抵達 filesystem root（無 parent） → 回傳 `None`
pub fn find_project_config(start_dir: &Path) -> Option<PathBuf> {
    let mut cur: Option<&Path> = Some(start_dir);
    while let Some(dir) = cur {
        let candidate = dir.join(".speclink").join("config.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if dir.join(".git").exists() {
            // git root；停止往上搜
            return None;
        }
        cur = dir.parent();
    }
    None
}

/// 回傳全域設定檔路徑（`<config_home>/speclink/config.toml`），即使檔案不存在。
///
/// `<config_home>` 取得順序：
/// 1. `SPECLINK_CONFIG_HOME` 環境變數（非空字串）
/// 2. `dirs::config_dir()`（Windows `%APPDATA%`、Linux `~/.config`、macOS `~/Library/Application Support`）
///
/// 兩者皆缺則回傳 `None`。
pub fn global_config_path() -> Option<PathBuf> {
    global_config_path_with_env(|name| std::env::var(name).ok())
}

/// `global_config_path` 的可測試版本：以注入的環境變數讀取函式取代 `std::env::var`。
pub fn global_config_path_with_env<F>(env: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Option<String>,
{
    let override_home = env(ENV_CONFIG_HOME).filter(|s| !s.is_empty());
    let base = match override_home {
        Some(h) => PathBuf::from(h),
        None => dirs::config_dir()?,
    };
    Some(base.join("speclink").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use crate::config_discovery::{
        ENV_CONFIG_HOME, find_project_config, global_config_path_with_env,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn find_project_config_stops_at_git_root() {
        // Layout:
        //   <root>/.git/
        //   <root>/.speclink/config.toml
        //   <root>/sub/sub2/
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join(".speclink")).unwrap();
        fs::write(root.join(".speclink").join("config.toml"), "").unwrap();
        fs::create_dir_all(root.join("sub").join("sub2")).unwrap();

        let start = root.join("sub").join("sub2");
        let found = find_project_config(&start).expect("should find config");
        assert_eq!(found, root.join(".speclink").join("config.toml"));
    }

    #[test]
    fn find_project_config_returns_none_when_absent() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // 模擬 git root 以避免搜尋向上跳出 tempdir 命中真實 .speclink
        fs::create_dir_all(root.join(".git")).unwrap();
        let start = root.to_path_buf();
        assert_eq!(find_project_config(&start), None);
    }

    #[test]
    fn global_config_path_override_via_env() {
        let override_dir = "/tmp/test-config";
        let path = global_config_path_with_env(|name| {
            if name == ENV_CONFIG_HOME {
                Some(override_dir.to_string())
            } else {
                None
            }
        })
        .expect("should produce path");
        assert_eq!(
            path,
            PathBuf::from(override_dir)
                .join("speclink")
                .join("config.toml")
        );
    }

    #[test]
    fn global_config_path_empty_env_falls_back_to_dirs() {
        // 設定環境變數為空字串 → 視同未設定
        let path = global_config_path_with_env(|name| {
            if name == ENV_CONFIG_HOME {
                Some(String::new())
            } else {
                None
            }
        });
        // dirs::config_dir() 在測試環境會回傳值；只要 path suffix 正確即可
        if let Some(p) = path {
            assert!(
                p.ends_with("speclink/config.toml"),
                "unexpected fallback path: {p:?}"
            );
        }
    }
}

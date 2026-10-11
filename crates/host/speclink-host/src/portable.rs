//! Windows 隨身碟模式：EXE 旁的 speclink-portable 標記啟用獨立 data 目錄。
//! 不修改 HOME／USERPROFILE，也不影響沒有標記的安裝版。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn data_dir_for(exe: &Path, windows: bool) -> Option<PathBuf> {
    let dir = exe.parent()?;
    (windows && dir.join("speclink-portable").is_file()).then(|| dir.join("data"))
}

/// 首次讀取後固定本次行程的模式，避免執行中移除標記導致改用本機設定。
pub fn data_dir() -> Option<&'static Path> {
    static DIRECTORY: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIRECTORY
        .get_or_init(|| {
            std::env::current_exe().ok().and_then(|exe| data_dir_for(&exe, cfg!(windows)))
        })
        .as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_is_opt_in_and_only_applies_to_windows() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("speclink-desktop.exe");
        assert_eq!(data_dir_for(&exe, true), None);
        std::fs::write(dir.path().join("speclink-portable"), "").unwrap();
        assert_eq!(data_dir_for(&exe, true), Some(dir.path().join("data")));
        assert_eq!(data_dir_for(&exe, false), None);
        assert_eq!(data_dir_for(&dir.path().join("speclink.exe"), true), data_dir_for(&exe, true));
    }

    #[test]
    fn data_follows_executable_when_package_moves() {
        let parent = tempfile::tempdir().unwrap();
        let original = parent.path().join("original");
        let moved = parent.path().join("moved");
        std::fs::create_dir(&original).unwrap();
        std::fs::write(original.join("speclink-portable"), "").unwrap();
        std::fs::rename(&original, &moved).unwrap();
        assert_eq!(data_dir_for(&moved.join("speclink.exe"), true), Some(moved.join("data")));
    }
}

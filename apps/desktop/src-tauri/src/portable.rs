//! portable 模式在建立 WebView 前決定資料位置，不使用本機 AppData。

use std::path::Path;

pub fn windows(
    config: &mut tauri::Config,
    data: Option<&Path>,
) -> Vec<tauri::utils::config::WindowConfig> {
    if data.is_none() {
        return Vec::new();
    }
    let windows = config.app.windows.iter().filter(|w| w.create).cloned().collect();
    for window in &mut config.app.windows {
        window.create = false;
    }
    windows
}

pub fn prepare(data: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(data.join("desktop"))?;
    std::fs::create_dir_all(data.join("speclink"))?;
    std::fs::create_dir_all(data.join("webview"))?;
    // 唯讀隨身碟直接報錯，不退回到本機目錄。
    let probe = data.join(format!(".write-check-{}", std::process::id()));
    std::fs::OpenOptions::new().write(true).create_new(true).open(&probe)?;
    std::fs::remove_file(probe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_created_only_in_the_portable_directory() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        prepare(&data).unwrap();
        for name in ["desktop", "speclink", "webview"] {
            assert!(data.join(name).is_dir());
        }
        assert_eq!(std::fs::read_dir(data).unwrap().count(), 3);
    }

    #[test]
    fn unusable_data_path_fails_instead_of_falling_back() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        std::fs::write(&data, "not a directory").unwrap();
        assert!(prepare(&data).is_err());
        assert_eq!(std::fs::read_to_string(data).unwrap(), "not a directory");
    }

    #[test]
    fn portable_delays_window_creation_until_absolute_data_path_is_set() {
        let mut config = tauri::Config::default();
        config.app.windows.push(Default::default());
        assert!(windows(&mut config, None).is_empty());
        assert!(config.app.windows[0].create);
        let deferred = windows(&mut config, Some(Path::new("data")));
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred[0].label, config.app.windows[0].label);
        assert!(!config.app.windows[0].create);
    }
}

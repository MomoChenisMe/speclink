//! Device preferences; never reads or writes project policy.
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use speclink_host::network::{validate_proxy_url, ProxyMode, ProxyPreferences};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimePreferences {
    pub format_version: u8,
    pub debug_enabled: bool,
    pub proxy: ProxyPreferences,
}
impl Default for RuntimePreferences {
    fn default() -> Self {
        Self {
            format_version: 1,
            debug_enabled: false,
            proxy: ProxyPreferences::default(),
        }
    }
}
pub fn read(root: &Path) -> Result<RuntimePreferences, String> {
    let path = root.join("runtime-preferences.json");
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(RuntimePreferences::default())
        }
        Err(_) => return Err("preferences_invalid".into()),
    };
    use std::io::Read;
    let mut bytes = Vec::new();
    file.take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "preferences_invalid")?;
    if bytes.len() > 16 * 1024 {
        return Err("preferences_invalid".into());
    }
    let value: RuntimePreferences =
        serde_json::from_slice(&bytes).map_err(|_| "preferences_invalid")?;
    validate(&value).map_err(|_| "preferences_invalid")?;
    Ok(value)
}
pub fn validate(value: &RuntimePreferences) -> Result<(), String> {
    if value.format_version != 1 {
        return Err("preferences_invalid".into());
    }
    match value.proxy.mode {
        ProxyMode::Manual => {
            validate_proxy_url(value.proxy.url.as_deref().unwrap_or(""))?;
        }
        _ if value.proxy.url.is_some() => return Err("proxy_invalid".into()),
        _ => {}
    }
    Ok(())
}
pub fn write(root: &Path, value: &RuntimePreferences) -> Result<(), String> {
    validate(value)?;
    let persist = || -> std::io::Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(root)?;
        let mut file = tempfile::NamedTempFile::new_in(root)?;
        file.write_all(&serde_json::to_vec_pretty(value)?)?;
        file.as_file().sync_all()?;
        file.persist(root.join("runtime-preferences.json"))
            .map_err(|e| e.error)?;
        Ok(())
    };
    persist().map_err(|_| "settings_write_failed".into())
}

#[derive(Clone)]
pub struct DeviceRuntime {
    pub root: std::path::PathBuf,
    pub diagnostics: crate::diagnostics::Diagnostics,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    #[serde(flatten)]
    pub preferences: RuntimePreferences,
    pub read_error: Option<String>,
    pub diagnostics_status: String,
}
impl DeviceRuntime {
    pub fn new(root: std::path::PathBuf) -> Self {
        let enabled = read(&root).is_ok_and(|p| p.debug_enabled);
        let diagnostics = crate::diagnostics::Diagnostics::new(root.join("diagnostics"), enabled);
        Self { root, diagnostics }
    }
    pub fn read(&self) -> RuntimeSnapshot {
        let (preferences, read_error) = match read(&self.root) {
            Ok(value) => (value, None),
            Err(code) => (RuntimePreferences::default(), Some(code)),
        };
        RuntimeSnapshot {
            preferences,
            read_error,
            diagnostics_status: self.diagnostics.status().into(),
        }
    }
    pub fn write(&self, preferences: RuntimePreferences) -> Result<RuntimeSnapshot, String> {
        write(&self.root, &preferences)?;
        self.diagnostics.set_enabled(preferences.debug_enabled);
        Ok(self.read())
    }
    pub fn policy(&self) -> Result<speclink_host::network::ProxyPolicy, String> {
        crate::network::policy_at(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_file_does_not_create_preferences() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(read(root.path()).unwrap(), RuntimePreferences::default());
        assert!(!root.path().join("runtime-preferences.json").exists());
    }
    #[test]
    fn preferences_round_trip_and_invalid_version() {
        let root = tempfile::tempdir().unwrap();
        let value = RuntimePreferences {
            debug_enabled: true,
            ..Default::default()
        };
        write(root.path(), &value).unwrap();
        assert_eq!(read(root.path()).unwrap(), value);
        let json = std::fs::read_to_string(root.path().join("runtime-preferences.json")).unwrap();
        assert!(json.contains("debugEnabled"));
        assert!(!json.contains("debug_enabled"));
        std::fs::write(
            root.path().join("runtime-preferences.json"),
            "{\"formatVersion\":99}",
        )
        .unwrap();
        assert_eq!(read(root.path()).unwrap_err(), "preferences_invalid");
        write(root.path(), &RuntimePreferences::default()).unwrap();
        assert_eq!(read(root.path()).unwrap(), RuntimePreferences::default());
    }
    #[test]
    fn invalid_proxy_and_failed_write_preserve_previous_file() {
        let root = tempfile::tempdir().unwrap();
        let good = RuntimePreferences::default();
        write(root.path(), &good).unwrap();
        let bad = RuntimePreferences {
            proxy: ProxyPreferences {
                mode: ProxyMode::Manual,
                url: Some("http://user:secret@proxy:8080".into()),
            },
            ..good.clone()
        };
        assert_eq!(write(root.path(), &bad).unwrap_err(), "proxy_invalid");
        assert_eq!(read(root.path()).unwrap(), good);
        let occupied = root.path().join("not-a-directory");
        std::fs::write(&occupied, "original").unwrap();
        assert_eq!(
            write(&occupied, &good).unwrap_err(),
            "settings_write_failed"
        );
        assert_eq!(std::fs::read_to_string(occupied).unwrap(), "original");
    }
}

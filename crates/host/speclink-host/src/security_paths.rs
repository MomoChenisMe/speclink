//! Device-owned standard location. A lease pins the path for an entire operation.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const PREFERENCES: &str = "security-data-preferences.json";
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Preferences {
    format_version: u8,
    standards_path: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSnapshot {
    pub standards_path: Option<String>,
    pub effective_path: String,
    pub is_default: bool,
    pub access: &'static str,
}
pub struct PathLease {
    pub standards_root: PathBuf,
    pub lock: File,
}

pub fn validate_path(raw: &str) -> Result<()> {
    let path = Path::new(raw);
    ensure!(
        !raw.is_empty() && !raw.contains('\0') && !raw.contains("://") && path.is_absolute(),
        "invalid_path: absolute directory required"
    );
    ensure!(
        !path.components().any(|c| matches!(c, Component::ParentDir)),
        "invalid_path: parent traversal"
    );
    #[cfg(windows)]
    {
        use std::path::Prefix;
        ensure!(
            matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_) | Prefix::UNC(_, _))),
            "invalid_path: unsupported device path"
        );
    }
    Ok(())
}
fn check_directory(path: &Path, create: bool) -> Result<()> {
    let resolved = speclink_core::util::resolve_system_path(path);
    let path = resolved.as_path();
    validate_path(&path.to_string_lossy())?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            check_directory(parent, create)?;
        }
    }
    match fs::symlink_metadata(path) {
        Err(e) if create && e.kind() == std::io::ErrorKind::NotFound => {
            match fs::create_dir(path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return check_directory(path, false)
                }
                Err(e) => return Err(e.into()),
            }
        }
        Err(e) => return Err(e.into()),
        Ok(metadata) => {
            ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "invalid_path: linked or non-directory path"
            );
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                ensure!(
                    metadata.file_attributes() & 0x400 == 0,
                    "invalid_path: reparse directory"
                );
            }
        }
    }
    Ok(())
}
fn load(config: &Path) -> Result<Preferences> {
    ensure!(
        config.is_absolute(),
        "data_directory_unavailable: absolute config directory required"
    );
    let path = config.join(PREFERENCES);
    let metadata = match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Preferences {
                format_version: 1,
                standards_path: None,
            })
        }
        other => other?,
    };
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() <= 16 * 1024,
        "preferences_invalid"
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "preferences_invalid"
        );
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(16 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 16 * 1024, "preferences_invalid");
    let value: Preferences = serde_json::from_slice(&bytes).context("preferences_invalid")?;
    ensure!(value.format_version == 1, "preferences_invalid");
    if let Some(raw) = &value.standards_path {
        validate_path(raw)?;
    }
    Ok(value)
}
fn effective(value: &Preferences, application_data: &Path) -> Result<PathBuf> {
    ensure!(application_data.is_absolute(), "data_directory_unavailable");
    Ok(speclink_core::util::resolve_system_path(&value
        .standards_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| application_data.join("security/standards"))))
}
fn snapshot(value: Preferences, application_data: &Path) -> Result<PathSnapshot> {
    let root = effective(&value, application_data)?;
    let access = match check_directory(&root, value.standards_path.is_none())
        .and_then(|_| fs::read_dir(&root).map(|_| ()).map_err(Into::into))
    {
        Err(_) => "unavailable",
        Ok(()) => {
            if tempfile::NamedTempFile::new_in(&root).is_ok() {
                "readWrite"
            } else {
                "readOnly"
            }
        }
    };
    Ok(PathSnapshot {
        is_default: value.standards_path.is_none(),
        standards_path: value.standards_path,
        effective_path: root.to_string_lossy().into_owned(),
        access,
    })
}
pub fn read_at(config: &Path, application_data: &Path) -> Result<PathSnapshot> {
    snapshot(load(config)?, application_data)
}
fn gate(config: &Path, exclusive: bool) -> Result<File> {
    check_directory(config, true)?;
    let path = config.join(".security-data-path.lock");
    if let Ok(metadata) = fs::symlink_metadata(&path) {
        ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "preferences_invalid"
        );
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    if exclusive {
        file.try_lock()
    } else {
        file.try_lock_shared()
    }
    .map_err(|_| anyhow::anyhow!("busy: standard data operation in progress"))?;
    Ok(file)
}
pub fn lease_at(config: &Path, application_data: &Path) -> Result<PathLease> {
    let lock = gate(config, false)?;
    let value = load(config)?;
    let standards_root = effective(&value, application_data)?;
    if value.standards_path.is_some() {
        check_directory(&standards_root, false).context("data_directory_unavailable")?;
        fs::read_dir(&standards_root).context("data_directory_unavailable")?;
    }
    Ok(PathLease {
        standards_root,
        lock,
    })
}
pub fn lease(application_data: &Path) -> Result<PathLease> {
    lease_at(&crate::context::global_config_dir(), application_data)
}
pub fn write_at(
    config: &Path,
    application_data: &Path,
    standards_path: Option<String>,
) -> Result<PathSnapshot> {
    let _lock = gate(config, true)?;
    let _old = load(config)?;
    if let Some(raw) = &standards_path {
        validate_path(raw)?;
    }
    let value = Preferences {
        format_version: 1,
        standards_path,
    };
    let result = snapshot(value, application_data)?;
    ensure!(
        result.access != "unavailable",
        "data_directory_unavailable: selected directory is not readable"
    );
    let value = Preferences {
        format_version: 1,
        standards_path: result.standards_path.clone(),
    };
    let mut temporary = tempfile::NamedTempFile::new_in(config)?;
    temporary.write_all(&serde_json::to_vec_pretty(&value)?)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(config.join(PREFERENCES))
        .map_err(|e| anyhow::anyhow!("settings_write_failed: {}", e.error))?;
    Ok(result)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn system_alias_is_usable_but_user_links_remain_rejected() {
        let root=tempfile::tempdir_in("/tmp").unwrap();
        let config=root.path().join("config");
        let data=root.path().join("device");
        let shared=root.path().join("shared");
        fs::create_dir(&shared).unwrap();
        let result=write_at(&config,&data,Some(shared.to_string_lossy().into())).unwrap();
        assert_eq!(PathBuf::from(result.effective_path),shared.canonicalize().unwrap());
        let linked=root.path().join("user-link");
        std::os::unix::fs::symlink(&shared,&linked).unwrap();
        assert!(write_at(&config,&data,Some(linked.to_string_lossy().into())).is_err());
        assert_eq!(lease_at(&config,&data).unwrap().standards_root,shared.canonicalize().unwrap());
    }
}

//! PAT credential storage and token resolution.
//!
//! Credentials live in a YAML map (`<url origin> → <token>`) in the
//! user-level config directory — never inside the repo. `SPECLINK_TOKEN`
//! always wins over the file (CI/headless override).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The credentials file inside an explicit base directory (test seam).
pub fn credentials_path_in(dir: &Path) -> PathBuf {
    dir.join("credentials.yaml")
}

/// The credentials file in the user-level config directory.
pub fn credentials_path() -> PathBuf {
    credentials_path_in(&speclink_config_dir())
}

/// The user-level config directory (same base as the global CLI config).
pub fn speclink_config_dir() -> PathBuf {
    speclink_core::config::global_config_dir()
}

/// The origin (`scheme://host[:port]`) of a connection URL — credentials are
/// keyed by origin so one login covers every project on the same server.
pub fn origin_of(url: &str) -> String {
    match url.find("://") {
        Some(scheme_end) => {
            let host_start = scheme_end + 3;
            let end = url[host_start..]
                .find('/')
                .map(|i| host_start + i)
                .unwrap_or(url.len());
            url[..end].to_string()
        }
        None => url.trim_end_matches('/').to_string(),
    }
}

fn read_map(path: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_yaml::from_str(&text).ok())
        .unwrap_or_default()
}

/// Store `token` for `origin` in the credentials file under `dir`,
/// creating the file with owner-only permissions (Unix 0600; Windows relies
/// on the user-profile directory ACL). Other origins in the file survive.
pub fn save_token_at(dir: &Path, origin: &str, token: &str) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = credentials_path_in(dir);
    let mut map = read_map(&path);
    map.insert(origin.to_string(), token.to_string());
    let yaml = serde_yaml::to_string(&map)?;

    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        // Create with 0600 so the token is never world-readable, not even
        // briefly; tighten a pre-existing file's mode as well.
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        f.write_all(yaml.as_bytes())?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    std::fs::write(&path, yaml)?;

    Ok(path)
}

/// Read the token stored for `origin`, if any. A missing file or missing
/// entry is simply `None` (treated as "not logged in").
pub fn load_token_at(dir: &Path, origin: &str) -> Option<String> {
    read_map(&credentials_path_in(dir)).remove(origin)
}

/// Resolution order: an explicit environment token (SPECLINK_TOKEN) wins
/// over the credentials file; neither present → `None`. An empty environment
/// value counts as unset — it never silently disables file credentials.
pub fn resolve_token_at(dir: &Path, origin: &str, env_token: Option<String>) -> Option<String> {
    env_token
        .filter(|t| !t.trim().is_empty())
        .or_else(|| load_token_at(dir, origin))
}

/// [`resolve_token_at`] against the real config dir and process environment.
pub fn resolve_token(origin: &str) -> Option<String> {
    resolve_token_at(
        &speclink_config_dir(),
        origin,
        std::env::var("SPECLINK_TOKEN").ok(),
    )
}

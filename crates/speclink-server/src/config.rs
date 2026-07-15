//! Startup configuration, loaded fail closed (reference-server spec「啟動組態
//! fail closed」).
//!
//! The server starts from a single YAML file that declares the store driver,
//! the public origin, and the identity database. A file that is missing,
//! unparseable, declares an unknown driver, carries a residual `tokens` or
//! `projects` section, or is otherwise shaped wrong makes startup fail with a
//! reason pointing at the file — never a partial-default start. Authentication
//! is the identity store's PATs; the old bootstrap `tokens` section is gone, and
//! the Project/Repo registry now lives in the server database (server-setup
//! capability), not the config.

use crate::events::EventSettings;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The store drivers the server can bind. `sqlite` is the default persistent
/// option, `serverfs` the plain-directory alternative, `postgres` the option for
/// teams that already run one; `memory` exists only for test configurations.
pub const SUPPORTED_DRIVERS: &[&str] = &["sqlite", "serverfs", "postgres", "memory"];

/// The validated server configuration. The Project/Repo registry is no longer a
/// config concern — it lives in the server database (server-setup capability).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub store: StoreConfig,
    pub identity: IdentityConfig,
    /// The server's external origin, used to build invite URLs and to validate
    /// the Origin of change-making POSTs (決策 4).
    pub public_url: String,
    /// Event-stream tunables (retention, connection buffer, heartbeat). An
    /// absent `events` section uses the defaults; a malformed one fails startup.
    pub events: EventSettings,
}

/// The store backend to bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreConfig {
    /// A single SQLite database file.
    Sqlite { path: PathBuf },
    /// A plain data directory, driven by the filesystem driver. Single-node,
    /// and requires a filesystem with working advisory locks (a local disk or
    /// a mount that honours flock).
    ServerFs { path: PathBuf },
    /// A PostgreSQL database, named by a connection URL. Single-node: the
    /// driver serializes writers, it does not coordinate a cluster. The
    /// password may be left out of the URL and supplied by
    /// `SPECLINK_POSTGRES_PASSWORD` instead.
    Postgres { url: String },
    /// The in-memory reference store — test configurations only.
    Memory,
}

/// Where the server's identity store lives. A separate database from the store
/// (決策 1); the `memory` variant is for test configurations only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityConfig {
    /// A single SQLite identity database file.
    Sqlite { path: PathBuf },
    /// An in-memory identity store — test configurations only.
    Memory,
}

/// The actor a request authenticates as: a stable id and the display identity
/// recorded in history and events. Sourced from the PAT's owning user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorConfig {
    pub id: String,
    pub display: String,
}

/// Why loading the configuration failed. Every variant names the file so the
/// operator can act; the reason follows.
#[derive(Debug)]
pub enum ConfigError {
    /// The file could not be read (missing, no permission, …).
    Unreadable { path: String, source: String },
    /// The file is not parseable as the expected YAML shape.
    Unparseable { path: String, reason: String },
    /// The store driver is not one this server supports.
    UnknownDriver { path: String, driver: String },
    /// The identity driver is not one this server supports.
    UnknownIdentityDriver { path: String, driver: String },
    /// A required field for the chosen shape is absent.
    MissingField { path: String, field: String },
    /// The config still carries the retired bootstrap `tokens` section.
    ResidualTokens { path: String },
    /// The config still carries the retired Project/Repo `projects` section,
    /// replaced by the server database's registry.
    ResidualProjects { path: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Unreadable { path, source } => {
                write!(f, "cannot read config file '{path}': {source}")
            }
            ConfigError::Unparseable { path, reason } => {
                write!(f, "cannot parse config file '{path}': {reason}")
            }
            ConfigError::UnknownDriver { path, driver } => write!(
                f,
                "config file '{path}': unknown store driver '{driver}' (supported: {})",
                SUPPORTED_DRIVERS.join(", ")
            ),
            ConfigError::UnknownIdentityDriver { path, driver } => write!(
                f,
                "config file '{path}': unknown identity driver '{driver}' (supported: {})",
                SUPPORTED_DRIVERS.join(", ")
            ),
            ConfigError::MissingField { path, field } => {
                write!(f, "config file '{path}': missing required field '{field}'")
            }
            ConfigError::ResidualTokens { path } => write!(
                f,
                "config file '{path}': the 'tokens' section has been replaced by the identity store — remove it and declare an 'identity' database instead"
            ),
            ConfigError::ResidualProjects { path } => write!(
                f,
                "config file '{path}': the 'projects' section has been replaced by the server database's registry — remove it and register projects through /setup or the admin interface"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

// --- raw YAML shapes (required fields have no default: a missing one is a
// parse failure, i.e. fail closed on shape) ---

#[derive(Deserialize)]
struct RawConfig {
    store: RawStore,
    #[serde(default)]
    identity: Option<RawIdentity>,
    #[serde(default)]
    public_url: Option<String>,
    /// Event-stream tunables. Absent → all defaults; a malformed shape fails the
    /// whole parse (fail closed), like any other config error.
    #[serde(default)]
    events: Option<RawEvents>,
    /// Presence detection for the retired bootstrap `tokens` section: any value
    /// here fails startup with a reason.
    #[serde(default)]
    tokens: Option<serde_yaml::Value>,
    /// Presence detection for the retired Project/Repo `projects` section: any
    /// value here fails startup with a reason pointing at the registry.
    #[serde(default)]
    projects: Option<serde_yaml::Value>,
}

#[derive(Deserialize)]
struct RawEvents {
    #[serde(default)]
    retention: Option<u64>,
    #[serde(default)]
    buffer: Option<usize>,
    #[serde(default)]
    heartbeat_secs: Option<u64>,
}

#[derive(Deserialize)]
struct RawStore {
    driver: String,
    #[serde(default)]
    path: Option<String>,
    /// The `postgres` driver's connection URL.
    #[serde(default)]
    url: Option<String>,
}

#[derive(Deserialize)]
struct RawIdentity {
    driver: String,
    #[serde(default)]
    path: Option<String>,
}

/// Load and validate the configuration at `path`, fail closed.
pub fn load(path: &Path) -> Result<ServerConfig, ConfigError> {
    let shown = path.display().to_string();
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Unreadable {
        path: shown.clone(),
        source: e.to_string(),
    })?;
    let raw: RawConfig = serde_yaml::from_str(&text).map_err(|e| ConfigError::Unparseable {
        path: shown.clone(),
        reason: e.to_string(),
    })?;

    // A residual bootstrap `tokens` section is a retired shape: fail closed and
    // name its replacement.
    if raw.tokens.is_some() {
        return Err(ConfigError::ResidualTokens { path: shown });
    }

    // The Project/Repo registry moved into the server database; a residual
    // `projects` section is a retired shape that fails closed and names the
    // registry as its replacement.
    if raw.projects.is_some() {
        return Err(ConfigError::ResidualProjects { path: shown });
    }

    let store = match raw.store.driver.as_str() {
        "sqlite" => {
            let path = raw.store.path.ok_or_else(|| ConfigError::MissingField {
                path: shown.clone(),
                field: "store.path".to_string(),
            })?;
            StoreConfig::Sqlite { path: PathBuf::from(path) }
        }
        "serverfs" => {
            let path = raw.store.path.ok_or_else(|| ConfigError::MissingField {
                path: shown.clone(),
                field: "store.path".to_string(),
            })?;
            StoreConfig::ServerFs { path: PathBuf::from(path) }
        }
        "postgres" => {
            let url = raw.store.url.ok_or_else(|| ConfigError::MissingField {
                path: shown.clone(),
                field: "store.url".to_string(),
            })?;
            StoreConfig::Postgres { url }
        }
        "memory" => StoreConfig::Memory,
        other => {
            return Err(ConfigError::UnknownDriver {
                path: shown,
                driver: other.to_string(),
            })
        }
    };

    let raw_identity = raw.identity.ok_or_else(|| ConfigError::MissingField {
        path: shown.clone(),
        field: "identity".to_string(),
    })?;
    let identity = match raw_identity.driver.as_str() {
        "sqlite" => {
            let path = raw_identity.path.ok_or_else(|| ConfigError::MissingField {
                path: shown.clone(),
                field: "identity.path".to_string(),
            })?;
            IdentityConfig::Sqlite { path: PathBuf::from(path) }
        }
        "memory" => IdentityConfig::Memory,
        other => {
            return Err(ConfigError::UnknownIdentityDriver {
                path: shown,
                driver: other.to_string(),
            })
        }
    };

    let public_url = raw
        .public_url
        .unwrap_or_else(|| "http://localhost:8080".to_string());

    // An absent section is all defaults; a present one overrides only the
    // fields it names. A malformed shape already failed the parse above.
    let defaults = EventSettings::default();
    let events = match raw.events {
        Some(e) => EventSettings {
            retention: e.retention.unwrap_or(defaults.retention),
            buffer: e.buffer.unwrap_or(defaults.buffer),
            heartbeat: e.heartbeat_secs.map(Duration::from_secs).unwrap_or(defaults.heartbeat),
        },
        None => defaults,
    };

    Ok(ServerConfig { store, identity, public_url, events })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Write `text` to a temp file and load it.
    fn load_text(text: &str) -> Result<ServerConfig, ConfigError> {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(text.as_bytes()).expect("write config");
        load(file.path())
    }

    #[test]
    fn a_memory_config_loads_with_identity() {
        let cfg = load_text("store:\n  driver: memory\nidentity:\n  driver: memory\n")
            .expect("valid memory config loads");
        assert_eq!(cfg.store, StoreConfig::Memory);
        assert_eq!(cfg.identity, IdentityConfig::Memory);
    }

    #[test]
    fn a_residual_tokens_section_fails_closed_naming_its_replacement() {
        let err = load_text(
            "store:\n  driver: memory\nidentity:\n  driver: memory\ntokens:\n  - token: secret\n    actor:\n      display: Alice\n",
        )
        .expect_err("a retired tokens section must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::ResidualTokens { .. }));
        assert!(shown.contains("identity store"), "the reason points at the identity store: {shown}");
    }

    #[test]
    fn a_residual_projects_section_fails_closed_naming_its_replacement() {
        let err = load_text(
            "store:\n  driver: memory\nidentity:\n  driver: memory\nprojects:\n  - key: demo\n    repos:\n      - backend\n",
        )
        .expect_err("a retired projects section must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::ResidualProjects { .. }));
        assert!(shown.contains("registry"), "the reason points at the registry: {shown}");
    }

    #[test]
    fn a_sqlite_config_carries_its_path() {
        let cfg = load_text(
            "store:\n  driver: sqlite\n  path: /var/lib/speclink/store.db\nidentity:\n  driver: sqlite\n  path: /var/lib/speclink/identity.db\n",
        )
        .expect("valid sqlite config loads");
        assert_eq!(cfg.store, StoreConfig::Sqlite { path: PathBuf::from("/var/lib/speclink/store.db") });
        assert_eq!(
            cfg.identity,
            IdentityConfig::Sqlite { path: PathBuf::from("/var/lib/speclink/identity.db") }
        );
    }

    #[test]
    fn public_url_defaults_when_absent_and_is_honored_when_present() {
        let default = load_text("store:\n  driver: memory\nidentity:\n  driver: memory\n")
            .expect("loads");
        assert_eq!(default.public_url, "http://localhost:8080");
        let set = load_text(
            "store:\n  driver: memory\nidentity:\n  driver: memory\npublic_url: https://speclink.example\n",
        )
        .expect("loads");
        assert_eq!(set.public_url, "https://speclink.example");
    }

    #[test]
    fn a_missing_identity_section_fails_closed() {
        let err = load_text("store:\n  driver: memory\n").expect_err("identity is required");
        assert!(matches!(err, ConfigError::MissingField { field, .. } if field == "identity"));
    }

    #[test]
    fn an_unknown_identity_driver_fails_closed() {
        let err = load_text("store:\n  driver: memory\nidentity:\n  driver: postgres\n")
            .expect_err("an unsupported identity driver must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::UnknownIdentityDriver { .. }));
        assert!(shown.contains("postgres"), "names the bad identity driver: {shown}");
    }

    #[test]
    fn identity_sqlite_without_a_path_fails_closed() {
        let err = load_text("store:\n  driver: memory\nidentity:\n  driver: sqlite\n")
            .expect_err("identity sqlite needs a path");
        assert!(matches!(err, ConfigError::MissingField { field, .. } if field == "identity.path"));
    }

    #[test]
    fn a_missing_file_fails_closed_with_the_path() {
        let err = load(Path::new("/no/such/speclink-config.yaml"))
            .expect_err("a missing config file must fail startup");
        assert!(matches!(err, ConfigError::Unreadable { .. }));
        assert!(
            err.to_string().contains("/no/such/speclink-config.yaml"),
            "the reason names the file: {err}"
        );
    }

    #[test]
    fn an_unparseable_file_fails_closed_with_path_and_reason() {
        let err = load_text("store: : : not yaml\n  - broken").expect_err("bad YAML must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::Unparseable { .. }));
        assert!(shown.contains("cannot parse config file"), "reason points at parsing: {shown}");
    }

    #[test]
    fn a_wrong_shape_fails_closed() {
        // Valid YAML, but the required `store` section is absent.
        let err = load_text("projects: []\ntokens: []\n").expect_err("missing store must fail startup");
        assert!(matches!(err, ConfigError::Unparseable { .. }), "shape mismatch is a parse failure");
    }

    #[test]
    fn an_unknown_driver_fails_closed_and_lists_supported_drivers() {
        let err = load_text("store:\n  driver: mysql\n")
            .expect_err("an unsupported driver must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::UnknownDriver { .. }));
        assert!(shown.contains("mysql"), "names the bad driver: {shown}");
        assert!(
            shown.contains("sqlite")
                && shown.contains("serverfs")
                && shown.contains("postgres")
                && shown.contains("memory"),
            "lists supported drivers: {shown}"
        );
    }

    #[test]
    fn a_postgres_store_section_carries_its_connection_url() {
        let config = load_text(
            "store:\n  driver: postgres\n  url: postgres://localhost/speclink\n\
             identity:\n  driver: memory\npublic_url: http://localhost:8080\n",
        )
        .expect("a postgres store section is valid");
        assert_eq!(
            config.store,
            StoreConfig::Postgres {
                url: "postgres://localhost/speclink".to_string()
            }
        );
    }

    #[test]
    fn a_postgres_store_without_a_url_fails_closed() {
        let err = load_text("store:\n  driver: postgres\nidentity:\n  driver: memory\n")
            .expect_err("postgres without a url must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::MissingField { .. }));
        assert!(shown.contains("store.url"), "names the missing field: {shown}");
    }

    #[test]
    fn sqlite_without_a_path_fails_closed() {
        let err = load_text("store:\n  driver: sqlite\n").expect_err("sqlite needs a path");
        assert!(matches!(err, ConfigError::MissingField { field, .. } if field == "store.path"));
    }

    #[test]
    fn a_serverfs_config_carries_its_data_directory() {
        let cfg = load_text(
            "store:\n  driver: serverfs\n  path: /var/lib/speclink/store\nidentity:\n  driver: memory\n",
        )
        .expect("valid serverfs config loads");
        assert_eq!(
            cfg.store,
            StoreConfig::ServerFs { path: PathBuf::from("/var/lib/speclink/store") }
        );
    }

    #[test]
    fn serverfs_without_a_path_fails_closed() {
        let err = load_text("store:\n  driver: serverfs\n").expect_err("serverfs needs a path");
        assert!(matches!(err, ConfigError::MissingField { field, .. } if field == "store.path"));
    }

    #[test]
    fn a_misspelled_serverfs_is_an_unknown_driver_not_a_near_miss() {
        // Nothing guesses at what the operator meant: a driver name that is
        // not on the list is refused outright, so a typo can never quietly
        // start the server on a different persistence layer than intended.
        for typo in ["server-fs", "serverFS", "fs"] {
            let err = load_text(&format!("store:\n  driver: {typo}\n  path: /tmp/x\n"))
                .unwrap_err();
            assert!(
                matches!(err, ConfigError::UnknownDriver { .. }),
                "{typo} must not resolve to serverfs"
            );
        }
    }

    #[test]
    fn an_absent_events_section_uses_all_defaults() {
        let cfg = load_text("store:\n  driver: memory\nidentity:\n  driver: memory\n").expect("loads");
        assert_eq!(cfg.events, EventSettings::default(), "no events section means all defaults");
    }

    #[test]
    fn an_events_section_overrides_only_the_fields_it_names() {
        let cfg = load_text(
            "store:\n  driver: memory\nidentity:\n  driver: memory\nevents:\n  retention: 8\n  heartbeat_secs: 5\n",
        )
        .expect("loads");
        let defaults = EventSettings::default();
        assert_eq!(cfg.events.retention, 8, "a named field overrides");
        assert_eq!(cfg.events.heartbeat, Duration::from_secs(5));
        assert_eq!(cfg.events.buffer, defaults.buffer, "an unnamed field keeps its default");
    }

    #[test]
    fn a_malformed_events_section_fails_closed() {
        let err = load_text(
            "store:\n  driver: memory\nidentity:\n  driver: memory\nevents:\n  retention: not-a-number\n",
        )
        .expect_err("a bad events shape fails startup");
        assert!(matches!(err, ConfigError::Unparseable { .. }), "shape mismatch is a parse failure");
    }
}

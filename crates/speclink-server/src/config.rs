//! Startup configuration, loaded fail closed (design 決策三).
//!
//! The server starts from a single YAML file that declares the store driver,
//! the Project/Repo registry, and the bootstrap token → actor mapping. A file
//! that is missing, unparseable, declares an unknown driver, or is otherwise
//! shaped wrong makes startup fail with a reason pointing at the file — never a
//! partial-default start. This is bootstrap authentication only; a full account
//! system replaces the token section in a later knife.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The store drivers the server can bind. `sqlite` is the default persistent
/// option; `memory` exists only for test configurations.
pub const SUPPORTED_DRIVERS: &[&str] = &["sqlite", "memory"];

/// The validated server configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub store: StoreConfig,
    pub projects: Vec<ProjectConfig>,
    pub tokens: Vec<TokenConfig>,
}

/// The store backend to bind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreConfig {
    /// A single SQLite database file.
    Sqlite { path: PathBuf },
    /// The in-memory reference store — test configurations only.
    Memory,
}

/// One registered project: its URL key, display name, and the repos it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub key: String,
    pub name: String,
    pub repos: Vec<String>,
}

/// One bootstrap bearer token and the actor it authenticates as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenConfig {
    pub token: String,
    pub actor: ActorConfig,
}

/// The actor a token maps to: a stable id and the display identity recorded in
/// history and events.
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
    /// A required field for the chosen shape is absent.
    MissingField { path: String, field: String },
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
            ConfigError::MissingField { path, field } => {
                write!(f, "config file '{path}': missing required field '{field}'")
            }
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
    projects: Vec<RawProject>,
    #[serde(default)]
    tokens: Vec<RawToken>,
}

#[derive(Deserialize)]
struct RawStore {
    driver: String,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Deserialize)]
struct RawProject {
    key: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    repos: Vec<String>,
}

#[derive(Deserialize)]
struct RawToken {
    token: String,
    actor: RawActor,
}

#[derive(Deserialize)]
struct RawActor {
    #[serde(default)]
    id: Option<String>,
    display: String,
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

    let store = match raw.store.driver.as_str() {
        "sqlite" => {
            let path = raw.store.path.ok_or_else(|| ConfigError::MissingField {
                path: shown.clone(),
                field: "store.path".to_string(),
            })?;
            StoreConfig::Sqlite { path: PathBuf::from(path) }
        }
        "memory" => StoreConfig::Memory,
        other => {
            return Err(ConfigError::UnknownDriver {
                path: shown,
                driver: other.to_string(),
            })
        }
    };

    let projects = raw
        .projects
        .into_iter()
        .map(|p| ProjectConfig {
            name: p.name.unwrap_or_else(|| p.key.clone()),
            key: p.key,
            repos: p.repos,
        })
        .collect();

    let tokens = raw
        .tokens
        .into_iter()
        .map(|t| TokenConfig {
            token: t.token,
            actor: ActorConfig {
                id: t.actor.id.unwrap_or_else(|| t.actor.display.clone()),
                display: t.actor.display,
            },
        })
        .collect();

    Ok(ServerConfig { store, projects, tokens })
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
    fn a_memory_config_loads_with_registry_and_tokens() {
        let cfg = load_text(
            "store:\n  driver: memory\nprojects:\n  - key: demo\n    name: Demo\n    repos:\n      - backend\ntokens:\n  - token: secret\n    actor:\n      id: u_1\n      display: Alice <a@example.com>\n",
        )
        .expect("valid memory config loads");
        assert_eq!(cfg.store, StoreConfig::Memory);
        assert_eq!(cfg.projects[0].key, "demo");
        assert_eq!(cfg.projects[0].repos, ["backend"]);
        assert_eq!(cfg.tokens[0].token, "secret");
        assert_eq!(cfg.tokens[0].actor.display, "Alice <a@example.com>");
    }

    #[test]
    fn a_sqlite_config_carries_its_path() {
        let cfg = load_text("store:\n  driver: sqlite\n  path: /var/lib/speclink/store.db\n")
            .expect("valid sqlite config loads");
        assert_eq!(cfg.store, StoreConfig::Sqlite { path: PathBuf::from("/var/lib/speclink/store.db") });
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
        let err = load_text("store:\n  driver: postgres\n")
            .expect_err("an unsupported driver must fail startup");
        let shown = err.to_string();
        assert!(matches!(err, ConfigError::UnknownDriver { .. }));
        assert!(shown.contains("postgres"), "names the bad driver: {shown}");
        assert!(shown.contains("sqlite") && shown.contains("memory"), "lists supported drivers: {shown}");
    }

    #[test]
    fn sqlite_without_a_path_fails_closed() {
        let err = load_text("store:\n  driver: sqlite\n").expect_err("sqlite needs a path");
        assert!(matches!(err, ConfigError::MissingField { field, .. } if field == "store.path"));
    }
}

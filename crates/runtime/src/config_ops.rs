//! `speclink config show` / `config set` / `config edit` 的 runtime entry points.
//!
//! 對應 `config-rw` capability requirements:
//! - 「`speclink config show` SHALL read config.yaml and return `Versioned<Config>`」
//! - 「`speclink config set <key> <value>` SHALL patch config.yaml with optimistic concurrency」
//! - 「`speclink config edit` SHALL replace config.yaml contents via interactive editor or stdin」
//!
//! `ConfigStore::take_warnings()` 在每次 read/write 後被 drain，包成 `RuntimeWarning`
//! 加進 op JSON envelope `warnings` 陣列（design contract「warnings via take_warnings()」）。

#![allow(clippy::doc_markdown)]

use std::path::Path;

use speclink_provider::{
    Actor, Config, ConfigStore, ConfigValue, ConfigWarning, Etag, JsonPath, Versioned,
    WriteConfigRequest,
};
use speclink_provider_local::LocalConfigStore;

use crate::error::{RuntimeError, RuntimeWarning};
use crate::git::GitProbe;
use crate::paths::resolve_state_root;

/// Config I/O 的 entry。
pub struct ConfigOperations<G: GitProbe> {
    git: G,
}

impl<G: GitProbe> ConfigOperations<G> {
    /// 建立 handle；不接觸 disk。
    pub fn new(git: G) -> Self {
        Self { git }
    }

    fn build_store(&self, working_dir: &Path) -> Result<LocalConfigStore, RuntimeError> {
        let state_root = resolve_state_root::<G>(&self.git, working_dir)?;
        Ok(LocalConfigStore::new(working_dir.to_path_buf(), state_root))
    }

    /// 讀 config + 收集 warnings。
    pub fn read_config(
        &self,
        working_dir: &Path,
    ) -> Result<(Versioned<Config>, Vec<RuntimeWarning>), RuntimeError> {
        let store = self.build_store(working_dir)?;
        let v = store.read_config().map_err(map_provider_error)?;
        let warnings = convert_warnings(store.take_warnings());
        Ok((v, warnings))
    }

    /// Set 單 key（`Bool` only for A5）；CAS / unknown key / malformed 由 store 抛 error。
    pub fn set_config(
        &self,
        working_dir: &Path,
        key: JsonPath,
        value: ConfigValue,
        expected_etag: Option<Etag>,
        actor: Option<Actor>,
    ) -> Result<(Versioned<Config>, Vec<RuntimeWarning>), RuntimeError> {
        let store = self.build_store(working_dir)?;
        let v = store
            .write_config(WriteConfigRequest::Set {
                key,
                value,
                expected_etag,
                actor,
            })
            .map_err(map_provider_error)?;
        let warnings = convert_warnings(store.take_warnings());
        Ok((v, warnings))
    }

    /// Edit 整檔覆寫（`content` 由 CLI 從 stdin / editor buffer 取得）。
    pub fn edit_config(
        &self,
        working_dir: &Path,
        content: String,
        expected_etag: Option<Etag>,
        actor: Option<Actor>,
    ) -> Result<(Versioned<Config>, Vec<RuntimeWarning>), RuntimeError> {
        let store = self.build_store(working_dir)?;
        let v = store
            .write_config(WriteConfigRequest::Edit {
                content,
                expected_etag,
                actor,
            })
            .map_err(map_provider_error)?;
        let warnings = convert_warnings(store.take_warnings());
        Ok((v, warnings))
    }
}

fn convert_warnings(ws: Vec<ConfigWarning>) -> Vec<RuntimeWarning> {
    ws.into_iter()
        .map(|w| RuntimeWarning {
            code: w.code.to_string(),
            message: w.message,
            details: None,
        })
        .collect()
}

fn map_provider_error(e: speclink_provider::ProviderError) -> RuntimeError {
    use speclink_provider::ProviderError as PE;
    match e {
        PE::RequiresGit { context } => RuntimeError::RequiresGit { context },
        PE::AlreadyInitialized { path } => RuntimeError::AlreadyInitialized { path },
        PE::NotInitialized { path } => RuntimeError::NotInitialized { path },
        PE::ConfigNotFound { path } => RuntimeError::ConfigNotFound { path },
        PE::ConfigMalformed { reason } => RuntimeError::ConfigMalformed { reason },
        PE::ConfigKeyNotFound { key } => RuntimeError::ConfigKeyNotFound {
            key,
            hint: String::new(),
        },
        PE::StateEtagMismatch { expected, actual } => {
            RuntimeError::StateEtagMismatch { expected, actual }
        }
        PE::ConfigEditModeRequired => RuntimeError::ConfigEditModeRequired,
        PE::Internal(s) => RuntimeError::Internal(s),
        other => RuntimeError::Provider(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_provider::codes;

    #[test]
    fn convert_warnings_maps_static_str_to_owned_string() {
        let cw = ConfigWarning {
            code: codes::CONFIG_EXTERNAL_EDIT_DETECTED,
            message: "test".to_string(),
        };
        let rws = convert_warnings(vec![cw]);
        assert_eq!(rws.len(), 1);
        assert_eq!(rws[0].code, "config.external_edit_detected");
    }
}

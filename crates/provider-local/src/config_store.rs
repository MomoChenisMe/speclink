//! `LocalConfigStore` — filesystem-backed config.yaml I/O + state.db v5 cache。
//!
//! 對應 `config-rw` capability requirements:
//! - 「`speclink config show` SHALL read config.yaml and return `Versioned<Config>`」
//! - 「Read path SHALL fall back to defaults when config is missing or malformed」
//! - 「Read path SHALL detect external file edits and reconcile via audit log」
//!   (external-edit reconcile section 5 接，4.2 先落地 happy / fallback 兩條 path)
//! - 「`ConfigStore` trait SHALL be exposed via `Provider::config_store()`」
//!
//! Etag 公式：`v<version>.<sha256[:12]>`，對齊 design decision「Config etag 命名
//! 格式對齊 artifact etag」。Fallback (missing / malformed) 走特例 etag
//! `v0.malformed-fallback`、附 warning `config.malformed_using_defaults`、不寫
//! audit row（read path 對 malformed 永遠 non-mutating）。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};
use speclink_provider::{
    Actor, Config, ConfigStore, ConfigValue, ConfigWarning, Etag, JsonPath, JsonPathSegment,
    ProviderError, Versioned, WriteConfigRequest,
};

use crate::paths::ARTIFACT_ROOT;
use crate::state_db::{ConfigWriteArgs, StateDb, StateDbError};

/// `config.yaml` 缺失 / malformed 時的 fallback etag literal。
const FALLBACK_ETAG: &str = "v0.malformed-fallback";

/// Warning code：read path fallback 觸發。
pub(crate) const WARN_MALFORMED_FALLBACK: &str = "config.malformed_using_defaults";

/// Warning code：read path 偵測到 sha 與 config_state row 不一致時觸發。
pub(crate) const WARN_EXTERNAL_EDIT: &str = "config.external_edit_detected";

/// LocalProvider 的 `ConfigStore` 實作。
///
/// 持有 working_dir / state_root 兩條路徑；每次 read/write 開新 SQLite connection
/// 跑 migration（與其他 LocalStore 一致行為，靠 SQLite WAL 跨 connection 隔離）。
pub struct LocalConfigStore {
    working_dir: PathBuf,
    state_root: PathBuf,
    /// 累積 warning；`take_warnings()` 消費並清空。
    pending: Mutex<Vec<ConfigWarning>>,
}

impl LocalConfigStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(working_dir: PathBuf, state_root: PathBuf) -> Self {
        Self {
            working_dir,
            state_root,
            pending: Mutex::new(Vec::new()),
        }
    }

    /// Working tree root 路徑。
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// State root 路徑。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    /// `.speclink/config.yaml` 絕對路徑。
    fn config_path(&self) -> PathBuf {
        self.working_dir.join(ARTIFACT_ROOT).join("config.yaml")
    }

    /// 開 state.db connection + 確保 v5 schema。
    fn open_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let db_path = self.state_root.join("state.db");
        let db = StateDb::open(&db_path)
            .map_err(|e| ProviderError::Internal(format!("open state.db: {e}")))?;
        db.migrate(5)
            .map_err(|e| ProviderError::Internal(format!("migrate state.db: {e}")))?;
        Ok(db)
    }

    /// 把 warning 推到 pending buffer（read/write 路徑共用）。
    pub(crate) fn push_warning(&self, code: &'static str, message: impl Into<String>) {
        let mut guard = self.pending.lock().expect("pending mutex poisoned");
        guard.push(ConfigWarning {
            code,
            message: message.into(),
        });
    }

    /// 計算 etag literal `v<version>.<sha256[:12]>`。
    pub(crate) fn format_etag(version: i64, sha_hex: &str) -> String {
        debug_assert!(sha_hex.len() >= 12, "sha hex SHALL be at least 12 chars");
        format!("v{}.{}", version, &sha_hex[..12])
    }

    /// 讀 config.yaml + parse；缺失 / 解析失敗回 `Err(())`，由 caller 走 fallback。
    fn try_parse_config(path: &Path) -> Result<(Vec<u8>, Config), ()> {
        let bytes = fs::read(path).map_err(|_| ())?;
        let value: Config = serde_yaml::from_slice(&bytes).map_err(|_| ())?;
        Ok((bytes, value))
    }
}

/// 取得檔案最近修改時間（自 UNIX epoch 起算的奈秒）；錯誤回 0。
fn file_mtime_ns(path: &Path) -> i64 {
    use std::time::UNIX_EPOCH;
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
}

impl LocalConfigStore {
    /// 寫 path 共用骨架：載入 + CAS + atomic write + db tx commit。
    ///
    /// `validate_and_compose` 由 caller 提供，回傳「新 Config + 新 YAML bytes + mode +
    ///   keys_changed_json」。它可在 CAS 之前 raise `config.malformed` /
    ///   `config.key_not_found` 之類 error（read path 仍會跑、但結束前不會寫檔）。
    fn write_inner(
        &self,
        expected_etag: Option<&Etag>,
        actor: Option<&Actor>,
        validate_and_compose: impl FnOnce(
            &Config,
        ) -> Result<
            (Config, Vec<u8>, &'static str, String),
            ProviderError,
        >,
    ) -> Result<Versioned<Config>, ProviderError> {
        let path = self.config_path();
        // 1) 讀現檔；缺失 → config.not_found（write path 不走 fallback）。
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                return Err(ProviderError::ConfigNotFound {
                    path: path.display().to_string(),
                });
            }
        };
        let current: Config =
            serde_yaml::from_slice(&bytes).map_err(|e| ProviderError::ConfigMalformed {
                reason: format!("current config.yaml YAML parse failed: {e}"),
            })?;
        let file_sha = hex::encode(Sha256::digest(&bytes));

        // 2) 讀 config_state row + 組 current etag = v<row.version>.<file_sha[:12]>。
        let db = self.open_db()?;
        db.seed_config_state(&path)
            .map_err(|e| ProviderError::Internal(format!("seed config_state: {e}")))?;
        let row = db
            .read_config_state()
            .map_err(|e| ProviderError::Internal(format!("read config_state: {e}")))?;
        let current_etag = Self::format_etag(row.version, &file_sha);

        // 3) CAS（user-supplied or internal）。
        if let Some(expected) = expected_etag {
            if expected.as_str() != current_etag {
                return Err(ProviderError::StateEtagMismatch {
                    expected: Some(expected.as_str().to_string()),
                    actual: current_etag,
                });
            }
        }

        // 4) validate + compose new bytes（raise key_not_found / malformed 在這層）。
        let (new_config, new_bytes, mode, keys_changed_json) = validate_and_compose(&current)?;
        let new_sha = hex::encode(Sha256::digest(&new_bytes));
        let new_size = i64::try_from(new_bytes.len()).unwrap_or(i64::MAX);

        // 5) Atomic file write（tempfile in same parent + persist rename）。
        atomic_write(&path, &new_bytes)?;
        let new_mtime_ns = file_mtime_ns(&path);

        // 6) db tx：CAS UPDATE + INSERT audit。
        let etag_after = Self::format_etag(row.version + 1, &new_sha);
        let actor_json = actor
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| ProviderError::Internal(format!("serialize actor: {e}")))?;
        match db.commit_config_write(ConfigWriteArgs {
            expected_version: row.version,
            new_sha: &new_sha,
            new_size,
            new_mtime_ns,
            mode,
            keys_changed_json: &keys_changed_json,
            etag_before: &current_etag,
            etag_after: &etag_after,
            actor_json: actor_json.as_deref(),
        }) {
            Ok(_) => {}
            Err(StateDbError::CasConflict { current_version }) => {
                return Err(ProviderError::StateEtagMismatch {
                    expected: Some(current_etag),
                    actual: format!("v{current_version}.<concurrent>"),
                });
            }
            Err(e) => return Err(ProviderError::Internal(format!("config write tx: {e}"))),
        }

        Ok(Versioned {
            value: new_config,
            etag: Etag::from_literal(etag_after),
        })
    }

    fn write_set(
        &self,
        key: &JsonPath,
        value: ConfigValue,
        expected_etag: Option<&Etag>,
        actor: Option<&Actor>,
    ) -> Result<Versioned<Config>, ProviderError> {
        let segs = key.segments();
        // A5 只接受 rules.<bool review flag>。
        let known_field = match segs {
            [JsonPathSegment::Field(top), JsonPathSegment::Field(field)] if top == "rules" => {
                field.as_str()
            }
            _ => {
                return Err(ProviderError::ConfigKeyNotFound {
                    key: jsonpath_display(key),
                });
            }
        };

        let key_display = jsonpath_display(key);
        let bool_val = match &value {
            ConfigValue::Bool(b) => *b,
            _ => {
                // Type 不符（非 bool）→ malformed。
                return Err(ProviderError::ConfigMalformed {
                    reason: format!("key `{key_display}` requires a boolean value"),
                });
            }
        };

        self.write_inner(expected_etag, actor, move |current| {
            let mut next = current.clone();
            match known_field {
                "require_artifact_review" => next.rules.require_artifact_review = bool_val,
                "require_code_review" => next.rules.require_code_review = bool_val,
                _ => {
                    return Err(ProviderError::ConfigKeyNotFound {
                        key: key_display.clone(),
                    });
                }
            }
            let bytes = serde_yaml::to_string(&next)
                .map_err(|e| ProviderError::Internal(format!("serialize patched config: {e}")))?
                .into_bytes();
            let keys_json = format!("[{}]", serde_json::Value::String(key_display.clone()));
            Ok((next, bytes, "set", keys_json))
        })
    }

    fn write_edit(
        &self,
        content: String,
        expected_etag: Option<&Etag>,
        actor: Option<&Actor>,
    ) -> Result<Versioned<Config>, ProviderError> {
        // Pre-validate content (parse + type check) BEFORE entering write_inner，避免半套狀態。
        let parsed: Config =
            serde_yaml::from_str(&content).map_err(|e| ProviderError::ConfigMalformed {
                reason: format!("edit content YAML parse failed: {e}"),
            })?;
        self.write_inner(expected_etag, actor, move |_current| {
            let bytes = content.into_bytes();
            Ok((parsed, bytes, "edit", "[\"__edit__\"]".to_string()))
        })
    }
}

/// Atomic write helper：tempfile in same parent + `persist` (rename)。
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ProviderError> {
    let parent = path.parent().ok_or_else(|| {
        ProviderError::Internal(format!("config path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| {
        ProviderError::Internal(format!("create config parent {}: {e}", parent.display()))
    })?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| ProviderError::Internal(format!("create tempfile: {e}")))?;
    tmp.write_all(bytes)
        .map_err(|e| ProviderError::Internal(format!("write tempfile: {e}")))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| ProviderError::Internal(format!("fsync tempfile: {e}")))?;
    tmp.persist(path).map_err(|e| {
        ProviderError::Internal(format!("rename tempfile to {}: {e}", path.display()))
    })?;
    Ok(())
}

/// JSONPath subset 顯示為 dot-separated 字串（A5 不含 index）。
fn jsonpath_display(p: &JsonPath) -> String {
    p.segments()
        .iter()
        .map(|s| match s {
            JsonPathSegment::Field(name) => name.clone(),
            JsonPathSegment::Index(i) => format!("[{i}]"),
        })
        .collect::<Vec<_>>()
        .join(".")
}

impl ConfigStore for LocalConfigStore {
    fn read_config(&self) -> Result<Versioned<Config>, ProviderError> {
        // 1) Try parse；缺失 / malformed 走 fallback path（warning + defaults + 特例 etag）。
        let Ok((bytes, value)) = Self::try_parse_config(&self.config_path()) else {
            self.push_warning(
                WARN_MALFORMED_FALLBACK,
                "config.yaml missing or malformed; using built-in defaults",
            );
            return Ok(Versioned {
                value: Config::default(),
                etag: Etag::from_literal(FALLBACK_ETAG.to_string()),
            });
        };

        // 2) Happy path：算 sha、讀 config_state row、組 etag。
        let sha = hex::encode(Sha256::digest(&bytes));
        let db = self.open_db()?;
        // 確保 config_state row 存在（v4→v5 升版場景或從未 seed 過的 fresh state.db）。
        db.seed_config_state(&self.config_path())
            .map_err(|e| ProviderError::Internal(format!("seed config_state: {e}")))?;
        let row = db
            .read_config_state()
            .map_err(|e| ProviderError::Internal(format!("read config_state: {e}")))?;

        // 3) External-edit detection：sha 不一致 → reconcile（CAS bump version + audit row）。
        if row.content_sha256 != sha {
            let etag_before = Self::format_etag(row.version, &row.content_sha256);
            let etag_after = Self::format_etag(row.version + 1, &sha);
            let size = i64::try_from(bytes.len()).unwrap_or(i64::MAX);
            let mtime_ns = file_mtime_ns(&self.config_path());
            db.reconcile_external_edit(
                row.version,
                &sha,
                size,
                mtime_ns,
                &etag_before,
                &etag_after,
            )
            .map_err(|e| ProviderError::Internal(format!("reconcile external edit: {e}")))?;
            self.push_warning(
                WARN_EXTERNAL_EDIT,
                format!("config.yaml changed externally; reconciled to {etag_after}"),
            );
            return Ok(Versioned {
                value,
                etag: Etag::from_literal(etag_after),
            });
        }

        let etag = Etag::from_literal(Self::format_etag(row.version, &sha));
        Ok(Versioned { value, etag })
    }

    fn write_config(
        &self,
        request: WriteConfigRequest,
    ) -> Result<Versioned<Config>, ProviderError> {
        match request {
            WriteConfigRequest::Set {
                key,
                value,
                expected_etag,
                actor,
            } => self.write_set(&key, value, expected_etag.as_ref(), actor.as_ref()),
            WriteConfigRequest::Edit {
                content,
                expected_etag,
                actor,
            } => self.write_edit(content, expected_etag.as_ref(), actor.as_ref()),
        }
    }

    fn read_defaults(&self) -> Config {
        Config::default()
    }

    fn take_warnings(&self) -> Vec<ConfigWarning> {
        let mut guard = self.pending.lock().expect("pending mutex poisoned");
        std::mem::take(&mut *guard)
    }
}

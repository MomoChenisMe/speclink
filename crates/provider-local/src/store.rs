//! `LocalProjectStore` — 將 SpecLink project 綁定到本機檔案系統。
//!
//! 本層只負責「provider 層」的責任：`state.db` 開啟、migration、project row CRUD、
//! `link.yaml` 讀寫。Git check、`.gitignore` 寫入、prepare-then-commit 由
//! `speclink-runtime` 層串接（見 `crates/runtime/src/bootstrap.rs`）。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use speclink_provider::{
    InitOptions, LinkYaml, ProjectInfo, ProjectStatus, ProjectStore, ProviderError,
};
use uuid::Uuid;

use crate::link_yaml;
use crate::state_db::StateDb;

const ARTIFACT_ROOT: &str = ".speclink";

/// LocalProvider 的 project store 實作。
///
/// `working_dir` 是 git working tree root；`state_root` 是 `<git-common-dir>/speclink/`。
pub struct LocalProjectStore {
    working_dir: PathBuf,
    state_root: PathBuf,
}

impl LocalProjectStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(working_dir: PathBuf, state_root: PathBuf) -> Self {
        Self {
            working_dir,
            state_root,
        }
    }

    /// 工作樹路徑。
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// State root 路徑。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn state_db_path(&self) -> PathBuf {
        self.state_root.join("state.db")
    }

    fn open_state_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let db = StateDb::open(&self.state_db_path())
            .map_err(|e| ProviderError::Internal(format!("open state.db: {e}")))?;
        db.migrate(1)
            .map_err(|e| ProviderError::Internal(format!("migrate state.db: {e}")))?;
        Ok(db)
    }

    fn now_rfc3339() -> String {
        use time::OffsetDateTime;
        use time::format_description::well_known::Rfc3339;
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
    }
}

#[async_trait]
impl ProjectStore for LocalProjectStore {
    async fn init(&self, opts: InitOptions) -> Result<ProjectInfo, ProviderError> {
        let existing = link_yaml::read(&self.working_dir)?;
        if existing.is_some() && !opts.force {
            return Err(ProviderError::AlreadyInitialized {
                path: link_yaml::link_yaml_path(&self.working_dir)
                    .display()
                    .to_string(),
            });
        }

        let db = self.open_state_db()?;

        let project_id = match existing.as_ref() {
            Some(prev) if opts.force => prev.project_id.clone(),
            _ => Uuid::new_v4().to_string(),
        };
        let instance_id = Uuid::new_v4().to_string();
        let created_at = Self::now_rfc3339();
        let fingerprint = link_yaml::working_dir_fingerprint(&self.working_dir);

        // 簡化：本層只負責「init 一個新 project」場景；force 模式由 runtime 層處理。
        db.insert_project_row(
            &project_id,
            &instance_id,
            &self.working_dir.to_string_lossy(),
            &created_at,
        )
        .map_err(|e| ProviderError::Internal(format!("insert project row: {e}")))?;

        let link = LinkYaml {
            version: 1,
            project_id: project_id.clone(),
            instance_id,
            provider: "local".to_string(),
            created_at,
            working_dir_fingerprint: fingerprint,
        };
        link_yaml::write(&self.working_dir, &link)?;

        Ok(ProjectInfo {
            project_id,
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: relative_state_root_for_display(&self.working_dir, &self.state_root),
        })
    }

    async fn status(&self) -> Result<ProjectStatus, ProviderError> {
        let link =
            link_yaml::read(&self.working_dir)?.ok_or_else(|| ProviderError::NotInitialized {
                path: link_yaml::link_yaml_path(&self.working_dir)
                    .display()
                    .to_string(),
            })?;
        Ok(ProjectStatus {
            project_id: link.project_id,
            provider: link.provider,
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: relative_state_root_for_display(&self.working_dir, &self.state_root),
            git_head: String::new(), // 由 runtime 層補
            requires_git: true,
        })
    }

    async fn link(&self, project_id: &str) -> Result<ProjectInfo, ProviderError> {
        let db = self.open_state_db()?;
        let exists = db
            .has_project(project_id)
            .map_err(|e| ProviderError::Internal(format!("query project row: {e}")))?;
        if !exists {
            return Err(ProviderError::LinkTargetNotFound {
                project_id: project_id.to_string(),
            });
        }

        let instance_id = Uuid::new_v4().to_string();
        let created_at = Self::now_rfc3339();
        let fingerprint = link_yaml::working_dir_fingerprint(&self.working_dir);
        let link = LinkYaml {
            version: 1,
            project_id: project_id.to_string(),
            instance_id,
            provider: "local".to_string(),
            created_at,
            working_dir_fingerprint: fingerprint,
        };
        link_yaml::write(&self.working_dir, &link)?;
        Ok(ProjectInfo {
            project_id: project_id.to_string(),
            artifact_root: ARTIFACT_ROOT.to_string(),
            state_root: relative_state_root_for_display(&self.working_dir, &self.state_root),
        })
    }

    async fn unlink(&self) -> Result<(), ProviderError> {
        link_yaml::remove(&self.working_dir)
    }

    async fn get_link(&self) -> Result<Option<LinkYaml>, ProviderError> {
        link_yaml::read(&self.working_dir)
    }

    async fn save_link(&self, link: &LinkYaml) -> Result<(), ProviderError> {
        link_yaml::write(&self.working_dir, link)
    }
}

/// 把 absolute state root 轉成相對於 working_dir 的 POSIX 風格顯示路徑。
/// 若無法 strip prefix，回傳 absolute 顯示。
fn relative_state_root_for_display(working_dir: &Path, state_root: &Path) -> String {
    let candidate = match state_root.strip_prefix(working_dir) {
        Ok(rel) => rel.to_path_buf(),
        Err(_) => state_root.to_path_buf(),
    };
    candidate
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_link() -> LinkYaml {
        LinkYaml {
            version: 1,
            project_id: "11111111-1111-4111-8111-111111111111".to_string(),
            instance_id: "22222222-2222-4222-8222-222222222222".to_string(),
            provider: "local".to_string(),
            created_at: "2026-05-22T10:00:00Z".to_string(),
            working_dir_fingerprint: "a".repeat(64),
        }
    }

    fn make_store(tmp: &TempDir) -> LocalProjectStore {
        let working = tmp.path().to_path_buf();
        let state = working.join(".git").join("speclink");
        std::fs::create_dir_all(&state).expect("state dir");
        LocalProjectStore::new(working, state)
    }

    #[tokio::test]
    async fn save_link_writes_yaml_then_get_link_reads_it_back() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let link = sample_link();
        store.save_link(&link).await.expect("save link");

        let read_back = store
            .get_link()
            .await
            .expect("get link")
            .expect("link should exist after save");
        assert_eq!(read_back, link);

        let yaml_path = tmp.path().join(".speclink").join("link.yaml");
        let raw = std::fs::read_to_string(&yaml_path).expect("read raw yaml");
        assert!(raw.contains("version: 1"));
        assert!(raw.contains(&link.project_id));
    }

    #[tokio::test]
    async fn get_link_returns_none_when_link_yaml_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let res = store.get_link().await.expect("get link should not error");
        assert!(res.is_none(), "expected None on missing link.yaml");
    }

    #[tokio::test]
    async fn init_creates_state_db_and_link_yaml() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let info = store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect("init");
        assert!(!info.project_id.is_empty());
        assert!(tmp.path().join(".speclink").join("link.yaml").exists());
        assert!(
            tmp.path()
                .join(".git")
                .join("speclink")
                .join("state.db")
                .exists()
        );
    }

    #[tokio::test]
    async fn init_rejects_when_already_initialized_without_force() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect("first init");
        let err = store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect_err("second init should fail");
        assert_eq!(err.code(), speclink_provider::codes::ALREADY_INITIALIZED);
    }

    #[tokio::test]
    async fn link_to_known_project_id_succeeds() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        let info = store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect("init");
        // remove link.yaml to simulate fresh clone
        std::fs::remove_file(tmp.path().join(".speclink").join("link.yaml")).unwrap();
        let relinked = store.link(&info.project_id).await.expect("link");
        assert_eq!(relinked.project_id, info.project_id);
        assert!(tmp.path().join(".speclink").join("link.yaml").exists());
    }

    #[tokio::test]
    async fn link_to_unknown_project_id_returns_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        // initialise state.db so link can query it
        store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect("init");
        let err = store
            .link("00000000-0000-0000-0000-000000000000")
            .await
            .expect_err("link should fail");
        assert_eq!(err.code(), speclink_provider::codes::LINK_TARGET_NOT_FOUND);
    }

    #[tokio::test]
    async fn unlink_keeps_state_db_and_schemas_unchanged() {
        let tmp = TempDir::new().expect("tempdir");
        let store = make_store(&tmp);
        store
            .init(InitOptions {
                working_dir: tmp.path().to_path_buf(),
                force: false,
            })
            .await
            .expect("init");
        // Place a sentinel under .speclink/schemas to simulate seeded schema files.
        let schemas = tmp.path().join(".speclink").join("schemas");
        std::fs::create_dir_all(&schemas).expect("schemas dir");
        std::fs::write(schemas.join("spec.json"), "{}").expect("write schema");

        let state_db = tmp.path().join(".git").join("speclink").join("state.db");
        let before_sha = sha256_of(&state_db);

        store.unlink().await.expect("unlink");

        assert!(!tmp.path().join(".speclink").join("link.yaml").exists());
        assert!(state_db.exists());
        assert!(schemas.join("spec.json").exists());
        assert_eq!(before_sha, sha256_of(&state_db));
    }

    fn sha256_of(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let bytes = std::fs::read(path).expect("read file");
        let mut h = Sha256::new();
        h.update(&bytes);
        hex::encode(h.finalize())
    }
}

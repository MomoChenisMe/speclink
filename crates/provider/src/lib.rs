//! SpecLink Provider trait 與共用型別。
//!
//! 本 crate 定義 `ProjectStore` trait 與其輸入輸出型別，供 `speclink-runtime`
//! 對 trait 編程；具體實作位於 `speclink-provider-local`（與未來的
//! `speclink-provider-http`）。

#![allow(clippy::doc_markdown)]

pub mod error;
pub mod types;

pub use error::{ProviderError, codes};
pub use types::{InitOptions, LinkYaml, ProjectInfo, ProjectStatus};

/// SpecLink project 的 CRUD 介面。
///
/// 介面包含 6 個 method：`init` / `status` / `link` / `unlink` / `get_link` / `save_link`。
/// 任何新增 method 必須對應一份新的 capability spec。
#[async_trait::async_trait]
pub trait ProjectStore: Send + Sync {
    /// 在 `opts.working_dir` 建立新的 SpecLink project（artifact root + state root）。
    async fn init(&self, opts: InitOptions) -> Result<ProjectInfo, ProviderError>;

    /// 回報當前 working dir 的 project status。未 init 回 `NotInitialized`。
    async fn status(&self) -> Result<ProjectStatus, ProviderError>;

    /// 將當前 working dir 綁定到 state.db 內既存的 project row。
    async fn link(&self, project_id: &str) -> Result<ProjectInfo, ProviderError>;

    /// 移除 `.speclink/link.yaml`；不刪 state.db 與 `.speclink/schemas/`。
    async fn unlink(&self) -> Result<(), ProviderError>;

    /// 讀取 `.speclink/link.yaml`，未 init 時回 `Ok(None)`。
    async fn get_link(&self) -> Result<Option<LinkYaml>, ProviderError>;

    /// 寫入 `.speclink/link.yaml`（覆寫既有檔）。
    async fn save_link(&self, link: &LinkYaml) -> Result<(), ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_yaml_v1_schema_round_trips() {
        let yaml = "\
version: 1
project_id: 11111111-1111-4111-8111-111111111111
instance_id: 22222222-2222-4222-8222-222222222222
provider: local
created_at: 2026-05-22T10:00:00Z
working_dir_fingerprint: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
";
        let parsed: LinkYaml = serde_yaml::from_str(yaml).expect("parse v1 link.yaml");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.project_id, "11111111-1111-4111-8111-111111111111");
        assert_eq!(parsed.provider, "local");

        let serialized = serde_yaml::to_string(&parsed).expect("serialize link.yaml");
        assert!(
            serialized.contains("11111111-1111-4111-8111-111111111111"),
            "expected project_id to survive round-trip, got: {serialized}"
        );
        assert!(
            serialized.contains("version: 1"),
            "expected version: 1 in serialized YAML, got: {serialized}"
        );
    }

    #[test]
    fn project_status_has_six_fields() {
        let json = r#"{"project_id":"id","provider":"local","artifact_root":".speclink","state_root":".git/speclink","git_head":"deadbeef","requires_git":true}"#;
        let parsed: ProjectStatus = serde_json::from_str(json).expect("parse ProjectStatus");
        assert_eq!(parsed.provider, "local");
        assert!(parsed.requires_git);

        let serialized = serde_json::to_string(&parsed).expect("serialize ProjectStatus");
        for needle in [
            "project_id",
            "provider",
            "artifact_root",
            "state_root",
            "git_head",
            "requires_git",
        ] {
            assert!(
                serialized.contains(needle),
                "expected field {needle} in serialized ProjectStatus, got: {serialized}"
            );
        }
    }

    #[test]
    fn provider_error_codes_match_declared_namespace() {
        assert_eq!(codes::REQUIRES_GIT, "project.requires_git");
        assert_eq!(codes::ALREADY_INITIALIZED, "project.already_initialized");
        assert_eq!(codes::NOT_INITIALIZED, "project.not_initialized");
        assert_eq!(
            codes::LINK_TARGET_NOT_FOUND,
            "project.link_target_not_found"
        );

        assert_eq!(
            ProviderError::RequiresGit {
                context: "x".into()
            }
            .code(),
            codes::REQUIRES_GIT
        );
        assert_eq!(
            ProviderError::AlreadyInitialized { path: "p".into() }.code(),
            codes::ALREADY_INITIALIZED
        );
        assert_eq!(
            ProviderError::NotInitialized { path: "p".into() }.code(),
            codes::NOT_INITIALIZED
        );
        assert_eq!(
            ProviderError::LinkTargetNotFound {
                project_id: "u".into()
            }
            .code(),
            codes::LINK_TARGET_NOT_FOUND
        );
    }

    /// Trait shape check (compile-time): `DummyStore` must implement every method.
    /// 如果未來 trait 新增 method，這支 dummy impl 會 build fail，提醒同步更新。
    #[allow(dead_code)]
    struct DummyStore;

    #[async_trait::async_trait]
    impl ProjectStore for DummyStore {
        async fn init(&self, _opts: InitOptions) -> Result<ProjectInfo, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn status(&self) -> Result<ProjectStatus, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn link(&self, _project_id: &str) -> Result<ProjectInfo, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn unlink(&self) -> Result<(), ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn get_link(&self) -> Result<Option<LinkYaml>, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn save_link(&self, _link: &LinkYaml) -> Result<(), ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
    }
}

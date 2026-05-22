//! SpecLink Provider trait 與共用型別。
//!
//! 本 crate 定義 `ProjectStore` / `ChangeStore` / `ArtifactStore` trait 與其輸入輸出型別，
//! 供 `speclink-runtime` 對 trait 編程；具體實作位於 `speclink-provider-local`。

#![allow(clippy::doc_markdown)]

pub mod error;
pub mod types;

pub use error::{ProviderError, codes};
pub use types::{
    ArtifactKind, ChangeRow, Etag, EtagError, ExpectedEtag, IdError, InitOptions, LinkYaml,
    ProjectInfo, ProjectStatus, Versioned, validate_kebab_id,
};

/// SpecLink project 的 CRUD 介面。
#[async_trait::async_trait]
pub trait ProjectStore: Send + Sync {
    async fn init(&self, opts: InitOptions) -> Result<ProjectInfo, ProviderError>;
    async fn status(&self) -> Result<ProjectStatus, ProviderError>;
    async fn link(&self, project_id: &str) -> Result<ProjectInfo, ProviderError>;
    async fn unlink(&self) -> Result<(), ProviderError>;
    async fn get_link(&self) -> Result<Option<LinkYaml>, ProviderError>;
    async fn save_link(&self, link: &LinkYaml) -> Result<(), ProviderError>;
}

/// `change` 表的 CRUD 介面（slice A：4 個 method）。
#[async_trait::async_trait]
pub trait ChangeStore: Send + Sync {
    /// 建立新 change：寫 `change` row + 建立 `.speclink/changes/<name>/` 目錄。
    async fn create_change(&self, name: &str, schema_id: &str) -> Result<ChangeRow, ProviderError>;

    /// 列舉所有 change（依 `updated_at` desc 排序）。
    async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError>;

    /// 取得單一 change row；找不到回 [`ProviderError::ChangeNotFound`]。
    async fn get_change(&self, name: &str) -> Result<ChangeRow, ProviderError>;

    /// 刪除 change row + 目錄。
    async fn delete_change(&self, name: &str) -> Result<(), ProviderError>;
}

/// Artifact 讀寫介面。
#[async_trait::async_trait]
pub trait ArtifactStore: Send + Sync {
    /// 讀取 artifact + 即時算出 sha256 Etag。
    async fn read_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<Versioned<Vec<u8>>, ProviderError>;

    /// 寫入 artifact，套用 etag 並發控制；atomic rename。
    async fn write_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
        bytes: &[u8],
        expected: ExpectedEtag,
    ) -> Result<Versioned<()>, ProviderError>;

    /// 列舉某 change 下所有 spec 的 capability id（filesystem-backed）。
    async fn list_spec_capabilities(&self, change: &str) -> Result<Vec<String>, ProviderError>;
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
        assert!(serialized.contains("11111111-1111-4111-8111-111111111111"));
        assert!(serialized.contains("version: 1"));
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
            assert!(serialized.contains(needle));
        }
    }

    #[test]
    fn provider_error_codes_match_declared_namespace() {
        // bootstrap-slice codes
        assert_eq!(codes::REQUIRES_GIT, "project.requires_git");
        assert_eq!(codes::ALREADY_INITIALIZED, "project.already_initialized");
        assert_eq!(codes::NOT_INITIALIZED, "project.not_initialized");
        assert_eq!(
            codes::LINK_TARGET_NOT_FOUND,
            "project.link_target_not_found"
        );
        // slice-A new codes
        assert_eq!(codes::CHANGE_NOT_FOUND, "change.not_found");
        assert_eq!(codes::CHANGE_DUPLICATE_NAME, "change.duplicate_name");
        assert_eq!(codes::CHANGE_INVALID_NAME, "change.invalid_name");
        assert_eq!(codes::ARTIFACT_KIND_INVALID, "artifact.kind_invalid");
        assert_eq!(
            codes::ARTIFACT_CAPABILITY_REQUIRED,
            "artifact.capability_required"
        );
        assert_eq!(codes::ARTIFACT_NOT_FOUND, "artifact.not_found");
        assert_eq!(
            codes::ARTIFACT_VERSION_CONFLICT,
            "artifact.version_conflict"
        );

        // bootstrap variants
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

        // slice-A variants
        assert_eq!(
            ProviderError::ChangeNotFound { name: "x".into() }.code(),
            codes::CHANGE_NOT_FOUND
        );
        assert_eq!(
            ProviderError::ChangeDuplicateName { name: "x".into() }.code(),
            codes::CHANGE_DUPLICATE_NAME
        );
        assert_eq!(
            ProviderError::ChangeInvalidName {
                name: "x".into(),
                reason: "y".into()
            }
            .code(),
            codes::CHANGE_INVALID_NAME
        );
        assert_eq!(
            ProviderError::ArtifactKindInvalid {
                kind: "summary".into()
            }
            .code(),
            codes::ARTIFACT_KIND_INVALID
        );
        assert_eq!(
            ProviderError::ArtifactCapabilityRequired.code(),
            codes::ARTIFACT_CAPABILITY_REQUIRED
        );
        assert_eq!(
            ProviderError::ArtifactNotFound { path: "p".into() }.code(),
            codes::ARTIFACT_NOT_FOUND
        );
        assert_eq!(
            ProviderError::ArtifactVersionConflict {
                expected: None,
                actual: Etag::from_bytes(b""),
            }
            .code(),
            codes::ARTIFACT_VERSION_CONFLICT
        );
    }

    #[test]
    fn provider_error_retryable_only_for_version_conflict() {
        // bootstrap variants 全部 non-retryable
        assert!(
            !ProviderError::RequiresGit {
                context: "x".into()
            }
            .retryable()
        );
        assert!(!ProviderError::AlreadyInitialized { path: "p".into() }.retryable());
        assert!(!ProviderError::NotInitialized { path: "p".into() }.retryable());
        assert!(
            !ProviderError::LinkTargetNotFound {
                project_id: "u".into()
            }
            .retryable()
        );

        // slice-A：只有 ArtifactVersionConflict 可重試
        assert!(!ProviderError::ChangeNotFound { name: "x".into() }.retryable());
        assert!(!ProviderError::ChangeDuplicateName { name: "x".into() }.retryable());
        assert!(
            !ProviderError::ChangeInvalidName {
                name: "x".into(),
                reason: "y".into()
            }
            .retryable()
        );
        assert!(
            !ProviderError::ArtifactKindInvalid {
                kind: "summary".into()
            }
            .retryable()
        );
        assert!(!ProviderError::ArtifactCapabilityRequired.retryable());
        assert!(!ProviderError::ArtifactNotFound { path: "p".into() }.retryable());
        assert!(
            ProviderError::ArtifactVersionConflict {
                expected: None,
                actual: Etag::from_bytes(b""),
            }
            .retryable()
        );
        assert!(!ProviderError::Internal("x".into()).retryable());
    }

    /// Trait shape check (compile-time): three dummy types each implementing one trait.
    /// 若未來 trait 新增 method，dummy 會 build fail，提醒同步更新。
    #[allow(dead_code)]
    struct DummyProjectStore;

    #[async_trait::async_trait]
    impl ProjectStore for DummyProjectStore {
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

    #[allow(dead_code)]
    struct DummyChangeStore;

    #[async_trait::async_trait]
    impl ChangeStore for DummyChangeStore {
        async fn create_change(
            &self,
            _name: &str,
            _schema_id: &str,
        ) -> Result<ChangeRow, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn list_changes(&self) -> Result<Vec<ChangeRow>, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn get_change(&self, _name: &str) -> Result<ChangeRow, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn delete_change(&self, _name: &str) -> Result<(), ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
    }

    #[allow(dead_code)]
    struct DummyArtifactStore;

    #[async_trait::async_trait]
    impl ArtifactStore for DummyArtifactStore {
        async fn read_artifact(
            &self,
            _change: &str,
            _kind: ArtifactKind,
            _capability: Option<&str>,
        ) -> Result<Versioned<Vec<u8>>, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn write_artifact(
            &self,
            _change: &str,
            _kind: ArtifactKind,
            _capability: Option<&str>,
            _bytes: &[u8],
            _expected: ExpectedEtag,
        ) -> Result<Versioned<()>, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
        async fn list_spec_capabilities(
            &self,
            _change: &str,
        ) -> Result<Vec<String>, ProviderError> {
            Err(ProviderError::Internal("dummy".into()))
        }
    }
}

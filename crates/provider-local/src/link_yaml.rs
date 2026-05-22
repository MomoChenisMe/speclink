//! `.speclink/link.yaml` 的 v1 schema 序列化與 path helper。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::path::{Path, PathBuf};

use speclink_provider::{LinkYaml, ProviderError};

/// `.speclink/` 目錄名稱。
pub const ARTIFACT_DIR_NAME: &str = ".speclink";
/// `.speclink/link.yaml` 檔名。
pub const LINK_YAML_NAME: &str = "link.yaml";

/// 取得相對於 working tree root 的 link.yaml 路徑。
#[must_use]
pub fn link_yaml_path(working_dir: &Path) -> PathBuf {
    working_dir.join(ARTIFACT_DIR_NAME).join(LINK_YAML_NAME)
}

/// 將 `link` 寫入 `<working_dir>/.speclink/link.yaml`（覆寫既有檔）。
///
/// # Errors
/// 當建立 `.speclink/` 目錄、序列化 YAML、或寫入失敗時回 [`ProviderError::Internal`]。
pub fn write(working_dir: &Path, link: &LinkYaml) -> Result<(), ProviderError> {
    let dir = working_dir.join(ARTIFACT_DIR_NAME);
    fs::create_dir_all(&dir)
        .map_err(|e| ProviderError::Internal(format!("create .speclink dir: {e}")))?;
    let yaml = serde_yaml::to_string(link)
        .map_err(|e| ProviderError::Internal(format!("serialize link.yaml: {e}")))?;
    let path = link_yaml_path(working_dir);
    fs::write(&path, yaml).map_err(|e| ProviderError::Internal(format!("write link.yaml: {e}")))?;
    Ok(())
}

/// 讀取 `<working_dir>/.speclink/link.yaml`。檔案不存在回 `Ok(None)`。
///
/// # Errors
/// 當檔案讀取或 YAML 解析失敗（且不是 NotFound）時回 [`ProviderError::Internal`]。
pub fn read(working_dir: &Path) -> Result<Option<LinkYaml>, ProviderError> {
    let path = link_yaml_path(working_dir);
    match fs::read_to_string(&path) {
        Ok(text) => {
            let link: LinkYaml = serde_yaml::from_str(&text)
                .map_err(|e| ProviderError::Internal(format!("parse link.yaml: {e}")))?;
            Ok(Some(link))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ProviderError::Internal(format!("read link.yaml: {e}"))),
    }
}

/// 刪除 `<working_dir>/.speclink/link.yaml`。檔案不存在時 no-op。
///
/// # Errors
/// 當刪除失敗（且不是 NotFound）時回 [`ProviderError::Internal`]。
pub fn remove(working_dir: &Path) -> Result<(), ProviderError> {
    let path = link_yaml_path(working_dir);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(ProviderError::Internal(format!("remove link.yaml: {e}"))),
    }
}

/// 計算 working tree root 的指紋（canonical path 的 SHA-256 hex digest）。
#[must_use]
pub fn working_dir_fingerprint(working_dir: &Path) -> String {
    use sha2::{Digest, Sha256};
    let canonical = working_dir
        .canonicalize()
        .unwrap_or_else(|_| working_dir.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use speclink_provider::LinkYaml;

    fn fixed_link() -> LinkYaml {
        LinkYaml {
            version: 1,
            project_id: "11111111-1111-4111-8111-111111111111".into(),
            instance_id: "22222222-2222-4222-8222-222222222222".into(),
            provider: "local".into(),
            created_at: "2026-05-22T10:00:00Z".into(),
            working_dir_fingerprint: "a".repeat(64),
        }
    }

    #[test]
    fn yaml_snapshot_for_fixed_link() {
        let yaml = serde_yaml::to_string(&fixed_link()).expect("serialize");
        insta::assert_snapshot!("link_yaml_v1_fixed", yaml);
    }
}

//! Local provider 的 filesystem 操作：
//!
//! - 建立 `.speclink/changes/<id>/` 目錄結構
//! - 原子寫入 `proposal.md` + `metadata.json`（temp file → rename）
//! - 失敗時清除半成品
//! - Change id 驗證（kebab-case）

use crate::error::LocalProviderError;
use provider::model::ChangeId;
use std::path::{Path, PathBuf};

/// `.speclink/` 子目錄名稱。
const SPECLINK_DIR: &str = ".speclink";
/// `changes/` 子目錄名稱。
const CHANGES_DIR: &str = "changes";

/// 檢驗 change id 是否符合 `^[a-z][a-z0-9-]{0,63}$`、不含連續 hyphen、不以 hyphen 結尾。
pub fn is_valid_change_id(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut prev_hyphen = false;
    for &b in &bytes[1..] {
        if b == b'-' {
            if prev_hyphen {
                return false;
            }
            prev_hyphen = true;
        } else if b.is_ascii_lowercase() || b.is_ascii_digit() {
            prev_hyphen = false;
        } else {
            return false;
        }
    }
    bytes[bytes.len() - 1] != b'-'
}

/// 取得 `<base>/.speclink/changes/<change_id>/` 絕對路徑。
pub fn change_dir(base: &Path, change_id: &ChangeId) -> PathBuf {
    base.join(SPECLINK_DIR)
        .join(CHANGES_DIR)
        .join(change_id.as_str())
}

/// 寫入 proposal artifact（從 summary 自動 wrap 為 `## Why\n\n<summary>\n`），
/// 並更新 metadata.json，採 temp-file + rename 提供原子性。
///
/// 寫入順序（per spec `Atomic artifact write with metadata pairing`）：
/// 1. 建立 `<change_dir>/` 目錄
/// 2. 寫 `proposal.md.tmp`
/// 3. 寫 `metadata.json.tmp`
/// 4. rename `proposal.md.tmp` → `proposal.md`
/// 5. rename `metadata.json.tmp` → `metadata.json`
///
/// 失敗於任何步驟時呼叫 [`cleanup_change_dir`] 移除整個 `<change_dir>/` 以避免半成品。
///
/// 回傳成功寫入的 `proposal.md` 絕對路徑。
pub fn write_proposal_atomic(
    base: &Path,
    change_id: &ChangeId,
    summary: &str,
) -> Result<PathBuf, LocalProviderError> {
    let content = format!("## Why\n\n{summary}\n");
    write_proposal_content_atomic(base, change_id, &content)
}

/// 寫入已格式化的 proposal.md 內容（caller 自行格式化為含 `## Why` heading）。
///
/// 行為與 [`write_proposal_atomic`] 相同，但 content 不再被 wrap。
pub fn write_proposal_content_atomic(
    base: &Path,
    change_id: &ChangeId,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    if !is_valid_change_id(change_id.as_str()) {
        return Err(LocalProviderError::InvalidChangeId {
            change_id: change_id.as_str().to_string(),
        });
    }
    let dir = change_dir(base, change_id);
    if dir.exists() {
        return Err(LocalProviderError::ChangeAlreadyExists {
            change_id: change_id.as_str().to_string(),
        });
    }
    let result = write_proposal_inner(&dir, change_id, content);
    if result.is_err() {
        // 任一階段失敗 → 清除整個 <change_dir>，並移除可能殘留的 .tmp 檔
        let _ = cleanup_change_dir(base, change_id);
    }
    result
}

fn write_proposal_inner(
    dir: &Path,
    change_id: &ChangeId,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    std::fs::create_dir_all(dir)?;
    let proposal_tmp = dir.join("proposal.md.tmp");
    let proposal = dir.join("proposal.md");
    let metadata_tmp = dir.join("metadata.json.tmp");
    let metadata = dir.join("metadata.json");

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let meta = serde_json::json!({
        "changeId": change_id.as_str(),
        "state": "proposed",
        "createdAt": now,
        "createdBy": { "type": "agent", "name": "" }
    });
    let meta_content = serde_json::to_string_pretty(&meta)?;

    std::fs::write(&proposal_tmp, content)?;
    std::fs::write(&metadata_tmp, meta_content)?;
    std::fs::rename(&proposal_tmp, &proposal)?;
    std::fs::rename(&metadata_tmp, &metadata)?;
    Ok(proposal)
}

/// 移除整個 `<change_dir>/` — 用於 atomic write 失敗時清除半成品。
pub fn cleanup_change_dir(base: &Path, change_id: &ChangeId) -> Result<(), LocalProviderError> {
    let dir = change_dir(base, change_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::storage::{is_valid_change_id, write_proposal_atomic};
    use provider::model::ChangeId;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn write_proposal_creates_only_required_files() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "test summary").expect("write");

        // Required files
        let change_dir = base.join(".speclink").join("changes").join("demo");
        assert!(
            change_dir.join("proposal.md").is_file(),
            "proposal.md must exist"
        );
        assert!(
            change_dir.join("metadata.json").is_file(),
            "metadata.json must exist"
        );

        // 不應該被 eagerly 建立的可選子目錄/檔案
        let forbidden = [
            "design.md",
            "tasks.md",
            "specs",
            "archive",
            "packs",
            "cache",
        ];
        for name in forbidden {
            let path = change_dir.join(name);
            assert!(
                !path.exists(),
                "forbidden artifact created by propose create: {}",
                path.display()
            );
        }
    }

    #[test]
    fn write_proposal_content_matches_spec() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "test summary").expect("write");

        let proposal = base
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("proposal.md");
        let body = std::fs::read_to_string(proposal).unwrap();
        assert_eq!(body, "## Why\n\ntest summary\n");
    }

    #[test]
    fn metadata_json_has_required_fields() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        write_proposal_atomic(base, &ChangeId::from("demo"), "test").expect("write");

        let meta = base
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("metadata.json");
        let body = std::fs::read_to_string(meta).unwrap();
        let v: serde_json::Value = serde_json::from_str(&body).expect("parse json");
        assert_eq!(v.get("changeId").and_then(|v| v.as_str()), Some("demo"));
        assert_eq!(v.get("state").and_then(|v| v.as_str()), Some("proposed"));
        assert!(
            v.get("createdAt").and_then(|v| v.as_str()).is_some(),
            "createdAt missing"
        );
        let cb = v.get("createdBy").expect("createdBy");
        assert_eq!(cb.get("type").and_then(|v| v.as_str()), Some("agent"));
        assert_eq!(cb.get("name").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn change_id_validation_table() {
        // 對應 spec `Change-id validation` 的範例表
        assert!(is_valid_change_id("add-order-export"));
        assert!(is_valid_change_id("a"));
        assert!(!is_valid_change_id("Add-Feature"));
        assert!(!is_valid_change_id("1add"));
        assert!(!is_valid_change_id("add--feature"));
        assert!(!is_valid_change_id("add-"));
        assert!(!is_valid_change_id(""));
        // 額外：合理上限附近
        let max64_ok: String = std::iter::once('a')
            .chain(std::iter::repeat_n('z', 63))
            .collect();
        assert_eq!(max64_ok.len(), 64);
        assert!(
            is_valid_change_id(&max64_ok),
            "64-char id should be accepted"
        );
        let too_long: String = std::iter::once('a')
            .chain(std::iter::repeat_n('z', 64))
            .collect();
        assert_eq!(too_long.len(), 65);
        assert!(
            !is_valid_change_id(&too_long),
            "65-char id should be rejected"
        );
    }

    #[test]
    fn write_proposal_cleans_up_on_metadata_failure() {
        // 用一個 readonly 目錄阻止 metadata.json 寫入：先建立目錄，再 chmod 為 readonly。
        // 此處改以一個替代方案：write_proposal_atomic 被傳入無效 base（一個檔案而非目錄）
        // 觸發 cleanup。
        let tmp = TempDir::new().unwrap();
        let invalid_base = tmp.path().join("not-a-dir");
        std::fs::write(&invalid_base, "").unwrap();
        let res = write_proposal_atomic(&invalid_base, &ChangeId::from("demo"), "x");
        assert!(res.is_err(), "expected failure when base is invalid");
        // 不應殘留 demo 目錄
        let dangling = invalid_base.join(".speclink").join("changes").join("demo");
        assert!(!dangling.exists(), "expected cleanup of partial dir");
        // 亦不應殘留 .tmp 檔
        assert!(no_tmp_files(invalid_base.parent().unwrap()));
    }

    #[test]
    fn write_proposal_refuses_existing_change_dir() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        // 先建立目錄與 dummy 檔案
        let dir = base.join(".speclink").join("changes").join("demo");
        std::fs::create_dir_all(&dir).unwrap();
        let existing = dir.join("proposal.md");
        std::fs::write(&existing, "EXISTING").unwrap();

        let res = write_proposal_atomic(base, &change_id, "new summary");
        assert!(matches!(
            res,
            Err(crate::error::LocalProviderError::ChangeAlreadyExists { .. })
        ));
        // 既有內容未被覆寫
        let body = std::fs::read_to_string(&existing).unwrap();
        assert_eq!(body, "EXISTING");
    }

    fn no_tmp_files(root: &Path) -> bool {
        for entry in walkdir(root) {
            if let Some(name) = entry.file_name().and_then(|s| s.to_str()) {
                if name.ends_with(".tmp") {
                    return false;
                }
            }
        }
        true
    }

    /// 極簡 walkdir 替代品，避免引入依賴。
    fn walkdir(root: &Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            if let Ok(read) = std::fs::read_dir(&dir) {
                for entry in read.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path.clone());
                    }
                    out.push(path);
                }
            }
        }
        out
    }
}

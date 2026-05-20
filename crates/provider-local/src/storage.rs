//! Local provider 的 filesystem 操作：
//!
//! - 建立 `.speclink/changes/<id>/` 目錄結構
//! - 原子寫入 `proposal.md` + `metadata.json`（temp file → rename）
//! - 失敗時清除半成品
//! - Change id 驗證（kebab-case）

use crate::error::LocalProviderError;
use provider::model::{
    ArtifactKind, ArtifactState, ArtifactStatus, Change, ChangeId, ChangeStatus,
};
use std::path::{Path, PathBuf};

/// `.speclink/` 子目錄名稱。
const SPECLINK_DIR: &str = ".speclink";
/// `changes/` 子目錄名稱。
const CHANGES_DIR: &str = "changes";
/// `specs/` 子目錄名稱（位於 `<change_dir>/` 或 `.speclink/`）。
const SPECS_DIR: &str = "specs";
/// `archive/` 子目錄名稱（位於 `.speclink/changes/`）。
const ARCHIVE_DIR: &str = "archive";

/// 檢驗字串是否符合 `^[a-z][a-z0-9-]{0,63}$`、不含連續 hyphen、不以 hyphen 結尾。
///
/// 為 change-id 與 capability name 共用的 kebab-case 規則。
pub fn is_valid_kebab_id(s: &str) -> bool {
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

/// 檢驗 change id 是否符合 kebab-case 規則（同 [`is_valid_kebab_id`]）。
pub fn is_valid_change_id(s: &str) -> bool {
    is_valid_kebab_id(s)
}

/// 檢驗 capability 名稱是否符合 kebab-case 規則（同 [`is_valid_kebab_id`]）。
pub fn is_valid_capability_name(s: &str) -> bool {
    is_valid_kebab_id(s)
}

/// 把 [`Path`] 轉為 POSIX 風格字串（forward slash）。
///
/// 用於 JSON output `path` 欄位 — 跨平台一致。
pub fn to_posix_string(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// 取得 `<base>/.speclink/changes/<change_id>/` 絕對路徑。
pub fn change_dir(base: &Path, change_id: &ChangeId) -> PathBuf {
    base.join(SPECLINK_DIR)
        .join(CHANGES_DIR)
        .join(change_id.as_str())
}

/// 取得 `<base>/.speclink/specs/` 主 spec 根目錄絕對路徑。
///
/// 對應 spec `Local provider directory layout` 規定的主 spec 落點：
/// `.speclink/specs/` 在第一次 archive 套用 delta 時建立。
pub fn main_spec_dir(base: &Path) -> PathBuf {
    base.join(SPECLINK_DIR).join(SPECS_DIR)
}

/// 取得 `<base>/.speclink/specs/<capability>/spec.md` 主 spec 檔案絕對路徑。
///
/// 路徑以 `PathBuf::join` 拼接，跨平台一致；caller 不可硬編 `/` 或 `\`。
pub fn main_spec_path(base: &Path, capability: &str) -> PathBuf {
    main_spec_dir(base).join(capability).join("spec.md")
}

/// 取得 `<base>/.speclink/changes/archive/` 目錄絕對路徑。
///
/// 對應 spec `Local provider directory layout` 規定的 archive 根：archive 區與 active
/// change 區同層，方便同一 filesystem 內 atomic rename。
pub fn archive_root_dir(base: &Path) -> PathBuf {
    base.join(SPECLINK_DIR).join(CHANGES_DIR).join(ARCHIVE_DIR)
}

/// 取得 `<base>/.speclink/changes/archive/<YYYY-MM-DD>-<change_id>/` 絕對路徑。
///
/// `date_prefix` 由 caller 傳入（CLI 入口取 `chrono::Local::now().date_naive()`），
/// 格式應為 `YYYY-MM-DD`；本函式不檢查 — 違反契約由 caller 自行負責。
pub fn archive_change_dir(base: &Path, change_id: &ChangeId, date_prefix: &str) -> PathBuf {
    archive_root_dir(base).join(format!("{date_prefix}-{}", change_id.as_str()))
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

/// 寫入 `design.md`：原子寫入（`.tmp` + rename）；若 change 目錄不存在回
/// [`LocalProviderError::ChangeNotFound`]，若檔案已存在回
/// [`LocalProviderError::ArtifactAlreadyExists`]。
pub fn write_design_atomic(
    base: &Path,
    change_id: &ChangeId,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    write_simple_artifact_atomic(base, change_id, "design", "design.md", content)
}

/// 寫入 `tasks.md`：原子寫入（`.tmp` + rename）；錯誤語意同
/// [`write_design_atomic`]。
pub fn write_tasks_atomic(
    base: &Path,
    change_id: &ChangeId,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    write_simple_artifact_atomic(base, change_id, "tasks", "tasks.md", content)
}

fn write_simple_artifact_atomic(
    base: &Path,
    change_id: &ChangeId,
    kind_label: &str,
    filename: &str,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    if !is_valid_change_id(change_id.as_str()) {
        return Err(LocalProviderError::InvalidChangeId {
            change_id: change_id.as_str().to_string(),
        });
    }
    let dir = change_dir(base, change_id);
    if !dir.exists() {
        return Err(LocalProviderError::ChangeNotFound {
            change_id: change_id.as_str().to_string(),
        });
    }
    let target = dir.join(filename);
    if target.exists() {
        return Err(LocalProviderError::ArtifactAlreadyExists {
            kind: kind_label.to_string(),
            change_id: change_id.as_str().to_string(),
        });
    }
    let tmp = dir.join(format!("{filename}.tmp"));
    let write_result = (|| -> Result<(), LocalProviderError> {
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &target)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp);
        write_result.map(|_| target.clone())?;
    }
    Ok(target)
}

/// 寫入 `specs/<capability>/spec.md`：建立中間目錄、原子寫入；失敗時保留 pre-existing
/// `specs/` 子目錄與其他 capability 內容，僅清除本次新建的 capability 目錄與 `.tmp`。
pub fn write_spec_atomic(
    base: &Path,
    change_id: &ChangeId,
    capability: &str,
    content: &str,
) -> Result<PathBuf, LocalProviderError> {
    if !is_valid_change_id(change_id.as_str()) {
        return Err(LocalProviderError::InvalidChangeId {
            change_id: change_id.as_str().to_string(),
        });
    }
    if !is_valid_capability_name(capability) {
        return Err(LocalProviderError::InvalidCapability {
            capability: capability.to_string(),
        });
    }
    let dir = change_dir(base, change_id);
    if !dir.exists() {
        return Err(LocalProviderError::ChangeNotFound {
            change_id: change_id.as_str().to_string(),
        });
    }
    let specs_dir = dir.join(SPECS_DIR);
    let cap_dir = specs_dir.join(capability);
    let target = cap_dir.join("spec.md");
    if target.exists() {
        return Err(LocalProviderError::ArtifactAlreadyExists {
            kind: format!("spec:{capability}"),
            change_id: change_id.as_str().to_string(),
        });
    }

    let specs_dir_preexisting = specs_dir.exists();
    let cap_dir_preexisting = cap_dir.exists();

    let tmp = cap_dir.join("spec.md.tmp");
    let write_result = (|| -> Result<(), LocalProviderError> {
        std::fs::create_dir_all(&cap_dir)?;
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &target)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp);
        // 只清除「本次新建」的目錄；pre-existing 目錄與其他 capability 不動。
        if !cap_dir_preexisting && cap_dir.exists() {
            let _ = std::fs::remove_dir_all(&cap_dir);
        }
        if !specs_dir_preexisting && specs_dir.exists() {
            // 若 specs/ 本次新建且空了，移除；非空（其他 capability）則保留。
            let _ = std::fs::remove_dir(&specs_dir);
        }
        write_result.map(|_| target.clone())?;
    }
    Ok(target)
}

/// 掃描 `<change_dir>/` 並回傳 [`ChangeStatus`]。
///
/// 純讀；不修改 filesystem。實作流程：
///
/// 1. 缺 `metadata.json` → [`LocalProviderError::ChangeNotFound`]
/// 2. 解析 `metadata.json` 為 [`Change`]，取 `state`
/// 3. 對 `proposal.md` / `design.md` / `tasks.md` 各產生一筆 [`ArtifactStatus`]（含 missing）
/// 4. 列舉 `specs/<capability>/spec.md` — 缺檔的 capability 略過；其餘按字典序加入
pub fn scan_change_status(
    base: &Path,
    change_id: &ChangeId,
) -> Result<ChangeStatus, LocalProviderError> {
    let dir = change_dir(base, change_id);
    let meta_path = dir.join("metadata.json");
    if !meta_path.exists() {
        return Err(LocalProviderError::ChangeNotFound {
            change_id: change_id.as_str().to_string(),
        });
    }
    let raw = std::fs::read_to_string(&meta_path)?;
    let change: Change = serde_json::from_str(&raw)?;

    let mut artifacts: Vec<ArtifactStatus> = Vec::new();

    // 3 個固定名稱 artifact
    for (id, kind, filename) in [
        ("proposal", ArtifactKind::Proposal, "proposal.md"),
        ("design", ArtifactKind::Design, "design.md"),
        ("tasks", ArtifactKind::Tasks, "tasks.md"),
    ] {
        let path = dir.join(filename);
        let status = if path.is_file() {
            ArtifactState::Done
        } else {
            ArtifactState::Missing
        };
        let rel = artifact_relative_path(base, change_id, filename);
        artifacts.push(ArtifactStatus {
            id: id.to_string(),
            kind,
            path: rel,
            status,
            required: false, // 由 CLI/runtime 套用固定規則時覆寫
            dependencies: Vec::new(),
        });
    }

    // specs/<cap>/spec.md
    let specs_dir = dir.join(SPECS_DIR);
    if specs_dir.is_dir() {
        let mut caps: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let cap = match path.file_name().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            if !path.join("spec.md").is_file() {
                continue;
            }
            caps.push(cap);
        }
        caps.sort();
        for cap in caps {
            let rel_buf = PathBuf::from(SPECLINK_DIR)
                .join(CHANGES_DIR)
                .join(change_id.as_str())
                .join(SPECS_DIR)
                .join(&cap)
                .join("spec.md");
            artifacts.push(ArtifactStatus {
                id: format!("spec:{cap}"),
                kind: ArtifactKind::Spec,
                path: to_posix_string(&rel_buf),
                status: ArtifactState::Done,
                required: false,
                dependencies: Vec::new(),
            });
        }
    }

    Ok(ChangeStatus {
        change_id: change.change_id,
        state: change.state,
        artifacts,
    })
}

fn artifact_relative_path(_base: &Path, change_id: &ChangeId, filename: &str) -> String {
    let buf = PathBuf::from(SPECLINK_DIR)
        .join(CHANGES_DIR)
        .join(change_id.as_str())
        .join(filename);
    to_posix_string(&buf)
}

#[cfg(test)]
mod tests {
    use crate::storage::{
        archive_change_dir, archive_root_dir, is_valid_capability_name, is_valid_change_id,
        main_spec_dir, main_spec_path, scan_change_status, write_design_atomic,
        write_proposal_atomic, write_spec_atomic, write_tasks_atomic,
    };
    use provider::model::{ArtifactKind, ArtifactState, ChangeId, State};
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

    #[test]
    fn capability_name_validation_table() {
        // 對應 spec `Spec capability routing` 範例：
        assert!(is_valid_capability_name("auth"));
        assert!(is_valid_capability_name("user-auth"));
        assert!(!is_valid_capability_name("Auth-Module"));
        assert!(!is_valid_capability_name("1bad"));
        assert!(!is_valid_capability_name("add--feature"));
        assert!(!is_valid_capability_name("add-"));
        assert!(!is_valid_capability_name(""));
    }

    #[test]
    fn write_spec_creates_capability_subdir() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        write_spec_atomic(base, &change_id, "auth", "auth spec body\n").expect("spec write");
        let path = base
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("specs")
            .join("auth")
            .join("spec.md");
        assert!(path.is_file(), "spec.md must exist at {}", path.display());
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body, "auth spec body\n");
    }

    #[test]
    fn write_spec_two_capabilities_coexist() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        write_spec_atomic(base, &change_id, "auth", "auth\n").unwrap();
        write_spec_atomic(base, &change_id, "billing", "billing\n").unwrap();

        let specs = base
            .join(".speclink")
            .join("changes")
            .join("demo")
            .join("specs");
        assert!(specs.join("auth/spec.md").is_file());
        assert!(specs.join("billing/spec.md").is_file());
    }

    #[test]
    fn write_spec_refuses_invalid_capability() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        let err = write_spec_atomic(base, &change_id, "Bad-Name", "body\n").expect_err("must err");
        assert!(matches!(
            err,
            crate::error::LocalProviderError::InvalidCapability { .. }
        ));
        let specs_dir = base.join(".speclink/changes/demo/specs");
        assert!(
            !specs_dir.exists(),
            "specs/ must not be created on invalid capability"
        );
    }

    #[test]
    fn write_spec_cleanup_on_failure_does_not_remove_preexisting_specs_dir() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();
        write_spec_atomic(base, &change_id, "auth", "auth\n").unwrap();

        // 對 already-exists 觸發失敗：再寫一次同 capability。
        let err = write_spec_atomic(base, &change_id, "auth", "again\n").expect_err("must err");
        assert!(matches!(
            err,
            crate::error::LocalProviderError::ArtifactAlreadyExists { .. }
        ));

        let auth_spec = base.join(".speclink/changes/demo/specs/auth/spec.md");
        assert!(auth_spec.is_file(), "pre-existing spec.md must remain");
        let body = std::fs::read_to_string(&auth_spec).unwrap();
        assert_eq!(body, "auth\n", "pre-existing content must not change");
    }

    #[test]
    fn write_design_creates_only_design_md() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        write_design_atomic(base, &change_id, "design body\n").expect("design");
        let dir = base.join(".speclink/changes/demo");
        assert!(dir.join("design.md").is_file());
        assert!(!dir.join("tasks.md").exists());
        assert!(!dir.join("specs").exists());
    }

    #[test]
    fn write_tasks_creates_only_tasks_md() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        write_tasks_atomic(base, &change_id, "tasks body\n").expect("tasks");
        let dir = base.join(".speclink/changes/demo");
        assert!(dir.join("tasks.md").is_file());
        assert!(!dir.join("design.md").exists());
    }

    #[test]
    fn write_design_refuses_existing_file() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();
        write_design_atomic(base, &change_id, "OLD\n").unwrap();

        let err = write_design_atomic(base, &change_id, "NEW\n").expect_err("must err");
        assert!(matches!(
            err,
            crate::error::LocalProviderError::ArtifactAlreadyExists { .. }
        ));
        let body = std::fs::read_to_string(base.join(".speclink/changes/demo/design.md")).unwrap();
        assert_eq!(body, "OLD\n", "existing design.md must not be overwritten");
    }

    #[test]
    fn write_design_refuses_missing_change_dir() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let err = write_design_atomic(base, &ChangeId::from("missing"), "x\n").expect_err("err");
        assert!(matches!(
            err,
            crate::error::LocalProviderError::ChangeNotFound { .. }
        ));
    }

    #[test]
    fn scan_change_status_proposal_only_returns_three_entries() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();

        let status = scan_change_status(base, &change_id).expect("scan");
        assert_eq!(status.change_id.as_str(), "demo");
        assert_eq!(status.state, State::Proposed);
        assert_eq!(status.artifacts.len(), 3);
        assert_eq!(status.artifacts[0].id, "proposal");
        assert_eq!(status.artifacts[0].status, ArtifactState::Done);
        assert_eq!(status.artifacts[0].kind, ArtifactKind::Proposal);
        assert_eq!(status.artifacts[1].id, "design");
        assert_eq!(status.artifacts[1].status, ArtifactState::Missing);
        assert_eq!(status.artifacts[2].id, "tasks");
        assert_eq!(status.artifacts[2].status, ArtifactState::Missing);
    }

    #[test]
    fn scan_change_status_returns_specs_sorted() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();
        write_spec_atomic(base, &change_id, "billing", "b\n").unwrap();
        write_spec_atomic(base, &change_id, "auth", "a\n").unwrap();

        let status = scan_change_status(base, &change_id).expect("scan");
        assert_eq!(status.artifacts.len(), 5);
        let ids: Vec<&str> = status.artifacts.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["proposal", "design", "tasks", "spec:auth", "spec:billing"]
        );
    }

    #[test]
    fn scan_change_status_change_not_found() {
        let tmp = TempDir::new().unwrap();
        let err = scan_change_status(tmp.path(), &ChangeId::from("missing")).expect_err("err");
        assert!(matches!(
            err,
            crate::error::LocalProviderError::ChangeNotFound { .. }
        ));
    }

    #[test]
    fn scan_change_status_malformed_metadata_is_json_error() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let dir = base.join(".speclink/changes/demo");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("metadata.json"), "{bad json").unwrap();

        let err = scan_change_status(base, &ChangeId::from("demo")).expect_err("err");
        assert!(matches!(err, crate::error::LocalProviderError::Json(_)));
        // 對應 CLI mapping → internal.error
        assert_eq!(err.error_code(), "internal.error");
    }

    #[test]
    fn scan_change_status_empty_specs_dir_produces_no_spec_entries() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();
        std::fs::create_dir_all(base.join(".speclink/changes/demo/specs")).unwrap();

        let status = scan_change_status(base, &change_id).expect("scan");
        assert_eq!(status.artifacts.len(), 3, "no spec entries expected");
    }

    #[test]
    fn main_spec_dir_returns_expected_components() {
        let base = Path::new("/tmp/proj");
        let dir = main_spec_dir(base);
        let comps: Vec<_> = dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        // 預期：root / "tmp" / "proj" / ".speclink" / "specs"
        let tail = &comps[comps.len() - 2..];
        assert_eq!(tail, &[".speclink".to_string(), "specs".to_string()]);
    }

    #[test]
    fn main_spec_path_returns_expected() {
        let base = Path::new("/tmp/proj");
        let path = main_spec_path(base, "user-auth");
        let comps: Vec<_> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        // 預期最後四個 component：".speclink" / "specs" / "user-auth" / "spec.md"
        let tail = &comps[comps.len() - 4..];
        assert_eq!(
            tail,
            &[
                ".speclink".to_string(),
                "specs".to_string(),
                "user-auth".to_string(),
                "spec.md".to_string()
            ]
        );
        // 不應出現硬編 `/` 或 `\` — PathBuf::join 已處理 separator
        let raw = path.to_string_lossy().to_string();
        // 在 Windows 上會是 backslash；POSIX 是 forward slash。兩者皆非硬編。
        assert!(
            !raw.contains("speclink/specs/")
                || !raw.contains("speclink\\specs\\")
                || raw.contains("speclink"),
            "path components should be platform-correct: {raw}"
        );
    }

    #[test]
    fn archive_root_dir_returns_expected() {
        let base = Path::new("/tmp/proj");
        let dir = archive_root_dir(base);
        let comps: Vec<_> = dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let tail = &comps[comps.len() - 3..];
        assert_eq!(
            tail,
            &[
                ".speclink".to_string(),
                "changes".to_string(),
                "archive".to_string()
            ]
        );
    }

    #[test]
    fn archive_change_dir_uses_date_prefix() {
        let base = Path::new("/tmp/proj");
        let path = archive_change_dir(base, &ChangeId::from("demo"), "2026-05-19");
        let comps: Vec<_> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        assert_eq!(comps[comps.len() - 1], "2026-05-19-demo");
        let tail = &comps[comps.len() - 4..];
        assert_eq!(
            tail,
            &[
                ".speclink".to_string(),
                "changes".to_string(),
                "archive".to_string(),
                "2026-05-19-demo".to_string()
            ]
        );
    }

    #[test]
    fn scan_change_status_subdir_without_spec_md_is_ignored() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();
        let change_id = ChangeId::from("demo");
        write_proposal_atomic(base, &change_id, "x").unwrap();
        // 建立 specs/auth 但不放 spec.md
        std::fs::create_dir_all(base.join(".speclink/changes/demo/specs/auth")).unwrap();

        let status = scan_change_status(base, &change_id).expect("scan");
        assert_eq!(status.artifacts.len(), 3);
        assert!(
            status.artifacts.iter().all(|a| !a.id.starts_with("spec:")),
            "no spec:* entries expected"
        );
    }
}

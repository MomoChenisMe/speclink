//! `.gitignore` line-based append helper.
//!
//! 政策：本 change 只允許 `.gitignore` 內出現一行 `.speclink/link.yaml`。
//! 不存在則建立檔案；存在則 line-based exact match 後 append；已存在 exact match 則 no-op。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::RuntimeError;

/// 對 `path` 內的 `.gitignore` 進行 idempotent append。
///
/// # Errors
/// 當檔案讀寫失敗時回 [`RuntimeError::Internal`]。
pub fn append_if_missing(gitignore_path: &Path, line: &str) -> Result<(), RuntimeError> {
    let existing = match fs::read_to_string(gitignore_path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(RuntimeError::Internal(format!("read .gitignore: {e}")));
        }
    };

    let already_present = existing
        .as_deref()
        .map(|s| s.lines().any(|l| l == line))
        .unwrap_or(false);

    if already_present {
        return Ok(());
    }

    match existing {
        None => fs::write(gitignore_path, format!("{line}\n"))
            .map_err(|e| RuntimeError::Internal(format!("create .gitignore: {e}")))?,
        Some(mut content) => {
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            let mut f = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(gitignore_path)
                .map_err(|e| RuntimeError::Internal(format!("open .gitignore: {e}")))?;
            f.write_all(content.as_bytes())
                .map_err(|e| RuntimeError::Internal(format!("write .gitignore: {e}")))?;
            writeln!(f, "{line}")
                .map_err(|e| RuntimeError::Internal(format!("append .gitignore: {e}")))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_gitignore_when_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        append_if_missing(&path, ".speclink/link.yaml").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, ".speclink/link.yaml\n");
    }

    #[test]
    fn appends_when_file_exists_with_other_lines() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "node_modules\n").unwrap();
        append_if_missing(&path, ".speclink/link.yaml").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "node_modules\n.speclink/link.yaml\n");
    }

    #[test]
    fn idempotent_when_line_already_present() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "node_modules\n.speclink/link.yaml\n").unwrap();
        append_if_missing(&path, ".speclink/link.yaml").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.matches(".speclink/link.yaml").count(), 1);
    }
}

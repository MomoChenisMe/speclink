//! Git CLI shell-out wrapper.
//!
//! 抽象出 git CLI 呼叫，方便測試 mock。

#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::RuntimeError;

/// Git probe trait：抽象出 git CLI 呼叫，方便測試 mock。
pub trait GitProbe: Send + Sync {
    /// 回傳 git common dir（已 join 為絕對路徑）。
    ///
    /// # Errors
    /// 當 working_dir 不在 git working tree 內、或 git 執行檔不存在時，回
    /// [`RuntimeError::RequiresGit`]。
    fn common_dir(&self, working_dir: &Path) -> Result<PathBuf, RuntimeError>;

    /// 回傳當前 HEAD 的 commit SHA；空 repository（無 commit）時回空字串。
    ///
    /// # Errors
    /// 當 working_dir 不在 git working tree 內時回 [`RuntimeError::RequiresGit`]。
    fn head_sha(&self, working_dir: &Path) -> Result<String, RuntimeError>;
}

/// 預設實作：直接 spawn `git` 子程序。
pub struct RealGitProbe;

impl GitProbe for RealGitProbe {
    fn common_dir(&self, working_dir: &Path) -> Result<PathBuf, RuntimeError> {
        let output = Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .current_dir(working_dir)
            .output()
            .map_err(|e| RuntimeError::RequiresGit {
                context: format!("cannot spawn git: {e}"),
            })?;
        if !output.status.success() {
            return Err(RuntimeError::RequiresGit {
                context: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if raw.is_empty() {
            return Err(RuntimeError::RequiresGit {
                context: "git rev-parse returned empty common-dir".into(),
            });
        }
        let candidate = PathBuf::from(raw);
        if candidate.is_absolute() {
            Ok(candidate)
        } else {
            Ok(working_dir.join(candidate))
        }
    }

    fn head_sha(&self, working_dir: &Path) -> Result<String, RuntimeError> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(working_dir)
            .output()
            .map_err(|e| RuntimeError::RequiresGit {
                context: format!("cannot spawn git: {e}"),
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
        // Empty repo (no commits yet) is acceptable — return empty string.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unknown revision") || stderr.contains("ambiguous argument 'HEAD'") {
            return Ok(String::new());
        }
        Err(RuntimeError::RequiresGit {
            context: stderr.trim().to_string(),
        })
    }
}

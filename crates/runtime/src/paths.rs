//! Path resolution: artifact root, state root via `git rev-parse --git-common-dir`,
//! and shared display helper for `state_root` field in CLI output.

#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};

use crate::error::RuntimeError;
use crate::git::GitProbe;

/// `.speclink/` 相對於 working tree root 的路徑（永遠是這個常量）。
pub const ARTIFACT_ROOT: &str = ".speclink";

/// State root namespace under git common dir。
pub const STATE_ROOT_NAMESPACE: &str = "speclink";

/// 回傳 `<working_dir>/.speclink`（absolute）。
#[must_use]
pub fn artifact_root(working_dir: &Path) -> PathBuf {
    working_dir.join(ARTIFACT_ROOT)
}

/// 透過 `git_probe` 解析 state root：`<git-common-dir>/speclink/`。
///
/// # Errors
/// 當 `git_probe.common_dir` 失敗（git 不在 PATH、working_dir 不在 git working tree）時
/// 回 [`RuntimeError::RequiresGit`]。
pub fn resolve_state_root<G: GitProbe>(
    git_probe: &G,
    working_dir: &Path,
) -> Result<PathBuf, RuntimeError> {
    let common = git_probe.common_dir(working_dir)?;
    Ok(common.join(STATE_ROOT_NAMESPACE))
}

/// 把 state root 顯示為相對於 working_dir 的 POSIX 路徑（用於 `ProjectInfo` / `status`）。
///
/// 行為：
/// - 當 state root 在 working_dir 內（main repo 場景）：回傳 POSIX 樣式相對路徑
///   （例如 `.git/speclink`）。
/// - 當 state root 不在 working_dir 內（linked worktree 場景）：回傳 canonical 絕對路徑，
///   不再用 `components().join("/")` 拼，避免在 POSIX 上產生開頭 `//` 雙斜線。
#[must_use]
pub fn display_state_root(working_dir: &Path, state_root: &Path) -> String {
    let working_canonical = working_dir
        .canonicalize()
        .unwrap_or_else(|_| working_dir.to_path_buf());
    let state_canonical = state_root
        .canonicalize()
        .unwrap_or_else(|_| state_root.to_path_buf());
    match state_canonical.strip_prefix(&working_canonical) {
        Ok(rel) => rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/"),
        Err(_) => state_canonical.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_uses_relative_path_when_state_root_inside_working_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let working = tmp.path().canonicalize().unwrap();
        let state = working.join(".git").join("speclink");
        std::fs::create_dir_all(&state).unwrap();
        assert_eq!(display_state_root(&working, &state), ".git/speclink");
    }

    #[test]
    fn display_uses_absolute_path_when_state_root_outside_working_dir() {
        let main_tmp = tempfile::TempDir::new().unwrap();
        let wt_tmp = tempfile::TempDir::new().unwrap();
        let main = main_tmp.path().canonicalize().unwrap();
        let wt = wt_tmp.path().canonicalize().unwrap();
        let state = main.join(".git").join("speclink");
        std::fs::create_dir_all(&state).unwrap();
        let displayed = display_state_root(&wt, &state);
        assert!(
            !displayed.starts_with("//"),
            "display MUST NOT begin with //, got: {displayed:?}"
        );
        assert!(
            displayed.contains("speclink"),
            "display MUST contain the speclink namespace, got: {displayed:?}"
        );
    }
}

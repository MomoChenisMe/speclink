//! Dev-time pre-flight checks for SpecLink slice apply.
//!
//! 提供 unit-testable helper 供 spectra `/spectra-apply` skill 在 walking-skeleton
//! slice A3 開工前驗證 A2 已 archive。本模組純為開發流程 guardrail；不在 CLI
//! 表面暴露任何指令。
//!
//! 對齊 `add-state-machine-and-apply` proposal 的 prerequisite：
//! 「本 change apply 前 user 必須先 archive `add-change-and-artifact-io`，
//!  使 `openspec/specs/change-store/spec.md` 出現在 working tree」。

#![allow(clippy::doc_markdown)]

use std::path::Path;

/// `openspec/specs/change-store/spec.md` 是否存在於 `working_dir` 下。
///
/// `true` 代表 A2 (`add-change-and-artifact-io`) baseline 已 archive，A3 的
/// `change-store` MODIFIED capability spec 可以對齊既有 baseline；`false` 代表
/// 必須先跑 `/spectra-archive add-change-and-artifact-io`。
#[must_use]
pub fn precheck_a2_archived(working_dir: &Path) -> bool {
    working_dir
        .join("openspec")
        .join("specs")
        .join("change-store")
        .join("spec.md")
        .is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn returns_true_when_change_store_spec_exists() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp
            .path()
            .join("openspec")
            .join("specs")
            .join("change-store");
        fs::create_dir_all(&dir).expect("create spec dir");
        fs::write(dir.join("spec.md"), "## stub").expect("write spec");
        assert!(precheck_a2_archived(tmp.path()));
    }

    #[test]
    fn returns_false_when_spec_dir_missing() {
        let tmp = TempDir::new().expect("tempdir");
        assert!(!precheck_a2_archived(tmp.path()));
    }

    #[test]
    fn returns_false_when_spec_file_missing_but_dir_exists() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp
            .path()
            .join("openspec")
            .join("specs")
            .join("change-store");
        fs::create_dir_all(&dir).expect("create spec dir");
        assert!(!precheck_a2_archived(tmp.path()));
    }
}

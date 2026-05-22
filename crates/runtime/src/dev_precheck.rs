//! Dev-time pre-flight checks for SpecLink slice apply.
//!
//! 提供 unit-testable helper 供 spectra `/spectra-apply` skill 在 walking-skeleton
//! slice A3 開工前驗證 A2 已 archive。本模組純為開發流程 guardrail；不在 CLI
//! 表面暴露任何指令。
//!
//! 對齊 walking-skeleton slice apply 的 prerequisite：
//! - A3 (`add-state-machine-and-apply`) apply 前須先 archive A2，使
//!   `openspec/specs/change-store/spec.md` 出現在 working tree。
//! - A4 (`add-archive`) apply 前須先 archive A3，使
//!   `openspec/specs/state-machine/spec.md` 與
//!   `openspec/specs/apply-task-ops/spec.md` 出現在 working tree。

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

/// `openspec/specs/state-machine/spec.md` 與 `openspec/specs/apply-task-ops/spec.md`
/// 都存在於 `working_dir` 下。
///
/// `true` 代表 A3 (`add-state-machine-and-apply`) baseline 已 archive，A4 的
/// `state-machine` MODIFIED capability spec 可以對齊既有 baseline；`false` 代表
/// 必須先跑 `/spectra-archive add-state-machine-and-apply`。
#[must_use]
pub fn precheck_a3_archived(working_dir: &Path) -> bool {
    let specs = working_dir.join("openspec").join("specs");
    specs.join("state-machine").join("spec.md").is_file()
        && specs.join("apply-task-ops").join("spec.md").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_spec(working_dir: &Path, capability: &str) {
        let dir = working_dir.join("openspec").join("specs").join(capability);
        fs::create_dir_all(&dir).expect("create spec dir");
        fs::write(dir.join("spec.md"), "## stub").expect("write spec");
    }

    #[test]
    fn returns_true_when_change_store_spec_exists() {
        let tmp = TempDir::new().expect("tempdir");
        write_spec(tmp.path(), "change-store");
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

    #[test]
    fn a3_precheck_true_when_both_specs_exist() {
        let tmp = TempDir::new().expect("tempdir");
        write_spec(tmp.path(), "state-machine");
        write_spec(tmp.path(), "apply-task-ops");
        assert!(precheck_a3_archived(tmp.path()));
    }

    #[test]
    fn a3_precheck_false_when_state_machine_spec_missing() {
        let tmp = TempDir::new().expect("tempdir");
        write_spec(tmp.path(), "apply-task-ops");
        assert!(!precheck_a3_archived(tmp.path()));
    }

    #[test]
    fn a3_precheck_false_when_apply_task_ops_spec_missing() {
        let tmp = TempDir::new().expect("tempdir");
        write_spec(tmp.path(), "state-machine");
        assert!(!precheck_a3_archived(tmp.path()));
    }

    #[test]
    fn a3_precheck_false_when_both_specs_missing() {
        let tmp = TempDir::new().expect("tempdir");
        assert!(!precheck_a3_archived(tmp.path()));
    }
}

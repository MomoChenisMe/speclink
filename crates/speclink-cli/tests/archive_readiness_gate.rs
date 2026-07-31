//! `speclink archive` 單筆封存的任務完成度守門 — fs 模式契約
//! (change-lifecycle spec「單筆封存的任務完成度守門」)。
//!
//! 任務未完成(總數>0 且未全勾)的單筆封存拒絕:非零 exit code、stderr 載明
//! N/M 證據與兩條出路、零檔案效果。--mark-tasks-complete 維持既有語意:
//! 先全勾再封存。成功路徑(全完成/0 任務/批次)的既有測試不在此檔、不修改。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const DELTA_SPEC: &str = "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Throwaway project: one structurally valid change `demo` (proposal +
    /// delta spec + the given tasks.md), no git — @trace probes fail soft.
    fn new(tag: &str, tasks_md: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-archive-gate-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("demo-cap")).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), tasks_md).unwrap();
        std::fs::write(change.join("specs").join("demo-cap").join("spec.md"), DELTA_SPEC).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn archive_dir(&self) -> PathBuf {
        self.dir.join("openspec").join("changes").join("archive")
    }

    /// The dated archived change directory (`<date>-demo`), if any.
    fn archived_demo(&self) -> Option<PathBuf> {
        let entries = std::fs::read_dir(self.archive_dir()).ok()?;
        entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with("-demo")))
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn incomplete_change_refuses_with_evidence_and_no_archive_dir() {
    // spec scenario「任務未完成的單筆封存被拒」:3 任務 1 勾 → 非零 exit,
    // stderr 載明 1/3 與兩條出路,changes/archive/ 無新目錄、change 原地不動。
    let p = TempProject::new("refuse", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success(), "incomplete change must refuse archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1/3"), "evidence N/M on stderr: {stderr}");
    assert!(stderr.contains("--mark-tasks-complete"), "exit route named: {stderr}");
    assert!(p.archived_demo().is_none(), "no archived directory appears");
    assert!(
        p.dir.join("openspec").join("changes").join("demo").join("tasks.md").is_file(),
        "change stays in place"
    );
    let tasks =
        std::fs::read_to_string(p.dir.join("openspec").join("changes").join("demo").join("tasks.md"))
            .unwrap();
    assert_eq!(tasks, "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n", "tasks.md byte-identical");
}

#[test]
fn mark_tasks_complete_archives_and_checks_every_task() {
    // spec scenario「--mark-tasks-complete 放行並先全勾」:exit 0,封存後的
    // tasks.md 全部任務為已勾,active change 目錄消失。
    let p = TempProject::new("mark", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let out = p.run(&["archive", "demo", "--mark-tasks-complete"]);
    assert!(
        out.status.success(),
        "mark-tasks-complete must archive: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !p.dir.join("openspec").join("changes").join("demo").exists(),
        "active change moved into the archive"
    );
    let archived = p.archived_demo().expect("dated archive directory exists");
    let tasks = std::fs::read_to_string(archived.join("tasks.md")).unwrap();
    assert!(!tasks.contains("- [ ]"), "every task checked after archive: {tasks}");
    assert_eq!(tasks.matches("- [x]").count(), 3, "all three tasks present and checked: {tasks}");
}

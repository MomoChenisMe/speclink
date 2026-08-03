//! `speclink task done` — 任務完成蘊含開工標記（change-lifecycle spec）。
//!
//! 指令面是 parity 凍結對象：人眼輸出、`--json` payload、錯誤訊息與順序、exit
//! code 皆須與現行一致；新增的檔案效果僅為首次完成時 change meta 蓋開工章。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";
const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Throwaway project: one change with the given meta, optionally a tasks.md,
    /// rooted in a git repo with a deterministic local identity.
    fn new(tag: &str, tasks_md: Option<&str>) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-taskdone-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        if let Some(t) = tasks_md {
            std::fs::write(change.join("tasks.md"), t).unwrap();
        }
        let git = |args: &[&str]| {
            let ok = Command::new("git")
                .args(args)
                .current_dir(&dir)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.name", "Sandbox Tester"]);
        git(&["config", "user.email", "sandbox@example.com"]);
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .output()
            .expect("run speclink binary")
    }

    fn meta(&self) -> String {
        std::fs::read_to_string(
            self.dir
                .join("openspec")
                .join("changes")
                .join("demo")
                .join(".openspec.yaml"),
        )
        .unwrap()
    }

    fn tasks(&self) -> String {
        std::fs::read_to_string(
            self.dir
                .join("openspec")
                .join("changes")
                .join("demo")
                .join("tasks.md"),
        )
        .unwrap()
    }

    fn touched_exists(&self) -> bool {
        self.dir
            .join(".speclink")
            .join("touched")
            .join("demo.json")
            .exists()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[test]
fn first_completion_stamps_started_at_and_keeps_output_shape() {
    let p = TempProject::new("first", Some(TASKS));
    let out = p.run(&["task", "done", "1", "--change", "demo"]);

    // 人眼輸出與 exit code 凍結（piped stdout → 無色彩）。
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "✓ Task 1 marked as done: 1.1 first task\n"
    );
    assert!(p.tasks().contains("- [x] 1.1 first task"));

    // 刻意檔案效果分歧：meta 蓋開工章，既有欄位逐字元保留。
    let meta = p.meta();
    assert!(meta.starts_with(META), "existing meta must be preserved verbatim: {meta}");
    assert!(
        meta.contains(&format!("started_at: {}", today())),
        "first completion must stamp started_at: {meta}"
    );
    assert!(
        meta.contains("started_by: Sandbox Tester <sandbox@example.com>"),
        "git identity must be attributed: {meta}"
    );
    assert!(!meta.contains("started_with"), "no agent source → started_with absent: {meta}");
}

#[test]
fn json_payload_shape_is_frozen_and_still_stamps() {
    let p = TempProject::new("json", Some(TASKS));
    let out = p.run(&["task", "done", "2", "--change", "demo", "--json"]);

    assert!(out.status.success());
    // Compact single-line payload, key order frozen.
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "{\"change\":\"demo\",\"status\":\"done\",\"task_desc\":\"1.2 second task\",\"task_id\":\"2\"}\n"
    );
    assert!(p.meta().contains("started_at: "), "stamping applies on the --json path too");
}

#[test]
fn already_started_change_keeps_first_stamp_verbatim() {
    let p = TempProject::new("started", Some(TASKS));
    let stamped = format!("{META}started_at: 2026-07-01\nstarted_by: First <first@example.com>\n");
    std::fs::write(
        p.dir
            .join("openspec")
            .join("changes")
            .join("demo")
            .join(".openspec.yaml"),
        &stamped,
    )
    .unwrap();

    let out = p.run(&["task", "done", "1", "--change", "demo"]);
    assert!(out.status.success());
    assert_eq!(p.meta(), stamped, "first stamp must be kept verbatim");
}

#[test]
fn already_done_task_errors_and_changes_nothing() {
    let done_tasks = "## 1. Group\n\n- [x] 1.1 first task\n- [ ] 1.2 second task\n";
    let p = TempProject::new("already", Some(done_tasks));
    let out = p.run(&["task", "done", "1", "--change", "demo"]);

    assert!(!out.status.success(), "already-done must keep the error exit");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Task 1 is already done\n"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(p.tasks(), done_tasks, "tasks.md must be untouched");
    assert_eq!(p.meta(), META, "no stamp on the error path");
    assert!(!p.touched_exists(), "no touched record on the error path");
}

#[test]
fn missing_tasks_md_errors_before_id_validation_and_does_not_stamp() {
    let p = TempProject::new("notasks", None);

    // 有效 id：現行錯誤訊息不變。
    let out = p.run(&["task", "done", "1", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: tasks.md not found for change 'demo'\n"
    );

    // 無效 id：tasks.md 缺失檢查先於 id 驗證（順序凍結）。
    let out = p.run(&["task", "done", "abc", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: tasks.md not found for change 'demo'\n"
    );

    assert_eq!(p.meta(), META, "missing tasks.md must not stamp");
}

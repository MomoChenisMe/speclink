//! `speclink task undone` — 任務取消勾選動詞的 fs 模式契約（verb-contract spec）。
//!
//! 與 `task done` 全面對稱：人眼輸出、`--json` payload、錯誤訊息與順序、exit
//! code；刻意分歧是零側效——不寫 touched 記錄、不動 meta 開工標記。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";
// 第 3 個任務已勾選（spec Example: 取消第 3 個任務的 payload）。
const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n- [x] 1.3 Third\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Throwaway project: one change with the given meta, optionally a tasks.md,
    /// rooted in a git repo with a deterministic local identity.
    fn new(tag: &str, tasks_md: Option<&str>) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-taskundone-{tag}-{}",
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

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_speclink"));
        c.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE");
        c
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    /// 以 CLICOLOR_FORCE=1 執行——piped stdout 也強制上色，用來驗證綠色 ✓。
    fn run_forced_color(&self, args: &[&str]) -> Output {
        self.cmd(args)
            .env("CLICOLOR_FORCE", "1")
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

#[test]
fn success_prints_green_check_flips_only_target_and_has_no_side_effects() {
    let p = TempProject::new("success", Some(TASKS));
    let out = p.run_forced_color(&["task", "undone", "3", "--change", "demo"]);

    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "\x1b[32m✓\x1b[0m Task 3 marked as not done: 1.3 Third\n"
    );
    assert_eq!(
        p.tasks(),
        "## 1. Group\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n- [ ] 1.3 Third\n",
        "only the target line flips back to open"
    );
    // 零側效：不寫 touched 記錄、不動 meta 開工標記。
    assert!(!p.touched_exists(), "undone must not record touched files");
    assert_eq!(p.meta(), META, "undone must not stamp or alter meta");
}

#[test]
fn no_color_flag_strips_ansi_from_success_output() {
    let p = TempProject::new("nocolor", Some(TASKS));
    let out = p.run_forced_color(&["--no-color", "task", "undone", "3", "--change", "demo"]);

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "✓ Task 3 marked as not done: 1.3 Third\n",
        "--no-color output must contain no ANSI sequences"
    );
}

#[test]
fn json_payload_shape_is_symmetric_with_done() {
    let p = TempProject::new("json", Some(TASKS));
    let out = p.run(&["task", "undone", "3", "--change", "demo", "--json"]);

    assert!(out.status.success());
    // Compact single-line payload, key order frozen: change, status, task_desc, task_id
    // (spec Example: 取消第 3 個任務的 payload).
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "{\"change\":\"demo\",\"status\":\"undone\",\"task_desc\":\"1.3 Third\",\"task_id\":\"3\"}\n"
    );
    assert!(!p.touched_exists());
    assert_eq!(p.meta(), META);
}

#[test]
fn already_not_done_task_errors_and_changes_nothing() {
    let p = TempProject::new("already", Some(TASKS));
    let out = p.run(&["task", "undone", "1", "--change", "demo"]);

    assert!(!out.status.success(), "already-not-done must exit non-zero");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Task 1 is already not done\n"
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
    assert_eq!(p.tasks(), TASKS, "tasks.md must be untouched");
    assert_eq!(p.meta(), META);
    assert!(!p.touched_exists());
}

#[test]
fn invalid_task_id_errors_match_done_shapes() {
    let p = TempProject::new("badid", Some(TASKS));

    // 非數字。
    let out = p.run(&["task", "undone", "abc", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Invalid task ID 'abc': must be a number or a tsk_-prefixed stable ID\n"
    );

    // id < 1。
    let out = p.run(&["task", "undone", "0", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stderr), "Error: Task ID must be >= 1\n");

    // id 超界。
    let out = p.run(&["task", "undone", "9", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Task 9 not found (total: 3)\n"
    );

    assert_eq!(p.tasks(), TASKS, "error paths must leave tasks.md untouched");
    assert_eq!(p.meta(), META);
    assert!(!p.touched_exists());
}

#[test]
fn missing_tasks_md_errors_before_id_validation() {
    let p = TempProject::new("notasks", None);

    // 有效 id：與 task done 同訊息。
    let out = p.run(&["task", "undone", "1", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: tasks.md not found for change 'demo'\n"
    );

    // 無效 id：tasks.md 缺失檢查先於 id 驗證（順序與 done 一致）。
    let out = p.run(&["task", "undone", "abc", "--change", "demo"]);
    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: tasks.md not found for change 'demo'\n"
    );

    assert_eq!(p.meta(), META);
}

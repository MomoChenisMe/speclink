//! `speclink task done` / `task undone` 的 tsk_ stable ID 值域（task-identity
//! spec、verb-contract 值域擴充）：--json 形狀與數字值域一致、行尾註解原文
//! 保留、非法值錯誤敘述修訂、查無此 ID 與超界對稱。
//! 數字值域的凍結輸出由 task_done_stamps.rs／task_undone.rs 釘住。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";
const TID_A: &str = "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV";
const TID_B: &str = "tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ";

/// 帶 ID 的任務檔：第 1 任務未勾（供 done）、第 2 任務已勾（供 undone）。
fn tasks_md() -> String {
    format!(
        "## 1. Group\n\n- [ ] 1.1 First <!-- speclink-task:{TID_A} -->\n- [x] 1.2 Second <!-- speclink-task:{TID_B} -->\n"
    )
}

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, tasks_md: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-taskstableid-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("tasks.md"), tasks_md).unwrap();
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

    fn tasks(&self) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("changes").join("demo").join("tasks.md"),
        )
        .unwrap()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn done_by_stable_id_json_shape_matches_numeric_domain() {
    let p = TempProject::new("donejson", &tasks_md());
    let out = p.run(&["task", "done", TID_A, "--change", "demo", "--json"]);

    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    // Compact 單行 JSON、鍵序與數字值域一致；task_id 原樣回填 argv。
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!(
            "{{\"change\":\"demo\",\"status\":\"done\",\"task_desc\":\"1.1 First\",\"task_id\":\"{TID_A}\"}}\n"
        )
    );
    // 僅目標行翻轉，行尾註解原文保留。
    assert_eq!(
        p.tasks(),
        format!(
            "## 1. Group\n\n- [x] 1.1 First <!-- speclink-task:{TID_A} -->\n- [x] 1.2 Second <!-- speclink-task:{TID_B} -->\n"
        )
    );
}

#[test]
fn undone_by_stable_id_json_shape_and_comment_preserved() {
    let p = TempProject::new("undonejson", &tasks_md());
    let out = p.run(&["task", "undone", TID_B, "--change", "demo", "--json"]);

    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!(
            "{{\"change\":\"demo\",\"status\":\"undone\",\"task_desc\":\"1.2 Second\",\"task_id\":\"{TID_B}\"}}\n"
        )
    );
    assert_eq!(
        p.tasks(),
        format!(
            "## 1. Group\n\n- [ ] 1.1 First <!-- speclink-task:{TID_A} -->\n- [ ] 1.2 Second <!-- speclink-task:{TID_B} -->\n"
        )
    );
}

#[test]
fn stable_id_human_output_echoes_the_id() {
    let p = TempProject::new("human", &tasks_md());
    let out = p.run(&["task", "done", TID_A, "--change", "demo"]);

    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("✓ Task {TID_A} marked as done: 1.1 First\n"),
        "human output echoes the typed stable id (piped stdout → no color)"
    );
}

#[test]
fn neither_numeric_nor_tsk_prefixed_value_errors() {
    let p = TempProject::new("badvalue", &tasks_md());
    let before = p.tasks();
    let out = p.run(&["task", "done", "abc", "--change", "demo"]);

    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Invalid task ID 'abc': must be a number or a tsk_-prefixed stable ID\n"
    );
    assert_eq!(p.tasks(), before, "error paths must leave tasks.md untouched");
}

#[test]
fn unknown_stable_id_errors_symmetric_to_out_of_range() {
    let p = TempProject::new("unknown", &tasks_md());
    let before = p.tasks();
    let out = p.run(&["task", "done", "tsk_01ZZZZZZZZZZZZZZZZZZZZZZZZZZ", "--change", "demo"]);

    assert!(!out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "Error: Task tsk_01ZZZZZZZZZZZZZZZZZZZZZZZZZZ not found (total: 2)\n"
    );
    assert_eq!(p.tasks(), before);
}

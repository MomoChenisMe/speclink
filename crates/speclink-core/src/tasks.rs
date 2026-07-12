//! Task list parsing, completion, and touched-file tracking.

use crate::store::Store;
use crate::util;
use crate::workspace::Workspace;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single checkbox task.
#[derive(Debug, Clone)]
pub struct Task {
    /// 1-based sequential index across all checkboxes in file order.
    pub id: usize,
    pub description: String,
    pub done: bool,
    pub parallel: bool,
}

/// Parse tasks.md into an ordered list of checkbox tasks. Dash and star bullets both
/// count (matches Spectra: `* [ ]` is a task).
pub fn parse(tasks_md: &str) -> Vec<Task> {
    let mut out = Vec::new();
    let mut id = 0usize;
    for line in tasks_md.lines() {
        let trimmed = line.trim_start();
        let unbulleted = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "));
        let (done, rest) = match unbulleted {
            Some(r) if r.starts_with("[ ] ") => (false, &r[4..]),
            Some(r) if r.starts_with("[x] ") || r.starts_with("[X] ") => (true, &r[4..]),
            _ => continue,
        };
        id += 1;
        let (parallel, desc) = match rest.strip_prefix("[P] ") {
            Some(d) => (true, d),
            None => (false, rest),
        };
        out.push(Task {
            id,
            description: desc.trim().to_string(),
            done,
            parallel,
        });
    }
    out
}

/// Progress tuple: (total, complete, remaining).
pub fn progress(tasks: &[Task]) -> (usize, usize, usize) {
    let total = tasks.len();
    let complete = tasks.iter().filter(|t| t.done).count();
    (total, complete, total - complete)
}

/// Flip the id-th checkbox to done. Returns (new_content, task_description) or None if not found /
/// already done.
pub fn mark_done(tasks_md: &str, target_id: usize) -> Option<(String, String, bool)> {
    flip_task(tasks_md, target_id, true)
}

/// Flip the id-th checkbox in either direction. Returns (new_content, task_description,
/// already_in_target_state) or None if not found. Indent, bullet style (Spectra rewrites
/// `* [ ]` to `* [x]`), and trailing newline are preserved.
fn flip_task(tasks_md: &str, target_id: usize, to_done: bool) -> Option<(String, String, bool)> {
    let mut id = 0usize;
    let mut already = false;
    let mut desc = String::new();
    let mut found = false;
    let mut out_lines: Vec<String> = Vec::new();
    for line in tasks_md.lines() {
        let trimmed = line.trim_start();
        let bullet = if trimmed.starts_with("- ") {
            '-'
        } else if trimmed.starts_with("* ") {
            '*'
        } else {
            '\0'
        };
        let body = if bullet != '\0' { &trimmed[2..] } else { "" };
        let is_open = bullet != '\0' && body.starts_with("[ ] ");
        let is_done = bullet != '\0' && (body.starts_with("[x] ") || body.starts_with("[X] "));
        if is_open || is_done {
            id += 1;
            if id == target_id {
                found = true;
                let indent = &line[..line.len() - trimmed.len()];
                let rest = &body[4..];
                already = if to_done { is_done } else { is_open };
                let clean = rest.strip_prefix("[P] ").unwrap_or(rest);
                desc = clean.trim().to_string();
                let checkbox = if to_done { "[x]" } else { "[ ]" };
                out_lines.push(format!("{indent}{bullet} {checkbox} {rest}"));
                continue;
            }
        }
        out_lines.push(line.to_string());
    }
    if !found {
        return None;
    }
    // Preserve trailing newline if the original had one.
    let mut new_content = out_lines.join("\n");
    if tasks_md.ends_with('\n') {
        new_content.push('\n');
    }
    Some((new_content, desc, already))
}

/// Outcome of [`complete`]: the task's cleaned description and whether it was
/// already checked (in which case nothing was written).
#[derive(Debug, Clone)]
pub struct CompleteOutcome {
    pub description: String,
    pub already: bool,
}

/// Complete a task — the single collaboration point shared by every tool path
/// (CLI `task done`, desktop checkbox): check the box, write tasks.md back,
/// record touched files, and stamp the work-started marker on the change's
/// first completion (idempotent via [`crate::inprogress::add`] — first stamp
/// wins, attribution the caller cannot supply is absent).
///
/// An already-done task is reported through the `already` flag with zero file
/// effects; presentation (CLI error vs. GUI idempotent success) stays with the
/// caller.
pub fn complete(
    store: &dyn Store,
    ws: &Workspace,
    change: &str,
    task_id: usize,
    identity: Option<&str>,
    agent: Option<&str>,
) -> Result<CompleteOutcome> {
    let text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change}'"))?;
    let total = parse(&text).len();
    let (new_content, desc, already) = mark_done(&text, task_id)
        .ok_or_else(|| anyhow::anyhow!("Task {task_id} not found (total: {total})"))?;
    if already {
        return Ok(CompleteOutcome { description: desc, already: true });
    }
    store.write_artifact(change, "tasks.md", &new_content)?;

    // Record touched files: only those not already attributed to an earlier task;
    // when nothing new is dirty, no entry is appended at all (matches Spectra).
    let mut record = TouchedRecord::load(ws, change);
    record.change = change.to_string();
    let seen = record.all_files();
    let files: Vec<String> = git_changed_files(&ws.root)
        .into_iter()
        .filter(|f| !seen.contains(f))
        .collect();
    if !files.is_empty() {
        record.touched.push(TouchedEntry {
            task_id: task_id.to_string(),
            task_desc: desc.clone(),
            files,
        });
        record.save(ws)?;
    }

    crate::inprogress::add(store, change, identity, agent)?;
    Ok(CompleteOutcome { description: desc, already: false })
}

/// Outcome of [`uncomplete`]: the task's cleaned description and whether it
/// was already unchecked (in which case nothing was written).
#[derive(Debug, Clone)]
pub struct UncompleteOutcome {
    pub description: String,
    pub already: bool,
}

/// Uncheck a task — the reverse verb shared by every tool path (CLI
/// `task undone`, desktop checkbox): flip the box back to `[ ]` and write
/// tasks.md. A pure state flip with zero side effects: touched records and the
/// work-started stamp are history and stay untouched, which is why the
/// signature takes no [`Workspace`].
///
/// An already-unchecked task is reported through the `already` flag with zero
/// file effects; presentation (CLI error vs. GUI idempotent success) stays
/// with the caller.
pub fn uncomplete(store: &dyn Store, change: &str, task_id: usize) -> Result<UncompleteOutcome> {
    let text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change}'"))?;
    let total = parse(&text).len();
    let (new_content, desc, already) = mark_undone(&text, task_id)
        .ok_or_else(|| anyhow::anyhow!("Task {task_id} not found (total: {total})"))?;
    if already {
        return Ok(UncompleteOutcome { description: desc, already: true });
    }
    store.write_artifact(change, "tasks.md", &new_content)?;
    Ok(UncompleteOutcome { description: desc, already: false })
}

/// Flip the id-th checkbox back to open. Returns (new_content, task_description, already_open)
/// or None if not found.
fn mark_undone(tasks_md: &str, target_id: usize) -> Option<(String, String, bool)> {
    flip_task(tasks_md, target_id, false)
}

// --- Touched-file tracking ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchedEntry {
    pub task_id: String,
    pub task_desc: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TouchedRecord {
    pub change: String,
    #[serde(default)]
    pub touched: Vec<TouchedEntry>,
}

impl TouchedRecord {
    pub fn load(ws: &Workspace, change: &str) -> TouchedRecord {
        let p = ws.touched_dir().join(format!("{change}.json"));
        match util::read_opt(&p) {
            Some(s) => serde_json::from_str(&s).unwrap_or(TouchedRecord {
                change: change.to_string(),
                touched: Vec::new(),
            }),
            None => TouchedRecord {
                change: change.to_string(),
                touched: Vec::new(),
            },
        }
    }

    pub fn save(&self, ws: &Workspace) -> std::io::Result<()> {
        let p = ws.touched_dir().join(format!("{}.json", self.change));
        let json = serde_json::to_string_pretty(self).unwrap_or_default();
        util::write_file(&p, &json)
    }

    /// Union of all files across entries (for @trace).
    pub fn all_files(&self) -> Vec<String> {
        let mut set = Vec::new();
        for e in &self.touched {
            for f in &e.files {
                if !set.contains(f) {
                    set.push(f.clone());
                }
            }
        }
        set
    }
}

/// Files changed in the git work tree, relative to root, forward-slashed.
///
/// Untracked directories are expanded to individual files (`-uall`). The spec directory and
/// speclink work directory are excluded, since @trace records *code* changes, not spec artifacts.
pub fn git_changed_files(root: &Path) -> Vec<String> {
    // Only when the project root is itself the git root (matches Spectra): a project
    // nested inside an ancestor repo records nothing, instead of walking up and
    // capturing dirty files from outside the project.
    if !root.join(".git").exists() {
        return Vec::new();
    }
    // NB: use the RAW (untrimmed) output — porcelain's first column is a significant leading space
    // for work-tree-modified files (" M path"); trimming it shifts the path by one character.
    let Some(out) = util::git_raw(root, &["status", "--porcelain", "-uall"]) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for raw_line in out.lines() {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        if line.len() < 4 {
            continue;
        }
        // Format: "XY <path>" possibly "XY <old> -> <new>"; path always starts at column 3.
        let path_part = &line[3..];
        let path = if let Some(idx) = path_part.find(" -> ") {
            &path_part[idx + 4..]
        } else {
            path_part
        };
        let path = path.trim_matches('"').replace('\\', "/");
        if path.is_empty() || path.ends_with('/') {
            continue; // skip directory entries
        }
        // Exclude spec artifacts, work files, and tool-scaffolding dirs from the code trace
        // (Spectra records CLAUDE.md / config but not .claude/.agents/.cursor/.gemini or .gitignore).
        if path.starts_with("openspec/")
            || path.starts_with(".speclink/")
            || path.starts_with(".git/")
            || path.starts_with(".claude/")
            || path.starts_with(".agents/")
            || path.starts_with(".cursor/")
            || path.starts_with(".gemini/")
            || path == ".gitignore"
        {
            continue;
        }
        files.push(path);
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    const META_UNSTARTED: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";
    const TASKS_TWO_OPEN: &str = "## 1. Group\n\n- [ ] 1.1 first task\n- [ ] 1.2 second task\n";

    /// Throwaway host workspace rooted in the OS temp dir; removed on drop.
    struct TempWs {
        ws: Workspace,
    }

    impl TempWs {
        fn new(tag: &str) -> TempWs {
            let dir = std::env::temp_dir().join(format!(
                "speclink-core-tasks-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempWs {
                ws: Workspace {
                    root: dir,
                    spec_dir_name: "openspec".to_string(),
                },
            }
        }

        /// Workspace root as a git repo carrying one dirty (untracked) code file.
        fn with_dirty_file(tag: &str, rel: &str) -> TempWs {
            let t = TempWs::new(tag);
            let ok = std::process::Command::new("git")
                .arg("-C")
                .arg(&t.ws.root)
                .args(["init", "-q"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            assert!(ok, "git init failed");
            let p = t.ws.root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, "content\n").unwrap();
            t
        }

        fn touched_json(&self, change: &str) -> Option<String> {
            util::read_opt(&self.ws.touched_dir().join(format!("{change}.json")))
        }
    }

    impl Drop for TempWs {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.ws.root);
        }
    }

    fn store_with(meta: &str, tasks_md: &str) -> TestStore {
        let store = TestStore::with_meta("demo", meta);
        store.put_artifact("demo", "tasks.md", tasks_md);
        store
    }

    #[test]
    fn complete_first_task_marks_stamps_and_records_touched() {
        let t = TempWs::with_dirty_file("first", "src/app.rs");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);

        let out = complete(&store, &t.ws, "demo", 1, Some("Tester <t@example.com>"), None).unwrap();

        assert!(!out.already);
        assert_eq!(out.description, "1.1 first task");
        let tasks = store.read_artifact("demo", "tasks.md").unwrap();
        assert!(tasks.contains("- [x] 1.1 first task"), "task 1 must be checked: {tasks}");
        assert!(tasks.contains("- [ ] 1.2 second task"), "task 2 must stay open: {tasks}");
        // Touched record gains this task's entry with the unclaimed dirty file.
        let json = t.touched_json("demo").expect("touched record written");
        let rec: TouchedRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rec.change, "demo");
        assert_eq!(rec.touched.len(), 1);
        assert_eq!(rec.touched[0].task_id, "1");
        assert_eq!(rec.touched[0].task_desc, "1.1 first task");
        assert!(rec.touched[0].files.contains(&"src/app.rs".to_string()));
        // Meta gains the work stamp; existing fields byte-for-byte preserved.
        let meta = store.meta("demo");
        assert!(
            meta.starts_with(META_UNSTARTED),
            "existing meta fields must be preserved verbatim: {meta}"
        );
        assert!(meta.contains(&format!("started_at: {}", util::today())));
        assert!(meta.contains("started_by: Tester <t@example.com>"));
        assert!(!meta.contains("started_with"));
    }

    #[test]
    fn complete_without_new_dirty_files_appends_no_touched_entry() {
        // No .git in the workspace root → git_changed_files is empty → nothing appended,
        // no record file created (matches the CLI's current semantics).
        let t = TempWs::new("nodirty");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
        let out = complete(&store, &t.ws, "demo", 1, None, None).unwrap();
        assert!(!out.already);
        assert_eq!(t.touched_json("demo"), None, "no unclaimed dirty files must append nothing");
    }

    #[test]
    fn complete_on_started_change_keeps_first_stamp_verbatim() {
        let started = format!(
            "{META_UNSTARTED}started_at: 2026-07-01\nstarted_by: First <first@example.com>\nstarted_with: claude\n"
        );
        let t = TempWs::new("started");
        let store = store_with(&started, TASKS_TWO_OPEN);
        complete(&store, &t.ws, "demo", 2, Some("Second <second@example.com>"), Some("codex"))
            .unwrap();
        assert_eq!(store.meta("demo"), started, "first stamp must be kept verbatim");
        assert_eq!(*store.meta_writes.borrow(), 0, "already-started change must not write meta");
    }

    #[test]
    fn complete_already_done_task_reports_already_without_any_file_effect() {
        let t = TempWs::with_dirty_file("already", "src/lib.rs");
        let tasks_md = "- [x] 1.1 finished\n- [ ] 1.2 open\n";
        let store = store_with(META_UNSTARTED, tasks_md);
        let out = complete(&store, &t.ws, "demo", 1, Some("Tester <t@example.com>"), None).unwrap();
        assert!(out.already);
        assert_eq!(out.description, "1.1 finished");
        assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), tasks_md);
        assert_eq!(*store.artifact_writes.borrow(), 0, "already-done must not rewrite tasks.md");
        assert_eq!(*store.meta_writes.borrow(), 0, "already-done must not stamp meta");
        assert_eq!(t.touched_json("demo"), None, "already-done must not record touched files");
    }

    #[test]
    fn complete_with_absent_identity_and_agent_stamps_only_started_at() {
        // Attribution follows the created_* rule: what the caller cannot attribute
        // is absent, not defaulted.
        let t = TempWs::new("absent");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
        complete(&store, &t.ws, "demo", 1, None, None).unwrap();
        let meta = store.meta("demo");
        assert!(meta.contains("started_at: "));
        assert!(!meta.contains("started_by"));
        assert!(!meta.contains("started_with"));
    }

    #[test]
    fn complete_out_of_range_task_id_errors_without_writes() {
        let t = TempWs::new("range");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
        let err = complete(&store, &t.ws, "demo", 5, None, None).unwrap_err();
        assert_eq!(err.to_string(), "Task 5 not found (total: 2)");
        assert_eq!(*store.artifact_writes.borrow(), 0);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    // Checked tasks in both bullet styles, one indented, one still open.
    const TASKS_MIXED_DONE: &str =
        "## 1. Group\n\n- [x] 1.1 first task\n    - [x] 1.2 indented task\n* [X] 1.3 star task\n- [ ] 1.4 open task\n";

    #[test]
    fn uncomplete_flips_only_target_line_preserving_indent_and_trailing_newline() {
        let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
        let out = uncomplete(&store, "demo", 2).unwrap();
        assert!(!out.already);
        assert_eq!(out.description, "1.2 indented task");
        assert_eq!(
            store.read_artifact("demo", "tasks.md").unwrap(),
            "## 1. Group\n\n- [x] 1.1 first task\n    - [ ] 1.2 indented task\n* [X] 1.3 star task\n- [ ] 1.4 open task\n"
        );
        // Pure state flip: tasks.md is the only write, meta stays byte-for-byte.
        assert_eq!(*store.artifact_writes.borrow(), 1);
        assert_eq!(*store.meta_writes.borrow(), 0, "uncomplete must not touch meta");
        assert_eq!(store.meta("demo"), META_UNSTARTED);
    }

    #[test]
    fn uncomplete_star_bullet_keeps_style_and_no_trailing_newline() {
        let tasks_md = "- [x] 1.1 first\n* [X] 1.2 star task";
        let store = store_with(META_UNSTARTED, tasks_md);
        let out = uncomplete(&store, "demo", 2).unwrap();
        assert!(!out.already);
        assert_eq!(out.description, "1.2 star task");
        assert_eq!(
            store.read_artifact("demo", "tasks.md").unwrap(),
            "- [x] 1.1 first\n* [ ] 1.2 star task",
            "star bullet style and absent trailing newline must be preserved"
        );
    }

    #[test]
    fn uncomplete_already_open_task_reports_already_without_any_file_effect() {
        let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
        let out = uncomplete(&store, "demo", 4).unwrap();
        assert!(out.already);
        assert_eq!(out.description, "1.4 open task");
        assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), TASKS_MIXED_DONE);
        assert_eq!(*store.artifact_writes.borrow(), 0, "already-open must not rewrite tasks.md");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn uncomplete_out_of_range_task_id_errors_without_writes() {
        let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
        let err = uncomplete(&store, "demo", 9).unwrap_err();
        assert_eq!(err.to_string(), "Task 9 not found (total: 4)");
        assert_eq!(*store.artifact_writes.borrow(), 0);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn uncomplete_missing_tasks_md_errors() {
        let store = TestStore::with_meta("demo", META_UNSTARTED);
        let err = uncomplete(&store, "demo", 1).unwrap_err();
        assert_eq!(err.to_string(), "tasks.md not found for change 'demo'");
    }
}

//! GUI 管理操作：change metadata 讀取與 active change 刪除。
//!
//! delete 是 desktop 層操作（引擎與 CLI 皆無 delete 動詞）——僅作用於 active change
//! 目錄、經路徑安全檢查、由 UI 以確認對話框把關。

use std::path::Path;

use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// 回傳 change 的 metadata（camelCase：createdBy/createdWith/created）。無此 change 回 `None`。
pub fn change_meta_at(root: &Path, change: &str) -> Option<Value> {
    let ctx = init_core_context(root)?;
    let change = ctx.store.find_change(change)?;
    Some(json!({
        "schema": change.meta.schema,
        "created": change.meta.created,
        "createdBy": change.meta.created_by,
        "createdWith": change.meta.created_with,
        "fromDiscussion": change.meta.from_discussion,
    }))
}

/// 刪除一個 active change 的目錄。不存在或名稱不安全時回 `Err`；成功後該 change 不再出現於清單。
pub fn delete_change_at(root: &Path, change: &str) -> Result<(), String> {
    if !crate::query::is_safe_path_param(change) {
        return Err(format!("invalid change name: {change}"));
    }
    let ctx = init_core_context(root)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let found = ctx
        .store
        .find_change(change)
        .ok_or_else(|| format!("change not found: {change}"))?;
    // found.dir 來自 store 的 active change 列舉（changes/<name>），不含 archive。
    std::fs::remove_dir_all(&found.dir).map_err(|e| format!("delete failed: {e}"))
}

/// 勾選/取消 tasks.md 的第 `ordinal`（1-based，僅計 checkbox 行）個任務。
/// 僅改該行的 `[ ]`/`[x]` 標記；ordinal 越界或無 tasks.md 回 `Err`。
pub fn set_task_done_at(root: &Path, change: &str, ordinal: usize, done: bool) -> Result<(), String> {
    edit_tasks(root, change, |lines, idx| {
        let line_no = *idx.get(ordinal.checked_sub(1).ok_or("ordinal must be 1-based")?)
            .ok_or_else(|| format!("task {ordinal} not found"))?;
        let line = &lines[line_no];
        let marker = if done { "[x]" } else { "[ ]" };
        let re = regex::Regex::new(r"^(\s*-\s*)\[[ xX]\]").unwrap();
        lines[line_no] = re.replace(line, format!("${{1}}{marker}")).into_owned();
        Ok(())
    })
}

/// 把第 `from` 個任務移到第 `to` 個位置（皆 1-based、僅計 checkbox 行）。
/// 只搬 checkbox 行本身，群組標題與其他行不動；越界回 `Err`。
pub fn move_task_at(root: &Path, change: &str, from: usize, to: usize) -> Result<(), String> {
    edit_tasks(root, change, |lines, idx| {
        let n = idx.len();
        if from == 0 || to == 0 || from > n || to > n {
            return Err(format!("task index out of range (1..={n})"));
        }
        if from == to {
            return Ok(());
        }
        let moved = lines.remove(idx[from - 1]);
        // 移除後重算剩餘 checkbox 行位置，決定插入點。
        let idx2: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| is_task_line(l))
            .map(|(i, _)| i)
            .collect();
        let insert_at = if to - 1 < idx2.len() { idx2[to - 1] } else { idx2.last().map(|i| i + 1).unwrap_or(lines.len()) };
        lines.insert(insert_at, moved);
        Ok(())
    })
}

fn is_task_line(line: &str) -> bool {
    regex::Regex::new(r"^\s*-\s*\[[ xX]\]\s").unwrap().is_match(line)
}

/// 讀 tasks.md → 以（行陣列, checkbox 行索引）呼叫編輯器 → 寫回。
fn edit_tasks(
    root: &Path,
    change: &str,
    edit: impl FnOnce(&mut Vec<String>, &[usize]) -> Result<(), String>,
) -> Result<(), String> {
    if !crate::query::is_safe_path_param(change) {
        return Err(format!("invalid change name: {change}"));
    }
    let ctx = init_core_context(root)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let text = ctx
        .store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| format!("tasks.md not found for change: {change}"))?;
    // 保留檔尾換行狀態
    let had_trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let idx: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| is_task_line(l))
        .map(|(i, _)| i)
        .collect();
    edit(&mut lines, &idx)?;
    let mut out = lines.join("\n");
    if had_trailing_newline {
        out.push('\n');
    }
    ctx.store
        .write_artifact(change, "tasks.md", &out)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..")
    }

    /// 建含一個 active change 的暫存 fixture 專案。
    fn fixture_with_change(tag: &str, name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("speclink-manage-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let dir = root.join("openspec").join("changes").join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("proposal.md"), "## Why\ntest\n").unwrap();
        fs::write(
            dir.join(".openspec.yaml"),
            "schema: spec-driven\ncreated: 2026-07-05\ncreated_by: momo\ncreated_with: claude\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn change_meta_returns_camel_case_fields() {
        let meta = change_meta_at(&repo_root(), "desktop-shell-and-browser").expect("meta exists");
        assert!(meta.get("created").is_some());
        // createdBy/createdWith 欄位存在（值可為 null），確認 camelCase 命名
        assert!(meta.get("createdBy").is_some() || meta.get("createdWith").is_some());
    }

    #[test]
    fn change_meta_unknown_change_is_none() {
        assert!(change_meta_at(&repo_root(), "no-such-change-xyz").is_none());
    }

    #[test]
    fn delete_change_removes_active_change() {
        let root = fixture_with_change("del", "doomed-change");
        let ctx = crate::init_core_context(&root).unwrap();
        assert!(ctx.store.change_exists("doomed-change"));
        delete_change_at(&root, "doomed-change").expect("delete ok");
        let ctx2 = crate::init_core_context(&root).unwrap();
        assert!(!ctx2.store.change_exists("doomed-change"));
        let _ = fs::remove_dir_all(&root);
    }

    fn fixture_with_tasks(tag: &str) -> PathBuf {
        let root = fixture_with_change(tag, "task-change");
        let dir = root.join("openspec").join("changes").join("task-change");
        fs::write(
            dir.join("tasks.md"),
            "## 1. Group A\n\n- [ ] 1.1 first\n- [x] 1.2 second\n\n## 2. Group B\n\n- [ ] 2.1 third\n",
        )
        .unwrap();
        root
    }

    fn read_tasks(root: &PathBuf) -> String {
        fs::read_to_string(
            root.join("openspec").join("changes").join("task-change").join("tasks.md"),
        )
        .unwrap()
    }

    #[test]
    fn set_task_done_toggles_only_the_target_line() {
        let root = fixture_with_tasks("toggle");
        set_task_done_at(&root, "task-change", 1, true).expect("check first");
        set_task_done_at(&root, "task-change", 2, false).expect("uncheck second");
        let text = read_tasks(&root);
        assert!(text.contains("- [x] 1.1 first"));
        assert!(text.contains("- [ ] 1.2 second"));
        assert!(text.contains("- [ ] 2.1 third"), "untouched line intact");
        assert!(text.contains("## 1. Group A"), "group headings intact");
        assert!(set_task_done_at(&root, "task-change", 99, true).is_err(), "out of range errors");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn move_task_reorders_checkbox_lines_only() {
        let root = fixture_with_tasks("move");
        // 把第 3 個任務移到第 1 位
        move_task_at(&root, "task-change", 3, 1).expect("move up");
        let text = read_tasks(&root);
        let tasks: Vec<&str> = text.lines().filter(|l| l.trim_start().starts_with("- [")).collect();
        assert_eq!(tasks[0], "- [ ] 2.1 third");
        assert_eq!(tasks[1], "- [ ] 1.1 first");
        assert_eq!(tasks[2], "- [x] 1.2 second");
        // 群組標題數不變
        assert_eq!(text.matches("## ").count(), 2);
        assert!(move_task_at(&root, "task-change", 0, 1).is_err(), "0 index errors");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn delete_change_unknown_or_unsafe_errors() {
        let root = fixture_with_change("safe", "kept-change");
        assert!(delete_change_at(&root, "no-such-xyz").is_err(), "unknown change errors");
        assert!(delete_change_at(&root, "../specs").is_err(), "traversal rejected");
        // 原 change 不受影響
        let ctx = crate::init_core_context(&root).unwrap();
        assert!(ctx.store.change_exists("kept-change"));
        let _ = fs::remove_dir_all(&root);
    }
}

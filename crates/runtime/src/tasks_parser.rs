//! `tasks.md` 解析與單檔 in-memory 更新邏輯（無 I/O）。
//!
//! 對應 spec `Tasks.md task id format` 與 spec `Atomic tasks.md update for task done`
//! 的 parsing 與 update 行為。本模組無 filesystem 操作，僅在純字串層級工作。

use provider::model::TaskStatus;
use thiserror::Error;

/// `tasks.md` 解析失敗。
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("tasks.md parse error: {message}")]
pub struct TasksParseError {
    /// 解析失敗描述（包含 1-based 行號與原始內容）。
    pub message: String,
}

/// 解析後的 tasks.md 結構。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTasks {
    /// 依出現順序排列的 section。
    pub sections: Vec<TaskSection>,
}

/// `tasks.md` 內的 `## N. <heading>` section。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSection {
    /// Section 編號（heading 中的 `N`）。
    pub number: u32,
    /// Section heading 文字（不含 `## N. ` 前綴）。
    pub heading: String,
    /// 此 section 內的 task items（依出現順序）。
    pub tasks: Vec<TaskItem>,
}

/// `tasks.md` 內的單一 task checkbox 行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    /// 完整 task id（`"N.M"` 形式）。
    pub task_id: String,
    /// 當前 checkbox 狀態。
    pub status: TaskStatus,
    /// Task 描述：不含 checkbox / task id 前綴，保留 `[P]` marker。
    pub description: String,
    /// 此 checkbox 在 tasks.md 中的行號（1-based）。
    pub line_number: usize,
}

/// 檢驗 task id 是否符合 `^\d+\.\d+$`：
/// 兩段十進位整數、無前導零（除非整段就是 `0`，但 0 不允許作 section / task 序號）、
/// 無第三層、無空白、無字母。
///
/// 對應 spec `Tasks.md task id format`。
pub fn is_valid_task_id(s: &str) -> bool {
    let Some((left, right)) = s.split_once('.') else {
        return false;
    };
    is_valid_segment(left) && is_valid_segment(right) && !right.contains('.')
}

fn is_valid_segment(seg: &str) -> bool {
    if seg.is_empty() {
        return false;
    }
    // 拒絕前導 0（單一字元 "0" 也不允許 — task / section 編號自 1 起算）
    if seg.starts_with('0') {
        return false;
    }
    seg.chars().all(|c| c.is_ascii_digit())
}

/// 解析 `tasks.md` 內容。
///
/// 規則：
/// - section heading 行：`^## N\. <heading>$`（N 為正整數、無前導 0）
/// - checkbox 行：`^- \[( |x)\] N\.M <description>$`
/// - `N` 必須與目前 section 的 `N` 相同
/// - 出現三層 task id（`N.M.P`）即 `tasks.parse_error`
/// - 出現孤立 checkbox（在任何 section heading 之前） 即 `tasks.parse_error`
pub fn parse_tasks(content: &str) -> Result<ParsedTasks, TasksParseError> {
    let mut sections: Vec<TaskSection> = Vec::new();
    let mut current_section: Option<TaskSection> = None;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        // Section heading 偵測
        if let Some(rest) = line.strip_prefix("## ") {
            // 預期形式 "N. heading"
            let Some(dot_idx) = rest.find('.') else {
                // 非 task section heading（例如 `## Notes`） — 終止當前 section，但允許後續再啟動。
                if let Some(s) = current_section.take() {
                    sections.push(s);
                }
                continue;
            };
            let (num_str, after) = rest.split_at(dot_idx);
            let after = after.strip_prefix('.').unwrap_or(after);
            if !is_valid_segment(num_str) {
                // 不是合法的 section number — 視為非 task heading
                if let Some(s) = current_section.take() {
                    sections.push(s);
                }
                continue;
            }
            let heading_text = after.strip_prefix(' ').unwrap_or(after).to_string();
            let number: u32 = num_str.parse().map_err(|_| TasksParseError {
                message: format!("line {line_number}: section number not a u32: '{num_str}'"),
            })?;
            if let Some(s) = current_section.take() {
                sections.push(s);
            }
            current_section = Some(TaskSection {
                number,
                heading: heading_text,
                tasks: Vec::new(),
            });
            continue;
        }

        // Checkbox 偵測
        if let Some(rest) = match_checkbox_prefix(line) {
            // rest 已去掉 "- [x] " / "- [ ] "
            let (status, body) = rest;
            // 解析 "N.M ..." — 不允許三層 N.M.P
            let space_idx = body.find(' ').ok_or_else(|| TasksParseError {
                message: format!("line {line_number}: checkbox missing description: '{line}'"),
            })?;
            let id_str = &body[..space_idx];
            let desc = &body[space_idx + 1..];
            // 拒絕三層
            if id_str.matches('.').count() != 1 {
                return Err(TasksParseError {
                    message: format!("line {line_number}: task id must be N.M (got '{id_str}')"),
                });
            }
            if !is_valid_task_id(id_str) {
                return Err(TasksParseError {
                    message: format!("line {line_number}: invalid task id '{id_str}'"),
                });
            }
            let (n_str, _) = id_str.split_once('.').unwrap();
            let n: u32 = n_str.parse().unwrap();
            let section = current_section.as_mut().ok_or_else(|| TasksParseError {
                message: format!(
                    "line {line_number}: task '{id_str}' has no preceding section heading"
                ),
            })?;
            if section.number != n {
                return Err(TasksParseError {
                    message: format!(
                        "line {line_number}: task '{id_str}' does not match current section {}",
                        section.number
                    ),
                });
            }
            section.tasks.push(TaskItem {
                task_id: id_str.to_string(),
                status,
                description: desc.to_string(),
                line_number,
            });
            continue;
        }
        // 其他行（空白、prose 等）：忽略；不終止 current_section。
    }
    if let Some(s) = current_section.take() {
        sections.push(s);
    }
    // 檢查 task id 與 section number 在整份檔案內唯一性
    let mut seen_section: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut seen_task: std::collections::HashSet<String> = std::collections::HashSet::new();
    for s in &sections {
        if !seen_section.insert(s.number) {
            return Err(TasksParseError {
                message: format!("duplicate section number '{}'", s.number),
            });
        }
        for t in &s.tasks {
            if !seen_task.insert(t.task_id.clone()) {
                return Err(TasksParseError {
                    message: format!("duplicate task id '{}'", t.task_id),
                });
            }
        }
    }
    Ok(ParsedTasks { sections })
}

fn match_checkbox_prefix(line: &str) -> Option<(TaskStatus, &str)> {
    if let Some(rest) = line.strip_prefix("- [ ] ") {
        return Some((TaskStatus::Todo, rest));
    }
    if let Some(rest) = line.strip_prefix("- [x] ") {
        return Some((TaskStatus::Done, rest));
    }
    None
}

/// 將 `tasks.md` 中對應 task id 的 checkbox 從 `[ ]` 改為 `[x]`（idempotent）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateResult {
    /// 套用後的完整 tasks.md 內容。
    pub new_content: String,
    /// 套用前的 task 狀態。
    pub previous_status: TaskStatus,
    /// Task 描述（與 [`TaskItem::description`] 相同，保留 `[P]` marker）。
    pub task_description: String,
}

/// `mark_task_done_in_content` 的錯誤型別。
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TasksUpdateError {
    /// task id 格式不符。
    #[error("invalid task id: '{task_id}'")]
    InvalidId {
        /// 不合法的 task id 原始字串。
        task_id: String,
    },
    /// task id 在 tasks.md 中找不到對應 checkbox。
    #[error("task '{task_id}' not found")]
    NotFound {
        /// 缺少的 task id。
        task_id: String,
    },
    /// tasks.md 解析失敗。
    #[error(transparent)]
    Parse(#[from] TasksParseError),
}

/// 在純字串內容上將指定 task 的 checkbox 翻為 done；不寫檔。
///
/// 行為對應 spec `Atomic tasks.md update for task done`：
/// 若該 task 已是 `[x]`，回 `previous_status = Done` 且 `new_content == content`；
/// 否則僅替換該行中第一個 `[ ]` 為 `[x]`，其餘 byte 完全保留（包含 line endings、空白）。
pub fn mark_task_done_in_content(
    content: &str,
    task_id: &str,
) -> Result<UpdateResult, TasksUpdateError> {
    if !is_valid_task_id(task_id) {
        return Err(TasksUpdateError::InvalidId {
            task_id: task_id.to_string(),
        });
    }
    let parsed = parse_tasks(content).map_err(TasksUpdateError::Parse)?;
    let item = parsed
        .sections
        .iter()
        .flat_map(|s| s.tasks.iter())
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| TasksUpdateError::NotFound {
            task_id: task_id.to_string(),
        })?;

    if item.status == TaskStatus::Done {
        return Ok(UpdateResult {
            new_content: content.to_string(),
            previous_status: TaskStatus::Done,
            task_description: item.description.clone(),
        });
    }

    let new_content = replace_first_unchecked_on_line(content, item.line_number);
    Ok(UpdateResult {
        new_content,
        previous_status: TaskStatus::Todo,
        task_description: item.description.clone(),
    })
}

/// 將指定 1-based 行號上的第一個 `[ ]` 換成 `[x]`，其他 byte 不動。
///
/// 行為對應 spec `Atomic tasks.md update for task done` 的「只動 checkbox 字元」條款。
fn replace_first_unchecked_on_line(content: &str, line_number: usize) -> String {
    // 採用 byte-level 線性掃描：找到第 `line_number` 行起點與終點（不含換行）後，
    // 對該範圍做一次 `replacen("[ ]", "[x]", 1)`，並把結果拼回原內容。
    let mut start = 0usize;
    let mut current_line = 1usize;
    let bytes = content.as_bytes();
    while current_line < line_number && start < bytes.len() {
        match content[start..].find('\n') {
            Some(rel) => {
                start += rel + 1;
                current_line += 1;
            }
            None => {
                // 找不到行尾代表 content 行數不足
                return content.to_string();
            }
        }
    }
    if current_line != line_number {
        return content.to_string();
    }
    let line_end = content[start..]
        .find('\n')
        .map(|rel| start + rel)
        .unwrap_or(bytes.len());
    let prefix = &content[..start];
    let line = &content[start..line_end];
    let suffix = &content[line_end..];
    let new_line = line.replacen("[ ]", "[x]", 1);
    format!("{prefix}{new_line}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- is_valid_task_id (task 4.1) --

    #[test]
    fn valid_task_id_accepts_canonical() {
        assert!(is_valid_task_id("1.1"));
        assert!(is_valid_task_id("10.3"));
        assert!(is_valid_task_id("100.50"));
    }

    #[test]
    fn valid_task_id_rejects_malformed() {
        for s in [
            "1", "1.1.2", "01.1", "1.0", "0.1", "1.", ".1", "", " 1.1", "1.1 ", "a.1", "1.a",
            "1..1",
        ] {
            assert!(
                !is_valid_task_id(s),
                "id '{s}' should be rejected but was accepted"
            );
        }
    }

    // -- parse_tasks (task 5.1, 5.2) --

    #[test]
    fn parse_tasks_happy_path() {
        let content = "## 1. Setup\n\n- [ ] 1.1 Install deps\n- [ ] 1.2 Configure env\n\n## 2. Build\n\n- [x] 2.1 Compile\n";
        let parsed = parse_tasks(content).expect("ok");
        assert_eq!(parsed.sections.len(), 2);
        assert_eq!(parsed.sections[0].number, 1);
        assert_eq!(parsed.sections[0].heading, "Setup");
        assert_eq!(parsed.sections[0].tasks.len(), 2);
        assert_eq!(parsed.sections[0].tasks[0].task_id, "1.1");
        assert_eq!(parsed.sections[0].tasks[0].status, TaskStatus::Todo);
        assert_eq!(parsed.sections[0].tasks[0].description, "Install deps");
        assert_eq!(parsed.sections[0].tasks[0].line_number, 3);
        assert_eq!(parsed.sections[0].tasks[1].line_number, 4);
        assert_eq!(parsed.sections[1].number, 2);
        assert_eq!(parsed.sections[1].tasks[0].task_id, "2.1");
        assert_eq!(parsed.sections[1].tasks[0].status, TaskStatus::Done);
        assert_eq!(parsed.sections[1].tasks[0].description, "Compile");
        assert_eq!(parsed.sections[1].tasks[0].line_number, 8);
    }

    #[test]
    fn parse_tasks_three_level_id_is_error() {
        let content = "## 1. Setup\n\n- [ ] 1.1.1 Subtask\n";
        let err = parse_tasks(content).expect_err("err");
        assert!(err.message.contains("N.M"), "{err:?}");
    }

    #[test]
    fn parse_tasks_section_mismatch_is_error() {
        let content = "## 1. Setup\n\n- [ ] 2.1 Mismatch\n";
        let err = parse_tasks(content).expect_err("err");
        assert!(err.message.contains("does not match"), "{err:?}");
    }

    #[test]
    fn parse_tasks_floating_task_is_error() {
        let content = "- [ ] 1.1 Floating\n";
        let err = parse_tasks(content).expect_err("err");
        assert!(err.message.contains("no preceding section"), "{err:?}");
    }

    #[test]
    fn parse_tasks_duplicate_task_id_is_error() {
        let content = "## 1. Setup\n\n- [ ] 1.1 First\n- [ ] 1.1 Dupe\n";
        let err = parse_tasks(content).expect_err("err");
        assert!(err.message.contains("duplicate"), "{err:?}");
    }

    // -- mark_task_done_in_content (task 6.1, 6.2, 6.3, 6.4, 6.5) --

    #[test]
    fn mark_task_done_happy_path() {
        let content = "## 1. Setup\n\n- [ ] 1.1 First\n- [ ] 1.2 Second\n";
        let result = mark_task_done_in_content(content, "1.1").expect("ok");
        assert_eq!(
            result.new_content,
            "## 1. Setup\n\n- [x] 1.1 First\n- [ ] 1.2 Second\n"
        );
        assert_eq!(result.previous_status, TaskStatus::Todo);
        assert_eq!(result.task_description, "First");
    }

    #[test]
    fn mark_task_done_idempotent_when_already_done() {
        let content = "## 1. Setup\n\n- [x] 1.1 Done\n";
        let result = mark_task_done_in_content(content, "1.1").expect("ok");
        assert_eq!(result.previous_status, TaskStatus::Done);
        assert_eq!(result.new_content, content);
        assert_eq!(result.task_description, "Done");
    }

    #[test]
    fn mark_task_done_not_found() {
        let content = "## 1. Setup\n\n- [ ] 1.1 First\n";
        let err = mark_task_done_in_content(content, "1.99").expect_err("err");
        assert!(matches!(err, TasksUpdateError::NotFound { .. }));
    }

    #[test]
    fn mark_task_done_invalid_id() {
        let content = "## 1. Setup\n\n- [ ] 1.1 First\n";
        let err = mark_task_done_in_content(content, "1.1.2").expect_err("err");
        assert!(matches!(err, TasksUpdateError::InvalidId { .. }));
    }

    #[test]
    fn mark_task_done_preserves_parallel_marker() {
        let content = "## 2. Refactor\n\n- [ ] 2.3 [P] Refactor parser\n";
        let result = mark_task_done_in_content(content, "2.3").expect("ok");
        assert_eq!(
            result.new_content,
            "## 2. Refactor\n\n- [x] 2.3 [P] Refactor parser\n"
        );
        assert_eq!(result.task_description, "[P] Refactor parser");
    }

    #[test]
    fn mark_task_done_multiline_description_keeps_only_first_line() {
        // 第二行為「縮排說明」— 不應被視為 task description 的一部分。
        let content = "## 1. Setup\n\n- [ ] 1.1 First\n  Continuation note\n";
        let result = mark_task_done_in_content(content, "1.1").expect("ok");
        assert_eq!(result.task_description, "First");
        assert_eq!(
            result.new_content,
            "## 1. Setup\n\n- [x] 1.1 First\n  Continuation note\n"
        );
    }

    #[test]
    fn mark_task_done_preserves_trailing_text_after_target_line() {
        // 確保只動 target 行
        let content = "## 1. Setup\n\n- [ ] 1.1 Task\nExtra prose\n  indented note\n";
        let result = mark_task_done_in_content(content, "1.1").expect("ok");
        assert_eq!(
            result.new_content,
            "## 1. Setup\n\n- [x] 1.1 Task\nExtra prose\n  indented note\n"
        );
    }
}

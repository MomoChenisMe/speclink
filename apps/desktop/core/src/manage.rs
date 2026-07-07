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
        "startedAt": change.meta.started_at,
        "startedBy": change.meta.started_by,
        "startedWith": change.meta.started_with,
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
/// ordinal 越界或無 tasks.md 回 `Err`。
///
/// done=true 走引擎的任務完成協作函式——與 CLI `task done` 相同的檔案效果
/// （勾章、touched 記錄、首次完成蓋開工章；identity 沿 git 身分、agent 缺席）。
/// 與 CLI 唯一的差異：任務已完成時視為冪等成功（GUI toggle 語意下重複 done=true
/// 只可能來自競態，不以錯誤打斷使用者），不寫任何檔案。
/// done=false（取消勾選）維持桌面行編輯，不蓋章、不記 touched。
pub fn set_task_done_at(root: &Path, change: &str, ordinal: usize, done: bool) -> Result<(), String> {
    if done {
        if !crate::query::is_safe_path_param(change) {
            return Err(format!("invalid change name: {change}"));
        }
        let ctx = init_core_context(root)
            .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
        let identity = speclink_core::util::git_identity(&ctx.workspace.root);
        speclink_core::tasks::complete(
            &ctx.store,
            &ctx.workspace,
            change,
            ordinal,
            identity.as_deref(),
            None,
        )
        .map(|_| ()) // already → 冪等成功（引擎保證零檔案效果）
        .map_err(|e| e.to_string())
    } else {
        edit_tasks(root, change, |lines, idx| {
            let line_no = *idx
                .get(ordinal.checked_sub(1).ok_or("ordinal must be 1-based")?)
                .ok_or_else(|| format!("task {ordinal} not found"))?;
            let line = &lines[line_no];
            let re = regex::Regex::new(r"^(\s*-\s*)\[[ xX]\]").unwrap();
            lines[line_no] = re.replace(line, "${1}[ ]").into_owned();
            Ok(())
        })
    }
}

/// 把第 `from` 個任務移到以第 `to` 個任務為錨的位置（皆 1-based、僅計 checkbox 行）。
/// 只搬 checkbox 行本身，群組標題與其他行不動；越界回 `Err`。
/// `before`：None＝方向推斷（向上插錨前、向下插錨後——組界時貼齊手勢方向的群組）；
/// Some(true)＝明確插於錨任務行之前（跨過群組標題即成為錨所屬群組的組首）；
/// Some(false)＝明確插於錨任務行之後。搬移成功後重算編號前綴（design D2），一次寫回。
pub fn move_task_at(
    root: &Path,
    change: &str,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<(), String> {
    edit_tasks(root, change, |lines, idx| {
        let n = idx.len();
        if from == 0 || to == 0 || from > n || to > n {
            return Err(format!("task index out of range (1..={n})"));
        }
        if from == to {
            return Ok(());
        }
        let moved = lines.remove(idx[from - 1]);
        // 移除後重算剩餘 checkbox 行位置；錨任務（原第 to 個）在移除後的 0-based 位置。
        let idx2: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| is_task_line(l))
            .map(|(i, _)| i)
            .collect();
        let anchor = if to < from { to - 1 } else { to - 2 };
        // 側別決定貼邊；未指定時以方向推斷（向上插前、向下插後）——否則向下拖到
        // 群組末位會越過群組邊界、被吞進下一群組（順序相同、群組歸屬錯誤）。
        let insert_before = before.unwrap_or(to < from);
        let insert_at = if insert_before {
            idx2[anchor]
        } else {
            idx2[anchor] + 1
        };
        lines.insert(insert_at, moved);
        renumber_task_prefixes(lines);
        Ok(())
    })
}

/// 重算任務編號前綴（design D2）：群組編號取自「## N.」標題自身的數字；
/// 群組內第 k 個 checkbox 行、文字以「數字.數字＋空白」開頭者，前綴重寫為「N.k」。
/// 其餘一律逐字元保留——無前綴、子版號（1.2.3）、無數字標題的群組、首個標題前的
/// 任務、群組標題與非 checkbox 行都不改寫（重編號永不弄丟使用者文字）。
fn renumber_task_prefixes(lines: &mut [String]) {
    let prefix_re = regex::Regex::new(r"^(\s*-\s*\[[ xX]\]\s+)(\d+\.\d+)(\s)").unwrap();
    let mut group: Option<u64> = None;
    let mut k = 0usize;
    for line in lines.iter_mut() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("## ") {
            let rest = rest.trim_start();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            group = if !digits.is_empty() && rest[digits.len()..].starts_with('.') {
                digits.parse().ok()
            } else {
                None
            };
            k = 0;
            continue;
        }
        if is_task_line(line) {
            k += 1;
            if let Some(g) = group {
                *line = prefix_re.replace(line, format!("${{1}}{g}.{k}${{3}}")).into_owned();
            }
        }
    }
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
    fn change_meta_returns_camel_case_fields_including_started_station() {
        let fx = crate::testfixture::FixtureRoot::new("m-meta");
        fx.add_change(
            "demo",
            "schema: spec-driven\ncreated: 2026-07-05\ncreated_by: momo\ncreated_with: claude\nstarted_at: 2026-07-06\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n",
        );
        let meta = change_meta_at(fx.root(), "demo").expect("meta exists");
        assert_eq!(meta["created"], "2026-07-05");
        assert_eq!(meta["createdBy"], "momo");
        assert_eq!(meta["createdWith"], "claude");
        // 抽屜「誰於何時開工」的資料來源：started 站以 camelCase 帶出。
        assert_eq!(meta["startedAt"], "2026-07-06");
        assert_eq!(meta["startedBy"], "Worker <w@example.com>");
        assert_eq!(meta["startedWith"], "claude");
    }

    #[test]
    fn change_meta_without_started_fields_yields_nulls() {
        let fx = crate::testfixture::FixtureRoot::new("m-meta-old");
        fx.add_change("demo", "schema: spec-driven\ncreated: 2026-07-05\n");
        let meta = change_meta_at(fx.root(), "demo").expect("meta exists");
        assert!(meta.get("startedAt").is_some(), "key present");
        assert!(meta["startedAt"].is_null());
        assert!(meta["startedBy"].is_null());
        assert!(meta["startedWith"].is_null());
    }

    #[test]
    fn change_meta_unknown_change_is_none() {
        let fx = crate::testfixture::FixtureRoot::new("m-meta-unknown");
        fx.add_change("demo", "schema: spec-driven\n");
        assert!(change_meta_at(fx.root(), "no-such-change-xyz").is_none());
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
        // 把第 3 個任務移到第 1 位——落入群組 1 並依新順序重編號。
        move_task_at(&root, "task-change", 3, 1, None).expect("move up");
        let text = read_tasks(&root);
        let tasks: Vec<&str> = text.lines().filter(|l| l.trim_start().starts_with("- [")).collect();
        assert_eq!(tasks[0], "- [ ] 1.1 third");
        assert_eq!(tasks[1], "- [ ] 1.2 first");
        assert_eq!(tasks[2], "- [x] 1.3 second");
        // 群組標題數不變
        assert_eq!(text.matches("## ").count(), 2);
        assert!(move_task_at(&root, "task-change", 0, 1, None).is_err(), "0 index errors");
        let _ = fs::remove_dir_all(&root);
    }

    fn fixture_with_tasks_md(tag: &str, tasks: &str) -> PathBuf {
        let root = fixture_with_change(tag, "task-change");
        let dir = root.join("openspec").join("changes").join("task-change");
        fs::write(dir.join("tasks.md"), tasks).unwrap();
        root
    }

    fn task_lines(root: &PathBuf) -> Vec<String> {
        read_tasks(root)
            .lines()
            .filter(|l| l.trim_start().starts_with("- ["))
            .map(|l| l.to_string())
            .collect()
    }

    #[test]
    fn move_within_group_renumbers_prefixes_per_spec_example() {
        // spec Example「組內移動重編號」：把 1.1 甲拖到末位 → 乙丙甲，前綴重寫 1.1/1.2/1.3。
        let root = fixture_with_tasks_md(
            "renum-within",
            "## 1. 群組\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n",
        );
        move_task_at(&root, "task-change", 1, 3, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲"],
            "prefixes must follow the new order"
        );
        assert!(read_tasks(&root).contains("## 1. 群組"), "group heading untouched");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn move_across_groups_takes_the_new_groups_numbering() {
        // 把群組 1 的乙拖到群組 2 的丙之後 → 乙取得新群組編號 2.2，丁後移 2.3。
        let root = fixture_with_tasks_md(
            "renum-cross",
            "## 1. 前段\n\n說明文字原樣保留。\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
        );
        move_task_at(&root, "task-change", 2, 3, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.1 甲", "- [ ] 2.1 丙", "- [ ] 2.2 乙", "- [ ] 2.3 丁"]
        );
        let text = read_tasks(&root);
        assert!(text.contains("說明文字原樣保留。"), "prose lines byte-identical");
        assert!(text.contains("## 1. 前段") && text.contains("## 2. 後段"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn tasks_without_numeric_prefix_keep_their_text_verbatim() {
        let root = fixture_with_tasks_md(
            "renum-noprefix",
            "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 補充說明不帶編號\n- [ ] 1.2 乙\n",
        );
        move_task_at(&root, "task-change", 3, 1, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.1 乙", "- [ ] 1.2 甲", "- [ ] 補充說明不帶編號"],
            "unprefixed task text must stay untouched while others renumber"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn groups_without_numeric_heading_are_not_renumbered() {
        let root = fixture_with_tasks_md(
            "renum-unnumbered",
            "## 準備\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n",
        );
        move_task_at(&root, "task-change", 1, 2, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
            "a heading without a numeric prefix must leave its tasks' numbers alone"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn before_true_crosses_the_heading_and_becomes_group_head() {
        // spec Example「標題落點成組首」：把 1.2 乙拖到「## 2. 後段」標題上——
        // 前端解析為（to=組首任務 ordinal, before=true）→ 乙插於丙行之前、
        // 跨過標題成為群組 2 第一個任務。
        let root = fixture_with_tasks_md(
            "side-head",
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
        );
        move_task_at(&root, "task-change", 2, 3, Some(true)).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.1 甲", "- [ ] 2.1 乙", "- [ ] 2.2 丙", "- [ ] 2.3 丁"],
            "before=true must insert ahead of the anchor line, across the heading"
        );
        let text = read_tasks(&root);
        let g2 = text.split("## 2. 後段").nth(1).unwrap();
        assert!(g2.contains("乙"), "乙 must live under group 2: {text}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn before_false_explicitly_inserts_after_the_anchor() {
        // 明確側別覆蓋方向推斷：向上移動＋before=false → 落在錨任務之後。
        let root = fixture_with_tasks_md(
            "side-after",
            "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n- [ ] 1.3 丙\n",
        );
        move_task_at(&root, "task-change", 3, 1, Some(false)).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.1 甲", "- [ ] 1.2 丙", "- [ ] 1.3 乙"],
            "before=false anchors after task 1 even on an upward move"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn side_parameter_keeps_out_of_range_errors_and_none_inference() {
        let root = fixture_with_tasks_md(
            "side-oob",
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n\n## 2. 後段\n\n- [ ] 2.1 丁\n",
        );
        let before_text = read_tasks(&root);
        assert!(move_task_at(&root, "task-change", 9, 1, Some(true)).is_err());
        assert!(move_task_at(&root, "task-change", 0, 2, Some(false)).is_err());
        assert_eq!(read_tasks(&root), before_text, "failed side moves must not rewrite");
        // None 維持方向推斷：向下拖到群組末位仍留在原群組（既有行為）。
        move_task_at(&root, "task-change", 1, 3, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲", "- [ ] 2.1 丁"]
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn downward_move_to_group_end_stays_in_the_origin_group() {
        // 真實視窗抓到的邊界缺陷：向下拖到群組末位時，「插在下一個 checkbox 前」
        // 會把任務吞進下一群組——向下移動必須落在目標 checkbox 之後（留在原群組）。
        let root = fixture_with_tasks_md(
            "renum-boundary",
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n\n## 2. 後段\n\n- [ ] 2.1 丁\n- [ ] 2.2 戊\n",
        );
        move_task_at(&root, "task-change", 1, 3, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲", "- [ ] 2.1 丁", "- [ ] 2.2 戊"],
            "a downward move onto the last task of a group must not leak into the next group"
        );
        let text = read_tasks(&root);
        let g2 = text.split("## 2. 後段").nth(1).unwrap();
        assert!(!g2.contains("甲"), "甲 must stay under group 1: {text}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sub_versioned_prefixes_are_never_mangled() {
        // sharp-edge：`1.2.3` 的「1.2」後面是「.」不是空白——比對要求前綴後必須
        // 是空白，否則改寫會把子版號絞成「2.1.3」這種殘骸。
        let root = fixture_with_tasks_md(
            "renum-subver",
            "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2.3 子版號文字\n",
        );
        move_task_at(&root, "task-change", 2, 1, None).expect("move ok");
        // 子版號列逐字元保留；甲成為組內第 2 個 checkbox，依「群組編號.組內序」重寫 1.2。
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.2.3 子版號文字", "- [ ] 1.2 甲"],
            "sub-versioned prefixes must be preserved verbatim"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn overflowing_group_numbers_fail_soft_to_no_renumber() {
        // sharp-edge：標題數字超出 u64——解析失敗即視為無數字標題，任務保留原文。
        let root = fixture_with_tasks_md(
            "renum-overflow",
            "## 99999999999999999999999. 巨數\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n",
        );
        move_task_at(&root, "task-change", 1, 2, None).expect("move ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
            "unparseable group numbers must not rewrite anything"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn set_task_done_does_not_renumber() {
        // 勾選不改順序，也不得觸發重編號——就算既有編號是亂的。
        let root = fixture_with_tasks_md("renum-toggle", "## 1. 群組\n\n- [ ] 1.9 甲\n- [ ] 1.5 乙\n");
        set_task_done_at(&root, "task-change", 1, true).expect("toggle ok");
        assert_eq!(task_lines(&root), vec!["- [x] 1.9 甲", "- [ ] 1.5 乙"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn out_of_range_move_leaves_the_file_untouched() {
        let root = fixture_with_tasks_md("renum-oob", "## 1. 群組\n\n- [ ] 1.1 甲\n");
        let before = read_tasks(&root);
        assert!(move_task_at(&root, "task-change", 9, 1, None).is_err());
        assert!(move_task_at(&root, "task-change", 0, 1, None).is_err());
        assert_eq!(read_tasks(&root), before, "failed moves must not rewrite the file");
        let _ = fs::remove_dir_all(&root);
    }

    // --- 完成語意：GUI 勾任務與 CLI 完成語意一致（desktop-app spec）---

    const META_UNSTARTED: &str =
        "schema: spec-driven\ncreated: 2026-07-05\ncreated_by: momo\ncreated_with: claude\n";

    /// fixture 根轉為 git repo（本地身分固定）；`dirty_rel` 給一個未認領的 dirty 程式檔。
    fn giterize(root: &Path, dirty_rel: Option<&str>) {
        let git = |args: &[&str]| {
            let ok = std::process::Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        };
        git(&["init", "-q"]);
        git(&["config", "user.name", "Desk Tester"]);
        git(&["config", "user.email", "desk@example.com"]);
        if let Some(rel) = dirty_rel {
            let p = root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, "dirty\n").unwrap();
        }
    }

    fn change_file(fx: &crate::testfixture::FixtureRoot, name: &str, file: &str) -> PathBuf {
        fx.root().join("openspec").join("changes").join(name).join(file)
    }

    fn meta_of(fx: &crate::testfixture::FixtureRoot, name: &str) -> String {
        fs::read_to_string(change_file(fx, name, ".openspec.yaml")).unwrap()
    }

    fn touched_of(fx: &crate::testfixture::FixtureRoot, name: &str) -> Option<String> {
        fs::read_to_string(
            fx.root().join(".speclink").join("touched").join(format!("{name}.json")),
        )
        .ok()
    }

    fn set_readonly(p: &Path, on: bool) {
        let mut perm = fs::metadata(p).unwrap().permissions();
        perm.set_readonly(on);
        fs::set_permissions(p, perm).unwrap();
    }

    #[test]
    fn set_task_done_true_stamps_meta_and_records_touched() {
        let fx = crate::testfixture::FixtureRoot::new("done-stamp");
        fx.add_change("demo", META_UNSTARTED);
        giterize(fx.root(), Some("src/main.rs"));

        set_task_done_at(fx.root(), "demo", 1, true).expect("check ok");

        let tasks = fs::read_to_string(change_file(&fx, "demo", "tasks.md")).unwrap();
        assert!(tasks.contains("- [x] 1.1 First task"), "task must be checked: {tasks}");
        // 與 CLI task done 相同的檔案效果：meta 蓋開工章（identity 沿 git 身分、agent 缺席）。
        let meta = meta_of(&fx, "demo");
        assert!(meta.starts_with(META_UNSTARTED), "existing meta preserved verbatim: {meta}");
        assert!(
            meta.contains(&format!("started_at: {}", speclink_core::util::today())),
            "first completion must stamp started_at: {meta}"
        );
        assert!(
            meta.contains("started_by: Desk Tester <desk@example.com>"),
            "git identity must be attributed: {meta}"
        );
        assert!(!meta.contains("started_with"), "no agent source → started_with absent: {meta}");
        // touched 記錄與 CLI 同語意：未認領 dirty 檔記於本任務項下。
        let touched = touched_of(&fx, "demo").expect("touched record written");
        assert!(touched.contains("src/main.rs"), "dirty file must be recorded: {touched}");
        assert!(touched.contains("\"task_id\": \"1\""), "entry attributed to task 1: {touched}");
    }

    #[test]
    fn set_task_done_true_on_done_task_is_an_idempotent_no_op() {
        let fx = crate::testfixture::FixtureRoot::new("done-idem");
        fx.add_change("demo", META_UNSTARTED);
        // 唯讀 tasks.md：任何寫入企圖都會失敗——冪等成功必須完全不寫檔。
        let tasks_path = change_file(&fx, "demo", "tasks.md");
        let before = fs::read_to_string(&tasks_path).unwrap();
        set_readonly(&tasks_path, true);
        let r = set_task_done_at(fx.root(), "demo", 2, true);
        set_readonly(&tasks_path, false);
        assert!(r.is_ok(), "duplicate done=true must be an idempotent success: {r:?}");
        assert_eq!(fs::read_to_string(&tasks_path).unwrap(), before);
        assert_eq!(meta_of(&fx, "demo"), META_UNSTARTED, "no stamp on the no-op path");
        assert!(touched_of(&fx, "demo").is_none(), "no touched record on the no-op path");
    }

    #[test]
    fn uncheck_and_move_never_touch_meta_or_touched_record() {
        let fx = crate::testfixture::FixtureRoot::new("no-sideeffect");
        fx.add_change("demo", META_UNSTARTED);
        // dirty 檔在場更能證明「不記 touched」是行為而非碰巧無料可記。
        giterize(fx.root(), Some("src/lib.rs"));
        let touched_before = "{\n  \"change\": \"demo\",\n  \"touched\": []\n}";
        fx.write(".speclink/touched/demo.json", touched_before);

        set_task_done_at(fx.root(), "demo", 2, false).expect("uncheck ok");
        move_task_at(fx.root(), "demo", 1, 2, None).expect("move ok");

        assert_eq!(meta_of(&fx, "demo"), META_UNSTARTED, "meta must stay byte-identical");
        assert_eq!(
            touched_of(&fx, "demo").as_deref(),
            Some(touched_before),
            "touched record must stay byte-identical"
        );
    }

    #[test]
    fn desktop_ordinal_matches_engine_task_id_on_mixed_fixture() {
        // D3 風險釘死：同一 tasks.md（群組標題＋巢狀縮排＋非 checkbox 行混排），
        // desktop ordinal N 與引擎 task id N 必指同一任務。
        let mixed = "## 1. 群組甲\n\n前言說明不列入計數。\n\n- [ ] 1.1 首任務\n  - [ ] 巢狀子任務\n  - 純列表項非 checkbox\n- [ ] 1.2 次任務\n\n## 2. 群組乙\n\n- [ ] 2.1 尾任務\n";
        for n in 1..=4 {
            let fx = crate::testfixture::FixtureRoot::new(&format!("align-{n}"));
            fx.add_change("demo", META_UNSTARTED);
            fx.write("openspec/changes/demo/tasks.md", mixed);
            set_task_done_at(fx.root(), "demo", n, true).expect("check ok");
            let text = fs::read_to_string(change_file(&fx, "demo", "tasks.md")).unwrap();
            let parsed = speclink_core::tasks::parse(&text);
            assert_eq!(parsed.len(), 4, "engine must see the same 4 checkboxes");
            assert!(
                parsed[n - 1].done,
                "engine task id {n} must be exactly the task desktop ordinal {n} checked: {text}"
            );
            assert_eq!(parsed.iter().filter(|t| t.done).count(), 1);
        }
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

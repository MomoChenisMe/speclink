//! GUI 管理操作：change metadata 讀取與 active change 刪除。
//!
//! delete 是 desktop 層操作（引擎與 CLI 皆無 delete 動詞）——僅作用於 active change
//! 目錄、經路徑安全檢查、由 UI 以確認對話框把關。

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use speclink_core::store::Store;

use crate::init_core_context;

/// 每專案根一次的 git 身分快取（design D1：identity 每根快取）：完成路徑高頻取用，
/// 而 GUI 進程 spawn git 在部分環境極慢（防毒掃描，實測單次 ~3 秒）——首次取得後
/// app 存續期內重用，身分變更需重啟 app 才生效。持鎖跨首次抓取：同根並發首抓只 spawn 一次。
static IDENTITY_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<PathBuf, Option<String>>>,
> = std::sync::OnceLock::new();

/// 任務寫回全域鎖（design D3：寫入序列化（全域寫鎖））：寫回 command 移至執行緒池後
/// 並發成為可能——讀-改-寫不序列化會互相覆蓋（遺失更新）。寫回入口整段持鎖，
/// 依提交順序落盤；讀取路徑不取鎖維持快路徑。
static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 取得寫回鎖（poisoned 時取回內值——鎖只護順序、無不變量需要放棄）。
fn write_guard() -> std::sync::MutexGuard<'static, ()> {
    WRITE_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

/// 取得（並快取）指定根的 git 身分；供 app 啟動／切根時背景預熱與完成路徑取用。
pub fn cached_git_identity(root: &Path) -> Option<String> {
    let cache = IDENTITY_CACHE.get_or_init(Default::default);
    let mut map = cache.lock().unwrap_or_else(|p| p.into_inner());
    map.entry(root.to_path_buf())
        .or_insert_with(|| speclink_core::util::git_identity(root))
        .clone()
}

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
    let _guard = write_guard();
    if done {
        if !crate::query::is_safe_path_param(change) {
            return Err(format!("invalid change name: {change}"));
        }
        let ctx = init_core_context(root)
            .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
        let identity = cached_git_identity(&ctx.workspace.root);
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

/// 批次設定全部任務的完成狀態（desktop-task-interactions design D1：批次動詞單指令雙用）。
/// done=true＝全部標完成，done=false＝全部取消勾選；一次讀檔、一次寫回。
/// 側效沿單發勾選語意：done=true 且有翻轉時 touched 記錄一次（歸於本次首個翻轉的任務）、
/// 首次完成蓋開工章；done=false 純行編輯，無任何側效。
/// 目標狀態已達成時冪等成功、零檔案效果。
pub fn set_all_tasks_at(root: &Path, change: &str, done: bool) -> Result<(), String> {
    let _guard = write_guard();
    if !crate::query::is_safe_path_param(change) {
        return Err(format!("invalid change name: {change}"));
    }
    let ctx = init_core_context(root)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let text = ctx
        .store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| format!("tasks.md not found for change: {change}"))?;
    let had_trailing_newline = text.ends_with('\n');
    // 逐行鏡射引擎 tasks::parse 的 checkbox 判定——翻轉集合與引擎所見一致
    // （dash／star 兩種 bullet、[ ]/[x]/[X] 三態）。
    let mut first_flipped: Option<(usize, String)> = None;
    let mut ordinal = 0usize;
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let body = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* "));
            let Some(body) = body else { return line.to_string() };
            let cur = if body.starts_with("[ ] ") {
                false
            } else if body.starts_with("[x] ") || body.starts_with("[X] ") {
                true
            } else {
                return line.to_string();
            };
            ordinal += 1;
            if cur == done {
                return line.to_string();
            }
            if first_flipped.is_none() {
                first_flipped = Some((ordinal, body[4..].trim().to_string()));
            }
            let indent = &line[..line.len() - trimmed.len()];
            let bullet = &trimmed[..2];
            let mark = if done { "[x]" } else { "[ ]" };
            format!("{indent}{bullet}{mark}{}", &body[3..])
        })
        .collect();
    // 目標狀態已達成→冪等成功、零檔案效果。
    let Some((first_id, first_desc)) = first_flipped else { return Ok(()) };
    let mut out = lines.join("\n");
    if had_trailing_newline {
        out.push('\n');
    }
    ctx.store
        .write_artifact(change, "tasks.md", &out)
        .map_err(|e| e.to_string())?;
    if done {
        // 側效沿單發完成語意（tasks::complete 同款）：未認領 dirty 檔記一筆
        // touched（歸於本次首個翻轉任務），首次完成蓋開工章（inprogress::add 冪等）。
        let mut record = speclink_core::tasks::TouchedRecord::load(&ctx.workspace, change);
        record.change = change.to_string();
        let seen = record.all_files();
        let files: Vec<String> = speclink_core::tasks::git_changed_files(&ctx.workspace.root)
            .into_iter()
            .filter(|f| !seen.contains(f))
            .collect();
        if !files.is_empty() {
            record.touched.push(speclink_core::tasks::TouchedEntry {
                task_id: first_id.to_string(),
                task_desc: first_desc,
                files,
            });
            record.save(&ctx.workspace).map_err(|e| e.to_string())?;
        }
        let identity = cached_git_identity(&ctx.workspace.root);
        speclink_core::inprogress::add(&ctx.store, change, identity.as_deref(), None)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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
    let _guard = write_guard();
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

/// 看板拖排寫回（design D3/D5）：以鄰居識別碼表達落點，計中點 rank 寫回被拖卡的
/// meta（變更）或 frontmatter（討論）。目標欄內有缺 rank 卡時先依當前顯示序整欄
/// 補章（等距鍵）再套用移動；已消失的鄰居視為開放端（欄頂／欄底），不 panic。
pub fn reorder_card_at(
    root: &Path,
    kind: &str,
    id: &str,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> Result<(), String> {
    let _guard = write_guard();
    if !crate::query::is_safe_path_param(id) {
        return Err(format!("invalid card id: {id}"));
    }
    for n in [prev_id, next_id].into_iter().flatten() {
        if !crate::query::is_safe_path_param(n) {
            return Err(format!("invalid neighbor id: {n}"));
        }
    }
    let ctx = init_core_context(root)
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))?;
    let store: &dyn Store = &ctx.store;
    match kind {
        "change" => reorder_change(store, id, prev_id, next_id),
        "discussion" => reorder_discussion(store, id, prev_id, next_id),
        other => Err(format!("invalid card kind: {other}")),
    }
}

/// 變更卡的所屬欄（與前端 changeStage 同構）：任務全完成→ready(2)；
/// 已開工或有完成數→in-progress(1)；其餘→proposed(0)。
fn change_stage(store: &dyn Store, c: &speclink_core::model::Change) -> u8 {
    let (complete, total) = speclink_core::listing::task_counts(store, c);
    if total > 0 && complete >= total {
        2
    } else if c.meta.started_at.is_some() || complete > 0 {
        1
    } else {
        0
    }
}

fn reorder_change(
    store: &dyn Store,
    id: &str,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> Result<(), String> {
    let all = crate::query::board_sorted_changes(store);
    let dragged = all
        .iter()
        .find(|c| c.name == id)
        .ok_or_else(|| format!("change not found: {id}"))?;
    let stage = change_stage(store, dragged);
    let column: Vec<_> = all.iter().filter(|c| change_stage(store, c) == stage).collect();
    // 整欄補章（design D3）：欄內有缺 rank 卡 → 依顯示序等距派發，只涵蓋本欄。
    let ranks: std::collections::HashMap<&str, String> =
        if column.iter().any(|c| c.meta.board_rank.is_none()) {
            let keys = crate::rank::spread(column.len());
            for (c, key) in column.iter().zip(&keys) {
                speclink_core::model::set_board_rank(store, &c.name, key)
                    .map_err(|e| e.to_string())?;
            }
            column.iter().map(|c| c.name.as_str()).zip(keys).collect()
        } else {
            column
                .iter()
                .map(|c| (c.name.as_str(), c.meta.board_rank.clone().expect("all ranked")))
                .collect()
        };
    let key = neighbor_midpoint(&ranks, prev_id, next_id);
    speclink_core::model::set_board_rank(store, id, &key).map_err(|e| e.to_string())
}

fn reorder_discussion(
    store: &dyn Store,
    id: &str,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> Result<(), String> {
    let active = crate::discussions::board_sorted_active(store);
    if !active.iter().any(|(_, i)| i.slug == id) {
        return Err(format!("discussion not found: {id}"));
    }
    let ranks: std::collections::HashMap<&str, String> =
        if active.iter().any(|(r, _)| r.is_none()) {
            let keys = crate::rank::spread(active.len());
            for ((_, i), key) in active.iter().zip(&keys) {
                speclink_core::discuss::set_board_rank(store, &i.slug, key)
                    .map_err(|e| e.to_string())?;
            }
            active.iter().map(|(_, i)| i.slug.as_str()).zip(keys).collect()
        } else {
            active
                .iter()
                .map(|(r, i)| (i.slug.as_str(), r.clone().expect("all ranked")))
                .collect()
        };
    let key = neighbor_midpoint(&ranks, prev_id, next_id);
    speclink_core::discuss::set_board_rank(store, id, &key).map_err(|e| e.to_string())
}

/// 以鄰居現值推導新鍵：不在現存集合的鄰居（已封存／刪除的 race）視為開放端；
/// 現值逆序（stale 落點）時棄上界保底——寧可落位偏移，不產生非法鍵。
fn neighbor_midpoint(
    ranks: &std::collections::HashMap<&str, String>,
    prev_id: Option<&str>,
    next_id: Option<&str>,
) -> String {
    let prev = prev_id.and_then(|p| ranks.get(p)).cloned();
    let next = next_id.and_then(|n| ranks.get(n)).cloned();
    let next = match (&prev, &next) {
        (Some(a), Some(b)) if a >= b => None,
        _ => next,
    };
    crate::rank::midpoint(prev.as_deref(), next.as_deref())
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

    // spec「任務寫回非阻塞且序列化」Scenario「並發寫回序列化」（design D3：寫入序列化（全域寫鎖））
    #[test]
    fn concurrent_task_writes_serialize_without_lost_updates() {
        // 讀-改-寫競態窗口極窄，多輪並發提高暴露機率；有鎖後每輪皆確定無遺失。
        for round in 0..30 {
            let root = fixture_with_tasks_md(
                &format!("wlock-{round}"),
                "## 1. G\n\n- [x] 1.1 a\n- [x] 1.2 b\n",
            );
            let r1 = root.clone();
            let r2 = root.clone();
            let t1 = std::thread::spawn(move || set_task_done_at(&r1, "task-change", 1, false));
            let t2 = std::thread::spawn(move || set_task_done_at(&r2, "task-change", 2, false));
            t1.join().unwrap().expect("uncheck 1 ok");
            t2.join().unwrap().expect("uncheck 2 ok");
            let text = read_tasks(&root);
            assert!(
                text.contains("- [ ] 1.1 a") && text.contains("- [ ] 1.2 b"),
                "round {round}: lost update, tasks.md = {text:?}"
            );
            let _ = fs::remove_dir_all(&root);
        }
    }

    /// 對指定目錄 git init 並設定身分（cached_git_identity 測試用，與 giterize 的固定身分區隔）。
    fn git_with_identity(root: &Path, name: &str, email: &str) {
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", name],
            vec!["config", "user.email", email],
        ] {
            let ok = std::process::Command::new("git")
                .args(&args)
                .current_dir(root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        }
    }

    // spec「任務寫回非阻塞且序列化」Scenario「git 身分快取重用」（design D1）
    #[test]
    fn cached_git_identity_is_per_root_and_stable() {
        let a = crate::testfixture::FixtureRoot::new("idcache-a");
        let b = crate::testfixture::FixtureRoot::new("idcache-b");
        git_with_identity(a.root(), "Alice", "alice@example.com");
        git_with_identity(b.root(), "Bob", "bob@example.com");
        // 快取按根區分。
        assert_eq!(cached_git_identity(a.root()).as_deref(), Some("Alice <alice@example.com>"));
        assert_eq!(cached_git_identity(b.root()).as_deref(), Some("Bob <bob@example.com>"));
        // 同根重複呼叫回傳快取值——事後改 config 不重抓（app 存續期語意，重啟才更新）。
        let _ = std::process::Command::new("git")
            .args(["config", "user.name", "Changed"])
            .current_dir(a.root())
            .output();
        assert_eq!(cached_git_identity(a.root()).as_deref(), Some("Alice <alice@example.com>"));
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

    // --- 批次任務動詞（desktop-task-interactions spec「任務分頁提供批次操作工具列」）---

    #[test]
    fn set_all_tasks_done_checks_everything_and_stamps_once() {
        // 全勾單次寫回；開工章語意與逐一勾選一致；touched 僅記一筆（歸於首個翻轉任務）。
        let fx = crate::testfixture::FixtureRoot::new("all-done");
        fx.add_change("demo", META_UNSTARTED);
        giterize(fx.root(), Some("src/main.rs"));

        set_all_tasks_at(fx.root(), "demo", true).expect("batch check ok");

        let tasks = fs::read_to_string(change_file(&fx, "demo", "tasks.md")).unwrap();
        let parsed = speclink_core::tasks::parse(&tasks);
        assert_eq!(parsed.len(), 2, "checkbox count unchanged");
        assert!(parsed.iter().all(|t| t.done), "every task must be checked: {tasks}");
        assert!(tasks.contains("## 1. Group"), "non-checkbox lines intact");
        let meta = meta_of(&fx, "demo");
        assert!(meta.starts_with(META_UNSTARTED), "existing meta preserved verbatim: {meta}");
        assert!(
            meta.contains(&format!("started_at: {}", speclink_core::util::today())),
            "batch first completion must stamp started_at: {meta}"
        );
        assert!(
            meta.contains("started_by: Desk Tester <desk@example.com>"),
            "git identity must be attributed: {meta}"
        );
        let touched = touched_of(&fx, "demo").expect("touched record written");
        assert!(touched.contains("src/main.rs"), "dirty file recorded: {touched}");
        assert_eq!(
            touched.matches("task_id").count(),
            1,
            "batch must record exactly one touched entry: {touched}"
        );
    }

    #[test]
    fn set_all_tasks_done_when_already_done_is_a_no_write_no_op() {
        let fx = crate::testfixture::FixtureRoot::new("all-idem");
        fx.add_change("demo", META_UNSTARTED);
        set_all_tasks_at(fx.root(), "demo", true).expect("first batch ok");
        let tasks_path = change_file(&fx, "demo", "tasks.md");
        let tasks_before = fs::read_to_string(&tasks_path).unwrap();
        let meta_before = meta_of(&fx, "demo");
        // 唯讀鎖檔：冪等路徑必須零寫檔。
        set_readonly(&tasks_path, true);
        let r = set_all_tasks_at(fx.root(), "demo", true);
        set_readonly(&tasks_path, false);
        assert!(r.is_ok(), "repeat batch done must be idempotent success: {r:?}");
        assert_eq!(fs::read_to_string(&tasks_path).unwrap(), tasks_before);
        assert_eq!(meta_of(&fx, "demo"), meta_before, "no re-stamp on the no-op path");
    }

    #[test]
    fn set_all_tasks_reset_unchecks_without_stamp_or_touched() {
        // 重置：全部取消勾選、不蓋開工章、不記 touched（dirty 檔在場更能證明是行為）。
        let fx = crate::testfixture::FixtureRoot::new("all-reset");
        fx.add_change("demo", META_UNSTARTED);
        giterize(fx.root(), Some("src/lib.rs"));

        set_all_tasks_at(fx.root(), "demo", false).expect("batch reset ok");

        let tasks = fs::read_to_string(change_file(&fx, "demo", "tasks.md")).unwrap();
        assert!(
            speclink_core::tasks::parse(&tasks).iter().all(|t| !t.done),
            "every task must be unchecked: {tasks}"
        );
        assert_eq!(meta_of(&fx, "demo"), META_UNSTARTED, "reset must not stamp");
        assert!(touched_of(&fx, "demo").is_none(), "reset must not record touched");
        // 全未勾再重置：冪等成功、零寫檔（唯讀鎖檔驗證）。
        let tasks_path = change_file(&fx, "demo", "tasks.md");
        set_readonly(&tasks_path, true);
        let r = set_all_tasks_at(fx.root(), "demo", false);
        set_readonly(&tasks_path, false);
        assert!(r.is_ok(), "repeat reset must be idempotent success: {r:?}");
    }

    #[test]
    fn set_all_tasks_preserves_structure_and_rejects_bad_input() {
        // 群組標題與行序原樣保留、僅 checkbox 標記翻轉；守衛與單發一致。
        let root = fixture_with_tasks("all-preserve");
        set_all_tasks_at(&root, "task-change", true).expect("batch ok");
        assert_eq!(
            task_lines(&root),
            vec!["- [x] 1.1 first", "- [x] 1.2 second", "- [x] 2.1 third"],
            "only checkbox marks may change"
        );
        let text = read_tasks(&root);
        assert!(text.contains("## 1. Group A") && text.contains("## 2. Group B"));
        assert!(text.ends_with('\n'), "trailing newline preserved");
        assert!(set_all_tasks_at(&root, "no-such-xyz", true).is_err(), "unknown change errors");
        assert!(set_all_tasks_at(&root, "../specs", true).is_err(), "traversal rejected");
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

    // --- 看板拖排寫回（design D3/D5；desktop-card-reorder） ---

    /// 含 board_rank 的 change meta（add_change 的 tasks 固定 1 完成 1 未完成
    /// → 全卡同屬 in-progress 欄）。
    fn ranked_meta(rank: &str) -> String {
        format!("schema: spec-driven\ncreated: 2026-07-01\ncreated_by: momo\nboard_rank: {rank}\n")
    }

    /// 看板欄內排序後的 change 名（走 list_changes_at 的顯示序）。
    fn board_names(root: &Path) -> Vec<String> {
        crate::query::list_changes_at(root)["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect()
    }

    fn rank_of(meta: &str) -> Option<String> {
        speclink_core::model::ChangeMeta::from_text(Some(meta)).board_rank
    }

    #[test]
    fn reorder_change_steady_state_writes_only_dragged_meta() {
        // spec「欄內拖排以中點 rank 單檔寫回」：穩態下只改被拖卡 meta，
        // 其餘內容逐位元組不變、鄰居檔案不動。
        let fx = crate::testfixture::FixtureRoot::new("r-steady");
        fx.add_change("one", &ranked_meta("b"));
        fx.add_change("two", &ranked_meta("f"));
        fx.add_change("three", &ranked_meta("t"));
        let one_before = meta_of(&fx, "one");
        let two_before = meta_of(&fx, "two");

        reorder_card_at(fx.root(), "change", "three", Some("one"), Some("two"))
            .expect("steady-state reorder ok");

        assert_eq!(meta_of(&fx, "one"), one_before, "neighbor files must not change");
        assert_eq!(meta_of(&fx, "two"), two_before, "neighbor files must not change");
        let three = meta_of(&fx, "three");
        let new_rank = rank_of(&three).expect("dragged card keeps a rank");
        assert!(
            "b" < new_rank.as_str() && new_rank.as_str() < "f",
            "new rank must be strictly between neighbors: {new_rank}"
        );
        // 除 board_rank 行外逐位元組不變（原 rank 行原位代換）。
        assert_eq!(
            three.replacen(&format!("board_rank: {new_rank}\n"), "board_rank: t\n", 1),
            ranked_meta("t"),
            "only the board_rank line may differ: {three}"
        );
        assert_eq!(board_names(fx.root()), ["one", "three", "two"]);
    }

    #[test]
    fn reorder_change_stamps_whole_column_when_unranked_present() {
        // spec「欄內存在缺 rank 卡時整欄補章」：依當前顯示序整欄派發後套用移動，
        // 不波及他欄。
        let fx = crate::testfixture::FixtureRoot::new("r-stamp");
        fx.add_change("a", META_UNSTARTED);
        fx.add_change("b", META_UNSTARTED);
        fx.add_change("c", META_UNSTARTED);
        // 他欄卡（proposed：零完成、未開工）不得被補章。
        fx.add_change("other-col", META_UNSTARTED);
        fx.write(
            "openspec/changes/other-col/tasks.md",
            "## 1. Group\n\n- [ ] 1.1 First task\n- [ ] 1.2 Second task\n",
        );
        let other_before = meta_of(&fx, "other-col");

        // 把 c 拖到欄頂（a 之前）。
        reorder_card_at(fx.root(), "change", "c", None, Some("a")).expect("stamp reorder ok");

        let (ra, rb, rc) = (
            rank_of(&meta_of(&fx, "a")).expect("a stamped"),
            rank_of(&meta_of(&fx, "b")).expect("b stamped"),
            rank_of(&meta_of(&fx, "c")).expect("c ranked"),
        );
        assert!(rc < ra && ra < rb, "order must be c < a < b, got c={rc} a={ra} b={rb}");
        assert_eq!(meta_of(&fx, "other-col"), other_before, "other column must be untouched");
        assert_eq!(board_names(fx.root()), ["other-col", "c", "a", "b"], "unranked column first, then ranked");
        // 補章後 meta 仍以既有欄位開頭（byte 保留）。
        assert!(meta_of(&fx, "a").starts_with(META_UNSTARTED));
    }

    #[test]
    fn reorder_discussion_writes_single_frontmatter_file() {
        // 討論卡同語意：中點寫回單檔、鄰居不動。
        let fx = crate::testfixture::FixtureRoot::new("r-disc");
        let doc = |slug: &str, rank: &str| {
            format!(
                "---\ntopic: T {slug}\nslug: {slug}\nstatus: open\nboard_rank: {rank}\ncreated: 2026-07-01\n---\n\n# Discussion: T {slug}\n\n## Context\n\nx\n\n## Rounds\n\n## Conclusion\n\n<!-- p -->\n"
            )
        };
        fx.write("openspec/discussions/alpha.md", &doc("alpha", "b"));
        fx.write("openspec/discussions/beta.md", &doc("beta", "n"));
        fx.write("openspec/discussions/gamma.md", &doc("gamma", "t"));
        let alpha_before = fs::read_to_string(fx.root().join("openspec/discussions/alpha.md")).unwrap();

        reorder_card_at(fx.root(), "discussion", "gamma", Some("alpha"), Some("beta"))
            .expect("discussion reorder ok");

        assert_eq!(
            fs::read_to_string(fx.root().join("openspec/discussions/alpha.md")).unwrap(),
            alpha_before,
            "neighbor discussion files must not change"
        );
        let gamma = fs::read_to_string(fx.root().join("openspec/discussions/gamma.md")).unwrap();
        let new_rank = {
            let line = gamma.lines().find(|l| l.starts_with("board_rank:")).expect("rank line");
            line.trim_start_matches("board_rank:").trim().to_string()
        };
        assert!("b" < new_rank.as_str() && new_rank.as_str() < "n", "b < {new_rank} < n violated");
        assert_eq!(
            gamma.replacen(&format!("board_rank: {new_rank}\n"), "board_rank: t\n", 1),
            doc("gamma", "t"),
            "only the frontmatter rank line may differ"
        );
    }

    #[test]
    fn reorder_survives_vanished_neighbors_without_corrupting_meta() {
        // spec「鄰居於寫回前消失」：以現存鄰居重導或落欄頂／欄底，不損壞 meta、不 panic。
        let fx = crate::testfixture::FixtureRoot::new("r-race");
        fx.add_change("solo", &ranked_meta("f"));
        reorder_card_at(fx.root(), "change", "solo", Some("ghost-prev"), Some("ghost-next"))
            .expect("vanished neighbors must not fail");
        let meta = meta_of(&fx, "solo");
        let parsed = speclink_core::model::ChangeMeta::from_text(Some(&meta));
        assert_eq!(parsed.schema.as_deref(), Some("spec-driven"), "meta must keep parsing");
        assert!(parsed.board_rank.is_some(), "card keeps a valid rank");
    }

    #[test]
    fn reorder_rejects_bad_kind_unsafe_or_missing_ids() {
        let fx = crate::testfixture::FixtureRoot::new("r-guard");
        fx.add_change("demo", &ranked_meta("n"));
        let before = meta_of(&fx, "demo");
        assert!(reorder_card_at(fx.root(), "bogus", "demo", None, None).is_err(), "kind whitelist");
        assert!(reorder_card_at(fx.root(), "change", "../evil", None, None).is_err(), "traversal rejected");
        assert!(reorder_card_at(fx.root(), "change", "ghost", None, None).is_err(), "missing card errors");
        assert!(
            reorder_card_at(fx.root(), "change", "demo", Some("../evil"), None).is_err(),
            "unsafe neighbor ids rejected"
        );
        assert!(reorder_card_at(fx.root(), "discussion", "ghost", None, None).is_err(), "missing discussion errors");
        assert_eq!(meta_of(&fx, "demo"), before, "rejected calls must not write");
    }
}

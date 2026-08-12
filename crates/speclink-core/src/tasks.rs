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
    /// Manual-verification task (`[M]`): human acceptance work the gates exclude
    /// from their "code tasks all complete" predicate.
    pub manual: bool,
    /// Immutable identity from the line's trailing speclink-task comment; None on unstamped lines.
    pub stable_id: Option<String>,
}

/// Opening marker of the stable-ID comment embedded at a task line's end.
const STABLE_ID_OPEN: &str = "<!-- speclink-task:";

/// Split a task body into (display text, stable ID) by stripping a trailing
/// speclink-task marker comment. A marker anywhere but the line end is left in
/// the display text untouched.
fn split_stable_id(body: &str) -> (&str, Option<&str>) {
    let trimmed = body.trim_end();
    if let Some(start) = trimmed.rfind(STABLE_ID_OPEN) {
        if let Some(id) = trimmed[start..]
            .strip_prefix(STABLE_ID_OPEN)
            .and_then(|r| r.strip_suffix("-->"))
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            return (trimmed[..start].trim_end(), Some(id));
        }
    }
    (trimmed, None)
}

/// Generate a fresh task stable ID: `tsk_` + 26-char ULID (lexicographically time-ordered).
pub fn new_stable_id() -> String {
    format!("tsk_{}", ulid::Ulid::new())
}

/// Append a fresh stable-ID comment to a task line that lacks one. Returns
/// None when the line is not a task line or already carries an ID.
fn stamp_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let body = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))?;
    let is_task =
        body.starts_with("[ ] ") || body.starts_with("[x] ") || body.starts_with("[X] ");
    if !is_task || split_stable_id(body).1.is_some() {
        return None;
    }
    Some(format!("{} <!-- speclink-task:{} -->", line.trim_end(), new_stable_id()))
}

/// Stamp every unstamped task line with a fresh stable ID; stamped lines and
/// non-task lines pass through byte-for-byte.
pub fn stamp_all(tasks_md: &str) -> String {
    let mut out_lines: Vec<String> = Vec::new();
    for line in tasks_md.lines() {
        out_lines.push(stamp_line(line).unwrap_or_else(|| line.to_string()));
    }
    let mut out = out_lines.join("\n");
    if tasks_md.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Stable ID values carried by more than one task, in first-seen order.
pub fn duplicate_stable_ids(tasks: &[Task]) -> Vec<String> {
    let mut seen: Vec<&String> = Vec::new();
    let mut dups = Vec::new();
    for t in tasks {
        if let Some(id) = &t.stable_id {
            if seen.contains(&id) {
                if !dups.contains(id) {
                    dups.push(id.clone());
                }
            } else {
                seen.push(id);
            }
        }
    }
    dups
}

/// Strip the marker slot that follows a checkbox: `[M]` and the legacy `[P]` in
/// either order, each at most once. Only `[M]` carries meaning — `[P]` is
/// stripped for display tolerance of archived and external files, and yields no
/// flag. Returns (manual, remaining body).
fn strip_markers(rest: &str) -> (bool, &str) {
    let (mut legacy_parallel, mut manual) = (false, false);
    let mut body = rest;
    loop {
        if let (false, Some(d)) = (legacy_parallel, body.strip_prefix("[P] ")) {
            legacy_parallel = true;
            body = d;
            continue;
        }
        if let (false, Some(d)) = (manual, body.strip_prefix("[M] ")) {
            manual = true;
            body = d;
            continue;
        }
        return (manual, body);
    }
}

/// Parse tasks.md into an ordered list of checkbox tasks. Dash and star bullets both
/// count (`* [ ]` is a task).
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
        let (manual, desc) = strip_markers(rest);
        let (display, stable_id) = split_stable_id(desc);
        out.push(Task {
            id,
            description: display.trim().to_string(),
            done,
            manual,
            stable_id: stable_id.map(str::to_string),
        });
    }
    out
}

/// How a `[M]` marker missed the prefix slot [`strip_markers`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MisplacedMarker {
    /// `- [ ] 3.2 [M] …` — the task number took the slot, pushing the marker past it.
    AfterNumber,
    /// `- [ ]  [M] …` — extra space after the checkbox, so the slot never matched.
    PrefixSlotMissed,
}

/// The manual-verification marker literal. The prefix-slot stripping, the
/// misplacement detection, and validate's repair examples all build on this one
/// string — the slot syntax has no second definition site.
pub const MANUAL_MARKER: &str = "[M]";

/// A task whose `[M]` marker sits outside the prefix slot, so the parser read it
/// as description text and counted the task as code work. Carries the checkbox
/// state and stable ID so a repair example can reproduce the line faithfully.
#[derive(Debug, Clone)]
pub struct Misplaced {
    pub task_id: usize,
    pub description: String,
    pub kind: MisplacedMarker,
    pub done: bool,
    pub stable_id: Option<String>,
}

/// Find tasks that meant to carry `[M]` but wrote it where the parser cannot see it.
/// Only the start of the description is examined — a `[M]` further in is prose, and
/// task lists that discuss the marker are full of those.
pub fn misplaced_markers(tasks: &[Task]) -> Vec<Misplaced> {
    tasks
        .iter()
        .filter_map(|t| {
            let mut tokens = t.description.split_whitespace();
            let first = tokens.next()?;
            let kind = if first == MANUAL_MARKER {
                MisplacedMarker::PrefixSlotMissed
            } else if tokens.next() == Some(MANUAL_MARKER)
                && first.chars().any(|c| c.is_ascii_digit())
                && first.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                MisplacedMarker::AfterNumber
            } else {
                return None;
            };
            Some(Misplaced {
                task_id: t.id,
                description: t.description.clone(),
                kind,
                done: t.done,
                stable_id: t.stable_id.clone(),
            })
        })
        .collect()
}

/// Task counts in two groups: every task, and code tasks alone (`[M]` excluded).
/// The single source both the station gates and the stamp freshness anchors read
/// — no caller filters manual tasks on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub total: usize,
    pub complete: usize,
    pub remaining: usize,
    pub code_total: usize,
    pub code_complete: usize,
    pub code_remaining: usize,
}

impl Counts {
    /// The "code tasks all complete" predicate. Vacuously true with no code
    /// tasks — a zero-task change and an all-`[M]` change both pass.
    pub fn code_done(&self) -> bool {
        self.code_remaining == 0
    }
}

/// Count tasks in both groups (see [`Counts`]).
pub fn counts(tasks: &[Task]) -> Counts {
    let (mut complete, mut code_total, mut code_complete) = (0, 0, 0);
    for t in tasks {
        complete += usize::from(t.done);
        if !t.manual {
            code_total += 1;
            code_complete += usize::from(t.done);
        }
    }
    let total = tasks.len();
    Counts {
        total,
        complete,
        remaining: total - complete,
        code_total,
        code_complete,
        code_remaining: code_total - code_complete,
    }
}

/// Read a change's tasks.md and count both groups — the one entry point the
/// station gates, the stamp freshness anchors and the listings all read.
pub fn counts_for(store: &dyn Store, change: &str) -> Counts {
    counts(&parse(&store.read_artifact(change, "tasks.md").unwrap_or_default()))
}

/// Progress tuple: (total, complete, remaining).
pub fn progress(tasks: &[Task]) -> (usize, usize, usize) {
    let c = counts(tasks);
    (c.total, c.complete, c.remaining)
}

/// How a task verb addresses its target: 1-based ordinal (display/compat)
/// or tsk_ stable ID (first-class identity).
#[derive(Debug, Clone)]
pub enum TaskAddr {
    Ordinal(usize),
    Stable(String),
}

impl std::fmt::Display for TaskAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskAddr::Ordinal(n) => write!(f, "{n}"),
            TaskAddr::Stable(id) => f.write_str(id),
        }
    }
}

/// Parse, guard duplicate stable IDs (corrupt-file class: refuse, never pick
/// one silently), and resolve the address. Returns (1-based ordinal, total).
fn resolve_addr(tasks_md: &str, addr: &TaskAddr) -> Result<(usize, usize)> {
    let tasks = parse(tasks_md);
    let dups = duplicate_stable_ids(&tasks);
    if !dups.is_empty() {
        anyhow::bail!("Duplicate task IDs in tasks.md: {}", dups.join(", "));
    }
    let total = tasks.len();
    let ordinal = match addr {
        TaskAddr::Ordinal(n) => *n,
        TaskAddr::Stable(id) => tasks
            .iter()
            .find(|t| t.stable_id.as_deref() == Some(id.as_str()))
            .map(|t| t.id)
            .ok_or_else(|| anyhow::anyhow!("Task {id} not found (total: {total})"))?,
    };
    Ok((ordinal, total))
}

/// Flip the id-th checkbox to done. Returns (new_content, task_description,
/// already_done, stable_id) or None if not found. An unstamped target line is
/// stamped with a fresh stable ID in the same rewrite.
pub fn mark_done(tasks_md: &str, target_id: usize) -> Option<(String, String, bool, Option<String>)> {
    flip_task(tasks_md, target_id, true)
}

/// Flip the id-th checkbox in either direction. Returns (new_content, task_description,
/// already_in_target_state, stable_id) or None if not found. Indent, bullet style (`* [ ]`
/// is rewritten to `* [x]`), and trailing newline are preserved.
fn flip_task(
    tasks_md: &str,
    target_id: usize,
    to_done: bool,
) -> Option<(String, String, bool, Option<String>)> {
    let mut id = 0usize;
    let mut already = false;
    let mut desc = String::new();
    let mut stable_id: Option<String> = None;
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
                // 顯示描述剝除全部前綴標記——與 parse() 共用 strip_markers,
                // 剝離規則只有一份真相(spec「任務行的手動測試標記與解析」)。
                let (_, clean) = strip_markers(rest);
                let (display, stable) = split_stable_id(clean);
                desc = display.trim().to_string();
                stable_id = stable.map(str::to_string);
                let checkbox = if to_done { "[x]" } else { "[ ]" };
                let mut new_line = format!("{indent}{bullet} {checkbox} {rest}");
                // task done stamps the unstamped target line in the same write;
                // undone never stamps (pure state flip), and an already-done
                // target is never written, so no phantom id may leak out.
                if to_done && !already && stable.is_none() {
                    let fresh = new_stable_id();
                    new_line =
                        format!("{} <!-- speclink-task:{fresh} -->", new_line.trim_end());
                    stable_id = Some(fresh);
                }
                out_lines.push(new_line);
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
    Some((new_content, desc, already, stable_id))
}

/// Outcome of [`complete`]: the task's cleaned description, whether it was
/// already checked (in which case nothing was written), the resolved 1-based
/// ordinal, and the task's stable ID (existing or stamped by this write;
/// None only on the no-write `already` path of an unstamped task).
#[derive(Debug, Clone)]
pub struct CompleteOutcome {
    pub description: String,
    pub already: bool,
    pub ordinal: usize,
    pub stable_id: Option<String>,
}

/// Host-injected attribution for completion evidence: display identity, agent
/// source, and repo binding key. What the caller cannot attribute stays
/// absent in the evidence — never defaulted.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompleteAttribution<'a> {
    pub identity: Option<&'a str>,
    pub agent: Option<&'a str>,
    pub repo: Option<&'a str>,
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
    addr: &TaskAddr,
    attr: &CompleteAttribution,
) -> Result<CompleteOutcome> {
    // Fail-closed gate before any write: a checked task implies the
    // work-started stamp, which must not land on a corrupt metadata document.
    crate::model::check_meta_text(change, store.read_change_meta(change).as_deref())?;
    let text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change}'"))?;
    let (ordinal, total) = resolve_addr(&text, addr)?;
    let (new_content, desc, already, stable_id) = mark_done(&text, ordinal)
        .ok_or_else(|| anyhow::anyhow!("Task {addr} not found (total: {total})"))?;
    if already {
        return Ok(CompleteOutcome { description: desc, already: true, ordinal, stable_id });
    }
    store.write_artifact(change, "tasks.md", &new_content)?;

    // Record touched files: only those not already attributed to an earlier task;
    // when nothing new is dirty, no entry is appended at all (frozen behavior).
    // The v1 `touched` entry stays alongside the v2 evidence entry — the commit
    // skill's documented file-list channel keeps its exact shape.
    let mut record = TouchedRecord::load(ws, change);
    record.change = change.to_string();
    let seen = record.all_files();
    let files: Vec<String> = git_changed_files(ws)
        .into_iter()
        .filter(|f| !seen.contains(f))
        .collect();
    if !files.is_empty() {
        record.touched.push(TouchedEntry {
            task_id: ordinal.to_string(),
            task_desc: desc.clone(),
            files: files.clone(),
        });
        record.entries.push(EvidenceEntry {
            task_id: stable_id.clone().unwrap_or_else(|| ordinal.to_string()),
            task_desc: desc.clone(),
            actor: attr.identity.map(str::to_string),
            repo: attr.repo.map(str::to_string),
            head_commit: head_commit(&ws.root),
            touched_files: files,
            recorded_at: chrono::Utc::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        });
        record.save(ws)?;
    }

    crate::inprogress::add(store, change, attr.identity, attr.agent)?;
    Ok(CompleteOutcome { description: desc, already: false, ordinal, stable_id })
}

/// Outcome of [`uncomplete`]: the task's cleaned description, whether it was
/// already unchecked (in which case nothing was written), the resolved
/// 1-based ordinal, and the task's stable ID (None on unstamped lines —
/// undone never stamps).
#[derive(Debug, Clone)]
pub struct UncompleteOutcome {
    pub description: String,
    pub already: bool,
    pub ordinal: usize,
    pub stable_id: Option<String>,
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
pub fn uncomplete(store: &dyn Store, change: &str, addr: &TaskAddr) -> Result<UncompleteOutcome> {
    // Same fail-closed gate as [`complete`]: lifecycle state of a change with
    // corrupt metadata must not be edited until the document is repaired.
    crate::model::check_meta_text(change, store.read_change_meta(change).as_deref())?;
    let text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change}'"))?;
    let (ordinal, total) = resolve_addr(&text, addr)?;
    let (new_content, desc, already, stable_id) = mark_undone(&text, ordinal)
        .ok_or_else(|| anyhow::anyhow!("Task {addr} not found (total: {total})"))?;
    if already {
        return Ok(UncompleteOutcome { description: desc, already: true, ordinal, stable_id });
    }
    store.write_artifact(change, "tasks.md", &new_content)?;
    Ok(UncompleteOutcome { description: desc, already: false, ordinal, stable_id })
}

/// Flip the id-th checkbox back to open. Returns (new_content, task_description,
/// already_open, stable_id) or None if not found. Never stamps.
fn mark_undone(tasks_md: &str, target_id: usize) -> Option<(String, String, bool, Option<String>)> {
    flip_task(tasks_md, target_id, false)
}

/// Outcome of [`move_task`]: the moved task's cleaned description after the
/// move (prefixes already renumbered).
#[derive(Debug, Clone)]
pub struct MoveTaskOutcome {
    pub description: String,
}

/// True when the line is a checkbox task in [`parse`]'s domain (dash and star
/// bullets both count) — move ordinals share the same 1-based addressing as
/// task done/undone.
fn is_checkbox_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(body) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) else {
        return false;
    };
    body.starts_with("[ ] ") || body.starts_with("[x] ") || body.starts_with("[X] ")
}

/// 0-based indices of the checkbox lines.
fn checkbox_line_indices(lines: &[String]) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| is_checkbox_line(l))
        .map(|(i, _)| i)
        .collect()
}

/// 重算任務編號前綴：群組編號取自「## N.」標題自身的數字；群組內第 k 個
/// checkbox 行、文字以「數字.數字＋空白」開頭者，前綴重寫為「N.k」。其餘一律
/// 逐字元保留——無前綴、子版號（1.2.3）、無數字標題的群組、首個標題前的任務、
/// 群組標題與非 checkbox 行都不改寫（重編號永不弄丟使用者文字）。
fn renumber_task_prefixes(lines: &mut [String]) {
    let prefix_re = regex::Regex::new(r"^(\s*[-*]\s*\[[ xX]\]\s+)(\d+\.\d+)(\s)").unwrap();
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
        if is_checkbox_line(line) {
            k += 1;
            if let Some(g) = group {
                *line = prefix_re.replace(line, format!("${{1}}{g}.{k}${{3}}")).into_owned();
            }
        }
    }
}

/// 把第 `from` 個任務移到以第 `to` 個任務為錨的位置（皆 1-based、僅計 checkbox
/// 行）——桌面拖排與 server 端點共用的唯一搬移引擎。只搬 checkbox 行本身，群組
/// 標題與其他行不動；越界回 `Err` 且零寫入。`before`：None＝方向推斷（向上插錨
/// 前、向下插錨後——組界時貼齊手勢方向的群組）；Some(true)＝明確插於錨任務行之
/// 前（跨過群組標題即成為錨所屬群組的組首）；Some(false)＝明確插於錨任務行之
/// 後。搬移成功後重算編號前綴、保留檔尾換行狀態，一次寫回。
pub fn move_task(
    store: &dyn Store,
    change: &str,
    from: usize,
    to: usize,
    before: Option<bool>,
) -> Result<MoveTaskOutcome> {
    let text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| anyhow::anyhow!("tasks.md not found for change '{change}'"))?;
    let had_trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let idx = checkbox_line_indices(&lines);
    let n = idx.len();
    if from == 0 || to == 0 || from > n || to > n {
        // Refusal 標記：命令層歸類 refused（server 端 409），非 internal error——
        // index 定址在他人同時編輯下可位移，越界是可預期的競態拒絕。
        return Err(anyhow::Error::new(crate::command::Refusal(format!(
            "task index out of range (1..={n})"
        ))));
    }
    let moved_ordinal = if from == to {
        from
    } else {
        let moved = lines.remove(idx[from - 1]);
        // 移除後重算剩餘 checkbox 行位置；錨任務（原第 to 個）在移除後的 0-based 位置。
        let idx2 = checkbox_line_indices(&lines);
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
        renumber_task_prefixes(&mut lines);
        checkbox_line_indices(&lines)
            .iter()
            .position(|&i| i == insert_at)
            .expect("the inserted line is a checkbox line")
            + 1
    };
    let mut out = lines.join("\n");
    if had_trailing_newline {
        out.push('\n');
    }
    store.write_artifact(change, "tasks.md", &out)?;
    let description = parse(&out)
        .into_iter()
        .find(|t| t.id == moved_ordinal)
        .map(|t| t.description)
        .unwrap_or_default();
    Ok(MoveTaskOutcome { description })
}

// --- Touched-file tracking / per-task evidence ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchedEntry {
    pub task_id: String,
    pub task_desc: String,
    pub files: Vec<String>,
}

/// The three basis digests a completion was recorded against, each in the
/// EffectiveWorkflowPolicy digest form ("sha256:<hex>").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasisDigests {
    pub spec: String,
    pub tasks: String,
    pub policy: String,
}

/// Per-task completion evidence (v2): who, when, on which commit, over which
/// files (spec verify-evidence). Every field is a historical fact — nothing
/// here is judged, so nothing here can go stale.
/// Unattributable fields are absent, not defaulted; unknown fields on existing
/// records (a `basisDigests` from an earlier format) are ignored on read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEntry {
    /// The task's stable ID (done stamps its target, so this is the tsk_ id).
    pub task_id: String,
    pub task_desc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    pub touched_files: Vec<String>,
    /// UTC RFC3339 timestamp of the recording.
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TouchedRecord {
    /// Format version: absent = v1 (file-list records only); 2 = per-task
    /// evidence present. Writes always stamp v2; reads accept both.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    pub change: String,
    /// v1 file-list channel — kept on writes so existing consumers (commit
    /// skill file attribution) read the exact shape they always have.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub touched: Vec<TouchedEntry>,
    /// v2 per-task evidence entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<EvidenceEntry>,
}

impl TouchedRecord {
    /// Read the change's record: the change directory's `.evidence.json` is
    /// the home, and only when it is absent does the pre-move
    /// `.speclink/touched/<change>.json` get consulted (read-only bridge for
    /// records written before the move). Absent everywhere = empty record.
    pub fn load(ws: &Workspace, change: &str) -> TouchedRecord {
        let text = util::read_opt(&ws.change_evidence_file(change))
            .or_else(|| util::read_opt(&ws.legacy_touched_file(change)));
        match text {
            Some(s) => serde_json::from_str(&s).unwrap_or(TouchedRecord {
                change: change.to_string(),
                ..Default::default()
            }),
            None => TouchedRecord {
                change: change.to_string(),
                ..Default::default()
            },
        }
    }

    /// Writes always land in the change directory. The legacy bridge file is
    /// removed once the new home is written: its content was already carried
    /// forward by the fallback read, and a leftover would be read back as
    /// evidence for a future change reusing this name.
    pub fn save(&self, ws: &Workspace) -> std::io::Result<()> {
        let p = ws.change_evidence_file(&self.change);
        // Writes are always v2, whatever version was read.
        let mut rec = self.clone();
        rec.version = Some(2);
        let json = serde_json::to_string_pretty(&rec).unwrap_or_default();
        util::write_file(&p, &json)?;
        let _ = util::remove_file(&ws.legacy_touched_file(&self.change));
        Ok(())
    }

    /// Union of all files across v1 and v2 entries (for @trace), in
    /// first-seen order.
    pub fn all_files(&self) -> Vec<String> {
        let mut set = Vec::new();
        for f in self
            .touched
            .iter()
            .flat_map(|e| e.files.iter())
            .chain(self.entries.iter().flat_map(|e| e.touched_files.iter()))
        {
            if !set.contains(f) {
                set.push(f.clone());
            }
        }
        set
    }
}

/// Content digest in the EffectiveWorkflowPolicy form ("sha256:<hex>").
fn sha256_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

/// The three basis digests of a change's current state (spec / tasks /
/// policy). Computed on demand by drift, which compares a bundle's expected
/// basis against the change as it stands right now — never stored, so there is
/// nothing to fall out of date.
pub fn current_basis_digests(store: &dyn Store, change: &str) -> BasisDigests {
    BasisDigests {
        spec: spec_basis_digest(store, change),
        tasks: sha256_digest(
            store.read_artifact(change, "tasks.md").unwrap_or_default().as_bytes(),
        ),
        policy: sha256_digest(store.read_workflow_config().unwrap_or_default().as_bytes()),
    }
}

/// Digest of the change's delta specs: capability names and contents in
/// store order (sorted by contract), NUL-separated so boundaries can't blur.
fn spec_basis_digest(store: &dyn Store, change: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for cap in store.delta_capabilities(change) {
        hasher.update(cap.as_bytes());
        hasher.update([0]);
        if let Some(body) = store.read_artifact(change, &format!("specs/{cap}/spec.md")) {
            hasher.update(body.as_bytes());
        }
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
}

/// Current HEAD commit of the workspace root's own repo, None when absent.
/// Guarded on `.git` like [`git_changed_files`]: a project nested inside an
/// ancestor repo must not record the ancestor's HEAD.
fn head_commit(root: &Path) -> Option<String> {
    if !root.join(".git").exists() {
        return None;
    }
    util::git(root, &["rev-parse", "HEAD"]).filter(|s| !s.is_empty())
}

/// Decode porcelain's C-style quoting. Even with `core.quotepath=false` git
/// still quotes a path containing `"`, `\`, or a control character, escaping
/// the offending bytes as `\"`, `\\`, `\t` or three-digit octal. An unquoted
/// path is already literal and comes back untouched.
fn unquote_porcelain_path(raw: &str) -> String {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return raw.to_string();
    }
    let src = raw[1..raw.len() - 1].as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if src[i] != b'\\' {
            out.push(src[i]);
            i += 1;
            continue;
        }
        i += 1;
        let Some(&c) = src.get(i) else { break };
        i += 1;
        match c {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'0'..=b'7' => {
                // One raw byte, written as up to three octal digits.
                let mut v = u16::from(c - b'0');
                for _ in 0..2 {
                    match src.get(i) {
                        Some(&d) if (b'0'..=b'7').contains(&d) => {
                            v = v * 8 + u16::from(d - b'0');
                            i += 1;
                        }
                        _ => break,
                    }
                }
                out.push(v as u8);
            }
            // `\"`, `\\`, and anything unexpected stand for themselves.
            _ => out.push(c),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Files changed in the git work tree, relative to the workspace root,
/// forward-slashed.
///
/// Untracked directories are expanded to individual files (`-uall`). The workspace's spec
/// directory (whatever it is named) and the speclink work directory are excluded, since
/// evidence records *code* changes, not spec artifacts.
pub fn git_changed_files(ws: &Workspace) -> Vec<String> {
    let root = &ws.root;
    // Only when the project root is itself the git root: a project
    // nested inside an ancestor repo records nothing, instead of walking up and
    // capturing dirty files from outside the project.
    if !root.join(".git").exists() {
        return Vec::new();
    }
    let spec_prefix = format!("{}/", ws.spec_dir_name);
    // NB: use the RAW (untrimmed) output — porcelain's first column is a significant leading space
    // for work-tree-modified files (" M path"); trimming it shifts the path by one character.
    // `core.quotepath=false` keeps non-ASCII paths raw, matching every other
    // git call in the codebase; without it these paths arrive octal-escaped and
    // never compare equal to a path read from anywhere else.
    let Some(out) =
        util::git_raw(root, &["-c", "core.quotepath=false", "status", "--porcelain", "-uall"])
    else {
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
        let path = unquote_porcelain_path(path).replace('\\', "/");
        if path.is_empty() || path.ends_with('/') {
            continue; // skip directory entries
        }
        // Exclude spec artifacts, work files, and tool-scaffolding dirs from the code trace
        // (CLAUDE.md / config are recorded, .claude/.agents/.cursor/.gemini and .gitignore are not).
        if path.starts_with(&spec_prefix)
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
    use std::path::PathBuf;

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

        /// The evidence record's home: the change directory's `.evidence.json`,
        /// spelled out literally so the test pins the on-disk location rather
        /// than trusting the path helper it verifies.
        fn evidence_json(&self, change: &str) -> Option<String> {
            util::read_opt(
                &self
                    .ws
                    .root
                    .join("openspec")
                    .join("changes")
                    .join(change)
                    .join(".evidence.json"),
            )
        }

        /// The legacy `.speclink/touched/<change>.json` location — read-only
        /// compatibility, never a write target.
        fn legacy_json(&self, change: &str) -> Option<String> {
            util::read_opt(&self.ws.touched_dir().join(format!("{change}.json")))
        }

        /// Seed a record at the legacy location (the pre-move on-disk shape).
        fn seed_legacy(&self, change: &str, json: &str) -> PathBuf {
            let p = self.ws.touched_dir().join(format!("{change}.json"));
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, json).unwrap();
            p
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

        let out = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution { identity: Some("Tester <t@example.com>"), ..Default::default() }).unwrap();

        assert!(!out.already);
        assert_eq!(out.description, "1.1 first task");
        let tasks = store.read_artifact("demo", "tasks.md").unwrap();
        assert!(tasks.contains("- [x] 1.1 first task"), "task 1 must be checked: {tasks}");
        assert!(tasks.contains("- [ ] 1.2 second task"), "task 2 must stay open: {tasks}");
        // Touched record gains this task's entry with the unclaimed dirty file.
        let json = t.evidence_json("demo").expect("touched record written");
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
        let out = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default()).unwrap();
        assert!(!out.already);
        assert_eq!(t.evidence_json("demo"), None, "no unclaimed dirty files must append nothing");
    }

    #[test]
    fn complete_on_started_change_keeps_first_stamp_verbatim() {
        let started = format!(
            "{META_UNSTARTED}started_at: 2026-07-01\nstarted_by: First <first@example.com>\nstarted_with: claude\n"
        );
        let t = TempWs::new("started");
        let store = store_with(&started, TASKS_TWO_OPEN);
        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(2), &CompleteAttribution { identity: Some("Second <second@example.com>"), agent: Some("codex"), repo: None })
            .unwrap();
        assert_eq!(store.meta("demo"), started, "first stamp must be kept verbatim");
        assert_eq!(*store.meta_writes.borrow(), 0, "already-started change must not write meta");
    }

    #[test]
    fn complete_already_done_task_reports_already_without_any_file_effect() {
        let t = TempWs::with_dirty_file("already", "src/lib.rs");
        let tasks_md = "- [x] 1.1 finished\n- [ ] 1.2 open\n";
        let store = store_with(META_UNSTARTED, tasks_md);
        let out = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution { identity: Some("Tester <t@example.com>"), ..Default::default() }).unwrap();
        assert!(out.already);
        assert_eq!(out.description, "1.1 finished");
        assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), tasks_md);
        assert_eq!(*store.artifact_writes.borrow(), 0, "already-done must not rewrite tasks.md");
        assert_eq!(*store.meta_writes.borrow(), 0, "already-done must not stamp meta");
        assert_eq!(t.evidence_json("demo"), None, "already-done must not record touched files");
    }

    #[test]
    fn complete_with_absent_identity_and_agent_stamps_only_started_at() {
        // Attribution follows the created_* rule: what the caller cannot attribute
        // is absent, not defaulted.
        let t = TempWs::new("absent");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default()).unwrap();
        let meta = store.meta("demo");
        assert!(meta.contains("started_at: "));
        assert!(!meta.contains("started_by"));
        assert!(!meta.contains("started_with"));
    }

    #[test]
    fn complete_out_of_range_task_id_errors_without_writes() {
        let t = TempWs::new("range");
        let store = store_with(META_UNSTARTED, TASKS_TWO_OPEN);
        let err = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(5), &CompleteAttribution::default()).unwrap_err();
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
        let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(2)).unwrap();
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
        let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(2)).unwrap();
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
        let out = uncomplete(&store, "demo", &TaskAddr::Ordinal(4)).unwrap();
        assert!(out.already);
        assert_eq!(out.description, "1.4 open task");
        assert_eq!(store.read_artifact("demo", "tasks.md").unwrap(), TASKS_MIXED_DONE);
        assert_eq!(*store.artifact_writes.borrow(), 0, "already-open must not rewrite tasks.md");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn uncomplete_out_of_range_task_id_errors_without_writes() {
        let store = store_with(META_UNSTARTED, TASKS_MIXED_DONE);
        let err = uncomplete(&store, "demo", &TaskAddr::Ordinal(9)).unwrap_err();
        assert_eq!(err.to_string(), "Task 9 not found (total: 4)");
        assert_eq!(*store.artifact_writes.borrow(), 0);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn uncomplete_missing_tasks_md_errors() {
        let store = TestStore::with_meta("demo", META_UNSTARTED);
        let err = uncomplete(&store, "demo", &TaskAddr::Ordinal(1)).unwrap_err();
        assert_eq!(err.to_string(), "tasks.md not found for change 'demo'");
    }

    // --- Stable task ID: comment parsing, duplicate detection, generator ---

    const TASKS_WITH_IDS: &str = "## 1. Group\n\n- [ ] 1.1 first task <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n- [x] 1.2 second task <!-- speclink-task:tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ -->\n";

    #[test]
    fn parse_extracts_stable_id_and_strips_comment_from_description() {
        let tasks = parse(TASKS_WITH_IDS);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].description, "1.1 first task");
        assert_eq!(tasks[0].stable_id.as_deref(), Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"));
        assert_eq!(tasks[1].description, "1.2 second task");
        assert_eq!(tasks[1].stable_id.as_deref(), Some("tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ"));
    }

    #[test]
    fn parse_without_comment_yields_no_stable_id_and_unchanged_description() {
        let tasks = parse(TASKS_TWO_OPEN);
        assert!(tasks.iter().all(|t| t.stable_id.is_none()), "no comment → no stable id");
        assert_eq!(tasks[0].description, "1.1 first task");
        assert_eq!(tasks[1].description, "1.2 second task");
        // sharp-edge：空 ID 註解不是身分——空字串 ID 會讓 key/定址靜默歧義。
        let empty = parse("- [ ] 1.1 x <!-- speclink-task: -->\n");
        assert!(empty[0].stable_id.is_none(), "empty marker id must not count as identity");
    }

    // --- spec「任務行的手動測試標記與解析」---

    #[test]
    fn parse_reads_manual_marker_and_strips_it_from_description() {
        let tasks = parse("- [ ] [M] 手測匯入\n- [ ] 寫解析器\n");
        assert!(tasks[0].manual, "[M] 前綴須解析為 manual");
        assert_eq!(tasks[0].description, "手測匯入", "描述須剝除標記");
        assert!(!tasks[1].manual, "無標記任務不得為 manual");
        assert_eq!(tasks[1].description, "寫解析器");
    }

    #[test]
    fn parse_accepts_both_markers_in_either_order() {
        for line in ["- [x] [M] [P] 手測匯入\n", "- [x] [P] [M] 手測匯入\n"] {
            let t = &parse(line)[0];
            assert!(t.manual, "[M] 須順序不敏感：{line}");
            assert_eq!(t.description, "手測匯入", "兩標記皆須剝除：{line}");
        }
    }

    #[test]
    fn parse_strips_legacy_parallel_marker_without_carrying_a_flag() {
        // spec Example「前綴解析」表逐列：任務行 → (manual, 描述)。
        // 案例表與 packages/ui/src/tasks.ts 的 stripMarkers 測試對齊——UI 解析
        // 與引擎同構,動標記規則兩處要一起改。
        let rows: [(&str, bool, &str); 6] = [
            ("- [ ] [M] 手測匯入\n", true, "手測匯入"),
            ("- [x] [P] 舊任務\n", false, "舊任務"),
            ("- [x] [P] [M] 混用\n", true, "混用"),
            ("- [ ] 寫解析器\n", false, "寫解析器"),
            // checkbox 後恰一個空格才進標記槽:兩空格＝標記不成立、字面留在描述。
            ("- [ ]  [M] 手測\n", false, "[M] 手測"),
            // 標記後多餘空白剝除後修剪(display.trim())。
            ("- [ ] [M]  雙空格描述\n", true, "雙空格描述"),
        ];
        for (line, manual, desc) in rows {
            let t = &parse(line)[0];
            assert_eq!(t.manual, manual, "manual 判定：{line}");
            assert_eq!(t.description, desc, "描述須剝除全部前綴標記：{line}");
        }
        // 舊 `[P]` 只剝不承載——與同內容的無標記行解析結果全等
        let legacy = &parse("- [x] [P] 舊任務\n")[0];
        let plain = &parse("- [x] 舊任務\n")[0];
        assert_eq!((legacy.manual, &legacy.description), (plain.manual, &plain.description));
        // 至多一次：第二個 [P] 留在描述裡,不被吃掉
        assert_eq!(parse("- [ ] [P] [P] 兩個\n")[0].description, "[P] 兩個");
    }

    #[test]
    fn flip_task_reports_marker_free_description() {
        // spec「任務行的手動測試標記與解析」:顯示描述 SHALL 剝除全部前綴標記——
        // 勾選回報的 desc 與 parse() 的乾淨描述必須同一份真相。
        let rows: [(&str, &str); 3] = [
            ("- [ ] [M] 手測匯入\n", "手測匯入"),
            ("- [ ] [P] 舊任務\n", "舊任務"),
            ("- [ ] [P] [M] 混用\n", "混用"),
        ];
        for (md, want) in rows {
            let (_, desc, _, _) = mark_done(md, 1).expect("task found");
            assert_eq!(desc, want, "flip 回報描述須剝除標記:{md}");
        }
    }

    #[test]
    fn parse_leaves_unmarked_lines_untouched() {
        let tasks = parse(TASKS_WITH_IDS);
        assert!(tasks.iter().all(|t| !t.manual));
        assert_eq!(tasks[0].description, "1.1 first task");
        assert_eq!(tasks[0].stable_id.as_deref(), Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"));
    }

    // --- spec「標記位置的 change 驗證檢查」---

    #[test]
    fn misplaced_markers_follow_the_spec_example_table() {
        // spec Example「誤置判定」表逐列：任務行 → 判定。
        let rows: [(&str, Option<MisplacedMarker>); 5] = [
            ("- [ ] [M] 3.2 手測匯入\n", None),
            ("- [ ] 3.2 [M] 手測匯入\n", Some(MisplacedMarker::AfterNumber)),
            ("- [ ] 1.10 [M] 手測\n", Some(MisplacedMarker::AfterNumber)),
            ("- [ ]  [M] 手測\n", Some(MisplacedMarker::PrefixSlotMissed)),
            ("- [ ] 說明 `[M]` 剝除規則\n", None),
        ];
        for (line, want) in rows {
            let found = misplaced_markers(&parse(line));
            assert_eq!(found.first().map(|m| m.kind), want, "誤置判定：{line}");
        }
    }

    #[test]
    fn misplaced_markers_report_task_id_and_description() {
        let md = "- [x] 1.1 寫解析器\n- [ ] 6.2 [M] 手動驗收\n";
        let found = misplaced_markers(&parse(md));
        assert_eq!(found.len(), 1, "只有第二行誤置");
        assert_eq!(found[0].task_id, 2, "序號須為全檔 checkbox 順序");
        assert_eq!(found[0].description, "6.2 [M] 手動驗收", "描述原樣回報供訊息引文");
        assert!(!found[0].done, "未勾任務回報 done=false");
        assert_eq!(found[0].stable_id, None, "無 ID 註解回報 None");
    }

    #[test]
    fn misplaced_markers_carry_checkbox_state_and_stable_id() {
        // 修復例要能忠實重建原行:勾選狀態與尾部 ID 註解都得跟著回報,
        // 否則照訊息逐字改行會退勾、斷任務身分。
        let md = "- [x] 5.2 [M] 手動驗收 <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n";
        let found = misplaced_markers(&parse(md));
        assert_eq!(found.len(), 1);
        assert!(found[0].done, "已勾任務回報 done=true");
        assert_eq!(
            found[0].stable_id.as_deref(),
            Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            "ID 註解須原樣攜帶"
        );
        assert_eq!(found[0].description, "5.2 [M] 手動驗收", "描述仍為去尾顯示文字");
    }

    #[test]
    fn misplaced_markers_ignore_mid_description_mentions() {
        // 反引號包裹或行文中段提及 [M] 不構成違規——本 repo 既有 tasks.md 大量存在。
        let md = "- [x] 1.1 前綴剝除迴圈同時接受 `[P]` 與 `[M]` 的說明文字\n\
                  - [ ] 2.1 改寫 [M] 起草指引\n\
                  - [ ] [M] 手測匯入\n";
        assert!(misplaced_markers(&parse(md)).is_empty(), "中段提及與正確前綴皆不得命中");
    }

    #[test]
    fn misplaced_markers_check_done_tasks_alike() {
        // 誤置是格式錯誤,與完成狀態無關。
        let found = misplaced_markers(&parse("- [x] 3.3 [M] 已勾的手測\n"));
        assert_eq!(found.len(), 1, "已勾任務同等檢查");
        assert_eq!(found[0].kind, MisplacedMarker::AfterNumber);
    }

    #[test]
    fn misplaced_markers_stay_silent_on_clean_task_lists() {
        // 無命中時零輸出——validate 既有輸出逐位元不變的前提。
        for md in ["", "- [ ] 1.1 寫解析器\n- [x] [M] 手測\n- [ ] [P] [M] 混用\n"] {
            assert!(misplaced_markers(&parse(md)).is_empty(), "乾淨清單須零回報：{md:?}");
        }
    }

    // --- spec「寫碼任務全完成預測子」---

    #[test]
    fn counts_split_code_tasks_from_manual_ones() {
        // spec Example 表逐列：全量(完成/總數) → 寫碼(完成/總數) → 預測子
        let rows: [(&str, (usize, usize), (usize, usize), bool); 4] = [
            // 9/10 全量、9/9 寫碼 → 成立
            (
                "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] e\n\
                 - [x] f\n- [x] g\n- [x] h\n- [x] i\n- [ ] [M] 手測\n",
                (9, 10),
                (9, 9),
                true,
            ),
            // 7/10 全量、7/8 寫碼 → 不成立
            (
                "- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [x] e\n\
                 - [x] f\n- [x] g\n- [ ] h\n- [ ] [M] m1\n- [ ] [M] m2\n",
                (7, 10),
                (7, 8),
                false,
            ),
            // 0/0 → 空真成立
            ("", (0, 0), (0, 0), true),
            // 全為 [M]：0/2 全量、0/0 寫碼 → 空真成立
            ("- [ ] [M] m1\n- [ ] [M] m2\n", (0, 2), (0, 0), true),
        ];
        for (md, (complete, total), (code_complete, code_total), predicate) in rows {
            let c = counts(&parse(md));
            assert_eq!((c.complete, c.total), (complete, total), "全量計數：{md}");
            assert_eq!(
                (c.code_complete, c.code_total),
                (code_complete, code_total),
                "寫碼計數：{md}"
            );
            assert_eq!(c.code_done(), predicate, "預測子：{md}");
        }
    }

    #[test]
    fn progress_tuple_stays_on_full_counts() {
        let tasks = parse("- [x] a\n- [ ] [M] 手測\n");
        assert_eq!(progress(&tasks), (2, 1, 1), "既有 progress 契約仍為全量計數");
    }

    #[test]
    fn duplicate_stable_ids_are_detected_and_listed() {
        let md = "- [ ] 1.1 a <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n\
                  - [ ] 1.2 b <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n\
                  - [ ] 1.3 c <!-- speclink-task:tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ -->\n";
        let tasks = parse(md);
        assert_eq!(
            duplicate_stable_ids(&tasks),
            vec!["tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()]
        );
        assert!(duplicate_stable_ids(&parse(TASKS_WITH_IDS)).is_empty());
    }

    #[test]
    fn new_stable_id_has_tsk_prefix_and_26_ulid_chars() {
        let id = new_stable_id();
        let ulid = id.strip_prefix("tsk_").expect("id must carry tsk_ prefix");
        assert_eq!(ulid.len(), 26, "ULID must be 26 chars: {id}");
        assert!(ulid.chars().all(|c| c.is_ascii_alphanumeric()), "ULID must be alphanumeric: {id}");
    }

    #[test]
    fn new_stable_id_is_time_ordered() {
        let a = new_stable_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = new_stable_id();
        assert!(a < b, "later id must sort after earlier: {a} vs {b}");
    }

    // --- 蓋章時機：產出全檔蓋章、task done 單行補章（spec task-identity）---

    #[test]
    fn stamp_all_assigns_unique_ids_and_preserves_everything_else() {
        let md = "## 1. Group\n\n- [ ] 1.1 first task\n- [x] 1.2 second task <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n* [ ] 1.3 star task\n\nProse line.\n";
        let stamped = stamp_all(md);
        let tasks = parse(&stamped);
        assert_eq!(tasks.len(), 3);
        assert!(tasks.iter().all(|t| t.stable_id.is_some()), "every task gains an id: {stamped}");
        assert_eq!(
            tasks[1].stable_id.as_deref(),
            Some("tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV"),
            "already-stamped line keeps its id"
        );
        let mut ids: Vec<String> = tasks.iter().filter_map(|t| t.stable_id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3, "ids must be unique");
        assert_eq!(tasks[0].description, "1.1 first task");
        assert_eq!(tasks[2].description, "1.3 star task");
        let orig: Vec<&str> = md.lines().collect();
        let new: Vec<&str> = stamped.lines().collect();
        assert_eq!(orig.len(), new.len());
        for (o, n) in orig.iter().zip(&new) {
            let t = o.trim_start();
            if t.starts_with("- [ ]") || t.starts_with("* [ ]") {
                assert!(n.starts_with(o), "stamp appends at the line end: {n}");
                assert!(n.ends_with("-->"), "stamp is a trailing comment: {n}");
            } else {
                assert_eq!(o, n, "non-target lines byte-identical");
            }
        }
        assert!(stamped.ends_with('\n'), "trailing newline preserved");
    }

    #[test]
    fn reorder_keeps_stable_ids_and_swaps_ordinals() {
        let stamped = stamp_all("- [ ] 1.1 alpha\n- [ ] 1.2 beta\n");
        let before = parse(&stamped);
        let mut lines: Vec<&str> = stamped.lines().collect();
        lines.swap(0, 1);
        let swapped = format!("{}\n", lines.join("\n"));
        let after = parse(&swapped);
        assert_eq!(after[0].description, "1.2 beta");
        assert_eq!(after[0].stable_id, before[1].stable_id, "id follows the task, not the slot");
        assert_eq!(after[1].stable_id, before[0].stable_id);
        assert_eq!((after[0].id, after[1].id), (1, 2), "ordinals follow position");
    }

    #[test]
    fn mark_done_stamps_only_the_unstamped_target_line() {
        let md = "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n- [ ] 1.3 third\n- [ ] 1.4 fourth\n";
        let (new_content, desc, already, _) = mark_done(md, 3).unwrap();
        assert!(!already);
        assert_eq!(desc, "1.3 third");
        let orig: Vec<&str> = md.lines().collect();
        let new: Vec<&str> = new_content.lines().collect();
        assert_eq!(orig.len(), new.len());
        for (i, (o, n)) in orig.iter().zip(&new).enumerate() {
            if i == 4 {
                assert!(
                    n.starts_with("- [x] 1.3 third <!-- speclink-task:tsk_"),
                    "target line must be checked and stamped: {n}"
                );
                assert!(n.ends_with("-->"));
            } else {
                assert_eq!(o, n, "line {i} must stay byte-identical");
            }
        }
        let tasks = parse(&new_content);
        assert!(tasks[2].stable_id.is_some(), "freshly stamped id is addressable on re-read");
    }

    #[test]
    fn mark_done_on_stamped_line_keeps_comment_verbatim() {
        let md = "- [ ] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n";
        let (new_content, desc, _, _) = mark_done(md, 1).unwrap();
        assert_eq!(
            new_content,
            "- [x] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n"
        );
        assert_eq!(desc, "1.1 first", "description strips the marker");
    }

    #[test]
    fn mark_undone_never_stamps() {
        let md = "- [x] 1.1 first\n";
        let (new_content, desc, _, _) = mark_undone(md, 1).unwrap();
        assert_eq!(new_content, "- [ ] 1.1 first\n", "undone flips without stamping");
        assert_eq!(desc, "1.1 first");
    }

    // --- evidence 記錄 v2（spec verify-evidence: task done 寫入逐任務 evidence）---

    impl TempWs {
        /// Workspace root as a git repo with a fixed identity, one commit
        /// (HEAD exists), and one unclaimed dirty file.
        fn with_commit_and_dirty(tag: &str, dirty_rel: &str) -> TempWs {
            let t = TempWs::new(tag);
            let git = |args: &[&str]| {
                let ok = std::process::Command::new("git")
                    .arg("-C")
                    .arg(&t.ws.root)
                    .args(args)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                assert!(ok, "git {args:?} failed");
            };
            git(&["init", "-q"]);
            git(&["config", "user.name", "Evidence Tester"]);
            git(&["config", "user.email", "ev@example.com"]);
            std::fs::write(t.ws.root.join("seed.txt"), "seed\n").unwrap();
            git(&["add", "seed.txt"]);
            git(&["commit", "-q", "-m", "seed"]);
            let p = t.ws.root.join(dirty_rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, "dirty\n").unwrap();
            t
        }
    }

    #[test]
    fn git_changed_files_decodes_quoted_non_ascii_paths() {
        // `git status --porcelain` C-quotes any non-ASCII path unless
        // core.quotepath=false. The change-diff resolver already passes that
        // flag, so an escaped path here would never compare equal to the
        // resolver's — silently opening the dirty-at-start fail-closed guard.
        let t = TempWs::with_commit_and_dirty("quotepath", "src/app.rs");
        std::fs::write(t.ws.root.join("規格.md"), "內容\n").unwrap();
        let files = git_changed_files(&t.ws);
        assert!(
            files.contains(&"規格.md".to_string()),
            "non-ASCII path must come back decoded: {files:?}"
        );
        assert!(
            !files.iter().any(|f| f.contains('\\')),
            "no octal escapes may survive: {files:?}"
        );
    }

    #[test]
    fn the_spec_dir_exclusion_follows_the_workspace_name() {
        // 排除的是「這個 workspace 的 spec 目錄」，不是字面上的 openspec/——
        // 自訂目錄名的專案裡，`.evidence.json` 落在 `<custom>/changes/<name>/` 之下，
        // 寫死字串會讓第二次 task done 把證據檔自己記成碰過的程式檔。
        let mut t = TempWs::with_dirty_file("customdir", "src/app.rs");
        t.ws.spec_dir_name = "customspec".to_string();
        let ev = t.ws.root.join("customspec/changes/demo/.evidence.json");
        std::fs::create_dir_all(ev.parent().unwrap()).unwrap();
        std::fs::write(&ev, "{}\n").unwrap();

        let files = git_changed_files(&t.ws);
        assert_eq!(
            files,
            vec!["src/app.rs".to_string()],
            "spec artifacts under the workspace's own dir name stay out: {files:?}"
        );
    }

    #[test]
    fn complete_writes_v2_evidence_entry_with_attribution_and_no_basis() {
        let t = TempWs::with_commit_and_dirty("evidence", "src/app.rs");
        let store = store_with(
            META_UNSTARTED,
            "- [ ] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n",
        );
        store.put_artifact("demo", "specs/auth/spec.md", "## ADDED Requirements\n");
        let attr = CompleteAttribution {
            identity: Some("Tester <t@example.com>"),
            agent: None,
            repo: Some("main"),
        };
        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &attr).unwrap();

        let json = t.evidence_json("demo").expect("evidence record written");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], 2, "writes are always v2: {json}");
        assert_eq!(v["change"], "demo");
        let e = &v["entries"][0];
        assert_eq!(e["taskId"], "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(e["actor"], "Tester <t@example.com>");
        assert_eq!(e["repo"], "main");
        let head = e["headCommit"].as_str().expect("headCommit present");
        assert_eq!(head.len(), 40, "full HEAD sha expected: {head}");
        let files: Vec<&str> =
            e["touchedFiles"].as_array().unwrap().iter().filter_map(|f| f.as_str()).collect();
        assert!(files.contains(&"src/app.rs"), "dirty file recorded: {files:?}");
        assert!(e.get("basisDigests").is_none(), "the entry records no verification basis: {json}");
        let at = e["recordedAt"].as_str().expect("recordedAt present");
        assert!(at.ends_with('Z'), "recordedAt is UTC: {at}");
        chrono::DateTime::parse_from_rfc3339(at).expect("recordedAt parses as RFC3339");
    }

    #[test]
    fn v1_record_reads_with_unchanged_file_list_semantics() {
        // spec Scenario「舊格式記錄可讀」：change 目錄缺席時回退舊路徑，v1 語意不變。
        let t = TempWs::new("v1compat");
        t.seed_legacy(
            "demo",
            "{\n  \"change\": \"demo\",\n  \"touched\": [\n    {\n      \"task_id\": \"1\",\n      \"task_desc\": \"1.1 legacy\",\n      \"files\": [\"src/a.rs\", \"src/b.rs\"]\n    }\n  ]\n}",
        );
        let rec = TouchedRecord::load(&t.ws, "demo");
        assert_eq!(
            rec.all_files(),
            vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
            "v1 file-list semantics unchanged"
        );
    }

    #[test]
    fn a_record_carrying_basis_digests_still_reads_and_new_writes_omit_them() {
        // spec Scenario「舊格式記錄可讀」：前一版寫入的 basisDigests 是未知欄位——
        // 讀取端忽略它、all_files 不變；本次寫入的 entry 不再帶該欄位。
        let t = TempWs::with_commit_and_dirty("basis-compat", "src/app.rs");
        util::write_file(
            &t.ws.change_evidence_file("demo"),
            r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_OLD","taskDesc":"1.1 old","touchedFiles":["src/old.rs"],"basisDigests":{"spec":"sha256:0","tasks":"sha256:0","policy":"sha256:0"},"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
        )
        .unwrap();

        let rec = TouchedRecord::load(&t.ws, "demo");
        assert_eq!(rec.entries.len(), 1, "the legacy-shaped entry survives the read");
        assert_eq!(rec.entries[0].task_id, "tsk_OLD");
        assert_eq!(rec.all_files(), vec!["src/old.rs".to_string()], "all_files unchanged");

        let store = store_with(META_UNSTARTED, "- [ ] 1.1 new\n");
        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default())
            .unwrap();
        let json = t.evidence_json("demo").expect("record rewritten");
        assert!(
            !json.contains("basisDigests"),
            "writes no longer record a verification basis: {json}"
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["entries"].as_array().unwrap().len(), 2, "the old entry is carried forward");
    }

    #[test]
    fn v2_record_at_the_legacy_path_reads_back_through_the_fallback() {
        // spec Scenario「舊格式記錄可讀」：v1 與 v2 皆可自舊路徑讀回。
        let t = TempWs::new("v2compat");
        t.seed_legacy(
            "demo",
            r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_LEGACY","taskDesc":"1.1 legacy","touchedFiles":["src/legacy.rs"],"basisDigests":{"spec":"sha256:aa","tasks":"sha256:bb","policy":"sha256:cc"},"recordedAt":"2026-07-01T00:00:00Z"}]}"#,
        );
        let rec = TouchedRecord::load(&t.ws, "demo");
        assert_eq!(rec.version, Some(2));
        assert_eq!(rec.entries.len(), 1, "v2 entries must survive the fallback read");
        assert_eq!(rec.entries[0].task_id, "tsk_LEGACY");
        assert_eq!(rec.all_files(), vec!["src/legacy.rs".to_string()]);
    }

    #[test]
    fn evidence_lands_in_the_change_directory_not_the_legacy_path() {
        // spec Scenario「完成任務後證據齊全」：記錄的家是 change 目錄的 `.evidence.json`。
        let t = TempWs::with_commit_and_dirty("home", "src/app.rs");
        let store = store_with(
            META_UNSTARTED,
            "- [ ] 1.1 first <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n",
        );
        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default())
            .unwrap();

        let json = t.evidence_json("demo").expect("evidence written to the change directory");
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], 2);
        assert_eq!(v["change"], "demo");
        assert_eq!(v["entries"][0]["taskId"], "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(
            t.legacy_json("demo"),
            None,
            "the legacy path must never be a write target again"
        );
    }

    #[test]
    fn a_legacy_record_is_read_back_then_rewritten_to_the_change_directory() {
        // spec Scenario「舊格式記錄可讀」的後半：下一次 task done 寫入後，新位置成為讀取來源。
        // 舊檔隨之移除——留著的話，這個名字日後重建 change 時，第一次 load 會把
        // 死帳讀成活帳（seen 汙染、零證據提示被吞）。
        let t = TempWs::with_commit_and_dirty("migrate", "src/app.rs");
        let seeded = "{\"change\":\"demo\",\"touched\":[{\"task_id\":\"1\",\"task_desc\":\"1.1 legacy\",\"files\":[\"src/legacy.rs\"]}]}";
        let legacy = t.seed_legacy("demo", seeded);
        let store = store_with(META_UNSTARTED, "- [ ] 1.1 first\n- [ ] 1.2 second\n");

        complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default())
            .unwrap();

        let rec: TouchedRecord =
            serde_json::from_str(&t.evidence_json("demo").expect("new home written")).unwrap();
        assert!(
            rec.all_files().contains(&"src/legacy.rs".to_string()),
            "the legacy record must be carried forward, not dropped: {:?}",
            rec.all_files()
        );
        assert!(rec.all_files().contains(&"src/app.rs".to_string()));
        assert!(
            !legacy.exists(),
            "the legacy bridge file is removed once the record lands in the change directory"
        );
        assert_eq!(
            TouchedRecord::load(&t.ws, "demo").all_files(),
            rec.all_files(),
            "the change directory is now the read source"
        );
    }

    #[test]
    fn uncomplete_never_writes_or_alters_evidence() {
        let t = TempWs::new("undone-evidence");
        let seeded = "{\"change\":\"demo\",\"touched\":[{\"task_id\":\"1\",\"task_desc\":\"1.1 a\",\"files\":[\"src/a.rs\"]}]}";
        let p = t.seed_legacy("demo", seeded);
        let store = store_with(META_UNSTARTED, "- [x] 1.1 a\n");
        uncomplete(&store, "demo", &TaskAddr::Ordinal(1)).unwrap();
        assert_eq!(
            std::fs::read_to_string(&p).unwrap(),
            seeded,
            "undone must not write or alter any evidence record"
        );
        assert_eq!(t.evidence_json("demo"), None, "undone must not create the new-home record");
    }

    // --- 任務搬移＋重編號（自 desktop core 遷入；spec「任務搬移端點與重編號效果」）---

    fn tasks_text(store: &TestStore) -> String {
        store.read_artifact("demo", "tasks.md").unwrap()
    }

    fn move_lines(store: &TestStore) -> Vec<String> {
        tasks_text(store)
            .lines()
            .filter(|l| l.trim_start().starts_with("- ["))
            .map(|l| l.to_string())
            .collect()
    }

    #[test]
    fn move_within_group_renumbers_prefixes_per_spec_example() {
        // spec Example「組內移動重編號」：把 1.1 甲拖到末位 → 乙丙甲，前綴重寫 1.1/1.2/1.3。
        let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n");
        let out = move_task(&store, "demo", 1, 3, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲"],
            "prefixes must follow the new order"
        );
        assert_eq!(out.description, "1.3 甲", "outcome carries the post-move description");
        assert!(tasks_text(&store).contains("## 1. 群組"), "group heading untouched");
    }

    #[test]
    fn move_up_infers_insert_before_the_anchor() {
        let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n");
        let out = move_task(&store, "demo", 3, 1, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.1 丙", "- [ ] 1.2 甲", "- [x] 1.3 乙"],
            "an upward move must insert before the anchor"
        );
        assert_eq!(out.description, "1.1 丙");
    }

    #[test]
    fn move_across_groups_takes_the_new_groups_numbering() {
        // spec scenario「跨群組搬移重編號」：把第 1 個任務移到第 3 個任務之後。
        let store = store_with(
            META_UNSTARTED,
            "## 1. 前段\n\n說明文字原樣保留。\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
        );
        let out = move_task(&store, "demo", 1, 3, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.1 乙", "- [ ] 2.1 丙", "- [ ] 2.2 甲", "- [ ] 2.3 丁"]
        );
        assert_eq!(out.description, "2.2 甲", "moved task takes the new group's numbering");
        let text = tasks_text(&store);
        assert!(text.contains("說明文字原樣保留。"), "prose lines byte-identical");
        assert!(text.contains("## 1. 前段") && text.contains("## 2. 後段"));
    }

    #[test]
    fn downward_move_to_group_end_stays_in_the_origin_group() {
        // 向下拖到群組末位必須落在錨 checkbox 之後（留在原群組），不得吞進下一群組。
        let store = store_with(
            META_UNSTARTED,
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [x] 1.2 乙\n- [ ] 1.3 丙\n\n## 2. 後段\n\n- [ ] 2.1 丁\n- [ ] 2.2 戊\n",
        );
        move_task(&store, "demo", 1, 3, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [x] 1.1 乙", "- [ ] 1.2 丙", "- [ ] 1.3 甲", "- [ ] 2.1 丁", "- [ ] 2.2 戊"],
            "a downward move onto the last task of a group must not leak into the next group"
        );
        let text = tasks_text(&store);
        let g2 = text.split("## 2. 後段").nth(1).unwrap();
        assert!(!g2.contains("甲"), "甲 must stay under group 1: {text}");
    }

    #[test]
    fn before_true_crosses_the_heading_and_becomes_group_head() {
        // 明確 before=true：插於錨任務行之前，跨過群組標題成為錨所屬群組的組首。
        let store = store_with(
            META_UNSTARTED,
            "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
        );
        move_task(&store, "demo", 2, 3, Some(true)).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.1 甲", "- [ ] 2.1 乙", "- [ ] 2.2 丙", "- [ ] 2.3 丁"],
            "before=true must insert ahead of the anchor line, across the heading"
        );
        let text = tasks_text(&store);
        let g2 = text.split("## 2. 後段").nth(1).unwrap();
        assert!(g2.contains("乙"), "乙 must live under group 2: {text}");
    }

    #[test]
    fn before_false_explicitly_inserts_after_the_anchor() {
        // 明確側別覆蓋方向推斷：向上移動＋before=false → 落在錨任務之後。
        let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n- [ ] 1.3 丙\n");
        move_task(&store, "demo", 3, 1, Some(false)).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.1 甲", "- [ ] 1.2 丙", "- [ ] 1.3 乙"],
            "before=false anchors after task 1 even on an upward move"
        );
    }

    #[test]
    fn renumber_leaves_unprefixed_and_sub_versioned_text_verbatim() {
        // 重編號只改「數字.數字＋空白」前綴：無前綴與子版號（1.2.3）逐字元保留。
        let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 補充說明不帶編號\n- [ ] 1.2 乙\n");
        move_task(&store, "demo", 3, 1, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.1 乙", "- [ ] 1.2 甲", "- [ ] 補充說明不帶編號"],
            "unprefixed task text must stay untouched while others renumber"
        );
        let store = store_with(META_UNSTARTED, "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2.3 子版號文字\n");
        move_task(&store, "demo", 2, 1, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.2.3 子版號文字", "- [ ] 1.2 甲"],
            "sub-versioned prefixes must be preserved verbatim"
        );
    }

    #[test]
    fn groups_without_numeric_heading_are_not_renumbered() {
        let store = store_with(META_UNSTARTED, "## 準備\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
        move_task(&store, "demo", 1, 2, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
            "a heading without a numeric prefix must leave its tasks' numbers alone"
        );
        // sharp-edge：標題數字超出 u64——解析失敗即視為無數字標題，任務保留原文。
        let store = store_with(META_UNSTARTED, "## 99999999999999999999999. 巨數\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
        move_task(&store, "demo", 1, 2, None).unwrap();
        assert_eq!(
            move_lines(&store),
            vec!["- [ ] 1.2 乙", "- [ ] 1.1 甲"],
            "unparseable group numbers must not rewrite anything"
        );
    }

    #[test]
    fn out_of_range_move_errors_without_writes() {
        // spec scenario「越界拒絕零副作用」：只有 3 個任務時 from=5 拒絕，內容不變。
        const TASKS: &str = "## 1. 群組\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n- [ ] 1.3 丙\n";
        let store = store_with(META_UNSTARTED, TASKS);
        for (from, to) in [(5usize, 1usize), (0, 1), (1, 0), (1, 9)] {
            let err = move_task(&store, "demo", from, to, None).unwrap_err();
            assert!(
                err.to_string().contains("out of range"),
                "({from},{to}) must name the out-of-range refusal: {err}"
            );
        }
        assert_eq!(tasks_text(&store), TASKS, "failed moves must not rewrite the file");
        assert_eq!(*store.artifact_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn move_missing_tasks_md_errors() {
        let store = TestStore::with_meta("demo", META_UNSTARTED);
        let err = move_task(&store, "demo", 1, 2, None).unwrap_err();
        assert_eq!(err.to_string(), "tasks.md not found for change 'demo'");
        assert_eq!(*store.artifact_writes.borrow(), 0);
    }

    #[test]
    fn move_preserves_trailing_newline_state() {
        let store = store_with(META_UNSTARTED, "- [ ] 1.1 甲\n- [ ] 1.2 乙\n");
        move_task(&store, "demo", 1, 2, None).unwrap();
        assert!(tasks_text(&store).ends_with("\n"), "trailing newline preserved");
        let store = store_with(META_UNSTARTED, "- [ ] 1.1 甲\n- [ ] 1.2 乙");
        move_task(&store, "demo", 1, 2, None).unwrap();
        assert!(!tasks_text(&store).ends_with("\n"), "absent trailing newline preserved");
    }
}

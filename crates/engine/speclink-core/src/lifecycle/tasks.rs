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

/// The manual-task marker literal. The prefix-slot stripping, the
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
                // 剝離規則只有一份真相(spec「任務行的手動任務標記與解析」)。
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
    /// The files this completion recorded as evidence — empty when nothing new
    /// was attributable (no record was written either).
    pub touched_files: Vec<String>,
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

/// Where a completion's touched-file candidates come from (design 決策一).
/// A closed choice with no default: the Host either resolved the candidates at
/// its own boundary (server mode — the wire request's `touchedFiles` and
/// `headCommit`) or hands over the workspace to probe. A caller cannot lose
/// evidence by forgetting to say which.
pub enum TouchedCandidates<'a> {
    /// The Host already resolved the candidates (and, when the sender reported
    /// one, the commit they were observed on); the workspace is never probed.
    Injected {
        files: &'a [String],
        head_commit: Option<&'a str>,
    },
    /// Probe the local host workspace for git-dirty files.
    ProbeWorkspace(&'a Workspace),
}

impl TouchedCandidates<'_> {
    /// The candidate paths before attribution filtering. Injected paths are
    /// wire input from another machine: separators normalize to the logical
    /// forward-slash canon, duplicates collapse, and paths that cannot name a
    /// file inside the repo (absolute, drive-lettered, `..`-escaping) or that
    /// the probe path always excludes (tool scaffolding) are dropped. The one
    /// probe exclusion that cannot be applied here is the sender's spec-dir
    /// prefix — its name is the sender's workspace knowledge, and the sending
    /// CLI already filters it before the wire.
    fn resolve(&self) -> Vec<String> {
        match self {
            TouchedCandidates::Injected { files, .. } => {
                let mut out: Vec<String> = Vec::new();
                for f in files.iter() {
                    let f = f.replace('\\', "/");
                    if f.is_empty() || !is_repo_relative(&f) || is_scaffolding_path(&f) {
                        continue;
                    }
                    if !out.contains(&f) {
                        out.push(f);
                    }
                }
                out
            }
            TouchedCandidates::ProbeWorkspace(ws) => git_changed_files(ws),
        }
    }

    /// The commit the candidates were observed on: wire-supplied for injected
    /// candidates, read from the workspace HEAD for probed ones. What the
    /// sender did not report stays absent — never filled from an unrelated
    /// checkout. An injected value that is not a full commit sha (the same
    /// shape the probe records) is junk wire input and stays absent too,
    /// mirroring the path filtering above.
    fn resolved_head_commit(&self) -> Option<String> {
        match self {
            TouchedCandidates::Injected { head_commit, .. } => head_commit
                .filter(|h| {
                    (h.len() == 40 || h.len() == 64)
                        && h.bytes().all(|b| b.is_ascii_hexdigit())
                })
                .map(str::to_string),
            TouchedCandidates::ProbeWorkspace(ws) => head_commit(&ws.root),
        }
    }
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
    change: &str,
    addr: &TaskAddr,
    attr: &CompleteAttribution,
    candidates: TouchedCandidates,
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
        return Ok(CompleteOutcome {
            description: desc,
            already: true,
            ordinal,
            stable_id,
            touched_files: Vec::new(),
        });
    }
    store.write_artifact(change, "tasks.md", &new_content)?;

    // Record touched files: only those not already attributed to an earlier task;
    // when nothing new is dirty, no entry is appended at all (frozen behavior).
    // The v1 `touched` entry stays alongside the v2 evidence entry — the commit
    // skill's documented file-list channel keeps its exact shape.
    let mut record = TouchedRecord::load(store, change);
    record.change = change.to_string();
    let seen = record.all_files();
    let files: Vec<String> = candidates
        .resolve()
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
            head_commit: candidates.resolved_head_commit(),
            touched_files: files.clone(),
            recorded_at: chrono::Utc::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        });
        record.save(store)?;
    }

    crate::inprogress::add(store, change, attr.identity, attr.agent)?;
    Ok(CompleteOutcome {
        description: desc,
        already: false,
        ordinal,
        stable_id,
        touched_files: files,
    })
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
    /// Read the change's record through the storage seam. Where the record
    /// physically lives — the change directory's `.evidence.json` for the
    /// filesystem supplier, a change-scoped document for a TeamStore — is the
    /// supplier's business. Absent = empty record; a record that does not
    /// parse is treated the same way, so a corrupt file never blocks a
    /// completion.
    pub fn load(store: &dyn Store, change: &str) -> TouchedRecord {
        let empty = || TouchedRecord {
            change: change.to_string(),
            ..Default::default()
        };
        match store.read_evidence(change) {
            Some(s) => serde_json::from_str(&s).unwrap_or_else(|_| empty()),
            None => empty(),
        }
    }

    /// Write the record back through the storage seam. Writes are always v2,
    /// whatever version was read.
    pub fn save(&self, store: &dyn Store) -> anyhow::Result<()> {
        let mut rec = self.clone();
        rec.version = Some(2);
        // A serialization failure is loud: writing a defaulted empty string
        // would commit an empty record over real evidence (and sweep the
        // legacy file with it).
        let json = serde_json::to_string_pretty(&rec)?;
        store.write_evidence(&self.change, &json)
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

/// Full HEAD sha of the repo at `root`, or None when there is none to read.
/// Guarded on `.git` like [`git_changed_files`]: a project nested inside an
/// ancestor repo must not record the ancestor's HEAD. The probe path records
/// this locally; the remote CLI reads it here to report it over the wire.
pub fn head_commit(root: &Path) -> Option<String> {
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

/// True when a logical path can name a file inside the repo: relative, no
/// drive letter, and no `..` segment that could climb out.
fn is_repo_relative(path: &str) -> bool {
    if path.starts_with('/') || path.contains(':') {
        return false;
    }
    path.split('/').all(|seg| seg != "..")
}

/// The fixed set of work files and tool-scaffolding locations that never enter
/// a code trace, whatever the candidate source (.claude/.agents/.cursor/.gemini
/// and .gitignore are not code; .speclink/ and .git/ are machinery).
fn is_scaffolding_path(path: &str) -> bool {
    path.starts_with(".speclink/")
        || path.starts_with(".git/")
        || path.starts_with(".claude/")
        || path.starts_with(".agents/")
        || path.starts_with(".cursor/")
        || path.starts_with(".gemini/")
        || path == ".gitignore"
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
        // Exclude spec artifacts and the fixed scaffolding set from the code
        // trace (CLAUDE.md / config are recorded).
        if path.starts_with(&spec_prefix) || is_scaffolding_path(&path) {
            continue;
        }
        files.push(path);
    }
    files
}

#[cfg(test)]
mod tests;

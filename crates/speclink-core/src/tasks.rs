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
        let (display, stable_id) = split_stable_id(desc);
        out.push(Task {
            id,
            description: display.trim().to_string(),
            done,
            parallel,
            stable_id: stable_id.map(str::to_string),
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
/// already_in_target_state, stable_id) or None if not found. Indent, bullet style (Spectra
/// rewrites `* [ ]` to `* [x]`), and trailing newline are preserved.
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
                let clean = rest.strip_prefix("[P] ").unwrap_or(rest);
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
    // when nothing new is dirty, no entry is appended at all (matches Spectra).
    // The v1 `touched` entry stays alongside the v2 evidence entry — the commit
    // skill's documented file-list channel keeps its exact shape.
    let mut record = TouchedRecord::load(ws, change);
    record.change = change.to_string();
    let seen = record.all_files();
    let files: Vec<String> = git_changed_files(&ws.root)
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
            // tasks.md was written above, so the current basis IS this
            // completion's post-write basis.
            basis_digests: current_basis_digests(store, change),
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

/// Per-task completion evidence (v2): who, where, against which basis
/// (spec verify-evidence). Unattributable fields are absent, not defaulted.
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
    pub basis_digests: BasisDigests,
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
    pub fn load(ws: &Workspace, change: &str) -> TouchedRecord {
        let p = ws.touched_dir().join(format!("{change}.json"));
        match util::read_opt(&p) {
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

    pub fn save(&self, ws: &Workspace) -> std::io::Result<()> {
        let p = ws.touched_dir().join(format!("{}.json", self.change));
        // Writes are always v2, whatever version was read.
        let mut rec = self.clone();
        rec.version = Some(2);
        let json = serde_json::to_string_pretty(&rec).unwrap_or_default();
        util::write_file(&p, &json)
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
/// policy) — the single computation shared by evidence recording (task done)
/// and the Host's VerifyBundle, so recorded and judged bases always agree.
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

        let out = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution { identity: Some("Tester <t@example.com>"), ..Default::default() }).unwrap();

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
        let out = complete(&store, &t.ws, "demo", &TaskAddr::Ordinal(1), &CompleteAttribution::default()).unwrap();
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
        assert_eq!(t.touched_json("demo"), None, "already-done must not record touched files");
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
    fn complete_writes_v2_evidence_entry_with_attribution_and_basis() {
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

        let json = t.touched_json("demo").expect("evidence record written");
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
        for k in ["spec", "tasks", "policy"] {
            let d = e["basisDigests"][k].as_str().unwrap_or_default();
            assert!(d.starts_with("sha256:"), "basis digest {k} must be sha256-form, got '{d}'");
        }
        let at = e["recordedAt"].as_str().expect("recordedAt present");
        assert!(at.ends_with('Z'), "recordedAt is UTC: {at}");
        chrono::DateTime::parse_from_rfc3339(at).expect("recordedAt parses as RFC3339");
    }

    #[test]
    fn v1_record_reads_with_unchanged_file_list_semantics() {
        let t = TempWs::new("v1compat");
        let p = t.ws.touched_dir().join("demo.json");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(
            &p,
            "{\n  \"change\": \"demo\",\n  \"touched\": [\n    {\n      \"task_id\": \"1\",\n      \"task_desc\": \"1.1 legacy\",\n      \"files\": [\"src/a.rs\", \"src/b.rs\"]\n    }\n  ]\n}",
        )
        .unwrap();
        let rec = TouchedRecord::load(&t.ws, "demo");
        assert_eq!(
            rec.all_files(),
            vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
            "v1 file-list semantics unchanged"
        );
    }

    #[test]
    fn uncomplete_never_writes_or_alters_evidence() {
        let t = TempWs::new("undone-evidence");
        let p = t.ws.touched_dir().join("demo.json");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        let seeded = "{\"change\":\"demo\",\"touched\":[{\"task_id\":\"1\",\"task_desc\":\"1.1 a\",\"files\":[\"src/a.rs\"]}]}";
        std::fs::write(&p, seeded).unwrap();
        let store = store_with(META_UNSTARTED, "- [x] 1.1 a\n");
        uncomplete(&store, "demo", &TaskAddr::Ordinal(1)).unwrap();
        assert_eq!(
            std::fs::read_to_string(&p).unwrap(),
            seeded,
            "undone must not write or alter any evidence record"
        );
    }
}

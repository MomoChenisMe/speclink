//! Task list parsing, completion, and touched-file tracking.

use crate::paths::Paths;
use crate::util;
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

/// Parse tasks.md into an ordered list of checkbox tasks.
pub fn parse(tasks_md: &str) -> Vec<Task> {
    let mut out = Vec::new();
    let mut id = 0usize;
    for line in tasks_md.lines() {
        let trimmed = line.trim_start();
        let (done, rest) = if let Some(r) = trimmed.strip_prefix("- [ ] ") {
            (false, r)
        } else if let Some(r) = trimmed.strip_prefix("- [x] ") {
            (true, r)
        } else if let Some(r) = trimmed.strip_prefix("- [X] ") {
            (true, r)
        } else {
            continue;
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
    let mut id = 0usize;
    let mut already = false;
    let mut desc = String::new();
    let mut found = false;
    let mut out_lines: Vec<String> = Vec::new();
    for line in tasks_md.lines() {
        let trimmed = line.trim_start();
        let is_open = trimmed.starts_with("- [ ] ");
        let is_done = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
        if is_open || is_done {
            id += 1;
            if id == target_id {
                found = true;
                let indent = &line[..line.len() - trimmed.len()];
                let rest = if is_open {
                    &trimmed[6..]
                } else {
                    already = true;
                    &trimmed[6..]
                };
                let clean = rest.strip_prefix("[P] ").unwrap_or(rest);
                desc = clean.trim().to_string();
                out_lines.push(format!("{indent}- [x] {rest}"));
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
    pub fn load(paths: &Paths, change: &str) -> TouchedRecord {
        let p = paths.touched_dir().join(format!("{change}.json"));
        match std::fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap_or(TouchedRecord {
                change: change.to_string(),
                touched: Vec::new(),
            }),
            Err(_) => TouchedRecord {
                change: change.to_string(),
                touched: Vec::new(),
            },
        }
    }

    pub fn save(&self, paths: &Paths) -> std::io::Result<()> {
        let p = paths.touched_dir().join(format!("{}.json", self.change));
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

//! Filesystem, git, and misc helpers.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Read a file to string, returning None if it does not exist / cannot be read.
pub fn read_opt(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Write a file, creating parent directories as needed. Content is written verbatim.
pub fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// Recursively collect all files under `dir` (relative paths are not resolved; returns absolute-ish
/// paths as joined). Symlinks are not followed. Order is sorted for determinism.
pub fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_inner(dir, &mut out);
    out.sort();
    out
}

fn walk_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_inner(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}

/// True if the path exists.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Remove a file (thin wrapper so flow modules stay free of direct std::fs calls).
pub fn remove_file(path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

/// True if a file exists and has non-whitespace content.
pub fn has_content(path: &Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(s) => !s.trim().is_empty(),
        Err(_) => false,
    }
}

/// Run `git -C <root> <args...>`, returning trimmed stdout on success.
pub fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Run `git -C <root> <args...>`, returning raw (untrimmed) stdout on success. Needed for
/// `status --porcelain`, whose first column may be a significant leading space.
pub fn git_raw(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// Whether the directory is inside a git work tree.
pub fn git_available(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"]).as_deref() == Some("true")
}

/// Build the "Name <email>" identity string from git config, if available.
pub fn git_identity(root: &Path) -> Option<String> {
    let name = git(root, &["config", "user.name"]);
    let email = git(root, &["config", "user.email"]);
    match (name, email) {
        (Some(n), Some(e)) if !n.is_empty() && !e.is_empty() => Some(format!("{n} <{e}>")),
        (Some(n), _) if !n.is_empty() => Some(n),
        _ => None,
    }
}

/// Today's date as YYYY-MM-DD (local time).
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Convert an absolute path to a forward-slash string.
pub fn to_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Simple time-seeded pseudo-random index in [0, n). Not cryptographic.
pub fn pseudo_random(n: usize) -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // xorshift-ish mix
    let mut x = (nanos as u64) ^ 0x9E3779B97F4A7C15;
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    if n == 0 {
        0
    } else {
        (x as usize) % n
    }
}

/// Slugify a topic string into kebab-case (lowercase, spaces/underscores → '-', strip others).
pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if ch == '-' || ch == '_' || ch.is_whitespace() {
            if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
        } else if !ch.is_ascii() {
            // keep unicode letters (e.g., CJK) as-is to preserve meaning
            out.push(ch);
            prev_dash = false;
        }
        // else: drop punctuation
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

//! Filesystem, git, and misc helpers.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Read a file to string, returning None if it does not exist / cannot be read.
pub fn read_opt(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// 看板排序鍵（board_rank）的合法值：非空、僅小寫 ASCII 英文字母。
/// 邊界驗證擋掉會破壞 meta/frontmatter 解析的值（換行注入、YAML 指示字元）。
pub(crate) fn is_valid_board_rank(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase())
}

/// Write a file, creating parent directories as needed. Content is written verbatim.
///
/// The write lands atomically: content goes to a temp file in the destination's own
/// directory, which is then renamed onto the destination. A concurrent reader therefore
/// observes either the old document or the new one — never an empty or half-written file.
/// If the rename fails (Windows can refuse it while another process holds the destination
/// open), fall back to writing the destination directly: no worse than before atomicity,
/// rather than turning a platform limit into a failed verb.
pub fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    // A bare `std::fs::rename` cannot be passed here: its inferred lifetimes are not
    // general enough for the higher-ranked bound the injection point needs.
    write_file_with_rename(path, content, |from, to| std::fs::rename(from, to))
}

/// `write_file` with the rename step injected, so the fallback path can be exercised
/// without a platform that actually refuses the rename.
fn write_file_with_rename(
    path: &Path,
    content: &str,
    rename: impl Fn(&Path, &Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let target = resolve_symlink_target(path);
    let target = target.as_path();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = temp_write_path(target);
    if let Err(e) = std::fs::write(&tmp, content) {
        let _ = std::fs::remove_file(&tmp);
        // 只有權限類失敗（父目錄不可寫但目的檔可寫）退回直寫——行為不劣於
        // 原子化前。其他失敗（磁碟滿、暫存名過長）直接浮出且目的檔不變：
        // 退回的直寫多半同樣會失敗，卻已先 truncate 把目的檔清空。
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            return std::fs::write(target, content);
        }
        return Err(e);
    }
    // 目的檔既有 mode 套回暫存檔（best-effort），rename 後才不會被重設成 umask
    // 預設（0600 的設定檔不該被放寬成 0644）。Windows 不做：唯讀屬性複製到暫存
    // 檔會反過來擋掉失敗路徑上的清理，且該平台無 umask 重設問題（ACL 隨目錄繼承）。
    #[cfg(unix)]
    if let Ok(meta) = std::fs::metadata(target) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    if rename(&tmp, target).is_err() {
        let direct = std::fs::write(target, content);
        let _ = std::fs::remove_file(&tmp);
        return direct;
    }
    Ok(())
}

/// Follow a symlinked destination to its final target. The pre-atomic direct write
/// wrote through the link; the atomic replacement must land on the same target, or
/// the link would silently become a regular file and the topology be lost. Bounded
/// walk; on a cycle the last path wins and gets replaced like any file.
fn resolve_symlink_target(path: &Path) -> PathBuf {
    let mut target = path.to_path_buf();
    for _ in 0..8 {
        let Ok(link) = std::fs::read_link(&target) else {
            break;
        };
        target = if link.is_absolute() {
            link
        } else {
            target.parent().map(|p| p.join(&link)).unwrap_or(link)
        };
    }
    target
}

/// Temp path beside `path`: same directory (so the rename stays within one filesystem
/// and thus atomic), unique per process and per call so concurrent writers never collide
/// on one temp name. Directory enumerators (e.g. `walk_files`) may glimpse the entry in
/// the write-to-rename window; only a crash inside that window leaves one behind, and
/// the `.tmp` tail marks such a leftover for what it is.
fn temp_write_path(path: &Path) -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.{n}.tmp", std::process::id()));
    path.with_file_name(name)
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

/// Build a `git -C <root>` command. On Windows, spawn with CREATE_NO_WINDOW: console
/// programs launched from a GUI process otherwise flash a console window per spawn.
/// All callers run short non-interactive commands (config/status) with piped stdio,
/// which need no console.
fn git_command(root: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(root);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Run `git -C <root> <args...>`, returning trimmed stdout on success.
pub fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = git_command(root).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Run `git -C <root> <args...>`, returning raw (untrimmed) stdout on success. Needed for
/// `status --porcelain`, whose first column may be a significant leading space.
pub fn git_raw(root: &Path, args: &[&str]) -> Option<String> {
    let output = git_command(root).args(args).output().ok()?;
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

/// Today's date as YYYY-MM-DD (local time).
pub fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Render a string as a YAML scalar safe to append to a metadata document.
///
/// A newline inside an identity/agent/path string would inject arbitrary YAML
/// fields; a bare `:`/leading indicator would break the whole document's parse
/// (which silently falls back to defaults). Flatten control characters and
/// double-quote values a plain scalar cannot carry.
pub fn yaml_scalar(s: &str) -> String {
    let flat = s.replace(|c: char| c.is_control(), " ");
    let risky = flat.is_empty()
        || flat.contains([':', '#', '"'])
        || flat.ends_with(' ')
        || flat.starts_with([' ', '[', '{', '\'', '&', '*', '!', '|', '>', '%', '@', '`', '-', '?']);
    if risky {
        format!("\"{}\"", flat.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        flat
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Throwaway directory in the OS temp dir; removed on drop.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            let path = std::env::temp_dir().join(format!(
                "speclink-core-util-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// File names present in a directory, sorted.
    fn dir_entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn write_file_writes_content_verbatim() {
        let d = TempDir::new("verbatim");
        let path = d.path.join("note.md");
        write_file(&path, "line one\nline two\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "line one\nline two\n");
    }

    #[test]
    fn write_file_creates_parent_dirs() {
        let d = TempDir::new("parents");
        let path = d.path.join("a").join("b").join("c.md");
        write_file(&path, "deep").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "deep");
    }

    #[test]
    fn write_file_overwrites_existing() {
        let d = TempDir::new("overwrite");
        let path = d.path.join("note.md");
        write_file(&path, "old content that is longer").unwrap();
        write_file(&path, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    /// Spec `本地檔案寫入原子落盤`: the success path leaves no temp file in the
    /// destination directory.
    #[test]
    fn write_file_leaves_no_temp_file_behind() {
        let d = TempDir::new("no-residue");
        let path = d.path.join("note.md");
        write_file(&path, "first").unwrap();
        assert_eq!(dir_entries(&d.path), vec!["note.md".to_string()]);
        write_file(&path, "second").unwrap();
        assert_eq!(dir_entries(&d.path), vec!["note.md".to_string()]);
    }

    /// Symlinked destination keeps its topology: the write lands on the link's
    /// target (as the pre-atomic direct write did), and the link itself survives
    /// instead of being replaced by a regular file.
    #[cfg(unix)]
    #[test]
    fn write_file_writes_through_a_symlinked_destination() {
        let d = TempDir::new("symlink");
        let target = d.path.join("target.md");
        let link = d.path.join("link.md");
        write_file(&target, "old").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        write_file(&link, "new").unwrap();
        assert!(
            std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink(),
            "link must stay a symlink"
        );
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
        assert_eq!(std::fs::read_to_string(&link).unwrap(), "new");
    }

    /// Overwriting keeps the destination's permission bits (best-effort): a 0600
    /// config must not silently relax to the umask default after the rename.
    #[cfg(unix)]
    #[test]
    fn write_file_preserves_destination_mode() {
        use std::os::unix::fs::PermissionsExt;
        let d = TempDir::new("mode");
        let path = d.path.join("secret.yaml");
        write_file(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        write_file(&path, "new").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "mode must survive the atomic replace");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    /// Spec `本地檔案寫入原子落盤`: when the temp file cannot even be created (an
    /// unwritable parent dir with a writable destination inside), fall back to the
    /// direct write — behavior no worse than before atomicity.
    #[cfg(unix)]
    #[test]
    fn write_file_falls_back_to_direct_write_when_temp_cannot_be_created() {
        use std::os::unix::fs::PermissionsExt;
        let d = TempDir::new("locked-dir");
        let path = d.path.join("note.md");
        write_file(&path, "old").unwrap();
        std::fs::set_permissions(&d.path, std::fs::Permissions::from_mode(0o555)).unwrap();
        let res = write_file(&path, "new");
        std::fs::set_permissions(&d.path, std::fs::Permissions::from_mode(0o755)).unwrap();
        res.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        assert_eq!(dir_entries(&d.path), vec!["note.md".to_string()]);
    }

    /// Spec `本地檔案寫入原子落盤`: only a permission-class temp failure may fall
    /// back to the direct write. Any other temp-write failure (disk full, a temp
    /// name past NAME_MAX) must surface the error with the destination untouched —
    /// falling back would truncate the destination before failing.
    #[cfg(unix)]
    #[test]
    fn write_file_surfaces_non_permission_temp_failure_without_touching_dest() {
        let d = TempDir::new("temp-name-too-long");
        // 250 bytes: legal for the file itself, but + ".<pid>.<n>.tmp" the temp
        // name passes NAME_MAX (255) and its creation fails with ENAMETOOLONG.
        let long_name = format!("{}.md", "n".repeat(247));
        let path = d.path.join(&long_name);
        // Setup bypasses write_file: its temp name would hit the same limit.
        std::fs::write(&path, "old").unwrap();
        assert!(
            write_file(&path, "new").is_err(),
            "non-permission temp failure must surface"
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "old",
            "destination must stay untouched"
        );
    }

    /// Spec `本地檔案寫入原子落盤`: when the rename cannot complete (Windows refuses
    /// it while another process holds the destination open), the content still lands
    /// and the temp file is cleaned up.
    #[test]
    fn write_file_falls_back_to_direct_write_when_rename_fails() {
        let d = TempDir::new("rename-fails");
        let path = d.path.join("note.md");
        write_file(&path, "old").unwrap();

        write_file_with_rename(&path, "new", |_, _| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "sharing violation",
            ))
        })
        .unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        assert_eq!(dir_entries(&d.path), vec!["note.md".to_string()]);
    }

    /// Spec `本地檔案寫入原子落盤` / scenario `並行讀者不見半份內容`: while one
    /// thread rewrites the file, a concurrent reader only ever observes one of
    /// the two complete documents — never an empty, truncated, or mixed read.
    /// Unix only: the atomic guarantee is best-effort on Windows.
    #[cfg(unix)]
    #[test]
    fn write_file_concurrent_reader_never_sees_partial_content() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let d = TempDir::new("concurrent");
        let path = d.path.join("shared.md");
        let doc_a = "a".repeat(64 * 1024);
        let doc_b = "b".repeat(64 * 1024);
        write_file(&path, &doc_a).unwrap();

        let done = Arc::new(AtomicBool::new(false));

        let writer = {
            let path = path.clone();
            let (doc_a, doc_b) = (doc_a.clone(), doc_b.clone());
            let done = Arc::clone(&done);
            std::thread::spawn(move || {
                for i in 0..200 {
                    let doc = if i % 2 == 0 { &doc_b } else { &doc_a };
                    write_file(&path, doc).unwrap();
                }
                done.store(true, Ordering::Relaxed);
            })
        };

        let reader = {
            let path = path.clone();
            let done = Arc::clone(&done);
            std::thread::spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    let got = std::fs::read_to_string(&path).unwrap_or_default();
                    if got != doc_a && got != doc_b {
                        return Some(got.len());
                    }
                }
                None
            })
        };

        writer.join().unwrap();
        let torn = reader.join().unwrap();
        assert!(
            torn.is_none(),
            "reader observed a partial document of {} bytes",
            torn.unwrap_or(0)
        );
    }
}

//! Integration tests for `runtime::bootstrap`.
//!
//! 對應 spec requirement（6 個 named test 對應 6.1–6.6）：
//! - `speclink init` initializes a SpecLink project in a git working tree
//! - `speclink init` MUST reject non-git working directories
//! - `speclink init` MUST refuse re-initialization without `--force`
//! - `speclink init --force` MUST re-init while preserving `state.db`
//! - `.gitignore` policy MUST be a single line for `.speclink/link.yaml`
//! - Init MUST commit artifact and state changes only after every prepare step succeeds

use std::fs;
use std::path::Path;
use std::process::Command;

use speclink_runtime::{Bootstrap, RealGitProbe, RuntimeError};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        cmd,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init(dir: &Path) {
    run(Command::new("git")
        .arg("init")
        .arg("--initial-branch=main")
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir));
    run(Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir));
}

fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).expect("read for sha256");
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

#[tokio::test]
async fn bootstrap_init_success() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    let boot = Bootstrap::new(RealGitProbe);
    let info = boot.init(&working, false).await.expect("init succeeds");
    assert!(!info.project_id.is_empty(), "project_id non-empty");

    assert!(working.join(".speclink").join("link.yaml").exists());
    assert!(working.join(".speclink").join("schemas").exists());
    assert!(
        working
            .join(".git")
            .join("speclink")
            .join("state.db")
            .exists()
    );
    assert!(working.join(".git").join("speclink").join("locks").exists());

    let gitignore = fs::read_to_string(working.join(".gitignore")).expect("read .gitignore");
    let count = gitignore
        .lines()
        .filter(|l| *l == ".speclink/link.yaml")
        .count();
    assert_eq!(
        count, 1,
        "exactly one .speclink/link.yaml line, got {gitignore:?}"
    );
}

#[tokio::test]
async fn bootstrap_init_rejects_non_git() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    // No `git init` here.

    let boot = Bootstrap::new(RealGitProbe);
    let err = boot
        .init(&working, false)
        .await
        .expect_err("expected error");
    assert!(matches!(err, RuntimeError::RequiresGit { .. }));

    assert!(
        !working.join(".speclink").exists(),
        ".speclink must not be created"
    );
}

#[tokio::test]
async fn bootstrap_init_conflict_without_force() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("first init");
    let mtime_before = fs::metadata(working.join(".speclink").join("link.yaml"))
        .expect("metadata")
        .modified()
        .expect("mtime");

    let err = boot
        .init(&working, false)
        .await
        .expect_err("second init must fail");
    assert!(matches!(err, RuntimeError::AlreadyInitialized { .. }));

    let mtime_after = fs::metadata(working.join(".speclink").join("link.yaml"))
        .expect("metadata")
        .modified()
        .expect("mtime");
    assert_eq!(mtime_before, mtime_after, "link.yaml mtime must not change");
}

#[tokio::test]
async fn bootstrap_init_force_preserves_state_db() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("first init");
    let state_db = working.join(".git").join("speclink").join("state.db");
    let sha_before = sha256_of(&state_db);
    let size_before = fs::metadata(&state_db).expect("metadata").len();
    let link_path = working.join(".speclink").join("link.yaml");
    let link_before = fs::read_to_string(&link_path).expect("read link");

    // Sleep small amount so created_at differs.
    std::thread::sleep(std::time::Duration::from_millis(1100));

    boot.init(&working, true).await.expect("force re-init");
    let sha_after = sha256_of(&state_db);
    let size_after = fs::metadata(&state_db).expect("metadata").len();
    assert_eq!(sha_before, sha_after, "state.db content must be preserved");
    assert_eq!(size_before, size_after, "state.db size must be preserved");

    let link_after = fs::read_to_string(&link_path).expect("read link");
    assert_ne!(
        link_before, link_after,
        "link.yaml must be rewritten on force"
    );
    // .gitignore stays idempotent
    let gitignore = fs::read_to_string(working.join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        gitignore.matches(".speclink/link.yaml").count(),
        1,
        "exactly one .speclink/link.yaml line after force re-init"
    );
}

#[tokio::test]
async fn gitignore_policy_missing_file_creates_with_line() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    let g = fs::read_to_string(working.join(".gitignore")).expect("read");
    assert_eq!(g, ".speclink/link.yaml\n");
}

#[tokio::test]
async fn gitignore_policy_appends_to_existing_file() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    fs::write(working.join(".gitignore"), "node_modules\n").expect("seed");
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("init");
    let g = fs::read_to_string(working.join(".gitignore")).expect("read");
    assert_eq!(g, "node_modules\n.speclink/link.yaml\n");
}

#[tokio::test]
async fn gitignore_policy_idempotent_on_force_reinit() {
    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);
    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&working, false).await.expect("first init");
    boot.init(&working, true).await.expect("force re-init");
    let g = fs::read_to_string(working.join(".gitignore")).expect("read");
    assert_eq!(g.matches(".speclink/link.yaml").count(), 1);
}

#[tokio::test]
#[cfg(unix)]
async fn bootstrap_init_partial_failure_cleanup() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().expect("tempdir");
    let working = canonical(tmp.path());
    git_init(&working);

    // 將 .git/ 設成 read-only，bootstrap 在準備 state.db 階段失敗。
    let dot_git = working.join(".git");
    let mut perm = fs::metadata(&dot_git).expect("metadata").permissions();
    perm.set_mode(0o555);
    fs::set_permissions(&dot_git, perm).expect("chmod .git");

    let boot = Bootstrap::new(RealGitProbe);
    let res = boot.init(&working, false).await;

    // restore perms so tempdir RAII can cleanup later
    let mut restore = fs::metadata(&dot_git).expect("metadata").permissions();
    restore.set_mode(0o755);
    fs::set_permissions(&dot_git, restore).expect("chmod restore");

    assert!(
        res.is_err(),
        "init should fail when state root is unwritable"
    );
    assert!(
        !working.join(".speclink").join("link.yaml").exists(),
        ".speclink/link.yaml must not exist after failed init"
    );
    // schemas dir may or may not exist depending on order; spec says NOT to remain partial:
    assert!(
        !working.join(".speclink").join("schemas").exists(),
        ".speclink/schemas must not exist after failed init"
    );
    assert!(
        !working
            .join(".git")
            .join("speclink")
            .join("state.db")
            .exists(),
        "state.db must not exist after failed init"
    );
}

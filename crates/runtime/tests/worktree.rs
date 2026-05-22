//! Integration tests for linked-worktree behavior and state_root display.

use std::fs;
use std::path::Path;
use std::process::Command;

use speclink_runtime::{Bootstrap, Operations, RealGitProbe};
use tempfile::TempDir;

fn run(cmd: &mut Command) {
    let out = cmd.output().expect("spawn");
    assert!(
        out.status.success(),
        "git command failed: {:?}\nstderr: {}",
        cmd,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_init_with_commit(dir: &Path) {
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
    run(Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(dir));
}

fn canonical(p: &Path) -> std::path::PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

fn project_id_from_link(link_path: &Path) -> String {
    let raw = fs::read_to_string(link_path).expect("read link.yaml");
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("project_id: ") {
            return rest.trim().to_string();
        }
    }
    panic!("project_id not found in {link_path:?}:\n{raw}");
}

fn instance_id_from_link(link_path: &Path) -> String {
    let raw = fs::read_to_string(link_path).expect("read link.yaml");
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("instance_id: ") {
            return rest.trim().to_string();
        }
    }
    panic!("instance_id not found in {link_path:?}:\n{raw}");
}

#[tokio::test]
async fn bootstrap_init_in_linked_worktree_shares_project_id() {
    let tmp_owned = TempDir::new().expect("tempdir");
    let base = canonical(tmp_owned.path());
    let main = base.join("main");
    fs::create_dir(&main).expect("main dir");
    git_init_with_commit(&main);

    let boot = Bootstrap::new(RealGitProbe);
    let main_info = boot.init(&main, false).await.expect("main init");
    let main_link = main.join(".speclink/link.yaml");
    let main_project_id = project_id_from_link(&main_link);
    let main_instance_id = instance_id_from_link(&main_link);
    assert_eq!(main_project_id, main_info.project_id);

    // git worktree add to a directory that does NOT yet exist (git requires absent path).
    let wt = base.join("wt");
    run(Command::new("git")
        .args(["worktree", "add", "-b", "feature"])
        .arg(&wt)
        .current_dir(&main));

    // Sleep so worktree's instance_id / created_at differ from main's.
    std::thread::sleep(std::time::Duration::from_millis(20));

    let wt_info = boot.init(&wt, false).await.expect("worktree init");
    assert_eq!(
        wt_info.project_id, main_project_id,
        "worktree init MUST share main's project_id, got {:?}",
        wt_info.project_id
    );

    let wt_link = wt.join(".speclink/link.yaml");
    assert!(wt_link.exists(), "wt link.yaml must exist");
    let wt_project_id = project_id_from_link(&wt_link);
    assert_eq!(wt_project_id, main_project_id);
    let wt_instance_id = instance_id_from_link(&wt_link);
    assert_ne!(
        wt_instance_id, main_instance_id,
        "wt instance_id MUST differ from main's"
    );

    // Verify main state.db still has exactly 1 project row.
    let state_db = main.join(".git/speclink/state.db");
    assert!(state_db.exists());
    let conn = rusqlite::Connection::open(&state_db).expect("open state.db");
    let count: u32 = conn
        .query_row("SELECT COUNT(*) FROM project", [], |r| r.get(0))
        .expect("query count");
    assert_eq!(
        count, 1,
        "main state.db must remain at exactly 1 project row after worktree init"
    );
    let stored_id: String = conn
        .query_row("SELECT id FROM project", [], |r| r.get(0))
        .expect("query id");
    assert_eq!(stored_id, main_project_id);
}

#[tokio::test]
async fn state_root_display_has_no_leading_double_slash() {
    let tmp_owned = TempDir::new().expect("tempdir");
    let base = canonical(tmp_owned.path());
    let main = base.join("main");
    fs::create_dir(&main).expect("main dir");
    git_init_with_commit(&main);

    let boot = Bootstrap::new(RealGitProbe);
    boot.init(&main, false).await.expect("main init");

    let wt = base.join("wt");
    run(Command::new("git")
        .args(["worktree", "add", "-b", "feature2"])
        .arg(&wt)
        .current_dir(&main));

    let wt_info = boot.init(&wt, false).await.expect("worktree init");
    assert!(
        !wt_info.state_root.starts_with("//"),
        "state_root MUST NOT start with //, got: {:?}",
        wt_info.state_root
    );
    assert!(
        !wt_info.state_root.is_empty(),
        "state_root MUST be non-empty"
    );

    // Status from worktree must also produce clean state_root.
    let ops = Operations::new(RealGitProbe);
    let status = ops.status(&wt).await.expect("status from wt");
    assert!(
        !status.state_root.starts_with("//"),
        "Operations::status state_root MUST NOT start with //, got: {:?}",
        status.state_root
    );
}

//! 無 host workspace 派發（Node host store 的 `workspace: None`）不得讓封存的
//! linked worktree 守門改判「行程 cwd」——review Round 1 的 must-fix 重現測試。
//!
//! 這個檔案刻意只有一個測試、自成一個測試 binary：它會改行程 cwd
//! （`set_current_dir`），與其他測試同進程並跑會互相污染，單測試 binary 才安全。
//! 情境：store 指向一個與 git 毫無關係的專案，行程 cwd 卻恰好在別個 repo 的
//! `speclink/` 分支 linked worktree 內——守門若讀 cwd 就會對不相干的 store 誤拒。

use std::path::{Path, PathBuf};
use std::process::Command;

use speclink_core::command::{execute, Command as Verb, ExecutionContext};
use speclink_fs::FsStore;

fn git(cwd: &Path, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(cwd).output().expect("run git");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn a_hostless_archive_ignores_the_processes_cwd() {
    let base = std::env::temp_dir().join(format!("speclink-fs-hostless-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);

    // 與 git 無關的 store 專案：一個零任務、有效 meta 的 change `demo`。
    let project = base.join("project");
    let change = project.join("openspec").join("changes").join("demo");
    std::fs::create_dir_all(&change).unwrap();
    std::fs::write(change.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-07-01\n")
        .unwrap();
    std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();

    // 別處的 git repo ＋ speclink/ 分支的 linked worktree，行程 cwd 移進去。
    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::write(repo.join("README.md"), "scratch\n").unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.name", "Sandbox Tester"]);
    git(&repo, &["config", "user.email", "sandbox@example.com"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "init"]);
    let worktree: PathBuf = base.join("repo.worktrees").join("x");
    git(&repo, &["worktree", "add", "-q", "-b", "speclink/x", worktree.to_str().unwrap()]);
    std::env::set_current_dir(&worktree).expect("enter the linked worktree");

    let store = FsStore::new(&project, "openspec");
    let result = execute(
        &store,
        &ExecutionContext::default(),
        Verb::Archive {
            change: Some("demo".to_string()),
            skip_specs: true,
            no_validate: true,
            mark_tasks_complete: false,
            carry_review: false,
            carry_verify: false,
        },
    );

    assert!(
        result.is_ok(),
        "a hostless dispatch has no local environment to judge — the cwd's worktree \
         must not refuse an unrelated store's archive: {:?}",
        result.err()
    );
    assert!(
        !change.exists(),
        "the store's active change moved into its archive"
    );

    std::env::set_current_dir(std::env::temp_dir()).unwrap();
    let _ = std::fs::remove_dir_all(&base);
}

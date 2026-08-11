//! `speclink archive` 單筆封存的兩道環境／狀態守門 — fs 模式契約
//! (change-lifecycle spec「單筆封存的任務完成度守門」與「封存的 linked
//! worktree 環境守門」)。
//!
//! 任務未完成(總數>0 且未全勾)的單筆封存拒絕:非零 exit code、stderr 載明
//! N/M 證據與兩條出路、零檔案效果。--mark-tasks-complete 維持既有語意:
//! 先全勾再封存。成功路徑(全完成/0 任務/批次)的既有測試不在此檔、不修改。
//!
//! linked worktree 守門走真實 git repo ＋ `git worktree add`:`.git` 是檔案
//! 這個判準無法以捏造檔案取代(分支事實仍得由 git 回報)。主 checkout 的行為
//! 不變由本檔上半部與 archive_merge_gate／archive_evidence_gate 的既有綠燈確認。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
/// 新開 capability 的 delta 自帶合格 Purpose——缺席會被封存的 Purpose 守門擋下
/// （spec archive-merge「新 capability 的 Purpose 自 delta 帶入」）。
const DELTA_SPEC: &str = "## Purpose\n\n本 capability 是測試用的示範能力，涵蓋一個可觀察行為與其成功路徑，供封存流程的守門測試取用。\n\n## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Throwaway project: one structurally valid change `demo` (proposal +
    /// delta spec + the given tasks.md), no git — @trace probes fail soft.
    fn new(tag: &str, tasks_md: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-archive-gate-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(change.join("specs").join("demo-cap")).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), tasks_md).unwrap();
        std::fs::write(change.join("specs").join("demo-cap").join("spec.md"), DELTA_SPEC).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn archive_dir(&self) -> PathBuf {
        self.dir.join("openspec").join("changes").join("archive")
    }

    /// The dated archived change directory (`<date>-demo`), if any.
    fn archived_demo(&self) -> Option<PathBuf> {
        let entries = std::fs::read_dir(self.archive_dir()).ok()?;
        entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with("-demo")))
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn incomplete_change_refuses_with_evidence_and_no_archive_dir() {
    // spec scenario「任務未完成的單筆封存被拒」:3 任務 1 勾 → 非零 exit,
    // stderr 載明 1/3 與兩條出路,changes/archive/ 無新目錄、change 原地不動。
    let p = TempProject::new("refuse", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let out = p.run(&["archive", "demo"]);
    assert!(!out.status.success(), "incomplete change must refuse archive");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1/3"), "evidence N/M on stderr: {stderr}");
    assert!(stderr.contains("--mark-tasks-complete"), "exit route named: {stderr}");
    assert!(p.archived_demo().is_none(), "no archived directory appears");
    assert!(
        p.dir.join("openspec").join("changes").join("demo").join("tasks.md").is_file(),
        "change stays in place"
    );
    let tasks =
        std::fs::read_to_string(p.dir.join("openspec").join("changes").join("demo").join("tasks.md"))
            .unwrap();
    assert_eq!(tasks, "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n", "tasks.md byte-identical");
}

#[test]
fn mark_tasks_complete_archives_and_checks_every_task() {
    // spec scenario「--mark-tasks-complete 放行並先全勾」:exit 0,封存後的
    // tasks.md 全部任務為已勾,active change 目錄消失。
    let p = TempProject::new("mark", "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
    let out = p.run(&["archive", "demo", "--mark-tasks-complete"]);
    assert!(
        out.status.success(),
        "mark-tasks-complete must archive: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !p.dir.join("openspec").join("changes").join("demo").exists(),
        "active change moved into the archive"
    );
    let archived = p.archived_demo().expect("dated archive directory exists");
    let tasks = std::fs::read_to_string(archived.join("tasks.md")).unwrap();
    assert!(!tasks.contains("- [ ]"), "every task checked after archive: {tasks}");
    assert_eq!(tasks.matches("- [x]").count(), 3, "all three tasks present and checked: {tasks}");
}

/// 真實 git repo ＋ sibling linked worktree 的沙盒。change `demo` 的 tasks.md
/// 由呼叫端給定:全勾時任何拒絕都只可能來自環境守門,不會與完成度守門混淆。
struct GitProject {
    root: PathBuf,
    repo: PathBuf,
}

impl GitProject {
    fn new(tag: &str, tasks_md: &str) -> GitProject {
        let root = std::env::temp_dir()
            .join(format!("speclink-cli-archive-worktree-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let repo = root.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(repo.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();

        let p = GitProject { root, repo };
        p.add_change("demo", tasks_md);
        p.git(&["init", "-q", "-b", "main"]);
        p.git(&["config", "user.name", "Sandbox Tester"]);
        p.git(&["config", "user.email", "sandbox@example.com"]);
        // Windows 的 git 預設 core.autocrlf=true,`git worktree add` 會在 checkout
        // 時把 LF 換成 CRLF——沙盒裡的 tasks.md 於是與寫進去的位元不同,逐位元
        // 比對(前置寫入未發生)便會誤紅。沙盒的行尾由測試自己決定,不隨機器的
        // git 設定漂移。
        p.git(&["config", "core.autocrlf", "false"]);
        p.git(&["add", "-A"]);
        p.git(&["commit", "-q", "-m", "init"]);
        p
    }

    /// 再寫一個結構完整的 change(未提交)。worktree 內要看得到的話,呼叫端得
    /// 自行 git add ＋ commit —— `git worktree add` 只帶得走已提交的內容。每個
    /// change 各配一個 capability,同一沙盒裡連續封存兩筆才不會撞正典。
    fn add_change(&self, name: &str, tasks_md: &str) {
        let change = self.repo.join("openspec").join("changes").join(name);
        std::fs::create_dir_all(change.join("specs").join(format!("{name}-cap"))).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), tasks_md).unwrap();
        std::fs::write(
            change.join("specs").join(format!("{name}-cap")).join("spec.md"),
            DELTA_SPEC,
        )
        .unwrap();
    }

    /// 封存後的日期目錄(`<date>-<name>`)是否已出現在 repo 本體(比照
    /// `TempProject::archived_demo`,多帶 change 名以支援同沙盒兩筆)。
    fn archived(&self, name: &str) -> bool {
        let Ok(entries) = std::fs::read_dir(self.repo.join("openspec").join("changes").join("archive"))
        else {
            return false;
        };
        entries
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().ends_with(&format!("-{name}")))
    }

    fn git(&self, args: &[&str]) {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.repo)
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// 在 sibling 巢建立指定分支的 linked worktree(其 `.git` 為檔案)。
    fn add_worktree(&self, dir: &str, branch: &str) -> PathBuf {
        let path = self.root.join("repo.worktrees").join(dir);
        self.git(&["worktree", "add", "-q", "-b", branch, path.to_str().expect("utf-8 path")]);
        path
    }

    fn run_in(&self, cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(cwd);
        for key in ["SPECLINK_STORE_URL", "SPECLINK_TOKEN", "NO_COLOR"] {
            cmd.env_remove(key);
        }
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for GitProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn archive_inside_a_speclink_worktree_refuses_with_zero_file_effects() {
    // spec scenario「worktree 內封存被拒且零檔案效果」:非零 exit,stderr 同時
    // 帶 worktree 事實與 worktree-merge 指路,change 原地不動、正典無寫入、
    // 解封存備份目錄不生成。
    let p = GitProject::new("refuse", "- [x] 1.1 a\n");
    let wt = p.add_worktree("demo", "speclink/demo");

    let out = p.run_in(&wt, &["archive", "demo"], &[("NO_COLOR", "1")]);
    assert!(!out.status.success(), "archiving inside a linked worktree must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("worktree"), "worktree fact on stderr: {stderr}");
    assert!(stderr.contains("worktree-merge"), "worktree-merge route named: {stderr}");
    assert!(
        wt.join("openspec").join("changes").join("demo").join("tasks.md").is_file(),
        "change stays in place"
    );
    assert!(
        !wt.join("openspec").join("changes").join("archive").exists(),
        "no archive directory appears"
    );
    assert!(
        !wt.join("openspec").join("specs").exists(),
        "no canonical spec is written"
    );
    assert!(!wt.join(".speclink").join("snapshots").exists(), "no unarchive backup is written");
}

#[test]
fn archive_inside_a_non_speclink_branch_worktree_behaves_like_a_main_checkout() {
    // spec scenario「非 speclink 分支的 worktree 放行」:`.git` 同樣是檔案,
    // 但分支不合慣例 → 封存照常成功。
    let p = GitProject::new("passthrough", "- [x] 1.1 a\n");
    let wt = p.add_worktree("feature", "feature/anything");

    let out = p.run_in(&wt, &["archive", "demo"], &[("NO_COLOR", "1")]);
    assert!(
        out.status.success(),
        "a non-speclink branch must archive: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !wt.join("openspec").join("changes").join("demo").exists(),
        "active change moved into the archive"
    );
}

#[test]
fn archive_inside_a_worktree_without_git_fails_open() {
    // spec scenario「git 不可用時 fail-open」:PATH 清空後分支事實取不到,
    // 守門放行——無 git 的環境不得因此永遠無法封存。
    let p = GitProject::new("no-git", "- [x] 1.1 a\n");
    let wt = p.add_worktree("demo", "speclink/demo");

    let out = p.run_in(&wt, &["archive", "demo"], &[("NO_COLOR", "1"), ("PATH", "")]);
    assert!(
        out.status.success(),
        "git being unavailable must not block archive: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !wt.join("openspec").join("changes").join("demo").exists(),
        "active change moved into the archive"
    );
}

#[test]
fn archive_from_a_main_checkout_on_a_speclink_branch_passes_through() {
    // spec scenario「主 checkout 零額外開銷」的反向案例:`.git` 是目錄,分支名
    // 卻恰為 speclink/demo。兩次執行分工明確——
    //  (a) git 正常可用:守門的 fs 短路若被拿掉,分支事實取得到就會誤拒,這是
    //      本測試的紅燈來源;
    //  (b) PATH 清空:主 checkout 路徑完全不依賴 git。
    // 兩者不能併成一次:守門 fail-open,PATH 清空時分支必然取不到,單靠清空
    // PATH 觀察不到 fs 短路還在不在。
    let p = GitProject::new("main-checkout", "- [x] 1.1 a\n");
    p.add_change("demo2", "- [x] 1.1 a\n");
    p.git(&["checkout", "-q", "-b", "speclink/demo"]);

    let out = p.run_in(&p.repo, &["archive", "demo"], &[("NO_COLOR", "1")]);
    assert!(
        out.status.success(),
        "a main checkout must archive even on a speclink/ branch: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !p.repo.join("openspec").join("changes").join("demo").exists(),
        "active change moved into the archive"
    );
    assert!(p.archived("demo"), "dated archive directory for demo exists");

    let out = p.run_in(&p.repo, &["archive", "demo2"], &[("NO_COLOR", "1"), ("PATH", "")]);
    assert!(
        out.status.success(),
        "a main checkout must not depend on git: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.archived("demo2"), "dated archive directory for demo2 exists");
}

#[test]
fn refused_archive_leaves_the_mark_tasks_complete_pre_write_undone() {
    // spec scenario「拒絕時 --mark-tasks-complete 前置寫入零效果」:守門排在
    // 前置全勾寫入之前,被拒的封存不得留下一份已被全勾的 tasks.md。守門若被
    // 挪到前置寫入之後,exit code 依舊非零,只有這份逐位元比對會轉紅。
    const TASKS: &str = "- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n";
    let p = GitProject::new("pre-write", TASKS);
    let wt = p.add_worktree("demo", "speclink/demo");

    let out = p.run_in(&wt, &["archive", "demo", "--mark-tasks-complete"], &[("NO_COLOR", "1")]);
    assert!(!out.status.success(), "archiving inside a linked worktree must refuse");
    // 釘住拒絕來源:TASKS 本來就 1/3,若旗標失效改由完成度守門拒絕,exit 同樣
    // 非零、tasks.md 同樣不動——只有這句能分辨拒絕真的來自 worktree 守門。
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("worktree-merge"), "the worktree gate did the refusing: {stderr}");
    let tasks =
        std::fs::read_to_string(wt.join("openspec").join("changes").join("demo").join("tasks.md"))
            .unwrap();
    assert_eq!(tasks, TASKS, "tasks.md byte-identical — the pre-write never happened");
}

#[test]
fn bulk_archive_inside_a_speclink_worktree_refuses_too() {
    // spec scenario「bulk 封存同受守門」:多個 change 名即 bulk 路徑。輸出去向
    // 是契約——中止報告(含守門全文)走 stdout,bulk 失敗摘要走 stderr,分流斷言。
    let p = GitProject::new("bulk", "- [x] 1.1 a\n");
    p.add_change("demo2", "- [x] 1.1 a\n");
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "add demo2"]);
    let wt = p.add_worktree("demo", "speclink/demo");

    let out = p.run_in(&wt, &["archive", "demo", "demo2"], &[("NO_COLOR", "1")]);
    assert!(!out.status.success(), "bulk archive inside a linked worktree must refuse");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // 中止報告只有 bulk 路徑會印:少了這句,多 change 名若被改回單筆路徑,
    // 後面的斷言仍會綠,bulk 就悄悄失去覆蓋。
    assert!(stdout.contains("Bulk archive aborted at 'demo'"), "the bulk path ran: {stdout}");
    // "linked worktree" 而非裸 "worktree":fixture 路徑 repo.worktrees/ 本身
    // 就含後者,裸字比對零資訊。
    assert!(stdout.contains("linked worktree"), "worktree fact on stdout: {stdout}");
    assert!(stdout.contains("worktree-merge"), "worktree-merge route on stdout: {stdout}");
    assert!(stderr.contains("bulk archive failed at 'demo'"), "bulk summary on stderr: {stderr}");
    for name in ["demo", "demo2"] {
        assert!(
            wt.join("openspec").join("changes").join(name).join("tasks.md").is_file(),
            "{name} stays in place"
        );
    }
    assert!(
        !wt.join("openspec").join("changes").join("archive").exists(),
        "no archive directory appears"
    );
    assert!(!wt.join("openspec").join("specs").exists(), "no canonical spec is written");
    assert!(!wt.join(".speclink").join("snapshots").exists(), "no unarchive backup is written");
}

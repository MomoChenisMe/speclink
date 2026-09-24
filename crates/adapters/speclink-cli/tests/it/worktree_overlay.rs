//! `speclink list` 的 worktree 觀察面（worktree-overlay spec）——真實 git repo
//! ＋ `git worktree add` 的端到端情境。
//!
//! 兩條輸出路徑都是契約：`--json` 的可空 worktree 欄位與人眼行尾的固定字面
//! 「 [worktree]」。政策關閉、非主 checkout、分支不合慣例三種情形下輸出必須與
//! 本能力導入前逐位元一致——回歸對照就是同一顆二進位在政策關閉時的輸出。
//!
//! 路徑斷言一律先正規化（macOS 的 /var 是 /private/var 的 symlink，Windows 的
//! 分隔符與磁碟機代號亦不可假設），因此比對 `Path` 而非字串。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const TASKS: &str = "## 1. Group\n\n- [ ] 1.1 first\n- [ ] 1.2 second\n";
const PROPOSAL: &str = "## Why\n\nDemo change.\n";

/// 主 checkout ＋（可選）一個 linked worktree 的沙盒。
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    /// 建一個含單一 change `add-dark-mode` 的 git repo。`worktree_policy` 決定
    /// `openspec/config.yaml` 是否寫入 `worktree: true`。
    fn new(tag: &str, worktree_policy: bool) -> Fixture {
        let root = std::env::temp_dir().join(format!(
            "speclink-cli-worktree-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let repo = root.join("repo");
        let change = repo.join("openspec").join("changes").join("add-dark-mode");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(repo.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let config = if worktree_policy {
            "schema: spec-driven\nworktree: true\n"
        } else {
            "schema: spec-driven\n"
        };
        std::fs::write(repo.join("openspec").join("config.yaml"), config).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("tasks.md"), TASKS).unwrap();
        std::fs::write(change.join("proposal.md"), PROPOSAL).unwrap();

        let f = Fixture { root, repo };
        f.git(&["init", "-q", "-b", "main"]);
        f.git(&["config", "user.name", "Sandbox Tester"]);
        f.git(&["config", "user.email", "sandbox@example.com"]);
        // Windows 的 git 預設 core.autocrlf=true，`git worktree add` 會在 checkout
        // 時把 LF 換成 CRLF——worktree 副本的 .openspec.yaml 於是與 META 的位元
        // 不同，引擎沿用各行行尾寫回後，拿 LF 字面逐位元比對便會誤紅。沙盒的
        // 行尾由測試自己決定，不隨機器的 git 設定漂移。
        f.git(&["config", "core.autocrlf", "false"]);
        f.git(&["add", "-A"]);
        f.git(&["commit", "-q", "-m", "init"]);
        f
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

    /// 以慣例分支 `speclink/<change>` 在 sibling 巢建立 worktree，回傳其路徑。
    fn add_worktree(&self, change: &str) -> PathBuf {
        let path = self.root.join("repo.worktrees").join(change);
        let branch = format!("speclink/{change}");
        self.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            &branch,
            path.to_str().expect("utf-8 path"),
        ]);
        path
    }

    /// 非慣例分支的 worktree。
    fn add_worktree_on_branch(&self, dir: &str, branch: &str) -> PathBuf {
        let path = self.root.join("repo.worktrees").join(dir);
        self.git(&["worktree", "add", "-q", "-b", branch, path.to_str().unwrap()]);
        path
    }

    fn remove_worktree(&self, path: &Path) {
        self.git(&["worktree", "remove", "--force", path.to_str().unwrap()]);
    }

    /// 於指定目錄執行 speclink（環境變數一律清乾淨，避免宿主設定滲入）。
    fn run_in(&self, cwd: &Path, args: &[&str]) -> Output {
        self.run_in_env(cwd, args, &[("NO_COLOR", "1")])
    }

    /// 同上，但由呼叫端決定色彩相關環境變數——測試 stdout 恆為 pipe，`is_terminal()`
    /// 本就關色，所以「有色路徑」必須以 CLICOLOR_FORCE 明確逼出來，否則兩趟其實
    /// 跑的是同一條無色路徑。
    fn run_in_env(&self, cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(cwd);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_WORKTREE",
            "SPECLINK_STORE_URL",
            "SPECLINK_TOKEN",
            "NO_COLOR",
            "CLICOLOR",
            "CLICOLOR_FORCE",
        ] {
            cmd.env_remove(key);
        }
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_in(&self.repo.clone(), args)
    }

    /// 逼出有色路徑（CLICOLOR_FORCE 覆寫 NO_COLOR；`--no-color` 旗標仍勝過它）。
    fn run_colored(&self, args: &[&str]) -> Output {
        self.run_in_env(&self.repo.clone(), args, &[("CLICOLOR_FORCE", "1")])
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("stdout is JSON ({e}): {}", stdout_of(out));
    })
}

/// `list --json` 的第一個 change 條目。
fn first_change(out: &Output) -> serde_json::Value {
    json_of(out)["changes"][0].clone()
}

/// 勾掉一個 worktree 副本內的任務（不經 CLI，直接改檔——測的是讀取面）。
fn check_one_task(worktree: &Path) {
    let tasks = worktree
        .join("openspec")
        .join("changes")
        .join("add-dark-mode")
        .join("tasks.md");
    let text = std::fs::read_to_string(&tasks).unwrap();
    std::fs::write(&tasks, text.replacen("- [ ] 1.1", "- [x] 1.1", 1)).unwrap();
}

/// 路徑比對用的正規化：symlink 展開後比 `Path`，不比字串（跨平台）。
fn same_path(a: &str, b: &Path) -> bool {
    let norm = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    norm(Path::new(a)) == norm(b)
}

#[test]
fn mapped_change_reports_the_worktree_field_and_the_worktree_task_counts() {
    // Spec scenario 映射成立 ＋ 任務計數即時反映 worktree 副本.
    let f = Fixture::new("mapped", true);
    let wt = f.add_worktree("add-dark-mode");
    check_one_task(&wt);

    let out = f.run(&["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let item = first_change(&out);
    assert_eq!(item["name"], "add-dark-mode");
    assert_eq!(item["completedTasks"], 1, "主副本仍是 0/2，值來自 worktree: {item}");
    assert_eq!(item["totalTasks"], 2);
    let worktree = &item["worktree"];
    assert_eq!(worktree["branch"], "speclink/add-dark-mode", "got: {item}");
    assert!(
        same_path(worktree["path"].as_str().expect("path string"), &wt),
        "worktree path points at the linked worktree: {item}"
    );
}

#[test]
fn mapped_change_line_carries_the_fixed_worktree_marker() {
    // Spec scenario 人眼輸出的標示（--no-color 同字面）——兩趟必須真的走不同
    // 色彩路徑，否則「兩者字面一致」等於沒被驗證。
    let f = Fixture::new("marker", true);
    f.add_worktree("add-dark-mode");

    let line_of = |out: &Output| {
        assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
        let text = stdout_of(out);
        text.lines()
            .find(|l| l.contains("add-dark-mode"))
            .map(str::to_string)
            .unwrap_or_else(|| panic!("no change line in: {text:?}"))
    };

    let colored = line_of(&f.run_colored(&["list"]));
    let plain = line_of(&f.run_colored(&["list", "--no-color"]));
    assert!(colored.contains('\u{1b}'), "the colored run must actually emit ANSI: {colored:?}");
    assert!(!plain.contains('\u{1b}'), "--no-color must win over CLICOLOR_FORCE: {plain:?}");
    assert!(colored.ends_with(" [worktree]"), "got: {colored:?}");
    assert!(plain.ends_with(" [worktree]"), "got: {plain:?}");
}

#[test]
fn removing_the_worktree_restores_the_main_copy_view() {
    // Spec scenario worktree 移除後還原.
    let f = Fixture::new("removed", true);
    let wt = f.add_worktree("add-dark-mode");
    check_one_task(&wt);
    assert_eq!(first_change(&f.run(&["list", "--json"]))["completedTasks"], 1);

    f.remove_worktree(&wt);
    let item = first_change(&f.run(&["list", "--json"]));
    assert!(item.get("worktree").is_none(), "欄位消失: {item}");
    assert_eq!(item["completedTasks"], 0, "數值回讀主副本: {item}");
    let text = stdout_of(&f.run(&["list"]));
    assert!(!text.contains("[worktree]"), "標示不殘留: {text:?}");
}

#[test]
fn policy_off_never_overlays_and_leaves_both_outputs_untouched() {
    // Spec scenario 政策關閉時零介入：同一顆二進位、同一份內容，僅政策開關不同。
    let off = Fixture::new("policy-off", false);
    let wt = off.add_worktree("add-dark-mode");
    check_one_task(&wt);

    let bare = Fixture::new("policy-off-bare", false);

    assert_eq!(
        stdout_of(&off.run(&["list", "--json"])),
        stdout_of(&bare.run(&["list", "--json"])),
        "政策關閉時 --json 與沒有 worktree 時逐位元一致"
    );
    assert_eq!(
        stdout_of(&off.run(&["list"])),
        stdout_of(&bare.run(&["list"])),
        "人眼輸出同樣逐位元一致"
    );
    // 有色路徑另算一條輸出契約，同樣不得被 overlay 波及。
    assert_eq!(
        stdout_of(&off.run_colored(&["list"])),
        stdout_of(&bare.run_colored(&["list"])),
        "有色人眼輸出逐位元一致"
    );
}

#[test]
fn running_inside_a_linked_worktree_does_not_overlay() {
    // Spec scenario linked worktree 內執行不套用映射.
    let f = Fixture::new("inside", true);
    let wt = f.add_worktree("add-dark-mode");
    check_one_task(&wt);

    let out = f.run_in(&wt, &["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let item = first_change(&out);
    assert!(item.get("worktree").is_none(), "worktree 內不自我 overlay: {item}");
}

#[test]
fn a_branch_outside_the_convention_is_skipped() {
    // Spec scenario 分支不合慣例即略過.
    let f = Fixture::new("branch", true);
    let wt = f.add_worktree_on_branch("add-dark-mode", "feature/add-dark-mode");
    check_one_task(&wt);

    let item = first_change(&f.run(&["list", "--json"]));
    assert!(item.get("worktree").is_none(), "got: {item}");
    assert_eq!(item["completedTasks"], 0, "值仍來自主副本: {item}");
}

#[test]
fn a_worktree_whose_change_is_unknown_is_skipped() {
    // Spec scenario 同名 change 不存在即略過（stderr 無警告）.
    let f = Fixture::new("ghost", true);
    f.add_worktree("ghost-change");

    let out = f.run(&["list", "--json"]);
    assert!(out.status.success());
    let item = first_change(&out);
    assert!(item.get("worktree").is_none(), "got: {item}");
    assert!(
        String::from_utf8_lossy(&out.stderr).trim().is_empty(),
        "stderr 無警告: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_worktree_without_the_change_directory_reads_back_the_main_copy() {
    // Spec scenario worktree 內 spec 目錄不可讀即回讀主副本——THEN 同時要求任務
    // 計數與狀態來自主副本，那是 CLI 層才觀察得到的事實。
    let f = Fixture::new("no-change-dir", true);
    let wt = f.add_worktree("add-dark-mode");
    check_one_task(&wt);
    std::fs::remove_dir_all(wt.join("openspec").join("changes").join("add-dark-mode")).unwrap();

    let out = f.run(&["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let item = first_change(&out);
    assert!(item.get("worktree").is_none(), "映射不成立: {item}");
    assert_eq!(item["completedTasks"], 0, "計數回讀主副本: {item}");
    assert_eq!(item["totalTasks"], 2, "計數回讀主副本: {item}");
    assert!(!stdout_of(&f.run(&["list"])).contains("[worktree]"), "人眼無標示");
}

#[test]
fn git_failing_outright_leaves_list_working() {
    // Spec scenario git 失敗時 fail-open：PATH 清空後 git 不可執行，discovery
    // 回空表，list 照常以 exit 0 輸出。
    let f = Fixture::new("no-git", true);
    f.add_worktree("add-dark-mode");

    let out = f.run_in_env(&f.repo.clone(), &["list", "--json"], &[("NO_COLOR", "1"), ("PATH", "")]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let item = first_change(&out);
    assert!(item.get("worktree").is_none(), "git 不可用時無欄位: {item}");
}

#[test]
fn an_unparseable_policy_document_disables_the_overlay_without_failing_list() {
    // Spec scenario 壞政策文件不使 list 失敗：觀察面的政策讀取 fail-open，不得把
    // 原本會成功的 list 變成失敗。
    let f = Fixture::new("bad-policy", true);
    f.add_worktree("add-dark-mode");
    std::fs::write(f.repo.join("openspec").join("config.yaml"), ": not yaml : [\n").unwrap();

    let out = f.run(&["list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let item = first_change(&out);
    assert!(item.get("worktree").is_none(), "壞政策視同關閉: {item}");
}

#[test]
fn a_corrupt_worktree_meta_surfaces_the_existing_diagnostic() {
    // Spec scenario worktree 副本中介資料損壞如實診斷.
    let f = Fixture::new("corrupt", true);
    let wt = f.add_worktree("add-dark-mode");
    std::fs::write(
        wt.join("openspec")
            .join("changes")
            .join("add-dark-mode")
            .join(".openspec.yaml"),
        ": not yaml : [\n",
    )
    .unwrap();

    let item = first_change(&f.run(&["list", "--json"]));
    assert!(item.get("metaError").is_some(), "got: {item}");
    assert!(item.get("worktree").is_some(), "映射仍成立: {item}");
}

#[test]
fn change_depends_writes_the_worktree_copy_of_a_mapped_change() {
    // change-plan spec「change depends 動詞寫入宣告依賴」：主 checkout 且政策開啟時，
    // 目標 change 已映射到 worktree 就寫那份副本——plan 讀的也是它，宣告立刻可見；
    // 主副本的 meta 不動。前置 add-auth 在分支前就存在，兩份 roster 都有它。
    let f = Fixture::new("depends-worktree", true);
    let auth = f.repo.join("openspec").join("changes").join("add-auth");
    std::fs::create_dir_all(&auth).unwrap();
    std::fs::write(auth.join(".openspec.yaml"), META).unwrap();
    f.git(&["add", "-A"]);
    f.git(&["commit", "-q", "-m", "add-auth"]);
    let wt = f.add_worktree("add-dark-mode");

    let out = f.run(&["change", "depends", "add-dark-mode", "--on", "add-auth"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let meta_of = |root: &Path| {
        std::fs::read_to_string(
            root.join("openspec")
                .join("changes")
                .join("add-dark-mode")
                .join(".openspec.yaml"),
        )
        .unwrap()
    };
    assert_eq!(meta_of(&wt), format!("{META}depends_on: add-auth\n"));
    assert_eq!(meta_of(&f.repo), META, "the main copy stays untouched");
    let plan = json_of(&f.run(&["plan", "--json"]));
    let dark = plan["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "add-dark-mode")
        .cloned()
        .unwrap();
    assert_eq!(dark["dependsOn"], serde_json::json!(["add-auth"]), "{plan}");
    assert_eq!(dark["blockedBy"], serde_json::json!(["add-auth"]), "{plan}");

    // 對照：政策關閉時沒有映射，寫的是主副本（既有行為）。
    let off = Fixture::new("depends-worktree-off", false);
    let wt = off.add_worktree("add-dark-mode");
    let out = off.run(&["change", "depends", "add-dark-mode", "--on", "add-dark-mode", "--remove"]);
    assert!(!out.status.success(), "self reference still refuses");
    let auth = off.repo.join("openspec").join("changes").join("add-auth");
    std::fs::create_dir_all(&auth).unwrap();
    std::fs::write(auth.join(".openspec.yaml"), META).unwrap();
    let out = off.run(&["change", "depends", "add-dark-mode", "--on", "add-auth"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(meta_of(&off.repo), format!("{META}depends_on: add-auth\n"));
    assert_eq!(meta_of(&wt), META, "policy off never touches the worktree copy");
}

#[test]
fn change_rank_writes_the_worktree_copy_of_a_mapped_change() {
    // change-plan spec「change rank 動詞寫入看板順序鍵」：與 change depends 同一
    // 聚合面——主 checkout 且政策開啟時，已映射的 change 寫它的 worktree 副本，
    // plan 讀的也是那份，順序立刻可見；主副本不動。錨點 add-auth 已有 rank，
    // 這一欄不需補章，只寫被移動的那一檔。
    let f = Fixture::new("rank-worktree", true);
    let auth = f.repo.join("openspec").join("changes").join("add-auth");
    std::fs::create_dir_all(&auth).unwrap();
    std::fs::write(auth.join(".openspec.yaml"), format!("{META}board_rank: n\n")).unwrap();
    f.git(&["add", "-A"]);
    f.git(&["commit", "-q", "-m", "add-auth"]);
    let wt = f.add_worktree("add-dark-mode");

    let out = f.run(&["change", "rank", "add-dark-mode", "--before", "add-auth"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let meta_of = |root: &Path, change: &str| {
        std::fs::read_to_string(root.join("openspec").join("changes").join(change).join(".openspec.yaml"))
            .unwrap()
    };
    assert!(meta_of(&wt, "add-dark-mode").contains("board_rank: "), "the worktree copy is ranked");
    assert_eq!(meta_of(&f.repo, "add-dark-mode"), META, "the main copy stays untouched");
    assert_eq!(meta_of(&f.repo, "add-auth"), format!("{META}board_rank: n\n"));
    let plan = json_of(&f.run(&["plan", "--json"]));
    let names: Vec<&str> = plan["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["add-dark-mode", "add-auth"], "{plan}");
}

/// 一個只在主 checkout 新建的 change（worktree 分支之後才出現，副本裡沒有它）。
fn add_main_only(f: &Fixture, change: &str) {
    let dir = f.repo.join("openspec").join("changes").join(change);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(".openspec.yaml"), META).unwrap();
}

fn meta_at(root: &Path, change: &str) -> String {
    std::fs::read_to_string(root.join("openspec").join("changes").join(change).join(".openspec.yaml"))
        .unwrap()
}

fn plan_order(f: &Fixture) -> Vec<String> {
    json_of(&f.run(&["plan", "--json"]))["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn change_rank_sees_a_change_created_after_the_worktree_branched() {
    // 欄序與 plan 同一視角（主 roster ＋映射 change 的副本）：add-late 只在主
    // checkout，映射到 worktree 的 add-dark-mode 仍可排到它前面。add-late 的補章
    // 寫主 checkout，add-dark-mode 的新 rank 寫它自己的 worktree 副本。
    let f = Fixture::new("rank-late-anchor", true);
    let wt = f.add_worktree("add-dark-mode");
    add_main_only(&f, "add-late");

    let out = f.run(&["change", "rank", "add-dark-mode", "--before", "add-late"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(meta_at(&f.repo, "add-late").contains("board_rank: "), "add-late stamped in main");
    assert!(meta_at(&wt, "add-dark-mode").contains("board_rank: "), "the worktree copy is ranked");
    assert_eq!(meta_at(&f.repo, "add-dark-mode"), META, "the main copy of a mapped change is untouched");
    assert_eq!(plan_order(&f), ["add-dark-mode", "add-late"]);
}

#[test]
fn change_rank_stamps_each_change_in_its_own_copy() {
    // 補章逐 change 寫回各自的副本：add-dark-mode 映射到 worktree、缺 rank，要被
    // 補章；它的章必須落在 worktree 副本（plan 讀那份），不能落在主副本。
    let f = Fixture::new("rank-stamp-homes", true);
    let wt = f.add_worktree("add-dark-mode");
    add_main_only(&f, "add-late");

    let out = f.run(&["change", "rank", "add-late", "--after", "add-dark-mode"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(meta_at(&wt, "add-dark-mode").contains("board_rank: "), "the stamp lands in the worktree copy");
    assert_eq!(meta_at(&f.repo, "add-dark-mode"), META, "the main copy is never stamped");
    assert!(meta_at(&f.repo, "add-late").contains("board_rank: "));
    assert_eq!(plan_order(&f), ["add-dark-mode", "add-late"]);
}

#[test]
fn plan_reads_the_worktree_copy_for_the_stage_derivation() {
    // change-plan spec「plan 動詞輸出波次與阻擋清單」：主 checkout 且政策開啟時，
    // plan 與 list 同一聚合面——worktree 內勾掉的任務讓階段判定為 in-progress，
    // 主副本本身仍是 0/2 未開工。
    let f = Fixture::new("plan-stage", true);
    let wt = f.add_worktree("add-dark-mode");
    check_one_task(&wt);

    let out = f.run(&["plan", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let change = json_of(&out)["changes"][0].clone();
    assert_eq!(change["name"], "add-dark-mode");
    assert_eq!(change["stage"], "in-progress", "stage comes from the worktree copy: {change}");

    // 對照：政策關閉時只讀主副本，同一顆 worktree 的勾選不進階段判定。
    let off = Fixture::new("plan-stage-off", false);
    let wt = off.add_worktree("add-dark-mode");
    check_one_task(&wt);
    let change = json_of(&off.run(&["plan", "--json"]))["changes"][0].clone();
    assert_eq!(change["stage"], "proposed", "policy off reads the main copy: {change}");
}

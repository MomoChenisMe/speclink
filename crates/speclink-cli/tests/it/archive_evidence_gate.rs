//! `speclink archive` 的零證據提示與 @trace 形狀 — fs 模式契約
//! (spec verify-evidence「archive trace 注入與零證據提示」)。
//!
//! 四件事在此釘死：批次封存不再要求整潔工作樹（其存在理由——髒檔會混入每個
//! change 的 @trace code 清單——已隨清單移除而消失）、零證據的封存照常成功且
//! stderr 恰一行提示（有證據時一字不印）、正典 @trace 僅 source 與 updated
//! 兩欄且一律注入、證據記錄隨 change 目錄進封存區或隨 discard 消失。封存從不
//! 因 evidence 被擋——放行旗標與守門在討論 evidence-gate-false-blocks 後整套
//! 退場，`--help` 亦不得再出現。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
const DELTA_SPEC: &str = "## ADDED Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 專案根是 git repo 且帶一個髒的程式檔：`task done` 需要可歸屬的髒檔才會
    /// 寫證據，而髒工作樹本身已不再是封存的前提。
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-evidence-gate-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        git(&dir, &["init", "-q"]);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("dirty.rs"), "fn dirty() {}\n").unwrap();
        TempProject { dir }
    }

    /// 一份結構完備、任務全勾的 change。
    fn add_change(&self, name: &str, capability: &str) {
        let change = self.dir.join("openspec").join("changes").join(name);
        std::fs::create_dir_all(change.join("specs").join(capability)).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [x] 1.1 a\n").unwrap();
        std::fs::write(change.join("specs").join(capability).join("spec.md"), DELTA_SPEC).unwrap();
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

    fn change_dir(&self, name: &str) -> PathBuf {
        self.dir.join("openspec").join("changes").join(name)
    }

    fn canonical_spec(&self, capability: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("specs").join(capability).join("spec.md"),
        )
        .unwrap()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn git(root: &Path, args: &[&str]) {
    let ok = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?} failed");
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// 讓 change 帶上真實的證據記錄——走引擎自己的寫入路徑（需要可歸屬的髒檔）。
fn record_real_evidence(p: &TempProject, change: &str) {
    let out = p.run(&["task", "undone", "--change", change, "1"]);
    assert!(out.status.success(), "undone: {}", stderr_of(&out));
    let out = p.run(&["task", "done", "--change", change, "1"]);
    assert!(out.status.success(), "done: {}", stderr_of(&out));
    assert!(
        p.change_dir(change).join(".evidence.json").is_file(),
        "task done must write the evidence record into the change directory"
    );
}

#[test]
fn a_change_without_evidence_archives_and_gets_exactly_one_note() {
    // spec Scenario「零證據照常封存並提示」：純規格 change 掙不到證據——封存照常
    // 成功（exit 0），stderr 恰一行提示點名該 change，且不含任何守門或旗標字眼。
    let p = TempProject::new("note-missing");
    p.add_change("demo", "demo-cap");
    let out = p.run(&["archive", "demo", "--no-color"]);
    assert!(out.status.success(), "zero evidence must not block: {}", stderr_of(&out));
    let stderr = stderr_of(&out);
    let notes: Vec<&str> =
        stderr.lines().filter(|l| l.contains("no task evidence recorded")).collect();
    assert_eq!(notes.len(), 1, "exactly one note line: {stderr}");
    assert!(notes[0].contains("demo"), "the note names the change: {stderr}");
    assert!(!stderr.contains("\u{1b}["), "--no-color leaves no ANSI escapes: {stderr}");
    assert!(!stderr.contains("waive"), "no flag is offered — there is nothing to waive: {stderr}");
    assert!(!p.change_dir("demo").exists(), "the change still archives");
    let canon = p.canonical_spec("demo-cap");
    assert!(canon.contains("<!-- @trace\nsource: demo\nupdated: "), "trace shape: {canon}");
    assert!(!canon.contains("code:"), "no file list in the canon: {canon}");
}

#[test]
fn archive_help_offers_no_evidence_waiver() {
    // 討論 evidence-gate-false-blocks：門拆了，放行旗標就沒有存在的理由——
    // 介面上不得留下任何殘跡。
    let p = TempProject::new("help");
    let out = p.run(&["archive", "--help"]);
    assert!(out.status.success(), "help exits zero: {}", stderr_of(&out));
    let text = format!("{}{}", stdout_of(&out), stderr_of(&out));
    assert!(!text.contains("waive"), "no waiver flag survives in help: {text}");
}

#[test]
fn a_recorded_change_archives_and_the_canon_gets_a_two_field_trace() {
    // spec Scenario「trace 兩欄一律注入」的端對端面：走 task done 產生證據後封存，
    // 正典的 @trace 只有 source 與 updated，且證據隨目錄搬進封存區。
    let p = TempProject::new("trace-two-field");
    p.add_change("demo", "demo-cap");
    record_real_evidence(&p, "demo");

    let out = p.run(&["archive", "demo", "--no-color"]);
    assert!(out.status.success(), "archive succeeds: {}", stderr_of(&out));
    assert!(
        !stderr_of(&out).contains("no task evidence recorded"),
        "spec Scenario「有證據時一字不印」: {}",
        stderr_of(&out)
    );
    let canon = p.canonical_spec("demo-cap");
    assert!(canon.contains("<!-- @trace"), "trace injected: {canon}");
    assert!(canon.contains("source: demo"), "{canon}");
    assert!(canon.contains("updated: "), "{canon}");
    assert!(!canon.contains("code:"), "no file list in the canon: {canon}");
    assert!(!canon.contains("src/a.rs"), "no file list in the canon: {canon}");

    let archived = std::fs::read_dir(p.dir.join("openspec").join("changes").join("archive"))
        .expect("archive dir")
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with("-demo"))
        .expect("archived change dir");
    assert!(
        archived.path().join(".evidence.json").is_file(),
        "the evidence record rides the directory move into the archive"
    );
}

#[test]
fn discard_takes_the_evidence_record_and_the_legacy_orphan_with_it() {
    // spec Scenario「證據隨 change 生命週期移動」的 discard 半邊：記錄住在 change
    // 目錄裡，廢棄即隨目錄消失——這裡直接斷言新位置的路徑不存在，而不是仰賴
    // 「目錄被刪」的副作用。舊路徑的殘檔同時被掃掉：留著會被日後重用同名的
    // change 經回退讀取誤認為自己的記錄。
    let p = TempProject::new("discard-evidence");
    p.add_change("demo", "demo-cap");
    record_real_evidence(&p, "demo");
    let evidence = p.change_dir("demo").join(".evidence.json");
    let legacy = p.dir.join(".speclink").join("touched").join("demo.json");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(&legacy, "{\"change\":\"demo\",\"touched\":[]}").unwrap();

    let out = p.run(&["discard", "demo", "--force", "--no-color"]);
    assert!(out.status.success(), "discard succeeds: {}", stderr_of(&out));
    assert!(!p.change_dir("demo").exists(), "the change directory is gone");
    assert!(!evidence.exists(), "the evidence record dies with the change: {evidence:?}");
    assert!(!legacy.exists(), "no legacy orphan survives the discard: {legacy:?}");
}

#[test]
fn bulk_archive_runs_with_a_dirty_work_tree() {
    // design「bulk 整潔工作樹守門移除」：髒工作樹不再是批次封存的前置條件，
    // 因為髒檔早已不會進入任何 change 的 @trace。
    let p = TempProject::new("bulk-dirty");
    p.add_change("alpha", "alpha-cap");
    p.add_change("beta", "beta-cap");

    let out = p.run(&["archive", "--all", "--no-color"]);
    let stdout = stdout_of(&out);
    assert!(out.status.success(), "a dirty work tree must not block bulk archive: {}", stderr_of(&out));
    assert!(
        !stderr_of(&out).contains("clean work tree"),
        "the clean-work-tree requirement is gone: {}",
        stderr_of(&out)
    );
    assert!(!p.change_dir("alpha").exists(), "alpha archived: {stdout}");
    assert!(!p.change_dir("beta").exists(), "beta archived: {stdout}");
    assert!(
        p.dir.join("src").join("dirty.rs").is_file(),
        "the dirty file is none of archive's business"
    );
}

//! `speclink review` 子命令家族的整合覆蓋（design D4）：add-round／show／
//! show --json（camelCase payload 對外契約）／stamp [--accept]／discard 的
//! exit code 與 stdout/stderr 去向，以及 archive 的未結工單三處置與
//! `--carry-review`。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct TempProject {
    dir: PathBuf,
}

const ROUND_WITH_FINDINGS: &str =
    "**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — unwrap on user input\n";
const CLEAN_ROUND: &str = "**Scope**: src/lib.rs\n";

impl TempProject {
    /// 一個含 change `demo` 的專案：meta＋tasks.md（依 `tasks` 內容）＋
    /// 工作樹檔 src/lib.rs（stamp 指紋的範圍檔）。
    fn with_change(tag: &str, tasks: &str) -> TempProject {
        let dir =
            std::env::temp_dir().join(format!("speclink-cli-review-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-07-01\n")
            .unwrap();
        std::fs::write(change.join("tasks.md"), tasks).unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("lib.rs"), "fn demo() {}\n").unwrap();
        TempProject { dir }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_speclink"));
        c.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        c
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .cmd(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().expect("wait speclink binary")
    }

    fn ticket_path(&self) -> PathBuf {
        self.dir.join("openspec").join("changes").join("demo").join("review.md")
    }

    fn meta(&self) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("changes").join("demo").join(".openspec.yaml"),
        )
        .unwrap()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const TASKS_DONE: &str = "- [x] 1.1 first\n- [x] 1.2 second\n";
const TASKS_PARTIAL: &str = "- [x] 1.1 first\n- [ ] 1.2 second\n";

// --- review add-round ---

#[test]
fn add_round_creates_the_ticket_and_reports_the_round() {
    // spec「審查工單的建立與追加」Scenario 首輪建立工單：exit 0、review.md 建立
    // 且含 Round 1、stdout 確認訊息。
    let p = TempProject::with_change("addround", TASKS_DONE);
    let out = p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("Round 1"), "stdout confirms: {}", stdout_of(&out));
    let doc = std::fs::read_to_string(p.ticket_path()).expect("ticket created");
    assert!(doc.contains("## Round 1"), "{doc}");
}

#[test]
fn add_round_missing_change_fails_on_stderr() {
    // Scenario change 不存在：非零、stderr 說明找不到變更、無檔案建立。
    let p = TempProject::with_change("addround-ghost", TASKS_DONE);
    let out = p.run_stdin(&["review", "add-round", "ghost", "--stdin"], ROUND_WITH_FINDINGS);
    assert!(!out.status.success(), "missing change → non-zero");
    assert!(stderr_of(&out).contains("ghost"), "stderr names it: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty(), "errors go to stderr only");
}

#[test]
fn add_round_without_scope_fails_and_writes_nothing() {
    // Scenario 內容缺少 Scope：非零、stderr 說明格式要求、工單不變。
    let p = TempProject::with_change("addround-noscope", TASKS_DONE);
    let out = p.run_stdin(&["review", "add-round", "demo", "--stdin"], "- just prose\n");
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("**Scope**:"), "stderr explains: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists(), "refusal must not create the ticket");
}

// --- review show ---

#[test]
fn show_prints_the_ticket_without_ansi_under_no_color() {
    let p = TempProject::with_change("show-human", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["--no-color", "review", "show", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("Round 1"), "{text}");
    assert!(text.contains("src/lib.rs"), "{text}");
    assert!(!text.contains('\u{1b}'), "--no-color must strip ANSI: {text:?}");
}

#[test]
fn show_json_payload_carries_the_camel_case_contract() {
    // design D4：`--json` 欄位 camelCase——change／rounds[].index／rounds[].scope
    // ／rounds[].findings[].severity／lastRound，含型別。
    let p = TempProject::with_change("show-json", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["change"], "demo");
    let rounds = v["rounds"].as_array().expect("rounds array");
    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0]["index"], 1);
    assert_eq!(rounds[0]["scope"][0], "src/lib.rs");
    let finding = &rounds[0]["findings"][0];
    assert_eq!(finding["severity"], "CRITICAL");
    assert_eq!(finding["path"], "src/lib.rs");
    assert!(finding["text"].is_string());
    assert_eq!(v["lastRound"]["index"], 2);
    assert_eq!(v["lastRound"]["findings"].as_array().map(Vec::len), Some(0));
}

#[test]
fn show_without_ticket_fails_semantically() {
    // Scenario 無工單：非零、stderr 說明該 change 無審查工單。
    let p = TempProject::with_change("show-none", TASKS_DONE);
    let out = p.run(&["review", "show", "demo"]);
    assert!(!out.status.success());
    assert!(
        stderr_of(&out).contains("no review ticket"),
        "stderr explains: {}",
        stderr_of(&out)
    );
}

// --- review stamp ---

#[test]
fn stamp_refuses_incomplete_tasks() {
    // spec「蓋章守門與蓋章效果」Scenario 任務未全完成即拒絕。
    let p = TempProject::with_change("stamp-tasks", TASKS_PARTIAL);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "stamp", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("1/2"), "counts named: {}", stderr_of(&out));
    assert!(p.ticket_path().exists(), "ticket survives refusal");
    assert!(!p.meta().contains("reviewed_at"), "no stamp on refusal");
}

#[test]
fn stamp_refuses_findings_without_accept_then_accepts() {
    // Scenario 末輪有未解 findings（無 --accept 拒絕、帶 --accept 放行）。
    let p = TempProject::with_change("stamp-accept", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let refused = p.run(&["review", "stamp", "demo"]);
    assert!(!refused.status.success());
    assert!(stderr_of(&refused).contains("--accept"), "hint: {}", stderr_of(&refused));
    let accepted = p.run(&["review", "stamp", "demo", "--accept"]);
    assert!(accepted.status.success(), "stderr: {}", stderr_of(&accepted));
    assert!(!p.ticket_path().exists(), "ticket deleted by the stamp");
    assert!(p.meta().contains("reviewed_at: "), "meta: {}", p.meta());
}

#[test]
fn stamp_clean_round_writes_the_anchors_and_deletes_the_ticket() {
    // Scenario 乾淨蓋章：meta 帶章（任務錨＋指紋錨）且 review.md 不存在。
    let p = TempProject::with_change("stamp-clean", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "stamp", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists());
    let meta = p.meta();
    assert!(meta.contains("reviewed_at: "), "{meta}");
    assert!(meta.contains("reviewed_tasks_total: 2"), "{meta}");
    assert!(meta.contains("reviewed_scope:"), "{meta}");
    assert!(meta.contains("  - path: src/lib.rs"), "{meta}");
    assert!(meta.contains("    hash: "), "{meta}");
}

// --- review discard ---

#[test]
fn discard_deletes_the_ticket_and_leaves_meta() {
    // spec「放棄審查」：exit 0、review.md 不存在、.openspec.yaml 不變。
    let p = TempProject::with_change("discard", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let before = p.meta();
    let out = p.run(&["review", "discard", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists());
    assert_eq!(p.meta(), before, "metadata byte-identical");
}

#[test]
fn discard_without_ticket_fails_semantically() {
    let p = TempProject::with_change("discard-none", TASKS_DONE);
    let out = p.run(&["review", "discard", "demo"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("no review ticket"), "{}", stderr_of(&out));
}

// --- archive 未結工單三處置與 --carry-review ---

#[test]
fn archive_with_open_ticket_lists_three_disposals() {
    // spec「封存的未結工單守門」Scenario 有工單預設拒絕：stderr 同列三處置。
    let p = TempProject::with_change("archive-gate", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["archive", "demo", "--skip-specs", "--no-validate"]);
    assert!(!out.status.success());
    let stderr = stderr_of(&out);
    assert!(stderr.contains("review stamp"), "{stderr}");
    assert!(stderr.contains("review discard"), "{stderr}");
    assert!(stderr.contains("--carry-review"), "{stderr}");
    assert!(p.ticket_path().exists(), "change not moved");
}

#[test]
fn archive_carry_review_moves_the_ticket_into_the_archive() {
    // Scenario 明示帶走：封存成功、封存目錄內含 review.md。
    let p = TempProject::with_change("archive-carry", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["archive", "demo", "--skip-specs", "--no-validate", "--carry-review"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let archive_root = p.dir.join("openspec").join("changes").join("archive");
    let dated = std::fs::read_dir(&archive_root)
        .expect("archive dir exists")
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with("-demo"))
        .expect("archived change dir");
    assert!(dated.path().join("review.md").exists(), "fossil ticket travelled");
}

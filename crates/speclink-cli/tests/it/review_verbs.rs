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

    /// `with_change` 之上再 git init＋首次 commit：prepare／scope 的 Git 前提。
    fn with_git_change(tag: &str, tasks: &str) -> TempProject {
        let p = TempProject::with_change(tag, tasks);
        p.git(&["init", "-q"]);
        p.git(&["config", "user.name", "Sandbox Tester"]);
        p.git(&["config", "user.email", "sandbox@example.com"]);
        p.git(&["add", "-A"]);
        p.git(&["commit", "-q", "-m", "init"]);
        p
    }

    fn git(&self, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(&self.dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "git {args:?} failed");
    }

    fn write(&self, rel: &str, content: &str) {
        let p = self.dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn baseline_path(&self) -> PathBuf {
        self.dir.join(".speclink").join("review-scopes").join("demo").join("baseline.json")
    }

    /// 寫 host-local touched record（v1 檔案清單通道）。
    fn touched(&self, change: &str, files: &[&str]) {
        let dir = self.dir.join(".speclink").join("touched");
        std::fs::create_dir_all(&dir).unwrap();
        let files_json: Vec<String> = files.iter().map(|f| format!("\"{f}\"")).collect();
        std::fs::write(
            dir.join(format!("{change}.json")),
            format!(
                "{{\"version\":2,\"change\":\"{change}\",\"touched\":[{{\"task_id\":\"1\",\"task_desc\":\"t\",\"files\":[{}]}}]}}",
                files_json.join(",")
            ),
        )
        .unwrap();
    }

    fn snapshot_count(&self) -> usize {
        std::fs::read_dir(
            self.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots"),
        )
        .map(|it| it.count())
        .unwrap_or(0)
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

// --- review prepare（change-diff-scope spec「Apply 開始前記錄 host-local baseline」）---

#[test]
fn review_prepare_initial_capture_is_silent_and_writes_the_baseline() {
    // Scenario 首次 Apply 記錄乾淨 baseline：exit 0、stdout 為空，baseline 的
    // baseCommit 為 HEAD SHA、dirtyFilesAtStart 為 ["notes/local.txt"]、
    // confidence 為 initial，touched 記錄不存在。
    let p = TempProject::with_git_change("prepare-initial", TASKS_DONE);
    p.write("notes/local.txt", "scratch\n");
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty(), "initial capture is silent: {}", stdout_of(&out));
    let raw = std::fs::read_to_string(p.baseline_path()).expect("baseline written");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(v["change"], "demo");
    assert_eq!(v["confidence"], "initial");
    assert_eq!(v["dirtyFilesAtStart"], serde_json::json!(["notes/local.txt"]));
    assert_eq!(v["baseCommit"].as_str().map(str::len), Some(40), "full HEAD SHA: {v}");
    assert!(v["capturedAt"].is_string(), "camelCase capturedAt: {v}");
    assert!(
        !p.dir.join(".speclink").join("touched").join("demo.json").exists(),
        "prepare must not create a touched record"
    );
}

#[test]
fn review_prepare_late_warns_on_stderr_but_exits_zero() {
    // Scenario 已開始但 baseline 缺失：exit 0、stderr 說明 baseline 為 late、
    // 後續審查需明示 fixed point。
    let p = TempProject::with_git_change("prepare-late", TASKS_DONE);
    let meta = p.dir.join("openspec").join("changes").join("demo").join(".openspec.yaml");
    std::fs::write(
        &meta,
        "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-02\n",
    )
    .unwrap();
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(out.status.success(), "late is a warning, not a failure: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty(), "stdout stays empty: {}", stdout_of(&out));
    assert!(stderr_of(&out).contains("late"), "stderr names late: {}", stderr_of(&out));
    let raw = std::fs::read_to_string(p.baseline_path()).expect("late baseline written");
    assert!(raw.contains("\"confidence\": \"late\""), "{raw}");
}

#[test]
fn review_prepare_without_git_warns_and_records_unavailable() {
    // spec：無 Git checkout → confidence=unavailable、baseCommit=null、
    // stderr 警告但 exit 0（apply 可繼續）。
    let p = TempProject::with_change("prepare-nogit", TASKS_DONE);
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(out.status.success(), "unavailable is a warning: {}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty());
    assert!(!stderr_of(&out).is_empty(), "stderr must warn about the missing fixed point");
    let raw = std::fs::read_to_string(p.baseline_path()).expect("baseline written");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");
    assert_eq!(v["confidence"], "unavailable");
    assert!(v["baseCommit"].is_null(), "{v}");
}

#[test]
fn review_prepare_write_failure_fails_nonzero_and_leaves_meta() {
    // Scenario baseline 寫入失敗停止 Apply 起點：非零、change metadata 原狀。
    let p = TempProject::with_git_change("prepare-fail", TASKS_DONE);
    std::fs::create_dir_all(p.dir.join(".speclink")).unwrap();
    std::fs::write(p.dir.join(".speclink").join("review-scopes"), "not a dir").unwrap();
    let before = p.meta();
    let out = p.run(&["review", "prepare", "demo"]);
    assert!(!out.status.success(), "write failure must be non-zero");
    assert!(!stderr_of(&out).is_empty(), "stderr explains the failure");
    assert_eq!(p.meta(), before, "metadata must stay byte-identical");
}

#[test]
fn review_prepare_missing_change_fails_nonzero() {
    // spec：change 不存在 SHALL 非零結束，且不建立 sidecar。
    let p = TempProject::with_git_change("prepare-ghost", TASKS_DONE);
    let out = p.run(&["review", "prepare", "ghost"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("ghost"), "stderr names it: {}", stderr_of(&out));
    assert!(
        !p.dir.join(".speclink").join("review-scopes").join("ghost").exists(),
        "no sidecar for a missing change"
    );
}

// --- review scope（change-diff-scope spec「review scope 的 human 與 JSON 契約」
// 「歧義 scope 必須 fail closed 並以 hash-pinned selection 解鎖」）---

/// 兩檔三 hunk 的可自動歸屬 fixture：乾淨 baseline 後修改 touched 檔。
fn scope_fixture(tag: &str) -> TempProject {
    let p = TempProject::with_change(tag, TASKS_DONE);
    let wide: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    p.write("src/util.rs", &wide);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() {}\nfn added() {}\n");
    let edited = wide
        .replace("line 2\n", "line 2 edited\n")
        .replace("line 18\n", "line 18 edited\n");
    p.write("src/util.rs", &edited);
    p.touched("demo", &["src/lib.rs", "src/util.rs"]);
    p
}

#[test]
fn review_scope_resolved_json_payload_is_camel_case() {
    // spec Scenario「JSON resolved payload 可供 reviewer 使用」：exit 0、合法
    // JSON、state=resolved、paths 兩項、hunks 合計三項、patchHash 以 sha256: 開頭。
    let p = scope_fixture("scope-json");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["change"], "demo");
    assert_eq!(v["phase"], "discovery");
    assert_eq!(v["state"], "resolved");
    assert_eq!(v["baseCommit"].as_str().map(str::len), Some(40), "{v}");
    assert!(v["candidateHash"].as_str().unwrap().starts_with("sha256:"), "{v}");
    assert!(v["patchHash"].as_str().unwrap().starts_with("sha256:"), "{v}");
    assert_eq!(v["paths"], serde_json::json!(["src/lib.rs", "src/util.rs"]));
    let files = v["files"].as_array().expect("files array");
    assert_eq!(files.len(), 2);
    let total_hunks: usize = files.iter().map(|f| f["hunks"].as_array().unwrap().len()).sum();
    assert_eq!(total_hunks, 3, "{v}");
    for f in files {
        assert!(f["oldPath"].is_string() || f["oldPath"].is_null(), "{f}");
        assert!(f["newPath"].is_string() || f["newPath"].is_null(), "{f}");
        assert!(f["kind"].is_string(), "{f}");
        assert!(f["beforeHash"].is_string() || f["beforeHash"].is_null(), "{f}");
        assert!(f["afterHash"].is_string() || f["afterHash"].is_null(), "{f}");
        for h in f["hunks"].as_array().unwrap() {
            assert!(h["id"].is_string(), "{h}");
            for k in ["oldStart", "oldLines", "newStart", "newLines"] {
                assert!(h[k].is_number(), "range {k} is a number: {h}");
            }
        }
    }
    assert!(v["patch"].as_str().unwrap().contains("+fn added() {}"), "{v}");
}

#[test]
fn review_scope_human_output_has_no_ansi_under_no_color() {
    // spec：human 成功路徑將 phase、patchHash、路徑數與 hunk 數寫至 stdout，
    // `--no-color` 下不含 ANSI。
    let p = scope_fixture("scope-human");
    let out = p.run(&["--no-color", "review", "scope", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("discovery"), "phase shown: {text}");
    assert!(text.contains("sha256:"), "patchHash shown: {text}");
    // 數字必須連著它的單位一起斷言：光找 '2'／'3' 會被 64 字元 hex digest 命中。
    assert!(text.contains("2 file(s)"), "path count shown: {text}");
    assert!(text.contains("3 hunk(s)"), "hunk count shown: {text}");
    assert!(!text.contains('\u{1b}'), "--no-color must strip ANSI: {text:?}");
}

#[test]
fn review_scope_dirty_at_start_needs_input_nonzero_with_zero_snapshot_effects() {
    // spec Scenario「開始前已髒的 touched file 不被靜默認領」：非零、JSON state
    // 為 needsInput、ambiguousPaths 含該檔、snapshots 目錄不新增檔案。
    let p = TempProject::with_change("scope-dirty", TASKS_DONE);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    p.write("src/lib.rs", "fn demo() { dirty_before_start(); }\n");
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "prepare: {}", stderr_of(&prepared));
    p.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); }\n");
    p.touched("demo", &["src/lib.rs"]);
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "needsInput must exit non-zero");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON on stdout");
    assert_eq!(v["state"], "needsInput");
    assert!(v["candidateHash"].is_string() || v["candidateHash"].is_null(), "{v}");
    assert_eq!(v["ambiguousPaths"], serde_json::json!(["src/lib.rs"]));
    assert!(v["files"].is_array(), "{v}");
    assert_eq!(p.snapshot_count(), 0, "zero snapshot effects");
    // human 路徑：stderr 列 ambiguous paths 與三種處置。
    let human = p.run(&["--no-color", "review", "scope", "demo"]);
    assert!(!human.status.success());
    let stderr = stderr_of(&human);
    assert!(stderr.contains("src/lib.rs"), "{stderr}");
    assert!(stderr.contains("--base"), "{stderr}");
    assert!(stderr.contains("--include-hunk"), "{stderr}");
    assert!(stderr.contains("worktree"), "{stderr}");
    assert!(!stderr.contains('\u{1b}'), "--no-color strips ANSI on stderr too: {stderr:?}");
}

#[test]
fn review_scope_empty_touched_needs_input_and_never_reviews_the_worktree() {
    // spec：touchedFiles 缺失或為空 SHALL NOT 自動審查全 worktree。
    let p = TempProject::with_git_change("scope-empty", TASKS_DONE);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    p.write("src/lib.rs", "fn demo() { changed(); }\n");
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "empty touched must fail closed");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON on stdout");
    assert_eq!(v["state"], "needsInput");
    assert_eq!(p.snapshot_count(), 0);
}

#[test]
fn review_scope_active_overlap_needs_input() {
    // spec：另一 active change 的 touched record 認領同一路徑 → needsInput。
    let p = scope_fixture("scope-overlap");
    let other = p.dir.join("openspec").join("changes").join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-07-01\n")
        .unwrap();
    p.touched("other", &["src/lib.rs"]);
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "active overlap must fail closed");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON on stdout");
    assert_eq!(v["state"], "needsInput");
    assert!(
        v["ambiguousPaths"].as_array().unwrap().contains(&serde_json::json!("src/lib.rs")),
        "{v}"
    );
    assert_eq!(p.snapshot_count(), 0);
}

#[test]
fn review_scope_candidate_drift_rejects_the_stale_selection() {
    // spec Scenario「candidate 漂移拒絕舊選擇」：帶舊 candidateHash 重試 →
    // 非零、stderr 說明漂移、不建立 snapshot。
    let p = TempProject::with_change("scope-drift", TASKS_DONE);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    p.write("src/lib.rs", "fn demo() { dirty_before_start(); }\n");
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    p.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); }\n");
    p.touched("demo", &["src/lib.rs"]);
    let first = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!first.status.success(), "fixture must be ambiguous");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&first)).expect("JSON");
    let stale = v["candidateHash"].as_str().expect("candidate anchor").to_string();
    let hunk = v["files"][0]["hunks"][0]["id"].as_str().expect("hunk id").to_string();
    p.write("src/lib.rs", "fn demo() { dirty_before_start(); more(); drifted(); }\n");
    let out = p.run(&[
        "review",
        "scope",
        "demo",
        "--candidate-hash",
        &stale,
        "--include-hunk",
        &hunk,
    ]);
    assert!(!out.status.success(), "drifted candidate must be rejected");
    assert!(stderr_of(&out).contains("drift"), "{}", stderr_of(&out));
    assert_eq!(p.snapshot_count(), 0);
}

#[test]
fn review_scope_selection_without_candidate_hash_fails() {
    // spec：人工 selection SHALL 同時提供前次 candidateHash 與至少一個
    // include-hunk——單獨 --include-hunk 拒絕。
    let p = scope_fixture("scope-lonely-hunk");
    let out = p.run(&["review", "scope", "demo", "--include-hunk", &"a".repeat(64)]);
    assert!(!out.status.success(), "selection without --candidate-hash must be rejected");
    assert!(stderr_of(&out).contains("--candidate-hash"), "{}", stderr_of(&out));
    assert_eq!(p.snapshot_count(), 0);
}

#[test]
fn review_scope_missing_change_fails_nonzero() {
    // spec Scenario「找不到 change」：非零、stderr 說明、stdout 為空、不建立
    // baseline 或 snapshot。
    let p = TempProject::with_git_change("scope-ghost", TASKS_DONE);
    let out = p.run(&["review", "scope", "ghost", "--json"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("ghost"), "{}", stderr_of(&out));
    assert!(stdout_of(&out).is_empty(), "stdout stays empty on not-found");
    assert!(
        !p.dir.join(".speclink").join("review-scopes").join("ghost").exists(),
        "no sidecar for a missing change"
    );
}

// --- snapshot cleanup（change-diff-scope spec「frozen snapshot 綁定 discovery
// 與 validation patch」：stamp／discard 清 snapshots、保留 baseline）---

#[test]
fn stamp_clears_review_snapshots_and_keeps_the_baseline() {
    // spec Scenario「成功蓋章後清除 snapshots」。
    let p = TempProject::with_git_change("stamp-snapclean", TASKS_DONE);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    let snapdir = p.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots");
    std::fs::create_dir_all(&snapdir).unwrap();
    std::fs::write(snapdir.join(format!("{}.json", "a".repeat(64))), "{}").unwrap();
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "stamp", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.snapshot_count(), 0, "snapshots cleared by the stamp");
    assert!(p.baseline_path().exists(), "baseline survives the stamp");
    assert!(p.meta().contains("reviewed_at"), "canonical stamp landed");
}

#[test]
fn discard_clears_review_snapshots_and_keeps_the_baseline() {
    let p = TempProject::with_git_change("discard-snapclean", TASKS_DONE);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    let snapdir = p.dir.join(".speclink").join("review-scopes").join("demo").join("snapshots");
    std::fs::create_dir_all(&snapdir).unwrap();
    std::fs::write(snapdir.join(format!("{}.json", "b".repeat(64))), "{}").unwrap();
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["review", "discard", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.snapshot_count(), 0, "snapshots cleared by the discard");
    assert!(p.baseline_path().exists(), "baseline survives the discard");
    assert!(!p.ticket_path().exists());
}

#[test]
fn stamp_warns_but_succeeds_when_snapshot_cleanup_fails() {
    // spec：清除失敗 SHALL 以 stderr warning 回報，且 SHALL NOT 回滾已完成的
    // canonical 工單／metadata mutation。
    let p = TempProject::with_git_change("stamp-snapwarn", TASKS_DONE);
    let scopes = p.dir.join(".speclink").join("review-scopes").join("demo");
    std::fs::create_dir_all(&scopes).unwrap();
    // snapshots 路徑是「檔案」：remove_dir_all 必然失敗。
    std::fs::write(scopes.join("snapshots"), "not a dir").unwrap();
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "stamp", "demo"]);
    assert!(out.status.success(), "cleanup failure must not fail the stamp: {}", stderr_of(&out));
    assert!(
        stderr_of(&out).to_lowercase().contains("snapshot"),
        "stderr warns about the cleanup: {}",
        stderr_of(&out)
    );
    assert!(p.meta().contains("reviewed_at"), "canonical stamp landed anyway");
}

// --- structured rounds（review-station spec「審查工單的建立與追加／讀取」）---

#[test]
fn show_json_carries_nullable_phase_and_patch_hash() {
    // spec Scenario「讀取 structured 兩輪 JSON」＋「legacy JSON 使用 null」：
    // rounds[].phase／patchHash 為 string|null，欄位集合 local 契約。
    let p = TempProject::with_change("show-structured", TASKS_DONE);
    let hex = "a".repeat(64);
    let structured = format!(
        "**Phase**: discovery\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — unwrap on user input\n"
    );
    let out = p.run_stdin(&["review", "add-round", "demo", "--stdin"], &structured);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let validation = format!(
        "**Phase**: validation\n**Patch**: sha256:{}\n**Scope**: src/lib.rs\n",
        "b".repeat(64)
    );
    let out = p.run_stdin(&["review", "add-round", "demo", "--stdin"], &validation);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let out = p.run(&["review", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    let rounds = v["rounds"].as_array().expect("rounds");
    assert_eq!(rounds[0]["phase"], "discovery");
    assert_eq!(rounds[0]["patchHash"], format!("sha256:{hex}"));
    assert_eq!(v["lastRound"]["phase"], "validation");
    assert_eq!(v["lastRound"]["index"], 2);
    // 欄位集合釘死（local／remote 同構的 local 半邊）。
    let mut keys: Vec<&str> = rounds[0].as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["findings", "index", "patchHash", "phase", "scope"]);
}

#[test]
fn show_json_legacy_round_emits_explicit_nulls() {
    let p = TempProject::with_change("show-legacy-null", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["review", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    let round = v["rounds"][0].as_object().expect("round object");
    assert!(
        round.get("phase").is_some_and(serde_json::Value::is_null),
        "phase key present and null: {round:?}"
    );
    assert!(
        round.get("patchHash").is_some_and(serde_json::Value::is_null),
        "patchHash key present and null: {round:?}"
    );
}

// --- 6.1 完整 fixture：prepare → touched → discovery → structured Round 1 →
// remediation validation（2→1／1→1／1→0／accepted）＋ sharp-edges audit ---

/// 取 scope --json 的 resolved payload（斷言 state 並回傳）。
fn scope_json(p: &TempProject) -> serde_json::Value {
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(out.status.success(), "scope stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "resolved");
    v
}

fn add_structured_round(p: &TempProject, phase: &str, patch_hash: &str, scope: &str, findings: &str) {
    let content = format!("**Phase**: {phase}\n**Patch**: {patch_hash}\n**Scope**: {scope}\n\n{findings}");
    let out = p.run_stdin(&["review", "add-round", "demo", "--stdin"], &content);
    assert!(out.status.success(), "add-round stderr: {}", stderr_of(&out));
}

/// 驗證輪 fixture：A（findings 點名）、B（未點名候選檔）、notes/d.txt（開工前
/// 就髒、從未進審查面）。回傳的專案停在「Round 1 已記錄、修復已完成」的狀態。
fn validation_fixture(tag: &str) -> TempProject {
    let p = TempProject::with_change(tag, TASKS_DONE);
    p.write("src/a.rs", "alpha\n");
    p.write("src/b.rs", "beta\n");
    p.write("notes/d.txt", "scratch\n");
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "src/a.rs", "src/b.rs"]);
    p.git(&["commit", "-q", "-m", "init"]);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "prepare: {}", stderr_of(&prepared));
    p.write("src/a.rs", "alpha\nbad_a\n");
    p.write("src/b.rs", "beta\nround_one_b\n");
    p.touched("demo", &["src/a.rs", "src/b.rs"]);
    let r1 = scope_json(&p);
    let p1 = r1["patchHash"].as_str().unwrap().to_string();
    add_structured_round(
        &p,
        "discovery",
        &p1,
        "src/a.rs, src/b.rs",
        "- [CRITICAL] src/a.rs — Correctness: bad_a breaks the invariant\n",
    );
    // 修復：點名的 A、未點名的鄰居 B，外加審查面外的 notes/d.txt 也動了。
    p.write("src/a.rs", "alpha\ngood_a\n");
    p.write("src/b.rs", "beta\nround_one_b\nneighbour_fix\n");
    p.write("notes/d.txt", "scratch changed\n");
    p
}

#[test]
fn review_scope_validation_payload_carries_attribution_and_out_of_scope() {
    // spec Scenario「validation payload 帶出身標記與範圍外註記」：files 分別帶
    // attribution "finding"／"adjacent"，outOfScopeChanged 含被排除檔的路徑且該
    // 檔不在 files 中；discovery payload 的 files 則缺席 attribution。
    let p = validation_fixture("scope-attribution");
    let v = scope_json(&p);
    assert_eq!(v["phase"], "validation");
    let attribution = |path: &str| {
        v["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["newPath"] == path)
            .map(|f| f["attribution"].clone())
    };
    assert_eq!(attribution("src/a.rs"), Some(serde_json::json!("finding")), "{v}");
    assert_eq!(attribution("src/b.rs"), Some(serde_json::json!("adjacent")), "{v}");
    assert_eq!(v["outOfScopeChanged"], serde_json::json!(["notes/d.txt"]), "{v}");
    assert_eq!(v["paths"], serde_json::json!(["src/a.rs", "src/b.rs"]), "{v}");
    assert!(!v["patch"].as_str().unwrap().contains("notes/d.txt"), "{v}");
    assert!(v["patch"].as_str().unwrap().contains("+neighbour_fix"), "{v}");

    // discovery：無上輪可歸因 → attribution 缺席，清單欄位仍恆存在。
    let d = scope_json(&scope_fixture("scope-attribution-discovery"));
    assert_eq!(d["phase"], "discovery");
    for f in d["files"].as_array().unwrap() {
        assert!(f.get("attribution").is_none(), "discovery 不帶出身標記: {f}");
    }
    assert_eq!(d["outOfScopeChanged"], serde_json::json!([]), "{d}");
}

#[test]
fn validation_annotates_out_of_scope_movement_instead_of_needing_input() {
    // spec Scenario「範圍外變動註記不擋凍結」＋「needsInput 僅發生於 discovery」：
    // 保存面外、開工前就髒的檔案又變了 → 照常 exit 0 resolved，human 多一行
    // 範圍外變動 FYI，該檔不進審查面。
    let p = validation_fixture("scope-validation-fyi");
    let human = p.run(&["--no-color", "review", "scope", "demo"]);
    assert!(human.status.success(), "validation must not need input: {}", stderr_of(&human));
    let text = stdout_of(&human);
    assert!(text.contains("validation"), "phase shown: {text}");
    assert!(text.contains("1 finding, 1 adjacent, 0 new"), "三類出身計數列出: {text}");
    assert!(text.contains("notes/d.txt"), "範圍外變動路徑列出: {text}");
    assert!(!text.contains('\u{1b}'), "--no-color must strip ANSI: {text:?}");
    assert!(!stderr_of(&human).contains("needs input"), "{}", stderr_of(&human));
    assert_eq!(p.snapshot_count(), 2, "驗證輪快照照常寫入");
}

#[test]
fn review_full_remediation_loop_end_to_end() {
    // 完整迴圈：discovery 兩筆必修 → 修一筆（2→1 驗證只出 remediation patch）
    // → 修最後一筆（1→0）→ 乾淨蓋章；snapshots 清除、baseline 保留。
    let p = TempProject::with_change("e2e-loop", TASKS_DONE);
    p.write("src/a.rs", "alpha\n");
    p.write("src/b.rs", "beta\n");
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success(), "prepare: {}", stderr_of(&prepared));

    // Apply 期間的修改與 touched 記錄。
    p.write("src/a.rs", "alpha\nbad_a\n");
    p.write("src/b.rs", "beta\nbad_b\n");
    p.touched("demo", &["src/a.rs", "src/b.rs"]);

    // Round 1：discovery，兩筆必修。
    let r1 = scope_json(&p);
    assert_eq!(r1["phase"], "discovery");
    let p1 = r1["patchHash"].as_str().unwrap().to_string();
    add_structured_round(
        &p,
        "discovery",
        &p1,
        "src/a.rs, src/b.rs",
        "- [CRITICAL] src/a.rs — Correctness: bad_a breaks the invariant\n- [CRITICAL] src/b.rs — Correctness: bad_b breaks the invariant\n",
    );

    // Round 2（2→1）：只修 a；validation patch 只含 a 的 remediation，不重播
    // Round 1、不含未修改的 b。
    p.write("src/a.rs", "alpha\ngood_a\n");
    let r2 = scope_json(&p);
    assert_eq!(r2["phase"], "validation");
    let p2 = r2["patchHash"].as_str().unwrap().to_string();
    assert_ne!(p2, p1);
    let patch2 = r2["patch"].as_str().unwrap();
    assert!(patch2.contains("+good_a"), "remediation in: {patch2}");
    assert!(!patch2.contains("bad_b"), "unchanged finding file not re-emitted: {patch2}");
    assert_eq!(r2["paths"], serde_json::json!(["src/a.rs"]));
    add_structured_round(
        &p,
        "validation",
        &p2,
        "src/a.rs",
        "- [CRITICAL] src/b.rs — Correctness: bad_b breaks the invariant\n",
    );

    // Round 3（1→0）：修 b → 乾淨輪 → 蓋章。
    p.write("src/b.rs", "beta\ngood_b\n");
    let r3 = scope_json(&p);
    assert_eq!(r3["phase"], "validation");
    let p3 = r3["patchHash"].as_str().unwrap().to_string();
    let patch3 = r3["patch"].as_str().unwrap();
    assert!(patch3.contains("+good_b"), "final remediation in: {patch3}");
    assert!(!patch3.contains("good_a"), "resolved finding not re-validated: {patch3}");
    add_structured_round(&p, "validation", &p3, "src/b.rs", "");
    let stamped = p.run(&["review", "stamp", "demo"]);
    assert!(stamped.status.success(), "stamp: {}", stderr_of(&stamped));
    assert!(p.meta().contains("reviewed_at"), "canonical stamp landed");
    assert_eq!(p.snapshot_count(), 0, "snapshots cleared by the stamp");
    assert!(p.baseline_path().exists(), "baseline survives");
    assert!(!p.ticket_path().exists(), "ticket deleted by the stamp");
}

#[test]
fn review_no_progress_round_keeps_the_ticket_and_stamp_still_refuses() {
    // 1→1：remediation 什麼都沒修——validation 輪記錄原 finding 原文後，未帶
    // --accept 的 stamp 仍拒絕（工單保留；failed 是技能層決策，動詞不蓋章）。
    let p = TempProject::with_change("e2e-noprogress", TASKS_DONE);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    p.write("src/lib.rs", "fn demo() { bad(); }\n");
    p.touched("demo", &["src/lib.rs"]);
    let r1 = scope_json(&p);
    let p1 = r1["patchHash"].as_str().unwrap().to_string();
    const FINDING: &str = "- [CRITICAL] src/lib.rs — Correctness: bad() panics on empty input\n";
    add_structured_round(&p, "discovery", &p1, "src/lib.rs", FINDING);
    // 無修改的 validation：空 remediation patch，原 finding 原文續記。
    let r2 = scope_json(&p);
    assert_eq!(r2["phase"], "validation");
    let p2 = r2["patchHash"].as_str().unwrap().to_string();
    assert_eq!(r2["patch"], "", "nothing was remediated");
    add_structured_round(&p, "validation", &p2, "src/lib.rs", FINDING);
    let refused = p.run(&["review", "stamp", "demo"]);
    assert!(!refused.status.success(), "unresolved finding must refuse the stamp");
    assert!(stderr_of(&refused).contains("--accept"), "{}", stderr_of(&refused));
    assert!(p.ticket_path().exists(), "ticket survives the refusal");

    // accepted 帶保留：使用者明示 --accept 才蓋章。
    let accepted = p.run(&["review", "stamp", "demo", "--accept"]);
    assert!(accepted.status.success(), "stderr: {}", stderr_of(&accepted));
    assert!(p.meta().contains("reviewed_at"));
    assert_eq!(p.snapshot_count(), 0, "snapshots cleared");
    assert!(p.baseline_path().exists(), "baseline survives");
}

#[test]
fn review_scope_flag_sharp_edges_fail_closed() {
    // sharp-edges audit：--base／--candidate-hash／--include-hunk 的空值、
    // 重複、漂移與路徑穿越全部 fail closed，零 snapshot effects。
    let p = TempProject::with_change("edges", TASKS_DONE);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    // dirty-at-start 讓 scope 落在 needsInput，取得合法 candidate anchor。
    p.write("src/lib.rs", "fn demo() { dirty(); }\n");
    let prepared = p.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    p.write("src/lib.rs", "fn demo() { dirty(); more(); }\n");
    p.touched("demo", &["src/lib.rs"]);
    let out = p.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success());
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON");
    let anchor = v["candidateHash"].as_str().expect("anchor").to_string();
    let hunk = v["files"][0]["hunks"][0]["id"].as_str().expect("hunk id").to_string();

    let cases: Vec<Vec<&str>> = vec![
        // 空 --base：無效 rev。
        vec!["review", "scope", "demo", "--base", ""],
        // 亂 --base。
        vec!["review", "scope", "demo", "--base", "not-a-rev"],
        // 空 candidate hash＋合法 hunk：anchor 不符。
        vec!["review", "scope", "demo", "--candidate-hash", "", "--include-hunk", &hunk],
        // 合法 anchor＋空 hunk id：不存在。
        vec!["review", "scope", "demo", "--candidate-hash", &anchor, "--include-hunk", ""],
        // 重複 hunk id。
        vec![
            "review", "scope", "demo", "--candidate-hash", &anchor, "--include-hunk", &hunk,
            "--include-hunk", &hunk,
        ],
    ];
    for args in cases {
        let out = p.run(&args);
        assert!(!out.status.success(), "args {args:?} must fail closed");
        assert!(!stderr_of(&out).is_empty(), "args {args:?} explain on stderr");
        assert_eq!(p.snapshot_count(), 0, "args {args:?} leave zero snapshot effects");
    }

    // 路徑穿越：touched 夾帶 repo 外路徑——git 拒絕、scope 非零、零 effects。
    for evil in ["../outside.txt", "/etc/passwd"] {
        let q = TempProject::with_git_change(&format!("edges-{}", evil.len()), TASKS_DONE);
        let prepared = q.run(&["review", "prepare", "demo"]);
        assert!(prepared.status.success());
        q.touched("demo", &[evil]);
        let out = q.run(&["review", "scope", "demo", "--json"]);
        assert!(!out.status.success(), "traversal {evil} must fail closed");
        assert_eq!(q.snapshot_count(), 0, "traversal {evil} leaves zero snapshot effects");
    }

    // binary：dirty-at-start 的 binary candidate 沒有可選 hunk，選擇一律拒絕。
    let b = TempProject::with_change("edges-bin", TASKS_DONE);
    b.git(&["init", "-q"]);
    b.git(&["config", "user.name", "Sandbox Tester"]);
    b.git(&["config", "user.email", "sandbox@example.com"]);
    b.git(&["add", "-A"]);
    b.git(&["commit", "-q", "-m", "init"]);
    std::fs::create_dir_all(b.dir.join("assets")).unwrap();
    std::fs::write(b.dir.join("assets/logo.bin"), [0u8, 1, 2]).unwrap();
    let prepared = b.run(&["review", "prepare", "demo"]);
    assert!(prepared.status.success());
    std::fs::write(b.dir.join("assets/logo.bin"), [0u8, 9, 9]).unwrap();
    b.touched("demo", &["assets/logo.bin"]);
    let out = b.run(&["review", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "dirty binary is ambiguous");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("JSON");
    let bin_anchor = v["candidateHash"].as_str().expect("anchor").to_string();
    assert_eq!(
        v["files"][0]["hunks"].as_array().map(Vec::len),
        Some(0),
        "binary exposes no selectable hunks: {v}"
    );
    let out = b.run(&[
        "review", "scope", "demo", "--candidate-hash", &bin_anchor, "--include-hunk",
        &"f".repeat(64),
    ]);
    assert!(!out.status.success(), "binary can never be hunk-selected");
    assert_eq!(b.snapshot_count(), 0);
}

//! `speclink verify` 子命令家族的整合覆蓋（design D3／D8）：scope／add-round／
//! show／show --json（camelCase payload 對外契約）／stamp [--accept]／discard 的
//! exit code 與 stdout/stderr 去向，archive 的驗證工單三處置與 `--carry-verify`，
//! 以及兩站 snapshot namespace 的清理隔離。
//!
//! 驗證站的刻意不對稱（design D3）在此釘死：任務未全數完成時 `verify add-round`
//! 拒絕——工單語意限定為成品驗證，盤點輪不落工單。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const ROUND_WITH_FINDINGS: &str =
    "**Scope**: src/lib.rs\n\n- [CRITICAL] src/lib.rs — requirement R2 has no implementation\n";
const SUGGESTION_ROUND: &str =
    "**Scope**: src/lib.rs\n\n- [SUGGESTION] src/lib.rs — design says otherwise\n";
const CLEAN_ROUND: &str = "**Scope**: src/lib.rs\n";
const TASKS_DONE: &str = "- [x] 1.1 first\n- [x] 1.2 second\n";
const TASKS_PARTIAL: &str = "- [x] 1.1 first\n- [ ] 1.2 second\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn with_change(tag: &str, tasks: &str) -> TempProject {
        let dir =
            std::env::temp_dir().join(format!("speclink-cli-verify-{tag}-{}", std::process::id()));
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

    fn scope_dir(&self) -> PathBuf {
        self.dir.join(".speclink").join("review-scopes").join("demo")
    }

    fn baseline_path(&self) -> PathBuf {
        self.scope_dir().join("baseline.json")
    }

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

    /// 該站 snapshot 目錄的檔數（不存在讀作 0）。
    fn snapshot_count(&self, station_dir: &str) -> usize {
        std::fs::read_dir(self.scope_dir().join(station_dir)).map(|it| it.count()).unwrap_or(0)
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
        self.dir.join("openspec").join("changes").join("demo").join("verify.md")
    }

    fn review_ticket_path(&self) -> PathBuf {
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

// --- verify add-round（design D3：引擎守門）---

#[test]
fn add_round_refuses_until_every_code_task_is_done() {
    // spec Scenario「寫碼任務未完成即拒絕落工單」：部分完成 → 非零、stderr 說明
    // 驗證工單要求寫碼任務全數完成、無檔案建立。（`[M]` 放行的一面在
    // manual_task_gates 釘住。）
    let p = TempProject::with_change("addround-partial", TASKS_PARTIAL);
    let out = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    assert!(!out.status.success(), "incomplete code tasks → non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("1/2"), "stderr shows the count: {err}");
    assert!(err.contains("code task"), "message names code tasks: {err}");
    assert!(!p.ticket_path().exists(), "refusal must not create the ticket");
    assert!(stdout_of(&out).is_empty(), "errors go to stderr only");
}

#[test]
fn add_round_creates_the_ticket_and_reports_the_round() {
    // spec Scenario「首輪建立工單」：exit 0、verify.md 建立且含 Round 1。
    let p = TempProject::with_change("addround", TASKS_DONE);
    let out = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(stdout_of(&out).contains("Round 1"), "stdout confirms: {}", stdout_of(&out));
    let doc = std::fs::read_to_string(p.ticket_path()).expect("ticket created");
    assert!(doc.starts_with("# Verify — demo\n"), "{doc}");
    assert!(doc.contains("## Round 1"), "{doc}");
}

#[test]
fn add_round_appends_without_rewriting_earlier_rounds() {
    // spec Scenario「追加輪次不改寫既有輪」：Round 1 位元級不變。
    let p = TempProject::with_change("addround-append", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let before = std::fs::read_to_string(p.ticket_path()).unwrap();
    let out = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = std::fs::read_to_string(p.ticket_path()).unwrap();
    assert!(after.starts_with(&before), "append-only\nbefore: {before}\nafter: {after}");
    assert!(after.contains("## Round 2"), "{after}");
}

#[test]
fn add_round_without_scope_fails_and_writes_nothing() {
    // spec Scenario「內容缺少 Scope」：非零、stderr 說明格式要求、工單不建立。
    let p = TempProject::with_change("addround-noscope", TASKS_DONE);
    let out = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], "- just prose\n");
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("**Scope**:"), "stderr explains: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists(), "refusal must not create the ticket");
}

#[test]
fn structured_sequence_allows_one_discovery_then_validation_only() {
    // spec Scenario「第二個 discovery 被拒絕」＋「追加 structured validation」＋
    // 「phase 與 patch 必須成對」。
    let p = TempProject::with_change("addround-seq", TASKS_DONE);
    let round = |phase: &str, hex: &str| {
        format!("**Phase**: {phase}\n**Patch**: sha256:{hex}\n**Scope**: src/lib.rs\n")
    };
    let a = "a".repeat(64);
    let b = "b".repeat(64);

    let unpaired = p.run_stdin(
        &["verify", "add-round", "demo", "--stdin"],
        "**Phase**: discovery\n**Scope**: src/lib.rs\n",
    );
    assert!(!unpaired.status.success(), "Phase without Patch must be rejected");
    assert!(!p.ticket_path().exists(), "refusal writes nothing");

    let first = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], &round("discovery", &a));
    assert!(first.status.success(), "stderr: {}", stderr_of(&first));
    let after_first = std::fs::read_to_string(p.ticket_path()).unwrap();

    let second = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], &round("discovery", &b));
    assert!(!second.status.success(), "a second discovery must be rejected");
    assert!(stderr_of(&second).contains("validation"), "{}", stderr_of(&second));
    assert_eq!(std::fs::read_to_string(p.ticket_path()).unwrap(), after_first, "ticket unchanged");

    let third = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], &round("validation", &b));
    assert!(third.status.success(), "stderr: {}", stderr_of(&third));
    assert!(std::fs::read_to_string(p.ticket_path()).unwrap().contains("## Round 2"));
}

// --- verify show ---

#[test]
fn show_prints_the_ticket_without_ansi_under_no_color() {
    let p = TempProject::with_change("show-human", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["--no-color", "verify", "show", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("Round 1"), "{text}");
    assert!(text.contains("src/lib.rs"), "{text}");
    assert!(!text.contains('\u{1b}'), "--no-color must strip ANSI: {text:?}");
}

#[test]
fn show_json_payload_carries_the_camel_case_contract() {
    // spec「驗證工單的讀取」：change／rounds[].index／phase／patchHash／scope／
    // findings[].severity|path|text／lastRound，欄位 camelCase 且型別固定。
    let p = TempProject::with_change("show-json", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["verify", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["change"], "demo");
    let rounds = v["rounds"].as_array().expect("rounds array");
    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0]["index"], 1);
    assert_eq!(rounds[0]["scope"], serde_json::json!(["src/lib.rs"]));
    let f = &rounds[0]["findings"][0];
    assert_eq!(f["severity"], "CRITICAL");
    assert_eq!(f["path"], "src/lib.rs");
    assert!(f["text"].is_string(), "{f}");
    assert_eq!(v["lastRound"]["index"], 2);
    assert_eq!(v["lastRound"]["findings"].as_array().map(Vec::len), Some(0));
}

#[test]
fn show_json_legacy_round_emits_explicit_nulls() {
    // spec Scenario「legacy JSON 使用 null」：不含 Phase／Patch 的輪次兩欄明確
    // 輸出 null（缺欄與 null 對讀取端是兩件事）。
    let p = TempProject::with_change("show-json-legacy", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["verify", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert!(v["rounds"][0]["phase"].is_null(), "{v}");
    assert!(v["rounds"][0]["patchHash"].is_null(), "{v}");
    assert!(v["lastRound"]["phase"].is_null(), "{v}");
}

#[test]
fn show_json_carries_nullable_phase_and_patch_hash() {
    // spec Scenario「讀取 JSON」：structured 工單的 lastRound.phase 為 validation
    // 且 patchHash 為 `sha256:` digest。
    let p = TempProject::with_change("show-json-structured", TASKS_DONE);
    let a = "a".repeat(64);
    let b = "b".repeat(64);
    p.run_stdin(
        &["verify", "add-round", "demo", "--stdin"],
        &format!("**Phase**: discovery\n**Patch**: sha256:{a}\n**Scope**: src/lib.rs\n"),
    );
    p.run_stdin(
        &["verify", "add-round", "demo", "--stdin"],
        &format!("**Phase**: validation\n**Patch**: sha256:{b}\n**Scope**: src/lib.rs\n"),
    );
    let out = p.run(&["verify", "show", "demo", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["rounds"][0]["phase"], "discovery");
    assert_eq!(v["lastRound"]["phase"], "validation");
    assert_eq!(v["lastRound"]["patchHash"], format!("sha256:{b}"));
}

#[test]
fn show_without_ticket_fails_semantically() {
    // spec Scenario「無工單」：非零、stderr 說明該 change 無「驗證」工單。
    let p = TempProject::with_change("show-none", TASKS_DONE);
    let out = p.run(&["verify", "show", "demo"]);
    assert!(!out.status.success());
    let err = stderr_of(&out);
    assert!(err.contains("no verify ticket"), "stderr names the station: {err}");
}

// --- verify stamp / discard ---

#[test]
fn stamp_refuses_findings_without_accept_then_accepts() {
    // spec Scenario「末輪有未解 findings 且未帶 --accept」＋`--accept` 豁免。
    let p = TempProject::with_change("stamp-findings", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let refused = p.run(&["verify", "stamp", "demo"]);
    assert!(!refused.status.success());
    let err = stderr_of(&refused);
    assert!(err.contains("--accept"), "stderr offers --accept: {err}");
    assert!(err.contains("re-verify"), "station word is the verify one: {err}");
    assert!(err.contains("1 outstanding must-fix"), "stderr names the must-fix count: {err}");
    assert!(err.contains("CRITICAL/WARNING"), "stderr names the blocking severities: {err}");
    assert!(p.ticket_path().exists(), "ticket survives the refusal");

    let ok = p.run(&["verify", "stamp", "demo", "--accept"]);
    assert!(ok.status.success(), "stderr: {}", stderr_of(&ok));
    assert!(!p.ticket_path().exists(), "ticket deleted by the stamp");
    assert!(p.meta().contains("verified_at"), "{}", p.meta());
}

#[test]
fn stamp_allows_a_suggestion_only_round() {
    // spec Scenario「僅 SUGGESTION 的末輪乾淨蓋章」：SUGGESTION 不是必修，
    // 無 --accept 也放行——exit 0、五欄寫入、工單刪除。
    let p = TempProject::with_change("stamp-suggestion", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], SUGGESTION_ROUND);
    let out = p.run(&["verify", "stamp", "demo", "--agent", "claude"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists(), "ticket deleted by the stamp");
    let meta = p.meta();
    for key in ["verified_at:", "verified_tasks_total:", "verified_scope:", "verified_with:"] {
        assert!(meta.contains(key), "missing {key}: {meta}");
    }
}

#[test]
fn stamp_refuses_incomplete_tasks() {
    // 守門 (1)：工單開立後任務被退勾 → 非零，訊息用 verify 的動詞。
    let p = TempProject::with_change("stamp-partial", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    p.write("openspec/changes/demo/tasks.md", TASKS_PARTIAL);
    let out = p.run(&["verify", "stamp", "demo"]);
    assert!(!out.status.success());
    let err = stderr_of(&out);
    assert!(err.contains("1/2"), "{err}");
    assert!(err.contains("verify stamp"), "names the verb: {err}");
    assert!(!p.meta().contains("verified_at"), "{}", p.meta());
}

#[test]
fn stamp_clean_round_writes_the_anchors_and_deletes_the_ticket() {
    // spec Scenario「乾淨蓋章」：五個 verified 欄位齊備、工單不存在。
    let p = TempProject::with_change("stamp-clean", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["verify", "stamp", "demo", "--agent", "claude"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists());
    let meta = p.meta();
    for key in ["verified_at:", "verified_tasks_total:", "verified_scope:", "verified_with:"] {
        assert!(meta.contains(key), "missing {key}: {meta}");
    }
    assert!(meta.contains("verified_tasks_total: 2"), "{meta}");
}

#[test]
fn discard_removes_the_ticket_and_leaves_metadata_alone() {
    // spec Scenario「放棄既有工單」：exit 0、verify.md 不存在、meta 不變。
    let p = TempProject::with_change("discard", TASKS_DONE);
    let before = p.meta();
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["verify", "discard", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.ticket_path().exists());
    assert_eq!(p.meta(), before, "metadata untouched");

    let again = p.run(&["verify", "discard", "demo"]);
    assert!(!again.status.success(), "no ticket → non-zero");
}

// --- archive 守門與 --carry-verify ---

#[test]
fn archive_refuses_an_open_verify_ticket_with_three_disposals() {
    // spec Scenario「僅驗證工單時拒絕」：stderr 同列 stamp／discard／--carry-verify。
    let p = TempProject::with_change("archive-gate", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["archive", "demo", "--skip-specs", "--no-validate"]);
    assert!(!out.status.success(), "open ticket must refuse archive");
    let err = stderr_of(&out);
    for needle in ["verify stamp", "verify discard", "--carry-verify"] {
        assert!(err.contains(needle), "missing {needle}: {err}");
    }
    assert!(p.ticket_path().exists(), "change stays in place");
}

#[test]
fn archive_lists_both_stations_when_both_tickets_are_open() {
    // spec Scenario「雙工單並存」：兩組處置並列。
    let p = TempProject::with_change("archive-both", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["archive", "demo", "--skip-specs", "--no-validate"]);
    assert!(!out.status.success());
    let err = stderr_of(&out);
    for needle in ["--carry-review", "--carry-verify", "review stamp", "verify stamp"] {
        assert!(err.contains(needle), "missing {needle}: {err}");
    }
}

#[test]
fn archive_carry_verify_moves_the_ticket_into_the_archive() {
    // spec Scenario「明示帶走驗證工單」：封存成功且封存目錄內含 verify.md。
    let p = TempProject::with_change("archive-carry", TASKS_DONE);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["archive", "demo", "--skip-specs", "--no-validate", "--carry-verify"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let archive_root = p.dir.join("openspec").join("changes").join("archive");
    let dated = std::fs::read_dir(&archive_root)
        .expect("archive dir")
        .next()
        .expect("one archived change")
        .unwrap()
        .path();
    assert!(dated.join("verify.md").is_file(), "fossil ticket travels: {dated:?}");
}

#[test]
fn archive_carries_both_tickets_when_both_flags_ride_along() {
    // spec「`--carry-review` 與 `--carry-verify` 可同時帶」。
    let p = TempProject::with_change("archive-carry-both", TASKS_DONE);
    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&[
        "archive",
        "demo",
        "--skip-specs",
        "--no-validate",
        "--carry-review",
        "--carry-verify",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let archive_root = p.dir.join("openspec").join("changes").join("archive");
    let dated =
        std::fs::read_dir(&archive_root).unwrap().next().unwrap().unwrap().path();
    assert!(dated.join("review.md").is_file(), "{dated:?}");
    assert!(dated.join("verify.md").is_file(), "{dated:?}");
}

// --- verify scope（design D8：共用 resolver、站別 snapshot namespace）---

fn scope_fixture(tag: &str) -> TempProject {
    let p = TempProject::with_change(tag, TASKS_DONE);
    let wide: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    p.write("src/util.rs", &wide);
    p.git(&["init", "-q"]);
    p.git(&["config", "user.name", "Sandbox Tester"]);
    p.git(&["config", "user.email", "sandbox@example.com"]);
    p.git(&["add", "-A"]);
    p.git(&["commit", "-q", "-m", "init"]);
    // Apply baseline 由審查站的 prepare 建立，兩站共用（design D8）。
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
fn verify_scope_discovery_json_payload_is_camel_case_with_hunk_ranges() {
    // spec Scenario「discovery scope 復用 Host resolver」：phase=discovery、
    // state=resolved、hunks 帶 old/new ranges，verify-snapshots 新增對應檔。
    let p = scope_fixture("scope-json");
    let out = p.run(&["verify", "scope", "demo", "--json"]);
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
    for f in files {
        for h in f["hunks"].as_array().unwrap() {
            for k in ["oldStart", "oldLines", "newStart", "newLines"] {
                assert!(h[k].is_number(), "range {k} is a number: {h}");
            }
        }
    }
    assert_eq!(p.snapshot_count("verify-snapshots"), 1, "the verify snapshot is written");
    assert_eq!(p.snapshot_count("snapshots"), 0, "the review namespace stays empty");
}

#[test]
fn verify_scope_human_output_has_no_ansi_under_no_color() {
    let p = scope_fixture("scope-human");
    let out = p.run(&["--no-color", "verify", "scope", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("discovery"), "{text}");
    assert!(text.contains("sha256:"), "{text}");
    assert!(!text.contains('\u{1b}'), "--no-color must strip ANSI: {text:?}");
}

#[test]
fn verify_scope_validation_only_freezes_the_remediation_patch() {
    // spec Scenario「validation 只凍結修正 patch」：兩檔 discovery 後只改一檔，
    // 續輪 phase=validation 且 patch 不含未修改的另一檔。
    let p = scope_fixture("scope-validation");
    let first = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(first.status.success(), "stderr: {}", stderr_of(&first));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&first)).unwrap();
    let patch_hash = v["patchHash"].as_str().unwrap().to_string();
    let round = format!(
        "**Phase**: discovery\n**Patch**: {patch_hash}\n**Scope**: src/lib.rs, src/util.rs\n\n\
         - [CRITICAL] src/lib.rs — requirement R2 has no implementation\n"
    );
    let added = p.run_stdin(&["verify", "add-round", "demo", "--stdin"], &round);
    assert!(added.status.success(), "stderr: {}", stderr_of(&added));

    p.write("src/lib.rs", "fn demo() {}\nfn added() {}\nfn fixed() {}\n");
    let second = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(second.status.success(), "stderr: {}", stderr_of(&second));
    let v2: serde_json::Value = serde_json::from_str(&stdout_of(&second)).unwrap();
    assert_eq!(v2["phase"], "validation");
    assert_eq!(v2["paths"], serde_json::json!(["src/lib.rs"]), "only the fixed file: {v2}");
    assert!(v2["patch"].as_str().unwrap().contains("+fn fixed() {}"), "{v2}");
    assert!(!v2["patch"].as_str().unwrap().contains("line 2 edited"), "{v2}");
}

#[test]
fn verify_scope_missing_snapshot_fails_closed_without_falling_back() {
    // spec Scenario「snapshot 缺失不退回 discovery」：referenced snapshot 被移除 →
    // 非零、工單不變、不得用 touched 整檔建立新 discovery snapshot。
    let p = scope_fixture("scope-nosnap");
    let first = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(first.status.success(), "stderr: {}", stderr_of(&first));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&first)).unwrap();
    let patch_hash = v["patchHash"].as_str().unwrap().to_string();
    let round = format!(
        "**Phase**: discovery\n**Patch**: {patch_hash}\n**Scope**: src/lib.rs\n\n\
         - [CRITICAL] src/lib.rs — requirement R2 has no implementation\n"
    );
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], &round);
    let ticket_before = std::fs::read_to_string(p.ticket_path()).unwrap();
    std::fs::remove_dir_all(p.scope_dir().join("verify-snapshots")).unwrap();

    let out = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "missing snapshot must fail closed");
    assert_eq!(p.snapshot_count("verify-snapshots"), 0, "no snapshot may be written");
    assert_eq!(std::fs::read_to_string(p.ticket_path()).unwrap(), ticket_before, "ticket intact");
}

#[test]
fn verify_scope_needs_input_when_the_baseline_is_missing() {
    // spec：baseline 缺失 → JSON 路徑印 state=needsInput 後非零，零 snapshot 效果，
    // stderr 列可用處置。payload shape 與 `review scope` 同構。
    let p = TempProject::with_git_change("scope-nobaseline", TASKS_DONE);
    p.write("src/lib.rs", "fn demo() {}\nfn added() {}\n");
    p.touched("demo", &["src/lib.rs"]);
    let out = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "ambiguous scope must be non-zero");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "needsInput");
    assert_eq!(v["phase"], "discovery");
    assert!(v["ambiguousPaths"].is_array(), "{v}");
    assert!(v["files"].is_array(), "{v}");
    let err = stderr_of(&out);
    assert!(err.contains("verify scope for 'demo' needs input"), "station-worded: {err}");
    assert!(err.contains("--base"), "disposal named: {err}");
    assert_eq!(p.snapshot_count("verify-snapshots"), 0, "zero snapshot effects");
}

#[test]
fn verify_scope_dirty_at_start_and_active_overlap_fail_closed() {
    // spec：dirty-at-start 與其他 active change 的 touched 認領重疊 → needsInput。
    let dirty = scope_fixture("scope-dirty");
    // baseline 已記錄乾淨起點；把 touched 檔宣告為起始即髒需要重錄 baseline，
    // 故改以另一個 active change 認領同一路徑觸發 overlap。
    let other = dirty.dir.join("openspec").join("changes").join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join(".openspec.yaml"), "schema: spec-driven\ncreated: 2026-07-01\n")
        .unwrap();
    std::fs::write(other.join("tasks.md"), "- [ ] 1.1 x\n").unwrap();
    dirty.touched("other", &["src/lib.rs"]);
    let out = dirty.run(&["verify", "scope", "demo", "--json"]);
    assert!(!out.status.success(), "overlapping claim must fail closed");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid JSON");
    assert_eq!(v["state"], "needsInput");
    assert!(stderr_of(&out).contains("other"), "names the claiming change: {}", stderr_of(&out));
    assert_eq!(dirty.snapshot_count("verify-snapshots"), 0, "zero snapshot effects");
}

#[test]
fn verify_scope_hash_pinned_selection_rejects_candidate_drift() {
    // spec：hash-pinned selection 必須帶正確的 candidate hash，漂移即拒；
    // 選定 hunk 後只凍結所選文字段。
    let p = scope_fixture("scope-pinned");
    let probe = p.run(&["verify", "scope", "demo", "--json"]);
    assert!(probe.status.success(), "stderr: {}", stderr_of(&probe));
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&probe)).unwrap();
    let candidate = v["candidateHash"].as_str().unwrap().to_string();
    let hunk = v["files"][0]["hunks"][0]["id"].as_str().unwrap().to_string();

    let stale = format!("sha256:{}", "0".repeat(64));
    let drifted = p.run(&[
        "verify",
        "scope",
        "demo",
        "--json",
        "--candidate-hash",
        &stale,
        "--include-hunk",
        &hunk,
    ]);
    assert!(!drifted.status.success(), "a stale candidate hash must be rejected");

    let pinned = p.run(&[
        "verify",
        "scope",
        "demo",
        "--json",
        "--candidate-hash",
        &candidate,
        "--include-hunk",
        &hunk,
    ]);
    assert!(pinned.status.success(), "stderr: {}", stderr_of(&pinned));
    let v2: serde_json::Value = serde_json::from_str(&stdout_of(&pinned)).unwrap();
    assert_eq!(v2["state"], "resolved");
    assert_ne!(v2["patchHash"], v2["candidateHash"], "a narrowed selection re-hashes: {v2}");
}

#[test]
fn verify_scope_rejects_include_hunk_without_a_candidate_hash() {
    // audit（sharp edges）：選擇參數缺錨即拒，不得靜默改跑全候選。
    let p = scope_fixture("scope-flagguard");
    let out = p.run(&["verify", "scope", "demo", "--json", "--include-hunk", "h1"]);
    assert!(!out.status.success(), "selection without an anchor must fail loudly");
}

// --- 兩站 snapshot cleanup 隔離（design D8）---

#[test]
fn verify_stamp_clears_only_verify_snapshots() {
    // spec Scenario「兩站 snapshot 清理互不影響」：verify stamp 只清 verify
    // snapshots，Apply baseline 與 review snapshots 位元級不變。
    let p = TempProject::with_git_change("cleanup-stamp", TASKS_DONE);
    assert!(p.run(&["review", "prepare", "demo"]).status.success());
    let review_dir = p.scope_dir().join("snapshots");
    let verify_dir = p.scope_dir().join("verify-snapshots");
    std::fs::create_dir_all(&review_dir).unwrap();
    std::fs::create_dir_all(&verify_dir).unwrap();
    std::fs::write(review_dir.join(format!("{}.json", "a".repeat(64))), "{\"r\":1}").unwrap();
    std::fs::write(verify_dir.join(format!("{}.json", "b".repeat(64))), "{\"v\":1}").unwrap();
    let baseline_before = std::fs::read_to_string(p.baseline_path()).unwrap();

    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["verify", "stamp", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.snapshot_count("verify-snapshots"), 0, "verify snapshots cleared");
    assert_eq!(p.snapshot_count("snapshots"), 1, "review snapshots untouched");
    assert_eq!(std::fs::read_to_string(p.baseline_path()).unwrap(), baseline_before);
}

#[test]
fn review_stamp_leaves_verify_snapshots_alone() {
    // 反向：審查站清理不得帶走驗證站的續輪依據。
    let p = TempProject::with_git_change("cleanup-review", TASKS_DONE);
    assert!(p.run(&["review", "prepare", "demo"]).status.success());
    let review_dir = p.scope_dir().join("snapshots");
    let verify_dir = p.scope_dir().join("verify-snapshots");
    std::fs::create_dir_all(&review_dir).unwrap();
    std::fs::create_dir_all(&verify_dir).unwrap();
    std::fs::write(review_dir.join(format!("{}.json", "a".repeat(64))), "{\"r\":1}").unwrap();
    std::fs::write(verify_dir.join(format!("{}.json", "b".repeat(64))), "{\"v\":1}").unwrap();

    p.run_stdin(&["review", "add-round", "demo", "--stdin"], CLEAN_ROUND);
    let out = p.run(&["review", "stamp", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.snapshot_count("snapshots"), 0, "review snapshots cleared");
    assert_eq!(p.snapshot_count("verify-snapshots"), 1, "verify snapshots untouched");
}

#[test]
fn verify_discard_clears_only_verify_snapshots_and_keeps_the_review_ticket() {
    // spec「`verify discard` 只清除 verify snapshots」＋兩站工單互不遮蔽。
    let p = TempProject::with_git_change("cleanup-discard", TASKS_DONE);
    assert!(p.run(&["review", "prepare", "demo"]).status.success());
    let review_dir = p.scope_dir().join("snapshots");
    let verify_dir = p.scope_dir().join("verify-snapshots");
    std::fs::create_dir_all(&review_dir).unwrap();
    std::fs::create_dir_all(&verify_dir).unwrap();
    std::fs::write(review_dir.join(format!("{}.json", "a".repeat(64))), "{\"r\":1}").unwrap();
    std::fs::write(verify_dir.join(format!("{}.json", "b".repeat(64))), "{\"v\":1}").unwrap();

    p.run_stdin(&["review", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    p.run_stdin(&["verify", "add-round", "demo", "--stdin"], ROUND_WITH_FINDINGS);
    let out = p.run(&["verify", "discard", "demo"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.snapshot_count("verify-snapshots"), 0, "verify snapshots cleared");
    assert_eq!(p.snapshot_count("snapshots"), 1, "review snapshots untouched");
    assert!(p.baseline_path().exists(), "Apply baseline survives");
    assert!(p.review_ticket_path().exists(), "the review ticket survives");
}

//! `speclink validate --specs` 的正典規格驗證 — fs 模式契約
//! (spec spec-validation「validate --specs 驗證正典規格」)。
//!
//! 缺 `## Purpose` 區段或內容為空以 error 呈現並非零收尾；殘留的 archive 佔位
//! 以 warning 顯形（不依附 --strict）；內容過短只在 --strict 下報 warning。
//! `--specs` 單獨傳入時不驗 changes，兩旗標皆缺席時行為與接線前一致。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";
/// archive 佔位文案：長度恆超過門檻，只有前綴判準抓得到它。
const PLACEHOLDER_PURPOSE: &str =
    "TBD - created by archiving change 'old-one'. Update Purpose after archive.";
const GOOD_PURPOSE: &str =
    "本 capability 負責搜尋結果的排序與分頁，涵蓋查詢改寫、排序權重與空結果的可觀察行為。";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 一個 change `demo`（供「只驗規格」對照）＋呼叫端指定的正典規格。
    fn new(tag: &str, specs: &[(&str, Option<&str>)]) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-validate-specs-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 a\n").unwrap();
        for (cap, purpose) in specs {
            let dir_cap = dir.join("openspec").join("specs").join(cap);
            std::fs::create_dir_all(&dir_cap).unwrap();
            let body = match purpose {
                Some(p) => format!("# {cap} Specification\n\n## Purpose\n\n{p}\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n"),
                None => format!("# {cap} Specification\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n"),
            };
            std::fs::write(dir_cap.join("spec.md"), body).unwrap();
        }
        TempProject { dir }
    }

    /// 寫入 change `demo` 的一份 delta spec（守門案例用）。
    fn put_delta(&self, cap: &str, text: &str) {
        let dir =
            self.dir.join("openspec").join("changes").join("demo").join("specs").join(cap);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("spec.md"), text).unwrap();
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
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn placeholder_purpose_surfaces_as_a_warning_without_strict() {
    // spec Scenario「佔位 Purpose 以 warning 顯形」：非 strict 也報，且佔位句
    // 長於門檻——攔下它的是前綴判準，不是長度。
    let p = TempProject::new("placeholder", &[("search", Some(PLACEHOLDER_PURPOSE))]);
    let out = p.run(&["validate", "--specs", "--no-color"]);
    let stdout = stdout_of(&out);
    assert!(out.status.success(), "佔位只是 warning，不得非零收尾: {stdout}");
    assert!(stdout.contains("search"), "點名該規格: {stdout}");
    assert!(stdout.contains("warn:"), "以 warning 呈現: {stdout}");
}

#[test]
fn a_short_purpose_is_silent_until_strict() {
    // spec Scenario「過短 Purpose 僅 strict 報 warning」。
    let p = TempProject::new("short", &[("search", Some("太短。"))]);
    let lenient = p.run(&["validate", "--specs", "--no-color"]);
    assert!(lenient.status.success(), "非 strict 零收尾");
    assert!(!stdout_of(&lenient).contains("warn:"), "非 strict 不報過短: {}", stdout_of(&lenient));

    let strict = p.run(&["validate", "--specs", "--strict", "--no-color"]);
    let stdout = stdout_of(&strict);
    assert!(strict.status.success(), "過短是 warning，仍零收尾: {stdout}");
    assert!(stdout.contains("warn:") && stdout.contains("50"), "strict 報門檻: {stdout}");
}

#[test]
fn a_missing_purpose_section_fails_the_command() {
    // spec Scenario「缺 Purpose 區段報 error」：該規格 invalid、命令非零收尾。
    let p = TempProject::new("missing", &[("search", None), ("net", Some(GOOD_PURPOSE))]);
    let out = p.run(&["validate", "--specs", "--no-color"]);
    let stdout = stdout_of(&out);
    assert!(!out.status.success(), "任一 error 非零收尾: {stdout}");
    assert!(stdout.contains("✗ search — invalid"), "缺段的規格 invalid: {stdout}");
    assert!(stdout.contains("✓ net — valid"), "合格的規格照常通過: {stdout}");
    assert!(stdout.contains("specs/search/spec.md"), "error 帶邏輯路徑: {stdout}");
}

#[test]
fn specs_flag_with_an_item_name_is_rejected() {
    // spec Scenario「--specs 與 change 名稱同傳被拒」：組合語意不連貫（--specs
    // 無法指定單一規格），大聲拒絕並指路 --specs 單獨或 --all。
    let p = TempProject::new("conflict", &[("search", Some(GOOD_PURPOSE))]);
    let out = p.run(&["validate", "demo", "--specs", "--no-color"]);
    assert!(!out.status.success(), "the combination must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("--specs"), "the flag is named: {stderr}");
    assert!(stderr.contains("--all"), "the way out is named: {stderr}");
}

#[test]
fn specs_flag_alone_does_not_validate_changes() {
    // spec：`--specs` 單獨傳入時僅驗規格；預設（皆缺席）維持只驗 changes。
    let p = TempProject::new("scope", &[("search", Some(GOOD_PURPOSE))]);
    let specs_only = stdout_of(&p.run(&["validate", "--specs", "--no-color"]));
    assert!(specs_only.contains("search"), "規格有驗: {specs_only}");
    assert!(!specs_only.contains("demo"), "change 不該出現: {specs_only}");

    let default = stdout_of(&p.run(&["validate", "--no-color"]));
    assert!(default.contains("demo"), "預設驗 change: {default}");
    assert!(!default.contains("search"), "預設不驗規格: {default}");

    let all = stdout_of(&p.run(&["validate", "--all", "--no-color"]));
    assert!(all.contains("demo") && all.contains("search"), "--all 兩邊都驗: {all}");
}

// --- change 驗證納入合併守門（spec spec-validation「change 驗證納入合併守門」）---

/// 正典只有 `R1`，delta 卻 MODIFIED 一個已不存在的 `R9`——archive 會拒收的形狀。
const STALE_MODIFIED: &str = "## MODIFIED Requirements\n\n### Requirement: R9\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
/// 守門 error 的凍結組字（design D2）。
const GATE_ERROR: &str = "specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift demo)";

#[test]
fn a_stale_modified_delta_fails_change_validation() {
    // spec Scenario「MODIFIED 目標不存在時 validate 報 error」：人眼與 --json
    // 兩條輸出都是契約，兩邊都驗。
    let p = TempProject::new("gate", &[("auth", Some(GOOD_PURPOSE))]);
    p.put_delta("auth", STALE_MODIFIED);

    let out = p.run(&["validate", "demo", "--no-color"]);
    let stdout = stdout_of(&out);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success(), "守門違規非零收尾: {stdout}");
    assert!(stdout.contains("✗ demo — invalid"), "change 列為 invalid: {stdout}");
    assert!(stdout.contains(GATE_ERROR), "error 行逐字列出: {stdout}");
    assert!(stderr.contains("Validation failed."), "以 Validation failed. 收尾: {stderr}");

    let out = p.run(&["validate", "demo", "--json"]);
    let stdout = stdout_of(&out);
    assert!(!out.status.success(), "--json 同樣非零收尾: {stdout}");
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is JSON");
    let first = &v[0];
    for field in ["change", "errors", "valid", "warnings"] {
        assert!(!first[field].is_null(), "四欄齊備，缺 {field}: {v}");
    }
    assert_eq!(first["valid"], serde_json::json!(false), "valid 為 false: {v}");
    assert!(
        first["errors"].as_array().expect("errors 是陣列").iter().any(|e| e == GATE_ERROR),
        "errors 含守門字串: {v}"
    );
}

#[test]
fn validate_and_drift_name_the_same_stale_operation() {
    // spec archive-merge「過期判定單源共用」：validate 與 drift 讀同一支判定，
    // 對同一 delta 指向同一 capability 與需求名。
    let p = TempProject::new("gate-drift", &[("auth", Some(GOOD_PURPOSE))]);
    p.put_delta("auth", STALE_MODIFIED);

    let validated = stdout_of(&p.run(&["validate", "demo", "--no-color"]));
    assert!(validated.contains(GATE_ERROR), "validate 報守門 error: {validated}");

    let drifted = stdout_of(&p.run(&["drift", "demo", "--no-color"]));
    assert!(
        drifted.contains("MODIFIED 'R9' in auth"),
        "drift 的 spec assumption 指向同一 capability 與需求名: {drifted}"
    );
    assert!(
        drifted.contains("target requirement no longer exists in the canonical spec"),
        "reason 同一份字串: {drifted}"
    );
}

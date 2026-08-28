//! `speclink new artifact spec` 的 capability 命名主閘 — fs 模式契約
//! (spec capability-naming-guard「建立點主閘」「--new 旗標顯性宣告新 capability」)。
//!
//! 正典未收錄的名稱未帶 `--new` 即拒絕：非零收尾、stderr 含近似建議與兩條
//! 指引、不落盤；帶 `--new` 照現行流程建立；命中正典名稱時輸出位元級不變；
//! `--json` 成功 payload 形狀凍結，主閘拒絕時 stdout 無成功 payload。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const META: &str = "schema: spec-driven\ncreated: 2026-08-20\n";
const DELTA: &str = "## Purpose\n\n測試用 delta。\n\n## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 一個 change `demo` ＋正典規格 `auth`。
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-new-artifact-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // macOS 的 temp dir 是 /var → /private/var 的 symlink；Windows 的
        // canonicalize 會加 \\?\ 前綴，跳過。
        let dir = if cfg!(windows) { dir } else { dir.canonicalize().unwrap() };
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        let change = dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        let auth = dir.join("openspec").join("specs").join("auth");
        std::fs::create_dir_all(&auth).unwrap();
        std::fs::write(
            auth.join("spec.md"),
            "# auth Specification\n\n## Purpose\n\nAuth session lifecycle.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n",
        )
        .unwrap();
        TempProject { dir }
    }

    /// 跑 `speclink new artifact spec <cap> --change demo --stdin [extra...]`，
    /// stdin 餵一份合法 delta。
    fn new_spec(&self, cap: &str, extra: &[&str]) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(["new", "artifact", "spec", cap, "--change", "demo", "--stdin", "--no-color"])
            .args(extra)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child.stdin.take().unwrap().write_all(DELTA.as_bytes()).unwrap();
        child.wait_with_output().expect("run speclink binary")
    }

    fn delta_dir(&self, cap: &str) -> PathBuf {
        self.dir.join("openspec").join("changes").join("demo").join("specs").join(cap)
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn an_unlisted_capability_without_new_refuses_with_suggestions() {
    // spec Scenario「未收錄名稱未帶 --new 遭拒且不落盤」。
    let p = TempProject::new("refuse");
    let out = p.new_spec("authentication", &[]);
    assert!(!out.status.success(), "unlisted capability must exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("authentication"), "點名候選: {stderr}");
    assert!(stderr.contains("auth"), "近似建議: {stderr}");
    assert!(stderr.contains("Auth session lifecycle."), "建議附 Purpose 首行: {stderr}");
    assert!(stderr.contains("exact name"), "指引一：沿用既有名: {stderr}");
    assert!(stderr.contains("--new"), "指引二：帶 --new 重跑: {stderr}");
    assert!(!stderr.contains("\u{1b}["), "--no-color 下無色彩控制碼: {stderr:?}");
    assert!(!p.delta_dir("authentication").exists(), "拒絕不落盤");
}

#[test]
fn the_new_flag_creates_the_declared_capability() {
    // spec Scenario「帶 --new 建立新 capability 成功」。
    let p = TempProject::new("declare");
    let out = p.new_spec("token-rotation", &["--new"]);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "--new declares the capability: {stderr}");
    assert!(p.delta_dir("token-rotation").join("spec.md").exists(), "delta spec created");
}

#[test]
fn a_canonical_capability_keeps_the_exact_success_output() {
    // spec Scenario「命中正典名稱照常放行」：輸出與導入前位元級一致。
    let p = TempProject::new("canonical");
    let out = p.new_spec("auth", &[]);
    assert!(out.status.success(), "canonical name passes");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let path = p.delta_dir("auth").join("spec.md");
    assert_eq!(
        stdout,
        format!("✓ Created spec: {}\n  Content validated ✓\n", path.to_string_lossy()),
        "成功輸出逐位元不變"
    );
}

#[test]
fn json_success_payload_keeps_its_frozen_shape() {
    // spec Scenario「帶 --json 的成功與拒絕路徑」：成功 payload 欄位形狀凍結。
    let p = TempProject::new("json-ok");
    let out = p.new_spec("auth", &["--json"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("single-line JSON");
    let obj = v.as_object().expect("object payload");
    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        ["artifact", "change", "path", "status", "validated", "warnings"],
        "camelCase 欄位集合不變: {stdout}"
    );
    assert_eq!(obj["artifact"], "spec");
    assert_eq!(obj["change"], "demo");
    assert_eq!(obj["status"], "created");
    assert_eq!(obj["validated"], true);
}

#[test]
fn json_refusal_emits_no_success_payload() {
    // spec Scenario「帶 --json 的成功與拒絕路徑」：拒絕時非零收尾、stdout
    // 無成功 payload。
    let p = TempProject::new("json-refuse");
    let out = p.new_spec("authentication", &["--json"]);
    assert!(!out.status.success(), "refusal exits non-zero under --json too");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(!stdout.contains("\"status\""), "stdout 無成功 payload: {stdout}");
}

//! `speclink change schema` — fs 模式契約（workflow-schemas「change schema 動詞切換
//! 變更的 schema」與「schema 沒有 specs 產出時拒絕寫入 delta spec」）。
//!
//! 每個拒絕都斷言 `.meta.yaml` 逐位元不變；worktree 映射的情境在
//! worktree_overlay.rs（要真實 git repo）。user 層 schema 位置導進暫存專案，
//! 開發者的 home 不影響結果。

use std::path::PathBuf;
use std::process::{Command, Output};

const SPEC_META: &str = "schema: spec-driven\ncreated: 2026-10-01\n";
const NO_SPEC_META: &str = "schema: no-spec-change\ncreated: 2026-10-01\n";
const DELTA: &str = "## ADDED Requirements\n\n### Requirement: Login\n\nIt SHALL log in.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
const CANON_AUTH: &str = "# auth Specification\n\n## Purpose\n\nAuth session lifecycle used by the change schema tests.\n\n## Requirements\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-change-schema-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("speclink").join("specs").join("auth")).unwrap();
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::write(dir.join("speclink").join("specs").join("auth").join("spec.md"), CANON_AUTH)
            .unwrap();
        TempProject { dir }
    }

    /// 一個作用中變更：`.meta.yaml` 與一份 proposal。
    fn change(&self, name: &str, meta: &str) -> PathBuf {
        let dir = self.dir.join("speclink").join("changes").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".meta.yaml"), meta).unwrap();
        std::fs::write(dir.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        dir
    }

    fn meta(&self, name: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("speclink").join("changes").join(name).join(".meta.yaml"),
        )
        .unwrap()
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_stdin(args, None)
    }

    fn run_stdin(&self, args: &[&str], stdin: Option<&str>) -> Output {
        use std::io::Write;
        let mut child = Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env("NO_COLOR", "1")
            .env("HOME", self.dir.join("home"))
            .env("USERPROFILE", self.dir.join("home"))
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("run speclink binary");
        if let Some(text) = stdin {
            child.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
        }
        drop(child.stdin.take());
        child.wait_with_output().unwrap()
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

#[test]
fn change_schema_switches_to_no_spec_change() {
    // Scenario「切到不改規格」。
    let p = TempProject::new("switch");
    p.change("tidy", SPEC_META);
    let out = p.run(&["change", "schema", "tidy", "no-spec-change"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ tidy schema: spec-driven → no-spec-change\n");
    assert_eq!(p.meta("tidy"), NO_SPEC_META, "only the schema line changes");
}

#[test]
fn change_schema_json_reports_the_switch() {
    // Scenario「切回 spec-driven 並輸出 JSON」。
    let p = TempProject::new("json");
    p.change("tidy", NO_SPEC_META);
    let out = p.run(&["change", "schema", "tidy", "spec-driven", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json stdout");
    assert_eq!(
        json,
        serde_json::json!({"change": "tidy", "schema": "spec-driven", "previous": "no-spec-change", "changed": true})
    );
}

#[test]
fn change_schema_refusals_exit_non_zero_and_leave_the_meta_untouched() {
    // Scenario「有 delta 時拒絕切到不改規格」「解析不到的 schema 拒絕」「未結驗證工單時拒絕」。
    let p = TempProject::new("refusals");
    let c = p.change("c", SPEC_META);
    std::fs::create_dir_all(c.join("specs").join("auth")).unwrap();
    std::fs::write(c.join("specs").join("auth").join("spec.md"), DELTA).unwrap();
    let out = p.run(&["change", "schema", "c", "no-spec-change"]);
    assert!(!out.status.success());
    // 逐字：server 的 409 message（plan_api.rs）與這一行是同一句引擎原文。
    assert_eq!(
        stderr_of(&out),
        "Error: cannot switch 'c' to 'no-spec-change': 'no-spec-change' has no specs, but the change \
         has delta specs (auth) — remove them first\n"
    );
    assert_eq!(p.meta("c"), SPEC_META);

    let out = p.run(&["change", "schema", "c", "no-such-flow"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr_of(&out),
        "Error: Schema not found: Schema 'no-such-flow' not found in project, user, or built-in locations\n"
    );
    assert_eq!(p.meta("c"), SPEC_META);

    // Scenario「不合法的 schema 名稱在解析前拒絕」。
    for bad in ["../evil", "Spec-Driven"] {
        let out = p.run(&["change", "schema", "c", bad]);
        assert!(!out.status.success(), "{bad}");
        assert_eq!(
            stderr_of(&out),
            format!("Error: Invalid schema name '{bad}': must be lowercase kebab-case (e.g. my-flow)\n")
        );
        assert_eq!(p.meta("c"), SPEC_META);
    }

    std::fs::write(c.join("verify.md"), "# Verify\n").unwrap();
    let out = p.run(&["change", "schema", "c", "spec-driven"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("speclink verify discard c"), "{}", stderr_of(&out));
    assert_eq!(p.meta("c"), SPEC_META);
    assert_eq!(std::fs::read_to_string(c.join("verify.md")).unwrap(), "# Verify\n", "ticket stays");
}

#[test]
fn change_schema_clears_the_verify_stamp_and_says_so() {
    // Scenario「切換時移除驗證章、保留審查章與輪數」。
    let p = TempProject::new("clear-stamp");
    let stamps = "reviewed_at: 2026-10-02T10:00:00+08:00\nreviewed_rounds: 1\n\
verified_at: 2026-10-02T11:00:00+08:00\nverified_by: V <v@example.com>\nverified_with: claude\n\
verified_tasks_total: 3\nverified_rounds: 2\nverified_scope:\n  - path: src/a.rs\n    hash: bb\n";
    p.change("c", &format!("{NO_SPEC_META}{stamps}"));
    let out = p.run(&["change", "schema", "c", "spec-driven"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(
        stdout_of(&out),
        "✓ c schema: no-spec-change → spec-driven\n  verify stamp cleared — re-run verify under spec-driven\n"
    );
    assert_eq!(
        p.meta("c"),
        format!("{SPEC_META}reviewed_at: 2026-10-02T10:00:00+08:00\nreviewed_rounds: 1\nverified_rounds: 2\n")
    );
}

#[test]
fn change_schema_same_value_is_unchanged() {
    // Scenario「同一個 schema 不寫檔」。
    let p = TempProject::new("same");
    p.change("c", SPEC_META);
    let out = p.run(&["change", "schema", "c", "spec-driven"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c schema: spec-driven (unchanged)\n");
    assert_eq!(p.meta("c"), SPEC_META);
}

#[test]
fn change_schema_unknown_change_is_an_error() {
    // Scenario「變更不存在」。
    let p = TempProject::new("ghost");
    p.change("c", SPEC_META);
    let out = p.run(&["change", "schema", "ghost", "spec-driven"]);
    assert!(!out.status.success());
    assert!(stderr_of(&out).contains("ghost"), "{}", stderr_of(&out));
    assert!(!p.dir.join("speclink").join("changes").join("ghost").exists(), "nothing written");
    assert_eq!(p.meta("c"), SPEC_META);
}

#[test]
fn change_schema_refuses_a_change_folder_without_metadata() {
    // 名冊列得出目錄、卻沒有 `.meta.yaml`：拒絕且不憑空建出 meta。
    let p = TempProject::new("no-meta");
    let dir = p.dir.join("speclink").join("changes").join("bare");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
    let out = p.run(&["change", "schema", "bare", "no-spec-change"]);
    assert!(!out.status.success());
    assert_eq!(
        stderr_of(&out),
        "Error: cannot write schema for 'bare': speclink/changes/bare/.meta.yaml is missing\n"
    );
    assert!(!dir.join(".meta.yaml").exists(), "nothing written");
}

#[test]
fn change_schema_gate_on_new_artifact_spec() {
    // Scenario「不改規格變更寫 delta 被拒」與「spec-driven 變更照常寫入」。
    let p = TempProject::new("new-spec");
    let tidy = p.change("tidy", NO_SPEC_META);
    let out = p.run_stdin(&["new", "artifact", "spec", "auth", "--change", "tidy", "--stdin"], Some(DELTA));
    assert!(!out.status.success());
    let err = stderr_of(&out);
    assert!(
        err.contains("no-spec-change") && err.contains("speclink change schema tidy spec-driven"),
        "{err}"
    );
    assert!(!tidy.join("specs").exists(), "no specs/ appears");

    let c = p.change("c", SPEC_META);
    let out = p.run_stdin(&["new", "artifact", "spec", "auth", "--change", "c", "--stdin"], Some(DELTA));
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(c.join("specs").join("auth").join("spec.md").is_file());
}

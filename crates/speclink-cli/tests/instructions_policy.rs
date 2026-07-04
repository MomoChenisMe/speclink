//! Integration tests: the instructions payload takes its policy values (locale, tdd,
//! audit) from the FOUR-LAYER resolution (env > legacy `.speclink.yaml` key >
//! `openspec/config.yaml` > default) — no new `--json` fields; the toggles surface
//! inside the existing `instruction` text, locale in the existing `locale` field.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, app_yaml: &str, wf_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-instr-policy-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let change = dir.join("openspec").join("changes").join("demo-change");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        std::fs::write(dir.join("openspec").join("config.yaml"), wf_yaml).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
        TempProject { dir }
    }

    fn instructions(&self, artifact: &str, envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(["instructions", artifact, "--change", "demo-change", "--json"])
            .current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
        ] {
            cmd.env_remove(key);
        }
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn json_payload(out: &Output) -> serde_json::Value {
    assert!(
        out.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout is JSON")
}

fn instruction_text(payload: &serde_json::Value) -> String {
    payload["instruction"].as_str().unwrap_or_default().to_string()
}

#[test]
fn canonical_config_yaml_toggles_reach_the_tasks_instruction() {
    // Spec scenario 正典值生效: policy only in openspec/config.yaml, no legacy keys.
    let p = TempProject::new(
        "canonical",
        "tools:\n  - claude\n",
        "tdd: true\naudit: true\n",
    );
    let out = p.instructions("tasks", &[]);
    let payload = json_payload(&out);
    let instr = instruction_text(&payload);
    assert!(instr.contains("TDD"), "tasks instruction must carry the TDD discipline: {instr}");
    assert!(instr.contains("audit"), "tasks instruction must carry the audit discipline: {instr}");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim(),
        "",
        "no deprecation warning without legacy keys"
    );
}

#[test]
fn toggles_off_leave_the_instruction_untouched() {
    let p = TempProject::new("defaults", "tools:\n  - claude\n", "schema: spec-driven\n");
    let payload = json_payload(&p.instructions("tasks", &[]));
    let instr = instruction_text(&payload);
    assert!(!instr.contains("TDD"), "no TDD note when the toggle is off: {instr}");
    assert!(!instr.contains("audit"), "no audit note when the toggle is off: {instr}");
}

#[test]
fn legacy_app_key_beats_canonical_locale_and_warns() {
    // Spec scenario 舊鍵相容層勝過正典值: app locale tw vs config.yaml locale ja.
    let p = TempProject::new(
        "legacy-wins",
        "locale: tw\ntools:\n  - claude\n",
        "locale: ja\n",
    );
    let out = p.instructions("proposal", &[]);
    let payload = json_payload(&out);
    assert_eq!(payload["locale"].as_str().unwrap(), "Traditional Chinese (繁體中文)");
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(
        err.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "exactly one deprecation line: {err:?}"
    );
}

#[test]
fn env_var_overrides_both_files() {
    // Spec scenario 環境變數覆寫一切: SPECLINK_TDD=false beats tdd: true in both files.
    let p = TempProject::new(
        "env-wins",
        "tdd: true\ntools:\n  - claude\n",
        "tdd: true\n",
    );
    let payload = json_payload(&p.instructions("tasks", &[("SPECLINK_TDD", "false")]));
    let instr = instruction_text(&payload);
    assert!(!instr.contains("TDD"), "env override must switch TDD off: {instr}");

    // And the locale env var beats the legacy app key.
    let p = TempProject::new(
        "env-locale",
        "locale: tw\ntools:\n  - claude\n",
        "locale: ja\n",
    );
    let payload = json_payload(&p.instructions("proposal", &[("SPECLINK_LOCALE", "en")]));
    assert_eq!(payload["locale"].as_str().unwrap(), "English");
}

#[test]
fn invalid_bool_env_var_falls_through_to_canonical_value() {
    // Spec scenario 非法布林環境變數落到下一層: SPECLINK_AUDIT=yes is ignored.
    let p = TempProject::new("invalid-env", "tools:\n  - claude\n", "audit: true\n");
    let out = p.instructions("tasks", &[("SPECLINK_AUDIT", "yes")]);
    let payload = json_payload(&out);
    assert!(
        instruction_text(&payload).contains("audit"),
        "invalid env value must fall through to config.yaml's audit: true"
    );
}

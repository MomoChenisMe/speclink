//! Integration tests: the instructions payload takes its policy values (locale, tdd,
//! audit) from the THREE-LAYER resolution (env > `speclink/config.yaml` > default) —
//! `.speclink.yaml` policy keys are inert and warning-free. The toggles surface
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
        let change = dir.join("speclink").join("changes").join("demo-change");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        std::fs::write(dir.join("speclink").join("config.yaml"), wf_yaml).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
        TempProject { dir }
    }

    fn instructions(&self, artifact: &str, envs: &[(&str, &str)]) -> Output {
        self.run(&["instructions", artifact, "--change", "demo-change", "--json"], envs)
    }

    fn run(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
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
    // Spec scenario 正典值生效: policy only in speclink/config.yaml.
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
        "no warning on the policy path"
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
fn app_policy_keys_are_inert_and_warning_free() {
    // Spec scenario .speclink.yaml 政策鍵一律不生效: app locale tw + tdd true vs
    // config.yaml locale ja (tdd unset) → canonical/default values, silent stderr.
    let p = TempProject::new(
        "app-inert",
        "locale: tw\ntdd: true\ntools:\n  - claude\n",
        "locale: ja\n",
    );
    let out = p.instructions("proposal", &[]);
    let payload = json_payload(&out);
    assert_eq!(payload["locale"].as_str().unwrap(), "Japanese (日本語)");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim(),
        "",
        "an app policy key must not warn"
    );
    let out = p.instructions("tasks", &[]);
    let instr = instruction_text(&json_payload(&out));
    assert!(!instr.contains("TDD"), "app tdd key must stay inert: {instr}");
    assert_eq!(String::from_utf8_lossy(&out.stderr).trim(), "");
}

#[test]
fn env_var_overrides_both_files() {
    // Spec scenario 環境變數覆寫正典值: SPECLINK_TDD=false beats tdd: true in config.yaml.
    let p = TempProject::new(
        "env-wins",
        "tdd: true\ntools:\n  - claude\n",
        "tdd: true\n",
    );
    let payload = json_payload(&p.instructions("tasks", &[("SPECLINK_TDD", "false")]));
    let instr = instruction_text(&payload);
    assert!(!instr.contains("TDD"), "env override must switch TDD off: {instr}");

    // And the locale env var beats the canonical value (the app key stays inert).
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

#[test]
fn apply_payload_carries_effective_policy_values() {
    // Spec scenario fs 模式 payload 帶有效值 + Example table tdd 有效值進 payload:
    // (SPECLINK_TDD, config.yaml tdd) → payload tdd, audit riding along.
    let rows: &[(Option<&str>, &str, bool)] = &[
        (None, "tdd: true\n", true),
        (None, "", false),
        (Some("false"), "tdd: true\n", false),
        (Some("true"), "", true),
    ];
    for (i, (env_tdd, wf_yaml, want_tdd)) in rows.iter().enumerate() {
        let p = TempProject::new(&format!("apply-policy-{i}"), "tools:\n  - claude\n", wf_yaml);
        let envs: Vec<(&str, &str)> = env_tdd.iter().map(|v| ("SPECLINK_TDD", *v)).collect();
        let payload = json_payload(&p.instructions("apply", &envs));
        assert_eq!(
            payload["tdd"].as_bool(),
            Some(*want_tdd),
            "row {i}: SPECLINK_TDD={env_tdd:?}, config.yaml={wf_yaml:?}"
        );
        assert_eq!(payload["audit"].as_bool(), Some(false), "row {i}: audit defaults to false");
    }
    // audit follows the same resolution entry point.
    let p = TempProject::new("apply-policy-audit", "tools:\n  - claude\n", "audit: true\n");
    let payload = json_payload(&p.instructions("apply", &[]));
    assert_eq!(payload["audit"].as_bool(), Some(true));
}

#[test]
fn update_with_legacy_policy_keys_touches_no_config_and_stays_silent() {
    // Spec scenario 既有專案不受範本變更影響: update on a project whose
    // .speclink.yaml still carries old policy keys rewrites neither config file
    // and prints no deprecation warning (the warning mechanism is gone).
    let p = TempProject::new(
        "update-legacy",
        "locale: tw\ntdd: true\ntools:\n  - claude\n",
        "locale: ja\n",
    );
    let app_before = std::fs::read_to_string(p.dir.join(".speclink.yaml")).unwrap();
    let wf_before = std::fs::read_to_string(p.dir.join("speclink").join("config.yaml")).unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_speclink"));
    for key in [
        "SPECLINK_LOCALE",
        "SPECLINK_SPEC_LOCALE",
        "SPECLINK_TDD",
        "SPECLINK_AUDIT",
        "SPECLINK_STORE_URL",
        "SPECLINK_TOKEN",
    ] {
        cmd.env_remove(key);
    }
    let out = cmd.arg("update").current_dir(&p.dir).output().expect("run speclink update");
    assert!(out.status.success(), "update failed: {}", String::from_utf8_lossy(&out.stderr));

    assert_eq!(
        std::fs::read_to_string(p.dir.join(".speclink.yaml")).unwrap(),
        app_before,
        "update must not rewrite .speclink.yaml"
    );
    assert_eq!(
        std::fs::read_to_string(p.dir.join("speclink").join("config.yaml")).unwrap(),
        wf_before,
        "update must not rewrite speclink/config.yaml"
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).trim(),
        "",
        "no deprecation warning on stderr"
    );
}

// --- spec「instructions 對 specs 產出物回報解析後的規格語言」---

/// The instruction ends with the engine's language paragraph for `display`, one blank
/// line after the text before it, and with one closing newline.
fn assert_ends_with_language_paragraph(instr: &str, display: &str) {
    let paragraph = format!(
        "Write spec prose in {display}. Structural markers (`## ADDED/MODIFIED/REMOVED/RENAMED \
Requirements`, `### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative \
keywords (SHALL/MUST/SHALL NOT/MUST NOT) stay in English so the analyzer and archive tooling \
can parse them.\n"
    );
    let before = instr
        .strip_suffix(&paragraph)
        .unwrap_or_else(|| panic!("the instruction ends with the {display} language paragraph: {instr:?}"));
    assert!(
        before.ends_with("\n\n") && !before.ends_with("\n\n\n"),
        "one blank line before the language paragraph: {instr:?}"
    );
}

#[test]
fn specs_locale_is_the_spec_language_and_proposal_keeps_the_project_locale() {
    // Scenario 沒有設定 spec_locale 時規格語言是 English: locale tw, spec_locale unset.
    let p = TempProject::new("spec-lang-default", "tools:\n  - claude\n", "locale: tw\n");
    let specs = json_payload(&p.instructions("specs", &[]));
    assert!(specs["locale"].is_string(), "locale stays a string: {specs}");
    assert_eq!(specs["locale"], "English");
    let instr = instruction_text(&specs);
    assert_ends_with_language_paragraph(&instr, "English");
    assert!(!instr.contains("config.yaml"), "the instruction never points at config.yaml: {instr}");
    let proposal = json_payload(&p.instructions("proposal", &[]));
    assert_eq!(proposal["locale"], "Traditional Chinese (繁體中文)");
}

#[test]
fn spec_locale_env_var_overrides_the_config_file_for_the_specs_locale() {
    // Scenario 環境變數覆寫規格語言: spec_locale en + SPECLINK_SPEC_LOCALE=ja.
    let p = TempProject::new("spec-lang-env", "tools:\n  - claude\n", "spec_locale: en\n");
    let specs = json_payload(&p.instructions("specs", &[("SPECLINK_SPEC_LOCALE", "ja")]));
    assert_eq!(specs["locale"], "Japanese (日本語)");
    assert_ends_with_language_paragraph(&instruction_text(&specs), "Japanese (日本語)");
}

#[test]
fn human_instruction_section_ends_with_the_same_language_paragraph() {
    let p = TempProject::new("spec-lang-human", "tools:\n  - claude\n", "locale: tw\n");
    let json = instruction_text(&json_payload(&p.instructions("specs", &[])));
    let out = p.run(&["instructions", "specs", "--change", "demo-change", "--no-color"], &[]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let human = String::from_utf8_lossy(&out.stdout);
    assert!(!human.contains('\u{1b}'), "--no-color prints no color codes");
    // No locale field: the header is the three known lines, and nothing stands between
    // the instruction text and the next section.
    let (header, _) = human.split_once("\nInstruction:\n").expect("Instruction section");
    let labels: Vec<&str> = header.lines().filter_map(|l| l.split_once(": ")).map(|(label, _)| label).collect();
    assert_eq!(labels, ["Artifact", "Output", "Description"], "{header}");
    assert_ends_with_language_paragraph(&json, "English");
    assert!(
        human.contains(&format!("\nInstruction:\n{json}\n\nDependencies:\n")),
        "the Instruction section is the payload's instruction, then the next section: {human}"
    );
}

#[test]
fn broken_config_yaml_fails_specs_instructions_without_a_payload() {
    // Scenario 設定檔無法解析時不回報語言.
    let p = TempProject::new("spec-lang-broken", "tools:\n  - claude\n", ": not yaml : [\n");
    let out = p.instructions("specs", &[]);
    assert!(!out.status.success(), "a broken config.yaml fails the command");
    assert!(out.stdout.is_empty(), "no payload on stdout: {}", String::from_utf8_lossy(&out.stdout));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let reason = stderr.split_once("speclink/config.yaml:").map(|(_, reason)| reason.trim());
    assert!(
        reason.is_some_and(|r| !r.is_empty()),
        "stderr names the file and the parse reason: {stderr}"
    );
}

// --- spec「tasks 指示的紀律說明不指向帶 --skill 的 instructions」---

#[test]
fn tasks_instruction_never_points_at_skill_instructions() {
    let p = TempProject::new("tasks-no-skill", "tools:\n  - claude\n", "tdd: true\naudit: true\n");
    let instr = instruction_text(&json_payload(&p.instructions("tasks", &[])));
    assert!(instr.contains("The apply step supplies the full TDD discipline"), "{instr}");
    assert!(instr.contains("The apply step supplies the checklist"), "{instr}");
    assert!(!instr.contains("instructions --skill"), "{instr}");
}

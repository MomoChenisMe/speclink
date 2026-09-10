//! `speclink workflow-config` — the CLI verb that manages `openspec/config.yaml`
//! (workflow policy, context, rules), in both fs and remote mode.
//!
//! fs mode reads and writes the file through the workspace; remote mode reads
//! the server document with its revision, applies the SAME core rewrite, and
//! writes back guarded by that revision. Both share one rendering path, so the
//! `--json` payload shape is asserted identically for each.

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};

// --- fs fixture ---

/// A workflow config with a value in every surface the verb touches: two policy
/// keys set (locale, tdd), two left unset (spec_locale, audit), a context block,
/// and two rule sections.
const WF_YAML: &str = "\
schema: spec-driven
locale: tw
tdd: true
context: |
  Project context line one.
  Line two.
rules:
  design:
  - Name the crate boundary
  - List one alternative
  tasks:
  - Use checkbox format
";

const BAD_YAML: &str = ": not yaml : [\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, wf_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-workflow-config-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::write(dir.join("openspec").join("config.yaml"), wf_yaml).unwrap();
        TempProject { dir }
    }

    fn config_path(&self) -> PathBuf {
        self.dir.join("openspec").join("config.yaml")
    }

    fn config_bytes(&self) -> Vec<u8> {
        std::fs::read(self.config_path()).expect("read config.yaml")
    }

    fn config_text(&self) -> String {
        std::fs::read_to_string(self.config_path()).expect("read config.yaml")
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir).env("NO_COLOR", "1");
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_WORKTREE",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = self.cmd(args);
        for (k, v) in envs {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], stdin: &str) -> Output {
        run_with_stdin(self.cmd(args), stdin)
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn run_with_stdin(mut cmd: Command, stdin: &str) -> Output {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn speclink binary");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait speclink binary")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("stdout is JSON ({e}): {}", stdout_of(out));
    })
}

// --- show (fs) ---

#[test]
fn show_prints_canonical_policy_context_and_rules() {
    // Spec scenario fs 模式顯示正典值.
    let p = TempProject::new("show-human", WF_YAML);
    let out = p.run(&["workflow-config", "show"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(text.contains("locale"), "policy fields listed: {text}");
    assert!(text.contains("tw"), "locale value shown: {text}");
    let tdd_line = text.lines().find(|l| l.contains("tdd")).unwrap_or_default();
    assert!(tdd_line.contains("on"), "tdd shown as on: {text}");
    let audit_line = text.lines().find(|l| l.contains("audit")).unwrap_or_default();
    assert!(
        audit_line.contains("unset") && audit_line.contains("off"),
        "audit shown as unset with its default: {text}"
    );
    let worktree_line = text.lines().find(|l| l.contains("worktree")).unwrap_or_default();
    assert!(
        worktree_line.contains("unset") && worktree_line.contains("off"),
        "worktree shown as unset with its default: {text}"
    );
    let context_line = text.lines().find(|l| l.contains("context")).unwrap_or_default();
    assert!(context_line.contains('2'), "context line count shown: {text}");
    let rules_line = text.lines().find(|l| l.contains("rules")).unwrap_or_default();
    assert!(
        rules_line.contains("design") && rules_line.contains('2'),
        "rules section counts shown: {text}"
    );
    assert!(!text.contains('\u{1b}'), "no ANSI escapes: {text:?}");
}

#[test]
fn show_json_payload_is_camel_case_with_null_for_unset() {
    // Spec scenario --json payload 形狀.
    let p = TempProject::new("show-json", WF_YAML);
    let out = p.run(&["workflow-config", "show", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload = json_of(&out);
    assert_eq!(payload["locale"], "tw");
    assert!(payload["specLocale"].is_null(), "unset specLocale is null: {payload}");
    assert_eq!(payload["tdd"], true);
    assert_eq!(payload["audit"], false, "unset tdd/audit read as false");
    assert_eq!(payload["worktree"], false, "unset worktree reads as false");
    assert!(
        payload["context"].as_str().unwrap().contains("Project context line one."),
        "context text carried: {payload}"
    );
    assert_eq!(
        payload["rules"]["design"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["Name the crate boundary", "List one alternative"]
    );
    let keys: Vec<&str> = payload.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert!(!keys.contains(&"spec_locale"), "no snake_case keys: {keys:?}");
}

#[test]
fn show_json_reports_the_canonical_value_not_the_env_override() {
    // Spec scenario show 不應用環境變數覆寫.
    let p = TempProject::new("show-env", WF_YAML);
    let out = p.run_env(
        &["workflow-config", "show", "--json"],
        &[("SPECLINK_TDD", "false")],
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(json_of(&out)["tdd"], true, "canonical value wins over the env layer");
}

#[test]
fn show_fails_closed_on_an_unparseable_config() {
    // Spec scenario 壞 config fail-closed.
    let p = TempProject::new("show-bad", BAD_YAML);
    let out = p.run(&["workflow-config", "show"]);
    assert!(!out.status.success(), "must exit non-zero on a broken config");
    assert!(
        stderr_of(&out).contains("openspec/config.yaml"),
        "stderr names the file: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).trim().is_empty(), "no payload on stdout");
}

// --- set (fs) ---

#[test]
fn set_locale_writes_the_key_and_preserves_the_others() {
    // Spec scenario 設定 locale 保留其他鍵.
    let p = TempProject::new("set-locale", WF_YAML);
    let out = p.run(&["workflow-config", "set", "locale", "ja"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    // 手術改寫：僅 locale 行變動，其餘行逐位元保留（不再整檔重排）。
    assert_eq!(p.config_text(), WF_YAML.replace("locale: tw", "locale: ja"));
}

#[test]
fn set_leaves_no_temp_residue_beside_the_config() {
    // Spec scenario 設定寫入走同一原子入口——觀察面：寫入成功後 openspec/ 無暫存
    // 殘留、config.yaml 為完整全文。
    let p = TempProject::new("set-atomic-face", WF_YAML);
    let out = p.run(&["workflow-config", "set", "locale", "ja"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let residue: Vec<String> = std::fs::read_dir(p.dir.join("openspec"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(residue.is_empty(), "temp residue left behind: {residue:?}");
    assert_eq!(p.config_text(), WF_YAML.replace("locale: tw", "locale: ja"));
}

#[test]
fn set_rejects_an_unknown_key_without_touching_the_file() {
    // Spec scenario 未知 key 拒絕.
    let p = TempProject::new("set-unknown", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "theme", "dark"]);
    assert!(!out.status.success(), "unknown key must exit non-zero");
    let err = stderr_of(&out);
    for key in ["locale", "spec_locale", "tdd", "audit", "worktree"] {
        assert!(err.contains(key), "stderr lists the accepted keys ({key}): {err}");
    }
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_rejects_a_non_boolean_toggle_value() {
    // Spec scenario 非法布林值拒絕.
    let p = TempProject::new("set-bad-bool", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "tdd", "yes"]);
    assert!(!out.status.success(), "non-boolean value must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("tdd"), "stderr names the key: {err}");
    assert!(
        err.contains("true") && err.contains("false"),
        "stderr states the accepted values: {err}"
    );
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn set_false_removes_the_key() {
    // Spec scenario 設 false 移除鍵.
    let p = TempProject::new("set-false", "schema: spec-driven\naudit: true\n");
    let out = p.run(&["workflow-config", "set", "audit", "false"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    // 手術改寫：僅 audit 鍵行被移除，其餘行逐位元保留。
    assert_eq!(p.config_text(), "schema: spec-driven\n");
}

#[test]
fn set_worktree_true_lands_in_the_file_and_in_show() {
    // Spec scenario worktree 欄位寫入與呈現.
    let p = TempProject::new("set-worktree", WF_YAML);
    let out = p.run(&["workflow-config", "set", "worktree", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(
        p.config_text().contains("worktree: true"),
        "config.yaml carries the key: {}",
        p.config_text()
    );
    let payload = json_of(&p.run(&["workflow-config", "show", "--json"]));
    assert_eq!(payload["worktree"], true, "show reflects the write: {payload}");
    let text = stdout_of(&p.run(&["workflow-config", "show"]));
    let line = text.lines().find(|l| l.contains("worktree")).unwrap_or_default();
    assert!(line.contains("on"), "human output shows worktree on: {text}");
}

#[test]
fn set_worktree_rejects_a_non_boolean_value() {
    // Spec scenario worktree 非法值報錯.
    let p = TempProject::new("set-worktree-bad", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "worktree", "yes"]);
    assert!(!out.status.success(), "non-boolean value must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("worktree"), "stderr names the key: {err}");
    assert!(
        err.contains("true") && err.contains("false"),
        "stderr states the accepted values: {err}"
    );
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_another_key_preserves_an_existing_worktree_value() {
    // 完整目標狀態的回填缺口：編輯任一政策鍵都不得吃掉 worktree。
    let p = TempProject::new("set-keeps-worktree", "schema: spec-driven\nworktree: true\n");
    let out = p.run(&["workflow-config", "set", "audit", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(after.contains("worktree: true"), "worktree survives: {after}");
    assert!(after.contains("audit: true"), "the edit landed: {after}");
}

#[test]
fn set_dry_run_prints_a_unified_diff_and_writes_nothing() {
    // Spec scenario dry-run 印 diff 不落檔.
    let p = TempProject::new("set-dry-run", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "ja", "--dry-run"]);
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("--- a/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("+++ b/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("-locale: tw"), "removed line: {diff}");
    assert!(diff.contains("+locale: ja"), "added line: {diff}");
    // 手術改寫後 diff 收窄：除 locale 行外不得有其他 -/+ 變動行（不含整檔重排）。
    let changed: Vec<&str> = diff
        .lines()
        .filter(|l| {
            (l.starts_with('-') || l.starts_with('+'))
                && !l.starts_with("---")
                && !l.starts_with("+++")
        })
        .collect();
    assert_eq!(changed, ["-locale: tw", "+locale: ja"], "only the locale line changes: {diff}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_rejects_a_display_name_locale_value() {
    // Spec scenario 非法 locale 值拒絕（workflow-config-locale-validation）.
    let p = TempProject::new("set-bad-locale", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "繁體中文"]);
    assert!(!out.status.success(), "display-name value must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("locale"), "stderr names the key: {err}");
    assert!(err.contains("繁體中文"), "stderr echoes the received value: {err}");
    for code in ["tw", "ja", "en"] {
        assert!(err.contains(code), "stderr lists the accepted codes ({code}): {err}");
    }
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

#[test]
fn set_rejects_a_case_variant_locale_code() {
    // Spec Example 表：TW 拒絕（大小寫敏感）.
    let p = TempProject::new("set-case-locale", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "TW"]);
    assert!(!out.status.success(), "case variant must exit non-zero");
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn set_dry_run_rejects_an_invalid_locale_without_a_diff() {
    // Spec scenario 非法值帶 dry-run 同樣拒絕.
    let p = TempProject::new("set-dry-bad-locale", WF_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "spec_locale", "繁體中文", "--dry-run"]);
    assert!(!out.status.success(), "invalid value must exit non-zero even with --dry-run");
    assert!(!stdout_of(&out).contains("@@"), "no diff on stdout: {}", stdout_of(&out));
    let err = stderr_of(&out);
    for code in ["tw", "ja", "en", "auto"] {
        assert!(err.contains(code), "stderr lists the spec_locale codes ({code}): {err}");
    }
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn set_accepts_auto_for_spec_locale_and_empty_removes_the_key() {
    // Spec Example 表：spec_locale auto 成功；locale 空字串 成功（移除鍵）.
    let p = TempProject::new("set-auto-and-empty", WF_YAML);
    let out = p.run(&["workflow-config", "set", "spec_locale", "auto"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(p.config_text().contains("spec_locale: auto"), "got: {}", p.config_text());
    let out = p.run(&["workflow-config", "set", "locale", ""]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(!p.config_text().contains("locale: tw"), "locale key removed: {}", p.config_text());
}

#[test]
fn set_refuses_to_rewrite_an_unparseable_config() {
    // Spec scenario 壞 config 拒絕寫入.
    let p = TempProject::new("set-bad", BAD_YAML);
    let before = p.config_bytes();
    let out = p.run(&["workflow-config", "set", "locale", "tw"]);
    assert!(!out.status.success(), "must exit non-zero on a broken config");
    let err = stderr_of(&out);
    assert!(err.contains("openspec/config.yaml"), "stderr names the file: {err}");
    assert!(err.contains("refused"), "stderr says the write was refused: {err}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

// --- context (fs) ---

#[test]
fn context_stdin_sets_the_multiline_value() {
    // Spec scenario context 設定多行內容.
    let p = TempProject::new("context-set", WF_YAML);
    let out = p.run_stdin(
        &["workflow-config", "context", "--stdin"],
        "## Project\n\nA rewritten description.\nSecond line.\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(after.contains("A rewritten description."), "context written: {after}");
    assert!(after.contains("Second line."), "every line written: {after}");
    assert!(!after.contains("Project context line one."), "old context replaced: {after}");
    assert!(after.contains("locale: tw"), "policy fields untouched: {after}");
    assert!(after.contains("tdd: true"), "policy fields untouched: {after}");
    assert!(after.contains("Name the crate boundary"), "rules untouched: {after}");
}

#[test]
fn context_blank_stdin_removes_the_key() {
    // Spec scenario 空白 stdin 移除 context.
    let p = TempProject::new("context-clear", WF_YAML);
    let out = p.run_stdin(&["workflow-config", "context", "--stdin"], "   \n\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.config_text();
    assert!(!after.contains("context"), "context key removed: {after}");
    assert!(after.contains("locale: tw"), "policy fields untouched: {after}");
}

#[test]
fn context_without_the_stdin_flag_fails_with_usage() {
    // Spec: 兩個子指令未帶 --stdin 時以非零 exit code 說明用法.
    let p = TempProject::new("context-no-stdin", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(&["workflow-config", "context"], "ignored\n");
    assert!(!out.status.success(), "missing --stdin must exit non-zero");
    assert!(
        stderr_of(&out).contains("--stdin"),
        "stderr states the usage: {}",
        stderr_of(&out)
    );
    assert_eq!(p.config_bytes(), before, "no file effect");
}

// --- rules (fs) ---

#[test]
fn rules_replaces_the_whole_section() {
    // Spec scenario rules 整節代換.
    let p = TempProject::new("rules-replace", WF_YAML);
    let out = p.run_stdin(
        &["workflow-config", "rules", "design", "--stdin"],
        "First rule\nSecond rule\nThird rule\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let after = p.run(&["workflow-config", "show", "--json"]);
    let payload = json_of(&after);
    assert_eq!(
        payload["rules"]["design"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["First rule", "Second rule", "Third rule"],
        "the section is replaced wholesale, not merged"
    );
    assert_eq!(
        payload["rules"]["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["Use checkbox format"],
        "other artifact sections are untouched"
    );
}

#[test]
fn rules_empty_stdin_removes_the_section() {
    // Spec: stdin 為空時移除該 artifact 節.
    let p = TempProject::new("rules-clear", WF_YAML);
    let out = p.run_stdin(&["workflow-config", "rules", "design", "--stdin"], "\n");
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let payload = json_of(&p.run(&["workflow-config", "show", "--json"]));
    assert!(payload["rules"]["design"].is_null(), "design section removed: {payload}");
    assert!(!payload["rules"]["tasks"].is_null(), "tasks section kept: {payload}");
}

#[test]
fn rules_rejects_an_artifact_outside_the_schema() {
    // Spec scenario 未知 artifact 拒絕.
    let p = TempProject::new("rules-unknown", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(
        &["workflow-config", "rules", "blueprint", "--stdin"],
        "Anything\n",
    );
    assert!(!out.status.success(), "unknown artifact must exit non-zero");
    let err = stderr_of(&out);
    assert!(err.contains("blueprint"), "stderr names the artifact: {err}");
    assert!(err.contains("proposal"), "stderr lists the schema's artifacts: {err}");
    assert_eq!(p.config_bytes(), before, "no file effect");
}

#[test]
fn rules_dry_run_prints_a_unified_diff_and_writes_nothing() {
    // Spec scenario rules dry-run 印 diff.
    let p = TempProject::new("rules-dry-run", WF_YAML);
    let before = p.config_bytes();
    let out = p.run_stdin(
        &["workflow-config", "rules", "tasks", "--stdin", "--dry-run"],
        "Only one rule now\n",
    );
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("--- a/openspec/config.yaml"), "unified diff header: {diff}");
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("+  - Only one rule now"), "added rule line: {diff}");
    assert!(diff.contains("-  - Use checkbox format"), "removed rule line: {diff}");
    assert_eq!(p.config_bytes(), before, "config.yaml is byte-for-byte unchanged");
}

// --- help parity ---

/// The comma-separated list a "…: a, b, c" sentence ends with. Both the help
/// text and the unknown-key error are parsed through this ONE reader, so the
/// test never writes down its own copy of the policy keys — a third list would
/// just move the drift here.
fn keys_after_colon(text: &str) -> Vec<String> {
    let (_, list) = text.rsplit_once(": ").unwrap_or_else(|| {
        panic!("expected a '…: key, key' sentence: {text}");
    });
    list.trim().trim_end_matches('.').split(',').map(|k| k.trim().to_string()).collect()
}

/// The description clap renders above `Usage:` — everything before the first
/// blank line, whitespace collapsed so the assertion never depends on layout.
fn help_about(text: &str) -> String {
    let head: Vec<&str> = text.lines().take_while(|l| !l.trim().is_empty()).collect();
    head.join(" ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One entry of a clap table (`Commands:` or `Arguments:`): the matching row
/// plus any wrapped continuation lines (clap indents those deeper than the
/// entry column), whitespace collapsed. Wrap-safe so a longer key list can
/// never fail these tests over layout alone.
fn table_row(text: &str, name: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with(&format!("{name} ")) || l.trim_start() == name)
        .unwrap_or_else(|| panic!("no '{name}' row in the table: {text}"));
    let indent = lines[idx].len() - lines[idx].trim_start().len();
    let mut row = lines[idx].trim_start().to_string();
    for line in &lines[idx + 1..] {
        if line.trim().is_empty() || line.len() - line.trim_start().len() <= indent {
            break;
        }
        row.push(' ');
        row.push_str(line.trim_start());
    }
    row.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The rejection stderr of `set` with an unknown key — asserted to BE the
/// unknown-key error before anything parses it, so an unrelated failure
/// (fail-closed config, workspace discovery) can never masquerade as the
/// accepted-keys sentence.
fn unknown_key_stderr(p: &TempProject) -> String {
    let rejected = p.run(&["workflow-config", "set", "theme", "dark"]);
    assert!(!rejected.status.success(), "unknown key must exit non-zero");
    let err = stderr_of(&rejected).trim().to_string();
    assert!(err.contains("Unknown key"), "the rejection is the unknown-key error: {err}");
    err
}

#[test]
fn set_help_lists_exactly_the_accepted_policy_keys() {
    // Spec scenario set --help 列出全部政策鍵.
    let p = TempProject::new("set-help-keys", WF_YAML);
    let help = p.run(&["workflow-config", "set", "--help"]);
    assert!(help.status.success(), "help exits zero: {}", stderr_of(&help));

    let error_text = unknown_key_stderr(&p);
    let accepted = keys_after_colon(&error_text);
    let advertised = keys_after_colon(&help_about(&stdout_of(&help)));
    assert_eq!(
        advertised,
        accepted,
        "set --help must advertise exactly the keys the verb accepts\nhelp:  {}\nerror: {}",
        stdout_of(&help),
        error_text
    );

    // Help and the error sentence share one generator, so their equality alone
    // cannot catch an advertised key the verb no longer recognizes. Probe each
    // advertised key end-to-end: a junk value may be refused for its VALUE, but
    // never as an unknown KEY. (The reverse hole — a hidden key accepted but
    // advertised nowhere — has no list to probe from and stays out of scope.)
    for key in &advertised {
        let out = p.run(&["workflow-config", "set", key, "zzz", "--dry-run"]);
        let err = stderr_of(&out);
        assert!(
            !err.contains("Unknown key"),
            "advertised key '{key}' must be recognized by set: {err}"
        );
    }
}

#[test]
fn set_help_value_argument_names_every_boolean_key() {
    // Spec scenario set --help 標明布林鍵的合法值.
    let p = TempProject::new("set-help-value", WF_YAML);
    let out = p.run(&["workflow-config", "set", "--help"]);
    assert!(out.status.success(), "help exits zero: {}", stderr_of(&out));
    let text = stdout_of(&out);
    // The `Usage:` line carries `<VALUE>` too — table_row only matches the
    // Arguments row (and absorbs a wrapped or next-line description).
    let row = table_row(&text, "<VALUE>");
    for key in ["tdd", "audit", "worktree"] {
        assert!(row.contains(key), "<VALUE> description names the boolean key {key}: {row}");
    }
    assert!(
        row.contains("true") && row.contains("false"),
        "<VALUE> description states the accepted values: {row}"
    );
}

#[test]
fn parent_help_and_set_help_share_one_set_description() {
    // Spec scenario 父層 help 的 set 說明同源.
    let p = TempProject::new("set-help-parent", WF_YAML);
    let parent = p.run(&["workflow-config", "--help"]);
    assert!(parent.status.success(), "help exits zero: {}", stderr_of(&parent));
    let child = p.run(&["workflow-config", "set", "--help"]);
    assert!(child.status.success(), "help exits zero: {}", stderr_of(&child));

    let row = table_row(&stdout_of(&parent), "set");
    let from_parent = row.split_once(' ').unwrap().1.to_string();
    let from_child = help_about(&stdout_of(&child));
    assert_eq!(from_parent, from_child, "both help surfaces read from one description");

    assert_eq!(
        keys_after_colon(&from_parent),
        keys_after_colon(&unknown_key_stderr(&p)),
        "the command list advertises every accepted key: {from_parent}"
    );
}

// --- remote mode ---

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    path: String,
    body: String,
}

struct MockServer {
    server: Arc<tiny_http::Server>,
    base: String,
    captured: Arc<Mutex<Vec<Captured>>>,
}

const BINDING_BODY: &str = r#"{"actor":{"id":"u_1","name":"Tester"},"project":{"id":"prj_1","key":"demo","name":"Demo"},"repo":{"id":"repo_1","key":"backend","name":"Backend"},"apiVersion":"1","engineVersion":"0.1.0","capabilities":{"events":{"transports":[],"polling":{"url":"/sync-state","etag":true}}}}"#;

fn mock_server(mut routes: Vec<(&'static str, &'static str, u16, String)>) -> MockServer {
    if !routes.iter().any(|(_, suffix, _, _)| *suffix == "/binding") {
        routes.push(("GET", "/binding", 200, BINDING_BODY.to_string()));
    }
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("bind mock server"));
    let port = server.server_addr().to_ip().expect("ip").port();
    let base = format!("http://127.0.0.1:{port}/api/speclink/v1/projects/demo");
    let captured = Arc::new(Mutex::new(Vec::new()));
    let looper = Arc::clone(&server);
    let sink = Arc::clone(&captured);
    std::thread::spawn(move || {
        for mut req in looper.incoming_requests() {
            let mut body = String::new();
            let _ = req.as_reader().read_to_string(&mut body);
            let path = req.url().split('?').next().unwrap_or_default().to_string();
            sink.lock().unwrap().push(Captured {
                method: req.method().to_string(),
                path: path.clone(),
                body,
            });
            let hit = routes.iter().find(|(m, suffix, _, _)| {
                req.method().to_string() == *m
                    && path == format!("/api/speclink/v1/projects/demo{suffix}")
            });
            let (status, body) = match hit {
                Some((_, _, status, body)) => (*status, body.clone()),
                None => (
                    404,
                    r#"{"reason":"not_found","message":"no route","resource":"route","name":"?"}"#
                        .to_string(),
                ),
            };
            let resp = tiny_http::Response::from_string(body)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                        .unwrap(),
                );
            let _ = req.respond(resp);
        }
    });
    MockServer { server, base, captured }
}

impl MockServer {
    fn find(&self, method: &str, path_suffix: &str) -> Captured {
        let caps = self.captured.lock().unwrap();
        caps.iter()
            .find(|c| {
                c.method == method
                    && c.path == format!("/api/speclink/v1/projects/demo{path_suffix}")
            })
            .unwrap_or_else(|| panic!("no captured {method} {path_suffix}; got {caps:?}"))
            .clone()
    }

    fn none(&self, method: &str, path_suffix: &str) -> bool {
        let caps = self.captured.lock().unwrap();
        !caps.iter().any(|c| {
            c.method == method && c.path == format!("/api/speclink/v1/projects/demo{path_suffix}")
        })
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

struct RemoteProject {
    dir: PathBuf,
}

impl RemoteProject {
    fn new(tag: &str, url: &str) -> RemoteProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-workflow-config-remote-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".speclink.yaml"),
            format!("remote:\n  url: {url}\n  repo: backend\n"),
        )
        .unwrap();
        RemoteProject { dir }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args)
            .current_dir(&self.dir)
            .env("NO_COLOR", "1")
            .env("SPECLINK_TOKEN", "tok");
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_WORKTREE",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], stdin: &str) -> Output {
        run_with_stdin(self.cmd(args), stdin)
    }
}

impl Drop for RemoteProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// `GET /config` body carrying the fs fixture document at revision 7.
fn config_response() -> String {
    serde_json::json!({
        "schema": "spec-driven",
        "content": WF_YAML,
        "revision": 7,
    })
    .to_string()
}

#[test]
fn remote_show_json_has_the_same_shape_as_fs() {
    // Spec scenario remote 模式輸出形狀一致.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let p = RemoteProject::new("show", &mock.base);
    let out = p.run(&["workflow-config", "show", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let remote = json_of(&out);

    let fs = TempProject::new("remote-shape", WF_YAML);
    let fs_out = fs.run(&["workflow-config", "show", "--json"]);
    assert!(fs_out.status.success(), "stderr: {}", stderr_of(&fs_out));
    assert_eq!(remote, json_of(&fs_out), "remote and fs payloads are identical");
}

#[test]
fn remote_set_reads_then_writes_back_with_the_read_revision() {
    // Spec: remote 先讀 server 端現行內容與版本、套用同一改寫、寫回時附帶讀得的版本.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        ("PUT", "/config", 200, r#"{"revision":8}"#.into()),
    ]);
    let p = RemoteProject::new("set", &mock.base);
    let out = p.run(&["workflow-config", "set", "audit", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let put = mock.find("PUT", "/config");
    let body: serde_json::Value = serde_json::from_str(&put.body).expect("PUT body is JSON");
    assert_eq!(body["expectedRevision"], 7, "the read revision guards the write");
    let content = body["content"].as_str().expect("content string");
    assert!(content.contains("audit: true"), "the edit landed: {content}");
    assert!(content.contains("locale: tw"), "other keys preserved: {content}");
    assert!(
        !stdout_of(&out).contains('7') && !stdout_of(&out).contains("revision"),
        "the revision never surfaces in the command output: {}",
        stdout_of(&out)
    );
}

#[test]
fn remote_context_write_round_trips_through_show() {
    // Spec scenario remote context 寫入經版本檢查.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        ("PUT", "/config", 200, r#"{"revision":8}"#.into()),
    ]);
    let p = RemoteProject::new("context", &mock.base);
    let out = p.run_stdin(
        &["workflow-config", "context", "--stdin"],
        "Remote project description.\n",
    );
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let put = mock.find("PUT", "/config");
    let body: serde_json::Value = serde_json::from_str(&put.body).expect("PUT body is JSON");
    let content = body["content"].as_str().expect("content string");
    assert!(
        content.contains("Remote project description."),
        "context written to the server document: {content}"
    );
}

#[test]
fn remote_revision_conflict_asks_for_a_rerun_without_overwriting() {
    // Spec scenario remote 版本衝突提示重跑.
    let mock = mock_server(vec![
        ("GET", "/config", 200, config_response()),
        (
            "PUT",
            "/config",
            409,
            r#"{"status":409,"reason":"revision_conflict","message":"stale write"}"#.into(),
        ),
    ]);
    let p = RemoteProject::new("conflict", &mock.base);
    let out = p.run(&["workflow-config", "set", "tdd", "true"]);
    assert!(!out.status.success(), "a CAS refusal must exit non-zero");
    let err = stderr_of(&out);
    assert!(
        err.contains("updated by someone else") && err.contains("re-run"),
        "stderr explains the conflict and the fix: {err}"
    );
}

#[test]
fn remote_dry_run_prints_the_diff_without_writing() {
    // Spec: --dry-run 行為與 fs 一致（remote 亦不送出寫入）.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let p = RemoteProject::new("dry-run", &mock.base);
    let out = p.run(&["workflow-config", "set", "locale", "ja", "--dry-run"]);
    assert!(out.status.success(), "dry-run exits 0: {}", stderr_of(&out));
    let diff = stdout_of(&out);
    assert!(diff.contains("@@"), "unified diff hunk: {diff}");
    assert!(diff.contains("+locale: ja"), "added line: {diff}");
    assert!(mock.none("PUT", "/config"), "dry-run sends no write request");
}

#[test]
fn remote_offline_fails_semantically_without_queueing() {
    // Spec: 連線離線時以非零 exit code 失敗並輸出語義化訊息，不暫存或排隊寫入.
    let mock = mock_server(vec![("GET", "/config", 200, config_response())]);
    let base = mock.base.clone();
    let p = RemoteProject::new("offline", &base);
    drop(mock); // server gone — the connection is refused
    let out = p.run(&["workflow-config", "set", "tdd", "true"]);
    assert!(!out.status.success(), "offline must exit non-zero");
    let err = stderr_of(&out);
    assert!(
        err.contains("unreachable") || err.contains("connection"),
        "stderr is a semantic connection failure: {err}"
    );
    assert!(
        !p.dir.join(".speclink").exists(),
        "nothing is queued or spooled locally"
    );
}

// --- worktree 政策閘：技能足跡隨政策進出 ---
// Spec workspace-tools「worktree 技能的政策條件式生成」。

/// 受閘控技能於 claude 足跡下的目錄（跨平台以分段 join，不寫死分隔符）。
const GATED_SKILLS: [&str; 2] = [
    ".claude/skills/speclink-apply-with-worktree",
    ".claude/skills/speclink-worktree-merge",
];

fn dir_exists(p: &TempProject, rel: &str) -> bool {
    p.dir.join(rel.split('/').collect::<PathBuf>()).is_dir()
}

#[test]
fn update_generates_worktree_skills_only_while_the_policy_is_on() {
    // Scenario「政策開啟時注入兩顆技能」＋「政策由開改關後再生即清理」：同一個
    // fixture 走完開→關，證明 update 是收斂的（不是只在初次生成時判斷）。
    let p = TempProject::new("gate-flip", "schema: spec-driven\nworktree: true\n");

    let out = p.run(&["update"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    for rel in GATED_SKILLS {
        assert!(dir_exists(&p, rel), "政策開啟時須生成 {rel}");
    }

    std::fs::write(p.config_path(), "schema: spec-driven\n").unwrap();
    let out = p.run(&["update"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    for rel in GATED_SKILLS {
        assert!(!dir_exists(&p, rel), "政策關閉後須清理 {rel}");
    }
    // 其餘技能不受影響；指令檔一律不生成。
    assert!(dir_exists(&p, ".claude/skills/speclink-apply"), "非閘控技能須保留");
    assert!(!p.dir.join("CLAUDE.md").exists(), "指令檔不得生成");
}

#[test]
fn the_worktree_env_override_does_not_affect_generation() {
    // Scenario「環境變數不影響生成」：env 是執行期逃生口，注入只跟檔值走。
    let p = TempProject::new("gate-env", "schema: spec-driven\n");
    let out = p.run_env(&["update"], &[("SPECLINK_WORKTREE", "true")]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    for rel in GATED_SKILLS {
        assert!(!dir_exists(&p, rel), "env 不得使 {rel} 被生成");
    }
}

// --- set worktree：寫入後同步足跡，關閉前先擋活躍 worktree ---
// Spec workflow-config「worktree 欄位寫入的技能同步與關閉擋下」。

/// 把 fixture 變成一個真的 git 主 checkout（worktree 探索的前提）。
fn git_init(p: &TempProject) {
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&p.dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.test")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.test")
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "seed"]);
}

/// Attach a linked worktree on `speclink/<change>` with that change present in
/// both copies — the three conditions discovery requires for a mapping.
fn attach_worktree(p: &TempProject, change: &str) -> PathBuf {
    let change_dir = p.dir.join("openspec").join("changes").join(change);
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "## Why\n\nseed\n").unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&p.dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.test")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.test")
            .output()
            .expect("run git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["add", "-A"]);
    git(&["commit", "-qm", "change"]);
    let wt = p.dir.join("wt");
    git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        &format!("speclink/{change}"),
        wt.to_str().unwrap(),
    ]);
    wt
}

#[test]
fn set_worktree_true_then_false_moves_the_skill_footprint_with_it() {
    // Scenario「set true 寫入並注入技能」＋「set false 無活躍 worktree 時寫入並清理」。
    let p = TempProject::new("set-wt-sync", "schema: spec-driven\n");

    let out = p.run(&["workflow-config", "set", "worktree", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    for rel in GATED_SKILLS {
        assert!(dir_exists(&p, rel), "set true 後須注入 {rel}");
    }

    let out = p.run(&["workflow-config", "set", "worktree", "false"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    for rel in GATED_SKILLS {
        assert!(!dir_exists(&p, rel), "set false 後須清理 {rel}");
    }
}

#[test]
fn setting_another_policy_key_does_not_touch_the_skill_footprint() {
    // 同步只綁 worktree 鍵：改 locale 不該重寫技能檔（其他鍵的寫入語意不變）。
    let p = TempProject::new("set-wt-scope", "schema: spec-driven\n");
    let out = p.run(&["workflow-config", "set", "locale", "ja"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(
        !dir_exists(&p, ".claude/skills/speclink-apply"),
        "非 worktree 鍵的寫入不觸發足跡生成"
    );
}

#[test]
fn set_worktree_false_is_refused_while_a_linked_worktree_is_active() {
    // Scenario「set false 遇活躍 worktree 拒絕」：非零 exit、stderr 列 change 名／
    // 分支／路徑、config 位元組不變、足跡不動。
    let p = TempProject::new("set-wt-blocked", "schema: spec-driven\n");
    git_init(&p);
    let out = p.run(&["workflow-config", "set", "worktree", "true"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let wt = attach_worktree(&p, "add-auth");
    let before = p.config_bytes();

    let out = p.run(&["workflow-config", "set", "worktree", "false"]);

    assert!(!out.status.success(), "有活躍 worktree 時須以非零 exit 拒絕");
    let err = stderr_of(&out);
    assert!(err.contains("add-auth"), "stderr 須列 change 名: {err}");
    assert!(err.contains("speclink/add-auth"), "stderr 須列分支: {err}");
    // 路徑來自 `git worktree list`：git 在 Windows 以正斜線回報，且會把 %TMP% 的
    // 8.3 短名展開（RUNNERAD~1 → runneradmin），測試手上的 PathBuf 是反斜線＋短名。
    // 僅 Windows 走 canonicalize 展開短名並剝去 \\?\ 前綴（macOS 解 symlink 反而
    // 與 git 回報不同底）；拼法不是契約，指到同一個 worktree 才是。
    let wt = if cfg!(windows) { wt.canonicalize().expect("canonicalize worktree") } else { wt };
    let slashed = |s: &str| s.replace('\\', "/");
    let want = slashed(wt.to_str().unwrap());
    assert!(
        slashed(&err).contains(want.trim_start_matches("//?/")),
        "stderr 須列 worktree 路徑: {err}"
    );
    assert!(err.contains("worktree-merge"), "stderr 須指出收尾方式: {err}");
    assert_eq!(p.config_bytes(), before, "拒絕時 config.yaml 位元組不得變");
    for rel in GATED_SKILLS {
        assert!(dir_exists(&p, rel), "拒絕時足跡不得變動：{rel} 須仍在");
    }
}

#[test]
fn set_worktree_false_is_allowed_when_the_policy_is_already_off() {
    // 擋下的條件是「由 true 改 false」（spec 字面）——政策本來就關著時，
    // 技能本來就不在，殘留的 worktree 沒有「會被抽走收尾工具」的風險，
    // 這個 no-op 寫入不得被擋。
    let p = TempProject::new("set-wt-noop", "schema: spec-driven\n");
    git_init(&p);
    attach_worktree(&p, "add-auth");

    let out = p.run(&["workflow-config", "set", "worktree", "false"]);

    assert!(
        out.status.success(),
        "政策已關時 set false 為 no-op，不得被擋：{}",
        stderr_of(&out)
    );
    // 寫入預設值不落鍵（缺席＝false，既有政策寫入語意），故看 show 的正典值。
    let payload = json_of(&p.run(&["workflow-config", "show", "--json"]));
    assert_eq!(payload["worktree"], false, "值仍為 false: {payload}");
}

#[test]
fn a_failed_skill_sync_keeps_the_config_write_and_points_at_update() {
    // Scenario「技能同步失敗時 config 寫入仍成立」（design D2 的半套語意）：
    // config 為正典、exit code 非 0、stderr 指向重跑 speclink update。
    let p = TempProject::new("set-wt-sync-fail", "schema: spec-driven\n");
    // 技能目錄的位置擺一個同名檔案，讓足跡生成必敗。
    let blocker = p.dir.join(".claude").join("skills");
    std::fs::create_dir_all(blocker.parent().unwrap()).unwrap();
    std::fs::write(&blocker, "not a directory").unwrap();

    let out = p.run(&["workflow-config", "set", "worktree", "true"]);

    assert!(!out.status.success(), "同步失敗須以非零 exit 回報");
    let err = stderr_of(&out);
    assert!(err.contains("speclink update"), "stderr 須指向重跑 update: {err}");
    assert!(
        p.config_text().contains("worktree: true"),
        "config 已寫入為正典：{}",
        p.config_text()
    );
}

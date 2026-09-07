//! `speclink init` 的內建工具選集契約（spec「init 內建 Agent 工具選擇」、
//! 「remote 初始化與連接指令」與 design 的「CLI observable behavior」）。
//!
//! filesystem 與 Remote Store 兩條 init 共用同一個工具解析入口：顯式 `--tools`
//! 直接採用，缺旗標且 stdin 非互動終端時在任何寫入之前以非零 exit code 失敗。
//! `Command::output()` 的 stdin 是 /dev/null（非終端），因此本檔覆蓋的正是非互動
//! 路徑；互動 prompt 的單選／雙選／全否重試由 `verbs/init.rs` 的單元測試覆蓋。
//!
//! Credential isolation: 每次執行都把 USERPROFILE/HOME/XDG_CONFIG_HOME 指到
//! 拋棄式 "home"，測試絕不碰到真實使用者的憑證檔。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TempEnv {
    dir: PathBuf,
    home: PathBuf,
}

impl TempEnv {
    fn new(tag: &str) -> TempEnv {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-inittools-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        TempEnv { dir, home }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_WORKTREE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env_remove("CLICOLOR_FORCE")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("run speclink binary")
    }

    fn app_yaml(&self) -> serde_yaml::Value {
        let text = std::fs::read_to_string(self.dir.join(".speclink.yaml"))
            .expect(".speclink.yaml exists");
        serde_yaml::from_str(&text).expect(".speclink.yaml parses")
    }

    fn builtins(&self) -> Vec<String> {
        self.app_yaml()
            .get("tools")
            .and_then(|t| t.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn exists(&self, rel: &str) -> bool {
        self.dir.join(rel.split('/').collect::<PathBuf>()).exists()
    }

    /// 目錄的完整內容快照（相對路徑與檔案位元組），供「零寫入」斷言比對。
    fn snapshot(&self) -> Vec<(String, Vec<u8>)> {
        fn walk(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let rel = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
                if entry.path().is_dir() {
                    out.push((format!("{rel}/"), Vec::new()));
                    walk(&entry.path(), &rel, out);
                } else {
                    out.push((rel, std::fs::read(entry.path()).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.dir, "", &mut out);
        out
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

const URL: &str = "https://team.example.com/speclink/projects/foo";

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// 缺旗標與非法選集共用的錯誤形狀：單行 stderr、stdout 全空、exit code 非零。
fn assert_rejected_with_zero_writes(env: &TempEnv, out: &Output, before: &[(String, Vec<u8>)]) {
    assert!(!out.status.success(), "must exit non-zero");
    assert_eq!(stdout_of(out), "", "stdout must stay empty");
    assert_eq!(
        stderr_of(out).trim_end().lines().count(),
        1,
        "stderr must be a single line: {}",
        stderr_of(out)
    );
    assert_eq!(env.snapshot(), before, "no file may be created or modified");
}

// --- 顯式 --tools（spec Scenario「filesystem init 顯式選擇 Codex」） ---

#[test]
fn fs_init_explicit_tools_generate_exactly_the_selection() {
    // 三種顯式選法逐一：只有被選取的工具留下 Skills；指令檔一律不生成。
    let cases: [(&str, &[&str]); 3] = [
        ("claude", &["claude"]),
        ("codex", &["codex"]),
        ("claude,codex", &["claude", "codex"]),
    ];
    for (spec, want) in cases {
        let env = TempEnv::new(&format!("fs-explicit-{}", spec.replace(',', "-")));
        let out = env.run(&["init", "--tools", spec]);
        assert!(out.status.success(), "--tools {spec} stderr: {}", stderr_of(&out));

        let stdout = stdout_of(&out);
        assert!(stdout.contains("Initialized at"), "--tools {spec} stdout: {stdout}");
        assert!(stdout.contains("Generated files for:"), "--tools {spec} stdout: {stdout}");
        assert_eq!(env.builtins(), want, "--tools {spec}: recorded selection");

        let claude = want.contains(&"claude");
        assert_eq!(
            env.exists(".claude/skills/speclink-propose/SKILL.md"),
            claude,
            "--tools {spec}: Claude skills"
        );
        let codex = want.contains(&"codex");
        // spec Scenario「指令檔零受管區塊」：受管集合只剩技能檔。
        assert!(!env.exists("CLAUDE.md"), "--tools {spec}: CLAUDE.md 不得生成");
        assert!(!env.exists("AGENTS.md"), "--tools {spec}: AGENTS.md 不得生成");
        assert_eq!(
            env.exists(".agents/skills/speclink-propose/SKILL.md"),
            codex,
            "--tools {spec}: Codex skills"
        );
        assert!(env.exists("openspec/specs"), "--tools {spec}: filesystem spec tree");
    }
}

/// spec Scenario「Remote init 顯式選擇兩個工具」。
#[test]
fn remote_init_explicit_both_tools_writes_section_without_a_spec_tree() {
    let env = TempEnv::new("remote-explicit-both");
    let out = env.run(&[
        "init", "--store", "remote", "--url", URL, "--repo", "backend", "--tools", "claude,codex",
    ]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    assert_eq!(env.builtins(), vec!["claude", "codex"]);
    let yaml = env.app_yaml();
    let remote = yaml.get("remote").expect("remote section present");
    assert_eq!(remote.get("url").and_then(|v| v.as_str()), Some(URL));
    assert_eq!(remote.get("repo").and_then(|v| v.as_str()), Some("backend"));
    assert!(env.exists(".claude/skills/speclink-propose/SKILL.md"));
    assert!(env.exists(".agents/skills/speclink-propose/SKILL.md"));
    assert!(!env.exists("openspec"), "remote 模式不建立本機規格樹");
}

// --- 範本的政策示例（spec Scenario「新專案初始化的範本內容」） ---

/// 與 crates/speclink-cli/src/verbs/config.rs 的同名常數刻意重複（design D6：
/// 釘樁測試自帶清單才釘得住，且整合測試本就 import 不到 bin target 的私有項）。
/// 增刪政策鍵時兩處同步。
const POLICY_KEYS: [&str; 5] = ["locale", "spec_locale", "tdd", "audit", "worktree"];

const POLICY_ENV: [&str; 5] = [
    "SPECLINK_LOCALE",
    "SPECLINK_SPEC_LOCALE",
    "SPECLINK_TDD",
    "SPECLINK_AUDIT",
    "SPECLINK_WORKTREE",
];

/// 範本是否把 `key` 示範成一個鍵（`# key: …`）。以行為單位比對，`locale` 才不會
/// 被 `spec_locale` 這種較長的鍵名蒙混過關；縮排、註解符號與說明措辭都不參與。
fn documents_key(text: &str, key: &str) -> bool {
    text.lines().any(|line| {
        line.trim_start()
            .trim_start_matches('#')
            .trim_start()
            .starts_with(&format!("{key}:"))
    })
}

#[test]
fn init_workflow_config_template_documents_every_policy_key() {
    let env = TempEnv::new("template-policy");
    let out = env.run(&["init", "--tools", "claude"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));

    let workflow = std::fs::read_to_string(env.dir.join("openspec").join("config.yaml"))
        .expect("openspec/config.yaml exists");
    for key in POLICY_KEYS {
        assert!(documents_key(&workflow, key), "範本示範政策鍵 {key}:\n{workflow}");
    }
    for name in POLICY_ENV {
        assert!(workflow.contains(name), "範本的覆寫提示列出 {name}:\n{workflow}");
    }

    // 政策的正典歸屬只有 openspec/config.yaml 一處：.speclink.yaml 不得把任何
    // 政策鍵寫成鍵（含註解示例）。與上方同一把行首比對，措辭撞字不誤紅。
    let app = std::fs::read_to_string(env.dir.join(".speclink.yaml")).expect(".speclink.yaml exists");
    for key in POLICY_KEYS {
        assert!(!documents_key(&app, key), ".speclink.yaml 不得帶政策鍵 {key}:\n{app}");
    }
}

// --- 非互動缺少 --tools（spec Scenario「非互動 init 缺少 tools 零寫入失敗」） ---

#[test]
fn fs_init_without_tools_on_a_pipe_fails_with_zero_writes() {
    let env = TempEnv::new("fs-missing-tools");
    let before = env.snapshot();

    let out = env.run(&["init"]);

    assert_rejected_with_zero_writes(&env, &out, &before);
    let stderr = stderr_of(&out);
    for token in ["--tools", "claude", "codex"] {
        assert!(stderr.contains(token), "stderr must mention {token}: {stderr}");
    }
}

/// spec Scenario「Remote init 非互動缺少 tools 被拒」。
#[test]
fn remote_init_without_tools_on_a_pipe_fails_with_zero_writes() {
    let env = TempEnv::new("remote-missing-tools");
    let before = env.snapshot();

    let out = env.run(&["init", "--store", "remote", "--url", URL, "--repo", "backend"]);

    assert_rejected_with_zero_writes(&env, &out, &before);
    let stderr = stderr_of(&out);
    for token in ["--tools", "claude", "codex"] {
        assert!(stderr.contains(token), "stderr must mention {token}: {stderr}");
    }
}

// --- 空／未知的顯式值（spec Scenario「空或未知的顯式 tools 被拒」） ---

#[test]
fn explicit_empty_tools_value_is_rejected_with_zero_writes() {
    let env = TempEnv::new("empty-tools");
    let before = env.snapshot();

    let out = env.run(&["init", "--tools", ""]);

    assert_rejected_with_zero_writes(&env, &out, &before);
}

#[test]
fn explicit_unknown_tool_is_rejected_with_zero_writes() {
    let env = TempEnv::new("unknown-tool");
    let before = env.snapshot();

    let out = env.run(&["init", "--tools", "claude,vscode"]);

    assert_rejected_with_zero_writes(&env, &out, &before);
    assert!(
        stderr_of(&out).contains("vscode"),
        "stderr must name the offender: {}",
        stderr_of(&out)
    );
}

// --- --no-color（spec Scenario「no-color 不改變工具選擇語意」） ---

#[test]
fn no_color_init_emits_no_ansi_and_the_same_file_effects() {
    let colored = TempEnv::new("nocolor-baseline");
    let plain = TempEnv::new("nocolor-plain");

    let a = colored.run(&["init", "--tools", "claude"]);
    let b = plain.run(&["--no-color", "init", "--tools", "claude"]);

    assert!(a.status.success() && b.status.success());
    assert_eq!(a.status.code(), b.status.code(), "exit code identical");
    assert!(
        !stdout_of(&b).contains('\x1b') && !stderr_of(&b).contains('\x1b'),
        "--no-color output must carry no ANSI escape: {:?}",
        (stdout_of(&b), stderr_of(&b))
    );
    assert_eq!(plain.builtins(), colored.builtins(), "檔案效果與有色模式相同");
    assert_eq!(plain.exists("CLAUDE.md"), colored.exists("CLAUDE.md"));
    assert_eq!(
        plain.exists(".claude/skills/speclink-propose/SKILL.md"),
        colored.exists(".claude/skills/speclink-propose/SKILL.md")
    );
}

#[test]
fn force_reinit_with_a_new_selection_prunes_the_old_tool_and_keeps_the_two_line_summary() {
    // spec Scenario「init --force 切換工具時清除下架足跡」的 CLI 面：exit code 0、
    // stdout 仍是既有兩行，未選工具的技能目錄整組消失、選中工具補齊。
    let env = TempEnv::new("fs-force-switch");
    let first = env.run(&["init", "--tools", "claude"]);
    assert!(first.status.success(), "stderr: {}", stderr_of(&first));
    assert!(env.exists(".claude/skills/speclink-propose/SKILL.md"), "前置：Claude 技能已生成");

    let out = env.run(&["init", "--force", "--tools", "codex"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let stdout = stdout_of(&out);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "init 的摘要維持兩行，不新增清理明細：{stdout}");
    assert!(lines[0].contains("Initialized at"), "{stdout}");
    assert_eq!(lines[1], "Generated files for: codex", "{stdout}");
    assert_eq!(env.builtins(), ["codex"], "recorded selection");
    assert!(!env.exists(".claude/skills/speclink-propose/SKILL.md"), "未選工具的技能檔須移除");
    assert!(!env.exists(".claude"), "空掉的 .claude 目錄一併移除");
    assert!(env.exists(".agents/skills/speclink-propose/SKILL.md"), "選中工具補齊");
}

/// spec Scenario「削去後等同專案根的 skills_dir 被拒」與「與內建工具 skills 目錄相撞被拒」：
/// 描述子的目錄在驗證邊界擋下，update 零寫入、既有技能檔不動。
#[test]
fn update_rejects_a_descriptor_skills_dir_at_the_root_or_a_builtin_directory() {
    for (tag, bad, needle) in [
        ("root", "/", "project root"),
        ("builtin", ".claude/skills", "built-in"),
    ] {
        let env = TempEnv::new(&format!("descriptor-bad-dir-{tag}"));
        assert!(env.run(&["init", "--tools", "claude"]).status.success());
        std::fs::write(
            env.dir.join(".speclink.yaml"),
            format!("tools:\n  - claude\n  - name: wad-harness\n    skills_dir: {bad}\n"),
        )
        .unwrap();
        let before = env.snapshot();

        let out = env.run(&["update"]);

        assert!(!out.status.success(), "{tag}: must exit non-zero");
        let stderr = stderr_of(&out);
        assert!(stderr.contains("skills_dir"), "{tag}: must name the field: {stderr}");
        assert!(stderr.contains(needle), "{tag}: must give the reason: {stderr}");
        assert_eq!(env.snapshot(), before, "{tag}: 拒絕＝零寫入");
    }
}

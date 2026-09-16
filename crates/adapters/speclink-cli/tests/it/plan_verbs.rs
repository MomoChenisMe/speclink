//! `speclink plan` 與 `speclink change depends` 的 fs 模式契約（change-plan spec
//! 「plan 動詞輸出波次與阻擋清單」「change depends 動詞寫入宣告依賴」）。
//!
//! 兩條輸出路徑都是契約：`--json` 的 payload 形狀（camelCase、型別）與
//! `--no-color` 的人眼行；守門失敗一律 exit code 非零、meta 逐位元不變。

use std::path::PathBuf;
use std::process::{Command, Output};

const PROPOSED: &str = "schema: spec-driven\ncreated: 2026-09-01\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 空專案：只有 openspec/changes/ 目錄，change 由測試逐一放入。
    fn new(tag: &str) -> TempProject {
        let dir =
            std::env::temp_dir().join(format!("speclink-cli-plan-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        TempProject { dir }
    }

    fn change_dir(&self, name: &str) -> PathBuf {
        self.dir.join("openspec").join("changes").join(name)
    }

    /// 放入一個 change：meta 原文＋（可選）delta capability 名。
    fn put_change(&self, name: &str, meta: &str, caps: &[&str]) {
        let dir = self.change_dir(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".openspec.yaml"), meta).unwrap();
        for cap in caps {
            let spec = dir.join("specs").join(cap);
            std::fs::create_dir_all(&spec).unwrap();
            std::fs::write(spec.join("spec.md"), "## ADDED Requirements\n").unwrap();
        }
    }

    fn put_tasks(&self, name: &str, tasks_md: &str) {
        std::fs::write(self.change_dir(name).join("tasks.md"), tasks_md).unwrap();
    }

    fn archive(&self, dated_name: &str, meta: &str) {
        let dir = self
            .dir
            .join("openspec")
            .join("changes")
            .join("archive")
            .join(dated_name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".openspec.yaml"), meta).unwrap();
    }

    fn meta(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.change_dir(name).join(".openspec.yaml")).unwrap()
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_env(args, &[])
    }

    /// 同上，外加呼叫端指定的環境變數——宿主的 SPECLINK_*／色彩變數一律先清掉，
    /// 再套上測試要的那幾個。
    fn run_env(&self, args: &[&str], envs: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_STORE_URL",
            "SPECLINK_TOKEN",
            "SPECLINK_WORKTREE",
            "NO_COLOR",
            "CLICOLOR",
            "CLICOLOR_FORCE",
        ] {
            cmd.env_remove(key);
        }
        cmd.envs(envs.iter().copied());
        cmd.output().expect("run speclink binary")
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

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is JSON ({e}): {}", stdout_of(out)))
}

// --- plan：查詢動詞 ---

#[test]
fn plan_json_carries_the_four_keys_and_seven_camel_case_fields_per_change() {
    // Scenario JSON 輸出形狀：兩個提案中 change（b 依賴 a）。
    let p = TempProject::new("json-shape");
    p.put_change("a", PROPOSED, &["desktop-app"]);
    p.put_change(
        "b",
        "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n",
        &["desktop-app"],
    );
    let out = p.run(&["plan", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json = json_of(&out);
    let keys = |v: &serde_json::Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys(&json), ["changes", "next", "skipped", "waves"]);
    assert_eq!(json["skipped"], serde_json::json!([]));
    assert_eq!(json["next"], "a");
    assert_eq!(
        json["waves"],
        serde_json::json!([{ "index": 1, "changes": ["a"] }, { "index": 2, "changes": ["b"] }])
    );
    let changes = json["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 2);
    for c in changes {
        assert_eq!(
            keys(c),
            [
                "blockedBy",
                "dependsOn",
                "name",
                "overlaps",
                "ready",
                "stage",
                "wave"
            ]
        );
        assert!(c["name"].is_string() && c["wave"].is_u64() && c["stage"].is_string());
        assert!(c["dependsOn"].is_array() && c["overlaps"].is_array());
        assert!(c["blockedBy"].is_array() && c["ready"].is_boolean());
    }
    let b = &changes[1];
    assert_eq!(b["name"], "b");
    assert_eq!(b["stage"], "proposed");
    assert_eq!(b["wave"], 2);
    assert_eq!(b["dependsOn"], serde_json::json!(["a"]));
    assert_eq!(b["blockedBy"], serde_json::json!(["a"]));
    assert_eq!(b["ready"], false);
    assert_eq!(
        b["overlaps"],
        serde_json::json!([{ "change": "a", "capabilities": ["desktop-app"] }])
    );
}

#[test]
fn plan_human_output_prints_waves_and_next_without_ansi_under_no_color() {
    // Scenario 人眼輸出與 --no-color：CLICOLOR_FORCE 逼出有色路徑，--no-color 仍勝。
    let p = TempProject::new("human");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", "schema: spec-driven\ncreated: 2026-09-02\n", &[]);
    p.put_change(
        "c",
        "schema: spec-driven\ncreated: 2026-09-03\ndepends_on: a\n",
        &[],
    );
    let out = p.run_env(&["plan", "--no-color"], &[("CLICOLOR_FORCE", "1")]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(
        !text.contains('\u{1b}'),
        "no ANSI under --no-color: {text:?}"
    );
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "Wave 1 (parallel)", "got: {text}");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Wave 2") && !l.contains("parallel")),
        "got: {text}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("a") && l.contains("proposed")),
        "got: {text}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("c") && l.contains("blocked by: a")),
        "got: {text}"
    );
    assert_eq!(lines.last().copied(), Some("next: a"), "got: {text}");
}

#[test]
fn plan_refuses_a_dependency_cycle_with_an_empty_stdout() {
    // Scenario 依賴成環：a↔b，--json 與人眼皆非零、stdout 空。
    let p = TempProject::new("cycle");
    p.put_change("a", "schema: spec-driven\ndepends_on: b\n", &[]);
    p.put_change("b", "schema: spec-driven\ndepends_on: a\n", &[]);
    for args in [&["plan", "--json"][..], &["plan"][..]] {
        let out = p.run(args);
        assert!(!out.status.success(), "{args:?} must fail on a cycle");
        assert!(
            stderr_of(&out).contains("dependency cycle: a -> b -> a"),
            "{args:?} stderr: {}",
            stderr_of(&out)
        );
        assert!(
            out.stdout.is_empty(),
            "{args:?} stdout must be empty: {}",
            stdout_of(&out)
        );
    }
}

#[test]
fn plan_lists_corrupt_meta_under_skipped_and_still_plans_the_rest() {
    // Scenario 壞 metadata 列入 skipped。
    let p = TempProject::new("skipped");
    p.put_change("bad", ": : :\n\t bad yaml [unclosed\n", &["desktop-app"]);
    p.put_change("good", PROPOSED, &["desktop-app"]);
    let out = p.run(&["plan", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json = json_of(&out);
    assert_eq!(
        json["waves"],
        serde_json::json!([{ "index": 1, "changes": ["good"] }])
    );
    assert_eq!(json["changes"].as_array().unwrap().len(), 1);
    assert_eq!(json["changes"][0]["blockedBy"], serde_json::json!([]));
    let skipped = json["skipped"].as_array().unwrap();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0]["change"], "bad");
    assert!(skipped[0]["reason"].as_str().is_some_and(|r| !r.is_empty()));
    assert_eq!(json["next"], "good");
}

#[test]
fn plan_with_no_changes_yields_empty_waves_and_a_null_next() {
    let p = TempProject::new("empty");
    let out = p.run(&["plan", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json = json_of(&out);
    assert_eq!(json["waves"], serde_json::json!([]));
    assert_eq!(json["changes"], serde_json::json!([]));
    assert_eq!(json["next"], serde_json::Value::Null);
    assert_eq!(json["skipped"], serde_json::json!([]));
    let human = p.run(&["plan", "--no-color"]);
    assert!(human.status.success());
    assert!(
        stdout_of(&human).lines().any(|l| l == "next: none"),
        "got: {}",
        stdout_of(&human)
    );
}

#[test]
fn plan_reflects_stage_and_archived_prerequisites() {
    // 已就緒→進行中→提案中的基底順序；指向已封存的依賴視為滿足。
    let p = TempProject::new("stages");
    p.put_change("ready", "schema: spec-driven\ncreated: 2026-09-09\n", &[]);
    p.put_tasks("ready", "- [x] 1.1 a\n");
    p.put_change(
        "started",
        "schema: spec-driven\ncreated: 2026-09-05\nstarted_at: 2026-09-06\n",
        &[],
    );
    p.put_change(
        "later",
        "schema: spec-driven\ncreated: 2026-09-01\ndepends_on: old\n",
        &[],
    );
    p.archive("2026-08-01-old", PROPOSED);
    let json = json_of(&p.run(&["plan", "--json"]));
    let names: Vec<&str> = json["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["ready", "started", "later"]);
    let stages: Vec<&str> = json["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["stage"].as_str().unwrap())
        .collect();
    assert_eq!(stages, ["ready", "in-progress", "proposed"]);
    assert_eq!(json["changes"][2]["dependsOn"], serde_json::json!(["old"]));
    assert_eq!(json["changes"][2]["blockedBy"], serde_json::json!([]));
    assert_eq!(json["next"], "later");
}

// --- change depends：寫入動詞 ---

const STAMPED: &str = "schema: spec-driven\ncreated: 2026-09-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\n";

#[test]
fn depends_appends_one_line_and_keeps_every_other_byte() {
    // Scenario 寫入單一前置。
    let p = TempProject::new("depends-add");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", STAMPED, &[]);
    let out = p.run(&["change", "depends", "c", "--on", "a"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c depends on: a\n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: a\n").into_bytes()
    );
    assert_eq!(
        p.meta("a"),
        PROPOSED.as_bytes(),
        "the prerequisite's meta is untouched"
    );
}

#[test]
fn depends_appends_then_removes_within_the_comma_list() {
    // Scenario 追加與移除：`--on b` 後為 `a, b`，`--on a --remove` 後為 `b`。
    let p = TempProject::new("depends-list");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", PROPOSED, &[]);
    p.put_change("c", &format!("{STAMPED}depends_on: a\n"), &[]);
    let out = p.run(&["change", "depends", "c", "--on", "b"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c depends on: a, b\n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: a, b\n").into_bytes()
    );
    let out = p.run(&["change", "depends", "c", "--on", "a", "--remove"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c no longer depends on: a\n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: b\n").into_bytes()
    );
}

#[test]
fn depends_remove_to_empty_drops_the_whole_line() {
    // Scenario 移除至空即整行移除。
    let p = TempProject::new("depends-empty");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", &format!("{STAMPED}depends_on: a\n"), &[]);
    let out = p.run(&["change", "depends", "c", "--on", "a", "--remove"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.meta("c"), STAMPED.as_bytes());
}

#[test]
fn depends_guards_refuse_with_zero_writes() {
    // Scenario 守門拒絕零寫入：自己、不存在、已封存、成環，四次 meta 逐位元不變。
    let p = TempProject::new("depends-guards");
    p.put_change("a", &format!("{PROPOSED}depends_on: c\n"), &[]);
    p.put_change("c", STAMPED, &[]);
    p.archive("2026-08-01-old", PROPOSED);
    let cases: [(&[&str], &str); 4] = [
        (&["change", "depends", "c", "--on", "c"], "itself"),
        (
            &["change", "depends", "c", "--on", "ghost"],
            "no active change",
        ),
        (&["change", "depends", "c", "--on", "old"], "archived"),
        (
            &["change", "depends", "c", "--on", "a"],
            "dependency cycle: c -> a -> c",
        ),
    ];
    for (args, needle) in cases {
        let out = p.run(args);
        assert!(!out.status.success(), "{args:?} must refuse");
        assert!(
            stderr_of(&out).contains(needle),
            "{args:?} stderr: {}",
            stderr_of(&out)
        );
        assert!(out.stdout.is_empty(), "{args:?} stdout must be empty");
        assert_eq!(p.meta("c"), STAMPED.as_bytes(), "{args:?} must not write");
    }
    // 目標本身不是作用中 change：同樣拒絕。
    let out = p.run(&["change", "depends", "ghost", "--on", "a"]);
    assert!(!out.status.success());
    assert!(
        stderr_of(&out).contains("not found"),
        "stderr: {}",
        stderr_of(&out)
    );
}

#[test]
fn depends_re_adding_an_existing_edge_is_an_idempotent_success() {
    let p = TempProject::new("depends-idempotent");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", &format!("{STAMPED}depends_on: a\n"), &[]);
    let out = p.run(&["change", "depends", "c", "--on", "a"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c depends on: a\n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: a\n").into_bytes()
    );
}

#[test]
fn depends_keeps_crlf_line_endings() {
    // Scenario CRLF meta 檔保留行尾。
    let p = TempProject::new("depends-crlf");
    let crlf = STAMPED.replace('\n', "\r\n");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", &crlf, &[]);
    let out = p.run(&["change", "depends", "c", "--on", "a"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.meta("c"), format!("{crlf}depends_on: a\r\n").into_bytes());
}

#[test]
fn depends_json_reports_the_change_and_its_full_list() {
    let p = TempProject::new("depends-json");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", PROPOSED, &[]);
    p.put_change("c", &format!("{STAMPED}depends_on: a\n"), &[]);
    let out = p.run(&["change", "depends", "c", "--on", "b", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(
        json_of(&out),
        serde_json::json!({ "change": "c", "dependsOn": ["a", "b"] })
    );
}

#[test]
fn depends_writes_several_prerequisites_in_one_go() {
    // 多個 --on 於單次寫入完成。
    let p = TempProject::new("depends-multi");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", PROPOSED, &[]);
    p.put_change("c", STAMPED, &[]);
    let out = p.run(&["change", "depends", "c", "--on", "a", "b"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: a, b\n").into_bytes()
    );
    let json = json_of(&p.run(&["plan", "--json"]));
    assert_eq!(
        json["changes"][2]["blockedBy"],
        serde_json::json!(["a", "b"])
    );
}

#[test]
fn depends_matches_change_names_exactly_not_by_directory_probe() {
    // 守門以作用中 change 的名稱逐字比對：資料夾探針在大小寫不分的檔案系統
    // （macOS、Windows）會放行 `Add-Auth`，而 plan 只認 `add-auth`；空字串與
    // `archive`（changes/archive/ 永遠存在）同樣不是 change 名。四次皆零寫入。
    let p = TempProject::new("depends-exact-names");
    p.put_change("add-auth", PROPOSED, &[]);
    p.put_change("c", STAMPED, &[]);
    p.archive("2026-08-01-old", PROPOSED);
    let cases: [(&[&str], &str); 3] = [
        (
            &["change", "depends", "c", "--on", "Add-Auth"],
            "no active change",
        ),
        (&["change", "depends", "c", "--on", ""], "no active change"),
        (
            &["change", "depends", "c", "--on", "archive"],
            "no active change",
        ),
    ];
    for (args, needle) in cases {
        let out = p.run(args);
        assert!(!out.status.success(), "{args:?} must refuse");
        assert!(
            stderr_of(&out).contains(needle),
            "{args:?} stderr: {}",
            stderr_of(&out)
        );
        assert_eq!(p.meta("c"), STAMPED.as_bytes(), "{args:?} must not write");
    }
    // 目標名稱同樣逐字：`C` 不是 `c`。
    let out = p.run(&["change", "depends", "C", "--on", "add-auth"]);
    assert!(!out.status.success());
    assert!(
        stderr_of(&out).contains("Change 'C' not found."),
        "stderr: {}",
        stderr_of(&out)
    );
    assert_eq!(p.meta("c"), STAMPED.as_bytes());
    // 對照：逐字正確的名稱照常寫入。
    let out = p.run(&["change", "depends", "c", "--on", "add-auth"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: add-auth\n").into_bytes()
    );
}

#[test]
fn depends_remove_clears_a_prerequisite_that_is_archived_or_unknown() {
    // Scenario 移除指向已封存名稱的殘留項：前置封存後（或名稱已不存在）
    // `--remove` 照樣清掉，清空即整行移除；加邊仍照守門。
    let p = TempProject::new("depends-remove-residue");
    p.put_change("c", &format!("{STAMPED}depends_on: old, ghost\n"), &[]);
    p.archive("2026-08-01-old", PROPOSED);
    let out = p.run(&["change", "depends", "c", "--on", "old", "--remove"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c no longer depends on: old\n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: ghost\n").into_bytes()
    );
    let out = p.run(&["change", "depends", "c", "--on", "ghost", "--remove"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(p.meta("c"), STAMPED.as_bytes());
    let out = p.run(&["change", "depends", "c", "--on", "old"]);
    assert!(
        !out.status.success(),
        "adding an archived prerequisite still refuses"
    );
    assert!(
        stderr_of(&out).contains("already archived"),
        "stderr: {}",
        stderr_of(&out)
    );
    assert_eq!(p.meta("c"), STAMPED.as_bytes());
}

#[test]
fn list_and_status_json_ignore_depends_on_byte_for_byte() {
    // change-lifecycle Scenario depends_on 不影響既有輸出：同一 change 加／不加
    // `depends_on:` 行，list --json 與 status --change c --json 的 stdout 逐位元一致。
    let p = TempProject::new("list-status-unchanged");
    p.put_change("other", PROPOSED, &[]);
    p.put_change("c", STAMPED, &[]);
    p.put_tasks("c", "## 1. Group\n\n- [x] 1.1 first\n- [ ] 1.2 second\n");
    let before = (
        p.run(&["list", "--json"]),
        p.run(&["status", "--change", "c", "--json"]),
    );
    p.put_change("c", &format!("{STAMPED}depends_on: other\n"), &[]);
    let after = (
        p.run(&["list", "--json"]),
        p.run(&["status", "--change", "c", "--json"]),
    );
    for (label, a, b) in [
        ("list --json", &before.0, &after.0),
        ("status --json", &before.1, &after.1),
    ] {
        assert!(a.status.success() && b.status.success(), "{label} runs");
        assert!(!a.stdout.is_empty(), "{label} has output");
        assert_eq!(
            a.stdout, b.stdout,
            "{label} must not change with depends_on"
        );
        assert!(
            !stdout_of(b).contains("depends"),
            "{label} must not leak the field: {}",
            stdout_of(b)
        );
    }
}

#[test]
fn spec_example_soft_dependency_declared_after_propose() {
    // propose-skill Example「軟依賴落檔後的 plan 呈現」——兩列各一組值。
    // 列 1：add-b 建立在 add-a 的成果上 → change depends add-b --on add-a →
    // add-a 第 1 波、add-b 第 2 波。
    let p = TempProject::new("spec-example-propose");
    p.put_change("add-a", PROPOSED, &["cap-a"]);
    p.put_change(
        "add-b",
        "schema: spec-driven\ncreated: 2026-09-02\n",
        &["cap-b"],
    );
    let out = p.run(&["change", "depends", "add-b", "--on", "add-a"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json = json_of(&p.run(&["plan", "--json"]));
    assert_eq!(
        json["waves"],
        serde_json::json!([
            { "index": 1, "changes": ["add-a"] },
            { "index": 2, "changes": ["add-b"] }
        ])
    );
    // 列 2：add-a 與 add-c 無關 → 不下指令 → 同為第 1 波（可平行）。
    let p = TempProject::new("spec-example-propose-parallel");
    p.put_change("add-a", PROPOSED, &["cap-a"]);
    p.put_change(
        "add-c",
        "schema: spec-driven\ncreated: 2026-09-02\n",
        &["cap-c"],
    );
    let json = json_of(&p.run(&["plan", "--json"]));
    assert_eq!(
        json["waves"],
        serde_json::json!([{ "index": 1, "changes": ["add-a", "add-c"] }])
    );
}

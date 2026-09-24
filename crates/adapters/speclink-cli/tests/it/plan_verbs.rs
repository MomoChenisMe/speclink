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

    /// 寫入（或覆寫）一份 delta spec 的原文。
    fn put_delta(&self, name: &str, cap: &str, text: &str) {
        let spec = self.change_dir(name).join("specs").join(cap);
        std::fs::create_dir_all(&spec).unwrap();
        std::fs::write(spec.join("spec.md"), text).unwrap();
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

/// 一份只 MODIFIED 一個 requirement 的 delta 原文。
fn modified(requirement: &str) -> String {
    format!("## MODIFIED Requirements\n\n### Requirement: {requirement}\n\nbody\n")
}

#[test]
fn plan_json_has_nine_keys_and_requirement_overlap_shape() {
    // Scenario JSON 輸出形狀：兩個提案中 change（b 依賴 a），各自 MODIFIED
    // desktop-app 的同一個 requirement。
    let p = TempProject::new("json-shape");
    p.put_change("a", PROPOSED, &[]);
    p.put_change(
        "b",
        "schema: spec-driven\ncreated: 2026-09-02\ndepends_on: a\n",
        &[],
    );
    for name in ["a", "b"] {
        p.put_delta(name, "desktop-app", &modified("看板與任務"));
    }
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
                "archiveAfter",
                "blockedBy",
                "dependsOn",
                "name",
                "overlaps",
                "ready",
                "requirementOverlap",
                "stage",
                "wave"
            ]
        );
        assert!(c["name"].is_string() && c["wave"].is_u64() && c["stage"].is_string());
        assert!(c["dependsOn"].is_array() && c["overlaps"].is_array());
        assert!(c["blockedBy"].is_array() && c["ready"].is_boolean());
        assert!(c["requirementOverlap"].is_array() && c["archiveAfter"].is_array());
        for o in c["requirementOverlap"].as_array().unwrap() {
            assert_eq!(
                keys(o),
                [
                    "capability",
                    "change",
                    "conflict",
                    "otherOperation",
                    "ownOperation",
                    "requirement"
                ]
            );
            assert!(o["conflict"].is_boolean() && o["ownOperation"].is_string());
        }
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
    assert_eq!(b["archiveAfter"], serde_json::json!(["a"]));
    assert_eq!(
        b["requirementOverlap"],
        serde_json::json!([{
            "change": "a",
            "capability": "desktop-app",
            "requirement": "看板與任務",
            "ownOperation": "MODIFIED",
            "otherOperation": "MODIFIED",
            "conflict": false
        }])
    );
    assert_eq!(changes[0]["archiveAfter"], serde_json::json!([]));
}

#[test]
fn plan_strict_overlap_reproduces_old_waves() {
    // Scenario strict-overlap 退回目錄級推波：a、b 共用 desktop-app 但動到不同
    // requirement——預設同波；--strict-overlap 時 b 第 2 波、被 a 擋住。
    let p = TempProject::new("strict");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", "schema: spec-driven\ncreated: 2026-09-02\n", &[]);
    p.put_delta("a", "desktop-app", &modified("Run rewind point"));
    p.put_delta("b", "desktop-app", &modified("Conversation pin"));
    let default = json_of(&p.run(&["plan", "--json"]));
    assert_eq!(
        default["waves"],
        serde_json::json!([{ "index": 1, "changes": ["a", "b"] }])
    );
    let out = p.run(&["plan", "--strict-overlap", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let strict = json_of(&out);
    assert_eq!(
        strict["waves"],
        serde_json::json!([{ "index": 1, "changes": ["a"] }, { "index": 2, "changes": ["b"] }])
    );
    let b = &strict["changes"][1];
    assert_eq!(b["blockedBy"], serde_json::json!(["a"]));
    assert_eq!(b["ready"], false);
    assert_eq!(b["archiveAfter"], serde_json::json!([]));
    let human = stdout_of(&p.run(&["plan", "--strict-overlap", "--no-color"]));
    assert!(
        human.contains("Wave 2") && human.contains("blocked by: a"),
        "got: {human}"
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

/// 一份只 ADDED 一個 requirement 的 delta 原文。
fn added(requirement: &str) -> String {
    format!("## ADDED Requirements\n\n### Requirement: {requirement}\n\nbody\n")
}

/// 人眼輸出裡某個 change 那一行（去掉行首縮排）。
fn human_line(text: &str, name: &str) -> String {
    text.lines()
        .map(str::trim_start)
        .find(|l| l.starts_with(&format!("• {name} ")))
        .unwrap_or_else(|| panic!("no line for {name}: {text}"))
        .to_string()
}

#[test]
fn plan_human_output_orders_three_suffixes() {
    // Example「一個 change 三種尾註的順序」：e 依賴 a、與 b MODIFIED 同一個
    // requirement、與 c ADDED 同名 requirement。行首沿用既有 `• <名稱> [<階段>]`。
    let p = TempProject::new("three-suffixes");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", "schema: spec-driven\ncreated: 2026-09-02\n", &[]);
    p.put_change("c", "schema: spec-driven\ncreated: 2026-09-03\n", &[]);
    p.put_change(
        "e",
        "schema: spec-driven\ncreated: 2026-09-04\ndepends_on: a\n",
        &[],
    );
    p.put_delta("b", "desktop-app", &modified("看板與任務"));
    p.put_delta("c", "export", &added("匯出"));
    p.put_delta("e", "desktop-app", &modified("看板與任務"));
    p.put_delta("e", "export", &added("匯出"));
    let out = p.run_env(&["plan", "--no-color"], &[("CLICOLOR_FORCE", "1")]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let text = stdout_of(&out);
    assert!(!text.contains('\u{1b}'), "no ANSI under --no-color: {text:?}");
    assert_eq!(
        human_line(&text, "e"),
        "• e [proposed] — blocked by: a — archive after: b — conflicts with: c"
    );
    assert_eq!(human_line(&text, "c"), "• c [proposed] — conflicts with: e");
    assert_eq!(human_line(&text, "b"), "• b [proposed]", "b archives first: no suffix");
}

#[test]
fn plan_human_output_marks_archive_order_and_conflicts_without_blockers() {
    // Scenario 人眼輸出的封存順序與衝突尾註：b 的 archiveAfter 為 [a]、c 與 d 互為
    // ADDED 同名衝突，兩者都沒有宣告前置。
    let p = TempProject::new("archive-and-conflict");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("b", "schema: spec-driven\ncreated: 2026-09-02\n", &[]);
    p.put_change("c", "schema: spec-driven\ncreated: 2026-09-03\n", &[]);
    p.put_change("d", "schema: spec-driven\ncreated: 2026-09-04\n", &[]);
    p.put_delta("a", "desktop-app", &modified("看板與任務"));
    p.put_delta("b", "desktop-app", &modified("看板與任務"));
    p.put_delta("c", "export", &added("匯出"));
    p.put_delta("d", "export", &added("匯出"));
    let text = stdout_of(&p.run(&["plan", "--no-color"]));
    let b = human_line(&text, "b");
    let c = human_line(&text, "c");
    assert!(b.ends_with("archive after: a") && !b.contains("blocked by:"), "{b}");
    assert!(c.ends_with("conflicts with: d") && !c.contains("blocked by:"), "{c}");
    assert_eq!(text.lines().next(), Some("Wave 1 (parallel)"), "all four start together");
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

// --- change rank：看板順序鍵寫入動詞（spec「change rank 動詞寫入看板順序鍵」）---

/// meta 原文裡 `board_rank:` 那一行的值。
fn rank_line(meta: &[u8]) -> Option<String> {
    String::from_utf8_lossy(meta)
        .lines()
        .find_map(|l| l.strip_prefix("board_rank: ").map(str::to_string))
}

fn plan_names(p: &TempProject) -> Vec<String> {
    json_of(&p.run(&["plan", "--json"]))["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn change_rank_before_writes_board_rank_and_reorders_plan() {
    // Scenario 無 rank 的 change 排到另一個之前：欄內只有 a（rank n）與 c。
    const RANKED: &str = "schema: spec-driven\ncreated: 2026-09-01\nboard_rank: n\n";
    let p = TempProject::new("rank-before");
    p.put_change("a", RANKED, &[]);
    p.put_change("c", STAMPED, &[]);
    assert_eq!(plan_names(&p), ["a", "c"]);
    let out = p.run(&["change", "rank", "c", "--before", "a"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(stdout_of(&out), "✓ c ranked before a\n");
    let rank = rank_line(&p.meta("c")).expect("c gained a board_rank line");
    assert!(rank.as_str() < "n", "{rank} sorts before n");
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}board_rank: {rank}\n").into_bytes(),
        "every other byte is kept"
    );
    assert_eq!(p.meta("a"), RANKED.as_bytes(), "the anchor is untouched");
    assert_eq!(plan_names(&p), ["c", "a"]);
}

#[test]
fn change_rank_json_shape_has_no_rank_value() {
    let p = TempProject::new("rank-json");
    p.put_change("a", "schema: spec-driven\nboard_rank: n\n", &[]);
    p.put_change("c", PROPOSED, &[]);
    let out = p.run(&["change", "rank", "c", "--after", "a", "--json"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(
        json_of(&out),
        serde_json::json!({ "change": "c", "anchor": "a", "position": "after" })
    );
    // 寫入後 list --json 照舊不帶 rank（board_rank 不進 CLI 輸出）。
    let list = stdout_of(&p.run(&["list", "--json"]));
    assert!(!list.contains("board_rank") && !list.contains("boardRank"), "{list}");
}

#[test]
fn change_rank_refuses_already_ranked_and_force_overwrites() {
    // Scenario 已有 rank 未帶 --force 被拒／帶 --force 覆寫既有 rank。
    let p = TempProject::new("rank-force");
    p.put_change("a", "schema: spec-driven\nboard_rank: n\n", &[]);
    p.put_change("c", "schema: spec-driven\nboard_rank: b\n", &[]);
    let out = p.run(&["change", "rank", "c", "--after", "a"]);
    assert!(!out.status.success(), "an existing rank refuses without --force");
    let err = stderr_of(&out);
    assert!(
        err.contains("already has a board rank") && err.contains("--force"),
        "stderr: {err}"
    );
    assert!(out.stdout.is_empty());
    assert_eq!(p.meta("c"), b"schema: spec-driven\nboard_rank: b\n", "zero writes");
    let out = p.run(&["change", "rank", "c", "--after", "a", "--force"]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert!(rank_line(&p.meta("c")).unwrap().as_str() > "n");
    assert_eq!(plan_names(&p), ["a", "c"]);
}

#[test]
fn change_rank_refuses_dependency_cross() {
    // Scenario 跨宣告依賴被拒：c 依賴 a，c --before a → 兩份 meta 都不動。
    let p = TempProject::new("rank-dependency");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", &format!("{STAMPED}depends_on: a\n"), &[]);
    let out = p.run(&["change", "rank", "c", "--before", "a"]);
    assert!(!out.status.success(), "crossing a prerequisite refuses");
    let err = stderr_of(&out);
    assert!(
        err.contains("cannot move 'c' there") && err.contains("it depends on a"),
        "stderr: {err}"
    );
    assert_eq!(p.meta("a"), PROPOSED.as_bytes());
    assert_eq!(
        p.meta("c"),
        format!("{STAMPED}depends_on: a\n").into_bytes()
    );
}

#[test]
fn change_rank_refuses_different_stage() {
    // Scenario 不同階段被拒：a 進行中、c 提案中。
    let p = TempProject::new("rank-stage");
    const STARTED: &str = "schema: spec-driven\ncreated: 2026-09-01\nstarted_at: 2026-09-02\n";
    p.put_change("a", STARTED, &[]);
    p.put_change("c", PROPOSED, &[]);
    let out = p.run(&["change", "rank", "c", "--before", "a"]);
    assert!(!out.status.success(), "a rank only orders one stage");
    let err = stderr_of(&out);
    assert!(err.contains("in-progress") && err.contains("proposed"), "stderr: {err}");
    assert_eq!(p.meta("a"), STARTED.as_bytes());
    assert_eq!(p.meta("c"), PROPOSED.as_bytes());
}

#[test]
fn change_rank_needs_exactly_one_side() {
    // --before 與 --after 互斥且必擇一：兩者皆缺或皆給都是 argv 錯誤、零寫入。
    let p = TempProject::new("rank-side");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", PROPOSED, &[]);
    for args in [
        &["change", "rank", "c"][..],
        &["change", "rank", "c", "--before", "a", "--after", "a"][..],
    ] {
        let out = p.run(args);
        assert!(!out.status.success(), "{args:?} must be refused");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
    assert_eq!(p.meta("c"), PROPOSED.as_bytes());
}

#[test]
fn change_rank_remote_mode_refuses_without_request() {
    // Scenario remote 模式拒絕：只解析模式、不握手。listener 在線但從不回話——
    // 若發出任何請求，連線會躺在 backlog 裡被 accept() 撿到。
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let p = TempProject::new("rank-remote");
    p.put_change("a", PROPOSED, &[]);
    p.put_change("c", PROPOSED, &[]);
    std::fs::write(
        p.dir.join(".speclink.yaml"),
        format!("remote:\n  url: http://127.0.0.1:{port}/api/speclink/v1/projects/demo\n"),
    )
    .unwrap();
    let out = p.run(&["change", "rank", "c", "--before", "a"]);
    assert!(!out.status.success(), "remote mode refuses the verb");
    let err = stderr_of(&out);
    assert!(
        err.contains("change rank is not available in remote mode")
            && err.contains("the board order lives in the board resource"),
        "stderr: {err}"
    );
    assert!(out.stdout.is_empty());
    assert_eq!(p.meta("c"), PROPOSED.as_bytes(), "the local copy is untouched");
    listener.set_nonblocking(true).unwrap();
    match listener.accept() {
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
        other => panic!("no server request may be emitted, got {other:?}"),
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
    // 列 3：add-d 只有 5 個 task、add-a 有 40 個且無人依賴；add-a 已有 board_rank，
    // 所以排在 add-d 前面 → change rank add-d --before add-a → 同為第 1 波，
    // 配置順序 add-d 在前。
    queue_jump_example("spec-example-propose-rank", ("add-a", 40), ("add-d", 5));
}

/// `n` 個未勾選任務的 tasks.md。
fn open_tasks(n: usize) -> String {
    (1..=n).map(|i| format!("- [ ] 1.{i} t\n")).collect()
}

/// propose／ingest 的插隊例子：`big` 已有 board_rank（使用者拖排過，所以排在前面），
/// 小的 `small` 沒有 rank——技能執行 `change rank <small> --before <big>` 後兩者
/// 同為第 1 波、small 在前。兩者都沒有 rank 時新 tie-break 本就讓小的在前，
/// 技能不會插隊，所以這個前提是例子的一部分。
fn queue_jump_example(tag: &str, big: (&str, usize), small: (&str, usize)) {
    let p = TempProject::new(tag);
    p.put_change(big.0, "schema: spec-driven\ncreated: 2026-09-01\nboard_rank: n\n", &[]);
    p.put_tasks(big.0, &open_tasks(big.1));
    p.put_change(small.0, "schema: spec-driven\ncreated: 2026-09-20\n", &[]);
    p.put_tasks(small.0, &open_tasks(small.1));
    assert_eq!(plan_names(&p), [big.0, small.0], "the ranked big change sits in front");
    let out = p.run(&["change", "rank", small.0, "--before", big.0]);
    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    let json = json_of(&p.run(&["plan", "--json"]));
    assert_eq!(
        json["waves"],
        serde_json::json!([{ "index": 1, "changes": [small.0, big.0] }])
    );
}

#[test]
fn spec_example_ingest_shrunk_change_jumps_the_queue() {
    // ingest-skill Example「中途改需求後新增前置」列 5：add-b（提案中、無 rank）縮到
    // 4 個 task，add-a 有 30 個且無人依賴、已有 board_rank → change rank add-b
    // --before add-a → 兩者同波，配置順序 add-b 在前。
    queue_jump_example("spec-example-ingest-rank", ("add-a", 30), ("add-b", 4));
}

#[test]
fn change_rank_refuses_a_change_without_metadata_before_any_write() {
    // 缺 .openspec.yaml 的 change 仍在 plan 裡，但沒有地方寫 rank：不論它是被移動
    // 的那個、還是欄內要補章的那個，都在第一筆寫入前拒絕，零寫入。
    let p = TempProject::new("rank-no-meta");
    p.put_change("p1", "schema: spec-driven\ncreated: 2026-09-01\n", &[]);
    std::fs::create_dir_all(p.change_dir("p2")).unwrap();
    std::fs::write(p.change_dir("p2").join("proposal.md"), "## Why\nx\n").unwrap();
    p.put_change("p3", "schema: spec-driven\ncreated: 2026-09-03\n", &[]);
    assert_eq!(plan_names(&p), ["p1", "p3", "p2"], "the metadata-less change is planned");

    let out = p.run(&["change", "rank", "p2", "--before", "p1"]);
    assert!(!out.status.success(), "the moved change has no metadata");
    let err = stderr_of(&out);
    assert!(err.contains("openspec/changes/p2/.openspec.yaml is missing"), "stderr: {err}");

    let out = p.run(&["change", "rank", "p3", "--before", "p1"]);
    assert!(!out.status.success(), "a change to stamp has no metadata");
    let err = stderr_of(&out);
    assert!(err.contains("openspec/changes/p2/.openspec.yaml is missing"), "stderr: {err}");

    for change in ["p1", "p3"] {
        assert_eq!(rank_line(&p.meta(change)), None, "{change} is not stamped");
    }
    assert!(!p.change_dir("p2").join(".openspec.yaml").exists());
}

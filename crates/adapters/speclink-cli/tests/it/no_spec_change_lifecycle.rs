//! 內建 schema `no-spec-change` 的變更生命週期 — fs 模式契約（spec workflow-schemas
//! 「no-spec-change 內建 schema 的組成」）。
//!
//! 一個變更走完 new → status → instructions → validate → archive：沒有 specs artifact、
//! proposal 範本沒有 Capabilities、validate 不警告缺 delta、封存不需要 `--skip-specs`
//! 且正式規格不變。user 層 schema 位置導進暫存專案，開發者的 home 不影響結果。

use std::path::PathBuf;
use std::process::{Command, Output};

const CANON: &str = "# demo Specification\n\n## Purpose\n\nDemo capability used as the untouched canonical spec in this test.\n\n## Requirements\n\n### Requirement: Demo works\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-no-spec-change-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("speclink").join("specs").join("demo")).unwrap();
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        std::fs::write(dir.join("speclink").join("specs").join("demo").join("spec.md"), CANON)
            .unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env("NO_COLOR", "1")
            .env("HOME", self.dir.join("home"))
            .env("USERPROFILE", self.dir.join("home"))
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn change_dir(&self) -> PathBuf {
        self.dir.join("speclink").join("changes").join("tidy")
    }

    /// 正式規格目錄的全部檔案（相對路徑, 內容），排序後供前後比對。
    fn canonical_snapshot(&self) -> Vec<(String, String)> {
        let root = self.dir.join("speclink").join("specs");
        let mut out = Vec::new();
        for cap in std::fs::read_dir(&root).unwrap().flatten() {
            for f in std::fs::read_dir(cap.path()).unwrap().flatten() {
                let rel = format!(
                    "{}/{}",
                    cap.file_name().to_string_lossy(),
                    f.file_name().to_string_lossy()
                );
                out.push((rel, std::fs::read_to_string(f.path()).unwrap()));
            }
        }
        out.sort();
        out
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn ok(out: &Output, what: &str) -> String {
    assert!(
        out.status.success(),
        "{what} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn a_no_spec_change_change_runs_from_new_to_archive_without_touching_specs() {
    let p = TempProject::new("lifecycle");
    let before = p.canonical_snapshot();

    // 建立：schemaName、artifact 序列與 applyRequires 都來自內建定義，沒有 specs。
    ok(&p.run(&["new", "change", "tidy", "--schema", "no-spec-change"]), "new change");
    let status: serde_json::Value =
        serde_json::from_str(&ok(&p.run(&["status", "--change", "tidy", "--json"]), "status"))
            .expect("status --json");
    assert_eq!(status["schemaName"], "no-spec-change");
    let ids: Vec<&str> = status["artifacts"]
        .as_array()
        .expect("artifacts")
        .iter()
        .filter_map(|a| a["id"].as_str())
        .collect();
    assert_eq!(ids, ["proposal", "design", "tasks"]);
    assert_eq!(status["applyRequires"], serde_json::json!(["tasks"]));

    // proposal 範本：有 Why／What Changes／Impact，沒有 Capabilities。
    let proposal: serde_json::Value = serde_json::from_str(&ok(
        &p.run(&["instructions", "proposal", "--change", "tidy", "--json"]),
        "instructions proposal",
    ))
    .expect("instructions --json");
    let template = proposal["template"].as_str().expect("template");
    for section in ["## Why", "## What Changes", "## Impact"] {
        assert!(template.contains(section), "template 須含 {section}：{template}");
    }
    assert!(!template.contains("## Capabilities"), "template 不得含 Capabilities：{template}");
    let instruction = proposal["instruction"].as_str().expect("instruction");
    assert!(
        instruction.contains("list the canonical specs you checked"),
        "instruction 要求 Why 列出查過的正式規格：{instruction}"
    );

    // validate：沒有 delta 也不警告。
    std::fs::write(
        p.change_dir().join("proposal.md"),
        "## Why\n\nRename an internal helper. It alters no behavior the canonical specs promise; checked: demo.\n\n## What Changes\n\n- Rename the helper.\n\n## Impact\n\n- src/helper.rs\n",
    )
    .unwrap();
    std::fs::write(
        p.change_dir().join("tasks.md"),
        "## 1. Refactor\n\n- [x] 1.1 Rename the helper\n- [x] 1.2 Run the existing tests to confirm the demo behavior is unchanged\n",
    )
    .unwrap();
    let validated = p.run(&["validate", "tidy", "--json"]);
    let validated = ok(&validated, "validate");
    assert!(!validated.contains("No delta specs found"), "不得警告缺 delta：{validated}");
    let human = ok(&p.run(&["validate", "tidy"]), "validate (human)");
    assert!(!human.contains("No delta specs found"), "人眼輸出也不警告：{human}");
    // 同一專案沒有 delta 的 spec-driven 變更照現行輸出該警告。
    ok(&p.run(&["new", "change", "other"]), "new change other");
    let other = ok(&p.run(&["validate", "other", "--json"]), "validate other");
    assert!(other.contains("No delta specs found"), "spec-driven 照舊警告：{other}");

    // 封存：不帶 --skip-specs 也成功，正式規格逐檔不變。
    ok(&p.run(&["archive", "tidy"]), "archive");
    assert!(!p.change_dir().exists(), "變更移出 changes/");
    let archived = std::fs::read_dir(p.dir.join("speclink").join("changes").join("archive"))
        .expect("archive dir")
        .flatten()
        .any(|e| e.file_name().to_string_lossy().ends_with("-tidy"));
    assert!(archived, "變更移入封存目錄");
    assert_eq!(p.canonical_snapshot(), before, "speclink/specs/ 不得有任何變動");
}

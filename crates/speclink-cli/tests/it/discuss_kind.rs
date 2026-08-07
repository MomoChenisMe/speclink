//! CLI wiring for `discuss new --kind` (change add-improve-flow): the flag is a
//! whitelist (only `improve`), a legal value stamps the record's frontmatter and
//! rides the `--json` payload, an illegal one fails loudly without touching the
//! filesystem, and omitting the flag leaves both output paths exactly as they were.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A throwaway project with an empty discussions directory.
    fn empty(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-discuss-kind-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("discussions")).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            // Plain output must be deterministic regardless of the host shell.
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn discussions(&self) -> PathBuf {
        self.dir.join("openspec").join("discussions")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).expect("json stdout")
}

#[test]
fn discuss_new_with_kind_improve_stamps_frontmatter_and_json() {
    let p = TempProject::empty("improve");
    let out = p.run(&["discuss", "new", "核心結構改進", "--slug", "improve-core", "--kind", "improve"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let text = std::fs::read_to_string(p.discussions().join("improve-core.md")).expect("record");
    assert!(text.contains("\nkind: improve\n"), "frontmatter: {text}");

    let out = p.run(&["discuss", "show", "improve-core", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&out);
    assert_eq!(v["info"]["kind"], "improve", "show --json exposes kind");
}

#[test]
fn discuss_new_with_kind_json_reports_kind_as_a_string() {
    let p = TempProject::empty("json");
    let out = p.run(&[
        "discuss", "new", "核心結構改進", "--slug", "improve-x", "--kind", "improve", "--json",
    ]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&out);
    assert_eq!(v["kind"], "improve", "camelCase single-word key, string value");
    assert!(v["kind"].is_string(), "kind is a string: {}", v["kind"]);
}

#[test]
fn discuss_new_with_kind_keeps_the_three_line_creation_message() {
    // 人眼輸出沿用既有建立訊息格式，不因 kind 新增行。
    let p = TempProject::empty("human");
    let marked = p.run(&["discuss", "new", "主題", "--slug", "improve-a", "--kind", "improve"]);
    let plain = p.run(&["discuss", "new", "主題", "--slug", "plain-a"]);
    assert!(marked.status.success(), "stderr: {}", String::from_utf8_lossy(&marked.stderr));
    assert!(plain.status.success(), "stderr: {}", String::from_utf8_lossy(&plain.stderr));
    let marked_lines: Vec<_> = String::from_utf8_lossy(&marked.stdout).lines().map(str::to_string).collect();
    let plain_lines: Vec<_> = String::from_utf8_lossy(&plain.stdout).lines().map(str::to_string).collect();
    assert_eq!(marked_lines.len(), plain_lines.len(), "same line count: {marked_lines:?}");
    assert!(marked_lines[0].contains("Created discussion: improve-a"), "{marked_lines:?}");
    assert!(marked_lines[1].starts_with("  Topic: "), "{marked_lines:?}");
    assert!(marked_lines[2].starts_with("  Path: "), "{marked_lines:?}");
}

#[test]
fn discuss_new_rejects_kind_outside_the_whitelist_without_writing() {
    for (tag, bad) in [("refactor", "refactor"), ("case", "IMPROVE")] {
        let p = TempProject::empty(tag);
        let out = p.run(&["discuss", "new", "主題", "--slug", "alpha", "--kind", bad]);
        assert!(!out.status.success(), "illegal kind must exit non-zero ({bad})");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("improve"), "stderr names the only legal value: {stderr}");
        let leftover: Vec<_> = std::fs::read_dir(p.discussions()).unwrap().collect();
        assert!(leftover.is_empty(), "no record may be created: {leftover:?}");
    }
}

#[test]
fn discuss_new_without_kind_omits_the_key_everywhere() {
    let p = TempProject::empty("absent");
    let out = p.run(&["discuss", "new", "主題", "--slug", "plain-b", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&out);
    assert!(v.get("kind").is_none(), "no kind key without the flag: {v}");
    let text = std::fs::read_to_string(p.discussions().join("plain-b.md")).expect("record");
    assert!(!text.contains("kind:"), "frontmatter untouched: {text}");
}

#[test]
fn discuss_list_and_show_json_expose_kind_only_when_present() {
    let p = TempProject::empty("list");
    assert!(p
        .run(&["discuss", "new", "改進", "--slug", "improve-b", "--kind", "improve"])
        .status
        .success());
    assert!(p.run(&["discuss", "new", "一般", "--slug", "plain-c"]).status.success());

    let out = p.run(&["discuss", "list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v = json_of(&out);
    let items = v["discussions"].as_array().expect("discussions array");
    let marked = items.iter().find(|d| d["slug"] == "improve-b").expect("marked record listed");
    let plain = items.iter().find(|d| d["slug"] == "plain-c").expect("plain record listed");
    assert_eq!(marked["kind"], "improve");
    assert!(plain.get("kind").is_none(), "plain record omits the key: {plain}");

    let out = p.run(&["discuss", "show", "plain-c", "--json"]);
    let v = json_of(&out);
    assert!(v["info"].get("kind").is_none(), "show omits the key too: {}", v["info"]);
}

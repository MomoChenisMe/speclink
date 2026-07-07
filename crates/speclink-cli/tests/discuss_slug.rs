//! CLI wiring for `discuss new --slug` (change discuss-english-slug): the
//! override names the record file and frontmatter slug while the topic stays
//! verbatim; invalid values must fail loudly without touching the filesystem;
//! the no-flag fallback derivation is unchanged.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A throwaway project with an empty discussions directory.
    fn empty(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-discuss-slug-{tag}-{}",
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

#[test]
fn discuss_new_with_slug_override_names_file_and_keeps_topic() {
    let p = TempProject::empty("valid");
    let out = p.run(&["discuss", "new", "看板搜尋列", "--slug", "board-search-bar"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("board-search-bar"), "stdout: {stdout}");
    let file = p.discussions().join("board-search-bar.md");
    let text = std::fs::read_to_string(&file).expect("record exists under override slug");
    assert!(text.contains("slug: board-search-bar\n"), "text: {text}");
    assert!(text.contains("topic: 看板搜尋列\n"), "text: {text}");
}

#[test]
fn discuss_new_with_slug_json_reports_override_and_topic() {
    let p = TempProject::empty("json");
    let out = p.run(&["discuss", "new", "看板搜尋列", "--slug", "board-x", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json stdout");
    assert_eq!(v["slug"], "board-x");
    assert_eq!(v["topic"], "看板搜尋列");
}

#[test]
fn discuss_new_rejects_invalid_slug_without_writing() {
    let p = TempProject::empty("invalid");
    let out = p.run(&["discuss", "new", "主題", "--slug", "Bad_Slug"]);
    assert!(!out.status.success(), "invalid slug must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("kebab-case"), "stderr: {stderr}");
    let leftover: Vec<_> = std::fs::read_dir(p.discussions()).unwrap().collect();
    assert!(leftover.is_empty(), "no record may be created: {leftover:?}");
}

#[test]
fn discuss_new_without_slug_derives_from_topic_as_before() {
    let p = TempProject::empty("fallback");
    let out = p.run(&["discuss", "new", "Board Search"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(p.discussions().join("board-search.md").exists(), "derived filename unchanged");
}

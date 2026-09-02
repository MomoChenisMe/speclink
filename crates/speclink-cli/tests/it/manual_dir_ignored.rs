//! `openspec/manual/` 對 `speclink list --json` 與 `speclink validate --specs` 無感
//! （spec manual-pages「手冊頁的落點與檔名」；design「手冊落點 openspec/manual/ 且
//! list 與 validate 無感」）。
//!
//! 手冊是規格的衍生物，與源頭同住 `openspec/`；引擎動詞不得因多出這個目錄而改變
//! 輸出。以 `speclink init` 建工作區、放一個 change 與一份正典規格後，先記錄兩指令
//! 的 stdout 與 exit code，再放入一頁合規 frontmatter 六欄的 `index.md` 重跑，
//! stdout 逐位元比對、exit code 皆為 0。
//!
//! Credential isolation: 每次執行都把 USERPROFILE/HOME/XDG_CONFIG_HOME 指到
//! 拋棄式 "home"，測試絕不碰到真實使用者的憑證檔。

use std::path::PathBuf;
use std::process::{Command, Output};

const META: &str = "schema: spec-driven\ncreated: 2026-09-01\n";
const GOOD_PURPOSE: &str =
    "本 capability 負責搜尋結果的排序與分頁，涵蓋查詢改寫、排序權重與空結果的可觀察行為。";
/// manual-pages 契約的 frontmatter 六欄（首頁的 `sources` 得為空陣列）＋出處行。
const INDEX_PAGE: &str = "---\n\
title: 操作手冊\n\
section: 開始使用\n\
order: 10\n\
keywords: [手冊, 入門]\n\
sources: []\n\
generated: 2026-09-02\n\
---\n\
\n\
# 操作手冊\n\
\n\
一句話定位系統。\n\
\n\
**出處**：\n";

struct TempEnv {
    dir: PathBuf,
    home: PathBuf,
}

impl TempEnv {
    /// `speclink init --tools claude` 後補一個 change `demo` 與一份正典規格 `search`，
    /// 讓 list 與 validate 的輸出都非空。
    fn new(tag: &str) -> TempEnv {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-manual-dir-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let env = TempEnv { dir, home };

        let init = env.run(&["init", "--tools", "claude"]);
        assert!(init.status.success(), "init stderr: {}", stderr_of(&init));

        let change = env.dir.join("openspec").join("changes").join("demo");
        std::fs::create_dir_all(&change).unwrap();
        std::fs::write(change.join(".openspec.yaml"), META).unwrap();
        std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
        std::fs::write(change.join("tasks.md"), "- [ ] 1.1 a\n").unwrap();

        let spec = env.dir.join("openspec").join("specs").join("search");
        std::fs::create_dir_all(&spec).unwrap();
        std::fs::write(
            spec.join("spec.md"),
            format!("# search Specification\n\n## Purpose\n\n{GOOD_PURPOSE}\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n"),
        )
        .unwrap();
        env
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

    /// 放入一頁合規的手冊首頁（目錄不存在時建立），回傳其路徑。
    fn write_manual_index(&self) -> PathBuf {
        let manual = self.dir.join("openspec").join("manual");
        std::fs::create_dir_all(&manual).unwrap();
        let index = manual.join("index.md");
        std::fs::write(&index, INDEX_PAGE).unwrap();
        index
    }
}

impl Drop for TempEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// 兩次執行的 stdout 逐位元一致且 exit code 皆為 0。
fn assert_same_success(before: &Output, after: &Output, verb: &str) {
    assert!(before.status.success(), "{verb} 無手冊目錄時零收尾: {}", stderr_of(before));
    assert!(after.status.success(), "{verb} 有手冊目錄時零收尾: {}", stderr_of(after));
    assert_eq!(
        String::from_utf8_lossy(&after.stdout),
        String::from_utf8_lossy(&before.stdout),
        "{verb} 的 stdout 不得因 openspec/manual/ 出現而改變"
    );
}

#[test]
fn list_json_is_unchanged_by_the_manual_dir() {
    let env = TempEnv::new("list");
    let before = env.run(&["list", "--json"]);
    assert!(
        String::from_utf8_lossy(&before.stdout).contains("demo"),
        "對照組必須非空，否則比對無意義"
    );

    let index = env.write_manual_index();
    assert!(index.is_file(), "手冊首頁落在 openspec/manual/index.md");

    let after = env.run(&["list", "--json"]);
    assert_same_success(&before, &after, "list --json");
}

#[test]
fn validate_specs_is_unchanged_by_the_manual_dir() {
    let env = TempEnv::new("validate");
    let before = env.run(&["validate", "--specs", "--no-color"]);
    assert!(
        String::from_utf8_lossy(&before.stdout).contains("search"),
        "對照組必須點名正典規格，否則比對無意義"
    );

    env.write_manual_index();

    let after = env.run(&["validate", "--specs", "--no-color"]);
    assert_same_success(&before, &after, "validate --specs");
}

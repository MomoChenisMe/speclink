//! `speclink update` 的降級守門（spec「update 動詞的降級守門」）。
//!
//! 工作區指令檔的標記版號領先引擎時，update 會把它們改寫回舊內容——2026-08-05
//! 的實際事故。守門在任何寫入之前探測方向，領先即拒絕；`--allow-downgrade`
//! 是唯一的越過方式（不共用 `--force`，避免慣性帶旗標把守門靜默穿透）。
//!
//! Credential isolation: 每次執行都把 USERPROFILE/HOME/XDG_CONFIG_HOME 指到
//! 拋棄式 "home"，測試絕不碰到真實使用者的憑證檔。

use speclink_core::init::MARKER_VERSION;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TempEnv {
    dir: PathBuf,
    home: PathBuf,
}

impl TempEnv {
    /// 以 `init --tools claude` 生成一份現版工作區。
    fn new(tag: &str) -> TempEnv {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-downgrade-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        let env = TempEnv { dir, home };
        let out = env.run(&["init", "--tools", "claude"]);
        assert!(out.status.success(), "init stderr: {}", stderr_of(&out));
        env
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env_remove("CLICOLOR_FORCE")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("run speclink binary")
    }

    /// 把工作區的標記版號改成指定值（模擬由別版引擎生成的工作區）。
    fn set_marker(&self, version: &str) {
        let md = self.dir.join("CLAUDE.md");
        let text = std::fs::read_to_string(&md).expect("CLAUDE.md exists");
        std::fs::write(&md, text.replace(MARKER_VERSION, version)).unwrap();
    }

    fn marker_version(&self) -> String {
        let text = std::fs::read_to_string(self.dir.join("CLAUDE.md")).expect("CLAUDE.md exists");
        let start = text.find("<!-- SPECLINK:START").expect("marker present")
            + "<!-- SPECLINK:START".len();
        let rest = &text[start..];
        rest[..rest.find("-->").expect("marker closes")].trim().to_string()
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

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// 比現版領先一個主版號的標記版號。
fn ahead_of_current() -> String {
    let major: u64 = MARKER_VERSION
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .expect("MARKER_VERSION 主版號可解析");
    format!("v{}.0.0", major + 1)
}

/// Scenario「較新工作區拒絕 update」。
#[test]
fn a_newer_workspace_is_refused_with_zero_writes() {
    let env = TempEnv::new("refused");
    let ahead = ahead_of_current();
    env.set_marker(&ahead);
    let before = env.snapshot();

    let out = env.run(&["update"]);

    assert!(!out.status.success(), "must exit non-zero");
    let stderr = stderr_of(&out);
    assert_eq!(
        stderr.trim_end().lines().count(),
        1,
        "stderr must be a single line: {stderr}"
    );
    assert!(stderr.contains(&ahead), "stderr must name the workspace version: {stderr}");
    assert!(
        stderr.contains(MARKER_VERSION),
        "stderr must name the engine version: {stderr}"
    );
    assert_eq!(env.snapshot(), before, "no file may be created or modified");
}

/// Scenario「--allow-downgrade 越過守門」。
#[test]
fn allow_downgrade_regenerates_at_the_engine_version() {
    let env = TempEnv::new("allowed");
    env.set_marker(&ahead_of_current());

    let out = env.run(&["update", "--allow-downgrade"]);

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(env.marker_version(), MARKER_VERSION, "受管檔須再生為引擎現版");
}

/// Scenario「過期工作區照常更新」：守門引入前後行為相同。
#[test]
fn a_stale_workspace_still_updates_without_the_flag() {
    let env = TempEnv::new("stale");
    env.set_marker("v0.9.0");

    let out = env.run(&["update"]);

    assert!(out.status.success(), "stderr: {}", stderr_of(&out));
    assert_eq!(env.marker_version(), MARKER_VERSION, "受管檔須再生為引擎現版");
}

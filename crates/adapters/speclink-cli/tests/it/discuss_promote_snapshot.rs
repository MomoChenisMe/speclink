//! Baseline pin for the promote-flow sink (design D1/D2): `discuss promote`
//! (default-name and --name forms) and `discuss list --json` outputs are
//! recorded verbatim BEFORE the flow moves into speclink-core, and must stay
//! bit-identical after. Discussion verbs are speclink-specific, so this
//! self-baseline is the regression guard.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A throwaway project with one concluded discussion per given slug
    /// (fixed `created` date so list output is deterministic).
    fn with_discussions(tag: &str, slugs: &[(&str, &str)]) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-promote-snap-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let discussions = dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        // macOS 的 temp_dir 在 /var → /private/var symlink 下，CLI 由 getcwd 回報實體
        // 路徑，期望字串必須同底才能逐位元比對；Windows 的 canonicalize 會加 \\?\ 前綴
        // 反而與 CLI 輸出不符，故僅在非 Windows 平台解析。
        let dir = if cfg!(windows) { dir } else { dir.canonicalize().unwrap() };
        for (slug, topic) in slugs {
            let doc = format!(
                "---\n\
                 topic: {topic}\n\
                 slug: {slug}\n\
                 status: concluded\n\
                 created: 2026-01-02\n\
                 ---\n\
                 \n\
                 # Discussion: {topic}\n\
                 \n\
                 ## Context\n\
                 \n\
                 Fixture context.\n\
                 \n\
                 ## Rounds\n\
                 \n\
                 ### Round 1 — assumptions (2026-01-02)\n\
                 \n\
                 **Focus**: scope\n\
                 \n\
                 ## Conclusion\n\
                 \n\
                 **Decision**: build {slug}\n"
            );
            std::fs::write(discussions.join(format!("{slug}.md")), doc).unwrap();
        }
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

    fn change_dir(&self, name: &str) -> PathBuf {
        self.dir.join("openspec").join("changes").join(name)
    }

    fn slash(p: &std::path::Path) -> String {
        p.to_string_lossy().replace('\\', "/")
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

#[test]
fn promote_default_name_stdout_is_pinned() {
    let p = TempProject::with_discussions("default", &[("alpha-search", "Alpha search")]);
    let out = p.run(&["discuss", "promote", "alpha-search"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let expected = format!(
        "✓ Promoted discussion 'alpha-search' → change 'alpha-search'\n\
         \x20 Path: {}\n\
         \x20 Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts\n",
        p.change_dir("alpha-search").to_string_lossy()
    );
    assert_eq!(stdout_of(&out), expected);
}

#[test]
fn promote_explicit_name_stdout_is_pinned() {
    let p = TempProject::with_discussions("named", &[("beta-cache", "Beta cache")]);
    let out = p.run(&["discuss", "promote", "beta-cache", "--name", "cache-layer"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let expected = format!(
        "✓ Promoted discussion 'beta-cache' → change 'cache-layer'\n\
         \x20 Path: {}\n\
         \x20 Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts\n",
        p.change_dir("cache-layer").to_string_lossy()
    );
    assert_eq!(stdout_of(&out), expected);
}

#[test]
fn promote_json_payload_is_pinned() {
    let p = TempProject::with_discussions("json", &[("gamma-x", "Gamma x")]);
    let out = p.run(&["discuss", "promote", "gamma-x", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let expected = format!(
        "{{\n\
         \x20 \"change\": \"gamma-x\",\n\
         \x20 \"path\": \"{}\",\n\
         \x20 \"slug\": \"gamma-x\",\n\
         \x20 \"status\": \"promoted\"\n\
         }}\n",
        TempProject::slash(&p.change_dir("gamma-x"))
    );
    assert_eq!(stdout_of(&out), expected);
}

#[test]
fn discuss_list_json_is_pinned() {
    let p = TempProject::with_discussions(
        "list",
        &[("alpha-search", "Alpha search"), ("beta-cache", "Beta cache")],
    );
    let out = p.run(&["discuss", "list", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let doc = |slug: &str, topic: &str| {
        format!(
            "    {{\n\
             \x20     \"archived\": false,\n\
             \x20     \"created\": \"2026-01-02\",\n\
             \x20     \"path\": \"{}\",\n\
             \x20     \"rounds\": 1,\n\
             \x20     \"slug\": \"{slug}\",\n\
             \x20     \"status\": \"concluded\",\n\
             \x20     \"topic\": \"{topic}\"\n\
             \x20   }}",
            TempProject::slash(&p.dir.join("openspec").join("discussions").join(format!("{slug}.md")))
        )
    };
    let expected = format!(
        "{{\n  \"discussions\": [\n{},\n{}\n  ]\n}}\n",
        doc("alpha-search", "Alpha search"),
        doc("beta-cache", "Beta cache")
    );
    assert_eq!(stdout_of(&out), expected);
}

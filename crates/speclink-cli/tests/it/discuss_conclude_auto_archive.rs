//! Integration coverage for the conclude closing step (conclusion-gated-discussion-archive):
//! concluding a spun-out discussion whose changes have all left the in-flight set
//! auto-archives the record and reports it; the ordinary conclude output stays
//! byte-identical to the pre-change baseline.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A project holding one live discussion written from `doc`.
    fn with_discussion(tag: &str, slug: &str, doc: &str) -> TempProject {
        let dir = std::env::temp_dir()
            .join(format!("speclink-cli-conclude-auto-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let discussions = dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        std::fs::write(discussions.join(format!("{slug}.md")), doc).unwrap();
        TempProject { dir }
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_speclink"));
        c.args(args)
            .current_dir(&self.dir)
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        let mut child = c
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().expect("wait speclink binary")
    }

    fn live_exists(&self, slug: &str) -> bool {
        self.dir.join("openspec").join("discussions").join(format!("{slug}.md")).exists()
    }

    fn archived_exists(&self, slug: &str) -> bool {
        let archive = self.dir.join("openspec").join("discussions").join("archive");
        std::fs::read_dir(&archive)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .any(|e| e.file_name().to_string_lossy().ends_with(&format!("-{slug}.md")))
            })
            .unwrap_or(false)
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

fn ok(out: &Output) {
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

/// A promoted discussion whose only spun-out change is no longer in flight
/// (archived), and whose conclusion is still the scaffold placeholder.
fn promoted_unconcluded_doc(slug: &str) -> String {
    format!(
        "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: cut-a\ncreated: 2026-01-02\n---\n\n\
         # Discussion: {slug}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
         ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
    )
}

/// A plain open discussion (never spun out).
fn open_doc(slug: &str) -> String {
    format!(
        "---\ntopic: {slug}\nslug: {slug}\nstatus: open\ncreated: 2026-01-02\n---\n\n\
         # Discussion: {slug}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
         ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
    )
}

#[test]
fn conclude_auto_archives_and_reports_in_human_output() {
    let p = TempProject::with_discussion("human", "alpha", &promoted_unconcluded_doc("alpha"));

    let out = p.run_stdin(&["discuss", "conclude", "alpha", "--stdin"], "**Decision**: done\n");
    ok(&out);

    let text = stdout_of(&out);
    assert!(text.contains("Concluded discussion 'alpha'"), "stdout: {text}");
    assert!(
        text.to_lowercase().contains("archived"),
        "closing step must be announced: {text}"
    );
    assert!(!p.live_exists("alpha"), "record leaves openspec/discussions/");
    assert!(p.archived_exists("alpha"), "record lands in discussions/archive/");
}

#[test]
fn conclude_auto_archive_json_carries_auto_archived_true() {
    let p = TempProject::with_discussion("json", "alpha", &promoted_unconcluded_doc("alpha"));

    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--json"],
        "**Decision**: done\n",
    );
    ok(&out);

    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid json");
    assert_eq!(v["status"], "concluded");
    assert_eq!(v["autoArchived"], serde_json::json!(true));
    assert!(p.archived_exists("alpha"));
}

#[test]
fn ordinary_conclude_output_stays_byte_identical() {
    // 未觸發閉環（promoted_to 缺席）→ 人眼與 --json 皆維持既有基線：無多行、無新鍵。
    let p = TempProject::with_discussion("base-h", "alpha", &open_doc("alpha"));
    let out = p.run_stdin(&["discuss", "conclude", "alpha", "--stdin"], "**Decision**: done\n");
    ok(&out);
    let text = stdout_of(&out);
    assert_eq!(text.lines().count(), 1, "exactly the existing single line: {text}");
    assert!(text.contains("Concluded discussion 'alpha'"));
    assert!(p.live_exists("alpha"), "record stays live");

    let p2 = TempProject::with_discussion("base-j", "beta", &open_doc("beta"));
    let out2 =
        p2.run_stdin(&["discuss", "conclude", "beta", "--stdin", "--json"], "**Decision**: done\n");
    ok(&out2);
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out2)).expect("valid json");
    assert!(v.get("autoArchived").is_none(), "no new key on the ordinary path: {v}");
    assert_eq!(v["status"], "concluded");
}

//! Integration coverage for the discuss content guard (discuss-content-guard):
//! content verbs read piped stdin even without an explicit `--stdin`, and empty
//! content is rejected at the front door instead of silently writing an empty section.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A project holding one open discussion `slug` (scaffolded layout).
    fn with_open_discussion(tag: &str, slug: &str) -> TempProject {
        let dir =
            std::env::temp_dir().join(format!("speclink-cli-guard-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let discussions = dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        let doc = format!(
            "---\ntopic: {slug}\nslug: {slug}\nstatus: open\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {slug}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
        );
        std::fs::write(discussions.join(format!("{slug}.md")), doc).unwrap();
        TempProject { dir }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_speclink"));
        c.args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        c
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .cmd(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().expect("wait speclink binary")
    }

    fn discussion(&self, slug: &str) -> String {
        std::fs::read_to_string(
            self.dir
                .join("openspec")
                .join("discussions")
                .join(format!("{slug}.md")),
        )
        .unwrap()
    }
}

/// Piped content without an explicit `--stdin` is still read and written — a forgotten
/// flag no longer silently drops the content.
#[test]
fn add_round_reads_piped_stdin_without_flag() {
    let p = TempProject::with_open_discussion("noflag", "alpha");
    let out = p.run_stdin(
        &["discuss", "add-round", "alpha", "--mode", "assumptions"],
        "**Focus**: piped without flag\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        p.discussion("alpha").contains("piped without flag"),
        "piped round content must be written"
    );
}

/// Empty piped content is rejected (the core guard surfaces via the CLI): non-zero exit
/// and no Round appended.
#[test]
fn add_round_rejects_empty_piped_content() {
    let p = TempProject::with_open_discussion("empty", "alpha");
    let out = p.run_stdin(
        &["discuss", "add-round", "alpha", "--stdin", "--mode", "assumptions"],
        "   \n",
    );
    assert!(!out.status.success(), "empty content must error");
    assert!(
        !p.discussion("alpha").contains("### Round 1"),
        "no round may be written on empty content"
    );
}

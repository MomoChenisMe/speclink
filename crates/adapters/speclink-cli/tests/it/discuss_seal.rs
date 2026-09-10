//! Integration coverage for the seal verb and `show --json` fromDiscussions
//! (change discussion-reflection-seal): `discuss link` no longer marks the
//! discussion promoted, `discuss seal` marks it once the chain is forged, and
//! `show --json` exposes the `fromDiscussions` chain.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn with_concluded(tag: &str, slug: &str, topic: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!("speclink-cli-seal-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let discussions = dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        let doc = format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
             ### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: build {slug}\n"
        );
        std::fs::write(discussions.join(format!("{slug}.md")), doc).unwrap();
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .output()
            .expect("run speclink binary")
    }

    fn discussion_text(&self, slug: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("discussions").join(format!("{slug}.md")),
        )
        .unwrap()
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

#[test]
fn link_stays_concluded_then_seal_promotes() {
    let p = TempProject::with_concluded("flow", "alpha", "Alpha");
    ok(&p.run(&["new", "change", "cut-a", "--agent", "claude"]));

    // link forges the chain but must NOT mark the discussion promoted.
    ok(&p.run(&["discuss", "link", "alpha", "cut-a"]));
    let after_link = p.discussion_text("alpha");
    assert!(after_link.contains("status: concluded\n"), "link must not promote: {after_link}");
    assert!(!after_link.contains("promoted_to"), "link must not write promoted_to: {after_link}");

    // seal marks it promoted once content has landed.
    ok(&p.run(&["discuss", "seal", "alpha", "cut-a"]));
    let after_seal = p.discussion_text("alpha");
    assert!(after_seal.contains("status: promoted\n"), "seal must promote: {after_seal}");
    assert!(after_seal.contains("promoted_to: cut-a\n"), "seal must accumulate promoted_to: {after_seal}");
}

#[test]
fn seal_json_payload_has_camelcase_slug_and_change() {
    let p = TempProject::with_concluded("json", "alpha", "Alpha");
    ok(&p.run(&["new", "change", "cut-a", "--agent", "claude"]));
    ok(&p.run(&["discuss", "link", "alpha", "cut-a"]));
    let out = p.run(&["discuss", "seal", "alpha", "cut-a", "--json"]);
    ok(&out);
    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid json");
    assert_eq!(v["slug"], "alpha");
    assert_eq!(v["change"], "cut-a");
}

#[test]
fn seal_no_color_success_has_no_ansi() {
    let p = TempProject::with_concluded("nocolor", "alpha", "Alpha");
    ok(&p.run(&["new", "change", "cut-a", "--agent", "claude"]));
    ok(&p.run(&["discuss", "link", "alpha", "cut-a"]));
    let out = p.run(&["discuss", "seal", "alpha", "cut-a", "--no-color"]);
    ok(&out);
    assert!(!stdout_of(&out).contains('\u{1b}'), "seal --no-color must emit no ANSI escapes");
}

#[test]
fn seal_rejects_when_chain_not_forged() {
    let p = TempProject::with_concluded("guard", "alpha", "Alpha");
    ok(&p.run(&["new", "change", "cut-a", "--agent", "claude"]));
    // No link → chain not forged.
    let out = p.run(&["discuss", "seal", "alpha", "cut-a"]);
    assert!(!out.status.success(), "seal must fail without a forged chain");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("not linked"), "stderr should explain the missing chain: {err}");
    assert!(p.discussion_text("alpha").contains("status: concluded\n"), "discussion untouched");
}

#[test]
fn show_json_exposes_from_discussions() {
    let p = TempProject::with_concluded("show", "alpha", "Alpha");
    ok(&p.run(&["new", "change", "cut-a", "--agent", "claude"]));

    // Before link: empty array.
    let out0 = p.run(&["show", "cut-a", "--json"]);
    ok(&out0);
    let v0: serde_json::Value = serde_json::from_str(&stdout_of(&out0)).expect("valid json");
    assert_eq!(v0["fromDiscussions"], serde_json::json!([]));

    // After link: contains the slug.
    ok(&p.run(&["discuss", "link", "alpha", "cut-a"]));
    let out1 = p.run(&["show", "cut-a", "--json"]);
    ok(&out1);
    let v1: serde_json::Value = serde_json::from_str(&stdout_of(&out1)).expect("valid json");
    assert_eq!(v1["fromDiscussions"], serde_json::json!(["alpha"]));
}

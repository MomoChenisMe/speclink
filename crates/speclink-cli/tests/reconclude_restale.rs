//! Integration coverage for the reconclude-restale flow: re-concluding a sealed
//! discussion flags its active changes (`restale_from`), the flag surfaces via
//! conclude output / `show` / `list` / `analyze`, and a re-seal clears it.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// A project with a concluded discussion `slug`, a change `change` proposed,
    /// linked to the discussion and sealed — i.e. an already-reflected discussion.
    fn sealed(tag: &str, slug: &str, change: &str) -> TempProject {
        let dir =
            std::env::temp_dir().join(format!("speclink-cli-restale-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let discussions = dir.join("openspec").join("discussions");
        std::fs::create_dir_all(&discussions).unwrap();
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        let doc = format!(
            "---\ntopic: {slug}\nslug: {slug}\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {slug}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
             ### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: build {slug}\n"
        );
        std::fs::write(discussions.join(format!("{slug}.md")), doc).unwrap();
        let p = TempProject { dir };
        ok(&p.run(&["new", "change", change, "--agent", "claude"]));
        // A minimal proposal so `analyze`'s Gaps dimension runs (needs ≥1 artifact).
        ok(&p.run_stdin(
            &["new", "artifact", "proposal", "--change", change, "--stdin"],
            "## Why\n\nFixture.\n\n## What Changes\n\n- Nothing.\n\n## Impact\n\n- Affected code: none.\n",
        ));
        ok(&p.run(&["discuss", "link", slug, change]));
        ok(&p.run(&["discuss", "seal", slug, change]));
        p
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_speclink"));
        c.args(args)
            .current_dir(&self.dir)
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        c
    }

    fn run(&self, args: &[&str]) -> Output {
        self.cmd(args).output().expect("run speclink binary")
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .cmd(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn speclink binary");
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().expect("wait speclink binary")
    }

    fn change_meta(&self, change: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("changes").join(change).join(".openspec.yaml"),
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

fn json_of(out: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout_of(out)).expect("valid json")
}

#[test]
fn reconclude_flags_sealed_change_and_conclude_reports() {
    let p = TempProject::sealed("flag", "alpha", "cut-a");

    // Re-conclude the already-sealed discussion → cut-a is flagged for re-ingest.
    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin"],
        "**Decision**: revised direction\n",
    );
    ok(&out);
    let text = stdout_of(&out);
    assert!(text.contains("cut-a"), "conclude must report the flagged change: {text}");
    assert!(text.contains("re-ingest"), "conclude must mention re-ingest: {text}");
    assert!(
        p.change_meta("cut-a").contains("restale_from: alpha"),
        "meta: {}",
        p.change_meta("cut-a")
    );
}

#[test]
fn conclude_json_carries_restale_flagged() {
    let p = TempProject::sealed("json", "alpha", "cut-a");
    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--json"],
        "**Decision**: revised\n",
    );
    ok(&out);
    let v = json_of(&out);
    assert_eq!(v["status"], "concluded");
    assert_eq!(v["restaleFlagged"], serde_json::json!(["cut-a"]));
}

#[test]
fn show_and_list_expose_restale_from() {
    let p = TempProject::sealed("show", "alpha", "cut-a");
    ok(&p.run_stdin(&["discuss", "conclude", "alpha", "--stdin"], "**Decision**: revised\n"));

    let show = json_of(&p.run(&["show", "cut-a", "--json"]));
    assert_eq!(show["restaleFrom"], serde_json::json!(["alpha"]));

    let list = json_of(&p.run(&["list", "--json"]));
    let cut = list["changes"].as_array().unwrap().iter().find(|c| c["name"] == "cut-a").unwrap();
    assert_eq!(cut["restaleFrom"], serde_json::json!(["alpha"]));
}

#[test]
fn analyze_emits_restale_finding() {
    let p = TempProject::sealed("analyze", "alpha", "cut-a");
    ok(&p.run_stdin(&["discuss", "conclude", "alpha", "--stdin"], "**Decision**: revised\n"));

    let v = json_of(&p.run(&["analyze", "cut-a", "--json"]));
    let findings = v["findings"].as_array().expect("findings array");
    let restale = findings.iter().find(|f| {
        f["summary"].as_str().map(|s| s.contains("re-concluded")).unwrap_or(false)
    });
    let f = restale.expect("a restale finding");
    assert!(f["summary"].as_str().unwrap().contains("alpha"), "finding names the discussion: {f}");
}

#[test]
fn reseal_clears_restale_flag() {
    let p = TempProject::sealed("reseal", "alpha", "cut-a");
    ok(&p.run_stdin(&["discuss", "conclude", "alpha", "--stdin"], "**Decision**: revised\n"));
    assert!(p.change_meta("cut-a").contains("restale_from: alpha"));

    // Simulate re-ingest completing: re-seal clears the flag.
    ok(&p.run(&["discuss", "seal", "alpha", "cut-a"]));
    assert!(
        !p.change_meta("cut-a").contains("restale_from"),
        "re-seal must clear the flag: {}",
        p.change_meta("cut-a")
    );
    let show = json_of(&p.run(&["show", "cut-a", "--json"]));
    assert_eq!(show["restaleFrom"], serde_json::json!([]));
}

#[test]
fn conclude_before_seal_flags_nothing() {
    // A concluded-but-not-yet-sealed discussion (no promoted_to) flags nothing on
    // re-conclude, and the --json payload omits restaleFlagged (parity-preserving).
    let dir = std::env::temp_dir().join(format!("speclink-cli-restale-preseal-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let discussions = dir.join("openspec").join("discussions");
    std::fs::create_dir_all(&discussions).unwrap();
    std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
    std::fs::write(
        discussions.join("alpha.md"),
        "---\ntopic: alpha\nslug: alpha\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
         # Discussion: alpha\n\n## Context\n\nFixture.\n\n## Rounds\n\n## Conclusion\n\n**Decision**: x\n",
    )
    .unwrap();
    let p = TempProject { dir };
    let out = p.run_stdin(&["discuss", "conclude", "alpha", "--stdin", "--json"], "**Decision**: y\n");
    ok(&out);
    let v = json_of(&out);
    assert_eq!(v["status"], "concluded");
    assert!(v.get("restaleFlagged").is_none(), "no restaleFlagged when nothing sealed: {v}");
}

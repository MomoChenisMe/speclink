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

    /// Run without piping stdin (verbs that take no `--stdin`).
    fn run(&self, args: &[&str]) -> Output {
        self.run_stdin(args, "")
    }

    /// An in-flight change whose `from_discussion` points at `slug`, with every
    /// task done so the archive completeness gate passes.
    fn put_change(&self, name: &str, slug: &str) {
        let dir = self.dir.join("openspec").join("changes").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".openspec.yaml"),
            format!("schema: spec-driven\ncreated: 2026-01-02\nfrom_discussion: {slug}\n"),
        )
        .unwrap();
        std::fs::write(dir.join("tasks.md"), "- [x] 1.1 done\n").unwrap();
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
    assert!(v.get("held").is_none(), "no held key without --hold: {v}");
    assert_eq!(v["status"], "concluded");
}

// --- --hold（discussion-spinout-hold）---

#[test]
fn conclude_with_hold_keeps_the_record_live_and_reports_it() {
    // 閉環條件成立、但帶 --hold → 記錄留在途，人眼多一行保留訊息、無 Auto-archived 行。
    let p = TempProject::with_discussion("hold-h", "alpha", &promoted_unconcluded_doc("alpha"));

    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--hold"],
        "**Decision**: cut-b later\n",
    );
    ok(&out);

    let text = stdout_of(&out);
    assert!(text.contains("Concluded discussion 'alpha'"), "stdout: {text}");
    assert!(text.contains("Held live (a later spin-out is planned)"), "stdout: {text}");
    assert!(!text.contains("Auto-archived"), "閉環不觸發: {text}");
    assert!(p.live_exists("alpha"), "record stays in openspec/discussions/");
    assert!(!p.archived_exists("alpha"));

    let doc = std::fs::read_to_string(
        p.dir.join("openspec").join("discussions").join("alpha.md"),
    )
    .unwrap();
    assert!(doc.contains("hold: true"), "frontmatter carries the flag: {doc}");
    assert!(doc.contains("**Decision**: cut-b later"), "conclusion landed in the same write");
}

#[test]
fn conclude_with_hold_json_carries_held_true_without_auto_archived() {
    let p = TempProject::with_discussion("hold-j", "alpha", &promoted_unconcluded_doc("alpha"));

    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--json", "--hold"],
        "**Decision**: cut-b later\n",
    );
    ok(&out);

    let v: serde_json::Value = serde_json::from_str(&stdout_of(&out)).expect("valid json");
    assert_eq!(v["status"], "concluded");
    assert_eq!(v["held"], serde_json::json!(true));
    assert!(v.get("autoArchived").is_none(), "閉環未觸發，無此鍵: {v}");
    assert!(p.live_exists("alpha"));
}

#[test]
fn staged_spin_out_lifecycle_holds_then_releases_the_record() {
    // 分期兩刀的生命週期（規格範例）：
    // alpha 的 promoted_to 為 cut-a、以 --hold 結論 → 封存 cut-a 後 alpha 留在途
    // → promote 出 cut-b 清掉旗標 → 封存 cut-b 後 alpha 隨行進封存區。
    let p = TempProject::with_discussion("lifecycle", "alpha", &promoted_unconcluded_doc("alpha"));
    p.put_change("cut-a", "alpha");

    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--hold"],
        "**Decision**: cut-a now, cut-b after it lands\n",
    );
    ok(&out);
    assert!(p.live_exists("alpha"), "在途變更引用中，結論不封存記錄");

    // 刀 A 封存：帶 hold 的來源討論不隨行，封存輸出也不列它。
    let out = p.run(&["archive", "cut-a", "--yes", "--skip-specs"]);
    ok(&out);
    let text = stdout_of(&out);
    assert!(!text.contains("alpha"), "隨行封存清單不含帶 hold 的討論: {text}");
    assert!(p.live_exists("alpha"), "帶 hold 的記錄留在途");
    assert!(!p.archived_exists("alpha"));

    // 刀 B 轉出：promoted_to 累加、旗標清除。
    let out = p.run(&["discuss", "promote", "alpha", "--name", "cut-b"]);
    ok(&out);
    let doc =
        std::fs::read_to_string(p.dir.join("openspec").join("discussions").join("alpha.md"))
            .unwrap();
    assert!(doc.contains("promoted_to: cut-a, cut-b"), "累加下一刀: {doc}");
    assert!(!doc.contains("hold: true"), "轉出清掉旗標: {doc}");

    // 刀 B 封存：記錄隨行進封存區（既有生命週期）。
    std::fs::write(
        p.dir.join("openspec").join("changes").join("cut-b").join("tasks.md"),
        "- [x] 1.1 done\n",
    )
    .unwrap();
    let out = p.run(&["archive", "cut-b", "--yes", "--skip-specs"]);
    ok(&out);
    assert!(!p.live_exists("alpha"), "旗標已清，記錄隨最後一刀封存");
    assert!(p.archived_exists("alpha"));
}

#[test]
fn conclude_with_hold_refuses_a_record_without_frontmatter() {
    // 規格「無 frontmatter 的記錄拒絕 --hold」：帶 --hold 非零收場、記錄不變；
    // 不帶 --hold 沿 pre-scaffold 既有路徑照常結論。
    let bare = "# Discussion: bare\n\n## Rounds\n";
    let p = TempProject::with_discussion("bare", "bare", bare);
    let path = p.dir.join("openspec").join("discussions").join("bare.md");

    let out = p.run_stdin(&["discuss", "conclude", "bare", "--stdin", "--hold"], "**Decision**: x\n");
    assert!(!out.status.success(), "帶 --hold 必須拒絕");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no frontmatter"), "stderr 說明原因: {err}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), bare, "記錄逐位元不變");

    let out = p.run_stdin(&["discuss", "conclude", "bare", "--stdin"], "**Decision**: x\n");
    ok(&out);
    assert!(std::fs::read_to_string(&path).unwrap().contains("## Conclusion"));
}

#[test]
fn discuss_archive_ignores_the_hold_flag() {
    // 手動封存＝放棄後續刀的明示出口：帶 hold 的記錄照常封存。
    let p = TempProject::with_discussion("manual", "alpha", &promoted_unconcluded_doc("alpha"));
    ok(&p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--hold"],
        "**Decision**: cut-b later\n",
    ));

    let out = p.run(&["discuss", "archive", "alpha"]);
    ok(&out);

    assert!(!p.live_exists("alpha"));
    assert!(p.archived_exists("alpha"));
}

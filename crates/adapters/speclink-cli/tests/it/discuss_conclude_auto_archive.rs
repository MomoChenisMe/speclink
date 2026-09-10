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
        self.finish_tasks(name);
    }

    /// 在途記錄的全文。
    fn discussion(&self, slug: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("discussions").join(format!("{slug}.md")),
        )
        .unwrap()
    }

    /// 讓 `name` 通過封存的完整度守門（promote 建出的骨架沒有已完成任務）。
    fn finish_tasks(&self, name: &str) {
        std::fs::write(
            self.dir.join("openspec").join("changes").join(name).join("tasks.md"),
            "- [x] 1.1 done\n",
        )
        .unwrap();
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
fn staged_spin_out_lifecycle_holds_across_every_cut_until_archived_by_hand() {
    // 規格 Example「分期三刀的生命週期」：alpha 尚未轉出任何變更，以 --hold 結論
    // 一次後依序轉出並封存三刀——每刀封存後記錄都留在途、封存輸出不列它；最後
    // 由使用者跑一次 `discuss archive` 明示收尾。
    let p = TempProject::with_discussion("lifecycle", "alpha", &open_doc("alpha"));

    let out = p.run_stdin(
        &["discuss", "conclude", "alpha", "--stdin", "--hold"],
        "**Decision**: three cuts in order\n",
    );
    ok(&out);
    assert!(p.live_exists("alpha"), "帶 hold 的結論不封存記錄");

    let mut accumulated = String::new();
    for cut in ["cut-a", "cut-b", "cut-c"] {
        ok(&p.run(&["discuss", "promote", "alpha", "--name", cut]));
        if !accumulated.is_empty() {
            accumulated.push_str(", ");
        }
        accumulated.push_str(cut);
        let doc = p.discussion("alpha");
        assert!(doc.contains(&format!("promoted_to: {accumulated}\n")), "依序累加: {doc}");
        assert!(doc.contains("hold: true"), "旗標全程保留（{cut} 轉出後）: {doc}");

        p.finish_tasks(cut);
        let out = p.run(&["archive", cut, "--yes", "--skip-specs"]);
        ok(&out);
        let text = stdout_of(&out);
        assert!(!text.contains("alpha"), "隨行封存清單不列帶 hold 的討論: {text}");
        assert!(p.live_exists("alpha"), "{cut} 封存後記錄仍在途");
        assert!(!p.archived_exists("alpha"));
    }

    ok(&p.run(&["discuss", "archive", "alpha"]));
    assert!(!p.live_exists("alpha"), "使用者明示收尾後記錄離開在途");
    assert!(p.archived_exists("alpha"));
}

/// 帶 `hold: true` 的已結論已轉出討論——分期系列在途、下一刀還沒建立。
fn held_doc(slug: &str) -> String {
    format!(
        "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: cut-a\ncreated: 2026-01-02\nhold: true\n---\n\n\
         # Discussion: {slug}\n\n## Context\n\nFixture.\n\n## Rounds\n\n\
         ## Conclusion\n\n**Decision**: cut-a now, cut-b next\n"
    )
}

/// stdout，把專案自身的目錄換成佔位符（人眼路徑、JSON 的斜線路徑、JSON 逃脫過的
/// 反斜線路徑三種寫法都換），只差 hold 行的兩個專案才好逐位元比對。
fn normalized_stdout(p: &TempProject, out: &Output) -> String {
    let raw = p.dir.to_string_lossy().to_string();
    stdout_of(out)
        .replace(&raw.replace('\\', "\\\\"), "<DIR>")
        .replace(&raw, "<DIR>")
        .replace(&raw.replace('\\', "/"), "<DIR>")
}

/// 對「只差一行 hold」的兩份相同記錄跑同一組指令，斷言輸出逐位元相同（轉出動詞
/// 從不報告 hold），回傳帶旗標那份的正規化 stdout 與轉出後的記錄全文。
fn held_vs_plain(
    tag: &str,
    slug: &str,
    run_it: impl Fn(&TempProject) -> Output,
) -> (String, String) {
    let held = TempProject::with_discussion(&format!("{tag}-held"), slug, &held_doc(slug));
    let plain = TempProject::with_discussion(
        &format!("{tag}-plain"),
        slug,
        &held_doc(slug).replace("hold: true\n", ""),
    );
    let held_out = run_it(&held);
    let plain_out = run_it(&plain);
    ok(&held_out);
    ok(&plain_out);
    let shown = normalized_stdout(&held, &held_out);
    assert_eq!(shown, normalized_stdout(&plain, &plain_out), "旗標不改變輸出");
    (shown, held.discussion(slug))
}

/// 轉出後的記錄：promoted_to 累加了 cut-b、status 為 promoted、旗標逐字保留；
/// 輸出（人眼或 --json）不提 hold。
fn assert_spun_out_and_still_held(shown: &str, doc: &str) {
    assert!(doc.contains("promoted_to: cut-a, cut-b\n"), "promoted_to 累加: {doc}");
    assert!(doc.contains("status: promoted"), "status 為 promoted: {doc}");
    assert!(doc.contains("hold: true"), "旗標逐字保留: {doc}");
    assert!(!shown.contains("hold"), "輸出不提旗標: {shown}");
}

/// 第三條轉出路徑：先 link 到既有變更（記錄逐位元不變），再 seal 補標。
fn link_then_seal(p: &TempProject, extra: &[&str]) -> Output {
    p.put_change("cut-b", "gamma");
    let before = p.discussion("gamma");
    ok(&p.run(&["discuss", "link", "gamma", "cut-b"]));
    assert_eq!(p.discussion("gamma"), before, "link 當下記錄逐位元不變");
    let mut args = vec!["discuss", "seal", "gamma", "cut-b"];
    args.extend_from_slice(extra);
    p.run(&args)
}

#[test]
fn spin_out_paths_keep_the_hold_flag_and_leave_human_output_unchanged() {
    // 規格「轉出保留旗標」：三個轉出路徑（discuss promote、new change
    // --from-discussion、discuss seal）對帶 hold 的記錄都累加 promoted_to 且旗標
    // 逐字保留；人眼輸出與不帶旗標時逐位元相同。
    let (shown, doc) = held_vs_plain("promote-human", "alpha", |p| {
        p.run(&["discuss", "promote", "alpha", "--name", "cut-b"])
    });
    assert_spun_out_and_still_held(&shown, &doc);

    let (shown, doc) = held_vs_plain("newchange-human", "beta", |p| {
        p.run(&["new", "change", "cut-b", "--from-discussion", "beta"])
    });
    assert_spun_out_and_still_held(&shown, &doc);

    let (shown, doc) = held_vs_plain("seal-human", "gamma", |p| link_then_seal(p, &[]));
    assert_spun_out_and_still_held(&shown, &doc);
}

#[test]
fn spin_out_paths_keep_the_hold_flag_and_leave_json_output_unchanged() {
    // 同上，--json 那一面（`new change` 沒有 --json 旗標，只有兩條路徑）。
    let (shown, doc) = held_vs_plain("promote-json", "alpha", |p| {
        p.run(&["discuss", "promote", "alpha", "--name", "cut-b", "--json"])
    });
    assert_spun_out_and_still_held(&shown, &doc);

    let (shown, doc) = held_vs_plain("seal-json", "gamma", |p| link_then_seal(p, &["--json"]));
    assert_spun_out_and_still_held(&shown, &doc);
}

#[test]
fn discuss_list_json_never_carries_a_hold_key() {
    // 規格「討論列表回應攜帶 concluded」的 THEN：本地 CLI 的 discuss list --json
    // 逐位元不變——hold 住在 head 的 serde(skip) 欄位裡，這裡釘一句守門。
    let p = TempProject::with_discussion("list-json", "alpha", &held_doc("alpha"));
    let out = p.run(&["discuss", "list", "--json"]);
    ok(&out);
    let text = stdout_of(&out);
    assert!(text.contains("\"slug\": \"alpha\""), "帶 hold 的記錄照列: {text}");
    assert!(!text.contains("\"hold\""), "CLI JSON 無 hold 鍵: {text}");
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

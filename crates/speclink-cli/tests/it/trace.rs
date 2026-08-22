//! `speclink trace <capability>` 的整合測試（change feature-provenance-skill）。
//!
//! 固定日期的封存目錄以手寫 fixture 鋪設（照 store_fs.rs 的 write(rel) 形狀），
//! 涵蓋：人讀縮排樹（由舊至新、封存目錄名、來源討論、兄弟變更、Requirement
//! 歸屬）、--json 凍結形狀、--no-color 行為、找不到 capability 的近零建議、
//! 單環髒資料（@trace 歸屬無封存目錄）的寬容組裝。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CANON_CHECKOUT: &str = "# checkout Specification\n\n## Purpose\n\nCheckout flow.\n\n## Requirements\n\n\
### Requirement: R1\n\nIt SHALL work.\n\n<!-- @trace\nsource: alpha\nupdated: 2026-07-10\n-->\n\n\
### Requirement: R2\n\nIt SHALL persist.\n\n<!-- @trace\nsource: ghost\nupdated: 2026-08-02\n-->\n";

const META_ALPHA: &str = "schema: spec-driven\ncreated: 2026-07-09\nfrom_discussion: origin-talk\narchived_at: 2026-07-10\n";
const META_PLAIN: &str = "schema: spec-driven\ncreated: 2026-08-01\n";

const EVIDENCE_ALPHA: &str = r#"{
  "version": 2,
  "change": "alpha",
  "touched": [{"task_id": "1", "task_desc": "d", "files": ["src/a.rs", "src/b.rs"]}],
  "entries": [
    {"taskId": "tsk_01", "taskDesc": "d", "touchedFiles": ["src/a.rs", "src/b.rs"], "recordedAt": "2026-07-10T00:00:00Z"},
    {"taskId": "tsk_02", "taskDesc": "d2", "touchedFiles": ["src/c.rs"], "recordedAt": "2026-07-10T00:00:00Z"}
  ]
}"#;

const DISCUSSION: &str = "---\ntopic: how checkout came to be\nslug: origin-talk\nstatus: promoted\npromoted_to: alpha, cousin\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: build it\n";

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// 正典 checkout（R1→alpha、R2→ghost 髒歸屬）＋封存 alpha（有討論、有
    /// evidence）、beta（皆無）、cousin（兄弟變更，動 billing）＋live 討論。
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!("speclink-cli-trace-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // macOS 的 temp dir 是 /var → /private/var 的 symlink；Windows 的
        // canonicalize 會加 \\?\ 前綴，跳過。
        let dir = if cfg!(windows) { dir } else { dir.canonicalize().unwrap() };
        let p = TempProject { dir };
        p.write(".speclink.yaml", "tools:\n  - claude\n");
        p.write("openspec/specs/checkout/spec.md", CANON_CHECKOUT);
        p.write(
            "openspec/specs/billing/spec.md",
            "# billing Specification\n\n## Purpose\n\nBilling flow.\n\n## Requirements\n\n### Requirement: B1\n\nIt SHALL bill.\n",
        );
        p.write("openspec/changes/archive/2026-07-10-alpha/.openspec.yaml", META_ALPHA);
        p.write("openspec/changes/archive/2026-07-10-alpha/specs/checkout/spec.md", "delta");
        p.write("openspec/changes/archive/2026-07-10-alpha/.evidence.json", EVIDENCE_ALPHA);
        p.write("openspec/changes/archive/2026-08-02-beta/.openspec.yaml", META_PLAIN);
        p.write("openspec/changes/archive/2026-08-02-beta/specs/checkout/spec.md", "delta");
        p.write("openspec/changes/archive/2026-07-12-cousin/.openspec.yaml", META_PLAIN);
        p.write("openspec/changes/archive/2026-07-12-cousin/specs/billing/spec.md", "delta");
        p.write("openspec/discussions/origin-talk.md", DISCUSSION);
        p
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn trace(&self, args: &[&str]) -> Output {
        self.trace_with_env(args, &[])
    }

    fn trace_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.arg("trace")
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN");
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.output().expect("run speclink binary")
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

fn assert_ordered(haystack: &str, needles: &[&str]) {
    let mut from = 0usize;
    for needle in needles {
        let at = haystack[from..]
            .find(needle)
            .unwrap_or_else(|| panic!("expected `{needle}` after byte {from} in:\n{haystack}"));
        from += at + needle.len();
    }
}

#[test]
fn the_human_tree_walks_the_chain_oldest_first_with_fanout_and_attribution() {
    let p = TempProject::new("human");
    let out = p.trace(&["checkout", "--no-color"]);
    assert!(out.status.success(), "trace 成功 exit 0: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = stdout_of(&out);

    // 由舊至新：alpha（2026-07-10）在 beta（2026-08-02）前；區段順序
    // changes → discussions → requirements。
    assert_ordered(
        &stdout,
        &[
            "checkout",
            "2026-07-10-alpha",
            "origin-talk",
            "tsk_01",
            "src/a.rs",
            "2026-08-02-beta",
            "origin-talk",  // 討論區段
            "cousin",       // 兄弟變更
            "billing",      // 兄弟觸及的 capability
            "R1",
            "alpha",
            "R2",
            "ghost",
        ],
    );
    // 無討論、無 evidence 的標示（措辭由實作凍結，這裡釘語意錨點）。
    assert!(stdout.contains("(none)"), "beta 無來源討論須標示無: {stdout}");
    assert!(stdout.contains("(no record)"), "beta 無 evidence 須標示無記錄: {stdout}");
    // 髒歸屬 ghost 只入 requirements，不入 changes 清單。
    assert!(!stdout.contains("-ghost"), "ghost 無封存目錄不得出現在 changes: {stdout}");
}

#[test]
fn json_payload_keeps_the_frozen_camel_case_shape() {
    let p = TempProject::new("json");
    let out = p.trace(&["checkout", "--json"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = stdout_of(&out);
    // payload 外無雜訊：整個 stdout 就是單一 JSON 物件。
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is a single JSON object");

    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["capability", "changes", "discussions", "requirements"]);
    assert_eq!(v["capability"], "checkout");

    let changes = v["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 2, "cousin 不動 checkout、ghost 無封存目錄，皆不入鏈");
    assert_eq!(changes[0]["name"], "alpha");
    assert_eq!(changes[0]["archivedDir"], "2026-07-10-alpha");
    assert_eq!(changes[0]["fromDiscussion"], "origin-talk");
    let evidence = changes[0]["evidence"].as_array().expect("evidence 存在則為陣列");
    assert_eq!(evidence[0]["taskId"], "tsk_01");
    assert_eq!(evidence[0]["files"].as_array().unwrap().len(), 2);
    assert_eq!(changes[1]["name"], "beta");
    assert!(changes[1]["fromDiscussion"].is_null(), "缺欄為 null");
    assert!(changes[1]["evidence"].is_null(), "無 .evidence.json 為 null");

    let requirements = v["requirements"].as_array().unwrap();
    let pairs: Vec<(&str, &str)> = requirements
        .iter()
        .map(|r| (r["name"].as_str().unwrap(), r["source"].as_str().unwrap()))
        .collect();
    assert_eq!(pairs, [("R1", "alpha"), ("R2", "ghost")], "髒歸屬照列於 requirements");

    let discussions = v["discussions"].as_array().unwrap();
    assert_eq!(discussions.len(), 1);
    assert_eq!(discussions[0]["slug"], "origin-talk");
    assert_eq!(discussions[0]["archived"], false);
    let promoted = discussions[0]["promotedTo"].as_array().unwrap();
    assert_eq!(promoted.len(), 2);
    assert_eq!(promoted[0]["change"], "alpha");
    assert_eq!(promoted[0]["capabilities"].as_array().unwrap(), &["checkout"]);
    assert_eq!(promoted[1]["change"], "cousin");
    assert_eq!(promoted[1]["capabilities"].as_array().unwrap(), &["billing"]);
}

#[test]
fn no_color_keeps_the_same_content_without_escape_codes() {
    let p = TempProject::new("nocolor");
    let plain = p.trace(&["checkout", "--no-color"]);
    let forced = p.trace_with_env(&["checkout"], &[("CLICOLOR_FORCE", "1")]);
    assert!(plain.status.success() && forced.status.success());

    let plain_out = stdout_of(&plain);
    assert!(!plain_out.contains('\u{1b}'), "--no-color 下無色彩控制碼: {plain_out:?}");

    // 剝掉 SGR 色碼後內容必須逐位元相同。
    let mut stripped = String::new();
    let mut rest = stdout_of(&forced);
    while let Some(at) = rest.find('\u{1b}') {
        stripped.push_str(&rest[..at]);
        let tail = &rest[at..];
        let end = tail.find('m').map(|i| i + 1).unwrap_or(tail.len());
        rest = tail[end..].to_string();
    }
    stripped.push_str(&rest);
    assert_eq!(stripped, plain_out, "--no-color 僅省略色碼、內容不變");
}

#[test]
fn an_unknown_capability_fails_with_suggestions_and_no_payload() {
    let p = TempProject::new("unknown");
    let out = p.trace(&["checkou", "--json", "--no-color"]);
    assert!(!out.status.success(), "無正典規格必須非零 exit");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("'checkou' is not in the canonical specs"), "{stderr}");
    assert!(stderr.contains("checkout"), "近似建議指向既有名: {stderr}");
    let count = stderr.matches("  - ").count();
    assert!((1..=3).contains(&count), "至多三筆建議: {stderr}");
    assert!(stdout_of(&out).trim().is_empty(), "--json 下 stdout 無成功 payload");
}

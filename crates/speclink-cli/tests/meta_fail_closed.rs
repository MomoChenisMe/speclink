//! Integration tests: change metadata is fail-closed — a `.openspec.yaml` that
//! EXISTS but cannot be parsed marks the change invalid in `list` (which stays
//! available) and stops every verb that needs the change's metadata semantics,
//! naming the file and the parse reason. Valid-workspace output stays frozen.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

const BAD_YAML: &str = ": : :\n\t bad yaml [unclosed\n";
const GOOD_META: &str = "schema: spec-driven\ncreated: 2026-07-01\n";

impl TempProject {
    /// Project skeleton with one valid change (`good-change`) and one whose
    /// `.openspec.yaml` is corrupt (`broken-change`).
    fn new(tag: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-meta-fail-closed-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), "tools:\n  - claude\n").unwrap();
        for (name, meta) in [("good-change", GOOD_META), ("broken-change", BAD_YAML)] {
            let change = dir.join("openspec").join("changes").join(name);
            std::fs::create_dir_all(&change).unwrap();
            std::fs::write(change.join(".openspec.yaml"), meta).unwrap();
            std::fs::write(change.join("proposal.md"), "## Why\n\nDemo.\n").unwrap();
            std::fs::write(change.join("tasks.md"), "- [ ] 1.1 Do the thing\n").unwrap();
        }
        TempProject { dir }
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_speclink"));
        cmd.args(args).current_dir(&self.dir);
        for key in [
            "SPECLINK_LOCALE",
            "SPECLINK_SPEC_LOCALE",
            "SPECLINK_TDD",
            "SPECLINK_AUDIT",
            "SPECLINK_STORE_URL",
        ] {
            cmd.env_remove(key);
        }
        cmd.output().expect("run speclink binary")
    }

    fn meta_of(&self, name: &str) -> String {
        std::fs::read_to_string(
            self.dir.join("openspec").join("changes").join(name).join(".openspec.yaml"),
        )
        .unwrap()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The frozen path+reason prefix every fail-closed error must carry.
const BROKEN_FILE_PREFIX: &str = "invalid openspec/changes/broken-change/.openspec.yaml: ";

// --- list: stays available, marks the broken item (spec「list 對壞 metadata 標 invalid 而不失效」) ---

#[test]
fn list_human_output_appends_invalid_marker_to_the_broken_line_only() {
    let p = TempProject::new("list-human");
    let out = p.run(&["list", "--sort", "name", "--no-color"]);
    assert!(out.status.success(), "list must stay available: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let broken_line = stdout.lines().find(|l| l.contains("broken-change")).expect("broken listed");
    assert!(
        broken_line.ends_with("(invalid .openspec.yaml)"),
        "broken line carries the frozen marker: {broken_line}"
    );
    let good_line = stdout.lines().find(|l| l.contains("good-change")).expect("good listed");
    assert_eq!(
        good_line, "  • good-change [0/1] — Demo.",
        "valid line stays byte-identical to today's rendering"
    );
}

#[test]
fn list_json_adds_meta_error_string_only_on_the_broken_item() {
    let p = TempProject::new("list-json");
    let out = p.run(&["list", "--sort", "name", "--json"]);
    assert!(out.status.success());
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let changes = payload["changes"].as_array().expect("changes array");
    assert_eq!(changes.len(), 2, "broken change must not be dropped");

    let broken = changes.iter().find(|c| c["name"] == "broken-change").unwrap();
    assert!(
        broken["metaError"].is_string(),
        "broken item carries metaError as a string: {broken}"
    );
    assert!(!broken["metaError"].as_str().unwrap().is_empty());

    let good = changes.iter().find(|c| c["name"] == "good-change").unwrap();
    let keys: Vec<&str> = good.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec!["completedTasks", "name", "status", "summary", "totalTasks"],
        "valid item's field shape is unchanged (no metaError key)"
    );
}

// --- single-change queries fail closed (spec「單一 change 查詢對壞 metadata fail closed」) ---

#[test]
fn status_on_broken_meta_exits_non_zero_naming_file_and_reason() {
    let p = TempProject::new("status");
    let out = p.run(&["status", "--change", "broken-change"]);
    assert!(!out.status.success(), "status must fail closed");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(BROKEN_FILE_PREFIX),
        "stderr names the file then the reason: {err}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        "no status payload on stdout"
    );
}

// --- lifecycle writes refuse (spec「壞 metadata 使生命週期寫入 fail closed」) ---

#[test]
fn in_progress_add_on_broken_meta_refuses_and_leaves_the_file_byte_identical() {
    let p = TempProject::new("inprogress");
    let out = p.run(&["in-progress", "add", "broken-change"]);
    assert!(!out.status.success(), "in-progress add must refuse");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(BROKEN_FILE_PREFIX),
        "stderr names the file"
    );
    assert_eq!(p.meta_of("broken-change"), BAD_YAML, "no started_* lines stamped");
}

#[test]
fn discard_with_force_on_broken_meta_refuses_and_keeps_the_change() {
    let p = TempProject::new("discard");
    let out = p.run(&["discard", "broken-change", "--force"]);
    assert!(!out.status.success(), "discard --force must refuse on corrupt meta");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(BROKEN_FILE_PREFIX),
        "stderr names the file"
    );
    assert!(
        p.dir.join("openspec").join("changes").join("broken-change").join("proposal.md").is_file(),
        "change directory fully preserved"
    );
    assert_eq!(p.meta_of("broken-change"), BAD_YAML);
}

//! spec-dir-migration「規格資料夾的預設位置與舊佈局相容」: which folder a project
//! reads (explicit `spec_dir` → `speclink/` → `openspec/` → `speclink`) and the
//! one-line stderr warning each undecided layout carries. The warning never
//! touches stdout or the exit code.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WARNING: &str = "speclink: warning:";

struct TempProject {
    dir: PathBuf,
    home: PathBuf,
}

impl TempProject {
    fn new(tag: &str) -> TempProject {
        let base = std::env::temp_dir().join(format!(
            "speclink-cli-layout-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        TempProject { dir, home }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// One change named `name` under the `spec_dir` folder, its data file under
    /// the name that folder's era used (`.openspec.yaml` in `openspec/`).
    fn change(&self, spec_dir: &str, name: &str) {
        let file = if spec_dir == "openspec" { ".openspec.yaml" } else { ".meta.yaml" };
        self.write(
            &format!("{spec_dir}/changes/{name}/{file}"),
            "schema: spec-driven\ncreated: 2026-10-01\n",
        );
        self.write(&format!("{spec_dir}/changes/{name}/proposal.md"), "## Why\n\nx\n");
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(args)
            .current_dir(&self.dir)
            .env_remove("SPECLINK_LOCALE")
            .env_remove("SPECLINK_SPEC_LOCALE")
            .env_remove("SPECLINK_TDD")
            .env_remove("SPECLINK_AUDIT")
            .env_remove("SPECLINK_STORE_URL")
            .env_remove("SPECLINK_TOKEN")
            .env("USERPROFILE", &self.home)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn warnings(out: &Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.starts_with(WARNING))
        .map(str::to_string)
        .collect()
}

fn listed(out: &Output) -> Vec<String> {
    let json: serde_json::Value = serde_json::from_str(&stdout_of(out)).expect("list --json parses");
    json["changes"]
        .as_array()
        .expect("changes array")
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

fn is_dir(root: &Path, rel: &str) -> bool {
    root.join(rel.split('/').collect::<PathBuf>()).is_dir()
}

#[test]
fn a_new_project_gets_a_speclink_folder() {
    let p = TempProject::new("fresh");
    let out = p.run(&["init", "--tools", "claude"]);
    assert!(out.status.success(), "init: {}", String::from_utf8_lossy(&out.stderr));
    for rel in ["speclink/specs", "speclink/changes/archive"] {
        assert!(is_dir(&p.dir, rel), "{rel} exists");
    }
    assert!(p.dir.join("speclink").join("config.yaml").is_file());
    assert!(!p.dir.join("openspec").exists(), "no openspec/ for a new project");
    let app = std::fs::read_to_string(p.dir.join(".speclink.yaml")).unwrap();
    assert!(
        !app.lines().any(|l| l.trim_start().starts_with("spec_dir:")),
        "no active spec_dir line: {app}"
    );
    assert!(warnings(&out).is_empty(), "{:?}", warnings(&out));
}

#[test]
fn a_legacy_layout_reads_openspec_and_points_at_update() {
    let p = TempProject::new("legacy");
    p.write(".speclink.yaml", "tools: [claude]\n");
    p.change("openspec", "old-work");
    let out = p.run(&["list", "--json"]);
    assert!(out.status.success());
    assert_eq!(listed(&out), ["old-work"]);
    let w = warnings(&out);
    assert_eq!(w.len(), 1, "one warning line: {w:?}");
    assert!(w[0].contains("speclink update"), "{}", w[0]);

    // stdout is what the same tree reads with the folder named explicitly —
    // the warning lives on stderr only.
    p.write(".speclink.yaml", "tools: [claude]\nspec_dir: openspec\n");
    let explicit = p.run(&["list", "--json"]);
    assert_eq!(stdout_of(&out), stdout_of(&explicit), "stdout is byte-identical");
}

#[test]
fn an_openspec_project_points_at_init() {
    let p = TempProject::new("openspec-project");
    p.change("openspec", "imported");
    let out = p.run(&["list"]);
    assert!(out.status.success());
    assert!(stdout_of(&out).contains("imported"), "{}", stdout_of(&out));
    let w = warnings(&out);
    assert_eq!(w.len(), 1, "one warning line: {w:?}");
    assert!(w[0].contains("speclink init"), "{}", w[0]);
}

#[test]
fn both_folders_read_only_speclink_and_say_openspec_is_ignored() {
    let p = TempProject::new("both");
    p.write(".speclink.yaml", "tools: [claude]\n");
    p.change("speclink", "current");
    p.change("openspec", "legacy-only");
    let out = p.run(&["list", "--json"]);
    assert!(out.status.success());
    assert_eq!(listed(&out), ["current"]);
    let w = warnings(&out);
    assert_eq!(w.len(), 1, "one warning line: {w:?}");
    assert!(w[0].contains("openspec/") && w[0].contains("not read"), "{}", w[0]);
}

#[test]
fn an_explicit_spec_dir_and_the_default_layout_never_warn() {
    let explicit = TempProject::new("explicit");
    explicit.write(".speclink.yaml", "tools: [claude]\nspec_dir: openspec\n");
    explicit.change("openspec", "kept");
    let out = explicit.run(&["list"]);
    assert!(out.status.success());
    assert!(stdout_of(&out).contains("kept"));
    assert!(warnings(&out).is_empty(), "{:?}", warnings(&out));

    let default = TempProject::new("default");
    default.write(".speclink.yaml", "tools: [claude]\n");
    default.change("speclink", "fresh");
    let out = default.run(&["list"]);
    assert!(out.status.success());
    assert!(stdout_of(&out).contains("fresh"));
    assert!(warnings(&out).is_empty(), "{:?}", warnings(&out));
}

// --- store-abstraction「預設檔案系統佈局」through the CLI ---

fn speclink_project(tag: &str) -> TempProject {
    let p = TempProject::new(tag);
    p.write(".speclink.yaml", "tools: [claude]\n");
    p.change("speclink", "base");
    p.change("speclink", "base2");
    p
}

#[test]
fn a_legacy_data_file_alone_is_read_by_plan() {
    let p = speclink_project("legacy-plan");
    p.write("speclink/changes/old/.openspec.yaml", "schema: spec-driven\ndepends_on: base\n");
    let out = p.run(&["plan", "--json"]);
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout_of(&out)).unwrap();
    let old = json["changes"].as_array().unwrap().iter().find(|c| c["name"] == "old").unwrap();
    assert_eq!(old["dependsOn"], serde_json::json!(["base"]));
}

#[test]
fn writing_metadata_through_the_cli_drops_the_legacy_name() {
    let p = speclink_project("legacy-depends");
    p.write("speclink/changes/old/.openspec.yaml", "schema: spec-driven\n");
    let out = p.run(&["change", "depends", "old", "--on", "base2"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let dir = p.dir.join("speclink").join("changes").join("old");
    assert_eq!(
        std::fs::read_to_string(dir.join(".meta.yaml")).unwrap(),
        "schema: spec-driven\ndepends_on: base2\n"
    );
    assert!(!dir.join(".openspec.yaml").exists());
}

#[test]
fn the_new_data_file_wins_for_status() {
    let p = speclink_project("both-status");
    p.write("speclink/changes/both/.meta.yaml", "schema: no-spec-change\n");
    p.write("speclink/changes/both/.openspec.yaml", "schema: spec-driven\n");
    let out = p.run(&["status", "--change", "both", "--json"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_str(&stdout_of(&out)).unwrap();
    assert_eq!(json["schemaName"], "no-spec-change");
}

//! Integration tests for custom tool descriptors in `.speclink.yaml` `tools:`.
//!
//! Pinned behavior: `speclink update` generates skills + marker block for a valid
//! descriptor (exit 0); validation failures exit non-zero with a single-line semantic
//! error naming the field; removing a descriptor from `tools:` prunes its footprint
//! (skill dirs, marker block, empty files/dirs) on the next update.

use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    fn new(tag: &str, app_yaml: &str) -> TempProject {
        let dir = std::env::temp_dir().join(format!(
            "speclink-cli-descriptor-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        std::fs::write(dir.join(".speclink.yaml"), app_yaml).unwrap();
        TempProject { dir }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.dir.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.join(rel.split('/').collect::<PathBuf>())).unwrap()
    }

    fn exists(&self, rel: &str) -> bool {
        self.dir.join(rel.split('/').collect::<PathBuf>()).exists()
    }

    fn update(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_speclink"))
            .args(["update"])
            .current_dir(&self.dir)
            .output()
            .expect("run speclink binary")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

const WAD_DESCRIPTOR: &str = "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: tool-call\n";

fn stderr_line(out: &Output) -> String {
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    let lines: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected exactly one stderr line, got: {err:?}");
    lines[0].to_string()
}

// --- generation ---

#[test]
fn valid_descriptor_generates_skills_and_marker_block() {
    // Spec scenario 合法描述子生成對應工具檔.
    let p = TempProject::new("generate", WAD_DESCRIPTOR);
    let out = p.update();

    assert!(
        out.status.success(),
        "update must succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        p.exists(".wad/skills/speclink-apply/SKILL.md"),
        "skills_dir must receive speclink-*/SKILL.md files"
    );
    let md = p.read("WAD.md");
    assert!(md.contains("<!-- SPECLINK:START"), "marker block upserted: {md}");
    assert!(md.contains("<!-- SPECLINK:END -->"));
}

// --- validation failures: non-zero exit + single semantic line naming the field ---

#[test]
fn builtin_name_conflict_is_rejected() {
    // Spec scenario 名稱與內建工具衝突被拒.
    let p = TempProject::new(
        "conflict",
        "tools:\n  - name: claude\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n",
    );
    let out = p.update();

    assert!(!out.status.success(), "exit code must be non-zero");
    let line = stderr_line(&out);
    assert!(line.contains("name"), "must name the field: {line}");
    assert!(line.contains("claude"), "must name the conflicting value: {line}");
}

#[test]
fn path_escaping_project_root_is_rejected() {
    // Spec scenario 路徑逸出專案根被拒.
    let p = TempProject::new(
        "escape",
        "tools:\n  - name: wad-harness\n    skills_dir: ../outside/skills\n    instructions_file: WAD.md\n",
    );
    let out = p.update();

    assert!(!out.status.success(), "exit code must be non-zero");
    let line = stderr_line(&out);
    assert!(line.contains("skills_dir"), "must name the field: {line}");
    assert!(
        !p.dir.parent().unwrap().join("outside").exists(),
        "nothing may be written outside the project root"
    );
}

#[test]
fn unknown_invocation_is_rejected() {
    let p = TempProject::new(
        "invocation",
        "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: http\n",
    );
    let out = p.update();

    assert!(!out.status.success(), "exit code must be non-zero");
    let line = stderr_line(&out);
    assert!(line.contains("invocation"), "must name the field: {line}");
}

// --- removal cleanup ---

#[test]
fn removed_descriptor_footprint_is_pruned() {
    // Spec scenario 移除描述子後生成物被清理: generated dir removed, marker stripped,
    // a file left empty is deleted.
    let p = TempProject::new("prune", WAD_DESCRIPTOR);
    assert!(p.update().status.success());
    assert!(p.exists(".wad/skills/speclink-apply/SKILL.md"));
    assert!(p.exists("WAD.md"));

    p.write(".speclink.yaml", "tools:\n  - claude\n");
    let out = p.update();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(!p.exists(".wad/skills"), "speclink-* skill dirs and empty parents removed");
    assert!(!p.exists(".wad"), "empty descriptor dirs are removed entirely");
    assert!(!p.exists("WAD.md"), "marker-only instructions file is deleted");
}

#[test]
fn removed_descriptor_keeps_user_content_in_instructions_file() {
    let p = TempProject::new("prune-user-content", WAD_DESCRIPTOR);
    p.write("WAD.md", "# My own harness notes\n\nKeep me.\n");
    assert!(p.update().status.success());
    assert!(p.read("WAD.md").contains("<!-- SPECLINK:START"));

    p.write(".speclink.yaml", "tools: []\n");
    assert!(p.update().status.success());

    let md = p.read("WAD.md");
    assert!(!md.contains("<!-- SPECLINK:START"), "marker stripped: {md}");
    assert!(md.contains("Keep me."), "user content preserved: {md}");
}

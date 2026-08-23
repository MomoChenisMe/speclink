//! Integration tests for custom tool descriptors in `.speclink.yaml` `tools:`.
//!
//! Pinned behavior: `speclink update` generates skills for a valid
//! descriptor (exit 0); validation failures exit non-zero with a single-line semantic
//! error naming the field; removing a descriptor from `tools:` prunes its footprint
//! (skill dirs, any legacy marker block, empty files/dirs) on the next update.

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
fn valid_descriptor_generates_skills_and_no_instruction_file() {
    // Spec scenario 合法描述子生成對應工具檔：技能檔生成、exit 0，
    // 且不生成任何指令檔區塊。
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
    assert!(!p.exists("WAD.md"), "instructions_file must not be generated");
}

#[test]
fn descriptor_without_instructions_file_is_accepted() {
    // Spec requirement「tools 自訂描述子的接受與驗證」：instructions_file 選填。
    let p = TempProject::new(
        "no-instructions-file",
        "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    invocation: tool-call\n",
    );
    let out = p.update();

    assert!(
        out.status.success(),
        "descriptor without instructions_file must be accepted: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.exists(".wad/skills/speclink-apply/SKILL.md"));
    assert!(
        String::from_utf8_lossy(&out.stderr).trim().is_empty(),
        "no deprecation notice when the field is absent"
    );
}

#[test]
fn a_leftover_instructions_file_field_gets_a_deprecation_notice() {
    // Spec scenario「殘留 instructions_file 欄位得棄用提示」：exit 0、技能檔照常
    // 生成，stderr 帶一行棄用提示指明該欄位已不生效。
    let p = TempProject::new("deprecated-field", WAD_DESCRIPTOR);
    let out = p.update();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(p.exists(".wad/skills/speclink-apply/SKILL.md"));
    let line = stderr_line(&out);
    assert!(
        line.contains("instructions_file") && line.contains("deprecated"),
        "the notice must name the deprecated field: {line}"
    );
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

#[test]
fn update_reports_stripped_files_on_stdout() {
    // Spec scenario「更新時剝除內建工具的遺留 marker」的 stdout 面：摘要列出被剝除
    // 的檔案（core 只斷言 UpdateOutcome.stripped，這裡釘 CLI 的輸出行）。
    let p = TempProject::new("strip-stdout", "tools:\n  - claude\n");
    p.write(
        "CLAUDE.md",
        "<!-- SPECLINK:START v1.0.0 -->\n舊路由表\n<!-- SPECLINK:END -->\n使用者段落\n",
    );

    let out = p.update();

    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Stripped legacy Speclink blocks from: CLAUDE.md"),
        "stdout 須列出剝除檔案：{stdout}"
    );
    assert_eq!(p.read("CLAUDE.md"), "使用者段落\n");
}

// --- removal cleanup ---

#[test]
fn removed_descriptor_footprint_is_pruned() {
    // Spec scenario 移除描述子後生成物被清理: generated dir removed, any legacy
    // marker stripped, a file left empty is deleted.
    let p = TempProject::new("prune", WAD_DESCRIPTOR);
    // 舊版引擎注入過的純 marker 檔——描述子下架時要被整份清掉。
    p.write("WAD.md", "<!-- SPECLINK:START v1.0.0 -->\n\n舊路由表。\n\n<!-- SPECLINK:END -->\n");
    assert!(p.update().status.success());
    assert!(p.exists(".wad/skills/speclink-apply/SKILL.md"));

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
    // 先讓描述子登記足跡（下架時的清理目標由此而來）。
    assert!(p.update().status.success());
    // 再擺回一份舊版引擎注入過的檔案：marker 在上、使用者段落在下。
    p.write(
        "WAD.md",
        "<!-- SPECLINK:START v1.0.0 -->\n\n舊路由表。\n\n<!-- SPECLINK:END -->\n# My own harness notes\n\nKeep me.\n",
    );

    p.write(".speclink.yaml", "tools: []\n");
    assert!(p.update().status.success());

    let md = p.read("WAD.md");
    assert!(!md.contains("<!-- SPECLINK:START"), "marker stripped: {md}");
    assert!(md.contains("Keep me."), "user content preserved: {md}");
}

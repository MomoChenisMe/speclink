//! Skill assets must read documents through speclink verbs, never by opening
//! spec-directory file paths directly — a single-source skill then works in
//! both fs and remote modes (verb-contract spec: 技能資產不含直接讀檔指示).
//!
//! The detectable pattern for "a direct-read instruction": a read/glob/open
//! word followed (same line) by a spec-document path — `specs/`, `changes/`,
//! `discussions/` under the spec dir, or the LANGUAGE document. Writing
//! paths (capture targets, git-status filters, archive layouts) stay legal.

use regex::Regex;
use speclink_core::init;
use speclink_core::skills::{self, Tool};
use std::path::PathBuf;

struct TempRoot {
    dir: PathBuf,
}

impl TempRoot {
    fn new(tag: &str) -> TempRoot {
        let dir = std::env::temp_dir().join(format!(
            "speclink-skill-verbization-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempRoot { dir }
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn generated_skills_contain_no_direct_spec_document_reads() {
    let root = TempRoot::new("scan");
    init::init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();

    let direct_read = Regex::new(
        r"(?i)\b(read|glob|open)\b[^\n]{0,120}?(openspec[/\\](specs|changes|discussions)|\{\{SPEC_DIR\}\}(specs|changes|discussions|LANGUAGE)|openspec[/\\]LANGUAGE\.md)",
    )
    .unwrap();

    let mut offenders: Vec<String> = Vec::new();
    for skill in skills::registry() {
        let path = root
            .dir
            .join(".claude")
            .join("skills")
            .join(format!("speclink-{}", skill.name))
            .join("SKILL.md");
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in content.lines().enumerate() {
            if direct_read.is_match(line) {
                offenders.push(format!("speclink-{} SKILL.md:{}: {}", skill.name, i + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "skills must read documents through speclink verbs, not file paths:\n{}",
        offenders.join("\n")
    );
}

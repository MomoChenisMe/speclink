//! Skill assets must read documents through speclink verbs, never by opening
//! spec-directory file paths directly — a single-source skill then works in
//! both fs and remote modes (verb-contract spec: 技能資產不含直接讀檔指示).
//!
//! The detectable pattern for "a direct-read instruction": a read/glob/open
//! word followed (same line) by a spec-document path — `specs/`, `changes/`,
//! `discussions/` under the spec dir, or the LANGUAGE document. Writing
//! paths (capture targets, git-status filters, archive layouts) stay legal.
//!
//! This file also pins per-skill registry contracts (flags a generated skill
//! must carry) when a spec requires them — see the quality case below.

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

/// Spec「品質關卡技能的生成與正典化」: quality is a registered canon skill, not a
/// hand-written local one. It orchestrates on the main thread (no fork), the unified
/// fix step edits files (Edit/Write stay allowed), and codex generates it too.
#[test]
fn quality_is_a_registered_canon_skill() {
    let registry = skills::registry();
    let quality = registry
        .iter()
        .find(|s| s.name == "quality")
        .expect("the registry must carry the quality skill");
    assert!(!quality.fork, "quality orchestrates on the main thread");
    assert!(
        !quality.disallow_edit,
        "the unified fix step edits files on the main thread"
    );
    assert!(!quality.claude_only, "quality is generated for codex too");
    assert!(
        !quality.body.trim().is_empty(),
        "the quality skill body must not be empty"
    );
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

/// The three skills a host's agent uses first (discuss, propose, ingest) must name a
/// verb for every spec-document read or write, so the same text works with no spec
/// files in the checkout (verb-contract「store 文件讀取動詞」; discuss-skill「正式規格
/// 與變更文件一律經動詞讀取」; propose-skill「artifact 的讀取與改寫一律經動詞」;
/// ingest-skill「既有 artifact 的更新經動詞整份寫回」). A regression check for those
/// three only, scanned in the Claude rendering and the custom tool's tool-call form.
#[test]
fn first_wave_skills_read_and_write_through_verbs() {
    let claude = TempRoot::new("first-wave-claude");
    init::init(&claude.dir, &[Tool::Claude], true, "speclink").unwrap();

    let tool_call = TempRoot::new("first-wave-tool-call");
    std::fs::create_dir_all(tool_call.dir.join("speclink").join("changes")).unwrap();
    std::fs::write(
        tool_call.dir.join(".speclink.yaml"),
        "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: tool-call\n",
    )
    .unwrap();
    init::update(&tool_call.dir, false).unwrap();

    let path_word = Regex::new(r"\bpath\b").unwrap();
    let renderings = [
        ("claude", claude.dir.join(".claude").join("skills")),
        ("tool-call", tool_call.dir.join(".wad").join("skills")),
    ];
    let mut problems: Vec<String> = Vec::new();
    for (form, skills_dir) in &renderings {
        for name in ["discuss", "propose", "ingest"] {
            let file = skills_dir.join(format!("speclink-{name}")).join("SKILL.md");
            let content = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{form}: cannot read {}: {e}", file.display()));
            let mut problem = |what: String| problems.push(format!("{form} speclink-{name}: {what}"));

            for (i, line) in content.lines().enumerate() {
                if path_word.is_match(line) && line.contains("spec.md") {
                    problem(format!("line {} puts `path` and spec.md together: {}", i + 1, line.trim()));
                }
                if line.contains("Move the record file") && !line.contains("checkout") {
                    problem(format!("line {} moves the record with no checkout condition", i + 1));
                }
            }
            if !content.contains("speclink artifact cat") {
                problem("never names `speclink artifact cat`".into());
            }
            if name != "discuss" {
                if content.contains("config.yaml") {
                    problem("mentions config.yaml".into());
                }
                if !content.contains("--stdin --force") {
                    problem("never names the `--stdin --force` write-back".into());
                }
            } else if !content.contains("speclink show") {
                problem("never names `speclink show`".into());
            }
        }
    }
    assert!(
        problems.is_empty(),
        "discuss, propose and ingest must read and write spec documents through verbs:\n{}",
        problems.join("\n")
    );
}

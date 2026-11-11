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

/// The rendered discuss, propose and ingest of one rendering form.
struct FirstWave {
    form: &'static str,
    discuss: String,
    propose: String,
    ingest: String,
}

impl FirstWave {
    fn skills(&self) -> [(&'static str, &str); 3] {
        [("discuss", &self.discuss), ("propose", &self.propose), ("ingest", &self.ingest)]
    }
}

/// The three skills in the Claude rendering and in a custom tool's tool-call
/// form.
fn first_wave_renderings(tag: &str) -> Vec<FirstWave> {
    let claude = TempRoot::new(&format!("{tag}-claude"));
    init::init(&claude.dir, &[Tool::Claude], true, "speclink").unwrap();

    let tool_call = TempRoot::new(&format!("{tag}-tool-call"));
    std::fs::create_dir_all(tool_call.dir.join("speclink").join("changes")).unwrap();
    std::fs::write(
        tool_call.dir.join(".speclink.yaml"),
        "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: tool-call\n",
    )
    .unwrap();
    init::update(&tool_call.dir, false).unwrap();

    let read = |dir: &PathBuf, name: &str| {
        let file = dir.join(format!("speclink-{name}")).join("SKILL.md");
        std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("cannot read {}: {e}", file.display()))
    };
    [("claude", claude.dir.join(".claude").join("skills")), ("tool-call", tool_call.dir.join(".wad").join("skills"))]
        .into_iter()
        .map(|(form, dir)| FirstWave {
            form,
            discuss: read(&dir, "discuss"),
            propose: read(&dir, "propose"),
            ingest: read(&dir, "ingest"),
        })
        .collect()
}

/// The three skills a host's agent uses first (discuss, propose, ingest) must name a
/// verb for every spec-document read or write, so the same text works with no spec
/// files in the checkout (verb-contract「store 文件讀取動詞」; discuss-skill「正式規格
/// 與變更文件一律經動詞讀取」; propose-skill「artifact 的讀取與改寫一律經動詞」;
/// ingest-skill「既有 artifact 的更新經動詞整份寫回」). A regression check for those
/// three only, scanned in the Claude rendering and the custom tool's tool-call form.
#[test]
fn first_wave_skills_read_and_write_through_verbs() {
    // The list's `path` field — not another object's `.path` (a change's `worktree.path`).
    let path_word = Regex::new(r"(?:^|[^.\w])path\b").unwrap();
    let mut problems: Vec<String> = Vec::new();
    for rendering in first_wave_renderings("first-wave") {
        for (name, content) in rendering.skills() {
            let form = rendering.form;
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

/// The move-back condition the renderer injects into discuss and propose
/// (`{{SPEC_FOLDER_CONDITION}}`, skills-verbs-first-wave D9). Pinned here verbatim.
const SPEC_FOLDER_CONDITION: &str = "only when the spec folder is in this checkout; when it is not (a remote project, or a host that embeds the engine), move nothing — stop and tell the user that this step has to be done on the Server that holds the specs";

/// The sentence-level checks the first quality round asked for (skills-verbs-first-wave
/// D2): a canonical spec is read with its type named, the write-back and the move-back
/// condition are one shared text each, and a change checked out in a worktree is read
/// and written from that copy.
#[test]
fn first_wave_skills_share_their_wording_and_name_the_spec_type() {
    let fragment_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills/artifact-write-back.md");
    let fragment = std::fs::read_to_string(&fragment_file).unwrap_or_default();
    let fragment = fragment.trim();
    let show = Regex::new(r"speclink show <(?:capability|spec-id)>( --item-type spec)?").unwrap();
    let mut problems: Vec<String> = Vec::new();
    if fragment.is_empty() {
        problems.push(format!("{} is missing or empty", fragment_file.display()));
    } else if !fragment.contains("do not edit the file instead") {
        problems.push("the write-back fragment never says not to edit the file instead".into());
    }
    for rendering in first_wave_renderings("first-wave-shared") {
        let form = rendering.form;
        for (name, content) in rendering.skills() {
            let mut problem = |what: String| problems.push(format!("{form} speclink-{name}: {what}"));
            if content.contains("{{") {
                problem("a placeholder was left in".into());
            }
            if name != "discuss" && !fragment.is_empty() && content.matches(fragment).count() != 1 {
                problem(format!("carries the shared write-back {} times, not once", content.matches(fragment).count()));
            }
            if name != "ingest" {
                if content.matches(SPEC_FOLDER_CONDITION).count() != 1 {
                    problem("does not carry the shared move-back condition exactly once".into());
                }
                let reads: Vec<_> = show.captures_iter(content).collect();
                if reads.is_empty() || reads.iter().any(|c| c.get(1).is_none()) {
                    problem("reads a canonical spec without `--item-type spec`".into());
                }
            }
        }
        if !rendering.ingest.lines().any(|l| l.contains("worktree.path") && l.contains("steps 4 to 8")) {
            problems.push(format!("{form} speclink-ingest: never runs steps 4 to 8 from the change's worktree.path"));
        }
        let named = rendering
            .discuss
            .lines()
            .find(|l| l.trim_start().starts_with("3. If the user mentioned a specific change name"))
            .unwrap_or_default();
        if !(named.contains("worktree.path") && named.contains("specs/<capability>")) {
            problems.push(format!("{form} speclink-discuss: step 3 lacks the worktree.path and deltaSpecs reads"));
        }
    }
    assert!(problems.is_empty(), "the first-wave skills drifted from their shared wording:\n{}", problems.join("\n"));
}

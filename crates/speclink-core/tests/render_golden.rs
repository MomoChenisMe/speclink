//! Golden tests for skill / instruction-block rendering.
//!
//! Three render targets exist: built-in claude, built-in codex, and custom descriptors
//! (neutral body, wording decided by `invocation`). The claude and codex snapshots lock
//! the pre-existing output BIT-FOR-BIT — the neutral work must not drift them. The
//! neutral snapshots pin the cli and tool-call wordings.
//!
//! Regenerate goldens deliberately with: UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden

use speclink_core::init;
use speclink_core::skills::{self, Tool};
use std::path::{Path, PathBuf};

struct TempRoot {
    dir: PathBuf,
}

impl TempRoot {
    fn new(tag: &str) -> TempRoot {
        let dir = std::env::temp_dir().join(format!(
            "speclink-render-golden-{tag}-{}",
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

/// Collect the instructions file plus every generated SKILL.md (registry order) as
/// (relative path, content) pairs.
fn generated_files(root: &Path, instructions_file: &str, skills_dir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    out.push((
        instructions_file.to_string(),
        std::fs::read_to_string(root.join(instructions_file)).expect(instructions_file),
    ));
    for skill in skills::registry() {
        let rel = format!("{skills_dir}/speclink-{}/SKILL.md", skill.name);
        let path = root.join(rel.split('/').collect::<PathBuf>());
        if let Ok(content) = std::fs::read_to_string(&path) {
            out.push((rel, content));
        }
    }
    out
}

/// Aggregate the generated files into one deterministic snapshot string.
fn snapshot(root: &Path, instructions_file: &str, skills_dir: &str) -> String {
    generated_files(root, instructions_file, skills_dir)
        .into_iter()
        .map(|(rel, content)| format!("=== {rel} ===\n{content}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_matches_golden(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("missing golden {name} — generate it with UPDATE_GOLDEN=1 after reviewing the output")
    });
    assert_eq!(
        actual, expected,
        "rendered output drifted from golden {name} (regenerate deliberately with UPDATE_GOLDEN=1)"
    );
}

// --- built-in targets: bit-level regression locks ---

#[test]
fn claude_rendering_is_bit_identical_to_golden() {
    let root = TempRoot::new("claude");
    init::init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
    assert_matches_golden(
        "claude.snapshot.md",
        &snapshot(&root.dir, "CLAUDE.md", ".claude/skills"),
    );
}

#[test]
fn codex_rendering_is_bit_identical_to_golden() {
    let root = TempRoot::new("codex");
    init::init(&root.dir, &[Tool::Codex], true, "openspec").unwrap();
    assert_matches_golden(
        "codex.snapshot.md",
        &snapshot(&root.dir, "AGENTS.md", ".agents/skills"),
    );
}

// --- neutral target: descriptor-generated content ---

fn neutral_project(tag: &str, invocation: &str) -> TempRoot {
    let root = TempRoot::new(tag);
    std::fs::create_dir_all(root.dir.join("openspec").join("changes")).unwrap();
    std::fs::write(
        root.dir.join(".speclink.yaml"),
        format!(
            "tools:\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n    invocation: {invocation}\n"
        ),
    )
    .unwrap();
    init::update(&root.dir).unwrap();
    root
}

#[test]
fn neutral_body_has_no_slash_prefix_and_no_plan_mode_references() {
    // Spec requirement 中性渲染目標: no /speclink- prefix, no plan-mode wording.
    for invocation in ["cli", "tool-call"] {
        let root = neutral_project(&format!("neutral-scan-{invocation}"), invocation);
        for (rel, content) in generated_files(&root.dir, "WAD.md", ".wad/skills") {
            if let Some(i) = content.find("/speclink-").or_else(|| content.find("$speclink-")) {
                let lo = i.saturating_sub(120);
                panic!(
                    "{invocation}: {rel} must not carry a tool-specific skill prefix; found near:\n…{}…",
                    &content[lo..(i + 120).min(content.len())]
                );
            }
            assert!(
                !content.to_ascii_lowercase().contains("plan mode"),
                "{invocation}: {rel} must not reference plan mode"
            );
            assert!(
                !content.contains("{{"),
                "{invocation}: {rel} must have all placeholders substituted"
            );
        }
    }
}

#[test]
fn neutral_cli_wording_runs_speclink_verbs() {
    // Spec design: cli invocation wording is "run `speclink <verb>`".
    let root = neutral_project("neutral-cli", "cli");
    let snap = snapshot(&root.dir, "WAD.md", ".wad/skills");
    assert!(
        snap.contains("run `speclink"),
        "cli wording must instruct running speclink commands:\n{}",
        &snap[..snap.len().min(2000)]
    );
    assert_matches_golden("neutral-cli.snapshot.md", &snap);
}

#[test]
fn neutral_tool_call_wording_calls_the_speclink_tool() {
    // Spec scenario tool-call 措辭: verbs are referenced by calling the speclink tool
    // with an argv array.
    let root = neutral_project("neutral-tool-call", "tool-call");
    let snap = snapshot(&root.dir, "WAD.md", ".wad/skills");
    assert!(
        snap.contains("calling the speclink tool") && snap.contains("argv"),
        "tool-call wording must instruct calling the speclink tool with argv:\n{}",
        &snap[..snap.len().min(2000)]
    );
    assert_matches_golden("neutral-tool-call.snapshot.md", &snap);
}

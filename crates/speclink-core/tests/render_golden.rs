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

/// Line endings are normalized to LF on both sides: the rendered output mixes
/// the checkout's asset line endings (CRLF under core.autocrlf=true) with the
/// engine's own `\n` formatting, so a byte-level comparison would depend on
/// the machine's git config rather than on the content.
fn normalize_eol(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn assert_matches_golden(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(name);
    let actual = normalize_eol(actual);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("missing golden {name} — generate it with UPDATE_GOLDEN=1 after reviewing the output")
    });
    assert_eq!(
        actual,
        normalize_eol(&expected),
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

// --- commit skill: confirmation gate requires visible plan + message ---

/// Spec requirement commit 確認閘門所見即所簽: the rendered commit skill must
/// generate the commit message before the user-confirmation step, and its
/// guardrails must forbid asking for confirmation before the plan and message
/// have been output as visible text.
#[test]
fn commit_skill_confirmation_gate_sees_plan_and_message() {
    let cases = [
        ("commit-gate-claude", Tool::Claude, ".claude/skills"),
        ("commit-gate-codex", Tool::Codex, ".agents/skills"),
    ];
    for (tag, tool, skills_dir) in cases {
        let root = TempRoot::new(tag);
        init::init(&root.dir, &[tool], true, "openspec").unwrap();
        let rel = format!("{skills_dir}/speclink-commit/SKILL.md");
        let content = std::fs::read_to_string(root.dir.join(rel.split('/').collect::<PathBuf>()))
            .expect(&rel);
        let generate = content
            .find("**Generate commit message**")
            .unwrap_or_else(|| panic!("{rel}: missing generate-commit-message step"));
        let confirm = content
            .find("**User confirmation**")
            .unwrap_or_else(|| panic!("{rel}: missing user-confirmation step"));
        assert!(
            generate < confirm,
            "{rel}: the commit message must be generated before the user-confirmation step"
        );
        assert!(
            content.contains("output as visible message text"),
            "{rel}: guardrails must require the plan and message to be visible before AskUserQuestion"
        );
        assert!(
            content.contains("must not reference content"),
            "{rel}: guardrails must forbid referencing content that was never displayed"
        );
    }
}

// --- remote marker variant: (tool target) × (fs | remote) ---

/// The remote marker block must not steer the agent at local spec paths that
/// don't exist in remote mode; documents are reached through speclink verbs.
fn assert_remote_marker(content: &str, file: &str) {
    assert!(
        content.contains("SPECLINK:START"),
        "{file} carries the SPECLINK marker block"
    );
    assert!(
        !content.contains("openspec/specs") && !content.contains("openspec/changes"),
        "{file}: remote marker must not mention local spec paths:\n{content}"
    );
    assert!(
        content.contains("speclink"),
        "{file}: remote marker keeps the verb guidance"
    );
}

#[test]
fn remote_marker_replaces_paths_with_verb_guidance() {
    let root = TempRoot::new("remote-marker");
    init::init_remote(
        &root.dir,
        &[Tool::Claude, Tool::Codex],
        true,
        "https://team.example.com/api/speclink/v1/projects/foo",
        Some("backend"),
    )
    .unwrap();
    for file in ["CLAUDE.md", "AGENTS.md"] {
        let content = std::fs::read_to_string(root.dir.join(file)).expect(file);
        assert_remote_marker(&content, file);
    }
}

#[test]
fn remote_marker_claude_matches_golden() {
    let root = TempRoot::new("remote-marker-claude");
    init::init_remote(
        &root.dir,
        &[Tool::Claude],
        true,
        "https://team.example.com/api/speclink/v1/projects/foo",
        Some("backend"),
    )
    .unwrap();
    let content = std::fs::read_to_string(root.dir.join("CLAUDE.md")).unwrap();
    assert_matches_golden("remote-claude.marker.md", &content);
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

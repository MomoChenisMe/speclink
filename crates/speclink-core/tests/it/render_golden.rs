//! Golden tests for skill / instruction-block rendering.
//!
//! Three render targets exist: built-in claude, built-in codex, and custom descriptors
//! (neutral body, wording decided by `invocation`). The claude and codex snapshots lock
//! the pre-existing output BIT-FOR-BIT — the neutral work must not drift them. The
//! neutral snapshots pin the cli and tool-call wordings.
//!
//! Regenerate goldens deliberately with: UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::

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

/// The second policy dimension. The snapshots above lock the DEFAULT generation set
/// (worktree off), which contains neither gated skill — without this, their rendered
/// content is locked nowhere. One tool target is enough: the gate adds the same two
/// skills either way, and the codex wording is already covered by the content tests
/// and the neutral snapshots.
#[test]
fn claude_rendering_with_the_worktree_policy_on_is_bit_identical_to_golden() {
    let root = TempRoot::new("claude-worktree");
    init::init(&root.dir, &[Tool::Claude], true, "openspec").unwrap();
    std::fs::write(
        root.dir.join("openspec").join("config.yaml"),
        "schema: spec-driven\nworktree: true\n",
    )
    .unwrap();
    init::update(&root.dir).unwrap();
    assert_matches_golden(
        "claude-worktree.snapshot.md",
        &snapshot(&root.dir, "CLAUDE.md", ".claude/skills"),
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

// --- archive / commit skills: trace 與 evidence gate 的技能敘述 ---

/// Spec archive-skill「trace 與 evidence 的技能敘述」Scenario「技能檔無刪除
/// 步驟、整潔要求與守門敘述殘留」: the rendered archive skill must not instruct
/// anyone to delete the evidence record, demand a clean work tree before a bulk
/// archive, or describe an evidence gate — all three obligations are retired.
#[test]
fn archive_skill_has_no_deletion_clean_tree_or_evidence_gate_residue() {
    for (rel, content) in skill_for_both_tools("archive-residue", "archive") {
        for forbidden in [
            "rm -f .speclink/touched",
            "Clean up tracking file",
            "Clean work tree required",
            "requires a clean work tree",
            "--waive-evidence",
            "evidence gate",
            "Evidence stale",
        ] {
            assert!(
                !content.contains(forbidden),
                "{rel}: retired instruction still present: {forbidden:?}"
            );
        }
    }
}

/// Same requirement, Scenario「trace 與提示敘述到位」: @trace is described as two
/// fields injected unconditionally with no file list, and the zero-evidence note
/// is described as a note (not a refusal) with when to act on it.
#[test]
fn archive_skill_describes_two_field_trace_and_the_zero_evidence_note() {
    for (rel, content) in skill_for_both_tools("archive-trace-note", "archive") {
        for needle in [
            // @trace: 兩欄、一律注入、無檔案清單
            "`source` (the change\n   name) and `updated` (the archive date)",
            "Injection is unconditional",
            "the canon carries no file list",
            // 零證據提示：出現條件、非拒絕、應對
            "no task evidence recorded for change",
            "It is a note, not a refusal",
            "spec-only or docs-only change earns no code evidence",
            "speclink-apply` before archiving",
        ] {
            assert!(
                content.contains(needle),
                "{rel}: missing trace/note phrase {needle:?}"
            );
        }
    }
}

/// Spec verify-evidence「task done 寫入逐任務 evidence」的技能面: the commit
/// skill's file attribution reads the change directory's record, falls back to
/// the pre-move path, and treats the record itself as part of the commit.
#[test]
fn commit_skill_attributes_files_from_the_change_directory_record() {
    for (rel, content) in skill_for_both_tools("commit-evidence", "commit") {
        for needle in [
            "openspec/changes/<change-name>/.evidence.json",
            "the pre-move location `.speclink/touched/<change-name>.json`",
            "belongs to this change's commit",
        ] {
            assert!(
                content.contains(needle),
                "{rel}: missing evidence-record phrase {needle:?}"
            );
        }
    }
}

// --- review skill: locale binds the whole output chain ---

/// Spec requirement: 審查產出的語言綁定 — the review skill binds the resolved
/// `locale` across the whole output chain (sub-agent briefs, the round
/// presentation, the ticket record), keeps severity labels and axis prefixes
/// in English, and defaults to English when `locale` is absent.
#[test]
fn review_skill_binds_locale_across_output_chain() {
    let cases = [
        ("review-locale-claude", Tool::Claude, ".claude/skills"),
        ("review-locale-codex", Tool::Codex, ".agents/skills"),
    ];
    for (tag, tool, skills_dir) in cases {
        let root = TempRoot::new(tag);
        init::init(&root.dir, &[tool], true, "openspec").unwrap();
        let rel = format!("{skills_dir}/speclink-review/SKILL.md");
        let content = std::fs::read_to_string(root.dir.join(rel.split('/').collect::<PathBuf>()))
            .expect(&rel);
        for needle in [
            "finding descriptions are written in that language",
            "severity labels, the `Standards:` / `Correctness:` axis prefixes, file paths, and command lines stay in English",
            "If `locale` is absent, everything is English",
            "never translate",
        ] {
            assert!(
                content.contains(needle),
                "{rel}: missing locale-binding phrase {needle:?}"
            );
        }
    }
}

// --- apply / review skills: converge-review-remediation-rounds (D6–D8) ---

/// Read one generated skill for both built-in tools. `tag` keeps parallel
/// tests in distinct sandboxes — two tests sharing a tag race on the same
/// temp dir.
fn skill_for_both_tools(tag: &str, skill: &str) -> Vec<(String, String)> {
    generated_skill_for_both_tools(tag, skill, false)
}

/// [`skill_for_both_tools`] for the skills gated behind the worktree policy: their
/// fixture must turn the policy on, or they are not in the generation set at all.
fn worktree_skill_for_both_tools(tag: &str, skill: &str) -> Vec<(String, String)> {
    generated_skill_for_both_tools(tag, skill, true)
}

fn generated_skill_for_both_tools(tag: &str, skill: &str, worktree: bool) -> Vec<(String, String)> {
    let cases = [
        (format!("{tag}-claude"), Tool::Claude, ".claude/skills"),
        (format!("{tag}-codex"), Tool::Codex, ".agents/skills"),
    ];
    cases
        .into_iter()
        .map(|(tag, tool, skills_dir)| {
            let root = TempRoot::new(&tag);
            init::init(&root.dir, &[tool], true, "openspec").unwrap();
            if worktree {
                std::fs::write(
                    root.dir.join("openspec").join("config.yaml"),
                    "schema: spec-driven\nworktree: true\n",
                )
                .unwrap();
                init::update(&root.dir).unwrap();
            }
            let rel = format!("{skills_dir}/speclink-{skill}/SKILL.md");
            let content =
                std::fs::read_to_string(root.dir.join(rel.split('/').collect::<PathBuf>()))
                    .expect(&rel);
            (rel, content)
        })
        .collect()
}

/// Spec「審查流程的技能行為」×「Apply 開始前記錄 host-local baseline」: apply
/// captures the review baseline BEFORE the first in-progress add, and a failed
/// prepare stops the flow.
#[test]
fn apply_skill_prepares_the_baseline_before_in_progress() {
    for (rel, content) in skill_for_both_tools("apply-prepare", "apply") {
        let prepare = content
            .find("review prepare")
            .unwrap_or_else(|| panic!("{rel}: missing the review prepare step"));
        let in_progress = content
            .find("in-progress add")
            .unwrap_or_else(|| panic!("{rel}: missing the in-progress add step"));
        assert!(
            prepare < in_progress,
            "{rel}: review prepare must run before in-progress add"
        );
        assert!(
            content.contains("review prepare` fails"),
            "{rel}: must state what happens when prepare fails"
        );
        assert!(
            content.contains("do NOT run"),
            "{rel}: a failed prepare must forbid running in-progress add"
        );
    }
}

/// Spec「審查流程的技能行為」: the first round is the one discovery pass — both
/// axes receive the SAME frozen patch from review scope; needsInput waits for
/// an explicit disposal; whole-file / commit-graph scoping is gone.
#[test]
fn review_skill_freezes_one_discovery_patch_for_both_axes() {
    for (rel, content) in skill_for_both_tools("review-freeze", "review") {
        assert!(
            content.contains("speclink review scope"),
            "{rel}: scope resolution must go through the review scope verb"
        );
        assert!(
            content.contains("same frozen patch"),
            "{rel}: both axes must receive the same frozen patch"
        );
        assert!(
            content.contains("--candidate-hash") && content.contains("--include-hunk"),
            "{rel}: needsInput must name the hash-pinned selection disposal"
        );
        assert!(
            !content.contains("...HEAD"),
            "{rel}: commit-graph three-dot scoping must be gone"
        );
        assert!(
            !content.contains("git diff --name-only"),
            "{rel}: whole-file name-only scoping must be gone"
        );
    }
}

/// Spec「審查流程的技能行為」(validation 分流): follow-up rounds judge each
/// original finding resolved/unresolved plus direct regressions only — no new
/// exploration of unchanged areas, unresolved findings carried verbatim.
#[test]
fn review_skill_validation_only_validates_findings_and_direct_regressions() {
    for (rel, content) in skill_for_both_tools("review-validate", "review") {
        assert!(
            content.contains("resolved or unresolved"),
            "{rel}: validation judges each original finding resolved/unresolved"
        );
        assert!(
            content.contains("directly introduces"),
            "{rel}: only regressions the remediation patch directly introduces are reported"
        );
        assert!(
            content.contains("must NOT report new smells"),
            "{rel}: validation never explores unchanged areas for new findings"
        );
        assert!(
            content.contains("never reworded"),
            "{rel}: unresolved findings are carried verbatim, never reworded"
        );
    }
}

/// Spec「審查後的迴圈與收尾」: the blocking set must strictly shrink for the
/// loop to continue — 2→1 continues, 1→1 fails immediately, 1→0 stamps clean,
/// accepted-only goes through an explicit --accept.
#[test]
fn review_skill_strict_progress_terminates_the_loop() {
    for (rel, content) in skill_for_both_tools("review-progress", "review") {
        assert!(
            content.contains("strictly smaller"),
            "{rel}: continuation requires a strictly smaller blocking set"
        );
        assert!(
            content.contains("not strictly smaller") && content.contains("failed"),
            "{rel}: the first no-progress round ends the loop as failed"
        );
        assert!(
            content.contains("passed clean") && content.contains("passed with reservations"),
            "{rel}: the two stamp outcomes are named"
        );
        assert!(
            content.contains("--accept"),
            "{rel}: accepted-only rounds stamp via an explicit --accept"
        );
        assert!(
            content.contains("no fixed maximum"),
            "{rel}: no fixed round cap — strict shrinking is the only continuation rule"
        );
        assert!(
            content.contains("never a quality score"),
            "{rel}: the shrinking set must not be described as a quality verdict"
        );
        assert!(
            content.contains("retry") && content.contains("review stamp"),
            "{rel}: a zero-findings last round retries the stamp instead of re-reviewing"
        );
    }
}

/// Spec「續輪重大晚發問題的安全退出」: unrelated new observations never join
/// the current round; only evidenced, severe issues end the station as scope
/// changed / failed.
#[test]
fn review_skill_late_findings_have_a_guarded_exit() {
    for (rel, content) in skill_for_both_tools("review-late", "review") {
        assert!(
            content.contains("added to the current round"),
            "{rel}: unrelated late findings must not join the current round"
        );
        assert!(
            content.contains("scope changed"),
            "{rel}: the evidenced-severe exit is named scope changed"
        );
        assert!(
            content.contains("failing test") && content.contains("invariant"),
            "{rel}: the evidence bar names reproduction/failing-test/invariant"
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

// --- embedded-asset version lock: bump discipline as a red test ---
//
// Spec requirement 內嵌資產版本鎖定紀律 (design 決策 8). The lock file records the
// product-layer version and a fingerprint of every render output. Changing an asset
// without bumping MARKER_VERSION fails here; the failure message carries the fix.

const ASSETS_LOCK: &str = "assets.lock";
const LOCK_REGEN_ENV: &str = "UPDATE_ASSETS_LOCK";

/// FNV-1a 64-bit, written out here on purpose: `DefaultHasher` is not stable across
/// Rust versions (false red), and a cryptographic hash is overkill for change detection.
fn fingerprint(input: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Every render output the lock covers, aggregated deterministically: both marker
/// variants (fs and remote) for each built-in tool and for a descriptor, plus the three
/// skill render targets for every registered skill.
///
/// MARKER_VERSION is stamped INTO those outputs (marker header, skill frontmatter), so it
/// is normalized out before hashing: the lock tracks asset CONTENT, and a version bump on
/// unchanged content must stay green (spec: 僅遞增版號而 render 內容未變 SHALL 通過).
fn render_fingerprint_input() -> String {
    let custom = speclink_core::config::CustomTool {
        name: "lock-harness".to_string(),
        skills_dir: ".lock/skills".to_string(),
        instructions_file: "LOCK.md".to_string(),
        invocation: speclink_core::config::Invocation::Cli,
    };
    let mut parts = Vec::new();
    for store in [init::StoreKind::Fs, init::StoreKind::Remote] {
        for tool in [Tool::Claude, Tool::Codex] {
            // Both worktree variants: the policy chooses between two marker
            // renderings, so the lock must cover the content of each.
            for worktree in [false, true] {
                parts.push(init::instructions_body("openspec", tool, store, worktree));
            }
        }
        parts.push(init::custom_instructions_body("openspec", &custom, store));
    }
    for skill in skills::registry() {
        for target in [
            skills::RenderTarget::Builtin(Tool::Claude),
            skills::RenderTarget::Builtin(Tool::Codex),
            skills::RenderTarget::Custom(&custom),
        ] {
            parts.push(skills::render_skill_file_for(target, &skill, "openspec"));
        }
    }
    normalize_eol(&parts.join("\n")).replace(init::MARKER_VERSION, "{{PRODUCT_VERSION}}")
}

fn lock_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("golden")
        .join(ASSETS_LOCK)
}

/// The lock file is two `key: value` lines (version, fingerprint) — the format is owned
/// by this test alone.
fn read_lock() -> Option<(String, String)> {
    let text = std::fs::read_to_string(lock_path()).ok()?;
    let mut version = None;
    let mut hash = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key.trim() {
            "version" => version = Some(value.trim().to_string()),
            "fingerprint" => hash = Some(value.trim().to_string()),
            _ => {}
        }
    }
    Some((version?, hash?))
}

fn write_lock(version: &str, hash: &str) {
    let path = lock_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, format!("version: {version}\nfingerprint: {hash}\n")).unwrap();
}

/// The one message both the failure path and the regeneration guard point at, so an agent
/// that reached for the wrong env var is told what to do instead. UPDATE_GOLDEN and
/// UPDATE_ASSETS_LOCK are independent switches: regenerating goldens never touches the lock.
fn lock_fix_instructions() -> String {
    format!(
        "fix: bump MARKER_VERSION in crates/speclink-core/src/init.rs (render content changed), \
then regenerate the lock on a clean tree with \
`{LOCK_REGEN_ENV}=1 cargo test -p speclink-core --test it render_golden::`. \
Regenerating goldens (UPDATE_GOLDEN=1) does NOT update this lock."
    )
}

#[test]
fn embedded_assets_are_locked_to_the_product_version() {
    let current_version = init::MARKER_VERSION;
    let current_hash = fingerprint(&render_fingerprint_input());
    let locked = read_lock();

    if std::env::var(LOCK_REGEN_ENV).is_ok() {
        // Regeneration guard: a changed fingerprint under an unchanged version must NOT
        // be written through — that is exactly the discipline this lock exists to enforce.
        if let Some((locked_version, locked_hash)) = &locked {
            assert!(
                !(locked_hash != &current_hash && locked_version == current_version),
                "refusing to rewrite {ASSETS_LOCK}: render output changed but MARKER_VERSION is \
still {current_version}.\n{}",
                lock_fix_instructions()
            );
        }
        write_lock(current_version, &current_hash);
        return;
    }

    let Some((locked_version, locked_hash)) = locked else {
        panic!(
            "missing {ASSETS_LOCK} — generate it on a clean tree with \
`{LOCK_REGEN_ENV}=1 cargo test -p speclink-core --test it render_golden::`.\n{}",
            lock_fix_instructions()
        );
    };
    if locked_hash == current_hash {
        return;
    }
    assert_ne!(
        locked_version, current_version,
        "embedded asset render output changed while MARKER_VERSION stayed at \
{current_version}.\n{}",
        lock_fix_instructions()
    );
    panic!(
        "MARKER_VERSION moved ({locked_version} → {current_version}) but {ASSETS_LOCK} still \
records the old fingerprint.\n{}",
        lock_fix_instructions()
    );
}

// --- worktree skills: generation, composition and the stop points ---

/// Spec「apply-with-worktree 技能的生成與組合」: both tool targets generate the
/// composed skill, and its body carries the WHOLE apply body verbatim (not a
/// summary, not a reference).
#[test]
fn apply_with_worktree_embeds_the_entire_apply_body() {
    for (rel, content) in worktree_skill_for_both_tools("apply-wt-compose", "apply-with-worktree") {
        let tool = if rel.starts_with(".claude") { Tool::Claude } else { Tool::Codex };
        let apply_body = skills::substitute(
            skills::skill_body("apply").expect("apply body"),
            tool,
            "openspec",
        );
        assert!(
            normalize_eol(&content).contains(normalize_eol(apply_body.trim_end()).as_str()),
            "{rel}: the apply body must appear verbatim, not paraphrased"
        );
    }
}

/// Spec「apply-with-worktree 技能的前置指示」: the policy gate refuses to run and
/// names the enabling command; the branch and nest conventions are stated, with
/// reuse over recreation.
#[test]
fn apply_with_worktree_states_the_policy_gate_and_the_creation_convention() {
    for (rel, content) in worktree_skill_for_both_tools("apply-wt-pre", "apply-with-worktree") {
        assert!(
            content.contains("workflow-config set worktree true"),
            "{rel}: must name the enabling command"
        );
        // The gate reads the EFFECTIVE policy: the SPECLINK_WORKTREE env layer
        // must be honored, matching the CLI's own resolution.
        assert!(
            content.contains("SPECLINK_WORKTREE"),
            "{rel}: the policy gate must honor the env override layer"
        );
        // The change's artifacts must reach HEAD before the worktree is created
        // — a worktree materialized from a HEAD without them is a dead end.
        let commit_step = content
            .find("into HEAD")
            .unwrap_or_else(|| panic!("{rel}: missing the commit-artifacts-into-HEAD step"));
        let worktree_add = content
            .find("git worktree add")
            .unwrap_or_else(|| panic!("{rel}: missing the worktree add step"));
        assert!(
            commit_step < worktree_add,
            "{rel}: artifacts must be committed before the worktree is created"
        );
        assert!(
            content.contains("本專案未啟用 worktree 流程"),
            "{rel}: must state the refusal in the user's terms"
        );
        assert!(
            content.contains("speclink/<change-name>"),
            "{rel}: must state the branch convention"
        );
        assert!(
            content.contains(".worktrees/<change-name>/"),
            "{rel}: must state the sibling nest convention"
        );
        assert!(
            content.contains("reuse it and continue there"),
            "{rel}: an existing worktree must be reused, not recreated"
        );
    }
}

/// Same requirement, the multi-change guard: more than one change name stops the
/// run for the user to pick one, batching is forbidden in so many words, and the
/// guard sits ahead of the policy gate (nothing else should run first).
#[test]
fn apply_with_worktree_refuses_more_than_one_change() {
    for (rel, content) in worktree_skill_for_both_tools("apply-wt-multi", "apply-with-worktree") {
        let guard = content
            .find("One change per run")
            .unwrap_or_else(|| panic!("{rel}: missing the one-change-per-run guard"));
        let policy_gate = content
            .find("Check the worktree policy")
            .unwrap_or_else(|| panic!("{rel}: missing the policy gate"));
        assert!(guard < policy_gate, "{rel}: the multi-change guard must precede the policy gate");
        assert!(
            content.contains("AskUserQuestion"),
            "{rel}: the user must pick which change to run"
        );
        assert!(
            content.contains("Do **NOT** run them one after another"),
            "{rel}: silent sequential batching must be forbidden"
        );
        assert!(
            content.contains("一個 change 一個 session"),
            "{rel}: must print the multi-session recipe in the user's terms"
        );
    }
}

/// Same requirement, the progress-vs-code guard: the evidence record is checked
/// against the main tree between the artifact commit and worktree creation, and a
/// dirty path stops for a three-way choice led by the commit route.
#[test]
fn apply_with_worktree_detects_progress_parted_from_code() {
    for (rel, content) in worktree_skill_for_both_tools("apply-wt-split", "apply-with-worktree") {
        let check = content
            .find(".evidence.json")
            .unwrap_or_else(|| panic!("{rel}: missing the evidence-record check"));
        let commit_step = content.find("into HEAD").expect("commit step");
        let worktree_add = content.find("git worktree add").expect("worktree add step");
        assert!(
            commit_step < check && check < worktree_add,
            "{rel}: the split check belongs after the artifact commit and before worktree creation"
        );
        // Absent or empty record: nothing was implemented, so the run continues.
        assert!(
            content.contains("Continue to P4 silently"),
            "{rel}: an empty or absent record must pass silently"
        );
        // Dirty: stop, and lead with the route that actually fixes it.
        let commit_route = content
            .find("先收程式碼再開 worktree")
            .unwrap_or_else(|| panic!("{rel}: missing the recommended commit route"));
        let carry_on = content.find("照樣繼續").expect("carry-on option");
        let stop = content.find("停止").expect("stop option");
        assert!(
            commit_route < carry_on && carry_on < stop,
            "{rel}: the three options must be offered in recommendation order"
        );
        assert!(
            content.contains("Do NOT create the worktree before the user has chosen"),
            "{rel}: worktree creation must wait for the user's choice"
        );
    }
}

/// Spec「apply-with-worktree 技能的收尾指示」: commit inside the worktree, stop
/// before merging, keep the worktree, and hand off by name.
#[test]
fn apply_with_worktree_stops_before_the_merge_and_hands_off() {
    for (rel, content) in worktree_skill_for_both_tools("apply-wt-post", "apply-with-worktree") {
        assert!(
            content.contains("Do NOT** merge") && content.contains("Do NOT** run `git worktree remove`"),
            "{rel}: must forbid both merging and removing the worktree"
        );
        assert!(
            content.contains("worktree-merge"),
            "{rel}: must name the follow-up skill"
        );
    }
}

// --- config skill: criterion 1 disproves engine injection AND station canon ---

/// Spec config-skill「技能規定固定輸入來源與四條內容判準」Scenario「渲染產物含四條
/// 判準與反證步驟」/「品質站正典不得重述進 rules」: criterion 1 carries two
/// disproof routes — the instructions payload for injected content, and the
/// generated quality-station skill for the canon it holds (the review station's
/// smell baseline reaches no payload, so the first route cannot see it) — and
/// the guardrail list restates the prohibition. The `speclink-review` reference
/// is kept honest by asserting the generated review skill still carries the
/// smell baseline the criterion points at.
#[test]
fn config_skill_criterion_one_disproves_station_canon_too() {
    for (rel, config) in skill_for_both_tools("config-station-canon", "config") {
        let start = config
            .find("### Criterion 1")
            .unwrap_or_else(|| panic!("{rel}: missing the criterion 1 section"));
        let len = config[start..]
            .find("### Criterion 2")
            .unwrap_or_else(|| panic!("{rel}: missing the criterion 2 section"));
        let criterion_one = &config[start..start + len];
        for needle in [
            // route (a): engine-injected content, disproved per line by payload
            "speclink instructions <artifact> --json",
            // route (b): station canon, disproved against the generated station
            // skills — only the ones present in the tool's skills directory
            "Quality-station canon",
            "same skills directory",
            "`speclink-review`",
            "present in that directory",
            "the station skill is its single home",
            "a second canon",
        ] {
            assert!(
                criterion_one.contains(needle),
                "{rel}: criterion 1 is missing the disproof phrase {needle:?}"
            );
        }
        // the guardrail restatement lives in the Guardrails section itself
        let guard = config
            .find("## Guardrails")
            .unwrap_or_else(|| panic!("{rel}: missing the guardrails section"));
        assert!(
            config[guard..].contains("**Don't restate quality-station canon**"),
            "{rel}: guardrails must forbid restating quality-station canon"
        );
    }
    // the reference target must still carry the canon it is named for — if the
    // smell baseline ever moves out of the review skill, this goes red instead
    // of criterion 1 pointing at an empty home. Rendering is deterministic per
    // tool, so a separate render pins the same content the criterion's reader
    // would open.
    for (review_rel, review) in skill_for_both_tools("config-station-review-home", "review") {
        for needle in ["Fowler code smells", "**Mysterious Name**"] {
            assert!(
                review.contains(needle),
                "{review_rel}: the smell baseline criterion 1 points at is no longer here ({needle:?} missing) — move the criterion's reference with it"
            );
        }
    }
}

/// Spec「worktree-merge 技能的生成」＋「收尾流程指示」: a standalone template whose
/// preflight, conflict and cleanup stop points are all stated.
#[test]
fn worktree_merge_skill_states_preflight_conflict_and_cleanup() {
    for (rel, content) in worktree_skill_for_both_tools("wt-merge", "worktree-merge") {
        // preflight: both trees clean, and never acting on the user's behalf
        assert!(
            content.contains("Main tree not clean") && content.contains("Worktree not fully committed"),
            "{rel}: both preflight conditions must be stated"
        );
        // preflight: the merge target is verified, not assumed — merging while
        // parked on another speclink/* branch must stop, and the target branch
        // is announced so a wrong destination is visible before the merge.
        assert!(
            content.contains("branch --show-current"),
            "{rel}: preflight must check which branch the main checkout is on"
        );
        assert!(
            content.contains("Do **NOT** stash on their behalf"),
            "{rel}: must forbid stashing for the user"
        );
        assert!(
            content.contains("Do **NOT** commit on their behalf"),
            "{rel}: must forbid committing for the user"
        );
        // merge ladder: rebase the branch onto the verified target inside the
        // worktree first, then land it as a fast-forward so the graph stays a
        // straight line instead of collecting a merge node per parallel change.
        assert!(
            content.contains("rebase \"<target-branch>\""),
            "{rel}: the ladder must rebase the branch onto the verified merge target"
        );
        assert!(
            content.contains("merge --ff-only"),
            "{rel}: a successful rebase must land as a fast-forward"
        );
        // the rebase runs inside the worktree, so the prerequisite must not
        // claim otherwise — a reader who believes it stops or rebases in the
        // wrong tree.
        assert!(
            !content.contains("not in the worktree"),
            "{rel}: the prerequisite contradicts step 3, which rebases inside the worktree"
        );
        // a fast-forward can be refused when the target moved on mid-flight —
        // with parallel worktrees that is a live race, and leaving it unstated
        // strands the agent on a mutating step.
        assert!(
            content.contains("If the fast-forward is refused"),
            "{rel}: a refused fast-forward must have a stated exit"
        );
        // ladder fallback: a rebase conflict restores the branch untouched and
        // drops back to the plain merge, so the worst case equals the old flow.
        assert!(
            content.contains("rebase --abort") && content.contains("fall back to a plain merge"),
            "{rel}: a rebase conflict must restore the branch and fall back to the plain merge"
        );
        // conflict: abort, report, never edit
        assert!(
            content.contains("merge --abort"),
            "{rel}: a conflict must abort rather than leave a half-merge"
        );
        assert!(
            content.contains("do **NOT** commit a partial merge"),
            "{rel}: must forbid committing a partial merge"
        );
        // the guardrail list must carry the rebase red line too, not just merge
        assert!(
            content.contains("Never resolve rebase conflicts"),
            "{rel}: the guardrails must forbid resolving rebase conflicts"
        );
        // both ladder exits reach the success output, so it has to say which
        // one was taken — a merge node the user was not told about is a surprise
        assert!(
            content.contains("Landed as"),
            "{rel}: the success output must state whether it landed as a fast-forward"
        );
        // cleanup and hand-off
        assert!(
            content.contains("worktree remove") && content.contains("branch -d"),
            "{rel}: a successful merge must remove the worktree and delete the branch"
        );
        // Prefix-agnostic: the slash token renders as `/speclink-` for Claude
        // and `$speclink-` for codex, so match the skill names themselves.
        assert!(
            content.contains("speclink-review")
                && content.contains("speclink-verify")
                && content.contains("speclink-archive"),
            "{rel}: must point at the quality stations and archive"
        );
        // standalone: it must NOT drag the apply body along
        assert!(
            !content.contains("Rationalization Table"),
            "{rel}: worktree-merge is a standalone template, not a composition"
        );
    }
}

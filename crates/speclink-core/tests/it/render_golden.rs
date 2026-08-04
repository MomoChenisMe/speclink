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
    let cases = [
        (format!("{tag}-claude"), Tool::Claude, ".claude/skills"),
        (format!("{tag}-codex"), Tool::Codex, ".agents/skills"),
    ];
    cases
        .into_iter()
        .map(|(tag, tool, skills_dir)| {
            let root = TempRoot::new(&tag);
            init::init(&root.dir, &[tool], true, "openspec").unwrap();
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
            parts.push(init::instructions_body("openspec", tool, store));
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

//! Generated skills read and write spec documents through speclink verbs, never by a
//! file path in the spec folder — the same skill text then works when the spec folder
//! is not in the checkout (verb-contract spec「store 文件讀取動詞」).
//!
//! Three rule groups run over every registered skill, rendered for Claude and for a
//! custom tool's tool-call form:
//!
//! 1. Rendered for a checkout WITHOUT the spec folder (a remote project, a host that
//!    embeds the engine): no spec-folder path, no `.evidence.json`, no `.meta.yaml`.
//! 2. Rendered for a checkout WITH the spec folder: a passage that carries a spec-folder
//!    path is a git step or a path shown to the user — it holds no read or write word.
//!    A passage is the lines up to the next blank line, so a path cannot escape the rule
//!    by moving to the line after the word; the words are matched with their endings
//!    (`reads`, `editing`, `rewritten`).
//! 3. No rendering leaves a placeholder behind.
//!
//! The positive conditions sit beside them: which skills carry the shared write-back,
//! which read evidence, deltas and canonical specs through the verb, and how dormancy
//! is judged.
//!
//! This file also pins per-skill registry contracts (flags a generated skill
//! must carry) when a spec requires them — see the quality case below.

use regex::Regex;
use speclink_core::config::{CustomTool, Invocation};
use speclink_core::skills::{self, RenderTarget, SpecLocation, Tool};

/// The spec folder every rendering here uses: `openspec/` then marks a spec-folder path.
const SPEC_DIR: &str = "openspec";

/// Skills that change an artifact that already exists: each carries the shared
/// write-back section exactly once.
const WRITE_BACK_SKILLS: [&str; 8] =
    ["propose", "ingest", "archive", "commit", "apply", "apply-with-worktree", "review", "verify"];

/// Skills that need the files a change touched.
const EVIDENCE_SKILLS: [&str; 3] = ["commit", "apply-with-worktree", "improve"];

/// Skills that ask whether a change has delta specs, and read the canonical spec beside them.
const DELTA_SKILLS: [&str; 3] = ["archive", "commit", "verify"];

/// Skills that judge how long a change sat idle.
const DORMANCY_SKILLS: [&str; 3] = ["apply", "apply-with-worktree", "drift"];

/// The one sentence apply and drift share for that judgement.
const DORMANCY_RULE: &str = "A change is dormant when the `created` of its entry in `speclink list --json` is more than 5 days ago and its `modifiedAt` is more than 3 days ago. An entry with no `modifiedAt` is judged on `created` alone — say then that the last activity cannot be read. An entry with no `created` is not dormant.";

/// What discuss and propose say about moving an archived discussion record back when
/// the spec folder is not in the checkout.
const MOVE_BACK_ELSEWHERE: &str = "stop and tell the user that this step has to be done on the Server that holds the specs";

/// One rendered skill file.
struct Rendering {
    form: &'static str,
    skill: &'static str,
    location: SpecLocation,
    content: String,
}

impl Rendering {
    fn label(&self) -> String {
        format!("{} speclink-{} ({:?})", self.form, self.skill, self.location)
    }

    fn exempt(&self) -> bool {
        crate::SPEC_FOLDER_WRITERS.iter().any(|(skill, _)| *skill == self.skill)
    }
}

fn tool_call_harness() -> CustomTool {
    CustomTool {
        name: "wad-harness".to_string(),
        skills_dir: ".wad/skills".to_string(),
        instructions_file: None,
        invocation: Invocation::ToolCall,
    }
}

/// Every registered skill as the two forms generate it; the neutral form has no
/// Claude-only skill.
fn renderings(location: SpecLocation) -> Vec<Rendering> {
    let harness = tool_call_harness();
    let mut out = Vec::new();
    for skill in skills::registry() {
        let forms = [
            ("claude", RenderTarget::Builtin(Tool::Claude), true),
            ("tool-call", RenderTarget::Custom(&harness), !skill.claude_only),
        ];
        for (form, target, generated) in forms {
            if generated {
                let content = skills::render_skill_file_for(target, &skill, SPEC_DIR, location);
                out.push(Rendering { form, skill: skill.name, location, content });
            }
        }
    }
    out
}

fn both_sides() -> Vec<Rendering> {
    SpecLocation::BOTH.into_iter().flat_map(renderings).collect()
}

fn numbered(index: usize, line: &str) -> String {
    format!("line {}: {}", index + 1, line.trim())
}

/// Rule 1: the lines that name a file in the spec folder.
fn spec_path_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            line.contains("openspec/") || crate::SPEC_FOLDER_FILE_NAMES.iter().any(|name| line.contains(name))
        })
        .map(|(index, line)| numbered(index, line))
        .collect()
}

/// The passages of `lines`, as line ranges. A passage is a run of lines with no blank
/// line between them; one that ends with a colon also takes the run after it, so a
/// lead-in sentence stays with the list or code block it introduces.
fn passages(lines: &[&str]) -> Vec<std::ops::Range<usize>> {
    let mut found: Vec<std::ops::Range<usize>> = Vec::new();
    let mut start = 0;
    while start < lines.len() {
        let end = start + lines[start..].iter().take_while(|line| !line.trim().is_empty()).count();
        if end > start {
            match found.last_mut() {
                Some(previous) if lines[previous.end - 1].trim_end().ends_with(':') => previous.end = end,
                _ => found.push(start..end),
            }
        }
        start = end + 1;
    }
    found
}

/// Rule 2: the passages that carry a spec-folder path together with a read or write
/// word, each reported at the line of its path with the word that was found. The words
/// are the verb-contract spec's list with their endings and prefixes (`reads`, `re-read`,
/// `rewritten`, `overwrite`), `write` itself, and `check whether` beside `check for`.
fn path_read_write_lines(content: &str) -> Vec<String> {
    let word = Regex::new(
        r"(?i)\b(?:re-?|over)?(?:read(?:s|ing)?|open(?:s|ed|ing)?|glob(?:s|bed|bing)?|grep(?:s|ped|ping)?|cats?|check(?:s|ed|ing)? (?:for|whether|that|if)|compar(?:e|es|ed|ing) (?:[^\s.;:]+ ){0,4}against|edit(?:s|ed|ing)?|writ(?:e|es|ing|ten)|wrote)\b",
    )
    .unwrap();
    let lines: Vec<&str> = content.lines().collect();
    let mut found = Vec::new();
    for range in passages(&lines) {
        if let Some(hit) = word.find(&lines[range.clone()].join("\n")) {
            found.extend(
                range
                    .filter(|index| lines[*index].contains("openspec/"))
                    .map(|index| format!("{} (the passage says `{}`)", numbered(index, lines[index]), hit.as_str())),
            );
        }
    }
    found
}

/// Rule 3.
fn leftover_placeholder_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("{{"))
        .map(|(index, line)| numbered(index, line))
        .collect()
}

/// One problem line per finding, each naming the rendering.
fn problems(rendering: &Rendering, what: &str, lines: Vec<String>) -> Vec<String> {
    lines.into_iter().map(|line| format!("{}: {what} — {line}", rendering.label())).collect()
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

/// Scenario「remote 專案的技能檔不含規格路徑」.
#[test]
fn skills_for_a_checkout_without_the_spec_folder_name_no_spec_path() {
    let mut found: Vec<String> = Vec::new();
    for rendering in renderings(SpecLocation::Elsewhere) {
        found.extend(problems(&rendering, "placeholder left in", leftover_placeholder_lines(&rendering.content)));
        if !rendering.exempt() {
            found.extend(problems(&rendering, "names a file in the spec folder", spec_path_lines(&rendering.content)));
        }
    }
    assert!(
        found.is_empty(),
        "a skill rendered for a checkout without the spec folder must not name a spec-folder path, \
`.evidence.json` or `.meta.yaml` — use a verb, or move the step into the `{{{{#SPEC_IN_CHECKOUT}}}}` block:\n{}",
        found.join("\n")
    );
}

/// Scenario「本機專案的技能檔不以路徑讀寫」.
#[test]
fn skills_for_a_checkout_with_the_spec_folder_never_read_or_write_by_path() {
    let mut found: Vec<String> = Vec::new();
    for rendering in renderings(SpecLocation::InCheckout) {
        found.extend(problems(&rendering, "placeholder left in", leftover_placeholder_lines(&rendering.content)));
        if !rendering.exempt() {
            found.extend(problems(
                &rendering,
                "reads or writes a spec document by path",
                path_read_write_lines(&rendering.content),
            ));
        }
    }
    assert!(
        found.is_empty(),
        "a line with a spec-folder path is a git step or a path shown to the user; reads and writes \
name a speclink verb:\n{}",
        found.join("\n")
    );
}

/// Scenario「守門抓到新加入的路徑讀取」: the rules, run on a sample source that reads a
/// spec file by path — once inside the in-checkout block, once outside any block.
#[test]
fn the_rules_catch_a_path_read_added_to_a_source() {
    let sample = "{{#SPEC_IN_CHECKOUT}}\ncat \"{{SPEC_DIR}}changes/<name>/.evidence.json\"\ngit add \"{{SPEC_DIR}}changes/<name>\"\n{{/SPEC_IN_CHECKOUT}}\nThen read `{{SPEC_DIR}}changes/<name>/tasks.md`.\n";
    let claude = RenderTarget::Builtin(Tool::Claude);

    let in_checkout = skills::substitute(sample, claude, SPEC_DIR, SpecLocation::InCheckout);
    assert_eq!(
        path_read_write_lines(&in_checkout),
        vec![
            "line 1: cat \"openspec/changes/<name>/.evidence.json\" (the passage says `cat`)".to_string(),
            "line 2: git add \"openspec/changes/<name>\" (the passage says `cat`)".to_string(),
            "line 3: Then read `openspec/changes/<name>/tasks.md`. (the passage says `cat`)".to_string(),
        ],
        "one passage: every path line in it is reported"
    );
    // A git step alone, and a path that is only shown, are legal.
    assert!(path_read_write_lines("git add \"openspec/changes/<name>\"\n\n**Archived to:** openspec/changes/archive/<name>/\n").is_empty());
    // The word on one line and the path on the next is the same passage; a blank line ends it.
    assert_eq!(path_read_write_lines("Now read the file at\n`openspec/changes/<name>/tasks.md`.\n").len(), 1);
    assert!(path_read_write_lines("Now read the design.\n\nIts file is `openspec/changes/<name>/design.md`.\n").is_empty());
    // Word endings do not hide the word.
    for sentence in [
        "The step reads `openspec/x`.",
        "Keep editing `openspec/x`.",
        "`openspec/x` is rewritten.",
        "It re-reads `openspec/x`.",
        "It is compared against `openspec/x`.",
        "Checking for `openspec/x`.",
        "It greps `openspec/x`.",
    ] {
        assert_eq!(path_read_write_lines(sentence).len(), 1, "{sentence}");
    }
    // The sentences this rule first let through: several words before `against`, a
    // `check whether`, a plain `write`, and a lead-in sentence with its block after a blank line.
    for passage in [
        "Then compare each delta against `openspec/specs/<capability>/spec.md`.",
        "Check whether delta specs exist at `openspec/changes/<name>/specs/`.",
        "Write the result to `openspec/x`.",
        "It overwrites `openspec/x`.",
        "Read these files:\n\n```\nopenspec/changes/<name>/tasks.md\n```",
    ] {
        assert_eq!(path_read_write_lines(passage).len(), 1, "{passage}");
    }
    // A lead-in that ends with a colon takes only the passage right after it.
    assert!(path_read_write_lines("Read the list:\n\n- one\n\n`openspec/x` is shown.\n").is_empty());
    // Other words that only contain one of them do not count.
    assert!(path_read_write_lines("Already located under `openspec/x`; the thread spreads.").is_empty());

    let elsewhere = skills::substitute(sample, claude, SPEC_DIR, SpecLocation::Elsewhere);
    assert_eq!(
        spec_path_lines(&elsewhere),
        vec!["line 1: Then read `openspec/changes/<name>/tasks.md`.".to_string()],
        "the block is gone; the path outside it is caught"
    );
    assert_eq!(spec_path_lines("See `.meta.yaml` and\n`.evidence.json`.\n").len(), 2);
}

/// The positive conditions: every skill that changes an existing artifact, reads the
/// touched files, checks deltas or judges dormancy names the verb for it — in both
/// forms and on both sides.
#[test]
fn skills_name_the_verb_for_every_read_and_write_back() {
    let fragment_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills/artifact-write-back.md");
    let fragment_source = std::fs::read_to_string(&fragment_file).unwrap_or_default();
    let mut found: Vec<String> = Vec::new();
    if !fragment_source.contains("do not edit the file instead") {
        found.push("the write-back fragment never says not to edit the file instead".into());
    }
    for rendering in both_sides() {
        let content = &rendering.content;
        let mut problem = |what: &str| found.push(format!("{}: {what}", rendering.label()));
        let has = |skills: &[&str]| skills.contains(&rendering.skill);

        if has(&WRITE_BACK_SKILLS) {
            let fragment = skills::resolve_blocks("artifact-write-back.md", &fragment_source, rendering.location)
                .expect("the write-back fragment's blocks are well-formed");
            let fragment = fragment.trim();
            if fragment.is_empty() || content.matches(fragment).count() != 1 {
                problem("does not carry the shared write-back exactly once");
            }
            if !content.contains("--stdin --force") {
                problem("never names the `--stdin --force` write-back");
            }
        }
        if has(&EVIDENCE_SKILLS) && !content.contains("speclink evidence show") {
            problem("never names `speclink evidence show`");
        }
        if has(&DELTA_SKILLS) {
            if !content.contains("deltaCapabilities") {
                problem("never names `deltaCapabilities`");
            }
            if !content.contains("--item-type spec") {
                problem("never reads a canonical spec with `speclink show <capability> --item-type spec`");
            }
            // The field is left out of the entry when the change has no delta spec.
            if !content.lines().any(|line| line.contains("deltaCapabilities") && line.contains("absent or empty")) {
                problem("never says that `deltaCapabilities` may be absent or empty");
            }
        }
        if has(&DORMANCY_SKILLS) {
            if content.matches(DORMANCY_RULE).count() != 1 {
                problem("does not carry the shared dormancy rule (`created`, `modifiedAt`) exactly once");
            }
            if content.lines().any(|line| line.contains("git log") && line.contains("openspec/")) {
                problem("judges dormancy with `git log` on the spec folder");
            }
        }
        if ["analyze", "trace", "improve"].contains(&rendering.skill) && !content.contains("speclink artifact cat") {
            problem("never names `speclink artifact cat`");
        }
    }
    assert!(found.is_empty(), "skills must name the verb for these reads and writes:\n{}", found.join("\n"));
}

/// The wording the first three skills were pinned to (discuss-skill「正式規格與變更文件
/// 一律經動詞讀取」; propose-skill「artifact 的讀取與改寫一律經動詞」; ingest-skill「既有
/// artifact 的更新經動詞整份寫回」), kept on both sides: a canonical spec is read with
/// its type named, a change checked out in a worktree is read and written from that
/// copy, and an archived discussion record is moved back only where the spec folder is.
#[test]
fn discuss_propose_and_ingest_keep_their_pinned_wording() {
    // The list's `path` field — not another object's `.path` (a change's `worktree.path`).
    let path_word = Regex::new(r"(?:^|[^.\w])path\b").unwrap();
    let show = Regex::new(r"speclink show <(?:capability|spec-id)>( --item-type spec)?").unwrap();
    let mut found: Vec<String> = Vec::new();
    for rendering in both_sides() {
        let (name, content) = (rendering.skill, &rendering.content);
        if !["discuss", "propose", "ingest"].contains(&name) {
            continue;
        }
        let mut problem = |what: String| found.push(format!("{}: {what}", rendering.label()));

        for (i, line) in content.lines().enumerate() {
            if path_word.is_match(line) && line.contains("spec.md") {
                problem(format!("line {} puts `path` and spec.md together: {}", i + 1, line.trim()));
            }
        }
        if !content.contains("speclink artifact cat") {
            problem("never names `speclink artifact cat`".into());
        }
        if name != "discuss" && content.contains("config.yaml") {
            problem("mentions config.yaml".into());
        }
        if name != "ingest" {
            let reads: Vec<_> = show.captures_iter(content).collect();
            if reads.is_empty() || reads.iter().any(|c| c.get(1).is_none()) {
                problem("reads a canonical spec without `--item-type spec`".into());
            }
            let moves_back = content.contains("Move the record file");
            let stops = content.matches(MOVE_BACK_ELSEWHERE).count() == 1;
            if rendering.location == SpecLocation::InCheckout && (!moves_back || stops) {
                problem("must carry the move-back step, and only that".into());
            }
            if rendering.location == SpecLocation::Elsewhere && (moves_back || !stops) {
                problem("must stop instead of moving the record back".into());
            }
        }
        if name == "ingest" && !content.lines().any(|l| l.contains("worktree.path") && l.contains("steps 4 to 8")) {
            problem("never runs steps 4 to 8 from the change's worktree.path".into());
        }
        if name == "discuss" {
            let named = content
                .lines()
                .find(|l| l.trim_start().starts_with("3. If the user mentioned a specific change name"))
                .unwrap_or_default();
            if !(named.contains("worktree.path") && named.contains("specs/<capability>")) {
                problem("step 3 lacks the worktree.path and deltaSpecs reads".into());
            }
        }
    }
    assert!(found.is_empty(), "discuss, propose and ingest drifted from their pinned wording:\n{}", found.join("\n"));
}

// --- hand-offs: a reference to another skill never reads as a verb
// (workspace-tools spec「中性渲染目標」) ---

fn harness(invocation: Invocation) -> CustomTool {
    CustomTool { invocation, ..tool_call_harness() }
}

/// Scenario「交棒寫成技能名稱」「真的動詞仍是動詞寫法」: in both neutral forms a skill
/// reference is the skill's name and a verb stays a verb; the five built-in tools keep
/// their own spelling.
#[test]
fn a_neutral_skill_reference_is_the_skill_name_and_a_verb_stays_a_verb() {
    let source = "Hand off to `/speclink:apply <change-name>`, `/speclink:archive <name>` or `/speclink:review <name>`; this step runs `speclink archive <name>`.";
    for invocation in [Invocation::Cli, Invocation::ToolCall] {
        let custom = harness(invocation);
        assert_eq!(
            skills::substitute(source, RenderTarget::Custom(&custom), SPEC_DIR, SpecLocation::InCheckout),
            "Hand off to `speclink-apply <change-name>`, `speclink-archive <name>` or `speclink-review <name>`; this step runs `speclink archive <name>`.",
            "{invocation:?}"
        );
    }
    let builtin = Tool::ALL.map(|tool| skills::substitute("/speclink:apply <name>", RenderTarget::Builtin(tool), SPEC_DIR, SpecLocation::InCheckout));
    assert_eq!(
        builtin,
        [
            "/speclink-apply <name>",
            "$speclink-apply <name>",
            "/speclink-apply <name>",
            "/skill:speclink-apply <name>",
            "/speclink-apply <name>",
        ]
    );
}

/// How often `text` writes `prefix` + `name` as a whole word (`speclink apply` is not
/// `speclink apply-with-worktree`).
fn count_named(text: &str, prefix: &str, name: &str) -> usize {
    Regex::new(&format!(r"{}{}(?:[^a-z-]|$)", regex::escape(prefix), regex::escape(name)))
        .unwrap()
        .find_iter(text)
        .count()
}

/// The skills that share their name with a real verb (`speclink --help`): only these
/// may appear as `speclink <name>` in a neutral rendering, and only where the source
/// wrote the verb.
const VERB_NAMED_SKILLS: [&str; 8] =
    ["analyze", "archive", "config", "discuss", "drift", "review", "trace", "verify"];

/// What a skill reference of the source becomes in the reference rendering below: a
/// spelling the renderer leaves alone, so the hand-offs can be told from the verbs.
const HAND_OFF_MARK: &str = "@hand-off:";

/// Every hand-off of every skill, measured against a second neutral rendering of the
/// same body whose skill references were marked beforehand: the neutral forms write the
/// verb exactly where the source wrote a verb, and the skill name everywhere else.
#[test]
fn neutral_renderings_never_turn_a_hand_off_into_a_verb() {
    let registry = skills::registry();
    let mut found: Vec<String> = Vec::new();
    let mut hand_offs = 0;
    for skill in registry.iter().filter(|s| !s.claude_only) {
        for invocation in [Invocation::Cli, Invocation::ToolCall] {
            let custom = harness(invocation);
            for location in SpecLocation::BOTH {
                let marked = skill.body.replace("/speclink:", HAND_OFF_MARK);
                let reference = skills::substitute(&marked, RenderTarget::Custom(&custom), SPEC_DIR, location);
                let neutral = skills::substitute(skill.body, RenderTarget::Custom(&custom), SPEC_DIR, location);
                for other in &registry {
                    let references = count_named(&reference, HAND_OFF_MARK, other.name);
                    hand_offs += references;
                    let (verbs, wanted_verbs) = (
                        count_named(&neutral, "speclink ", other.name),
                        count_named(&reference, "speclink ", other.name),
                    );
                    let (names, wanted_names) = (
                        count_named(&neutral, "speclink-", other.name),
                        count_named(&reference, "speclink-", other.name) + references,
                    );
                    if verbs > 0 && !VERB_NAMED_SKILLS.contains(&other.name) {
                        found.push(format!(
                            "{invocation:?} speclink-{}: `speclink {}` reads as a verb, and there is no such verb",
                            skill.name, other.name
                        ));
                    }
                    if (verbs, names) != (wanted_verbs, wanted_names) {
                        found.push(format!(
                            "{invocation:?} speclink-{} ({location:?}): `speclink {name}` appears {verbs} times (the source writes the verb {wanted_verbs} times) and `speclink-{name}` {names} times (expected {wanted_names})",
                            skill.name,
                            name = other.name
                        ));
                    }
                }
            }
        }
    }
    assert!(hand_offs > 0, "the sources carry hand-offs to check");
    assert!(
        found.is_empty(),
        "a hand-off to another skill must render as `speclink-<name>`, never as the verb `speclink <name>`:\n{}",
        found.join("\n")
    );
}

/// Scenario「呼叫說明分辨技能與動詞」: both neutral forms open with the sentence, and the
/// note's own example is not a skill name sent as a verb.
#[test]
fn the_invocation_note_tells_a_skill_from_a_verb() {
    let registry = skills::registry();
    let apply = registry.iter().find(|s| s.name == "apply").expect("registry carries apply");
    for invocation in [Invocation::Cli, Invocation::ToolCall] {
        let custom = harness(invocation);
        let rendered = skills::render_skill_file_for(RenderTarget::Custom(&custom), apply, SPEC_DIR, SpecLocation::InCheckout);
        let note = rendered
            .split("## Invocation")
            .nth(1)
            .and_then(|rest| rest.split("\n---\n").next())
            .unwrap_or_else(|| panic!("{invocation:?}: no invocation note"));
        assert!(
            note.contains("`speclink-<name>` is another skill, not a verb"),
            "{invocation:?}: the note must tell a skill from a verb:\n{note}"
        );
        for skill in &registry {
            assert!(
                !note.contains(&format!("[\"{}\"", skill.name)),
                "{invocation:?}: the note's example sends the skill name `{}` as a verb:\n{note}",
                skill.name
            );
        }
    }
}

// --- per-skill scenarios: what each side of a skill says ---

/// One skill in both forms, for one side.
fn rendered(skill: &str, location: SpecLocation) -> Vec<Rendering> {
    let found: Vec<Rendering> = renderings(location).into_iter().filter(|r| r.skill == skill).collect();
    assert!(!found.is_empty(), "no rendering of {skill}");
    found
}

/// The text from `start` up to `end`.
fn section<'a>(rendering: &'a Rendering, start: &str, end: &str) -> &'a str {
    let content = &rendering.content;
    let from = content.find(start).unwrap_or_else(|| panic!("{}: missing {start:?}", rendering.label()));
    let len = content[from..].find(end).unwrap_or_else(|| panic!("{}: missing {end:?}", rendering.label()));
    &content[from..from + len]
}

fn assert_has(rendering: &Rendering, text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "{}: missing {needle:?}", rendering.label());
    }
}

fn assert_lacks(rendering: &Rendering, text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(!text.contains(needle), "{}: must not carry {needle:?}", rendering.label());
    }
}

/// archive-skill「封存前的讀取與 delta 改寫經動詞」Scenario「delta 的檢查不用路徑」
/// 「改寫 delta 經整份寫回」: on both sides the task count, the delta check and the delta
/// fix name their verbs, and the rule "only the deltas change" stays.
#[test]
fn archive_skill_checks_and_fixes_deltas_through_verbs() {
    for location in SpecLocation::BOTH {
        for rendering in rendered("archive", location) {
            let tasks = section(&rendering, "**Check task completion status**", "**Plan order hint**");
            assert_has(&rendering, tasks, &["`completedTasks` and `totalTasks` from `speclink list --json`"]);
            let deltas = section(&rendering, "**Assess delta spec completeness**", "**Perform the archive**");
            assert_has(
                &rendering,
                deltas,
                &[
                    "`deltaCapabilities` in `speclink list --json`",
                    "`speclink artifact cat specs/<capability> --change \"<name>\"`",
                    "`speclink show <capability> --item-type spec`",
                    "fix each delta with a write-back (see **Write-back** above the steps)",
                    "do NOT edit the main specs",
                ],
            );
            assert_lacks(&rendering, deltas, &["openspec/", "in place"]);
        }
    }
}

/// Same requirement, Scenario「remote 專案封存後的告知」「remote 專案遇到封存名稱衝突」:
/// the summary names the archived name on both sides; the path, the rename-or-delete
/// options, the commit reminder and the manual reminder exist only where the spec folder is.
#[test]
fn archive_skill_keeps_folder_steps_for_a_checkout_with_the_spec_folder() {
    let folder_steps = [
        "**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/",
        "suggest renaming existing archive",
        "Rename the existing archive",
        "Delete the existing archive",
        "plain git commit",
        "the manual may be stale now",
    ];
    let server_steps = [
        "stop and tell the user that the existing archive has to be handled on the Server that holds the specs",
        "An archived change with this name already exists on the Server",
        "the archive leaves nothing here to commit",
    ];
    for rendering in rendered("archive", SpecLocation::InCheckout) {
        assert_has(&rendering, &rendering.content, &folder_steps);
        assert_lacks(&rendering, &rendering.content, &server_steps);
        assert_eq!(rendering.content.matches("**Archived as:** YYYY-MM-DD-<name>").count(), 3, "{}", rendering.label());
    }
    for rendering in rendered("archive", SpecLocation::Elsewhere) {
        assert_has(&rendering, &rendering.content, &server_steps);
        assert_lacks(&rendering, &rendering.content, &folder_steps);
        assert_lacks(&rendering, &rendering.content, &["**Archived to:**"]);
        assert_eq!(rendering.content.matches("**Archived as:** YYYY-MM-DD-<name>").count(), 3, "{}", rendering.label());
    }
}

/// commit-skill「commit 的檔案清單取自 evidence 動詞」Scenario「檔案清單以動詞取得」
/// 「確認閘門與實際一致」: on both sides the touched files and the task progress come
/// from verbs, and what the gate shows is what gets staged.
#[test]
fn commit_skill_collects_files_and_progress_through_verbs() {
    for location in SpecLocation::BOTH {
        for rendering in rendered("commit", location) {
            let evidence = section(&rendering, "**Read the evidence record**", "**Collect artifact files**");
            assert_has(
                &rendering,
                evidence,
                &[
                    "`speclink evidence show \"<change-name>\" --json`",
                    "`entries` holds one entry per completed task",
                    "`files` is every touched file",
                    "the pre-move location `.speclink/touched/<change-name>.json`",
                ],
            );
            assert_lacks(&rendering, evidence, &["openspec/", ".evidence.json"]);
            assert_has(
                &rendering,
                &rendering.content,
                &[
                    "Task progress is the change's `completedTasks` and `totalTasks` in `speclink list --json`",
                    "`deltaCapabilities` in `speclink list --json`",
                    "fix each delta with a write-back (see **Write-back** above the steps)",
                    "Proceed with the displayed files and the displayed commit message",
                    "Stage each confirmed file individually",
                ],
            );
            assert_lacks(&rendering, &rendering.content, &["rewrite the delta files in place"]);
        }
    }
}

/// Same requirement, Scenario「remote 專案的 commit 只含程式碼」「本機專案的 commit 含
/// 規格檔」: the spec-folder file list, the archive's file moves and the manual reminder
/// exist only where the spec folder is; elsewhere the plan says the spec changes are
/// already on the Server.
#[test]
fn commit_skill_stages_spec_files_only_where_the_spec_folder_is() {
    let folder_steps = [
        "filter the output to files under `openspec/changes/<name>/`",
        "belongs to this change's commit",
        "Add these archive-related file changes to the commit set",
        "### Main Spec Updates (from archive's delta application)",
        "the manual may be stale now",
        "Only artifact files will be committed",
    ];
    let server_steps = [
        "A commit made here holds code files only",
        "(none in this commit — the spec changes are already on the Server)",
        "the files shown are exactly the files step 8 stages",
        "The archive ran on the Server and moved no file in this checkout",
        "No spec file was staged",
    ];
    for rendering in rendered("commit", SpecLocation::InCheckout) {
        assert_has(&rendering, &rendering.content, &folder_steps);
        assert_lacks(&rendering, &rendering.content, &server_steps);
    }
    for rendering in rendered("commit", SpecLocation::Elsewhere) {
        assert_has(&rendering, &rendering.content, &server_steps);
        assert_lacks(&rendering, &rendering.content, &folder_steps);
    }
}

/// worktree-apply-skill「worktree 技能的規格檔步驟依 checkout 產生」: committing the
/// change's artifacts before the worktree exists, and staging them at the wrap-up, are
/// steps for a checkout with the spec folder; elsewhere one sentence says the spec
/// documents are on the Server. Both sides take the touched files from the evidence verb.
#[test]
fn worktree_skill_commits_spec_files_only_where_the_spec_folder_is() {
    let folder_steps = [
        "git status --porcelain -- \"openspec/changes/<change-name>/\"",
        "git add \"openspec/changes/<change-name>\"",
        "P3 commits the change's **artifacts**",
        "its artifacts under `openspec/changes/<change-name>/` plus the source files",
        "holds only this change's artifacts and evidence-recorded source files",
    ];
    let server_steps = [
        "The spec documents are on the Server, not in this checkout: there is nothing to commit before the worktree is created",
        "the spec documents are on the Server and are not part of this commit",
        "holds only this change's evidence-recorded source files — no spec document",
    ];
    let both = [
        "speclink evidence show \"<change-name>\" --json\ngit status --porcelain -- <each path in that output's `files`>",
        "`speclink evidence show \"<change-name>\" --json` lists in `files`",
    ];
    let cases: [(SpecLocation, &[&str], &[&str]); 2] =
        [(SpecLocation::InCheckout, &folder_steps, &server_steps), (SpecLocation::Elsewhere, &server_steps, &folder_steps)];
    for (location, has, lacks) in cases {
        for rendering in rendered("apply-with-worktree", location) {
            assert_has(&rendering, &rendering.content, has);
            assert_has(&rendering, &rendering.content, &both);
            assert_lacks(&rendering, &rendering.content, lacks);
            assert_lacks(&rendering, &rendering.content, &["cat \""]);
        }
    }
}

/// baseline-skill「規格不在 checkout 時先拒絕」＋ manual-skill「remote 模式的限制敘述」:
/// the two exempt skills write into the spec folder, so rendered for a checkout without
/// it they refuse before the first writing step — in the skill file itself, with no
/// run-time read of `.speclink.yaml`. Rendered for a local project they are unchanged.
#[test]
fn the_exempt_skills_refuse_first_when_the_spec_folder_is_elsewhere() {
    let cases = [
        (
            "baseline",
            "## Step 0: Refuse — the spec folder is not in this checkout",
            "## Step 1: Check the current state",
            ["they would stay in this checkout and never reach the Server", "Write no file"],
        ),
        (
            "manual",
            "This skill file was generated for a project whose spec folder is not in this checkout",
            "Read `.speclink.yaml` at the workspace root",
            ["Generation is not supported here, whatever the check below finds", "Zero files are written. Tour mode is unaffected."],
        ),
    ];
    for (skill, refusal, first_step, says) in cases {
        for rendering in rendered(skill, SpecLocation::Elsewhere) {
            let at = |needle: &str| {
                rendering.content.find(needle).unwrap_or_else(|| panic!("{}: missing {needle:?}", rendering.label()))
            };
            assert!(at(refusal) < at(first_step), "{}: the refusal comes first", rendering.label());
            assert_has(&rendering, &rendering.content, &says);
        }
        for rendering in rendered(skill, SpecLocation::InCheckout) {
            assert_lacks(&rendering, &rendering.content, &[refusal]);
            assert_has(&rendering, &rendering.content, &[first_step]);
        }
    }
}

/// verb-contract「store 文件讀取動詞」Scenario「閒置的判斷不用 git 歷史」: the entry is read
/// before the step that writes to the change — marking it in-progress, or creating its
/// worktree, moves `modifiedAt` to now, and a dormancy rule judged afterwards never fires.
#[test]
fn dormancy_is_judged_on_the_entry_read_before_the_change_is_written_to() {
    for location in SpecLocation::BOTH {
        for rendering in rendered("apply", location) {
            let at = |needle: &str| {
                rendering.content.find(needle).unwrap_or_else(|| panic!("{}: missing {needle:?}", rendering.label()))
            };
            let kept = at("keep its `created` and `modifiedAt` for step 3d");
            assert!(kept < at("speclink in-progress add \"<name>\""), "{}: the entry is kept before step 2 writes", rendering.label());
            assert!(at(DORMANCY_RULE) < at("Judge on the entry step 2 kept"), "{}", rendering.label());
        }
        for rendering in rendered("apply-with-worktree", location) {
            let at = |needle: &str| {
                rendering.content.find(needle).unwrap_or_else(|| panic!("{}: missing {needle:?}", rendering.label()))
            };
            let kept = at("Keep the selected change's `created` and `modifiedAt` from `speclink list --json` now");
            assert!(kept < at("git worktree add"), "{}: the entry is kept before the worktree exists", rendering.label());
            assert_has(&rendering, &rendering.content, &["judges dormancy on the values kept here"]);
        }
    }
}

/// verb-contract「store 文件讀取動詞」: a stamp is written into the change's metadata, so
/// "commit the stamp" is a git step on a spec file — it exists only where the spec folder
/// is. Elsewhere the skills say the stamp is on the Server.
#[test]
fn committing_a_stamp_is_a_step_for_a_checkout_with_the_spec_folder() {
    let cases: [(&str, &[&str], &[&str]); 3] = [
        ("review", &["commit the stamp's meta changes first"], &["the stamp is on the Server"]),
        ("verify", &["commit the stamp's meta changes first"], &["the stamp is on the Server"]),
        (
            "apply-with-worktree",
            &["記得補一次提交", "commit the stamps, then", "### P3. Get the change's artifacts into HEAD", "stamps land in that copy"],
            &["章在 Server 上，不用補提交", "the stamps are on the Server", "### P3. Nothing to commit for the artifacts", "stamps go to the Server"],
        ),
    ];
    for (skill, folder_steps, server_steps) in cases {
        for rendering in rendered(skill, SpecLocation::InCheckout) {
            assert_has(&rendering, &rendering.content, folder_steps);
            assert_lacks(&rendering, &rendering.content, server_steps);
        }
        for rendering in rendered(skill, SpecLocation::Elsewhere) {
            assert_has(&rendering, &rendering.content, server_steps);
            assert_lacks(&rendering, &rendering.content, folder_steps);
        }
    }
}

/// commit-skill「提交後回報落地」: the report comes after the commit, is left to the
/// merge on a worktree's change branch, has nothing to report to once the change is
/// archived, and a refusal never touches the commit. The note about the modified data
/// file is for a checkout with the spec folder only.
#[test]
fn commit_skill_reports_the_landing_after_the_commit() {
    for location in SpecLocation::BOTH {
        for rendering in rendered("commit", location) {
            let step = section(&rendering, "**Report the landing**", "**Show result**");
            assert_has(
                &rendering,
                step,
                &[
                    "Do this only after `git commit` succeeded",
                    "Take the commit from `git rev-parse HEAD` and pass it",
                    "speclink change landed \"<change-name>\" --commit \"<commit>\"",
                    "is `speclink/<change-name>`, the change's branch of a linked worktree",
                    "reports it after the merge",
                    "The archive sub-flow ran (step 7a)",
                    "On any other branch, report",
                    "tell the user its message and continue with step 10",
                    "do not undo it, and do not make it again",
                ],
            );
            let content = &rendering.content;
            let (commit, report) = (content.find("git commit -m").unwrap(), content.find("speclink change landed").unwrap());
            assert!(commit < report, "{}: the report follows the commit", rendering.label());
            assert_has(&rendering, content, &["**Landing:** reported ✓"]);
        }
    }
    let data_file_note = ["that file shows as modified after this step", "it goes into this change's next commit"];
    for rendering in rendered("commit", SpecLocation::InCheckout) {
        assert_has(&rendering, &rendering.content, &data_file_note);
    }
    for rendering in rendered("commit", SpecLocation::Elsewhere) {
        assert_lacks(&rendering, &rendering.content, &data_file_note);
    }
}

/// worktree-merge-skill「合併後回報落地」: the report follows a successful merge and
/// names the change's branch; the conflict stop has no report; a refusal never reverts
/// the merge and the wrap-up goes on.
#[test]
fn worktree_merge_skill_reports_the_landing_after_a_successful_merge() {
    for location in SpecLocation::BOTH {
        for rendering in rendered("worktree-merge", location) {
            let step = section(&rendering, "**Report the landing, carry the quality baseline over, then clean up**", "**Confirm and hand off**");
            assert_has(
                &rendering,
                step,
                &[
                    "After a successful merge",
                    "Take the commit from `git -C <main-checkout> rev-parse HEAD` and pass it",
                    "speclink change landed \"<change-name>\" --commit \"<commit>\" --branch \"speclink/<change-name>\"",
                    "tell the user its message and go on with the rest of this step",
                    "do not revert it",
                ],
            );
            let (report, prepare) = (step.find("speclink change landed").unwrap(), step.find("speclink review prepare").unwrap());
            assert!(report < prepare, "{}: the report comes first in the step", rendering.label());
            let conflict = section(&rendering, "**Conflict — stop immediately**", "**Report the landing, carry");
            assert_lacks(&rendering, conflict, &["change landed"]);
            assert_has(&rendering, &rendering.content, &["**Landing reported:** ✓"]);
        }
    }
    let data_file_note = ["the main checkout has that one modified file from here on", "Leave it uncommitted and name it in the hand-off"];
    for rendering in rendered("worktree-merge", SpecLocation::InCheckout) {
        assert_has(&rendering, &rendering.content, &data_file_note);
    }
    for rendering in rendered("worktree-merge", SpecLocation::Elsewhere) {
        assert_lacks(&rendering, &rendering.content, &data_file_note);
    }
}

/// The quality skill's resume rule for an artifact fix a validation patch cannot show:
/// with the spec folder in the checkout, git lists the changed files; elsewhere the
/// skill asks the user and reads the passages through the verb — both sides have an exit.
#[test]
fn quality_resume_can_judge_an_artifact_fix_on_both_sides() {
    let asks = "ask the user (see **Asking the user**) whether the earlier session fixed findings";
    for rendering in rendered("quality", SpecLocation::InCheckout) {
        assert_has(&rendering, &rendering.content, &[asks, "git status --porcelain -- \"openspec/changes/<change-name>/\""]);
    }
    for rendering in rendered("quality", SpecLocation::Elsewhere) {
        assert_has(
            &rendering,
            &rendering.content,
            &[asks, "with a write-back", "`speclink artifact cat <artifact-id> --change \"<change-name>\"`"],
        );
        assert_lacks(&rendering, &rendering.content, &["No write-back in this session", "cannot be seen"]);
    }
}

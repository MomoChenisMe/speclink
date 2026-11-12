//! Mechanical guards for the skill sources under `assets/skills/` (skill-authoring
//! spec「寫法規範的機械守門」). Six checks run on every `.md` source: line count,
//! the emphasis-word cap, no history in prose, a fallback for tool-dependent
//! capabilities, no hard-coded skill references or `--agent` values (those must
//! go through the render markers so every tool gets its own spelling), and spec-folder
//! paths only inside the block that is rendered when the spec folder is in the checkout.

use regex::Regex;
use speclink_core::skills::{self, SpecLocation};

/// The spec every failure message points the reader to.
const SPEC_REF: &str = "skill-authoring 規格『技能原稿的跨模型寫法規範』";

/// Emphasis-word cap per source. A new source must be registered here; raising a
/// cap is a deliberate edit to this table.
const EMPHASIS_CAPS: &[(&str, usize)] = &[
    ("analyze.md", 1),
    ("apply-worktree-post.md", 4),
    ("apply-worktree-pre.md", 21),
    ("apply.md", 8),
    ("archive.md", 4),
    ("artifact-write-back.md", 0),
    ("audit.md", 1),
    ("baseline.md", 8),
    ("clarify.md", 4),
    ("commit.md", 6),
    ("config.md", 4),
    ("conversation-language.md", 0),
    ("discuss.md", 5),
    ("drift.md", 2),
    ("improve.md", 2),
    ("ingest.md", 10),
    ("manual.md", 5),
    ("propose.md", 6),
    ("quality.md", 3),
    ("review-intro.md", 1),
    ("review.md", 19),
    ("station-shared.md", 0),
    ("sync.md", 5),
    ("tdd.md", 4),
    ("trace.md", 3),
    ("verify-intro.md", 1),
    ("verify.md", 20),
    ("worktree-merge.md", 18),
];

/// The internal skills: taken through `instructions --skill`, never rendered into a
/// skill file, so the sixth check has nothing to protect in them.
const INTERNAL_SOURCES: [&str; 3] = ["sync.md", "clarify.md", "tdd.md"];

/// The sources the sixth check skips, each with its reason: the skills that write into
/// the spec folder ([`crate::SPEC_FOLDER_WRITERS`]) and the internal skills. The table
/// is closed: a source that is not in it is checked.
fn spec_path_exempt() -> Vec<(String, &'static str)> {
    let writers = crate::SPEC_FOLDER_WRITERS.iter().map(|(skill, reason)| (format!("{skill}.md"), *reason));
    let internal = INTERNAL_SOURCES.map(|source| {
        (source.to_string(), "internal skill: taken through `instructions --skill`, never rendered into a skill file")
    });
    writers.chain(internal).collect()
}

/// The lines that name a spec-folder file and stay when the spec folder is not in the
/// checkout — everything outside a `{{#SPEC_IN_CHECKOUT}}` block — as (line, spelling).
/// The renderer decides what stays; a malformed block is its to report ([`block_error`]).
fn spec_paths_outside_blocks(name: &str, text: &str) -> Vec<(String, &'static str)> {
    let Ok(kept) = skills::resolve_blocks(name, text, SpecLocation::Elsewhere) else {
        return Vec::new();
    };
    let words: Vec<&str> = std::iter::once("{{SPEC_DIR}}").chain(crate::SPEC_FOLDER_FILE_NAMES).collect();
    kept.lines()
        .flat_map(|line| {
            words.iter().filter(move |word| line.contains(**word)).map(move |word| (line.trim().to_string(), *word))
        })
        .collect()
}

/// The renderer's own verdict on the source's conditional blocks.
fn block_error(name: &str, text: &str) -> Option<String> {
    skills::resolve_blocks(name, text, SpecLocation::InCheckout).err().map(|err| err.to_string())
}

/// Lines that name the block condition and still reach a rendering — a misspelt marker
/// (`{{ #SPEC_IN_CHECKOUT}}`) is plain text to the renderer, and both sides of the block
/// would then be rendered.
fn misspelt_markers(name: &str, text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for location in SpecLocation::BOTH {
        let kept = skills::resolve_blocks(name, text, location).unwrap_or_default();
        for line in kept.lines().filter(|line| line.to_ascii_uppercase().contains("SPEC_IN_CHECKOUT")) {
            if !lines.iter().any(|seen| seen == line.trim()) {
                lines.push(line.trim().to_string());
            }
        }
    }
    lines
}

/// A spec-folder path written with the folder's name instead of `{{SPEC_DIR}}` — it
/// would pass the placeholder check and name a folder the project may not use.
fn hardcoded_spec_paths(text: &str) -> Vec<String> {
    Regex::new(r"(?:^|[^.\w/-])((?:speclink|openspec)/(?:changes|specs|discussions|manual|LANGUAGE\.md|config\.yaml))")
        .unwrap()
        .captures_iter(text)
        .map(|caps| caps[1].to_string())
        .collect()
}

fn line_count(text: &str) -> usize {
    text.lines().count()
}

fn emphasis_count(text: &str) -> usize {
    Regex::new(r"\b(MUST|NEVER|SHALL|ALWAYS|IMPORTANT|CRITICAL|STOP|NOT)\b")
        .unwrap()
        .find_iter(text)
        .count()
}

/// The text with fenced code blocks and inline code removed: a timestamp that
/// demonstrates a format lives there and is not history.
fn prose(text: &str) -> String {
    let inline_code = Regex::new(r"``[^`\n]*?``|`[^`\n]*`").unwrap();
    let mut kept = String::new();
    for (_, line) in crate::lines_outside_fences(text) {
        kept.push_str(&inline_code.replace_all(line.trim_end_matches(['\n', '\r']), ""));
        kept.push('\n');
    }
    kept
}

fn history_hits(text: &str) -> Vec<String> {
    // 以「不接數字」為界而不用 \b：緊貼中文（於2026-08-10加入）或帶時間的日期也要抓到。
    let date = Regex::new(r"(?:^|[^0-9])(\d{4}-\d{2}-\d{2})(?:[^0-9]|$)").unwrap();
    let decision = Regex::new(r"design D\d+|design 決策").unwrap();
    let prose = prose(text);
    let mut hits: Vec<String> = date
        .captures_iter(&prose)
        .map(|caps| caps[1].to_string())
        .collect();
    hits.extend(decision.find_iter(text).map(|hit| hit.as_str().to_string()));
    hits
}

fn missing_fallbacks(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    let mut missing = Vec::new();
    if text.contains("AskUserQuestion") && !lower.contains("plain text") {
        missing.push("用到 AskUserQuestion，卻沒寫工具不可用時改以 plain text 詢問並等待回覆");
    }
    let spawns = ["sub-agent", "subagent", "agent tool"]
        .iter()
        .any(|word| lower.contains(word));
    let fallback = ["cannot spawn", "can't spawn"]
        .iter()
        .any(|word| lower.contains(word));
    if spawns && !fallback {
        missing.push("用到子代理，卻沒寫不能開子代理（cannot spawn）時由主執行緒自己執行");
    }
    missing
}

fn hardcoded_references(text: &str) -> Vec<String> {
    let slash = crate::slash_skill_reference();
    // `$speclink-` 在原稿裡沒有正當用途，後面接什麼都算（含 `$speclink-<名稱>`）。
    let dollar = Regex::new(r"\$speclink-[^\s`]*").unwrap();
    let agent_value = Regex::new(r"--agent (?:claude|codex|copilot|pi|cursor)\b").unwrap();
    let mut hits: Vec<String> = Vec::new();
    for line in text.lines() {
        hits.extend(slash.captures_iter(line).map(|caps| caps[1].to_string()));
        hits.extend(dollar.find_iter(line).map(|hit| hit.as_str().to_string()));
        hits.extend(
            agent_value
                .find_iter(line)
                .map(|hit| hit.as_str().to_string()),
        );
    }
    hits
}

/// Every violation of one source, each message naming the file, the check and [`SPEC_REF`].
/// `exempt` is the sixth check's table ([`spec_path_exempt`] in the real scan).
fn violations(name: &str, text: &str, caps: &[(&str, usize)], exempt: &[(String, &str)]) -> Vec<String> {
    let mut found = Vec::new();
    let lines = line_count(text);
    if lines > 500 {
        found.push(format!("{name}：行數 {lines} 超過 500（{SPEC_REF}）"));
    }
    let emphasis = emphasis_count(text);
    match caps.iter().find(|(file, _)| *file == name) {
        Some((_, cap)) if emphasis > *cap => {
            found.push(format!("{name}：強調字上限 {cap}，實際 {emphasis}（{SPEC_REF}）"))
        }
        Some(_) => {}
        None => found.push(format!(
            "{name}：強調字上限沒有登記，請在 EMPHASIS_CAPS 登記這份原稿的上限（目前 {emphasis} 個；{SPEC_REF}）"
        )),
    }
    for hit in history_hits(text) {
        found.push(format!("{name}：不寫歷史，命中 `{hit}`（{SPEC_REF}）"));
    }
    for missing in missing_fallbacks(text) {
        found.push(format!("{name}：工具退路，{missing}（{SPEC_REF}）"));
    }
    for hit in hardcoded_references(text) {
        found.push(format!(
            "{name}：渲染記號，命中寫死的 `{hit}`；技能引用寫成 /speclink:<名稱>、--agent 的值寫成 {{{{TOOL}}}}（{SPEC_REF}）"
        ));
    }
    if let Some(error) = block_error(name, text) {
        found.push(format!("{name}：規格路徑只在條件區塊裡，條件區塊寫壞了：{error}（{SPEC_REF}）"));
    }
    for line in misspelt_markers(name, text) {
        found.push(format!(
            "{name}：規格路徑只在條件區塊裡，這一行提到 SPEC_IN_CHECKOUT 卻不是區塊記號：`{line}`；記號要獨佔一行，寫成 {{{{#SPEC_IN_CHECKOUT}}}}、{{{{^SPEC_IN_CHECKOUT}}}} 或 {{{{/SPEC_IN_CHECKOUT}}}}（{SPEC_REF}）"
        ));
    }
    if !exempt.iter().any(|(file, _)| file == name) {
        for hit in hardcoded_spec_paths(text) {
            found.push(format!(
                "{name}：規格路徑只在條件區塊裡，命中寫死的 `{hit}`；規格資料夾寫成 {{{{SPEC_DIR}}}} 並放進 {{{{#SPEC_IN_CHECKOUT}}}} 區塊（{SPEC_REF}）"
            ));
        }
        for (line, word) in spec_paths_outside_blocks(name, text) {
            found.push(format!(
                "{name}：規格路徑只在條件區塊裡，`{word}` 在 {{{{#SPEC_IN_CHECKOUT}}}} 區塊之外：`{line}`；讀寫改用動詞，git 步驟與顯示路徑的句子放進區塊（{SPEC_REF}）"
            ));
        }
    }
    found
}

/// Spec Example「強調字的計次」.
#[test]
fn emphasis_count_matches_the_spec_examples() {
    assert_eq!(emphasis_count("Do NOT stop here"), 1);
    assert_eq!(emphasis_count("This is not optional"), 0);
    assert_eq!(emphasis_count("See the NOTE below"), 0);
    assert_eq!(emphasis_count("STOP and ask. NEVER guess."), 2);
}

#[test]
fn line_count_is_the_same_for_lf_and_crlf() {
    let lf = "one\ntwo\nthree\n";
    let crlf = "one\r\ntwo\r\nthree\r\n";
    assert_eq!(line_count(lf), 3);
    assert_eq!(line_count(crlf), 3);
}

#[test]
fn a_source_over_500_lines_fails_with_its_line_count() {
    let text = "line\n".repeat(501);
    let found = violations("long.md", &text, &[("long.md", 0)], &[]);
    assert!(
        found.iter().any(|v| v.contains("long.md")
            && v.contains("行數")
            && v.contains("501")
            && v.contains(SPEC_REF)),
        "{found:?}"
    );
}

#[test]
fn emphasis_over_the_cap_fails_with_cap_and_count() {
    let found = violations(
        "review.md",
        "MUST do this. NEVER that.\n",
        &[("review.md", 1)],
        &[],
    );
    assert!(
        found.iter().any(|v| v.contains("review.md")
            && v.contains("強調字上限")
            && v.contains("上限 1")
            && v.contains("實際 2")
            && v.contains(SPEC_REF)),
        "{found:?}"
    );
    assert!(violations("review.md", "MUST do this.\n", &[("review.md", 2)], &[]).is_empty());
}

#[test]
fn dates_in_code_do_not_count_as_history() {
    let fenced = "Example:\n\n```\ngenerated: 2026-09-05T23:31:00+08:00\n```\n\nThe stamp is `2026-09-05` here.\n";
    assert!(
        history_hits(fenced).is_empty(),
        "{:?}",
        history_hits(fenced)
    );
    let indented = "1. Step\n\n   ```yaml\n   updated: 2026-09-05\n   ```\n";
    assert!(
        history_hits(indented).is_empty(),
        "{:?}",
        history_hits(indented)
    );
}

#[test]
fn dates_and_design_decision_ids_in_prose_are_history() {
    assert_eq!(
        history_hits("此規則於 2026-08-10 加入"),
        vec!["2026-08-10".to_string()]
    );
    assert_eq!(
        history_hits("see design D3 for why"),
        vec!["design D3".to_string()]
    );
    assert_eq!(
        history_hits("依 design 決策而定"),
        vec!["design 決策".to_string()]
    );
    // 緊貼中文或帶時間的日期也算。
    assert_eq!(
        history_hits("於2026-08-10加入"),
        vec!["2026-08-10".to_string()]
    );
    assert_eq!(
        history_hits("updated at 2026-08-10T10:00"),
        vec!["2026-08-10".to_string()]
    );
    let found = violations("x.md", "此規則於 2026-08-10 加入\n", &[("x.md", 0)], &[]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("x.md") && v.contains("不寫歷史") && v.contains("2026-08-10")),
        "{found:?}"
    );
}

#[test]
fn ask_user_question_needs_a_plain_text_fallback() {
    assert!(!missing_fallbacks("Use the **AskUserQuestion tool**.").is_empty());
    assert!(missing_fallbacks(
        "Use the AskUserQuestion tool; if it is unavailable, ask as Plain Text."
    )
    .is_empty());
    let found = violations("x.md", "Use the AskUserQuestion tool.\n", &[("x.md", 0)], &[]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("x.md") && v.contains("工具退路")),
        "{found:?}"
    );
}

#[test]
fn sub_agents_need_a_cannot_spawn_fallback() {
    assert!(!missing_fallbacks("Launch two sub-agents in parallel.").is_empty());
    assert!(missing_fallbacks(
        "Launch two sub-agents; if you cannot spawn them, run both passes yourself."
    )
    .is_empty());
    assert!(missing_fallbacks(
        "Use the Agent tool; if you can't spawn one, run the pass yourself."
    )
    .is_empty());
    // 「inline」這個字不算退路：無關的句子也會含它。
    assert!(!missing_fallbacks("Use the Agent tool, or run the pass inline.").is_empty());
    assert!(
        !missing_fallbacks("Launch a sub-agent. Inline back until a real need shows.").is_empty()
    );
}

#[test]
fn hardcoded_references_are_found_but_skill_paths_are_not() {
    assert_eq!(
        hardcoded_references("run `/speclink-apply <name>`"),
        vec!["/speclink-apply".to_string()]
    );
    assert_eq!(
        hardcoded_references("then $speclink-apply"),
        vec!["$speclink-apply".to_string()]
    );
    assert_eq!(
        hardcoded_references("then $speclink-<name>"),
        vec!["$speclink-<name>".to_string()]
    );
    assert_eq!(
        hardcoded_references("speclink review stamp x --agent codex"),
        vec!["--agent codex".to_string()]
    );
    // Scenario「寫死的 pi 或 cursor agent 值」。
    assert_eq!(
        hardcoded_references("speclink review stamp x --agent cursor"),
        vec!["--agent cursor".to_string()]
    );
    assert_eq!(
        hardcoded_references("speclink new change a --agent pi"),
        vec!["--agent pi".to_string()]
    );
    assert!(hardcoded_references("see .claude/skills/speclink-propose/SKILL.md").is_empty());
    assert!(hardcoded_references("run `/speclink:apply <name>` with --agent {{TOOL}}").is_empty());
    let found = violations("x.md", "run `/speclink-apply <name>`\n", &[("x.md", 0)], &[]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("x.md") && v.contains("渲染記號") && v.contains("/speclink-apply")),
        "{found:?}"
    );
}

#[test]
fn an_unregistered_source_is_asked_to_register() {
    let found = violations("example.md", "plain text\n", &[("other.md", 0)], &[]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("example.md") && v.contains("EMPHASIS_CAPS")),
        "{found:?}"
    );
}

/// Scenario「區塊外的規格路徑」「區塊內的規格路徑通過」: each of the three spellings fails
/// outside the block — also inside the block for the other case — and passes inside it.
#[test]
fn spec_paths_are_allowed_only_in_the_in_checkout_block() {
    let inside = "{{#SPEC_IN_CHECKOUT}}\ngit add \"{{SPEC_DIR}}changes/<name>\"\nIt carries `.evidence.json` and `.meta.yaml`.\n{{/SPEC_IN_CHECKOUT}}\n";
    assert!(violations("x.md", inside, &[("x.md", 0)], &[]).is_empty());

    let outside = "Read `{{SPEC_DIR}}changes/<name>/tasks.md`.\n";
    let found = violations("x.md", outside, &[("x.md", 0)], &[]);
    assert!(
        found.iter().any(|v| v.contains("x.md")
            && v.contains("規格路徑只在條件區塊裡")
            && v.contains("Read `{{SPEC_DIR}}changes/<name>/tasks.md`.")
            && v.contains(SPEC_REF)),
        "{found:?}"
    );

    let other_case = "{{^SPEC_IN_CHECKOUT}}\nSee `.evidence.json`.\n{{/SPEC_IN_CHECKOUT}}\nAnd `.meta.yaml`.\n";
    assert_eq!(
        spec_paths_outside_blocks("x.md", other_case),
        vec![("See `.evidence.json`.".to_string(), ".evidence.json"), ("And `.meta.yaml`.".to_string(), ".meta.yaml")]
    );
}

/// Scenario「豁免的原稿不受第六項檢查」: a listed source passes; the table is closed, so
/// the same text under another name fails.
#[test]
fn an_exempt_source_skips_the_spec_path_check() {
    let text = "Write `{{SPEC_DIR}}specs/<capability>/spec.md`.\n";
    let exempt = [("baseline.md".to_string(), "writes the canonical specs")];
    assert!(violations("baseline.md", text, &[("baseline.md", 0)], &exempt).is_empty());
    assert!(!violations("other.md", text, &[("other.md", 0)], &exempt).is_empty());
}

/// The exemption table names real sources and gives each a reason.
#[test]
fn the_exemption_table_is_closed_and_reasoned() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills");
    let exempt = spec_path_exempt();
    assert_eq!(exempt.len(), crate::SPEC_FOLDER_WRITERS.len() + INTERNAL_SOURCES.len());
    for (file, reason) in exempt {
        assert!(dir.join(&file).is_file(), "{file} is exempt but is not a source");
        assert!(reason.len() > 20, "{file} needs a reason");
    }
}

/// A malformed conditional block fails the scan with the renderer's file and line.
#[test]
fn a_malformed_block_fails_with_its_file_and_line() {
    let found = violations("x.md", "a\n{{#SPEC_IN_CHECKOUT}}\nb\n", &[("x.md", 0)], &[]);
    assert!(found.iter().any(|v| v.contains("條件區塊寫壞了") && v.contains("x.md:2:")), "{found:?}");
}

/// A misspelt marker fails the scan. The renderer takes a stray space before the sigil
/// or a single brace for plain text — both sides of the block would be rendered — so
/// the line is reported as it stands; a wrong name after the sigil is a render error.
#[test]
fn a_misspelt_block_marker_fails() {
    for marker in ["{{ #SPEC_IN_CHECKOUT}}", "{#SPEC_IN_CHECKOUT}"] {
        let text = format!("a\n{marker}\nb\n");
        let found = violations("x.md", &text, &[("x.md", 0)], &[]);
        assert!(found.iter().any(|v| v.contains("不是區塊記號") && v.contains(marker)), "{marker}: {found:?}");
    }
    for marker in ["{{#SPEC_IN_CHECKOUT }}", "{{#spec_in_checkout}}"] {
        let text = format!("a\n{marker}\nb\n");
        let found = violations("x.md", &text, &[("x.md", 0)], &[]);
        assert!(found.iter().any(|v| v.contains("條件區塊寫壞了") && v.contains("x.md:2:")), "{marker}: {found:?}");
    }
    assert!(misspelt_markers("x.md", "  {{#SPEC_IN_CHECKOUT}}\nx\n  {{/SPEC_IN_CHECKOUT}}\n").is_empty());
}

/// The spec folder's name written out instead of `{{SPEC_DIR}}` fails the scan; the work
/// directory `.speclink/`, a `speclink/<change>` branch and a skill directory do not.
#[test]
fn a_hardcoded_spec_folder_path_fails() {
    assert_eq!(hardcoded_spec_paths("Read `speclink/changes/<name>/tasks.md`."), vec!["speclink/changes".to_string()]);
    assert_eq!(hardcoded_spec_paths("under openspec/specs/"), vec!["openspec/specs".to_string()]);
    for legal in [
        "the snapshots in `.speclink/review-scopes/<change>/`",
        "branch `speclink/<change-name>`",
        "see .claude/skills/speclink-propose/SKILL.md",
        "`docs/specs/specs/auth/spec.md`",
    ] {
        assert!(hardcoded_spec_paths(legal).is_empty(), "{legal}");
    }
    let found = violations("x.md", "Read `speclink/changes/<name>/tasks.md`.\n", &[("x.md", 0)], &[]);
    assert!(found.iter().any(|v| v.contains("寫死的 `speclink/changes`")), "{found:?}");
}

/// The real scan: every source under assets/skills/, in file-name order.
#[test]
fn every_skill_source_follows_the_authoring_rules() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/skills");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".md"))
        .collect();
    names.sort();
    let exempt = spec_path_exempt();
    let found: Vec<String> = names
        .iter()
        .flat_map(|name| {
            violations(
                name,
                &std::fs::read_to_string(dir.join(name)).unwrap(),
                EMPHASIS_CAPS,
                &exempt,
            )
        })
        .collect();
    assert!(found.is_empty(), "\n{}", found.join("\n"));
}

//! Mechanical guards for the skill sources under `assets/skills/` (skill-authoring
//! spec「寫法規範的機械守門」). Five checks run on every `.md` source: line count,
//! the emphasis-word cap, no history in prose, a fallback for tool-dependent
//! capabilities, and no hard-coded skill references or `--agent` values (those must
//! go through the render markers so every tool gets its own spelling).

use regex::Regex;

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
    ("audit.md", 1),
    ("baseline.md", 8),
    ("clarify.md", 4),
    ("commit.md", 6),
    ("config.md", 4),
    ("discuss.md", 5),
    ("drift.md", 2),
    ("improve.md", 2),
    ("ingest.md", 10),
    ("manual.md", 5),
    ("propose.md", 6),
    ("quality.md", 3),
    ("review.md", 16),
    ("sync.md", 5),
    ("tdd.md", 4),
    ("trace.md", 3),
    ("verify.md", 21),
    ("worktree-merge.md", 18),
];

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
    let agent_value = Regex::new(r"--agent (?:claude|codex|copilot)\b").unwrap();
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
fn violations(name: &str, text: &str, caps: &[(&str, usize)]) -> Vec<String> {
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
    let found = violations("long.md", &text, &[("long.md", 0)]);
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
    );
    assert!(
        found.iter().any(|v| v.contains("review.md")
            && v.contains("強調字上限")
            && v.contains("上限 1")
            && v.contains("實際 2")
            && v.contains(SPEC_REF)),
        "{found:?}"
    );
    assert!(violations("review.md", "MUST do this.\n", &[("review.md", 2)]).is_empty());
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
    let found = violations("x.md", "此規則於 2026-08-10 加入\n", &[("x.md", 0)]);
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
    let found = violations("x.md", "Use the AskUserQuestion tool.\n", &[("x.md", 0)]);
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
    assert!(hardcoded_references("see .claude/skills/speclink-propose/SKILL.md").is_empty());
    assert!(hardcoded_references("run `/speclink:apply <name>` with --agent {{TOOL}}").is_empty());
    let found = violations("x.md", "run `/speclink-apply <name>`\n", &[("x.md", 0)]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("x.md") && v.contains("渲染記號") && v.contains("/speclink-apply")),
        "{found:?}"
    );
}

#[test]
fn an_unregistered_source_is_asked_to_register() {
    let found = violations("example.md", "plain text\n", &[("other.md", 0)]);
    assert!(
        found
            .iter()
            .any(|v| v.contains("example.md") && v.contains("EMPHASIS_CAPS")),
        "{found:?}"
    );
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
    let found: Vec<String> = names
        .iter()
        .flat_map(|name| {
            violations(
                name,
                &std::fs::read_to_string(dir.join(name)).unwrap(),
                EMPHASIS_CAPS,
            )
        })
        .collect();
    assert!(found.is_empty(), "\n{}", found.join("\n"));
}

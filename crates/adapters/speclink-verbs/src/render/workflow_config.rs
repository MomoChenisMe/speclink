//! Output of the workflow config verbs: `show` (human and JSON), the one-line
//! confirmation of a write, and the unified diff of a `--dry-run`.

use std::collections::BTreeMap;

use serde::Serialize;
use speclink_core::command::{WorkflowConfigShowOutcome, WorkflowConfigWriteOutcome};
use speclink_core::config::{
    ChangedKey, ReviewBaseline, ReviewConfig, SubagentModelConfig, WorkflowConfig,
    BUILTIN_SUBAGENT_MODELS,
};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};
use crate::families::workflow_config::{ReviewTarget, SubagentModelTarget, WorkflowConfigCommands};

/// `workflow-config show --json` payload. camelCase field names are the contract;
/// the values are CANONICAL (what the document says), never the three-layer
/// resolution — effective policy is the instructions payload's job.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShowJson {
    schema: Option<String>,
    locale: Option<String>,
    spec_locale: Option<String>,
    tdd: bool,
    audit: bool,
    worktree: bool,
    context: Option<String>,
    rules: BTreeMap<String, Vec<String>>,
    review: ReviewConfig,
    subagent_model: SubagentModelConfig,
    builtin_subagent_models: BTreeMap<&'static str, &'static [&'static str]>,
}

fn show_json(cfg: &WorkflowConfig) -> JsonForm {
    JsonForm::pretty(&ShowJson {
        schema: cfg.schema.clone(),
        locale: cfg.locale.clone(),
        spec_locale: cfg.spec_locale.clone(),
        tdd: cfg.tdd.unwrap_or(false),
        audit: cfg.audit.unwrap_or(false),
        worktree: cfg.worktree.unwrap_or(false),
        context: cfg.context_text(),
        rules: cfg.rules.clone(),
        review: cfg.review(),
        subagent_model: cfg.subagent_model(),
        builtin_subagent_models: BUILTIN_SUBAGENT_MODELS.iter().copied().collect(),
    })
}

/// `show`: the canonical view of the document `file`. The sub-agent models
/// are in the JSON form only.
pub fn show_human(file: &str, cfg: &WorkflowConfig, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} {file}", paint.bold("Workflow config:"));
    outln!(t);
    // An unset text key shows what it defaults to.
    let or_unset = |value: &Option<String>, default: &str| match value {
        Some(v) => v.clone(),
        None => format!("unset ({default})"),
    };
    outln!(t, "  {:<13}{}", "schema", or_unset(&cfg.schema, "spec-driven"));
    outln!(t, "  {:<13}{}", "locale", or_unset(&cfg.locale, "English"));
    outln!(t, "  {:<13}{}", "spec_locale", or_unset(&cfg.spec_locale, "specs in English"));
    outln!(t, "  {:<13}{}", "tdd", toggle_display(cfg.tdd));
    outln!(t, "  {:<13}{}", "audit", toggle_display(cfg.audit));
    outln!(t, "  {:<13}{}", "worktree", toggle_display(cfg.worktree));
    let context = match cfg.context_text() {
        Some(text) => format!("{} lines", text.lines().count()),
        None => "none".to_string(),
    };
    outln!(t, "  {:<13}{context}", "context");
    let rules: Vec<String> = cfg
        .rules
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .map(|(artifact, entries)| format!("{artifact} {}", entries.len()))
        .collect();
    let rules = if rules.is_empty() { "none".to_string() } else { rules.join(", ") };
    outln!(t, "  {:<13}{rules}", "rules");
    let review = cfg.review();
    let review = if review == ReviewConfig::default() {
        "none".to_string()
    } else {
        format!(
            "{}, critical {}, warning {}, suggestion {}",
            review.baseline.as_str(),
            review.critical.len(),
            review.warning.len(),
            review.suggestion.len()
        )
    };
    outln!(t, "  {:<13}{review}", "review");
    t
}

/// A toggle's canonical display: `false` is never stored, so "not set" and
/// "set to false" are the same state — name the default so it reads as one.
fn toggle_display(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "on",
        Some(false) | None => "unset (off)",
    }
}

pub fn show(outcome: &WorkflowConfigShowOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(show_json(&outcome.config)), |paint| {
        show_human(&outcome.file, &outcome.config, paint)
    })
}

/// A write verb's output: with `--dry-run` the unified diff of the document
/// and nothing else, otherwise the one-line confirmation. `command` gives
/// what only argv holds (the key and value of `set`, the tool of a model).
/// A write has no JSON form.
pub fn written(
    command: &WorkflowConfigCommands,
    outcome: &WorkflowConfigWriteOutcome,
    options: RenderOptions,
) -> Rendered {
    Rendered::of(options, None, |paint| {
        let mut t = Text::default();
        if command.dry_run() {
            let before = outcome.before.as_deref().unwrap_or_default();
            out!(t, "{}", unified_diff(&outcome.file, before, &outcome.after));
        } else {
            outln!(t, "{} {}", paint.green("✓"), summary(command, outcome));
        }
        t
    })
}

/// The text of the confirmation line. What the edit carried in stdin (the
/// line count of a context, the entry count of a list) is read back from the
/// document after the write: the command verified that it holds exactly that.
fn summary(command: &WorkflowConfigCommands, outcome: &WorkflowConfigWriteOutcome) -> String {
    use WorkflowConfigCommands as C;
    let after = &outcome.config;
    let changed = |key: ChangedKey| outcome.changed_keys.contains(&key);
    match command {
        // `show` is not a write; it has no confirmation.
        C::Show { .. } => String::new(),
        C::Set { key, value, .. } => format!("{key} = {value}"),
        C::Context { .. } => match after.context_text() {
            Some(text) => format!("context set ({} lines)", text.lines().count()),
            None => "context removed".to_string(),
        },
        C::Rules { artifact, .. } => {
            list_summary(&format!("rules.{artifact}"), after.rules.get(artifact).map_or(0, Vec::len))
        }
        C::Review { command } => match command.target() {
            ReviewTarget::List { list, .. } => {
                list_summary(&format!("review.{}", list.key()), list.entries(&mut after.review()).len())
            }
            ReviewTarget::Baseline { baseline, .. } => {
                let default = if baseline == ReviewBaseline::default() { " (default)" } else { "" };
                format!("review.baseline = {}{default}", baseline.as_str())
            }
        },
        // A write that leaves the block as it is says so.
        C::SubagentModel { command } => match command.target() {
            SubagentModelTarget::Cell { station, args } => {
                let key = format!("subagent_model.{}.{}", station.key(), args.tool);
                match args.model.trim() {
                    _ if !changed(ChangedKey::SubagentModel(station)) => format!("{key} unchanged"),
                    "" => format!("{key} removed"),
                    model => format!("{key} = {model}"),
                }
            }
            SubagentModelTarget::Choices { tool, .. } => {
                let key = format!("subagent_model.choices.{tool}");
                if !changed(ChangedKey::SubagentChoices) {
                    return format!("{key} unchanged");
                }
                list_summary(&key, after.subagent_model().choices.get(tool).map_or(0, Vec::len))
            }
        },
    }
}

/// The confirmation of a list write: an empty list removes its key.
fn list_summary(key: &str, count: usize) -> String {
    if count == 0 {
        format!("{key} removed")
    } else {
        format!("{key} = {count} entries")
    }
}

/// Unified diff over lines, generated here rather than shelled out to a system
/// `diff` (Windows has none): one hunk over the changed span between the common
/// prefix and suffix, with up to three lines of context on each side, and the
/// span itself compared line by line. Empty when the two texts are identical.
pub fn unified_diff(label: &str, old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    // The lines both texts start with, and the lines both end with.
    let mut prefix = 0;
    while prefix < old_lines.len() && prefix < new_lines.len() && old_lines[prefix] == new_lines[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old_lines.len() - prefix
        && suffix < new_lines.len() - prefix
        && old_lines[old_lines.len() - 1 - suffix] == new_lines[new_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }
    const CONTEXT: usize = 3;
    let start = prefix - prefix.min(CONTEXT);
    let old_span_end = old_lines.len() - suffix;
    let new_span_end = new_lines.len() - suffix;
    let old_end = old_span_end + suffix.min(CONTEXT);
    let new_end = new_span_end + suffix.min(CONTEXT);
    let mut out = format!("--- a/{label}\n+++ b/{label}\n");
    out.push_str(&format!(
        "@@ -{} +{} @@\n",
        hunk_range(start, old_end - start),
        hunk_range(start, new_end - start)
    ));
    for line in &old_lines[start..prefix] {
        out.push_str(&format!(" {line}\n"));
    }
    push_changed_span(&mut out, &old_lines[prefix..old_span_end], &new_lines[prefix..new_span_end]);
    for line in &old_lines[old_span_end..old_end] {
        out.push_str(&format!(" {line}\n"));
    }
    out
}

/// The changed span line by line, by longest common subsequence: lines both
/// sides keep are context, the rest `-`/`+`, each run's removals before its
/// additions. The table is |o|×|n| — negligible at config.yaml's size. A host
/// can pass stdin of any size, so a span past [`LCS_CELL_LIMIT`] is not
/// compared line by line: it prints as one block, every removal and then
/// every addition (a correct diff, only less fine).
fn push_changed_span(out: &mut String, old: &[&str], new: &[&str]) {
    /// The largest table the line-by-line comparison builds: 4M cells, about
    /// 32 MB. A 2000-line block against another 2000-line block still fits.
    const LCS_CELL_LIMIT: usize = 4_000_000;
    if old.len().saturating_mul(new.len()) > LCS_CELL_LIMIT {
        for line in old {
            out.push_str(&format!("-{line}\n"));
        }
        for line in new {
            out.push_str(&format!("+{line}\n"));
        }
        return;
    }
    // lcs[i][j]: the common-subsequence length of old[i..] and new[j..].
    let mut lcs = vec![vec![0usize; new.len() + 1]; old.len() + 1];
    for i in (0..old.len()).rev() {
        for j in (0..new.len()).rev() {
            lcs[i][j] = if old[i] == new[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut added = String::new();
    while i < old.len() || j < new.len() {
        if i < old.len() && j < new.len() && old[i] == new[j] {
            out.push_str(&std::mem::take(&mut added));
            out.push_str(&format!(" {}\n", old[i]));
            i += 1;
            j += 1;
        } else if j == new.len() || (i < old.len() && lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push_str(&format!("-{}\n", old[i]));
            i += 1;
        } else {
            added.push_str(&format!("+{}\n", new[j]));
            j += 1;
        }
    }
    out.push_str(&added);
}

/// One side of a hunk header. An empty side is `0,0` by unified-diff convention
/// (there is no line 1 to point at).
fn hunk_range(start: usize, count: usize) -> String {
    if count == 0 {
        "0,0".to_string()
    } else {
        format!("{},{count}", start + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::families::workflow_config::WorkflowConfigCommands;
    use crate::{ParsedVerb, SpecVerb};
    use speclink_core::command::{WorkflowConfigShowOutcome, WorkflowConfigWriteOutcome};
    use speclink_core::config::{changed_keys, WorkflowConfig};

    const HUMAN: RenderOptions = RenderOptions { json: false, color: false };
    const COLOR: RenderOptions = RenderOptions { json: false, color: true };
    const JSON: RenderOptions = RenderOptions { json: true, color: false };
    const FILE: &str = "speclink/config.yaml";

    /// The fixed document of the family's baseline (task 1.1).
    const FULL: &str = "\
# team workflow config
schema: spec-driven
locale: tw
tdd: true

# shared context
context: |
  Project context line one.
  Line two.
rules:
  design:
  - Name the crate boundary
  - List one alternative
  tasks:
  - Use checkbox format
review:
  critical:
  - 所有 SQL 一律用參數化查詢
  warning:
  - No unwrap in library code
subagent_model:
  # team default
  review:
    claude: opus
  choices:
    codex:
    - gpt-6.1-sol
    - gpt-6-astra
";

    fn config(text: &str) -> WorkflowConfig {
        WorkflowConfig::from_text(Some(text)).expect("the document parses")
    }

    fn shown(text: Option<&str>) -> WorkflowConfigShowOutcome {
        WorkflowConfigShowOutcome {
            file: FILE.to_string(),
            config: WorkflowConfig::from_text(text).expect("the document parses"),
            exists: text.is_some(),
        }
    }

    /// The result of a write that turned `before` into `after`.
    fn outcome(before: &str, after: &str, written: bool) -> WorkflowConfigWriteOutcome {
        WorkflowConfigWriteOutcome {
            file: FILE.to_string(),
            before: Some(before.to_string()),
            after: after.to_string(),
            config: config(after),
            changed_keys: changed_keys(&config(before), &config(after)),
            written,
        }
    }

    fn parsed(argv: &[&str]) -> ParsedVerb {
        crate::parse(argv.iter().copied()).unwrap_or_else(|e| panic!("{argv:?}: {}", e.message))
    }

    fn command(verb: &ParsedVerb) -> &WorkflowConfigCommands {
        match verb.spec() {
            SpecVerb::WorkflowConfig(a) => &a.command,
            other => panic!("not a workflow-config verb: {other:?}"),
        }
    }

    /// The confirmation line of `argv` for a write that turned FULL into `after`.
    fn confirmation(argv: &[&str], after: &str) -> String {
        let verb = parsed(argv);
        let out = written(command(&verb), &outcome(FULL, after, true), HUMAN);
        assert_eq!(out.stderr, "", "{argv:?}");
        assert_eq!(out.json, None, "{argv:?}: a write has no JSON form");
        out.stdout
    }

    // --- show: the baseline's stdout, whole ---

    #[test]
    fn show_human_is_the_baseline_text() {
        let out = show(&shown(Some(FULL)), HUMAN);
        assert_eq!(
            out.stdout,
            "\
Workflow config: speclink/config.yaml

  schema       spec-driven
  locale       tw
  spec_locale  unset (specs in English)
  tdd          on
  audit        unset (off)
  worktree     unset (off)
  context      2 lines
  rules        design 2, tasks 1
  review       extend, critical 1, warning 1, suggestion 0
"
        );
        assert_eq!(out.stderr, "");
    }

    #[test]
    fn show_human_of_a_missing_document_names_every_default() {
        let out = show(&shown(None), HUMAN);
        assert_eq!(
            out.stdout,
            "\
Workflow config: speclink/config.yaml

  schema       unset (spec-driven)
  locale       unset (English)
  spec_locale  unset (specs in English)
  tdd          unset (off)
  audit        unset (off)
  worktree     unset (off)
  context      none
  rules        none
  review       none
"
        );
    }

    #[test]
    fn show_colors_only_the_title() {
        let out = show(&shown(Some(FULL)), COLOR);
        assert!(
            out.stdout.starts_with("\x1b[1mWorkflow config:\x1b[0m speclink/config.yaml\n\n  schema       spec-driven\n"),
            "{:?}",
            out.stdout
        );
        assert_eq!(out.stdout.matches('\x1b').count(), 2);
    }

    #[test]
    fn show_json_is_the_baseline_text() {
        let out = show(&shown(Some(FULL)), JSON);
        assert_eq!(
            out.stdout,
            r#"{
  "schema": "spec-driven",
  "locale": "tw",
  "specLocale": null,
  "tdd": true,
  "audit": false,
  "worktree": false,
  "context": "Project context line one.\nLine two.",
  "rules": {
    "design": [
      "Name the crate boundary",
      "List one alternative"
    ],
    "tasks": [
      "Use checkbox format"
    ]
  },
  "review": {
    "baseline": "extend",
    "critical": [
      "所有 SQL 一律用參數化查詢"
    ],
    "warning": [
      "No unwrap in library code"
    ],
    "suggestion": []
  },
  "subagentModel": {
    "review": {
      "claude": "opus"
    },
    "verify": {},
    "choices": {
      "codex": [
        "gpt-6.1-sol",
        "gpt-6-astra"
      ]
    }
  },
  "builtinSubagentModels": {
    "claude": [
      "sonnet",
      "opus",
      "haiku",
      "fable"
    ]
  }
}
"#
        );
        // The value is there whether or not the JSON form was asked for.
        let value = out.json.expect("show has a structured value");
        assert_eq!(show(&shown(Some(FULL)), HUMAN).json, Some(value.clone()));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&out.stdout).unwrap(), value);
    }

    // --- the write verbs: the baseline's confirmation lines ---

    #[test]
    fn each_write_verb_confirms_in_one_line() {
        let set = |from: &str, to: &str| FULL.replace(from, to);
        let rows: Vec<(&[&str], String, &str)> = vec![
            (&["workflow-config", "set", "tdd", "false"], set("tdd: true\n", ""), "✓ tdd = false\n"),
            (&["workflow-config", "set", "locale", ""], set("locale: tw\n", ""), "✓ locale = \n"),
            (
                &["workflow-config", "context", "--stdin"],
                set("  Project context line one.\n  Line two.\n", "  New context.\n  Second line.\n\n  Fourth line.\n"),
                "✓ context set (4 lines)\n",
            ),
            (
                &["workflow-config", "context", "--stdin"],
                set("context: |\n  Project context line one.\n  Line two.\n", ""),
                "✓ context removed\n",
            ),
            (
                &["workflow-config", "rules", "proposal", "--stdin"],
                set("rules:\n", "rules:\n  proposal:\n  - Rule one\n  - Rule two\n"),
                "✓ rules.proposal = 2 entries\n",
            ),
            (
                &["workflow-config", "rules", "design", "--stdin"],
                set("  design:\n  - Name the crate boundary\n  - List one alternative\n", ""),
                "✓ rules.design removed\n",
            ),
            (
                &["workflow-config", "review", "critical", "--stdin"],
                set("  - 所有 SQL 一律用參數化查詢\n", "  - one rule\n  - second rule\n"),
                "✓ review.critical = 2 entries\n",
            ),
            (
                &["workflow-config", "review", "warning", "--stdin"],
                set("  warning:\n  - No unwrap in library code\n", ""),
                "✓ review.warning removed\n",
            ),
            (
                &["workflow-config", "review", "baseline", "replace"],
                set("review:\n  critical:\n", "review:\n  baseline: replace\n  critical:\n"),
                "✓ review.baseline = replace\n",
            ),
            (&["workflow-config", "review", "baseline", "extend"], FULL.to_string(), "✓ review.baseline = extend (default)\n"),
            (
                &["workflow-config", "subagent-model", "review", "codex", "gpt-6.1-sol"],
                set("    claude: opus\n", "    claude: opus\n    codex: gpt-6.1-sol\n"),
                "✓ subagent_model.review.codex = gpt-6.1-sol\n",
            ),
            (
                &["workflow-config", "subagent-model", "review", "claude", ""],
                set("  review:\n    claude: opus\n", ""),
                "✓ subagent_model.review.claude removed\n",
            ),
            (
                &["workflow-config", "subagent-model", "verify", "claude", " sonnet "],
                set("  choices:\n", "  verify:\n    claude: sonnet\n  choices:\n"),
                "✓ subagent_model.verify.claude = sonnet\n",
            ),
            (
                &["workflow-config", "subagent-model", "choices", "claude", "--stdin"],
                set("  choices:\n", "  choices:\n    claude:\n    - a\n    - b\n"),
                "✓ subagent_model.choices.claude = 2 entries\n",
            ),
            (
                &["workflow-config", "subagent-model", "choices", "codex", "--stdin"],
                set("  choices:\n    codex:\n    - gpt-6.1-sol\n    - gpt-6-astra\n", ""),
                "✓ subagent_model.choices.codex removed\n",
            ),
            // The block did not move: the line says so.
            (
                &["workflow-config", "subagent-model", "review", "claude", "opus"],
                FULL.to_string(),
                "✓ subagent_model.review.claude unchanged\n",
            ),
            (
                &["workflow-config", "subagent-model", "verify", "claude", ""],
                FULL.to_string(),
                "✓ subagent_model.verify.claude unchanged\n",
            ),
            (
                &["workflow-config", "subagent-model", "choices", "claude", "--stdin"],
                FULL.to_string(),
                "✓ subagent_model.choices.claude unchanged\n",
            ),
        ];
        for (argv, after, line) in rows {
            assert_eq!(confirmation(argv, &after), line, "{argv:?}");
        }
    }

    #[test]
    fn the_confirmation_mark_is_green_in_color() {
        let verb = parsed(&["workflow-config", "set", "tdd", "false"]);
        let after = FULL.replace("tdd: true\n", "");
        let out = written(command(&verb), &outcome(FULL, &after, true), COLOR);
        assert_eq!(out.stdout, "\x1b[32m✓\x1b[0m tdd = false\n");
    }

    // --- dry-run: the baseline's diff, whole ---

    #[test]
    fn a_dry_run_prints_the_unified_diff_and_nothing_else() {
        let verb = parsed(&["workflow-config", "rules", "proposal", "--stdin", "--dry-run"]);
        let after = FULL.replace("rules:\n", "rules:\n  proposal:\n  - Rule one\n  - Rule two\n");
        for options in [HUMAN, COLOR] {
            let out = written(command(&verb), &outcome(FULL, &after, false), options);
            assert_eq!(
                out.stdout,
                "\
--- a/speclink/config.yaml
+++ b/speclink/config.yaml
@@ -8,6 +8,9 @@
   Project context line one.
   Line two.
 rules:
+  proposal:
+  - Rule one
+  - Rule two
   design:
   - Name the crate boundary
   - List one alternative
"
            );
            assert_eq!(out.stderr, "");
        }
    }

    #[test]
    fn a_dry_run_of_a_missing_document_diffs_against_nothing() {
        let verb = parsed(&["workflow-config", "set", "tdd", "true", "--dry-run"]);
        let created = WorkflowConfigWriteOutcome {
            file: FILE.to_string(),
            before: None,
            ..outcome("", "tdd: true\n", false)
        };
        assert_eq!(
            written(command(&verb), &created, HUMAN).stdout,
            "--- a/speclink/config.yaml\n+++ b/speclink/config.yaml\n@@ -0,0 +1,1 @@\n+tdd: true\n"
        );
    }

    #[test]
    fn a_dry_run_that_changes_nothing_prints_nothing() {
        let verb = parsed(&["workflow-config", "set", "tdd", "true", "--dry-run"]);
        assert_eq!(written(command(&verb), &outcome(FULL, FULL, false), HUMAN).stdout, "");
    }

    // --- unified_diff ---

    /// The hunk body of a diff: every line after the `@@` header.
    fn hunk_body(diff: &str) -> Vec<&str> {
        diff.lines().skip_while(|l| !l.starts_with("@@")).skip(1).collect()
    }

    /// The `(old, new)` line counts the `@@ -a,b +c,d @@` header declares.
    fn header_counts(diff: &str) -> (usize, usize) {
        let header = diff.lines().find(|l| l.starts_with("@@")).expect("hunk header");
        let count = |side: &str| side.split(',').nth(1).unwrap().parse::<usize>().unwrap();
        let mut sides = header.trim_matches('@').split_whitespace();
        (count(sides.next().unwrap()), count(sides.next().unwrap()))
    }

    #[test]
    fn unified_diff_is_empty_for_identical_text() {
        assert_eq!(unified_diff("f", "a\nb\n", "a\nb\n"), "");
    }

    #[test]
    fn unified_diff_of_one_contiguous_change_is_unchanged() {
        // 只有單一連續改動時，中段沒有共同行——輸出與改版前位元級相同。
        let old = "a\nb\nc\nd\ne\nf\ng\nh\n";
        let new = "a\nb\nc\nd\nX\nY\nf\ng\nh\n";
        assert_eq!(
            unified_diff("f", old, new),
            "--- a/f\n+++ b/f\n@@ -2,7 +2,8 @@\n b\n c\n d\n-e\n+X\n+Y\n f\n g\n h\n"
        );
    }

    #[test]
    fn unified_diff_shows_lines_between_two_changes_as_context() {
        // 兩處改動夾著未改行：中間行只以 context 出現，同一位置的 `-` 先於 `+`。
        let old = "rules:\n  tasks:\n  - 甲\n  - 乙\n  - 丙\n";
        let new = "rules:\n  tasks:\n  - 甲二\n  - 乙\n  - 丙\n  - 丁\n";
        assert_eq!(
            hunk_body(&unified_diff("f", old, new)),
            [" rules:", "   tasks:", "-  - 甲", "+  - 甲二", "   - 乙", "   - 丙", "+  - 丁"]
        );
    }

    #[test]
    fn unified_diff_lists_removals_before_additions_at_each_position() {
        let old = "k\nx1\nx2\nm\ny\n";
        let new = "k\np1\nm\nq1\nq2\n";
        assert_eq!(
            hunk_body(&unified_diff("f", old, new)),
            [" k", "-x1", "-x2", "+p1", " m", "-y", "+q1", "+q2"]
        );
    }

    // The line-by-line comparison needs a table of |old|×|new| cells. A host
    // can hand over stdin of any size, so past a fixed size the span prints
    // as one block: every removal, then every addition.
    #[test]
    fn unified_diff_of_a_very_large_span_is_one_block() {
        let lines = |tag: &str| -> String {
            (0..3000).map(|i| if i % 2 == 0 { format!("same {i}\n") } else { format!("{tag} {i}\n") }).collect()
        };
        let (old, new) = (lines("old"), lines("new"));
        let diff = unified_diff("f", &old, &new);
        let body = hunk_body(&diff);
        // 第一行（`same 0`）兩邊相同，是共同的前綴；最後一行不同，所以沒有共同的
        // 後綴。其餘 2999 行整段是一個區塊。
        assert_eq!(body.first(), Some(&" same 0"));
        let span = &body[1..];
        let removed = span.iter().take_while(|l| l.starts_with('-')).count();
        assert_eq!(removed, 2999, "every old line of the span, in one run");
        assert!(span[removed..].iter().all(|l| l.starts_with('+')), "then every new line");
        assert_eq!(span.len(), 2 * 2999);
        assert_eq!(header_counts(&diff), (3000, 3000));
    }

    #[test]
    fn unified_diff_header_counts_match_the_hunk_body() {
        let cases = [
            ("a\nb\nc\n", "a\nB\nc\nd\n"),
            ("1\n2\n3\n4\n5\n6\n7\n8\n9\n", "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n"),
            ("k\nx1\nx2\nm\ny\n", "k\np1\nm\nq1\nq2\n"),
            ("", "only\n"),
            ("gone\n", ""),
        ];
        for (old, new) in cases {
            let diff = unified_diff("f", old, new);
            let body = hunk_body(&diff);
            let old_side = body.iter().filter(|l| l.starts_with(' ') || l.starts_with('-')).count();
            let new_side = body.iter().filter(|l| l.starts_with(' ') || l.starts_with('+')).count();
            assert_eq!(header_counts(&diff), (old_side, new_side), "{diff}");
        }
    }
}

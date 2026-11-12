//! Output of `evidence show`: the files each completed task of a change touched.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;
use speclink_core::command::EvidenceShowOutcome;
use speclink_core::tasks::EvidenceEntry;

use super::{value_of, JsonForm, Paint, RenderOptions, Rendered, Text};

/// The JSON form, in its field order. `files` is every entry's files, each
/// once, in byte order: the engine's result does not hold it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceJson<'a> {
    change: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    archived_as: Option<&'a str>,
    entries: &'a [EvidenceEntry],
    files: BTreeSet<&'a str>,
}

fn json_of(o: &EvidenceShowOutcome) -> EvidenceJson<'_> {
    EvidenceJson {
        change: &o.change,
        archived_as: o.archived_as.as_deref(),
        entries: &o.entries,
        files: o.entries.iter().flat_map(|e| e.touched_files.iter().map(String::as_str)).collect(),
    }
}

/// `evidence show --json`: `{ change, archivedAs?, entries, files }`.
pub fn show_json(o: &EvidenceShowOutcome) -> Value {
    value_of(&json_of(o))
}

/// `evidence show`: the title line, the archived name when the record came
/// from the archive, then one block per task. No record is one line.
pub fn show_human(o: &EvidenceShowOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    if o.entries.is_empty() {
        outln!(t, "No evidence recorded for '{}'.", o.change);
        return t;
    }
    outln!(t, "Evidence: {}  ({} task(s))", paint.bold(&o.change), o.entries.len());
    if let Some(dated) = &o.archived_as {
        outln!(t, "  [archived as {dated}]");
    }
    for entry in &o.entries {
        outln!(t);
        outln!(t, "  {}  {}", paint.bold(&entry.task_id), entry.task_desc);
        outln!(t, "    recorded {}", entry.recorded_at);
        for file in &entry.touched_files {
            outln!(t, "    {file}");
        }
    }
    t
}

pub fn show(o: &EvidenceShowOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&json_of(o))), |paint| show_human(o, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Paint, RenderOptions};
    use serde_json::json;
    use speclink_core::command::EvidenceShowOutcome;
    use speclink_core::tasks::EvidenceEntry;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    fn entry(task_id: &str, desc: &str, files: &[&str], at: &str) -> EvidenceEntry {
        EvidenceEntry {
            task_id: task_id.to_string(),
            task_desc: desc.to_string(),
            actor: None,
            repo: None,
            head_commit: None,
            touched_files: files.iter().map(|f| f.to_string()).collect(),
            recorded_at: at.to_string(),
        }
    }

    /// The spec's two records: task 1.1 on a.rs and b.rs, task 1.2 on b.rs and c.rs.
    fn demo(archived_as: Option<&str>) -> EvidenceShowOutcome {
        EvidenceShowOutcome {
            change: "demo".to_string(),
            archived_as: archived_as.map(str::to_string),
            entries: vec![
                entry("tsk_a", "1.1 First", &["src/b.rs", "src/a.rs"], "2026-11-12T03:00:00Z"),
                entry("tsk_b", "1.2 Second", &["src/b.rs", "src/c.rs"], "2026-11-12T04:00:00Z"),
            ],
        }
    }

    fn none(change: &str) -> EvidenceShowOutcome {
        EvidenceShowOutcome { change: change.to_string(), archived_as: None, entries: Vec::new() }
    }

    #[test]
    fn the_human_form_has_a_title_line_and_one_block_per_task() {
        assert_eq!(
            show_human(&demo(None), &PLAIN).stdout,
            "Evidence: demo  (2 task(s))\n\n  tsk_a  1.1 First\n    recorded 2026-11-12T03:00:00Z\n    src/b.rs\n    src/a.rs\n\n  tsk_b  1.2 Second\n    recorded 2026-11-12T04:00:00Z\n    src/b.rs\n    src/c.rs\n"
        );
        assert_eq!(show_human(&demo(None), &PLAIN).stderr, "");
    }

    #[test]
    fn color_is_on_the_title_line_and_the_task_lines_only() {
        let colored = show_human(&demo(None), &COLOR).stdout;
        let plain = show_human(&demo(None), &PLAIN).stdout;
        assert!(!plain.contains('\u{1b}'), "--no-color leaves out the color codes: {plain:?}");
        assert_eq!(
            colored,
            "Evidence: \x1b[1mdemo\x1b[0m  (2 task(s))\n\n  \x1b[1mtsk_a\x1b[0m  1.1 First\n    recorded 2026-11-12T03:00:00Z\n    src/b.rs\n    src/a.rs\n\n  \x1b[1mtsk_b\x1b[0m  1.2 Second\n    recorded 2026-11-12T04:00:00Z\n    src/b.rs\n    src/c.rs\n"
        );
        // Without the codes the two are the same text.
        assert_eq!(colored.replace("\x1b[1m", "").replace("\x1b[0m", ""), plain);
    }

    #[test]
    fn an_archived_change_adds_the_line_that_names_it() {
        let out = show_human(&demo(Some("2026-11-12-demo")), &PLAIN).stdout;
        assert!(
            out.starts_with("Evidence: demo  (2 task(s))\n  [archived as 2026-11-12-demo]\n\n  tsk_a  1.1 First\n"),
            "{out}"
        );
        assert_eq!(out.matches("archived as").count(), 1);
    }

    #[test]
    fn no_record_is_one_line() {
        assert_eq!(show_human(&none("fresh"), &PLAIN).stdout, "No evidence recorded for 'fresh'.\n");
        assert_eq!(show_human(&none("fresh"), &COLOR).stdout, "No evidence recorded for 'fresh'.\n");
    }

    #[test]
    fn the_json_form_is_change_entries_and_the_sorted_union_of_the_files() {
        assert_eq!(
            show_json(&demo(None)),
            json!({
                "change": "demo",
                "entries": [
                    {
                        "taskId": "tsk_a",
                        "taskDesc": "1.1 First",
                        "touchedFiles": ["src/b.rs", "src/a.rs"],
                        "recordedAt": "2026-11-12T03:00:00Z",
                    },
                    {
                        "taskId": "tsk_b",
                        "taskDesc": "1.2 Second",
                        "touchedFiles": ["src/b.rs", "src/c.rs"],
                        "recordedAt": "2026-11-12T04:00:00Z",
                    },
                ],
                "files": ["src/a.rs", "src/b.rs", "src/c.rs"],
            })
        );
    }

    #[test]
    fn the_json_form_names_the_archive_only_for_an_archived_change() {
        assert!(show_json(&demo(None)).get("archivedAs").is_none());
        assert_eq!(show_json(&demo(Some("2026-11-12-demo")))["archivedAs"], "2026-11-12-demo");
    }

    #[test]
    fn an_entry_carries_actor_repo_and_head_commit_only_when_it_has_them() {
        let mut o = demo(None);
        o.entries[1].actor = Some("Alice".to_string());
        o.entries[1].repo = Some("web".to_string());
        o.entries[1].head_commit = Some("abc123".to_string());
        let value = show_json(&o);
        let keys = |v: &serde_json::Value| -> Vec<String> { v.as_object().unwrap().keys().cloned().collect() };
        assert_eq!(keys(&value["entries"][0]), ["recordedAt", "taskDesc", "taskId", "touchedFiles"]);
        assert_eq!(
            keys(&value["entries"][1]),
            ["actor", "headCommit", "recordedAt", "repo", "taskDesc", "taskId", "touchedFiles"]
        );
        assert_eq!(value["entries"][1]["headCommit"], "abc123");
    }

    #[test]
    fn the_json_form_of_no_record_has_two_empty_arrays() {
        assert_eq!(show_json(&none("fresh")), json!({ "change": "fresh", "entries": [], "files": [] }));
    }

    #[test]
    fn files_sort_by_bytes() {
        let o = EvidenceShowOutcome {
            change: "demo".to_string(),
            archived_as: None,
            entries: vec![entry("tsk_a", "1.1", &["b.rs", "Z.rs", "a/x.rs", "a.rs", "b.rs"], "2026-11-12T03:00:00Z")],
        };
        assert_eq!(show_json(&o)["files"], json!(["Z.rs", "a.rs", "a/x.rs", "b.rs"]));
    }

    #[test]
    fn show_prints_the_json_text_in_field_order_with_a_final_newline() {
        let out = show(&demo(Some("2026-11-12-demo")), RenderOptions { json: true, color: true });
        assert!(
            out.stdout.starts_with("{\n  \"change\": \"demo\",\n  \"archivedAs\": \"2026-11-12-demo\",\n  \"entries\": [\n"),
            "{}",
            out.stdout
        );
        assert!(
            out.stdout.ends_with("  \"files\": [\n    \"src/a.rs\",\n    \"src/b.rs\",\n    \"src/c.rs\"\n  ]\n}\n"),
            "{}",
            out.stdout
        );
        assert_eq!(out.json, Some(show_json(&demo(Some("2026-11-12-demo")))));
        // The human form still carries the structured value.
        let human = show(&demo(None), RenderOptions::default());
        assert!(human.stdout.starts_with("Evidence: demo"));
        assert_eq!(human.json, Some(show_json(&demo(None))));
    }
}

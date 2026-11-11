//! Output of the read verbs: list, show, status.

use serde_json::{json, Value};
use speclink_core::command::{ListOutcome, ShowChange, ShowOutcome};
use speclink_core::status::StatusReport;

use super::{value_of, JsonForm, Paint, RenderOptions, Rendered, Text};

// A list always carries at least one section; an outcome with neither reads
// as an empty specs section.
static NO_SPECS: Value = Value::Null;

/// `list --json`: `{ changes }`, `{ specs }` or both, as the flags asked.
pub fn list_json(list: &ListOutcome) -> Value {
    // `--specs` alone omits the changes section (`changes` is None).
    let Some(items) = &list.changes else {
        return json!({ "specs": list.specs.as_ref().unwrap_or(&NO_SPECS) });
    };
    match &list.specs {
        Some(specs) => json!({ "changes": items, "specs": specs }),
        None => json!({ "changes": items }),
    }
}

/// `list`: one line per change, then the specs section when asked for.
pub fn list_human(list: &ListOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    let Some(items) = &list.changes else {
        specs_section(&mut t, list.specs.as_ref().unwrap_or(&NO_SPECS), paint);
        return t;
    };
    if items.is_empty() {
        outln!(t, "No active changes.");
        if let Some(specs) = &list.specs {
            outln!(t);
            specs_section(&mut t, specs, paint);
        }
        return t;
    }
    outln!(t, "{}", paint.bold("Changes:"));
    for c in items {
        // The progress marker is omitted entirely for changes with zero tasks.
        let marker = if c.total_tasks > 0 {
            format!(" [{}/{}]", c.completed_tasks, c.total_tasks)
        } else {
            String::new()
        };
        // The dim wrapper always prints — an empty summary yields the empty
        // \x1b[2m\x1b[0m pair in color mode and nothing in plain mode.
        let suffix = if c.summary.is_empty() {
            String::new()
        } else {
            format!(" — {}", c.summary)
        };
        // Fail-closed diagnostic (frozen marker text): only corrupt-metadata
        // lines gain the trailing marker; valid lines stay byte-identical.
        let invalid = if c.meta_error.is_some() {
            format!(" {}", paint.red("(invalid .meta.yaml)"))
        } else {
            String::new()
        };
        // Fixed literal, no color: `--no-color` must read exactly the same.
        let worktree = if c.worktree.is_some() { " [worktree]" } else { "" };
        outln!(
            t,
            "  {} {}{marker}{}{invalid}{worktree}",
            paint.cyan("•"),
            c.name,
            paint.dim(&suffix)
        );
    }
    if let Some(specs) = &list.specs {
        outln!(t);
        specs_section(&mut t, specs, paint);
    }
    t
}

/// The specs section, from the outcome's `{id, path}` items.
fn specs_section(t: &mut Text, specs: &Value, paint: &Paint) {
    let items = specs.as_array().map(Vec::as_slice).unwrap_or_default();
    if items.is_empty() {
        outln!(t, "No specs.");
        return;
    }
    outln!(t, "{}", paint.bold("Specs:"));
    for s in items {
        outln!(t, "  {} {}", paint.cyan("•"), s["id"].as_str().unwrap_or_default());
    }
}

pub fn list(list: &ListOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&list_json(list))), |paint| list_human(list, paint))
}

fn delta_spec_files(c: &ShowChange) -> Vec<String> {
    c.delta_capabilities.iter().map(|cap| format!("{cap}/spec.md")).collect()
}

/// `show --json`: a spec as `{ files, name }`, a change as its documents.
pub fn show_json(show: &ShowOutcome) -> Value {
    match show {
        ShowOutcome::Spec { name, content } => json!({
            "files": [{ "content": content, "name": "spec.md" }],
            "name": name,
        }),
        ShowOutcome::Change(c) => json!({
            "name": c.name,
            "schema": c.schema,
            "created": c.created,
            "proposal": c.proposal,
            "design": c.design,
            "tasks": c.tasks,
            "deltaSpecs": delta_spec_files(c),
            "fromDiscussions": c.from_discussions,
            "restaleFrom": c.restale_from,
        }),
    }
}

/// `show`: the header lines, then the proposal and the delta spec list.
pub fn show_human(show: &ShowOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    match show {
        ShowOutcome::Spec { name, content } => {
            outln!(t, "{}: {name}", paint.bold("Spec"));
            outln!(t);
            outln!(t, "{}", paint.dim("--- spec.md ---"));
            out!(t, "{content}");
            outln!(t); // A trailing newline is always emitted (an extra blank when content ends with \n)
        }
        ShowOutcome::Change(c) => {
            let caps = delta_spec_files(c);
            outln!(t, "{}: {}", paint.bold("Change"), c.name);
            if let Some(schema_name) = &c.schema {
                outln!(t, "{}: {schema_name}", paint.bold("Schema"));
            }
            if let Some(created) = &c.created {
                outln!(t, "{}: {created}", paint.bold("Created"));
            }
            // The header's trailing blank line prints only when a section follows.
            if c.proposal.is_some() || !caps.is_empty() {
                outln!(t);
            }
            // The Proposal section renders whenever the FILE exists (even empty) — frozen behavior.
            if let Some(proposal) = &c.proposal {
                outln!(t, "{}", paint.dim("--- Proposal ---"));
                out!(t, "{proposal}");
                if !caps.is_empty() {
                    // The proposal's own trailing newline determines the blank-line count before the header.
                    out!(t, "\n\n{}\n", paint.dim("--- Delta Specs ---"));
                    for cap in &caps {
                        outln!(t, "  {cap}");
                    }
                } else {
                    // A newline is always appended after the proposal body (an extra blank
                    // line when the file already ends with one), same as the spec branch above.
                    outln!(t);
                }
            } else if !caps.is_empty() {
                // No proposal, but delta specs exist — still render the section.
                outln!(t, "{}", paint.dim("--- Delta Specs ---"));
                for cap in &caps {
                    outln!(t, "  {cap}");
                }
            }
        }
    }
    t
}

pub fn show(show: &ShowOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&show_json(show))), |paint| show_human(show, paint))
}

/// `status --json`: the report as the engine serializes it.
pub fn status_json(report: &StatusReport) -> Value {
    value_of(report)
}

/// `status`: one line per artifact with its state.
pub fn status_human(report: &StatusReport, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{}: {}", paint.bold("Change"), report.change_name);
    outln!(t, "{}: {}", paint.bold("Schema"), report.schema_name);
    outln!(t);
    for art in &report.artifacts {
        let sym = match art.status.as_str() {
            "done" => paint.green("✓"),
            "ready" => paint.yellow("○"),
            _ => paint.red("✗"),
        };
        outln!(t, "  {sym} {} ({})", paint.bold(&art.id), paint.dim(&art.output_path));
        if art.status == "blocked" && !art.blocked_by.is_empty() {
            outln!(t, "    blocked by: {}", art.blocked_by.join(", "));
        }
    }
    outln!(t);
    if report.is_complete {
        outln!(t, "  {} All artifacts complete", paint.green("✓"));
    }
    t
}

pub fn status(report: &StatusReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(report)), |paint| status_human(report, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::command::ShowChange;
    use speclink_core::listing::{ListChangeJson, ListWorktreeJson};
    use speclink_core::status::ArtifactStatusJson;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    fn change(name: &str, done: usize, total: usize, summary: &str) -> ListChangeJson {
        ListChangeJson {
            completed_tasks: done,
            name: name.to_string(),
            status: "in-progress".to_string(),
            summary: summary.to_string(),
            total_tasks: total,
            restale_from: Vec::new(),
            meta_error: None,
            delta_capabilities: Vec::new(),
            worktree: None,
        }
    }

    fn specs() -> Value {
        json!([{ "id": "auth", "path": "/p/specs/auth" }, { "id": "billing", "path": "/p/specs/billing" }])
    }

    fn both() -> ListOutcome {
        ListOutcome {
            changes: Some(vec![
                change("alpha", 1, 4, "Sign-in needs a second rule."),
                change("empty", 0, 0, ""),
            ]),
            specs: Some(specs()),
        }
    }

    // --- list ---

    #[test]
    fn list_human_prints_the_changes_then_the_specs() {
        assert_eq!(
            list_human(&both(), &PLAIN).stdout,
            "Changes:\n  • alpha [1/4] — Sign-in needs a second rule.\n  • empty\n\nSpecs:\n  • auth\n  • billing\n"
        );
    }

    #[test]
    fn list_human_colors_each_piece_and_marks_bad_metadata_and_worktrees() {
        let mut broken = change("beta", 0, 2, "Billing needs a yearly invoice…");
        broken.meta_error = Some("bad yaml".to_string());
        let mut mapped = change("gamma", 0, 0, "");
        mapped.worktree =
            Some(ListWorktreeJson { path: "/w/gamma".to_string(), branch: "speclink/gamma".to_string() });
        let list = ListOutcome { changes: Some(vec![broken, mapped]), specs: None };
        assert_eq!(
            list_human(&list, &COLOR).stdout,
            "\x1b[1mChanges:\x1b[0m\n  \x1b[36m•\x1b[0m beta [0/2]\x1b[2m — Billing needs a yearly invoice…\x1b[0m \x1b[31m(invalid .meta.yaml)\x1b[0m\n  \x1b[36m•\x1b[0m gamma\x1b[2m\x1b[0m [worktree]\n"
        );
    }

    #[test]
    fn list_human_with_no_changes_and_with_specs_only() {
        let none = ListOutcome { changes: Some(Vec::new()), specs: Some(json!([])) };
        assert_eq!(list_human(&none, &PLAIN).stdout, "No active changes.\n\nNo specs.\n");
        let specs_only = ListOutcome { changes: None, specs: Some(specs()) };
        assert_eq!(list_human(&specs_only, &PLAIN).stdout, "Specs:\n  • auth\n  • billing\n");
    }

    #[test]
    fn list_json_carries_the_sections_that_were_asked_for() {
        let alpha = json!({
            "completedTasks": 1, "name": "alpha", "status": "in-progress",
            "summary": "Sign-in needs a second rule.", "totalTasks": 4,
        });
        let empty = json!({ "completedTasks": 0, "name": "empty", "status": "in-progress", "totalTasks": 0 });
        assert_eq!(list_json(&both()), json!({ "changes": [alpha, empty], "specs": specs() }));
        let changes_only = ListOutcome { changes: Some(vec![change("empty", 0, 0, "")]), specs: None };
        assert_eq!(list_json(&changes_only), json!({ "changes": [empty] }));
        let specs_only = ListOutcome { changes: None, specs: Some(specs()) };
        assert_eq!(list_json(&specs_only), json!({ "specs": specs() }));
    }

    #[test]
    fn list_prints_the_json_form_indented_with_a_final_newline() {
        let changes_only = ListOutcome { changes: Some(Vec::new()), specs: None };
        let out = list(&changes_only, RenderOptions { json: true, color: true });
        assert_eq!(out.stdout, "{\n  \"changes\": []\n}\n");
        assert_eq!(out.json, Some(json!({ "changes": [] })));
        let human = list(&changes_only, RenderOptions::default());
        assert_eq!(human.stdout, "No active changes.\n");
        assert_eq!(human.json, Some(json!({ "changes": [] })));
    }

    // --- show ---

    fn shown_change() -> ShowOutcome {
        ShowOutcome::Change(ShowChange {
            name: "alpha".to_string(),
            schema: Some("spec-driven".to_string()),
            created: Some("2026-07-01T10:00:00+08:00".to_string()),
            proposal: Some("## Why\n\nSign-in needs a second rule.\n".to_string()),
            design: None,
            tasks: Some("- [ ] 1.1 x\n".to_string()),
            delta_capabilities: vec!["auth".to_string()],
            from_discussions: Vec::new(),
            restale_from: Vec::new(),
        })
    }

    #[test]
    fn show_human_prints_a_change_with_its_proposal_and_delta_specs() {
        assert_eq!(
            show_human(&shown_change(), &PLAIN).stdout,
            "Change: alpha\nSchema: spec-driven\nCreated: 2026-07-01T10:00:00+08:00\n\n--- Proposal ---\n## Why\n\nSign-in needs a second rule.\n\n\n--- Delta Specs ---\n  auth/spec.md\n"
        );
    }

    #[test]
    fn show_human_prints_a_change_with_no_sections_and_one_with_delta_specs_only() {
        let bare = |caps: Vec<String>| {
            ShowOutcome::Change(ShowChange {
                name: "empty".to_string(),
                schema: None,
                created: None,
                proposal: None,
                design: None,
                tasks: None,
                delta_capabilities: caps,
                from_discussions: Vec::new(),
                restale_from: Vec::new(),
            })
        };
        assert_eq!(show_human(&bare(Vec::new()), &PLAIN).stdout, "Change: empty\n");
        assert_eq!(
            show_human(&bare(vec!["auth".to_string()]), &COLOR).stdout,
            "\x1b[1mChange\x1b[0m: empty\n\n\x1b[2m--- Delta Specs ---\x1b[0m\n  auth/spec.md\n"
        );
    }

    #[test]
    fn show_human_prints_a_spec_with_one_newline_after_its_content() {
        let spec = ShowOutcome::Spec { name: "auth".to_string(), content: "# auth\n".to_string() };
        assert_eq!(show_human(&spec, &PLAIN).stdout, "Spec: auth\n\n--- spec.md ---\n# auth\n\n");
    }

    #[test]
    fn show_json_carries_the_documents_of_a_change_and_the_file_of_a_spec() {
        assert_eq!(
            show_json(&shown_change()),
            json!({
                "created": "2026-07-01T10:00:00+08:00",
                "deltaSpecs": ["auth/spec.md"],
                "design": null,
                "fromDiscussions": [],
                "name": "alpha",
                "proposal": "## Why\n\nSign-in needs a second rule.\n",
                "restaleFrom": [],
                "schema": "spec-driven",
                "tasks": "- [ ] 1.1 x\n",
            })
        );
        let spec = ShowOutcome::Spec { name: "auth".to_string(), content: "# auth\n".to_string() };
        assert_eq!(
            show_json(&spec),
            json!({ "files": [{ "content": "# auth\n", "name": "spec.md" }], "name": "auth" })
        );
    }

    // --- status ---

    fn report() -> StatusReport {
        let artifact = |id: &str, path: &str, status: &str, blocked_by: &[&str]| ArtifactStatusJson {
            id: id.to_string(),
            output_path: path.to_string(),
            status: status.to_string(),
            blocked_by: blocked_by.iter().map(|s| s.to_string()).collect(),
        };
        StatusReport {
            change_name: "empty".to_string(),
            schema_name: "spec-driven".to_string(),
            is_complete: false,
            apply_requires: vec!["tasks".to_string()],
            artifacts: vec![
                artifact("proposal", "proposal.md", "ready", &[]),
                artifact("design", "design.md", "blocked", &["proposal"]),
            ],
        }
    }

    #[test]
    fn status_human_prints_each_artifact_with_its_state() {
        assert_eq!(
            status_human(&report(), &PLAIN).stdout,
            "Change: empty\nSchema: spec-driven\n\n  ○ proposal (proposal.md)\n  ✗ design (design.md)\n    blocked by: proposal\n\n"
        );
        assert_eq!(
            status_human(&report(), &COLOR).stdout,
            "\x1b[1mChange\x1b[0m: empty\n\x1b[1mSchema\x1b[0m: spec-driven\n\n  \x1b[33m○\x1b[0m \x1b[1mproposal\x1b[0m (\x1b[2mproposal.md\x1b[0m)\n  \x1b[31m✗\x1b[0m \x1b[1mdesign\x1b[0m (\x1b[2mdesign.md\x1b[0m)\n    blocked by: proposal\n\n"
        );
    }

    #[test]
    fn status_human_closes_a_complete_change_with_one_line() {
        let mut done = report();
        done.is_complete = true;
        done.artifacts.truncate(1);
        done.artifacts[0].status = "done".to_string();
        assert_eq!(
            status_human(&done, &PLAIN).stdout,
            "Change: empty\nSchema: spec-driven\n\n  ✓ proposal (proposal.md)\n\n  ✓ All artifacts complete\n"
        );
    }

    #[test]
    fn status_json_is_the_report_and_its_text_keeps_the_field_order() {
        let mut done = report();
        done.artifacts.truncate(1);
        assert_eq!(
            status_json(&done),
            json!({
                "changeName": "empty", "schemaName": "spec-driven", "isComplete": false,
                "applyRequires": ["tasks"],
                "artifacts": [{ "id": "proposal", "outputPath": "proposal.md", "status": "ready" }],
            })
        );
        assert_eq!(
            status(&done, RenderOptions { json: true, color: false }).stdout,
            "{\n  \"changeName\": \"empty\",\n  \"schemaName\": \"spec-driven\",\n  \"isComplete\": false,\n  \"applyRequires\": [\n    \"tasks\"\n  ],\n  \"artifacts\": [\n    {\n      \"id\": \"proposal\",\n      \"outputPath\": \"proposal.md\",\n      \"status\": \"ready\"\n    }\n  ]\n}\n"
        );
    }
}

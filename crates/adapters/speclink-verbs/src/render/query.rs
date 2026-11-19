//! Output of the read verbs: list, show, status.

use serde_json::{json, Value};
use speclink_core::command::{ListOutcome, ShowChange, ShowOutcome};
use speclink_core::status::StatusReport;

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// `list --json`: `{ changes }`, `{ specs }` or both, as the flags asked.
/// With `--detail` the engine filled each item's five detail fields; an item
/// serializes the ones that have a value, so there is nothing to add here.
pub fn list_json(list: &ListOutcome) -> Value {
    // `--specs` alone omits the changes section (`changes` is None).
    let Some(items) = &list.changes else {
        return json!({ "specs": list.specs });
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
        if let Some(specs) = &list.specs {
            specs_section(&mut t, specs, paint);
        }
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
/// With `detail` (`--json --detail`) a stamped station also carries its
/// `scope`; a spec has no stations, so its form is the same either way.
pub fn show_json(show: &ShowOutcome, detail: bool) -> Value {
    match show {
        ShowOutcome::Spec { name, content } => json!({
            "files": [{ "content": content, "name": "spec.md" }],
            "name": name,
        }),
        ShowOutcome::Change(c) => {
            let mut shown = json!({
                "name": c.name,
                "schema": c.schema,
                "created": c.created,
                "proposal": c.proposal,
                "design": c.design,
                "tasks": c.tasks,
                "deltaSpecs": delta_spec_files(c),
                "fromDiscussions": c.from_discussions,
                "restaleFrom": c.restale_from,
            });
            // Optional keys: each is there only when the change has the record.
            // A station with no ticket and no stamp has nothing recorded, so a
            // change without records prints what it printed before these keys.
            for (key, station) in [("review", &c.review), ("verify", &c.verify)] {
                if let Some(station) = station.as_ref().filter(|s| s.is_recorded()) {
                    shown[key] = if detail { station.to_detail_json() } else { station.to_json() };
                }
            }
            if let Some(branch) = &c.branch {
                shown["branch"] = json!(branch);
            }
            if let Some(landed) = &c.landed {
                shown["landed"] = json!({ "commit": landed.commit, "at": landed.at });
                if let Some(by) = &landed.by {
                    shown["landed"]["by"] = json!(by);
                }
            }
            // The record serializes itself, so `list --json --detail` gives
            // the same object.
            if let Some(claimed) = &c.claimed {
                shown["claimed"] = json!(claimed);
            }
            if !c.fields.is_empty() {
                shown["fields"] = json!(c.fields);
            }
            shown
        }
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

/// `show`. `detail` is the `--detail` flag; it changes the JSON form only, and
/// the parse never lets it come without `--json`.
pub fn show(show: &ShowOutcome, detail: bool, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&show_json(show, detail))), |paint| show_human(show, paint))
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

/// `status --json` is the report as the engine serializes it.
pub fn status(report: &StatusReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(report)), |paint| status_human(report, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::command::{ClaimRecord, ShowChange};
    use speclink_core::listing::{ListChangeJson, ListWorktreeJson};
    use speclink_core::station::StationProgress;
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
            created: None,
            modified_at: None,
            fields: Default::default(),
            review: None,
            verify: None,
            claimed: None,
            branch: None,
            landed: None,
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
    fn list_json_carries_the_two_time_fields_and_the_human_form_does_not() {
        let mut timed = change("alpha", 1, 4, "Sign-in needs a second rule.");
        timed.created = Some("2026-11-01".to_string());
        timed.modified_at = Some("2026-11-12T03:00:00Z".to_string());
        let list = ListOutcome { changes: Some(vec![timed, change("empty", 0, 0, "")]), specs: None };
        let value = list_json(&list);
        assert_eq!(value["changes"][0]["created"], "2026-11-01");
        assert_eq!(value["changes"][0]["modifiedAt"], "2026-11-12T03:00:00Z");
        // A change the store cannot date, with no usable metadata, has neither key.
        assert!(value["changes"][1].get("created").is_none(), "{value}");
        assert!(value["changes"][1].get("modifiedAt").is_none(), "{value}");
        let untimed = ListOutcome {
            changes: Some(vec![change("alpha", 1, 4, "Sign-in needs a second rule."), change("empty", 0, 0, "")]),
            specs: None,
        };
        assert_eq!(list_human(&list, &PLAIN), list_human(&untimed, &PLAIN));
        assert_eq!(list_human(&list, &COLOR), list_human(&untimed, &COLOR));
    }

    /// `change` with a record of each of the five detail kinds.
    fn detailed_change() -> ListChangeJson {
        use speclink_core::listing::ListLandedJson;
        use speclink_core::station::StationStatus;
        ListChangeJson {
            review: Some(StationStatus {
                progress: StationProgress::Stamped,
                stamped_at: Some("2026-11-02".to_string()),
                stamped_by: Some("Rev <r@example.com>".to_string()),
                stamped_with: None,
                rounds: Some(3),
                tasks_total: Some(12),
                commit: Some("0123456789ab".to_string()),
                scope: vec![speclink_core::model::ReviewedScopeEntry { path: "src/a.rs".into(), hash: "aa".into() }],
            }),
            verify: Some(StationStatus { progress: StationProgress::InProgress, ..StationStatus::none() }),
            claimed: Some(ClaimRecord {
                by: "Alice".to_string(),
                by_id: Some("u_1".to_string()),
                at: Some("2026-11-18T10:00:00+00:00".to_string()),
            }),
            branch: Some("speclink/demo".to_string()),
            landed: Some(ListLandedJson { commit: "0123456789abcdef0123456789abcdef01234567".to_string() }),
            ..change("demo", 1, 4, "Sign-in needs a second rule.")
        }
    }

    #[test]
    fn list_json_carries_the_five_detail_keys_of_a_change_that_has_the_records() {
        // spec verb-contract「清單的詳細輸出」：鍵名 camelCase，形狀與 show --json 相同，
        // landed 只有 commit。
        let list = ListOutcome { changes: Some(vec![detailed_change(), change("other", 0, 0, "")]), specs: None };
        let json = list_json(&list);
        let demo = &json["changes"][0];
        assert_eq!(
            demo["review"],
            json!({
                "state": "stamped",
                "stampedAt": "2026-11-02",
                "stampedBy": "Rev <r@example.com>",
                "rounds": 3,
                "tasksTotal": 12,
                "commit": "0123456789ab",
            }),
            "no scope and no stampedWith key"
        );
        assert_eq!(demo["verify"], json!({ "state": "inProgress" }));
        assert_eq!(demo["claimed"], json!({ "by": "Alice", "byId": "u_1", "at": "2026-11-18T10:00:00+00:00" }));
        assert_eq!(demo["branch"], json!("speclink/demo"));
        assert_eq!(demo["landed"], json!({ "commit": "0123456789abcdef0123456789abcdef01234567" }));
        // The keys that were there before are unchanged, and a change with no
        // record has none of the five.
        let before = list_json(&ListOutcome {
            changes: Some(vec![change("demo", 1, 4, "Sign-in needs a second rule.")]),
            specs: None,
        });
        for (key, value) in before["changes"][0].as_object().unwrap() {
            assert_eq!(&demo[key], value, "{key}");
        }
        for key in ["review", "verify", "claimed", "branch", "landed"] {
            assert!(json["changes"][1].get(key).is_none(), "{key}: {json}");
        }
    }

    #[test]
    fn list_json_has_the_same_shapes_as_show_json_for_the_same_records() {
        // 「形狀 SHALL 與 show --json 的同名鍵相同」：同一份記錄，兩個動詞的物件相等。
        let shown = show_json(&shown_change_with_records_of(&detailed_change()), false);
        let listed = list_json(&ListOutcome { changes: Some(vec![detailed_change()]), specs: None });
        for key in ["review", "verify", "claimed", "branch"] {
            assert_eq!(listed["changes"][0][key], shown[key], "{key}");
        }
        assert_eq!(listed["changes"][0]["landed"]["commit"], shown["landed"]["commit"]);
    }

    #[test]
    fn list_text_is_unchanged_for_changes_with_no_detail_record() {
        // 不帶 --detail 時五個欄位都沒有值：JSON 的文字與引入它們之前逐位元相同。
        let list = ListOutcome { changes: Some(vec![change("alpha", 1, 4, "Sign-in needs a second rule.")]), specs: None };
        assert_eq!(
            super::list(&list, RenderOptions { json: true, color: false }).stdout,
            "{\n  \"changes\": [\n    {\n      \"completedTasks\": 1,\n      \"name\": \"alpha\",\n      \"status\": \"in-progress\",\n      \"summary\": \"Sign-in needs a second rule.\",\n      \"totalTasks\": 4\n    }\n  ]\n}\n"
        );
        // A station that never started is no record: nothing is printed for it.
        let unstarted = ListChangeJson {
            review: Some(speclink_core::station::StationStatus::none()),
            verify: Some(speclink_core::station::StationStatus::none()),
            ..change("alpha", 1, 4, "Sign-in needs a second rule.")
        };
        assert_eq!(
            list_json(&ListOutcome { changes: Some(vec![unstarted]), specs: None }),
            list_json(&list)
        );
        // The human form never shows the detail.
        let detailed = ListOutcome { changes: Some(vec![detailed_change()]), specs: None };
        let plain = ListOutcome { changes: Some(vec![change("demo", 1, 4, "Sign-in needs a second rule.")]), specs: None };
        assert_eq!(list_human(&detailed, &PLAIN), list_human(&plain, &PLAIN));
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
            review: None,
            verify: None,
            branch: None,
            landed: None,
            claimed: None,
            fields: Default::default(),
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
                review: None,
                verify: None,
                branch: None,
                landed: None,
                claimed: None,
                fields: Default::default(),
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
            show_json(&shown_change(), false),
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
            show_json(&spec, false),
            json!({ "files": [{ "content": "# auth\n", "name": "spec.md" }], "name": "auth" })
        );
    }

    /// `shown_change` with both stations, a branch and a landing record.
    fn shown_change_with_records() -> ShowOutcome {
        use speclink_core::command::LandedRecord;
        use speclink_core::station::StationStatus;
        let ShowOutcome::Change(mut change) = shown_change() else { unreachable!() };
        change.review = Some(StationStatus {
            progress: StationProgress::Stamped,
            stamped_at: Some("2026-11-02".to_string()),
            stamped_by: Some("Rev <r@example.com>".to_string()),
            stamped_with: None,
            rounds: Some(3),
            tasks_total: Some(12),
            commit: Some("0123456789ab".to_string()),
            scope: vec![speclink_core::model::ReviewedScopeEntry { path: "src/a.rs".into(), hash: "aa".into() }],
        });
        change.verify = Some(StationStatus { progress: StationProgress::InProgress, ..StationStatus::none() });
        change.branch = Some("feature/x".to_string());
        change.landed = Some(LandedRecord {
            commit: "89abcdef0123".to_string(),
            at: "2026-11-03T09:00:00+08:00".to_string(),
            by: None,
        });
        ShowOutcome::Change(change)
    }

    /// `shown_change` with the records `item` carries.
    fn shown_change_with_records_of(item: &ListChangeJson) -> ShowOutcome {
        use speclink_core::command::LandedRecord;
        let ShowOutcome::Change(mut change) = shown_change() else { unreachable!() };
        change.review = item.review.clone();
        change.verify = item.verify.clone();
        change.claimed = item.claimed.clone();
        change.branch = item.branch.clone();
        change.landed = item.landed.as_ref().map(|landed| LandedRecord {
            commit: landed.commit.clone(),
            at: "2026-11-03T09:00:00+08:00".to_string(),
            by: None,
        });
        ShowOutcome::Change(change)
    }

    #[test]
    fn show_json_adds_the_stations_the_branch_and_the_landing_when_the_change_has_them() {
        // spec change-landing「分支與落地記錄的讀取」與兩個關卡規格的 show 情境。
        let shown = show_json(&shown_change_with_records(), false);
        assert_eq!(
            shown["review"],
            json!({
                "state": "stamped",
                "stampedAt": "2026-11-02",
                "stampedBy": "Rev <r@example.com>",
                "rounds": 3,
                "tasksTotal": 12,
                "commit": "0123456789ab",
            }),
            "no scope and no stampedWith key"
        );
        assert_eq!(shown["verify"], json!({ "state": "inProgress" }));
        assert_eq!(shown["branch"], json!("feature/x"));
        assert_eq!(shown["landed"], json!({ "commit": "89abcdef0123", "at": "2026-11-03T09:00:00+08:00" }));
        // The keys that were there before are unchanged.
        let before = show_json(&shown_change(), false);
        for (key, value) in before.as_object().unwrap() {
            assert_eq!(&shown[key], value, "{key}");
        }
    }

    // --- free fields (host-free-fields D5) ---

    fn fields_of(pairs: &[(&str, &str)]) -> speclink_core::fields::Fields {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn list_json_carries_the_fields_of_the_changes_that_have_some_and_the_human_form_does_not() {
        // 規格「清單帶出欄位」「人眼輸出不變」：demo 有兩個欄位，other 沒有。
        let with = ListChangeJson { fields: fields_of(&[("ticket", "PROJ-1"), ("source", "portal")]), ..change("demo", 0, 0, "") };
        let listed = |items| ListOutcome { changes: Some(items), specs: None };
        let json = list_json(&listed(vec![with, change("other", 0, 0, "")]));
        assert_eq!(json["changes"][0]["fields"], json!({ "source": "portal", "ticket": "PROJ-1" }));
        assert!(json["changes"][1].get("fields").is_none(), "{json}");
        let with = ListChangeJson { fields: fields_of(&[("ticket", "PROJ-1")]), ..change("demo", 0, 0, "") };
        assert_eq!(
            list_human(&listed(vec![with]), &PLAIN).stdout,
            list_human(&listed(vec![change("demo", 0, 0, "")]), &PLAIN).stdout
        );
    }

    #[test]
    fn show_json_carries_the_fields_only_when_the_change_has_some_and_the_human_form_does_not() {
        let ShowOutcome::Change(plain) = shown_change() else { unreachable!() };
        let with = ShowOutcome::Change(ShowChange { fields: fields_of(&[("ticket", "PROJ-1"), ("n", "123")]), ..plain });
        let shown = show_json(&with, false);
        assert_eq!(shown["fields"], json!({ "n": "123", "ticket": "PROJ-1" }));
        assert!(shown["fields"]["n"].is_string(), "a value is always a string");
        // 沒有欄位的變更：沒有這個鍵，其餘和之前一樣。
        let before = show_json(&shown_change(), false);
        assert!(before.get("fields").is_none());
        let mut without = shown.clone();
        without.as_object_mut().unwrap().remove("fields");
        assert_eq!(without, before);
        assert_eq!(show_human(&with, &PLAIN).stdout, show_human(&shown_change(), &PLAIN).stdout);
    }

    #[test]
    fn show_json_adds_the_claim_record_and_the_human_form_does_not() {
        // spec「認領記錄的讀取」：by，以及有記錄才出現的 byId 與 at。
        let with = |claimed: ClaimRecord| {
            let ShowOutcome::Change(plain) = shown_change() else { unreachable!() };
            ShowOutcome::Change(ShowChange { claimed: Some(claimed), ..plain })
        };
        let full = with(ClaimRecord {
            by: "Alice".to_string(),
            by_id: Some("u_1".to_string()),
            at: Some("2026-11-18T10:00:00+00:00".to_string()),
        });
        let shown = show_json(&full, false);
        assert_eq!(shown["claimed"], json!({ "by": "Alice", "byId": "u_1", "at": "2026-11-18T10:00:00+00:00" }));
        // 其餘欄位與沒有認領時相同；人眼輸出不變。
        let before = show_json(&shown_change(), false);
        let mut without = shown.clone();
        without.as_object_mut().unwrap().remove("claimed");
        assert_eq!(without, before);
        assert_eq!(show_human(&full, &PLAIN).stdout, show_human(&shown_change(), &PLAIN).stdout);
        assert_eq!(show(&shown_change(), false, RenderOptions { json: true, color: false }).stdout.contains("claimed"), false);

        // 沒有識別的認領不輸出 byId；舊版 Server 只給得出 by。
        let unkeyed = with(ClaimRecord {
            by: "Alice <a@example.com>".to_string(),
            by_id: None,
            at: Some("2026-11-18T10:00:00+00:00".to_string()),
        });
        assert_eq!(
            show_json(&unkeyed, false)["claimed"],
            json!({ "by": "Alice <a@example.com>", "at": "2026-11-18T10:00:00+00:00" })
        );
        let by_only = with(ClaimRecord { by: "Alice".to_string(), by_id: None, at: None });
        assert_eq!(show_json(&by_only, false)["claimed"], json!({ "by": "Alice" }));
    }

    #[test]
    fn show_json_has_no_key_for_a_record_the_change_does_not_have() {
        let shown = show_json(&shown_change(), false);
        for key in ["review", "verify", "branch", "landed", "claimed"] {
            assert!(shown.get(key).is_none(), "{key}");
        }
        // A station that never started is no record either: the output is the
        // one from before these keys.
        let ShowOutcome::Change(mut change) = shown_change() else { unreachable!() };
        change.review = Some(speclink_core::station::StationStatus::none());
        change.verify = Some(speclink_core::station::StationStatus::none());
        let change = ShowOutcome::Change(change);
        assert_eq!(show_json(&change, false), shown);
        // verb-contract「各種關卡記錄對應的輸出」的第一列：沒有記錄的關卡帶 --detail 也不輸出。
        assert_eq!(show_json(&change, true), shown);
    }

    #[test]
    fn show_json_with_the_detail_adds_the_scope_of_a_stamped_station_only() {
        // spec verb-contract「單筆變更的詳細輸出」：已蓋章的 review 帶 scope，
        // inProgress 的 verify 不帶；其他鍵與不帶詳細時相同。
        let shown = shown_change_with_records();
        let detailed = show_json(&shown, true);
        assert_eq!(detailed["review"]["scope"], json!([{ "path": "src/a.rs", "hash": "aa" }]));
        assert_eq!(detailed["verify"], json!({ "state": "inProgress" }));
        let mut without_scope = detailed.clone();
        without_scope["review"].as_object_mut().unwrap().remove("scope");
        assert_eq!(without_scope, show_json(&shown, false));
    }

    #[test]
    fn show_without_the_detail_prints_the_bytes_it_printed_before() {
        // 回歸對照：不帶詳細的 show --json 逐位元不變（這段文字取自加入 --detail 之前的輸出）。
        let before = "{\n  \"branch\": \"feature/x\",\n  \"created\": \"2026-07-01T10:00:00+08:00\",\n  \"deltaSpecs\": [\n    \"auth/spec.md\"\n  ],\n  \"design\": null,\n  \"fromDiscussions\": [],\n  \"landed\": {\n    \"at\": \"2026-11-03T09:00:00+08:00\",\n    \"commit\": \"89abcdef0123\"\n  },\n  \"name\": \"alpha\",\n  \"proposal\": \"## Why\\n\\nSign-in needs a second rule.\\n\",\n  \"restaleFrom\": [],\n  \"review\": {\n    \"commit\": \"0123456789ab\",\n    \"rounds\": 3,\n    \"stampedAt\": \"2026-11-02\",\n    \"stampedBy\": \"Rev <r@example.com>\",\n    \"state\": \"stamped\",\n    \"tasksTotal\": 12\n  },\n  \"schema\": \"spec-driven\",\n  \"tasks\": \"- [ ] 1.1 x\\n\",\n  \"verify\": {\n    \"state\": \"inProgress\"\n  }\n}\n";
        let options = RenderOptions { json: true, color: false };
        assert_eq!(show(&shown_change_with_records(), false, options).stdout, before);
        assert!(show(&shown_change_with_records(), true, options).stdout.contains("\"scope\""));
    }

    #[test]
    fn show_of_a_spec_is_the_same_with_and_without_the_detail() {
        let spec = ShowOutcome::Spec { name: "auth".to_string(), content: "# auth\n".to_string() };
        assert_eq!(show_json(&spec, true), show_json(&spec, false));
        let options = RenderOptions { json: true, color: false };
        assert_eq!(show(&spec, true, options), show(&spec, false, options));
    }

    #[test]
    fn show_human_is_the_same_with_and_without_the_records() {
        for paint in [&PLAIN, &COLOR] {
            assert_eq!(
                show_human(&shown_change_with_records(), paint).stdout,
                show_human(&shown_change(), paint).stdout
            );
        }
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
            status(&done, RenderOptions::default()).json.unwrap(),
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

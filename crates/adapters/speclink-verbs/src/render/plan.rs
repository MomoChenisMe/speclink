//! Output of the execution-order verbs: `plan`, `change depends`,
//! `change schema` and `change rank`.

use serde_json::{json, Value};
use speclink_core::command::{DependsOutcome, PlanReport, SchemaOutcome};
use speclink_core::plan::Side;

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// `plan`: one `Wave N` heading per wave (`(parallel)` when it holds more
/// than one change), one line per change with its stage and up to three notes
/// in a fixed order — `blocked by:`, `archive after:`, `conflicts with:`,
/// each left out when empty — then the skipped changes, the unpromoted cuts,
/// and `next:` last.
pub fn plan_human(plan: &PlanReport, paint: &Paint) -> Text {
    let mut t = Text::default();
    for wave in &plan.waves {
        let title = if wave.changes.len() > 1 {
            format!("Wave {} (parallel)", wave.index)
        } else {
            format!("Wave {}", wave.index)
        };
        outln!(t, "{}", paint.bold(&title));
        for name in &wave.changes {
            // Every wave member is a plan change.
            let Some(c) = plan.changes.iter().find(|c| &c.name == name) else {
                continue;
            };
            let mut conflicts: Vec<&str> = Vec::new();
            for o in c.requirement_overlap.iter().filter(|o| o.conflict) {
                if !conflicts.contains(&o.change.as_str()) {
                    conflicts.push(&o.change);
                }
            }
            let mut notes = String::new();
            for (label, names) in [
                ("blocked by", c.blocked_by.iter().map(String::as_str).collect()),
                ("archive after", c.archive_after.iter().map(String::as_str).collect()),
                ("conflicts with", conflicts),
            ] {
                if !names.is_empty() {
                    notes.push_str(&format!(" — {label}: {}", names.join(", ")));
                }
            }
            if !notes.is_empty() {
                notes = paint.dim(&notes);
            }
            outln!(t, "  {} {} [{}]{notes}", paint.cyan("•"), c.name, c.stage.as_str());
        }
    }
    if !plan.skipped.is_empty() {
        outln!(t, "{}", paint.bold("Skipped"));
        for s in &plan.skipped {
            outln!(t, "  {} {} {}", paint.cyan("•"), s.change, paint.red("(invalid .meta.yaml)"));
        }
    }
    if !plan.unpromoted_cuts.is_empty() {
        outln!(t, "{}", paint.bold("Unpromoted cuts"));
        for cut in &plan.unpromoted_cuts {
            let title = if cut.title.is_empty() {
                String::new()
            } else {
                paint.dim(&format!(" — {}", cut.title))
            };
            outln!(
                t,
                "  {} {} {}: {}{title}",
                paint.cyan("•"),
                cut.discussion,
                cut.cut,
                cut.unpromoted.join(", ")
            );
        }
    }
    match &plan.next {
        Some(next) => outln!(t, "next: {next}"),
        None => outln!(t, "next: none"),
    }
    t
}

/// `plan --json` is the report as the engine serializes it.
pub fn plan(plan: &PlanReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(plan)), |paint| plan_human(plan, paint))
}

/// What `change depends` reports: the declaration after the write, and the
/// names the caller gave.
#[derive(Debug, Clone, Copy)]
pub struct Depends<'a> {
    pub change: &'a str,
    /// The full declaration after the write.
    pub depends_on: &'a [String],
    /// The `--on` names of this call.
    pub on: &'a [String],
    pub remove: bool,
}

/// `change depends --json`: the declaration after the write.
pub fn depends_json(d: &Depends<'_>) -> Value {
    json!({
        "change": d.change,
        "dependsOn": d.depends_on,
    })
}

/// `change depends`: the full declaration after an add, the dropped names
/// after a remove.
pub fn depends_human(d: &Depends<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    if d.remove {
        outln!(t, "{} {} no longer depends on: {}", paint.green("✓"), d.change, d.on.join(", "));
    } else {
        outln!(t, "{} {} depends on: {}", paint.green("✓"), d.change, d.depends_on.join(", "));
    }
    t
}

pub fn depends(d: &Depends<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&depends_json(d))), |paint| depends_human(d, paint))
}

/// `change depends`, from the engine's result and the `--on` names and
/// `--remove` flag of the call.
pub fn depends_declared(
    o: &DependsOutcome,
    on: &[String],
    remove: bool,
    options: RenderOptions,
) -> Rendered {
    depends(&Depends { change: &o.change, depends_on: &o.depends_on, on, remove }, options)
}

/// `change schema --json`: the four facts (the stamp-cleared line is human
/// output only).
pub fn schema_json(o: &SchemaOutcome) -> Value {
    json!({
        "change": o.change,
        "schema": o.schema,
        "previous": o.previous,
        "changed": o.changed,
    })
}

/// `change schema`: one success line, and a second when the switch removed
/// the verify stamp.
pub fn schema_human(o: &SchemaOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    if o.changed {
        outln!(t, "{} {} schema: {} → {}", paint.green("✓"), o.change, o.previous, o.schema);
    } else {
        outln!(t, "{} {} schema: {} (unchanged)", paint.green("✓"), o.change, o.schema);
    }
    if o.verify_stamp_cleared {
        outln!(t, "  verify stamp cleared — re-run verify under {}", o.schema);
    }
    t
}

pub fn schema(o: &SchemaOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&schema_json(o))), |paint| schema_human(o, paint))
}

/// What `change rank` reports. The rank value itself is never shown.
#[derive(Debug, Clone, Copy)]
pub struct Ranked<'a> {
    pub change: &'a str,
    pub anchor: &'a str,
    pub side: Side,
}

/// `change rank --json`.
pub fn rank_json(r: &Ranked<'_>) -> Value {
    json!({
        "change": r.change,
        "anchor": r.anchor,
        "position": r.side.as_str(),
    })
}

/// `change rank`: one success line naming the side it landed on.
pub fn rank_human(r: &Ranked<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} {} ranked {} {}", paint.green("✓"), r.change, r.side.as_str(), r.anchor);
    t
}

pub fn rank(r: &Ranked<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&rank_json(r))), |paint| rank_human(r, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::model::Stage;
    use speclink_core::plan::{DeltaOp, PlanChange, RequirementOverlap, Skipped, UnpromotedCut, Wave};

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);
    const JSON: RenderOptions = RenderOptions { json: true, color: false };

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn placed(name: &str, wave: usize, stage: Stage) -> PlanChange {
        PlanChange {
            name: name.to_string(),
            wave,
            stage,
            depends_on: Vec::new(),
            overlaps: Vec::new(),
            blocked_by: Vec::new(),
            ready: true,
            requirement_overlap: Vec::new(),
            archive_after: Vec::new(),
        }
    }

    fn report() -> PlanReport {
        let overlap = |change: &str, conflict: bool| RequirementOverlap {
            change: change.to_string(),
            capability: "auth".to_string(),
            requirement: "Password sign-in".to_string(),
            own_operation: DeltaOp::Modified,
            other_operation: DeltaOp::Modified,
            conflict,
        };
        let mut beta = placed("beta", 2, Stage::Proposed);
        beta.blocked_by = names(&["alpha"]);
        beta.archive_after = names(&["ready"]);
        beta.ready = false;
        // Two conflicts with one change read as one name; a non-conflict is not listed.
        beta.requirement_overlap = vec![overlap("alpha", true), overlap("alpha", true), overlap("ready", false)];
        PlanReport {
            waves: vec![
                Wave { index: 1, changes: names(&["ready", "alpha"]) },
                Wave { index: 2, changes: names(&["beta"]) },
            ],
            changes: vec![placed("ready", 1, Stage::Ready), placed("alpha", 1, Stage::InProgress), beta],
            next: Some("beta".to_string()),
            skipped: vec![Skipped { change: "broken".to_string(), reason: "bad yaml".to_string() }],
            unpromoted_cuts: vec![
                UnpromotedCut {
                    discussion: "topic-held".to_string(),
                    cut: "cut 1".to_string(),
                    title: "the first part".to_string(),
                    changes: names(&["beta"]),
                    unpromoted: names(&["beta"]),
                },
                UnpromotedCut {
                    discussion: "topic-held".to_string(),
                    cut: "cut 2".to_string(),
                    title: String::new(),
                    changes: names(&["later-part", "last-part"]),
                    unpromoted: names(&["later-part", "last-part"]),
                },
            ],
        }
    }

    #[test]
    fn plan_human_prints_the_waves_the_notes_and_the_closing_sections() {
        assert_eq!(
            plan_human(&report(), &PLAIN).stdout,
            "Wave 1 (parallel)\n  • ready [ready]\n  • alpha [in-progress]\nWave 2\n  • beta [proposed] — blocked by: alpha — archive after: ready — conflicts with: alpha\nSkipped\n  • broken (invalid .meta.yaml)\nUnpromoted cuts\n  • topic-held cut 1: beta — the first part\n  • topic-held cut 2: later-part, last-part\nnext: beta\n"
        );
    }

    #[test]
    fn plan_human_colors_the_headings_the_bullets_and_the_notes() {
        assert_eq!(
            plan_human(&report(), &COLOR).stdout,
            "\x1b[1mWave 1 (parallel)\x1b[0m\n  \x1b[36m•\x1b[0m ready [ready]\n  \x1b[36m•\x1b[0m alpha [in-progress]\n\x1b[1mWave 2\x1b[0m\n  \x1b[36m•\x1b[0m beta [proposed]\x1b[2m — blocked by: alpha — archive after: ready — conflicts with: alpha\x1b[0m\n\x1b[1mSkipped\x1b[0m\n  \x1b[36m•\x1b[0m broken \x1b[31m(invalid .meta.yaml)\x1b[0m\n\x1b[1mUnpromoted cuts\x1b[0m\n  \x1b[36m•\x1b[0m topic-held cut 1: beta\x1b[2m — the first part\x1b[0m\n  \x1b[36m•\x1b[0m topic-held cut 2: later-part, last-part\nnext: beta\n"
        );
    }

    #[test]
    fn plan_human_with_nothing_placed_is_the_next_line_only() {
        let empty = PlanReport {
            waves: Vec::new(),
            changes: Vec::new(),
            next: None,
            skipped: Vec::new(),
            unpromoted_cuts: Vec::new(),
        };
        assert_eq!(plan_human(&empty, &COLOR).stdout, "next: none\n");
        assert_eq!(
            plan(&empty, JSON).stdout,
            "{\n  \"waves\": [],\n  \"changes\": [],\n  \"next\": null,\n  \"skipped\": [],\n  \"unpromotedCuts\": []\n}\n"
        );
    }

    #[test]
    fn plan_json_is_the_report() {
        let value = plan(&report(), RenderOptions::default()).json.unwrap();
        assert_eq!(value["waves"][0], json!({ "index": 1, "changes": ["ready", "alpha"] }));
        assert_eq!(value["changes"][2]["blockedBy"], json!(["alpha"]));
        assert_eq!(value["changes"][2]["stage"], "proposed");
        assert_eq!(value["next"], "beta");
        assert_eq!(value["skipped"], json!([{ "change": "broken", "reason": "bad yaml" }]));
        assert_eq!(value["unpromotedCuts"][0]["cut"], "cut 1");
    }

    #[test]
    fn depends_human_shows_the_declaration_after_an_add_and_the_dropped_names_after_a_remove() {
        let depends_on = names(&["alpha", "ready", "done-one"]);
        let on = names(&["ready", "done-one"]);
        let added = Depends { change: "beta", depends_on: &depends_on, on: &on, remove: false };
        assert_eq!(
            depends_human(&added, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m beta depends on: alpha, ready, done-one\n"
        );
        let left = names(&["ready"]);
        let dropped = names(&["alpha"]);
        let removed = Depends { change: "beta", depends_on: &left, on: &dropped, remove: true };
        assert_eq!(depends_human(&removed, &PLAIN).stdout, "✓ beta no longer depends on: alpha\n");
        assert_eq!(depends_json(&removed), json!({ "change": "beta", "dependsOn": ["ready"] }));
        assert_eq!(
            depends(&added, JSON).stdout,
            "{\n  \"change\": \"beta\",\n  \"dependsOn\": [\n    \"alpha\",\n    \"ready\",\n    \"done-one\"\n  ]\n}\n"
        );
    }

    fn switched(changed: bool, cleared: bool) -> SchemaOutcome {
        SchemaOutcome {
            change: "beta".to_string(),
            schema: if changed { "lite".to_string() } else { "spec-driven".to_string() },
            previous: "spec-driven".to_string(),
            changed,
            verify_stamp_cleared: cleared,
        }
    }

    #[test]
    fn schema_human_tells_a_switch_from_no_change_and_reports_a_cleared_stamp() {
        assert_eq!(
            schema_human(&switched(false, false), &COLOR).stdout,
            "\x1b[32m✓\x1b[0m beta schema: spec-driven (unchanged)\n"
        );
        assert_eq!(
            schema_human(&switched(true, true), &PLAIN).stdout,
            "✓ beta schema: spec-driven → lite\n  verify stamp cleared — re-run verify under lite\n"
        );
    }

    #[test]
    fn schema_json_carries_the_four_facts() {
        assert_eq!(
            schema_json(&switched(true, true)),
            json!({ "change": "beta", "schema": "lite", "previous": "spec-driven", "changed": true })
        );
        assert_eq!(
            schema(&switched(false, false), JSON).stdout,
            "{\n  \"change\": \"beta\",\n  \"changed\": false,\n  \"previous\": \"spec-driven\",\n  \"schema\": \"spec-driven\"\n}\n"
        );
    }

    #[test]
    fn rank_human_and_json_name_the_side() {
        let before = Ranked { change: "beta", anchor: "empty", side: Side::Before };
        assert_eq!(rank_human(&before, &COLOR).stdout, "\x1b[32m✓\x1b[0m beta ranked before empty\n");
        let after = Ranked { change: "beta", anchor: "broken", side: Side::After };
        assert_eq!(rank_human(&after, &PLAIN).stdout, "✓ beta ranked after broken\n");
        assert_eq!(rank_json(&after), json!({ "change": "beta", "anchor": "broken", "position": "after" }));
        assert_eq!(
            rank(&before, JSON).stdout,
            "{\n  \"anchor\": \"empty\",\n  \"change\": \"beta\",\n  \"position\": \"before\"\n}\n"
        );
    }
}

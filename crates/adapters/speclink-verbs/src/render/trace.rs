//! Output of `trace`: one capability's provenance chain.

use serde_json::Value;
use speclink_core::trace::TraceReport;

use super::{value_of, JsonForm, Paint, RenderOptions, Rendered, Text};

/// `trace --json`: the report as the engine serializes it.
pub fn trace_json(report: &TraceReport) -> Value {
    value_of(report)
}

/// `trace`: the archived changes, the discussions, then the requirements.
pub fn trace_human(r: &TraceReport, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "Capability: {}", paint.bold(&r.capability));

    outln!(t);
    outln!(t, "Changes (oldest first):");
    if r.changes.is_empty() {
        outln!(t, "  {}", paint.dim("(none)"));
    }
    for c in &r.changes {
        outln!(t, "  {} ({})", paint.bold(&c.name), paint.dim(&c.archived_dir));
        match &c.from_discussion {
            Some(slug) => outln!(t, "    {} {}", paint.dim("discussion:"), slug),
            None => outln!(t, "    {} {}", paint.dim("discussion:"), paint.dim("(none)")),
        }
        match &c.evidence {
            // A record with zero tasks is shown as empty, apart from "no record".
            Some(tasks) if tasks.is_empty() => {
                outln!(t, "    {} {}", paint.dim("evidence:"), paint.dim("(empty)"))
            }
            Some(tasks) => {
                outln!(t, "    {}", paint.dim("evidence:"));
                for task in tasks {
                    outln!(t, "      {}: {}", task.task_id, task.files.join(", "));
                }
            }
            None => outln!(t, "    {} {}", paint.dim("evidence:"), paint.dim("(no record)")),
        }
    }

    outln!(t);
    outln!(t, "Discussions:");
    if r.discussions.is_empty() {
        outln!(t, "  {}", paint.dim("(none)"));
    }
    for d in &r.discussions {
        let status = if d.archived { "(archived)" } else { "(live)" };
        outln!(t, "  {} {}", paint.bold(&d.slug), paint.dim(status));
        for p in &d.promoted_to {
            let caps =
                if p.capabilities.is_empty() { paint.dim("(none)") } else { p.capabilities.join(", ") };
            outln!(t, "    -> {}: {caps}", p.change);
        }
    }

    outln!(t);
    outln!(t, "Requirements:");
    if r.requirements.is_empty() {
        outln!(t, "  {}", paint.dim("(none)"));
    }
    for req in &r.requirements {
        outln!(t, "  {} <- {}", paint.bold(&req.name), req.source);
    }
    t
}

pub fn trace(report: &TraceReport, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(report)), |paint| trace_human(report, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::trace::{
        ChangeTrace, DiscussionTrace, EvidenceTask, PromotedChange, RequirementTrace,
    };

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    fn change(name: &str, discussion: Option<&str>, evidence: Option<Vec<EvidenceTask>>) -> ChangeTrace {
        ChangeTrace {
            name: name.to_string(),
            archived_dir: format!("2026-11-11-{name}"),
            from_discussion: discussion.map(str::to_string),
            evidence,
        }
    }

    fn auth() -> TraceReport {
        TraceReport {
            capability: "auth".to_string(),
            requirements: vec![RequirementTrace { name: "Session".to_string(), source: "gamma".to_string() }],
            changes: vec![change(
                "gamma",
                Some("topic-promoted"),
                Some(vec![EvidenceTask {
                    task_id: "tsk_01M72VMQXQS7PB683AFTT6RM7W".to_string(),
                    files: vec!["src/session.rs".to_string()],
                }]),
            )],
            discussions: vec![DiscussionTrace {
                slug: "topic-promoted".to_string(),
                archived: true,
                promoted_to: vec![PromotedChange {
                    change: "gamma".to_string(),
                    capabilities: vec!["auth".to_string()],
                }],
            }],
        }
    }

    #[test]
    fn trace_human_prints_the_three_sections() {
        assert_eq!(
            trace_human(&auth(), &PLAIN).stdout,
            "Capability: auth\n\nChanges (oldest first):\n  gamma (2026-11-11-gamma)\n    discussion: topic-promoted\n    evidence:\n      tsk_01M72VMQXQS7PB683AFTT6RM7W: src/session.rs\n\nDiscussions:\n  topic-promoted (archived)\n    -> gamma: auth\n\nRequirements:\n  Session <- gamma\n"
        );
        assert_eq!(
            trace_human(&auth(), &COLOR).stdout,
            "Capability: \x1b[1mauth\x1b[0m\n\nChanges (oldest first):\n  \x1b[1mgamma\x1b[0m (\x1b[2m2026-11-11-gamma\x1b[0m)\n    \x1b[2mdiscussion:\x1b[0m topic-promoted\n    \x1b[2mevidence:\x1b[0m\n      tsk_01M72VMQXQS7PB683AFTT6RM7W: src/session.rs\n\nDiscussions:\n  \x1b[1mtopic-promoted\x1b[0m \x1b[2m(archived)\x1b[0m\n    -> gamma: auth\n\nRequirements:\n  \x1b[1mSession\x1b[0m <- gamma\n"
        );
    }

    #[test]
    fn trace_human_names_each_absent_piece() {
        let empty = TraceReport {
            capability: "billing".to_string(),
            requirements: Vec::new(),
            changes: Vec::new(),
            discussions: Vec::new(),
        };
        assert_eq!(
            trace_human(&empty, &PLAIN).stdout,
            "Capability: billing\n\nChanges (oldest first):\n  (none)\n\nDiscussions:\n  (none)\n\nRequirements:\n  (none)\n"
        );
        let sparse = TraceReport {
            capability: "billing".to_string(),
            requirements: Vec::new(),
            changes: vec![change("one", None, None), change("two", None, Some(Vec::new()))],
            discussions: vec![DiscussionTrace {
                slug: "topic-live".to_string(),
                archived: false,
                promoted_to: vec![PromotedChange { change: "one".to_string(), capabilities: Vec::new() }],
            }],
        };
        assert_eq!(
            trace_human(&sparse, &PLAIN).stdout,
            "Capability: billing\n\nChanges (oldest first):\n  one (2026-11-11-one)\n    discussion: (none)\n    evidence: (no record)\n  two (2026-11-11-two)\n    discussion: (none)\n    evidence: (empty)\n\nDiscussions:\n  topic-live (live)\n    -> one: (none)\n\nRequirements:\n  (none)\n"
        );
    }

    #[test]
    fn trace_json_is_the_report_and_its_text_keeps_the_field_order() {
        assert_eq!(
            trace_json(&auth()),
            json!({
                "capability": "auth",
                "requirements": [{ "name": "Session", "source": "gamma" }],
                "changes": [{
                    "name": "gamma",
                    "archivedDir": "2026-11-11-gamma",
                    "fromDiscussion": "topic-promoted",
                    "evidence": [{ "taskId": "tsk_01M72VMQXQS7PB683AFTT6RM7W", "files": ["src/session.rs"] }],
                }],
                "discussions": [{
                    "slug": "topic-promoted",
                    "archived": true,
                    "promotedTo": [{ "change": "gamma", "capabilities": ["auth"] }],
                }],
            })
        );
        let text = trace(&auth(), RenderOptions { json: true, color: false }).stdout;
        assert!(text.starts_with("{\n  \"capability\": \"auth\",\n  \"requirements\": [\n"), "{text}");
        assert!(text.ends_with("      ]\n    }\n  ]\n}\n"), "{text}");
    }
}

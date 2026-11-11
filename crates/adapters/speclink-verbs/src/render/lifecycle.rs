//! Output of the change lifecycle verbs: archive, discard, claim.

use serde_json::{json, Value};
use speclink_core::archive::ArchiveOutcome;

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// `archive` of one change: the result lines on stdout, and a one-line note
/// on stderr when the change has no task evidence.
pub fn archive_human(outcome: &ArchiveOutcome, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Archived: {} → {}", paint.green("✓"), outcome.change_name, outcome.dated_name);
    if !outcome.caps.is_empty() {
        let names: Vec<&str> = outcome.caps.iter().map(|c| c.capability.as_str()).collect();
        let added: usize = outcome.caps.iter().map(|c| c.added).sum();
        let modified: usize = outcome.caps.iter().map(|c| c.modified).sum();
        let removed: usize = outcome.caps.iter().map(|c| c.removed).sum();
        let renamed: usize = outcome.caps.iter().map(|c| c.renamed).sum();
        outln!(
            t,
            "Specs applied: {} (added: {added}, modified: {modified}, removed: {removed}, renamed: {renamed})",
            names.join(", ")
        );
    }
    if outcome.snapshot_created {
        outln!(t, "Snapshot created for unarchive support.");
    }
    for (slug, file) in &outcome.archived_discussions {
        outln!(t, "Discussion archived: {slug} → discussions/archive/{file}");
    }
    // The zero-evidence note: one line, never blocks, never changes the exit
    // code. A spec-only change earns no evidence; the note lets an agent check
    // whether apply was skipped by mistake.
    if !outcome.evidence_recorded {
        errln!(
            t,
            "note: no task evidence recorded for change '{}' — fine for spec-only changes; otherwise check that tasks went through apply",
            outcome.change_name
        );
    }
    t
}

/// `archive` has no JSON form.
pub fn archive(outcome: &ArchiveOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |paint| archive_human(outcome, paint))
}

/// `archive` against a server too old to send the full result: the change
/// and the capabilities it updated.
pub fn archive_summary_human(change: &str, capabilities: &[&str], paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Archived change: {change}", paint.green("✓"));
    if !capabilities.is_empty() {
        outln!(t, "  Specs updated: {}", capabilities.join(", "));
    }
    t
}

pub fn archive_summary(change: &str, capabilities: &[&str], options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |paint| archive_summary_human(change, capabilities, paint))
}

/// `discard --json`: the change and the discussions it was unlinked from,
/// each `(slug, status after the unlink)`.
pub fn discard_json(change: &str, unlinked: &[(String, String)]) -> Value {
    let discussions: Vec<Value> =
        unlinked.iter().map(|(slug, status)| json!({ "slug": slug, "status": status })).collect();
    json!({
        "change": change,
        "unlinkedDiscussions": discussions,
    })
}

/// `discard`: the success line, then one line per unlinked discussion.
pub fn discard_human(change: &str, unlinked: &[(String, String)], paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Discarded change: {change}", paint.green("✓"));
    for (slug, status) in unlinked {
        outln!(t, "  Discussion unlinked: {slug} → {status}");
    }
    t
}

pub fn discard(change: &str, unlinked: &[(String, String)], options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&discard_json(change, unlinked))), |paint| {
        discard_human(change, unlinked, paint)
    })
}

/// `claim`: one success line.
pub fn claim_human(name: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Claimed change: {name}", paint.green("✓"));
    t
}

/// `claim` has no JSON form.
pub fn claim(name: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, None, |paint| claim_human(name, paint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::archive::CapCounts;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);

    fn archived() -> ArchiveOutcome {
        ArchiveOutcome {
            change_name: "done-one".to_string(),
            dated_name: "2026-11-11-done-one".to_string(),
            caps: vec![
                CapCounts { capability: "billing".to_string(), added: 1, modified: 0, removed: 0, renamed: 0 },
                CapCounts { capability: "auth".to_string(), added: 0, modified: 2, removed: 1, renamed: 1 },
            ],
            snapshot_created: true,
            skipped_specs: false,
            archived_discussions: vec![("topic-promoted".to_string(), "2026-11-11-topic-promoted.md".to_string())],
            evidence_recorded: false,
        }
    }

    #[test]
    fn archive_human_prints_the_result_and_notes_missing_evidence_on_stderr() {
        let text = archive_human(&archived(), &PLAIN);
        assert_eq!(
            text.stdout,
            "✓ Archived: done-one → 2026-11-11-done-one\nSpecs applied: billing, auth (added: 1, modified: 2, removed: 1, renamed: 1)\nSnapshot created for unarchive support.\nDiscussion archived: topic-promoted → discussions/archive/2026-11-11-topic-promoted.md\n"
        );
        assert_eq!(
            text.stderr,
            "note: no task evidence recorded for change 'done-one' — fine for spec-only changes; otherwise check that tasks went through apply\n"
        );
    }

    #[test]
    fn archive_human_with_evidence_and_no_spec_change_is_one_line() {
        let mut bare = archived();
        bare.caps.clear();
        bare.snapshot_created = false;
        bare.archived_discussions.clear();
        bare.evidence_recorded = true;
        let text = archive_human(&bare, &COLOR);
        assert_eq!(text.stdout, "\x1b[32m✓\x1b[0m Archived: done-one → 2026-11-11-done-one\n");
        assert_eq!(text.stderr, "");
        let out = archive(&bare, RenderOptions { json: true, color: false });
        assert_eq!(out.stdout, "✓ Archived: done-one → 2026-11-11-done-one\n");
        assert_eq!(out.json, None);
    }

    #[test]
    fn archive_summary_human_lists_the_updated_capabilities() {
        assert_eq!(
            archive_summary_human("done-one", &["billing", "auth"], &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Archived change: done-one\n  Specs updated: billing, auth\n"
        );
        assert_eq!(archive_summary_human("done-one", &[], &PLAIN).stdout, "✓ Archived change: done-one\n");
    }

    #[test]
    fn discard_human_and_json_list_the_unlinked_discussions() {
        let unlinked = vec![("topic-held".to_string(), "concluded".to_string())];
        assert_eq!(
            discard_human("beta", &unlinked, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Discarded change: beta\n  Discussion unlinked: topic-held → concluded\n"
        );
        assert_eq!(
            discard_json("beta", &unlinked),
            json!({ "change": "beta", "unlinkedDiscussions": [{ "slug": "topic-held", "status": "concluded" }] })
        );
        assert_eq!(
            discard("alpha", &[], RenderOptions { json: true, color: false }).stdout,
            "{\n  \"change\": \"alpha\",\n  \"unlinkedDiscussions\": []\n}\n"
        );
        assert_eq!(discard("alpha", &[], RenderOptions::default()).stdout, "✓ Discarded change: alpha\n");
    }

    #[test]
    fn claim_human_is_one_line_and_has_no_json_form() {
        assert_eq!(claim_human("alpha", &COLOR).stdout, "\x1b[32m✓\x1b[0m Claimed change: alpha\n");
        let out = claim("alpha", RenderOptions { json: true, color: false });
        assert_eq!(out.stdout, "✓ Claimed change: alpha\n");
        assert_eq!(out.json, None);
    }
}

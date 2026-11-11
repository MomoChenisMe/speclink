//! Output of the discuss family: every discussion subcommand.

use serde::Serialize;
use serde_json::{json, Value};
use speclink_core::command::{
    DiscussArchiveOutcome, DiscussConcludeOutcome, DiscussPromoteOutcome, DiscussRoundOutcome,
    DiscussShowOutcome,
};
use speclink_core::discuss::{DiscussionHit, DiscussionInfo};

use super::{JsonForm, Paint, RenderOptions, Rendered, Text};

/// The three lines of `discuss new`, by name.
#[derive(Debug, Clone, Copy)]
pub struct NewDiscussionLines<'a> {
    pub slug: &'a str,
    pub topic: &'a str,
    pub path: &'a str,
}

/// `discuss new`: the success line, the topic and the path.
pub fn new_human(lines: NewDiscussionLines<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Created discussion: {}", paint.green("✓"), lines.slug);
    outln!(t, "  Topic: {}", lines.topic);
    outln!(t, "  Path: {}", lines.path);
    t
}

/// `discuss new --json` is the new record's info.
pub fn new(info: &DiscussionInfo, options: RenderOptions) -> Rendered {
    let lines = NewDiscussionLines { slug: &info.slug, topic: &info.topic, path: &info.path };
    new_from(info, lines, options)
}

/// `discuss new` where the JSON form is the source's own shape: remote mode
/// prints the server's response as it came, with no field the server did not
/// send.
pub fn new_from<T: Serialize + ?Sized>(
    json: &T,
    lines: NewDiscussionLines<'_>,
    options: RenderOptions,
) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(json)), |paint| new_human(lines, paint))
}

/// `discuss list --json`.
pub fn list_json(items: &[DiscussionInfo]) -> Value {
    json!({ "discussions": items })
}

/// `discuss list`: one line per record, or a one-line notice when there is
/// none. `archived` picks the wording.
pub fn list_human(items: &[DiscussionInfo], archived: bool) -> Text {
    let mut t = Text::default();
    if items.is_empty() {
        let what = if archived { "archived discussions" } else { "discussions" };
        outln!(t, "No {what} found.");
        return t;
    }
    let heading = if archived { "Archived discussions:" } else { "Discussions:" };
    outln!(t, "{heading}");
    for d in items {
        outln!(t, "  • {} [{}] ({} rounds) — {}", d.slug, d.status, d.rounds, d.topic);
    }
    t
}

pub fn list(items: &[DiscussionInfo], archived: bool, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&list_json(items))), |_| list_human(items, archived))
}

/// `discuss search --json`: `{ "hits": [...] }`, with or without hits.
pub fn search_json(hits: &[DiscussionHit]) -> Value {
    json!({ "hits": hits })
}

/// `discuss search`: one line per hit in the list's idiom, then one indented
/// line per match (`<where> <kind>: <text>`). Zero hits is a one-line notice.
pub fn search_human(terms: &[String], hits: &[DiscussionHit], paint: &Paint) -> Text {
    let mut t = Text::default();
    let query = terms.join(" ");
    if hits.is_empty() {
        outln!(t, "No discussions match \"{query}\".");
        return t;
    }
    outln!(t, "Discussions matching \"{query}\":");
    for hit in hits {
        let d = &hit.info;
        let place = if d.archived { "archived" } else { "live" };
        outln!(t, "  • {} [{}, {place}] ({}) — {}", d.slug, d.status, d.created, d.topic);
        for m in &hit.matches {
            outln!(t, "      {} {}: {}", paint.dim(&m.where_), m.kind, m.text);
        }
    }
    t
}

pub fn search(terms: &[String], hits: &[DiscussionHit], options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&search_json(hits))), |paint| {
        search_human(terms, hits, paint)
    })
}

/// `discuss show --json`: the record's info and its text.
pub fn show_json(show: &DiscussShowOutcome) -> Value {
    json!({ "info": show.info, "content": show.content })
}

/// `discuss show`: the record as it is stored.
pub fn show_human(show: &DiscussShowOutcome) -> Text {
    let mut t = Text::default();
    out!(t, "{}", show.content);
    t
}

pub fn show(show: &DiscussShowOutcome, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&show_json(show))), |_| show_human(show))
}

/// `discuss context --json`.
pub fn context_json(slug: &str) -> Value {
    json!({ "slug": slug, "context": "set" })
}

/// `discuss context`: one success line.
pub fn context_human(slug: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Set context for discussion '{slug}'", paint.green("✓"));
    t
}

pub fn context(slug: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&context_json(slug))), |paint| context_human(slug, paint))
}

/// What `discuss add-round` reports.
#[derive(Debug, Clone, Copy)]
pub struct RecordedRound<'a> {
    pub slug: &'a str,
    pub round: usize,
    pub mode: &'a str,
}

/// `discuss add-round --json`.
pub fn add_round_json(r: &RecordedRound<'_>) -> Value {
    json!({ "slug": r.slug, "round": r.round, "mode": r.mode })
}

/// `discuss add-round`: one success line.
pub fn add_round_human(r: &RecordedRound<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Recorded round {} ({}) to discussion '{}'", paint.green("✓"), r.round, r.mode, r.slug);
    t
}

pub fn add_round(r: &RecordedRound<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&add_round_json(r))), |paint| add_round_human(r, paint))
}

/// `discuss add-round`, from the engine's result.
pub fn round_recorded(o: &DiscussRoundOutcome, options: RenderOptions) -> Rendered {
    add_round(&RecordedRound { slug: &o.slug, round: o.round, mode: &o.mode }, options)
}

/// What `discuss conclude` reports.
#[derive(Debug, Clone, Copy)]
pub struct Concluded<'a> {
    pub slug: &'a str,
    /// The changes a re-conclude flagged for re-ingest.
    pub restale_flagged: &'a [String],
    /// The closing step archived the record.
    pub auto_archived: bool,
    /// The record stays live for a later spin-out.
    pub held: bool,
    /// Why the closing step could not archive the record, for a caller that
    /// reports it inside the value. The CLI reports it as its exit error
    /// (see [`closing_error_message`]) and gives `None` here.
    pub closing_error: Option<&'a str>,
}

/// `discuss conclude --json`: `restaleFlagged`, `autoArchived`, `held` and
/// `closingError` appear only when they carry something.
pub fn conclude_json(c: &Concluded<'_>) -> Value {
    let mut payload = json!({ "slug": c.slug, "status": "concluded" });
    if !c.restale_flagged.is_empty() {
        payload["restaleFlagged"] = json!(c.restale_flagged);
    }
    if c.auto_archived {
        payload["autoArchived"] = json!(true);
    }
    if c.held {
        payload["held"] = json!(true);
    }
    if let Some(reason) = c.closing_error {
        payload["closingError"] = json!(reason);
    }
    payload
}

/// `discuss conclude`: the success line, then one line per fact that holds.
pub fn conclude_human(c: &Concluded<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Concluded discussion '{}'", paint.green("✓"), c.slug);
    if !c.restale_flagged.is_empty() {
        outln!(
            t,
            "  Flagged {} change(s) for re-ingest: {}",
            c.restale_flagged.len(),
            c.restale_flagged.join(", ")
        );
    }
    if c.auto_archived {
        outln!(t, "  Auto-archived the record (all promoted changes are archived)");
    }
    if c.held {
        outln!(t, "  Held live (a later spin-out is planned)");
    }
    t
}

pub fn conclude(c: &Concluded<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&conclude_json(c))), |paint| conclude_human(c, paint))
}

impl<'a> From<&'a DiscussConcludeOutcome> for Concluded<'a> {
    /// The engine's result without its closing error: where that goes is the
    /// caller's decision.
    fn from(o: &'a DiscussConcludeOutcome) -> Self {
        Concluded {
            slug: &o.slug,
            restale_flagged: &o.restale_flagged,
            auto_archived: o.auto_archived,
            held: o.held,
            closing_error: None,
        }
    }
}

/// `discuss conclude`, from the engine's result. A closing step that failed
/// is not part of this output: the caller reports it with
/// [`closing_error_message`].
pub fn concluded(o: &DiscussConcludeOutcome, options: RenderOptions) -> Rendered {
    conclude(&Concluded::from(o), options)
}

/// The error text when the conclusion landed but the closing step could not
/// archive the record. The CLI fails with it after the output.
pub fn closing_error_message(reason: &str) -> String {
    format!("conclude closing step failed to archive the record: {reason}")
}

/// Where an archived record is, as the output shows it.
fn archived_to(file: &str) -> String {
    format!("discussions/archive/{file}")
}

/// `discuss archive --json`.
pub fn archive_json(slug: &str, archived_to: &str) -> Value {
    json!({ "slug": slug, "archived_to": archived_to })
}

/// `discuss archive`: one success line.
pub fn archive_human(slug: &str, archived_to: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Archived discussion: {slug} → {archived_to}", paint.green("✓"));
    t
}

pub fn archive(slug: &str, archived_to: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&archive_json(slug, archived_to))), |paint| {
        archive_human(slug, archived_to, paint)
    })
}

/// `discuss archive`, from the engine's result.
pub fn archived(o: &DiscussArchiveOutcome, options: RenderOptions) -> Rendered {
    archive(&o.slug, &archived_to(&o.archived_file), options)
}

/// `discuss discard --json`.
pub fn discard_json(slug: &str) -> Value {
    json!({ "slug": slug, "status": "discarded" })
}

/// `discuss discard`: one success line.
pub fn discard_human(slug: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Discarded discussion: {slug}", paint.green("✓"));
    t
}

pub fn discard(slug: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&discard_json(slug))), |paint| discard_human(slug, paint))
}

/// The new change's directory in both spellings: the human line keeps the
/// platform separators the path was built with, while `--json` is always
/// slashed. The two are the same on Unix and differ on Windows.
#[derive(Debug, Clone, Copy)]
pub struct PromotedPath<'a> {
    pub shown: &'a str,
    pub wire: &'a str,
}

/// What `discuss promote` reports. `path` is absent in remote mode: the new
/// change's directory is a store-side location, so the Path line and the hint
/// after it are both left out.
#[derive(Debug, Clone, Copy)]
pub struct Promoted<'a> {
    pub slug: &'a str,
    pub change: &'a str,
    pub path: Option<PromotedPath<'a>>,
}

/// `discuss promote --json`.
pub fn promote_json(p: &Promoted<'_>) -> Value {
    let mut payload = json!({
        "change": p.change,
        "slug": p.slug,
        "status": "promoted",
    });
    if let Some(path) = &p.path {
        payload["path"] = json!(path.wire);
    }
    payload
}

/// `discuss promote`: the success line, then the path and the next step.
pub fn promote_human(p: &Promoted<'_>, paint: &Paint) -> Text {
    let mut t = Text::default();
    outln!(t, "{} Promoted discussion '{}' → change '{}'", paint.green("✓"), p.slug, p.change);
    if let Some(path) = &p.path {
        outln!(t, "  Path: {}", path.shown);
        outln!(t, "  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts");
    }
    t
}

pub fn promote(p: &Promoted<'_>, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&promote_json(p))), |paint| promote_human(p, paint))
}

/// `discuss promote`, from the engine's result: a local store, so the new
/// change's directory is shown.
pub fn promoted(o: &DiscussPromoteOutcome, options: RenderOptions) -> Rendered {
    let shown = o.path.to_string_lossy();
    let wire = speclink_core::util::to_slash(&o.path);
    promote(
        &Promoted {
            slug: &o.slug,
            change: &o.change,
            path: Some(PromotedPath { shown: &shown, wire: &wire }),
        },
        options,
    )
}

/// `discuss link` or `discuss seal`: the two differ only in their verb and
/// the closing note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscussBind {
    Link,
    Seal,
}

impl DiscussBind {
    /// The JSON status, the success line's verb, and its closing note.
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            DiscussBind::Link => ("linked", "Linked", ""),
            DiscussBind::Seal => ("sealed", "Sealed", " (marked promoted)"),
        }
    }
}

/// `discuss link --json` and `discuss seal --json`.
pub fn bind_json(bind: DiscussBind, slug: &str, change: &str) -> Value {
    let (status, _, _) = bind.parts();
    json!({
        "change": change,
        "slug": slug,
        "status": status,
    })
}

/// `discuss link` and `discuss seal`: one success line.
pub fn bind_human(bind: DiscussBind, slug: &str, change: &str, paint: &Paint) -> Text {
    let mut t = Text::default();
    let (_, verb, suffix) = bind.parts();
    outln!(t, "{} {verb} discussion '{slug}' → change '{change}'{suffix}", paint.green("✓"));
    t
}

pub fn bind(bind: DiscussBind, slug: &str, change: &str, options: RenderOptions) -> Rendered {
    Rendered::of(options, Some(JsonForm::pretty(&bind_json(bind, slug, change))), |paint| {
        bind_human(bind, slug, change, paint)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use speclink_core::discuss::DiscussionMatch;

    const PLAIN: Paint = Paint::new(false);
    const COLOR: Paint = Paint::new(true);
    const JSON: RenderOptions = RenderOptions { json: true, color: false };

    fn info(slug: &str, topic: &str, status: &str, rounds: usize, archived: bool) -> DiscussionInfo {
        DiscussionInfo {
            slug: slug.to_string(),
            topic: topic.to_string(),
            status: status.to_string(),
            rounds,
            created: "2026-11-11".to_string(),
            created_by: Some("Base Line <base@example.com>".to_string()),
            kind: None,
            path: format!("/p/speclink/discussions/{slug}.md"),
            archived,
            concluded: false,
            head: Default::default(),
        }
    }

    // --- new ---

    #[test]
    fn new_human_prints_the_slug_the_topic_and_the_path() {
        let lines = NewDiscussionLines {
            slug: "fresh-topic",
            topic: "Fresh topic",
            path: "/p/speclink/discussions/fresh-topic.md",
        };
        assert_eq!(
            new_human(lines, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Created discussion: fresh-topic\n  Topic: Fresh topic\n  Path: /p/speclink/discussions/fresh-topic.md\n"
        );
    }

    #[test]
    fn new_json_is_the_info_and_its_text_keeps_the_field_order() {
        let fresh = info("fresh-topic", "Fresh topic", "open", 0, false);
        assert_eq!(
            new(&fresh, RenderOptions::default()).json.unwrap(),
            json!({
                "slug": "fresh-topic", "topic": "Fresh topic", "status": "open", "rounds": 0,
                "created": "2026-11-11", "createdBy": "Base Line <base@example.com>",
                "path": "/p/speclink/discussions/fresh-topic.md", "archived": false,
            })
        );
        assert_eq!(
            new(&fresh, JSON).stdout,
            "{\n  \"slug\": \"fresh-topic\",\n  \"topic\": \"Fresh topic\",\n  \"status\": \"open\",\n  \"rounds\": 0,\n  \"created\": \"2026-11-11\",\n  \"createdBy\": \"Base Line <base@example.com>\",\n  \"path\": \"/p/speclink/discussions/fresh-topic.md\",\n  \"archived\": false\n}\n"
        );
        assert_eq!(
            new(&fresh, RenderOptions::default()).stdout,
            "✓ Created discussion: fresh-topic\n  Topic: Fresh topic\n  Path: /p/speclink/discussions/fresh-topic.md\n"
        );
    }

    #[test]
    fn new_from_prints_the_source_it_was_given() {
        let lines = NewDiscussionLines { slug: "fresh", topic: "Fresh topic", path: "discussions/fresh.md" };
        let wire = json!({ "slug": "fresh", "topic": "Fresh topic", "path": "discussions/fresh.md" });
        let out = new_from(&wire, lines, JSON);
        assert_eq!(out.stdout, "{\n  \"path\": \"discussions/fresh.md\",\n  \"slug\": \"fresh\",\n  \"topic\": \"Fresh topic\"\n}\n");
        assert_eq!(out.json, Some(wire.clone()));
        assert_eq!(
            new_from(&wire, lines, RenderOptions::default()).stdout,
            "✓ Created discussion: fresh\n  Topic: Fresh topic\n  Path: discussions/fresh.md\n"
        );
    }

    // --- list ---

    #[test]
    fn list_human_prints_one_line_per_record_under_the_right_heading() {
        let items = vec![
            info("topic-done", "Topic done", "concluded", 1, false),
            info("topic-live", "Topic live", "open", 2, false),
        ];
        assert_eq!(
            list_human(&items, false).stdout,
            "Discussions:\n  • topic-done [concluded] (1 rounds) — Topic done\n  • topic-live [open] (2 rounds) — Topic live\n"
        );
        assert_eq!(
            list_human(&items[..1], true).stdout,
            "Archived discussions:\n  • topic-done [concluded] (1 rounds) — Topic done\n"
        );
        assert_eq!(list_human(&[], false).stdout, "No discussions found.\n");
        assert_eq!(list_human(&[], true).stdout, "No archived discussions found.\n");
    }

    #[test]
    fn list_json_wraps_the_records() {
        let items = vec![info("topic-live", "Topic live", "open", 2, false)];
        assert_eq!(
            list_json(&items),
            json!({ "discussions": [{
                "archived": false, "created": "2026-11-11", "createdBy": "Base Line <base@example.com>",
                "path": "/p/speclink/discussions/topic-live.md", "rounds": 2, "slug": "topic-live",
                "status": "open", "topic": "Topic live",
            }] })
        );
        assert_eq!(list(&[], false, JSON).stdout, "{\n  \"discussions\": []\n}\n");
    }

    // --- search ---

    fn hit() -> DiscussionHit {
        let found = |kind: &str, where_: &str, text: &str| DiscussionMatch {
            kind: kind.to_string(),
            where_: where_.to_string(),
            text: text.to_string(),
        };
        DiscussionHit {
            info: info("topic-old", "Topic old", "concluded", 1, true),
            matches: vec![
                found("topic", "frontmatter", "Topic old"),
                found("ruled-out", "round-1", "**Ruled out**: way B"),
            ],
        }
    }

    #[test]
    fn search_human_prints_each_hit_with_its_matches() {
        let terms = vec!["topic".to_string(), "way".to_string()];
        assert_eq!(
            search_human(&terms, &[hit()], &COLOR).stdout,
            "Discussions matching \"topic way\":\n  • topic-old [concluded, archived] (2026-11-11) — Topic old\n      \x1b[2mfrontmatter\x1b[0m topic: Topic old\n      \x1b[2mround-1\x1b[0m ruled-out: **Ruled out**: way B\n"
        );
        let mut live = hit();
        live.info.archived = false;
        live.matches.clear();
        assert_eq!(
            search_human(&terms, &[live], &PLAIN).stdout,
            "Discussions matching \"topic way\":\n  • topic-old [concluded, live] (2026-11-11) — Topic old\n"
        );
        assert_eq!(
            search_human(&["zzzznomatch".to_string()], &[], &PLAIN).stdout,
            "No discussions match \"zzzznomatch\".\n"
        );
    }

    #[test]
    fn search_json_wraps_the_hits() {
        let value = search_json(&[hit()]);
        assert_eq!(value["hits"][0]["slug"], "topic-old");
        assert_eq!(
            value["hits"][0]["matches"][1],
            json!({ "kind": "ruled-out", "where": "round-1", "text": "**Ruled out**: way B" })
        );
        assert_eq!(search(&[], &[], JSON).stdout, "{\n  \"hits\": []\n}\n");
    }

    // --- show ---

    #[test]
    fn show_human_is_the_record_as_stored_and_show_json_adds_the_info() {
        let shown = DiscussShowOutcome {
            info: Some(info("topic-live", "Topic live", "open", 2, false)),
            content: "---\ntopic: Topic live\n---\n".to_string(),
        };
        assert_eq!(show_human(&shown).stdout, "---\ntopic: Topic live\n---\n");
        let value = show_json(&shown);
        assert_eq!(value["content"], "---\ntopic: Topic live\n---\n");
        assert_eq!(value["info"]["slug"], "topic-live");
        let bare = DiscussShowOutcome { info: None, content: "text".to_string() };
        assert_eq!(show(&bare, JSON).stdout, "{\n  \"content\": \"text\",\n  \"info\": null\n}\n");
    }

    // --- context, add-round, conclude ---

    #[test]
    fn context_and_add_round_are_one_line_each() {
        assert_eq!(
            context_human("topic-live", &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Set context for discussion 'topic-live'\n"
        );
        assert_eq!(context_json("topic-live"), json!({ "slug": "topic-live", "context": "set" }));
        assert_eq!(
            context("topic-live", JSON).stdout,
            "{\n  \"context\": \"set\",\n  \"slug\": \"topic-live\"\n}\n"
        );
        let round = RecordedRound { slug: "topic-live", round: 3, mode: "grill" };
        assert_eq!(
            add_round_human(&round, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Recorded round 3 (grill) to discussion 'topic-live'\n"
        );
        assert_eq!(add_round_json(&round), json!({ "slug": "topic-live", "round": 3, "mode": "grill" }));
        assert_eq!(
            add_round(&round, JSON).stdout,
            "{\n  \"mode\": \"grill\",\n  \"round\": 3,\n  \"slug\": \"topic-live\"\n}\n"
        );
    }

    #[test]
    fn conclude_adds_a_line_and_a_field_for_each_fact_that_holds() {
        let plain = Concluded {
            slug: "topic-live",
            restale_flagged: &[],
            auto_archived: false,
            held: false,
            closing_error: None,
        };
        assert_eq!(conclude_human(&plain, &COLOR).stdout, "\x1b[32m✓\x1b[0m Concluded discussion 'topic-live'\n");
        assert_eq!(conclude_json(&plain), json!({ "slug": "topic-live", "status": "concluded" }));

        let flagged = vec!["beta".to_string(), "later-part".to_string()];
        let full = Concluded {
            slug: "topic-held",
            restale_flagged: &flagged,
            auto_archived: true,
            held: true,
            closing_error: None,
        };
        assert_eq!(
            conclude_human(&full, &PLAIN).stdout,
            "✓ Concluded discussion 'topic-held'\n  Flagged 2 change(s) for re-ingest: beta, later-part\n  Auto-archived the record (all promoted changes are archived)\n  Held live (a later spin-out is planned)\n"
        );
        assert_eq!(
            conclude_json(&full),
            json!({
                "slug": "topic-held", "status": "concluded",
                "restaleFlagged": ["beta", "later-part"], "autoArchived": true, "held": true,
            })
        );
        let held = Concluded { closing_error: None, held: true, ..plain };
        let held = Concluded { slug: "topic-held", ..held };
        assert_eq!(
            conclude(&held, JSON).stdout,
            "{\n  \"held\": true,\n  \"slug\": \"topic-held\",\n  \"status\": \"concluded\"\n}\n"
        );
        assert_eq!(
            closing_error_message("the file is locked"),
            "conclude closing step failed to archive the record: the file is locked"
        );
    }

    #[test]
    fn conclude_json_carries_the_closing_error_only_when_the_caller_gives_one() {
        let failed = Concluded {
            slug: "topic-held",
            restale_flagged: &[],
            auto_archived: false,
            held: false,
            closing_error: Some("the file is locked"),
        };
        assert_eq!(
            conclude_json(&failed),
            json!({ "slug": "topic-held", "status": "concluded", "closingError": "the file is locked" })
        );
        // The human text is the same either way: the caller reports the failure.
        assert_eq!(conclude_human(&failed, &PLAIN).stdout, "✓ Concluded discussion 'topic-held'\n");
        let fine = Concluded { closing_error: None, ..failed };
        assert_eq!(conclude_json(&fine), json!({ "slug": "topic-held", "status": "concluded" }));
    }

    // --- archive, discard, promote, link, seal ---

    #[test]
    fn archive_and_discard_are_one_line_each() {
        let to = archived_to("2026-11-11-topic-done.md");
        assert_eq!(to, "discussions/archive/2026-11-11-topic-done.md");
        assert_eq!(
            archive_human("topic-done", &to, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Archived discussion: topic-done → discussions/archive/2026-11-11-topic-done.md\n"
        );
        assert_eq!(
            archive("topic-done", &to, JSON).stdout,
            "{\n  \"archived_to\": \"discussions/archive/2026-11-11-topic-done.md\",\n  \"slug\": \"topic-done\"\n}\n"
        );
        assert_eq!(archive_json("topic-done", &to), json!({ "slug": "topic-done", "archived_to": to }));
        assert_eq!(
            discard_human("topic-live", &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Discarded discussion: topic-live\n"
        );
        assert_eq!(discard_json("topic-live"), json!({ "slug": "topic-live", "status": "discarded" }));
        assert_eq!(
            discard("topic-live", JSON).stdout,
            "{\n  \"slug\": \"topic-live\",\n  \"status\": \"discarded\"\n}\n"
        );
    }

    #[test]
    fn promote_prints_the_path_and_the_hint_together_or_not_at_all() {
        let local = Promoted {
            slug: "topic-done",
            change: "topic-done",
            path: Some(PromotedPath { shown: "C:\\p\\changes\\topic-done", wire: "C:/p/changes/topic-done" }),
        };
        assert_eq!(
            promote_human(&local, &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Promoted discussion 'topic-done' → change 'topic-done'\n  Path: C:\\p\\changes\\topic-done\n  Proposal prefilled from the conclusion — run /speclink-propose to complete the artifacts\n"
        );
        assert_eq!(
            promote_json(&local),
            json!({ "change": "topic-done", "slug": "topic-done", "status": "promoted", "path": "C:/p/changes/topic-done" })
        );
        let remote = Promoted { slug: "topic-held", change: "later-part", path: None };
        assert_eq!(
            promote_human(&remote, &PLAIN).stdout,
            "✓ Promoted discussion 'topic-held' → change 'later-part'\n"
        );
        assert_eq!(
            promote(&remote, JSON).stdout,
            "{\n  \"change\": \"later-part\",\n  \"slug\": \"topic-held\",\n  \"status\": \"promoted\"\n}\n"
        );
    }

    #[test]
    fn link_and_seal_differ_in_their_verb_and_closing_note() {
        assert_eq!(
            bind_human(DiscussBind::Link, "topic-done", "beta", &COLOR).stdout,
            "\x1b[32m✓\x1b[0m Linked discussion 'topic-done' → change 'beta'\n"
        );
        assert_eq!(
            bind_human(DiscussBind::Seal, "topic-held", "beta", &PLAIN).stdout,
            "✓ Sealed discussion 'topic-held' → change 'beta' (marked promoted)\n"
        );
        assert_eq!(
            bind_json(DiscussBind::Link, "topic-done", "beta"),
            json!({ "change": "beta", "slug": "topic-done", "status": "linked" })
        );
        assert_eq!(
            bind(DiscussBind::Seal, "topic-held", "beta", JSON).stdout,
            "{\n  \"change\": \"beta\",\n  \"slug\": \"topic-held\",\n  \"status\": \"sealed\"\n}\n"
        );
    }
}

//! Discussion documents — speclink's enhancement over Spectra's ephemeral discuss.
//!
//! Each discussion is a single append-only document (stored by the Store as a
//! live discussion under its slug) so an iterative conversation accumulates a
//! durable record that `propose` can later consume. Archived discussions are
//! renamed by the store with a `<created>-` date prefix — like archived
//! changes — so a slug can be reused by a later discussion.

use crate::store::{DiscussionDoc, Store};
use crate::util;
use crate::workspace::Workspace;
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DiscussionInfo {
    pub slug: String,
    pub topic: String,
    pub status: String,
    pub rounds: usize,
    pub created: String,
    pub path: String,
    pub archived: bool,
}

fn frontmatter_value(text: &str, key: &str) -> Option<String> {
    let mut in_fm = false;
    for (i, line) in text.lines().enumerate() {
        if i == 0 && line.trim() == "---" {
            in_fm = true;
            continue;
        }
        if in_fm {
            if line.trim() == "---" {
                break;
            }
            if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

fn count_rounds(text: &str) -> usize {
    // `### Round ` is the scaffolded layout; `## Round ` tolerates pre-scaffold documents.
    text.lines()
        .filter(|l| l.starts_with("### Round ") || l.starts_with("## Round "))
        .count()
}

/// Byte range of a level-2 section's body: after the `## <name>` line, up to the next `## `
/// line or EOF. `None` when the header is absent.
fn section_body_range(text: &str, name: &str) -> Option<(usize, usize)> {
    let header = format!("## {name}");
    let mut offset = 0;
    let mut start: Option<usize> = None;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        if let Some(s) = start {
            if trimmed.starts_with("## ") && !trimmed.starts_with("###") {
                return Some((s, offset));
            }
        } else if trimmed == header {
            start = Some(offset + line.len());
        }
        offset += line.len();
    }
    start.map(|s| (s, text.len()))
}

/// Replace a level-2 section's body, keeping its header. `None` when the section is absent.
fn replace_section(text: &str, name: &str, body: &str) -> Option<String> {
    let (s, e) = section_body_range(text, name)?;
    let tail = &text[e..];
    let mid = if tail.is_empty() {
        format!("\n{}\n", body.trim_end())
    } else {
        format!("\n{}\n\n", body.trim_end())
    };
    Some(format!("{}{}{}", &text[..s], mid, tail))
}

fn strip_html_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<!--") {
        out.push_str(&rest[..i]);
        match rest[i..].find("-->") {
            Some(j) => rest = &rest[i + j + 3..],
            None => rest = "",
        }
    }
    out.push_str(rest);
    out
}

fn info_from_doc(doc: &DiscussionDoc) -> DiscussionInfo {
    DiscussionInfo {
        slug: frontmatter_value(&doc.text, "slug").unwrap_or_else(|| doc.slug.clone()),
        topic: frontmatter_value(&doc.text, "topic").unwrap_or_else(|| doc.slug.clone()),
        status: frontmatter_value(&doc.text, "status").unwrap_or_else(|| "open".to_string()),
        rounds: count_rounds(&doc.text),
        created: frontmatter_value(&doc.text, "created").unwrap_or_default(),
        path: util::to_slash(&doc.path),
        archived: doc.archived,
    }
}

/// Load a live discussion for mutation; a helpful error distinguishes "archived" from "missing".
fn load_live(store: &dyn Store, slug: &str) -> Result<String> {
    match store.read_live_discussion(slug) {
        Some(t) => Ok(t),
        None => {
            if store.archived_discussion_exists(slug) {
                bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to continue it");
            }
            bail!("discussion '{slug}' not found — run `speclink discuss new` first")
        }
    }
}

/// Create a new discussion document. Errors if a live one already exists.
pub fn new_discussion(store: &dyn Store, topic: &str) -> Result<DiscussionInfo> {
    let slug = util::slugify(topic);
    if slug.is_empty() {
        bail!("could not derive a slug from topic '{topic}'");
    }
    if store.live_discussion_exists(&slug) {
        bail!(
            "discussion '{slug}' already exists at {}",
            util::to_slash(&store.live_discussion_path(&slug))
        );
    }
    let created = util::today();
    let content = format!(
        "---\n\
         topic: {topic}\n\
         slug: {slug}\n\
         status: open\n\
         created: {created}\n\
         ---\n\
         \n\
         # Discussion: {topic}\n\
         \n\
         <!--\n\
         Document rules:\n\
         - Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.\n\
         \x20 A changed position gets a new round that names what changed and why.\n\
         - Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.\n\
         - The conclusion must resolve or explicitly defer every open question left by the rounds.\n\
         -->\n\
         \n\
         ## Context\n\
         \n\
         <!-- What prompted this discussion, the mode chosen (assumptions | interview) and why,\n\
         and the related changes/specs. Set once via `speclink discuss context <slug> --stdin`. -->\n\
         \n\
         ## Rounds\n\
         \n\
         <!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->\n\
         \n\
         ## Conclusion\n\
         \n\
         <!-- Written by `speclink discuss conclude`:\n\
         **Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->\n"
    );
    let path = store.write_live_discussion(&slug, &content)?;
    Ok(DiscussionInfo {
        slug,
        topic: topic.to_string(),
        status: "open".to_string(),
        rounds: 0,
        created,
        path: util::to_slash(&path),
        archived: false,
    })
}

/// List live discussions (sorted by slug).
pub fn list_discussions(store: &dyn Store) -> Vec<DiscussionInfo> {
    let mut out: Vec<DiscussionInfo> = store
        .list_live_discussions()
        .iter()
        .map(info_from_doc)
        .collect();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
}

/// List archived discussions (sorted by archived file name, i.e. by archive date).
pub fn list_archived(store: &dyn Store) -> Vec<DiscussionInfo> {
    let mut out: Vec<DiscussionInfo> = store
        .list_archived_discussions()
        .iter()
        .map(info_from_doc)
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

pub fn show_discussion(store: &dyn Store, slug: &str) -> Option<String> {
    store.read_discussion(slug).map(|d| d.text)
}

pub fn info(store: &dyn Store, slug: &str) -> Option<DiscussionInfo> {
    store.read_discussion(slug).map(|d| info_from_doc(&d))
}

/// Set (or replace) the `## Context` section — the one-time framing written after mode pick.
pub fn set_context(store: &dyn Store, slug: &str, content: &str) -> Result<()> {
    let text = load_live(store, slug)?;
    match replace_section(&text, "Context", content) {
        Some(t) => {
            store.write_live_discussion(slug, &t)?;
            Ok(())
        }
        None => bail!(
            "discussion '{slug}' has no '## Context' section (pre-scaffold layout) — edit the file directly"
        ),
    }
}

/// Append a discussion round. Content is supplied verbatim (from the skill via stdin).
pub fn add_round(store: &dyn Store, slug: &str, mode: &str, content: &str) -> Result<usize> {
    let mut text = load_live(store, slug)?;
    let round_no = count_rounds(&text) + 1;
    let date = util::today();
    // Scaffolded layout: insert at the end of the `## Rounds` section. Pre-scaffold
    // documents fall back to appending a level-2 round at the end.
    if let Some((_, e)) = section_body_range(&text, "Rounds") {
        let entry = format!(
            "### Round {round_no} — {mode} ({date})\n\n{}\n\n",
            content.trim_end()
        );
        text.insert_str(e, &entry);
    } else {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!(
            "\n## Round {round_no} — {mode} ({date})\n\n{}\n",
            content.trim_end()
        ));
    }
    store.write_live_discussion(slug, &text)?;
    Ok(round_no)
}

/// The text of the `## Conclusion` section, if the discussion has one (the scaffold's
/// placeholder comment does not count as content).
pub fn conclusion_text(store: &dyn Store, slug: &str) -> Option<String> {
    let text = store.read_discussion(slug)?.text;
    let (s, e) = section_body_range(&text, "Conclusion")?;
    let body = strip_html_comments(&text[s..e]).trim().to_string();
    (!body.is_empty()).then_some(body)
}

/// Mark a discussion as promoted to a change (the discussion side of the bidirectional link).
/// A discussion can fan out into several changes, so `promoted_to` is a comma-separated
/// accumulator: repeated promotes append the new change name rather than being dropped.
pub fn mark_promoted(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    let mut text = load_live(store, slug)?;
    for from in ["status: open", "status: concluded"] {
        if text.contains(from) {
            text = text.replacen(from, "status: promoted", 1);
            break;
        }
    }
    match frontmatter_value(&text, "promoted_to") {
        Some(existing) => {
            if !existing.split(',').map(str::trim).any(|c| c == change) {
                text = text.replacen(
                    &format!("promoted_to: {existing}"),
                    &format!("promoted_to: {existing}, {change}"),
                    1,
                );
            }
        }
        None => {
            text = text.replacen(
                "status: promoted\n",
                &format!("status: promoted\npromoted_to: {change}\n"),
                1,
            );
        }
    }
    store.write_live_discussion(slug, &text)?;
    Ok(())
}

/// Outcome of promoting a discussion into a change.
#[derive(Debug)]
pub struct PromoteOutcome {
    pub change: String,
    pub path: PathBuf,
}

/// Strip an archive-style `YYYY-MM-DD-` prefix from a candidate change name —
/// archived names are historical references, not active names to reuse. Kept
/// only when something remains after the prefix.
fn strip_date_prefix(name: &str) -> &str {
    let b = name.as_bytes();
    let dated = b.len() > 11
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[10] == b'-';
    if dated { &name[11..] } else { name }
}

/// Promote a discussion into a new change (the whole flow, shared by CLI and
/// desktop): refuse archived records, derive the change name (explicit name or
/// the slug, minus any archive date prefix), create the change with a
/// `from_discussion` link, prefill the proposal's Why from the conclusion
/// (topic as fallback), and mark the discussion promoted. Any failure before a
/// step leaves the later steps unexecuted, so a name collision never marks the
/// discussion.
pub fn promote(
    ws: &Workspace,
    store: &dyn Store,
    slug: &str,
    name: Option<&str>,
) -> Result<PromoteOutcome> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to promote it")
        }
        Some(_) => {}
    }
    let change_name = strip_date_prefix(name.unwrap_or(slug)).to_string();
    let schema =
        crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())
            .schema_name();
    let dir = crate::newcmd::new_change(ws, store, &change_name, None, &schema, None, Some(slug))?;
    // Prefill the proposal's Why from the discussion conclusion (topic as fallback);
    // the remaining sections stay as TBD markers for /speclink-propose to complete.
    let why = conclusion_text(store, slug).unwrap_or_else(|| {
        info(store, slug).map(|i| i.topic).unwrap_or_else(|| slug.to_string())
    });
    let proposal = format!(
        "## Why\n\n{why}\n\n## What Changes\n\n<!-- TBD: derive from the discussion -->\n\n## Capabilities\n\n### New Capabilities\n\n<!-- TBD -->\n\n## Impact\n\n<!-- TBD -->\n"
    );
    store.write_artifact(&change_name, "proposal.md", &proposal)?;
    mark_promoted(store, slug, &change_name)?;
    Ok(PromoteOutcome { change: change_name, path: dir })
}

/// The change names a discussion has fanned out into — the frontmatter's
/// comma-separated `promoted_to` accumulator, live or archived. Kept out of
/// `DiscussionInfo` so `discuss list --json` stays bit-identical (design D2).
pub fn promoted_to(store: &dyn Store, slug: &str) -> Vec<String> {
    let Some(doc) = store.read_discussion(slug) else {
        return Vec::new();
    };
    frontmatter_value(&doc.text, "promoted_to")
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Archive a live discussion under its creation date. Returns the archived
/// file name, or `None` when no live discussion exists. Same-day name
/// collisions are resolved by the store so co-archival never fails on a
/// reused slug.
pub fn archive_discussion(store: &dyn Store, slug: &str) -> Result<Option<String>> {
    let Some(text) = store.read_live_discussion(slug) else {
        return Ok(None);
    };
    let created = frontmatter_value(&text, "created")
        .filter(|c| !c.is_empty())
        .unwrap_or_else(util::today);
    store.archive_discussion(slug, &created)
}

/// Delete a live discussion outright — the exit for a record that turned out not to be
/// needed. Refuses once rounds exist (unless `force`): a discussion that examined real
/// trade-offs should keep its reasoning via `conclude` + `archive` instead.
pub fn discard_discussion(store: &dyn Store, slug: &str, force: bool) -> Result<()> {
    let Some(text) = store.read_live_discussion(slug) else {
        if store.archived_discussion_exists(slug) {
            bail!("discussion '{slug}' is archived — archived records are kept, not discarded");
        }
        bail!("discussion '{slug}' not found");
    };
    let rounds = count_rounds(&text);
    if rounds > 0 && !force {
        bail!(
            "discussion '{slug}' has {rounds} recorded round(s) — `conclude` + `archive` keeps the reasoning; pass --force to delete anyway"
        );
    }
    store.delete_live_discussion(slug)?;
    Ok(())
}

/// Write the conclusion into the `## Conclusion` section (replacing the placeholder — or a
/// previous conclusion, so a revised conclusion stays a single section) and mark the
/// discussion concluded.
pub fn conclude(store: &dyn Store, slug: &str, content: &str) -> Result<()> {
    let mut text = load_live(store, slug)?;
    // Flip status: open -> concluded in frontmatter.
    text = text.replacen("status: open", "status: concluded", 1);
    text = match replace_section(&text, "Conclusion", content) {
        Some(t) => t,
        None => {
            // Pre-scaffold document: append the section.
            if !text.ends_with('\n') {
                text.push('\n');
            }
            format!("{text}\n## Conclusion\n\n{}\n", content.trim_end())
        }
    };
    store.write_live_discussion(slug, &text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::Store;
    use crate::teststore::TestStore;
    use crate::workspace::Workspace;

    fn ghost_ws() -> Workspace {
        // Nonexistent root: git probes fail soft (no identity stamped), so the
        // flow is fully deterministic on any machine.
        Workspace {
            root: std::env::temp_dir().join("speclink-discuss-test-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        }
    }

    /// A scaffolded discussion document with a written conclusion.
    fn concluded_doc(slug: &str, topic: &str, decision: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: concluded\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: {decision}\n"
        )
    }

    /// A scaffolded discussion whose conclusion is still the placeholder comment.
    fn open_doc(slug: &str, topic: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: open\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n\
             ## Conclusion\n\n<!-- Written by `speclink discuss conclude` -->\n"
        )
    }

    // --- promote flow (design D1) ---

    #[test]
    fn promote_rejects_missing_discussion() {
        let store = TestStore::default();
        let err = super::promote(&ghost_ws(), &store, "ghost", None).unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
    }

    #[test]
    fn promote_rejects_archived_discussion() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old-topic".to_string(), concluded_doc("old-topic", "Old", "done"));
        let err = super::promote(&ghost_ws(), &store, "old-topic", None).unwrap_err();
        assert!(err.to_string().contains("archived"), "err: {err}");
        assert!(!store.change_exists("old-topic"), "no change may be created");
    }

    #[test]
    fn promote_derives_change_name_from_slug_by_default() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        let outcome = super::promote(&ghost_ws(), &store, "alpha-search", None).unwrap();
        assert_eq!(outcome.change, "alpha-search");
        assert!(store.change_exists("alpha-search"));
    }

    #[test]
    fn promote_uses_explicit_name_when_given() {
        let store = TestStore::with_live_discussion(
            "beta-cache",
            &concluded_doc("beta-cache", "Beta cache", "add cache layer"),
        );
        let outcome =
            super::promote(&ghost_ws(), &store, "beta-cache", Some("cache-layer")).unwrap();
        assert_eq!(outcome.change, "cache-layer");
        assert!(store.change_exists("cache-layer"));
        assert!(!store.change_exists("beta-cache"));
    }

    #[test]
    fn promote_strips_archive_date_prefix_from_derived_name() {
        // Archive-style date prefixes are historical references, not active
        // change names — derivation normalizes them away (either form).
        let store = TestStore::with_live_discussion(
            "2026-07-06-retro",
            &concluded_doc("2026-07-06-retro", "Retro", "do the retro"),
        );
        let outcome = super::promote(&ghost_ws(), &store, "2026-07-06-retro", None).unwrap();
        assert_eq!(outcome.change, "retro");

        let store2 = TestStore::with_live_discussion(
            "gamma-x",
            &concluded_doc("gamma-x", "Gamma x", "ship gamma"),
        );
        let outcome2 =
            super::promote(&ghost_ws(), &store2, "gamma-x", Some("2026-01-02-gamma-cut")).unwrap();
        assert_eq!(outcome2.change, "gamma-cut");
    }

    #[test]
    fn promote_creates_change_with_from_discussion_meta() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&ghost_ws(), &store, "alpha-search", None).unwrap();
        let meta = store.meta("alpha-search");
        assert!(meta.starts_with("schema: spec-driven\ncreated: "), "meta: {meta}");
        assert!(meta.contains("from_discussion: alpha-search\n"), "meta: {meta}");
    }

    #[test]
    fn promote_prefills_proposal_why_from_conclusion() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&ghost_ws(), &store, "alpha-search", None).unwrap();
        let proposal = store.read_artifact("alpha-search", "proposal.md").unwrap();
        assert_eq!(
            proposal,
            "## Why\n\n**Decision**: build alpha search\n\n## What Changes\n\n<!-- TBD: derive from the discussion -->\n\n## Capabilities\n\n### New Capabilities\n\n<!-- TBD -->\n\n## Impact\n\n<!-- TBD -->\n"
        );
    }

    #[test]
    fn promote_prefills_topic_when_no_conclusion() {
        // Placeholder-only conclusion → the topic is the Why fallback.
        let store =
            TestStore::with_live_discussion("open-one", &open_doc("open-one", "Open topic"));
        super::promote(&ghost_ws(), &store, "open-one", None).unwrap();
        let proposal = store.read_artifact("open-one", "proposal.md").unwrap();
        assert!(proposal.starts_with("## Why\n\nOpen topic\n"), "proposal: {proposal}");
    }

    #[test]
    fn promote_marks_promoted_and_accumulates_on_fan_out() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&ghost_ws(), &store, "alpha-search", None).unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("status: promoted\n"), "text: {text}");
        assert!(text.contains("promoted_to: alpha-search\n"), "text: {text}");

        // Second cut: promoted_to becomes a comma-separated accumulator.
        super::promote(&ghost_ws(), &store, "alpha-search", Some("second-cut")).unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("promoted_to: alpha-search, second-cut\n"), "text: {text}");
    }

    #[test]
    fn promote_fails_when_change_already_exists_and_leaves_discussion_untouched() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        store.metas.borrow_mut().insert("alpha-search".to_string(), "schema: spec-driven\n".to_string());
        let before = store.discussion("alpha-search");
        let err = super::promote(&ghost_ws(), &store, "alpha-search", None).unwrap_err();
        assert!(err.to_string().contains("already exists"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), before, "discussion must not be marked");
    }

    // --- promoted_to query (design D2) ---

    #[test]
    fn promoted_to_absent_yields_empty() {
        let store =
            TestStore::with_live_discussion("open-one", &open_doc("open-one", "Open topic"));
        assert!(super::promoted_to(&store, "open-one").is_empty());
        assert!(super::promoted_to(&store, "no-such-slug").is_empty());
    }

    #[test]
    fn promoted_to_single_value() {
        let mut doc = concluded_doc("alpha-search", "Alpha search", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        assert_eq!(super::promoted_to(&store, "alpha-search"), vec!["first-cut".to_string()]);
    }

    #[test]
    fn promoted_to_comma_accumulated_values() {
        let mut doc = concluded_doc("alpha-search", "Alpha search", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut, second-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        assert_eq!(
            super::promoted_to(&store, "alpha-search"),
            vec!["first-cut".to_string(), "second-cut".to_string()]
        );
    }

    #[test]
    fn promoted_to_reads_archived_records_too() {
        // The archived page needs the fan-out list for auto-archived discussions.
        let mut doc = concluded_doc("done-topic", "Done", "x");
        doc = doc.replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: only-cut\n",
            1,
        );
        let store = TestStore::default();
        store.archived_discussions.borrow_mut().insert("done-topic".to_string(), doc);
        assert_eq!(super::promoted_to(&store, "done-topic"), vec!["only-cut".to_string()]);
    }
}

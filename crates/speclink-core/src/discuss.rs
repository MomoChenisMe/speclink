//! Discussion documents — speclink's enhancement over Spectra's ephemeral discuss.
//!
//! Each discussion is a single append-only document (stored by the Store as a
//! live discussion under its slug) so an iterative conversation accumulates a
//! durable record that `propose` can later consume. Archived discussions are
//! renamed by the store with a `<created>-` date prefix — like archived
//! changes — so a slug can be reused by a later discussion.

use crate::store::{DiscussionDoc, Store};
use crate::util;
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
    /// 建立者（"Name <email>"），discuss new 由 git 身分蓋章；缺席時省略。
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none", default)]
    pub created_by: Option<String>,
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
        created_by: frontmatter_value(&doc.text, "created_by"),
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

/// Kebab-case gate for the slug override: lowercase ASCII letters/digits in
/// single-hyphen-separated runs. Deliberately stricter than the topic-derived
/// fallback (which keeps CJK) — the override exists to produce English names.
fn is_valid_slug_override(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Create a new discussion document. Errors if a live one already exists.
/// `slug_override` names the record file directly (validated ASCII kebab-case);
/// without it the slug falls back to deriving from the topic.
pub fn new_discussion(
    store: &dyn Store,
    topic: &str,
    slug_override: Option<&str>,
    created_by: Option<&str>,
) -> Result<DiscussionInfo> {
    let slug = match slug_override {
        Some(s) => {
            if !is_valid_slug_override(s) {
                bail!(
                    "invalid slug '{s}' — must be ASCII kebab-case: lowercase letters/digits \
                     separated by single hyphens (e.g. board-search-bar)"
                );
            }
            s.to_string()
        }
        None => util::slugify(topic),
    };
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
    // 建立者章（比照 change 的 newcmd）：有 git 身分才蓋，無身分省略該行。
    let created_by_line = created_by
        .map(|id| format!("created_by: {id}\n"))
        .unwrap_or_default();
    let content = format!(
        "---\n\
         topic: {topic}\n\
         slug: {slug}\n\
         status: open\n\
         created: {created}\n\
         {created_by_line}\
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
        created_by: created_by.map(str::to_string),
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

/// Reject blank content at the write boundary. The CLI turns a forgotten `--stdin` into an
/// empty string, so guarding here — one place, covering local CLI / remote CLI / desktop —
/// makes that silent failure a loud error instead of a written-but-empty section.
fn ensure_content(content: &str) -> Result<()> {
    if content.trim().is_empty() {
        bail!("discussion content is empty — pass non-empty content via stdin (did you forget --stdin?)");
    }
    Ok(())
}

/// Set (or replace) the `## Context` section — the one-time framing written after mode pick.
pub fn set_context(store: &dyn Store, slug: &str, content: &str) -> Result<()> {
    ensure_content(content)?;
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
    ensure_content(content)?;
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

/// The discard-side inverse of [`mark_promoted`]: unlink a discarded change from a
/// discussion. Removes the change name from the record's `promoted_to` comma
/// accumulator; when other change names remain the record stays `promoted`, but once
/// the list empties the `promoted_to` line is dropped and the status reverts — to
/// `concluded` when the record carries a real conclusion, else `open` (a promote/link
/// can raise an `open` discussion, so the revert restores its true prior state). The
/// Context/Rounds/Conclusion sections are never touched — only the frontmatter link
/// fields change (same layer `mark_promoted` writes). Returns the record's status
/// after unlinking (`"promoted"` when merely shrunk, else the reverted status), or
/// `None` when there was nothing to do: no live record for the slug (skipped, not an
/// error — the record may be archived or gone), or the change was not in the list
/// (idempotent — re-running discard leaves an already-unlinked record byte-identical).
pub fn unlink_discarded(store: &dyn Store, slug: &str, change: &str) -> Result<Option<String>> {
    let Some(mut text) = store.read_live_discussion(slug) else {
        return Ok(None);
    };
    let Some(existing) = frontmatter_value(&text, "promoted_to") else {
        return Ok(None);
    };
    let current: Vec<&str> =
        existing.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let remaining: Vec<&str> = current.iter().copied().filter(|s| *s != change).collect();
    if remaining.len() == current.len() {
        // change was never linked here — idempotent no-op, no write
        return Ok(None);
    }
    if remaining.is_empty() {
        // last link died: drop the promoted_to line and revert the status
        let reverted = if conclusion_text(store, slug).is_some() { "concluded" } else { "open" };
        text = text.replacen(&format!("promoted_to: {existing}\n"), "", 1);
        text = text.replacen("status: promoted", &format!("status: {reverted}"), 1);
        store.write_live_discussion(slug, &text)?;
        Ok(Some(reverted.to_string()))
    } else {
        // still referenced by other changes: shrink the list, keep promoted
        text = text.replacen(
            &format!("promoted_to: {existing}"),
            &format!("promoted_to: {}", remaining.join(", ")),
            1,
        );
        store.write_live_discussion(slug, &text)?;
        Ok(Some("promoted".to_string()))
    }
}

/// Stamp the re-ingest-pending flag on every **active** change in a re-concluded
/// discussion's `promoted_to`. The conclude-side mirror of [`unlink_discarded`]: a
/// discussion that was already reflected (its `promoted_to` is non-empty because
/// `seal` wrote it) and is now re-concluded flags each of its changes as stale
/// against the new conclusion. Change names that resolve to no active meta —
/// archived or gone — are skipped (their spec deltas are already in canon; a
/// re-ingest is impossible). Each active change's `restale_from` comma accumulator
/// gains this slug (idempotent: already present skips the write). Returns the active
/// change names carrying the flag, for CLI reporting. `promoted_to` absent/empty, or
/// resolving entirely to non-active changes, writes no change meta.
fn stamp_restale(store: &dyn Store, slug: &str, discussion_text: &str) -> Result<Vec<String>> {
    let Some(promoted) = frontmatter_value(discussion_text, "promoted_to") else {
        return Ok(Vec::new());
    };
    let mut flagged = Vec::new();
    for change in promoted.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let Some(mut meta) = store.read_change_meta(change) else {
            continue; // archived or gone — not an active change, skip
        };
        // 壞 metadata 卡跳過（沿 archived/gone 的 skip 原則）：不得對壞檔
        // append，也不得使 conclude 因單一壞檔中止——使用者修檔後重新 conclude。
        let Ok(parsed) = crate::model::ChangeMeta::from_text(Some(&meta)) else {
            continue;
        };
        let existing = parsed.restale_from.as_deref().map(str::trim).unwrap_or("");
        if parsed.restale_from().iter().any(|s| s == slug) {
            // already flagged for this slug — idempotent, skip the change-side write
        } else if existing.is_empty() {
            if !meta.ends_with('\n') && !meta.is_empty() {
                meta.push('\n');
            }
            meta.push_str(&format!("restale_from: {slug}\n"));
            store.write_change_meta(change, &meta)?;
        } else {
            meta = meta.replacen(
                &format!("restale_from: {existing}"),
                &format!("restale_from: {existing}, {slug}"),
                1,
            );
            store.write_change_meta(change, &meta)?;
        }
        flagged.push(change.to_string());
    }
    Ok(flagged)
}

/// Clear one discussion slug from a change's `restale_from` accumulator — the seal-side
/// inverse of [`stamp_restale`]. When the slug is the sole value the whole line is
/// dropped; otherwise the remaining slugs are kept. The slug being absent (or no
/// `restale_from` field at all) is an idempotent no-op that skips the write. Only the
/// `restale_from` field is touched; every other meta field stays byte-identical.
fn clear_restale(store: &dyn Store, change: &str, slug: &str) -> Result<()> {
    let Some(mut meta) = store.read_change_meta(change) else {
        return Ok(());
    };
    // Change meta is bare YAML (no `---` frontmatter fence), so parse via ChangeMeta
    // like `link`/`stamp_restale` do — `frontmatter_value` only reads discussion docs.
    // 深度防禦：唯一呼叫者 seal 已對壞 metadata 守門，此處到達即應可解析；
    // 萬一未來新增未守門的呼叫者，fail closed 而非靜默疊寫。
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    let existing = match parsed.restale_from.as_deref().map(str::trim) {
        Some(e) if !e.is_empty() => e.to_string(),
        _ => return Ok(()), // no restale_from — nothing to clear
    };
    let current: Vec<&str> =
        existing.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let remaining: Vec<&str> = current.iter().copied().filter(|s| *s != slug).collect();
    if remaining.len() == current.len() {
        return Ok(()); // slug not present — idempotent no-op, no write
    }
    if remaining.is_empty() {
        meta = meta.replacen(&format!("restale_from: {existing}\n"), "", 1);
    } else {
        meta = meta.replacen(
            &format!("restale_from: {existing}"),
            &format!("restale_from: {}", remaining.join(", ")),
            1,
        );
    }
    store.write_change_meta(change, &meta)?;
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
    store: &dyn Store,
    slug: &str,
    name: Option<&str>,
    actor: Option<&str>,
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
        crate::config::WorkflowConfig::from_text(store.read_workflow_config().as_deref())?
            .schema_name();
    let dir = crate::newcmd::new_change(store, &change_name, None, &schema, None, Some(slug), actor)?;
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

/// Link a discussion to an EXISTING change — the ingest-side counterpart of
/// `promote` (which scaffolds a new change). Forges ONLY the change-side chain:
/// `from_discussion` in the change metadata. Marking the discussion promoted is
/// NOT done here — that reflection is sealed by [`seal`] once ingest has folded
/// the discussion's content in, so a linked-but-unfilled change never reads as
/// "已轉出". The discussion record is left byte-identical by this call. Archive
/// co-travel still engages: it is driven by the change-side `from_discussion`,
/// not by the discussion's status.
/// The discussion↔change relationship is many-to-many: a change already born of
/// one discussion can be re-linked to a later one (an ingest that revisits an
/// earlier decision), so `from_discussion` is a comma-separated accumulator that
/// appends rather than rejecting. Guards run before any write (a rejection leaves
/// the change meta byte-identical); re-linking the same pair is an idempotent
/// success that skips the change-side write.
pub fn link(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to link it")
        }
        Some(_) => {}
    }
    let Some(mut meta) = store.read_change_meta(change) else {
        bail!("Change '{change}' not found.");
    };
    // Fail-closed gate: corrupt metadata must not read as "no source
    // discussion" and take the from_discussion append.
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    let existing = parsed.from_discussion.as_deref().map(str::trim).unwrap_or("");
    if parsed.from_discussions().iter().any(|s| s == slug) {
        // chain already forged for this slug — idempotent, skip the change-side write
    } else if existing.is_empty() {
        // no source discussion yet — add the line (tolerating a missing trailing newline)
        if !meta.ends_with('\n') && !meta.is_empty() {
            meta.push('\n');
        }
        meta.push_str(&format!("from_discussion: {slug}\n"));
        store.write_change_meta(change, &meta)?;
    } else {
        // already born of another discussion — append this slug to the comma list
        meta = meta.replacen(
            &format!("from_discussion: {existing}"),
            &format!("from_discussion: {existing}, {slug}"),
            1,
        );
        store.write_change_meta(change, &meta)?;
    }
    Ok(())
}

/// 內容落地後的封印：把討論標記已轉出（status: promoted、promoted_to 累加變更名）。
/// `link` 只鑄變更側鏈、不再翻狀態——「標記已轉出」的職責移交本動詞，由 ingest 於
/// artifacts 落地完成時呼叫。前置守衛全數通過方寫入：討論存在且未封存、變更存在、且
/// 變更 meta 的 from_discussion 清單已含該 slug（鏈須先由 link／promote／new change
/// 鑄妥）。守衛失敗回可記錄的 Err，兩側檔案逐位元不變。冪等：promoted_to 已含該變更名
/// 時 `mark_promoted` 改寫等值內容。
pub fn seal(store: &dyn Store, slug: &str, change: &str) -> Result<()> {
    match info(store, slug) {
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
        Some(i) if i.archived => {
            bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to seal it")
        }
        Some(_) => {}
    }
    let Some(meta) = store.read_change_meta(change) else {
        bail!("Change '{change}' not found.");
    };
    // Fail-closed gate: a corrupt document must report itself, not a missing
    // from_discussion chain.
    let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).map_err(|reason| {
        crate::model::MetaError { change: change.to_string(), reason }
    })?;
    if !parsed
        .from_discussions()
        .iter()
        .any(|s| s == slug)
    {
        bail!("Change '{change}' is not linked to discussion '{slug}' — run `speclink discuss link` first.");
    }
    mark_promoted(store, slug, change)?;
    // Sealing is the honest "content landed" act: clear this discussion's re-ingest flag
    // from the change (the seal-side inverse of the conclude-time stamp). Per-slug — a
    // change stale against another discussion keeps that slug pending its own re-seal.
    clear_restale(store, change, slug)
}

/// 討論卡的看板欄內排序鍵（frontmatter 的 `board_rank`）。沿 `promoted_to` 的
/// 同款模式：獨立讀取函式、不進 `DiscussionInfo`，`discuss list --json` 逐位元不變。
pub fn board_rank(store: &dyn Store, slug: &str) -> Option<String> {
    let doc = store.read_discussion(slug)?;
    frontmatter_value(&doc.text, "board_rank").filter(|v| !v.is_empty())
}

/// 寫入（或原位更新）一筆 live 討論的看板排序鍵：既有 `board_rank:` 行原位代換，
/// 否則插入 frontmatter 尾端（closing `---` 前）；其餘內容逐位元組保留。
/// 非法 rank、封存或不存在的討論皆回明確錯誤（封存記錄不上看板）。
pub fn set_board_rank(store: &dyn Store, slug: &str, rank: &str) -> Result<()> {
    if !crate::util::is_valid_board_rank(rank) {
        bail!("invalid board rank '{rank}' — lowercase ASCII letters only");
    }
    let text = load_live(store, slug)?;
    let line = format!("board_rank: {rank}\n");
    let mut out = String::with_capacity(text.len() + line.len());
    let mut state = 0u8; // 0＝等開頭 ---、1＝frontmatter 內、2＝frontmatter 後
    let mut done = false;
    for (i, l) in text.split_inclusive('\n').enumerate() {
        match state {
            0 => {
                out.push_str(l);
                state = if i == 0 && l.trim_end() == "---" { 1 } else { 2 };
            }
            1 => {
                if l.trim_end() == "---" {
                    if !done {
                        out.push_str(&line);
                        done = true;
                    }
                    out.push_str(l);
                    state = 2;
                } else if !done && l.starts_with("board_rank:") {
                    out.push_str(&line);
                    done = true;
                } else {
                    out.push_str(l);
                }
            }
            _ => out.push_str(l),
        }
    }
    if !done {
        bail!("discussion '{slug}' has no frontmatter — cannot set board rank");
    }
    store.write_live_discussion(slug, &out)?;
    Ok(())
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
        // Typed refusal: same frozen text, classified `refused` by the command layer.
        return Err(crate::command::Refusal(format!(
            "discussion '{slug}' has {rounds} recorded round(s) — `conclude` + `archive` keeps the reasoning; pass --force to delete anyway"
        ))
        .into());
    }
    store.delete_live_discussion(slug)?;
    Ok(())
}

/// Write the conclusion into the `## Conclusion` section (replacing the placeholder — or a
/// previous conclusion, so a revised conclusion stays a single section) and mark the
/// discussion concluded.
pub fn conclude(store: &dyn Store, slug: &str, content: &str) -> Result<Vec<String>> {
    ensure_content(content)?;
    let mut text = load_live(store, slug)?;
    // Flip status: open -> concluded in frontmatter. A promoted discussion (status:
    // promoted) has no "status: open" to match, so a re-conclude preserves promoted.
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
    // Re-concluding an already-reflected discussion (promoted_to non-empty) flags each
    // of its active changes as stale against the new conclusion. Returns the flagged
    // change names for the CLI to report; empty when nothing was reflected yet.
    stamp_restale(store, slug, &text)
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

    // --- 空內容 guard（discuss-content-guard；拒絕靜默寫入空區段） ---

    #[test]
    fn add_round_rejects_empty_content() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::add_round(&store, "alpha", "assumptions", "").is_err());
        assert!(super::add_round(&store, "alpha", "assumptions", "   \n\t ").is_err());
        assert_eq!(store.discussion("alpha"), doc, "空內容不得改動記錄");
    }

    #[test]
    fn conclude_rejects_empty_content_and_keeps_status() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::conclude(&store, "alpha", "").is_err());
        assert!(super::conclude(&store, "alpha", "  \n ").is_err());
        assert_eq!(
            store.discussion("alpha"),
            doc,
            "空 conclude 不得翻狀態或改動記錄"
        );
    }

    #[test]
    fn set_context_rejects_empty_content() {
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        assert!(super::set_context(&store, "alpha", "").is_err());
        assert!(super::set_context(&store, "alpha", "   ").is_err());
        assert_eq!(store.discussion("alpha"), doc, "空內容不得覆寫 Context");
    }

    // --- board_rank（看板排序欄位；desktop-card-reorder） ---

    #[test]
    fn board_rank_reads_frontmatter_only() {
        // 讀取限 frontmatter：本文出現「board_rank:」字樣不得誤讀。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        assert!(super::board_rank(&store, "alpha").is_none());

        let with_rank = open_doc("alpha", "Alpha")
            .replacen("status: open\n", "status: open\nboard_rank: n\n", 1);
        let store2 = TestStore::with_live_discussion("alpha", &with_rank);
        assert_eq!(super::board_rank(&store2, "alpha").as_deref(), Some("n"));

        let body_decoy = open_doc("alpha", "Alpha") + "\nboard_rank: fake\n";
        let store3 = TestStore::with_live_discussion("alpha", &body_decoy);
        assert!(super::board_rank(&store3, "alpha").is_none());
    }

    #[test]
    fn set_board_rank_inserts_into_frontmatter_preserving_rest_verbatim() {
        // spec「meta 寫入路徑對 board_rank 互不破壞」討論側：插入 frontmatter
        // 尾端（closing --- 前），其餘內容逐位元組不變。
        let doc = open_doc("alpha", "Alpha");
        let store = TestStore::with_live_discussion("alpha", &doc);
        super::set_board_rank(&store, "alpha", "n").unwrap();
        let expected = doc.replacen(
            "created: 2026-01-02\n---\n",
            "created: 2026-01-02\nboard_rank: n\n---\n",
            1,
        );
        assert_eq!(store.discussion("alpha"), expected);
    }

    #[test]
    fn set_board_rank_replaces_existing_frontmatter_line_in_place() {
        let doc = open_doc("alpha", "Alpha")
            .replacen("status: open\n", "status: open\nboard_rank: b\n", 1);
        let store = TestStore::with_live_discussion("alpha", &doc);
        super::set_board_rank(&store, "alpha", "abn").unwrap();
        assert_eq!(
            store.discussion("alpha"),
            doc.replacen("board_rank: b\n", "board_rank: abn\n", 1)
        );
    }

    #[test]
    fn set_board_rank_rejects_invalid_values_and_non_live_records() {
        // 值驗證同變更側（僅小寫英文字母）；封存記錄不上看板、不可寫。
        let store = TestStore::with_live_discussion("alpha", &open_doc("alpha", "Alpha"));
        for bad in ["", "N", "a1", "a b", "a\nstatus: forged"] {
            assert!(
                super::set_board_rank(&store, "alpha", bad).is_err(),
                "invalid rank {bad:?} must be rejected"
            );
        }
        assert_eq!(store.discussion("alpha"), open_doc("alpha", "Alpha"), "no write on reject");

        let archived = TestStore::default();
        archived
            .archived_discussions
            .borrow_mut()
            .insert("old".to_string(), concluded_doc("old", "Old", "done"));
        assert!(super::set_board_rank(&archived, "old", "n").is_err());
        assert!(super::set_board_rank(&archived, "ghost", "n").is_err());
    }

    #[test]
    fn discussion_info_json_is_unchanged_by_board_rank() {
        // spec「board_rank 不進 CLI 輸出且既有輸出逐位元不變」討論側：
        // DiscussionInfo 不攜帶 rank（沿 promoted_to 的獨立讀取模式），
        // 含 rank 的記錄序列化結果與無 rank 時逐位元一致。
        let doc = open_doc("alpha", "Alpha");
        let with_rank = doc.replacen("status: open\n", "status: open\nboard_rank: n\n", 1);
        let info_of = |text: &str| {
            serde_json::to_string(&super::info_from_doc(&crate::store::DiscussionDoc {
                slug: "alpha".to_string(),
                text: text.to_string(),
                path: std::path::PathBuf::from("discussions/alpha.md"),
                archived: false,
            }))
            .unwrap()
        };
        let ranked_json = info_of(&with_rank);
        assert_eq!(ranked_json, info_of(&doc), "board_rank must not affect discuss list --json");
        assert!(!ranked_json.contains("board_rank") && !ranked_json.contains("boardRank"));
    }

    // --- promote flow (design D1) ---

    #[test]
    fn promote_rejects_missing_discussion() {
        let store = TestStore::default();
        let err = super::promote(&store, "ghost", None, None).unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
    }

    #[test]
    fn promote_rejects_archived_discussion() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old-topic".to_string(), concluded_doc("old-topic", "Old", "done"));
        let err = super::promote(&store, "old-topic", None, None).unwrap_err();
        assert!(err.to_string().contains("archived"), "err: {err}");
        assert!(!store.change_exists("old-topic"), "no change may be created");
    }

    #[test]
    fn promote_derives_change_name_from_slug_by_default() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        let outcome = super::promote(&store, "alpha-search", None, None).unwrap();
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
            super::promote(&store, "beta-cache", Some("cache-layer"), None).unwrap();
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
        let outcome = super::promote(&store, "2026-07-06-retro", None, None).unwrap();
        assert_eq!(outcome.change, "retro");

        let store2 = TestStore::with_live_discussion(
            "gamma-x",
            &concluded_doc("gamma-x", "Gamma x", "ship gamma"),
        );
        let outcome2 =
            super::promote(&store2, "gamma-x", Some("2026-01-02-gamma-cut"), None).unwrap();
        assert_eq!(outcome2.change, "gamma-cut");
    }

    #[test]
    fn promote_creates_change_with_from_discussion_meta() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&store, "alpha-search", None, None).unwrap();
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
        super::promote(&store, "alpha-search", None, None).unwrap();
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
        super::promote(&store, "open-one", None, None).unwrap();
        let proposal = store.read_artifact("open-one", "proposal.md").unwrap();
        assert!(proposal.starts_with("## Why\n\nOpen topic\n"), "proposal: {proposal}");
    }

    #[test]
    fn promote_marks_promoted_and_accumulates_on_fan_out() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        super::promote(&store, "alpha-search", None, None).unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("status: promoted\n"), "text: {text}");
        assert!(text.contains("promoted_to: alpha-search\n"), "text: {text}");

        // Second cut: promoted_to becomes a comma-separated accumulator.
        super::promote(&store, "alpha-search", Some("second-cut"), None).unwrap();
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
        let err = super::promote(&store, "alpha-search", None, None).unwrap_err();
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

    // --- unlink on discard（spec「討論隨變更廢棄解鏈」；design D2） ---

    /// concluded_doc 提升為 promoted，promoted_to 設為指定清單。
    fn promoted_concluded(slug: &str, topic: &str, decision: &str, to: &str) -> String {
        concluded_doc(slug, topic, decision).replacen(
            "status: concluded\n",
            &format!("status: promoted\npromoted_to: {to}\n"),
            1,
        )
    }

    #[test]
    fn unlink_reverts_to_concluded_when_last_link_dies() {
        // spec Example「回退前後的 frontmatter」＋「最後連結死亡回退 concluded」：
        // 唯一值移除 → promoted_to 行消失、status 回 concluded；Context/Rounds/Conclusion
        // 逐位元不變（回退後 == 原 concluded 記錄）。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "build alpha search", "cut-a"),
        );
        let reverted = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(reverted.as_deref(), Some("concluded"));
        assert_eq!(
            store.discussion("alpha-search"),
            concluded_doc("alpha-search", "Alpha search", "build alpha search"),
            "promoted_to 行消失、status 回 concluded、其餘逐位元不變"
        );
    }

    #[test]
    fn unlink_shrinks_list_and_keeps_promoted_when_others_remain() {
        // spec「仍有其他變更時維持 promoted」：多值僅縮減、status 維持 promoted。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a, cut-b"),
        );
        let reverted = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(reverted.as_deref(), Some("promoted"));
        assert_eq!(
            store.discussion("alpha-search"),
            promoted_concluded("alpha-search", "Alpha search", "x", "cut-b"),
            "移除 cut-a、保留 cut-b、status 維持 promoted"
        );
    }

    #[test]
    fn unlink_reverts_to_open_when_no_conclusion() {
        // spec「無結論的討論回退 open」：Conclusion 為空的 open 討論經 link 後廢棄 → 回 open。
        let raised = open_doc("open-one", "Open topic").replacen(
            "status: open\n",
            "status: promoted\npromoted_to: cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("open-one", &raised);
        let reverted = super::unlink_discarded(&store, "open-one", "cut").unwrap();
        assert_eq!(reverted.as_deref(), Some("open"));
        assert_eq!(
            store.discussion("open-one"),
            open_doc("open-one", "Open topic"),
            "promoted_to 行消失、status 回 open、其餘逐位元不變"
        );
    }

    #[test]
    fn unlink_skips_missing_record_without_error() {
        // spec「缺失記錄跳過」：無 live 記錄（不存在或僅存於 archive）→ Ok(None)、不失敗。
        let empty = TestStore::default();
        assert_eq!(super::unlink_discarded(&empty, "ghost", "cut").unwrap(), None);

        let archived = TestStore::default();
        archived
            .archived_discussions
            .borrow_mut()
            .insert("old".into(), promoted_concluded("old", "Old", "x", "cut"));
        assert_eq!(super::unlink_discarded(&archived, "old", "cut").unwrap(), None);
    }

    #[test]
    fn unlink_is_idempotent_on_already_unlinked_record() {
        // spec「對已解鏈的討論重跑冪等」：重跑對 promoted_to 已無該名的記錄 → Ok(None)、不改檔。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a"),
        );
        super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        let after_first = store.discussion("alpha-search");
        let rerun = super::unlink_discarded(&store, "alpha-search", "cut-a").unwrap();
        assert_eq!(rerun, None, "已解鏈記錄重跑不回報狀態");
        assert_eq!(store.discussion("alpha-search"), after_first, "重跑不改檔");
    }

    #[test]
    fn unlink_ignores_a_change_that_was_never_linked() {
        // 冪等的另一面：promoted_to 有值但不含目標變更名 → 不動、不失敗、不回報。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "cut-a, cut-b"),
        );
        let before = store.discussion("alpha-search");
        assert_eq!(super::unlink_discarded(&store, "alpha-search", "cut-z").unwrap(), None);
        assert_eq!(store.discussion("alpha-search"), before);
    }

    // --- link flow（spec「討論以 link 動詞併入既有變更」；design D1–D4） ---

    #[test]
    fn link_writes_change_meta_and_leaves_discussion_untouched() {
        // link 只鑄變更側鏈：變更 meta 增寫 from_discussion，討論記錄逐位元不變
        // （「已轉出」標記移交 seal）。
        let doc = concluded_doc("alpha-search", "Alpha search", "build alpha search");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store
            .metas
            .borrow_mut()
            .insert("existing-cut".into(), "schema: spec-driven\ncreated: 2026-01-03\n".into());
        super::link(&store, "alpha-search", "existing-cut").unwrap();
        let meta = store.meta("existing-cut");
        assert!(meta.contains("from_discussion: alpha-search\n"), "meta: {meta}");
        assert_eq!(store.discussion("alpha-search"), doc, "討論逐位元不變（link 不再標記 promoted）");
    }

    #[test]
    fn link_accepts_open_discussion_without_marking() {
        // 前置條件與 promote 一致：open 討論也可併入；但 link 不翻狀態，討論仍 open。
        let doc = open_doc("open-one", "Open topic");
        let store = TestStore::with_live_discussion("open-one", &doc);
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        super::link(&store, "open-one", "cut").unwrap();
        assert!(store.meta("cut").contains("from_discussion: open-one\n"));
        assert_eq!(store.discussion("open-one"), doc, "討論仍 open、逐位元不變");
    }

    #[test]
    fn link_rejects_missing_discussion_without_writes() {
        let store = TestStore::default();
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::link(&store, "ghost", "cut").unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_rejects_archived_discussion_without_writes() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old-topic".into(), concluded_doc("old-topic", "Old", "done"));
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::link(&store, "old-topic", "cut").unwrap_err();
        assert!(err.to_string().contains("archived"), "err: {err}");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_rejects_missing_change_without_discussion_write() {
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        let err = super::link(&store, "alpha-search", "no-such-change").unwrap_err();
        assert!(err.to_string().contains("not found"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion must be untouched");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn link_appends_to_from_discussion_when_change_already_linked() {
        // spec「出身自討論的變更再併入新討論」：change meta 的 from_discussion 於既有值
        // 尾端累加本 slug、既有值保留；本討論標 promoted；先前連結的討論記錄逐位元不變。
        let doc = concluded_doc("beta-cache", "Beta cache", "x");
        let store = TestStore::with_live_discussion("beta-cache", &doc);
        let other = concluded_doc("other-topic", "Other", "y").replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: cut\n",
            1,
        );
        store.discussions.borrow_mut().insert("other-topic".into(), other.clone());
        store
            .metas
            .borrow_mut()
            .insert("cut".into(), "schema: spec-driven\nfrom_discussion: other-topic\n".into());
        super::link(&store, "beta-cache", "cut").unwrap();
        let meta = store.meta("cut");
        assert!(meta.contains("from_discussion: other-topic, beta-cache\n"), "meta: {meta}");
        assert_eq!(store.discussion("beta-cache"), doc, "本討論逐位元不變（link 不標記）");
        assert_eq!(store.discussion("other-topic"), other, "prior discussion untouched");
    }

    #[test]
    fn link_is_idempotent_when_slug_already_in_from_discussion_list() {
        // spec「同一組合重跑為冪等」（該討論僅為 from_discussion 清單其中一員）：
        // change 側不再寫、討論側改寫等值內容。
        let store = TestStore::with_live_discussion(
            "beta-cache",
            &concluded_doc("beta-cache", "Beta cache", "x"),
        );
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search, beta-cache\n".into(),
        );
        super::link(&store, "beta-cache", "cut").unwrap();
        let meta_after = store.meta("cut");
        let writes_after = *store.meta_writes.borrow();
        super::link(&store, "beta-cache", "cut").unwrap();
        assert_eq!(store.meta("cut"), meta_after, "meta must be unchanged");
        assert!(
            meta_after.contains("from_discussion: alpha-search, beta-cache\n"),
            "existing list preserved, not appended: {meta_after}"
        );
        assert_eq!(
            *store.meta_writes.borrow(),
            writes_after,
            "change side must not rewrite when slug already present"
        );
    }

    #[test]
    fn link_rejects_corrupt_change_meta_without_writes() {
        // spec「link 對壞 metadata 拒絕且兩側皆不寫」：壞檔不得被解讀為
        // 「無 from_discussion 鏈」而追加行——兩側檔案逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("broken-cut".into(), BAD.into());
        let err = super::link(&store, "alpha-search", "broken-cut").unwrap_err();
        assert!(
            err.to_string().contains("openspec/changes/broken-cut/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(store.meta("broken-cut"), BAD, "change meta byte-identical");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    // --- seal flow（spec「內容落地以 seal 動詞標記已轉出」） ---

    #[test]
    fn seal_marks_promoted_when_chain_forged() {
        // 鏈已鑄妥（變更 meta 含 from_discussion: slug）→ 討論翻 promoted、累加 promoted_to。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "build alpha search"),
        );
        store.metas.borrow_mut().insert(
            "existing-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        super::seal(&store, "alpha-search", "existing-cut").unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("status: promoted\n"), "text: {text}");
        assert!(text.contains("promoted_to: existing-cut\n"), "text: {text}");
    }

    #[test]
    fn seal_rejects_when_chain_not_forged_without_writes() {
        // 變更存在但 meta 的 from_discussion 未含該 slug → 拒絕、兩側逐位元不變。
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        let err = super::seal(&store, "alpha-search", "cut").unwrap_err();
        assert!(err.to_string().contains("not linked"), "err: {err}");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion untouched");
        assert_eq!(store.meta("cut"), "schema: spec-driven\n", "change meta untouched");
    }

    #[test]
    fn seal_rejects_corrupt_change_meta_not_misreporting_the_chain() {
        // spec「seal 對壞 metadata 拒絕且不誤報鏈缺失」：錯誤指出 metadata
        // 損壞（而非 from_discussion 不含該 slug）；兩側檔案逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let doc = concluded_doc("alpha-search", "Alpha search", "x");
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert("broken-cut".into(), BAD.into());
        let err = super::seal(&store, "alpha-search", "broken-cut").unwrap_err();
        assert!(
            err.to_string().contains("openspec/changes/broken-cut/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert!(
            !err.to_string().contains("not linked"),
            "must not misreport a missing chain: {err}"
        );
        assert_eq!(store.meta("broken-cut"), BAD, "change meta byte-identical");
        assert_eq!(store.discussion("alpha-search"), doc, "discussion byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn seal_rejects_missing_discussion_and_missing_change() {
        // 討論不存在。
        let store = TestStore::default();
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: ghost\n".into(),
        );
        assert!(super::seal(&store, "ghost", "cut").unwrap_err().to_string().contains("not found"));
        // 變更不存在（討論存在）。
        let store2 =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "x"));
        assert!(super::seal(&store2, "alpha", "no-such-change").unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn seal_rejects_archived_discussion() {
        let store = TestStore::default();
        store
            .archived_discussions
            .borrow_mut()
            .insert("old".into(), concluded_doc("old", "Old", "x"));
        store.metas.borrow_mut().insert(
            "cut".into(),
            "schema: spec-driven\nfrom_discussion: old\n".into(),
        );
        assert!(super::seal(&store, "old", "cut").unwrap_err().to_string().contains("archived"));
    }

    #[test]
    fn seal_is_idempotent_when_already_promoted() {
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &promoted_concluded("alpha-search", "Alpha search", "x", "existing-cut"),
        );
        store.metas.borrow_mut().insert(
            "existing-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        let before = store.discussion("alpha-search");
        super::seal(&store, "alpha-search", "existing-cut").unwrap();
        assert_eq!(store.discussion("alpha-search"), before, "重跑不改檔");
    }

    #[test]
    fn link_same_pair_is_idempotent() {
        // spec「同一組合重跑為冪等」：Ok、兩側內容逐位元不變、變更側不再寫。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "x"),
        );
        store.metas.borrow_mut().insert("cut".into(), "schema: spec-driven\n".into());
        super::link(&store, "alpha-search", "cut").unwrap();
        let meta_after = store.meta("cut");
        let doc_after = store.discussion("alpha-search");
        let writes_after = *store.meta_writes.borrow();
        super::link(&store, "alpha-search", "cut").unwrap();
        assert_eq!(store.meta("cut"), meta_after);
        assert_eq!(store.discussion("alpha-search"), doc_after);
        assert_eq!(*store.meta_writes.borrow(), writes_after, "change side must not rewrite");
    }

    #[test]
    fn seal_accumulates_promoted_to_on_fan_out() {
        // spec「promoted_to 逗號累加、既有值保留」：已 promoted 的討論再經 seal 併入
        // 另一變更（fan-out 累加現由 seal 承接，非 link）。
        let doc = concluded_doc("alpha-search", "Alpha search", "x").replacen(
            "status: concluded\n",
            "status: promoted\npromoted_to: first-cut\n",
            1,
        );
        let store = TestStore::with_live_discussion("alpha-search", &doc);
        store.metas.borrow_mut().insert(
            "second-cut".into(),
            "schema: spec-driven\nfrom_discussion: alpha-search\n".into(),
        );
        super::seal(&store, "alpha-search", "second-cut").unwrap();
        let text = store.discussion("alpha-search");
        assert!(text.contains("promoted_to: first-cut, second-cut\n"), "text: {text}");
    }

    #[test]
    fn link_tolerates_meta_without_trailing_newline() {
        // meta 讀-改-寫的尾換行容錯（inprogress 同款模式）。
        let store = TestStore::with_live_discussion(
            "alpha-search",
            &concluded_doc("alpha-search", "Alpha search", "x"),
        );
        store
            .metas
            .borrow_mut()
            .insert("cut".into(), "schema: spec-driven\ncreated: 2026-01-03".into());
        super::link(&store, "alpha-search", "cut").unwrap();
        let meta = store.meta("cut");
        assert!(
            meta.contains("created: 2026-01-03\nfrom_discussion: alpha-search\n"),
            "meta: {meta}"
        );
    }

    // --- discuss new：slug 覆寫與後備衍生（spec「討論記錄以 --slug 覆寫檔名」「未帶 --slug 時自主題衍生檔名」） ---

    #[test]
    fn new_discussion_rejects_invalid_slug_override() {
        // spec「非法值一覽」Example 表：大寫、非 ASCII、底線、空白、首尾連字號、連續連字號、空字串。
        let store = TestStore::default();
        for bad in [
            "Board-Search",
            "看板搜尋",
            "board_search",
            "board search",
            "-board",
            "board-",
            "board--search",
            "",
        ] {
            let err = super::new_discussion(&store, "看板搜尋列", Some(bad), None).unwrap_err();
            assert!(err.to_string().contains("kebab-case"), "slug {bad:?} err: {err}");
        }
        assert!(store.list_live_discussions().is_empty(), "invalid slug must not create files");
    }

    #[test]
    fn new_discussion_accepts_valid_slug_override_and_keeps_topic() {
        let store = TestStore::default();
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-2"), None).unwrap();
        assert_eq!(info.slug, "board-search-2");
        assert_eq!(info.topic, "看板搜尋列");
        let text = store
            .read_live_discussion("board-search-2")
            .expect("record stored under override slug");
        assert!(text.contains("slug: board-search-2\n"), "text: {text}");
        assert!(text.contains("topic: 看板搜尋列\n"), "text: {text}");
    }

    #[test]
    fn new_discussion_slug_override_conflicts_with_existing() {
        let store = TestStore::with_live_discussion("taken", &open_doc("taken", "Taken"));
        let before = store.discussion("taken");
        let err = super::new_discussion(&store, "另一個主題", Some("taken"), None).unwrap_err();
        assert!(err.to_string().contains("already exists"), "err: {err}");
        assert_eq!(store.discussion("taken"), before, "existing record must not be overwritten");
    }

    #[test]
    fn new_discussion_fallback_derivation_is_unchanged() {
        // spec「衍生規則對照」Example 表：後備行為與本變更前逐位元一致。
        for (topic, want) in [
            ("Board Search", "board-search"),
            ("config context 與 rules GUI 編輯", "config-context-與-rules-gui-編輯"),
            ("看板 搜尋列", "看板-搜尋列"),
        ] {
            let store = TestStore::default();
            let info = super::new_discussion(&store, topic, None, None).unwrap();
            assert_eq!(info.slug, want, "topic: {topic}");
            assert_eq!(info.topic, topic);
            assert!(store.read_live_discussion(want).is_some(), "file under derived slug");
        }
        // 純 ASCII 標點主題衍生為空 → 報錯。
        let store = TestStore::default();
        let err = super::new_discussion(&store, "!?!", None, None).unwrap_err();
        assert!(err.to_string().contains("could not derive"), "err: {err}");
    }

    // --- discuss new：蓋建立者章（spec「討論記錄蓋建立者章」） ---

    #[test]
    fn new_discussion_stamps_created_by_when_identity_present() {
        let store = TestStore::default();
        let id = "Base Line <base@example.com>";
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-3"), Some(id)).unwrap();
        // frontmatter 蓋 created_by、且 DiscussionInfo（→ --json createdBy）帶同值。
        let text = store.read_live_discussion("board-search-3").expect("record stored");
        assert!(text.contains(&format!("created_by: {id}\n")), "frontmatter: {text}");
        assert_eq!(info.created_by.as_deref(), Some(id));
    }

    #[test]
    fn new_discussion_omits_created_by_when_identity_absent() {
        let store = TestStore::default();
        let info = super::new_discussion(&store, "看板搜尋列", Some("board-search-4"), None).unwrap();
        // 無身分：frontmatter 不含 created_by、createdBy 缺席。
        let text = store.read_live_discussion("board-search-4").expect("record stored");
        assert!(!text.contains("created_by:"), "frontmatter should omit created_by: {text}");
        assert_eq!(info.created_by, None);
    }

    // --- restale flag：conclude 蓋章 / seal 清除（reconclude-restale） ---

    /// A promoted discussion (status: promoted, promoted_to set) with a written conclusion.
    fn promoted_doc(slug: &str, topic: &str, promoted_to: &str, decision: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: promoted\npromoted_to: {promoted_to}\ncreated: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n### Round 1 — assumptions (2026-01-02)\n\n**Focus**: scope\n\n\
             ## Conclusion\n\n**Decision**: {decision}\n"
        )
    }

    #[test]
    fn conclude_stamps_restale_on_active_promoted_change() {
        let store =
            TestStore::with_live_discussion("alpha", &promoted_doc("alpha", "Alpha", "cut-a", "old"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\n".to_string(),
        );
        let flagged = super::conclude(&store, "alpha", "**Decision**: new direction").unwrap();
        assert_eq!(flagged, vec!["cut-a".to_string()]);
        assert!(store.meta("cut-a").contains("restale_from: alpha"), "meta: {}", store.meta("cut-a"));
        // 討論維持 promoted、promoted_to 不變，僅 Conclusion 改寫。
        let disc = store.discussion("alpha");
        assert!(disc.contains("status: promoted\n"), "stays promoted: {disc}");
        assert!(disc.contains("promoted_to: cut-a\n"), "promoted_to intact: {disc}");
        assert!(disc.contains("**Decision**: new direction"), "conclusion rewritten: {disc}");
    }

    #[test]
    fn conclude_restale_skips_archived_change() {
        // promoted_to 同含 active 與已歸檔變更；僅 active 被蓋。
        let store = TestStore::with_live_discussion(
            "alpha",
            &promoted_doc("alpha", "Alpha", "cut-a, arch-b", "old"),
        );
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        // arch-b 僅存於封存（read_change_meta 回 None）——非 active。
        store
            .archived_metas
            .borrow_mut()
            .insert("arch-b".to_string(), "schema: spec-driven\n".to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new").unwrap();
        assert_eq!(flagged, vec!["cut-a".to_string()], "only active flagged");
        assert!(store.meta("cut-a").contains("restale_from: alpha"));
        assert!(!store.change_exists("arch-b"), "archived never active");
        assert_eq!(
            store.archived_metas.borrow().get("arch-b").unwrap(),
            "schema: spec-driven\n",
            "archived meta untouched"
        );
    }

    #[test]
    fn conclude_restale_skips_corrupt_meta_change_without_writing() {
        // fail-closed 掃尾：promoted_to 指向的 change metadata 損壞時跳過該卡
        // （沿 archived/gone 的 skip 原則——單一壞檔不得使 conclude 中止），
        // 壞檔逐位元不變、其餘 active change 照常蓋章。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_live_discussion(
            "alpha",
            &promoted_doc("alpha", "Alpha", "cut-a, broken-b", "old"),
        );
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        store.metas.borrow_mut().insert("broken-b".to_string(), BAD.to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new").unwrap();
        assert_eq!(flagged, vec!["cut-a".to_string()], "corrupt change is not flagged");
        assert!(store.meta("cut-a").contains("restale_from: alpha"));
        assert_eq!(store.meta("broken-b"), BAD, "corrupt meta must not be appended to");
    }

    #[test]
    fn conclude_promoted_to_empty_stamps_nothing() {
        // concluded-but-not-promoted：promoted_to 缺席 → 不蓋章。
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "old"));
        store.metas.borrow_mut().insert("cut-a".to_string(), "schema: spec-driven\n".to_string());
        let flagged = super::conclude(&store, "alpha", "**Decision**: new").unwrap();
        assert!(flagged.is_empty());
        assert_eq!(*store.meta_writes.borrow(), 0, "no change meta written");
        assert_eq!(store.meta("cut-a"), "schema: spec-driven\n", "change meta untouched");
    }

    #[test]
    fn conclude_restale_stamp_is_idempotent() {
        let store =
            TestStore::with_live_discussion("alpha", &promoted_doc("alpha", "Alpha", "cut-a", "old"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nrestale_from: alpha\n".to_string(),
        );
        let before = store.meta("cut-a");
        let flagged = super::conclude(&store, "alpha", "**Decision**: newer").unwrap();
        assert_eq!(flagged, vec!["cut-a".to_string()], "still reported stale");
        assert_eq!(store.meta("cut-a"), before, "no duplicate accumulation");
        assert_eq!(*store.meta_writes.borrow(), 0, "idempotent — no change meta write");
    }

    #[test]
    fn seal_clears_restale_slug_keeping_others() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\nrestale_from: alpha, beta\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        let meta = store.meta("cut-a");
        assert!(meta.contains("restale_from: beta\n"), "alpha cleared, beta kept: {meta}");
        assert!(!meta.contains("restale_from: alpha"), "alpha gone: {meta}");
        assert!(store.discussion("alpha").contains("status: promoted\n"), "sealed → promoted");
    }

    #[test]
    fn seal_clears_restale_line_when_last_slug() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\nrestale_from: alpha\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        let meta = store.meta("cut-a");
        assert!(!meta.contains("restale_from"), "restale_from line dropped: {meta}");
        assert!(meta.contains("from_discussion: alpha\n"), "other fields intact: {meta}");
    }

    #[test]
    fn seal_restale_clear_is_noop_when_absent() {
        let store =
            TestStore::with_live_discussion("alpha", &concluded_doc("alpha", "Alpha", "done"));
        store.metas.borrow_mut().insert(
            "cut-a".to_string(),
            "schema: spec-driven\nfrom_discussion: alpha\n".to_string(),
        );
        super::seal(&store, "alpha", "cut-a").unwrap();
        assert!(!store.meta("cut-a").contains("restale_from"), "no restale_from introduced");
    }
}

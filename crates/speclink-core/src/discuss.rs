//! Discussion documents — speclink's enhancement over Spectra's ephemeral discuss.
//!
//! Each discussion is a single append-only document at `openspec/discussions/<slug>.md` so an
//! iterative conversation accumulates a durable record that `propose` can later consume.
//! Archived discussions move to `openspec/discussions/archive/<created>-<slug>.md` — date-
//! prefixed like `changes/archive/`, so a slug can be reused by a later discussion.

use crate::paths::Paths;
use crate::util;
use anyhow::{bail, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct DiscussionInfo {
    pub slug: String,
    pub topic: String,
    pub status: String,
    pub rounds: usize,
    pub created: String,
    pub path: String,
    pub archived: bool,
}

fn discussion_path(paths: &Paths, slug: &str) -> PathBuf {
    paths.discussions_dir().join(format!("{slug}.md"))
}

fn archive_dir(paths: &Paths) -> PathBuf {
    paths.discussions_dir().join("archive")
}

/// Strip a `YYYY-MM-DD-` prefix from an archived file stem.
fn strip_date_prefix(name: &str) -> Option<&str> {
    let b = name.as_bytes();
    if b.len() > 11
        && b[..4].iter().all(|c| c.is_ascii_digit())
        && b[4] == b'-'
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[7] == b'-'
        && b[8..10].iter().all(|c| c.is_ascii_digit())
        && b[10] == b'-'
    {
        Some(&name[11..])
    } else {
        None
    }
}

/// Whether `s` is a `-N` same-day disambiguation suffix.
fn is_dup_suffix(s: &str) -> bool {
    s.strip_prefix('-')
        .map(|d| !d.is_empty() && d.bytes().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

/// Locate an archived discussion by slug (`archive/<YYYY-MM-DD>-<slug>[-N].md`) — return the
/// newest, ranked by (date prefix, `-N` suffix number; plain name counts as 1).
fn find_archived(paths: &Paths, slug: &str) -> Option<PathBuf> {
    let mut candidates: Vec<(String, u64, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(archive_dir(paths)) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(stem) = name.strip_suffix(".md") else { continue };
            let Some(rest) = strip_date_prefix(stem) else { continue };
            let seq = if rest == slug {
                Some(1)
            } else {
                rest.strip_prefix(slug)
                    .filter(|s| is_dup_suffix(s))
                    .and_then(|s| s[1..].parse::<u64>().ok())
            };
            if let Some(seq) = seq {
                candidates.push((stem[..10].to_string(), seq, path));
            }
        }
    }
    candidates.sort();
    candidates.pop().map(|(_, _, p)| p)
}

/// Resolve a slug to its document: live first, then the archive.
fn resolve_path(paths: &Paths, slug: &str) -> Option<(PathBuf, bool)> {
    let live = discussion_path(paths, slug);
    if live.is_file() {
        return Some((live, false));
    }
    find_archived(paths, slug).map(|p| (p, true))
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

fn read_info_at(path: &PathBuf, slug: &str, archived: bool) -> Option<DiscussionInfo> {
    let text = util::read_opt(path)?;
    Some(DiscussionInfo {
        slug: frontmatter_value(&text, "slug").unwrap_or_else(|| slug.to_string()),
        topic: frontmatter_value(&text, "topic").unwrap_or_else(|| slug.to_string()),
        status: frontmatter_value(&text, "status").unwrap_or_else(|| "open".to_string()),
        rounds: count_rounds(&text),
        created: frontmatter_value(&text, "created").unwrap_or_default(),
        path: util::to_slash(path),
        archived,
    })
}

/// Load a live discussion for mutation; a helpful error distinguishes "archived" from "missing".
fn load_live(paths: &Paths, slug: &str) -> Result<(PathBuf, String)> {
    let path = discussion_path(paths, slug);
    match util::read_opt(&path) {
        Some(t) => Ok((path, t)),
        None => {
            if find_archived(paths, slug).is_some() {
                bail!("discussion '{slug}' is archived — move it out of discussions/archive/ to continue it");
            }
            bail!("discussion '{slug}' not found — run `speclink discuss new` first")
        }
    }
}

/// Create a new discussion document. Errors if a live one already exists.
pub fn new_discussion(paths: &Paths, topic: &str) -> Result<DiscussionInfo> {
    let slug = util::slugify(topic);
    if slug.is_empty() {
        bail!("could not derive a slug from topic '{topic}'");
    }
    let path = discussion_path(paths, &slug);
    if path.exists() {
        bail!("discussion '{slug}' already exists at {}", util::to_slash(&path));
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
    util::write_file(&path, &content)?;
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
pub fn list_discussions(paths: &Paths) -> Vec<DiscussionInfo> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(paths.discussions_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(slug) = name.strip_suffix(".md") else { continue };
            if let Some(info) = read_info_at(&path, slug, false) {
                out.push(info);
            }
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
}

/// List archived discussions (sorted by archived file name, i.e. by archive date).
pub fn list_archived(paths: &Paths) -> Vec<DiscussionInfo> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(archive_dir(paths)) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let Some(stem) = name.strip_suffix(".md") else { continue };
            let slug = strip_date_prefix(stem).unwrap_or(stem);
            if let Some(info) = read_info_at(&path, slug, true) {
                out.push(info);
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

pub fn show_discussion(paths: &Paths, slug: &str) -> Option<String> {
    let (path, _) = resolve_path(paths, slug)?;
    util::read_opt(&path)
}

pub fn info(paths: &Paths, slug: &str) -> Option<DiscussionInfo> {
    let (path, archived) = resolve_path(paths, slug)?;
    read_info_at(&path, slug, archived)
}

/// Set (or replace) the `## Context` section — the one-time framing written after mode pick.
pub fn set_context(paths: &Paths, slug: &str, content: &str) -> Result<()> {
    let (path, text) = load_live(paths, slug)?;
    match replace_section(&text, "Context", content) {
        Some(t) => {
            util::write_file(&path, &t)?;
            Ok(())
        }
        None => bail!(
            "discussion '{slug}' has no '## Context' section (pre-scaffold layout) — edit the file directly"
        ),
    }
}

/// Append a discussion round. Content is supplied verbatim (from the skill via stdin).
pub fn add_round(paths: &Paths, slug: &str, mode: &str, content: &str) -> Result<usize> {
    let (path, mut text) = load_live(paths, slug)?;
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
    util::write_file(&path, &text)?;
    Ok(round_no)
}

/// The text of the `## Conclusion` section, if the discussion has one (the scaffold's
/// placeholder comment does not count as content).
pub fn conclusion_text(paths: &Paths, slug: &str) -> Option<String> {
    let (path, _) = resolve_path(paths, slug)?;
    let text = util::read_opt(&path)?;
    let (s, e) = section_body_range(&text, "Conclusion")?;
    let body = strip_html_comments(&text[s..e]).trim().to_string();
    (!body.is_empty()).then_some(body)
}

/// Mark a discussion as promoted to a change (the discussion side of the bidirectional link).
pub fn mark_promoted(paths: &Paths, slug: &str, change: &str) -> Result<()> {
    let (path, mut text) = load_live(paths, slug)?;
    for from in ["status: open", "status: concluded"] {
        if text.contains(from) {
            text = text.replacen(from, "status: promoted", 1);
            break;
        }
    }
    if !text.contains("\npromoted_to:") {
        text = text.replacen(
            "status: promoted\n",
            &format!("status: promoted\npromoted_to: {change}\n"),
            1,
        );
    }
    util::write_file(&path, &text)?;
    Ok(())
}

/// Move a live discussion to `discussions/archive/<created>-<slug>.md`. Returns the archived
/// file name, or `None` when no live discussion exists. Same-day name collisions get a `-N`
/// suffix so co-archival never fails on a reused slug.
pub fn archive_discussion(paths: &Paths, slug: &str) -> Result<Option<String>> {
    let src = discussion_path(paths, slug);
    if !src.is_file() {
        return Ok(None);
    }
    let created = util::read_opt(&src)
        .and_then(|t| frontmatter_value(&t, "created"))
        .filter(|c| !c.is_empty())
        .unwrap_or_else(util::today);
    let dir = archive_dir(paths);
    std::fs::create_dir_all(&dir)?;
    let base = format!("{created}-{slug}");
    let mut name = format!("{base}.md");
    let mut n = 2;
    while dir.join(&name).exists() {
        name = format!("{base}-{n}.md");
        n += 1;
    }
    std::fs::rename(&src, dir.join(&name))?;
    Ok(Some(name))
}

/// Write the conclusion into the `## Conclusion` section (replacing the placeholder — or a
/// previous conclusion, so a revised conclusion stays a single section) and mark the
/// discussion concluded.
pub fn conclude(paths: &Paths, slug: &str, content: &str) -> Result<()> {
    let (path, mut text) = load_live(paths, slug)?;
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
    util::write_file(&path, &text)?;
    Ok(())
}

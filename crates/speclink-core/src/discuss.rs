//! Discussion documents — speclink's enhancement over Spectra's ephemeral discuss.
//!
//! Each discussion is persisted to `openspec/discussions/<slug>/discussion.md` so an iterative
//! conversation accumulates a durable record that `propose` can later consume.

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
}

fn discussion_path(paths: &Paths, slug: &str) -> PathBuf {
    paths.discussions_dir().join(slug).join("discussion.md")
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
    text.lines().filter(|l| l.starts_with("## Round ")).count()
}

fn read_info(paths: &Paths, slug: &str) -> Option<DiscussionInfo> {
    let path = discussion_path(paths, slug);
    let text = util::read_opt(&path)?;
    Some(DiscussionInfo {
        slug: slug.to_string(),
        topic: frontmatter_value(&text, "topic").unwrap_or_else(|| slug.to_string()),
        status: frontmatter_value(&text, "status").unwrap_or_else(|| "open".to_string()),
        rounds: count_rounds(&text),
        created: frontmatter_value(&text, "created").unwrap_or_default(),
        path: util::to_slash(&path),
    })
}

/// Create a new discussion document. Errors if it already exists.
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
        "---\ntopic: {topic}\nslug: {slug}\nstatus: open\ncreated: {created}\n---\n\n# Discussion: {topic}\n\n<!-- Rounds are appended below as the discussion evolves. -->\n"
    );
    util::write_file(&path, &content)?;
    Ok(DiscussionInfo {
        slug,
        topic: topic.to_string(),
        status: "open".to_string(),
        rounds: 0,
        created,
        path: util::to_slash(&path),
    })
}

/// List all discussions (sorted by slug).
pub fn list_discussions(paths: &Paths) -> Vec<DiscussionInfo> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(paths.discussions_dir()) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let slug = entry.file_name().to_string_lossy().to_string();
                if let Some(info) = read_info(paths, &slug) {
                    out.push(info);
                }
            }
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
}

pub fn show_discussion(paths: &Paths, slug: &str) -> Option<String> {
    util::read_opt(&discussion_path(paths, slug))
}

pub fn info(paths: &Paths, slug: &str) -> Option<DiscussionInfo> {
    read_info(paths, slug)
}

/// Append a discussion round. Content is supplied verbatim (from the skill via stdin).
pub fn add_round(paths: &Paths, slug: &str, mode: &str, content: &str) -> Result<usize> {
    let path = discussion_path(paths, slug);
    let mut text = match util::read_opt(&path) {
        Some(t) => t,
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
    };
    let round_no = count_rounds(&text) + 1;
    let date = util::today();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!(
        "\n## Round {round_no} — {mode} ({date})\n\n{}\n",
        content.trim_end()
    ));
    util::write_file(&path, &text)?;
    Ok(round_no)
}

/// Append a conclusion section and mark the discussion concluded.
pub fn conclude(paths: &Paths, slug: &str, content: &str) -> Result<()> {
    let path = discussion_path(paths, slug);
    let mut text = match util::read_opt(&path) {
        Some(t) => t,
        None => bail!("discussion '{slug}' not found — run `speclink discuss new` first"),
    };
    // Flip status: open -> concluded in frontmatter.
    text = text.replacen("status: open", "status: concluded", 1);
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("\n## Conclusion\n\n{}\n", content.trim_end()));
    util::write_file(&path, &text)?;
    Ok(())
}

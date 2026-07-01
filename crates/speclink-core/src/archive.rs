//! Archive a completed change: apply deltas to canonical specs, inject @trace, snapshot, move.

use crate::model::{self, Change};
use crate::paths::Paths;
use crate::tasks::TouchedRecord;
use crate::util;
use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CapCounts {
    pub capability: String,
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub renamed: usize,
}

#[derive(Debug)]
pub struct ArchiveOutcome {
    pub change_name: String,
    pub dated_name: String,
    pub caps: Vec<CapCounts>,
    pub snapshot_created: bool,
    pub skipped_specs: bool,
}

#[derive(Debug, Default)]
pub struct ArchiveOptions {
    pub skip_specs: bool,
    pub no_validate: bool,
    pub mark_tasks_complete: bool,
}

struct DeltaReq {
    operation: String,
    name: String,
    block: String,
}

fn parse_delta(text: &str) -> Vec<DeltaReq> {
    let mut reqs = Vec::new();
    let mut operation = String::new();
    let mut cur: Option<(String, String, Vec<String>)> = None; // (op, name, lines)
    let flush = |cur: &mut Option<(String, String, Vec<String>)>, reqs: &mut Vec<DeltaReq>| {
        if let Some((op, name, lines)) = cur.take() {
            // Preserve the delta's inter-requirement spacing verbatim (only strip a single
            // trailing newline that `lines.join` cannot have produced anyway).
            reqs.push(DeltaReq {
                operation: op,
                name,
                block: lines.join("\n"),
            });
        }
    };
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(op) = t.strip_prefix("## ") {
            if op.trim_end().ends_with("Requirements") {
                flush(&mut cur, &mut reqs);
                operation = op.split_whitespace().next().unwrap_or("").to_string();
                continue;
            }
        }
        if let Some(name) = t.strip_prefix("### Requirement:") {
            flush(&mut cur, &mut reqs);
            cur = Some((operation.clone(), name.trim().to_string(), vec![line.to_string()]));
        } else if let Some((_, _, lines)) = cur.as_mut() {
            lines.push(line.to_string());
        }
    }
    flush(&mut cur, &mut reqs);
    reqs
}

fn trace_block(change: &str, date: &str, files: &[String]) -> String {
    let mut s = String::from("<!-- @trace\n");
    s.push_str(&format!("source: {change}\n"));
    s.push_str(&format!("updated: {date}\n"));
    s.push_str("code:\n");
    for f in files {
        s.push_str(&format!("  - {f}\n"));
    }
    s.push_str("-->");
    s
}

pub fn archive(paths: &Paths, change: &Change, opts: &ArchiveOptions) -> Result<ArchiveOutcome> {
    let date = util::today();
    let dated_name = format!("{date}-{}", change.name);
    let archive_target = paths.archive_dir().join(&dated_name);
    if archive_target.exists() {
        bail!(
            "an archived change named '{}' already exists; rename before archiving",
            dated_name
        );
    }

    // The @trace `code:` list is the set of code files changed in the work tree at archive time
    // (git status, excluding the spec/work dirs), sorted — matching Spectra. When the tree is
    // clean the list is empty and the @trace block is omitted entirely.
    let trace_files = {
        let mut f = crate::tasks::git_changed_files(&paths.root);
        f.sort();
        f.dedup();
        f
    };
    let _ = TouchedRecord::load(paths, &change.name); // touched.json remains for the commit skill

    let mut caps = Vec::new();
    let mut created_specs: Vec<String> = Vec::new();

    if !opts.skip_specs {
        for cap in model::delta_capabilities(&change.dir) {
            let delta_path = change.dir.join("specs").join(&cap).join("spec.md");
            let delta_text = util::read_opt(&delta_path).unwrap_or_default();
            let reqs = parse_delta(&delta_text);

            let canonical_path = paths.specs_dir().join(&cap).join("spec.md");
            let existed = canonical_path.exists();
            let counts = apply_delta_to_canonical(
                &canonical_path,
                &cap,
                &change.name,
                &date,
                &reqs,
                &trace_files,
            )?;
            if !existed {
                created_specs.push(format!("specs/{cap}/spec.md"));
            }
            caps.push(counts);
        }
    }

    // Snapshot for unarchive support.
    let snapshot_dir = paths.snapshots_dir().join(&dated_name);
    let snapshot = serde_json::json!({ "created_specs": created_specs });
    util::write_file(
        &snapshot_dir.join("created_specs.json"),
        &serde_json::to_string_pretty(&snapshot).unwrap_or_default(),
    )?;

    // Move change dir into archive.
    std::fs::create_dir_all(paths.archive_dir())?;
    std::fs::rename(&change.dir, &archive_target)?;

    // Stamp archived_by / archived_at into the archived change metadata.
    let meta_path = archive_target.join(".openspec.yaml");
    if let Some(mut meta) = util::read_opt(&meta_path) {
        if !meta.ends_with('\n') {
            meta.push('\n');
        }
        if let Some(id) = util::git_identity(&paths.root) {
            meta.push_str(&format!("archived_by: {id}\n"));
        }
        meta.push_str(&format!("archived_at: {date}\n"));
        util::write_file(&meta_path, &meta)?;
    }

    Ok(ArchiveOutcome {
        change_name: change.name.clone(),
        dated_name,
        caps,
        snapshot_created: true,
        skipped_specs: opts.skip_specs,
    })
}

/// Parse a canonical spec into (header, requirement blocks). `header` is everything up to the
/// first `### Requirement:` (including the `## Requirements` line); each block is the full text of
/// a requirement (through its `@trace`), with `---` separators and surrounding blank lines stripped.
fn parse_canonical(text: &str) -> (String, Vec<(String, String)>) {
    let marker = "### Requirement:";
    let split_at = text.find(marker).unwrap_or(text.len());
    let header = text[..split_at].to_string();
    let body = &text[split_at..];

    let mut blocks: Vec<(String, String)> = Vec::new();
    let mut name = String::new();
    let mut lines: Vec<String> = Vec::new();
    let flush = |name: &mut String, lines: &mut Vec<String>, blocks: &mut Vec<(String, String)>| {
        if lines.is_empty() {
            return;
        }
        // Strip trailing `---` separator and blank lines.
        while matches!(lines.last().map(|s| s.trim()), Some("") | Some("---")) {
            lines.pop();
        }
        blocks.push((std::mem::take(name), lines.join("\n")));
        lines.clear();
    };
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix(marker) {
            flush(&mut name, &mut lines, &mut blocks);
            name = rest.trim().to_string();
        }
        lines.push(line.to_string());
    }
    flush(&mut name, &mut lines, &mut blocks);
    (header, blocks)
}

fn rename_target(block: &str) -> Option<String> {
    // Support "**TO:** New Name" or "TO: New Name" inside a RENAMED block.
    for line in block.lines() {
        let t = line.trim().trim_start_matches("- ").replace("**", "");
        if let Some(v) = t.strip_prefix("TO:") {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn apply_delta_to_canonical(
    canonical_path: &PathBuf,
    cap: &str,
    change: &str,
    date: &str,
    reqs: &[DeltaReq],
    trace_files: &[String],
) -> Result<CapCounts> {
    // Spectra omits the @trace block entirely when there are no touched code files.
    let trace = trace_block(change, date, trace_files);
    let make_block = |r: &DeltaReq| {
        if trace_files.is_empty() {
            r.block.clone()
        } else {
            format!("{}\n\n{}", r.block, trace)
        }
    };
    let mut counts = CapCounts {
        capability: cap.to_string(),
        added: 0,
        modified: 0,
        removed: 0,
        renamed: 0,
    };

    if !canonical_path.exists() {
        // Fresh canonical: ADDED and MODIFIED both become requirement sections.
        let mut blocks: Vec<String> = Vec::new();
        for r in reqs {
            match r.operation.as_str() {
                "ADDED" => {
                    blocks.push(make_block(r));
                    counts.added += 1;
                }
                "MODIFIED" => {
                    blocks.push(make_block(r));
                    counts.modified += 1;
                }
                _ => {}
            }
        }
        let mut out = String::new();
        out.push_str(&format!("# {cap} Specification\n\n"));
        out.push_str("## Purpose\n\n");
        out.push_str(&format!(
            "TBD - created by archiving change '{change}'. Update Purpose after archive.\n\n"
        ));
        out.push_str("## Requirements\n\n");
        let joined: Vec<String> = blocks.iter().map(|b| b.trim_end().to_string()).collect();
        out.push_str(&joined.join("\n\n---\n"));
        util::write_file(canonical_path, &out)?;
        return Ok(counts);
    }

    // Merge into an existing canonical spec.
    let existing = util::read_opt(canonical_path).unwrap_or_default();
    let (header, mut blocks) = parse_canonical(&existing);
    // Spectra splices out a removed requirement's text but leaves its preceding `---`; when the
    // LAST requirement is removed this leaves a dangling separator, which we reproduce below.
    let orig_last = blocks.last().map(|(n, _)| n.clone());
    for r in reqs {
        match r.operation.as_str() {
            "ADDED" => {
                // Skip an ADDED requirement that already exists (no duplicate, not counted).
                if !blocks.iter().any(|(n, _)| *n == r.name) {
                    blocks.push((r.name.clone(), make_block(r)));
                    counts.added += 1;
                }
            }
            "MODIFIED" => {
                // Only apply MODIFIED to an existing requirement; skip if absent (matches Spectra,
                // which flags it via analyze's gapModifiedNotFound rather than materializing it).
                if let Some(slot) = blocks.iter_mut().find(|(n, _)| *n == r.name) {
                    slot.1 = make_block(r);
                    counts.modified += 1;
                }
            }
            "REMOVED" => {
                let before = blocks.len();
                blocks.retain(|(n, _)| *n != r.name);
                if blocks.len() != before {
                    counts.removed += 1;
                }
            }
            "RENAMED" => {
                if let Some(to) = rename_target(&r.block) {
                    if let Some(slot) = blocks.iter_mut().find(|(n, _)| *n == r.name) {
                        slot.1 = slot
                            .1
                            .replacen(&format!("### Requirement: {}", r.name), &format!("### Requirement: {to}"), 1);
                        slot.0 = to;
                        counts.renamed += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let mut out = header;
    let joined: Vec<String> = blocks.iter().map(|(_, b)| b.trim_end().to_string()).collect();
    out.push_str(&joined.join("\n\n---\n"));
    // Dangling separator when the original last requirement was removed (matches Spectra).
    let last_removed = orig_last
        .map(|n| !blocks.iter().any(|(bn, _)| *bn == n))
        .unwrap_or(false);
    if last_removed && !blocks.is_empty() {
        out.push_str("\n\n---\n");
    }
    util::write_file(canonical_path, &out)?;
    Ok(counts)
}

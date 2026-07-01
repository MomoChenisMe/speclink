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

    let touched = TouchedRecord::load(paths, &change.name);
    let trace_files = touched.all_files();

    let mut caps = Vec::new();
    let mut created_specs: Vec<String> = Vec::new();

    if !opts.skip_specs {
        for cap in model::delta_capabilities(&change.dir) {
            let delta_path = change.dir.join("specs").join(&cap).join("spec.md");
            let delta_text = util::read_opt(&delta_path).unwrap_or_default();
            let reqs = parse_delta(&delta_text);

            let mut counts = CapCounts {
                capability: cap.clone(),
                added: 0,
                modified: 0,
                removed: 0,
                renamed: 0,
            };
            for r in &reqs {
                match r.operation.as_str() {
                    "ADDED" => counts.added += 1,
                    "MODIFIED" => counts.modified += 1,
                    "REMOVED" => counts.removed += 1,
                    "RENAMED" => counts.renamed += 1,
                    _ => {}
                }
            }

            let canonical_path = paths.specs_dir().join(&cap).join("spec.md");
            let existed = canonical_path.exists();
            apply_delta_to_canonical(
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

    Ok(ArchiveOutcome {
        change_name: change.name.clone(),
        dated_name,
        caps,
        snapshot_created: true,
        skipped_specs: opts.skip_specs,
    })
}

fn apply_delta_to_canonical(
    canonical_path: &PathBuf,
    cap: &str,
    change: &str,
    date: &str,
    reqs: &[DeltaReq],
    trace_files: &[String],
) -> Result<()> {
    let trace = trace_block(change, date, trace_files);

    // Build the requirement sections (ADDED + MODIFIED are written into canonical).
    let mut blocks: Vec<String> = Vec::new();
    for r in reqs {
        if r.operation == "ADDED" || r.operation == "MODIFIED" {
            // Preserve the delta's raw trailing spacing before the trace block.
            blocks.push(format!("{}\n\n{}", r.block, trace));
        }
    }

    if !canonical_path.exists() {
        let mut out = String::new();
        out.push_str(&format!("# {cap} Specification\n\n"));
        out.push_str("## Purpose\n\n");
        out.push_str(&format!(
            "TBD - created by archiving change '{change}'. Update Purpose after archive.\n\n"
        ));
        out.push_str("## Requirements\n\n");
        out.push_str(&blocks.join("\n\n---\n"));
        util::write_file(canonical_path, &out)?;
    } else {
        // Append ADDED/MODIFIED requirements to the existing canonical spec.
        let mut existing = util::read_opt(canonical_path).unwrap_or_default();
        if !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str("\n---\n");
        existing.push_str(&blocks.join("\n\n---\n"));
        util::write_file(canonical_path, &existing)?;
    }
    Ok(())
}

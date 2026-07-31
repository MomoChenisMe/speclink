//! Archive a completed change: apply deltas to canonical specs, inject @trace, snapshot, move.

use crate::model::{self, Change};
use crate::store::Store;
use crate::tasks::TouchedRecord;
use crate::util;
use crate::workspace::Workspace;
use anyhow::{bail, Result};

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
    /// The linked discussions archived along with the change: (slug, archived file name).
    /// A change can carry several source discussions (`from_discussion` is a comma
    /// accumulator), so each is judged independently — empty when none co-travel.
    pub archived_discussions: Vec<(String, String)>,
}

#[derive(Debug, Default)]
pub struct ArchiveOptions {
    pub skip_specs: bool,
    pub no_validate: bool,
    pub mark_tasks_complete: bool,
}

pub(crate) struct DeltaReq {
    pub(crate) operation: String,
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) block: String,
}

pub(crate) fn parse_delta(text: &str) -> Vec<DeltaReq> {
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

/// `actor` is the Host-resolved display identity — None stamps no archived_by.
pub fn archive(
    ws: &Workspace,
    store: &dyn Store,
    change: &Change,
    opts: &ArchiveOptions,
    actor: Option<&str>,
) -> Result<ArchiveOutcome> {
    // Fail-closed gate: archiving stamps and moves the metadata document —
    // refuse a corrupt one before any validation or file effect.
    crate::model::require_valid_meta(change)?;

    // Task-readiness gate (spec「單筆封存的任務完成度守門」): an incomplete change
    // refuses to archive unless the --mark-tasks-complete flag rides along. The
    // exemption is the flag itself, not the runtime's pre-write — direct callers
    // (desktop) get the same semantics without it. Condition mirrors the bulk
    // pre-filter: only total > 0 gates, a zero-task change passes.
    if !opts.mark_tasks_complete {
        let tasks_md = store.read_artifact(&change.name, "tasks.md").unwrap_or_default();
        let (total, complete, _) = crate::tasks::progress(&crate::tasks::parse(&tasks_md));
        if total > 0 && complete < total {
            return Err(crate::command::Refusal(format!(
                "change '{}' has {complete}/{total} tasks complete — archive refuses an \
                 incomplete change; complete the remaining tasks, or pass \
                 --mark-tasks-complete to check them all and archive",
                change.name
            ))
            .into());
        }
    }

    let date = util::today();
    let dated_name = format!("{date}-{}", change.name);
    if store.archived_change_exists(&dated_name) {
        bail!("Archived change '{}' already exists", dated_name);
    }

    // Single-change archive validates first: a structurally invalid change refuses to
    // archive unless --no-validate is passed. The error strings drop validate's
    // "Parse error: " prefix — that is the frozen rendering here.
    if !opts.no_validate {
        let schema = crate::schema::spec_driven();
        let result = crate::validate::validate_change(store, change, &schema, false);
        if !result.valid {
            let details: Vec<String> = result
                .errors
                .iter()
                .map(|e| e.replace(": Parse error: ", ": "))
                .collect();
            bail!("Validation failed:\n{}", details.join("\n"));
        }
    }

    // The @trace `code:` list (spec verify-evidence「archive trace 由 evidence
    // 建立」): a change with v2 evidence aggregates its recorded entries; a
    // v1-only (or absent) record keeps the current producer — the work tree's
    // git state at archive time. Sorted either way; an
    // empty list omits the @trace block entirely. The output format is frozen.
    // touched.json itself remains in place for the commit skill.
    let record = TouchedRecord::load(ws, &change.name);
    let trace_files = {
        let mut f = if record.entries.is_empty() {
            crate::tasks::git_changed_files(&ws.root)
        } else {
            record.all_files()
        };
        f.sort();
        f.dedup();
        f
    };

    let mut caps = Vec::new();
    let mut created_specs: Vec<String> = Vec::new();
    let snapshot_dir = ws.snapshots_dir().join(&dated_name);
    let mut snapshot_created = false;

    if !opts.skip_specs {
        for cap in store.delta_capabilities(&change.name) {
            let delta_rel = model::delta_spec_artifact(&cap);
            let delta_text = store.read_artifact(&change.name, &delta_rel).unwrap_or_default();
            // Even with --no-validate, apply time hard-fails on a delta that
            // parses to zero operations, leaving the change in place.
            if store.artifact_exists(&change.name, &delta_rel)
                && !model::has_delta_operation(&delta_text)
            {
                bail!(
                    "Failed to parse delta spec: Invalid format: Delta spec must contain \
at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)"
                );
            }
            let reqs = parse_delta(&delta_text);
            let renames = model::rename_pairs(&delta_text);

            // Read the pre-apply canonical once: it decides fresh-vs-merge, feeds the
            // merge, and is the snapshot backup content.
            let existing = store.read_canonical_spec(&cap);
            if let Some(existing_text) = &existing {
                // Back up the pre-apply canonical spec for unarchive support
                // (snapshots/<date>-<name>/specs/<cap>/spec.md holds the previous bytes).
                let backup_path = snapshot_dir.join("specs").join(&cap).join("spec.md");
                util::write_file(&backup_path, existing_text)
                    .map_err(|e| anyhow::anyhow!("Failed to backup spec: {e}"))?;
                snapshot_created = true;
            }
            let counts = apply_delta_to_canonical(
                store,
                &cap,
                &change.name,
                &date,
                &reqs,
                &renames,
                &trace_files,
                existing.as_deref(),
            )?;
            if existing.is_none() {
                created_specs.push(cap.clone());
            }
            caps.push(counts);
        }
    }

    // Snapshot manifest: a bare array of created capability names, written only when a spec
    // was created (frozen byte-for-byte: `["cap-x"]`, no trailing newline).
    if !created_specs.is_empty() {
        util::write_file(
            &snapshot_dir.join("created_specs.json"),
            &serde_json::to_string(&created_specs)
                .map_err(|e| anyhow::anyhow!("Failed to serialize created_specs: {e}"))?,
        )
        .map_err(|e| anyhow::anyhow!("Failed to write created_specs.json: {e}"))?;
        snapshot_created = true;
    }

    // Move change into the archive under its dated name.
    store.archive_change(&change.name, &dated_name)?;

    // Clear the app-side "started" marker for this change, if present.
    let _ = util::remove_file(
        &ws.work_dir()
            .join("changes")
            .join(format!("{}.started", change.name)),
    );

    // Stamp archived_by / archived_at into the archived change metadata.
    if let Some(mut meta) = store.read_archived_meta(&dated_name) {
        if !meta.ends_with('\n') {
            meta.push('\n');
        }
        if let Some(id) = actor {
            meta.push_str(&format!("archived_by: {id}\n"));
        }
        meta.push_str(&format!("archived_at: {date}\n"));
        store.write_archived_meta(&dated_name, &meta)?;
    }

    // A change promoted from (or linked to) a discussion carries its record along into the
    // archive — but only the last change to reference it: a discussion can fan out into
    // several changes, and siblings still in flight need the record to stay live. Each source
    // discussion is judged independently (`from_discussion` is a comma accumulator). (This
    // change was already moved above, so it no longer shows up in list_changes.)
    let archived_discussions: Vec<(String, String)> = change
        .meta
        .from_discussions()
        .into_iter()
        .filter_map(|slug| {
            let still_referenced = model::list_changes(store)
                .iter()
                .any(|c| c.meta.from_discussions().iter().any(|s| *s == slug));
            if still_referenced {
                return None;
            }
            crate::discuss::archive_discussion(store, &slug)
                .ok()
                .flatten()
                .map(|file| (slug, file))
        })
        .collect();

    Ok(ArchiveOutcome {
        change_name: change.name.clone(),
        dated_name,
        caps,
        snapshot_created,
        skipped_specs: opts.skip_specs,
        archived_discussions,
    })
}

/// Parse a canonical spec into (header, requirement blocks). `header` is everything up to the
/// first `### Requirement:` (including the `## Requirements` line); each block is the full text of
/// a requirement (through its `@trace`), with `---` separators and surrounding blank lines stripped.
pub(crate) fn parse_canonical(text: &str) -> (String, Vec<(String, String)>) {
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

/// Strip `<!-- BEFORE: … -->` review-aid comments from a delta block. Deltas may carry a
/// short previous-value note on MODIFIED requirements (speclink convention); it is for
/// reviewers of the change and must not survive into the canonical spec.
fn strip_before_notes(block: &str) -> String {
    if !block.contains("<!-- BEFORE:") {
        // No note: leave the block byte-identical (its spacing is preserved verbatim).
        return block.to_string();
    }
    let lines: Vec<&str> = block.lines().collect();
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_start().starts_with("<!-- BEFORE:") {
            // Skip to the end of the comment (single- or multi-line) …
            while i < lines.len() && !lines[i].trim_end().ends_with("-->") {
                i += 1;
            }
            i += 1;
            // … and swallow one following blank only when the note sat between blanks
            // (avoiding a double gap). Right under a requirement header the blank after
            // the note is the header's own separator; keep it.
            let prev_blank = out.last().map(|l| l.trim().is_empty()).unwrap_or(false);
            if prev_blank && i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
            continue;
        }
        out.push(lines[i]);
        i += 1;
    }
    out.join("\n")
}

#[allow(clippy::too_many_arguments)]
fn apply_delta_to_canonical(
    store: &dyn Store,
    cap: &str,
    change: &str,
    date: &str,
    reqs: &[DeltaReq],
    renames: &[(String, String)],
    trace_files: &[String],
    existing: Option<&str>,
) -> Result<CapCounts> {
    // The @trace block is omitted entirely when there are no touched code files.
    let trace = trace_block(change, date, trace_files);
    let make_block = |r: &DeltaReq, fresh: bool| {
        let body = strip_before_notes(&r.block);
        if trace_files.is_empty() {
            body
        } else if fresh {
            // A fresh canonical keeps the delta's own trailing spacing before @trace
            // (an inter-block blank line therefore yields two blanks — probed).
            format!("{body}\n\n{trace}")
        } else {
            // Merging into an existing canonical normalizes the gap by operation
            // (probed): MODIFIED gets 2 blanks, ADDED 1, regardless of delta spacing.
            let gap = if r.operation == "MODIFIED" { "\n\n\n" } else { "\n\n" };
            format!("{}{gap}{trace}", body.trim_end())
        }
    };
    let mut counts = CapCounts {
        capability: cap.to_string(),
        added: 0,
        modified: 0,
        removed: 0,
        renamed: 0,
    };

    let Some(existing) = existing else {
        // Fresh canonical: ADDED and MODIFIED both become requirement sections.
        let mut blocks: Vec<String> = Vec::new();
        for r in reqs {
            match r.operation.as_str() {
                "ADDED" => {
                    blocks.push(make_block(r, true));
                    counts.added += 1;
                }
                "MODIFIED" => {
                    blocks.push(make_block(r, true));
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
        // The file ends with a newline UNLESS the last block ends with an @trace
        // comment (`-->`), which is written without one.
        if !out.ends_with('\n') && !out.ends_with("-->") {
            out.push('\n');
        }
        store.write_canonical_spec(cap, &out)?;
        return Ok(counts);
    };

    // Merge into an existing canonical spec.
    let (header, mut blocks) = parse_canonical(existing);
    // A removed requirement's text is spliced out but its preceding `---` stays; when the
    // LAST requirement is removed this leaves a dangling separator, reproduced below.
    let orig_last = blocks.last().map(|(n, _)| n.clone());
    for r in reqs {
        match r.operation.as_str() {
            "ADDED" => {
                // Skip an ADDED requirement that already exists (no duplicate, not counted).
                if !blocks.iter().any(|(n, _)| *n == r.name) {
                    blocks.push((r.name.clone(), make_block(r, false)));
                    counts.added += 1;
                }
            }
            "MODIFIED" => {
                // Only apply MODIFIED to an existing requirement; skip if absent (an absent one
                // is flagged via analyze's gapModifiedNotFound rather than materialized).
                if let Some(slot) = blocks.iter_mut().find(|(n, _)| *n == r.name) {
                    slot.1 = make_block(r, false);
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
            // RENAMED DeltaReqs (header form) are handled via `renames` below.
            _ => {}
        }
    }

    // Speclink divergence #4: RENAMED is actually executed — the canonical requirement
    // header is renamed in either documented syntax and counted under `renamed:`.
    for (from, to) in renames {
        if let Some(slot) = blocks.iter_mut().find(|(n, _)| n == from) {
            slot.1 = slot.1.replacen(
                &format!("### Requirement: {from}"),
                &format!("### Requirement: {to}"),
                1,
            );
            slot.0 = to.clone();
            counts.renamed += 1;
        }
    }

    let mut out = header;
    let joined: Vec<String> = blocks.iter().map(|(_, b)| b.trim_end().to_string()).collect();
    out.push_str(&joined.join("\n\n---\n"));
    // Dangling separator when the original last requirement was removed (frozen output shape).
    let last_removed = orig_last
        .map(|n| !blocks.iter().any(|(bn, _)| *bn == n))
        .unwrap_or(false);
    if last_removed && !blocks.is_empty() {
        out.push_str("\n\n---\n");
    }
    // Trailing newline (frozen): ensured only when no @trace was injected this run —
    // with injection the file stays exactly as joined (no newline even when the last
    // requirement is not the traced one), and never one after `-->`.
    if trace_files.is_empty() && !out.ends_with('\n') && !out.ends_with("-->") {
        out.push('\n');
    }
    store.write_canonical_spec(cap, &out)?;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::{archive, ArchiveOptions};
    use crate::store::Store;
    use crate::teststore::TestStore;
    use crate::util;
    use crate::workspace::Workspace;

    #[test]
    fn archive_preserves_started_fields_and_stamps_the_archived_station() {
        // A change carrying all three started_* fields (plus created_*) must
        // arrive in the archive with every lifecycle station intact —
        // started_* byte-for-byte, archived_at appended by the stamp. The host
        // root deliberately does not exist: the skip-specs path touches no
        // host files (git probes fail soft, no snapshot is written), so the
        // test needs no filesystem at all.
        let ws = Workspace {
            root: std::env::temp_dir().join("speclink-archive-test-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        };
        let meta = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\nstarted_at: 2026-07-03\nstarted_by: Worker <w@example.com>\nstarted_with: claude\n";
        let store = TestStore::with_meta("demo", meta);
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        let change = crate::model::find_change(&store, "demo").unwrap();

        let outcome = archive(
            &ws,
            &store,
            &change,
            &ArchiveOptions {
                skip_specs: true,
                no_validate: true,
                mark_tasks_complete: false,
            },
            None,
        )
        .unwrap();

        let today = util::today();
        assert_eq!(outcome.dated_name, format!("{today}-demo"));
        let archived = store.read_archived_meta(&outcome.dated_name).unwrap();
        assert!(
            archived.starts_with(meta),
            "created_* and started_* must survive archive byte-for-byte, got: {archived}"
        );
        assert!(archived.contains(&format!("archived_at: {today}\n")));
        // All three stations coexist on the archived document.
        for field in ["created:", "started_at:", "started_by:", "started_with:", "archived_at:"] {
            assert!(archived.contains(field), "missing station field {field}");
        }
        assert!(!store.change_exists("demo"), "active change moved into the archive");
    }

    // --- 封存共行逐 slug（design D3；spec「多來源討論的變更封存逐一共行」）---

    fn ghost_ws() -> Workspace {
        Workspace {
            root: std::env::temp_dir().join("speclink-archive-co-travel-ghost-root"),
            spec_dir_name: "openspec".to_string(),
        }
    }

    fn skip_opts() -> ArchiveOptions {
        ArchiveOptions { skip_specs: true, no_validate: true, mark_tasks_complete: false }
    }

    fn discussion_doc(slug: &str) -> String {
        format!(
            "---\ntopic: {slug}\nslug: {slug}\nstatus: promoted\npromoted_to: cut\ncreated: 2026-07-01\n---\n\n## Conclusion\n\n**Decision**: x\n"
        )
    }

    #[test]
    fn archive_co_travels_every_unreferenced_source_discussion() {
        // 兩份來源討論皆無其他在途變更引用 → 兩份皆隨行封存。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1, d2\n",
        );
        store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
        store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
        store.discussions.borrow_mut().insert("d2".into(), discussion_doc("d2"));
        let change = crate::model::find_change(&store, "cut").unwrap();

        let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

        let slugs: Vec<&str> =
            outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(slugs, vec!["d1", "d2"], "both unreferenced discussions co-archive");
        assert!(store.archived_discussion_exists("d1"));
        assert!(store.archived_discussion_exists("d2"));
    }

    #[test]
    fn archive_leaves_discussion_still_referenced_by_another_change() {
        // d2 仍被另一在途變更 cut2 引用 → 僅 d1 隨行，d2 留在途。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: d1, d2\n",
        );
        store.metas.borrow_mut().insert(
            "cut2".into(),
            "schema: spec-driven\ncreated: 2026-07-02\nfrom_discussion: d2\n".into(),
        );
        store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
        store.discussions.borrow_mut().insert("d1".into(), discussion_doc("d1"));
        store.discussions.borrow_mut().insert("d2".into(), discussion_doc("d2"));
        let change = crate::model::find_change(&store, "cut").unwrap();

        let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

        let slugs: Vec<&str> =
            outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(slugs, vec!["d1"], "only the unreferenced discussion co-archives");
        assert!(store.archived_discussion_exists("d1"));
        assert!(!store.archived_discussion_exists("d2"), "d2 stays live — still referenced");
        assert!(store.live_discussion_exists("d2"));
    }

    #[test]
    fn archive_single_source_discussion_co_travels_as_before() {
        // 單一來源情境：與變更前一致——恰一份討論隨行封存。
        let store = TestStore::with_meta(
            "cut",
            "schema: spec-driven\ncreated: 2026-07-01\nfrom_discussion: only\n",
        );
        store.put_artifact("cut", "tasks.md", "- [x] 1.1 done\n");
        store.discussions.borrow_mut().insert("only".into(), discussion_doc("only"));
        let change = crate::model::find_change(&store, "cut").unwrap();

        let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();

        let slugs: Vec<&str> =
            outcome.archived_discussions.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(slugs, vec!["only"]);
        assert!(store.archived_discussion_exists("only"));
    }

    // --- 單筆封存任務完成度守門（design D1；spec change-lifecycle「單筆封存的任務完成度守門」）---

    fn gate_store(tasks_md: &str) -> TestStore {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", tasks_md);
        store
    }

    #[test]
    fn incomplete_tasks_refuse_archive_with_evidence_and_zero_writes() {
        // spec Example 守門判定：3 任務僅 1 勾、未帶 --mark-tasks-complete → 拒絕，
        // 訊息載明 1/3 與兩條出路，store 零寫入。
        let store = gate_store("- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
        let change = crate::model::find_change(&store, "demo").unwrap();
        let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
            .expect_err("incomplete tasks must refuse archive");
        assert!(err.to_string().contains("1/3"), "evidence N/M in message: {err}");
        assert!(err.to_string().contains("--mark-tasks-complete"), "exit route named: {err}");
        assert!(
            err.downcast_ref::<crate::command::Refusal>().is_some(),
            "typed Refusal so the runtime classifies refused"
        );
        assert!(store.change_exists("demo"), "change stays in place");
        assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
        assert!(store.canonical.borrow().is_empty(), "no canonical spec writes");
        assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
        assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
    }

    #[test]
    fn all_tasks_complete_passes_the_gate() {
        // spec Example 守門判定：3/3 → 照常封存。
        let store = gate_store("- [x] 1.1 a\n- [x] 1.2 b\n- [x] 1.3 c\n");
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
        assert!(!store.change_exists("demo"), "change moved into the archive");
        assert!(store.archived_change_exists(&outcome.dated_name));
    }

    #[test]
    fn zero_tasks_passes_the_gate() {
        // spec Example 守門判定：任務總數 0 → 照常封存（條件與批次預過濾一致：總數>0 才擋）。
        let store = gate_store("## Tasks\n\n(none)\n");
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
        assert!(!store.change_exists("demo"), "zero-task change archives as before");
    }

    #[test]
    fn mark_tasks_complete_flag_passes_the_gate_without_pre_write() {
        // design D1：豁免＝旗標本身——未經 runtime pre-write 的直呼入口（desktop）
        // 帶旗標時語意一致。
        let store = gate_store("- [x] 1.1 a\n- [ ] 1.2 b\n- [ ] 1.3 c\n");
        let change = crate::model::find_change(&store, "demo").unwrap();
        let opts =
            ArchiveOptions { skip_specs: true, no_validate: true, mark_tasks_complete: true };
        archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
        assert!(!store.change_exists("demo"), "flag exempts the gate");
    }

    // --- archive trace 由 evidence 建立（spec verify-evidence）---

    const DELTA_SPEC: &str = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

    fn temp_root(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("speclink-archive-trace-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn trace_store() -> TestStore {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        store.put_artifact("demo", "specs/auth/spec.md", DELTA_SPEC);
        store
    }

    fn apply_opts() -> ArchiveOptions {
        ArchiveOptions { skip_specs: false, no_validate: true, mark_tasks_complete: false }
    }

    fn git(root: &std::path::Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "git {args:?} failed");
    }

    #[test]
    fn trace_from_v2_evidence_is_byte_isomorphic_with_the_current_producer() {
        // 甲：v2 evidence 記錄檔案清單、workspace 無 git（現行產生者拿不到檔案）；
        // 乙：無記錄、git 工作樹髒同一組檔案（現行產生者）。相同檔案清單 →
        // 封存後正典 spec 逐位元一致。甲的 basis digest 為捏造（必然 stale），
        // 本地 archive 不受 gate 阻擋（gate 檢查僅供遠端 Host）。
        let root_a = temp_root("v2");
        let ws_a = Workspace { root: root_a.clone(), spec_dir_name: "openspec".to_string() };
        std::fs::create_dir_all(ws_a.touched_dir()).unwrap();
        std::fs::write(
            ws_a.touched_dir().join("demo.json"),
            "{\n  \"version\": 2,\n  \"change\": \"demo\",\n  \"entries\": [\n    {\n      \"taskId\": \"tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV\",\n      \"taskDesc\": \"1.1 done\",\n      \"touchedFiles\": [\"src/b.rs\", \"src/a.rs\"],\n      \"basisDigests\": { \"spec\": \"sha256:0\", \"tasks\": \"sha256:0\", \"policy\": \"sha256:0\" },\n      \"recordedAt\": \"2026-07-13T00:00:00Z\"\n    }\n  ]\n}",
        )
        .unwrap();
        let store_a = trace_store();
        let change_a = crate::model::find_change(&store_a, "demo").unwrap();
        archive(&ws_a, &store_a, &change_a, &apply_opts(), None).unwrap();
        let canon_a = store_a.read_canonical_spec("auth").unwrap();

        let root_b = temp_root("git");
        git(&root_b, &["init", "-q"]);
        for rel in ["src/a.rs", "src/b.rs"] {
            let p = root_b.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, "content\n").unwrap();
        }
        let ws_b = Workspace { root: root_b.clone(), spec_dir_name: "openspec".to_string() };
        let store_b = trace_store();
        let change_b = crate::model::find_change(&store_b, "demo").unwrap();
        archive(&ws_b, &store_b, &change_b, &apply_opts(), None).unwrap();
        let canon_b = store_b.read_canonical_spec("auth").unwrap();

        assert_eq!(canon_a, canon_b, "same file list → byte-identical canonical output");
        assert!(canon_a.contains("<!-- @trace"), "trace block injected: {canon_a}");
        assert!(canon_a.contains("  - src/a.rs\n"), "aggregated files listed sorted: {canon_a}");
        assert!(canon_a.contains("  - src/b.rs\n"));
        let _ = std::fs::remove_dir_all(&root_a);
        let _ = std::fs::remove_dir_all(&root_b);
    }

    #[test]
    fn v1_only_record_keeps_the_current_producer_without_error() {
        // v1 舊檔（無 entries）沿現行路徑：trace 仍取 archive 當下的 git 工作樹
        // 狀態，v1 檔案清單不取代之；全程無錯誤。
        let root = temp_root("v1");
        git(&root, &["init", "-q"]);
        let p = root.join("src").join("current.rs");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "content\n").unwrap();
        let ws = Workspace { root: root.clone(), spec_dir_name: "openspec".to_string() };
        std::fs::create_dir_all(ws.touched_dir()).unwrap();
        std::fs::write(
            ws.touched_dir().join("demo.json"),
            "{\"change\":\"demo\",\"touched\":[{\"task_id\":\"1\",\"task_desc\":\"1.1 done\",\"files\":[\"src/recorded.rs\"]}]}",
        )
        .unwrap();
        let store = trace_store();
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ws, &store, &change, &apply_opts(), None).unwrap();
        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(
            canon.contains("  - src/current.rs\n"),
            "v1-only record keeps the git-state producer: {canon}"
        );
        assert!(
            !canon.contains("src/recorded.rs"),
            "v1 file list must not replace the current producer: {canon}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

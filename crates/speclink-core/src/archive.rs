//! Archive a completed change: apply deltas to canonical specs, inject @trace, snapshot, move.

use crate::model::{self, Change};
use crate::store::Store;
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
    /// Whether the change carried any per-task evidence. A reported fact, never a
    /// gate: archiving without evidence is legitimate (a spec-only change earns
    /// none by construction), so the caller decides whether to say anything.
    pub evidence_recorded: bool,
}

#[derive(Debug, Default)]
pub struct ArchiveOptions {
    pub skip_specs: bool,
    pub no_validate: bool,
    pub mark_tasks_complete: bool,
    /// 帶著未結審查工單照樣封存（spec「封存的未結工單守門」的明示帶走處置）；
    /// 工單隨目錄搬入封存區，成為「曾審查未通過」標示的化石證據。
    pub carry_review: bool,
}

/// 未結工單守門（design D5；spec review-station「封存的未結工單守門」）：
/// review.md 存在且未帶 `--carry-review` → 拒絕、三處置齊列；帶旗標時工單隨
/// 目錄搬移，成為封存側「曾審查未通過」標示的化石證據。無工單時零效果——行為
/// 與導入前完全一致。runtime 於 `--mark-tasks-complete` 的前置寫入前先喚一次
/// （比照 guard_meta），`archive` 內再守一次供直接呼叫的入口（desktop）。
pub(crate) fn guard_open_review(store: &dyn Store, name: &str, carry_review: bool) -> Result<()> {
    if carry_review || !store.artifact_exists(name, crate::review::REVIEW_DOC) {
        return Ok(());
    }
    Err(crate::command::Refusal(format!(
        "change '{name}' has an open review ticket (review.md) — settle it before archiving:\n  \
         finish the review and stamp it:  speclink review stamp {name}\n  \
         abandon the review:              speclink review discard {name}\n  \
         archive it as-is:                speclink archive {name} --carry-review \
         (permanently shown as reviewed-not-passed)"
    ))
    .into())
}

/// linked worktree 的分支慣例前綴（speclink-host 的 worktree discovery 同字面；
/// 那份常數活在 host，core 取用不到，故在此獨立持有）。
const WORKTREE_BRANCH_PREFIX: &str = "speclink/";

/// linked worktree 環境守門（design D2；spec change-lifecycle「封存的 linked
/// worktree 環境守門」）：worktree 內封存會把解封存備份寫進 gitignored 的
/// `.speclink/snapshots/`，隨 `git worktree remove` 一併蒸發，且 delta 套的是
/// 分支點的過期正典——資料遺失級，故 fail-closed。
///
/// 兩條件同時成立才拒絕：workspace root 的 `.git` 是檔案（linked worktree 特徵，
/// 與 worktree overlay 的主副本判準同源），且當前分支具 `speclink/` 前綴。主
/// checkout 在第一個條件即短路，不 spawn git。git 不可用、指令失敗或輸出為空
/// （detached HEAD）→ 放行，沿 worktree discovery 的 fail-open 慣例。
///
/// runtime 於 `--mark-tasks-complete` 的前置寫入前先喚一次（比照 guard_meta 與
/// guard_open_review），`archive` 內再守一次供直接呼叫的入口（desktop）。
pub(crate) fn guard_linked_worktree(ws: &Workspace) -> Result<()> {
    // 無 host workspace 的派發（Node host store）拿到的是空 root 的合成
    // Workspace——沒有本地環境可判；空 root 接上 ".git" 會變成以行程 cwd
    // 判定，cwd 恰在任何 speclink worktree 內就會誤拒不相干 store 的封存。
    if ws.root.as_os_str().is_empty() {
        return Ok(());
    }
    if !ws.root.join(".git").is_file() {
        return Ok(());
    }
    let Some(branch) = util::git(&ws.root, &["branch", "--show-current"]) else {
        return Ok(());
    };
    if !branch.starts_with(WORKTREE_BRANCH_PREFIX) {
        return Ok(());
    }
    Err(crate::command::Refusal(format!(
        "archive must not run inside a linked worktree — this checkout is on branch \
         '{branch}', and archiving here writes the unarchive backup into the worktree's \
         gitignored .speclink/snapshots/ (gone with the worktree) while merging deltas \
         onto the branch point's stale canon;\n  \
         land the branch first:  speclink-worktree-merge, then archive from the main checkout"
    ))
    .into())
}

pub(crate) struct DeltaReq {
    pub(crate) operation: String,
    pub(crate) name: String,
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

/// A delta operation that no longer matches the canonical spec, or that contradicts
/// another operation in the same delta. The fail-closed merge gate's unit of refusal —
/// and the single judgement shared by drift's Specs dimension (which re-exports it as
/// `SpecAssumption`) and bulk archive's readiness pre-check (spec archive-merge
/// 「過期判定單源共用」). `operation` carries a comma-joined list on a multi-section
/// collision — the one deliberate widening of its single-token value domain.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MergeViolation {
    pub capability: String,
    pub operation: String,
    pub requirement: String,
    pub reason: String,
}

/// Refusal reasons — frozen strings, rendered verbatim by archive, drift and bulk.
const ADDED_EXISTS: &str = "already exists in the canonical spec — archive would refuse it";
const TARGET_GONE: &str = "target requirement no longer exists in the canonical spec";
const CANON_ABSENT: &str = "canonical spec for this capability does not exist";
const SECTION_COLLISION: &str = "appears more than once across this delta's operation sections";
const RENAME_TARGET_EXISTS: &str = "rename target already exists in the canonical spec";
const NO_RENAME_TARGET: &str = "RENAMED operation names no TO: target";
const MALFORMED_REMOVAL: &str =
    "malformed REMOVED-SCENARIO declaration (missing `-->` on the same line)";
const MALFORMED_BEFORE: &str = "malformed BEFORE comment (never closed with `-->`)";

/// Marker declaring that a MODIFIED block drops a canonical scenario on purpose.
/// One per line inside the block; stripped before the merged text reaches the canon.
const REMOVED_SCENARIO: &str = "<!-- REMOVED-SCENARIO:";

/// `#### Scenario:` names declared in a requirement block, trimmed — the comparison
/// semantics of requirement names.
fn scenario_names(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("#### Scenario:"))
        .map(|n| n.trim().to_string())
        .collect()
}

/// Scenario names a MODIFIED block explicitly gives up via `<!-- REMOVED-SCENARIO: X -->`.
/// Only same-line-terminated declarations count; a marker without its `-->` is malformed
/// and refused by the gate — never silently accepted or stripped.
fn declared_scenario_removals(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix(REMOVED_SCENARIO))
        .filter_map(|rest| rest.trim_end().strip_suffix("-->"))
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect()
}

/// Whether the block carries a `<!-- REMOVED-SCENARIO:` marker that never closes on its
/// own line — the note-stripper would swallow everything after it, so the gate refuses.
fn has_malformed_removal(block: &str) -> bool {
    block.lines().any(|l| {
        let t = l.trim_start();
        t.starts_with(REMOVED_SCENARIO) && !t.trim_end().ends_with("-->")
    })
}

/// Whether the block opens a `<!-- BEFORE:` comment that never closes — the multi-line
/// strip would swallow the rest of the block, so the gate refuses.
fn has_unclosed_before(block: &str) -> bool {
    let mut open = false;
    for line in block.lines() {
        if open {
            if line.trim_end().ends_with("-->") {
                open = false;
            }
        } else if line.trim_start().starts_with("<!-- BEFORE:")
            && !line.trim_end().ends_with("-->")
        {
            open = true;
        }
    }
    open
}

/// CRLF → LF, so name comparisons behave identically on Windows-authored deltas.
fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Every merge violation across the change's delta capabilities, empty when the deltas
/// still apply cleanly. Reads only Store spec facts, so drift, the bulk pre-check and
/// archive itself all reach the same verdict.
pub fn merge_violations(store: &dyn Store, change: &str) -> Vec<MergeViolation> {
    let mut out = Vec::new();
    for cap in store.delta_capabilities(change) {
        let delta_text = store
            .read_artifact(change, &model::delta_spec_artifact(&cap))
            .unwrap_or_default();
        let canonical = store.read_canonical_spec(&cap);
        out.extend(capability_violations(&cap, &delta_text, canonical.as_deref()));
    }
    out
}

/// The violation list of design「違規清單與聚合錯誤形狀」 for one capability — the six
/// designed classes plus the malformed-note and dangling-rename guards that keep them
/// airtight. RENAMED is judged through the shared pair scan (it covers both documented
/// syntaxes), so `parse_delta`'s header-form RENAMED entries are skipped throughout.
fn capability_violations(
    cap: &str,
    delta_text: &str,
    canonical: Option<&str>,
) -> Vec<MergeViolation> {
    let delta_text = normalize_newlines(delta_text);
    let reqs = parse_delta(&delta_text);
    let renames = model::rename_pairs(&delta_text);
    let mut out: Vec<MergeViolation> = Vec::new();
    let violation = |operation: &str, requirement: &str, reason: &str| MergeViolation {
        capability: cap.to_string(),
        operation: operation.to_string(),
        requirement: requirement.to_string(),
        reason: reason.to_string(),
    };

    // (3) One requirement name mentioned more than once across the operation sections —
    // spanning sections, duplicated inside one, or via a RENAMED endpoint. Mentions are
    // counted (not deduped): a self-contradicting delta must refuse, and the merge
    // relies on every cleared name being unique.
    let mut mentions: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for r in reqs.iter().filter(|r| r.operation != "RENAMED") {
        mentions.entry(r.name.as_str()).or_default().push(r.operation.as_str());
    }
    for (from, to) in &renames {
        mentions.entry(from.as_str()).or_default().push("RENAMED");
        mentions.entry(to.as_str()).or_default().push("RENAMED");
    }
    for (name, ops) in &mentions {
        if ops.len() > 1 {
            let mut listed: Vec<&str> = ops.clone();
            listed.sort_unstable();
            listed.dedup();
            out.push(violation(&listed.join(", "), name, SECTION_COLLISION));
        }
    }

    // A rename source that never pairs with a TO: target (orphan FROM in either form,
    // or an empty TO) would apply to nothing — under a fail-closed gate that is
    // refused, not ignored.
    for name in model::rename_dangling_sources(&delta_text) {
        out.push(violation("RENAMED", &name, NO_RENAME_TARGET));
    }

    // An unclosed `<!-- BEFORE:` makes the note-stripper swallow the rest of the block —
    // the requirement body would silently vanish from the canon, so the gate refuses.
    for r in reqs.iter().filter(|r| matches!(r.operation.as_str(), "ADDED" | "MODIFIED")) {
        if has_unclosed_before(&r.block) {
            out.push(violation(&r.operation, &r.name, MALFORMED_BEFORE));
        }
    }

    // (6) A capability with no canonical spec yet accepts ADDED only — a MODIFIED,
    // REMOVED or RENAMED there is an assumption about text that was never written.
    let Some(canonical) = canonical else {
        for r in reqs.iter().filter(|r| !matches!(r.operation.as_str(), "ADDED" | "RENAMED")) {
            out.push(violation(&r.operation, &r.name, CANON_ABSENT));
        }
        for (from, _) in &renames {
            out.push(violation("RENAMED", from, CANON_ABSENT));
        }
        return out;
    };

    let canonical = normalize_newlines(canonical);
    let blocks = parse_canonical(&canonical).1;
    let names: std::collections::BTreeSet<&str> =
        blocks.iter().map(|(n, _)| n.as_str()).collect();
    for r in reqs.iter().filter(|r| r.operation != "RENAMED") {
        match r.operation.as_str() {
            // (1) ADDED colliding with a requirement the canon already carries.
            "ADDED" if names.contains(r.name.as_str()) => {
                out.push(violation("ADDED", &r.name, ADDED_EXISTS));
            }
            // (2) MODIFIED/REMOVED whose source requirement is gone.
            "MODIFIED" | "REMOVED" if !names.contains(r.name.as_str()) => {
                out.push(violation(&r.operation, &r.name, TARGET_GONE));
            }
            // (5) MODIFIED wholesale-replaces its target, so every canonical scenario
            // must be carried over or explicitly given up. Judged on the note-stripped
            // text — exactly what the merge would write — so a scenario line quoted
            // inside a review comment never counts as carried.
            "MODIFIED" => {
                if has_malformed_removal(&r.block) {
                    out.push(violation("MODIFIED", &r.name, MALFORMED_REMOVAL));
                }
                let carried = scenario_names(&strip_review_notes(&r.block));
                let declared = declared_scenario_removals(&r.block);
                let dropped: Vec<String> = blocks
                    .iter()
                    .find(|(n, _)| n == &r.name)
                    .map(|(_, b)| scenario_names(b))
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|s| !carried.contains(s) && !declared.contains(s))
                    .collect();
                if !dropped.is_empty() {
                    out.push(violation(
                        "MODIFIED",
                        &r.name,
                        &format!(
                            "drops canonical scenario(s) {} — carry them over, or declare \
                             the removal with `{REMOVED_SCENARIO} <name> -->`",
                            dropped.join("、")
                        ),
                    ));
                }
            }
            _ => {}
        }
    }
    for (from, to) in &renames {
        // (2) RENAMED source gone, or (4) rename target already taken.
        if !names.contains(from.as_str()) {
            out.push(violation("RENAMED", from, TARGET_GONE));
        } else if names.contains(to.as_str()) {
            out.push(violation("RENAMED", to, RENAME_TARGET_EXISTS));
        }
    }
    out
}

/// The aggregated refusal (design「違規清單與聚合錯誤形狀」): every violation listed
/// at once so one repair round clears them all, closing with the remediation route.
pub(crate) fn merge_refusal(change: &str, violations: &[MergeViolation]) -> anyhow::Error {
    let mut msg = format!(
        "change '{change}' cannot be archived — {} delta operation(s) no longer match the \
         canonical spec:\n",
        violations.len()
    );
    for v in violations {
        msg.push_str(&format!(
            "  - {} / {} / {}: {}\n",
            v.capability, v.operation, v.requirement, v.reason
        ));
    }
    msg.push_str(&format!(
        "fix the delta before archiving:\n  \
         speclink drift {change}     — see what moved under the change\n  \
         /speclink-ingest {change}   — update the delta to the current canonical spec"
    ));
    crate::command::Refusal(msg).into()
}

/// One capability's merge result, computed in the plan phase and written in the
/// commit phase — nothing here has touched the filesystem yet.
struct CapPlan {
    capability: String,
    counts: CapCounts,
    /// The pre-apply canonical text to snapshot; `None` for a capability being created.
    backup: Option<String>,
    /// The merged canonical text to write.
    content: String,
}

/// The canonical @trace block: where a requirement came from and when it last
/// landed. Nothing else — the canon carries no file list, so nothing here ever
/// depends on the work tree's state at archive time.
fn trace_block(change: &str, date: &str) -> String {
    format!("<!-- @trace\nsource: {change}\nupdated: {date}\n-->")
}

/// `actor` is the Host-resolved display identity — None stamps no archived_by.
pub fn archive(
    ws: &Workspace,
    store: &dyn Store,
    change: &Change,
    opts: &ArchiveOptions,
    actor: Option<&str>,
) -> Result<ArchiveOutcome> {
    // Environment gate, ahead of every file effect: a linked worktree is the
    // wrong place to archive from at all (see guard_linked_worktree).
    guard_linked_worktree(ws)?;

    // Fail-closed gate: archiving stamps and moves the metadata document —
    // refuse a corrupt one before any validation or file effect.
    crate::model::require_valid_meta(change)?;

    guard_open_review(store, &change.name, opts.carry_review)?;

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

    // Evidence is reported, never judged (discussion evidence-gate-false-blocks):
    // read once here so the outcome carries the fact even though the change
    // directory moves out from under this path below.
    let evidence_recorded = !crate::tasks::TouchedRecord::load(ws, &change.name).entries.is_empty();

    // --- Plan phase: read every capability, validate all of them, compute the merged
    // text. Nothing is written here, so a violation ends the archive with zero file
    // effect (spec archive-merge「兩階段合併計畫與零半套寫入」).
    let mut plans: Vec<CapPlan> = Vec::new();
    let mut violations: Vec<MergeViolation> = Vec::new();

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

            // Read the pre-apply canonical once: it decides fresh-vs-merge, feeds the
            // merge, and is the snapshot backup content.
            let existing = store.read_canonical_spec(&cap);
            let found = capability_violations(&cap, &delta_text, existing.as_deref());
            if !found.is_empty() {
                // Keep reading the remaining capabilities: the refusal reports every
                // violation at once so one repair round clears them all.
                violations.extend(found);
                continue;
            }

            let (content, counts) =
                merge_capability(&cap, &change.name, &date, &delta_text, existing.as_deref());
            plans.push(CapPlan { capability: cap, counts, backup: existing, content });
        }
    }
    if !violations.is_empty() {
        return Err(merge_refusal(&change.name, &violations));
    }

    // --- Commit phase: snapshots first, then canonical specs, then the directory move.
    // A commit-phase I/O failure therefore always leaves a recoverable backup behind.
    let snapshot_dir = ws.snapshots_dir().join(&dated_name);
    let mut snapshot_created = false;
    for plan in &plans {
        if let Some(previous) = &plan.backup {
            // Back up the pre-apply canonical spec for unarchive support
            // (snapshots/<date>-<name>/specs/<cap>/spec.md holds the previous bytes).
            let backup_path = snapshot_dir.join("specs").join(&plan.capability).join("spec.md");
            util::write_file(&backup_path, previous)
                .map_err(|e| anyhow::anyhow!("Failed to backup spec: {e}"))?;
            snapshot_created = true;
        }
    }
    // Snapshot manifest: a bare array of created capability names, written only when a spec
    // was created (frozen byte-for-byte: `["cap-x"]`, no trailing newline).
    let created_specs: Vec<&String> =
        plans.iter().filter(|p| p.backup.is_none()).map(|p| &p.capability).collect();
    if !created_specs.is_empty() {
        util::write_file(
            &snapshot_dir.join("created_specs.json"),
            &serde_json::to_string(&created_specs)
                .map_err(|e| anyhow::anyhow!("Failed to serialize created_specs: {e}"))?,
        )
        .map_err(|e| anyhow::anyhow!("Failed to write created_specs.json: {e}"))?;
        snapshot_created = true;
    }
    for plan in &plans {
        // A mid-commit failure names the snapshot location (design risk table):
        // the pre-archive backups written above are the recovery path.
        store.write_canonical_spec(&plan.capability, &plan.content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to write canonical spec for '{}': {e} — pre-archive backups are \
                 under {}",
                plan.capability,
                snapshot_dir.display()
            )
        })?;
    }
    let caps: Vec<CapCounts> = plans.into_iter().map(|p| p.counts).collect();

    // Move change into the archive under its dated name.
    store.archive_change(&change.name, &dated_name)?;

    // Clear the app-side "started" marker for this change, if present.
    let _ = util::remove_file(
        &ws.work_dir()
            .join("changes")
            .join(format!("{}.started", change.name)),
    );
    // The legacy touched record dies with the change: its fact was read into the
    // outcome above, and a leftover would be read back as evidence for a future
    // change reusing this name.
    let _ = util::remove_file(&ws.legacy_touched_file(&change.name));

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
        evidence_recorded,
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

/// Strip the review-aid comments a delta block may carry — `<!-- BEFORE: … -->`
/// previous-value notes and `<!-- REMOVED-SCENARIO: … -->` removal declarations. Both
/// are for reviewers of the change and must not survive into the canonical spec.
fn strip_review_notes(block: &str) -> String {
    let is_note = |line: &str| {
        let t = line.trim_start();
        t.starts_with("<!-- BEFORE:") || t.starts_with(REMOVED_SCENARIO)
    };
    if !block.lines().any(is_note) {
        // No note: leave the block byte-identical (its spacing is preserved verbatim).
        return block.to_string();
    }
    let lines: Vec<&str> = block.lines().collect();
    let mut out: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if is_note(lines[i]) {
            if lines[i].trim_start().starts_with(REMOVED_SCENARIO) {
                // A removal declaration is one line by contract (the gate refuses a
                // marker without its same-line `-->`), so strip exactly one line —
                // a multi-line scan here could swallow the rest of the block.
                i += 1;
            } else {
                // Skip to the end of the BEFORE comment (single- or multi-line) …
                while i < lines.len() && !lines[i].trim_end().ends_with("-->") {
                    i += 1;
                }
                i += 1;
            }
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

/// The delta's own `## Purpose` section content, trimmed (design「新 capability 的
/// Purpose 自 delta 帶入」). `parse_delta` ignores this section, so it never affects
/// operation parsing; only a capability being created consumes it.
fn delta_purpose(text: &str) -> Option<String> {
    let normalized = normalize_newlines(text);
    let mut body: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in normalized.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("## ") {
            if inside {
                break;
            }
            inside = rest.trim() == "Purpose";
            continue;
        }
        if inside {
            body.push(line);
        }
    }
    let content = body.join("\n").trim().to_string();
    (!content.is_empty()).then_some(content)
}

/// The merged canonical text plus this capability's operation counts. Pure: the caller
/// writes it in the commit phase, so the plan phase can be discarded without a trace.
/// Only operations the gate already cleared reach here.
fn merge_capability(
    cap: &str,
    change: &str,
    date: &str,
    delta_text: &str,
    existing: Option<&str>,
) -> (String, CapCounts) {
    let reqs = &parse_delta(delta_text);
    let renames = &model::rename_pairs(delta_text);
    // Every materialized ADDED/MODIFIED requirement gets the block — injection
    // no longer hinges on a file list that no longer exists.
    let trace = trace_block(change, date);
    let make_block = |r: &DeltaReq, fresh: bool| {
        let body = strip_review_notes(&r.block);
        if fresh {
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
        // Fresh canonical: only ADDED reaches here — the gate refuses every other
        // operation against a capability the canon does not carry yet.
        let mut blocks: Vec<String> = Vec::new();
        for r in reqs {
            if r.operation == "ADDED" {
                blocks.push(make_block(r, true));
                counts.added += 1;
            }
        }
        let mut out = String::new();
        out.push_str(&format!("# {cap} Specification\n\n"));
        out.push_str("## Purpose\n\n");
        out.push_str(&match delta_purpose(delta_text) {
            Some(purpose) => format!("{purpose}\n\n"),
            None => format!(
                "TBD - created by archiving change '{change}'. Update Purpose after archive.\n\n"
            ),
        });
        out.push_str("## Requirements\n\n");
        let joined: Vec<String> = blocks.iter().map(|b| b.trim_end().to_string()).collect();
        out.push_str(&joined.join("\n\n---\n"));
        // The file ends with a newline UNLESS the last block ends with an @trace
        // comment (`-->`), which is written without one.
        if !out.ends_with('\n') && !out.ends_with("-->") {
            out.push('\n');
        }
        return (out, counts);
    };

    // Merge into an existing canonical spec.
    let (header, mut blocks) = parse_canonical(existing);
    // A removed requirement's text is spliced out but its preceding `---` stays; when the
    // LAST requirement is removed this leaves a dangling separator, reproduced below.
    let orig_last = blocks.last().map(|(n, _)| n.clone());
    for r in reqs {
        match r.operation.as_str() {
            "ADDED" => {
                // The gate guarantees the name is free — append and count it.
                blocks.push((r.name.clone(), make_block(r, false)));
                counts.added += 1;
            }
            "MODIFIED" => {
                // The gate guarantees the target exists.
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
    // Trailing newline: a merge that materialized no @trace (a pure REMOVED or
    // RENAMED delta never calls make_block) ends with the text-file newline;
    // once a trace was injected the file stays exactly as joined — no newline
    // even when the last requirement is not the traced one. Never one after
    // `-->`, even when the tail trace came from an earlier archive.
    if counts.added == 0 && counts.modified == 0 && !out.ends_with('\n') && !out.ends_with("-->") {
        out.push('\n');
    }
    (out, counts)
}

#[cfg(test)]
mod tests {
    use super::{archive, ArchiveOptions};
    use crate::store::Store;
    use crate::tasks::TouchedRecord;
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
                carry_review: false,
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
        ArchiveOptions {
            skip_specs: true,
            no_validate: true,
            mark_tasks_complete: false,
            carry_review: false,
        }
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
            ArchiveOptions { mark_tasks_complete: true, ..skip_opts() };
        archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
        assert!(!store.change_exists("demo"), "flag exempts the gate");
    }

    // --- 封存的未結工單守門（design D5；spec review-station「封存的未結工單守門」）---

    const TICKET: &str = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n\n- [WARNING] src/a.rs — possible smell\n";

    #[test]
    fn open_review_ticket_refuses_archive_with_three_disposals() {
        // spec Scenario「有工單預設拒絕」：stderr 同列 stamp／discard／--carry-review
        // 三處置，change 未被搬移、零寫入。
        let store = gate_store("- [x] 1.1 a\n");
        store.put_artifact("demo", crate::review::REVIEW_DOC, TICKET);
        let change = crate::model::find_change(&store, "demo").unwrap();
        let err = archive(&ghost_ws(), &store, &change, &skip_opts(), None)
            .expect_err("open ticket must refuse archive");
        let msg = err.to_string();
        assert!(msg.contains("review stamp"), "stamp disposal named: {msg}");
        assert!(msg.contains("review discard"), "discard disposal named: {msg}");
        assert!(msg.contains("--carry-review"), "carry disposal named: {msg}");
        assert!(
            err.downcast_ref::<crate::command::Refusal>().is_some(),
            "typed Refusal so the runtime classifies refused"
        );
        assert!(store.change_exists("demo"), "change stays in place");
        assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
        assert_eq!(*store.meta_writes.borrow(), 0, "zero meta writes");
        assert_eq!(*store.artifact_writes.borrow(), 0, "zero artifact writes");
    }

    #[test]
    fn carry_review_archives_and_the_ticket_travels() {
        // spec Scenario「明示帶走」：--carry-review 放行，封存目錄內含 review.md
        //（化石工單——封存側「曾審查未通過」標示的證據）。
        let store = gate_store("- [x] 1.1 a\n");
        store.put_artifact("demo", crate::review::REVIEW_DOC, TICKET);
        let change = crate::model::find_change(&store, "demo").unwrap();
        let opts = ArchiveOptions { carry_review: true, ..skip_opts() };
        let outcome = archive(&ghost_ws(), &store, &change, &opts, None).unwrap();
        assert!(!store.change_exists("demo"), "change moved into the archive");
        assert_eq!(
            store.read_archived_artifact(&outcome.dated_name, crate::review::REVIEW_DOC).as_deref(),
            Some(TICKET),
            "ticket rides the directory move byte-identically"
        );
    }

    #[test]
    fn archive_without_ticket_is_unaffected_by_the_gate() {
        // spec Scenario「無工單行為不變」：本檔其餘測試全數無工單、即回歸網；
        // 此處釘住 carry_review 預設 false 下無工單照常封存、封存區無工單檔。
        let store = gate_store("- [x] 1.1 a\n");
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&ghost_ws(), &store, &change, &skip_opts(), None).unwrap();
        assert!(!store.change_exists("demo"), "change moved into the archive");
        assert!(
            store.read_archived_artifact(&outcome.dated_name, crate::review::REVIEW_DOC).is_none(),
            "no fossil ticket appears out of nowhere"
        );
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
        ArchiveOptions { skip_specs: false, ..skip_opts() }
    }

    /// A workspace whose root exists on disk, so evidence can be written under it.
    struct TraceWs {
        ws: Workspace,
    }

    impl TraceWs {
        fn new(tag: &str) -> TraceWs {
            TraceWs {
                ws: Workspace { root: temp_root(tag), spec_dir_name: "openspec".to_string() },
            }
        }

        /// Record one v2 evidence entry for "demo" — the record's mere presence
        /// is all archive reads now.
        fn record_evidence(&self) {
            let record = TouchedRecord {
                version: Some(2),
                change: "demo".to_string(),
                touched: Vec::new(),
                entries: vec![crate::tasks::EvidenceEntry {
                    task_id: "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                    task_desc: "1.1 done".to_string(),
                    actor: Some("Tester <t@example.com>".to_string()),
                    repo: None,
                    head_commit: None,
                    touched_files: vec!["src/a.rs".to_string()],
                    recorded_at: "2026-07-13T00:00:00Z".to_string(),
                }],
            };
            record.save(&self.ws).unwrap();
        }
    }

    impl Drop for TraceWs {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.ws.root);
        }
    }

    #[test]
    fn trace_carries_only_source_and_updated_on_a_fresh_canonical() {
        // spec Scenario「trace 兩欄一律注入」：ADDED 物化到新正典，trace 僅兩欄、無 code 清單。
        let t = TraceWs::new("fresh");
        let store = trace_store();
        t.record_evidence();
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        assert!(outcome.evidence_recorded, "a change with a v2 entry reports evidence recorded");
        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(
            canon.contains(&format!("<!-- @trace\nsource: demo\nupdated: {}\n-->", util::today())),
            "trace block is exactly source + updated: {canon}"
        );
        assert!(!canon.contains("code:"), "no file list may survive: {canon}");
        assert!(!canon.contains("  - src/a.rs"), "no file list may survive: {canon}");
    }

    #[test]
    fn trace_is_injected_for_modified_even_with_a_clean_work_tree() {
        // spec Scenario「trace 兩欄一律注入」：注入不再依檔案清單有無決定——
        // 乾淨工作樹、無任何髒檔的 MODIFIED 一樣拿到 trace。
        let t = TraceWs::new("modified");
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n",
        );
        store.canonical.borrow_mut().insert("auth".to_string(), CANON_R1.to_string());
        t.record_evidence();
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(canon.contains("<!-- @trace"), "MODIFIED gets a trace block: {canon}");
        assert!(canon.contains("source: demo"), "{canon}");
        assert!(!canon.contains("code:"), "no file list may survive: {canon}");
    }

    #[test]
    fn a_change_without_evidence_archives_and_reports_it() {
        // spec Scenario「零證據照常封存並提示」的引擎面：無任何 v2 entry 不再是拒絕
        // 理由——封存照常完成，outcome 帶著「沒有證據」這個事實供 CLI 呈現。
        let t = TraceWs::new("no-evidence");
        let store = trace_store();
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        assert!(!outcome.evidence_recorded, "zero entries is reported, not refused");
        assert!(!store.change_exists("demo"), "the change still archives");
        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(
            canon.contains(&format!("<!-- @trace\nsource: demo\nupdated: {}\n-->", util::today())),
            "an evidence-less archive injects the same two-field trace: {canon}"
        );
    }

    #[test]
    fn evidence_content_never_blocks_the_archive() {
        // 討論 evidence-gate-false-blocks：記錄的內容（含前版寫入的 basis digests）
        // 不再被判讀——只要記錄在，封存就通過,連「過期」這個概念都不存在了。
        let t = TraceWs::new("stale-shaped");
        let store = trace_store();
        // 前一版格式：帶 basisDigests 且必然對不上當前基準。
        util::write_file(
            &t.ws.change_evidence_file("demo"),
            r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_LEGACY","taskDesc":"1.1 done","touchedFiles":["src/a.rs"],"basisDigests":{"spec":"sha256:0","tasks":"sha256:0","policy":"sha256:0"},"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
        )
        .unwrap();
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        assert!(outcome.evidence_recorded, "the entry counts however its basis reads");
        assert!(!store.change_exists("demo"), "no staleness judgment stands in the way");
    }

    #[test]
    fn archive_sweeps_the_legacy_touched_record_with_the_change() {
        // 舊路徑殘檔不得比 change 活得久：留著的話，同名新 change 的第一次 load
        // 會把死帳讀成活帳（seen 汙染、零證據提示被吞）。封存比照 `.started`
        // 標記順手帶走；事實（evidence_recorded）在清除前讀取，不受影響。
        let t = TraceWs::new("legacy-sweep");
        let store = trace_store();
        let legacy = t.ws.legacy_touched_file("demo");
        util::write_file(
            &legacy,
            r#"{"version":2,"change":"demo","entries":[{"taskId":"tsk_LEGACY","taskDesc":"1.1 done","touchedFiles":["src/a.rs"],"recordedAt":"2026-07-13T00:00:00Z"}]}"#,
        )
        .unwrap();
        let change = crate::model::find_change(&store, "demo").unwrap();
        let outcome = archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        assert!(outcome.evidence_recorded, "the legacy record still counts as this change's fact");
        assert!(!legacy.exists(), "the legacy touched record dies with the change");
    }

    #[test]
    fn a_pure_removed_merge_keeps_the_trailing_newline() {
        // 純 REMOVED（或純 RENAMED）的合併不注入任何 @trace——這種輸出維持文字檔
        // 的結尾換行；以 `-->` 收尾者除外（不論來自本輪注入或前次封存的殘尾，
        // 見下一測試），凍結為無結尾換行的形狀。
        let t = TraceWs::new("removed-newline");
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## REMOVED Requirements\n\n### Requirement: R1\n\n**Reason**: retired.\n**Migration**: none.\n",
        );
        store.canonical.borrow_mut().insert("auth".to_string(), CANON_R1_R2.to_string());
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(!canon.contains("@trace"), "a pure REMOVED merge injects nothing: {canon}");
        assert!(
            canon.ends_with('\n') && !canon.ends_with("\n\n"),
            "a trace-less merge ends with exactly one trailing newline: {:?}",
            &canon[canon.len().saturating_sub(20)..]
        );
    }

    #[test]
    fn a_trace_tailed_canon_keeps_its_frozen_tail_through_a_pure_removed_merge() {
        // 補結尾換行的規則有一道例外，與 fresh 路徑同規則：末塊以先前封存注入的
        // `-->` 收尾的正典維持凍結形狀——`-->` 之後永不補換行。
        let t = TraceWs::new("trace-tail");
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        store.put_artifact(
            "demo",
            "specs/auth/spec.md",
            "## REMOVED Requirements\n\n### Requirement: R1\n\n**Reason**: retired.\n**Migration**: none.\n",
        );
        let traced_tail_canon = format!(
            "{}\n\n<!-- @trace\nsource: earlier\nupdated: 2026-07-01\n-->",
            CANON_R1_R2.trim_end()
        );
        store.canonical.borrow_mut().insert("auth".to_string(), traced_tail_canon);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&t.ws, &store, &change, &apply_opts(), None).unwrap();

        let canon = store.read_canonical_spec("auth").unwrap();
        assert!(
            canon.ends_with("-->"),
            "no newline may ever follow a trailing `-->`: {:?}",
            &canon[canon.len().saturating_sub(20)..]
        );
    }

    // --- 封存合併 fail-closed 守門（design「違規清單與聚合錯誤形狀」；
    //     spec archive-merge「封存合併 fail-closed 守門」）---

    const CANON_R1: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    const CANON_R1_R2: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n---\n\n### Requirement: R2\n\nIt SHALL also work.\n\n#### Scenario: fine\n\n- **WHEN** used\n- **THEN** fine\n";
    const ADDED_R1: &str = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

    /// 一份可封存的 change（任務全勾、無工單），delta 與正典由呼叫端指定。
    fn merge_store(deltas: &[(&str, &str)], canon: &[(&str, &str)]) -> TestStore {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
        for (cap, text) in deltas {
            store.put_artifact("demo", &crate::model::delta_spec_artifact(cap), text);
        }
        for (cap, text) in canon {
            store.canonical.borrow_mut().insert((*cap).to_string(), (*text).to_string());
        }
        store
    }

    /// 封存必須被守門拒絕：typed Refusal、正典與 change 零效果，回傳錯誤訊息供逐條斷言。
    fn refuse_merge(store: &TestStore) -> String {
        let before = store.canonical.borrow().clone();
        let change = crate::model::find_change(store, "demo").unwrap();
        let err = archive(&ghost_ws(), store, &change, &apply_opts(), None)
            .expect_err("a violating delta must refuse archive");
        assert!(
            err.downcast_ref::<crate::command::Refusal>().is_some(),
            "typed Refusal so the runtime classifies refused: {err}"
        );
        assert!(store.change_exists("demo"), "change stays in place");
        assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
        assert_eq!(*store.canonical.borrow(), before, "canonical specs untouched");
        err.to_string()
    }

    #[test]
    fn added_requirement_already_in_canon_refuses_archive() {
        // spec Scenario「過期 ADDED 被拒絕」：撞名的 ADDED 不再靜默跳過。
        let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("auth"), "capability named: {msg}");
        assert!(msg.contains("ADDED"), "operation named: {msg}");
        assert!(msg.contains("R1"), "requirement named: {msg}");
        assert!(msg.contains("already exists"), "reason given: {msg}");
    }

    #[test]
    fn modified_removed_renamed_missing_target_refuses_archive() {
        // spec Scenario「缺目標的 MODIFIED 被拒絕」：三種操作缺來源需求時一致拒絕。
        for (op, delta) in [
            ("MODIFIED", "## MODIFIED Requirements\n\n### Requirement: Ghost\n\nIt SHALL change.\n"),
            ("REMOVED", "## REMOVED Requirements\n\n### Requirement: Ghost\n"),
            (
                "RENAMED",
                "## RENAMED Requirements\n\n- FROM: `### Requirement: Ghost`\n- TO: `### Requirement: Spirit`\n",
            ),
        ] {
            let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
            let msg = refuse_merge(&store);
            assert!(msg.contains(op), "operation named ({op}): {msg}");
            assert!(msg.contains("Ghost"), "requirement named ({op}): {msg}");
            assert!(msg.contains("no longer exists"), "reason given ({op}): {msg}");
        }
    }

    #[test]
    fn same_requirement_in_two_operation_sections_refuses_archive() {
        // spec Scenario「多區段互撞被拒絕」：同名需求橫跨 MODIFIED 與 REMOVED。
        let delta = "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## REMOVED Requirements\n\n### Requirement: R1\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("R1"), "requirement named: {msg}");
        assert!(
            msg.contains("MODIFIED") && msg.contains("REMOVED"),
            "both colliding operations listed: {msg}"
        );
    }

    #[test]
    fn renamed_target_name_already_in_canon_refuses_archive() {
        // spec Scenario「多區段互撞被拒絕」之 RENAMED 目標側：改名後會撞既有需求。
        let store = merge_store(
            &[(
                "auth",
                "## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: R2`\n",
            )],
            &[("auth", CANON_R1_R2)],
        );
        let msg = refuse_merge(&store);
        assert!(msg.contains("RENAMED"), "operation named: {msg}");
        assert!(msg.contains("R2"), "rename target named: {msg}");
        assert!(msg.contains("already exists"), "reason given: {msg}");
    }

    #[test]
    fn fresh_capability_with_non_added_operation_refuses_archive() {
        // spec Scenario「新 capability 僅接受 ADDED」：正典不存在時 MODIFIED 不再物化成新規格。
        let store = merge_store(
            &[("fresh", "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL change.\n")],
            &[],
        );
        let msg = refuse_merge(&store);
        assert!(msg.contains("fresh"), "capability named: {msg}");
        assert!(msg.contains("MODIFIED"), "operation named: {msg}");
        assert!(msg.contains("does not exist"), "reason given: {msg}");
        assert!(store.canonical.borrow().is_empty(), "no canonical spec materialized");
    }

    #[test]
    fn every_violation_is_reported_at_once_with_remediation_guidance() {
        // spec Scenario「違規聚合一次回報」：跨 capability 的違規單次列齊，並附 drift → ingest 動線。
        let store = merge_store(
            &[
                ("auth", ADDED_R1),
                ("billing", "## MODIFIED Requirements\n\n### Requirement: Ghost\n\nIt SHALL change.\n"),
            ],
            &[("auth", CANON_R1), ("billing", CANON_R1)],
        );
        let msg = refuse_merge(&store);
        assert!(msg.contains("auth") && msg.contains("R1"), "first violation listed: {msg}");
        assert!(
            msg.contains("billing") && msg.contains("Ghost"),
            "second violation listed in the same report: {msg}"
        );
        assert!(msg.contains("drift"), "drift remediation named: {msg}");
        assert!(msg.contains("ingest"), "ingest remediation named: {msg}");
    }

    #[test]
    fn no_validate_does_not_unlock_the_merge_gate() {
        // spec Scenario「no-validate 不解鎖守門」：文件驗證略過，合併守門照常拒絕。
        let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        let opts = ArchiveOptions { skip_specs: false, no_validate: true, ..skip_opts() };
        let err = archive(&ghost_ws(), &store, &change, &opts, None)
            .expect_err("--no-validate must not unlock the merge gate");
        assert!(err.to_string().contains("already exists"), "gate still speaks: {err}");
    }

    #[test]
    fn skip_specs_bypasses_the_merge_gate_as_before() {
        // 既有逃生口：--skip-specs 整段跳過規格套用，守門自然不觸發。
        let store = merge_store(&[("auth", ADDED_R1)], &[("auth", CANON_R1)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &skip_opts(), None)
            .expect("--skip-specs keeps its existing escape-hatch semantics");
        assert!(!store.change_exists("demo"), "change moved into the archive");
        assert_eq!(
            store.read_canonical_spec("auth").as_deref(),
            Some(CANON_R1),
            "canonical spec untouched when spec application is skipped"
        );
    }

    // --- 兩階段合併計畫與零半套寫入（design「兩階段合併」；
    //     spec archive-merge「兩階段合併計畫與零半套寫入」）---

    #[test]
    fn one_violating_capability_leaves_every_capability_untouched() {
        // spec Scenario「任一 capability 違規則全部不寫」：雙 capability 其一合法、
        // 其一違規 → 兩正典皆未變、無 snapshot 落地、change 仍在進行區原位。
        let root = temp_root("two-phase");
        let ws = Workspace { root: root.clone(), spec_dir_name: "openspec".to_string() };
        let store = merge_store(
            &[
                ("auth", "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
                ("billing", ADDED_R1),
            ],
            &[("auth", CANON_R1), ("billing", CANON_R1)],
        );
        let before = store.canonical.borrow().clone();
        let change = crate::model::find_change(&store, "demo").unwrap();

        let err = archive(&ws, &store, &change, &apply_opts(), None)
            .expect_err("a single violating capability refuses the whole archive");

        assert!(err.to_string().contains("billing"), "the violating capability is named: {err}");
        assert_eq!(*store.canonical.borrow(), before, "no canonical spec was written");
        assert!(store.change_exists("demo"), "change stays in the active area");
        assert!(store.archived_metas.borrow().is_empty(), "nothing archived");
        assert!(
            !ws.snapshots_dir().exists(),
            "zero file effect: no snapshot directory was created"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn all_snapshots_land_before_any_canonical_write() {
        // spec Scenario「snapshot 先於正典寫入」：對第一個 capability 的正典寫入注入
        // 失敗；順序正確時第二個 capability 的 snapshot 已在磁碟上（交錯寫入則不會）。
        let root = temp_root("write-order");
        let ws = Workspace { root: root.clone(), spec_dir_name: "openspec".to_string() };
        let store = merge_store(
            &[
                ("auth", "## ADDED Requirements\n\n### Requirement: Fresh A\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
                ("billing", "## ADDED Requirements\n\n### Requirement: Fresh B\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n"),
            ],
            &[("auth", CANON_R1), ("billing", CANON_R1)],
        );
        *store.fail_canonical_write.borrow_mut() = Some("auth".to_string());
        let change = crate::model::find_change(&store, "demo").unwrap();

        let err = archive(&ws, &store, &change, &apply_opts(), None)
            .expect_err("the injected canonical write failure surfaces");
        // design 風險表：commit 階段失敗的錯誤訊息指出 snapshot 位置。
        assert!(
            err.to_string().contains(&ws.snapshots_dir().display().to_string()),
            "the failure names the snapshot location: {err}"
        );

        let dated = format!("{}-demo", util::today());
        for cap in ["auth", "billing"] {
            assert!(
                ws.snapshots_dir().join(&dated).join("specs").join(cap).join("spec.md").is_file(),
                "every snapshot backup lands before the first canonical write ({cap})"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    // --- MODIFIED 的 scenario 保全與明示刪除聲明（design「scenario superset check
    //     與明示刪除聲明」；spec archive-merge 同名需求）---

    /// spec Example 的正典目標需求：兩個 scenario「逾時重試」與「離線佇列」。
    const CANON_TWO_SCENARIOS: &str = "# net Specification\n\n## Purpose\n\nNet.\n\n## Requirements\n\n### Requirement: 重試策略\n\nIt SHALL retry.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry\n\n#### Scenario: 離線佇列\n\n- **WHEN** offline\n- **THEN** queue\n";

    #[test]
    fn modified_dropping_a_canonical_scenario_refuses_and_names_it() {
        // spec Scenario「漏抄 scenario 被拒絕並點名」：delta 只留「逾時重試」、
        // 無刪除聲明 → 拒絕並點名遺失的「離線佇列」。
        let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
        let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("重試策略"), "requirement named: {msg}");
        assert!(msg.contains("離線佇列"), "the dropped scenario is named: {msg}");
        assert!(!msg.contains("逾時重試"), "the surviving scenario is not flagged: {msg}");
    }

    #[test]
    fn declared_scenario_removal_passes_and_the_note_is_stripped() {
        // spec Scenario「明示聲明後允許刪除」：聲明放行，合併後正典含「逾時重試」、
        // 不含「離線佇列」、也不含聲明註解本身。
        let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- REMOVED-SCENARIO: 離線佇列 -->\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
        let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None)
            .expect("an explicit removal declaration lets the merge through");
        let canon = store.read_canonical_spec("net").expect("canonical spec written");
        assert!(canon.contains("#### Scenario: 逾時重試"), "kept scenario survives: {canon}");
        assert!(!canon.contains("離線佇列"), "declared scenario is gone: {canon}");
        assert!(!canon.contains("REMOVED-SCENARIO"), "the declaration itself is stripped: {canon}");
    }

    #[test]
    fn crlf_authored_delta_matches_scenario_names_the_same_way() {
        // design Risk「Windows 換行使 scenario 名比對失準」：CRLF 樣本的判定與 LF 一致
        // ——完整抄錄放行、漏抄則拒絕並點名。
        let complete = "## MODIFIED Requirements\r\n\r\n### Requirement: 重試策略\r\n\r\nIt SHALL retry harder.\r\n\r\n#### Scenario: 逾時重試\r\n\r\n- **WHEN** timeout\r\n- **THEN** retry twice\r\n\r\n#### Scenario: 離線佇列\r\n\r\n- **WHEN** offline\r\n- **THEN** queue\r\n";
        let store = merge_store(&[("net", complete)], &[("net", CANON_TWO_SCENARIOS)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None)
            .expect("a CRLF delta carrying every scenario passes the superset check");

        let partial = "## MODIFIED Requirements\r\n\r\n### Requirement: 重試策略\r\n\r\nIt SHALL retry harder.\r\n\r\n#### Scenario: 逾時重試\r\n\r\n- **WHEN** timeout\r\n- **THEN** retry twice\r\n";
        let store = merge_store(&[("net", partial)], &[("net", CANON_TWO_SCENARIOS)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("離線佇列"), "CRLF authoring still names the dropped scenario: {msg}");
    }

    // --- 新 capability 的 Purpose 自 delta 帶入（design 同名決策；
    //     spec archive-merge「新 capability 的 Purpose 自 delta 帶入」）---

    const DELTA_WITH_PURPOSE: &str = "## Purpose\n\n本 capability 管理權杖輪替與撤銷。\n\n## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

    #[test]
    fn delta_purpose_becomes_the_new_canonical_purpose() {
        // spec Scenario「delta 提供 Purpose」：新建正典的 Purpose 為 delta 區段內容，非占位文字。
        let store = merge_store(&[("token", DELTA_WITH_PURPOSE)], &[]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None).unwrap();
        let canon = store.read_canonical_spec("token").expect("canonical spec created");
        assert!(
            canon.contains("## Purpose\n\n本 capability 管理權杖輪替與撤銷。\n"),
            "delta Purpose copied verbatim: {canon}"
        );
        assert!(!canon.contains("TBD"), "no placeholder skeleton remains: {canon}");
        assert!(
            !canon.contains("本 capability 管理權杖輪替與撤銷。\n\n## ADDED"),
            "the delta's operation heading does not leak into the canon: {canon}"
        );
    }

    #[test]
    fn delta_without_purpose_keeps_the_placeholder_skeleton() {
        // spec Scenario 反面：delta 未提供 Purpose → 沿用現行 TBD 骨架。
        let store = merge_store(
            &[("token", "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n")],
            &[],
        );
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None).unwrap();
        let canon = store.read_canonical_spec("token").expect("canonical spec created");
        assert!(
            canon.contains("TBD - created by archiving change 'demo'."),
            "placeholder skeleton unchanged: {canon}"
        );
    }

    #[test]
    fn delta_purpose_never_rewrites_an_existing_canonical_purpose() {
        // spec Scenario「既有正典 Purpose 不受 delta 影響」。
        let delta = "## Purpose\n\n這段不該進正典。\n\n## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let change = crate::model::find_change(&store, "demo").unwrap();
        archive(&ghost_ws(), &store, &change, &apply_opts(), None).unwrap();
        let canon = store.read_canonical_spec("auth").expect("canonical spec merged");
        assert!(canon.contains("## Purpose\n\nAuth.\n"), "existing Purpose survives: {canon}");
        assert!(!canon.contains("這段不該進正典"), "delta Purpose is not applied: {canon}");
    }

    // --- 自相矛盾 delta 與註解剝除的守門補強（design「違規清單與聚合錯誤形狀」）---

    #[test]
    fn duplicate_added_names_in_one_delta_refuse_archive() {
        // 自相矛盾的 delta 必須拒絕：同一 delta 內重複 ADDED 名稱若放行，
        // 合併端（已無去重）會在正典寫出兩個同名需求。
        let delta = "## ADDED Requirements\n\n### Requirement: Fresh\n\nA.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n### Requirement: Fresh\n\nB.\n\n#### Scenario: ok2\n\n- **WHEN** used\n- **THEN** works\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("Fresh"), "requirement named: {msg}");
        assert!(msg.contains("more than once"), "duplication reason given: {msg}");
    }

    #[test]
    fn two_renames_to_the_same_target_refuse_archive() {
        // A→C 與 B→C 兩對 rename 指向同一目標（C 不在正典）若放行，
        // 合併後正典出現兩個名為 C 的需求——以 mention 計數攔下。
        let delta = "## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: C`\n\n- FROM: `### Requirement: R2`\n- TO: `### Requirement: C`\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1_R2)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains('C'), "colliding rename target named: {msg}");
        assert!(msg.contains("more than once"), "duplication reason given: {msg}");
    }

    #[test]
    fn scenario_quoted_only_inside_a_before_note_is_not_carried() {
        // 守門必須以剝除後文字判定：BEFORE 註解內引用的 scenario 行不算已抄錄，
        // 否則寫入前剝除會整段刪掉 → 正典靜默掉 scenario。
        let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- BEFORE:\n#### Scenario: 離線佇列\n-->\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
        let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("離線佇列"), "the dropped scenario is named: {msg}");
    }

    #[test]
    fn unterminated_removed_scenario_declaration_refuses_archive() {
        // 畸形聲明必須拒絕：漏打 `-->` 的聲明若被接受，多行剝除會吞掉其後整個
        // block → 需求本體消失。
        let delta = "## MODIFIED Requirements\n\n### Requirement: 重試策略\n\n<!-- REMOVED-SCENARIO: 離線佇列\n\nIt SHALL retry harder.\n\n#### Scenario: 逾時重試\n\n- **WHEN** timeout\n- **THEN** retry twice\n";
        let store = merge_store(&[("net", delta)], &[("net", CANON_TWO_SCENARIOS)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("REMOVED-SCENARIO"), "malformed declaration named: {msg}");
    }

    #[test]
    fn renamed_header_without_to_line_refuses_archive() {
        // fail-closed 守門：header 形式的 RENAMED 缺 TO: 行套用不到任何目標，
        // 必須拒絕而非靜默忽略。
        let delta = "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## RENAMED Requirements\n\n### Requirement: R1\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("RENAMED") && msg.contains("R1"), "dangling rename named: {msg}");
        assert!(msg.contains("TO:"), "missing-target reason given: {msg}");
    }

    #[test]
    fn bullet_rename_without_to_refuses_archive() {
        // fail-closed 守門的 bullet 形式對稱面：孤兒 `- FROM:`（無 TO 行）與
        // 空值 TO 皆套用不到目標，必須拒絕而非靜默忽略。
        const ADDED_OK: &str = "## ADDED Requirements\n\n### Requirement: Brand new\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n";
        let orphan = format!(
            "{ADDED_OK}## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n"
        );
        let store = merge_store(&[("auth", &orphan)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("RENAMED") && msg.contains("R1"), "orphan FROM named: {msg}");

        let empty_to = format!(
            "{ADDED_OK}## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: ``\n"
        );
        let store = merge_store(&[("auth", &empty_to)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("RENAMED") && msg.contains("R1"), "empty TO named: {msg}");
    }

    #[test]
    fn unterminated_before_note_refuses_archive() {
        // 註解剝除守門：未終結的 `<!-- BEFORE:` 會讓剝除吞到 block 結尾，
        // 需求內容靜默消失——與畸形 REMOVED-SCENARIO 同類，必須拒絕。
        let delta = "## ADDED Requirements\n\n### Requirement: Brand new\n\n<!-- BEFORE:\nold text\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
        let store = merge_store(&[("auth", delta)], &[("auth", CANON_R1)]);
        let msg = refuse_merge(&store);
        assert!(msg.contains("BEFORE"), "malformed BEFORE note named: {msg}");
    }

}

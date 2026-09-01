//! Change lifecycle verbs: archive, discard, claim.
//!
//! archive and discard are Dual; claim is RemoteOnly (the fs store has no
//! ownership concept, so dispatch refuses it before this file is reached).

use anyhow::{bail, Result};
use clap::Args;
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, run};
use crate::remote_base::RemoteCtx;
use core::store::Store;
use core::workspace::Workspace;

#[derive(Args)]
pub(crate) struct ArchiveArgs {
    /// Changes to archive (several allowed; auto-detects when omitted and only one exists)
    #[arg(value_name = "CHANGE")]
    changes: Vec<String>,
    /// Archive every ready change (tasks complete, valid, nothing the merge gate refuses)
    #[arg(long)]
    all: bool,
    /// Skip confirmation
    #[arg(short = 'y', long)]
    yes: bool,
    /// Skip spec updates
    #[arg(long = "skip-specs")]
    skip_specs: bool,
    /// Skip validation before archiving
    #[arg(long = "no-validate")]
    no_validate: bool,
    /// Mark all incomplete tasks as complete before archiving
    #[arg(long = "mark-tasks-complete")]
    mark_tasks_complete: bool,
    /// Archive despite an open review ticket (the ticket travels with the
    /// change and is permanently shown as reviewed-not-passed)
    #[arg(long = "carry-review")]
    carry_review: bool,
    /// Archive despite an open verify ticket (the ticket travels with the
    /// change and is permanently shown as verified-not-passed)
    #[arg(long = "carry-verify")]
    carry_verify: bool,
}
#[derive(Args)]
pub(crate) struct DiscardArgs {
    /// Change to discard
    change: String,
    /// Discard even when the change has started work (started_at or checked tasks)
    #[arg(long)]
    force: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
#[derive(Args)]
pub(crate) struct ClaimArgs {
    /// Change name
    pub(crate) name: String,
}
pub(crate) fn cmd_archive(a: ArchiveArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    if a.all || a.changes.len() > 1 {
        return cmd_archive_bulk(&ws, &store, &a);
    }
    // mark-tasks-complete 與封存語意（含 in-progress 標記不動）單點在 runtime。
    let outcome: core::archive::ArchiveOutcome = run(
        &store,
        Some(&ws),
        core::command::Command::Archive {
            change: a.changes.first().cloned(),
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
            carry_review: a.carry_review,
            carry_verify: a.carry_verify,
        },
    )?;
    print_archive_outcome(&outcome);
    Ok(())
}
fn print_archive_outcome(outcome: &core::archive::ArchiveOutcome) {
    println!("{} Archived: {} → {}", color::green("✓"), outcome.change_name, outcome.dated_name);
    if !outcome.caps.is_empty() {
        let names: Vec<&str> = outcome.caps.iter().map(|c| c.capability.as_str()).collect();
        let added: usize = outcome.caps.iter().map(|c| c.added).sum();
        let modified: usize = outcome.caps.iter().map(|c| c.modified).sum();
        let removed: usize = outcome.caps.iter().map(|c| c.removed).sum();
        let renamed: usize = outcome.caps.iter().map(|c| c.renamed).sum();
        println!(
            "Specs applied: {} (added: {added}, modified: {modified}, removed: {removed}, renamed: {renamed})",
            names.join(", ")
        );
    }
    if outcome.snapshot_created {
        println!("Snapshot created for unarchive support.");
    }
    for (slug, file) in &outcome.archived_discussions {
        println!("Discussion archived: {slug} → discussions/archive/{file}");
    }
    // 零證據提示（spec verify-evidence「archive trace 注入與零證據提示」）：一行、
    // 不擋人、不影響 exit code——純規格變更本來就掙不到證據，這行只是讓代理回頭
    // 確認是不是漏走了 apply。
    if !outcome.evidence_recorded {
        eprintln!(
            "note: no task evidence recorded for change '{}' — fine for spec-only changes; otherwise check that tasks went through apply",
            outcome.change_name
        );
    }
}
/// Bulk archive (speclink-specific). Semantics: archives in created-date order, skips
/// not-ready changes with a reason (never silently), and fail-fasts on the first actual
/// archive error with a three-part report. The work tree's state is irrelevant — @trace
/// no longer carries a file list, so a dirty file cannot leak into any archived change.
fn cmd_archive_bulk(ws: &Workspace, store: &dyn Store, a: &ArchiveArgs) -> Result<()> {
    let mut changes: Vec<core::model::Change> = if a.all {
        core::model::list_changes(store)
    } else {
        let mut v = Vec::new();
        for name in &a.changes {
            v.push(
                core::model::find_change(store, name)
                    .ok_or_else(|| anyhow::anyhow!("Change '{name}' not found."))?,
            );
        }
        v
    };
    if changes.is_empty() {
        println!("No active changes to archive.");
        return Ok(());
    }
    changes.sort_by(|x, y| {
        (x.meta.created.as_deref().unwrap_or(""), &x.name)
            .cmp(&(y.meta.created.as_deref().unwrap_or(""), &y.name))
    });

    let schema = core::schema::spec_driven();
    let mut archived: Vec<String> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    for (idx, change) in changes.iter().enumerate() {
        // Readiness: nothing the merge gate would refuse (the pre-check reads the
        // engine's own judgement — spec archive-merge「過期判定單源共用」), valid,
        // tasks complete. --skip-specs bypasses spec application, so the gate never
        // runs there and the pre-check must not filter on it either.
        if !a.skip_specs {
            let refused = core::archive::merge_violations(store, &change.name);
            if !refused.is_empty() {
                let mut reason = format!(
                    "{} delta operation(s) archive would refuse — run /speclink-drift {}",
                    refused.len(),
                    change.name
                );
                // Purpose 守門的違規點名到 capability（spec archive-merge「新
                // capability 缺 Purpose 的違規呈現三處一致」）：只給計數會讓
                // 使用者以為是過期 delta，走錯 drift → ingest 的修法。
                let purpose_caps: Vec<&str> = refused
                    .iter()
                    .filter(|v| v.is_purpose_gate())
                    .map(|v| v.capability.as_str())
                    .collect();
                if !purpose_caps.is_empty() {
                    reason.push_str(&format!(
                        " (new capability {} lacks a qualifying `## Purpose`)",
                        purpose_caps.join(", ")
                    ));
                }
                skipped.push((change.name.clone(), reason));
                continue;
            }
        }
        if !a.no_validate {
            let res = core::validate::validate_change(store, change, &schema, false);
            if !res.valid {
                skipped.push((change.name.clone(), "validation failed".to_string()));
                continue;
            }
        }
        let tasks = core::tasks::parse(
            &store.read_artifact(&change.name, "tasks.md").unwrap_or_default(),
        );
        let (total, complete, _) = core::tasks::progress(&tasks);
        if total > 0 && complete < total && !a.mark_tasks_complete {
            skipped.push((change.name.clone(), format!("tasks incomplete ({complete}/{total})")));
            continue;
        }
        let archive_cmd = core::command::Command::Archive {
            change: Some(change.name.clone()),
            skip_specs: a.skip_specs,
            no_validate: a.no_validate,
            mark_tasks_complete: a.mark_tasks_complete,
            carry_review: a.carry_review,
            carry_verify: a.carry_verify,
        };
        match run::<core::archive::ArchiveOutcome>(store, Some(ws), archive_cmd) {
            Ok(outcome) => {
                print_archive_outcome(&outcome);
                archived.push(outcome.change_name);
            }
            Err(e) => {
                // Fail-fast: earlier archives are already applied and cannot be rolled back.
                println!();
                println!("Bulk archive aborted at '{}': {e}", change.name);
                if !archived.is_empty() {
                    println!("  Archived: {}", archived.join(", "));
                }
                let untouched: Vec<&str> =
                    changes[idx + 1..].iter().map(|c| c.name.as_str()).collect();
                if !untouched.is_empty() {
                    println!("  Untouched: {}", untouched.join(", "));
                }
                bail!("bulk archive failed at '{}'", change.name);
            }
        }
    }

    println!();
    for (name, why) in &skipped {
        println!("! Skipped: {name} — {why}");
    }
    println!(
        "Bulk archive: {} archived, {} skipped",
        archived.len(),
        skipped.len()
    );
    Ok(())
}
pub(crate) fn cmd_discard(a: DiscardArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let outcome: core::discard::DiscardOutcome = run(
        &store,
        Some(&ws),
        core::command::Command::Discard { change: a.change.clone(), force: a.force },
    )?;
    render_discard(&outcome.change_name, &outcome.unlinked_discussions, a.json)
}
/// fs 與 remote 共用的 discard 渲染：--json 為 change＋unlinkedDiscussions、
/// 人眼列出每筆 unlink 後狀態。
fn render_discard(change: &str, unlinked: &[(String, String)], json: bool) -> Result<()> {
    if json {
        let discussions: Vec<serde_json::Value> = unlinked
            .iter()
            .map(|(slug, status)| serde_json::json!({ "slug": slug, "status": status }))
            .collect();
        return print_json(&serde_json::json!({
            "change": change,
            "unlinkedDiscussions": discussions,
        }));
    }
    println!("{} Discarded change: {change}", color::green("✓"));
    for (slug, status) in unlinked {
        println!("  Discussion unlinked: {slug} → {status}");
    }
    Ok(())
}
pub(crate) fn remote_archive(ctx: &RemoteCtx, a: &ArchiveArgs) -> Result<()> {
    if a.all || a.changes.len() > 1 {
        bail!("bulk archive is not supported in remote mode — archive changes one at a time");
    }
    let Some(name) = a.changes.first().cloned().map(Some).unwrap_or(None) else {
        bail!("Please specify a change to archive.");
    };
    let resp = ctx.client.archive(&name, a.carry_review, a.carry_verify)?;
    // datedName 是哨兵：新 server 給得出完整封存結果，就走 fs 同一支渲染；
    // 舊 server 什麼都沒帶，退回既有的兩行輸出而不是半套。
    match to_archive_outcome(name, resp) {
        Ok(outcome) => print_archive_outcome(&outcome),
        Err((name, resp)) => {
            println!("{} Archived change: {name}", color::green("✓"));
            if !resp.specs.is_empty() {
                let caps: Vec<&str> = resp.specs.iter().map(|s| s.capability.as_str()).collect();
                println!("  Specs updated: {}", caps.join(", "));
            }
        }
    }
    Ok(())
}
/// The wire archive response reshaped into the engine's outcome, so the fs
/// rendering path prints it as-is. `Err` hands back the inputs when the
/// sentinel (`datedName`) is absent — an old server whose response cannot
/// honestly fill the outcome.
fn to_archive_outcome(
    change_name: String,
    resp: speclink_protocol::command::ArchiveResponse,
) -> std::result::Result<
    core::archive::ArchiveOutcome,
    (String, speclink_protocol::command::ArchiveResponse),
> {
    let Some(dated_name) = resp.dated_name.clone() else {
        return Err((change_name, resp));
    };
    Ok(core::archive::ArchiveOutcome {
        change_name,
        dated_name,
        caps: resp
            .specs
            .into_iter()
            .map(|s| core::archive::CapCounts {
                capability: s.capability,
                added: s.added,
                modified: s.modified,
                removed: s.removed,
                renamed: s.renamed,
            })
            .collect(),
        snapshot_created: resp.snapshot_created.unwrap_or(false),
        // remote 的封存永遠不跳過 specs（端點無此旗標）——欄位對渲染無影響。
        skipped_specs: false,
        archived_discussions: resp
            .archived_discussions
            .into_iter()
            .map(|d| (d.slug, d.file))
            .collect(),
        // 缺席讀作「有證據」，零證據提示因此不會憑空冒出來。
        evidence_recorded: resp.evidence_recorded.unwrap_or(true),
    })
}
/// remote discard：直通 DELETE 端點（--force 為 query 參數）。guard 拒絕由
/// server 以引擎凍結訊息（含「pass --force」指引）回來，經標準錯誤翻譯原文
/// 呈現——與 fs 模式同語意（design 決策 3／6）。
pub(crate) fn remote_discard(ctx: &RemoteCtx, a: &DiscardArgs) -> Result<()> {
    let outcome = ctx.client.discard(&a.change, a.force)?;
    let unlinked: Vec<(String, String)> = outcome
        .unlinked_discussions
        .into_iter()
        .map(|d| (d.slug, d.status))
        .collect();
    render_discard(&outcome.change, &unlinked, a.json)
}
pub(crate) fn remote_claim(ctx: &RemoteCtx, name: &str) -> Result<()> {
    let _ = ctx.client.claim(name)?;
    println!("{} Claimed change: {name}", color::green("✓"));
    Ok(())
}

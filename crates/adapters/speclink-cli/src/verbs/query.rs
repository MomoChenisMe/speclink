//! Read verbs: list, show, status.
//!
//! Each verb's output comes from one shared function (speclink-verbs'
//! `render::query`) that takes the core outcome type; the remote arm converts
//! the wire payload first, so both modes print the same bytes.

use anyhow::{bail, Result};
use speclink_core as core;
use speclink_protocol::query as protocol_query;

use crate::common::{
    emit, engine_command, info_if_no_changes, open_project, plain_command, render_options, run,
    worktree_overlay,
};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::listing::ListChangeJson;
use speclink_verbs::families::query::{ListArgs, ShowArgs, StatusArgs};
use speclink_verbs::render::query as render;
use speclink_verbs::{ToCommand, VerbInputs};

pub(crate) fn cmd_list(a: ListArgs) -> Result<()> {
    let (ws, fs_store) = open_project()?;
    // Worktree overlay (D3): only a local MAIN checkout with the policy on gets
    // here — everywhere else `facts` is empty, git is never spawned, and both
    // the store and the payload are exactly what they were before this feature.
    let (facts, overlaid) = worktree_overlay(&ws, &fs_store);
    let cmd = engine_command(a.to_command(VerbInputs {
        worktrees: speclink_host::worktree::payload_objects(&facts),
        ..Default::default()
    }))?;
    let list: core::command::ListOutcome =
        run(if facts.is_empty() { &fs_store } else { &overlaid }, Some(&ws), cmd)?;
    emit(render::list(&list, render_options(a.json)));
    Ok(())
}
/// The remote list's engine-shaped outcome, assembled from typed wire pieces.
/// The assembly lives on this side of the include because the remote intercept
/// layer speaks protocol DTOs only (tests/it/no_raw_wire_json.rs), while the
/// engine's `ListOutcome` carries its specs section as an opaque payload.
fn remote_list_outcome(
    changes: Option<Vec<ListChangeJson>>,
    specs: Option<Vec<protocol_query::SpecSummary>>,
) -> Result<core::command::ListOutcome> {
    let specs = specs.map(serde_json::to_value).transpose()?;
    Ok(core::command::ListOutcome { changes, specs })
}
pub(crate) fn cmd_show(a: ShowArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let cmd = plain_command(&a)?;
    let show: core::command::ShowOutcome = run(&store, Some(&ws), cmd)?;
    emit(render::show(&show, render_options(a.json)));
    Ok(())
}
pub(crate) fn cmd_status(a: StatusArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    if info_if_no_changes(&store, a.change.as_deref()) {
        return Ok(());
    }
    let cmd = plain_command(&a)?;
    let report: core::status::StatusReport = run(&store, Some(&ws), cmd)?;
    emit(render::status(&report, render_options(a.json)));
    Ok(())
}
/// The wire summary reshaped into the core listing item — the same rendering
/// path then keeps stdout byte-identical to fs mode.
fn to_list_change_json(c: &protocol_query::ChangeSummary) -> ListChangeJson {
    ListChangeJson {
        completed_tasks: c.completed_tasks,
        name: c.name.clone(),
        status: c.status.clone(),
        summary: c.summary.clone(),
        total_tasks: c.total_tasks,
        restale_from: c.restale_from.clone(),
        meta_error: c.meta_error.clone(),
        delta_capabilities: c.delta_capabilities.clone(),
        // 兩個時間欄位照 server 給的帶回：server 沒給（儲存答不出）就缺席。
        created: c.created.clone(),
        modified_at: c.modified_at.clone(),
        // remote 恆缺席：worktree 是本機主 checkout 的觀察面，
        // server 端沒有這回事（spec scenario remote list 恆無 worktree 欄位）。
        worktree: None,
    }
}
pub(crate) fn remote_list(ctx: &RemoteCtx, a: &ListArgs) -> Result<()> {
    // 組出 fs 模式同一個 ListOutcome 再交給共用渲染：`--specs` 單獨給定時
    // changes 為 None，與引擎的 List outcome 同構。
    let specs_only = a.specs && !a.changes;
    let changes = if specs_only {
        None
    } else {
        let mut items: Vec<ListChangeJson> =
            ctx.client.list_changes()?.changes.iter().map(to_list_change_json).collect();
        // `--sort name` 在本地重排對齊 fs；`created`／`modified` 的排序鍵
        // （meta created、store mtime）不在 wire 上，維持 server 回傳序。
        if a.sort == "name" {
            items.sort_by(|x, y| x.name.cmp(&y.name));
        }
        Some(items)
    };
    let specs = if a.specs { Some(ctx.client.list_specs()?.specs) } else { None };
    emit(render::list(&remote_list_outcome(changes, specs)?, render_options(a.json)));
    Ok(())
}
/// The wire status reshaped into the fs report type — the same rendering
/// path keeps stdout byte-identical to fs mode; server extras (repo,
/// lifecycle, versions) simply don't cross this boundary.
fn to_status_report(status: protocol_query::ChangeStatus) -> core::status::StatusReport {
    core::status::StatusReport {
        change_name: status.change_name,
        schema_name: status.schema_name,
        is_complete: status.is_complete,
        apply_requires: status.apply_requires,
        artifacts: status
            .artifacts
            .into_iter()
            .map(|a| core::status::ArtifactStatusJson {
                id: a.id,
                output_path: a.output_path,
                status: a.status,
                blocked_by: a.missing_deps,
            })
            .collect(),
    }
}
pub(crate) fn remote_status(ctx: &RemoteCtx, a: &StatusArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    let report = to_status_report(ctx.client.get_change(&name)?);
    emit(render::status(&report, render_options(a.json)));
    Ok(())
}
/// remote show 的讀取組合（design D4）：以既有讀 API 組出與 fs 模式相同的
/// ShowOutcome，交給共用的輸出函式。item 與 --item-type 的判別序
/// 對齊引擎 run_show：type==spec 直取規格；否則 change 優先、缺席時規格
/// 遞補（type==change 不遞補）；錯誤訊息沿引擎凍結文本。
fn remote_show_outcome(
    ctx: &RemoteCtx,
    item: Option<&str>,
    item_type: Option<&str>,
) -> Result<core::command::ShowOutcome> {
    let Some(item) = item else {
        bail!("Please specify an item name.");
    };
    if let Some(t) = item_type {
        if t != "change" && t != "spec" {
            bail!("Unknown type: {t}. Use 'change' or 'spec'.");
        }
    }
    let is_spec = ctx.client.list_specs()?.specs.iter().any(|s| s.id == item);
    let status = if item_type == Some("spec") {
        None
    } else {
        match ctx.client.get_change(item) {
            Ok(status) => Some(status),
            Err(e) if e.status == Some(404) => None,
            Err(e) => return Err(e.into()),
        }
    };
    let show_spec =
        item_type == Some("spec") || (item_type != Some("change") && status.is_none() && is_spec);
    if show_spec {
        if !is_spec {
            bail!("Spec '{item}' not found.");
        }
        let content = ctx.client.spec_document(item)?.content;
        return Ok(core::command::ShowOutcome::Spec { name: item.to_string(), content });
    }
    let Some(status) = status else {
        if item_type == Some("change") {
            bail!("Change '{item}' not found.");
        }
        bail!("Item '{item}' not found as a change or spec.");
    };
    // 成對規則由 server 套用：created 出現即代表 meta 的 schema+created 成對。
    let (schema, created) = match status.created {
        Some(created) => (Some(status.schema_name.clone()), Some(created)),
        None => (None, None),
    };
    let read_artifact = |artifact: &str| -> Result<Option<String>> {
        match ctx.client.get_artifact(item, artifact) {
            Ok(content) => Ok(Some(content.content)),
            Err(e) if e.status == Some(404) => Ok(None),
            Err(e) => Err(e.into()),
        }
    };
    // restaleFrom 走清單摘要（設計上 show 組合的「兩份清單」之一）。
    let restale_from = ctx
        .client
        .list_changes()?
        .changes
        .into_iter()
        .find(|c| c.name == item)
        .map(|c| c.restale_from)
        .unwrap_or_default();
    Ok(core::command::ShowOutcome::Change(core::command::ShowChange {
        name: status.change_name,
        schema,
        created,
        proposal: read_artifact("proposal")?,
        design: read_artifact("design")?,
        tasks: read_artifact("tasks")?,
        delta_capabilities: status.delta_capabilities,
        from_discussions: status.from_discussions,
        restale_from,
        // A server that does not report these sends none of them: the keys
        // then stay out of `--json`, as for a change without the records.
        review: status.review.as_ref().and_then(speclink_remote::convert::station_status),
        verify: status.verify.as_ref().and_then(speclink_remote::convert::station_status),
        branch: status.branch,
        landed: status.landed_commit.map(|commit| core::command::LandedRecord {
            commit,
            at: status.landed_at.unwrap_or_default(),
            by: status.landed_by,
        }),
    }))
}

/// show 的 remote 家族臂：wire → ShowOutcome → 與 fs 同一支輸出函式。臂在族內
/// 具名，`remote_show_outcome` 因此不必離開本檔。
pub(crate) fn remote_show(ctx: &RemoteCtx, a: ShowArgs) -> Result<()> {
    let show = remote_show_outcome(ctx, a.item.as_deref(), a.item_type.as_deref())?;
    emit(render::show(&show, render_options(a.json)));
    Ok(())
}

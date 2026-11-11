//! Work-progress verbs: task done/undone and the in-progress marker.
//!
//! Both are Dual and share the flip output (speclink-verbs' `render::progress`),
//! so the success line and the `--json` shape are written once and reused by
//! the remote arm.

use anyhow::{bail, Result};
use speclink_core as core;

use crate::common::{emit, open_project, plain_command, render_options, run};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::progress::{
    InProgressArgs, InProgressCommands, TaskArgs, TaskCommands,
};
use speclink_verbs::render::progress::{self as render, FlippedTask, TaskFlip};

pub(crate) fn cmd_task(a: TaskArgs) -> Result<()> {
    let (flip, json) = match a.command {
        TaskCommands::Done { json, .. } => (TaskFlip::Done, json),
        TaskCommands::Undone { json, .. } => (TaskFlip::Undone, json),
    };
    let (ws, store) = open_project()?;
    // fs mode gives no touched files: the engine probes this workspace itself.
    let cmd = plain_command(&a)?;
    let o: core::command::TaskFlipOutcome = run(&store, Some(&ws), cmd)?;
    emit_task_flip(
        flip,
        &o.change,
        TaskIdentity { refused: &o.task_id.to_string(), arg: &o.task_id_arg },
        &o.description,
        o.already,
        json,
    )
}
/// 同一個任務的兩種識別，具名綁在一起——兩個相鄰同型 `&str` 位置參數
/// 寫反不會被編譯器擋，具名欄位會。
struct TaskIdentity<'a> {
    /// 拒絕訊息用的識別：fs 給引擎解析出的序號，remote 只有 argv。
    refused: &'a str,
    /// 原始 argv——stdout 的人眼行與 `--json` 都用它。
    arg: &'a str,
}
/// `already` 維持現行錯誤結束（引擎已保證零檔案效果）；其餘印出共用的輸出。
/// remote 只有 argv 一種識別，兩個欄位都餵它。
fn emit_task_flip(
    flip: TaskFlip,
    change: &str,
    id: TaskIdentity<'_>,
    description: &str,
    already: bool,
    json: bool,
) -> Result<()> {
    if already {
        bail!("{}", render::already_message(flip, id.refused));
    }
    emit(render::task_flip(
        flip,
        &FlippedTask { change, task_id: id.arg, description },
        render_options(json),
    ));
    Ok(())
}
pub(crate) fn cmd_in_progress(a: InProgressArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let cmd = plain_command(&a)?;
    match a.command {
        InProgressCommands::Add { .. } => {
            let _: core::command::InProgressOutcome = run(&store, Some(&ws), cmd)?;
        }
        InProgressCommands::Remove { .. } => {
            let o: core::command::InProgressRemoveOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::in_progress_remove(&o.name, o.removed, render_options(false)));
        }
    }
    Ok(())
}
fn remote_task_done(
    ctx: &RemoteCtx,
    task_id: &str,
    change: Option<&str>,
    json: bool,
) -> Result<()> {
    let Some(change) = remote_resolve_change(ctx, change, "Use --change to specify one:")? else {
        return Ok(());
    };
    // Attribution: same git-derived touched-file set the fs path records,
    // plus the HEAD they were observed on (the entry's headCommit — spec
    // verify-evidence). An empty set / absent commit is what git yielding
    // nothing looks like.
    let touched = core::tasks::git_changed_files(&ctx.ws);
    let head = core::tasks::head_commit(&ctx.ws.root);
    let resp = ctx.client.task_done(&change, task_id, &touched, head.as_deref())?;
    // remote 只有 argv 一種識別，拒絕訊息與 stdout 兩處都餵它。
    emit_task_flip(
        TaskFlip::Done,
        &change,
        TaskIdentity { refused: task_id, arg: task_id },
        &resp.task_desc,
        resp.already_done,
        json,
    )
}
fn remote_task_undone(
    ctx: &RemoteCtx,
    task_id: &str,
    change: Option<&str>,
    json: bool,
) -> Result<()> {
    let Some(change) = remote_resolve_change(ctx, change, "Use --change to specify one:")? else {
        return Ok(());
    };
    let resp = ctx.client.task_undone(&change, task_id)?;
    emit_task_flip(
        TaskFlip::Undone,
        &change,
        TaskIdentity { refused: task_id, arg: task_id },
        &resp.task_desc,
        resp.already_undone,
        json,
    )
}
/// task 的 remote 家族臂：子指令 enum 窮盡 match、無 catch-all。
pub(crate) fn remote_task(ctx: &RemoteCtx, a: TaskArgs) -> Result<()> {
    match a.command {
        TaskCommands::Done { task_id, change, json } => {
            remote_task_done(ctx, &task_id, change.as_deref(), json)
        }
        TaskCommands::Undone { task_id, change, json } => {
            remote_task_undone(ctx, &task_id, change.as_deref(), json)
        }
    }
}
/// in-progress 的 remote 家族臂：子指令 enum 窮盡 match、無 catch-all。
pub(crate) fn remote_in_progress(ctx: &RemoteCtx, a: InProgressArgs) -> Result<()> {
    match a.command {
        InProgressCommands::Add { name } => {
            // 路由至 server（started_by 由 server 認證身分蓋章）；靜默 exit 0
            // 的 parity 凍結形狀兩模式一致。
            ctx.client.in_progress_add(&name)?;
            Ok(())
        }
        InProgressCommands::Remove { name } => remote_in_progress_remove(ctx, &name),
    }
}
fn remote_in_progress_remove(ctx: &RemoteCtx, name: &str) -> Result<()> {
    // 回應的 removed 區分實際移除與未開工冪等,兩者印不同的行(舊 server 的
    // 裸 Ack 讀作已移除,即其原本的意思);守門 409 與 404 的 message 為引擎
    // 凍結文字,經 `?` 轉發後 stderr 與 fs 模式逐位元一致。
    let removed = ctx.client.in_progress_remove(name)?.removed;
    emit(render::in_progress_remove(name, removed, render_options(false)));
    Ok(())
}

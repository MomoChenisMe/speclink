//! Work-progress verbs: task done/undone and the in-progress marker.
//!
//! Both are Dual and share the flip renderer, so the success line and the
//! `--json` shape are written once and reused by the remote arm.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, run_command};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::store::Store;

#[derive(Args)]
pub(crate) struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommands,
}
#[derive(Subcommand)]
enum TaskCommands {
    /// Mark a task as done and record touched files
    Done {
        /// Task ID (1-based sequential index)
        task_id: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mark a task as not done (pure state flip, no side effects)
    Undone {
        /// Task ID (1-based sequential index)
        task_id: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args)]
pub(crate) struct InProgressArgs {
    #[command(subcommand)]
    command: InProgressCommands,
}
#[derive(Subcommand)]
enum InProgressCommands {
    /// Mark a change as in-progress
    Add {
        /// Change name
        name: String,
    },
    /// Remove the in-progress marker — only when the change carries no work
    /// traces (no checked tasks, no touched records); unknown names error
    Remove {
        /// Change name
        name: String,
    },
}
pub(crate) fn cmd_task(a: TaskArgs) -> Result<()> {
    match a.command {
        TaskCommands::Done { task_id, change, json } => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::TaskDone { task_id: task_id.clone(), change },
            )?;
            let core::command::CommandOutcome::TaskDone(o) = outcome else {
                unreachable!("task done yields a task-flip outcome");
            };
            render_task_flip(
                TaskFlip::Done,
                &o.change,
                TaskIdentity { refused: &o.task_id.to_string(), arg: &o.task_id_arg },
                &o.description,
                o.already,
                json,
            )?;
        }
        TaskCommands::Undone { task_id, change, json } => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::TaskUndone { task_id: task_id.clone(), change },
            )?;
            let core::command::CommandOutcome::TaskUndone(o) = outcome else {
                unreachable!("task undone yields a task-flip outcome");
            };
            render_task_flip(
                TaskFlip::Undone,
                &o.change,
                TaskIdentity { refused: &o.task_id.to_string(), arg: &o.task_id_arg },
                &o.description,
                o.already,
                json,
            )?;
        }
    }
    Ok(())
}
/// `task done` 與 `task undone` 的輸出只差狀態字與動詞片語，共用一支渲染。
#[derive(Clone, Copy)]
enum TaskFlip {
    Done,
    Undone,
}
/// 同一個任務的兩種識別，具名綁在一起——兩個相鄰同型 `&str` 位置參數
/// 寫反不會被編譯器擋，具名欄位會。
struct TaskIdentity<'a> {
    /// 拒絕訊息用的識別：fs 給引擎解析出的序號，remote 只有 argv。
    refused: &'a str,
    /// 原始 argv——stdout 的人眼行與 `--json` 都用它。
    arg: &'a str,
}
impl TaskFlip {
    /// 一次給齊三段文字：--json 的 status、已是該狀態時的拒絕片語、成功行的
    /// 動詞片語——單一 match，不重複分派。
    fn parts(self) -> (&'static str, &'static str, &'static str) {
        match self {
            TaskFlip::Done => ("done", "is already done", "marked as done"),
            TaskFlip::Undone => ("undone", "is already not done", "marked as not done"),
        }
    }
}
/// `already` 維持現行錯誤結束（引擎已保證零檔案效果）。`--json` 是緊湊單行，
/// 欄位順序凍結，兩種翻轉對稱。remote 只有 argv 一種識別，兩個欄位都餵它。
fn render_task_flip(
    flip: TaskFlip,
    change: &str,
    id: TaskIdentity<'_>,
    description: &str,
    already: bool,
    json: bool,
) -> Result<()> {
    let (status, already_phrase, verb_phrase) = flip.parts();
    if already {
        bail!("Task {} {already_phrase}", id.refused);
    }
    if json {
        let v = serde_json::json!({
            "change": change,
            "status": status,
            "task_desc": description,
            "task_id": id.arg,
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Task {} {verb_phrase}: {description}", color::green("✓"), id.arg);
    Ok(())
}
/// 實際移除與「本來就沒開工」印不同的行——引擎與 wire 都據實回報，渲染只讀事實。
fn render_in_progress_remove(name: &str, removed: bool) {
    if removed {
        println!(
            "{} Removed the in-progress marker from '{name}' — back to proposed",
            color::green("✓")
        );
    } else {
        println!("Change '{name}' has no in-progress marker — already proposed");
    }
}
pub(crate) fn cmd_in_progress(a: InProgressArgs) -> Result<()> {
    match a.command {
        InProgressCommands::Add { name } => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            run_command(store, Some(&ws), core::command::Command::InProgressAdd { name })?;
        }
        InProgressCommands::Remove { name } => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome =
                run_command(store, Some(&ws), core::command::Command::InProgressRemove { name })?;
            let core::command::CommandOutcome::InProgressRemove(o) = outcome else {
                unreachable!("in-progress remove yields an in-progress-remove outcome");
            };
            render_in_progress_remove(&o.name, o.removed);
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
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    // Attribution: same git-derived touched-file set the fs path records.
    // Best-effort — remote mode already resolved, so a config error here is
    // unreachable; an empty set is the existing no-workspace behavior.
    let ws = core::workspace::Workspace::discover_cwd().ok().flatten();
    let touched: Vec<String> = ws
        .map(|w| core::tasks::git_changed_files(&w))
        .unwrap_or_default();
    let resp = ctx.client.task_done(&change, task_id, &touched)?;
    // remote 只有 argv 一種識別，拒絕訊息與 stdout 兩處都餵它。
    render_task_flip(
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
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    let resp = ctx.client.task_undone(&change, task_id)?;
    render_task_flip(
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
    render_in_progress_remove(name, removed);
    Ok(())
}

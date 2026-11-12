//! `trace` — 單一 capability 的溯源鏈（封存演進、來源討論、evidence、
//! Requirement 歸屬）。組裝在 core（`speclink_core::trace`），輸出在
//! speclink-verbs（`render::trace`），這裡只把兩者接起來。Dual：本機臂對本機 store
//! 執行引擎命令；remote 臂向 server 取同一份報告、轉回引擎型別，再交給同一個輸出
//! 函式。

use crate::common::{emit, open_project, plain_command, render_options, run};
use crate::remote_base::RemoteCtx;
use anyhow::Result;
use core::trace::TraceReport;
use speclink_core as core;
use speclink_verbs::families::trace::TraceArgs;
use speclink_verbs::render::trace as render;

pub(crate) fn cmd_trace(a: TraceArgs) -> Result<()> {
    let (_ws, store) = open_project()?;
    let report: TraceReport =
        run(&store, None, plain_command(&a)?)?;
    emit(render::trace(&report, render_options(a.json)));
    Ok(())
}

/// Remote arm: the server assembles the chain with the same engine code; an
/// unknown capability arrives as a 404 carrying the engine's line and its
/// near names, which `?` prints exactly as fs mode does.
pub(crate) fn remote_trace(ctx: &RemoteCtx, a: &TraceArgs) -> Result<()> {
    let report = speclink_remote::convert::trace_report(ctx.client.trace(&a.capability)?);
    emit(render::trace(&report, render_options(a.json)));
    Ok(())
}

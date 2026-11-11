//! `trace` — 單一 capability 的溯源鏈（封存演進、來源討論、evidence、
//! Requirement 歸屬）。組裝在 core（`speclink_core::trace`），輸出在
//! speclink-verbs（`render::trace`），這裡只把兩者接起來。local-only：remote 拒絕宣告在 dispatch 的 fs_only。

use crate::common::{emit, open_project, plain_command, render_options, run};
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

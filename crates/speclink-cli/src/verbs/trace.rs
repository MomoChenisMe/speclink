//! `trace` — 單一 capability 的溯源鏈（封存演進、來源討論、evidence、
//! Requirement 歸屬）。組裝在 core（`speclink_core::trace`），這裡只做參數
//! 解析與雙輸出渲染。local-only：remote 拒絕宣告在 dispatch 的 fs_only。

use crate::color;
use crate::common::{open_project, print_json, run_command};
use anyhow::Result;
use clap::Args;
use core::store::Store;
use core::trace::TraceReport;
use speclink_core as core;

#[derive(Args)]
pub(crate) struct TraceArgs {
    /// Capability name (canonical spec directory name)
    capability: String,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

pub(crate) fn cmd_trace(a: TraceArgs) -> Result<()> {
    let (_ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(store, None, core::command::Command::Trace { capability: a.capability })?;
    let core::command::CommandOutcome::Trace(report) = outcome else {
        unreachable!("trace yields a trace outcome");
    };
    if a.json {
        return print_json(&report);
    }
    render_trace(&report);
    Ok(())
}

fn render_trace(r: &TraceReport) {
    println!("Capability: {}", color::bold(&r.capability));

    println!();
    println!("Changes (oldest first):");
    if r.changes.is_empty() {
        println!("  {}", color::dim("(none)"));
    }
    for c in &r.changes {
        println!("  {} ({})", color::bold(&c.name), color::dim(&c.archived_dir));
        match &c.from_discussion {
            Some(slug) => println!("    {} {}", color::dim("discussion:"), slug),
            None => println!("    {} {}", color::dim("discussion:"), color::dim("(none)")),
        }
        match &c.evidence {
            // 記錄存在但零筆 task：與「無記錄」區隔，標示為空。
            Some(tasks) if tasks.is_empty() => {
                println!("    {} {}", color::dim("evidence:"), color::dim("(empty)"))
            }
            Some(tasks) => {
                println!("    {}", color::dim("evidence:"));
                for t in tasks {
                    println!("      {}: {}", t.task_id, t.files.join(", "));
                }
            }
            None => println!("    {} {}", color::dim("evidence:"), color::dim("(no record)")),
        }
    }

    println!();
    println!("Discussions:");
    if r.discussions.is_empty() {
        println!("  {}", color::dim("(none)"));
    }
    for d in &r.discussions {
        let status = if d.archived { "(archived)" } else { "(live)" };
        println!("  {} {}", color::bold(&d.slug), color::dim(status));
        for p in &d.promoted_to {
            let caps = if p.capabilities.is_empty() {
                color::dim("(none)")
            } else {
                p.capabilities.join(", ")
            };
            println!("    -> {}: {caps}", p.change);
        }
    }

    println!();
    println!("Requirements:");
    if r.requirements.is_empty() {
        println!("  {}", color::dim("(none)"));
    }
    for req in &r.requirements {
        println!("  {} <- {}", color::bold(&req.name), req.source);
    }
}

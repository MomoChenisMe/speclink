//! Execution-order verbs: `plan` (read) and `change depends` (write).
//!
//! Both are Dual (add-change-plan-remote D4): the fs arms read and write the
//! local openspec/ tree; the remote arms call the server endpoints and render
//! the response through the same functions, so both modes print alike.

use anyhow::Result;
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, run, worktree_facts, worktree_overlay};
use crate::remote_base::RemoteCtx;
use core::store::Store;

#[derive(Args)]
pub(crate) struct PlanArgs {
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub(crate) struct ChangeVerbArgs {
    #[command(subcommand)]
    command: ChangeCommands,
}

#[derive(Subcommand)]
enum ChangeCommands {
    /// Declare (or with --remove, drop) prerequisite changes in a change's metadata
    Depends {
        /// Change name
        name: String,
        /// Prerequisite change name(s)
        #[arg(long, num_args = 1.., required = true, value_name = "OTHER")]
        on: Vec<String>,
        /// Remove the given prerequisites instead of adding them
        #[arg(long)]
        remove: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn cmd_change(a: ChangeVerbArgs) -> Result<()> {
    match a.command {
        ChangeCommands::Depends {
            name,
            on,
            remove,
            json,
        } => {
            // A change checked out in a worktree lives in that copy (design
            // D6): the guards, the cycle check and the write all run against
            // it, so `plan` — which reads that copy — sees the declaration at
            // once. Without a mapping the checkout's own store is the store;
            // a write through the overlay would land on the main copy and
            // silently split the change's read and write.
            let (ws, fs_store) = open_project()?;
            let home = worktree_facts(&ws, &fs_store)
                .get(&name)
                .map(|e| speclink_fs::FsStore::new(&e.path, &ws.spec_dir_name));
            let store: &dyn Store = match &home {
                Some(worktree) => worktree,
                None => &fs_store,
            };
            let o: core::command::DependsOutcome = run(
                store,
                Some(&ws),
                core::command::Command::ChangeDepends {
                    name,
                    on: on.clone(),
                    remove,
                },
            )?;
            render_depends(&o.change, &o.depends_on, &on, remove, json)
        }
    }
}

/// Remote arm: the server runs the same guards and write; its 404 and refused
/// 409 carry the engine's line, which `?` prints exactly as fs mode does.
pub(crate) fn remote_change(ctx: &RemoteCtx, a: ChangeVerbArgs) -> Result<()> {
    match a.command {
        ChangeCommands::Depends {
            name,
            on,
            remove,
            json,
        } => {
            let written = ctx.client.set_depends(&name, &on, remove)?;
            render_depends(&written.change, &written.depends_on, &on, remove, json)
        }
    }
}

/// One success line (design D4): the full declaration after an add, the
/// dropped names after a remove; `--json` is the declaration after the write.
fn render_depends(
    change: &str,
    depends_on: &[String],
    on: &[String],
    remove: bool,
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({
            "change": change,
            "dependsOn": depends_on,
        }));
    }
    if remove {
        println!(
            "{} {} no longer depends on: {}",
            color::green("✓"),
            change,
            on.join(", ")
        );
    } else {
        println!(
            "{} {} depends on: {}",
            color::green("✓"),
            change,
            depends_on.join(", ")
        );
    }
    Ok(())
}

pub(crate) fn cmd_plan(a: PlanArgs) -> Result<()> {
    let (ws, fs_store) = open_project()?;
    // Same observation surface as `list` (design D6): in a main checkout with
    // the worktree policy on, a worktree's checked tasks and started stamp feed
    // the stage derivation; everywhere else the plain store is read.
    let (facts, overlaid) = worktree_overlay(&ws, &fs_store);
    let plan: core::command::PlanReport = run(
        if facts.is_empty() {
            &fs_store
        } else {
            &overlaid
        },
        Some(&ws),
        core::command::Command::Plan { ranks: None },
    )?;
    render_plan(&plan, a.json)
}

/// Remote arm: the server plans the scope (its board resource is the rank
/// source); the response converts back to the engine's report so the renderer
/// below prints it — a cycle arrives as a refused 409 carrying the engine line.
pub(crate) fn remote_plan(ctx: &RemoteCtx, a: &PlanArgs) -> Result<()> {
    let plan = speclink_remote::convert::plan_report(ctx.client.plan()?)?;
    render_plan(&plan, a.json)
}

/// Human rendering (design D3): one `Wave N` heading per wave (`(parallel)`
/// when it holds more than one change), one line per change with its stage
/// and blockers, skipped changes after the waves, and `next:` last. Every
/// styled piece goes through `color`, so `--no-color` prints no ANSI at all.
fn render_plan(plan: &core::command::PlanReport, json: bool) -> Result<()> {
    if json {
        return print_json(plan);
    }
    for wave in &plan.waves {
        let title = if wave.changes.len() > 1 {
            format!("Wave {} (parallel)", wave.index)
        } else {
            format!("Wave {}", wave.index)
        };
        println!("{}", color::bold(&title));
        for name in &wave.changes {
            let c = plan
                .changes
                .iter()
                .find(|c| &c.name == name)
                .expect("wave members are plan changes");
            let blocked = if c.blocked_by.is_empty() {
                String::new()
            } else {
                color::dim(&format!(" — blocked by: {}", c.blocked_by.join(", ")))
            };
            println!(
                "  {} {} [{}]{blocked}",
                color::cyan("•"),
                c.name,
                c.stage.as_str()
            );
        }
    }
    if !plan.skipped.is_empty() {
        println!("{}", color::bold("Skipped"));
        for s in &plan.skipped {
            println!(
                "  {} {} {}",
                color::cyan("•"),
                s.change,
                color::red("(invalid .openspec.yaml)")
            );
        }
    }
    match &plan.next {
        Some(next) => println!("next: {next}"),
        None => println!("next: none"),
    }
    Ok(())
}

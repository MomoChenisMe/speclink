//! Execution-order verbs: `plan` (read) and `change depends` (write).
//!
//! Both are FsOnly for now (change-plan design D6): they read and write the
//! local openspec/ tree; the remote arms come with the server knife.

use anyhow::Result;
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, run, worktree_facts, worktree_overlay};
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
            render_depends(&o, &on, remove, json)
        }
    }
}

/// One success line (design D4): the full declaration after an add, the
/// dropped names after a remove; `--json` is the declaration after the write.
fn render_depends(
    o: &core::command::DependsOutcome,
    on: &[String],
    remove: bool,
    json: bool,
) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({
            "change": o.change,
            "dependsOn": o.depends_on,
        }));
    }
    if remove {
        println!(
            "{} {} no longer depends on: {}",
            color::green("✓"),
            o.change,
            on.join(", ")
        );
    } else {
        println!(
            "{} {} depends on: {}",
            color::green("✓"),
            o.change,
            o.depends_on.join(", ")
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
        core::command::Command::Plan,
    )?;
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

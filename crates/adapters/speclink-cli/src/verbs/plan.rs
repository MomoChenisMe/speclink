//! Execution-order verbs: `plan` (read), `change depends` and `change rank`
//! (writes).
//!
//! `plan` and `change depends` are Dual (add-change-plan-remote D4): the fs
//! arms read and write the local openspec/ tree; the remote arms call the
//! server endpoints and render the response through the same functions, so
//! both modes print alike. `change rank` is FsOnly: it writes the card metas'
//! board rank, and a remote scope keeps its order in the board resource.

use anyhow::Result;
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, run, worktree_facts, worktree_overlay};
use crate::remote_base::RemoteCtx;
use core::store::Store;
use core::workspace::Workspace;

#[derive(Args)]
pub(crate) struct PlanArgs {
    /// Let a delta capability shared with an earlier change push the wave and block it again (the rule before requirement-level overlap)
    #[arg(long)]
    pub(crate) strict_overlap: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
pub(crate) struct ChangeVerbArgs {
    #[command(subcommand)]
    command: ChangeCommands,
}

impl ChangeVerbArgs {
    /// `change rank` — the one FsOnly member of the Dual `change` family.
    pub(crate) fn is_rank(&self) -> bool {
        matches!(self.command, ChangeCommands::Rank { .. })
    }
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
    /// Move a change right before or after another change of its stage by rewriting its board rank
    #[command(group(clap::ArgGroup::new("side").required(true).args(["before", "after"])))]
    Rank {
        /// Change name
        name: String,
        /// Place it right before this change
        #[arg(long, value_name = "OTHER")]
        before: Option<String>,
        /// Place it right after this change
        #[arg(long, value_name = "OTHER")]
        after: Option<String>,
        /// Move it even when it already has a board rank (someone ordered it by hand)
        #[arg(long)]
        force: bool,
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
            let home = home_store_for(&ws, &worktree_facts(&ws, &fs_store), &name);
            let o: core::command::DependsOutcome = run(
                &home,
                Some(&ws),
                core::command::Command::ChangeDepends {
                    name,
                    on: on.clone(),
                    remove,
                },
            )?;
            render_depends(&o.change, &o.depends_on, &on, remove, json)
        }
        ChangeCommands::Rank {
            name,
            before,
            after,
            force,
            json,
        } => {
            let (anchor, side) = match before {
                Some(anchor) => (anchor, core::plan::Side::Before),
                None => (after.expect("clap requires --before or --after"), core::plan::Side::After),
            };
            // The column is read as `plan` orders it — the main roster with each
            // mapped change's worktree copy — and each rank is written into its
            // own change's copy, the one that view reads. The overlay itself
            // writes the main copy only, so the writes are routed here.
            let (ws, fs_store) = open_project()?;
            let (facts, overlaid) = worktree_overlay(&ws, &fs_store);
            let write = |change: &str, key: &str| {
                core::model::set_board_rank(&home_store_for(&ws, &facts, change), change, key)
            };
            core::plan::set_rank(&overlaid, &write, &name, &anchor, side, force)?;
            render_rank(&name, &anchor, side, json)
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
        ChangeCommands::Rank { .. } => {
            unreachable!("dispatch refuses change rank in remote mode before the handshake")
        }
    }
}

/// One success line naming the side it landed on; `--json` carries the same
/// three facts — the rank value itself never reaches CLI output.
fn render_rank(change: &str, anchor: &str, side: core::plan::Side, json: bool) -> Result<()> {
    if json {
        return print_json(&serde_json::json!({
            "change": change,
            "anchor": anchor,
            "position": side.as_str(),
        }));
    }
    println!(
        "{} {} ranked {} {}",
        color::green("✓"),
        change,
        side.as_str(),
        anchor
    );
    Ok(())
}

/// The store a change lives in: its worktree copy while the change is mapped
/// to one and that copy still holds it, else the main checkout. The existence
/// check is the one the worktree overlay's reads fall back on, so a write
/// routed here lands in the copy those reads came from.
fn home_store_for(
    ws: &Workspace,
    facts: &speclink_host::worktree::WorktreeFacts,
    change: &str,
) -> speclink_fs::FsStore {
    facts
        .get(change)
        .map(|e| speclink_fs::FsStore::new(&e.path, &ws.spec_dir_name))
        .filter(|copy| copy.change_exists(change))
        .unwrap_or_else(|| speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name))
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
        if a.strict_overlap {
            core::command::Command::PlanStrict
        } else {
            core::command::Command::Plan { ranks: None }
        },
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
/// and up to three notes in a fixed order — `blocked by:`, `archive after:`,
/// `conflicts with:`, each left out when empty — skipped changes after the
/// waves, and `next:` last. Every styled piece goes through `color`, so
/// `--no-color` prints no ANSI at all.
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
            let mut conflicts: Vec<&str> = Vec::new();
            for o in c.requirement_overlap.iter().filter(|o| o.conflict) {
                if !conflicts.contains(&o.change.as_str()) {
                    conflicts.push(&o.change);
                }
            }
            let mut notes = String::new();
            for (label, names) in [
                ("blocked by", c.blocked_by.iter().map(String::as_str).collect()),
                ("archive after", c.archive_after.iter().map(String::as_str).collect()),
                ("conflicts with", conflicts),
            ] {
                if !names.is_empty() {
                    notes.push_str(&format!(" — {label}: {}", names.join(", ")));
                }
            }
            if !notes.is_empty() {
                notes = color::dim(&notes);
            }
            println!(
                "  {} {} [{}]{notes}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_host::worktree::{WorktreeEntry, WorktreeFacts};

    #[test]
    fn home_store_follows_the_copy_that_still_holds_the_change() {
        // 寫入路由與 overlay 的讀取同一個存在判定：映射的 worktree 還有這個 change
        // 就用那份；探查之後被移除（副本裡已沒有 change 目錄）就退回主 checkout。
        let base = std::env::temp_dir().join(format!("speclink-home-store-{}", std::process::id()));
        let (main, wt) = (base.join("main"), base.join("wt"));
        for (root, rank) in [(&main, "m"), (&wt, "w")] {
            let dir = root.join("openspec/changes/c");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(".openspec.yaml"), format!("schema: spec-driven\nboard_rank: {rank}\n")).unwrap();
        }
        let ws = Workspace { root: main.clone(), spec_dir_name: "openspec".to_string() };
        let facts: WorktreeFacts = [(
            "c".to_string(),
            WorktreeEntry { path: wt.clone(), branch: "speclink/c".to_string() },
        )]
        .into();
        let rank = |store: &speclink_fs::FsStore| {
            core::model::ChangeMeta::from_text(store.read_change_meta("c").as_deref())
                .expect("meta parses")
                .board_rank
        };
        assert_eq!(rank(&home_store_for(&ws, &facts, "c")).as_deref(), Some("w"));
        std::fs::remove_dir_all(wt.join("openspec/changes/c")).unwrap();
        assert_eq!(rank(&home_store_for(&ws, &facts, "c")).as_deref(), Some("m"));
        assert_eq!(rank(&home_store_for(&ws, &WorktreeFacts::new(), "c")).as_deref(), Some("m"));
        std::fs::remove_dir_all(&base).unwrap();
    }
}

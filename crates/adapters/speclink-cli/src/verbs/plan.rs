//! Execution-order verbs: `plan` (read), `change depends`, `change schema` and
//! `change rank` (writes); and `change landed`, the landing report of the same
//! `change` family (station-status-and-landing D5).
//!
//! The family's `field` verbs (host-free-fields D4) are here too.
//!
//! All of them are Dual (add-change-plan-remote D4; change-schema-verb D5;
//! skills-verbs-first-wave D6): the fs arms read and write the local spec
//! tree; the remote arms call the server endpoints and pass the response to
//! the same output functions (speclink-verbs' `render::plan`), so both modes
//! print alike. `change rank` writes the card metas' board rank locally; the
//! server writes its scope's board resource.

use anyhow::Result;
use speclink_core as core;

use crate::common::{
    emit, engine_command, open_project, plain_command, render_options, run, worktree_facts, worktree_overlay,
};
use crate::remote_base::RemoteCtx;
use core::store::Store;
use core::workspace::Workspace;
use speclink_verbs::families::plan::{rank_anchor, ChangeCommands, ChangeVerbArgs, PlanArgs, NO_COMMIT_TO_REPORT};
use speclink_verbs::render::fields::{self as render_fields, FieldKind};
use speclink_verbs::render::plan::{self as render, Depends};
use speclink_verbs::{ToCommand, VerbInputs};

pub(crate) fn cmd_change(a: ChangeVerbArgs) -> Result<()> {
    match &a.command {
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
            let home = home_store_for(&ws, &worktree_facts(&ws, &fs_store), name);
            let cmd = plain_command(&a)?;
            let o: core::command::DependsOutcome = run(&home, Some(&ws), cmd)?;
            emit(render::depends_declared(&o, on, *remove, render_options(*json)));
        }
        ChangeCommands::Schema { name, json, .. } => {
            // Routed like `change depends`: a mapped change switches in its
            // worktree copy, the one `list` and `plan` read — and a
            // project-layer schema resolves from that same copy.
            let (ws, fs_store) = open_project()?;
            let home = home_workspace_for(&ws, &worktree_facts(&ws, &fs_store), name);
            let store = speclink_fs::FsStore::new(&home.root, &home.spec_dir_name);
            let cmd = plain_command(&a)?;
            let o: core::command::SchemaOutcome = run(&store, Some(&home), cmd)?;
            emit(render::schema(&o, render_options(*json)));
        }
        ChangeCommands::Rank {
            name,
            before,
            after,
            force,
            json,
        } => {
            let (anchor, side) = rank_anchor(before, after);
            // The column is read as `plan` orders it — the main roster with each
            // mapped change's worktree copy — and each rank is written into its
            // own change's copy, the one that view reads. The overlay itself
            // writes the main copy only, so the writes are routed here.
            let (ws, fs_store) = open_project()?;
            let (facts, overlaid) = worktree_overlay(&ws, &fs_store);
            let write = |change: &str, key: &str| {
                core::model::set_board_rank(&home_store_for(&ws, &facts, change), change, key)
            };
            core::plan::set_rank(&overlaid, &write, name, &anchor, side, *force)?;
            let ranked = core::command::ChangeRankOutcome { change: name.clone(), anchor, position: side };
            emit(render::rank(&ranked, render_options(*json)));
        }
        ChangeCommands::Landed { json, .. } => {
            // The checkout's own copy takes the record, not a worktree's: the
            // report follows a commit or a merge made here, and after a merge
            // the worktree's copy is about to be removed with it. Without
            // `--commit` the commit is this checkout's HEAD.
            let (ws, store) = open_project()?;
            let inputs = VerbInputs { head_commit: core::tasks::head_commit(&ws.root), ..Default::default() };
            let cmd = engine_command(a.to_command(inputs))?;
            let o: core::command::ChangeLandedOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::landed(&o, render_options(*json)));
        }
        ChangeCommands::Field(field) => {
            // Routed like `change depends`: a change checked out in a worktree
            // lives in that copy, the one `list` reads.
            let (ws, fs_store) = open_project()?;
            let home = home_store_for(&ws, &worktree_facts(&ws, &fs_store), field.subject());
            let cmd = plain_command(&a)?;
            let o: core::command::FieldsOutcome = run(&home, Some(&ws), cmd)?;
            emit(render_fields::edited(FieldKind::Change, &o, render_options(field.json())));
        }
    }
    Ok(())
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
            emit(render::depends(
                &Depends {
                    change: &written.change,
                    depends_on: &written.depends_on,
                    on: &on,
                    remove,
                },
                render_options(json),
            ));
        }
        ChangeCommands::Schema { name, schema, json } => {
            let w = ctx.client.set_schema(&name, &schema)?;
            let o = core::command::SchemaOutcome {
                change: w.change,
                schema: w.schema,
                previous: w.previous,
                changed: w.changed,
                verify_stamp_cleared: w.verify_stamp_cleared,
            };
            emit(render::schema(&o, render_options(json)));
        }
        ChangeCommands::Rank { name, before, after, force, json } => {
            let (anchor, side) = rank_anchor(&before, &after);
            let r = ctx.client.rank_change(&name, &anchor, side.as_str(), force)?;
            let Some(position) = core::plan::Side::parse(&r.position) else {
                anyhow::bail!(
                    "unexpected server response — update speclink or report a bug (position '{}')",
                    r.position
                );
            };
            let ranked = core::command::ChangeRankOutcome { change: r.change, anchor: r.anchor, position };
            emit(render::rank(&ranked, render_options(json)));
        }
        ChangeCommands::Landed { name, commit, branch, json } => {
            // The server has no repo: the commit comes from the flag or from
            // this checkout's HEAD, and nothing is sent without one.
            let Some(commit) = commit.or_else(|| core::tasks::head_commit(&ctx.ws.root)) else {
                anyhow::bail!(NO_COMMIT_TO_REPORT);
            };
            let r = ctx.client.change_landed(&name, &commit, branch.as_deref())?;
            let landed = core::command::ChangeLandedOutcome { change: r.change, commit: r.commit, branch: r.branch };
            emit(render::landed(&landed, render_options(json)));
        }
        ChangeCommands::Field(field) => ctx.edit_fields(FieldKind::Change, &field)?,
    }
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
    let home = home_workspace_for(ws, facts, change);
    speclink_fs::FsStore::new(&home.root, &home.spec_dir_name)
}

/// The checkout [`home_store_for`] writes in, as a workspace: the mapped
/// worktree while that copy still holds the change, else the main checkout.
fn home_workspace_for(
    ws: &Workspace,
    facts: &speclink_host::worktree::WorktreeFacts,
    change: &str,
) -> Workspace {
    facts
        .get(change)
        .map(|e| Workspace { root: e.path.clone(), spec_dir_name: ws.spec_dir_name.clone() })
        .filter(|copy| speclink_fs::FsStore::new(&copy.root, &copy.spec_dir_name).change_exists(change))
        .unwrap_or_else(|| ws.clone())
}

pub(crate) fn cmd_plan(a: PlanArgs) -> Result<()> {
    let (ws, fs_store) = open_project()?;
    // Same observation surface as `list` (design D6): in a main checkout with
    // the worktree policy on, a worktree's checked tasks and started stamp feed
    // the stage derivation; everywhere else the plain store is read.
    let (facts, overlaid) = worktree_overlay(&ws, &fs_store);
    // fs mode gives no rank table: each change's own board rank orders it.
    let cmd = plain_command(&a)?;
    let plan: core::command::PlanReport =
        run(if facts.is_empty() { &fs_store } else { &overlaid }, Some(&ws), cmd)?;
    emit(render::plan(&plan, render_options(a.json)));
    Ok(())
}

/// Remote arm: the server plans the scope (its board resource is the rank
/// source); the response converts back to the engine's report so the shared
/// output function renders it — a cycle arrives as a refused 409 carrying the
/// engine line.
pub(crate) fn remote_plan(ctx: &RemoteCtx, a: &PlanArgs) -> Result<()> {
    // `--strict-overlap` asks the server for the plan before requirement-level
    // overlap; the payload has the same shape either way.
    let response = if a.strict_overlap { ctx.client.plan_strict()? } else { ctx.client.plan()? };
    let plan = speclink_remote::convert::plan_report(response)?;
    emit(render::plan(&plan, render_options(a.json)));
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

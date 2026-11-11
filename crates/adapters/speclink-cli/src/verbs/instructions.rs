//! The instructions verb: artifact guidance and the apply payload.
//!
//! Dual, with one pre-mode branch: `--skill` prints a constant skill body and
//! never touches the store, so dispatch checks it before resolving the mode.

use anyhow::{bail, Result};
use speclink_core as core;
use speclink_protocol::query as protocol_query;
use speclink_remote::client::ContextSnapshotOutcome;

use crate::common::{emit, info_if_no_changes, open_project, plain_command, render_options, run};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::instructions::InstructionsArgs;
use speclink_verbs::render::instructions as render;

/// `instructions --skill` 的 ModeFree 路徑：常數技能文本，dispatch 已分流。
pub(crate) fn cmd_instructions_skill(skill: &str) -> Result<()> {
    let body = core::skills::skill_body(skill)
        .ok_or_else(|| anyhow::anyhow!("Unknown skill: {skill}"))?;
    print!("{body}");
    Ok(())
}
pub(crate) fn cmd_instructions(a: InstructionsArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    if info_if_no_changes(&store, a.change.as_deref()) {
        return Ok(());
    }
    let cmd = plain_command(&a)?;
    let instr: core::command::InstructionsOutcome = run(&store, Some(&ws), cmd)?;
    emit(render::instructions(&instr, render_options(a.json)));
    Ok(())
}
fn to_apply_instructions(
    p: protocol_query::ApplyInstructions,
) -> core::instructions::ApplyInstructions {
    core::instructions::ApplyInstructions {
        change_name: p.change_name,
        change_dir: p.change_dir,
        schema_name: p.schema_name,
        context_files: p.context_files,
        progress: core::instructions::Progress {
            total: p.progress.total,
            complete: p.progress.complete,
            remaining: p.progress.remaining,
            code_total: p.progress.code_total,
            code_complete: p.progress.code_complete,
            code_remaining: p.progress.code_remaining,
        },
        tasks: p
            .tasks
            .into_iter()
            .map(|t| core::instructions::TaskJson {
                id: t.id,
                description: t.description,
                done: t.done,
                manual: t.manual,
            })
            .collect(),
        state: p.state,
        missing_artifacts: p.missing_artifacts,
        locale: p.locale,
        tdd: p.tdd,
        audit: p.audit,
        instruction: p.instruction,
        // Deliberately fs-only (local file checks) — the wire contract
        // omits it, so the remote payload never renders one.
        preflight: None,
    }
}
fn to_artifact_instructions(
    p: protocol_query::ArtifactInstructions,
) -> core::instructions::ArtifactInstructions {
    core::instructions::ArtifactInstructions {
        change_name: p.change_name,
        artifact_id: p.artifact_id,
        schema_name: p.schema_name,
        change_dir: p.change_dir,
        output_path: p.output_path,
        description: p.description,
        instruction: p.instruction,
        context: p.context,
        rules: p.rules,
        locale: p.locale,
        template: p.template,
        dependencies: p
            .dependencies
            .into_iter()
            .map(|d| core::instructions::Dependency {
                id: d.id,
                done: d.done,
                path: d.path,
                description: d.description,
            })
            .collect(),
        unlocks: p.unlocks,
    }
}
/// Snapshot source for the projection materializer: one consistent snapshot
/// already fetched from the Context API. Flow narrowing is the materializer's
/// job (design 決策三), so the provider returns the fetched snapshot verbatim —
/// the provider seam keeps the Context API call out of the materializer.
struct VerbContextProvider {
    snapshot: speclink_protocol::context::ContextSnapshot,
}
impl speclink_host::projection::SnapshotProvider for VerbContextProvider {
    fn snapshot(
        &self,
        _request: &speclink_protocol::context::ContextSnapshotRequest,
    ) -> Result<speclink_protocol::context::ContextSnapshot> {
        Ok(self.snapshot.clone())
    }
}
/// The remote verb flow's projection refresh: fetch one consistent Context API
/// snapshot for this change's apply flow, then materialize it and point
/// contextFiles into the projection. The manifest's current snapshot id travels
/// as `If-None-Match`, so an unchanged scope returns 304 and the rewrite is
/// skipped (免重寫). Projection trouble is a loud warning, never a verb failure —
/// the instructions payload is intact either way, and a failed fetch marks the
/// existing projection stale rather than serving it silently.
fn point_context_files_at_projection(
    ctx: &RemoteCtx,
    name: &str,
    context_files: &mut std::collections::BTreeMap<String, String>,
) {
    let request = speclink_protocol::context::ContextSnapshotRequest {
        change: Some(name.to_string()),
        flow: Some("apply".to_string()),
    };
    let known = speclink_host::projection::current_snapshot_id(&ctx.ws);
    match ctx.client.context_snapshot(&request, known.as_deref()) {
        // Unchanged since the projection's snapshot id: leave it untouched.
        Ok(ContextSnapshotOutcome::Unchanged) => {}
        Ok(ContextSnapshotOutcome::Fresh(snapshot)) => {
            let provider = VerbContextProvider { snapshot };
            match speclink_host::projection::materialize(&ctx.ws, &provider, &request) {
                Ok(out) => {
                    for w in &out.warnings {
                        eprintln!("speclink: warning: {w}");
                    }
                }
                Err(e) => eprintln!("speclink: warning: context projection not refreshed: {e:#}"),
            }
        }
        Err(e) => {
            eprintln!("speclink: warning: context projection not refreshed: {e}");
            // Keep the existing projection but flag it stale. No projection yet
            // is a no-op (mark_stale bails, which we ignore).
            let _ = speclink_host::projection::mark_stale(&ctx.ws);
        }
    }
    core::instructions::project_context_files(
        context_files,
        &speclink_host::projection::projection_dir(&ctx.ws)
            .join(speclink_host::projection::mirror_root(&ctx.ws)),
        name,
    );
}
pub(crate) fn remote_instructions(ctx: &RemoteCtx, a: &InstructionsArgs) -> Result<()> {
    if a.schema.is_some() {
        bail!("--schema is not supported in remote mode — the server's workflow config decides the schema");
    }
    let Some(name) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    // No-arg default mirrors fs mode: the first incomplete artifact (the
    // server's artifact list is already in display order), else "apply".
    let artifact = match a.artifact.as_deref() {
        Some(s) => s.to_string(),
        None => ctx
            .client
            .get_change(&name)?
            .artifacts
            .iter()
            .find(|x| x.status != "done")
            .map(|x| x.id.clone())
            .unwrap_or_else(|| "apply".to_string()),
    };
    // Both payloads take the fs arm's outcome shape and go through the same
    // output function.
    let instr = if artifact == "apply" {
        let mut p = to_apply_instructions(ctx.client.apply_instructions(&name)?);
        point_context_files_at_projection(ctx, &name, &mut p.context_files);
        core::command::InstructionsOutcome::Apply(p)
    } else {
        core::command::InstructionsOutcome::Artifact(to_artifact_instructions(
            ctx.client.artifact_instructions(&name, &artifact)?,
        ))
    };
    emit(render::instructions(&instr, render_options(a.json)));
    Ok(())
}

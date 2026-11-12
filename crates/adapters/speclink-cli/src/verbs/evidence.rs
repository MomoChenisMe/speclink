//! `evidence show` — the files each completed task of a change touched.
//!
//! Dual: the fs arm runs the engine command on the local store; the remote
//! arm reads the same record off the wire and turns it back into the engine's
//! result, so one output function prints both.

use anyhow::Result;
use speclink_core as core;

use crate::common::{emit, open_project, plain_command, render_options, run};
use crate::remote_base::RemoteCtx;
use speclink_verbs::families::evidence::{EvidenceArgs, EvidenceCommands};
use speclink_verbs::render::evidence as render;

pub(crate) fn cmd_evidence(a: EvidenceArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let shown: core::command::EvidenceShowOutcome = run(&store, Some(&ws), plain_command(&a)?)?;
    let EvidenceCommands::Show { json, .. } = a.command;
    emit(render::show(&shown, render_options(json)));
    Ok(())
}

/// Remote arm: the server looks the name up with the engine's own rule (an
/// active change, else its archive); its 404 carries the engine's line, which
/// `?` prints exactly as fs mode does. A server from before that lookup is
/// refused by the conversion.
pub(crate) fn remote_evidence(ctx: &RemoteCtx, a: EvidenceArgs) -> Result<()> {
    match a.command {
        EvidenceCommands::Show { change, json } => {
            let shown = speclink_remote::convert::evidence_show(ctx.client.change_evidence(&change)?)?;
            emit(render::show(&shown, render_options(json)));
        }
    }
    Ok(())
}

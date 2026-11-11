//! Creation verbs: `new change` and `new artifact`.
//!
//! Dual. The `Path:` line is fs-only by design — a server-side path means
//! nothing to a local caller — so the two modes differ there on purpose.

use anyhow::{bail, Result};
use speclink_core as core;
use speclink_protocol::command::CreateChangeRequest;

use crate::common::{
    emit, engine_command, open_project, plain_command, read_stdin, render_options, run, stdin_for,
};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::new::{NewArgs, NewArtifactArgs, NewChangeArgs, NewCommands};
use speclink_verbs::render::new::{self as render, NewArtifact, NewChangeLines};
use speclink_verbs::{ToCommand, VerbInputs};

pub(crate) fn cmd_new(a: NewArgs) -> Result<()> {
    match a.command {
        NewCommands::Change(c) => cmd_new_change(c),
        NewCommands::Artifact(c) => cmd_new_artifact(c),
    }
}
fn cmd_new_change(a: NewChangeArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let cmd = plain_command(&a)?;
    let o: core::command::NewChangeOutcome = run(&store, Some(&ws), cmd)?;
    emit(render::change_created(&o, a.from_discussion.as_deref(), render_options(false)));
    Ok(())
}
fn cmd_new_artifact(a: NewArtifactArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let stdin = stdin_for(a.stdin_policy());
    let cmd = engine_command(a.to_command(VerbInputs { stdin, ..Default::default() }))?;
    let o: core::command::NewArtifactOutcome = run(&store, Some(&ws), cmd)?;
    emit(render::artifact_created(&o, &a.artifact_type, render_options(a.json)));
    Ok(())
}
/// new 的 remote 家族臂：子指令 enum 窮盡 match、無 catch-all。
pub(crate) fn remote_new(ctx: &RemoteCtx, a: NewArgs) -> Result<()> {
    match a.command {
        NewCommands::Change(c) => remote_new_change(ctx, &c),
        NewCommands::Artifact(c) => remote_new_artifact(ctx, &c),
    }
}
fn remote_new_change(ctx: &RemoteCtx, a: &NewChangeArgs) -> Result<()> {
    let resp = ctx.client.create_change(CreateChangeRequest {
        name: a.name.clone(),
        schema: a.schema.clone(),
        description: a.description.clone(),
        agent: a.agent.clone(),
        from_discussion: a.from_discussion.clone(),
        last: a.last,
    })?;
    // Path 行是明文分歧（design D5）：server 端目錄對本機使用者無意義。
    emit(render::new_change(
        &a.name,
        NewChangeLines {
            path: None,
            schema: resp.schema.as_deref().filter(|s| !s.is_empty()),
            from_discussion: a.from_discussion.as_deref(),
        },
        render_options(false),
    ));
    Ok(())
}
/// Map the fs artifact TYPE argument onto the contract's artifact path.
fn remote_artifact_path(artifact_type: &str, capability: Option<&str>) -> Result<(String, &'static str)> {
    match artifact_type {
        "proposal" => Ok(("proposal".to_string(), "proposal")),
        "design" => Ok(("design".to_string(), "design")),
        "tasks" => Ok(("tasks".to_string(), "tasks")),
        "spec" => {
            let cap = capability
                .ok_or_else(|| anyhow::anyhow!("Capability name required for spec artifacts"))?;
            Ok((format!("specs/{cap}"), "specs"))
        }
        other => bail!("Unknown artifact type '{other}'. Valid types: proposal, design, tasks, spec"),
    }
}
fn remote_new_artifact(ctx: &RemoteCtx, a: &NewArtifactArgs) -> Result<()> {
    let (artifact_path, schema_artifact_id) =
        remote_artifact_path(&a.artifact_type, a.capability.as_deref())?;
    let content = if a.stdin {
        read_stdin()
    } else {
        // Template comes from the server's workflow schema, rendered by the
        // embedded engine (built-in/user schema definitions are engine-local).
        let schema_name = ctx.client.config()?.schema;
        let name = if schema_name.is_empty() { "spec-driven".to_string() } else { schema_name };
        match core::schema::resolve_with(None, Some(&speclink_host::context::global_config_dir()), &name) {
            Some(Ok(schema)) => schema
                .artifact(schema_artifact_id)
                .and_then(|art| art.template.clone())
                .unwrap_or_default(),
            _ => String::new(),
        }
    };
    let Some(change) =
        remote_resolve_change(ctx, a.change.as_deref(), "Use --change to specify one:")?
    else {
        return Ok(());
    };
    // --force overwrites: re-read the current version so the write still
    // asserts what it replaces; plain create asserts absence (If-Match: 0).
    let version = if a.force {
        ctx.client
            .get_artifact(&change, &artifact_path)
            .map(|got| got.version)
            .unwrap_or(0)
    } else {
        0
    };
    ctx.client.put_artifact(&change, &artifact_path, &content, version)?;
    emit(render::new_artifact(
        &NewArtifact {
            artifact_type: &a.artifact_type,
            change: &change,
            path: &artifact_path,
            validated: a.stdin,
        },
        render_options(a.json),
    ));
    Ok(())
}

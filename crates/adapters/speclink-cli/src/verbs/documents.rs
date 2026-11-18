//! Document verbs: `artifact cat`, `language show` and `language replace`.
//!
//! All are Dual: the fs arm goes through the store, the remote arm reads or
//! writes the same document over the wire, and an artifact id's shape is
//! validated locally either way.

use anyhow::{bail, Result};

use crate::common::{emit, engine_command, open_project, plain_command, render_options, run, stdin_for};
use speclink_core::command::LanguageReplaceOutcome;
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::documents::{
    ArtifactArgs, ArtifactCommands, LanguageArgs, LanguageCommands,
};
use speclink_verbs::render::documents as render;
use speclink_verbs::{ToCommand, VerbInputs};

/// Map a `speclink artifact cat` id onto the store's artifact file path
/// (`specs/<capability>` addresses a delta spec).
fn artifact_rel_path(artifact: &str) -> Result<String> {
    match artifact {
        "proposal" => Ok("proposal.md".to_string()),
        "design" => Ok("design.md".to_string()),
        "tasks" => Ok("tasks.md".to_string()),
        _ => match artifact.strip_prefix("specs/") {
            Some(cap) if !cap.is_empty() && !cap.contains('/') => {
                Ok(format!("specs/{cap}/spec.md"))
            }
            _ => bail!(
                "Unknown artifact '{artifact}'. Use proposal, design, tasks, or specs/<capability>"
            ),
        },
    }
}
/// The fs arm of the two reads: run the read and print the document as-is.
pub(crate) fn cmd_document(a: impl ToCommand) -> Result<()> {
    let (ws, store) = open_project()?;
    let content: String = run(&store, Some(&ws), plain_command(&a)?)?;
    emit(render::document(&content, render_options(false)));
    Ok(())
}
/// language 的本機家族臂：`show` 讀出文件，`replace` 把 stdin 整份寫入。
pub(crate) fn cmd_language(a: LanguageArgs) -> Result<()> {
    match a.command {
        LanguageCommands::Show => cmd_document(a),
        LanguageCommands::Replace { json, .. } => {
            let (ws, store) = open_project()?;
            let stdin = stdin_for(a.stdin_policy());
            let cmd = engine_command(a.to_command(VerbInputs { stdin, ..Default::default() }))?;
            let o: LanguageReplaceOutcome = run(&store, Some(&ws), cmd)?;
            emit(render::replaced(&o, render_options(json)));
            Ok(())
        }
    }
}
/// artifact 的 remote 家族臂：子指令 enum 窮盡 match、無 catch-all——新增
/// 子指令時本機與 remote 兩臂皆編譯不過。
pub(crate) fn remote_artifact(ctx: &RemoteCtx, a: ArtifactArgs) -> Result<()> {
    match a.command {
        ArtifactCommands::Cat { artifact, change } => {
            remote_artifact_cat(ctx, &artifact, change.as_deref())
        }
    }
}
/// language 的 remote 家族臂。
pub(crate) fn remote_language(ctx: &RemoteCtx, a: LanguageArgs) -> Result<()> {
    match a.command {
        LanguageCommands::Show => {
            emit(render::document(&ctx.client.language()?.content, render_options(false)));
            Ok(())
        }
        LanguageCommands::Replace { json, .. } => {
            // 內容原樣上線；空內容由 Server 端的引擎拒絕（檢查只有一處）。
            let content = stdin_for(a.stdin_policy()).unwrap_or_default();
            let created = ctx.client.replace_language(&content)?.created;
            emit(render::replaced(&LanguageReplaceOutcome { created }, render_options(json)));
            Ok(())
        }
    }
}
fn remote_artifact_cat(ctx: &RemoteCtx, artifact: &str, change: Option<&str>) -> Result<()> {
    // Validate the id shape locally so both modes reject the same inputs.
    let _ = artifact_rel_path(artifact)?;
    let Some(change) = remote_resolve_change(ctx, change, "Use --change to specify one:")? else {
        return Ok(());
    };
    let content = ctx.client.get_artifact(&change, artifact)?.content;
    emit(render::document(&content, render_options(false)));
    Ok(())
}

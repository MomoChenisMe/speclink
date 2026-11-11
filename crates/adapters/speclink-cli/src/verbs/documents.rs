//! Document-read verbs: `artifact cat` and `language show`.
//!
//! Both are Dual: the fs arm reads through the store, the remote arm reads the
//! same document off the wire, and the id shape is validated locally either way.

use anyhow::{bail, Result};

use crate::common::{open_project, plain_command, run};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use speclink_verbs::families::documents::{
    ArtifactArgs, ArtifactCommands, LanguageArgs, LanguageCommands,
};
use speclink_verbs::ToCommand;

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
/// The fs arm of both verbs: run the read and print the document as-is.
pub(crate) fn cmd_document(a: impl ToCommand) -> Result<()> {
    let (ws, store) = open_project()?;
    let content: String = run(&store, Some(&ws), plain_command(&a)?)?;
    print!("{content}");
    Ok(())
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
            print!("{}", ctx.client.language()?.content);
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
    print!("{}", ctx.client.get_artifact(&change, artifact)?.content);
    Ok(())
}

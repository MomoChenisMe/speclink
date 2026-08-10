//! Document-read verbs: `artifact cat` and `language show`.
//!
//! Both are Dual: the fs arm reads through the store, the remote arm reads the
//! same document off the wire, and the id shape is validated locally either way.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::common::{open_project, run_command};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::store::Store;

#[derive(Args)]
pub(crate) struct ArtifactArgs {
    #[command(subcommand)]
    command: ArtifactCommands,
}
#[derive(Subcommand)]
enum ArtifactCommands {
    /// Print an artifact's content (proposal | design | tasks | specs/<capability>)
    Cat {
        /// Artifact id
        artifact: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
    },
}
#[derive(Args)]
pub(crate) struct LanguageArgs {
    #[command(subcommand)]
    command: LanguageCommands,
}
#[derive(Subcommand)]
enum LanguageCommands {
    /// Print the project's shared vocabulary (LANGUAGE document)
    Show,
}
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
pub(crate) fn cmd_artifact(a: ArtifactArgs) -> Result<()> {
    match a.command {
        ArtifactCommands::Cat { artifact, change } => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(
                store,
                Some(&ws),
                core::command::Command::ArtifactCat { artifact, change },
            )?;
            let core::command::CommandOutcome::ArtifactCat(content) = outcome else {
                unreachable!("artifact cat yields raw content");
            };
            print!("{content}");
            Ok(())
        }
    }
}
pub(crate) fn cmd_language(a: LanguageArgs) -> Result<()> {
    match a.command {
        LanguageCommands::Show => {
            let (ws, store) = open_project()?;
            let store: &dyn Store = &store;
            let outcome = run_command(store, Some(&ws), core::command::Command::LanguageShow)?;
            let core::command::CommandOutcome::Language(content) = outcome else {
                unreachable!("language show yields raw content");
            };
            print!("{content}");
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
            print!("{}", ctx.client.language()?.content);
            Ok(())
        }
    }
}
fn remote_artifact_cat(ctx: &RemoteCtx, artifact: &str, change: Option<&str>) -> Result<()> {
    // Validate the id shape locally so both modes reject the same inputs.
    let _ = artifact_rel_path(artifact)?;
    let change = match change {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
    };
    print!("{}", ctx.client.get_artifact(&change, artifact)?.content);
    Ok(())
}

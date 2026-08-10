//! Creation verbs: `new change` and `new artifact`.
//!
//! Dual. The `Path:` line is fs-only by design — a server-side path means
//! nothing to a local caller — so the two modes differ there on purpose.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;
use speclink_protocol::command::CreateChangeRequest;

use crate::color;
use crate::common::{open_project, read_stdin, run_command};
use crate::remote_base::{remote_resolve_change, RemoteCtx};
use core::store::Store;

#[derive(Args)]
pub(crate) struct NewArgs {
    #[command(subcommand)]
    command: NewCommands,
}
#[derive(Subcommand)]
enum NewCommands {
    /// Create a new change
    Change(NewChangeArgs),
    /// Create a new artifact file for a change
    Artifact(NewArtifactArgs),
}
#[derive(Args)]
struct NewChangeArgs {
    /// Change name (kebab-case)
    name: String,
    /// Description
    #[arg(long)]
    description: Option<String>,
    /// Workflow schema to use
    #[arg(long)]
    schema: Option<String>,
    /// AI agent that created this change (e.g., claude, codex, gemini)
    #[arg(long)]
    agent: Option<String>,
    /// Link this change to a discussion document (writes from_discussion metadata)
    #[arg(long = "from-discussion")]
    from_discussion: Option<String>,
}
#[derive(Args)]
struct NewArtifactArgs {
    /// Artifact type: proposal, design, tasks, spec
    #[arg(name = "TYPE")]
    artifact_type: String,
    /// Capability name (required for spec type)
    capability: Option<String>,
    /// Change name
    #[arg(long)]
    change: Option<String>,
    /// Read content from stdin instead of using empty template
    #[arg(long)]
    stdin: bool,
    /// Overwrite existing artifact
    #[arg(long)]
    force: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
pub(crate) fn cmd_new(a: NewArgs) -> Result<()> {
    match a.command {
        NewCommands::Change(c) => cmd_new_change(c),
        NewCommands::Artifact(c) => cmd_new_artifact(c),
    }
}
fn cmd_new_change(a: NewChangeArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::NewChange {
            name: a.name.clone(),
            description: a.description.clone(),
            schema: a.schema.clone(),
            agent: a.agent.clone(),
            from_discussion: a.from_discussion.clone(),
        },
    )?;
    let core::command::CommandOutcome::NewChange(o) = outcome else {
        unreachable!("new change yields a new-change outcome");
    };
    render_new_change(
        &o.name,
        NewChangeLines {
            path: Some(&o.dir.to_string_lossy()),
            schema: Some(&o.schema),
            from_discussion: a.from_discussion.as_deref(),
        },
    );
    Ok(())
}
/// `new change` 成功輸出的選印行，具名綁定——三個相鄰同型參數寫反不會被
/// 編譯器擋，具名欄位會。`path` 缺席是 remote 的明文分歧（design D5）：
/// server 端目錄對本機使用者無意義；`schema` 同樣只在來源給得出時印。
struct NewChangeLines<'a> {
    path: Option<&'a str>,
    schema: Option<&'a str>,
    from_discussion: Option<&'a str>,
}
fn render_new_change(name: &str, lines: NewChangeLines<'_>) {
    println!("{} Created change: {name}", color::green("✓"));
    if let Some(path) = lines.path {
        println!("  Path: {path}");
    }
    if let Some(schema) = lines.schema {
        println!("  Schema: {schema}");
    }
    if let Some(slug) = lines.from_discussion {
        println!("  From discussion: {slug}");
    }
}
fn cmd_new_artifact(a: NewArtifactArgs) -> Result<()> {
    let (ws, store) = open_project()?;
    let store: &dyn Store = &store;
    let content = if a.stdin {
        Some(read_stdin())
    } else {
        None
    };
    let outcome = run_command(
        store,
        Some(&ws),
        core::command::Command::NewArtifact {
            kind: a.artifact_type.clone(),
            capability: a.capability.clone(),
            change: a.change.clone(),
            content,
            force: a.force,
        },
    )?;
    let core::command::CommandOutcome::NewArtifact(o) = outcome else {
        unreachable!("new artifact yields a new-artifact outcome");
    };
    if a.json {
        // Compact single-line JSON, frozen shape ("artifact" echoes the
        // input token, not the schema artifact id).
        let v = serde_json::json!({
            "artifact": a.artifact_type,
            "change": o.change,
            "path": o.path.to_string_lossy(),
            "status": "created",
            "validated": o.had_content,
            "warnings": [],
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Created {}: {}", color::green("✓"), a.artifact_type, o.path.to_string_lossy());
    if o.had_content {
        println!("  Content validated ✓");
    }
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
    })?;
    // Path 行是明文分歧（design D5）：server 端目錄對本機使用者無意義。
    render_new_change(
        &a.name,
        NewChangeLines {
            path: None,
            schema: resp.schema.as_deref().filter(|s| !s.is_empty()),
            from_discussion: a.from_discussion.as_deref(),
        },
    );
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
    let change = match a.change.as_deref() {
        Some(n) => n.to_string(),
        None => match remote_resolve_change(ctx, None, "Use --change to specify one:")? {
            Some(n) => n,
            None => return Ok(()),
        },
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
    if a.json {
        let v = serde_json::json!({
            "artifact": a.artifact_type,
            "change": change,
            "path": artifact_path,
            "status": "created",
            "validated": a.stdin,
            "warnings": [],
        });
        println!("{}", serde_json::to_string(&v)?);
        return Ok(());
    }
    println!("{} Created {}: {}", color::green("✓"), a.artifact_type, artifact_path);
    if a.stdin {
        println!("  Content validated ✓");
    }
    Ok(())
}

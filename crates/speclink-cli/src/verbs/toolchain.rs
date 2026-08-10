//! Toolchain verbs: schemas, templates, schema, completion, feedback, demo.
//!
//! ModeFree by declaration (design D1) except `demo`, which dispatch refuses in
//! remote mode — none of them reach the remote client.

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use speclink_core as core;

use crate::color;
use crate::common::{open_project, print_json, require_workspace};
use crate::Cli;

#[derive(Args)]
pub(crate) struct JsonFlag {
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
#[derive(Args)]
pub(crate) struct TemplatesArgs {
    /// Schema name
    #[arg(long)]
    schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}
#[derive(Args)]
pub(crate) struct FeedbackArgs {
    /// Feedback message
    message: String,
    /// Detailed body
    #[arg(long)]
    body: Option<String>,
}
#[derive(Args)]
pub(crate) struct SchemaArgs {
    #[command(subcommand)]
    command: SchemaCommands,
}
#[derive(Subcommand)]
enum SchemaCommands {
    /// Show where a schema is resolved from
    Which {
        /// Schema name
        name: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show all resolution paths
        #[arg(long)]
        all: bool,
    },
    /// Validate a schema
    Validate {
        /// Schema name
        name: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Fork (copy) a schema
    Fork {
        /// Source schema
        source: String,
        /// New schema name
        name: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Overwrite if exists
        #[arg(long)]
        force: bool,
    },
    /// Create a new custom schema
    Init {
        /// Schema name
        name: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Artifact IDs (comma-separated)
        #[arg(long)]
        artifacts: Option<String>,
        /// Set as default schema
        #[arg(long)]
        default: bool,
        /// Overwrite if exists
        #[arg(long)]
        force: bool,
    },
}
#[derive(Args)]
pub(crate) struct CompletionArgs {
    #[command(subcommand)]
    command: CompletionCommands,
}
#[derive(Subcommand)]
enum CompletionCommands {
    /// Generate completion script
    Generate {
        /// Shell type
        shell: Option<String>,
    },
    /// Install completion
    Install {
        /// Shell type
        shell: Option<String>,
        /// Verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Uninstall completion
    Uninstall {
        /// Shell type
        shell: Option<String>,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
pub(crate) fn cmd_schemas(a: JsonFlag) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd()?;
    let schemas = core::schema::list_all(ws.as_ref(), Some(&speclink_host::context::global_config_dir()));
    if a.json {
        let items: Vec<_> = schemas
            .iter()
            .map(|s| {
                serde_json::json!({
                    "artifacts": s.artifact_ids,
                    "description": s.description,
                    "name": s.name,
                    "source": s.source,
                })
            })
            .collect();
        return print_json(&items);
    }
    println!("Available schemas:");
    for s in &schemas {
        match &s.description {
            Some(d) => println!("  {} ({}) — {}", s.name, s.source, d),
            None => println!("  {} ({})", s.name, s.source),
        }
    }
    Ok(())
}
pub(crate) fn cmd_templates(a: TemplatesArgs) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd()?;
    let schema_name = a.schema.unwrap_or_else(|| "spec-driven".to_string());
    let schema = match core::schema::resolve_with(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &schema_name) {
        Some(Ok(s)) => s,
        Some(Err(e)) => bail!("{e}"),
        None => bail!("{}", core::schema::not_found_msg(&schema_name)),
    };
    if a.json {
        let items: Vec<_> = schema
            .artifacts
            .iter()
            .map(|art| {
                serde_json::json!({
                    "artifactId": art.id,
                    "hasContent": art.template.as_deref().map(|t| !t.is_empty()).unwrap_or(false),
                    "templateName": art.template_name,
                })
            })
            .collect();
        return print_json(&items);
    }
    println!("Templates ({})", schema.name);
    for art in &schema.artifacts {
        let sym = if art.template.as_deref().map(|t| !t.is_empty()).unwrap_or(false) { "✓" } else { "✗" };
        println!("  {sym} {} → {}", art.id, art.template_name);
    }
    Ok(())
}
pub(crate) fn cmd_feedback(a: FeedbackArgs) -> Result<()> {
    let _ = a.body;
    println!("Thanks for your feedback!");
    println!("Please open an issue at https://github.com/speclink-app/speclink/issues");
    println!("Message: {}", a.message);
    Ok(())
}
pub(crate) fn cmd_schema(a: SchemaArgs) -> Result<()> {
    let ws = core::workspace::Workspace::discover_cwd()?;
    match a.command {
        SchemaCommands::Which { name, all: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            let sources = core::schema::sources(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &n);
            if sources.is_empty() {
                // Unknown schema is informational, not an error (exit 0).
                println!("Schema: {n}");
                println!("Not found.");
                return Ok(());
            }
            let display = |s: &core::schema::SchemaSource| match &s.path {
                Some(p) => (p.to_string_lossy().to_string(), s.source),
                None => ("(embedded in binary)".to_string(), s.source),
            };
            if json {
                let items: Vec<_> = sources
                    .iter()
                    .map(|s| {
                        let (p, src) = display(s);
                        serde_json::json!({ "path": p, "source": src })
                    })
                    .collect();
                return print_json(&serde_json::json!({
                    "name": n,
                    "resolved": sources[0].source,
                    "sources": items,
                }));
            }
            println!("Schema: {n}");
            for (i, s) in sources.iter().enumerate() {
                let (p, src) = display(s);
                if i == 0 {
                    println!("  → {p} ({src})");
                } else {
                    println!("    {p} ({src})");
                }
            }
        }
        SchemaCommands::Validate { name, verbose: _, json } => {
            let n = name.unwrap_or_else(|| "spec-driven".to_string());
            match core::schema::resolve_with(ws.as_ref(), Some(&speclink_host::context::global_config_dir()), &n) {
                Some(Ok(s)) => {
                    let count = s.artifacts.len();
                    if json {
                        return print_json(&serde_json::json!({
                            "artifactCount": count,
                            "name": s.name,
                            "valid": true,
                        }));
                    }
                    println!("{} Schema '{}' is valid ({count} artifacts)", color::green("✓"), s.name);
                }
                Some(Err(detail)) => {
                    println!("Schema '{n}' is invalid: {detail}");
                    bail!("Schema validation failed: {detail}");
                }
                None => {
                    let detail = core::schema::not_found_msg(&n);
                    println!("Schema '{n}' is invalid: {detail}");
                    bail!("Schema validation failed: {detail}");
                }
            }
        }
        SchemaCommands::Fork { source, name, force, json: _ } => {
            let ws = require_workspace()?;
            let new_name = core::schema::fork(&ws, Some(&speclink_host::context::global_config_dir()), &source, name.as_deref(), force)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{} Forked '{source}' → '{new_name}'", color::green("✓"));
        }
        SchemaCommands::Init { name, description, artifacts, default: _, force } => {
            let ws = require_workspace()?;
            let dir = core::schema::init_schema(
                &ws,
                &name,
                artifacts.as_deref(),
                description.as_deref(),
                force,
            )
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{} Created schema '{name}' at {}", color::green("✓"), dir.display());
        }
    }
    Ok(())
}
/// Validated display name for a completion shell. Elvish IS supported, but the error message
/// only lists the four common shells — frozen verbatim.
fn completion_shell(shell: Option<&str>) -> Result<&'static str> {
    match shell.unwrap_or("bash") {
        "bash" => Ok("Bash"),
        "zsh" => Ok("Zsh"),
        "fish" => Ok("Fish"),
        "powershell" => Ok("PowerShell"),
        "elvish" => Ok("Elvish"),
        other => bail!("Unsupported shell: {other}. Use bash, zsh, fish, or powershell."),
    }
}
pub(crate) fn cmd_completion(a: CompletionArgs) -> Result<()> {
    match a.command {
        CompletionCommands::Generate { shell } => {
            use clap::CommandFactory;
            let sh = match completion_shell(shell.as_deref())? {
                "Zsh" => clap_complete::Shell::Zsh,
                "Fish" => clap_complete::Shell::Fish,
                "PowerShell" => clap_complete::Shell::PowerShell,
                "Elvish" => clap_complete::Shell::Elvish,
                _ => clap_complete::Shell::Bash,
            };
            let mut cmd = Cli::command();
            if sh == clap_complete::Shell::Bash {
                // The frozen bash script (from an older clap_complete) offers positional
                // value names as completion candidates ("[CHANGE]", "<KEY>"); newer
                // clap_complete dropped them, so they are re-injected here.
                let mut buf: Vec<u8> = Vec::new();
                clap_complete::generate(sh, &mut cmd, "speclink", &mut buf);
                let script = String::from_utf8_lossy(&buf).to_string();
                print!("{}", bash_inject_positionals(&script, &cmd));
                return Ok(());
            }
            clap_complete::generate(sh, &mut cmd, "speclink", &mut std::io::stdout());
        }
        CompletionCommands::Install { shell, verbose: _ } => {
            // The shell profile is never written to; guidance is printed instead.
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Shell completion for {name} — generate and source the output.");
            println!("Run: speclink completion generate {name} > completion_script");
            println!("Then source it in your shell profile.");
        }
        CompletionCommands::Uninstall { shell, yes: _ } => {
            let name = completion_shell(shell.as_deref())?;
            println!("Note: Remove the completion script for {name} from your shell profile.");
        }
    }
    Ok(())
}
/// Append positional value-name placeholders (`<KEY>`, `[CHANGE]`) to each `opts="..."`
/// line of a clap_complete bash script, matching the frozen older clap_complete output.
/// Command paths are recovered from the script's own `parent,child) cmd="label"` arms.
fn bash_inject_positionals(script: &str, root: &clap::Command) -> String {
    use std::collections::HashMap;
    // label -> command path (root label "speclink" -> []).
    let mut paths: HashMap<String, Vec<String>> = HashMap::new();
    paths.insert("speclink".to_string(), Vec::new());
    let lines: Vec<&str> = script.lines().collect();
    for w in lines.windows(2) {
        let arm = w[0].trim();
        let assign = w[1].trim();
        let (Some(arm), Some(label)) = (
            arm.strip_suffix(')'),
            assign.strip_prefix("cmd=\"").and_then(|s| s.strip_suffix('"')),
        ) else {
            continue;
        };
        if let Some((parent, child)) = arm.split_once(',') {
            if let Some(parent_path) = paths.get(parent).cloned() {
                let mut p = parent_path;
                p.push(child.to_string());
                paths.insert(label.to_string(), p);
            }
        }
    }
    let placeholder = |path: &[String]| -> String {
        let mut c = root;
        for name in path {
            match c.get_subcommands().find(|s| s.get_name() == *name) {
                Some(sub) => c = sub,
                None => return String::new(),
            }
        }
        if c.has_subcommands() {
            return String::new();
        }
        let mut out = String::new();
        for a in c.get_positionals() {
            let name = a
                .get_value_names()
                .and_then(|v| v.first().map(|s| s.to_string()))
                .unwrap_or_else(|| a.get_id().to_string().to_uppercase());
            if a.is_required_set() {
                out.push_str(&format!(" <{name}>"));
            } else {
                out.push_str(&format!(" [{name}]"));
            }
        }
        out
    };
    let mut out = String::new();
    let mut current_label: Option<String> = None;
    for line in script.lines() {
        let t = line.trim();
        if let Some(l) = t.strip_suffix(')') {
            if paths.contains_key(l) {
                current_label = Some(l.to_string());
            }
        }
        if let (Some(label), true) = (&current_label, t.starts_with("opts=\"")) {
            if let Some(path) = paths.get(label) {
                let ph = placeholder(path);
                if !ph.is_empty() {
                    if let Some(stripped) = line.strip_suffix('"') {
                        out.push_str(stripped);
                        out.push_str(&ph);
                        out.push('"');
                        out.push('\n');
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
pub(crate) fn cmd_demo() -> Result<()> {
    let (ws, store) = open_project()?;
    let outcome = core::demo::generate(&store, speclink_host::context::git_identity(&ws.root).as_deref())?;
    println!("{} Created demo change: {}", color::green("✓"), outcome.name);
    println!("  Theme: {}", outcome.theme);
    println!("  Path: {}", core::util::to_slash(&outcome.path));
    Ok(())
}

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use speclink_core as core;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

/// Speclink — spec management CLI
#[derive(Parser)]
#[command(name = "speclink", version, about = "Speclink — spec management CLI", disable_help_subcommand = false)]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Speclink in a project
    Init(InitArgs),
    /// Update instruction files
    Update(UpdateArgs),
    /// List changes or specs
    List(ListArgs),
    /// Show change or spec content
    Show(ShowArgs),
    /// Validate changes and specs
    Validate(ValidateArgs),
    /// Analyze change artifacts for consistency and gaps
    Analyze(ChangeArg),
    /// Detect drift between a change and the current codebase state
    Drift(ChangeArg),
    /// Archive a completed change
    Archive(ArchiveArgs),
    /// Show artifact DAG status
    Status(StatusArgs),
    /// Get instructions for an artifact
    Instructions(InstructionsArgs),
    /// Create a new change or resource
    New(NewArgs),
    /// List available workflow schemas
    Schemas(JsonFlag),
    /// Show template paths
    Templates(TemplatesArgs),
    /// Submit feedback
    Feedback(FeedbackArgs),
    /// Schema management commands
    Schema(SchemaArgs),
    /// Config management commands
    Config(ConfigArgs),
    /// Shell completion commands
    Completion(CompletionArgs),
    /// Task operations
    Task(TaskArgs),
    /// Manage in-progress markers
    #[command(name = "in-progress")]
    InProgress(InProgressArgs),
    /// Generate a demo change with sample data
    Demo,
    /// Discussion documents (record and evolve a discussion)
    Discuss(DiscussArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Project path (defaults to current directory)
    path: Option<PathBuf>,
    /// AI tools to generate files for (e.g., claude, codex)
    #[arg(long)]
    tools: Option<String>,
    /// Overwrite existing files
    #[arg(long)]
    force: bool,
    /// Custom openspec directory path (default: openspec)
    #[arg(long)]
    dir: Option<String>,
}

#[derive(Args)]
struct UpdateArgs {
    path: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    specs: bool,
    #[arg(long)]
    changes: bool,
    #[arg(long)]
    sort: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ShowArgs {
    item: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long = "item-type")]
    item_type: Option<String>,
    #[arg(long = "deltas-only")]
    deltas_only: bool,
    #[arg(short = 'r', long)]
    requirements: bool,
}

#[derive(Args)]
struct ValidateArgs {
    item: Option<String>,
    #[arg(long)]
    all: bool,
    #[arg(long)]
    changes: bool,
    #[arg(long)]
    specs: bool,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ChangeArg {
    change: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ArchiveArgs {
    change: Option<String>,
    #[arg(short = 'y', long)]
    yes: bool,
    #[arg(long = "skip-specs")]
    skip_specs: bool,
    #[arg(long = "no-validate")]
    no_validate: bool,
    #[arg(long = "mark-tasks-complete")]
    mark_tasks_complete: bool,
}

#[derive(Args)]
struct StatusArgs {
    #[arg(long)]
    change: Option<String>,
    #[arg(long)]
    schema: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct InstructionsArgs {
    artifact: Option<String>,
    #[arg(long)]
    change: Option<String>,
    #[arg(long)]
    schema: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    skill: Option<String>,
}

#[derive(Args)]
struct NewArgs {
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
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    schema: Option<String>,
    #[arg(long)]
    agent: Option<String>,
}

#[derive(Args)]
struct NewArtifactArgs {
    #[arg(name = "TYPE")]
    artifact_type: String,
    capability: Option<String>,
    #[arg(long)]
    change: Option<String>,
    #[arg(long)]
    stdin: bool,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TemplatesArgs {
    #[arg(long)]
    schema: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct FeedbackArgs {
    message: String,
    #[arg(long)]
    body: Option<String>,
}

#[derive(Args)]
struct SchemaArgs {
    #[command(subcommand)]
    command: SchemaCommands,
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Show where a schema is resolved from
    Which { name: Option<String>, #[arg(long)] all: bool },
    /// Validate a schema
    Validate { name: Option<String>, #[arg(long)] verbose: bool },
    /// Fork (copy) a schema
    Fork { source: String, name: Option<String>, #[arg(long)] force: bool },
    /// Create a new custom schema
    Init { name: String, #[arg(long)] artifacts: Option<String>, #[arg(long)] default: bool, #[arg(long)] description: Option<String> },
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
enum ConfigCommands {
    Path,
    List,
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
    Reset,
    Edit,
}

#[derive(Args)]
struct CompletionArgs {
    #[command(subcommand)]
    command: CompletionCommands,
}

#[derive(Subcommand)]
enum CompletionCommands {
    Generate { shell: String },
    Install { shell: Option<String> },
    Uninstall { shell: Option<String> },
}

#[derive(Args)]
struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommands,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Mark a task as done and record touched files
    Done {
        task_id: usize,
        #[arg(long)]
        change: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct InProgressArgs {
    #[command(subcommand)]
    command: InProgressCommands,
}

#[derive(Subcommand)]
enum InProgressCommands {
    /// Mark a change as in-progress
    Add { name: String },
}

#[derive(Args)]
struct DiscussArgs {
    #[command(subcommand)]
    command: DiscussCommands,
}

#[derive(Subcommand)]
enum DiscussCommands {
    /// Create a new discussion document
    New { topic: String, #[arg(long)] json: bool },
    /// List discussions
    List { #[arg(long)] json: bool },
    /// Show a discussion document
    Show { slug: String, #[arg(long)] json: bool },
    /// Append a round to a discussion (content from stdin)
    #[command(name = "add-round")]
    AddRound { slug: String, #[arg(long, default_value = "interview")] mode: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Conclude a discussion (content from stdin)
    Conclude { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read_stdin() -> String {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    buf
}

fn print_json<T: serde::Serialize>(v: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn require_paths() -> Result<core::paths::Paths> {
    core::paths::Paths::discover_cwd()
        .ok_or_else(|| anyhow::anyhow!("no speclink project found (run `speclink init` first)"))
}

include!("commands.rs");

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use speclink_core as core;

mod color;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

/// Version string with the architecture suffix Spectra appends (e.g. "2.3.1 (x64)").
const VERSION: &str = {
    #[cfg(target_arch = "x86_64")]
    {
        concat!(env!("CARGO_PKG_VERSION"), " (x64)")
    }
    #[cfg(target_arch = "aarch64")]
    {
        concat!(env!("CARGO_PKG_VERSION"), " (arm64)")
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        env!("CARGO_PKG_VERSION")
    }
};

/// Speclink — spec management CLI
#[derive(Parser)]
#[command(name = "speclink", version = VERSION, about = "Speclink — spec management CLI", disable_help_subcommand = false)]
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
    path: Option<String>,
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
    /// Project path (defaults to current directory)
    path: Option<PathBuf>,
    /// Overwrite existing files
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct ListArgs {
    /// Show only specs
    #[arg(long)]
    specs: bool,
    /// Show only changes
    #[arg(long)]
    changes: bool,
    /// Sort by: name, modified, created
    #[arg(long, default_value = "modified")]
    sort: String,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ShowArgs {
    /// Item to show (change or spec name)
    item: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
    /// Item type: change, spec
    #[arg(long = "item-type", value_name = "type")]
    item_type: Option<String>,
    /// Show only delta specs
    #[arg(long = "deltas-only")]
    deltas_only: bool,
    /// Show requirements
    #[arg(short = 'r', long)]
    requirements: bool,
}

#[derive(Args)]
struct ValidateArgs {
    /// Item to validate
    item: Option<String>,
    /// Validate all items
    #[arg(long)]
    all: bool,
    /// Validate only changes
    #[arg(long)]
    changes: bool,
    /// Validate only specs
    #[arg(long)]
    specs: bool,
    /// Strict mode
    #[arg(long)]
    strict: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ChangeArg {
    /// Change name (auto-detects if only one exists)
    change: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ArchiveArgs {
    /// Changes to archive (several allowed; auto-detects when omitted and only one exists)
    #[arg(value_name = "CHANGE")]
    changes: Vec<String>,
    /// Archive every ready change (tasks complete, valid, no stale delta assumptions)
    #[arg(long)]
    all: bool,
    /// Skip confirmation
    #[arg(short = 'y', long)]
    yes: bool,
    /// Skip spec updates
    #[arg(long = "skip-specs")]
    skip_specs: bool,
    /// Skip validation before archiving
    #[arg(long = "no-validate")]
    no_validate: bool,
    /// Mark all incomplete tasks as complete before archiving
    #[arg(long = "mark-tasks-complete")]
    mark_tasks_complete: bool,
}

#[derive(Args)]
struct StatusArgs {
    /// Change name
    #[arg(long)]
    change: Option<String>,
    /// Schema name
    #[arg(long)]
    schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct InstructionsArgs {
    /// Artifact ID or "apply"
    artifact: Option<String>,
    /// Change name
    #[arg(long)]
    change: Option<String>,
    /// Schema name
    #[arg(long)]
    schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
    /// Embedded skill name (outputs skill body directly)
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

#[derive(Args)]
struct JsonFlag {
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct TemplatesArgs {
    /// Schema name
    #[arg(long)]
    schema: Option<String>,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct FeedbackArgs {
    /// Feedback message
    message: String,
    /// Detailed body
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
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommands,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show config file path
    Path,
    /// List all settings
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Get a config value
    Get {
        /// Config key
        key: String,
    },
    /// Set a config value
    Set {
        /// Config key
        key: String,
        /// Config value
        value: String,
        /// Treat value as string
        #[arg(long)]
        string: bool,
        /// Allow unknown keys
        #[arg(long = "allow-unknown")]
        allow_unknown: bool,
    },
    /// Remove a config key
    Unset {
        /// Config key
        key: String,
    },
    /// Reset config
    Reset {
        /// Reset all settings
        #[arg(long)]
        all: bool,
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Edit config in $EDITOR
    Edit,
}

#[derive(Args)]
struct CompletionArgs {
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

#[derive(Args)]
struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommands,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Mark a task as done and record touched files
    Done {
        /// Task ID (1-based sequential index)
        task_id: String,
        /// Change name
        #[arg(long)]
        change: Option<String>,
        /// Output as JSON
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
    Add {
        /// Change name
        name: String,
    },
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
    List {
        /// Show archived discussions instead of live ones
        #[arg(long)]
        archived: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a discussion document
    Show { slug: String, #[arg(long)] json: bool },
    /// Set the discussion's Context section (content from stdin)
    Context { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Append a round to a discussion (content from stdin)
    #[command(name = "add-round")]
    AddRound { slug: String, #[arg(long, default_value = "interview")] mode: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Conclude a discussion (content from stdin)
    Conclude { slug: String, #[arg(long)] stdin: bool, #[arg(long)] json: bool },
    /// Archive a discussion (move to discussions/archive/<created>-<slug>.md)
    Archive { slug: String, #[arg(long)] json: bool },
    /// Discard a live discussion (delete the file; --force required once rounds exist)
    Discard {
        slug: String,
        /// Delete even when the discussion has recorded rounds
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Promote a discussion into a change scaffold (proposal prefilled from the conclusion)
    Promote {
        slug: String,
        /// Change name (defaults to the discussion slug)
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    color::init(cli.no_color);
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

/// Discover the host workspace, or fail with the standard not-initialized error.
fn require_workspace() -> Result<core::workspace::Workspace> {
    core::workspace::Workspace::discover_cwd()
        .ok_or_else(|| anyhow::anyhow!("Not initialized. Run 'speclink init' to initialize."))
}

/// The CLI assembly point: discover the workspace and build the filesystem
/// storage adapter for it. Core flows receive the store as `&dyn Store`.
fn open_project() -> Result<(core::workspace::Workspace, speclink_fs::FsStore)> {
    let ws = require_workspace()?;
    let store = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
    Ok((ws, store))
}

include!("commands.rs");

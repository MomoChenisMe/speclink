use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use speclink_core as core;

mod color;
mod common;
mod remote_base;
mod verbs;
use common::warn_leftover_remote_file;
use remote_base::{remote_ctx, RemoteCtx};
use std::process::ExitCode;
use verbs::checks::{
    cmd_analyze, cmd_drift, cmd_validate, remote_analyze, remote_drift, remote_validate, ChangeArg,
    ValidateArgs,
};
use verbs::config::{cmd_config, cmd_workflow_config, ConfigArgs, WorkflowConfigArgs};
use verbs::connection::{cmd_auth, cmd_link, cmd_unlink, AuthArgs, LinkArgs};
use verbs::discuss::{cmd_discuss, remote_discuss, DiscussArgs};
use verbs::documents::{
    cmd_artifact, cmd_language, remote_artifact, remote_language, ArtifactArgs, LanguageArgs,
};
use verbs::init::{cmd_init, cmd_update, InitArgs, UpdateArgs};
use verbs::instructions::{
    cmd_instructions, cmd_instructions_skill, remote_instructions, InstructionsArgs,
};
use verbs::lifecycle::{
    cmd_archive, cmd_discard, remote_archive, remote_claim, remote_discard, ArchiveArgs, ClaimArgs,
    DiscardArgs,
};
use verbs::new::{cmd_new, remote_new, NewArgs};
use verbs::progress::{
    cmd_in_progress, cmd_task, remote_in_progress, remote_task, InProgressArgs, TaskArgs,
};
use verbs::query::{
    cmd_list, cmd_show, cmd_status, remote_list, remote_show, remote_status, ListArgs, ShowArgs,
    StatusArgs,
};
use verbs::station::{cmd_review, cmd_verify, ReviewArgs, VerifyArgs};
use verbs::toolchain::{
    cmd_completion, cmd_demo, cmd_feedback, cmd_schema, cmd_schemas, cmd_templates, CompletionArgs,
    FeedbackArgs, JsonFlag, SchemaArgs, TemplatesArgs,
};

/// The frozen architecture suffix, absent on architectures we do not ship.
const ARCH: Option<&str> = {
    #[cfg(target_arch = "x86_64")]
    {
        Some("x64")
    }
    #[cfg(target_arch = "aarch64")]
    {
        Some("arm64")
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        None
    }
};

/// Version string with the architecture suffix and the engine (artifact layer)
/// version, e.g. "2.3.1 (arm64, engine v1.14.0)". Built at runtime because
/// `MARKER_VERSION` lives in another crate and cannot be `concat!`ed here — it
/// is what makes "which engine is this binary" a one-command question.
static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let pkg = env!("CARGO_PKG_VERSION");
    let engine = core::init::MARKER_VERSION;
    match ARCH {
        Some(arch) => format!("{pkg} ({arch}, engine {engine})"),
        None => format!("{pkg} (engine {engine})"),
    }
});

/// Speclink — spec management CLI
#[derive(Parser)]
#[command(name = "speclink", version = VERSION.as_str(), about = "Speclink — spec management CLI", disable_help_subcommand = false)]
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
    /// Discard a change (delete it; --force required once work has started)
    Discard(DiscardArgs),
    /// Claim a change for implementation (remote mode)
    Claim(ClaimArgs),
    /// Connect this repo to a remote spec store
    Link(LinkArgs),
    /// Remove the remote store connection
    Unlink,
    /// Authentication against the remote store
    Auth(AuthArgs),
    /// Read store documents (artifact contents)
    Artifact(ArtifactArgs),
    /// Shared vocabulary document
    Language(LanguageArgs),
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
    /// Workflow config commands (openspec/config.yaml)
    #[command(name = "workflow-config")]
    WorkflowConfig(WorkflowConfigArgs),
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
    /// Review quality station (ticket rounds, stamping)
    Review(ReviewArgs),
    /// Verify quality station (ticket rounds, stamping)
    Verify(VerifyArgs),
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

fn dispatch(cli: Cli) -> Result<()> {
    warn_leftover_remote_file();
    // 31 個頂層動詞的模式形狀宣告（design D5 分類表）：ModeFree 直呼、Dual
    // 兩臂必填、FsOnly／RemoteOnly 明寫拒絕。分岔決策只活在這一層。
    match cli.command {
        // --- ModeFree：dispatch 不做模式判定；link／unlink／auth 是連線管理，
        // 不消費模式而是改模式，連線解析自理 ---
        Commands::Init(a) => cmd_init(a),
        Commands::Update(a) => cmd_update(a),
        Commands::Link(a) => cmd_link(a),
        Commands::Unlink => cmd_unlink(),
        Commands::Auth(a) => cmd_auth(a),
        Commands::Schemas(a) => cmd_schemas(a),
        Commands::Templates(a) => cmd_templates(a),
        Commands::Feedback(a) => cmd_feedback(a),
        Commands::Schema(a) => cmd_schema(a),
        Commands::Config(a) => cmd_config(a),
        Commands::Completion(a) => cmd_completion(a),
        // --- Dual：本機臂與 remote 臂皆為必填參數 ---
        Commands::List(a) => dual(a, cmd_list, |ctx, a| remote_list(ctx, &a)),
        Commands::Show(a) => dual(a, cmd_show, remote_show),
        Commands::Validate(a) => dual(a, cmd_validate, |ctx, a| remote_validate(ctx, &a)),
        Commands::Analyze(a) => dual(a, cmd_analyze, |ctx, a| remote_analyze(ctx, &a)),
        Commands::Drift(a) => dual(a, cmd_drift, |ctx, a| remote_drift(ctx, &a)),
        Commands::Archive(a) => dual(a, cmd_archive, |ctx, a| remote_archive(ctx, &a)),
        Commands::Discard(a) => dual(a, cmd_discard, |ctx, a| remote_discard(ctx, &a)),
        Commands::Artifact(a) => dual(a, cmd_artifact, remote_artifact),
        Commands::Language(a) => dual(a, cmd_language, remote_language),
        Commands::Status(a) => dual(a, cmd_status, |ctx, a| remote_status(ctx, &a)),
        Commands::Instructions(a) => {
            // `--skill` 印常數技能文本、不消費 store——檢查先於模式解析（凍結行為）。
            if let Some(skill) = a.skill.clone() {
                cmd_instructions_skill(&skill)
            } else {
                dual(a, cmd_instructions, |ctx, a| remote_instructions(ctx, &a))
            }
        }
        Commands::New(a) => dual(a, cmd_new, remote_new),
        Commands::WorkflowConfig(a) => cmd_workflow_config(a), // Dual（宣告於 cmd_workflow_config）
        Commands::Task(a) => dual(a, cmd_task, remote_task),
        Commands::InProgress(a) => dual(a, cmd_in_progress, remote_in_progress),
        Commands::Discuss(a) => dual(a, cmd_discuss, remote_discuss),
        // review／verify 為 Dual 家族：clap → StationVerb 正規化先行，雙臂
        // 宣告在家族函式尾端（station_dual；review 的 prepare 自成雙臂）。
        Commands::Review(a) => cmd_review(a), // Dual（宣告於 station_dual）
        Commands::Verify(a) => cmd_verify(a), // Dual（宣告於 station_dual）
        // --- FsOnly：只解析模式、不握手，remote 明寫拒絕 ---
        Commands::Demo => fs_only(DEMO_REMOTE_REFUSAL, cmd_demo),
        // --- RemoteOnly：fs 明寫拒絕 ---
        Commands::Claim(a) => remote_only(a, CLAIM_FS_REFUSAL, |ctx, a| remote_claim(ctx, &a.name)),
    }
}

// --- 模式形狀組合子（dispatch 宣告層）---
//
// 每個頂層動詞在 dispatch 表態四種形狀之一（design D1/D2）：ModeFree 直呼
// （模式解析不觸發）、Dual 兩臂皆為必填參數（缺一臂是編譯錯誤，不是執行期
// 靜默回退）、FsOnly 只解析模式不握手、RemoteOnly fs 即拒。模式判定惰性
// 執行：解析與握手都由形狀觸發，remote_ctx() 只從這一層呼叫。

/// Dual：模式解析一次——remote 模式握手後派 remote 臂，fs 模式派本機臂。
fn dual<A>(
    a: A,
    fs_arm: impl FnOnce(A) -> Result<()>,
    remote_arm: impl FnOnce(&RemoteCtx, A) -> Result<()>,
) -> Result<()> {
    match remote_ctx()? {
        Some(ctx) => remote_arm(&ctx, a),
        None => fs_arm(a),
    }
}

/// FsOnly：只解析 store 模式、不建立連線——remote 即拒絕（離線同拒、
/// server 零請求），fs 派本機臂。
fn fs_only(refusal: &'static str, fs_arm: impl FnOnce() -> Result<()>) -> Result<()> {
    if let Some(ws) = core::workspace::Workspace::discover_cwd()? {
        if matches!(
            speclink_host::context::resolve_store_mode(&ws)?.mode,
            core::workspace::StoreMode::Remote(_)
        ) {
            bail!("{refusal}");
        }
    }
    fs_arm()
}

/// RemoteOnly：fs 模式即拒絕、不觸 Store（在非專案目錄也同一句）；remote
/// 模式握手後派 remote 臂。
fn remote_only<A>(
    a: A,
    refusal: &'static str,
    remote_arm: impl FnOnce(&RemoteCtx, A) -> Result<()>,
) -> Result<()> {
    match remote_ctx()? {
        Some(ctx) => remote_arm(&ctx, a),
        None => bail!("{refusal}"),
    }
}

// --- claim ---

// claim 是 remote 生命週期的所有權概念，本機 fs store 沒有 claim 狀態——
// fs 模式 fail-loud。訊息與 runtime 的 Claim 分支共用同一份 frozen 文字
// （node dispatch 經該分支）。
const CLAIM_FS_REFUSAL: &str =
    "claim requires a remote store — this project uses the local fs store";

// 本質本機動詞：remote 模式明確拒絕（比照 claim 在 fs 的 fail-loud），
// 由 dispatch 的 fs_only 形狀執行——只判斷連線設定、不走 handshake。
const DEMO_REMOTE_REFUSAL: &str =
    "demo is not available in remote mode — it seeds a demo change into a local openspec/ tree";

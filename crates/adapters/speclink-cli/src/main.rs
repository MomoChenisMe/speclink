use anyhow::{bail, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use speclink_core as core;

mod color;
mod common;
mod remote_base;
mod verbs;
use common::{warn_leftover_remote_file, warn_spec_layout};
use remote_base::{remote_ctx, RemoteCtx};
use std::process::ExitCode;
use speclink_verbs::{GlobalArgs, SpecVerb};
use verbs::checks::{
    cmd_analyze, cmd_drift, cmd_validate, remote_analyze, remote_drift, remote_validate,
};
use verbs::config::{cmd_config, cmd_workflow_config, ConfigArgs};
use verbs::connection::{cmd_auth, cmd_link, cmd_unlink, AuthArgs, LinkArgs};
use verbs::discuss::{cmd_discuss, remote_discuss};
use verbs::documents::{cmd_document, remote_artifact, remote_language};
use verbs::init::{cmd_init, cmd_update, InitArgs, UpdateArgs};
use verbs::instructions::{cmd_instructions, cmd_instructions_skill, remote_instructions};
use verbs::lifecycle::{cmd_archive, cmd_discard, remote_archive, remote_claim, remote_discard};
use verbs::new::{cmd_new, remote_new};
use verbs::plan::{cmd_change, cmd_plan, remote_change, remote_plan};
use verbs::progress::{cmd_in_progress, cmd_task, remote_in_progress, remote_task};
use verbs::query::{cmd_list, cmd_show, cmd_status, remote_list, remote_show, remote_status};
use verbs::station::{cmd_review, cmd_verify};
use verbs::toolchain::{
    cmd_completion, cmd_demo, cmd_feedback, cmd_schema, cmd_schemas, cmd_templates, CompletionArgs,
    FeedbackArgs, JsonFlag, SchemaArgs, TemplatesArgs,
};
use verbs::trace::cmd_trace;

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
/// `ASSET_VERSION` lives in another crate and cannot be `concat!`ed here — it
/// is what makes "which engine is this binary" a one-command question.
static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    let pkg = env!("CARGO_PKG_VERSION");
    let engine = core::init::ASSET_VERSION;
    match ARCH {
        Some(arch) => format!("{pkg} ({arch}, engine {engine})"),
        None => format!("{pkg} (engine {engine})"),
    }
});

/// Speclink — spec management CLI
#[derive(Parser)]
#[command(name = "speclink", version = VERSION.as_str(), about = "Speclink — spec management CLI", disable_help_subcommand = false)]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,
    #[command(subcommand)]
    command: Commands,
}

/// The CLI's top-level commands: the shared spec verbs (`SpecVerb`, defined
/// in speclink-verbs) plus the CLI's own families. `HELP_ORDER` sets their
/// places in `--help`, because the flattened spec verbs sit between the
/// CLI's own commands there.
#[derive(Subcommand)]
enum Commands {
    /// Initialize Speclink in a project
    Init(InitArgs),
    /// Update instruction files
    Update(UpdateArgs),
    #[command(flatten)]
    Spec(SpecVerb),
    /// Connect this repo to a remote spec store
    Link(LinkArgs),
    /// Remove the remote store connection
    Unlink,
    /// Authentication against the remote store
    Auth(AuthArgs),
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
    /// Generate a demo change with sample data
    Demo,
}

/// Every top-level command in its `--help` order.
const HELP_ORDER: [&str; 34] = [
    "init", "update", "list", "show", "validate", "analyze", "drift", "trace", "plan", "change",
    "archive", "discard", "claim", "link", "unlink", "auth", "artifact", "language", "status",
    "instructions", "new", "schemas", "templates", "feedback", "schema", "config",
    "workflow-config", "completion", "task", "in-progress", "demo", "discuss", "review", "verify",
];

/// The CLI's command tree with each top-level command at its `HELP_ORDER`
/// place. `mut_subcommands` keeps the subcommand order.
pub(crate) fn cli_command() -> clap::Command {
    Cli::command().mut_subcommands(|c| match HELP_ORDER.iter().position(|n| *n == c.get_name()) {
        Some(place) => c.display_order(place),
        None => c,
    })
}

fn main() -> ExitCode {
    // What `Cli::parse()` does, on the ordered tree: clap prints and exits
    // on parse errors and help.
    let mut cmd = cli_command();
    let matches = cmd.get_matches_mut();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.format(&mut cmd).exit());
    color::init(cli.global.no_color);
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
    if !matches!(cli.command, Commands::Init(_) | Commands::Update(_)) {
        warn_spec_layout();
    }
    // 全部頂層動詞的模式形狀宣告（design D5 分類表）：ModeFree 直呼、Dual
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
        Commands::Spec(verb) => spec_dispatch(verb),
        // --- FsOnly：只解析模式、不握手，remote 明寫拒絕 ---
        Commands::Demo => fs_only(DEMO_REMOTE_REFUSAL, cmd_demo),
    }
}

/// The shared spec verbs' mode shapes — the same declaration layer as
/// `dispatch`, one arm per top-level verb.
fn spec_dispatch(verb: SpecVerb) -> Result<()> {
    match verb {
        // --- Dual ---
        SpecVerb::List(a) => dual(a, cmd_list, |ctx, a| remote_list(ctx, &a)),
        SpecVerb::Show(a) => dual(a, cmd_show, remote_show),
        SpecVerb::Validate(a) => dual(a, cmd_validate, |ctx, a| remote_validate(ctx, &a)),
        SpecVerb::Analyze(a) => dual(a, cmd_analyze, |ctx, a| remote_analyze(ctx, &a)),
        SpecVerb::Drift(a) => dual(a, cmd_drift, |ctx, a| remote_drift(ctx, &a)),
        SpecVerb::Artifact(a) => dual(a, cmd_document, remote_artifact),
        SpecVerb::Language(a) => dual(a, cmd_document, remote_language),
        SpecVerb::Status(a) => dual(a, cmd_status, |ctx, a| remote_status(ctx, &a)),
        SpecVerb::Archive(a) => dual(a, cmd_archive, |ctx, a| remote_archive(ctx, &a)),
        SpecVerb::Discard(a) => dual(a, cmd_discard, |ctx, a| remote_discard(ctx, &a)),
        SpecVerb::New(a) => dual(a, cmd_new, remote_new),
        SpecVerb::Task(a) => dual(a, cmd_task, remote_task),
        SpecVerb::InProgress(a) => dual(a, cmd_in_progress, remote_in_progress),
        // `plan --strict-overlap` 同理只限本機：server 沒有目錄級開關（verb-contract
        // 明文分歧第 6 項）——只解析模式、不握手即拒。
        SpecVerb::Plan(a) if a.strict_overlap => {
            fs_only(PLAN_STRICT_REMOTE_REFUSAL, || cmd_plan(a))
        }
        SpecVerb::Plan(a) => dual(a, cmd_plan, |ctx, a| remote_plan(ctx, &a)),
        // `change rank` 在 Dual 家族裡是 FsOnly：它寫卡片 meta 的 board_rank，remote
        // 的順序真相在 board resource——只解析模式、不握手即拒。
        SpecVerb::Change(a) if a.is_rank() => {
            fs_only(CHANGE_RANK_REMOTE_REFUSAL, || cmd_change(a))
        }
        SpecVerb::Change(a) => dual(a, cmd_change, remote_change),
        SpecVerb::Discuss(a) => dual(a, cmd_discuss, remote_discuss),
        // review／verify 為 Dual 家族：clap → StationVerb 正規化先行，雙臂
        // 宣告在家族函式尾端（station_dual；review 的 prepare 自成雙臂）。
        SpecVerb::Review(a) => cmd_review(a), // Dual（宣告於 station_dual）
        SpecVerb::Verify(a) => cmd_verify(a), // Dual（宣告於 station_dual）
        SpecVerb::WorkflowConfig(a) => cmd_workflow_config(a), // Dual（宣告於 cmd_workflow_config）
        SpecVerb::Instructions(a) => {
            // `--skill` 印常數技能文本、不消費 store——檢查先於模式解析（凍結行為）。
            if let Some(skill) = a.skill.clone() {
                cmd_instructions_skill(&skill)
            } else {
                dual(a, cmd_instructions, |ctx, a| remote_instructions(ctx, &a))
            }
        }
        // --- FsOnly ---
        SpecVerb::Trace(a) => fs_only(TRACE_REMOTE_REFUSAL, || cmd_trace(a)),
        // --- RemoteOnly：fs 明寫拒絕 ---
        SpecVerb::Claim(a) => remote_only(a, CLAIM_FS_REFUSAL, |ctx, a| remote_claim(ctx, &a.name)),
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
    "demo is not available in remote mode — it seeds a demo change into the local spec directory";

// trace 讀本機封存目錄、討論檔與 evidence 組裝溯源鏈；remote 端點屬本
// change 的 Non-Goal（v1 僅本地 CLI），比照 demo 的 FsOnly 形狀明寫拒絕。
const TRACE_REMOTE_REFUSAL: &str =
    "trace is not available in remote mode — it assembles the provenance chain from the local spec directory";

// Dual 家族裡只限本機的兩個子情形（理由見 dispatch 的對應分支）。
const CHANGE_RANK_REMOTE_REFUSAL: &str =
    "change rank is not available in remote mode: the board order lives in the board resource";
const PLAN_STRICT_REMOTE_REFUSAL: &str =
    "plan --strict-overlap is not available in remote mode: the server plans with requirement-level overlap only";

#[cfg(test)]
mod tests {
    // `HELP_ORDER` is where every top-level command sits in `--help`; a
    // command missing from it would drop to the end of the list.
    #[test]
    fn help_order_names_every_top_level_command_once() {
        let cmd = <super::Cli as clap::CommandFactory>::command();
        let mut names: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
        let mut order = super::HELP_ORDER.to_vec();
        names.sort_unstable();
        order.sort_unstable();
        assert_eq!(names, order);
    }
}

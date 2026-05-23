//! SpecLink CLI 入口。
//!
//! 解析 clap subcommand → 呼叫 runtime → 用 JSON envelope 輸出 + 對應 exit code。

#![allow(clippy::doc_markdown)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use speclink_cli::commands;
use speclink_cli::output::{Envelope, Warning, error, success};
use speclink_runtime::RuntimeError;
use speclink_runtime::tool_ops::{DescribeFormat, DescribePhase};

#[derive(Parser, Debug)]
#[command(
    name = "speclink",
    version,
    about = "SpecLink — Spec-Driven Development workflow engine"
)]
struct Cli {
    /// 以 stable JSON envelope 輸出（給 AI / CI / tooling 使用）。
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 在當前目錄初始化 SpecLink project（artifact root + state root）。
    Init {
        #[arg(long)]
        force: bool,
    },
    /// 顯示當前 project 狀態。
    Status,
    /// 把當前目錄綁定到既存 project_id。
    Link { project_id: String },
    /// 移除當前目錄與 SpecLink project 的綁定（不刪 state.db、不刪 schemas）。
    Unlink,

    /// 建立新 change 或 artifact。
    New {
        #[command(subcommand)]
        sub: NewSub,
    },
    /// 顯示 change metadata 與 artifact 清單。
    Show {
        #[command(subcommand)]
        sub: ShowSub,
    },
    /// destructive：刪除 change row + filesystem 目錄。
    Delete {
        #[command(subcommand)]
        sub: DeleteSub,
    },
    /// 列舉資源。
    List(ListArgs),
    /// Artifact 動詞（目前僅 `read`）。
    Artifact {
        #[command(subcommand)]
        sub: ArtifactSub,
    },

    /// Apply lifecycle 動詞：`start` / `pause`。
    Apply {
        #[command(subcommand)]
        sub: ApplySub,
    },

    /// Task workflow 動詞：`list` / `done` / `undo`。
    Task {
        #[command(subcommand)]
        sub: TaskSub,
    },

    /// Config 動詞：`show` / `set` / `edit`。
    Config {
        #[command(subcommand)]
        sub: ConfigSub,
    },

    /// Archive change：spec delta merge + state→archived + change dir 搬入 `.speclink/changes/archive/`。
    Archive {
        change_id: String,
        /// 跳過 spec delta merge、僅 transition state + 搬目錄（emergency 用）。
        #[arg(long)]
        skip_specs: bool,
        /// reserved for future analyze slice; currently no-op
        #[arg(long)]
        no_validate: bool,
        /// accepted for compatibility; archive does not prompt in this slice
        #[arg(long)]
        yes: bool,
    },

    /// 取得 artifact / workflow phase 的 AI instruction body + template + dependency。
    Instructions {
        /// Artifact kind 或 workflow phase kind（`proposal` / `spec` / `design` /
        /// `tasks` / `apply` / `ingest` / `archive` / `commit`；其他值回
        /// `instructions.unknown_kind`、exit 2）。
        kind: String,
        /// 套用 change context（existence check + schema_id resolution）。
        #[arg(long)]
        change: Option<String>,
        /// (reserved for Phase 2 `add-discuss-ops`, currently ignored)
        #[arg(long)]
        role: Option<String>,
        /// (reserved for Phase 2 `add-discuss-ops`, currently ignored)
        #[arg(long)]
        discussion: Option<String>,
    },

    /// Catalogue dump：把 37 個 operation 印成 json / text / copilot-sdk 三種 format。
    #[command(name = "describe-tools")]
    DescribeTools {
        /// Output format. MVP 支援 `json` / `text` / `copilot-sdk`；其餘 5 個 enum 值屬 [deferred]，runtime 回 `tool.format_not_supported`。
        #[arg(long, value_enum, default_value_t = CliFormat::Json)]
        format: CliFormat,
        /// 只輸出指定 operation id（comma-separated 或重複 `--filter`）。
        #[arg(long, value_delimiter = ',')]
        filter: Vec<String>,
        /// 只輸出指定 category（comma-separated 或重複 `--categories`）。
        #[arg(long, value_delimiter = ',')]
        categories: Vec<String>,
        /// 只輸出涉及指定 skill phase 的 op（comma-separated 或重複 `--phases`）。
        #[arg(long, value_enum, value_delimiter = ',')]
        phases: Vec<CliPhase>,
        /// 切換到 37 ops 全集；省略時只回 12 個 curated subset。
        #[arg(long)]
        full: bool,
    },
}

/// clap-facing format enum；與 `DescribeFormat` 對映。
#[derive(Copy, Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliFormat {
    Json,
    Text,
    CopilotSdk,
    Copilotkit,
    Openai,
    Langchain,
    Mcp,
    Claude,
}

impl From<CliFormat> for DescribeFormat {
    fn from(v: CliFormat) -> Self {
        match v {
            CliFormat::Json => DescribeFormat::Json,
            CliFormat::Text => DescribeFormat::Text,
            CliFormat::CopilotSdk => DescribeFormat::CopilotSdk,
            CliFormat::Copilotkit => DescribeFormat::Copilotkit,
            CliFormat::Openai => DescribeFormat::Openai,
            CliFormat::Langchain => DescribeFormat::Langchain,
            CliFormat::Mcp => DescribeFormat::Mcp,
            CliFormat::Claude => DescribeFormat::Claude,
        }
    }
}

/// clap-facing phase enum；與 `DescribePhase` 對映。
#[derive(Copy, Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum CliPhase {
    Discuss,
    Propose,
    Apply,
    Archive,
    Ingest,
}

impl From<CliPhase> for DescribePhase {
    fn from(v: CliPhase) -> Self {
        match v {
            CliPhase::Discuss => DescribePhase::Discuss,
            CliPhase::Propose => DescribePhase::Propose,
            CliPhase::Apply => DescribePhase::Apply,
            CliPhase::Archive => DescribePhase::Archive,
            CliPhase::Ingest => DescribePhase::Ingest,
        }
    }
}

#[derive(Subcommand, Debug)]
enum ApplySub {
    /// 把 change 推進到 `in_progress` 並 assign actor（或 reassign）。
    Start {
        change: String,
        /// AI agent host 識別碼；省略則用 `SPECLINK_AGENT_HOST` env 或 fallback `cli`。
        #[arg(long)]
        actor: Option<String>,
    },
    /// 把 change 從 `in_progress` 退回 `ready` 並清空 actor。
    Pause { change: String },
}

#[derive(Subcommand, Debug)]
enum TaskSub {
    /// 列舉 tasks.md 內所有 checkbox 行。
    List {
        #[arg(long)]
        change: String,
    },
    /// 把 1-based index 對應的 task 標記為 done。task indices are derived from current document order — editing tasks.md between `task list` and `task done` SHALL invalidate previously-seen indices.
    Done {
        index: usize,
        #[arg(long)]
        change: String,
    },
    /// 把 1-based index 對應的 task 標記回 todo。task indices are derived from current document order — editing tasks.md between `task list` and `task undo` SHALL invalidate previously-seen indices.
    Undo {
        index: usize,
        #[arg(long)]
        change: String,
    },
}

#[derive(Subcommand, Debug)]
enum NewSub {
    /// 建立新 change。
    Change { name: String },
    /// 寫入 artifact；bytes 從 stdin 讀取。
    Artifact {
        /// `proposal` / `design` / `tasks` / `spec`
        kind: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        capability: Option<String>,
        #[arg(long = "expected-etag")]
        expected_etag: Option<String>,
        /// 從 stdin 讀 bytes 寫入檔案。
        #[arg(long)]
        stdin: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ShowSub {
    /// 顯示 change metadata。
    Change { name: String },
}

#[derive(Subcommand, Debug)]
enum DeleteSub {
    /// 刪除 change（destructive）。
    Change {
        name: String,
        #[arg(long = "confirm-name")]
        confirm_name: Option<String>,
    },
}

#[derive(Args, Debug)]
struct ListArgs {
    /// 列舉所有 change。
    #[arg(long)]
    changes: bool,
    /// 列舉某 change 下的 spec capability id。
    #[arg(long)]
    specs: bool,
    /// `--specs` 用的 change 名稱。
    #[arg(long)]
    change: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ConfigSub {
    /// 讀取 `.speclink/config.yaml`、回傳整份 Config 或 `--key` 指定的 leaf。
    Show {
        /// JSONPath subset address；省略時回整份。
        #[arg(long)]
        key: Option<String>,
    },
    /// 對 `<key>` 套 `<value>` patch（單 key 修改）。
    Set {
        /// JSONPath subset address（如 `rules.require_code_review`）。
        key: String,
        /// 待寫入的 value 字串；解析順序為 `true/false/null → int → float → string`。
        value: String,
        /// 帶 expected etag 走 user CAS；省略走 internal CAS。
        #[arg(long = "expected-etag")]
        expected_etag: Option<String>,
    },
    /// 用 stdin 或 `$EDITOR` 整檔覆寫 config.yaml。
    Edit {
        /// 從 stdin 讀完整 YAML content。
        #[arg(long)]
        stdin: bool,
        /// 指定 editor command（覆蓋 `$EDITOR`）。
        #[arg(long)]
        editor: Option<String>,
        /// 帶 expected etag 走 user CAS；省略走 internal CAS。
        #[arg(long = "expected-etag")]
        expected_etag: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ArtifactSub {
    /// 讀 artifact。
    Read {
        kind: String,
        #[arg(long)]
        change: String,
        #[arg(long)]
        capability: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let use_json = cli.json;

    let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("failed to start async runtime: {e}");
            return ExitCode::from(1);
        }
    };

    let result: Result<(serde_json::Value, Vec<Warning>), RuntimeError> = rt.block_on(async move {
        match cli.command {
            Commands::Init { force } => commands::init::run(&working_dir, force).await,
            Commands::Status => commands::status::run(&working_dir).await,
            Commands::Link { project_id } => commands::link::run(&working_dir, &project_id).await,
            Commands::Unlink => commands::unlink::run(&working_dir).await,
            Commands::New { sub } => match sub {
                NewSub::Change { name } => commands::new_change::run(&working_dir, &name).await,
                NewSub::Artifact {
                    kind,
                    change,
                    capability,
                    expected_etag,
                    stdin,
                } => {
                    commands::new_artifact::run(
                        &working_dir,
                        &kind,
                        &change,
                        capability.as_deref(),
                        expected_etag.as_deref(),
                        stdin,
                    )
                    .await
                }
            },
            Commands::Show { sub } => match sub {
                ShowSub::Change { name } => commands::show_change::run(&working_dir, &name).await,
            },
            Commands::Delete { sub } => match sub {
                DeleteSub::Change { name, confirm_name } => {
                    commands::delete_change::run(&working_dir, &name, confirm_name.as_deref()).await
                }
            },
            Commands::List(args) => dispatch_list(&working_dir, args).await,
            Commands::Artifact { sub } => match sub {
                ArtifactSub::Read {
                    kind,
                    change,
                    capability,
                } => {
                    commands::artifact_read::run(
                        &working_dir,
                        &kind,
                        &change,
                        capability.as_deref(),
                    )
                    .await
                }
            },
            Commands::Apply { sub } => match sub {
                ApplySub::Start { change, actor } => {
                    commands::apply_start::run(&working_dir, &change, actor.as_deref()).await
                }
                ApplySub::Pause { change } => {
                    commands::apply_pause::run(&working_dir, &change).await
                }
            },
            Commands::Task { sub } => match sub {
                TaskSub::List { change } => commands::task_list::run(&working_dir, &change).await,
                TaskSub::Done { index, change } => {
                    commands::task_done::run(&working_dir, &change, index).await
                }
                TaskSub::Undo { index, change } => {
                    commands::task_undo::run(&working_dir, &change, index).await
                }
            },
            Commands::Config { sub } => match sub {
                ConfigSub::Show { key } => commands::config::run_show(&working_dir, key.as_deref()),
                ConfigSub::Set {
                    key,
                    value,
                    expected_etag,
                } => {
                    commands::config::run_set(&working_dir, &key, &value, expected_etag.as_deref())
                }
                ConfigSub::Edit {
                    stdin,
                    editor,
                    expected_etag,
                } => commands::config::run_edit(
                    &working_dir,
                    stdin,
                    editor.as_deref(),
                    expected_etag.as_deref(),
                ),
            },
            Commands::Archive {
                change_id,
                skip_specs,
                no_validate,
                yes,
            } => {
                commands::archive::run(&working_dir, &change_id, skip_specs, no_validate, yes).await
            }
            Commands::Instructions {
                kind,
                change,
                role,
                discussion,
            } => {
                commands::instructions::run(
                    &working_dir,
                    &kind,
                    change.as_deref(),
                    role.as_deref(),
                    discussion.as_deref(),
                )
                .await
            }
            Commands::DescribeTools {
                format,
                filter,
                categories,
                phases,
                full,
            } => commands::describe_tools::run(
                format.into(),
                filter,
                categories,
                phases.into_iter().map(Into::into).collect(),
                full,
            ),
        }
    });

    match result {
        Ok((data, warnings)) => {
            if use_json {
                let env: Envelope<serde_json::Value> = success(data, warnings);
                println!("{}", serde_json::to_string(&env).unwrap_or_default());
            } else {
                print_human_ok(&data);
            }
            ExitCode::from(0)
        }
        Err(err) => {
            let code = err.code();
            let exit = err.exit_code();
            let hint = hint_for(code);
            if use_json {
                let env = error(code, &err.to_string(), hint, err.retryable());
                println!("{}", serde_json::to_string(&env).unwrap_or_default());
            } else {
                eprintln!("error[{code}]: {err}");
                if let Some(h) = hint {
                    eprintln!("hint: {h}");
                }
            }
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let code_u8: u8 = if (0..=255).contains(&exit) {
                exit as u8
            } else {
                1
            };
            ExitCode::from(code_u8)
        }
    }
}

async fn dispatch_list(
    working_dir: &std::path::Path,
    args: ListArgs,
) -> Result<(serde_json::Value, Vec<Warning>), RuntimeError> {
    match (args.changes, args.specs, args.change) {
        (true, false, _) => commands::list_changes::run(working_dir).await,
        (false, true, Some(change)) => commands::list_specs::run(working_dir, &change).await,
        (false, true, None) => Err(RuntimeError::ChangeInvalidName {
            name: String::new(),
            reason: "`speclink list --specs` requires `--change <name>`".into(),
        }),
        (false, false, _) => Err(RuntimeError::Internal(
            "`speclink list` requires `--changes` or `--specs --change <name>`".into(),
        )),
        (true, true, _) => Err(RuntimeError::Internal(
            "`--changes` and `--specs` are mutually exclusive".into(),
        )),
    }
}

fn print_human_ok(data: &serde_json::Value) {
    println!("{}", speclink_cli::human::render_human(data));
}

fn hint_for(code: &str) -> Option<&'static str> {
    match code {
        "project.requires_git" => Some("Run `git init` first, then re-run `speclink init`."),
        "project.already_initialized" => {
            Some("Pass `--force` to re-initialize while preserving state.db.")
        }
        "project.not_initialized" => Some("Run `speclink init` first."),
        "project.link_target_not_found" => {
            Some("Check available project_ids via `speclink status`.")
        }
        "change.not_found" => Some("Verify the change name via `speclink list --changes`."),
        "change.duplicate_name" => {
            Some("Pick a different name or delete the existing change first.")
        }
        "change.invalid_name" => {
            Some("Change names match `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` with 1-64 byte length.")
        }
        "artifact.kind_invalid" => Some(
            "Use one of `proposal`, `design`, `tasks`, `spec`. Capability ids follow the same kebab-case grammar as change names.",
        ),
        "artifact.capability_required" => {
            Some("`speclink new artifact spec` requires `--capability <id>`.")
        }
        "artifact.not_found" => {
            Some("Confirm the file exists, or omit `--expected-etag` to create a new artifact.")
        }
        "artifact.version_conflict" => Some(
            "Re-read the artifact, supply its current `--expected-etag`, then retry the write.",
        ),
        // slice A3 — state machine + apply / task ops
        "state.invalid_value" => Some(
            "change.state column contains a value outside the legal six-state enum; database corruption suspected",
        ),
        "state.transition_invalid" => {
            Some("transition not permitted from current state; see legal transition table")
        }
        "state.version_conflict" => {
            Some("change row was modified by another agent; reread state and retry")
        }
        "state.db.schema_invalid" => {
            Some("state.db schema version is newer than this binary supports; upgrade binary")
        }
        "change.dag_incomplete" => Some(
            "change is missing required artifacts; write proposal.md, tasks.md, and at least one specs/<capability>/spec.md",
        ),
        "task.no_tasks_file" => Some(
            "tasks.md not found for this change; create it first via `speclink new artifact tasks --change <name>`",
        ),
        "instructions.unknown_kind" => {
            Some("Supported kinds: proposal, spec, design, tasks, apply, ingest, archive, commit")
        }
        "task.index_out_of_range" => Some(
            "task index out of range; re-run `speclink task list --change <name>` to see current indices",
        ),
        // slice A4 — archive
        "change.tasks_incomplete" => {
            Some("complete all tasks first with `speclink task done <i> --change <id>`")
        }
        "validation.archive_failed" => Some("run `speclink validate <id>` first"),
        // polish-config-error-messages — config edit mode required hint.
        "config.edit_mode_required" => Some(
            "pipe YAML via `--stdin`, pass `--editor <cmd>`, or set `$EDITOR` so `speclink config edit` can open an editor",
        ),
        // add-tool-describe-and-catalogue
        "tool.format_not_supported" => Some(
            "MVP supports `json`, `text`, `copilot-sdk`; other formats (copilotkit / openai / langchain / mcp / claude) are deferred to a post-MVP slice",
        ),
        "tool.unknown_op" => {
            Some("verify operation ids via `speclink describe-tools --format text --full`")
        }
        "tool.unknown_category" => Some(
            "valid categories: project, config, schema, discuss, change, artifact, apply, review, archive, spec, meta, doctor, tool",
        ),
        _ => None,
    }
}

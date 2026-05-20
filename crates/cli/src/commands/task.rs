//! `task done` 子命令 — 把 clap 解析後的 `TaskCommand` 串到 runtime + provider，並寫出
//! `Envelope<TaskDoneData>`。

use std::io::Write;
use std::sync::Arc;

use crate::cli::{TaskCommand, TaskDoneArgs};
use crate::exit_code::{ErrorCode, ExitCode, classify};
use crate::output::{Envelope, ErrorBody, TaskDoneData, Warning, request_id, task_update_to_data};
use provider::config::{GlobalConfig, ProjectConfig};
use provider::config_discovery::{find_project_config, global_config_path};
use provider::resolution::{
    ResolutionInputs, ResolvedProvider, Warning as ResolutionWarning, resolve,
};
use provider_local::LocalProvider;
use runtime::task::{MarkTaskDoneInput, mark_task_done};

const ENV_PROVIDER: &str = "SPECLINK_PROVIDER";

/// `speclink task <subcommand>` 入口。
pub async fn run(cmd: TaskCommand) -> ExitCode {
    match cmd {
        TaskCommand::Done(args) => run_done(args).await,
    }
}

async fn run_done(args: TaskDoneArgs) -> ExitCode {
    let json_out = args.flags.json;
    if args.flags.stdin {
        return emit_failure(
            ErrorCode("input.invalid").as_str(),
            "task done does not accept --stdin",
            json_out,
            ExitCode::from(2),
        );
    }

    let project_id = provider::model::ProjectId::from("default");
    let change_id = provider::model::ChangeId::from(args.change.clone());
    let task_id = args.task_id.clone();

    match execute(project_id, change_id, &args.change, task_id).await {
        Ok((data, warnings)) => emit_success(data, warnings, json_out),
        Err(e) => {
            let (code, ec) = classify(&e);
            emit_failure(ec.as_str(), &e.to_string(), json_out, code)
        }
    }
}

async fn execute(
    project_id: provider::model::ProjectId,
    change_id: provider::model::ChangeId,
    change_id_str: &str,
    task_id: String,
) -> Result<(TaskDoneData, Vec<Warning>), anyhow::Error> {
    let cwd = std::env::current_dir()?;
    let project_config: Option<ProjectConfig> = match find_project_config(&cwd) {
        Some(p) => Some(ProjectConfig::load(&p)?),
        None => None,
    };
    let global_config: Option<GlobalConfig> = match global_config_path() {
        Some(p) if p.is_file() => Some(GlobalConfig::load(&p)?),
        _ => None,
    };
    let env_provider = std::env::var(ENV_PROVIDER).ok();
    let inputs = ResolutionInputs {
        flag_provider: None,
        project_config: project_config.as_ref(),
        global_config: global_config.as_ref(),
        env_provider: env_provider.as_deref(),
    };
    let resolved = resolve(inputs)?;

    let provider = match resolved.provider {
        ResolvedProvider::Local { .. } => {
            Arc::new(LocalProvider::new(cwd.clone()).await?) as Arc<dyn provider::Provider>
        }
    };

    let update = mark_task_done(
        provider,
        MarkTaskDoneInput {
            project_id,
            change_id,
            task_id,
        },
    )
    .await?;
    let warnings: Vec<Warning> = resolved
        .warnings
        .into_iter()
        .map(map_resolution_warning)
        .collect();
    Ok((task_update_to_data(change_id_str, update), warnings))
}

fn map_resolution_warning(w: ResolutionWarning) -> Warning {
    Warning {
        code: w.code,
        message: w.message,
    }
}

fn emit_success(data: TaskDoneData, warnings: Vec<Warning>, json: bool) -> ExitCode {
    if !json {
        eprintln!(
            "Task {} in change '{}' marked {} (was {})",
            data.task_id, data.change_id, data.current_status, data.previous_status
        );
        return ExitCode::from(0);
    }
    let env = Envelope::success(data, warnings, request_id());
    write_envelope(&env);
    ExitCode::from(0)
}

fn emit_failure(code: &'static str, message: &str, json: bool, exit: ExitCode) -> ExitCode {
    if !json {
        eprintln!("error [{code}]: {message}");
        return exit;
    }
    let body = ErrorBody {
        code: code.to_string(),
        message: message.to_string(),
        details: serde_json::json!({}),
    };
    let env: Envelope<TaskDoneData> = Envelope::failure(body, request_id());
    write_envelope(&env);
    exit
}

fn write_envelope<T: serde::Serialize>(env: &Envelope<T>) {
    let s = match serde_json::to_string(env) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(s.as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

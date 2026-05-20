//! `status` 子命令 — side-effect-free 讀取 change 的 artifact 狀態。

use std::io::Write;
use std::sync::Arc;

use crate::cli::StatusArgs;
use crate::exit_code::{ErrorCode, ExitCode, classify};
use crate::output::{
    Envelope, ErrorBody, StatusData, Warning, change_status_to_status_data, request_id,
};
use provider::config::{GlobalConfig, ProjectConfig};
use provider::config_discovery::{find_project_config, global_config_path};
use provider::resolution::{
    ResolutionInputs, ResolvedProvider, Warning as ResolutionWarning, resolve,
};
use provider_local::LocalProvider;
use runtime::status::{GetStatusInput, get_status};

const ENV_PROVIDER: &str = "SPECLINK_PROVIDER";

/// `speclink status` 入口；負責 provider resolve + runtime 呼叫 + envelope 輸出。
pub async fn run(args: StatusArgs) -> ExitCode {
    let json_out = args.flags.json;
    if args.flags.stdin {
        return emit_failure(
            ErrorCode("input.invalid").as_str(),
            "status does not accept --stdin",
            json_out,
            ExitCode::from(2),
        );
    }

    let project_id = provider::model::ProjectId::from("default");
    let change_id = provider::model::ChangeId::from(args.change.clone());

    match execute(project_id, change_id).await {
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
) -> Result<(StatusData, Vec<Warning>), anyhow::Error> {
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

    let status = get_status(
        provider,
        GetStatusInput {
            project_id,
            change_id,
        },
    )
    .await?;
    let data = change_status_to_status_data(status);
    let warnings: Vec<Warning> = resolved
        .warnings
        .into_iter()
        .map(map_resolution_warning)
        .collect();
    Ok((data, warnings))
}

fn map_resolution_warning(w: ResolutionWarning) -> Warning {
    Warning {
        code: w.code,
        message: w.message,
    }
}

fn emit_success(data: StatusData, warnings: Vec<Warning>, json: bool) -> ExitCode {
    if !json {
        eprintln!(
            "Status of change '{}' ({}): {} artifact(s)",
            data.change_id,
            data.state,
            data.artifacts.len()
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
    let env: Envelope<StatusData> = Envelope::failure(body, request_id());
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

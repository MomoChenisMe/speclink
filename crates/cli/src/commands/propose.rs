//! `propose create` 子命令 — 將 clap 解析後的 args 串到 runtime 與 provider。

use std::io::Write;
use std::sync::Arc;

use crate::cli::ProposeCreateArgs;
use crate::exit_code::{ErrorCode, ExitCode, classify};
use crate::output::{Envelope, ErrorBody, ProposeCreateData, Warning, request_id};
use provider::config::{GlobalConfig, ProjectConfig};
use provider::config_discovery::{find_project_config, global_config_path};
use provider::resolution::{
    ResolutionInputs, ResolvedProvider, Warning as ResolutionWarning, resolve,
};
use provider_local::LocalProvider;
use runtime::propose::{CreateProposalInput, create_proposal};

const ENV_PROVIDER: &str = "SPECLINK_PROVIDER";

/// `speclink propose create` 入口；負責 wiring config → resolve → provider → runtime → envelope。
pub async fn run(args: ProposeCreateArgs) -> ExitCode {
    if args.flags.stdin {
        // Spec：本 change 沒有指令接受 stdin；收到 --stdin 一律 exit 2。
        return emit_failure_and_exit(
            ErrorCode("input.invalid"),
            "this command does not accept --stdin in this change",
            args.flags.json,
        );
    }

    let project_id = provider::model::ProjectId::from("default");
    let change_id = provider::model::ChangeId::from(args.change.clone());
    let summary = args.summary.clone();
    let json_out = args.flags.json;

    match execute(project_id, change_id, summary).await {
        Ok((output, warnings)) => emit_success(output, warnings, json_out),
        Err(e) => {
            let (code, ec) = classify(&e);
            emit_failure(ec.as_str(), &e.to_string(), json_out, code)
        }
    }
}

struct ExecutedOutput {
    change_id: String,
    artifact_path: String,
    mode: String,
}

async fn execute(
    project_id: provider::model::ProjectId,
    change_id: provider::model::ChangeId,
    summary: String,
) -> Result<(ExecutedOutput, Vec<Warning>), anyhow::Error> {
    let cwd = std::env::current_dir()?;

    // 1) 載入 project / global config
    let project_config: Option<ProjectConfig> = match find_project_config(&cwd) {
        Some(p) => Some(ProjectConfig::load(&p)?),
        None => None,
    };
    let global_config: Option<GlobalConfig> = match global_config_path() {
        Some(p) if p.is_file() => Some(GlobalConfig::load(&p)?),
        _ => None,
    };
    let env_provider = std::env::var(ENV_PROVIDER).ok();
    let env_provider_ref = env_provider.as_deref();

    // 2) Resolve provider
    let inputs = ResolutionInputs {
        flag_provider: None,
        project_config: project_config.as_ref(),
        global_config: global_config.as_ref(),
        env_provider: env_provider_ref,
    };
    let resolved = resolve(inputs)?;

    let provider = match resolved.provider {
        ResolvedProvider::Local { .. } => {
            // MVP：所有 mode 都是 local
            Arc::new(LocalProvider::new(cwd.clone()).await?) as Arc<dyn provider::Provider>
        }
    };

    // 3) Call runtime
    let input = CreateProposalInput {
        project_id,
        change_id: change_id.clone(),
        summary,
    };
    let output = create_proposal(provider, input).await?;

    let warnings: Vec<Warning> = resolved
        .warnings
        .into_iter()
        .map(map_resolution_warning)
        .collect();
    Ok((
        ExecutedOutput {
            change_id: output.change_id.as_str().to_string(),
            artifact_path: posix_path(&output.artifact_path),
            mode: "local".to_string(),
        },
        warnings,
    ))
}

fn map_resolution_warning(w: ResolutionWarning) -> Warning {
    Warning {
        code: w.code,
        message: w.message,
    }
}

fn posix_path(p: &str) -> String {
    p.replace('\\', "/")
}

fn emit_success(out: ExecutedOutput, warnings: Vec<Warning>, json: bool) -> ExitCode {
    if !json {
        // Human-readable mode
        eprintln!(
            "Created change '{}' at {}",
            out.change_id, out.artifact_path
        );
        return ExitCode::from(0);
    }
    let data = ProposeCreateData {
        change_id: out.change_id,
        state: "proposed".to_string(),
        artifact_path: out.artifact_path,
        mode: out.mode,
    };
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
    let env: Envelope<ProposeCreateData> = Envelope::failure(body, request_id());
    write_envelope(&env);
    exit
}

fn emit_failure_and_exit(code: ErrorCode, message: &str, json: bool) -> ExitCode {
    emit_failure(code.as_str(), message, json, ExitCode::from(2))
}

fn write_envelope<T: serde::Serialize>(env: &Envelope<T>) {
    let s = match serde_json::to_string(env) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut stdout = std::io::stdout().lock();
    // 明確寫 bytes 強制 LF 而非 \r\n；忽略寫入錯誤（exit code 已決定）。
    let _ = stdout.write_all(s.as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

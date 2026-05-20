//! `artifact write` 子命令 — 從 stdin 讀取 artifact 內容並透過 runtime 寫入。

use std::io::{Read, Write};
use std::sync::Arc;

use crate::cli::{ArtifactWriteArgs, ArtifactWriteCommand, ArtifactWriteSpecArgs};
use crate::exit_code::{ErrorCode, ExitCode, classify};
use crate::output::{ArtifactWriteData, Envelope, ErrorBody, Warning, request_id};
use provider::config::{GlobalConfig, ProjectConfig};
use provider::config_discovery::{find_project_config, global_config_path};
use provider::model::ArtifactKind;
use provider::resolution::{
    ResolutionInputs, ResolvedProvider, Warning as ResolutionWarning, resolve,
};
use provider_local::LocalProvider;
use runtime::artifact::{WriteArtifactInput, write_artifact};

const ENV_PROVIDER: &str = "SPECLINK_PROVIDER";

/// `speclink artifact write <kind>` 入口；負責 stdin 讀取 + provider resolve + runtime 呼叫 + envelope 輸出。
pub async fn run(cmd: ArtifactWriteCommand) -> ExitCode {
    match cmd {
        ArtifactWriteCommand::Design(args) => run_inner(ArtifactKind::Design, args, None).await,
        ArtifactWriteCommand::Tasks(args) => run_inner(ArtifactKind::Tasks, args, None).await,
        ArtifactWriteCommand::Spec(spec_args) => {
            let ArtifactWriteSpecArgs {
                change,
                capability,
                stdin,
                json,
                no_color,
                quiet,
            } = spec_args;
            let unified = ArtifactWriteArgs {
                change,
                stdin,
                capability: None, // spec arg 已單獨保留 capability，避免雙重來源
                json,
                no_color,
                quiet,
            };
            run_inner(ArtifactKind::Spec, unified, Some(capability)).await
        }
    }
}

async fn run_inner(
    kind: ArtifactKind,
    args: ArtifactWriteArgs,
    spec_capability: Option<String>,
) -> ExitCode {
    let json_out = args.json;

    // design / tasks 若帶 --capability → input.invalid
    if kind != ArtifactKind::Spec && args.capability.is_some() {
        return emit_input_invalid(
            "--capability is only valid for `artifact write spec`",
            json_out,
        );
    }

    // 讀 stdin
    let content = match read_stdin_utf8() {
        Ok(c) => c,
        Err(StdinError::Empty) => {
            return emit_input_invalid("stdin must not be empty", json_out);
        }
        Err(StdinError::InvalidUtf8) => {
            return emit_input_invalid("stdin must be valid UTF-8", json_out);
        }
        Err(StdinError::Io(msg)) => {
            return emit_internal_error(&format!("stdin read failed: {msg}"), json_out);
        }
    };
    let content = ensure_trailing_lf(content);

    let project_id = provider::model::ProjectId::from("default");
    let change_id = provider::model::ChangeId::from(args.change.clone());

    match execute(project_id, change_id, kind, content, spec_capability).await {
        Ok((output, warnings)) => emit_success(output, warnings, json_out),
        Err(e) => {
            let (code, ec) = classify(&e);
            emit_failure(ec.as_str(), &e.to_string(), json_out, code)
        }
    }
}

struct ExecutedOutput {
    change_id: String,
    artifact_id: String,
    kind: String,
    path: String,
    mode: String,
}

async fn execute(
    project_id: provider::model::ProjectId,
    change_id: provider::model::ChangeId,
    kind: ArtifactKind,
    content: String,
    capability: Option<String>,
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

    // 2) Resolve provider
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

    // 3) Call runtime
    let input = WriteArtifactInput {
        project_id,
        change_id,
        kind,
        content,
        capability,
    };
    let output = write_artifact(provider, input).await?;

    let warnings: Vec<Warning> = resolved
        .warnings
        .into_iter()
        .map(map_resolution_warning)
        .collect();
    Ok((
        ExecutedOutput {
            change_id: output.change_id.as_str().to_string(),
            artifact_id: output.artifact_id,
            kind: artifact_kind_str(output.kind).to_string(),
            path: output.path,
            mode: "local".to_string(),
        },
        warnings,
    ))
}

fn artifact_kind_str(k: ArtifactKind) -> &'static str {
    match k {
        ArtifactKind::Proposal => "proposal",
        ArtifactKind::Design => "design",
        ArtifactKind::Tasks => "tasks",
        ArtifactKind::Spec => "spec",
    }
}

fn map_resolution_warning(w: ResolutionWarning) -> Warning {
    Warning {
        code: w.code,
        message: w.message,
    }
}

enum StdinError {
    Empty,
    InvalidUtf8,
    Io(String),
}

fn read_stdin_utf8() -> Result<String, StdinError> {
    let mut buf = Vec::new();
    let stdin = std::io::stdin();
    let mut locked = stdin.lock();
    locked
        .read_to_end(&mut buf)
        .map_err(|e| StdinError::Io(e.to_string()))?;
    if buf.is_empty() {
        return Err(StdinError::Empty);
    }
    String::from_utf8(buf).map_err(|_| StdinError::InvalidUtf8)
}

fn ensure_trailing_lf(mut s: String) -> String {
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn emit_success(out: ExecutedOutput, warnings: Vec<Warning>, json: bool) -> ExitCode {
    if !json {
        eprintln!(
            "Wrote {} for change '{}' at {}",
            out.artifact_id, out.change_id, out.path
        );
        return ExitCode::from(0);
    }
    let data = ArtifactWriteData {
        change_id: out.change_id,
        artifact_id: out.artifact_id,
        kind: out.kind,
        path: out.path,
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
    let env: Envelope<ArtifactWriteData> = Envelope::failure(body, request_id());
    write_envelope(&env);
    exit
}

fn emit_input_invalid(message: &str, json: bool) -> ExitCode {
    emit_failure(
        ErrorCode("input.invalid").as_str(),
        message,
        json,
        ExitCode::from(2),
    )
}

fn emit_internal_error(message: &str, json: bool) -> ExitCode {
    emit_failure(
        ErrorCode("internal.error").as_str(),
        message,
        json,
        ExitCode::from(1),
    )
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

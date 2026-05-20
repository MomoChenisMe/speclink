//! `archive` 子命令 — 把 clap 解析後的 `ArchiveArgs` 串到 runtime + provider，並寫出
//! `Envelope<ArchiveData>`。
//!
//! Archive date 在此處決定（`chrono::Local::now().date_naive()`），測試可透過
//! `SPECLINK_TEST_ARCHIVE_DATE=YYYY-MM-DD` 環境變數覆寫，以利 snapshot / 整合測試固定日期。

use std::io::Write;
use std::sync::Arc;

use crate::cli::ArchiveArgs;
use crate::exit_code::{ErrorCode, ExitCode, classify};
use crate::output::{
    ArchiveData, Envelope, ErrorBody, Warning, archived_change_to_archive_data, request_id,
};
use provider::config::{GlobalConfig, ProjectConfig};
use provider::config_discovery::{find_project_config, global_config_path};
use provider::model::ArchiveOptions;
use provider::resolution::{
    ResolutionInputs, ResolvedProvider, Warning as ResolutionWarning, resolve,
};
use provider_local::LocalProvider;
use runtime::archive::{ArchiveInput, archive};

const ENV_PROVIDER: &str = "SPECLINK_PROVIDER";
const ENV_TEST_ARCHIVE_DATE: &str = "SPECLINK_TEST_ARCHIVE_DATE";

/// `speclink archive <change>` 入口。
pub async fn run(args: ArchiveArgs) -> ExitCode {
    let json_out = args.flags.json;

    // archive 不接受 --stdin
    if args.flags.stdin {
        return emit_failure(
            ErrorCode("input.invalid").as_str(),
            "archive does not accept --stdin",
            json_out,
            ExitCode::from(2),
        );
    }

    let archive_date = match resolve_archive_date() {
        Ok(d) => d,
        Err(msg) => {
            return emit_failure(
                ErrorCode("input.invalid").as_str(),
                &msg,
                json_out,
                ExitCode::from(2),
            );
        }
    };

    let project_id = provider::model::ProjectId::from("default");
    let change_id = provider::model::ChangeId::from(args.change.clone());
    let options = ArchiveOptions {
        dry_run: args.dry_run,
        archive_date,
    };

    match execute(project_id, change_id, options).await {
        Ok((data, warnings)) => emit_success(data, warnings, json_out),
        Err(e) => {
            let (code, ec) = classify(&e);
            emit_failure(ec.as_str(), &e.to_string(), json_out, code)
        }
    }
}

fn resolve_archive_date() -> Result<chrono::NaiveDate, String> {
    if let Ok(s) = std::env::var(ENV_TEST_ARCHIVE_DATE) {
        if !s.is_empty() {
            return chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|e| format!("{ENV_TEST_ARCHIVE_DATE} must be YYYY-MM-DD: {e}"));
        }
    }
    Ok(chrono::Local::now().date_naive())
}

async fn execute(
    project_id: provider::model::ProjectId,
    change_id: provider::model::ChangeId,
    options: ArchiveOptions,
) -> Result<(ArchiveData, Vec<Warning>), anyhow::Error> {
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

    let archived = archive(
        provider,
        ArchiveInput {
            project_id,
            change_id,
            options,
        },
    )
    .await?;

    let warnings: Vec<Warning> = resolved
        .warnings
        .into_iter()
        .map(map_resolution_warning)
        .collect();
    Ok((archived_change_to_archive_data(archived), warnings))
}

fn map_resolution_warning(w: ResolutionWarning) -> Warning {
    Warning {
        code: w.code,
        message: w.message,
    }
}

fn emit_success(data: ArchiveData, warnings: Vec<Warning>, json: bool) -> ExitCode {
    if !json {
        let suffix = if data.dry_run { " (dry-run)" } else { "" };
        eprintln!(
            "Archived '{}' to {}{}",
            data.change_id, data.archive_path, suffix
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
    let env: Envelope<ArchiveData> = Envelope::failure(body, request_id());
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

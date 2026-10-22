//! HTTP boundary only: authentication is resolved before the shared Host operation.
use crate::{auth::Binding, error::ApiError, state::AppState, verb};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use speclink_host::security::remote::{Operation, Outcome};
use speclink_protocol::security::*;

pub(crate) fn standard_repository(application_data:&std::path::Path)->anyhow::Result<speclink_fs::security::FsSecurityData> {
    let paths=speclink_host::security_paths::lease(application_data)?;
    Ok(speclink_fs::security::FsSecurityData::with_standard_root(application_data,&paths.standards_root)?.hold_path_lease(paths.lock))
}

async fn run(
    state: AppState,
    binding: Binding,
    change: String,
    operation: Operation,
) -> Result<Outcome, ApiError> {
    let store = state.store.clone();
    let ctx = binding.execution_context();
    let key = speclink_host::security::remote_project_key(
        &state.config.public_url,
        &binding.project.key,
        &binding.repo,
    )
    .map_err(ApiError::security)?;
    let result = tokio::task::spawn_blocking(move || {
        let data = standard_repository(
            &speclink_host::context::security_data_dir()?,
        )?;
        speclink_host::security::remote::execute(
            store.as_ref(),
            &ctx,
            &key,
            &change,
            &data,
            operation,
        )
    })
    .await
    .map_err(|_| ApiError::internal("security worker failed"))?
    .map_err(ApiError::security);
    if result.is_ok() {
        state.events.notify(&verb::scope_of(&binding));
    }
    result
}
pub async fn show(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, change)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    match run(state, binding, change, Operation::Show(None)).await? {
        Outcome::View(v) => Ok(Json(v).into_response()),
        _ => Err(ApiError::internal("security outcome")),
    }
}
pub async fn check(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, change)): Path<(String, String)>,
    Json(request): Json<SecurityCheckRequest>,
) -> Result<Response, ApiError> {
    match run(
        state,
        binding,
        change,
        Operation::Show(Some(request.security_source_attestation)),
    )
    .await?
    {
        Outcome::View(v) => Ok(Json(v).into_response()),
        _ => Err(ApiError::internal("security outcome")),
    }
}
pub async fn report(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, change, id)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let ctx = binding.execution_context();
    let report = tokio::task::spawn_blocking(move ||
        speclink_host::security::remote::report(state.store.as_ref(), &ctx, &change, &id)
    ).await.map_err(|_| ApiError::internal("security worker failed"))?.map_err(ApiError::security)?;
    Ok(Json(report).into_response())
}
pub async fn submit(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, change)): Path<(String, String)>,
    Json(request): Json<SecurityRoundRequest>,
) -> Result<Response, ApiError> {
    match run(state, binding, change, Operation::Submit(request)).await? {
        Outcome::Submitted(v) => Ok(Json(v).into_response()),
        _ => Err(ApiError::internal("security outcome")),
    }
}
pub async fn accept(
    State(state): State<AppState>,
    binding: Binding,
    Path((_key, change)): Path<(String, String)>,
    Json(request): Json<SecurityAcceptRequest>,
) -> Result<Response, ApiError> {
    match run(state, binding, change, Operation::Accept(request)).await? {
        Outcome::Accepted(v) => Ok(Json(v).into_response()),
        _ => Err(ApiError::internal("security outcome")),
    }
}

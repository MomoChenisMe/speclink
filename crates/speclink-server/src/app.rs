//! HTTP router assembly. Verb routes are layered on in later knives; this is
//! the single assembly point they extend.

use crate::auth::Binding;
use crate::device;
use crate::routes;
use crate::state::AppState;
use crate::web;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use speclink_protocol::binding::BindingResponse;

/// Build the HTTP router over the shared application state.
pub fn router(state: AppState) -> Router {
    let project = Router::new()
        .route("/binding", get(binding))
        .route("/changes", get(routes::list_changes).post(routes::create_change))
        .route("/changes/{name}", get(routes::get_change))
        .route("/changes/{name}/instructions/{*artifact}", get(routes::instructions))
        .route(
            "/changes/{name}/artifacts/{*artifact}",
            get(routes::get_artifact).put(routes::put_artifact),
        )
        .route("/changes/{name}/tasks/{task_id}/done", post(routes::task_done))
        .route("/changes/{name}/tasks/{task_id}/undone", post(routes::task_undone))
        .route("/changes/{name}/claim", post(routes::claim))
        .route("/changes/{name}/archive", post(routes::archive))
        .route("/discussions", get(routes::list_discussions).post(routes::create_discussion))
        .route("/discussions/{slug}", get(routes::show_discussion))
        .route("/discussions/{slug}/context", put(routes::set_discussion_context))
        .route("/discussions/{slug}/rounds", post(routes::add_discussion_round))
        .route("/discussions/{slug}/conclude", post(routes::conclude_discussion))
        .route("/discussions/{slug}/archive", post(routes::archive_discussion))
        .route("/discussions/{slug}/promote", post(routes::promote_discussion))
        .route("/specs", get(routes::list_specs))
        .route("/language", get(routes::language))
        .route("/config", get(routes::config))
        .route("/whoami", get(routes::whoami))
        .route("/sync-state", get(routes::sync_state))
        .route("/events", get(routes::events));
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/invite/{token}", get(web::invite_page).post(web::accept_invite))
        .route("/login", get(web::login_page).post(web::do_login))
        .route("/logout", post(web::do_logout))
        .route("/account", get(web::account_page))
        .route("/account/tokens", post(web::create_pat))
        .route("/account/tokens/{id}/revoke", post(web::revoke_pat))
        .route("/account/device/{id}/revoke", post(web::revoke_device_family))
        .route("/activate", get(web::activate_page).post(web::activate_submit))
        .route("/auth/device", post(device::initiate))
        .route("/auth/device/token", post(device::poll_token))
        .route("/auth/refresh", post(device::refresh))
        .route("/auth/revoke", post(device::revoke))
        .nest("/api/speclink/v1/projects/{key}", project)
        .with_state(state)
}

/// `GET /healthz` — process liveness. Answers as long as the process serves,
/// independent of store health.
async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// `GET /readyz` — readiness gated on the store backend. A store that cannot
/// serve answers non-2xx so a load balancer stops routing to this instance.
async fn readyz(State(state): State<AppState>) -> StatusCode {
    match state.store.health() {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// `GET /binding` — the handshake. The `Binding` extractor already ran the
/// authentication and binding precondition; a failure never reaches here.
async fn binding(binding: Binding) -> Json<BindingResponse> {
    Json(binding.to_response())
}

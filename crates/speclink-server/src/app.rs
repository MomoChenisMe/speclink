//! HTTP router assembly. Verb routes are layered on in later knives; this is
//! the single assembly point they extend.

use crate::admin;
use crate::auth::Binding;
use crate::context;
use crate::device;
use crate::routes;
use crate::setup;
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
        .route("/changes/{name}/drift", get(routes::drift))
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
        .route("/context", post(context::snapshot))
        .route("/events", get(routes::events));
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/setup", get(setup::setup_page).post(setup::setup_submit))
        .route("/invite/{token}", get(web::invite_page).post(web::accept_invite))
        .route("/login", get(web::login_page).post(web::do_login))
        .route("/logout", post(web::do_logout))
        .route("/account", get(web::account_page))
        .route("/account/tokens", post(web::create_pat))
        .route("/account/tokens/{id}/revoke", post(web::revoke_pat))
        .route("/account/device/{id}/revoke", post(web::revoke_device_family))
        .route("/activate", get(web::activate_page).post(web::activate_submit))
        .route("/admin", get(admin::admin_home))
        .route("/admin/users", get(admin::users_page))
        .route("/admin/users/invite", post(admin::web_invite))
        .route("/admin/users/{id}/suspend", post(admin::web_suspend_user))
        .route("/admin/users/{id}/reactivate", post(admin::web_reactivate_user))
        .route("/admin/users/{id}/membership", post(admin::web_set_membership))
        .route("/admin/users/{id}/admin-flag", post(admin::web_set_admin_flag))
        .route("/admin/registry", get(admin::registry_page))
        .route("/admin/registry/projects", post(admin::web_create_project))
        .route("/admin/registry/projects/{key}/rename", post(admin::web_rename_project))
        .route("/admin/registry/repos", post(admin::web_create_repo))
        .route("/admin/registry/repos/rename", post(admin::web_rename_repo))
        .route("/admin/credentials", get(admin::credentials_page))
        .route("/admin/credentials/tokens/{id}/revoke", post(admin::web_revoke_token))
        .route("/admin/credentials/families/{id}/revoke", post(admin::web_revoke_family))
        .route("/admin/audit", get(admin::audit_page))
        .route("/admin/system", get(admin::system_page))
        .route("/admin/data", get(admin::data_page))
        .route("/admin/data/export/{project}/{repo}", get(admin::web_export_scope))
        .route("/admin/data/migrate", post(admin::web_migrate_store))
        .route("/auth/device", post(device::initiate))
        .route("/auth/device/token", post(device::poll_token))
        .route("/auth/refresh", post(device::refresh))
        .route("/auth/revoke", post(device::revoke))
        .route("/auth/whoami", get(crate::auth::auth_whoami))
        .nest("/api/speclink/v1/admin", admin::api_router())
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

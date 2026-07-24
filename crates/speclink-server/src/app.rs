//! HTTP router assembly. Verb routes are layered on in later knives; this is
//! the single assembly point they extend.

use crate::admin;
use crate::assets;
use crate::auth::Binding;
use crate::context;
use crate::device;
use crate::read_api;
use crate::routes;
use crate::setup;
use crate::state::AppState;
use crate::web;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use speclink_protocol::binding::BindingResponse;

/// Workspace migration uploads a complete local `openspec/` snapshot in one
/// atomic request. Keep the larger allowance scoped to `/import`; ordinary API
/// routes retain Axum's conservative default body limit.
const IMPORT_BODY_LIMIT_BYTES: usize = 32 * 1024 * 1024;

/// Transport bound for the board-order routes: generous enough that an
/// over-cap request is still read in full and answered with a clean 413 by
/// the handler (which enforces the real content cap) instead of a dropped
/// connection mid-upload.
const BOARD_ORDER_BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

/// Build the HTTP router over the shared application state.
pub fn router(state: AppState) -> Router {
    let project = Router::new()
        .route("/binding", get(binding))
        .route(
            "/changes",
            get(routes::list_changes).post(routes::create_change),
        )
        .route(
            "/changes/{name}",
            get(routes::get_change).delete(routes::delete_change),
        )
        .route("/changes/{name}/drift", get(routes::drift))
        .route("/changes/{name}/validate", get(routes::validate_change))
        .route("/changes/{name}/analyze", get(routes::analyze_change))
        .route("/changes/{name}/tasks/move", post(routes::move_task))
        .route(
            "/changes/{name}/instructions/{*artifact}",
            get(routes::instructions),
        )
        .route(
            "/changes/{name}/artifacts/{*artifact}",
            get(routes::get_artifact).put(routes::put_artifact),
        )
        .route(
            "/changes/{name}/tasks/{task_id}/done",
            post(routes::task_done),
        )
        .route(
            "/changes/{name}/tasks/{task_id}/undone",
            post(routes::task_undone),
        )
        .route("/changes/{name}/claim", post(routes::claim))
        .route("/changes/{name}/archive", post(routes::archive))
        .route(
            "/discussions",
            get(routes::list_discussions).post(routes::create_discussion),
        )
        .route("/discussions/{slug}", get(routes::show_discussion))
        .route(
            "/discussions/{slug}/context",
            put(routes::set_discussion_context),
        )
        .route(
            "/discussions/{slug}/rounds",
            post(routes::add_discussion_round),
        )
        .route(
            "/discussions/{slug}/conclude",
            post(routes::conclude_discussion),
        )
        .route(
            "/discussions/{slug}/archive",
            post(routes::archive_discussion),
        )
        .route(
            "/discussions/{slug}/promote",
            post(routes::promote_discussion),
        )
        .route("/specs", get(routes::list_specs))
        .route("/specs/{capability}/document", get(read_api::spec_document))
        .route("/archived", get(read_api::archived_list))
        .route(
            "/archived/{dated_name}/artifacts/{*artifact}",
            get(read_api::archived_artifact),
        )
        .route(
            "/archived/{dated_name}/capabilities",
            get(read_api::archived_capabilities),
        )
        .route("/search", get(read_api::search))
        .route("/language", get(routes::language))
        .route("/config", get(routes::config).put(routes::put_config))
        .route(
            "/board-order",
            get(routes::board_order)
                .put(routes::put_board_order)
                .layer(DefaultBodyLimit::max(BOARD_ORDER_BODY_LIMIT_BYTES)),
        )
        .route(
            "/import",
            post(routes::import_bundle).layer(DefaultBodyLimit::max(IMPORT_BODY_LIMIT_BYTES)),
        )
        .route("/whoami", get(routes::whoami))
        .route("/sync-state", get(routes::sync_state))
        .route("/context", post(context::snapshot))
        .route("/events", get(routes::events));
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/setup", get(setup::setup_page).post(setup::setup_submit))
        .route(
            "/invite/{token}",
            get(web::invite_page).post(web::accept_invite),
        )
        .route("/login", get(web::login_page).post(web::do_login))
        .route("/logout", post(web::do_logout))
        .route("/account", get(web::account_page))
        .route("/account/tokens", post(web::create_pat))
        .route("/account/tokens/{id}/revoke", post(web::revoke_pat))
        .route(
            "/account/device/{id}/revoke",
            post(web::revoke_device_family),
        )
        .route(
            "/activate",
            get(web::activate_page).post(web::activate_submit),
        )
        .route("/admin", get(admin::admin_home))
        .route("/admin/users", get(admin::users_page))
        .route("/admin/users/invite", post(admin::web_invite))
        .route("/admin/users/{id}/suspend", post(admin::web_suspend_user))
        .route(
            "/admin/users/{id}/reactivate",
            post(admin::web_reactivate_user),
        )
        .route(
            "/admin/users/{id}/membership",
            post(admin::web_set_membership),
        )
        .route(
            "/admin/users/{id}/admin-flag",
            post(admin::web_set_admin_flag),
        )
        .route("/admin/registry", get(admin::registry_page))
        .route("/admin/registry/projects", post(admin::web_create_project))
        .route(
            "/admin/registry/projects/{key}/rename",
            post(admin::web_rename_project),
        )
        .route("/admin/registry/repos", post(admin::web_create_repo))
        .route("/admin/registry/repos/rename", post(admin::web_rename_repo))
        .route("/admin/credentials", get(admin::credentials_page))
        .route(
            "/admin/credentials/tokens/{id}/revoke",
            post(admin::web_revoke_token),
        )
        .route(
            "/admin/credentials/families/{id}/revoke",
            post(admin::web_revoke_family),
        )
        .route("/admin/audit", get(admin::audit_page))
        .route("/admin/system", get(admin::system_page))
        .route("/admin/data", get(admin::data_page))
        .route(
            "/admin/data/export/{project}/{repo}",
            get(admin::web_export_scope),
        )
        .route("/admin/data/migrate", post(admin::web_migrate_store))
        .route("/auth/device", post(device::initiate))
        .route("/auth/device/token", post(device::poll_token))
        .route("/auth/refresh", post(device::refresh))
        .route("/auth/revoke", post(device::revoke))
        .route("/auth/whoami", get(crate::auth::auth_whoami))
        .route("/api/speclink/v1/scopes", get(read_api::scopes))
        .nest(
            "/api/speclink/v1/web",
            web::api_router().merge(setup::web_router()),
        )
        .nest("/api/speclink/v1/admin", admin::api_router())
        .nest("/api/speclink/v1/projects/{key}", project)
        // 內嵌 SPA（D5）：內容雜湊資產與 browser-route fallback shell。fallback 只在
        // 上方任何明確 route 都未命中時執行，故永不吞掉 API／auth／health／下載 route。
        .route("/assets/{*path}", get(assets::asset))
        .fallback(assets::spa_fallback)
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

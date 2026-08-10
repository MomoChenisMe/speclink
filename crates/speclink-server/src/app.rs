//! HTTP router assembly. Verb routes are layered on in later knives; this is
//! the single assembly point they extend.

use crate::admin;
use crate::assets;
use crate::auth::Binding;
use crate::context;
use crate::device;
use crate::error::ApiError;
use crate::read_api;
use crate::routes;
use crate::setup;
use crate::state::AppState;
use crate::web;
use axum::body::{to_bytes, Body};
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
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
        .route(
            "/changes/{name}/review",
            get(routes::review_show).delete(routes::review_discard),
        )
        .route("/changes/{name}/review/rounds", post(routes::review_add_round))
        .route("/changes/{name}/review/stamp", post(routes::review_stamp))
        .route(
            "/changes/{name}/verify",
            get(routes::verify_show).delete(routes::verify_discard),
        )
        .route("/changes/{name}/verify/rounds", post(routes::verify_add_round))
        .route("/changes/{name}/verify/stamp", post(routes::verify_stamp))
        .route("/changes/{name}/claim", post(routes::claim))
        .route(
            "/changes/{name}/in-progress",
            post(routes::in_progress).delete(routes::in_progress_remove),
        )
        .route("/changes/{name}/archive", post(routes::archive))
        .route(
            "/discussions",
            get(routes::list_discussions).post(routes::create_discussion),
        )
        .route(
            "/discussions/{slug}",
            get(routes::show_discussion).delete(routes::delete_discussion),
        )
        .route("/discussions/{slug}/link", post(routes::link_discussion))
        .route("/discussions/{slug}/seal", post(routes::seal_discussion))
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
        .route("/events", get(routes::events))
        .layer(middleware::from_fn(read_body_before_routing));
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        // 唯一保留的 /admin HTML route：scope export bundle 下載（session-admin gated）。
        // 其餘 setup／invite／login／account／activate／admin 頁面與 form 已由內嵌 SPA
        // 與 `/api/speclink/v1/web` browser API 取代（D8 第六階段）。
        .route(
            "/admin/data/export/{project}/{repo}",
            get(admin::web_export_scope),
        )
        .route("/auth/device", post(device::initiate))
        .route("/auth/device/token", post(device::poll_token))
        .route("/auth/refresh", post(device::refresh))
        .route("/auth/revoke", post(device::revoke))
        .route("/auth/whoami", get(crate::auth::auth_whoami))
        .route("/api/speclink/v1/scopes", get(read_api::scopes))
        .nest(
            "/api/speclink/v1/web",
            web::api_router()
                .merge(setup::web_router())
                .merge(admin::web_router()),
        )
        .nest("/api/speclink/v1/admin", admin::api_router())
        .nest("/api/speclink/v1/projects/{key}", project)
        // 內嵌 SPA（D5）：內容雜湊資產與 browser-route fallback shell。fallback 只在
        // 上方任何明確 route 都未命中時執行，故永不吞掉 API／auth／health／下載 route。
        .route("/assets/{*path}", get(assets::asset))
        .fallback(assets::spa_fallback)
        .with_state(state)
}

/// 進路由前先把 request body 收滿再放行（project API 家族）。
///
/// `Binding` 這類 `FromRequestParts` extractor 跑在 body extractor 之前，401/403
/// 的早回應會讓 hyper 對還沒到的 body close_read；晚到的 body 段使 kernel 以
/// RST 收線，已送出的錯誤回應在對端緩衝區被整包丟棄——客戶端間歇讀到空 body
///（高負載下 context_api 測試 403 envelope 斷言的間歇紅）。body 先收滿，早回應
/// 永遠走在 body 之後，連線以 FIN 收場。
///
/// 收滿上限取本 route 家族的最大允量（`/import` 的 32MB）；各 route 的實際內容
/// 上限仍由其 `DefaultBodyLimit` 在 extractor 層把關，不因此放寬。
async fn read_body_before_routing(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    match to_bytes(body, IMPORT_BODY_LIMIT_BYTES).await {
        Ok(bytes) => next.run(Request::from_parts(parts, Body::from(bytes))).await,
        Err(e) => {
            ApiError::payload_too_large(format!("request body unreadable: {e}")).into_response()
        }
    }
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

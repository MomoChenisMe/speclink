//! HTTP router assembly. The project routes and the scope list go through
//! the host kit's HTTP door (`forward`, design D8); the server keeps its own
//! routes: login and accounts, the admin surfaces, the static assets, the
//! import and the event stream.

use crate::admin;
use crate::assets;
use crate::auth::Binding;
use crate::device;
use crate::error::ApiError;
use crate::forward;
use crate::routes;
use crate::setup;
use crate::state::AppState;
use crate::web;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, FromRequestParts, Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::Router;
use speclink_host::http::write_refused;

/// Workspace migration uploads a complete local `openspec/` snapshot in one
/// atomic request. Keep the larger allowance scoped to `/import`; ordinary API
/// routes retain Axum's conservative default body limit.
const IMPORT_BODY_LIMIT_BYTES: usize = forward::READ_LIMIT_BYTES;

/// Build the HTTP router over the shared application state.
pub fn router(state: AppState) -> Router {
    // The two project routes the server keeps for itself; every other path
    // under the project goes to the door.
    let project = Router::new()
        .route(
            "/import",
            post(routes::import_bundle).layer(DefaultBodyLimit::max(IMPORT_BODY_LIMIT_BYTES)),
        )
        .route("/events", get(routes::events))
        // 寫入檢查（fail-closed）：以上每條路由的非 GET／HEAD 方法都要 editor。
        .route_layer(middleware::from_fn_with_state(state.clone(), require_editor))
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
        .route("/api/speclink/v1/auth-methods", any(forward::root))
        .route("/api/speclink/v1/whoami", any(forward::root))
        .route("/api/speclink/v1/scopes", any(forward::root))
        .nest(
            "/api/speclink/v1/web",
            web::api_router()
                .merge(setup::web_router())
                .merge(admin::web_router()),
        )
        .nest("/api/speclink/v1/admin", admin::api_router())
        .nest("/api/speclink/v1/projects/{key}", project)
        .route("/api/speclink/v1/projects/{key}/{*rest}", any(forward::project))
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
/// 上限仍由其 `DefaultBodyLimit` 在 extractor 層把關，不因此放寬。轉交入口的
/// 路由（`forward`）自己先收滿本文，理由相同。
async fn read_body_before_routing(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    match forward::read_whole(body).await {
        Ok(bytes) => next.run(Request::from_parts(parts, Body::from(bytes))).await,
        Err(e) => e.into_response(),
    }
}

/// 留在 server 的專案路由（匯入與事件串流）的寫入檢查（server-verb-api「寫入端點
/// 一律要求 editor 角色」）：GET／HEAD 放行，其他方法要求 editor 角色，否則 403、
/// 處理函式不執行。轉交入口的路由由入口依 `can_write` 把關，訊息相同。解析好的
/// [`Binding`] 放進 request extensions，處理函式的 extractor 直接取用，不再解析第二次。
///
/// `route_layer` 也包住每條路由「方法不支援」的 405 處理：對既有路徑送沒有
/// 註冊的非 GET 方法（例如 `POST /changes/{name}/drift`），reader 先收到 403、
/// 沒帶憑證收到 401，只有 editor 會看到 405。寫入一樣擋下，只是狀態碼不同。
async fn require_editor(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD) {
        return next.run(req).await;
    }
    let (mut parts, body) = req.into_parts();
    let binding = match Binding::from_request_parts(&mut parts, &state).await {
        Ok(binding) => binding,
        Err(e) => return e.into_response(),
    };
    if !binding.editor {
        return ApiError(write_refused(&binding.project.key)).into_response();
    }
    parts.extensions.insert(binding);
    next.run(Request::from_parts(parts, body)).await
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

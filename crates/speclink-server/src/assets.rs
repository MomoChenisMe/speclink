//! 內嵌 SPA 資產服務與 browser fallback 安全邊界（server-web-console spec「SPA
//! 資產與 fallback 具可驗證的安全邊界」, 設計決策 D5）。
//!
//! `apps/server-web` 的 Vite production build 於編譯期內嵌（release）／debug 由
//! dist 動態讀取。`/assets/*` 服務內容雜湊檔並帶 immutable 快取與正確 MIME；
//! router fallback 對固定的 browser GET allowlist 回傳 SPA shell（no-cache 與
//! self-only CSP），其餘一律真 404——fallback 永不吞掉 `/api/*`、`/auth/*`、
//! health／readiness、下載、未知資產或未知 browser path。

use axum::extract::Path;
use axum::http::{header, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// Vite production bundle，內嵌自 `apps/server-web/dist`。缺 dist 時 release build
/// 於編譯期失敗（fail closed）。
#[derive(RustEmbed)]
#[folder = "../../apps/server-web/dist"]
struct WebAssets;

/// Release fail-closed：release（非 debug）build 於編譯期要求 production `index.html`
/// 存在，否則以編譯錯誤中止——絕不產生只有 API、沒有內嵌 SPA 的 server artifact
/// （server-release「缺少 Web build 使 release build 失敗」）。debug build 由 dist
/// 動態讀取，不套用此門檻。路徑相對於本源碼檔（`crates/speclink-server/src/`）。
#[cfg(not(debug_assertions))]
const _EMBEDDED_SPA_INDEX: &[u8] =
    include_bytes!("../../../apps/server-web/dist/index.html");

/// SPA shell 的 self-only Content Security Policy。不從 CDN 或 Google Fonts 載入；
/// script／font／img／xhr 皆限同源。`unsafe-inline` 只放寬給 style——Radix UI 原語
/// 於執行期設定 inline 定位樣式；script 維持嚴格。
const SHELL_CSP: &str = "default-src 'self'; \
script-src 'self'; \
style-src 'self' 'unsafe-inline'; \
img-src 'self' data:; \
font-src 'self'; \
connect-src 'self'; \
base-uri 'self'; \
form-action 'self'; \
frame-ancestors 'none'";

/// SPA 擁有的 browser GET route。這些 path 直接開啟或重新整理回傳 shell；其餘
/// 落到真 404（D5 的 allowlist）。
///
/// admin 面只列舉六個管理目的地，不 blanket `/admin/*`——`/admin/changes`、
/// `/admin/specs`、`/admin/discussions` 等 SHALL 維持 404（Non-Goal：管理面不提供
/// changes／specs／discussions 的檢視或編輯）。`/admin/data` 已不是目的地，但仍留在
/// 清單裡：舊書籤要拿到 shell 才能由 SPA 導向 `/admin/system`，回 404 只會是死連結。
fn is_browser_route(path: &str) -> bool {
    matches!(
        path,
        "/" | "/setup"
            | "/login"
            | "/activate"
            | "/account"
            | "/admin"
            | "/admin/users"
            | "/admin/registry"
            | "/admin/credentials"
            | "/admin/data"
            | "/admin/system"
            | "/admin/audit"
    ) || path.starts_with("/invite/")
}

/// 服務內嵌的 SPA shell（`index.html`），帶 no-cache 與 self-only CSP。bundle
/// 不存在時回 404（fail-closed，不回白屏）。
fn serve_shell() -> Response {
    match WebAssets::get("index.html") {
        Some(file) => (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8"),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
                (
                    header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static(SHELL_CSP),
                ),
            ],
            file.data.into_owned(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `GET /assets/{*path}` — 內容雜湊 immutable 資產。只服務內嵌 manifest 中的檔案；
/// 未知資產回真 404，永不回 shell。
pub async fn asset(Path(path): Path<String>) -> Response {
    let key = format!("assets/{path}");
    match WebAssets::get(&key) {
        Some(file) => {
            let mime = mime_guess::from_path(&key).first_or_octet_stream();
            let content_type = HeaderValue::from_str(mime.as_ref())
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
            (
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable"),
                    ),
                ],
                file.data.into_owned(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Router fallback——只對 allowlist 的 browser GET route 回 shell，其餘回真 404。
/// 只在無明確 route 命中時執行，故 `/api/*`、`/auth/*`、`/healthz`、`/readyz` 與
/// 下載 route 永不到此。
///
/// 404 走與其他 API 錯誤同一個 `{error:{code,message}}` JSON 外殼（spec「拼錯 API
/// 不被 SPA fallback 吞掉」要求 JSON 404）——裸的 `StatusCode::NOT_FOUND` 不帶
/// body 也不帶 content type，client 無從分辨「路徑不存在」與「代理層吞掉了回應」。
pub async fn spa_fallback(method: Method, uri: Uri) -> Response {
    if method == Method::GET && is_browser_route(uri.path()) {
        serve_shell()
    } else {
        crate::web::web_err(StatusCode::NOT_FOUND, "not_found", "找不到該路徑")
    }
}

//! SPA 資產服務與 browser fallback 安全邊界（server-web-console spec「SPA 資產與
//! fallback 具可驗證的安全邊界」, 設計決策 D5）。Production 的 Vite 資產於編譯期
//! 內嵌 binary：`/`（及其他 browser route）回傳帶 no-cache 與 self-only CSP 的 SPA
//! shell；`/assets/*` 回傳內容雜湊檔並帶 immutable cache 與正確 MIME；未知
//! asset／API／browser path 回真正的 404，永不回傳 shell。
//!
//! runtime 不依賴相鄰 dist、Node、CDN 或外部字型服務——測試在無外部檔案的
//! in-process server 上驗證資產全部來自同一 binary。

mod common;

use speclink_server::state::AppState;
use speclink_store::memory::MemoryStore;
use std::sync::Arc;

/// Start a server with no seeded identity — asset serving needs no auth.
fn server() -> String {
    let state = AppState {
        events: common::detached_events(),
        store: Arc::new(MemoryStore::new()),
        config: Arc::new(common::demo_config()),
        identity: common::empty_identity(),
    };
    common::start(state)
}

fn agent() -> ureq::Agent {
    ureq::builder().redirects(0).build()
}

fn get(base: &str, path: &str) -> Result<ureq::Response, ureq::Error> {
    agent().get(&format!("{base}{path}")).call()
}

fn status_body(result: Result<ureq::Response, ureq::Error>) -> (u16, String) {
    match result {
        Ok(resp) => {
            let status = resp.status();
            (status, resp.into_string().unwrap_or_default())
        }
        Err(ureq::Error::Status(code, resp)) => (code, resp.into_string().unwrap_or_default()),
        Err(e) => panic!("transport error: {e}"),
    }
}

/// The first `/assets/…<ext>` reference in the SPA shell HTML.
fn first_asset(html: &str, ext: &str) -> Option<String> {
    let mut search = html;
    while let Some(idx) = search.find("/assets/") {
        let rest = &search[idx..];
        let end = rest
            .find(|c| c == '"' || c == '\'' || c == ' ' || c == '>')
            .unwrap_or(rest.len());
        let path = &rest[..end];
        if path.ends_with(ext) {
            return Some(path.to_string());
        }
        search = &rest[1..];
    }
    None
}

/// A Vite content-hashed asset filename is `<name>-<hash>.<ext>` with a hash
/// segment of 8+ url-safe base64-ish chars — the guarantee immutable caching
/// and binary embedding rely on.
fn is_hashed(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    stem.rsplit_once('-')
        .map(|(_, h)| {
            h.len() >= 8 && h.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .unwrap_or(false)
}

#[test]
fn root_returns_spa_shell_with_no_cache_and_self_only_csp() {
    let base = server();
    let resp = get(&base, "/").expect("GET / succeeds");
    assert_eq!(resp.status(), 200, "root serves the SPA shell");
    let content_type = resp.header("content-type").unwrap_or_default().to_string();
    let cache = resp.header("cache-control").unwrap_or_default().to_string();
    let csp = resp
        .header("content-security-policy")
        .unwrap_or_default()
        .to_string();
    let body = resp.into_string().unwrap_or_default();

    assert!(
        content_type.contains("text/html"),
        "shell content-type should be html, got {content_type}"
    );
    assert!(
        cache.contains("no-cache"),
        "shell must not be cached, got {cache}"
    );
    assert!(
        csp.contains("'self'"),
        "shell must carry a self-only CSP, got {csp}"
    );
    assert!(
        !csp.to_lowercase().contains("googleapis"),
        "CSP must not permit external (Google Fonts) origins, got {csp}"
    );
    assert!(
        body.contains("id=\"root\""),
        "shell should host the SPA root mount element"
    );
}

#[test]
fn assets_are_content_hashed_and_served_immutable() {
    let base = server();
    let index = get(&base, "/")
        .expect("GET /")
        .into_string()
        .expect("index body");
    let js = first_asset(&index, ".js").expect("shell references a hashed JS asset");
    let css = first_asset(&index, ".css").expect("shell references a hashed CSS asset");

    assert!(is_hashed(&js), "JS chunk must be content-hashed: {js}");
    assert!(is_hashed(&css), "CSS chunk must be content-hashed: {css}");

    let resp = get(&base, &js).expect("GET the JS asset");
    assert_eq!(resp.status(), 200);
    let ct = resp.header("content-type").unwrap_or_default().to_string();
    let cache = resp.header("cache-control").unwrap_or_default().to_string();
    assert!(ct.contains("javascript"), "JS MIME, got {ct}");
    assert!(
        cache.contains("max-age=31536000") && cache.contains("immutable"),
        "hashed asset must be immutable-cached, got {cache}"
    );

    let resp = get(&base, &css).expect("GET the CSS asset");
    assert_eq!(resp.status(), 200);
    let ct = resp.header("content-type").unwrap_or_default().to_string();
    assert!(ct.contains("text/css"), "CSS MIME, got {ct}");
}

#[test]
fn unknown_asset_returns_404_not_shell() {
    let base = server();
    let (code, body) = status_body(get(&base, "/assets/missing-00000000.js"));
    assert_eq!(code, 404, "an asset not in the manifest is a real 404");
    assert!(
        !body.contains("id=\"root\""),
        "unknown asset must never return the SPA shell"
    );
}

#[test]
fn misspelled_browser_api_returns_404_not_shell() {
    let base = server();
    let (code, body) = status_body(get(&base, "/api/speclink/v1/web/unknown"));
    assert_eq!(code, 404, "a misspelled API path is a real 404");
    assert!(
        !body.contains("id=\"root\""),
        "the SPA fallback must not swallow /api/* paths"
    );
}

#[test]
fn health_and_ready_are_not_swallowed_by_fallback() {
    let base = server();
    let (code, body) = status_body(get(&base, "/healthz"));
    assert_eq!(code, 200);
    assert!(!body.contains("id=\"root\""), "/healthz stays a liveness probe");
    let (code, _) = status_body(get(&base, "/readyz"));
    assert_eq!(code, 200, "memory store is healthy");
}

#[test]
fn every_browser_route_serves_the_spa_shell_after_the_switch() {
    // D8 phase 6: the old server-rendered HTML pages/forms are gone; every defined
    // browser URL is now served by the SPA shell through the fallback, so a direct
    // open or refresh of /login, /setup, /invite/:token, /account, /activate,
    // /admin and /admin/* works. (server-release「Release binary 在空 runtime 載入
    // SPA」at the integration level.)
    let base = server();
    for path in [
        "/login",
        "/setup",
        "/activate",
        "/account",
        "/admin",
        "/admin/users",
        "/admin/audit",
        "/invite/some-token",
    ] {
        let (code, body) = status_body(get(&base, path));
        assert_eq!(code, 200, "{path} serves the SPA shell");
        assert!(
            body.contains("id=\"root\""),
            "{path} must return the SPA shell, not an HTML page"
        );
    }
    // Spec content stays a real 404 — the /admin UI never serves changes/specs/
    // discussions, and these paths are not in the browser-route allowlist.
    for path in ["/admin/changes", "/admin/specs", "/admin/discussions"] {
        let (code, body) = status_body(get(&base, path));
        assert_eq!(code, 404, "{path} is not a browser route");
        assert!(!body.contains("id=\"root\""), "{path} must not return the shell");
    }
}

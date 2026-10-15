## Problem

在 server 後台，使用者詳情頁（`/admin/users/<使用者 id>`）與專案詳情頁（`/admin/registry/<專案代號>`）只能從列表點進去。直接開這兩種網址、在詳情頁按重新整理，或打開別人分享的詳情頁連結，server 都回 JSON 404（`{"error":{"code":"not_found","message":"找不到該路徑"}}`），瀏覽器顯示一段 JSON，不是後台畫面。

受影響的是用 Remote Store 的團隊管理員：他們在後台管理成員資格與專案時，重新整理一下就看不到頁面。2026-10-15 重拍文件截圖時發現。

## Root Cause

`speclink-server` 只對「已定義的瀏覽器路由」回傳 SPA 頁殼（`crates/host/speclink-server/src/web/assets.rs` 的 `is_browser_route`）。這份允許清單寫死了 `/admin/users` 與 `/admin/registry` 兩個列表路徑，但 server-web-detail-pages 變更在 2026-10-15 加入的兩種詳情頁路徑沒有一起加進去。前端的 React Router 認得這兩種路徑，所以從列表點進去（前端切換路由，不經過 server）沒問題；只有瀏覽器向 server 直接要這個網址時才失敗。

`server-web-console` 正式規格已經要求 `/admin/*` 每一頁都「支援直接開啟與重新整理」，所以這是實作漏掉，不是規格改變。

## Proposed Solution

- `is_browser_route` 另外接受兩種詳情頁路徑：`/admin/users/` 或 `/admin/registry/` 之後接**恰好一段**非空的路徑片段（不含 `/`）。
- 多於一段（例如 `/admin/users/a/b`）、空片段（`/admin/users/`）與其他未定義的 `/admin/*` 路徑（例如 `/admin/changes`）照舊回 JSON 404，安全邊界不變。
- 在 server 整合測試補上兩種詳情頁的直接開啟案例，以及上述三種仍回 404 的反例。
- 正式規格「SPA 資產與 fallback 具可驗證的安全邊界」補一個情境，把詳情頁的直接開啟寫成可驗證的行為。

## Non-Goals

- 不改前端路由、詳情頁畫面或列表的導覽方式。
- 不檢查網址裡的使用者 id 或專案代號是否存在：server 只回頁殼，找不到資料時由前端既有的「找不到」畫面處理（server-web-detail-pages 已交付）。
- 不把允許清單改成「`/admin/` 底下全部回頁殼」：那樣會讓拼錯的路徑也回頁殼，違反規格「未知 browser path SHALL NOT 回傳 index.html」。
- 不動 CLI、桌面 app 與 API 路由。

## Success Criteria

- 對執行中的 server 直接 GET `/admin/users/<任一 id>` 與 `/admin/registry/<任一代號>`，回 200 與 SPA 頁殼（含 `id="root"`），瀏覽器顯示後台畫面。
- GET `/admin/users/a/b`、`/admin/users/`、`/admin/changes` 仍回 404，內容不是 SPA 頁殼。
- 既有測試 `every_browser_route_serves_the_spa_shell_after_the_switch` 與新增的案例全部通過：`cargo test -p speclink-server --test it web::assets`。

## Impact

- Affected specs: `server-web-console`（「SPA 資產與 fallback 具可驗證的安全邊界」補詳情頁直接開啟的情境）
- Affected crates: `speclink-server`（只動 web 層的瀏覽器路由判定）；CLI、Host、Store、桌面 app 不受影響。沒有 CLI 指令、`--json` 輸出或設定欄位的改動。
- Affected code:
  - Modified:
    - `crates/host/speclink-server/src/web/assets.rs`
    - `crates/host/speclink-server/tests/it/web/assets.rs`

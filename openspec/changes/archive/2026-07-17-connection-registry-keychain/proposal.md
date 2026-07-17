## Why

Phase 3 的 RemoteDataSource（下一刀）需要「拿得到憑證的連線」才能存在：server 端 device flow／PAT／rotation 已由 Phase 2 全部就位，但 client 端至今沒有任何 device flow 消費者（CLI 的 auth login 只有 PAT 貼上），Desktop 也沒有 server 連線與憑證的任何基礎。roadmap Phase 3 gate 明文「credential 存 OS Keychain；PAT 不進 localStorage、repo 或 URL」——這層地基必須先立。

## What Changes

- speclink-remote 新增 device flow typed client：以 protocol 既有 device DTOs 對 server 的 /auth/device、/auth/device/token、/auth/refresh、/auth/revoke 端點提供 initiate／poll／refresh／revoke 函式——落在 typed client crate 使 CLI 日後可共用（架構 §13.3）。
- speclink-server 新增 root 層 GET /auth/whoami（bearer → 身分顯示名，access token 與 PAT 皆可）：既有 /whoami 綁在 project scope（Binding 需 project key、membership、repo 解析），desktop 連線是 origin 層級、登入當下尚無 project——此端點是連線層級取得身分顯示名的唯一誠實來源，也是 PAT 登入前驗證的落點；speclink-remote 的 device 模組同步提供 whoami 函式。
- Desktop 新增 connection profile registry：saved servers（URL、顯示名、最後登入身分）存 app 設定目錄的 registry 檔，絕不含任何 secret。
- credential 逐 server origin 存 OS Keychain（Rust 側 keyring；macOS Keychain／Windows Credential Manager）：device 流程存 refresh credential、PAT 流程存 PAT；access token 短效僅存記憶體；refresh rotation 後新 refresh credential 回寫 Keychain。
- Desktop 端 device login：新增連線後預設走 device flow——POST /auth/device 探測，成功即開系統瀏覽器至 verification 頁、依 interval 輪詢至核准；端點不存在（404）或 headless 才顯示「使用 PAT」貼上 fallback（架構 §10.5）。
- 登出＝盡力撤銷 server 端（device family 或 PAT 的 revoke）＋刪除 Keychain entry；移除連線連帶登出。
- UI 最小面：設定頁新增「伺服器」頁籤——列表、新增（URL＋顯示名）、登入（device 預設／PAT fallback）、登入身分顯示、登出、移除。完整 Workspace chooser 與 §10.6 設定資訊架構重整屬後續刀。
- secret 衛生鐵律：PAT／refresh credential／access token 不進 TS 狀態、localStorage、registry 檔、log 與 URL；TS 只見連線狀態與身分顯示名；PAT 僅於使用者貼上時單次過境 invoke。

## Capabilities

### New Capabilities

- `desktop-connections`: Desktop 的 server 連線與憑證基礎層——connection registry 無 secret、credential 唯一落點為 OS Keychain、device login 預設與 PAT fallback、登出撤銷語意、secret 不進 TS/localStorage/log。

### Modified Capabilities

- `server-device-auth`: 新增 root 層 bearer 身分查詢端點 GET /auth/whoami——bearer 解析與 Binding 第一步一致（access token／PAT、失敗同一 401），不要求 project scope。

## Impact

- 相容性影響：desktop／client 側純新增（新命令、新頁籤、新 crate 模組）；不動既有 workspace-session 行為與 CLI credentials.yaml 機制。server 端新增一個唯讀路由（/auth/whoami），不動任何既有端點行為——原「server 端零改動」的假設在實作時發現不成立：/whoami 需 project scope，連線層級拿不到身分顯示名。與其他活躍 change 無共檔。
- Affected specs: `desktop-connections`（新增）、`server-device-auth`（修改）
- Affected code:
  - New: crates/speclink-remote/src/device.rs、apps/desktop/src-tauri/src/connections.rs、apps/desktop/src-tauri/src/credentials.rs、apps/desktop/src/adapter/connections.ts、apps/desktop/src/components/ServersPanel.tsx、apps/desktop/src/__tests__/serversPanel.test.tsx、crates/speclink-remote/tests/device_flow.rs、crates/speclink-server/tests/auth_whoami.rs
  - Modified: crates/speclink-remote/src/lib.rs、crates/speclink-remote/Cargo.toml、apps/desktop/src-tauri/Cargo.toml、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src/store.ts、apps/desktop/src/views/SettingsView.tsx、crates/speclink-protocol/src/query.rs、crates/speclink-server/src/routes.rs、crates/speclink-server/src/app.rs、Cargo.lock
  - Removed: 無

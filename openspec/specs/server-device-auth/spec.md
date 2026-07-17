# server-device-auth Specification

## Purpose

TBD - created by archiving change 'server-device-flow'. Update Purpose after archive.

## Requirements

### Requirement: device 授權發起與輪詢狀態機

server SHALL 提供 device 授權發起端點：回應含 device code（高熵，供輪詢）、user code（短碼，供核准頁人工輸入）、verification URI、到期時限與最小輪詢間隔，兩碼 SHALL 僅以 hash 落庫且共用到期。輪詢端點 SHALL 以 device code 回報狀態：未核准回 pending、輪詢間隔低於宣告值回 slow_down、逾期回 expired、被拒絕回 denied、已核准回 access token 與 refresh credential。狀態 SHALL 以 typed DTO 的 status 欄位表達，SHALL NOT 擴充 wire error reason registry；未知 device code SHALL 回 not_found 的 wire error。DTO SHALL 定義於 protocol crate（camelCase、可匯出 JSON Schema）。

#### Scenario: 完整核准流程

- **WHEN** client 發起 device 授權，使用者於核准頁登入並輸入 user code 核准，client 依宣告間隔輪詢
- **THEN** 核准前輪詢回 pending；核准後輪詢回 approved 與 access token、refresh credential；核准記錄綁定核准者身分

#### Scenario: 逾期與拒絕分明

- **WHEN** 一個授權請求超過到期時限後被輪詢；另一個授權請求被使用者於核准頁拒絕後被輪詢
- **THEN** 前者回 expired、後者回 denied；兩者皆不核發任何 token

#### Scenario: 輪詢過密退避

- **WHEN** client 以低於宣告最小間隔連續輪詢
- **THEN** 回 slow_down；不影響該授權請求的有效性

---

<!-- @trace
source: server-device-flow
updated: 2026-07-14
code:
  - crates/speclink-protocol/src/device.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/device.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: 核准頁 session 保護且明確確認

核准頁 SHALL 要求已登入的 session（未登入導向登入頁），使用者 SHALL 輸入 user code 並得到明確的核准/拒絕確認步驟；核准或拒絕 SHALL 記錄操作者身分。核准頁的變更型 POST SHALL 沿用同源驗證。未知、已用或逾期的 user code SHALL 得到同一無效回應，SHALL NOT 區分原因。

#### Scenario: 未登入不能核准

- **WHEN** 未登入的瀏覽器直接開啟核准頁並嘗試提交 user code
- **THEN** 被導向登入頁；該授權請求維持未核准

#### Scenario: 無效 user code 不洩漏狀態

- **WHEN** 於核准頁分別輸入不存在的、已核准過的、已逾期的 user code
- **THEN** 三者得到相同的無效回應文字

---

<!-- @trace
source: server-device-flow
updated: 2026-07-14
code:
  - crates/speclink-protocol/src/device.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/device.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: access token 短效且併入 bearer 前置

access token SHALL 帶可辨識 prefix、SHALL 短效（到期時限固定且遠短於 PAT 慣用值）、僅以 hash 落庫並綁定 user。API bearer 前置 SHALL 依 prefix 分流 PAT 與 access token，其後檢查一致：hash 命中、未撤銷、未過期、所屬 user 為 active、具該 project membership，逐請求查驗 SHALL NOT 有快取；停權 user SHALL 使其全部 device 憑證即時失效。無效類 SHALL 回 401 permission_denied 且不區分原因、非成員 SHALL 回 403 permission_denied。

#### Scenario: access token 與 PAT 等效通行

- **WHEN** 以核准流程取得的有效 access token 呼叫 binding 與查詢路由
- **THEN** 行為與等權限 PAT 一致：binding 的 actor 為核准者身分，查詢正常回應

#### Scenario: 過期 access token 拒於門外

- **WHEN** 以已過短效時限的 access token 呼叫任一路由
- **THEN** 回 401 permission_denied；server 未執行任何 engine 動詞

---

<!-- @trace
source: server-device-flow
updated: 2026-07-14
code:
  - crates/speclink-protocol/src/device.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/device.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: refresh rotation 與 family 撤銷

refresh credential SHALL 一次性：refresh 端點以有效 refresh credential 換發新 access token 與新 refresh credential，舊值 SHALL 立即失效。同一核准產生的憑證 SHALL 同屬一個 credential family；已失效 refresh credential 被重用 SHALL 使整個 family（現行 access token 與 refresh credential）即時撤銷，該次請求 SHALL 回 401 permission_denied。server SHALL 提供以 refresh credential 撤銷自身 family 的端點（登出語意）。帳號頁的 sessions 清單 SHALL 納入 device credential families 並支援逐一撤銷，撤銷 SHALL 即時生效。

#### Scenario: rotation 舊值失效

- **WHEN** 以 refresh credential 換發成功後，再以換發前的舊 refresh credential 請求 refresh
- **THEN** 第二次請求回 401；且該 family 全部憑證被撤銷，換發所得的新 access token 隨後的 API 呼叫也回 401

#### Scenario: 帳號頁撤銷 device session

- **WHEN** 使用者於帳號頁撤銷某 device credential family
- **THEN** 該 family 的 access token 與 refresh credential 的下一次使用皆回 401；其他 family 與 PAT 不受影響

---

<!-- @trace
source: server-device-flow
updated: 2026-07-14
code:
  - crates/speclink-protocol/src/device.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/device.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: identity schema 演進守門

device 憑證相關表 SHALL 落 identity 資料庫並使 schema version 遞增；舊版本資料庫 SHALL 經 migrate 升級且既有 users、memberships、PATs 與 sessions 資料完整保留；較新版本資料庫 SHALL 拒開。全部 device 憑證 SHALL 僅以 hash 落庫，SHALL NOT 出現在 log。

#### Scenario: 舊版 identity 資料庫升級無損

- **WHEN** 以前一 schema version 的 identity 資料庫（含既有 user 與 PAT）啟動新版 server 並執行 migrate
- **THEN** 升級成功；既有 user 可登入、既有 PAT 照常通行；device 憑證功能可用

<!-- @trace
source: server-device-flow
updated: 2026-07-14
code:
  - crates/speclink-protocol/src/device.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-server/Cargo.toml
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/device.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/src/lib.rs
  - crates/speclink-server/src/web.rs
  - crates/speclink-server/tests/auth_device.rs
  - crates/speclink-server/tests/device_e2e.rs
  - crates/speclink-server/tests/device_flow.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/refresh_rotation.rs
  - crates/speclink-server/tests/web_activate.rs
  - crates/speclink-server/tests/web_device_sessions.rs
-->

---
### Requirement: root 層 bearer 身分查詢

Server SHALL 於 root 層提供 GET /auth/whoami：以 Authorization bearer 解析身分，成功回傳該使用者的顯示名與識別。bearer 解析 SHALL 與 project-scoped Binding 的第一步一致——`spk_at_` 前綴走 device access token 驗證、其餘走 PAT 驗證；任何解析失敗 SHALL 回同一 401 permission_denied、SHALL NOT 區分原因。PAT 命中 SHALL 前進其 last-used。此端點 SHALL NOT 要求 project scope、API version header 或 repo header——它是登入完成當下、尚未選定 project 的 client 取得身分顯示名的來源。

#### Scenario: access token 查得身分

- **WHEN** 以 device flow 核准取得的 access token 呼叫 GET /auth/whoami
- **THEN** 回 200 與核准者的顯示名與識別

#### Scenario: PAT 查得身分且前進 last-used

- **WHEN** 以有效 PAT 呼叫 GET /auth/whoami
- **THEN** 回 200 與擁有者的顯示名；該 PAT 的 last-used 時間前進

#### Scenario: 無效 bearer 是同一 401

- **WHEN** 以缺席、格式錯誤、已撤銷或已過期的 bearer 呼叫 GET /auth/whoami
- **THEN** 回 401 permission_denied，回應不區分失敗原因

<!-- @trace
source: connection-registry-keychain
updated: 2026-07-17
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/SettingsView.tsx
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/device_flow.rs
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/tests/auth_whoami.rs
-->
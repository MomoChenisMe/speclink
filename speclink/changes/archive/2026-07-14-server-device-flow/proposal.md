## Why

平台藍圖 §13.3 把 device authorization 定為 Desktop 的首選登入：使用者在 Desktop 輸入 server URL，Desktop 取得 device code、系統瀏覽器開啟核准頁、使用者登入核准後 Desktop 輪詢換得短效 access token 與 rotating refresh credential——PAT 貼上只留給 CI、headless 與明確選擇的情境。roadmap §5 的 Phase 2 gate 要求「device flow 連 Desktop」在 server 側就緒；Phase 3 的 Desktop 遠端 Workspace 與 OS Keychain 整合都以此為前提。server-identity-pat 刀已備妥帳號、session 與登入頁，本刀在其上補齊 device flow 的端點、憑證種類與核准頁，並讓未來 client 有 typed DTO 可講——現在不做，Phase 3 的 Desktop 刀就得回頭補 server。

目標使用者：Phase 3 起的 Desktop/CLI 使用者（免貼 token 的登入體驗）與實作 RemoteDataSource 的開發者（以 typed DTO 對接 device flow）。

## What Changes

- 新增 device authorization 端點組：發起端點回 device code、user code、verification URI、到期與輪詢間隔；輪詢端點以 device code 換取結果——未核准回 pending、輪詢過密回 slow_down、逾期回 expired、拒絕回 denied、核准回短效 access token 加 rotating refresh credential。全部狀態以 typed DTO 表達，不擴充 wire error reason 八值 registry。
- 新增 /activate 核准頁（沿用 server-identity-pat 的 session 登入與同源防護）：登入的使用者輸入 user code，確認後核准或拒絕該 device 請求；核准綁定核准者身分。
- 新增 access token 與 refresh credential 兩種 identity 憑證：access token 短效、hash 落庫、走既有 bearer 前置（與 PAT 並存，逐請求查驗與即時失效語意一致）；refresh credential 輪換——每次換發舊值即失效，偵測到已失效 refresh 被重用即撤銷整個 credential family。帳號頁的 sessions 清單納入 device 憑證並可撤銷。
- device flow 的請求/回應 DTO 進 speclink-protocol（camelCase、JSON Schema 匯出、隨既有序列化往返測試），供 Phase 3 Desktop/CLI 消費；本刀不改 CLI client 行為。

## Capabilities

### New Capabilities

- `server-device-auth`: device authorization flow——發起與輪詢端點的狀態機、核准頁、短效 access token 與 rotating refresh credential 的生命週期、重用偵測與撤銷。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增端點與憑證種類，既有 PAT 認證、全部 API 路由與 CLI client 行為不變；parity 31 項、color 16 項、twin 8 情境凍結不動。identity 資料庫 schema 需擴充 device 憑證相關表（schema version 遞增，沿用既有版本守門與 migrate 路徑）。前置依賴：server-identity-pat 刀須先落地（session 登入、identity 儲存、帳號頁）。
- Affected specs: `server-device-auth`（新增）
- Affected code:
  - New: crates/speclink-protocol/src/device.rs、crates/speclink-server/src/device.rs、crates/speclink-server/tests/device_flow.rs、crates/speclink-server/tests/web_activate.rs、crates/speclink-server/tests/auth_device.rs、crates/speclink-server/tests/refresh_rotation.rs、crates/speclink-server/tests/web_device_sessions.rs、crates/speclink-server/tests/device_e2e.rs
  - Modified: crates/speclink-protocol/src/lib.rs、crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/web.rs、crates/speclink-server/src/app.rs（router 掛載端點）、crates/speclink-server/src/lib.rs、crates/speclink-server/Cargo.toml、crates/speclink-server/tests/identity.rs
  - Removed: 無

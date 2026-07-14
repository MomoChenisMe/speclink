## Why

第一子刀的認證是過渡設計：bearer token 是組態檔裡的明文靜態表，沒有帳號、沒有到期、沒有撤銷——當初的設計決策明載「帳號子刀落地後退場」。平台藍圖 §13.3 規定正式模型：Admin 建立一次性 invitation、使用者以本機密碼建帳號、在 /account 自助建立與撤銷 PAT（server 只存 prefix 與 hash、明文只顯示一次）、停權或降權後既有憑證立即失效；roadmap §5 的 Phase 2 gate 也把「一般使用者可經 invite 登入、自助建立/撤銷 PAT」列為驗收條件。沒有這一刀，任何多人使用都得共享組態檔明文 token，撤銷唯一手段是改檔重啟。

目標使用者：架設 speclink-server 的運維者（headless 建立邀請、不再管理明文 token 表）與團隊成員（自助管理自己的 PAT 與 sessions）。

## What Changes

- 新增 server 自有的 identity 儲存：獨立的 SQLite 資料庫檔（路徑由組態宣告；另有 memory 變體僅供測試），存 users（display、email、argon2 密碼 hash、active/suspended 狀態、admin 旗標）、project memberships、invitations（一次性 token hash、預指派 memberships、到期）、PATs（id、spk_pat_ 可辨識 prefix、SHA-256 hash、名稱、到期、撤銷、last-used）與 sessions。不與 TeamStore 資料庫混檔——TeamStore driver 的 schema 守門拒絕外來表，兩者各自演進。
- speclink-server binary 新增 invite 管理子命令：於主機上直接對 identity 資料庫建立邀請（email、顯示名、指派 project、可授 admin 旗標、到期），輸出一次性 invite URL。這是 §13.2 允許的 headless server CLI 管理路徑；第一位使用者由運維者以此建立，/admin Web UI 屬後續子刀。
- 新增最小 Web 入口（server-rendered HTML 表單，嵌入 binary，無外部資源）：invite 接受頁（設定密碼、耗用邀請）、/login 與 /logout（session cookie：HttpOnly、Secure、SameSite=Strict，POST 驗 Origin）、/account（sessions 與 PAT 清單）與 PAT 建立/撤銷（明文只在建立回應顯示一次）。
- **BREAKING（server 組態）**：組態檔的 bootstrap token 段退場，改宣告 identity 資料庫路徑；API 的 bearer 認證改查 identity 儲存的 PAT——逐請求檢查 hash、到期、撤銷、使用者狀態與 project membership，停權與降權即時生效。actor 非該 project 成員回 403 permission_denied。CLI client 端零變更：貼 PAT 的既有 remote auth 流程與標頭不變，只是 token 值來自 /account。

## Capabilities

### New Capabilities

- `server-identity`: server 的帳號、邀請、session 與 PAT 生命週期——一次性邀請、本機密碼登入、cookie 安全屬性、PAT hash 儲存與即時失效、identity 儲存獨立與版本守門。

### Modified Capabilities

- `reference-server`: 認證前置的 bearer 來源由組態靜態表改為 identity 儲存的 PAT（含停權/非成員的拒絕分類）；啟動組態的 tokens 段退場、新增 identity 資料庫宣告。

## Impact

- 相容性影響：server 組態檔不相容（tokens 段移除、新增 identity 段）——尚無正式部署，僅需遷移 repo 內測試組態與 e2e 播種（改為經 identity 儲存建 user 與 PAT）。CLI/桌面/本地模式零變更，parity 31 項、color 16 項、twin 8 情境凍結不動。新增 argon2 依賴只進 speclink-server crate。
- Affected specs: `server-identity`（新增）、`reference-server`（修改）
- Affected code:
  - New: crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/web.rs、crates/speclink-server/tests/identity.rs、crates/speclink-server/tests/invite.rs、crates/speclink-server/tests/web_invite.rs、crates/speclink-server/tests/web_account.rs、crates/speclink-server/tests/auth_pat.rs
  - Modified: crates/speclink-server/Cargo.toml、crates/speclink-server/src/main.rs、crates/speclink-server/src/config.rs、crates/speclink-server/src/auth.rs、crates/speclink-server/src/state.rs、crates/speclink-server/src/app.rs、crates/speclink-server/src/lib.rs、crates/speclink-server/tests/binding.rs、crates/speclink-server/tests/query_routes.rs、crates/speclink-server/tests/command_routes.rs、crates/speclink-server/tests/discussion_routes.rs、crates/speclink-server/tests/sync_state.rs、crates/speclink-server/tests/startup.rs、crates/speclink-server/tests/e2e_cli.rs、crates/speclink-server/tests/common/mod.rs、Cargo.lock
  - Removed: 無
